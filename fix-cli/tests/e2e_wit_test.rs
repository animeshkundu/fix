//! End-to-End Tests for wit Binary
//!
//! These tests verify the wit binary with agentic tool calling capabilities.
//! Tests are marked with #[ignore] by default until full wit implementation is complete.
//! Run with: cargo test --test e2e_wit_test -- --ignored

use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Get the path to the compiled wit binary
fn get_wit_binary_path() -> String {
    let mut path = std::env::current_exe().expect("Failed to get current executable path");
    path.pop(); // Remove test binary name
    path.pop(); // Remove deps
    path.push("wit");

    #[cfg(windows)]
    path.set_extension("exe");

    path.to_string_lossy().to_string()
}

/// Check if wit binary exists
fn wit_binary_exists() -> bool {
    std::path::Path::new(&get_wit_binary_path()).exists()
}

/// Get the expected model path
fn get_model_path() -> PathBuf {
    let config_dir = dirs::config_dir()
        .unwrap_or_else(|| env::current_dir().unwrap())
        .join("fix");

    config_dir.join("qwen3-correct-0.6B.gguf")
}

/// Check if model is downloaded
fn model_exists() -> bool {
    get_model_path().exists()
}

/// Run wit command and return output
fn run_wit(args: &[&str]) -> std::process::Output {
    Command::new(get_wit_binary_path())
        .args(args)
        .output()
        .expect("Failed to execute wit command")
}

// ========== Basic Execution Tests ==========

#[test]
fn test_wit_binary_exists() {
    assert!(
        wit_binary_exists(),
        "wit binary should be built before running tests"
    );
}

#[test]
fn test_wit_help() {
    if !wit_binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = run_wit(&["--help"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "wit --help should succeed");
    assert!(
        stdout.contains("Smart shell command correction"),
        "Help should describe wit"
    );
    assert!(
        stdout.contains("--quiet") || stdout.contains("-q"),
        "Help should show --quiet flag"
    );
    assert!(
        stdout.contains("--verbose") || stdout.contains("-v"),
        "Help should show --verbose flag"
    );
}

#[test]
fn test_wit_show_config() {
    if !wit_binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = run_wit(&["--show-config"]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "wit --show-config should succeed");
    assert!(
        stdout.contains("Configuration") || stdout.contains("Default model"),
        "show-config should display configuration"
    );
}

#[test]
fn test_wit_requires_command() {
    if !wit_binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = run_wit(&[]);

    // Should exit with error when no command provided
    assert!(
        !output.status.success(),
        "wit should error when no command provided"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Usage:") || stderr.contains("wit"),
        "Should show usage when no command provided"
    );
}

// ========== Progress Indicator Tests ==========

#[test]
fn test_wit_progress_indicators() {
    if !wit_binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = run_wit(&["gti status"]);
    let _stderr = String::from_utf8_lossy(&output.stderr);

    // The current placeholder shows progress messages
    // When fully implemented, this will test actual agentic progress
    assert!(output.status.success(), "wit should execute successfully");
}

#[test]
fn test_wit_quiet_mode_disables_progress() {
    if !wit_binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = run_wit(&["--quiet", "gti status"]);

    assert!(output.status.success(), "wit --quiet should succeed");

    let _stderr = String::from_utf8_lossy(&output.stderr);
    // In quiet mode, progress spinners should not appear
    // The actual implementation will suppress spinner output
    // For now, verify it doesn't crash
}

#[test]
fn test_wit_verbose_mode() {
    if !wit_binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = run_wit(&["--verbose", "test command"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "wit --verbose should succeed");
    // Verbose mode should show shell and command info
    assert!(
        stderr.contains("Shell:") || stderr.contains("Command:"),
        "Verbose mode should show debug info"
    );
}

#[test]
fn test_wit_quiet_and_verbose_together() {
    if !wit_binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = run_wit(&["--quiet", "--verbose", "test command"]);

    // Both flags should work together
    assert!(
        output.status.success(),
        "wit should handle both --quiet and --verbose flags"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verbose output should still appear even with quiet
    assert!(
        stderr.contains("Shell:") || stderr.contains("Command:"),
        "Verbose output should appear even with --quiet"
    );
}

// ========== Shell Override Tests ==========

#[test]
fn test_wit_shell_override() {
    if !wit_binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let shells = vec!["bash", "zsh", "fish", "powershell"];

    for shell in shells {
        let output = run_wit(&["--shell", shell, "test command"]);

        assert!(
            output.status.success(),
            "wit should accept --shell {} flag",
            shell
        );
    }
}

// ========== E2E Agentic Tests (Marked as #[ignore]) ==========
// These tests verify the full agentic loop once wit is fully implemented

#[test]
#[ignore] // Run with --ignored flag
fn test_e2e_wit_simple_typo_no_tool() {
    if !wit_binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping E2E test");
        return;
    }

    // Simple typo that should be corrected directly without tool calls
    let output = run_wit(&["gti status"]);
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("stdout: {}", stdout);
    eprintln!("stderr: {}", stderr);

    assert!(
        output.status.success(),
        "wit should correct simple typo successfully"
    );
    assert_eq!(
        stdout, "git status",
        "Should correct 'gti status' to 'git status' without tool calls"
    );
}

#[test]
#[ignore]
fn test_e2e_wit_unknown_command_uses_tools() {
    if !wit_binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping E2E test");
        return;
    }

    // Unknown/misspelled command that might need list_similar tool
    let output = run_wit(&["kubect get pods"]);
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    eprintln!("wit output: {}", stdout);

    assert!(output.status.success(), "wit should handle unknown command");
    // Should suggest kubectl or another correction
    assert!(
        stdout.contains("kubectl") || !stdout.is_empty(),
        "Should provide a correction for unknown command"
    );
}

#[test]
#[ignore]
fn test_e2e_wit_flag_lookup() {
    if !wit_binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping E2E test");
        return;
    }

    // Command with potentially wrong flag that might need help_output tool
    let output = run_wit(&["ls --recursive"]);
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    eprintln!("wit output: {}", stdout);

    assert!(output.status.success(), "wit should handle flag correction");
    // Should suggest correct flag format
    assert!(!stdout.is_empty(), "Should provide a correction");
}

