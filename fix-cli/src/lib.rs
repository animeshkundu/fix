//! fix_lib - Shared library for the fix CLI tools
//!
//! This library provides common functionality for shell command correction,
//! including model management, shell detection, and prompt building.

pub mod agent;
pub mod parser;
pub mod progress;
pub mod tools;

use indicatif::{ProgressBar, ProgressStyle};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

// ===== Constants =====

/// HuggingFace repository containing the model files
pub const HF_REPO: &str = "animeshkundu/cmd-correct";

/// Default model name used when no model is specified
pub const DEFAULT_MODEL: &str = "qwen3-correct-0.6B";

// ===== Configuration =====

/// Persistent configuration for the fix CLI
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    pub default_model: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_model: DEFAULT_MODEL.to_string(),
        }
    }
}

/// Represents an available model on HuggingFace
pub struct AvailableModel {
    pub name: String,
    pub size: u64,
}

// ===== Path Functions =====

/// Get the platform-specific configuration directory for the fix CLI
pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("fix")
}

/// Get the path to the configuration file
pub fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

/// Load configuration from disk, returning default if not found
pub fn load_config() -> Config {
    let path = config_path();
    if path.exists() {
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str(&content) {
                return config;
            }
        }
    }
    Config::default()
}

/// Save configuration to disk
pub fn save_config(config: &Config) -> Result<(), String> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(config_path(), content).map_err(|e| format!("Failed to save config: {}", e))
}

// ===== Model Management =====

/// Fetch available models from HuggingFace
pub fn fetch_available_models() -> Result<Vec<AvailableModel>, String> {
    let url = format!("https://huggingface.co/api/models/{}/tree/main", HF_REPO);
    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(&url).send().map_err(|e| {
        format!(
            "Failed to connect to HuggingFace. Check your internet connection.\nError: {}",
            e
        )
    })?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to fetch models: HTTP {}",
            response.status()
        ));
    }

    let files: Vec<serde_json::Value> = response.json().map_err(|e| e.to_string())?;

    Ok(files
        .iter()
        .filter_map(|f| {
            let path = f.get("path")?.as_str()?;
            if path.ends_with(".gguf") {
                Some(AvailableModel {
                    name: path.trim_end_matches(".gguf").to_string(),
                    size: f.get("size").and_then(|s| s.as_u64()).unwrap_or(0),
                })
            } else {
                None
            }
        })
        .collect())
}

/// List available models and print to stdout
pub fn list_models(config: &Config) -> Result<(), String> {
    eprintln!("Fetching available models...");
    let models = fetch_available_models()?;

    if models.is_empty() {
        println!("No models available in repository.");
        return Ok(());
    }

    println!("\nAvailable models:");
    for model in models {
        let size_mb = model.size as f64 / (1024.0 * 1024.0);
        let current = if model.name == config.default_model {
            " [current]"
        } else {
            ""
        };
        println!("  {}  ({:.0} MB){}", model.name, size_mb, current);
    }
    println!();
    Ok(())
}

/// Validate that a model exists on HuggingFace
pub fn validate_model_exists(model_name: &str) -> Result<(), String> {
    let models = fetch_available_models()?;
    if models.iter().any(|m| m.name == model_name) {
        Ok(())
    } else {
        let names: Vec<_> = models.iter().map(|m| m.name.as_str()).collect();
        Err(format!(
            "Model '{}' not found.\nAvailable models: {}",
            model_name,
            names.join(", ")
        ))
    }
}

