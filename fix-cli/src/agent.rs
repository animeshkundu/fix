//! Agentic loop for shell command correction
//!
//! This module implements an iterative correction loop that allows the model
//! to call tools and refine its answer over multiple iterations.

use crate::parser::{parse_response, ModelResponse};
use crate::tools::{Shell, Tool, ToolExecutor, ToolResult};
use std::collections::HashMap;

/// Maximum iterations for the agentic loop to prevent infinite loops
pub const MAX_ITERATIONS: usize = 3;

/// A message in the conversation context
#[derive(Debug, Clone)]
pub struct Message {
    pub role: MessageRole,
    pub content: String,
}

/// Role of a message in the conversation
#[derive(Debug, Clone, PartialEq)]
pub enum MessageRole {
    System,
    User,
    Assistant,
    ToolResult,
}

/// Conversation context for the agentic loop
#[derive(Debug, Clone)]
pub struct Context {
    /// Messages in the conversation
    messages: Vec<Message>,
    /// Current shell type
    shell: Shell,
}

impl Context {
    /// Create a new context with system prompt
    pub fn new(shell: Shell) -> Self {
        let system_prompt = format!(
            "You are a shell command corrector for {}. \
            You can use tools to help determine the correct command. \
            When you have the answer, output only the corrected command.",
            shell
        );

        Self {
            messages: vec![Message {
                role: MessageRole::System,
                content: system_prompt,
            }],
            shell,
        }
    }

    /// Add the user's failed command
    pub fn add_user(&mut self, command: &str) {
        self.messages.push(Message {
            role: MessageRole::User,
            content: command.to_string(),
        });
    }

    /// Add error message context
    pub fn add_error(&mut self, error: &str) {
        // Append error to the last user message or add as new message
        if let Some(last) = self.messages.last_mut() {
            if last.role == MessageRole::User {
                last.content = format!("{}\nError: {}", last.content, error);
                return;
            }
        }
        self.messages.push(Message {
            role: MessageRole::User,
            content: format!("Error: {}", error),
        });
    }

    /// Add assistant response
    pub fn add_assistant(&mut self, response: &str) {
        self.messages.push(Message {
            role: MessageRole::Assistant,
            content: response.to_string(),
        });
    }

    /// Add tool result
    pub fn add_tool_result(&mut self, tool_name: &str, result: &ToolResult) {
        let content = if result.success {
            format!("[{}]: {}", tool_name, result.output)
        } else {
            format!(
                "[{}] failed: {}",
                tool_name,
                result.error.as_deref().unwrap_or("Unknown error")
            )
        };

        self.messages.push(Message {
            role: MessageRole::ToolResult,
            content,
        });
    }

    /// Build a prompt string from the context
    pub fn build_prompt(&self) -> String {
        let mut prompt = String::new();

        for msg in &self.messages {
            let role_tag = match msg.role {
                MessageRole::System => "system",
                MessageRole::User => "user",
                MessageRole::Assistant => "assistant",
                MessageRole::ToolResult => "tool_result",
            };

            prompt.push_str(&format!(
                "<|im_start|>{}\n{}<|im_end|>\n",
                role_tag, msg.content
            ));
        }

        // Add assistant prompt for generation
        prompt.push_str("<|im_start|>assistant\n");
        prompt
    }

    /// Get the shell type
    pub fn shell(&self) -> Shell {
        self.shell
    }
}

/// Result of the agentic correction process
#[derive(Debug)]
pub struct AgentResult {
    /// The corrected command
    pub command: String,
    /// Number of iterations taken
    pub iterations: usize,
    /// Whether tools were used
    pub tools_used: bool,
}

