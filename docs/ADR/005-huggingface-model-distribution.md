# ADR-005: HuggingFace Model Distribution

## Status
Accepted

## Context

Users need easy access to trained models without manual download and configuration. Options:
1. **Bundle with binary** - Include model in release
2. **GitHub Releases** - Attach model to releases
3. **HuggingFace Hub** - Use HF's model hosting
4. **Custom CDN** - Self-hosted distribution

## Decision

Use **HuggingFace Hub** for model distribution with automatic download in the CLI.

Repository: `animeshkundu/cmd-correct`

## Rationale

1. **Standard location**: Developers expect models on HuggingFace
2. **API available**: Can query available models dynamically
3. **CDN included**: Fast downloads worldwide
4. **Version control**: Can add new models without CLI updates
5. **No bundling needed**: Keeps binary small

## Consequences

### Positive
- CLI stays small (~5MB binary, no bundled model)
- Users can choose which model to download
- New models available without CLI update
- Standard tooling (huggingface-cli) works

### Negative
- Requires internet for first use
- HuggingFace dependency (could go down)
- Need to handle download failures gracefully

## Implementation

### Model Naming Convention
```
qwen3-correct-{size}.gguf
```
Examples:
- `qwen3-correct-0.6B.gguf` (378 MB)
- `qwen3-correct-1.7B.gguf` (future)

### API Endpoints

List models:
```
GET https://huggingface.co/api/models/animeshkundu/cmd-correct/tree/main
```

Download model:
```
GET https://huggingface.co/{repo}/resolve/main/{filename}
```

### CLI Integration

```rust
const HF_REPO: &str = "animeshkundu/cmd-correct";
const DEFAULT_MODEL: &str = "qwen3-correct-0.6B";

// Dynamic model list
fn fetch_available_models() -> Result<Vec<AvailableModel>, String> {
    let url = format!("https://huggingface.co/api/models/{}/tree/main", HF_REPO);
    // Query API, filter for .gguf files
}

// Download with progress bar
fn download_model(model_name: &str) -> Result<PathBuf, String> {
    let url = format!("https://huggingface.co/{}/resolve/main/{}.gguf", HF_REPO, model_name);
    // Download to config_dir(), show progress
}
```

### User Experience

```bash
# First run - auto-downloads default model
$ cmd-correct "gti status"
Downloading qwen3-correct-0.6B...
[========================================] 378 MB / 378 MB
git status

# List available models
$ cmd-correct --list-models
Available models:
  qwen3-correct-0.6B  (378 MB) [current]

# Switch models (persistent)
$ cmd-correct --use-model qwen3-correct-1.7B
```
