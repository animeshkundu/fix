//! wit - Smart shell command correction CLI with agentic capabilities
//!
//! A command-line tool that provides intelligent shell command correction
//! using an agentic loop with tool execution. This is the "smart" counterpart
//! to the fast `fix` command.
//!
//! Full implementation will be added in subsequent PRs.

use clap::Parser;
use fix_lib::{config_path, detect_shell, get_model_path, load_config};

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

    /// Show current configuration
    #[arg(long)]
    show_config: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let config = load_config();

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

    // For now, just print a placeholder message
    if args.command.is_empty() {
        eprintln!("Usage: wit <command>");
        eprintln!("       wit --show-config");
        eprintln!();
        eprintln!("Note: wit is currently a placeholder. Full agentic implementation coming soon.");
        std::process::exit(1);
    }

    let command = args.command.join(" ");
    let shell = args.shell.unwrap_or_else(detect_shell);

    if args.verbose {
        eprintln!("Shell: {}", shell);
        eprintln!("Command: {}", command);
    }

    // Placeholder: In future PRs, this will implement:
    // - Agentic loop with tool execution
    // - Progress indicators
    // - Context-aware corrections
    eprintln!("wit: Smart command correction not yet implemented.");
    eprintln!("Received command: {}", command);
    eprintln!();
    eprintln!("For immediate command correction, use 'fix' instead:");
    eprintln!("  fix \"{}\"", command);

    std::process::exit(0)
}
