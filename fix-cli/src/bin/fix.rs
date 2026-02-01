//! fix - Fast shell command correction CLI
//!
//! A command-line tool that corrects shell command typos using a local LLM.
//! Uses daemon mode by default on Unix to keep the model loaded for fast inference.
//! Example: `fix "gti status"` → `git status`

use clap::Parser;
use fix_lib::{
    build_prompt, config_path, detect_shell, download_model, find_model_path, get_model_path,
    list_models, load_config, save_config, suppress_llama_logs, validate_model_exists,
};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::token::data_array::LlamaTokenDataArray;
use std::path::PathBuf;

// Unix-specific imports for daemon mode
#[cfg(unix)]
use fix_lib::stderr_redirect;
#[cfg(unix)]
use serde::{Deserialize, Serialize};
#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::io::{BufRead, BufReader, Write};
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};
#[cfg(unix)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(unix)]
use std::sync::{Arc, Mutex};
#[cfg(unix)]
use std::time::{Duration, Instant};

/// Idle timeout before daemon auto-shuts down (1 hour)
#[cfg(unix)]
const IDLE_TIMEOUT_SECS: u64 = 3600;

/// Socket path for daemon communication
#[cfg(unix)]
fn socket_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("fix-daemon-{}.sock", users::get_current_uid()));
    path
}

/// PID file path for single instance check
#[cfg(unix)]
fn pid_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("fix-daemon-{}.pid", users::get_current_uid()));
    path
}

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

    /// Stop the daemon and unload model from memory (Unix only)
    #[arg(long)]
    stop: bool,

    /// Show daemon status (Unix only)
    #[arg(long)]
    status: bool,

    /// Run in direct mode (no daemon, load model each time)
    #[arg(long)]
    direct: bool,

    /// Run as daemon (internal use, Unix only)
    #[arg(long, hide = true)]
    daemon: bool,
}

/// Request sent to daemon
#[cfg(unix)]
#[derive(Serialize, Deserialize, Debug)]
struct DaemonRequest {
    command: String,
    shell: String,
    error: Option<String>,
    verbose: bool,
}

/// Response from daemon
#[cfg(unix)]
#[derive(Serialize, Deserialize, Debug)]
struct DaemonResponse {
    success: bool,
    output: String,
    error: Option<String>,
}

/// Check if daemon is running
#[cfg(unix)]
fn is_daemon_running() -> bool {
    let pid_file = pid_path();
    if !pid_file.exists() {
        return false;
    }

    if let Ok(pid_str) = fs::read_to_string(&pid_file) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            unsafe {
                if libc::kill(pid, 0) == 0 {
                    return socket_path().exists();
                }
            }
        }
    }

    let _ = fs::remove_file(&pid_file);
    let _ = fs::remove_file(socket_path());
    false
}

/// Start daemon in background
#[cfg(unix)]
fn start_daemon(model_path: &PathBuf, gpu_layers: u32) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("Failed to get executable: {}", e))?;

    let child = std::process::Command::new(&exe)
        .arg("--daemon")
        .arg("--model")
        .arg(model_path)
        .arg("--gpu-layers")
        .arg(gpu_layers.to_string())
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .map_err(|e| format!("Failed to start daemon: {}", e))?;

    fs::write(pid_path(), child.id().to_string())
        .map_err(|e| format!("Failed to write PID file: {}", e))?;

    for _ in 0..50 {
        std::thread::sleep(Duration::from_millis(100));
        if socket_path().exists() {
            return Ok(());
        }
    }

    Err("Daemon failed to start within timeout".to_string())
}

/// Stop the daemon
#[cfg(unix)]
fn stop_daemon() -> Result<(), String> {
    if !is_daemon_running() {
        return Ok(());
    }

    if let Ok(mut stream) = UnixStream::connect(socket_path()) {
        let request = serde_json::json!({"stop": true});
        let _ = writeln!(stream, "{}", request);
    }

    for _ in 0..20 {
        std::thread::sleep(Duration::from_millis(100));
        if !is_daemon_running() {
            break;
        }
    }

    let _ = fs::remove_file(pid_path());
    let _ = fs::remove_file(socket_path());

    Ok(())
}

/// Send request to daemon
#[cfg(unix)]
fn send_to_daemon(request: &DaemonRequest) -> Result<DaemonResponse, String> {
    let mut stream =
        UnixStream::connect(socket_path()).map_err(|e| format!("Failed to connect: {}", e))?;

    stream
        .set_read_timeout(Some(Duration::from_secs(30)))
        .map_err(|e| format!("Failed to set timeout: {}", e))?;

    let request_json =
        serde_json::to_string(request).map_err(|e| format!("Failed to serialize: {}", e))?;

    writeln!(stream, "{}", request_json).map_err(|e| format!("Failed to send: {}", e))?;

    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .map_err(|e| format!("Failed to read response: {}", e))?;

    serde_json::from_str(&response_line).map_err(|e| format!("Failed to parse response: {}", e))
}

