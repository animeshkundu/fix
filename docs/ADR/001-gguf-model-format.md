# ADR-001: GGUF Model Format

## Status
Accepted

## Context

We need an efficient model format for edge inference that:
- Loads quickly for interactive CLI use
- Supports quantization for smaller file sizes
- Works across platforms (macOS, Linux, Windows)
- Has good tooling for conversion and quantization

Options considered:
1. **GGUF** - llama.cpp native format
2. **ONNX** - Cross-platform ML format
3. **SafeTensors** - HuggingFace format
4. **PyTorch** - Native .pt files

## Decision

Use **GGUF** (GPT-Generated Unified Format) as the model format.

## Rationale

1. **Native llama.cpp support**: No conversion needed at runtime
2. **Quantization built-in**: Q4_K_M reduces 1.1GB F16 → 378MB
3. **Fast loading**: Memory-mapped, loads in milliseconds
4. **Wide ecosystem**: llama.cpp, ollama, LM Studio all use GGUF
5. **Metadata in file**: Tokenizer and config embedded

## Consequences

### Positive
- Sub-100ms model load time
- 70% size reduction with Q4_K_M quantization
- Single file distribution (no separate tokenizer files)
- Compatible with llama.cpp ecosystem tools

### Negative
- Requires conversion from HuggingFace format
- llama.cpp specific (not portable to other runtimes)
- Need to maintain conversion pipeline

## Implementation

Conversion pipeline in `shellfix/`:
1. Fine-tune → SafeTensors (LoRA adapters)
2. Fuse adapters → Full model SafeTensors
3. Convert → GGUF F16 (1.1GB)
4. Quantize → GGUF Q4_K_M (378MB)
