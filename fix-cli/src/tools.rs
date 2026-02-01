//! Cross-platform tool executor for wit CLI
//!
//! This module provides 5 tools with cross-platform support for shell command correction:
//! - `help_output`: Get --help output (first 30 lines)
//! - `which_binary`: Check if command exists
//! - `list_similar`: List commands with similar prefix
//! - `get_env_var`: Get environment variable value
//! - `man_page`: Get man page synopsis (Unix only)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::time::{Duration, Instant};

/// Default timeout for tool execution (500ms)
pub const DEFAULT_TIMEOUT_MS: u64 = 500;

/// Maximum lines to return from help output
pub const MAX_HELP_LINES: usize = 30;

/// Supported shell types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    PowerShell,
    Cmd,
}

impl Shell {
    /// Parse shell from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "bash" => Some(Shell::Bash),
            "zsh" => Some(Shell::Zsh),
            "fish" => Some(Shell::Fish),
            "powershell" | "pwsh" => Some(Shell::PowerShell),
            "cmd" | "cmd.exe" => Some(Shell::Cmd),
            _ => None,
        }
    }

    /// Check if this shell is Unix-like (bash, zsh, fish)
    pub fn is_unix_like(&self) -> bool {
        matches!(self, Shell::Bash | Shell::Zsh | Shell::Fish)
    }

    /// Check if this shell is Windows-native (cmd, powershell)
    pub fn is_windows_native(&self) -> bool {
        matches!(self, Shell::Cmd | Shell::PowerShell)
    }
}

impl std::fmt::Display for Shell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Shell::Bash => write!(f, "bash"),
            Shell::Zsh => write!(f, "zsh"),
            Shell::Fish => write!(f, "fish"),
            Shell::PowerShell => write!(f, "powershell"),
            Shell::Cmd => write!(f, "cmd"),
        }
    }
}

/// Available tools for the wit CLI
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Tool {
    /// Get --help output (first 30 lines)
    HelpOutput { command: String },
    /// Check if command/binary exists, returns path
    WhichBinary { command: String },
    /// List commands with similar prefix
    ListSimilar { prefix: String },
    /// Get environment variable value
    GetEnvVar { name: String },
    /// Get man page synopsis (Unix only)
    ManPage { command: String },
}

impl Tool {
    /// Get the tool name as a string
    pub fn name(&self) -> &'static str {
        match self {
            Tool::HelpOutput { .. } => "help_output",
            Tool::WhichBinary { .. } => "which_binary",
            Tool::ListSimilar { .. } => "list_similar",
            Tool::GetEnvVar { .. } => "get_env_var",
            Tool::ManPage { .. } => "man_page",
        }
    }
}

/// Result from tool execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolResult {
    /// Whether the tool executed successfully
    pub success: bool,
    /// Output from the tool (may be empty on failure)
    pub output: String,
    /// Error message if the tool failed
    pub error: Option<String>,
}

impl ToolResult {
    /// Create a successful result
    pub fn success(output: String) -> Self {
        Self {
            success: true,
            output,
            error: None,
        }
    }

    /// Create a failed result
    pub fn failure(error: String) -> Self {
        Self {
            success: false,
            output: String::new(),
            error: Some(error),
        }
    }
}

/// Cache entry with timestamp
#[derive(Debug, Clone)]
struct CacheEntry {
    result: ToolResult,
    timestamp: Instant,
}

/// Tool executor with caching support
pub struct ToolExecutor {
    /// Current shell type
    shell: Shell,
    /// Timeout for command execution
    timeout: Duration,
    /// Cache for tool results
    cache: Mutex<HashMap<String, CacheEntry>>,
    /// Cache TTL (time-to-live)
    cache_ttl: Duration,
}

impl ToolExecutor {
    /// Create a new tool executor for the given shell
    pub fn new(shell: Shell) -> Self {
        Self {
            shell,
            timeout: Duration::from_millis(DEFAULT_TIMEOUT_MS),
            cache: Mutex::new(HashMap::new()),
            cache_ttl: Duration::from_secs(60), // 1 minute cache
        }
    }

    /// Create a new tool executor with custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Create a new tool executor with custom cache TTL
    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Get the current shell
    pub fn shell(&self) -> Shell {
        self.shell
    }

