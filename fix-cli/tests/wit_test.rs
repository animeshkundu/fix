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
        .args(["--quiet", "gti status"])
        .output()
        .expect("Failed to execute wit --quiet command");

    // With quiet flag, should produce output on stdout (the corrected command)
    // Exit code may vary depending on model availability
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Either produces corrected output or shows an error about model
    assert!(
        stdout.contains("git") || stderr.contains("model") || stderr.contains("Could not"),
        "Should either output correction or show model-related message"
    );
}

#[test]
fn test_wit_accepts_verbose_flag() {
    if !wit_binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = Command::new(get_wit_binary_path())
        .args(["--verbose", "gti status"])
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
        .args(["--quiet", "--verbose", "gti status"])
        .output()
        .expect("Failed to execute wit with --quiet --verbose flags");

    let stderr = String::from_utf8_lossy(&output.stderr);
    // Verbose should show debug info even with quiet (quiet only suppresses spinner)
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
        stdout.contains("Configuration") || stdout.contains("Wit model"),
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

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Should either output the corrected command or show model download progress
    assert!(
        stdout.contains("git") || stderr.contains("Downloading") || stderr.contains("model"),
        "Should either output correction or show model download info. stdout: {}, stderr: {}",
        stdout,
        stderr
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
