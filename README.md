# fix

AI-powered shell command corrector using a fine-tuned local LLM.

**[Website](https://animeshkundu.github.io/fix/)** | **[Model](https://huggingface.co/animeshkundu/cmd-correct)**

## Quick Install

```bash
curl -fsSL https://animeshkundu.github.io/fix/install.sh | sh
```

Or download from [GitHub Releases](https://github.com/animeshkundu/fix/releases).

## Features

- Corrects typos and common mistakes in shell commands
- Runs entirely locally - no API calls, no data sent to cloud
- Fast inference using Metal GPU acceleration on Apple Silicon
- Supports multiple shells: bash, zsh, fish, powershell, cmd, tcsh
- Single binary with no runtime dependencies

## Installation

### One-liner (macOS/Linux)

```bash
curl -fsSL https://animeshkundu.github.io/fix/install.sh | sh
```

### Pre-built binaries

Download from [GitHub Releases](https://github.com/animeshkundu/fix/releases):

| Platform | Binary |
|----------|--------|
| macOS Apple Silicon | `fix-aarch64-apple-darwin.tar.gz` |
| macOS Intel | `fix-x86_64-apple-darwin.tar.gz` |
| Linux x64 | `fix-x86_64-unknown-linux-gnu.tar.gz` |
| Windows x64 | `fix-x86_64-pc-windows-msvc.zip` |

### Build from source

```bash
cd fix-cli

# macOS with Metal GPU (recommended for Apple Silicon)
cargo build --release --features metal

# Linux/Windows with CUDA
cargo build --release --features cuda

# CPU-only (any platform)
cargo build --release
```

The binary will be at `fix-cli/target/release/fix`.

### Model Setup

**Automatic (Recommended)**: The CLI automatically downloads the model from HuggingFace on first use:

```bash
fix "gti status"
# Downloads qwen3-correct-0.6B.gguf (~378 MB) on first run
```

**Manual**: Or specify a custom path with `--model /path/to/model.gguf`.

**Model Repository**: [animeshkundu/fix](https://huggingface.co/animeshkundu/cmd-correct)

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
```

### Model Management

```bash
# List available models from HuggingFace
fix --list-models

# Download and set a different model as default
fix --use-model qwen3-correct-0.6B

# Show current configuration
fix --show-config

# Force re-download of current model
fix --update "gti status"
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
```

## Shell Integration

### Bash/Zsh

Add to your `.bashrc` or `.zshrc`:

```bash
fuck() {
    local cmd=$(fc -ln -1)
    local corrected=$(fix "$cmd")
    echo "Correcting: $cmd -> $corrected"
    eval "$corrected"
}
```

### Fish

Add to `~/.config/fish/functions/fuck.fish`:

```fish
function fuck
    set -l cmd (history --max=1)
    set -l corrected (fix "$cmd")
    echo "Correcting: $cmd -> $corrected"
    eval $corrected
end
```

## Related Projects

This project was inspired by these excellent command correction tools:

- **[thefuck](https://github.com/nvbn/thefuck)** - The original shell command corrector by [@nvbn](https://github.com/nvbn). Uses rule-based matching with 100+ built-in rules for common tools. Written in Python.

- **[oops](https://github.com/animeshkundu/oops)** - A Rust rewrite of thefuck with additional rules. Faster startup time with the same rule-based approach.

**How fix differs:**
- Uses a fine-tuned LLM instead of rule-based matching
- Can handle novel typos and context that rules might miss
- Single binary with no Python/Node runtime needed
- Runs completely offline with local model inference

## Training & Model

The model was trained using:
- Synthetic data generation (~150k examples)
- LoRA fine-tuning on Qwen3-0.6B
- Q4_K_M quantization with importance matrix

### Training Data Coverage

| Category | Examples | Description |
|----------|----------|-------------|
| Single command typos | 35k | `gti` → `git`, `dockr` → `docker` |
| Chained commands | 35k | `git add . && git comit` → `git add . && git commit` |
| Natural language | 50k | `list files` → `ls` |
| Tool-specific | 30k | git, docker, npm, cargo, kubectl patterns |

### Model Variants

| Model | Size | Format | Use Case |
|-------|------|--------|----------|
| qwen3-correct-0.6B.gguf | 378 MB | GGUF Q4_K_M | Production (recommended) |

Models are hosted at [animeshkundu/cmd-correct](https://huggingface.co/animeshkundu/cmd-correct) and automatically downloaded on first use.

## License

MIT