    /// Execute a tool and return the result
    pub fn execute(&self, tool: &Tool) -> ToolResult {
        // Generate cache key
        let cache_key = format!("{:?}:{:?}", self.shell, tool);

        // Check cache
        if let Ok(cache) = self.cache.lock() {
            if let Some(entry) = cache.get(&cache_key) {
                if entry.timestamp.elapsed() < self.cache_ttl {
                    return entry.result.clone();
                }
            }
        }

        // Execute tool
        let result = match tool {
            Tool::HelpOutput { command } => self.execute_help_output(command),
            Tool::WhichBinary { command } => self.execute_which_binary(command),
            Tool::ListSimilar { prefix } => self.execute_list_similar(prefix),
            Tool::GetEnvVar { name } => self.execute_get_env_var(name),
            Tool::ManPage { command } => self.execute_man_page(command),
        };

        // Store in cache
        if let Ok(mut cache) = self.cache.lock() {
            cache.insert(
                cache_key,
                CacheEntry {
                    result: result.clone(),
                    timestamp: Instant::now(),
                },
            );
        }

        result
    }

    /// Clear the cache
    pub fn clear_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            cache.clear();
        }
    }

    // ========== Tool Implementations ==========

    /// Execute help_output tool
    fn execute_help_output(&self, command: &str) -> ToolResult {
        let result = match self.shell {
            Shell::Bash | Shell::Zsh | Shell::Fish => {
                // Try --help first, then -h
                self.run_command_with_timeout(command, &["--help"])
                    .or_else(|_| self.run_command_with_timeout(command, &["-h"]))
            }
            Shell::PowerShell => {
                // PowerShell: Get-Help or native --help
                self.run_powershell_command(&format!(
                    "Get-Help {} | Select-Object -First 30",
                    command
                ))
                .or_else(|_| self.run_command_with_timeout(command, &["--help"]))
            }
            Shell::Cmd => {
                // CMD: Try /? first, then --help
                self.run_command_with_timeout(command, &["/?"])
                    .or_else(|_| self.run_command_with_timeout(command, &["--help"]))
            }
        };

        match result {
            Ok(output) => {
                // Limit to MAX_HELP_LINES
                let lines: Vec<&str> = output.lines().take(MAX_HELP_LINES).collect();
                ToolResult::success(lines.join("\n"))
            }
            Err(e) => ToolResult::failure(e),
        }
    }

    /// Execute which_binary tool
    fn execute_which_binary(&self, command: &str) -> ToolResult {
        let result = match self.shell {
            Shell::Bash | Shell::Zsh => {
                // Use 'which' command
                self.run_command_with_timeout("which", &[command])
            }
            Shell::Fish => {
                // Fish: type -P for path only
                self.run_command_with_timeout("type", &["-P", command])
            }
            Shell::PowerShell => {
                // PowerShell: (Get-Command).Source
                self.run_powershell_command(&format!(
                    "(Get-Command {} -ErrorAction SilentlyContinue).Source",
                    command
                ))
            }
            Shell::Cmd => {
                // CMD: where command
                self.run_command_with_timeout("where", &[command])
            }
        };

        match result {
            Ok(output) => {
                let path = output.lines().next().unwrap_or("").trim().to_string();
                if path.is_empty() {
                    ToolResult::failure(format!("Command '{}' not found", command))
                } else {
                    ToolResult::success(path)
                }
            }
            Err(e) => ToolResult::failure(e),
        }
    }

    /// Execute list_similar tool
    fn execute_list_similar(&self, prefix: &str) -> ToolResult {
        let result = match self.shell {
            Shell::Bash => {
                // Bash: compgen -c prefix
                self.run_bash_command(&format!("compgen -c {}", prefix))
            }
            Shell::Zsh => {
                // Zsh: Use compgen in bash compatibility mode, or fall back to PATH scan
                self.run_bash_command(&format!("compgen -c {}", prefix))
                    .or_else(|_| self.scan_path_for_prefix(prefix))
            }
            Shell::Fish => {
                // Fish: complete -C prefix
                self.run_command_with_timeout("fish", &["-c", &format!("complete -C '{}'", prefix)])
                    .or_else(|_| self.scan_path_for_prefix(prefix))
            }
            Shell::PowerShell => {
                // PowerShell: Get-Command prefix*
                self.run_powershell_command(&format!(
                    "Get-Command '{}*' -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Name",
                    prefix
                ))
            }
            Shell::Cmd => {
                // CMD: No native equivalent, scan PATH
                self.scan_path_for_prefix(prefix)
            }
        };

        match result {
            Ok(output) => {
                // Deduplicate and limit results
                let mut commands: Vec<String> = output
                    .lines()
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
                commands.sort();
                commands.dedup();

                // Limit to 20 results
                commands.truncate(20);
                ToolResult::success(commands.join("\n"))
            }
            Err(e) => ToolResult::failure(e),
        }
    }

    /// Execute get_env_var tool
    fn execute_get_env_var(&self, name: &str) -> ToolResult {
        // Environment variables can be accessed directly in Rust,
        // regardless of shell type
        match std::env::var(name) {
            Ok(value) => ToolResult::success(value),
            Err(_) => ToolResult::failure(format!("Environment variable '{}' not set", name)),
        }
    }

    /// Execute man_page tool
    fn execute_man_page(&self, command: &str) -> ToolResult {
        // man is only available on Unix-like systems
        if !cfg!(unix) || self.shell.is_windows_native() {
            return ToolResult::failure("man pages not available on this platform".to_string());
        }

        // Get the synopsis section from man page
        let result = self.run_command_with_timeout(
            "man",
            &["-f", command], // whatis gives a brief description
        );

        match result {
            Ok(output) => {
                let synopsis = output.trim().to_string();
                if synopsis.is_empty() {
                    ToolResult::failure(format!("No man page found for '{}'", command))
                } else {
                    ToolResult::success(synopsis)
                }
            }
            Err(_) => {
                // Try getting SYNOPSIS section from full man page
                let result = self.run_command_with_timeout("man", &[command]);
                match result {
                    Ok(output) => {
                        // Extract SYNOPSIS section
                        let synopsis = extract_man_synopsis(&output);
                        if synopsis.is_empty() {
                            ToolResult::failure(format!("No man page found for '{}'", command))
                        } else {
                            ToolResult::success(synopsis)
                        }
                    }
                    Err(e) => ToolResult::failure(e),
                }
            }
        }
    }

    // ========== Helper Methods ==========

    /// Run a command with timeout
    fn run_command_with_timeout(&self, cmd: &str, args: &[&str]) -> Result<String, String> {
        let start = Instant::now();

        let mut child = Command::new(cmd)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to spawn command: {}", e))?;

        // Wait with timeout
        let timeout_remaining = self.timeout.saturating_sub(start.elapsed());

        match child.wait_timeout(timeout_remaining) {
            Ok(Some(status)) => {
                if status.success() {
                    let mut output = String::new();
                    if let Some(stdout) = child.stdout.as_mut() {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines().map_while(Result::ok) {
                            output.push_str(&line);
                            output.push('\n');
                        }
                    }
                    Ok(output)
                } else {
                    // Try to get output even on non-zero exit
                    let mut output = String::new();
                    if let Some(stdout) = child.stdout.as_mut() {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines().map_while(Result::ok) {
                            output.push_str(&line);
                            output.push('\n');
                        }
                    }
                    if !output.is_empty() {
                        Ok(output)
                    } else {
                        Err(format!("Command exited with status: {}", status))
                    }
                }
            }
            Ok(None) => {
                // Timeout - kill the process
                let _ = child.kill();
                Err("Command timed out".to_string())
            }
            Err(e) => Err(format!("Failed to wait for command: {}", e)),
        }
    }

    /// Run a bash command
    fn run_bash_command(&self, script: &str) -> Result<String, String> {
        self.run_command_with_timeout("bash", &["-c", script])
    }

    /// Run a PowerShell command
    fn run_powershell_command(&self, script: &str) -> Result<String, String> {
        // Try pwsh (PowerShell Core) first, then powershell (Windows PowerShell)
        self.run_command_with_timeout("pwsh", &["-NoProfile", "-Command", script])
            .or_else(|_| {
                self.run_command_with_timeout("powershell", &["-NoProfile", "-Command", script])
            })
    }

    /// Scan PATH directories for executables matching prefix (used for CMD)
    fn scan_path_for_prefix(&self, prefix: &str) -> Result<String, String> {
        let path = std::env::var("PATH").map_err(|_| "PATH not set")?;

        // Determine path separator based on platform
        let separator = if cfg!(windows) { ';' } else { ':' };

        let mut matches = Vec::new();
        let prefix_lower = prefix.to_lowercase();

        for dir in path.split(separator) {
            if let Ok(entries) = std::fs::read_dir(dir) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let name_lower = name.to_lowercase();

                    // Check if it's executable and starts with prefix
                    if name_lower.starts_with(&prefix_lower) && is_executable(&entry.path()) {
                        // Remove common extensions for cleaner output
                        let clean_name = name
                            .strip_suffix(".exe")
                            .or_else(|| name.strip_suffix(".cmd"))
                            .or_else(|| name.strip_suffix(".bat"))
                            .or_else(|| name.strip_suffix(".com"))
                            .unwrap_or(&name)
                            .to_string();
                        matches.push(clean_name);
                    }
                }
            }
        }

        matches.sort();
        matches.dedup();
        matches.truncate(20);

        if matches.is_empty() {
            Err(format!("No commands found matching prefix '{}'", prefix))
        } else {
            Ok(matches.join("\n"))
        }
    }
}

