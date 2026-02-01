# Automated ML Pipeline for oops-llm-training

> **Note**: This plan is **ADDITIVE** to the existing wit implementation plan at:
> `/Users/ani/Code/fine-tuning/.plans/fix-wit-two-binary-architecture.md`
>
> **This plan adds (NEW):**
> - GitHub Actions for data generation, validation, GGUF conversion
> - **Kaggle** for FREE GPU training (30 hrs/week, T4/P100)
> - Model quality validation (perplexity, exact match)
> - Automated HuggingFace upload
>
> **Key Decision**: No self-hosted runners or paid compute. All training via Kaggle free tier.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                    FREE ML TRAINING PIPELINE                                 │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  GitHub Actions (FREE CPU)              Kaggle (FREE GPU)                   │
│  ─────────────────────────             ─────────────────                    │
│  [1] Data validation         ───────>  [2] LoRA Training (T4/P100)         │
│  [3] Download trained model  <───────       30 GPU hrs/week                 │
│  [4] GGUF conversion (CPU)                                                  │
│  [5] Quantization (Q4_K_M)                                                  │
│  [6] Model testing                                                          │
│  [7] HuggingFace upload                                                     │
│                                                                             │
│  Trigger: Manual (workflow_dispatch)                                        │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## Research Summary

### Why Kaggle for Training

| Option | Free GPU Hours | GPU Type | Verdict |
|--------|---------------|----------|---------|
| GitHub Actions | 0 (CPU only) | None | ❌ Too slow for training |
| HF ZeroGPU | On-demand | H200 | ⚠️ Good for quick runs |
| **Kaggle** | 30 hrs/week | T4/P100 (16GB) | ✅ **Best for LoRA** |
| Modal | $30 credit | Various | ⚠️ Limited free tier |
| Google Colab | ~20 hrs/week | T4 | ⚠️ Poor automation |

**Kaggle advantages:**
- 30 GPU hours/week (enough for 5-6 training runs)
- T4/P100 with 16GB VRAM (sufficient for 4B LoRA)
- API automation via `kaggle kernels push`
- 9-hour session limit (plenty for LoRA fine-tuning)
- Notebooks can run in background

### GitHub Actions Free Tier (for non-training tasks)

| Spec | Value |
|------|-------|
| vCPU | 4 |
| RAM | 16 GB |
| Disk | 14 GB |
| Timeout | 6 hours |
| GPU | None |

**Good for**: Data generation, GGUF conversion, quantization, testing, uploads

### Model Resource Requirements

| Model | LoRA Memory | Kaggle T4 (16GB) | Training Time |
|-------|-------------|------------------|---------------|
| 0.6B | ~3 GB | ✅ Easy | ~1-2 hours |
| 1.7B | ~7 GB | ✅ Good | ~3-4 hours |
| 4B | ~10 GB | ✅ Fits | ~5-6 hours |

### Quality Metrics

| Metric | Tool | Threshold |
|--------|------|-----------|
| Perplexity | llama-perplexity | < baseline + 5% |
| Exact Match | pytest | > 85% |
| Inference Latency | benchmark | < 500ms |

---

## Implementation Plan

### File Structure

```
oops-llm-training/
├── .github/
│   ├── workflows/
│   │   ├── data-validation.yml     # PR validation (GitHub CPU)
│   │   ├── trigger-training.yml    # Upload data to Kaggle, trigger notebook
│   │   ├── post-training.yml       # GGUF conversion + quantization (GitHub CPU)
│   │   ├── test-model.yml          # Quality validation (GitHub CPU)
│   │   └── release-model.yml       # HuggingFace upload
│   └── scripts/
│       ├── upload_to_kaggle.py     # Push dataset to Kaggle
│       ├── trigger_kaggle.py       # Start Kaggle notebook via API
│       ├── download_from_kaggle.py # Fetch trained model
│       ├── run_inference_tests.py  # Exact match tests
│       └── upload_to_hf.py         # HuggingFace upload
├── kaggle/
│   ├── train-lora.ipynb            # Kaggle training notebook
│   └── kernel-metadata.json        # Kaggle kernel config
└── configs/
    └── train_qwen3_*.yaml          # Training configs
```

