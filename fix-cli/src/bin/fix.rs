//! fix - Fast shell command correction CLI
//!
//! A command-line tool that corrects shell command typos using a local LLM.
//! Example: `fix "gti status"` → `git status`

use clap::Parser;
use fix_lib::{
    build_prompt, config_path, detect_shell, download_model, find_model_path, get_model_path,
    list_models, load_config, save_config, stderr_redirect, suppress_llama_logs,
    validate_model_exists,
};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::token::data_array::LlamaTokenDataArray;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "fix")]
#[command(about = "Fix shell command typos using a local LLM", long_about = None)]
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
        eprintln!("Usage: fix <command>");
        eprintln!("       fix --list-models");
        eprintln!("       fix --use-model <name>");
        eprintln!("       fix --show-config");
        std::process::exit(1);
    }

    // Join command arguments into single string
    let command = args.command.join(" ");

    // Detect shell
    let shell = args.shell.unwrap_or_else(detect_shell);

    if args.verbose {
        eprintln!("Shell: {}", shell);
    }

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

    #[cfg(windows)]
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

    if args.verbose {
        eprintln!("Prompt length: {} chars", prompt.len());
    }

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

    #[cfg(windows)]
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
