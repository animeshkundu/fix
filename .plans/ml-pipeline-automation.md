# Automated ML Pipeline for oops-llm-training

> **Note**: This plan is **ADDITIVE** to the existing wit implementation plan at:
> `/Users/ani/Code/fine-tuning/.plans/fix-wit-two-binary-architecture.md`
>
> **Existing plan covers (DO NOT DUPLICATE):**
> - Two-binary architecture (fix + wit)
> - Training data generation (DS5 - 100K examples)
> - CLI implementation (tools, agent loop, progress)
> - Model training strategy & hyperparameters
> - Multi-agent orchestration with Copilot
>
> **This plan adds (NEW):**
> - GitHub Actions workflows for automation
> - Self-hosted runner setup for MLX
> - Automated GGUF conversion pipeline
> - Model quality validation (perplexity, exact match)
> - Automated HuggingFace upload
>
> **No overlap**: Training scripts, data generation, and CLI code are unchanged.

---

## Overview

Design and implement an automated CI/CD pipeline for the complete ML workflow:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                        AUTOMATED ML PIPELINE                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  [1] Dataset         [2] Training      [3] GGUF          [4] Quantize      │
│      Generation          (MLX)             Convert           (Q4_K_M)      │
│      ─────────────>  ─────────────>   ─────────────>    ─────────────>     │
│                                                                             │
│  [5] Validate        [6] Test          [7] Upload                          │
│      (Perplexity)        (Inference)       (HuggingFace)                   │
│      ─────────────>  ─────────────>   ─────────────>    ✓ RELEASE         │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Research Summary

### GitHub Actions for Apple Silicon ML

