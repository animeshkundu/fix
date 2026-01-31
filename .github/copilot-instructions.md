# GitHub Copilot Instructions for fix

## Project Overview

**fix** is a Rust CLI that corrects shell command typos using a local LLM.

Example: `fix "gti status"` → `git status`

## Tech Stack

- **Language**: Rust 2021 edition
- **Inference**: llama-cpp-2 crate (llama.cpp bindings)
- **GPU**: Metal (macOS), CUDA (Linux/Windows)
- **Model**: Qwen3-0.6B fine-tuned, GGUF format (~378MB)
- **CLI**: clap for argument parsing
- **Config**: serde + JSON for persistent settings

## Project Structure

```
fix-cli/src/main.rs  # All CLI logic (~565 lines)
fix-cli/Cargo.toml   # Dependencies
docs/                # Documentation
  ADR/               # Architecture Decision Records
  agent-instructions/# AI coding protocols
  specs/             # Technical specifications
website/             # GitHub Pages site
scripts/             # Automation (validate.sh)
```

## Coding Guidelines

### Style
- Format with `cargo fmt`
- No clippy warnings
- snake_case for functions/variables
- PascalCase for types/structs
- Use `anyhow` for error handling

### Patterns
```rust
// Error handling
.context("Failed to do X")?

// CLI arguments (clap derive)
#[derive(Parser)]
struct Args { ... }

// Configuration (serde)
#[derive(Serialize, Deserialize)]
struct Config { ... }
```

## Available Commands

```bash
# Build
cd fix-cli && cargo build --release --features metal

# Test
cd fix-cli && cargo test

# Lint
cd fix-cli && cargo clippy

# Format
cd fix-cli && cargo fmt

# Full validation
./scripts/validate.sh
```

## Before Making Changes

1. Check `docs/ADR/` for past architectural decisions
2. Read `docs/agent-instructions/` for protocols
3. Create a spec in `docs/specs/` for new features
4. Run `./scripts/validate.sh` before committing

## Key Decisions (from ADRs)

- **ADR-001**: GGUF model format for cross-platform inference
- **ADR-002**: Metal GPU for macOS acceleration
- **ADR-003**: Cross-platform support (macOS, Linux, Windows)
- **ADR-004**: Rust for native CLI performance
- **ADR-005**: HuggingFace Hub for model distribution

## Testing

- Write unit tests for new functions
- Test in `#[cfg(test)]` modules
- Run `cargo test` before commits
- Target 90% coverage for new code

## CI/CD Workflows

| Workflow | Purpose |
|----------|---------|
| ci.yml | Format, lint, build, test |
| release.yml | Cross-platform release builds |
| pages.yml | Deploy website |

## Don't

- Add dependencies without justification
- Skip formatting or linting
- Commit secrets or API keys
- Force push to main
- Delete existing ADRs
