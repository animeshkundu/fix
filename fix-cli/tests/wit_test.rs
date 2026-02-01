//! wit CLI Integration Tests
//!
//! These tests verify the wit binary's progress indicators and flags.

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

#[test]
fn test_wit_help_includes_quiet_flag() {
    if !wit_binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = Command::new(get_wit_binary_path())
        .arg("--help")
        .output()
        .expect("Failed to execute wit --help command");

    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("--quiet") || stdout.contains("-q"),
        "Help should show --quiet flag"
    );
    assert!(
        stdout.contains("Disable progress indicators"),
        "Help should describe quiet flag"
    );
}

#[test]
fn test_wit_accepts_quiet_flag() {
    if !wit_binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = Command::new(get_wit_binary_path())
        .args(["--quiet", "test command"])
        .output()
        .expect("Failed to execute wit --quiet command");

    // Should not crash with --quiet flag
    assert!(
        output.status.success(),
        "wit --quiet should exit cleanly"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verify it still produces output (just no spinners)
    assert!(
        stderr.contains("wit:") || stderr.contains("command"),
        "Should still show output in quiet mode"
    );
}

#[test]
fn test_wit_accepts_verbose_flag() {
    if !wit_binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = Command::new(get_wit_binary_path())
        .args(["--verbose", "test command"])
        .output()
        .expect("Failed to execute wit --verbose command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verbose mode should show shell information
    assert!(
        stderr.contains("Shell:") || stderr.contains("Command:"),
        "Verbose mode should show shell and command info"
    );
}

#[test]
fn test_wit_quiet_and_verbose_together() {
    if !wit_binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = Command::new(get_wit_binary_path())
        .args(["--quiet", "--verbose", "test command"])
        .output()
        .expect("Failed to execute wit with --quiet --verbose flags");

    // Both flags should work together
    assert!(
        output.status.success(),
        "wit should handle both flags"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Shell:") || stderr.contains("Command:"),
        "Verbose output should still appear with quiet"
    );
}

#[test]
fn test_wit_show_config() {
    if !wit_binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = Command::new(get_wit_binary_path())
        .arg("--show-config")
        .output()
        .expect("Failed to execute wit --show-config command");

    assert!(output.status.success(), "show-config should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Configuration") || stdout.contains("Default model"),
        "show-config should display configuration"
    );
}

#[test]
fn test_wit_with_command() {
    if !wit_binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = Command::new(get_wit_binary_path())
        .arg("gti status")
        .output()
        .expect("Failed to execute wit with command argument");

    // Should exit cleanly (even if it's a placeholder)
    assert!(
        output.status.success(),
        "wit should handle commands"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("Received command") || stderr.contains("gti status"),
        "Should acknowledge the command"
    );
}

#[test]
fn test_wit_without_command() {
    if !wit_binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = Command::new(get_wit_binary_path())
        .output()
        .expect("Failed to execute wit without arguments");

    // Should exit with error code when no command provided
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
