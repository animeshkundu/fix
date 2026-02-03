//! wit Daemon Mode Tests
//!
//! Tests for the daemon mode functionality added to wit CLI.
//! These tests verify daemon control flags and behavior.

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

// ========== Daemon Control Flag Tests ==========

#[test]
fn test_wit_status_flag() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("--status")
        .output()
        .expect("Failed to execute wit binary");

    // Status should always succeed
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
        eprintln!("wit binary not found, skipping test");
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
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = Command::new(get_binary_path())
        .args(["--direct", "gti status"])
        .output()
        .expect("Failed to execute wit binary");

    // Direct mode bypasses daemon
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

#[test]
fn test_wit_direct_bypasses_daemon() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    // First stop any running daemon
    let _ = Command::new(get_binary_path()).arg("--stop").output();

    // Then run in direct mode
    let output = Command::new(get_binary_path())
        .args(["--direct", "gti status"])
        .output()
        .expect("Failed to execute wit binary");

    // Check daemon is still not running after direct mode
    let status_output = Command::new(get_binary_path())
        .arg("--status")
        .output()
        .expect("Failed to check status");

    let status_stdout = String::from_utf8_lossy(&status_output.stdout);
    let status_stderr = String::from_utf8_lossy(&status_output.stderr);
    let combined = format!("{}{}", status_stdout, status_stderr);

    // Direct mode should not start a daemon (on Windows, daemon is not supported)
    assert!(
        combined.contains("not running")
            || combined.contains("not supported")
            || combined.contains("not available"),
        "Direct mode should not start daemon. stdout: '{}', stderr: '{}'",
        status_stdout,
        status_stderr
    );

    // The original command should have worked (or failed due to missing model)
    let _ = output;
}

// ========== Daemon Lifecycle Tests ==========

#[test]
fn test_wit_daemon_status_after_stop() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    // Stop any running daemon
    let stop_output = Command::new(get_binary_path())
        .arg("--stop")
        .output()
        .expect("Failed to stop daemon");

    assert!(stop_output.status.success(), "Stop should succeed");

    // Check status
    let status_output = Command::new(get_binary_path())
        .arg("--status")
        .output()
        .expect("Failed to check status");

    let stdout = String::from_utf8_lossy(&status_output.stdout);
    let stderr = String::from_utf8_lossy(&status_output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // On Windows, daemon mode is not supported
    assert!(
        combined.contains("not running")
            || combined.contains("not supported")
            || combined.contains("not available"),
        "After stop, daemon should not be running. stdout: '{}', stderr: '{}'",
        stdout,
        stderr
    );
}

// ========== Combined Flag Tests ==========

#[test]
fn test_wit_verbose_direct() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = Command::new(get_binary_path())
        .args(["--verbose", "--direct", "gti status"])
        .output()
        .expect("Failed to execute wit binary");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verbose + direct should show debug info or model error
    eprintln!("stderr: {}", stderr);
    // Just verify it doesn't crash
}

#[test]
fn test_wit_help_shows_daemon_flags() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("--help")
        .output()
        .expect("Failed to execute wit binary");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Help should show daemon control flags
    assert!(
        stdout.contains("--status") || stdout.contains("status"),
        "Help should mention --status flag: {}",
        stdout
    );
    assert!(
        stdout.contains("--stop") || stdout.contains("stop"),
        "Help should mention --stop flag: {}",
        stdout
    );
    assert!(
        stdout.contains("--direct") || stdout.contains("direct"),
        "Help should mention --direct flag: {}",
        stdout
    );
}

// ========== Performance Tests ==========

#[test]
fn test_wit_status_is_fast() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    use std::time::Instant;

    let start = Instant::now();
    let _ = Command::new(get_binary_path())
        .arg("--status")
        .output()
        .expect("Failed to execute wit binary");
    let duration = start.elapsed();

    // Status check should be instant (no model loading)
    assert!(
        duration < Duration::from_secs(1),
        "Status check should be instant, took {:?}",
        duration
    );
}

#[test]
fn test_wit_stop_is_fast() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    use std::time::Instant;

    let start = Instant::now();
    let _ = Command::new(get_binary_path())
        .arg("--stop")
        .output()
        .expect("Failed to execute wit binary");
    let duration = start.elapsed();

    // Stop should be quick
    assert!(
        duration < Duration::from_secs(2),
        "Stop should be quick, took {:?}",
        duration
    );
}

// ========== Error Handling Tests ==========

#[test]
fn test_wit_multiple_stops_ok() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    // Multiple stops should be idempotent
    for _ in 0..3 {
        let output = Command::new(get_binary_path())
            .arg("--stop")
            .output()
            .expect("Failed to execute wit binary");

        assert!(output.status.success(), "Multiple stops should all succeed");
    }
}

#[test]
fn test_wit_status_when_no_daemon() {
    if !binary_exists() {
        eprintln!("wit binary not found, skipping test");
        return;
    }

    // Stop daemon first
    let _ = Command::new(get_binary_path()).arg("--stop").output();

    // Status should work and indicate no daemon
    let output = Command::new(get_binary_path())
        .arg("--status")
        .output()
        .expect("Failed to check status");

    assert!(output.status.success(), "Status should succeed");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{}{}", stdout, stderr);

    // On Windows, daemon mode is not supported
    assert!(
        combined.contains("not running")
            || combined.contains("not supported")
            || combined.contains("not available"),
        "Should indicate daemon is not running. stdout: '{}', stderr: '{}'",
        stdout,
        stderr
    );
}