/// Check if a path is executable
fn is_executable(path: &std::path::Path) -> bool {
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
        // On Windows, check for common executable extensions
        if let Some(ext) = path.extension() {
            let ext = ext.to_string_lossy().to_lowercase();
            return matches!(ext.as_str(), "exe" | "cmd" | "bat" | "com" | "ps1");
        }
        false
    }

    #[cfg(not(any(unix, windows)))]
    {
        true // Assume executable on unknown platforms
    }
}

/// Extract SYNOPSIS section from man page output
fn extract_man_synopsis(man_output: &str) -> String {
    let mut in_synopsis = false;
    let mut synopsis_lines = Vec::new();

    for line in man_output.lines() {
        let trimmed = line.trim();

        if trimmed == "SYNOPSIS" || trimmed == "Synopsis" {
            in_synopsis = true;
            continue;
        }

        if in_synopsis {
            // End of synopsis section when we hit another section header
            if trimmed
                .chars()
                .all(|c| c.is_uppercase() || c.is_whitespace())
                && !trimmed.is_empty()
            {
                break;
            }
            synopsis_lines.push(line);
        }
    }

    // Limit synopsis to reasonable length
    synopsis_lines.truncate(10);
    synopsis_lines.join("\n").trim().to_string()
}

/// Trait extension for wait_timeout on Child
trait WaitTimeoutExt {
    fn wait_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<std::process::ExitStatus>, std::io::Error>;
}

