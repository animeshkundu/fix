# ADR-002: Metal GPU Acceleration

## Status
Accepted

## Context

The CLI needs GPU acceleration for fast inference. Primary target is Apple Silicon Macs, but we also need to support Linux and Windows.

Options considered:
1. **Metal** - Apple's GPU framework
2. **CUDA** - NVIDIA's GPU framework
3. **Vulkan** - Cross-platform GPU API
4. **CPU-only** - No GPU acceleration

## Decision

Use **Metal** as the primary GPU backend for macOS, with CUDA support for Linux/Windows via Cargo features.

## Rationale

1. **Apple Silicon dominance**: M1/M2/M3 Macs are common developer machines
2. **Unified memory**: Metal leverages unified memory architecture
3. **llama-cpp-2 support**: Native Metal backend via `--features metal`
4. **Dramatic speedup**: 10-50x faster than CPU inference

## Consequences

### Positive
- Sub-100ms inference on Apple Silicon
- 99 GPU layers offloaded by default
- No external dependencies (Metal is built into macOS)
- Excellent power efficiency

### Negative
- macOS-only for Metal (need CUDA for other platforms)
- Requires separate builds per platform
- Metal-specific debugging can be challenging

## Implementation

```toml
# Cargo.toml
[features]
default = []
metal = ["llama-cpp-2/metal"]
cuda = ["llama-cpp-2/cuda"]
```

Build commands:
```bash
# macOS
cargo build --release --features metal

# Linux/Windows with NVIDIA
cargo build --release --features cuda

# CPU fallback
cargo build --release
```

Runtime configuration:
```rust
let model_params = LlamaModelParams::default()
    .with_n_gpu_layers(args.gpu_layers); // default: 99
```
