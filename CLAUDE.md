# CLAUDE.md - AI Context for fix

## Project Overview

**fix** is a CLI that corrects shell command typos using a local LLM. It takes mistyped commands like `gti status` and outputs the correction `git status`.

- **Language**: Rust (2021 edition)
- **Inference**: llama-cpp via llama-cpp-2 crate
- **GPU**: Metal (macOS), CUDA (Linux/Windows)
- **Model**: Fine-tuned Qwen3-0.6B (~378MB GGUF)

## Before You Start

**Read the agent instructions**: `docs/agent-instructions/`

1. [00-core-philosophy.md](docs/agent-instructions/00-core-philosophy.md) - Docs = Code principle
2. [01-research-and-web.md](docs/agent-instructions/01-research-and-web.md) - Research requirements
3. [02-testing-and-validation.md](docs/agent-instructions/02-testing-and-validation.md) - Testing standards
4. [03-tooling-and-pipelines.md](docs/agent-instructions/03-tooling-and-pipelines.md) - CI/CD

## Directory Structure

```
fix-cli/           # Rust CLI (main codebase)
├── src/main.rs    # All CLI logic (~565 lines)
└── Cargo.toml     # Dependencies

docs/              # Documentation
├── ADR/           # Architecture decisions (check before changes)
├── agent-instructions/  # AI agent protocols
├── specs/         # Technical specifications
├── ARCHITECTURE.md
├── HANDOFF.md
└── HISTORY.md

website/           # GitHub Pages site
scripts/           # Automation scripts
```

## Development Commands

```bash
# Build (macOS with Metal GPU)
cd fix-cli && cargo build --release --features metal

# Build (CPU-only)
cd fix-cli && cargo build --release

# Run tests
cd fix-cli && cargo test

# Format code
cd fix-cli && cargo fmt

# Lint
cd fix-cli && cargo clippy

# Run all validation
./scripts/validate.sh

# Test the CLI
cd fix-cli && cargo run -- "gti status"
```

## Code Style

- **Format**: Always run `cargo fmt` before committing
- **Linting**: No clippy warnings (`cargo clippy -- -D warnings`)
- **Naming**: snake_case for functions/variables, PascalCase for types
- **Errors**: Use `anyhow` for error handling with context
- **Comments**: Only where logic isn't self-evident

## Key Files to Know

| File | Purpose |
|------|---------|
| `fix-cli/src/main.rs` | All CLI implementation |
| `fix-cli/Cargo.toml` | Dependencies and features |
| `docs/ADR/` | Past architectural decisions |
| `.github/workflows/` | CI, release, pages workflows |

## Boundaries

### Always Do
- Run `cargo fmt` before committing
- Run `./scripts/validate.sh` before pushing
- Update documentation when changing functionality
- Check `docs/ADR/` before architectural changes

### Ask First
- Adding new dependencies to Cargo.toml
- Changing CLI flags or public behavior
- Modifying GitHub Actions workflows
- Breaking changes to configuration

### Never Do
- Commit secrets, API keys, or tokens
- Force push to main
- Delete ADRs (amend with dated additions instead)
- Skip tests for "small" changes

## Testing Requirements

- Write tests for new functionality
- Run full test suite before committing
- Target 90% coverage for new code
- Test edge cases (empty input, special characters)

## Workflow

1. Read relevant ADRs and existing code
2. Create spec in `docs/specs/` if adding features
3. Implement with tests
4. Run `./scripts/validate.sh`
5. Update `docs/HISTORY.md` if significant change
6. Commit with descriptive message

## Links

- **Repo**: [github.com/animeshkundu/fix](https://github.com/animeshkundu/fix)
- **Model**: [huggingface.co/animeshkundu/cmd-correct](https://huggingface.co/animeshkundu/cmd-correct)
- **Website**: [animeshkundu.github.io/fix](https://animeshkundu.github.io/fix/)
- **Full docs**: See `docs/README.md`