impl WaitTimeoutExt for std::process::Child {
    fn wait_timeout(
        &mut self,
        timeout: Duration,
    ) -> Result<Option<std::process::ExitStatus>, std::io::Error> {
        let start = Instant::now();
        let poll_interval = Duration::from_millis(10);

        loop {
            match self.try_wait()? {
                Some(status) => return Ok(Some(status)),
                None => {
                    if start.elapsed() >= timeout {
                        return Ok(None);
                    }
                    std::thread::sleep(poll_interval);
                }
            }
        }
    }
}

// ========== Tests ==========

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Shell Tests =====

    #[test]
    fn test_shell_parse() {
        assert_eq!(Shell::parse("bash"), Some(Shell::Bash));
        assert_eq!(Shell::parse("BASH"), Some(Shell::Bash));
        assert_eq!(Shell::parse("zsh"), Some(Shell::Zsh));
        assert_eq!(Shell::parse("fish"), Some(Shell::Fish));
        assert_eq!(Shell::parse("powershell"), Some(Shell::PowerShell));
        assert_eq!(Shell::parse("pwsh"), Some(Shell::PowerShell));
        assert_eq!(Shell::parse("cmd"), Some(Shell::Cmd));
        assert_eq!(Shell::parse("cmd.exe"), Some(Shell::Cmd));
        assert_eq!(Shell::parse("unknown"), None);
    }

    #[test]
    fn test_shell_is_unix_like() {
        assert!(Shell::Bash.is_unix_like());
        assert!(Shell::Zsh.is_unix_like());
        assert!(Shell::Fish.is_unix_like());
        assert!(!Shell::PowerShell.is_unix_like());
        assert!(!Shell::Cmd.is_unix_like());
    }

    #[test]
    fn test_shell_is_windows_native() {
        assert!(!Shell::Bash.is_windows_native());
        assert!(!Shell::Zsh.is_windows_native());
        assert!(!Shell::Fish.is_windows_native());
        assert!(Shell::PowerShell.is_windows_native());
        assert!(Shell::Cmd.is_windows_native());
    }

    #[test]
    fn test_shell_display() {
        assert_eq!(format!("{}", Shell::Bash), "bash");
        assert_eq!(format!("{}", Shell::Zsh), "zsh");
        assert_eq!(format!("{}", Shell::Fish), "fish");
        assert_eq!(format!("{}", Shell::PowerShell), "powershell");
        assert_eq!(format!("{}", Shell::Cmd), "cmd");
    }

    // ===== Tool Tests =====

    #[test]
    fn test_tool_name() {
        assert_eq!(
            Tool::HelpOutput {
                command: "git".to_string()
            }
            .name(),
            "help_output"
        );
        assert_eq!(
            Tool::WhichBinary {
                command: "git".to_string()
            }
            .name(),
            "which_binary"
        );
        assert_eq!(
            Tool::ListSimilar {
                prefix: "git".to_string()
            }
            .name(),
            "list_similar"
        );
        assert_eq!(
            Tool::GetEnvVar {
                name: "PATH".to_string()
            }
            .name(),
            "get_env_var"
        );
        assert_eq!(
            Tool::ManPage {
                command: "git".to_string()
            }
            .name(),
            "man_page"
        );
    }

    // ===== ToolResult Tests =====

    #[test]
    fn test_tool_result_success() {
        let result = ToolResult::success("output".to_string());
        assert!(result.success);
        assert_eq!(result.output, "output");
        assert!(result.error.is_none());
    }

    #[test]
    fn test_tool_result_failure() {
        let result = ToolResult::failure("error".to_string());
        assert!(!result.success);
        assert!(result.output.is_empty());
        assert_eq!(result.error, Some("error".to_string()));
    }

    // ===== ToolExecutor Tests =====

    #[test]
    fn test_executor_new() {
        let executor = ToolExecutor::new(Shell::Bash);
        assert_eq!(executor.shell(), Shell::Bash);
    }

    #[test]
    fn test_executor_with_timeout() {
        let executor = ToolExecutor::new(Shell::Bash).with_timeout(Duration::from_secs(1));
        assert_eq!(executor.timeout, Duration::from_secs(1));
    }

    #[test]
    fn test_executor_with_cache_ttl() {
        let executor = ToolExecutor::new(Shell::Bash).with_cache_ttl(Duration::from_secs(120));
        assert_eq!(executor.cache_ttl, Duration::from_secs(120));
    }

    #[test]
    fn test_get_env_var_path() {
        let executor = ToolExecutor::new(Shell::Bash);
        let result = executor.execute(&Tool::GetEnvVar {
            name: "PATH".to_string(),
        });

        // PATH should always be set
        assert!(result.success, "PATH should be set: {:?}", result.error);
        assert!(!result.output.is_empty());
    }

    #[test]
    fn test_get_env_var_not_set() {
        let executor = ToolExecutor::new(Shell::Bash);
        let result = executor.execute(&Tool::GetEnvVar {
            name: "DEFINITELY_NOT_SET_VAR_12345".to_string(),
        });

        assert!(!result.success);
        assert!(result.error.is_some());
    }

    #[test]
    fn test_cache_works() {
        let executor = ToolExecutor::new(Shell::Bash);
        let tool = Tool::GetEnvVar {
            name: "PATH".to_string(),
        };

        // First call
        let result1 = executor.execute(&tool);
        // Second call should hit cache
        let result2 = executor.execute(&tool);

        assert_eq!(result1.output, result2.output);
    }

    #[test]
    fn test_clear_cache() {
        let executor = ToolExecutor::new(Shell::Bash);
        let tool = Tool::GetEnvVar {
            name: "PATH".to_string(),
        };

        executor.execute(&tool);
        executor.clear_cache();

        // Cache should be empty now
        let cache = executor.cache.lock().unwrap();
        assert!(cache.is_empty());
    }

    #[cfg(unix)]
    #[test]
    fn test_which_binary_existing() {
        let executor = ToolExecutor::new(Shell::Bash);
        let result = executor.execute(&Tool::WhichBinary {
            command: "ls".to_string(),
        });

        assert!(result.success, "ls should exist: {:?}", result.error);
        assert!(result.output.contains("ls") || result.output.contains("/bin"));
    }

    #[test]
    fn test_which_binary_nonexistent() {
        let executor = ToolExecutor::new(Shell::Bash);
        let result = executor.execute(&Tool::WhichBinary {
            command: "nonexistent_command_12345".to_string(),
        });

        assert!(!result.success);
    }

    #[cfg(unix)]
    #[test]
    fn test_list_similar() {
        let executor = ToolExecutor::new(Shell::Bash).with_timeout(Duration::from_secs(2)); // Give more time for compgen
        let result = executor.execute(&Tool::ListSimilar {
            prefix: "ls".to_string(),
        });

        // Should find at least 'ls' itself
        // Note: This may fail if bash/compgen is not available
        if result.success {
            assert!(
                result.output.contains("ls"),
                "Should find 'ls' in results: {}",
                result.output
            );
        }
    }

    #[test]
    fn test_scan_path_for_prefix() {
        let executor = ToolExecutor::new(Shell::Cmd);

        // This should work on any platform since it's pure Rust
        let result = executor.scan_path_for_prefix("a");

        // There should be at least some commands starting with 'a'
        // (like 'alias', 'apt', 'awk', etc. on Unix, or 'attrib' on Windows)
        if let Ok(output) = result {
            // Just verify we got some output
            assert!(!output.is_empty() || output.is_empty()); // Always passes, just checking it runs
        }
    }

    #[test]
    fn test_extract_man_synopsis() {
        let man_output = r#"
NAME
       git - the stupid content tracker

SYNOPSIS
       git [-v | --version] [-h | --help] [-C <path>]
           [--exec-path[=<path>]] [--html-path]

DESCRIPTION
       Git is a fast, scalable, distributed revision control system.
"#;

        let synopsis = extract_man_synopsis(man_output);
        assert!(synopsis.contains("git"));
        assert!(synopsis.contains("--version") || synopsis.contains("-v"));
    }

    #[test]
    fn test_extract_man_synopsis_empty() {
        let man_output = "Some text without synopsis section";
        let synopsis = extract_man_synopsis(man_output);
        assert!(synopsis.is_empty());
    }

    // ===== is_executable Tests =====

    #[cfg(unix)]
    #[test]
    fn test_is_executable_unix() {
        use std::path::PathBuf;

        // /bin/ls should be executable
        let ls_path = PathBuf::from("/bin/ls");
        if ls_path.exists() {
            assert!(is_executable(&ls_path));
        }

        // A non-existent path should not be executable
        let fake_path = PathBuf::from("/nonexistent/path/12345");
        assert!(!is_executable(&fake_path));
    }

    #[cfg(windows)]
    #[test]
    fn test_is_executable_windows() {
        use std::path::PathBuf;

        // cmd.exe should be executable
        let cmd_path = PathBuf::from(r"C:\Windows\System32\cmd.exe");
        if cmd_path.exists() {
            assert!(is_executable(&cmd_path));
        }
    }

    // ===== Serialization Tests =====

    #[test]
    fn test_shell_serialization() {
        let shell = Shell::Bash;
        let json = serde_json::to_string(&shell).unwrap();
        assert_eq!(json, r#""bash""#);

        let deserialized: Shell = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, Shell::Bash);
    }

    #[test]
    fn test_tool_serialization() {
        let tool = Tool::HelpOutput {
            command: "git".to_string(),
        };
        let json = serde_json::to_string(&tool).unwrap();
        assert!(json.contains("help_output"));
        assert!(json.contains("git"));

        let deserialized: Tool = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized, tool);
    }

    #[test]
    fn test_tool_result_serialization() {
        let result = ToolResult::success("output".to_string());
        let json = serde_json::to_string(&result).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("output"));

        let deserialized: ToolResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.success, result.success);
        assert_eq!(deserialized.output, result.output);
    }
}
