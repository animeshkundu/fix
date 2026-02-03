//! Parser for model structured output
//!
//! This module handles parsing of model responses to detect:
//! - Tool calls in `<tool_call>{...}</tool_call>` format
//! - Final answers in `<answer>...</answer>` format
//! - Raw text as final answers

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response type from parsing model output
#[derive(Debug, Clone, PartialEq)]
pub enum ModelResponse {
    /// Model requested a tool call
    ToolCall {
        name: String,
        args: HashMap<String, String>,
    },
    /// Model provided a final answer
    FinalAnswer(String),
}

/// Tool call structure for JSON deserialization
/// Supports both "args" and "arguments" fields for backward compatibility
#[derive(Debug, Clone, Deserialize, Serialize)]
struct ToolCallJson {
    name: String,
    /// Primary field used by training data
    #[serde(default)]
    arguments: HashMap<String, serde_json::Value>,
    /// Fallback field for compatibility
    #[serde(default)]
    args: HashMap<String, serde_json::Value>,
}

/// Parse model output to extract response type
///
/// Looks for:
/// 1. `<tool_call>{...}</tool_call>` - Tool request
/// 2. `<answer>...</answer>` - Explicit final answer
/// 3. Raw text - Treated as final answer
pub fn parse_response(output: &str) -> ModelResponse {
    let trimmed = output.trim();

    // Try to extract tool call
    if let Some(tool_call) = extract_tool_call(trimmed) {
        return tool_call;
    }

    // Try to extract explicit answer
    if let Some(answer) = extract_answer(trimmed) {
        return ModelResponse::FinalAnswer(answer);
    }

    // Fallback: treat raw text as final answer
    ModelResponse::FinalAnswer(clean_output(trimmed))
}

/// Extract tool call from `<tool_call>{...}</tool_call>` pattern
fn extract_tool_call(output: &str) -> Option<ModelResponse> {
    // Find the tool_call tags
    let start_tag = "<tool_call>";
    let end_tag = "</tool_call>";

    let start_idx = output.find(start_tag)?;
    let end_idx = output.find(end_tag)?;

    if end_idx <= start_idx {
        return None;
    }

    // Extract JSON content
    let json_start = start_idx + start_tag.len();
    let json_content = output[json_start..end_idx].trim();

    // Parse JSON
    let tool_call: ToolCallJson = serde_json::from_str(json_content).ok()?;

    // Prefer "arguments" (training data format), fall back to "args"
    let raw_args = if !tool_call.arguments.is_empty() {
        tool_call.arguments
    } else {
        tool_call.args
    };

    // Convert args to HashMap<String, String>
    let args: HashMap<String, String> = raw_args
        .into_iter()
        .map(|(k, v)| {
            let value_str = match v {
                serde_json::Value::String(s) => s,
                other => other.to_string(),
            };
            (k, value_str)
        })
        .collect();

    Some(ModelResponse::ToolCall {
        name: tool_call.name,
        args,
    })
}

/// Extract answer from `<answer>...</answer>` pattern
fn extract_answer(output: &str) -> Option<String> {
    let start_tag = "<answer>";
    let end_tag = "</answer>";

    let start_idx = output.find(start_tag)?;
    let end_idx = output.find(end_tag)?;

    if end_idx <= start_idx {
        return None;
    }

    let content_start = start_idx + start_tag.len();
    let answer = output[content_start..end_idx].trim();

    Some(clean_output(answer))
}

/// Clean model output by removing common artifacts
pub fn clean_output(output: &str) -> String {
    let mut result = output.trim();

    // Remove common ChatML artifacts
    if result.contains("<|im_end|>") {
        if let Some(idx) = result.find("<|im_end|>") {
            result = result[..idx].trim();
        }
    }

    if result.contains("<|im_start|>") {
        if let Some(idx) = result.find("<|im_start|>") {
            result = result[..idx].trim();
        }
    }

    // Remove thinking blocks
    if let Some(start) = result.find("<think>") {
        if let Some(end) = result.find("</think>") {
            let before = &result[..start];
            let after = &result[end + "</think>".len()..];
            result = if after.trim().is_empty() {
                before.trim()
            } else {
                // Return the part after thinking
                after.trim()
            };
        }
    }

    // Strip common prefixes
    let prefixes = [
        "command >",
        "command>",
        "command 2>&1",
        "Command:",
        "Output:",
        ">>>",
    ];

    for prefix in prefixes {
        if let Some(stripped) = result.strip_prefix(prefix) {
            result = stripped.trim();
        }
    }

    // Take only first line if multi-line
    result = result.lines().next().unwrap_or(result).trim();

    result.to_string()
}

