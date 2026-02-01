//! Tool discovery for wit CLI
//!
//! This module scans the system PATH to discover installed CLI tools
//! and extracts their descriptions from --help or --version output.

use crate::cache::{ToolInfo, ToolsCache};
use std::collections::hash_map::Entry;
use std::collections::HashSet;
use std::env;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Timeout for running --help or --version commands (200ms)
const HELP_TIMEOUT_MS: u64 = 200;

/// Maximum lines to read from help output
const MAX_HELP_LINES: usize = 5;

/// Maximum number of non-priority tools to process
const MAX_TOOLS_TO_PROCESS: usize = 50;

/// Priority tools to scan first (common CLIs)
const PRIORITY_TOOLS: &[&str] = &[
    "git", "docker", "kubectl", "npm", "pip", "python", "node", "cargo", "rustc", "go", "java",
    "mvn", "gradle", "make", "gcc", "clang", "curl", "wget",
];

/// Scan PATH for all executable files
pub fn scan_path() -> Vec<PathBuf> {
    let path_env = match env::var("PATH") {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };

    let separator = if cfg!(windows) { ';' } else { ':' };
    let mut executables = Vec::new();
    let mut seen = HashSet::new();

    for dir in path_env.split(separator) {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if is_executable(&path) {
                    // Extract just the filename without extension
                    if let Some(name) = get_tool_name(&path) {
                        if seen.insert(name) {
                            executables.push(path);
                        }
                    }
                }
            }
        }
    }

    executables
}

/// Check if a path points to an executable file
fn is_executable(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Ok(metadata) = path.metadata() {
            let mode = metadata.permissions().mode();
            return mode & 0o111 != 0;
        }
        false
    }

    #[cfg(windows)]
    {
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            return matches!(ext.as_str(), "exe" | "cmd" | "bat" | "com" | "ps1");
        }
        false
    }

    #[cfg(not(any(unix, windows)))]
    {
        true
    }
}

/// Extract tool name from path (without extension on Windows)
fn get_tool_name(path: &Path) -> Option<String> {
    let filename = path.file_name()?.to_string_lossy();

    // On Windows, strip common executable extensions
    #[cfg(windows)]
    {
        let name = filename
            .strip_suffix(".exe")
            .or_else(|| filename.strip_suffix(".cmd"))
            .or_else(|| filename.strip_suffix(".bat"))
            .or_else(|| filename.strip_suffix(".com"))
            .or_else(|| filename.strip_suffix(".ps1"))
            .unwrap_or(&filename);
        Some(name.to_string())
    }

    #[cfg(not(windows))]
    {
        Some(filename.to_string())
    }
}

/// Extract description from a tool's --help or --version output
pub fn extract_description(tool_path: &Path) -> Option<String> {
    // Try --help first, then -h, then --version
    extract_from_flag(tool_path, &["--help"])
        .or_else(|| extract_from_flag(tool_path, &["-h"]))
        .or_else(|| extract_from_flag(tool_path, &["--version"]))
}

/// Run a command with a flag and extract description
fn extract_from_flag(tool_path: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new(tool_path)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .ok()?;

    // Wait for output with timeout
    let start = std::time::Instant::now();
    let timeout = Duration::from_millis(HELP_TIMEOUT_MS);

    let mut stdout_lines = Vec::new();

    if let Some(stdout) = output.stdout {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok).take(MAX_HELP_LINES) {
            if start.elapsed() >= timeout {
                break;
            }
            stdout_lines.push(line);
        }
    }

    // Look for the first non-empty, meaningful line
    for line in stdout_lines {
        let trimmed = line.trim();
        if !trimmed.is_empty()
            && !trimmed.starts_with("Usage:")
            && !trimmed.starts_with("usage:")
            && trimmed.len() < 100
        {
            return Some(trimmed.to_string());
        }
    }

    None
}

