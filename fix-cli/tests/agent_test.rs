//! Agent Integration Tests
//!
//! These tests verify the agentic loop works correctly with real tool execution.

use fix_lib::agent::{agentic_correct, Context, MAX_ITERATIONS};
use fix_lib::parser::{parse_response, ModelResponse};
use fix_lib::tools::Shell;

// ===== Integration Tests =====

#[test]
fn test_agent_loop_with_mock_model_direct_answer() {
    // Simulate a model that returns a direct answer
    let result = agentic_correct("gti status", Shell::Bash, None, |_prompt| {
        "git status".to_string()
    });

    assert_eq!(result.command, "git status");
    assert_eq!(result.iterations, 1);
    assert!(!result.tools_used);
}

#[test]
fn test_agent_loop_with_mock_model_tool_call_then_answer() {
    let mut iteration = 0;

    let result = agentic_correct("gti status", Shell::Bash, None, |prompt| {
        iteration += 1;

        match iteration {
            1 => {
                // First iteration: request which_binary tool
                r#"<tool_call>{"name": "which_binary", "args": {"command": "git"}}</tool_call>"#
                    .to_string()
            }
            _ => {
                // After getting tool result, the prompt should contain the result
                assert!(
                    prompt.contains("tool_result") || prompt.contains("which_binary"),
                    "Prompt should contain tool result"
                );
                // Return answer
                "git status".to_string()
            }
        }
    });

    assert_eq!(result.command, "git status");
    assert_eq!(result.iterations, 2);
    assert!(result.tools_used);
}

#[test]
fn test_agent_loop_respects_max_iterations() {
    // Model that always requests a tool (should be stopped at MAX_ITERATIONS)
    let result = agentic_correct("test command", Shell::Bash, None, |_| {
        r#"<tool_call>{"name": "which_binary", "args": {"command": "test"}}</tool_call>"#
            .to_string()
    });

    assert_eq!(result.iterations, MAX_ITERATIONS);
    assert!(result.tools_used);
}

#[test]
fn test_agent_context_includes_error() {
    let result = agentic_correct(
        "gti status",
        Shell::Bash,
        Some("command not found: gti"),
        |prompt| {
            // Verify the prompt includes the error message
            assert!(
                prompt.contains("command not found"),
                "Prompt should include error message"
            );
            "git status".to_string()
        },
    );

    assert_eq!(result.command, "git status");
}

#[test]
fn test_agent_context_includes_shell() {
    let shells = [Shell::Bash, Shell::Zsh, Shell::Fish, Shell::PowerShell];

    for shell in shells {
        let result = agentic_correct("test", shell, None, |prompt| {
            // Verify the prompt includes the shell type
            let shell_str = shell.to_string();
            assert!(
                prompt.to_lowercase().contains(&shell_str.to_lowercase()),
                "Prompt should include shell: {}. Got: {}",
                shell_str,
                prompt
            );
            "test".to_string()
        });

        assert_eq!(result.iterations, 1);
    }
}

#[test]
fn test_agent_handles_multiple_tool_calls() {
    let mut iteration = 0;

    let result = agentic_correct("gti status", Shell::Bash, None, |_prompt| {
        iteration += 1;

        match iteration {
            1 => {
                // First: check if git exists
                r#"<tool_call>{"name": "which_binary", "args": {"command": "git"}}</tool_call>"#
                    .to_string()
            }
            2 => {
                // Second: list similar commands
                r#"<tool_call>{"name": "list_similar", "args": {"prefix": "gi"}}</tool_call>"#
                    .to_string()
            }
            _ => {
                // Finally: return answer
                "git status".to_string()
            }
        }
    });

    assert_eq!(result.command, "git status");
    assert_eq!(result.iterations, 3);
    assert!(result.tools_used);
}