/// Download a model from HuggingFace
pub fn download_model(model_name: &str) -> Result<PathBuf, String> {
    let url = format!(
        "https://huggingface.co/{}/resolve/main/{}.gguf",
        HF_REPO, model_name
    );
    let dest = config_dir().join(format!("{}.gguf", model_name));

    // Create directory if needed
    std::fs::create_dir_all(config_dir())
        .map_err(|e| format!("Failed to create config directory: {}", e))?;

    eprintln!("Downloading {}...", model_name);

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(3600)) // 1 hour timeout for large files
        .build()
        .map_err(|e| e.to_string())?;

    let response = client.get(&url).send().map_err(|e| {
        format!(
            "Failed to connect to HuggingFace. Check your internet connection.\nError: {}",
            e
        )
    })?;

    if !response.status().is_success() {
        return Err(format!("Download failed: HTTP {}", response.status()));
    }

    let total = response.content_length().unwrap_or(0);

    let pb = ProgressBar::new(total);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .unwrap()
            .progress_chars("=>-"),
    );

    // Write to a temp file first, then rename (atomic operation)
    let temp_dest = dest.with_extension("gguf.tmp");
    let mut file = File::create(&temp_dest).map_err(|e| format!("Failed to create file: {}", e))?;

    let mut downloaded = 0u64;
    let mut reader = response;
    let mut buf = [0u8; 8192];

    loop {
        let n = reader
            .read(&mut buf)
            .map_err(|e| format!("Download error: {}", e))?;
        if n == 0 {
            break;
        }
        file.write_all(&buf[..n])
            .map_err(|e| format!("Write error: {}", e))?;
        downloaded += n as u64;
        pb.set_position(downloaded);
    }

    pb.finish_and_clear();

    // Rename temp file to final destination
    std::fs::rename(&temp_dest, &dest)
        .map_err(|e| format!("Failed to finalize download: {}", e))?;

    eprintln!("✓ Downloaded to {}", dest.display());
    Ok(dest)
}

/// Get the expected path for a model by name
pub fn get_model_path(model_name: &str) -> PathBuf {
    config_dir().join(format!("{}.gguf", model_name))
}

/// Find or download a model by name
pub fn find_or_download_model(model_name: &str, force_download: bool) -> Result<PathBuf, String> {
    let model_path = get_model_path(model_name);

    if model_path.exists() && !force_download {
        return Ok(model_path);
    }

    if force_download {
        eprintln!("Re-downloading {}...", model_name);
    }

    // Validate model exists in repo before downloading
    eprintln!("Checking model availability...");
    validate_model_exists(model_name)?;

    download_model(model_name)
}

/// Find the model path to use, either from override, or configured default
pub fn find_model_path(
    override_path: Option<PathBuf>,
    config: &Config,
    force_update: bool,
) -> Result<PathBuf, String> {
    // If user specified a path, use it directly
    if let Some(path) = override_path {
        if path.exists() {
            return Ok(path);
        }
        return Err(format!("Model not found at: {}", path.display()));
    }

    // Otherwise, find or download the configured default model
    find_or_download_model(&config.default_model, force_update)
}

// ===== Shell Detection =====

/// Detect the current shell from environment variables
pub fn detect_shell() -> String {
    // Unix: check SHELL env var
    if let Ok(shell_path) = env::var("SHELL") {
        if let Some(name) = shell_path.rsplit('/').next() {
            return name.to_string();
        }
    }

    // PowerShell (works on all platforms)
    if env::var("PSModulePath").is_ok() {
        return "powershell".to_string();
    }

    // Windows fallback
    if cfg!(windows) {
        return "cmd".to_string();
    }

    // Unix fallback
    "bash".to_string()
}

// ===== Prompt Building =====

/// Build a ChatML-formatted prompt for the model
pub fn build_prompt(shell: &str, command: &str, _error: Option<&str>) -> String {
    // Match the exact format used in training data
    format!(
        "<|im_start|>system\n\
         You are a shell command corrector for {}. Output only the corrected command.<|im_end|>\n\
         <|im_start|>user\n\
         {}<|im_end|>\n\
         <|im_start|>assistant\n",
        shell, command
    )
}

// ===== Logging =====

/// Suppress llama.cpp log output
pub fn suppress_llama_logs() {
    unsafe {
        llama_cpp_sys_2::ggml_log_set(None, std::ptr::null_mut());
        llama_cpp_sys_2::llama_log_set(None, std::ptr::null_mut());
    }
}

// ===== stderr Redirect =====

#[cfg(unix)]
pub mod stderr_redirect {
    use std::fs::File;
    use std::os::unix::io::AsRawFd;

