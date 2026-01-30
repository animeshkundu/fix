# ADR 001: Dual Backend Inference Architecture

## Status
Accepted

## Context
We need an inference system that works across different platforms while optimizing for Apple Silicon performance. Users may have different hardware configurations and deployment requirements.

## Decision
Implement two inference backends:

1. **MLX Backend** - Native Apple Silicon inference using MLX framework
2. **llama.cpp Backend** - Cross-platform inference using quantized GGUF models

The system auto-selects the optimal backend based on:
1. If GGUF model exists → use llama.cpp
2. If MLX is available → use MLX
3. Fallback → llama.cpp

## Consequences

### Positive
- Optimal performance on Apple Silicon with MLX
- Cross-platform support via llama.cpp
- Smaller deployment with Q4_K_M quantization (~379MB vs 1.1GB)
- LoRA adapters can be swapped dynamically with MLX backend

### Negative
- Two codepaths to maintain
- Different feature sets (MLX supports dynamic adapters, llama.cpp has baked weights)
- Testing complexity increases

## Alternatives Considered
- Single backend (llama.cpp only) - Would lose native Apple Silicon optimization
- ONNX Runtime - Less mature for LLM inference
- vLLM - Overkill for single-user CLI tool
