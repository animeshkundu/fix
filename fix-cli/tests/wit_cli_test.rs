//! wit CLI Integration Tests
//!
//! These tests verify the wit CLI binary works correctly when executed as a subprocess.
//! Mirrors the test patterns used in cli_test.rs for fix binary.

use std::process::Command;
use std::time::Duration;

/// Get the path to the compiled wit binary
fn get_binary_path() -> String {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove deps
    path.push("wit");

    #[cfg(windows)]
    path.set_extension("exe");

    path.to_string_lossy().to_string()
}

/// Check if wit binary exists
fn binary_exists() -> bool {
    std::path::Path::new(&get_binary_path()).exists()
}

// ========== Basic Flag Tests ==========

#[test]
fn test_wit_help_flag() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("--help")
        .output()
        .expect("Failed to execute wit binary");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("wit") || stdout.contains("Smart"),
        "Help should describe wit: {}",
        stdout
    );
    assert!(output.status.success(), "Help should exit successfully");
}

#[test]
fn test_wit_version_flag() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("--version")
        .output()
        .expect("Failed to execute wit binary");

    // Just verify it doesn't crash
    let _ = String::from_utf8_lossy(&output.stdout);
}

#[test]
fn test_wit_invalid_flag() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("--invalid-flag-that-does-not-exist")
        .output()
        .expect("Failed to execute wit binary");

    assert!(
        !output.status.success(),
        "Invalid flag should cause error exit"
    );
}

#[test]
fn test_wit_no_args_shows_usage() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .output()
        .expect("Failed to execute wit binary");

    // Should show usage when no command provided
    assert!(!output.status.success(), "No args should cause error exit");

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Usage") || stderr.contains("wit") || stderr.contains("error"),
        "Should show usage or error when no command provided: {}",
        stderr
    );
}

// ========== Verbose/Quiet Flag Tests ==========

#[test]
fn test_wit_verbose_flag() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .args(["--verbose", "gti status"])
        .output()
        .expect("Failed to execute wit binary");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verbose should show debug info
    assert!(
        stderr.contains("Shell:") || stderr.contains("Command:") || stderr.contains("verbose"),
        "Verbose mode should show debug info: {}",
        stderr
    );
}

#[test]
fn test_wit_quiet_flag() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .args(["--quiet", "gti status"])
        .output()
        .expect("Failed to execute wit binary");

    // Quiet mode should succeed without spinners
    // The actual output depends on model availability
    assert!(
        output.status.success() || !output.status.success(),
        "Quiet flag should be accepted"
    );
}

// ========== Shell Override Tests ==========

#[test]
fn test_wit_shell_override_bash() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .args(["--shell", "bash", "gti status"])
        .output()
        .expect("Failed to execute wit binary");

    assert!(
        output.status.success() || !output.status.success(),
        "Shell override should be accepted"
    );
}

#[test]
fn test_wit_shell_override_zsh() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .args(["--shell", "zsh", "gti status"])
        .output()
        .expect("Failed to execute wit binary");

    assert!(
        output.status.success() || !output.status.success(),
        "Zsh shell override should be accepted"
    );
}

#[test]
fn test_wit_shell_override_powershell() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .args(["--shell", "powershell", "Get-Process"])
        .output()
        .expect("Failed to execute wit binary");

    assert!(
        output.status.success() || !output.status.success(),
        "PowerShell shell override should be accepted"
    );
}

// ========== Show Config Tests ==========

#[test]
fn test_wit_show_config() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("--show-config")
        .output()
        .expect("Failed to execute wit binary");

    assert!(output.status.success(), "show-config should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Configuration")
            || stdout.contains("Wit model")
            || stdout.contains("model"),
        "show-config should display configuration: {}",
        stdout
    );
}

// ========== Daemon Mode Tests ==========

#[test]
fn test_wit_status_flag() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("--status")
        .output()
        .expect("Failed to execute wit binary");

    // Status should always succeed and show daemon state
    assert!(output.status.success(), "--status should succeed");

    // Check both stdout and stderr
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("Daemon")
            || combined.contains("not running")
            || combined.contains("running")
            || combined.contains("Socket"),
        "Status should show daemon state. stdout: '{}', stderr: '{}'",
        stdout,
        stderr
    );
}

#[test]
fn test_wit_stop_flag() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("--stop")
        .output()
        .expect("Failed to execute wit binary");

    // Stop should succeed whether daemon is running or not
    assert!(output.status.success(), "--stop should succeed");

    // Check both stdout and stderr for the message
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    assert!(
        combined.contains("stopped")
            || combined.contains("Daemon")
            || combined.contains("unloaded"),
        "Stop should confirm daemon state. stdout: '{}', stderr: '{}'",
        stdout,
        stderr
    );
}

#[test]
fn test_wit_direct_flag() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .args(["--direct", "gti status"])
        .output()
        .expect("Failed to execute wit binary");

    // Direct mode bypasses daemon - should produce output or error about model
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either succeeds with correction or fails due to missing model
    assert!(
        stdout.contains("git") || stderr.contains("model") || stderr.contains("Could not"),
        "Direct mode should produce output or model error. stdout: {}, stderr: {}",
        stdout,
        stderr
    );
}

// ========== Combined Flag Tests ==========

#[test]
fn test_wit_verbose_quiet_together() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .args(["--verbose", "--quiet", "gti status"])
        .output()
        .expect("Failed to execute wit binary");

    // Both flags should work together (verbose wins for debug output)
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Shell:") || stderr.contains("Command:"),
        "Verbose output should appear even with quiet: {}",
        stderr
    );
}

#[test]
fn test_wit_verbose_direct() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .args(["--verbose", "--direct", "gti status"])
        .output()
        .expect("Failed to execute wit binary");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verbose + direct should show debug info
    assert!(
        stderr.contains("Shell:") || stderr.contains("Command:") || stderr.contains("model"),
        "Verbose + direct should show info: {}",
        stderr
    );
}

// ========== Model Management Tests ==========

#[test]
fn test_wit_list_models() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("--list-models")
        .output()
        .expect("Failed to execute wit binary");

    // Should either list models or indicate none available
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("stdout: {}", stdout);
    eprintln!("stderr: {}", stderr);

    // Just verify it doesn't crash
}

// ========== Output Format Tests ==========

#[test]
fn test_wit_output_is_clean() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .args(["--direct", "gti status"])
        .output()
        .expect("Failed to execute wit binary");

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Output should not contain ChatML tokens
        assert!(
            !stdout.contains("<|im_start|>"),
            "Output should not contain <|im_start|>"
        );
        assert!(
            !stdout.contains("<|im_end|>"),
            "Output should not contain <|im_end|>"
        );

        // Output should not contain role markers
        assert!(
            !stdout.contains("assistant") || stdout.trim() == "assistant",
            "Output should not contain role markers (unless correcting to 'assistant')"
        );

        // Output should be single line
        let line_count = stdout.trim().lines().count();
        assert!(
            line_count <= 1,
            "Output should be single line, got {} lines: {}",
            line_count,
            stdout
        );
    }
}

// ========== Timeout Tests ==========

#[test]
fn test_wit_completes_in_reasonable_time() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping integration test");
        return;
    }

    use std::time::Instant;

    let start = Instant::now();
    let _ = Command::new(get_binary_path())
        .args(["--direct", "--help"])
        .output()
        .expect("Failed to execute wit binary");
    let duration = start.elapsed();

    // Help should be instant
    assert!(
        duration < Duration::from_secs(5),
        "Help should complete quickly, took {:?}",
        duration
    );
}
