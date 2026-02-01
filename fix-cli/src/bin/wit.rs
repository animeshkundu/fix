//! wit - Smart shell command correction CLI with tool-assisted inference
//!
//! Uses a daemon mode by default to keep the model loaded for fast inference.
//! The daemon auto-starts on first use and unloads after 1 hour of inactivity.
//!
//! Note: Daemon mode is only available on Unix systems. On Windows, direct mode
//! is always used.

use clap::Parser;
#[cfg(unix)]
use fix_lib::stderr_redirect;
use fix_lib::{
    cache, config_path, detect_shell, discovery, download_model, find_or_download_model,
    get_model_path, load_config, progress::ProgressSpinner, save_config, suppress_llama_logs,
    tools::Shell, tools::Tool, tools::ToolExecutor, validate_model_exists, WIT_DEFAULT_MODEL,
};
use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::LlamaModel;
use llama_cpp_2::token::data_array::LlamaTokenDataArray;
#[cfg(unix)]
use serde::{Deserialize, Serialize};
#[cfg(unix)]
use std::fs;
#[cfg(unix)]
use std::io::{BufRead, BufReader, Write};
#[cfg(unix)]
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::PathBuf;
#[cfg(unix)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(unix)]
use std::sync::{Arc, Mutex};
use std::time::Duration;
#[cfg(unix)]
use std::time::Instant;

/// Idle timeout before daemon auto-shuts down (1 hour)
#[cfg(unix)]
const IDLE_TIMEOUT_SECS: u64 = 3600;

/// Socket path for daemon communication
#[cfg(unix)]
fn socket_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("wit-daemon-{}.sock", users::get_current_uid()));
    path
}

/// PID file path for single instance check
#[cfg(unix)]
fn pid_path() -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("wit-daemon-{}.pid", users::get_current_uid()));
    path
}

#[derive(Parser, Debug)]
#[command(name = "wit")]
#[command(about = "Smart shell command correction with tool-assisted inference", long_about = None)]
struct Args {
    /// The failed command to correct
    #[arg(num_args = 0..)]
    command: Vec<String>,

    /// Override shell detection (bash, zsh, fish, powershell, cmd, tcsh)
    #[arg(short, long)]
    shell: Option<String>,

    /// Path to a local GGUF model file (overrides default)
    #[arg(short, long)]
    model: Option<PathBuf>,

    /// Number of GPU layers to offload (default: all)
    #[arg(long, default_value = "99")]
    gpu_layers: u32,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Disable progress indicators
    #[arg(short, long)]
    quiet: bool,

    /// Show current configuration
    #[arg(long)]
    show_config: bool,

    /// Refresh the tool discovery cache
    #[arg(long)]
    refresh_tools: bool,

    /// Download and set wit model as default
    #[arg(long)]
    use_model: Option<String>,

    /// Stop the daemon and unload model from memory
    #[arg(long)]
    stop: bool,

    /// Show daemon status
    #[arg(long)]
    status: bool,

    /// Run in direct mode (no daemon, load model each time)
    #[arg(long)]
    direct: bool,

    /// Run as daemon (internal use)
    #[arg(long, hide = true)]
    daemon: bool,
}

/// Request sent to daemon
#[cfg(unix)]
#[derive(Serialize, Deserialize, Debug)]
struct DaemonRequest {
    command: String,
    shell: String,
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

    // Read PID and check if process exists
    if let Ok(pid_str) = fs::read_to_string(&pid_file) {
        if let Ok(pid) = pid_str.trim().parse::<i32>() {
            // Check if process is alive
            unsafe {
                if libc::kill(pid, 0) == 0 {
                    // Also verify socket exists
                    return socket_path().exists();
                }
            }
        }
    }

    // Stale PID file, clean up
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

    // Write PID file
    fs::write(pid_path(), child.id().to_string())
        .map_err(|e| format!("Failed to write PID file: {}", e))?;

    // Wait for daemon to be ready (socket created)
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

    // Send stop command via socket
    if let Ok(mut stream) = UnixStream::connect(socket_path()) {
        let request = serde_json::json!({"stop": true});
        let _ = writeln!(stream, "{}", request);
    }

    // Wait for daemon to stop
    for _ in 0..20 {
        std::thread::sleep(Duration::from_millis(100));
        if !is_daemon_running() {
            break;
        }
    }

