use clap::Parser;
use indicatif::{ProgressBar, ProgressStyle};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::token::data_array::LlamaTokenDataArray;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

const HF_REPO: &str = "animeshkundu/cmd-correct";
const DEFAULT_MODEL: &str = "qwen3-correct-0.6B";

#[derive(Parser, Debug)]
#[command(name = "cmd-correct")]
#[command(about = "AI-powered shell command corrector", long_about = None)]
struct Args {
    /// The failed command to correct (optional for management commands)
    #[arg(num_args = 0..)]
    command: Vec<String>,

    /// Error message from the failed command (optional)
    #[arg(short, long)]
    error: Option<String>,

    /// Override shell detection (bash, zsh, fish, powershell, cmd, tcsh)
    #[arg(short, long)]
    shell: Option<String>,

    /// Path to a local GGUF model file (overrides default)
    #[arg(short, long)]
    model: Option<PathBuf>,

    /// Number of GPU layers to offload (default: all)
    #[arg(long, default_value = "99")]
    gpu_layers: u32,

    /// Show model loading and inference logs
    #[arg(short, long)]
    verbose: bool,

    /// List available models from HuggingFace
    #[arg(long)]
    list_models: bool,

    /// Download and set a model as default
    #[arg(long)]
    use_model: Option<String>,

    /// Force re-download of current model
    #[arg(long)]
    update: bool,

    /// Show current configuration
    #[arg(long)]
    show_config: bool,
}

#[derive(Serialize, Deserialize)]
struct Config {
    default_model: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_model: DEFAULT_MODEL.to_string(),
        }
    }
}

struct AvailableModel {
    name: String,
    size: u64,
}

// Cross-platform config directory
fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
        .join("cmd-correct")
}

fn config_path() -> PathBuf {
    config_dir().join("config.json")
}

