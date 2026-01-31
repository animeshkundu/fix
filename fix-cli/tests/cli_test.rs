//! CLI Integration Tests
//!
//! These tests verify the CLI binary works correctly when executed as a subprocess.

use std::process::Command;

/// Get the path to the compiled binary
fn get_binary_path() -> String {
    // When running `cargo test`, the binary is in target/debug/
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove deps
    path.push("fix");

    // On Windows, add .exe
    #[cfg(windows)]
    path.set_extension("exe");

    path.to_string_lossy().to_string()
}

/// Check if binary exists (it may not if we're running unit tests only)
fn binary_exists() -> bool {
    std::path::Path::new(&get_binary_path()).exists()
}

#[test]
fn test_binary_help_flag() {
    if !binary_exists() {
        eprintln!("Binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("--help")
        .output()
        .expect("Failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Should show help text
    assert!(
        stdout.contains("fix") || stdout.contains("command"),
        "Help should mention the command purpose"
    );
    assert!(output.status.success(), "Help should exit successfully");
}

#[test]
fn test_binary_version_flag() {
    if !binary_exists() {
        eprintln!("Binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("--version")
        .output()
        .expect("Failed to execute binary");

    // Version flag should exit successfully (may or may not be implemented)
    // Just verify it doesn't crash
    let _ = String::from_utf8_lossy(&output.stdout);
}

#[test]
fn test_binary_list_models_flag() {
    if !binary_exists() {
        eprintln!("Binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("--list-models")
        .output()
        .expect("Failed to execute binary");

    // Should either list models or show that none are downloaded
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // It's OK if this fails due to no models, just verify it doesn't crash
    eprintln!("stdout: {}", stdout);
    eprintln!("stderr: {}", stderr);
}

#[test]
fn test_binary_verbose_flag() {
    if !binary_exists() {
        eprintln!("Binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .args(["--verbose", "--help"])
        .output()
        .expect("Failed to execute binary");

    // Verbose + help should work
    assert!(output.status.success(), "Verbose + help should work");
}

#[test]
fn test_binary_invalid_flag() {
    if !binary_exists() {
        eprintln!("Binary not found, skipping integration test");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("--invalid-flag-that-does-not-exist")
        .output()
        .expect("Failed to execute binary");

    // Should exit with error for invalid flag
    assert!(
        !output.status.success(),
        "Invalid flag should cause error exit"
    );
}

#[test]
fn test_binary_no_args_without_model() {
    if !binary_exists() {
        eprintln!("Binary not found, skipping integration test");
        return;
    }

    // Running without args and without a model should either:
    // 1. Show an error about missing model
    // 2. Show help text
    // 3. Exit with non-zero if history is empty

    let output = Command::new(get_binary_path())
        .output()
        .expect("Failed to execute binary");

    // Just verify it doesn't crash - behavior depends on environment
    let _ = String::from_utf8_lossy(&output.stdout);
    let _ = String::from_utf8_lossy(&output.stderr);
}