/// Execute the agentic correction loop
///
/// This function:
/// 1. Builds initial context with the failed command
/// 2. Iteratively generates responses and executes tools
/// 3. Returns when a final answer is reached or max iterations hit
///
/// # Arguments
/// * `input` - The failed command to correct
/// * `shell` - The shell type (bash, zsh, etc.)
/// * `error` - Optional error message from the failed command
/// * `generate_fn` - Function to generate model responses
///
/// # Returns
/// The corrected command string
pub fn agentic_correct<F>(
    input: &str,
    shell: Shell,
    error: Option<&str>,
    mut generate_fn: F,
) -> AgentResult
where
    F: FnMut(&str) -> String,
{
    let mut context = Context::new(shell);
    context.add_user(input);

    if let Some(err) = error {
        context.add_error(err);
    }

    let executor = ToolExecutor::new(shell);
    let mut tools_used = false;

    for iteration in 0..MAX_ITERATIONS {
        let prompt = context.build_prompt();
        let response = generate_fn(&prompt);

        match parse_response(&response) {
            ModelResponse::ToolCall { name, args } => {
                tools_used = true;

                // Execute the tool
                if let Some(tool) = create_tool(&name, &args) {
                    let result = executor.execute(&tool);
                    context.add_assistant(&response);
                    context.add_tool_result(&name, &result);
                } else {
                    // Unknown tool - add error and continue
                    context.add_assistant(&response);
                    context.add_tool_result(
                        &name,
                        &ToolResult::failure(format!("Unknown tool: {}", name)),
                    );
                }
            }
            ModelResponse::FinalAnswer(answer) => {
                return AgentResult {
                    command: answer,
                    iterations: iteration + 1,
                    tools_used,
                };
            }
        }
    }

    // Max iterations reached - return last context as fallback
    AgentResult {
        command: fallback_correction(input),
        iterations: MAX_ITERATIONS,
        tools_used,
    }
}

/// Create a Tool from name and arguments
fn create_tool(name: &str, args: &HashMap<String, String>) -> Option<Tool> {
    match name {
        "help_output" => {
            let command = args.get("command")?;
            Some(Tool::HelpOutput {
                command: command.clone(),
            })
        }
        "which_binary" => {
            let command = args.get("command")?;
            Some(Tool::WhichBinary {
                command: command.clone(),
            })
        }
        "list_similar" => {
            let prefix = args.get("prefix")?;
            Some(Tool::ListSimilar {
                prefix: prefix.clone(),
            })
        }
        "get_env_var" => {
            let name = args.get("name")?;
            Some(Tool::GetEnvVar { name: name.clone() })
        }
        "man_page" => {
            let command = args.get("command")?;
            Some(Tool::ManPage {
                command: command.clone(),
            })
        }
        _ => None,
    }
}

/// Fallback correction when iteration limit is reached
fn fallback_correction(input: &str) -> String {
    // Simple fallback: return the input as-is
    // In a full implementation, this could use the simple fix model
    input.to_string()
}

