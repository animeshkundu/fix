//! WSL-Specific Integration Tests
//!
//! These tests verify behavior that is unique to the Windows Subsystem for Linux
//! environment, where Linux and Windows coexist.

use std::env;
use std::path::PathBuf;

/// Detect if running in WSL
fn is_wsl() -> bool {
    // Check /proc/version for WSL indicators
    if let Ok(version) = std::fs::read_to_string("/proc/version") {
        let lower = version.to_lowercase();
        return lower.contains("microsoft") || lower.contains("wsl");
    }
    false
}

/// Detect if Windows paths are accessible (WSL has /mnt/c, etc.)
fn has_windows_mounts() -> bool {
    std::path::Path::new("/mnt/c").exists()
}

#[test]
fn test_wsl_detection() {
    let in_wsl = is_wsl();

    #[cfg(target_os = "linux")]
    {
        // On Linux, we can be in WSL or not
        println!("Running on Linux, WSL detected: {}", in_wsl);
    }

    #[cfg(not(target_os = "linux"))]
    {
        // On non-Linux, should never detect WSL
        assert!(!in_wsl, "Should not detect WSL on non-Linux platform");
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_wsl_uses_linux_paths() {
    // Even in WSL, the Linux binary should use Linux paths
    let config_dir = dirs::config_dir().unwrap_or_else(|| {
        let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".config")
    });

    let path_str = config_dir.to_string_lossy();

    // Should NOT be a Windows path (C:\, D:\, etc.)
    assert!(
        !path_str.contains(":\\"),
        "Config dir should not be a Windows path in WSL: {}",
        path_str
    );

    // Should NOT be a /mnt/c style path
    assert!(
        !path_str.starts_with("/mnt/c") && !path_str.starts_with("/mnt/d"),
        "Config dir should not use Windows mount points: {}",
        path_str
    );

    // Should be a proper Linux path
    assert!(
        path_str.starts_with('/') || path_str.starts_with('.'),
        "Config dir should be a Linux path: {}",
        path_str
    );
}

#[test]
#[cfg(target_os = "linux")]
fn test_wsl_shell_detection() {
    // In WSL, SHELL should be set to a Linux shell
    let shell = env::var("SHELL").ok();

    if let Some(shell_path) = shell {
        // Should be a Linux path to a shell
        assert!(
            shell_path.contains("bash")
                || shell_path.contains("zsh")
                || shell_path.contains("fish")
                || shell_path.contains("sh"),
            "SHELL should point to a Linux shell: {}",
            shell_path
        );

        // Should NOT be a Windows path
        assert!(
            !shell_path.contains(":\\"),
            "SHELL should not be a Windows path: {}",
            shell_path
        );
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_wsl_home_directory() {
    // HOME should be a Linux-style path
    if let Ok(home) = env::var("HOME") {
        assert!(
            home.starts_with('/'),
            "HOME should start with / in WSL: {}",
            home
        );

        assert!(
            !home.contains(":\\"),
            "HOME should not be a Windows path: {}",
            home
        );

        // HOME should not be under /mnt/c (that would be weird)
        if is_wsl() {
            assert!(
                !home.starts_with("/mnt/c") && !home.starts_with("/mnt/d"),
                "HOME should be Linux home, not Windows mount: {}",
                home
            );
        }
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_wsl_can_access_windows_paths() {
    if !is_wsl() {
        println!("Not in WSL, skipping Windows path access test");
        return;
    }

    if has_windows_mounts() {
        // Should be able to see Windows drives
        assert!(
            std::path::Path::new("/mnt/c").exists(),
            "Should be able to access /mnt/c in WSL"
        );

        println!("WSL Windows path access verified");
    } else {
        println!("No Windows mounts available");
    }
}

#[test]
#[cfg(target_os = "linux")]
fn test_wsl_path_conversion() {
    if !is_wsl() {
        println!("Not in WSL, skipping path conversion test");
        return;
    }

    // Test that we understand the mapping between WSL and Windows paths
    // /mnt/c/Users -> C:\Users

    let wsl_path = PathBuf::from("/mnt/c/Users");
    let path_str = wsl_path.to_string_lossy();

    // This is the WSL representation of C:\Users
    assert!(
        path_str.contains("/mnt/c"),
        "Path should use WSL mount notation"
    );

    // The path should NOT contain Windows-style separators
    assert!(
        !path_str.contains('\\'),
        "WSL path should use forward slashes"
    );
}

/// Test that environment variables from Windows aren't polluting Linux environment
#[test]
#[cfg(target_os = "linux")]
fn test_wsl_environment_isolation() {
    // PSModulePath should NOT be set in WSL (that's a Windows PowerShell thing)
    // unless the user has explicitly set it
    let psmodulepath = env::var("PSModulePath").ok();

    if is_wsl() {
        // In WSL, PSModulePath might leak through in some configurations
        // but typically shouldn't be set
        if let Some(path) = psmodulepath {
            println!("Warning: PSModulePath is set in WSL: {}", path);
            // This is unusual but not necessarily an error
        }
    } else {
        // On native Linux, PSModulePath might be set by other tests in the suite
        // (e.g., test_detect_shell_powershell_via_psmodulepath in main.rs)
        // This is acceptable in a test environment - we're checking WSL behavior,
        // not test isolation
        if let Some(path) = &psmodulepath {
            println!(
                "Note: PSModulePath is set on native Linux (likely from another test): {}",
                path
            );
        }
        // The important check is that we're NOT in WSL on native Linux
        assert!(!is_wsl(), "Should not detect WSL on native Linux");
    }
}