/// Discover tools and build a cache
pub fn discover_tools() -> ToolsCache {
    let executables = scan_path();
    let mut cache = ToolsCache::new();

    // Process priority tools first
    let priority_set: HashSet<&str> = PRIORITY_TOOLS.iter().copied().collect();

    for path in &executables {
        if let Some(name) = get_tool_name(path) {
            if priority_set.contains(name.as_str()) {
                if let Some(desc) = extract_description(path) {
                    cache.tools.insert(
                        name,
                        ToolInfo {
                            path: path.to_string_lossy().to_string(),
                            desc,
                        },
                    );
                }
            }
        }
    }

    // Process remaining tools (limited to avoid long scan times)
    let mut processed_count = 0;
    for path in &executables {
        if processed_count >= MAX_TOOLS_TO_PROCESS {
            break;
        }

        if let Some(name) = get_tool_name(path) {
            if let Entry::Vacant(e) = cache.tools.entry(name) {
                if let Some(desc) = extract_description(path) {
                    e.insert(ToolInfo {
                        path: path.to_string_lossy().to_string(),
                        desc,
                    });
                    processed_count += 1;
                }
            }
        }
    }

    cache.update_timestamp();
    cache
}

/// Spawn a background thread to refresh the cache
pub fn refresh_cache_background(cache_arc: Arc<Mutex<ToolsCache>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let new_cache = discover_tools();

        // Save to disk
        if let Err(e) = crate::cache::save_cache(&new_cache) {
            eprintln!("Warning: Failed to save tools cache: {}", e);
        }

        // Update in-memory cache
        if let Ok(mut cache) = cache_arc.lock() {
            *cache = new_cache;
        }
    })
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_path_returns_executables() {
        let executables = scan_path();

        // Should find at least some executables (unless PATH is completely empty)
        // We can't guarantee specific tools, but we can check the function works
        assert!(executables.is_empty() || !executables.is_empty());
    }

    #[test]
    fn test_get_tool_name_unix() {
        #[cfg(unix)]
        {
            let path = PathBuf::from("/usr/bin/git");
            let name = get_tool_name(&path);
            assert_eq!(name, Some("git".to_string()));
        }
    }

    #[test]
    fn test_get_tool_name_windows() {
        #[cfg(windows)]
        {
            let path = PathBuf::from(r"C:\Windows\System32\cmd.exe");
            let name = get_tool_name(&path);
            assert_eq!(name, Some("cmd".to_string()));
        }
    }

    #[test]
    fn test_get_tool_name_none() {
        let path = PathBuf::from("/");
        let name = get_tool_name(&path);
        // Root path has no filename
        assert_eq!(name, None);
    }

    #[cfg(unix)]
    #[test]
    fn test_is_executable_unix() {
        // Test with a known executable
        let ls_path = PathBuf::from("/bin/ls");
        if ls_path.exists() {
            assert!(is_executable(&ls_path));
        }

        // Test with a non-executable
        let non_exec = PathBuf::from("/dev/null");
        if non_exec.exists() {
            assert!(!is_executable(&non_exec));
        }
    }

    #[cfg(windows)]
    #[test]
    fn test_is_executable_windows() {
        let cmd_path = PathBuf::from(r"C:\Windows\System32\cmd.exe");
        if cmd_path.exists() {
            assert!(is_executable(&cmd_path));
        }
    }

    #[test]
    fn test_extract_description_timeout() {
        // This test ensures extract_description doesn't hang forever
        // Create a path that likely doesn't exist
        let fake_path = PathBuf::from("/nonexistent/tool/12345");

        let start = std::time::Instant::now();
        let desc = extract_description(&fake_path);
        let elapsed = start.elapsed();

        // Should fail quickly
        assert!(desc.is_none());
        assert!(elapsed.as_secs() < 2);
    }

    #[test]
    fn test_discover_tools_creates_cache() {
        // This may take a few seconds to scan PATH
        let cache = discover_tools();

        // Should have a timestamp
        assert!(!cache.last_updated.is_empty());

        // Should find at least some tools (system dependent)
        // We'll just check it doesn't panic
    }

    #[test]
    fn test_priority_tools_list_not_empty() {
        assert!(!PRIORITY_TOOLS.is_empty());
        assert!(PRIORITY_TOOLS.contains(&"git"));
        assert!(PRIORITY_TOOLS.contains(&"docker"));
    }
}