fn load_config() -> Config {
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

fn save_config(config: &Config) -> Result<(), String> {
    let dir = config_dir();
    std::fs::create_dir_all(&dir)
        .map_err(|e| format!("Failed to create config directory: {}", e))?;
    let content = serde_json::to_string_pretty(config).map_err(|e| e.to_string())?;
    std::fs::write(config_path(), content).map_err(|e| format!("Failed to save config: {}", e))
}

fn fetch_available_models() -> Result<Vec<AvailableModel>, String> {
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

fn list_models(config: &Config) -> Result<(), String> {
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

fn validate_model_exists(model_name: &str) -> Result<(), String> {
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

fn download_model(model_name: &str) -> Result<PathBuf, String> {
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

fn get_model_path(model_name: &str) -> PathBuf {
    config_dir().join(format!("{}.gguf", model_name))
}

fn find_or_download_model(model_name: &str, force_download: bool) -> Result<PathBuf, String> {
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

fn find_model_path(
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

fn detect_shell() -> String {
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

fn build_prompt(shell: &str, command: &str, _error: Option<&str>) -> String {
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

fn suppress_llama_logs() {
    unsafe {
        llama_cpp_sys_2::ggml_log_set(None, std::ptr::null_mut());
        llama_cpp_sys_2::llama_log_set(None, std::ptr::null_mut());
    }
}

#[cfg(unix)]
mod stderr_redirect {
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
mod stderr_redirect {
    pub fn redirect() -> Option<()> {
        // On Windows, log callback suppression is sufficient
        None
    }
    pub fn restore(_: ()) {}
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut config = load_config();

    // Handle management commands (no command required)
    if args.list_models {
        list_models(&config)?;
        return Ok(());
    }

    if args.show_config {
        let model_path = get_model_path(&config.default_model);
        println!("Configuration:");
        println!("  Default model: {}", config.default_model);
        println!("  Config path: {}", config_path().display());
        if model_path.exists() {
            println!("  Model path: {}", model_path.display());
        } else {
            println!("  Model path: (not downloaded)");
        }
        return Ok(());
    }

    if let Some(ref model_name) = args.use_model {
        eprintln!("Checking model availability...");
        validate_model_exists(model_name)?;

        // Download the model
        download_model(model_name)?;

        // Update config
        config.default_model = model_name.clone();
        save_config(&config)?;

        eprintln!("✓ Default model set to: {}", model_name);
        return Ok(());
    }

    // For inference, command is required
    if args.command.is_empty() {
        eprintln!("Usage: cmd-correct <command>");
        eprintln!("       cmd-correct --list-models");
        eprintln!("       cmd-correct --use-model <name>");
        eprintln!("       cmd-correct --show-config");
        std::process::exit(1);
    }

    // Join command arguments into single string
    let command = args.command.join(" ");

    // Detect shell
    let shell = args.shell.unwrap_or_else(detect_shell);

    // Find or download model
    let model_path = find_model_path(args.model, &config, args.update)?;

    // Suppress logs (cross-platform)
    if !args.verbose {
        suppress_llama_logs();
    }

    #[cfg(unix)]
    let saved_stderr = if !args.verbose {
        stderr_redirect::redirect()
    } else {
        None
    };

    // Initialize backend
    let backend = LlamaBackend::init()?;

    // Load model with GPU acceleration
    let model_params = LlamaModelParams::default().with_n_gpu_layers(args.gpu_layers);
    let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)
        .map_err(|e| format!("Failed to load model: {}", e))?;

    // Create context
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(std::num::NonZeroU32::new(512))
        .with_n_batch(512);
    let mut ctx = model
        .new_context(&backend, ctx_params)
        .map_err(|e| format!("Failed to create context: {}", e))?;

    // Build and tokenize prompt
    let prompt = build_prompt(&shell, &command, args.error.as_deref());
    let tokens = model
        .str_to_token(&prompt, llama_cpp_2::model::AddBos::Always)
        .map_err(|e| format!("Tokenization failed: {}", e))?;

    // Create batch and add tokens
    let mut batch = LlamaBatch::new(512, 1);
    for (i, token) in tokens.iter().enumerate() {
        let is_last = i == tokens.len() - 1;
        batch.add(*token, i as i32, &[0], is_last)?;
    }

    // Decode prompt
    ctx.decode(&mut batch)
        .map_err(|e| format!("Decode failed: {}", e))?;

    // Generate response
    let mut output = String::new();
    let max_tokens = 128; // Increased to allow for thinking tokens
    let eos_token = model.token_eos();
    let mut cur_pos = tokens.len() as i32;
    let mut in_thinking = false;
    let mut after_thinking = false;
    let mut should_break = false;

    for _ in 0..max_tokens {
        let candidates = ctx.candidates();
        let mut candidates_data = LlamaTokenDataArray::from_iter(candidates, false);

        // Sample token (greedy)
        let new_token = candidates_data.sample_token_greedy();

        // Check for EOS or special tokens
        if new_token == eos_token {
            break;
        }

        // Convert token to string
        if let Ok(piece) = model.token_to_str(new_token, llama_cpp_2::model::Special::Tokenize) {
            // Stop at special tokens
            if piece.contains("<|im_end|>") || piece.contains("<|im_start|>") {
                break;
            }

            // Track thinking state
            if piece.contains("<think>") {
                in_thinking = true;
            } else if piece.contains("</think>") {
                in_thinking = false;
                after_thinking = true;
                // Don't add closing tag to output
            } else if !in_thinking {
                // Skip leading whitespace/newlines after thinking block
                if after_thinking && piece.trim().is_empty() {
                    // Just skip adding to output, but continue with batch update
                } else {
                    after_thinking = false;
                    output.push_str(&piece);

                    // Stop at newline (we only want one line) - but only if we have actual content
                    let trimmed = output.trim();
                    if !trimmed.is_empty() && trimmed.contains('\n') {
                        should_break = true;
                    }
                }
            }
        }

        if should_break {
            break;
        }

        // Prepare next batch with correct position
        batch.clear();
        batch.add(new_token, cur_pos, &[0], true)?;
        cur_pos += 1;
        ctx.decode(&mut batch)
            .map_err(|e| format!("Decode failed: {}", e))?;
    }

    // Clean and print result (to stdout, which is not redirected)
    let result = output.trim();

    // Strip common model artifacts/prefixes
    let result = result
        .strip_prefix("command >")
        .or_else(|| result.strip_prefix("command>"))
        .or_else(|| result.strip_prefix("command 2>&1"))
        .or_else(|| result.strip_prefix("Command:"))
        .unwrap_or(result)
        .trim();

    // Take only the first line (ignore any garbage after newline)
    let result = result.lines().next().unwrap_or(result).trim();

    #[cfg(unix)]
    if let Some(saved) = saved_stderr {
        stderr_redirect::restore(saved);
    }

    if !result.is_empty() {
        println!("{}", result);
        Ok(())
    } else {
        eprintln!("Could not correct command");
        std::process::exit(1);
    }
}
