# ADR 002: LoRA Fine-tuning Strategy

## Status
Accepted

## Context
We need to fine-tune a small language model for shell command correction. Full fine-tuning would be expensive and produce large model files. We need an efficient approach.

## Decision
Use LoRA (Low-Rank Adaptation) for fine-tuning with these parameters:
- Rank: 8
- Alpha: 16 (scale factor 2.0)
- Target modules: q_proj, v_proj (query and value projections)
- Dropout: 0.0-0.05

## Consequences

### Positive
- Small adapter size (~11MB per checkpoint)
- Fast training (1-3 hours on Apple Silicon)
- Preserves base model capabilities
- Can store multiple adapters for different use cases
- Easy to merge adapters into base model for GGUF export

### Negative
- Slightly less expressive than full fine-tuning
- Limited to specific attention layers
- Requires adapter management in MLX inference

## Implementation Notes
- Training uses MLX's native LoRA implementation
- Adapters can be fused with `mlx_lm.fuse` for GGUF export
- Checkpoints saved every 500 steps for recovery
