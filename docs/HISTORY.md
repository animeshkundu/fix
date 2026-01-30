# Project History

## Timeline

### Phase 1: Research & Design
- Analyzed existing tools (thefuck, oops) for error patterns
- Selected Qwen2.5-0.5B as base model for size/performance tradeoff
- Designed synthetic data generation pipeline
- Chose LoRA for efficient fine-tuning

### Phase 2: Data Generation
- Built 4 complementary data generators (DS1-DS4)
- Created shell-specific YAML templates
- Generated ~150,000 training examples
- Implemented quality validation scripts

### Phase 3: Model Training
- Trained Qwen2.5-0.5B with 2000 iterations
- Achieved satisfactory correction accuracy
- Saved checkpoints at 500-step intervals

### Phase 4: Qwen3 Migration
- Trained Qwen3-0.6B variant with 3000 iterations
- Improved performance on complex corrections
- Created both F16 and Q4_K_M GGUF exports

### Phase 5: Production CLI
- Implemented Python CLI with typer
- Built Rust native CLI for shell integration
- Added Metal GPU acceleration
- Auto model discovery system

## Model Evolution

| Version | Base Model | Iterations | Status |
|---------|-----------|------------|--------|
| v1.0 | Qwen2.5-0.5B | 2000 | Production |
| v1.1 | Qwen3-0.6B | 3000 | Production |

## Key Decisions

1. **Chose LoRA over full fine-tuning** - 100x smaller adapters
2. **Dual backend (MLX + llama.cpp)** - Platform flexibility
3. **Rust CLI addition** - Faster shell integration
4. **Q4_K_M quantization** - 3x size reduction with minimal quality loss
5. **Synthetic data approach** - Scalable, controllable training data

## Lessons Learned

1. **Small models work well for focused tasks** - 0.5B is sufficient for command correction
2. **Synthetic data quality matters** - Template diversity is critical
3. **Low temperature (0.1) is key** - Deterministic outputs for commands
4. **Shell integration needs speed** - Hence Rust CLI
5. **LoRA adapters are practical** - Easy to version and deploy

## Future Directions

- [ ] Expand shell support (nushell, elvish)
- [ ] Add error explanation mode
- [ ] Integrate with IDE terminals
- [ ] Online learning from corrections
- [ ] Multi-language command support