// ========== Tests ==========

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Context Tests =====

    #[test]
    fn test_context_new() {
        let ctx = Context::new(Shell::Bash);
        assert_eq!(ctx.shell(), Shell::Bash);
        assert_eq!(ctx.messages.len(), 1);
        assert_eq!(ctx.messages[0].role, MessageRole::System);
    }

    #[test]
    fn test_context_add_user() {
        let mut ctx = Context::new(Shell::Bash);
        ctx.add_user("gti status");

        assert_eq!(ctx.messages.len(), 2);
        assert_eq!(ctx.messages[1].role, MessageRole::User);
        assert_eq!(ctx.messages[1].content, "gti status");
    }

    #[test]
    fn test_context_add_error() {
        let mut ctx = Context::new(Shell::Bash);
        ctx.add_user("gti status");
        ctx.add_error("command not found: gti");

        // Error should be appended to user message
        assert_eq!(ctx.messages.len(), 2);
        assert!(ctx.messages[1].content.contains("Error:"));
        assert!(ctx.messages[1].content.contains("command not found"));
    }

    #[test]
    fn test_context_add_tool_result_success() {
        let mut ctx = Context::new(Shell::Bash);
        ctx.add_user("test");
        ctx.add_tool_result(
            "which_binary",
            &ToolResult::success("/usr/bin/git".to_string()),
        );

        assert_eq!(ctx.messages.len(), 3);
        assert_eq!(ctx.messages[2].role, MessageRole::ToolResult);
        assert!(ctx.messages[2].content.contains("/usr/bin/git"));
    }

    #[test]
    fn test_context_add_tool_result_failure() {
        let mut ctx = Context::new(Shell::Bash);
        ctx.add_user("test");
        ctx.add_tool_result(
            "which_binary",
            &ToolResult::failure("not found".to_string()),
        );

        assert_eq!(ctx.messages.len(), 3);
        assert!(ctx.messages[2].content.contains("failed"));
        assert!(ctx.messages[2].content.contains("not found"));
    }

    #[test]
    fn test_context_build_prompt() {
        let mut ctx = Context::new(Shell::Bash);
        ctx.add_user("gti status");

        let prompt = ctx.build_prompt();

        assert!(prompt.contains("<|im_start|>system"));
        assert!(prompt.contains("<|im_start|>user"));
        assert!(prompt.contains("gti status"));
        assert!(prompt.contains("<|im_start|>assistant"));
        // Should end with assistant prompt for generation
        assert!(prompt.ends_with("<|im_start|>assistant\n"));
    }

    // ===== Create Tool Tests =====

    #[test]
    fn test_create_tool_help_output() {
        let mut args = HashMap::new();
        args.insert("command".to_string(), "git".to_string());

        let tool = create_tool("help_output", &args);
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "help_output");
    }

    #[test]
    fn test_create_tool_which_binary() {
        let mut args = HashMap::new();
        args.insert("command".to_string(), "docker".to_string());

        let tool = create_tool("which_binary", &args);
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "which_binary");
    }

    #[test]
    fn test_create_tool_list_similar() {
        let mut args = HashMap::new();
        args.insert("prefix".to_string(), "gi".to_string());

        let tool = create_tool("list_similar", &args);
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "list_similar");
    }

    #[test]
    fn test_create_tool_get_env_var() {
        let mut args = HashMap::new();
        args.insert("name".to_string(), "PATH".to_string());

        let tool = create_tool("get_env_var", &args);
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "get_env_var");
    }

    #[test]
    fn test_create_tool_man_page() {
        let mut args = HashMap::new();
        args.insert("command".to_string(), "ls".to_string());

        let tool = create_tool("man_page", &args);
        assert!(tool.is_some());
        assert_eq!(tool.unwrap().name(), "man_page");
    }

    #[test]
    fn test_create_tool_unknown() {
        let args = HashMap::new();
        let tool = create_tool("unknown_tool", &args);
        assert!(tool.is_none());
    }

    #[test]
    fn test_create_tool_missing_args() {
        let args = HashMap::new();
        let tool = create_tool("which_binary", &args);
        assert!(tool.is_none());
    }

    // ===== Agentic Loop Tests =====

    #[test]
    fn test_agentic_correct_immediate_answer() {
        // Simulate model that returns answer immediately
        let result = agentic_correct("gti status", Shell::Bash, None, |_| {
            "git status".to_string()
        });

        assert_eq!(result.command, "git status");
        assert_eq!(result.iterations, 1);
        assert!(!result.tools_used);
    }

    #[test]
    fn test_agentic_correct_with_answer_tags() {
        let result = agentic_correct("dcoker ps", Shell::Bash, None, |_| {
            "<answer>docker ps</answer>".to_string()
        });

        assert_eq!(result.command, "docker ps");
        assert_eq!(result.iterations, 1);
        assert!(!result.tools_used);
    }

    #[test]
    fn test_agentic_correct_with_tool_then_answer() {
        let mut call_count = 0;

        let result = agentic_correct("gti status", Shell::Bash, None, |_| {
            call_count += 1;
            if call_count == 1 {
                // First call: request a tool
                r#"<tool_call>{"name": "which_binary", "args": {"command": "git"}}</tool_call>"#
                    .to_string()
            } else {
                // Second call: provide answer
                "git status".to_string()
            }
        });

        assert_eq!(result.command, "git status");
        assert_eq!(result.iterations, 2);
        assert!(result.tools_used);
    }

    #[test]
    fn test_agentic_correct_max_iterations() {
        // Simulate model that keeps requesting tools
        let result = agentic_correct("test", Shell::Bash, None, |_| {
            r#"<tool_call>{"name": "which_binary", "args": {"command": "git"}}</tool_call>"#
                .to_string()
        });

        // Should hit max iterations and return fallback
        assert_eq!(result.iterations, MAX_ITERATIONS);
        assert!(result.tools_used);
    }

    #[test]
    fn test_agentic_correct_with_error_context() {
        let result = agentic_correct(
            "gti status",
            Shell::Bash,
            Some("command not found: gti"),
            |prompt| {
                // Verify error is in prompt
                assert!(prompt.contains("command not found"));
                "git status".to_string()
            },
        );

        assert_eq!(result.command, "git status");
    }

    #[test]
    fn test_agentic_correct_unknown_tool() {
        let mut call_count = 0;

        let result = agentic_correct("test", Shell::Bash, None, |_| {
            call_count += 1;
            if call_count == 1 {
                // Request unknown tool
                r#"<tool_call>{"name": "unknown_tool", "args": {}}</tool_call>"#.to_string()
            } else {
                // Should continue and provide answer
                "corrected command".to_string()
            }
        });

        assert_eq!(result.command, "corrected command");
        assert!(result.tools_used);
    }

    // ===== Fallback Tests =====

    #[test]
    fn test_fallback_correction() {
        let result = fallback_correction("gti status");
        assert_eq!(result, "gti status");
    }
}