### GitHub Secrets Required

```
KAGGLE_USERNAME   # Your Kaggle username
KAGGLE_KEY        # Kaggle API key
HF_TOKEN          # HuggingFace write token
```

---

### Phase 1: Data Validation Workflow (GitHub CPU - FREE)

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
        run: python scripts/check_distribution.py

      - name: Run tests
        run: pytest tests/ -v
```

---

### Phase 2: Training Pipeline (Kaggle GPU - FREE)

**Step 2a: Trigger Training (GitHub → Kaggle)**

```yaml
# .github/workflows/trigger-training.yml
name: Train Model

on:
  workflow_dispatch:
    inputs:
      model_size:
        description: 'Model size (0.6b, 1.7b, 4b)'
        required: true
        default: '1.7b'

jobs:
  trigger:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: '3.11'

      - name: Install Kaggle CLI
        run: pip install kaggle

      - name: Setup Kaggle credentials
        run: |
          mkdir -p ~/.kaggle
          echo '{"username":"${{ secrets.KAGGLE_USERNAME }}","key":"${{ secrets.KAGGLE_KEY }}"}' > ~/.kaggle/kaggle.json
          chmod 600 ~/.kaggle/kaggle.json

      - name: Upload dataset to Kaggle
        run: |
          kaggle datasets create -p data/final -r zip

      - name: Push and run training notebook
        run: |
          cd kaggle
          # Update notebook with model size
          sed -i 's/MODEL_SIZE = .*/MODEL_SIZE = "${{ inputs.model_size }}"/' train-lora.ipynb
          kaggle kernels push

      - name: Wait for training completion
        run: |
          # Poll Kaggle API until kernel completes (max 6 hours)
          python .github/scripts/wait_for_kaggle.py \
            --kernel animeshkundu/train-lora \
            --timeout 21600

      - name: Download trained model
        run: |
          kaggle kernels output animeshkundu/train-lora -p models/

      - name: Upload model artifact
        uses: actions/upload-artifact@v4
        with:
          name: trained-model-${{ inputs.model_size }}
          path: models/
          retention-days: 7
```

**Step 2b: Kaggle Training Notebook**

```python
# kaggle/train-lora.ipynb (key cells)

# Cell 1: Config
MODEL_SIZE = "1.7b"  # Updated by GitHub Actions
BASE_MODEL = f"Qwen/Qwen3-{MODEL_SIZE}"
OUTPUT_DIR = "/kaggle/working/output"

# Cell 2: Install dependencies
!pip install transformers peft datasets accelerate bitsandbytes

# Cell 3: Load and prepare
from transformers import AutoModelForCausalLM, AutoTokenizer, TrainingArguments
from peft import LoraConfig, get_peft_model
import torch

model = AutoModelForCausalLM.from_pretrained(
    BASE_MODEL,
    torch_dtype=torch.float16,
    device_map="auto"
)

lora_config = LoraConfig(
    r=16,
    lora_alpha=32,
    target_modules=["q_proj", "v_proj", "k_proj", "o_proj"],
    lora_dropout=0.05,
    bias="none"
)

model = get_peft_model(model, lora_config)

# Cell 4: Train
from transformers import Trainer
trainer = Trainer(
    model=model,
    args=TrainingArguments(
        output_dir=OUTPUT_DIR,
        per_device_train_batch_size=4,
        gradient_accumulation_steps=4,
        num_train_epochs=3,
        learning_rate=2e-4,
        fp16=True,
        save_strategy="epoch"
    ),
    train_dataset=train_dataset
)
trainer.train()

# Cell 5: Merge and save
model = model.merge_and_unload()
model.save_pretrained(f"{OUTPUT_DIR}/merged")
tokenizer.save_pretrained(f"{OUTPUT_DIR}/merged")
```

---

### Phase 3: GGUF Conversion (GitHub CPU - FREE)

```yaml
# .github/workflows/post-training.yml
name: Convert and Quantize

on:
  workflow_run:
    workflows: ["Train Model"]
    types: [completed]
  workflow_dispatch:
    inputs:
      model_size:
        required: true
        default: '1.7b'