    // Force cleanup
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
        .set_read_timeout(Some(Duration::from_secs(60)))
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

/// Analyze input command and determine which tools to run
fn select_tools_for_input(input: &str, shell: Shell) -> Vec<Tool> {
    let mut tools = Vec::new();
    let words: Vec<&str> = input.split_whitespace().collect();

    if words.is_empty() {
        return tools;
    }

    let first_word = words[0];

    // Always check if the first word might be a typo of a real command
    tools.push(Tool::ListSimilar {
        prefix: first_word.to_string(),
    });

    // Try to find the binary for common corrections
    if first_word.len() >= 2 {
        tools.push(Tool::WhichBinary {
            command: first_word.to_string(),
        });
    }

    // If the input looks like it might have a known command with typo
    let common_commands = [
        "git", "docker", "npm", "cargo", "python", "pip", "kubectl", "make",
    ];
    for cmd in common_commands {
        if levenshtein_distance(first_word, cmd) <= 2 && first_word != cmd {
            tools.push(Tool::WhichBinary {
                command: cmd.to_string(),
            });
            tools.push(Tool::HelpOutput {
                command: cmd.to_string(),
            });
            break;
        }
    }

    // For Windows shells, add PowerShell-specific checks
    if shell.is_windows_native()
        && (first_word.starts_with("Get-") || first_word.starts_with("Set-"))
    {
        tools.push(Tool::HelpOutput {
            command: first_word.to_string(),
        });
    }

    tools
}

/// Simple Levenshtein distance for typo detection
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0usize; b_len + 1]; a_len + 1];

    for (i, row) in matrix.iter_mut().enumerate().take(a_len + 1) {
        row[0] = i;
    }
    #[allow(clippy::needless_range_loop)]
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };
            matrix[i][j] = std::cmp::min(
                std::cmp::min(matrix[i - 1][j] + 1, matrix[i][j - 1] + 1),
                matrix[i - 1][j - 1] + cost,
            );
        }
    }

    matrix[a_len][b_len]
}

/// Build wit prompt with tool results in the training format
fn build_wit_prompt(shell: &str, input: &str, tool_results: &[(String, String)]) -> String {
    let mut prompt = String::new();

    prompt.push_str("<|im_start|>system\n");
    prompt.push_str(&format!(
        "You are a shell command assistant for {}. Use the provided tool results to generate the correct command. /no_think",
        shell
    ));
    prompt.push_str("<|im_end|>\n");

    prompt.push_str("<|im_start|>user\n");
    prompt.push_str(&format!("Input: {}\n\n", input));

    if !tool_results.is_empty() {
        prompt.push_str("Tool results:\n");
        for (tool_call, result) in tool_results {
            prompt.push_str(&format!("- {}: {}\n", tool_call, result));
        }
    }
    prompt.push_str("<|im_end|>\n");

    prompt.push_str("<|im_start|>assistant\n");

    prompt
}

/// Format tool call for display
fn format_tool_call(tool: &Tool) -> String {
    match tool {
        Tool::WhichBinary { command } => format!("which_binary({})", command),
        Tool::ListSimilar { prefix } => format!("list_similar({})", prefix),
        Tool::HelpOutput { command } => format!("help_output({})", command),
        Tool::GetEnvVar { name } => format!("get_env_var({})", name),
        Tool::ManPage { command } => format!("man_page({})", command),
    }
}

/// Run inference with loaded model
fn run_inference(
    model: &LlamaModel,
    backend: &LlamaBackend,
    command: &str,
    shell_str: &str,
    verbose: bool,
) -> Result<String, String> {
    let shell = Shell::parse(shell_str).unwrap_or(Shell::Bash);

    // Execute tools in parallel
    let tools_to_run = select_tools_for_input(command, shell);

    // Parallel tool execution using thread::scope
    let tool_results: Vec<(String, String)> = std::thread::scope(|s| {
        // Spawn a thread for each tool
        let handles: Vec<_> = tools_to_run
            .iter()
            .map(|tool| {
                s.spawn(move || {
                    let executor = ToolExecutor::new(shell);
                    let result = executor.execute(tool);
                    if result.success && !result.output.is_empty() {
                        let tool_call = format_tool_call(tool);
                        let output = if result.output.len() > 200 {
                            format!("{}...", &result.output[..200])
                        } else {
                            result.output
                        };
                        Some((tool_call, output))
                    } else {
                        None
                    }
                })
            })
            .collect();

        // Collect results, filtering out None values
        handles
            .into_iter()
            .filter_map(|h| h.join().ok().flatten())
            .collect()
    });

    if verbose {
        eprintln!("Tool results (parallel): {:?}", tool_results);
    }

    // Build prompt
    let prompt = build_wit_prompt(shell_str, command, &tool_results);

    // Create context
    let ctx_params = LlamaContextParams::default()
        .with_n_ctx(std::num::NonZeroU32::new(1024))
        .with_n_batch(512);
    let mut ctx = model
        .new_context(backend, ctx_params)
        .map_err(|e| format!("Failed to create context: {}", e))?;

    // Tokenize
    let tokens = model
        .str_to_token(&prompt, llama_cpp_2::model::AddBos::Always)
        .map_err(|e| format!("Tokenization failed: {}", e))?;

    // Create batch
    let mut batch = LlamaBatch::new(1024, 1);
    for (i, token) in tokens.iter().enumerate() {
        let is_last = i == tokens.len() - 1;
        batch
            .add(*token, i as i32, &[0], is_last)
            .map_err(|e| format!("Batch add failed: {}", e))?;
    }

    // Decode prompt
    ctx.decode(&mut batch)
        .map_err(|e| format!("Decode failed: {}", e))?;

    // Generate
    let mut output = String::new();
    let max_tokens = 256;
    let eos_token = model.token_eos();
    let mut cur_pos = tokens.len() as i32;
    let mut in_thinking = false;
    let mut after_thinking = false;

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

                    if !output.trim().is_empty() && output.trim().lines().count() > 10 {
                        break;
                    }
                }
            }
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
        .strip_prefix("|")
        .or_else(|| result.strip_prefix("| "))
        .unwrap_or(result)
        .trim();

    Ok(result.to_string())
}

