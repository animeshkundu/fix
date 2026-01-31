# AGENTS.md - AI Agent Guide for fix

## Project Summary
**fix** is a shell command correction system using a fine-tuned LLM (Qwen2.5-0.5B / Qwen3-0.6B) with dual inference backends (MLX and llama.cpp).

## Repository Map

### Core Inference (Most Important)
```
src/inference/
├── cli.py          # Python CLI - 4 commands: correct, batch, interactive, info
└── model.py        # Backend selection: MLXModel, LlamaCppModel, CommandCorrector
```

### Rust Native CLI
```
fix-cli/
├── src/main.rs     # Rust implementation with Metal GPU (~565 lines)
└── Cargo.toml      # Dependencies: llama-cpp-2, clap, dirs, reqwest, indicatif, serde
```

**Key Features:**
- Auto-downloads models from HuggingFace Hub
- Persistent config (default model saved to config.json)
- Cross-platform paths (macOS, Linux, Windows)
- Progress bar for downloads

### Training Infrastructure
```
src/training/
├── config.py           # TrainingConfig, InferenceConfig dataclasses
├── train.py            # MLX and PyTorch training backends
└── prepare_dataset.py  # Dataset formatting utilities
```

### Data Generation
```
src/data_generation/
├── generators/
│   ├── base_generator.py       # Base class, shell weights, typo generation
│   ├── single_command_gen.py   # DS1: 35k single command errors
│   ├── chained_command_gen.py  # DS2: 35k piped/chained commands
│   ├── natural_lang_gen.py     # DS3: 50k NL to command
│   └── tools_gen.py            # DS4: 30k top 100 CLI tools
├── templates/                   # Shell-specific YAML templates
└── rule_extractors/             # thefuck/oops rule parsers
```

### Models & Adapters
```
models/
├── fix-v1-q4km.gguf           # Production (379MB, Q4_K_M)
├── fix-v1-qwen3-Q4_K_M.gguf   # Qwen3 version (378MB)
├── qwen3-fix-merged/           # MLX format
└── imatrix.gguf                        # Importance matrix

adapters/
├── adapter_config.json                 # Qwen2.5 config
└── qwen3-fix/                  # Qwen3 LoRA (checkpoints at 500-3000)
```

### Configuration
```
configs/
├── train_mlx.yaml      # Qwen2.5: 2000 iters, batch 4, lr 1e-4
└── train_qwen3.yaml    # Qwen3: 3000 iters, batch 4, lr 1e-4
```

## Key Patterns

### Inference Flow
```python
# Backend auto-selection in model.py:
if gguf_path exists -> LlamaCppModel
elif mlx available -> MLXModel
else -> LlamaCppModel
```

### Prompt Format (ChatML)
```
<|im_start|>system
You are a shell command corrector for {shell}. Output only the corrected command.
<|im_end|>
<|im_start|>user
{incorrect_command}
<|im_end|>
<|im_start|>assistant
```

### Generation Parameters
```python
max_tokens=128, temperature=0.1, top_p=0.9, repetition_penalty=1.1
```

## Common Modification Points

| Task | File(s) |
|------|---------|
| Change inference params | `src/training/config.py` → InferenceConfig |
| Add shell support | `src/data_generation/templates/` + `base_generator.py` SHELL_WEIGHTS |
| Modify CLI commands | `src/inference/cli.py` |
| Change model paths | `src/training/config.py` or CLI flags |
| Rust CLI changes | `fix-cli/src/main.rs` |
| HuggingFace repo | `fix-cli/src/main.rs` → `HF_REPO` constant |
| Default model | `fix-cli/src/main.rs` → `DEFAULT_MODEL` constant |

## HuggingFace Integration (Rust CLI)

**Repository**: `animeshkundu/fix`

### CLI Model Management

```bash
# List available models (queries HF API)
fix --list-models

# Download and set default (persistent)
fix --use-model qwen3-correct-0.6B

# Show config
fix --show-config
```

### Config Locations

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/fix/` |
| Linux | `~/.config/fix/` |
| Windows | `%APPDATA%\fix\` |

### Key Functions (main.rs)

| Function | Purpose |
|----------|---------|
| `fetch_available_models()` | Query HuggingFace API for .gguf files |
| `download_model()` | Download with progress bar |
| `load_config()` / `save_config()` | Persistent settings |
| `config_dir()` | Cross-platform path resolution |

## Build & Run Commands

```bash
# Python inference
python -m src.inference.cli correct bash "gti status"

# Rust CLI
cd fix-cli && cargo run -- "gti status"

# Training
python -m mlx_lm.lora --config configs/train_qwen3.yaml

# GGUF conversion
python tools/llama.cpp/convert_hf_to_gguf.py <model> --outfile <output.gguf>
```

## Dependencies
- Python: mlx, mlx-lm, llama-cpp-python, typer, rich
- Rust: llama-cpp-2 (with metal), clap, dirs

## Shell Distribution
bash(35%), zsh(25%), powershell(20%), cmd(12%), fish(5%), tcsh(3%)

## Data Format
JSONL with ChatML messages array: system → user (incorrect) → assistant (correct)

## Related Repositories

- **Training**: `Model was trained with MLX LoRA fine-tuning
- **Pipeline**: `../shellfix/` - Automated DVC pipeline for end-to-end training
- **Models**: [animeshkundu/fix](https://huggingface.co/animeshkundu/fix) - HuggingFace model repository