| Option | Pros | Cons |
|--------|------|------|
| [Self-hosted M1/M2 runner](https://github.blog/changelog/2022-08-09-github-actions-self-hosted-runners-now-support-apple-m1-hardware/) | Full control, always available | Requires hardware setup |
| [GitHub-hosted M1 larger runner](https://github.blog/news-insights/product-news/introducing-the-new-apple-silicon-powered-m1-macos-larger-runner-for-github-actions/) | No setup, managed | Higher cost tier |
| Linux CPU fallback | Free tier | Very slow, no Metal |

**Recommendation**: Self-hosted M1/M2 runner for MLX performance, with Linux fallback for non-training tasks.

### Reference: IBM's GGUF Pipeline

[IBM/gguf](https://github.com/IBM/gguf) implements a production CI/CD pipeline:
- Converts HuggingFace → GGUF
- Quantizes with multiple formats
- Runs build-verification tests
- Three release tiers: Test → Preview → Public

### Model Quality Metrics

| Metric | Purpose | Tool | Threshold |
|--------|---------|------|-----------|
| [Perplexity](https://www.comet.com/site/blog/perplexity-for-llm-evaluation/) | Prediction uncertainty | llama-perplexity | < baseline + 5% |
| Exact Match | Task accuracy | pytest | > 85% |
| KL Divergence | Quantization loss | llama-quantize | < 0.01 |
| Inference Time | Latency | benchmark | < 500ms |

### HuggingFace Upload

- [huggingface_hub](https://github.com/huggingface/huggingface_hub) for programmatic upload
- [Hugging Push Action](https://github.com/marketplace/actions/hugging-push) for CI/CD
- Store `HF_TOKEN` as GitHub secret

---

## Current State (oops-llm-training)

### What Exists
- ✅ Data generation scripts (`scripts/generate_data.py`) - 180K examples
- ✅ Training scripts (`src/training/train.py`) - MLX + PyTorch
- ✅ YAML configs (`configs/train_qwen3*.yaml`)
- ✅ Dataset validation (`scripts/validate_dataset.py`)
- ❌ No GitHub Actions workflows
- ❌ No automated GGUF conversion
- ❌ No automated model testing
- ❌ No automated HuggingFace upload

### Key Files
```
oops-llm-training/
├── scripts/generate_data.py      # Dataset orchestrator
├── src/training/train.py         # MLX/PyTorch trainer
├── src/training/prepare_dataset.py
├── configs/train_qwen3_1.7b.yaml # Training config
└── .github/workflows/            # TO CREATE
```

---

## Proposed Architecture

### Workflow Triggers

| Workflow | Trigger | Purpose |
|----------|---------|---------|
| `data-validation.yml` | PR to `data/` | Validate dataset changes |
| `train-model.yml` | Manual / Tag | Train model on self-hosted runner |
| `convert-quantize.yml` | After training | GGUF + quantization |
| `test-model.yml` | After quantize | Quality validation |
| `release-model.yml` | After tests pass | Upload to HuggingFace |

### Self-Hosted Runner Setup

```yaml
# Required labels for the runner
runs-on: [self-hosted, macOS, ARM64, mlx]
```

Runner requirements:
- macOS 13+ on M1/M2/M3
- 32GB+ RAM recommended
- Python 3.11+ with MLX installed
- llama.cpp built locally

---

## Implementation Plan

### Phase 1: Infrastructure Setup

**1.1 Self-Hosted Runner**
- Configure M1/M2 Mac as GitHub Actions runner
- Install dependencies: Python, MLX, llama.cpp
- Register with `animeshkundu/oops-llm-training`

**1.2 GitHub Secrets**
```
HF_TOKEN          # HuggingFace write token
ANTHROPIC_API_KEY # For self-instruct generation (optional)
```

**1.3 Reusable Workflow Components**
```
.github/
├── workflows/
│   ├── data-validation.yml    # PR validation
│   ├── train-model.yml        # Training workflow
│   ├── convert-quantize.yml   # GGUF pipeline
│   ├── test-model.yml         # Quality tests
│   └── release-model.yml      # HF upload
├── actions/
│   ├── setup-mlx/             # Install MLX environment
│   ├── setup-llama-cpp/       # Build llama.cpp
│   └── validate-gguf/         # Test GGUF model
└── scripts/
    ├── run_perplexity.py      # Perplexity benchmark
    ├── run_inference_tests.py # Exact match tests
    └── upload_to_hf.py        # HuggingFace upload
```

### Phase 2: Dataset Validation Workflow

```yaml
# .github/workflows/data-validation.yml
name: Validate Dataset

on:
  pull_request:
    paths: ['data/**', 'src/data_generation/**']

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: '3.11'

      - name: Install dependencies
        run: pip install -r requirements.txt

      - name: Validate dataset format
        run: python scripts/validate_dataset.py

      - name: Check shell distribution
        run: python -c "from scripts.validate_dataset import check_distribution; check_distribution()"

      - name: Run data generation tests
        run: pytest tests/test_*_generator.py -v
```

### Phase 3: Training Workflow

```yaml
# .github/workflows/train-model.yml
name: Train Model

on:
  workflow_dispatch:
    inputs:
      model_name:
        description: 'Model to train (qwen3-0.6b, qwen3-1.7b)'
        required: true
        default: 'qwen3-1.7b'
      config:
        description: 'Training config file'
        required: true
        default: 'configs/train_qwen3_1.7b.yaml'

jobs:
  train:
    runs-on: [self-hosted, macOS, ARM64, mlx]
    timeout-minutes: 360  # 6 hours max

    steps:
      - uses: actions/checkout@v4

      - name: Setup Python
        run: |
          python3 -m venv .venv
          source .venv/bin/activate
          pip install -r requirements.txt

      - name: Generate dataset (if needed)
        run: |
          source .venv/bin/activate
          python scripts/generate_data.py --all

      - name: Prepare training data
        run: |
          source .venv/bin/activate
          python src/training/prepare_dataset.py \
            --input data/generated \
            --output data/final

      - name: Train model
        run: |
          source .venv/bin/activate
          python src/training/train.py \
            --config ${{ inputs.config }} \
            --backend mlx

      - name: Fuse LoRA adapters
        run: |
          source .venv/bin/activate
          python -m mlx_lm.fuse \
            --model Qwen/Qwen3-${{ inputs.model_name }} \
            --adapter-path adapters/${{ inputs.model_name }} \
            --save-path models/${{ inputs.model_name }}-fused

      - name: Upload fused model artifact
        uses: actions/upload-artifact@v4
        with:
          name: fused-model-${{ inputs.model_name }}
          path: models/${{ inputs.model_name }}-fused/
          retention-days: 7
```

### Phase 4: GGUF Conversion & Quantization

```yaml
# .github/workflows/convert-quantize.yml
name: Convert and Quantize

on:
  workflow_run:
    workflows: ["Train Model"]
    types: [completed]
  workflow_dispatch:
    inputs:
      model_name:
        required: true
        default: 'qwen3-1.7b'

jobs:
  convert:
    runs-on: [self-hosted, macOS, ARM64, mlx]
    if: ${{ github.event.workflow_run.conclusion == 'success' || github.event_name == 'workflow_dispatch' }}

    steps:
      - uses: actions/checkout@v4

      - name: Download fused model
        uses: actions/download-artifact@v4
        with:
          name: fused-model-${{ inputs.model_name }}
          path: models/${{ inputs.model_name }}-fused/

      - name: Convert to GGUF F16
        run: |
          python ../llama.cpp/convert_hf_to_gguf.py \
            models/${{ inputs.model_name }}-fused \
            --outfile models/${{ inputs.model_name }}-f16.gguf

      - name: Generate importance matrix
        run: |
          ../llama.cpp/build/bin/llama-imatrix \
            -m models/${{ inputs.model_name }}-f16.gguf \
            -f data/final/train.jsonl \
            -o models/imatrix-${{ inputs.model_name }}.dat \
            --chunks 100

      - name: Quantize to Q4_K_M
        run: |
          ../llama.cpp/build/bin/llama-quantize \
            --imatrix models/imatrix-${{ inputs.model_name }}.dat \
            models/${{ inputs.model_name }}-f16.gguf \
            models/${{ inputs.model_name }}-q4km.gguf \
            Q4_K_M

      - name: Upload GGUF artifacts
        uses: actions/upload-artifact@v4
        with:
          name: gguf-${{ inputs.model_name }}
          path: |
            models/${{ inputs.model_name }}-f16.gguf
            models/${{ inputs.model_name }}-q4km.gguf
          retention-days: 7
```

### Phase 5: Model Validation

```yaml
# .github/workflows/test-model.yml
name: Test Model Quality

on:
  workflow_run:
    workflows: ["Convert and Quantize"]
    types: [completed]

jobs:
  test:
    runs-on: [self-hosted, macOS, ARM64, mlx]

    steps:
      - uses: actions/checkout@v4

      - name: Download GGUF models
        uses: actions/download-artifact@v4
        with:
          name: gguf-${{ inputs.model_name }}
          path: models/

      - name: Perplexity test
        id: perplexity
        run: |
          PPL=$(../llama.cpp/build/bin/llama-perplexity \
            -m models/${{ inputs.model_name }}-q4km.gguf \
            -f data/test_prompts.txt \
            --chunks 50 | tail -1 | awk '{print $NF}')
          echo "perplexity=$PPL" >> $GITHUB_OUTPUT
          if (( $(echo "$PPL > 10.0" | bc -l) )); then
            echo "::error::Perplexity too high: $PPL"
            exit 1
          fi

      - name: Inference tests
        run: |
          python .github/scripts/run_inference_tests.py \
            --model models/${{ inputs.model_name }}-q4km.gguf \
            --test-file tests/inference_test_cases.json \
            --threshold 0.85

      - name: Latency benchmark
        run: |
          python .github/scripts/benchmark_latency.py \
            --model models/${{ inputs.model_name }}-q4km.gguf \
            --samples 100 \
            --max-latency-ms 500

      - name: Create test report
        if: always()
        run: |
          echo "## Model Quality Report" >> $GITHUB_STEP_SUMMARY
          echo "- Perplexity: ${{ steps.perplexity.outputs.perplexity }}" >> $GITHUB_STEP_SUMMARY
```

### Phase 6: HuggingFace Release

```yaml
# .github/workflows/release-model.yml
name: Release to HuggingFace

on:
  workflow_run:
    workflows: ["Test Model Quality"]
    types: [completed]
  workflow_dispatch:
    inputs:
      model_name:
        required: true
      version:
        required: true
        default: 'v1.0.0'

jobs:
  release:
    runs-on: ubuntu-latest
    if: ${{ github.event.workflow_run.conclusion == 'success' || github.event_name == 'workflow_dispatch' }}

    steps:
      - uses: actions/checkout@v4

      - name: Download GGUF models
        uses: actions/download-artifact@v4
        with:
          name: gguf-${{ inputs.model_name }}
          path: models/

      - name: Install huggingface_hub
        run: pip install huggingface_hub

      - name: Upload to HuggingFace
        env:
          HF_TOKEN: ${{ secrets.HF_TOKEN }}
        run: |
          python .github/scripts/upload_to_hf.py \
            --model-path models/${{ inputs.model_name }}-q4km.gguf \
            --repo-id animeshkundu/cmd-correct \
            --filename ${{ inputs.model_name }}.gguf \
            --commit-message "Release ${{ inputs.version }}"

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          tag_name: ${{ inputs.version }}
          files: |
            models/${{ inputs.model_name }}-q4km.gguf
          body: |
            ## Model Release ${{ inputs.version }}

            - Model: ${{ inputs.model_name }}
            - Format: GGUF Q4_K_M
            - HuggingFace: https://huggingface.co/animeshkundu/cmd-correct
```

---

## GitHub Copilot Agent Integration

### Tasks for Copilot Agents

Copilot agents can handle these medium-complexity tasks:

| Issue | Task | Model |
|-------|------|-------|
| Create test fixtures | Generate inference test cases | Claude Sonnet 4.5 |
| Write validation scripts | Implement perplexity benchmarks | Claude Opus 4.5 |
| Update configs | Modify YAML for new models | Claude Sonnet 4.5 |
| Fix test failures | Debug failing inference tests | Claude Opus 4.5 |

### Copilot-Assisted Workflow

```
1. Create issue: "Add inference test cases for PowerShell commands"
2. Assign to copilot-swe-agent[bot] with model selection
3. Copilot creates PR with test cases
4. CI validates the test format
5. Merge → tests run in next model validation
```

---

## Quality Gates

### Before Training
- [ ] Dataset has 180K+ examples
- [ ] Shell distribution matches target (bash 35%, zsh 25%, etc.)
- [ ] All data validation tests pass

### Before Release
- [ ] Perplexity < 10.0
- [ ] Exact match accuracy > 85%
- [ ] Inference latency < 500ms
- [ ] GGUF file validates with `gguf-parser`
- [ ] No regression from previous model

---

## Files to Create

| File | Purpose |
|------|---------|
| `.github/workflows/data-validation.yml` | PR validation |
| `.github/workflows/train-model.yml` | Training workflow |
| `.github/workflows/convert-quantize.yml` | GGUF conversion |
| `.github/workflows/test-model.yml` | Quality testing |
| `.github/workflows/release-model.yml` | HF upload |
| `.github/scripts/run_inference_tests.py` | Test runner |
| `.github/scripts/upload_to_hf.py` | HF upload script |
| `.github/scripts/benchmark_latency.py` | Latency tests |
| `tests/inference_test_cases.json` | Test fixtures |

---

## Implementation Timeline

**Relationship to Existing wit Plan:**

| Existing Plan (wit) | This Plan (CI/CD) | Dependency |
|---------------------|-------------------|------------|
| Week 1-2: Training data generation | - | - |
| Week 3: Validation pipeline | Week 1: Data validation workflow | Parallel |
| Week 4: CLI implementation | Week 2: Training workflow | After data complete |
| Week 5: Testing | Week 3: Model validation | After training |
| Week 6: Training + Release | Week 4: HuggingFace automation | Enables automation |

**CI/CD Timeline (runs parallel to wit implementation):**

1. **Week 1**: Self-hosted runner setup + data validation workflow
2. **Week 2**: Training workflow + GGUF conversion
3. **Week 3**: Model validation tests + quality gates
4. **Week 4**: HuggingFace release automation + documentation

**After both plans complete:**
- wit model trained via automated pipeline
- Release to HuggingFace happens automatically when tests pass

---

## Verification

### Test the Pipeline

1. **Data validation**: Create PR with dataset changes, verify CI runs
2. **Training**: Manually trigger `train-model.yml`, verify artifacts created
3. **Conversion**: Verify GGUF files are valid
4. **Testing**: Run quality tests, verify thresholds
5. **Release**: Verify model appears on HuggingFace

### Success Criteria

- [ ] End-to-end pipeline runs without manual intervention
- [ ] Model quality is validated automatically
- [ ] Only passing models are released to HuggingFace
- [ ] Pipeline completes in < 8 hours (including training)

---

## Sources

- [GitHub Actions Self-Hosted M1 Runners](https://github.blog/changelog/2022-08-09-github-actions-self-hosted-runners-now-support-apple-m1-hardware/)
- [IBM/gguf CI/CD Pipeline](https://github.com/IBM/gguf)
- [llama.cpp Quantization](https://github.com/ggml-org/llama.cpp/blob/master/tools/quantize/README.md)
- [HuggingFace Hub](https://github.com/huggingface/huggingface_hub)
- [Hugging Push Action](https://github.com/marketplace/actions/hugging-push)
- [LLM Evaluation with Perplexity](https://www.comet.com/site/blog/perplexity-for-llm-evaluation/)
- [GitHub Copilot Coding Agent](https://github.blog/ai-and-ml/github-copilot/github-copilot-coding-agent-101-getting-started-with-agentic-workflows-on-github/)