    pub fn redirect() -> Option<i32> {
        unsafe {
            let saved = libc::dup(libc::STDERR_FILENO);
            if let Ok(devnull) = File::open("/dev/null") {
                libc::dup2(devnull.as_raw_fd(), libc::STDERR_FILENO);
                return Some(saved);
            }
        }
        None
    }

    pub fn restore(saved: i32) {
        unsafe {
            libc::dup2(saved, libc::STDERR_FILENO);
            libc::close(saved);
        }
    }
}

#[cfg(windows)]
pub mod stderr_redirect {
    use std::fs::OpenOptions;
    use std::os::windows::io::AsRawHandle;

    pub struct SavedStderr {
        pub saved_fd: i32,
    }

    extern "C" {
        fn _open_osfhandle(osfhandle: isize, flags: i32) -> i32;
    }

    pub fn redirect() -> Option<SavedStderr> {
        unsafe {
            // Save current stderr file descriptor (2 = stderr)
            let saved_fd = libc::dup(2);
            if saved_fd < 0 {
                return None;
            }

            // Open NUL device
            let nul = OpenOptions::new().write(true).open("NUL").ok()?;
            let nul_handle = nul.as_raw_handle() as isize;

            // Get file descriptor from Windows handle
            let nul_fd = _open_osfhandle(nul_handle, 0);
            if nul_fd < 0 {
                libc::close(saved_fd);
                return None;
            }

            // Redirect stderr (fd 2) to NUL
            if libc::dup2(nul_fd, 2) < 0 {
                libc::close(saved_fd);
                libc::close(nul_fd);
                return None;
            }
            libc::close(nul_fd);

            // Forget the File to prevent closing the handle
            std::mem::forget(nul);

            Some(SavedStderr { saved_fd })
        }
    }

    pub fn restore(saved: SavedStderr) {
        unsafe {
            libc::dup2(saved.saved_fd, 2);
            libc::close(saved.saved_fd);
        }
    }
}

// ===== Linux Dependency Detection =====

#[cfg(target_os = "linux")]
pub fn check_library_exists(lib_name: &str) -> bool {
    use std::process::Command;

    // Method 1: Try ldconfig
    if let Ok(output) = Command::new("ldconfig").args(["-p"]).output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains(lib_name) {
                return true;
            }
        }
    }

    // Method 2: Check common library paths
    let lib_paths = [
        "/lib/x86_64-linux-gnu",
        "/usr/lib/x86_64-linux-gnu",
        "/lib64",
        "/usr/lib64",
        "/lib",
        "/usr/lib",
    ];

    for path in lib_paths {
        let full_path = format!("{}/{}", path, lib_name);
        if std::path::Path::new(&full_path).exists() {
            return true;
        }
    }

    false
}

#[cfg(target_os = "linux")]
pub fn detect_package_manager_command() -> &'static str {
    use std::path::Path;

    // Check /etc/os-release for distro identification
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        let content_lower = content.to_lowercase();

        // Debian/Ubuntu family
        if content_lower.contains("ubuntu")
            || content_lower.contains("debian")
            || content_lower.contains("mint")
            || content_lower.contains("pop")
        {
            return "sudo apt install libgomp1";
        }

        // RHEL family
        if content_lower.contains("fedora")
            || content_lower.contains("rhel")
            || content_lower.contains("centos")
            || content_lower.contains("rocky")
            || content_lower.contains("alma")
            || content_lower.contains("amazon")
        {
            return "sudo dnf install libgomp";
        }

        // Arch family
        if content_lower.contains("arch")
            || content_lower.contains("manjaro")
            || content_lower.contains("endeavour")
        {
            return "sudo pacman -S gcc-libs";
        }

        // openSUSE
        if content_lower.contains("suse") || content_lower.contains("opensuse") {
            return "sudo zypper install libgomp1";
        }

        // Alpine
        if content_lower.contains("alpine") {
            return "sudo apk add libgomp";
        }
    }

    // Fallback: detect by package manager binary
    if Path::new("/usr/bin/apt").exists() || Path::new("/usr/bin/apt-get").exists() {
        return "sudo apt install libgomp1";
    }
    if Path::new("/usr/bin/dnf").exists() {
        return "sudo dnf install libgomp";
    }
    if Path::new("/usr/bin/yum").exists() {
        return "sudo yum install libgomp";
    }
    if Path::new("/usr/bin/pacman").exists() {
        return "sudo pacman -S gcc-libs";
    }
    if Path::new("/usr/bin/zypper").exists() {
        return "sudo zypper install libgomp1";
    }
    if Path::new("/sbin/apk").exists() {
        return "sudo apk add libgomp";
    }

    "Install libgomp using your package manager (e.g., apt install libgomp1)"
}

