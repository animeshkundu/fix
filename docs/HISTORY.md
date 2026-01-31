# Development History - fix

## January 2025

### Cross-Platform Testing Infrastructure (Jan 31)

Added comprehensive testing across platforms, shells, and distributions (ADR-006):

**New CI Workflows:**
- `test-wsl.yml` - WSL-specific path and environment testing
- `test-windows-shells.yml` - PowerShell (pwsh + powershell.exe) and CMD testing
- `test-install.yml` - Installation script testing on all platforms
- `test-distros.yml` - Linux distribution testing (Alpine, Debian, Ubuntu 20.04, Fedora)

**Integration Tests (`fix-cli/tests/`):**
- `cli_test.rs` - Binary execution and flag testing
- `config_test.rs` - Cross-platform config path verification
- `wsl_test.rs` - WSL environment isolation tests
- `e2e_test.rs` - Model inference tests (with model caching)

**CI Improvements:**
- Expanded test matrix to run on macOS, Linux, and Windows
- E2E model inference tests on every push
- Model caching to reduce CI bandwidth
- Tests run with `--test-threads=1` to prevent env var race conditions

**Documentation:**
- `docs/testing-strategy.md` - Comprehensive testing documentation
- `docs/ADR/006-cross-platform-testing-strategy.md` - Architecture decision

### HuggingFace Integration (Jan 31)

Added automatic model download and management:

- **Model Repository**: Published to `animeshkundu/fix`
- **Auto-download**: CLI downloads model on first use if not present
- **Dynamic model list**: `--list-models` queries HuggingFace API for available models
- **Persistent config**: `--use-model` downloads and sets default (saved to config.json)
- **Cross-platform paths**: Uses `dirs` crate for platform-appropriate config locations
- **Progress bar**: Download progress with `indicatif` crate

New CLI flags:
- `--list-models` - Query available models from HuggingFace
- `--use-model <name>` - Download and set as default
- `--show-config` - Display current configuration
- `--update` - Force re-download current model

Dependencies added:
- `reqwest` (blocking, rustls-tls) - HTTP client
- `indicatif` - Progress bar
- `serde`, `serde_json` - Config serialization

### Cross-Platform Support (Jan 31)

- Cargo features for GPU backends: `metal`, `cuda`
- Platform-specific stderr redirection (`libc` on Unix only)
- Config paths via `dirs` crate (macOS, Linux, Windows)

### Initial Rust CLI (Jan 2025)

Ported inference from Python to native Rust:

- **llama-cpp-2**: Rust bindings for llama.cpp
- **Metal GPU**: Apple Silicon acceleration (99 GPU layers default)
- **Sub-100ms latency**: Fast inference for interactive use
- **Shell detection**: bash, zsh, fish, powershell, cmd, tcsh
- **Log suppression**: Clean output by disabling llama.cpp logs

Key design decisions:
- GGUF model format (see ADR-001)
- Metal as primary GPU backend (see ADR-002)
- Cross-platform architecture (see ADR-003)

## Model Timeline

| Date | Model | Size | Notes |
|------|-------|------|-------|
| Jan 31, 2025 | qwen3-correct-1.7B | ~1.0 GB | Q4_K_M quantized, larger model option |
| Jan 2025 | qwen3-correct-0.6B | 378 MB | Q4_K_M quantized, published to HuggingFace |
