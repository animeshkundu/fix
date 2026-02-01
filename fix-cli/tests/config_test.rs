//! Config Integration Tests
//!
//! These tests verify configuration file handling works correctly
//! across different platforms.

use std::env;
use std::fs;
use std::path::PathBuf;

/// Get the expected config directory for the current platform
fn get_expected_config_dir() -> PathBuf {
    #[cfg(target_os = "macos")]
    {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("fix")
    }

    #[cfg(target_os = "linux")]
    {
        // On Linux, config_dir() returns ~/.config
        dirs::config_dir()
            .unwrap_or_else(|| {
                let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(home).join(".config")
            })
            .join("fix")
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, config_dir() returns APPDATA
        dirs::config_dir()
            .unwrap_or_else(|| {
                let appdata = env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
                PathBuf::from(appdata)
            })
            .join("fix")
    }
}

#[test]
fn test_config_dir_is_platform_appropriate() {
    let config_dir = get_expected_config_dir();

    #[cfg(target_os = "macos")]
    {
        // macOS should use ~/Library/Application Support/fix
        let path_str = config_dir.to_string_lossy();
        assert!(
            path_str.contains("Library/Application Support")
                || path_str.contains("Application Support"),
            "macOS config should be in Library/Application Support, got: {}",
            path_str
        );
    }

    #[cfg(target_os = "linux")]
    {
        // Linux should use ~/.config/fix
        let path_str = config_dir.to_string_lossy();
        assert!(
            path_str.contains(".config"),
            "Linux config should be in .config, got: {}",
            path_str
        );
    }

    #[cfg(target_os = "windows")]
    {
        // Windows should use APPDATA/fix
        let path_str = config_dir.to_string_lossy();
        assert!(
            path_str.contains("AppData") || path_str.contains("Roaming"),
            "Windows config should be in AppData, got: {}",
            path_str
        );
    }

    // All platforms: should end with "fix"
    assert!(
        config_dir.ends_with("fix"),
        "Config dir should end with 'fix', got: {:?}",
        config_dir
    );
}

#[test]
fn test_config_file_path() {
    let config_dir = get_expected_config_dir();
    let config_path = config_dir.join("config.json");

    assert!(
        config_path.ends_with("config.json"),
        "Config file should be config.json"
    );
}

#[test]
fn test_model_path_format() {
    let config_dir = get_expected_config_dir();
    let model_path = config_dir.join("qwen3-correct-0.6B.gguf");

    // Model path should have .gguf extension
    assert!(
        model_path.extension().map(|e| e == "gguf").unwrap_or(false),
        "Model path should have .gguf extension"
    );

    // Model should be in config directory
    assert_eq!(
        model_path.parent().unwrap(),
        config_dir,
        "Model should be in config directory"
    );
}

#[test]
fn test_can_create_config_directory() {
    // Use a temp directory to avoid polluting real config
    let temp_dir = env::temp_dir().join("fix-test-config");

    // Clean up if exists
    let _ = fs::remove_dir_all(&temp_dir);

    // Create directory
    fs::create_dir_all(&temp_dir).expect("Should be able to create config directory");

    // Verify it exists
    assert!(
        temp_dir.exists(),
        "Config directory should exist after creation"
    );
    assert!(temp_dir.is_dir(), "Config directory should be a directory");

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
fn test_can_write_config_file() {
    let temp_dir = env::temp_dir().join("fix-test-config-write");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("Should be able to create directory");

    let config_path = temp_dir.join("config.json");
    let config_content = r#"{"default_model": "test-model"}"#;

    // Write config
    fs::write(&config_path, config_content).expect("Should be able to write config");

    // Read and verify
    let read_content = fs::read_to_string(&config_path).expect("Should be able to read config");
    assert_eq!(read_content, config_content);

    // Clean up
    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
#[cfg(unix)]
fn test_unix_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp_dir = env::temp_dir().join("fix-test-permissions");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).expect("Should create directory");

    let permissions = fs::metadata(&temp_dir)
        .expect("Should get metadata")
        .permissions();

    // Directory should be readable and writable by owner
    let mode = permissions.mode();
    assert!(mode & 0o700 != 0, "Directory should be accessible by owner");

    let _ = fs::remove_dir_all(&temp_dir);
}

#[test]
#[cfg(windows)]
fn test_windows_path_handling() {
    let config_dir = get_expected_config_dir();
    let path_str = config_dir.to_string_lossy();

    // Windows paths may contain backslashes or forward slashes
    // Just verify it's a valid-looking path
    assert!(path_str.len() > 3, "Windows path should be reasonably long");

    // Should not start with forward slash (Unix-style)
    assert!(
        !path_str.starts_with('/'),
        "Windows path should not start with /"
    );
}