#[cfg(target_os = "linux")]
#[allow(dead_code)]
pub fn check_linux_dependencies() {
    if !check_library_exists("libgomp.so.1") {
        eprintln!("error: Missing required library: libgomp.so.1");
        eprintln!();
        let install_cmd = detect_package_manager_command();
        eprintln!("Install it with:");
        eprintln!("  {}", install_cmd);
        eprintln!();
        eprintln!("Or rebuild fix from source (OpenMP disabled by default).");
        std::process::exit(1);
    }
}

// ===== Tests =====

#[cfg(test)]
mod tests {
    use super::*;

    // ===== Config Default Tests =====

    #[test]
    fn test_config_default_model() {
        let config = Config::default();
        assert_eq!(config.default_model, "qwen3-correct-0.6B");
    }

    #[test]
    fn test_default_model_constant() {
        assert_eq!(DEFAULT_MODEL, "qwen3-correct-0.6B");
    }

    #[test]
    fn test_hf_repo_constant() {
        assert_eq!(HF_REPO, "animeshkundu/cmd-correct");
    }

    // ===== Shell Detection Tests =====

    #[test]
    fn test_detect_shell_from_shell_env_bash() {
        let original = env::var("SHELL").ok();
        let original_ps = env::var("PSModulePath").ok();

        env::set_var("SHELL", "/bin/bash");
        env::remove_var("PSModulePath");

        let result = detect_shell();
        assert_eq!(result, "bash");

        // Restore
        match original {
            Some(val) => env::set_var("SHELL", val),
            None => env::remove_var("SHELL"),
        }
        match original_ps {
            Some(val) => env::set_var("PSModulePath", val),
            None => env::remove_var("PSModulePath"),
        }
    }

    #[test]
    fn test_detect_shell_from_shell_env_zsh() {
        let original = env::var("SHELL").ok();
        let original_ps = env::var("PSModulePath").ok();

        env::set_var("SHELL", "/usr/bin/zsh");
        env::remove_var("PSModulePath");

        let result = detect_shell();
        assert_eq!(result, "zsh");

        match original {
            Some(val) => env::set_var("SHELL", val),
            None => env::remove_var("SHELL"),
        }
        match original_ps {
            Some(val) => env::set_var("PSModulePath", val),
            None => env::remove_var("PSModulePath"),
        }
    }

    #[test]
    fn test_detect_shell_from_shell_env_fish() {
        let original = env::var("SHELL").ok();
        let original_ps = env::var("PSModulePath").ok();

        env::set_var("SHELL", "/usr/local/bin/fish");
        env::remove_var("PSModulePath");

        let result = detect_shell();
        assert_eq!(result, "fish");

        match original {
            Some(val) => env::set_var("SHELL", val),
            None => env::remove_var("SHELL"),
        }
        match original_ps {
            Some(val) => env::set_var("PSModulePath", val),
            None => env::remove_var("PSModulePath"),
        }
    }

    #[test]
    fn test_detect_shell_powershell_via_psmodulepath() {
        let original_shell = env::var("SHELL").ok();
        let original_ps = env::var("PSModulePath").ok();

        env::remove_var("SHELL");
        env::set_var("PSModulePath", "/some/module/path");

        let result = detect_shell();
        assert_eq!(result, "powershell");

        // Restore
        match original_shell {
            Some(val) => env::set_var("SHELL", val),
            None => env::remove_var("SHELL"),
        }
        match original_ps {
            Some(val) => env::set_var("PSModulePath", val),
            None => env::remove_var("PSModulePath"),
        }
    }

