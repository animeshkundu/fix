# Documentation Index

This directory contains all project documentation for the `fix` CLI.

## Quick Navigation

| Document | Purpose |
|----------|---------|
| [ARCHITECTURE.md](./ARCHITECTURE.md) | System design and diagrams |
| [HANDOFF.md](./HANDOFF.md) | Developer onboarding guide |
| [HISTORY.md](./HISTORY.md) | Development timeline and changes |

## For AI Agents

**Start here**: [agent-instructions/](./agent-instructions/)

Read the instruction files in order:
1. [00-core-philosophy.md](./agent-instructions/00-core-philosophy.md) - Fundamental principles
2. [01-research-and-web.md](./agent-instructions/01-research-and-web.md) - Research guidelines
3. [02-testing-and-validation.md](./agent-instructions/02-testing-and-validation.md) - Testing requirements
4. [03-tooling-and-pipelines.md](./agent-instructions/03-tooling-and-pipelines.md) - CI/CD and automation

## Architecture Decisions

See [ADR/](./ADR/) for all architecture decision records:

| ADR | Title |
|-----|-------|
| [001](./ADR/001-gguf-model-format.md) | GGUF Model Format |
| [002](./ADR/002-metal-gpu-acceleration.md) | Metal GPU Acceleration |
| [003](./ADR/003-cross-platform-support.md) | Cross-Platform Support |
| [004](./ADR/004-rust-cli-implementation.md) | Rust CLI Implementation |
| [005](./ADR/005-huggingface-model-distribution.md) | HuggingFace Model Distribution |

Use [000-template.md](./ADR/000-template.md) for new ADRs.

## Technical Specifications

See [specs/](./specs/) for feature specifications.

Write a spec before implementing significant features.

## Directory Structure

```
docs/
├── README.md                 # This file
├── ARCHITECTURE.md           # System design
├── HANDOFF.md                # Developer guide
├── HISTORY.md                # Changelog
├── ADR/                      # Architecture Decision Records
│   ├── 000-template.md
│   └── 001-005...
├── agent-instructions/       # AI agent protocols
│   ├── 00-core-philosophy.md
│   ├── 01-research-and-web.md
│   ├── 02-testing-and-validation.md
│   └── 03-tooling-and-pipelines.md
└── specs/                    # Technical specifications
    └── README.md
```

## Related Files

- [AGENTS.md](../AGENTS.md) - Repository map for AI navigation
- [CLAUDE.md](../CLAUDE.md) - AI context and commands
- [README.md](../README.md) - User-facing documentation