jobs:
  convert:
    runs-on: ubuntu-latest
    if: ${{ github.event.workflow_run.conclusion == 'success' || github.event_name == 'workflow_dispatch' }}

    steps:
      - uses: actions/checkout@v4

      - name: Download trained model
        uses: actions/download-artifact@v4
        with:
          name: trained-model-${{ inputs.model_size }}
          path: models/

      - name: Setup llama.cpp
        run: |
          git clone https://github.com/ggerganov/llama.cpp
          cd llama.cpp && make -j

      - name: Convert to GGUF F16
        run: |
          python llama.cpp/convert_hf_to_gguf.py \
            models/merged \
            --outfile models/qwen3-wit-${{ inputs.model_size }}-f16.gguf

      - name: Quantize to Q4_K_M
        run: |
          ./llama.cpp/llama-quantize \
            models/qwen3-wit-${{ inputs.model_size }}-f16.gguf \
            models/qwen3-wit-${{ inputs.model_size }}.gguf \
            Q4_K_M

      - name: Upload GGUF artifact
        uses: actions/upload-artifact@v4
        with:
          name: gguf-${{ inputs.model_size }}
          path: models/*.gguf
          retention-days: 7
```

---

### Phase 4: Model Testing (GitHub CPU - FREE)

```yaml
# .github/workflows/test-model.yml
name: Test Model Quality

on:
  workflow_run:
    workflows: ["Convert and Quantize"]
    types: [completed]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Download GGUF model
        uses: actions/download-artifact@v4
        with:
          name: gguf-${{ github.event.workflow_run.inputs.model_size }}
          path: models/

      - name: Setup llama.cpp
        run: |
          git clone https://github.com/ggerganov/llama.cpp
          cd llama.cpp && make -j

      - name: Perplexity test
        id: perplexity
        run: |
          PPL=$(./llama.cpp/llama-perplexity \
            -m models/*.gguf \
            -f tests/test_prompts.txt \
            --chunks 50 2>&1 | grep "perplexity" | tail -1 | awk '{print $NF}')
          echo "perplexity=$PPL" >> $GITHUB_OUTPUT
          echo "Perplexity: $PPL"

      - name: Inference tests
        run: |
          python .github/scripts/run_inference_tests.py \
            --model models/*.gguf \
            --test-file tests/inference_test_cases.json \
            --threshold 0.85

      - name: Create test report
        run: |
          echo "## Model Quality Report" >> $GITHUB_STEP_SUMMARY
          echo "- Perplexity: ${{ steps.perplexity.outputs.perplexity }}" >> $GITHUB_STEP_SUMMARY
```

---

### Phase 5: HuggingFace Release (GitHub CPU - FREE)

```yaml
# .github/workflows/release-model.yml
name: Release to HuggingFace

on:
  workflow_run:
    workflows: ["Test Model Quality"]
    types: [completed]
  workflow_dispatch:
    inputs:
      model_size:
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

      - name: Download GGUF model
        uses: actions/download-artifact@v4
        with:
          name: gguf-${{ inputs.model_size }}
          path: models/

      - name: Upload to HuggingFace
        env:
          HF_TOKEN: ${{ secrets.HF_TOKEN }}
        run: |
          pip install huggingface_hub
          python .github/scripts/upload_to_hf.py \
            --model-path models/qwen3-wit-${{ inputs.model_size }}.gguf \
            --repo-id animeshkundu/cmd-correct \
            --commit-message "Release ${{ inputs.version }}: qwen3-wit-${{ inputs.model_size }}"

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          tag_name: ${{ inputs.version }}
          files: models/*.gguf
          body: |
            ## wit Model Release ${{ inputs.version }}

            - Model: qwen3-wit-${{ inputs.model_size }}
            - Format: GGUF Q4_K_M
            - HuggingFace: https://huggingface.co/animeshkundu/cmd-correct
```

---

## Quality Gates

### Before Training
- [ ] Dataset has 100K+ examples
- [ ] Shell distribution matches target (bash 35%, zsh 25%, etc.)
- [ ] All data validation tests pass
- [ ] Kaggle has sufficient GPU quota remaining

### Before Release
- [ ] Perplexity < baseline + 5%
- [ ] Exact match accuracy > 85%
- [ ] GGUF file validates (llama.cpp can load it)
- [ ] No regression from previous model

---

## Files to Create

| File | Purpose |
|------|---------|
| `.github/workflows/data-validation.yml` | PR validation |
| `.github/workflows/trigger-training.yml` | Kaggle trigger |
| `.github/workflows/post-training.yml` | GGUF conversion |
| `.github/workflows/test-model.yml` | Quality testing |
| `.github/workflows/release-model.yml` | HF upload |
| `.github/scripts/upload_to_kaggle.py` | Push dataset |
| `.github/scripts/wait_for_kaggle.py` | Poll for completion |
| `.github/scripts/download_from_kaggle.py` | Fetch trained model |
| `.github/scripts/run_inference_tests.py` | Exact match tests |
| `.github/scripts/upload_to_hf.py` | HuggingFace upload |
| `kaggle/train-lora.ipynb` | Training notebook |
| `kaggle/kernel-metadata.json` | Kaggle config |
| `tests/inference_test_cases.json` | Test fixtures |

---

## Kaggle Setup (One-Time)

1. **Create Kaggle account** (if not exists)
2. **Generate API key**: kaggle.com → Settings → API → Create New Token
3. **Add GitHub secrets**:
   - `KAGGLE_USERNAME`
   - `KAGGLE_KEY`
4. **Create Kaggle dataset**: `animeshkundu/wit-training-data`
5. **Create Kaggle notebook**: `animeshkundu/train-lora`
6. **Enable GPU**: Notebook settings → Accelerator → GPU T4 x2

---

## Execution Flow

```
Manual Trigger (workflow_dispatch)
        │
        ▼
┌───────────────────┐
│ 1. Upload dataset │  GitHub Actions (CPU)
│    to Kaggle      │
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ 2. Start Kaggle   │  Kaggle API
│    notebook       │
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ 3. LoRA Training  │  Kaggle (T4 GPU) - 3-6 hours
│    on T4 GPU      │
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ 4. Download model │  GitHub Actions (CPU)
│    from Kaggle    │
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ 5. GGUF convert   │  GitHub Actions (CPU)
│    + quantize     │
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ 6. Run tests      │  GitHub Actions (CPU)
│                   │
└─────────┬─────────┘
          │
          ▼
┌───────────────────┐
│ 7. Upload to HF   │  GitHub Actions (CPU)
│                   │
└───────────────────┘
```

---

## Verification

### Test the Pipeline

1. **Data validation**: Create PR with dataset changes, verify CI runs
2. **Kaggle training**: Manually trigger workflow, verify Kaggle starts
3. **Artifact download**: Verify model downloads from Kaggle
4. **GGUF conversion**: Verify .gguf file is created
5. **Quality tests**: Verify perplexity and exact match thresholds
6. **HF upload**: Verify model appears on HuggingFace

### Success Criteria

- [ ] End-to-end pipeline runs without manual intervention
- [ ] Training completes within Kaggle's 9-hour limit
- [ ] Model quality is validated automatically
- [ ] Only passing models are released to HuggingFace
- [ ] Total pipeline time < 8 hours

---

## Cost Summary

| Component | Cost | Notes |
|-----------|------|-------|
| GitHub Actions | FREE | Public repo, unlimited minutes |
| Kaggle GPU | FREE | 30 hrs/week, T4/P100 |
| HuggingFace | FREE | Public models |
| **Total** | **$0** | |

---

## Sources

- [Kaggle Kernels API](https://github.com/Kaggle/kaggle-api)
- [Kaggle GPU Quotas](https://www.kaggle.com/docs/efficient-gpu-usage)
- [llama.cpp GGUF Conversion](https://github.com/ggerganov/llama.cpp)
- [HuggingFace Hub](https://github.com/huggingface/huggingface_hub)
- [LoRA Fine-Tuning](https://huggingface.co/docs/peft)
- [GitHub Actions Free Tier](https://docs.github.com/en/actions/learn-github-actions/usage-limits-billing-and-administration)
