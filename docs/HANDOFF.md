# Handoff Document

## Quick Start for New Developers/LLMs

### What is this project?
A fine-tuned LLM system that corrects shell commands. It takes an incorrect command like `gti status` and outputs `git status`.

### Repository Structure Overview

```
cmd-correct/
├── src/inference/          # Main inference code (READ THIS FIRST)
│   ├── cli.py             # Python CLI entry point
│   └── model.py           # MLX and llama.cpp backends
├── cmd-correct-cli/        # Rust native CLI
├── models/                 # Trained models (GGUF files)
├── adapters/               # LoRA adapters
├── configs/                # Training configs
└── data/                   # Training data
```

### Key Files to Understand

1. **`src/inference/model.py`** - Core inference logic
   - `CommandCorrector` class - Main API
   - `MLXModel` - Apple Silicon backend
   - `LlamaCppModel` - Cross-platform backend

2. **`src/inference/cli.py`** - Python CLI
   - 4 commands: correct, batch, interactive, info

3. **`cmd-correct-cli/src/main.rs`** - Rust CLI
   - Single binary, Metal GPU support

4. **`src/training/config.py`** - All configuration classes

### How to Run Inference

```bash
# Python CLI
python -m src.inference.cli correct bash "gti status"

# Or if installed
cmd-correct correct bash "gti status"

# Rust CLI (from cmd-correct-cli/)
cargo run -- "gti status"
```

### How to Train

```bash
# Using MLX
python -m mlx_lm.lora --config configs/train_mlx.yaml

# Or using the training script
python -m src.training.train --backend mlx --max-steps 2000
```

### Model Locations

| Model | Path | Use Case |
|-------|------|----------|
| Production (small) | `models/cmd-correct-v1-q4km.gguf` | Rust CLI |
| Production (Qwen3) | `models/cmd-correct-v1-qwen3-Q4_K_M.gguf` | Latest |
| MLX format | `models/cmd-correct-v1-mlx/` | Python MLX |
| Adapters | `adapters/` | LoRA weights |

### Common Tasks

#### Add a new shell
1. Create template in `src/data_generation/templates/{shell}.yaml`
2. Add shell to `SHELL_WEIGHTS` in `base_generator.py`
3. Regenerate data and retrain

#### Modify inference parameters
Edit `src/training/config.py`:
```python
max_tokens: 128
temperature: 0.1
top_p: 0.9
```

#### Export to GGUF
```bash
# Fuse LoRA adapters
python -m mlx_lm.fuse --model <base> --adapter-path adapters/ --save-path models/fused

# Convert to GGUF
python tools/llama.cpp/convert_hf_to_gguf.py models/fused --outfile models/output.gguf

# Quantize
./tools/llama.cpp/build/bin/llama-quantize models/output.gguf models/output-q4km.gguf Q4_K_M
```

### Testing

```bash
# Quick test
python -m src.inference.cli correct bash "gti status"
# Expected: git status

# Test Rust CLI
cd cmd-correct-cli && cargo run -- "gti status"
```

### Dependencies

- Python 3.10+
- MLX (for Apple Silicon)
- llama-cpp-python (for GGUF inference)
- Rust (for native CLI)

### Troubleshooting

| Issue | Solution |
|-------|----------|
| Model not found | Check model paths in config, or set --gguf flag |
| MLX import error | Install with `pip install mlx mlx-lm` |
| Slow inference | Use GGUF Q4_K_M model with GPU layers |
| Wrong output | Check temperature (should be 0.1 for determinism) |

### Contact
- Author: Animesh Kundu
- License: MIT
