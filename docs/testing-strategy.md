# Testing Strategy

This document describes the comprehensive testing strategy for the `fix` CLI tool, covering cross-platform testing, shell integration verification, and end-to-end model inference testing.

## Testing Layers

### Layer 1: Unit Tests (Existing)

Location: `fix-cli/src/main.rs`

Unit tests cover core logic without external dependencies:

| Category | Coverage |
|----------|----------|
| Config defaults | DEFAULT_MODEL, HF_REPO constants |
| Shell detection | Bash, Zsh, Fish, PowerShell, CMD, tcsh |
| Prompt building | ChatML format, shell variations, special chars |
| Path functions | config_dir, config_path, model_path |
| Serialization | Config JSON roundtrip |

Run with:
```bash
cargo test
```

### Layer 2: Integration Tests

Location: `fix-cli/tests/`

| File | Purpose |
|------|---------|
| `cli_test.rs` | Binary execution, flags, error handling |
| `config_test.rs` | Cross-platform config paths, file operations |
| `wsl_test.rs` | WSL-specific path and environment handling |
| `e2e_test.rs` | Model inference tests (requires model download) |

Run with:
```bash
# Standard integration tests
cargo test --test cli_test
cargo test --test config_test
cargo test --test wsl_test

# E2E tests (require model)
cargo test --test e2e_test -- --ignored
```

### Layer 3: Shell Integration Tests

Verifies shell wrapper functions work correctly across shells.

Tested in CI via:
- `.github/workflows/test-windows-shells.yml` (PowerShell, CMD)
- `.github/workflows/test-wsl.yml` (WSL bash)
- `.github/workflows/test-distros.yml` (bash, zsh, fish, tcsh)

### Layer 4: Installation Tests

Location: `.github/workflows/test-install.yml`

Tests the installation scripts:
- `website/install.sh` (Linux, macOS, WSL)
- `website/install.ps1` (Windows)

Verified:
- Script syntax
- OS/architecture detection
- Directory creation
- PATH modification
- Shell integration function syntax

### Layer 5: End-to-End Model Tests

Tests actual model inference in CI:
- Downloads model from HuggingFace
- Runs inference tests
- Verifies output format and correctness
- Model is cached between CI runs

## Platform Matrix

| Platform | Runner | Shell(s) Tested | Priority |
|----------|--------|-----------------|----------|
| macOS (Apple Silicon) | macos-14 | zsh | High |
| Ubuntu | ubuntu-latest | bash | High |
| Windows | windows-latest | PowerShell, CMD | Critical |
| WSL | windows-latest + WSL | bash | Critical |
| Alpine | container | ash/bash | Medium |
| Debian | container | bash | Medium |

## Shell Testing Matrix

| Shell | Priority | Test Platform | Detection Method |
|-------|----------|---------------|------------------|
| PowerShell | Critical | windows-latest | PSModulePath |
| CMD | Critical | windows-latest | Platform fallback |
| WSL bash | Critical | windows-latest + WSL | SHELL env |
| bash | High | ubuntu-latest | SHELL env |
| zsh | High | macos-14 | SHELL env |
| fish | Medium | ubuntu-latest | SHELL env |
| tcsh | Low | ubuntu-latest | SHELL env |

## CI Workflows

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `ci.yml` | push, PR | Build, test, E2E on all platforms |
| `test-wsl.yml` | push, PR | WSL-specific testing |
| `test-windows-shells.yml` | push, PR | PowerShell/CMD deep testing |
| `test-install.yml` | push, PR | Installation script testing |
| `test-distros.yml` | push, PR | Linux distribution compatibility |

## Running Tests Locally

### Quick Tests
```bash
cd fix-cli
cargo test
```

### Full Test Suite
```bash
cd fix-cli
cargo test --all-features
```

### E2E Tests (requires model)
```bash
cd fix-cli
# Download model first
cargo run -- --update

# Run E2E tests
cargo test --test e2e_test -- --ignored
```

### Platform-Specific Tests

**macOS:**
```bash
cargo test --features metal
```

**Windows (PowerShell):**
```powershell
cargo test
```

**Windows (CMD):**
```cmd
cargo test
```

## Test Coverage Goals

| Category | Target | Current |
|----------|--------|---------|
| Unit tests | 90%+ | Good |
| Integration tests | 80%+ | New |
| Shell detection | All 6 shells | Complete |
| Platform tests | 3 OS + WSL | Complete |
| E2E inference | Every push | Complete |

## Adding New Tests

### Unit Tests
Add to `fix-cli/src/main.rs` in the `#[cfg(test)]` module:
```rust
#[test]
fn test_new_feature() {
    // Test implementation
}
```

### Integration Tests
Create new file in `fix-cli/tests/`:
```rust
// fix-cli/tests/new_test.rs

#[test]
fn test_integration_scenario() {
    // Test implementation
}
```

### E2E Tests
Add to `fix-cli/tests/e2e_test.rs`:
```rust
#[test]
#[ignore] // Run with --ignored flag
fn test_e2e_new_scenario() {
    if !binary_exists() || !model_exists() {
        return;
    }
    // Test implementation
}
```

## Troubleshooting

### Test fails on Windows but passes on Unix
- Check for path separator issues (`/` vs `\`)
- Check for line ending issues (LF vs CRLF)
- Verify environment variable handling

### Shell detection returns wrong shell
- Verify `SHELL` environment variable is set correctly
- Check `PSModulePath` for PowerShell detection
- Use `--verbose` flag to see detected shell

### E2E tests fail with "model not found"
- Run `cargo run -- --update` to download model
- Check `~/.config/fix/` (Linux/macOS) or `%APPDATA%\fix\` (Windows)

### WSL tests fail
- Ensure WSL is properly configured in CI
- Check for path translation issues (`/mnt/c/` paths)
- Verify Linux binary is being tested, not Windows binary