/// Run daemon mode
#[cfg(unix)]
fn run_daemon(model_path: PathBuf, gpu_layers: u32) -> Result<(), Box<dyn std::error::Error>> {
    // Remove stale socket
    let _ = fs::remove_file(socket_path());

    // Suppress logs
    suppress_llama_logs();

    // Initialize backend and load model
    let backend = LlamaBackend::init()?;
    let model_params = LlamaModelParams::default().with_n_gpu_layers(gpu_layers);
    let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)
        .map_err(|e| format!("Failed to load model: {}", e))?;

    // Create socket
    let listener = UnixListener::bind(socket_path())?;
    listener.set_nonblocking(true)?;

    let last_activity = Arc::new(Mutex::new(Instant::now()));
    let should_stop = Arc::new(AtomicBool::new(false));

    // Main loop
    loop {
        // Check idle timeout
        {
            let last = last_activity.lock().unwrap();
            if last.elapsed() > Duration::from_secs(IDLE_TIMEOUT_SECS) {
                eprintln!("wit daemon: idle timeout, shutting down");
                break;
            }
        }

        // Check stop flag
        if should_stop.load(Ordering::Relaxed) {
            break;
        }

        // Accept connection (non-blocking)
        match listener.accept() {
            Ok((mut stream, _)) => {
                // Update activity
                *last_activity.lock().unwrap() = Instant::now();

                // Read request
                let mut reader = BufReader::new(&stream);
                let mut line = String::new();
                if reader.read_line(&mut line).is_err() {
                    continue;
                }

                // Check for stop command
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

                // Parse request
                let request: Result<DaemonRequest, _> = serde_json::from_str(&line);
                let response = match request {
                    Ok(req) => {
                        match run_inference(&model, &backend, &req.command, &req.shell, req.verbose)
                        {
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
                // No connection, sleep briefly
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(_) => {
                std::thread::sleep(Duration::from_millis(100));
            }
        }
    }

    // Cleanup
    let _ = fs::remove_file(socket_path());
    let _ = fs::remove_file(pid_path());

    Ok(())
}

/// Run in direct mode (no daemon)
fn run_direct(
    command: &str,
    shell_str: &str,
    model_path: PathBuf,
    gpu_layers: u32,
    verbose: bool,
    quiet: bool,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut spinner = ProgressSpinner::new(quiet);

    if !quiet {
        suppress_llama_logs();
    }

    #[cfg(unix)]
    let saved_stderr = if !verbose {
        stderr_redirect::redirect()
    } else {
        None
    };

    spinner.set_message("Loading model...");

    let backend = LlamaBackend::init()?;
    let model_params = LlamaModelParams::default().with_n_gpu_layers(gpu_layers);
    let model = LlamaModel::load_from_file(&backend, &model_path, &model_params)
        .map_err(|e| format!("Failed to load model: {}", e))?;

    spinner.set_message("Generating correction...");
    let result = run_inference(&model, &backend, command, shell_str, verbose)?;

    spinner.finish_with_message("✓");

    #[cfg(unix)]
    if let Some(saved) = saved_stderr {
        stderr_redirect::restore(saved);
    }

    Ok(result)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let mut config = load_config();

    // Handle daemon mode (internal) - Unix only
    #[cfg(unix)]
    if args.daemon {
        let model_path = args
            .model
            .unwrap_or_else(|| get_model_path(WIT_DEFAULT_MODEL));
        return run_daemon(model_path, args.gpu_layers);
    }

    // Handle --stop flag - Unix only (daemon mode)
    #[cfg(unix)]
    if args.stop {
        stop_daemon()?;
        eprintln!("✓ Daemon stopped, model unloaded");
        return Ok(());
    }

    #[cfg(not(unix))]
    if args.stop {
        eprintln!("Daemon mode not available on Windows");
        return Ok(());
    }

    // Handle --status flag - Unix only (daemon mode)
    #[cfg(unix)]
    if args.status {
        if is_daemon_running() {
            println!("Daemon: running");
            println!("Socket: {}", socket_path().display());
            println!("PID file: {}", pid_path().display());
        } else {
            println!("Daemon: not running");
        }
        return Ok(());
    }

    #[cfg(not(unix))]
    if args.status {
        println!("Daemon: not available on Windows (direct mode only)");
        return Ok(());
    }

    // Handle --refresh-tools flag
    if args.refresh_tools {
        eprintln!("Refreshing tool discovery cache...");
        let new_cache = discovery::discover_tools();
        cache::save_cache(&new_cache)?;
        eprintln!("✓ Cache refreshed successfully");
        eprintln!("  Discovered {} tools", new_cache.tools.len());
        return Ok(());
    }

    if args.show_config {
        let model_path = get_model_path(WIT_DEFAULT_MODEL);
        println!("Configuration:");
        println!("  Wit model: {}", WIT_DEFAULT_MODEL);
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
            println!("  Daemon: not available on Windows (direct mode only)");
        }

        let cache_path = cache::cache_path();
        println!("  Cache path: {}", cache_path.display());

        if let Ok(tools_cache) = cache::load_cache() {
            println!("  Cached tools: {}", tools_cache.tools.len());
        }

        return Ok(());
    }

    // Handle --use-model flag
    if let Some(ref model_name) = args.use_model {
        eprintln!("Checking model availability...");
        validate_model_exists(model_name)?;
        download_model(model_name)?;
        config.default_model = model_name.clone();
        save_config(&config)?;
        eprintln!("✓ Default model set to: {}", model_name);

        // Stop daemon so it picks up new model (Unix only)
        #[cfg(unix)]
        if is_daemon_running() {
            stop_daemon()?;
            eprintln!("✓ Daemon restarted to use new model");
        }
        return Ok(());
    }

    // For inference, command is required
    if args.command.is_empty() {
        eprintln!("Usage: wit <command>");
        eprintln!("       wit --show-config");
        eprintln!("       wit --refresh-tools");
        eprintln!("       wit --stop          # Unload model from memory");
        eprintln!("       wit --status        # Show daemon status");
        eprintln!("       wit --direct <cmd>  # Run without daemon");
        std::process::exit(1);
    }

    let command = args.command.join(" ");
    let shell_str = args.shell.unwrap_or_else(detect_shell);

    if args.verbose {
        eprintln!("Shell: {}", shell_str);
        eprintln!("Command: {}", command);
    }

    // Find or download model
    let model_path = if let Some(ref path) = args.model {
        path.clone()
    } else {
        find_or_download_model(WIT_DEFAULT_MODEL, false)?
    };

    // On Windows, always use direct mode. On Unix, use direct mode if --direct flag is set.
    #[cfg(not(unix))]
    let use_direct = true;
    #[cfg(unix)]
    let use_direct = args.direct;

    // Direct mode - no daemon
    if use_direct {
        let result = run_direct(
            &command,
            &shell_str,
            model_path,
            args.gpu_layers,
            args.verbose,
            args.quiet,
        )?;

        if !result.is_empty() {
            println!("{}", result);
        } else {
            eprintln!("Could not correct command");
            std::process::exit(1);
        }
        return Ok(());
    }

    // Daemon mode (default on Unix)
    #[cfg(unix)]
    {
        let mut spinner = ProgressSpinner::new(args.quiet);

        // Ensure daemon is running
        if !is_daemon_running() {
            spinner.set_message("Starting daemon...");
            start_daemon(&model_path, args.gpu_layers)?;
        }

        spinner.set_message("Correcting...");

        // Send request to daemon
        let request = DaemonRequest {
            command: command.clone(),
            shell: shell_str,
            verbose: args.verbose,
        };

        let response = send_to_daemon(&request)?;

        spinner.finish_with_message("✓");

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
