#!/usr/bin/env python3
"""
Verify dataset quality and distribution.

Checks:
1. Single-char corrections < 5%
2. No null corrections (incorrect == correct)
3. Dataset distribution matches targets
4. Shell diversity
"""
from __future__ import annotations

import json
import sys
from pathlib import Path
from collections import Counter


def is_single_char_correction(incorrect: str, correct: str) -> bool:
    """Check if correction is a trivial single-character change."""
    inc = incorrect.strip()
    cor = correct.strip()

    if len(inc) == len(cor):
        diff_count = sum(1 for a, b in zip(inc, cor) if a != b)
        return diff_count == 1
    elif abs(len(inc) - len(cor)) == 1:
        shorter, longer = sorted([inc, cor], key=len)
        for i in range(len(longer)):
            if shorter == longer[:i] + longer[i + 1:]:
                return True
    return False


def analyze_dataset(data_dir: Path) -> dict:
    """Analyze dataset files and return statistics."""
    stats = {
        "total": 0,
        "single_char": 0,
        "null_corrections": 0,
        "by_dataset": {},
        "by_shell": Counter(),
    }

    # Analyze each dataset
    for jsonl_file in sorted(data_dir.glob("*_analysis.jsonl")):
        if jsonl_file.name.endswith("_commands_analysis.jsonl"):
            continue  # Skip old files

        dataset_name = jsonl_file.stem.replace("_analysis", "")
        dataset_stats = {
            "total": 0,
            "single_char": 0,
            "null_corrections": 0,
            "shells": Counter(),
        }

        with open(jsonl_file, encoding="utf-8") as f:
            for line in f:
                example = json.loads(line)
                incorrect = example.get("incorrect_command", "")
                correct = example.get("correct_command", "")
                shell = example.get("shell", "unknown")

                dataset_stats["total"] += 1
                dataset_stats["shells"][shell] += 1
                stats["by_shell"][shell] += 1

                # Check for null corrections
                if incorrect.strip() == correct.strip():
                    dataset_stats["null_corrections"] += 1
                    stats["null_corrections"] += 1

                # Check for single-char corrections (only for DS1 and DS4)
                if dataset_name in ["ds1_single", "ds4_tools"]:
                    if is_single_char_correction(incorrect, correct):
                        dataset_stats["single_char"] += 1
                        stats["single_char"] += 1

        stats["total"] += dataset_stats["total"]
        stats["by_dataset"][dataset_name] = dataset_stats

    return stats


def print_report(stats: dict) -> bool:
    """Print analysis report and return True if all checks pass."""
    print("=" * 60)
    print("DATASET VERIFICATION REPORT")
    print("=" * 60)

    all_passed = True

    # Overall stats
    print(f"\nTotal examples: {stats['total']:,}")

    # Single-char check
    if stats["total"] > 0:
        single_char_pct = stats["single_char"] / stats["total"] * 100
        status = "PASS" if single_char_pct < 5 else "FAIL"
        if status == "FAIL":
            all_passed = False
        print(f"\nSingle-char corrections: {stats['single_char']:,} ({single_char_pct:.2f}%) [{status}]")
        print(f"  Target: < 5%")

    # Null corrections check
    status = "PASS" if stats["null_corrections"] == 0 else "FAIL"
    if status == "FAIL":
        all_passed = False
    print(f"\nNull corrections: {stats['null_corrections']} [{status}]")
    print(f"  Target: 0")

    # Dataset distribution
    print("\n" + "-" * 60)
    print("DATASET DISTRIBUTION")
    print("-" * 60)

    targets = {
        "ds1_single": (23.3, "Single command corrections"),
        "ds2_chained": (23.3, "Chained/piped commands"),
        "ds3_natural_language": (33.3, "Natural language translations"),
        "ds4_tools": (20.0, "Top 100 tools"),
    }

    for ds_name, (target_pct, desc) in targets.items():
        ds_stats = stats["by_dataset"].get(ds_name, {"total": 0})
        count = ds_stats["total"]
        pct = count / stats["total"] * 100 if stats["total"] > 0 else 0
        print(f"\n{ds_name}: {count:,} ({pct:.1f}%)")
        print(f"  {desc}")
        print(f"  Target: ~{target_pct}%")
        if ds_stats.get("null_corrections", 0) > 0:
            print(f"  WARNING: {ds_stats['null_corrections']} null corrections")
        if ds_stats.get("single_char", 0) > 0:
            sc_pct = ds_stats["single_char"] / count * 100 if count > 0 else 0
            print(f"  Single-char: {ds_stats['single_char']} ({sc_pct:.2f}%)")

    # Shell distribution
    print("\n" + "-" * 60)
    print("SHELL DISTRIBUTION")
    print("-" * 60)

    for shell, count in stats["by_shell"].most_common():
        pct = count / stats["total"] * 100 if stats["total"] > 0 else 0
        print(f"  {shell}: {count:,} ({pct:.1f}%)")

    print("\n" + "=" * 60)
    if all_passed:
        print("ALL CHECKS PASSED")
    else:
        print("SOME CHECKS FAILED - See details above")
    print("=" * 60)

    return all_passed


def main():
    # Default to generated directory
    if len(sys.argv) > 1:
        data_dir = Path(sys.argv[1])
    else:
        data_dir = Path(__file__).parent.parent / "data" / "generated"

    if not data_dir.exists():
        print(f"Error: Directory not found: {data_dir}")
        sys.exit(1)

    stats = analyze_dataset(data_dir)
    passed = print_report(stats)

    sys.exit(0 if passed else 1)


if __name__ == "__main__":
    main()
