# Technical Specifications

## Model Specifications

### Base Models

| Specification | Qwen2.5-0.5B | Qwen3-0.6B |
|--------------|--------------|------------|
| Parameters | 0.5B | 0.6B |
| Context Window | 32K | 32K |
| Vocabulary | 151,936 | 151,936 |
| Architecture | Transformer | Transformer |
| Precision (Training) | 4-bit | 4-bit |
| Source | mlx-community | mlx-community |

### Fine-tuned Models

| Model | Format | Size | Quantization |
|-------|--------|------|--------------|
| cmd-correct-v1.gguf | GGUF | 948MB | F16 |
| cmd-correct-v1-q4km.gguf | GGUF | 379MB | Q4_K_M |
| qwen3-cmd-correct-f16.gguf | GGUF | 1.1GB | F16 |
| cmd-correct-v1-qwen3-Q4_K_M.gguf | GGUF | 378MB | Q4_K_M |

### LoRA Adapter Specifications

```yaml
lora_rank: 8
lora_alpha: 16 (effective scale: 2.0)
lora_dropout: 0.0-0.05
target_modules: ["q_proj", "v_proj"]
trainable_layers: 16
adapter_size: ~11MB per checkpoint
```

## Training Specifications

### Hyperparameters

| Parameter | Qwen2.5 | Qwen3 |
|-----------|---------|-------|
| Iterations | 2000 | 3000 |
| Batch Size | 4 | 4 |
| Learning Rate | 1e-4 | 1e-4 |
| Max Sequence Length | 256 | 256 |
| Optimizer | Adam | Adam |
| Evaluation Steps | 200 | 200 |
| Save Frequency | 500 | 500 |
| Validation Batches | 50 | 50 |

### Dataset Statistics

| Dataset | Examples | Purpose |
|---------|----------|---------|
| DS1: Single Commands | 35,000 | Typos, flags, permissions, paths, syntax |
| DS2: Chained Commands | 35,000 | Pipes, redirects, command chaining |
| DS3: Natural Language | 50,000 | NL to command translation |
| DS4: Top 100 Tools | 30,000 | Tool-specific corrections |
| **Total** | **150,000** | |

### Data Split

| Split | Percentage | Examples |
|-------|------------|----------|
| Train | 90% | ~135,000 |
| Validation | 5% | ~7,500 |
| Test | 5% | ~7,500 |

## Inference Specifications

### Generation Parameters

```python
max_tokens: 128
temperature: 0.1        # Low for deterministic output
top_p: 0.9
repetition_penalty: 1.1
stop_tokens: ["<|im_end|>", "\n"]
```

### Context Configuration

```python
n_ctx: 512              # Context window for llama.cpp
n_threads: 4            # CPU threads
n_gpu_layers: 99        # GPU offload (Rust CLI)
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

## Performance Benchmarks

### Inference Latency (Approximate)

| Backend | Hardware | Latency |
|---------|----------|---------|
| MLX | M1 Pro | ~50-100ms |
| llama.cpp (Q4_K_M) | M1 Pro | ~100-200ms |
| llama.cpp (Q4_K_M) | CPU (8 core) | ~200-500ms |

### Model Size Comparison

| Format | Size | Compression |
|--------|------|-------------|
| F16 (float16) | 1.1GB | Baseline |
| Q4_K_M | 378-379MB | ~3x compression |

## API Specifications

### Python API

```python
from src.inference.model import CommandCorrector

corrector = CommandCorrector(
    model_path="models/cmd-correct-v1-mlx",
    adapter_path="adapters",
    gguf_path="models/cmd-correct-v1-q4km.gguf",
    backend="auto"  # "mlx", "llama_cpp", or "auto"
)

result = corrector.correct(shell="bash", command="gti status")
# Returns: "git status"
```

### CLI Specification

```bash
# Python CLI
cmd-correct correct <shell> <command> [--model PATH] [--adapter PATH] [--gguf PATH] [--quiet]
cmd-correct batch <shell> <file> [--output FILE]
cmd-correct interactive [shell]
cmd-correct info

# Rust CLI
cmd-correct <command> [-e ERROR] [-s SHELL] [-m MODEL] [--gpu-layers N] [-v]
```

## File Format Specifications

### Training Data (JSONL)

```json
{
  "messages": [
    {"role": "system", "content": "You are a shell command corrector for bash..."},
    {"role": "user", "content": "gti status"},
    {"role": "assistant", "content": "git status"}
  ]
}
```

### Adapter Config

```json
{
  "model": "mlx-community/Qwen3-0.6B-4bit",
  "lora_parameters": {
    "rank": 8,
    "dropout": 0.0,
    "scale": 20.0
  },
  "num_layers": 16,
  "learning_rate": 0.0001,
  "iters": 3000,
  "batch_size": 4,
  "max_seq_length": 256,
  "fine_tune_type": "lora"
}
```

## Dependencies

### Python Dependencies

| Package | Version | Purpose |
|---------|---------|---------|
| mlx | >=0.12 | Apple Silicon ML |
| mlx-lm | >=0.12 | LLM support for MLX |
| llama-cpp-python | >=0.2 | llama.cpp bindings |
| typer | >=0.12 | CLI framework |
| rich | >=13.0 | Terminal output |
| pyyaml | >=6.0 | Config loading |

### Rust Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| llama-cpp-2 | 0.1 | llama.cpp bindings |
| clap | 4 | CLI parsing |
| dirs | 6 | System directories |

## System Requirements

### Minimum

- Python 3.10+
- 4GB RAM
- Any CPU (x86_64 or ARM64)

### Recommended

- Apple Silicon (M1/M2/M3) for MLX backend
- 8GB RAM
- SSD storage for models

### Storage Requirements

| Component | Size |
|-----------|------|
| Models | ~5.9GB |
| Adapters | ~400MB |
| Training Data | ~2GB |
| llama.cpp tools | ~1.5GB |
| **Total** | **~10GB** |
