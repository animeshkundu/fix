# ADR 006: Cross-Platform Testing Strategy

## Status

Accepted

## Date

2025-01-31

## Context

The `fix` CLI supports multiple platforms (macOS, Linux, Windows) and shells (bash, zsh, fish, PowerShell, CMD, tcsh). Prior to this decision:

- Tests only ran on Ubuntu in CI, despite building on all platforms
- No testing for WSL (Windows Subsystem for Linux), which has unique path handling
- PowerShell and CMD were underserved in testing
- Installation scripts (install.sh, install.ps1) were untested
- No end-to-end model inference testing in CI
- No Linux distribution compatibility testing (glibc versions, musl)

This created risk of platform-specific bugs going undetected until users reported them.

## Decision

We will implement a comprehensive 5-layer testing strategy using GitHub Actions:

### Layer 1: Unit Tests
- Existing tests in `fix-cli/src/main.rs`
- Run on all platforms with `--test-threads=1` (prevents env var race conditions)

### Layer 2: Integration Tests
- New `fix-cli/tests/` directory with:
  - `cli_test.rs` - Binary execution tests
  - `config_test.rs` - Cross-platform config path tests
  - `wsl_test.rs` - WSL-specific environment tests
  - `e2e_test.rs` - Model inference tests (marked `#[ignore]`)

### Layer 3: Shell Integration Tests
- Dedicated workflows for:
  - WSL testing (`.github/workflows/test-wsl.yml`)
  - PowerShell/CMD testing (`.github/workflows/test-windows-shells.yml`)
  - Shell detection across bash, zsh, fish, tcsh

### Layer 4: Installation Tests
- Test `install.sh` on Linux, macOS, WSL
- Test `install.ps1` on Windows
- Verify PATH modification, directory creation, script syntax

### Layer 5: E2E & Distribution Tests
- Download model and run actual inference on every push
- Test on Alpine (musl), Debian (glibc 2.31), Ubuntu 20.04, Fedora
- Cache model between CI runs

### Priority Matrix

| Priority | Environment | Rationale |
|----------|-------------|-----------|
| Critical | WSL, PowerShell, CMD | Underserved, unique edge cases |
| High | bash, zsh, macOS, Ubuntu | Primary user base |
| Medium | fish, Alpine, Debian | Growing/niche but important |
| Low | tcsh | Minimal user base |

## Consequences

### Positive
- Platform-specific bugs caught before release
- Confidence in installation scripts
- WSL users get tested experience
- PowerShell/CMD users get first-class support
- Binary portability verified across glibc versions

### Negative
- CI run time increases (~8-10 minutes vs ~3 minutes)
- Model download adds bandwidth usage (mitigated by caching)
- More workflows to maintain (6 workflow files)

### Neutral
- Tests must run with `--test-threads=1` due to environment variable manipulation

## Alternatives Considered

### Alternative 1: Self-Hosted Runners
- Description: Use self-hosted runners for full control
- Pros: Cheaper for high volume, full customization
- Cons: March 2026 GitHub pricing changes, maintenance burden
- Why rejected: Overkill for public project with free CI minutes

### Alternative 2: Docker-Only Testing
- Description: Test all platforms via Docker containers
- Pros: Consistent environments, cheaper
- Cons: Can't test macOS Metal, heavy for simple CLI
- Why rejected: Doesn't cover macOS GPU features, missing Windows testing

### Alternative 3: Manual Testing Matrix
- Description: Document manual testing procedures
- Pros: No CI complexity
- Cons: Human error, time consuming, not scalable
- Why rejected: Doesn't meet automation goals

## Implementation Notes

### Workflow Files
- `.github/workflows/ci.yml` - Core CI with E2E tests
- `.github/workflows/test-wsl.yml` - WSL-specific
- `.github/workflows/test-windows-shells.yml` - PowerShell/CMD
- `.github/workflows/test-install.yml` - Installation scripts
- `.github/workflows/test-distros.yml` - Linux distributions

### Test Environment Requirements
- WSL tests use `Vampire/setup-wsl@v3` action
- Distro tests require cmake for llama-cpp-2 build
- E2E tests cache model at `~/.config/fix/`

### Running Tests Locally
```bash
# Quick tests
cargo test -- --test-threads=1

# E2E tests (requires model)
cargo test --test e2e_test -- --ignored --test-threads=1
```

## References

- [docs/testing-strategy.md](../testing-strategy.md) - Detailed testing documentation
- [ADR-003](003-cross-platform-support.md) - Cross-platform architecture
- [GitHub Actions Matrix Builds](https://docs.github.com/en/actions/using-jobs/using-a-matrix-for-your-jobs)

---

## Amendment Log

| Date | Change |
|------|--------|
| 2025-01-31 | Initial decision |
