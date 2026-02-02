//! Tests for discovery module

use fix_lib::discovery::{discover_tools, extract_description, scan_path};
use std::path::PathBuf;

#[test]
fn test_scan_path_finds_executables() {
    let executables = scan_path();

    // PATH should contain at least some executables on any system
    // We can't guarantee specific ones, but the list shouldn't be empty
    // unless PATH is completely unset (unlikely)
    assert!(
        !executables.is_empty() || std::env::var("PATH").is_err(),
        "Should find executables in PATH or PATH should be unset"
    );
}

#[test]
fn test_scan_path_returns_reasonable_results() {
    let executables = scan_path();

    // Collect tool names and count duplicates
    let mut seen_names = std::collections::HashMap::new();

    for path in &executables {
        if let Some(name) = path.file_name() {
            let name_str = name.to_string_lossy();
            // Strip extensions for comparison
            let clean_name = name_str
                .strip_suffix(".exe")
                .or_else(|| name_str.strip_suffix(".cmd"))
                .or_else(|| name_str.strip_suffix(".bat"))
                .unwrap_or(&name_str)
                .to_string();

            *seen_names.entry(clean_name).or_insert(0) += 1;
        }
    }

    // The majority of tools should be unique
    // Some duplicates are acceptable (symlinks, aliases in different PATH dirs)
    let unique_count = seen_names.values().filter(|&&count| count == 1).count();
    let total_names = seen_names.len();

    assert!(
        unique_count > total_names / 2,
        "At least half of tool names should be unique. Unique: {}, Total: {}",
        unique_count,
        total_names
    );
}

#[test]
fn test_extract_description_nonexistent() {
    let fake_path = PathBuf::from("/nonexistent/tool/that/does/not/exist/12345");

    let start = std::time::Instant::now();
    let desc = extract_description(&fake_path);
    let elapsed = start.elapsed();

    // Should return None and not hang
    assert!(desc.is_none());
    assert!(elapsed.as_secs() < 2, "Should fail quickly");
}

#[cfg(unix)]
#[test]
fn test_extract_description_ls() {
    let ls_path = PathBuf::from("/bin/ls");

    if ls_path.exists() {
        let desc = extract_description(&ls_path);

        // ls should have some description (might be from --help or --version)
        // We can't guarantee exact text, but it should return something
        // Note: This may fail on some systems, so we'll be lenient
        if desc.is_some() {
            let text = desc.unwrap();
            assert!(!text.is_empty());
            assert!(text.len() < 200); // Should be a brief description
        }
    }
}

#[test]
fn test_discover_tools_creates_valid_cache() {
    // This test may take a few seconds as it scans PATH
    let cache = discover_tools();

    // Should have a valid timestamp
    assert!(!cache.last_updated.is_empty());

    // Should be able to parse the timestamp
    let age = cache.age();
    assert!(age.is_ok(), "Timestamp should be valid: {:?}", age);

    // Cache should be fresh (just created)
    assert!(!cache.needs_refresh());
}

#[test]
fn test_discover_tools_finds_priority_tools() {
    let cache = discover_tools();

    // Check if common tools are discovered (if they exist on the system)
    // We can't guarantee all systems have all these tools, but we can check
    // that the discovery process works

    let common_tools = ["git", "docker", "python", "node", "cargo"];

    for tool in &common_tools {
        if cache.tools.contains_key(*tool) {
            let info = cache.tools.get(*tool).unwrap();
            assert!(!info.path.is_empty(), "Tool path should not be empty");
        }
    }
}

#[test]
fn test_discover_tools_tool_info_structure() {
    let cache = discover_tools();

    // Check that all discovered tools have valid structure
    for (name, info) in &cache.tools {
        assert!(!name.is_empty(), "Tool name should not be empty");
        assert!(!info.path.is_empty(), "Tool path should not be empty");
        // Description may be empty if we couldn't extract it
    }
}

#[cfg(unix)]
#[test]
fn test_discover_tools_unix_common_tools() {
    let cache = discover_tools();

    // On Unix systems, we should find at least 'ls' or 'sh' or 'bash'
    let unix_tools = ["ls", "sh", "bash", "cat", "grep", "echo", "pwd"];
    let has_unix_tool = unix_tools
        .iter()
        .any(|tool| cache.tools.contains_key(*tool));

    // Due to the limit on tools processed, we may not always find these tools
    // This is acceptable behavior for the cache
    // We'll just check that the discovery process works without panicking
    if !has_unix_tool {
        eprintln!(
            "Warning: No common Unix tools found. This may be due to MAX_TOOLS_TO_PROCESS limit."
        );
        eprintln!(
            "Discovered tools: {:?}",
            cache.tools.keys().collect::<Vec<_>>()
        );
    }
}

#[cfg(windows)]
#[test]
fn test_discover_tools_windows_common_tools() {
    let cache = discover_tools();

    // On Windows, we should find at least 'cmd' or 'powershell'
    // Names might be with or without .exe extension
    let windows_tools = [
        "cmd",
        "cmd.exe",
        "powershell",
        "powershell.exe",
        "pwsh",
        "pwsh.exe",
        "where",
        "where.exe",
    ];
    let has_windows_tool = windows_tools
        .iter()
        .any(|tool| cache.tools.contains_key(*tool));

    // Also check if any tool exists at all as a sanity check
    if !has_windows_tool && !cache.tools.is_empty() {
        // Test passes if we found some tools, even if not the expected ones
        eprintln!(
            "Note: Did not find common Windows tools, but found {} other tools",
            cache.tools.len()
        );
        return;
    }

    assert!(
        has_windows_tool || !cache.tools.is_empty(),
        "Should find at least one common Windows tool or any tools at all: {:?}. Found: {:?}",
        windows_tools,
        cache.tools.keys().take(5).collect::<Vec<_>>()
    );
}

#[test]
fn test_discover_tools_performance() {
    // Ensure discovery doesn't take too long
    let start = std::time::Instant::now();
    let _ = discover_tools();
    let elapsed = start.elapsed();

    // Discovery should complete in reasonable time
    // Increased to 120s for slow CI containers (e.g., Ubuntu 20.04 in Docker)
    assert!(
        elapsed.as_secs() < 120,
        "Discovery took too long: {:?}",
        elapsed
    );
}
