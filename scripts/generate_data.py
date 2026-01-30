#!/usr/bin/env python3
from __future__ import annotations

"""
Data Generation Orchestrator.

Orchestrates the generation of all training datasets:
- DS1: Single command corrections (35,000) - 23.3%
- DS2: Chained/piped commands (35,000) - 23.3%
- DS3: Natural language translations (50,000) - 33.3%
- DS4: Top 100 tools corrections (30,000) - 20%

Total: ~150,000 training examples

Distribution targets:
- Single-char corrections: <5%
- NL to command: >=30%
- Multi-command (piped/chained): >=20%
- Top 100 tools: >=20%
"""

import argparse
import json
import sys
from pathlib import Path
from typing import Any

# Add project root to path
project_root = Path(__file__).parent.parent
sys.path.insert(0, str(project_root))

from src.data_generation.generators.single_command_gen import SingleCommandGenerator
from src.data_generation.generators.chained_command_gen import ChainedCommandGenerator
from src.data_generation.generators.natural_lang_gen import NaturalLanguageGenerator
from src.data_generation.generators.tools_gen import ToolsGenerator


# Dataset configuration - Rebalanced to meet distribution targets
DATASET_CONFIG = {
    "ds1_single": {
        "generator": SingleCommandGenerator,
        "count": 35_000,  # Reduced from 50K to minimize single-char corrections
        "description": "Single command corrections (typos, wrong flags, etc.)",
    },
    "ds2_chained": {
        "generator": ChainedCommandGenerator,
        "count": 35_000,  # Increased from 30K for more multi-command coverage
        "description": "Chained and piped command corrections",
    },
    "ds3_natural_language": {
        "generator": NaturalLanguageGenerator,
        "count": 50_000,  # Increased from 40K to meet 30%+ NL target
        "description": "Natural language to command translations",
    },
    "ds4_tools": {
        "generator": ToolsGenerator,
        "count": 30_000,  # Unchanged - meets 20% target
        "description": "Top 100 CLI tools corrections",
    },
}


def generate_dataset(
    name: str,
    config: dict[str, Any],
    templates_dir: Path,
    output_dir: Path,
    seed: int = 42,
    sample_size: int | None = None,
) -> list[dict]:
    """Generate a single dataset."""
    print(f"\nGenerating {name}: {config['description']}")

    generator_class = config["generator"]
    count = sample_size if sample_size else config["count"]

    generator = generator_class(str(templates_dir), seed=seed)
    examples = generator.generate(count)

    print(f"  Generated {len(examples)} examples")

    # Save to JSONL (chat format for training)
    output_file = output_dir / f"{name}.jsonl"
    generator.save_jsonl(examples, output_file)
    print(f"  Saved to {output_file}")

    # Save analysis version
    analysis_file = output_dir / f"{name}_analysis.jsonl"
    generator.save_analysis_jsonl(examples, analysis_file)
    print(f"  Analysis saved to {analysis_file}")

    # Return examples for combining
    return [ex.to_chat_format() for ex in examples]


def combine_datasets(
    all_examples: list[dict],
    output_dir: Path,
    train_ratio: float = 0.9,
    val_ratio: float = 0.05,
    seed: int = 42,
) -> None:
    """Combine all datasets and split into train/val/test."""
    import random

    rng = random.Random(seed)

    # Shuffle all examples
    rng.shuffle(all_examples)

    total = len(all_examples)
    train_end = int(total * train_ratio)
    val_end = train_end + int(total * val_ratio)

    train_data = all_examples[:train_end]
    val_data = all_examples[train_end:val_end]
    test_data = all_examples[val_end:]

    print(f"\nDataset splits:")
    print(f"  Train: {len(train_data)} ({len(train_data)/total*100:.1f}%)")
    print(f"  Validation: {len(val_data)} ({len(val_data)/total*100:.1f}%)")
    print(f"  Test: {len(test_data)} ({len(test_data)/total*100:.1f}%)")

    # Save splits
    final_dir = output_dir.parent / "final"
    final_dir.mkdir(parents=True, exist_ok=True)

    for name, data in [("train", train_data), ("validation", val_data), ("test", test_data)]:
        output_file = final_dir / f"{name}.jsonl"
        with open(output_file, "w", encoding="utf-8") as f:
            for example in data:
                f.write(json.dumps(example, ensure_ascii=False) + "\n")
        print(f"  Saved {output_file}")


def print_statistics(output_dir: Path) -> None:
    """Print statistics about generated datasets."""
    print("\n" + "=" * 50)
    print("Dataset Statistics")
    print("=" * 50)

    # Count examples per file
    for jsonl_file in sorted(output_dir.glob("*.jsonl")):
        if "_analysis" in jsonl_file.name:
            continue

        count = sum(1 for _ in open(jsonl_file, encoding="utf-8"))
        print(f"  {jsonl_file.name}: {count:,} examples")

    # Count final splits
    final_dir = output_dir.parent / "final"
    if final_dir.exists():
        print("\nFinal splits:")
        for jsonl_file in sorted(final_dir.glob("*.jsonl")):
            count = sum(1 for _ in open(jsonl_file, encoding="utf-8"))
            print(f"  {jsonl_file.name}: {count:,} examples")


def main():
    parser = argparse.ArgumentParser(
        description="Generate training data for command correction model"
    )
    parser.add_argument(
        "--dataset",
        choices=["all", "ds1", "ds2", "ds3", "ds4"],
        default="all",
        help="Which dataset to generate (default: all)",
    )
    parser.add_argument(
        "--sample",
        type=int,
        default=None,
        help="Generate only N samples per dataset (for testing)",
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=42,
        help="Random seed for reproducibility",
    )
    parser.add_argument(
        "--templates-dir",
        type=Path,
        default=project_root / "src" / "data_generation" / "templates",
        help="Path to templates directory",
    )
    parser.add_argument(
        "--output-dir",
        type=Path,
        default=project_root / "data" / "generated",
        help="Path to output directory",
    )
    parser.add_argument(
        "--no-combine",
        action="store_true",
        help="Don't combine datasets into train/val/test splits",
    )

    args = parser.parse_args()

    # Ensure output directory exists
    args.output_dir.mkdir(parents=True, exist_ok=True)

    print("=" * 50)
    print("Command Correction Training Data Generator")
    print("=" * 50)
    print(f"Templates: {args.templates_dir}")
    print(f"Output: {args.output_dir}")
    print(f"Seed: {args.seed}")
    if args.sample:
        print(f"Sample size: {args.sample}")

    # Determine which datasets to generate
    if args.dataset == "all":
        datasets_to_generate = list(DATASET_CONFIG.keys())
    else:
        dataset_map = {
            "ds1": "ds1_single",
            "ds2": "ds2_chained",
            "ds3": "ds3_natural_language",
            "ds4": "ds4_tools",
        }
        datasets_to_generate = [dataset_map[args.dataset]]

    # Generate datasets
    all_examples = []
    for name in datasets_to_generate:
        config = DATASET_CONFIG[name]
        examples = generate_dataset(
            name=name,
            config=config,
            templates_dir=args.templates_dir,
            output_dir=args.output_dir,
            seed=args.seed,
            sample_size=args.sample,
        )
        all_examples.extend(examples)

    # Combine into train/val/test
    if not args.no_combine and len(datasets_to_generate) > 1:
        combine_datasets(all_examples, args.output_dir, seed=args.seed)

    # Print statistics
    print_statistics(args.output_dir)

    print("\n" + "=" * 50)
    print("Generation complete!")
    print("=" * 50)


if __name__ == "__main__":
    main()