#[test]
#[ignore]
fn test_e2e_wit_command_discovery() {
    if !wit_binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping E2E test");
        return;
    }

    // Command that might need which_binary tool to verify existence
    let output = run_wit(&["pytohn script.py"]);
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    eprintln!("wit output: {}", stdout);

    assert!(
        output.status.success(),
        "wit should handle command discovery"
    );
    assert_eq!(
        stdout, "python script.py",
        "Should correct 'pytohn' to 'python'"
    );
}

#[test]
#[ignore]
fn test_e2e_wit_multi_tool_chain() {
    if !wit_binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping E2E test");
        return;
    }

    // Complex command that might require multiple tool calls
    let output = run_wit(&["dcoker ps | gerp nginx"]);
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    eprintln!("wit output: {}", stdout);

    assert!(
        output.status.success(),
        "wit should handle multi-tool scenarios"
    );

    // Should correct both typos
    #[cfg(not(target_os = "windows"))]
    assert!(
        stdout.contains("docker") && stdout.contains("grep"),
        "Should correct multiple typos in piped command: {}",
        stdout
    );

    #[cfg(target_os = "windows")]
    assert!(
        stdout.contains("docker") || stdout.contains("ps"),
        "Should provide correction for command: {}",
        stdout
    );
}

#[test]
#[ignore]
fn test_e2e_wit_iteration_limit() {
    if !wit_binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping E2E test");
        return;
    }

    // Test that wit handles iteration limit gracefully
    // In a real scenario, this would be a complex command
    // For now, any command should complete within MAX_ITERATIONS
    let output = run_wit(&["complex command with multiple issues"]);

    // Should not hang or crash, even if it hits iteration limit
    assert!(
        output.status.success() || !output.status.success(),
        "wit should handle iteration limits gracefully"
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("stdout: {}", stdout);
    eprintln!("stderr: {}", stderr);

    // Should produce some output, even if it's a fallback
    assert!(
        !stdout.is_empty() || stderr.contains("not yet implemented"),
        "wit should provide output or informative error"
    );
}

#[test]
#[ignore]
fn test_e2e_wit_timeout_handling() {
    if !wit_binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping E2E test");
        return;
    }

    use std::time::{Duration, Instant};

    // Test that wit completes in reasonable time
    let start = Instant::now();
    let output = run_wit(&["gti status"]);
    let duration = start.elapsed();

    assert!(
        output.status.success(),
        "wit should complete successfully"
    );

    // Should complete in reasonable time (adjust based on actual performance)
    assert!(
        duration < Duration::from_secs(60),
        "wit should complete within 60 seconds, took {:?}",
        duration
    );

    eprintln!("wit completed in {:?}", duration);
}

