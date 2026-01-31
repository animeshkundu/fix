# fix

AI-powered shell command corrector using a fine-tuned local LLM.

## Features

- Corrects typos and common mistakes in shell commands
- Runs entirely locally - no API calls
- Fast inference using Metal GPU acceleration on Apple Silicon
- Supports multiple shells: bash, zsh, fish, powershell, cmd, tcsh

## Related Projects

This project was inspired by and can be used alongside:

- **[thefuck](https://github.com/nvbn/thefuck)** - The original shell command corrector (Python). Uses rule-based matching and shell history to suggest corrections. Great for learning common command patterns.

- **[oops](https://github.com/0atman/oops)** - A Rust rewrite of thefuck with improved performance. Contains 30+ rule-based correction modules for tools like git, docker, npm, cargo, and more.

fix takes a different approach by using a fine-tuned LLM for corrections, which can handle novel mistakes and context that rule-based systems might miss.

## Prerequisites

- **Rust toolchain** (1.70+) - Install via [rustup](https://rustup.rs/)
- **GGUF model file** - A quantized model trained for command correction (see Model section below)
- **macOS with Apple Silicon** (recommended) - For Metal GPU acceleration
  - Linux/Windows: Works but requires llama.cpp to be built for your platform

## Installation

### Build from source

```bash
cargo build --release
```

The binary will be at `target/release/fix`.

### Model

The CLI requires a GGUF-format model file trained for command correction. The model is a fine-tuned Qwen2.5-0.5B quantized to 4-bit (Q4_K_M).

**Model file**: `fix-v1-q4km.gguf` (~378 MB)

Place the model in one of these locations (searched in order):

1. Current directory
2. Next to the executable
3. `~/.config/fix/`
4. `~/.local/share/fix/` (Linux) or `~/Library/Application Support/fix/` (macOS)

Or specify a custom path with `--model /path/to/model.gguf`.

## Usage

```bash
# Basic usage - outputs only the corrected command
fix "gti status"
# Output: git status

# With verbose mode to see model loading info
fix --verbose "dockr ps"

# Specify shell explicitly
fix --shell fish "gut push"

# Provide error message for better context
fix --error "command not found: gti" "gti status"

# Specify custom model path
fix --model /path/to/model.gguf "gti status"
```

## Options

```
-e, --error <ERROR>      Error message from the failed command
-s, --shell <SHELL>      Override shell detection (bash, zsh, fish, powershell, cmd, tcsh)
-m, --model <MODEL>      Path to the GGUF model file
    --gpu-layers <N>     Number of GPU layers to offload (default: 99)
-v, --verbose            Show model loading and inference logs
-h, --help               Print help
```

## Shell Integration

### Bash/Zsh

Add to your `.bashrc` or `.zshrc`:

```bash
# Correct last command with 'fuck'
fuck() {
    local cmd=$(fc -ln -1)
    local corrected=$(fix "$cmd")
    echo "Correcting: $cmd -> $corrected"
    eval "$corrected"
}
```

### Fish

Add to your `~/.config/fish/functions/fuck.fish`:

```fish
function fuck
    set -l cmd (history --max=1)
    set -l corrected (fix "$cmd")
    echo "Correcting: $cmd -> $corrected"
    eval $corrected
end
```

## License

MIT
