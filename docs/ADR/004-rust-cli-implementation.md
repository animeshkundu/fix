# ADR 004: Rust CLI Implementation

## Status
Accepted

## Context
The Python CLI works well but has startup overhead and requires Python runtime. For shell integration where instant response is critical, we need a native solution.

## Decision
Implement a native Rust CLI using llama-cpp-2 bindings with Metal GPU support.

Features:
- Single static binary
- Auto model discovery (cwd, ~/.config, ~/.local/share)
- Metal GPU acceleration
- Stderr suppression for clean output
- Shell detection

## Consequences

### Positive
- Near-instant startup (<50ms)
- No runtime dependencies
- Single binary distribution
- Metal GPU acceleration
- Better shell integration experience

### Negative
- Separate codebase to maintain
- Only supports GGUF models (no MLX adapters)
- Rust compile times during development
- Limited to llama.cpp capabilities

## Implementation Details

```rust
// Model search locations (in order)
1. Current directory: fix-v1-q4km.gguf
2. Next to executable
3. ~/.config/fix/
4. ~/.local/share/fix/ (Linux)
5. ~/Library/Application Support/fix/ (macOS)

// GPU configuration
n_gpu_layers: 99 (full GPU offload)
n_ctx: 512
```

## Distribution
The Rust binary can be distributed via:
- GitHub releases
- Homebrew (future)
- Cargo install (future)