    #[test]
    fn test_detect_shell_fallback() {
        let original_shell = env::var("SHELL").ok();
        let original_ps = env::var("PSModulePath").ok();

        env::remove_var("SHELL");
        env::remove_var("PSModulePath");

        let result = detect_shell();

        // On Unix, should fall back to "bash"; on Windows, "cmd"
        #[cfg(unix)]
        assert_eq!(result, "bash");

        #[cfg(windows)]
        assert_eq!(result, "cmd");

        // Restore
        match original_shell {
            Some(val) => env::set_var("SHELL", val),
            None => env::remove_var("SHELL"),
        }
        match original_ps {
            Some(val) => env::set_var("PSModulePath", val),
            None => env::remove_var("PSModulePath"),
        }
    }

    // ===== Build Prompt Tests =====

    #[test]
    fn test_build_prompt_basic() {
        let prompt = build_prompt("bash", "gti status", None);

        assert!(prompt.contains("<|im_start|>system"));
        assert!(prompt.contains("shell command corrector for bash"));
        assert!(prompt.contains("<|im_start|>user"));
        assert!(prompt.contains("gti status"));
        assert!(prompt.contains("<|im_end|>"));
        assert!(prompt.contains("<|im_start|>assistant"));
    }

    #[test]
    fn test_build_prompt_different_shells() {
        let shells = vec!["bash", "zsh", "fish", "powershell", "cmd", "tcsh"];

        for shell in shells {
            let prompt = build_prompt(shell, "test command", None);
            assert!(
                prompt.contains(&format!("corrector for {}", shell)),
                "Prompt should contain shell name: {}",
                shell
            );
        }
    }

    #[test]
    fn test_build_prompt_special_characters() {
        let prompt = build_prompt("bash", "echo \"hello world\" | grep 'test'", None);

        assert!(prompt.contains("echo \"hello world\" | grep 'test'"));
    }

    #[test]
    fn test_build_prompt_empty_command() {
        let prompt = build_prompt("bash", "", None);

        // Should still produce valid ChatML structure
        assert!(prompt.contains("<|im_start|>system"));
        assert!(prompt.contains("<|im_start|>user"));
        assert!(prompt.contains("<|im_start|>assistant"));
    }

    #[test]
    fn test_build_prompt_multiline_command() {
        let cmd = "echo hello && \\\necho world";
        let prompt = build_prompt("bash", cmd, None);

        assert!(prompt.contains(cmd));
    }

    // ===== Path Function Tests =====

    #[test]
    fn test_config_dir_returns_path() {
        let dir = config_dir();

        // Should end with "fix"
        assert!(dir.ends_with("fix"));

        // Should not be empty
        assert!(!dir.as_os_str().is_empty());
    }

    #[test]
    fn test_config_path_returns_json_file() {
        let path = config_path();

        // Should end with "config.json"
        assert!(path.ends_with("config.json"));

        // Parent should be config_dir()
        assert_eq!(path.parent().unwrap(), config_dir());
    }

    #[test]
    fn test_get_model_path_appends_gguf() {
        let path = get_model_path("test-model");

        assert!(path.ends_with("test-model.gguf"));
        assert_eq!(path.parent().unwrap(), config_dir());
    }

    #[test]
    fn test_get_model_path_preserves_name() {
        let model_names = vec![
            "qwen3-correct-0.6B",
            "llama-7b-q4",
            "model_with_underscore",
            "model-with-dash",
        ];

        for name in model_names {
            let path = get_model_path(name);
            let filename = path.file_name().unwrap().to_str().unwrap();
            assert_eq!(filename, format!("{}.gguf", name));
        }
    }

    // ===== Config Serialization Tests =====

    #[test]
    fn test_config_serialization_roundtrip() {
        let config = Config {
            default_model: "test-model".to_string(),
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(config.default_model, deserialized.default_model);
    }

    #[test]
    fn test_config_deserialize_from_json() {
        let json = r#"{"default_model": "custom-model"}"#;
        let config: Config = serde_json::from_str(json).unwrap();

        assert_eq!(config.default_model, "custom-model");
    }
}
