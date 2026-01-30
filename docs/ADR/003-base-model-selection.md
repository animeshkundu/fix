# ADR 003: Base Model Selection

## Status
Accepted

## Context
We need a small, efficient base model for shell command correction. The model must be:
- Small enough for local inference (<1GB quantized)
- Instruction-tuned for following correction tasks
- Available in MLX-compatible format

## Decision
Use Qwen2.5-0.5B-Instruct as primary base model, with Qwen3-0.6B as secondary option.

| Criteria | Qwen2.5-0.5B | Qwen3-0.6B |
|----------|--------------|------------|
| Parameters | 0.5B | 0.6B |
| Training iterations | 2000 | 3000 |
| Quantized size (Q4_K_M) | 379MB | 378MB |
| Performance | Good | Better |

## Consequences

### Positive
- Fast inference (<200ms typical)
- Small deployment size
- MLX-optimized 4-bit versions available
- Good instruction-following capability

### Negative
- Limited capacity compared to larger models
- May struggle with complex multi-step corrections
- Qwen3 requires more training iterations for convergence

## Alternatives Considered
- Llama 3.2 1B - Larger, slower inference
- Phi-3 Mini - Good but larger deployment
- TinyLlama 1.1B - Less instruction-tuned
- Gemma 2B - Larger than needed
