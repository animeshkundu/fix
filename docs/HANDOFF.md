# Developer Handoff - fix

## What This Does

A Rust CLI that corrects shell command typos using a local LLM. Takes a mistyped command like `gti status` and outputs the corrected version `git status`.

## Quick Start

```bash
# Build with Metal GPU support (macOS)
cargo build --release --features metal

# Test it
./target/release/fix "gti status"
# Output: git status

# List available models
./target/release/fix --list-models

# Download and set a model as default
./target/release/fix --use-model qwen3-correct-0.6B
```

## Key Files

| File | Purpose |
|------|---------|
| `fix-cli/src/main.rs` | All CLI logic (~565 lines) |
| `fix-cli/Cargo.toml` | Dependencies and features |

## Architecture Overview

```
User Input → Shell Detection → Model Loading → Inference → Output
                                    ↓
                         Config (~/.config/fix/)
                                    ↓
                         HuggingFace Download (if needed)
```

## Key Features

1. **Auto-download**: Downloads model from HuggingFace on first use
2. **Persistent config**: Remembers default model across sessions
3. **Cross-platform**: Works on macOS, Linux, Windows
4. **GPU acceleration**: Metal (macOS), CUDA (Linux/Windows)

## CLI Flags

| Flag | Purpose |
|------|---------|
| `--list-models` | Query available models from HuggingFace |
| `--use-model <name>` | Download and set as default |
| `--show-config` | Display current configuration |
| `--update` | Force re-download current model |
| `--model <path>` | Use specific local model file |
| `--gpu-layers <n>` | Control GPU offload (default: 99) |
| `--verbose` | Show llama.cpp logs |

## Config Locations

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/fix/` |
| Linux | `~/.config/fix/` |
| Windows | `%APPDATA%\fix\` |

## Build Commands

```bash
# macOS with Metal
cargo build --release --features metal

# Linux/Windows with CUDA
cargo build --release --features cuda

# CPU-only (any platform)
cargo build --release
```

## Recent Changes (Jan 2025)

- Added HuggingFace model auto-download
- Added `--list-models`, `--use-model`, `--show-config` flags
- Implemented cross-platform config paths
- Model published to `animeshkundu/fix`

## Related Repos

- `Training code (private)
- `shellfix/` - Automated DVC pipeline for model training
