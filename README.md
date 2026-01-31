# cmd-correct

AI-powered shell command corrector using a fine-tuned local LLM.

## Features

- Corrects typos and common mistakes in shell commands
- Runs entirely locally - no API calls, no data sent to cloud
- Fast inference using Metal GPU acceleration on Apple Silicon
- Supports multiple shells: bash, zsh, fish, powershell, cmd, tcsh
- Single binary with no runtime dependencies

## Installation

### Build from source

```bash
cd cmd-correct-cli
cargo build --release
```

The binary will be at `cmd-correct-cli/target/release/cmd-correct`.

### Model Setup

Download the GGUF model file and place it in one of these locations:

1. Current working directory
2. Next to the executable
3. `~/.config/cmd-correct/`
4. `~/.local/share/cmd-correct/` (Linux) or `~/Library/Application Support/cmd-correct/` (macOS)

Or specify a custom path with `--model /path/to/model.gguf`.

**Model file**: `cmd-correct-v1-q4km.gguf` (~378 MB, 4-bit quantized)

## Usage

```bash
# Basic usage - outputs only the corrected command
cmd-correct "gti status"
# Output: git status

# With verbose mode to see model loading info
cmd-correct --verbose "dockr ps"

# Specify shell explicitly
cmd-correct --shell fish "gut push"

# Provide error message for better context
cmd-correct --error "command not found: gti" "gti status"
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
fuck() {
    local cmd=$(fc -ln -1)
    local corrected=$(cmd-correct "$cmd")
    echo "Correcting: $cmd -> $corrected"
    eval "$corrected"
}
```

### Fish

Add to `~/.config/fish/functions/fuck.fish`:

```fish
function fuck
    set -l cmd (history --max=1)
    set -l corrected (cmd-correct "$cmd")
    echo "Correcting: $cmd -> $corrected"
    eval $corrected
end
```

## Related Projects

This project was inspired by these excellent command correction tools:

- **[thefuck](https://github.com/nvbn/thefuck)** - The original shell command corrector by [@nvbn](https://github.com/nvbn). Uses rule-based matching with 100+ built-in rules for common tools. Written in Python.

- **[oops](https://github.com/animeshkundu/oops)** - A Rust rewrite of thefuck with additional rules. Faster startup time with the same rule-based approach.

**How cmd-correct differs:**
- Uses a fine-tuned LLM instead of rule-based matching
- Can handle novel typos and context that rules might miss
- Single binary with no Python/Node runtime needed
- Runs completely offline with local model inference

## Training & Model

For training infrastructure, dataset generation, and model publishing, see:

**[animeshkundu/oops-llm-training](https://github.com/animeshkundu/oops-llm-training)**

The training repo contains:
- Synthetic data generation pipeline (~150k examples)
- LoRA fine-tuning code for Qwen2.5-0.5B
- GGUF export and quantization scripts
- Rule extractors that parse [thefuck](https://github.com/nvbn/thefuck) and [oops](https://github.com/animeshkundu/oops) for training data

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
| cmd-correct-v1-q4km.gguf | 378 MB | GGUF Q4_K_M | Production (recommended) |
| cmd-correct-v1-f16.gguf | 1.1 GB | GGUF F16 | Higher accuracy |

## License

MIT