/// Run inference with loaded model
fn run_inference(
    model: &LlamaModel,
    backend: &LlamaBackend,
    command: &str,
    shell: &str,
    error: Option<&str>,
    verbose: bool,
) -> Result<String, String> {
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(std::num::NonZeroU32::new(512))
        .with_n_batch(512);
    let mut ctx = model
        .new_context(backend, ctx_params)
        .map_err(|e| format!("Failed to create context: {}", e))?;

    let prompt = build_prompt(shell, command, error);

    if verbose {
        eprintln!("Prompt length: {} chars", prompt.len());
    }

    let tokens = model
        .str_to_token(&prompt, llama_cpp_2::model::AddBos::Always)
        .map_err(|e| format!("Tokenization failed: {}", e))?;

    let mut batch = LlamaBatch::new(512, 1);
    for (i, token) in tokens.iter().enumerate() {
        let is_last = i == tokens.len() - 1;
        batch
            .add(*token, i as i32, &[0], is_last)
            .map_err(|e| format!("Batch add failed: {}", e))?;
    }

    ctx.decode(&mut batch)
        .map_err(|e| format!("Decode failed: {}", e))?;

    let mut output = String::new();
    let max_tokens = 128;
    let eos_token = model.token_eos();
    let mut cur_pos = tokens.len() as i32;
    let mut in_thinking = false;
    let mut after_thinking = false;
    let mut should_break = false;

    for _ in 0..max_tokens {
        let candidates = ctx.candidates();
        let mut candidates_data = LlamaTokenDataArray::from_iter(candidates, false);
        let new_token = candidates_data.sample_token_greedy();

        if new_token == eos_token {
            break;
        }

        if let Ok(piece) = model.token_to_str(new_token, llama_cpp_2::model::Special::Tokenize) {
            if piece.contains("<|im_end|>") || piece.contains("<|im_start|>") {
                break;
            }

            if piece.contains("<think>") {
                in_thinking = true;
            } else if piece.contains("</think>") {
                in_thinking = false;
                after_thinking = true;
            } else if !in_thinking {
                if after_thinking && piece.trim().is_empty() {
                    // Skip
                } else {
                    after_thinking = false;
                    output.push_str(&piece);

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

        batch.clear();
        batch
            .add(new_token, cur_pos, &[0], true)
            .map_err(|e| format!("Batch add failed: {}", e))?;
        cur_pos += 1;
        ctx.decode(&mut batch)
            .map_err(|e| format!("Decode failed: {}", e))?;
    }

    // Clean output
    let result = output.trim();
    let result = result
        .strip_prefix("command >")
        .or_else(|| result.strip_prefix("command>"))
        .or_else(|| result.strip_prefix("command 2>&1"))
        .or_else(|| result.strip_prefix("Command:"))
        .unwrap_or(result)
        .trim();

    let result = result.lines().next().unwrap_or(result).trim();

    Ok(result.to_string())
}

/// Run daemon mode (Unix only)
#[cfg(unix)]
fn run_daemon(model_path: PathBuf, gpu_layers: u32) -> Result<(), Box<dyn std::error::Error>> {
    let _ = fs::remove_file(socket_path());

    suppress_llama_logs();

    let backend = LlamaBackend::init()?;
    let model_params = LlamaModelParams::default().with_n_gpu_layers(gpu_layers);
    let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)
        .map_err(|e| format!("Failed to load model: {}", e))?;

    let listener = UnixListener::bind(socket_path())?;
    listener.set_nonblocking(true)?;

    let last_activity = Arc::new(Mutex::new(Instant::now()));
    let should_stop = Arc::new(AtomicBool::new(false));

    loop {
        {
            let last = last_activity.lock().unwrap();
            if last.elapsed() > Duration::from_secs(IDLE_TIMEOUT_SECS) {
                break;
            }
        }

        if should_stop.load(Ordering::Relaxed) {
            break;
        }

        match listener.accept() {
            Ok((mut stream, _)) => {
                *last_activity.lock().unwrap() = Instant::now();

                let mut reader = BufReader::new(&stream);
                let mut line = String::new();
                if reader.read_line(&mut line).is_err() {
                    continue;
                }

                if line.contains("\"stop\"") {
                    should_stop.store(true, Ordering::Relaxed);
                    let response = DaemonResponse {
                        success: true,
                        output: "Daemon stopping".to_string(),
                        error: None,
                    };
                    let _ = writeln!(stream, "{}", serde_json::to_string(&response).unwrap());
                    break;
                }

                let request: Result<DaemonRequest, _> = serde_json::from_str(&line);
                let response = match request {
                    Ok(req) => {
                        match run_inference(
                            &model,
                            &backend,
                            &req.command,
                            &req.shell,
                            req.error.as_deref(),
                            req.verbose,
                        ) {
                            Ok(output) => DaemonResponse {
                                success: true,
                                output,
                                error: None,
                            },
                            Err(e) => DaemonResponse {
                                success: false,
                                output: String::new(),
                                error: Some(e),
                            },
                        }
                    }
                    Err(e) => DaemonResponse {
                        success: false,
                        output: String::new(),
                        error: Some(format!("Invalid request: {}", e)),
                    },
                };

                let _ = writeln!(stream, "{}", serde_json::to_string(&response).unwrap());
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => {
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }

    let _ = fs::remove_file(socket_path());
    let _ = fs::remove_file(pid_path());

    Ok(())
}

/// Run in direct mode (no daemon)
fn run_direct(
    command: &str,
    shell: &str,
    error: Option<&str>,
    model_path: PathBuf,
    gpu_layers: u32,
    verbose: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    if !verbose {
        suppress_llama_logs();
    }

    #[cfg(unix)]
    let saved_stderr = if !verbose {
        stderr_redirect::redirect()
    } else {
        None
    };

    let backend = LlamaBackend::init()?;
    let model_params = LlamaModelParams::default().with_n_gpu_layers(gpu_layers);
    let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)
        .map_err(|e| format!("Failed to load model: {}", e))?;

    let result = run_inference(&model, &backend, command, shell, error, verbose)?;

    #[cfg(unix)]
    if let Some(saved) = saved_stderr {
        stderr_redirect::restore(saved);
    }

    Ok(result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut config = load_config();

    // Handle daemon mode (internal, Unix only)
    #[cfg(unix)]
    if args.daemon {
        let model_path = args
            .model
            .unwrap_or_else(|| get_model_path(&config.default_model));
        return run_daemon(model_path, args.gpu_layers);
    }

    // Handle --stop flag (Unix only)
    if args.stop {
        #[cfg(unix)]
        {
            stop_daemon()?;
            eprintln!("✓ Daemon stopped, model unloaded");
        }
        #[cfg(not(unix))]
        {
            eprintln!("Daemon mode is not supported on Windows");
        }
        return Ok(());
    }

    // Handle --status flag (Unix only)
    if args.status {
        #[cfg(unix)]
        {
            if is_daemon_running() {
                println!("Daemon: running");
                println!("Socket: {}", socket_path().display());
                println!("PID file: {}", pid_path().display());
            } else {
                println!("Daemon: not running");
            }
        }
        #[cfg(not(unix))]
        {
            println!("Daemon mode is not supported on Windows");
        }
        return Ok(());
    }

    // Handle management commands
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
        #[cfg(unix)]
        {
            println!("  Daemon running: {}", is_daemon_running());
            println!("  Socket: {}", socket_path().display());
        }
        #[cfg(not(unix))]
        {
            println!("  Daemon: not available (Windows)");
        }
        return Ok(());
    }

    if let Some(ref model_name) = args.use_model {
        eprintln!("Checking model availability...");
        validate_model_exists(model_name)?;
        download_model(model_name)?;
        config.default_model = model_name.clone();
        save_config(&config)?;
        eprintln!("✓ Default model set to: {}", model_name);

        #[cfg(unix)]
        if is_daemon_running() {
            stop_daemon()?;
            eprintln!("✓ Daemon restarted to use new model");
        }
        return Ok(());
    }

    // For inference, command is required
    if args.command.is_empty() {
        eprintln!("Usage: fix <command>");
        eprintln!("       fix --list-models");
        eprintln!("       fix --use-model <name>");
        eprintln!("       fix --show-config");
        #[cfg(unix)]
        {
            eprintln!("       fix --stop          # Unload model from memory");
            eprintln!("       fix --status        # Show daemon status");
            eprintln!("       fix --direct <cmd>  # Run without daemon");
        }
        std::process::exit(1);
    }

    let command = args.command.join(" ");
    let shell = args.shell.unwrap_or_else(detect_shell);

    if args.verbose {
        eprintln!("Shell: {}", shell);
        eprintln!("Command: {}", command);
    }

    // Find or download model
    let model_path = find_model_path(args.model, &config, args.update)?;

    // Direct mode (always on Windows, or when explicitly requested)
    #[cfg(not(unix))]
    let use_direct = true;
    #[cfg(unix)]
    let use_direct = args.direct;

    if use_direct {
        let result = run_direct(
            &command,
            &shell,
            args.error.as_deref(),
            model_path,
            args.gpu_layers,
            args.verbose,
        )?;

        if !result.is_empty() {
            println!("{}", result);
        } else {
            eprintln!("Could not correct command");
            std::process::exit(1);
        }
        return Ok(());
    }

    // Daemon mode (Unix only, default)
    #[cfg(unix)]
    {
        if !is_daemon_running() {
            start_daemon(&model_path, args.gpu_layers)?;
        }

        let request = DaemonRequest {
            command: command.clone(),
            shell,
            error: args.error,
            verbose: args.verbose,
        };

        let response = send_to_daemon(&request)?;

        if response.success {
            if !response.output.is_empty() {
                println!("{}", response.output);
            } else {
                eprintln!("Could not correct command");
                std::process::exit(1);
            }
        } else {
            eprintln!(
                "Error: {}",
                response
                    .error
                    .unwrap_or_else(|| "Unknown error".to_string())
            );
            std::process::exit(1);
        }
    }

    Ok(())
}
