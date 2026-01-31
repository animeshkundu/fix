use clap::Parser;
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::token::data_array::LlamaTokenDataArray;
use std::env;
use std::fs::File;
use std::os::unix::io::AsRawFd;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "cmd-correct")]
#[command(about = "AI-powered shell command corrector", long_about = None)]
#[command(trailing_var_arg = true)]
struct Args {
    /// The failed command to correct (all arguments after flags are joined)
    #[arg(required = true, num_args = 1..)]
    command: Vec<String>,

    /// Error message from the failed command (optional)
    #[arg(short, long)]
    error: Option<String>,

    /// Override shell detection (bash, zsh, fish, powershell, cmd, tcsh)
    #[arg(short, long)]
    shell: Option<String>,

    /// Path to the GGUF model file
    #[arg(short, long)]
    model: Option<PathBuf>,

    /// Number of GPU layers to offload (default: all)
    #[arg(long, default_value = "99")]
    gpu_layers: u32,

    /// Show model loading and inference logs
    #[arg(short, long)]
    verbose: bool,
}

fn detect_shell() -> String {
    // Try SHELL env var first
    if let Ok(shell_path) = env::var("SHELL") {
        let shell_name = shell_path.rsplit('/').next().unwrap_or("bash");
        return shell_name.to_string();
    }

    // Windows: check COMSPEC or PSModulePath
    if cfg!(windows) {
        if env::var("PSModulePath").is_ok() {
            return "powershell".to_string();
        }
        return "cmd".to_string();
    }

    "bash".to_string()
}

fn find_model_path(override_path: Option<PathBuf>) -> Result<PathBuf, String> {
    if let Some(path) = override_path {
        if path.exists() {
            return Ok(path);
        }
        return Err(format!("Model not found at: {}", path.display()));
    }

    // Check common locations
    let candidates = vec![
        // Current directory
        PathBuf::from("cmd-correct-v1-q4km.gguf"),
        // Next to executable
        env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("cmd-correct-v1-q4km.gguf")))
            .unwrap_or_default(),
        // Home config directory
        dirs::config_dir()
            .map(|d| d.join("cmd-correct").join("cmd-correct-v1-q4km.gguf"))
            .unwrap_or_default(),
        // Data directory
        dirs::data_dir()
            .map(|d| d.join("cmd-correct").join("cmd-correct-v1-q4km.gguf"))
            .unwrap_or_default(),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err("Model not found. Specify path with --model or place cmd-correct-v1-q4km.gguf in:\n  \
         - Current directory\n  \
         - Next to the executable\n  \
         - ~/.config/cmd-correct/\n  \
         - ~/.local/share/cmd-correct/".to_string())
}

fn build_prompt(shell: &str, command: &str, error: Option<&str>) -> String {
    let error_line = error
        .map(|e| format!("Error: {}\n", e))
        .unwrap_or_default();

    format!(
        "<|im_start|>system\n\
         You are a shell command corrector. Output only the corrected command./no_think\n\
         <|im_end|>\n\
         <|im_start|>user\n\
         Shell: {}\n\
         Command: {}\n\
         {}<|im_end|>\n\
         <|im_start|>assistant\n",
        shell, command, error_line
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Join command arguments into single string
    let command = args.command.join(" ");

    // Detect shell
    let shell = args.shell.unwrap_or_else(detect_shell);

    // Find model
    let model_path = find_model_path(args.model)?;

    // By default, redirect stderr to /dev/null to suppress llama.cpp logs
    // Use --verbose to see the logs
    let saved_stderr = if !args.verbose {
        unsafe {
            // Set the llama.cpp log callbacks to no-op
            llama_cpp_sys_2::ggml_log_set(None, std::ptr::null_mut());
            llama_cpp_sys_2::llama_log_set(None, std::ptr::null_mut());

            // Save stderr and redirect to /dev/null
            let saved = libc::dup(libc::STDERR_FILENO);
            let devnull = File::open("/dev/null").expect("Failed to open /dev/null");
            libc::dup2(devnull.as_raw_fd(), libc::STDERR_FILENO);
            Some(saved)
        }
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
    if !result.is_empty() {
        println!("{}", result);
        Ok(())
    } else {
        // If model didn't produce output, restore stderr for error message
        if let Some(saved) = saved_stderr {
            unsafe {
                libc::dup2(saved, libc::STDERR_FILENO);
                libc::close(saved);
            }
        }
        eprintln!("Could not correct command");
        std::process::exit(1);
    }
}
