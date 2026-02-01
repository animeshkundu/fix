//! wit - Smart shell command correction CLI with agentic capabilities
//!
//! A command-line tool that provides intelligent shell command correction
//! using an agentic loop with tool execution. This is the "smart" counterpart
//! to the fast `fix` command.

use clap::Parser;
use fix_lib::{
    cache, config_path, detect_shell, discovery, get_model_path, load_config,
    progress::ProgressSpinner,
};
use std::sync::{Arc, Mutex};

#[derive(Parser, Debug)]
#[command(name = "wit")]
#[command(about = "Smart shell command correction with agentic capabilities", long_about = None)]
struct Args {
    /// The failed command to correct
    #[arg(num_args = 0..)]
    command: Vec<String>,

    /// Override shell detection (bash, zsh, fish, powershell, cmd, tcsh)
    #[arg(short, long)]
    shell: Option<String>,

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
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = load_config();

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
        let model_path = get_model_path(&config.default_model);
        println!("Configuration:");
        println!("  Default model: {}", config.default_model);
        println!("  Config path: {}", config_path().display());
        if model_path.exists() {
            println!("  Model path: {}", model_path.display());
        } else {
            println!("  Model path: (not downloaded)");
        }

        // Show cache info
        let cache_path = cache::cache_path();
        println!("  Cache path: {}", cache_path.display());

        if let Ok(tools_cache) = cache::load_cache() {
            println!("  Cached tools: {}", tools_cache.tools.len());
            if let Ok(age) = tools_cache.age() {
                let hours = age.as_secs() / 3600;
                println!("  Cache age: {} hours", hours);
                if tools_cache.needs_refresh() {
                    println!("  Cache status: stale (needs refresh)");
                } else {
                    println!("  Cache status: fresh");
                }
            }
        } else {
            println!("  Cached tools: (cache not initialized)");
        }

        return Ok(());
    }

    // Load or create cache
    let tools_cache = cache::load_or_create_cache();
    let cache_arc = Arc::new(Mutex::new(tools_cache.clone()));

    // Check if cache needs refresh and spawn background thread if needed
    if tools_cache.needs_refresh() {
        if args.verbose {
            eprintln!("Cache is stale, refreshing in background...");
        }

        // Spawn background refresh (non-blocking)
        let _handle = discovery::refresh_cache_background(cache_arc.clone());

        // Note: We don't wait for the background thread - the main process continues
        // with the stale cache, and the next invocation will use the fresh cache
    }

    // For now, just print a placeholder message
    if args.command.is_empty() {
        eprintln!("Usage: wit <command>");
        eprintln!("       wit --show-config");
        eprintln!("       wit --refresh-tools");
        eprintln!();
        eprintln!("Note: wit is currently a placeholder. Full agentic implementation coming soon.");
        std::process::exit(1);
    }

    let command = args.command.join(" ");
    let shell = args.shell.unwrap_or_else(detect_shell);

    if args.verbose {
        eprintln!("Shell: {}", shell);
        eprintln!("Command: {}", command);

        // Show number of cached tools
        if let Ok(cache) = cache_arc.lock() {
            eprintln!("Cached tools: {}", cache.tools.len());
        }
    }

    // Create progress spinner
    let mut spinner = ProgressSpinner::new(args.quiet);

    // Simulate agentic workflow with progress indicators
    spinner.set_message("Discovering tools...");
    std::thread::sleep(std::time::Duration::from_millis(50));

    spinner.set_message("Checking command...");
    std::thread::sleep(std::time::Duration::from_millis(50));

    spinner.set_message("Analyzing...");
    std::thread::sleep(std::time::Duration::from_millis(50));

    spinner.set_message("Correcting...");
    std::thread::sleep(std::time::Duration::from_millis(50));

    spinner.finish_with_message("✓");

    // Placeholder: In future PRs, this will implement:
    // - Agentic loop with tool execution
    // - Context-aware corrections
    eprintln!("wit: Smart command correction not yet implemented.");
    eprintln!("Received command: {}", command);
    eprintln!();
    eprintln!("For immediate command correction, use 'fix' instead:");
    eprintln!("  fix \"{}\"", command);

    std::process::exit(0)
}
