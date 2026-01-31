# Architecture Overview

## System Design

fix is a native Rust CLI that corrects shell commands using local LLM inference.

```
┌─────────────────────────────────────────────────────────────────┐
│                        User Shell                               │
│  $ gti status                                                   │
│  command not found: gti                                         │
│  $ fuck                        # Shell function calls fix│
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      fix CLI                            │
├─────────────────────────────────────────────────────────────────┤
│  - Shell detection (SHELL env, platform detection)             │
│  - Prompt formatting (ChatML format)                           │
│  - Model path discovery (cwd, config dirs, custom path)        │
│  - GPU layer configuration                                      │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    llama-cpp-2 Backend                          │
├─────────────────────────────────────────────────────────────────┤
│  - GGUF model loading                                          │
│  - Metal GPU acceleration (Apple Silicon)                      │
│  - Token-by-token generation                                   │
│  - Greedy sampling for deterministic output                    │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      GGUF Model                                 │
├─────────────────────────────────────────────────────────────────┤
│  Base: Qwen2.5-0.5B-Instruct                                   │
│  Format: Q4_K_M quantization (~378MB)                          │
│  Training: ~150k synthetic shell command examples              │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Output                                     │
│  git status                                                     │
└─────────────────────────────────────────────────────────────────┘
```

## Directory Structure

```
fix/
├── fix-cli/              # Rust native CLI
│   ├── src/main.rs              # CLI implementation
│   ├── Cargo.toml               # Dependencies
│   └── Cargo.lock               # Locked versions
├── docs/                         # Documentation
│   ├── ARCHITECTURE.md          # This file
│   └── ADR/                     # Architecture Decision Records
│       └── 004-rust-cli-implementation.md
├── AGENTS.md                     # AI agent guidelines
├── README.md                     # User documentation
└── .gitignore
```

## Inference Flow

```
User Input          Prompt Formatting       Model Inference         Output
    │                     │                       │                    │
    ▼                     ▼                       ▼                    ▼
"gti status"  →   <|im_start|>system      →   Load GGUF        →  "git status"
    +               You are a shell           model with
  "bash"            command corrector...      Metal GPU
    +               <|im_end|>                    │
  (error)           <|im_start|>user              ▼
                    Shell: bash               Tokenize
                    Command: gti status       prompt
                    <|im_end|>                    │
                    <|im_start|>assistant         ▼
                                              Generate
                                              tokens
                                              (greedy)
                                                  │
                                                  ▼
                                              Stop at
                                              EOS/newline
```

## Key Design Decisions

### Why Rust + llama-cpp?
- Single binary distribution (no Python/Node runtime)
- Fast startup time (~100ms including model load)
- Metal GPU acceleration on Apple Silicon
- Cross-platform support via llama.cpp

### Why Local Inference?
- Privacy: commands never leave the machine
- Speed: no network latency
- Offline: works without internet
- Cost: no API fees

### Prompt Format (ChatML)
```
<|im_start|>system
You are a shell command corrector. Output only the corrected command./no_think
<|im_end|>
<|im_start|>user
Shell: bash
Command: gti status
Error: command not found: gti
<|im_end|>
<|im_start|>assistant
git status
```

## Supported Shells

| Shell | Detection | Platform |
|-------|-----------|----------|
| bash | SHELL env | Linux/macOS |
| zsh | SHELL env | macOS/Linux |
| fish | SHELL env | Cross-platform |
| powershell | PSModulePath env | Windows/Cross |
| cmd | COMSPEC env | Windows |
| tcsh | SHELL env | BSD/Linux |

## Model Discovery & Download

The CLI uses this flow to find or download models:

```
                    ┌─────────────────────────┐
                    │   --model flag set?     │
                    └───────────┬─────────────┘
                                │
                    ┌───────────▼─────────────┐
                    │  Yes: Use specified path │
                    │  No: Check config dir    │
                    └───────────┬─────────────┘
                                │
                    ┌───────────▼─────────────┐
                    │ Model exists locally?    │
                    └───────────┬─────────────┘
                         │             │
                        Yes           No
                         │             │
                         ▼             ▼
                    ┌─────────┐  ┌──────────────────┐
                    │  Load   │  │ Download from HF │
                    │  model  │  │ with progress bar│
                    └─────────┘  └──────────────────┘
```

### Config Directory (Cross-Platform)

| Platform | Path |
|----------|------|
| macOS | `~/Library/Application Support/fix/` |
| Linux | `~/.config/fix/` |
| Windows | `%APPDATA%\fix\` |

### Config File Structure

`config.json`:
```json
{
  "default_model": "qwen3-correct-0.6B"
}
```

### HuggingFace Integration

**Repository**: `animeshkundu/fix`

```
                    ┌─────────────────────────┐
                    │    --list-models        │
                    └───────────┬─────────────┘
                                │
                    ┌───────────▼─────────────┐
                    │ GET /api/models/{repo}/ │
                    │     tree/main           │
                    └───────────┬─────────────┘
                                │
                    ┌───────────▼─────────────┐
                    │ Filter for .gguf files  │
                    │ Display with sizes      │
                    └─────────────────────────┘


                    ┌─────────────────────────┐
                    │ --use-model <name>      │
                    └───────────┬─────────────┘
                                │
                    ┌───────────▼─────────────┐
                    │ Validate model exists   │
                    │ in HF repo              │
                    └───────────┬─────────────┘
                                │
                    ┌───────────▼─────────────┐
                    │ Download with progress  │
                    │ to config_dir()         │
                    └───────────┬─────────────┘
                                │
                    ┌───────────▼─────────────┐
                    │ Update config.json      │
                    │ default_model = <name>  │
                    └─────────────────────────┘
```
