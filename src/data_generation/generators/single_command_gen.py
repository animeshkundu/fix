"""
Single Command Generator (DS1).

Generates training examples for single command corrections including:
- Typos (15%) - Reduced to minimize single-char corrections
- Wrong flags (25%)
- Permission errors (15%)
- Path errors (20%)
- Syntax errors (25%)
"""
from __future__ import annotations

import re
from typing import Any

from .base_generator import (
    BaseGenerator,
    TrainingExample,
    get_random_variable,
    COMMON_VARIABLES,
)


class SingleCommandGenerator(BaseGenerator):
    """Generator for single command correction examples."""

    # Error type distribution - reduced typos to minimize single-char corrections
    ERROR_DISTRIBUTION = {
        "typo": 0.15,        # Reduced from 0.30
        "wrong_flag": 0.25,  # Increased from 0.20
        "permission": 0.15,
        "path": 0.20,        # Increased from 0.15
        "syntax": 0.25,      # Increased from 0.20
    }

    # Maximum percentage of single-char corrections allowed
    MAX_SINGLE_CHAR_PERCENTAGE = 0.05

    # Categories to process from templates
    CATEGORIES = [
        "navigation",
        "file_operations",
        "search",
        "processes",
        "network",
        "git",
        "packages",
        "compression",
        "text",
        "system",
        "docker",
        "kubernetes",
        "variables",
        "help",
    ]

    def __init__(self, templates_dir: str, seed: int = 42):
        """Initialize the single command generator."""
        super().__init__(templates_dir, seed)

    def generate(self, count: int) -> list[TrainingExample]:
        """Generate training examples for single command corrections."""
        examples = []

        # Calculate count per shell based on weights
        shell_counts = {
            shell: int(count * weight)
            for shell, weight in self.SHELL_WEIGHTS.items()
        }

        # Adjust for rounding
        total = sum(shell_counts.values())
        if total < count:
            # Add remaining to bash
            shell_counts["bash"] += count - total

        for shell, shell_count in shell_counts.items():
            if shell not in self.templates:
                continue

            shell_examples = self._generate_for_shell(shell, shell_count)
            examples.extend(shell_examples)

        # Filter out any None examples and null corrections
        examples = [
            ex for ex in examples
            if ex is not None and ex.incorrect_command.strip() != ex.correct_command.strip()
        ]

        # Regenerate if we don't have enough (due to filtered examples)
        max_retries = 3
        retry = 0
        while len(examples) < count and retry < max_retries:
            deficit = count - len(examples)
            # Generate more with bash as default (most templates)
            extra = self._generate_for_shell("bash", deficit * 2)
            extra = [
                ex for ex in extra
                if ex is not None and ex.incorrect_command.strip() != ex.correct_command.strip()
            ]
            examples.extend(extra)
            retry += 1

        # Shuffle to mix shells
        self.rng.shuffle(examples)

        # Enforce single-char correction limit (<5%)
        max_single_char = int(count * self.MAX_SINGLE_CHAR_PERCENTAGE)
        single_char_examples = []
        multi_char_examples = []

        for ex in examples:
            if self._is_single_char_correction(ex.incorrect_command, ex.correct_command):
                single_char_examples.append(ex)
            else:
                multi_char_examples.append(ex)

        # Keep only up to max_single_char single-char corrections
        if len(single_char_examples) > max_single_char:
            single_char_examples = single_char_examples[:max_single_char]

        # Combine and re-shuffle
        examples = multi_char_examples + single_char_examples
        self.rng.shuffle(examples)

        return examples[:count]

    def _generate_for_shell(self, shell: str, count: int) -> list[TrainingExample]:
        """Generate examples for a specific shell."""
        examples = []
        template = self.templates.get(shell, {})

        # Collect all entries from relevant categories
        all_entries = []
        for category in self.CATEGORIES:
            if category in template:
                entries = template[category]
                if isinstance(entries, list):
                    for entry in entries:
                        if isinstance(entry, dict) and "correct" in entry:
                            all_entries.append((category, entry))

        if not all_entries:
            return examples

        # Generate examples
        for _ in range(count):
            category, entry = self.rng.choice(all_entries)
            example = self._create_example_from_entry(shell, category, entry)
            if example:
                examples.append(example)

        return examples

    def _create_example_from_entry(
        self, shell: str, category: str, entry: dict[str, Any]
    ) -> TrainingExample | None:
        """Create a training example from a template entry."""
        correct_template = entry.get("correct", "")
        if not correct_template:
            return None

        # Extract variables first for consistency
        variables = self._extract_variables(correct_template)

        # Expand correct template with fixed variables
        correct_cmd = self.expand_template(correct_template, variables)

        # Select error type based on distribution
        error_type = self._select_error_type()

        # Generate incorrect command based on error type, passing variables for consistency
        incorrect_cmd = self._generate_incorrect(
            correct_cmd, entry, error_type, shell, variables
        )

        if incorrect_cmd == correct_cmd:
            # Fall back to typo if no error was introduced
            incorrect_cmd = self.generate_typo(correct_cmd, typo_rate=1.0)

        # Check for single-char corrections and try to regenerate with multi-char error
        max_retries = 3
        retry = 0
        while self._is_single_char_correction(incorrect_cmd, correct_cmd) and retry < max_retries:
            # Try missing_space first (guaranteed multi-char)
            incorrect_cmd = self.remove_space(correct_cmd)
            if incorrect_cmd == correct_cmd or self._is_single_char_correction(incorrect_cmd, correct_cmd):
                # Try flag error instead
                incorrect_cmd = self._generate_flag_error(correct_cmd, shell)
            if incorrect_cmd == correct_cmd or self._is_single_char_correction(incorrect_cmd, correct_cmd):
                # Try path error
                incorrect_cmd = self._generate_path_error(correct_cmd)
            retry += 1

        # Final null correction check
        if incorrect_cmd.strip() == correct_cmd.strip():
            return None

        return TrainingExample(
            shell=shell,
            incorrect_command=incorrect_cmd,
            correct_command=correct_cmd,
            category=category,
            error_type=error_type,
            source="template",
            metadata={"template": correct_template},
        )

    def _extract_variables(self, template: str) -> dict[str, str]:
        """Extract variable placeholders and generate consistent values."""
        variables = {}
        placeholders = re.findall(r"\{(\w+)\}", template)

        for placeholder in placeholders:
            if placeholder not in variables:
                variables[placeholder] = get_random_variable(placeholder, self.rng)

        return variables

    def _select_error_type(self) -> str:
        """Select an error type based on distribution."""
        types = list(self.ERROR_DISTRIBUTION.keys())
        weights = list(self.ERROR_DISTRIBUTION.values())
        return self.rng.choices(types, weights=weights, k=1)[0]

    def _generate_incorrect(
        self, correct: str, entry: dict, error_type: str, shell: str, variables: dict[str, str] = None
    ) -> str:
        """Generate an incorrect command based on error type."""
        if variables is None:
            variables = {}

        # First check if entry has predefined errors
        if "errors" in entry and entry["errors"]:
            # Use predefined error with some probability
            if self.rng.random() < 0.6:
                error = self.rng.choice(entry["errors"])
                # Use same variables as the correct command for consistency
                return self.expand_template(error, variables)

        # Generate error based on type
        if error_type == "typo":
            return self._generate_typo_error(correct)

        elif error_type == "wrong_flag":
            return self._generate_flag_error(correct, shell)

        elif error_type == "permission":
            return self._generate_permission_error(correct, shell)

        elif error_type == "path":
            return self._generate_path_error(correct)

        elif error_type == "syntax":
            return self._generate_syntax_error(correct, shell)

        return self.generate_typo(correct)

    def _is_single_char_correction(self, incorrect: str, correct: str) -> bool:
        """Check if correction is a trivial single-character change."""
        inc = incorrect.strip()
        cor = correct.strip()

        if len(inc) == len(cor):
            # Same length: count character differences
            diff_count = sum(1 for a, b in zip(inc, cor) if a != b)
            return diff_count == 1
        elif abs(len(inc) - len(cor)) == 1:
            # One character added or removed
            shorter, longer = sorted([inc, cor], key=len)
            for i in range(len(longer)):
                if shorter == longer[:i] + longer[i + 1:]:
                    return True
        return False

    def _generate_typo_error(self, correct: str) -> str:
        """Generate a typo-based error, favoring multi-char changes."""
        # Weight heavily toward missing_space (multi-char) to reduce single-char corrections
        error_subtype = self.rng.choices(
            ["char_swap", "char_delete", "char_double", "adjacent_key", "missing_space"],
            weights=[0.05, 0.05, 0.05, 0.05, 0.80],  # 80% missing_space
            k=1
        )[0]

        if error_subtype == "char_swap":
            return self.generate_typo(correct, typo_rate=1.0)

        elif error_subtype == "char_delete":
            # Delete a character from a word
            words = correct.split()
            if words:
                idx = self.rng.randint(0, len(words) - 1)
                word = words[idx]
                if len(word) > 2:
                    pos = self.rng.randint(1, len(word) - 1)
                    words[idx] = word[:pos] + word[pos + 1:]
                    return " ".join(words)

        elif error_subtype == "char_double":
            # Double a character
            words = correct.split()
            if words:
                idx = self.rng.randint(0, len(words) - 1)
                word = words[idx]
                if len(word) > 1:
                    pos = self.rng.randint(0, len(word) - 1)
                    words[idx] = word[:pos] + word[pos] + word[pos:]
                    return " ".join(words)

        elif error_subtype == "adjacent_key":
            return self.generate_typo(correct, typo_rate=1.0)

        elif error_subtype == "missing_space":
            return self.remove_space(correct)

        return self.generate_typo(correct)

    def _generate_flag_error(self, correct: str, shell: str) -> str:
        """Generate a flag-related error."""
        result = correct

        # Single dash to double dash or vice versa
        if " --" in result:
            result = result.replace(" --", " -", 1)
        elif " -" in result and " --" not in result:
            # Only convert single dash if it's not already a long flag
            parts = result.split()
            for i, part in enumerate(parts):
                if part.startswith("-") and not part.startswith("--") and len(part) > 2:
                    parts[i] = "-" + part
                    break
            result = " ".join(parts)

        # Missing flag value
        if self.rng.random() < 0.3:
            # Remove a flag argument
            parts = result.split()
            for i in range(len(parts) - 1):
                if parts[i].startswith("-") and not parts[i + 1].startswith("-"):
                    parts.pop(i + 1)
                    break
            result = " ".join(parts)

        return result if result != correct else self.generate_typo(correct)

    def _generate_permission_error(self, correct: str, shell: str) -> str:
        """Generate a permission-related error (missing sudo)."""
        # Commands that typically need sudo
        sudo_commands = [
            "apt", "apt-get", "yum", "dnf", "pacman",
            "systemctl", "service", "mount", "umount",
            "chown", "chmod", "useradd", "usermod",
        ]

        # Remove sudo if present (the model should add it back)
        if correct.startswith("sudo "):
            return correct[5:]

        # For commands that need sudo, return without sudo
        words = correct.split()
        if words and any(cmd in words[0] for cmd in sudo_commands):
            if not correct.startswith("sudo"):
                return correct  # Already missing sudo

        # For shell-specific handling
        if shell == "powershell":
            # PowerShell doesn't use sudo the same way
            return correct

        return correct

    def _generate_path_error(self, correct: str) -> str:
        """Generate a path-related error."""
        error_subtype = self.rng.choice([
            "relative_absolute",
            "missing_slash",
            "typo_in_path",
            "wrong_separator",
        ])

        result = correct

        if error_subtype == "relative_absolute":
            # Switch between relative and absolute
            if "./" in result:
                result = result.replace("./", "/", 1)
            elif result.startswith("/"):
                result = "." + result

        elif error_subtype == "missing_slash":
            # Remove a slash
            if "/" in result:
                # Find path segments and join two
                parts = result.split()
                for i, part in enumerate(parts):
                    if "/" in part:
                        segments = part.split("/")
                        if len(segments) > 2:
                            idx = self.rng.randint(0, len(segments) - 2)
                            segments[idx] = segments[idx] + segments[idx + 1]
                            segments.pop(idx + 1)
                            parts[i] = "/".join(segments)
                            break
                result = " ".join(parts)

        elif error_subtype == "typo_in_path":
            # Introduce typo in path
            result = self.generate_typo(correct, typo_rate=1.0)

        elif error_subtype == "wrong_separator":
            # Windows vs Unix path separator
            if "/" in result and self.rng.random() < 0.5:
                result = result.replace("/", "\\", 1)

        return result if result != correct else self.generate_typo(correct)

    def _generate_syntax_error(self, correct: str, shell: str) -> str:
        """Generate a syntax error."""
        error_subtype = self.rng.choice([
            "missing_quote",
            "missing_escape",
            "wrong_operator",
            "extra_character",
        ])

        result = correct

        if error_subtype == "missing_quote":
            # Remove one quote
            if '"' in result:
                # Find first quote and remove it
                result = result.replace('"', "", 1)
            elif "'" in result:
                result = result.replace("'", "", 1)

        elif error_subtype == "missing_escape":
            # Remove backslash
            if "\\" in result:
                result = result.replace("\\", "", 1)

        elif error_subtype == "wrong_operator":
            # Wrong comparison or redirect
            operators = [
                ("&&", "&"),
                ("||", "|"),
                (">>", ">"),
                ("==", "="),
            ]
            for correct_op, wrong_op in operators:
                if correct_op in result:
                    result = result.replace(correct_op, wrong_op, 1)
                    break

        elif error_subtype == "extra_character":
            # Add an extra character
            words = result.split()
            if words:
                idx = self.rng.randint(0, len(words) - 1)
                word = words[idx]
                pos = self.rng.randint(0, len(word))
                char = self.rng.choice("{}[]();")
                words[idx] = word[:pos] + char + word[pos:]
                result = " ".join(words)

        return result if result != correct else self.generate_typo(correct)


if __name__ == "__main__":
    import sys

    # Test generation
    templates_dir = sys.argv[1] if len(sys.argv) > 1 else "src/data_generation/templates"

    generator = SingleCommandGenerator(templates_dir)
    examples = generator.generate(100)

    print(f"Generated {len(examples)} examples")
    print("\nSample examples:")
    for example in examples[:10]:
        print(f"\nShell: {example.shell}")
        print(f"Error type: {example.error_type}")
        print(f"Incorrect: {example.incorrect_command}")
        print(f"Correct: {example.correct_command}")