#[test]
fn test_agent_handles_unknown_tool() {
    let mut iteration = 0;

    let result = agentic_correct("test", Shell::Bash, None, |_| {
        iteration += 1;

        match iteration {
            1 => {
                // Request an unknown tool
                r#"<tool_call>{"name": "nonexistent_tool", "args": {}}</tool_call>"#.to_string()
            }
            _ => {
                // Return answer after failed tool
                "corrected".to_string()
            }
        }
    });

    assert_eq!(result.command, "corrected");
    assert!(result.tools_used);
}

// ===== Context Unit Tests =====

#[test]
fn test_context_message_ordering() {
    let mut ctx = Context::new(Shell::Bash);
    ctx.add_user("test command");
    ctx.add_assistant("thinking...");
    ctx.add_tool_result(
        "which_binary",
        &fix_lib::tools::ToolResult::success("/usr/bin/test".to_string()),
    );

    let prompt = ctx.build_prompt();

    // Verify order: system, user, assistant, tool_result, then new assistant
    let system_pos = prompt.find("system").unwrap();
    let user_pos = prompt.find("user").unwrap();
    let assistant_pos = prompt.find("assistant").unwrap();
    let tool_pos = prompt.find("tool_result").unwrap();

    assert!(system_pos < user_pos);
    assert!(user_pos < assistant_pos);
    assert!(assistant_pos < tool_pos);
}

#[test]
fn test_context_shell_in_system_prompt() {
    for shell in [Shell::Bash, Shell::Zsh, Shell::Fish] {
        let ctx = Context::new(shell);
        let prompt = ctx.build_prompt();

        assert!(
            prompt.contains(&shell.to_string()),
            "System prompt should mention shell: {}",
            shell
        );
    }
}

// ===== Parser Integration Tests =====

#[test]
fn test_parser_integration_with_agent() {
    // Test that parser output matches what agent expects
    let tool_call =
        r#"<tool_call>{"name": "which_binary", "args": {"command": "git"}}</tool_call>"#;
    let response = parse_response(tool_call);

    match response {
        ModelResponse::ToolCall { name, args } => {
            assert_eq!(name, "which_binary");
            assert_eq!(args.get("command").unwrap(), "git");
        }
        _ => panic!("Expected ToolCall"),
    }
}

#[test]
fn test_parser_final_answer_variations() {
    let variations = vec![
        ("git status", "git status"),
        ("<answer>git status</answer>", "git status"),
        ("  git status  ", "git status"),
        ("git status<|im_end|>", "git status"),
    ];

    for (input, expected) in variations {
        let response = parse_response(input);
        match response {
            ModelResponse::FinalAnswer(answer) => {
                assert_eq!(answer, expected, "Input: {}", input);
            }
            _ => panic!("Expected FinalAnswer for input: {}", input),
        }
    }
}

// ===== Real Tool Execution Tests =====

#[test]
fn test_agent_with_real_env_var_tool() {
    let mut iteration = 0;

    let result = agentic_correct("echo $PATH", Shell::Bash, None, |prompt| {
        iteration += 1;

        match iteration {
            1 => {
                // Request env var
                r#"<tool_call>{"name": "get_env_var", "args": {"name": "PATH"}}</tool_call>"#
                    .to_string()
            }
            _ => {
                // Verify we got PATH in the context
                assert!(
                    prompt.contains("PATH") || prompt.contains("/"),
                    "Should have PATH info in context"
                );
                "echo $PATH".to_string()
            }
        }
    });

    assert!(result.tools_used);
    assert_eq!(result.iterations, 2);
}

#[cfg(unix)]
#[test]
fn test_agent_with_real_which_tool() {
    let mut iteration = 0;

    let result = agentic_correct("lss", Shell::Bash, None, |prompt| {
        iteration += 1;

        match iteration {
            1 => {
                // Check if ls exists
                r#"<tool_call>{"name": "which_binary", "args": {"command": "ls"}}</tool_call>"#
                    .to_string()
            }
            _ => {
                // Should have path to ls in context
                assert!(
                    prompt.contains("/bin") || prompt.contains("ls"),
                    "Should have ls path in context"
                );
                "ls".to_string()
            }
        }
    });

    assert!(result.tools_used);
    assert_eq!(result.command, "ls");
}