// ========== Tests ==========

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Tool Call Extraction Tests =====

    #[test]
    fn test_parse_tool_call_basic() {
        let output =
            r#"<tool_call>{"name": "which_binary", "args": {"command": "git"}}</tool_call>"#;
        let result = parse_response(output);

        match result {
            ModelResponse::ToolCall { name, args } => {
                assert_eq!(name, "which_binary");
                assert_eq!(args.get("command"), Some(&"git".to_string()));
            }
            _ => panic!("Expected ToolCall, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_tool_call_with_whitespace() {
        let output = r#"
            <tool_call>
                {"name": "help_output", "args": {"command": "docker"}}
            </tool_call>
        "#;
        let result = parse_response(output);

        match result {
            ModelResponse::ToolCall { name, args } => {
                assert_eq!(name, "help_output");
                assert_eq!(args.get("command"), Some(&"docker".to_string()));
            }
            _ => panic!("Expected ToolCall, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_tool_call_empty_args() {
        let output = r#"<tool_call>{"name": "list_similar", "args": {}}</tool_call>"#;
        let result = parse_response(output);

        match result {
            ModelResponse::ToolCall { name, args } => {
                assert_eq!(name, "list_similar");
                assert!(args.is_empty());
            }
            _ => panic!("Expected ToolCall, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_tool_call_no_args_field() {
        let output = r#"<tool_call>{"name": "get_env_var"}</tool_call>"#;
        let result = parse_response(output);

        match result {
            ModelResponse::ToolCall { name, args } => {
                assert_eq!(name, "get_env_var");
                assert!(args.is_empty());
            }
            _ => panic!("Expected ToolCall, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_tool_call_multiple_args() {
        let output = r#"<tool_call>{"name": "test_tool", "args": {"arg1": "val1", "arg2": "val2"}}</tool_call>"#;
        let result = parse_response(output);

        match result {
            ModelResponse::ToolCall { name, args } => {
                assert_eq!(name, "test_tool");
                assert_eq!(args.get("arg1"), Some(&"val1".to_string()));
                assert_eq!(args.get("arg2"), Some(&"val2".to_string()));
            }
            _ => panic!("Expected ToolCall, got {:?}", result),
        }
    }

    // ===== Answer Extraction Tests =====

    #[test]
    fn test_parse_answer_basic() {
        let output = "<answer>git status</answer>";
        let result = parse_response(output);

        assert_eq!(result, ModelResponse::FinalAnswer("git status".to_string()));
    }

    #[test]
    fn test_parse_answer_with_whitespace() {
        let output = r#"
            <answer>
                docker ps
            </answer>
        "#;
        let result = parse_response(output);

        assert_eq!(result, ModelResponse::FinalAnswer("docker ps".to_string()));
    }

    #[test]
    fn test_parse_answer_multiline_takes_first() {
        let output = "<answer>npm install\nnpm start</answer>";
        let result = parse_response(output);

        assert_eq!(
            result,
            ModelResponse::FinalAnswer("npm install".to_string())
        );
    }

    // ===== Raw Text as Answer Tests =====

    #[test]
    fn test_parse_raw_text_as_answer() {
        let output = "git status";
        let result = parse_response(output);

        assert_eq!(result, ModelResponse::FinalAnswer("git status".to_string()));
    }

    #[test]
    fn test_parse_raw_text_with_whitespace() {
        let output = "   docker ps   ";
        let result = parse_response(output);

        assert_eq!(result, ModelResponse::FinalAnswer("docker ps".to_string()));
    }

    #[test]
    fn test_parse_raw_text_multiline_takes_first() {
        let output = "npm install\nsomething else";
        let result = parse_response(output);

        assert_eq!(
            result,
            ModelResponse::FinalAnswer("npm install".to_string())
        );
    }

    // ===== Clean Output Tests =====

    #[test]
    fn test_clean_output_chatml_tokens() {
        assert_eq!(
            clean_output("git status<|im_end|>"),
            "git status".to_string()
        );
        assert_eq!(
            clean_output("docker ps<|im_start|>user"),
            "docker ps".to_string()
        );
    }

    #[test]
    fn test_clean_output_common_prefixes() {
        assert_eq!(
            clean_output("command > git status"),
            "git status".to_string()
        );
        assert_eq!(clean_output("Command: docker ps"), "docker ps".to_string());
        assert_eq!(clean_output(">>> npm install"), "npm install".to_string());
    }

    #[test]
    fn test_clean_output_thinking_block() {
        let output = "<think>Let me think about this...</think>git status";
        assert_eq!(clean_output(output), "git status".to_string());
    }

    #[test]
    fn test_clean_output_empty() {
        assert_eq!(clean_output(""), "".to_string());
        assert_eq!(clean_output("   "), "".to_string());
    }

    // ===== Edge Cases =====

    #[test]
    fn test_parse_invalid_tool_call_json() {
        // Invalid JSON should fall back to raw text
        let output = "<tool_call>not valid json</tool_call>";
        let result = parse_response(output);

        // Should treat as final answer since JSON parse fails
        match result {
            ModelResponse::FinalAnswer(_) => {}
            _ => panic!("Expected FinalAnswer for invalid JSON, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_malformed_tags() {
        // Missing closing tag
        let output = "<tool_call>{\"name\": \"test\"}";
        let result = parse_response(output);

        match result {
            ModelResponse::FinalAnswer(_) => {}
            _ => panic!("Expected FinalAnswer for malformed tags, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_empty_answer_tags() {
        let output = "<answer></answer>";
        let result = parse_response(output);

        assert_eq!(result, ModelResponse::FinalAnswer("".to_string()));
    }

    #[test]
    fn test_parse_tool_call_with_numeric_arg() {
        let output = r#"<tool_call>{"name": "test", "args": {"count": 5}}</tool_call>"#;
        let result = parse_response(output);

        match result {
            ModelResponse::ToolCall { name, args } => {
                assert_eq!(name, "test");
                assert_eq!(args.get("count"), Some(&"5".to_string()));
            }
            _ => panic!("Expected ToolCall, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_tool_call_priority_over_answer() {
        // If both tool_call and answer are present, tool_call takes priority
        let output = r#"<tool_call>{"name": "test"}</tool_call><answer>git status</answer>"#;
        let result = parse_response(output);

        match result {
            ModelResponse::ToolCall { name, .. } => {
                assert_eq!(name, "test");
            }
            _ => panic!("Expected ToolCall to take priority, got {:?}", result),
        }
    }

    // ===== Training Data Format Tests =====

    #[test]
    fn test_parse_tool_call_with_arguments_field() {
        // Training data uses "arguments" instead of "args"
        let output =
            r#"<tool_call>{"name": "which_binary", "arguments": {"command": "git"}}</tool_call>"#;
        let result = parse_response(output);

        match result {
            ModelResponse::ToolCall { name, args } => {
                assert_eq!(name, "which_binary");
                assert_eq!(args.get("command"), Some(&"git".to_string()));
            }
            _ => panic!("Expected ToolCall, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_tool_call_training_format_list_similar() {
        // Exact format from training data
        let output = r#"<tool_call>
{"name": "list_similar_commands", "arguments": {"prefix": "ip"}}
</tool_call>"#;
        let result = parse_response(output);

        match result {
            ModelResponse::ToolCall { name, args } => {
                assert_eq!(name, "list_similar_commands");
                assert_eq!(args.get("prefix"), Some(&"ip".to_string()));
            }
            _ => panic!("Expected ToolCall, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_tool_call_training_format_get_command_help() {
        // Training data tool name
        let output =
            r#"<tool_call>{"name": "get_command_help", "arguments": {"command": "docker"}}</tool_call>"#;
        let result = parse_response(output);

        match result {
            ModelResponse::ToolCall { name, args } => {
                assert_eq!(name, "get_command_help");
                assert_eq!(args.get("command"), Some(&"docker".to_string()));
            }
            _ => panic!("Expected ToolCall, got {:?}", result),
        }
    }

    #[test]
    fn test_parse_prefers_arguments_over_args() {
        // If both fields present, "arguments" takes priority
        let output = r#"<tool_call>{"name": "test", "arguments": {"a": "1"}, "args": {"b": "2"}}</tool_call>"#;
        let result = parse_response(output);

        match result {
            ModelResponse::ToolCall { name, args } => {
                assert_eq!(name, "test");
                // Should use "arguments" field
                assert_eq!(args.get("a"), Some(&"1".to_string()));
                // "args" field should be ignored
                assert_eq!(args.get("b"), None);
            }
            _ => panic!("Expected ToolCall, got {:?}", result),
        }
    }
}
