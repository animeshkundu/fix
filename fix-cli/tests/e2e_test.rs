//! End-to-End Model Inference Tests
//!
//! These tests require the model to be downloaded and test actual inference.
//! They are marked with #[ignore] by default to avoid running in quick test cycles.
//! Run with: cargo test --test e2e_test -- --ignored

use std::env;
use std::path::PathBuf;
use std::process::Command;

/// Get the path to the compiled binary
fn get_binary_path() -> String {
    let mut path = std::env::current_exe().unwrap();
    path.pop(); // Remove test binary name
    path.pop(); // Remove deps
    path.push("fix");

    #[cfg(windows)]
    path.set_extension("exe");

    path.to_string_lossy().to_string()
}

/// Check if binary exists
fn binary_exists() -> bool {
    std::path::Path::new(&get_binary_path()).exists()
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

#[test]
#[ignore] // Run with --ignored flag
fn test_e2e_typo_correction_git() {
    if !binary_exists() {
        eprintln!("Binary not found, skipping E2E test");
        return;
    }

    if !model_exists() {
        eprintln!(
            "Model not found at {:?}, skipping E2E test",
            get_model_path()
        );
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("gti status")
        .output()
        .expect("Failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr);

    eprintln!("stdout: {}", stdout);
    eprintln!("stderr: {}", stderr);

    assert_eq!(
        stdout, "git status",
        "Should correct 'gti status' to 'git status'"
    );
}

#[test]
#[ignore]
fn test_e2e_typo_correction_docker() {
    if !binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("dcoker ps")
        .output()
        .expect("Failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    assert_eq!(
        stdout, "docker ps",
        "Should correct 'dcoker ps' to 'docker ps'"
    );
}

#[test]
#[ignore]
fn test_e2e_typo_correction_npm() {
    if !binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("nmp install")
        .output()
        .expect("Failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    assert_eq!(
        stdout, "npm install",
        "Should correct 'nmp install' to 'npm install'"
    );
}

#[test]
#[ignore]
fn test_e2e_flag_correction() {
    if !binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("ls -la")
        .output()
        .expect("Failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // ls -la should be kept as-is on Unix, or translated to PowerShell equivalent on Windows
    #[cfg(not(target_os = "windows"))]
    assert!(
        stdout == "ls -la" || stdout == "ls -al",
        "Should not change correct 'ls -la' command, got: {}",
        stdout
    );

    #[cfg(target_os = "windows")]
    assert!(
        stdout.contains("Get-ChildItem")
            || stdout.contains("dir")
            || stdout == "ls -la"
            || stdout == "ls -al",
        "Should translate 'ls -la' to PowerShell equivalent or keep as-is, got: {}",
        stdout
    );
}

#[test]
#[ignore]
fn test_e2e_verbose_mode() {
    if !binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping");
        return;
    }

    let output = Command::new(get_binary_path())
        .args(["--verbose", "gti status"])
        .output()
        .expect("Failed to execute binary");

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Verbose mode should show debug info in stderr
    assert!(
        stderr.contains("Shell") || stderr.contains("shell") || stderr.contains("Prompt"),
        "Verbose mode should show debug info"
    );
}

#[test]
#[ignore]
fn test_e2e_special_characters() {
    if !binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("echo \"hello world\"")
        .output()
        .expect("Failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Should handle quoted strings properly
    assert!(
        stdout.contains("echo") && stdout.contains("hello"),
        "Should handle special characters: {}",
        stdout
    );
}

#[test]
#[ignore]
fn test_e2e_pipe_commands() {
    if !binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("cat file.txt | gerp pattern")
        .output()
        .expect("Failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

    // Should correct gerp to grep on Unix, or translate to PowerShell equivalent on Windows
    #[cfg(not(target_os = "windows"))]
    assert!(
        stdout.contains("grep"),
        "Should correct 'gerp' to 'grep' in pipe command: {}",
        stdout
    );

    #[cfg(target_os = "windows")]
    assert!(
        stdout.contains("grep") || stdout.contains("Select-String") || stdout.contains("findstr"),
        "Should correct pipe command with typo or translate to PowerShell: {}",
        stdout
    );
}

#[test]
#[ignore]
fn test_e2e_output_format() {
    if !binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping");
        return;
    }

    let output = Command::new(get_binary_path())
        .arg("gti status")
        .output()
        .expect("Failed to execute binary");

    let stdout = String::from_utf8_lossy(&output.stdout);

    // Output should be clean - just the command, no extra formatting
    assert!(
        !stdout.contains("<|im_start|>") && !stdout.contains("<|im_end|>"),
        "Output should not contain ChatML tokens"
    );

    assert!(
        !stdout.contains("assistant") && !stdout.contains("system"),
        "Output should not contain role prefixes"
    );

    // Should be a single line
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(
        lines.len(),
        1,
        "Output should be a single line, got {} lines",
        lines.len()
    );
}

#[test]
#[ignore]
fn test_e2e_inference_time() {
    if !binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping");
        return;
    }

    use std::time::Instant;

    let start = Instant::now();

    let output = Command::new(get_binary_path())
        .arg("gti status")
        .output()
        .expect("Failed to execute binary");

    let duration = start.elapsed();

    assert!(output.status.success(), "Command should succeed");

    // Inference should complete in reasonable time (< 30 seconds)
    assert!(
        duration.as_secs() < 30,
        "Inference took too long: {:?}",
        duration
    );

    eprintln!("Inference completed in {:?}", duration);
}

#[test]
#[ignore]
fn test_e2e_output_is_clean_command_only() {
    if !binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping");
        return;
    }

    // Test multiple inputs to ensure output is always clean
    let test_cases = vec!["gti status", "dcoker ps", "nmp install", "pytohn script.py"];

    for input in test_cases {
        let output = Command::new(get_binary_path())
            .arg(input)
            .output()
            .expect("Failed to execute binary");

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
            !stdout.contains("assistant"),
            "Output for '{}' contains 'assistant'",
            input
        );
        assert!(
            !stdout.contains("system"),
            "Output for '{}' contains 'system'",
            input
        );

        // Must not contain common model artifacts
        let artifacts = ["command >", "Command:", ">>>", "```", "Output:"];
        for artifact in artifacts {
            assert!(
                !stdout.contains(artifact),
                "Output for '{}' contains artifact '{}'. Full output: '{}'",
                input,
                artifact,
                stdout_trimmed
            );
        }

        eprintln!(
            "Clean output check passed for '{}' -> '{}'",
            input, stdout_trimmed
        );
    }
}

#[test]
#[ignore]
fn test_e2e_shell_override() {
    if !binary_exists() || !model_exists() {
        eprintln!("Binary or model not found, skipping");
        return;
    }

    // Test that --shell flag works correctly
    let shells = vec!["bash", "zsh", "fish", "powershell"];

    for shell in shells {
        let output = Command::new(get_binary_path())
            .args(["--shell", shell, "gti status"])
            .output()
            .expect("Failed to execute binary");

        let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();

        // Should still produce a valid correction regardless of shell
        assert!(!stdout.is_empty(), "Output for shell '{}' is empty", shell);

        // Output should be clean
        assert!(
            !stdout.contains("<|im_start|>") && !stdout.contains("<|im_end|>"),
            "Output for shell '{}' contains ChatML tokens",
            shell
        );

        eprintln!("Shell override '{}' -> '{}'", shell, stdout);
    }
}
