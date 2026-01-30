# Architecture Overview

## System Design

cmd-correct is a shell command correction system with dual inference backends (MLX and llama.cpp) and a native Rust CLI.

```
┌─────────────────────────────────────────────────────────────────┐
│                        User Interface                            │
├─────────────────────────────────────────────────────────────────┤
│  Python CLI (typer)  │  Rust CLI (clap)  │  Shell Integration   │
│  - correct           │  - Single binary   │  - fuck() function   │
│  - batch             │  - Metal GPU       │  - Fish/Zsh/Bash     │
│  - interactive       │  - Auto model find │                      │
│  - info              │                    │                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                     Inference Layer                              │
├──────────────────────────┬──────────────────────────────────────┤
│      MLX Backend         │        llama.cpp Backend              │
│  (Apple Silicon Native)  │      (Cross-platform CPU/GPU)        │
├──────────────────────────┼──────────────────────────────────────┤
│  - model.safetensors     │  - .gguf quantized models            │
│  - LoRA adapters         │  - Weights baked in                  │
│  - Metal acceleration    │  - Q4_K_M quantization               │
│  - Dynamic adapter load  │  - CPU/Metal inference               │
└──────────────────────────┴──────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Model Layer                                 │
├─────────────────────────────────────────────────────────────────┤
│  Base: Qwen2.5-0.5B-Instruct / Qwen3-0.6B                       │
│  Fine-tuned on: ~150,000 shell command examples                  │
│  Format: ChatML with system/user/assistant turns                 │
└─────────────────────────────────────────────────────────────────┘
```

## Directory Structure

```
cmd-correct/
├── src/                          # Python source
│   ├── inference/                # Inference layer
│   │   ├── cli.py               # Python CLI (typer)
│   │   └── model.py             # MLX & llama.cpp backends
│   ├── training/                 # Training utilities
│   │   ├── config.py            # Configuration classes
│   │   ├── train.py             # Training script
│   │   └── prepare_dataset.py   # Dataset preparation
│   └── data_generation/          # Synthetic data generation
│       ├── generators/           # 4 dataset generators
│       ├── rule_extractors/      # thefuck/oops parsers
│       └── templates/            # Shell-specific templates
├── cmd-correct-cli/              # Rust native CLI
│   ├── src/main.rs              # Rust implementation
│   └── Cargo.toml               # Rust dependencies
├── models/                       # Model artifacts
│   ├── cmd-correct-v1-mlx/      # MLX format (merged)
│   ├── *.gguf                   # Quantized GGUF models
│   └── qwen*/                   # Base/fine-tuned models
├── adapters/                     # LoRA adapters
│   ├── adapter_config.json      # Qwen2.5 adapter
│   └── qwen3-cmd-correct/       # Qwen3 adapter
├── configs/                      # Training configs
│   ├── train_mlx.yaml           # Qwen2.5 config
│   └── train_qwen3.yaml         # Qwen3 config
├── data/                         # Training data
│   ├── final/                   # train/val/test splits
│   ├── generated/               # DS1-DS4 datasets
│   └── raw/                     # thefuck/oops sources
├── scripts/                      # Utility scripts
└── tools/llama.cpp/              # GGUF conversion tools
```

## Data Flow

### Inference Flow

```
User Input          Prompt Formatting       Backend Selection       Model Inference
    │                     │                       │                       │
    ▼                     ▼                       ▼                       ▼
"gti status"  →  <|im_start|>system    →   GGUF exists?  →  llama.cpp inference
    +               You are...                  │                   │
  "bash"          <|im_end|>                    No                  │
                  <|im_start|>user          ▼                       │
                  gti status            MLX available?              │
                  <|im_end|>                │                       │
                  <|im_start|>assistant     Yes                     │
                                            ▼                       │
                                       MLX inference ◄──────────────┘
                                            │
                                            ▼
                                    "git status"
```

### Training Data Flow

```
DS1: Single Commands (35k)   ─┐
DS2: Chained Commands (35k)  ─┼─→ Shuffle → Split (90/5/5) → train.jsonl
DS3: Natural Language (50k)  ─┤                              val.jsonl
DS4: Top 100 Tools (30k)     ─┘                              test.jsonl
```

## Component Details

### Python CLI (`src/inference/cli.py`)

Commands:
- `correct <shell> <command>` - Single correction
- `batch <shell> <file>` - Batch processing
- `interactive [shell]` - REPL mode
- `info` - Model information

### Rust CLI (`cmd-correct-cli/`)

Single binary with:
- Auto model discovery (~/.config, ~/.local/share, cwd)
- Metal GPU acceleration via llama-cpp-2
- Stderr suppression for clean output

### Inference Backends

| Backend | Format | Size | Speed | Platform |
|---------|--------|------|-------|----------|
| MLX | safetensors + adapter | ~1.1GB | Fastest | Apple Silicon |
| llama.cpp | GGUF Q4_K_M | ~379MB | Fast | Cross-platform |

### Model Variants

| Model | Base | Format | Size | Status |
|-------|------|--------|------|--------|
| cmd-correct-v1 | Qwen2.5-0.5B | MLX + GGUF | 379MB-1.1GB | Production |
| qwen3-cmd-correct | Qwen3-0.6B | MLX + GGUF | 378MB-1.1GB | Production |

## Supported Shells

| Shell | Weight | Platform |
|-------|--------|----------|
| bash | 35% | Linux/macOS |
| zsh | 25% | macOS/Linux |
| powershell | 20% | Windows/Cross |
| cmd | 12% | Windows |
| fish | 5% | Cross-platform |
| tcsh | 3% | BSD/Linux |
