# fix

[![CI](https://github.com/animeshkundu/fix/actions/workflows/ci.yml/badge.svg)](https://github.com/animeshkundu/fix/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/animeshkundu/fix)](https://github.com/animeshkundu/fix/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Downloads](https://img.shields.io/github/downloads/animeshkundu/fix/total)](https://github.com/animeshkundu/fix/releases)

**Fix shell command typos instantly using a local LLM.**

No API keys. No internet required. Sub-100ms on Apple Silicon.

**[Website](https://animeshkundu.github.io/fix/)** · **[Releases](https://github.com/animeshkundu/fix/releases)** · **[Model](https://huggingface.co/animeshkundu/cmd-correct)**

## Quick Install

**macOS / Linux:**

```bash
curl -fsSL https://animeshkundu.github.io/fix/install.sh | sh
```

**Windows (PowerShell):**

```powershell
iwr -useb https://animeshkundu.github.io/fix/install.ps1 | iex
```

## Features

- Corrects typos and common mistakes in shell commands
- Runs entirely locally — no API calls, no data sent anywhere
- Fast inference with Metal GPU acceleration on Apple Silicon
- Supports multiple shells: bash, zsh, fish, powershell, cmd, tcsh
- Single binary with no runtime dependencies
- Auto-downloads model on first use (~400MB)

## Usage

```bash
# Basic correction
fix "gti status"
# → git status

# With error context for better results
fix -e "command not found: gti" "gti status"

# Specify shell explicitly
fix -s fish "gut push"
```

### Model Management

```bash
# List available models
fix --list-models

# Download and set a different model
fix --use-model qwen3-correct-0.6B

# Show current config
fix --show-config

# Force re-download
fix --update "gti status"
```

## Installation

### Pre-built Binaries

Download from [GitHub Releases](https://github.com/animeshkundu/fix/releases):

| Platform | Binary |
|----------|--------|
| macOS Apple Silicon | `fix-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `fix-x86_64-apple-darwin.tar.gz` |
| Linux x64 | `fix-x86_64-unknown-linux-gnu.tar.gz` |
| Windows x64 | `fix-x86_64-pc-windows-msvc.zip` |

### Build from Source

```bash
cd fix-cli

# macOS with Metal GPU (recommended for Apple Silicon)
cargo build --release --features metal

# Linux/Windows with CUDA
cargo build --release --features cuda

# CPU-only (any platform)
cargo build --release
```

## Options

```
-e, --error <ERROR>      Error message from the failed command
-s, --shell <SHELL>      Override shell detection (bash, zsh, fish, powershell, cmd, tcsh)
-m, --model <MODEL>      Path to a local GGUF model file
    --gpu-layers <N>     Number of GPU layers to offload (default: 99)
-v, --verbose            Show model loading and inference logs
    --list-models        List available models from HuggingFace
    --use-model <NAME>   Download and set a model as default
    --show-config        Show current configuration
    --update             Force re-download of current model
-h, --help               Print help
-V, --version            Print version
```

## Shell Integration

The installer can automatically configure shell integration. If you installed manually, add the following to your shell config.

After setup, type `fix` to correct your last command. The corrected command is **pre-filled for review** — press Enter to run it.

### Bash

Add to your `~/.bashrc`:

```bash
# fix - AI-powered shell command corrector
fix() {
    if [[ -n "$1" ]]; then
        command fix "$@"
    else
        local cmd=$(fc -ln -1 | sed 's/^[[:space:]]*//')
        local corrected=$(command fix "$cmd" 2>/dev/null)
        if [[ -n "$corrected" && "$corrected" != "$cmd" ]]; then
            echo "Correcting: $cmd → $corrected"
            read -e -i "$corrected" -p "» " final_cmd
            [[ -n "$final_cmd" ]] && eval "$final_cmd"
        else
            echo "No correction needed"
        fi
    fi
}
```

### Zsh

Add to your `~/.zshrc`:

```zsh
# fix - AI-powered shell command corrector
fix() {
    if [[ -n "$1" ]]; then
        command fix "$@"
    else
        local cmd=$(fc -ln -1 | sed 's/^[[:space:]]*//')
        local corrected=$(command fix "$cmd" 2>/dev/null)
        if [[ -n "$corrected" && "$corrected" != "$cmd" ]]; then
            echo "Correcting: $cmd → $corrected"
            print -z "$corrected"  # Pre-fills the command line
        else
            echo "No correction needed"
        fi
    fi
}
```

### Fish

Add to `~/.config/fish/functions/fix.fish`:

```fish
function fix --description 'Fix the last command'
    if test (count $argv) -gt 0
        command fix $argv
    else
        set -l cmd (string trim (history --max=1))
        set -l corrected (command fix "$cmd" 2>/dev/null)
        if test -n "$corrected" -a "$corrected" != "$cmd"
            echo "Correcting: $cmd → $corrected"
            commandline -r "$corrected"  # Pre-fills the command line
            commandline -f repaint
        else
            echo "No correction needed"
        end
    end
end
```

### PowerShell

Add to your `$PROFILE`:

```powershell
# fix - AI-powered shell command corrector
function fix {
    param([Parameter(ValueFromRemainingArguments=$true)]$args)
    $fixPath = "$env:LOCALAPPDATA\fix\fix.exe"
    if ($args) {
        & $fixPath @args
    } else {
        $lastCmd = (Get-History -Count 1).CommandLine
        $corrected = & $fixPath $lastCmd 2>$null
        if ($corrected -and $corrected -ne $lastCmd) {
            Write-Host "Correcting: " -NoNewline
            Write-Host $lastCmd -ForegroundColor Red
            Write-Host "       to: " -NoNewline
            Write-Host $corrected -ForegroundColor Green
            $response = Read-Host "Run? [Y/n]"
            if ($response -ne "n" -and $response -ne "N") {
                Invoke-Expression $corrected
            }
        } else {
            Write-Host "No correction needed"
        }
    }
}
```

### Cmd (Windows Command Prompt)

For Windows Command Prompt, use PowerShell instead — cmd.exe has limited history access. Alternatively, run `fix.exe` directly:

```batch
fix.exe "gti status"
```

### Tcsh

Add to your `~/.tcshrc`:

```tcsh
alias fixlast 'set _cmd = `history -h 1` && set _fix = `fix "$_cmd"` && echo "→ $_fix" && eval "$_fix"'
```

### Advanced: Auto-suggest on Error (Optional)

For automatic suggestions when commands fail, see the [advanced shell integration docs](https://animeshkundu.github.io/fix/#advanced-shell-integration).

## Model

The CLI automatically downloads a fine-tuned model from HuggingFace on first use:

| Model | Size | Description |
|-------|------|-------------|
| `qwen3-correct-0.6B.gguf` | 378 MB | Default model (Q4_K_M quantized) |
| `qwen3-correct-1.7B.gguf` | ~1.0 GB | Higher quality (Q4_K_M quantized) |

Models are stored in:
- **macOS**: `~/Library/Application Support/fix/`
- **Linux**: `~/.config/fix/`
- **Windows**: `%APPDATA%\fix\`

**Model Repository**: [animeshkundu/cmd-correct](https://huggingface.co/animeshkundu/cmd-correct)

## Related Projects

Inspired by:

- **[thefuck](https://github.com/nvbn/thefuck)** — The original shell command corrector. Uses rule-based matching with 100+ built-in rules.
- **[oops](https://github.com/animeshkundu/oops)** — A Rust rewrite of thefuck with additional rules.

**How fix differs:**
- Uses a fine-tuned LLM instead of rule-based matching
- Can handle novel typos and context that rules might miss
- Single binary with no Python/Node runtime
- Runs completely offline with local inference

## Contributing

Contributions are welcome. Please open an issue first to discuss what you'd like to change.

## License

MIT