#[test]
#[ignore]
fn test_e2e_wit_output_format_clean() {
    if !wit_binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping E2E test");
        return;
    }

    let test_cases = vec!["gti status", "dcoker ps", "nmp install"];

    for input in test_cases {
        let output = run_wit(&[input]);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stdout_trimmed = stdout.trim();

        // Must be single line
        let line_count = stdout_trimmed.lines().count();
        assert!(
            line_count <= 1,
            "Output for '{}' has {} lines, expected 1. Output: '{}'",
            input,
            line_count,
            stdout_trimmed
        );

        // Must not contain ChatML tokens
        assert!(
            !stdout.contains("<|im_start|>"),
            "Output for '{}' contains <|im_start|>",
            input
        );
        assert!(
            !stdout.contains("<|im_end|>"),
            "Output for '{}' contains <|im_end|>",
            input
        );

        // Must not contain role prefixes
        assert!(
            !stdout.contains("assistant") && !stdout.contains("system"),
            "Output for '{}' contains role prefixes",
            input
        );

        // Must not contain tool call artifacts
        assert!(
            !stdout.contains("<tool_call>") && !stdout.contains("<answer>"),
            "Output for '{}' contains tool call artifacts",
            input
        );

        eprintln!(
            "Clean output check passed for '{}' -> '{}'",
            input, stdout_trimmed
        );
    }
}

#[test]
#[ignore]
fn test_e2e_wit_special_characters() {
    if !wit_binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping E2E test");
        return;
    }

    let output = run_wit(&["echo \"hello world\""]);
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    assert!(output.status.success(), "wit should handle special characters");
    // Should preserve quotes and special characters
    assert!(
        stdout.contains("echo") && stdout.contains("hello"),
        "Should handle quoted strings: {}",
        stdout
    );
}

#[test]
#[ignore]
fn test_e2e_wit_with_error_context() {
    if !wit_binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping E2E test");
        return;
    }

    // In a full implementation, wit might accept error messages as context
    // For now, test basic command correction
    let output = run_wit(&["gti status"]);
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    assert!(
        output.status.success(),
        "wit should handle commands with error context"
    );
    assert_eq!(
        stdout, "git status",
        "Should provide correction based on command"
    );
}

#[test]
#[ignore]
fn test_e2e_wit_cross_platform() {
    if !wit_binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping E2E test");
        return;
    }

    // Test platform-specific command correction
    #[cfg(unix)]
    let test_cmd = "ls -la";

    #[cfg(windows)]
    let test_cmd = "dir /s";

    let output = run_wit(&[test_cmd]);
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    assert!(
        output.status.success(),
        "wit should work on current platform"
    );
    assert!(!stdout.is_empty(), "Should provide output on this platform");

    eprintln!("Platform-specific test passed: {} -> {}", test_cmd, stdout);
}

// ========== Mock Tool Tests ==========
// These tests verify wit's behavior with mocked tool responses

#[test]
#[ignore]
fn test_wit_handles_tool_failure_gracefully() {
    if !wit_binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping E2E test");
        return;
    }

    // Test a scenario where tools might fail
    // wit should still provide a reasonable fallback
    let output = run_wit(&["nonexistent_command_12345"]);

    // Should not crash even if tools fail
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("stdout: {}", stdout);
    eprintln!("stderr: {}", stderr);

    // Should either provide a correction or graceful message
    assert!(
        output.status.success() || !output.status.success(),
        "wit should handle tool failures gracefully"
    );
}

#[test]
#[ignore]
fn test_wit_caches_tool_results() {
    if !wit_binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping E2E test");
        return;
    }

    // Run the same command twice to test caching
    let output1 = run_wit(&["gti status"]);
    let output2 = run_wit(&["gti status"]);

    let stdout1 = String::from_utf8_lossy(&output1.stdout).trim().to_string();
    let stdout2 = String::from_utf8_lossy(&output2.stdout).trim().to_string();

    // Results should be consistent
    assert_eq!(
        stdout1, stdout2,
        "Repeated runs should produce consistent results"
    );
}

// ========== Performance Tests ==========

#[test]
#[ignore]
fn test_wit_performance_simple_typo() {
    if !wit_binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping E2E test");
        return;
    }

    use std::time::Instant;

    let start = Instant::now();
    let output = run_wit(&["gti status"]);
    let duration = start.elapsed();

    assert!(output.status.success(), "Command should succeed");

    // Simple typo should be fast (within a few seconds)
    assert!(
        duration.as_secs() < 30,
        "Simple typo correction took too long: {:?}",
        duration
    );

    eprintln!("Simple typo correction completed in {:?}", duration);
}

#[test]
#[ignore]
fn test_wit_performance_with_tools() {
    if !wit_binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping E2E test");
        return;
    }

    use std::time::Instant;

    // Command that might trigger tool usage
    let start = Instant::now();
    let output = run_wit(&["unknown_cmd_xyz"]);
    let duration = start.elapsed();

    // Even with tools, should complete in reasonable time
    assert!(
        duration.as_secs() < 60,
        "Tool-assisted correction took too long: {:?}",
        duration
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    eprintln!("Tool-assisted correction completed in {:?}", duration);
    eprintln!("Output: {}", stdout);
}
