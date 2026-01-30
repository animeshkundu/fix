"""
Chained Command Generator (DS2).

Generates training examples for chained/piped command corrections including:
- Pipe corrections (40%)
- Command chaining (30%)
- Redirection (20%)
- Subshell & grouping (10%)
"""
from __future__ import annotations

from typing import Any

from .base_generator import (
    BaseGenerator,
    TrainingExample,
    get_random_variable,
)


class ChainedCommandGenerator(BaseGenerator):
    """Generator for chained and piped command correction examples."""

    # Error type distribution
    ERROR_DISTRIBUTION = {
        "pipe": 0.40,
        "chaining": 0.30,
        "redirection": 0.20,
        "subshell": 0.10,
    }

    # Common pipe patterns per shell
    PIPE_PATTERNS = {
        "bash": [
            {
                "correct": "grep '{pattern}' {file} | sort -u | wc -l",
                "errors": [
                    "cat {file} | grep '{pattern}' | sort | uniq | wc",
                    "grep {pattern} {file} | sort -u | wc",
                ],
                "category": "text_processing",
            },
            {
                "correct": "find . -name '*.py' -exec grep -l '{pattern}' {} +",
                "errors": [
                    "find . -name '*.py' | xargs grep '{pattern}'",
                    "find . -name *.py -exec grep -l {pattern} {} \\;",
                ],
                "category": "search",
            },
            {
                "correct": "ps aux | grep -v grep | grep {process}",
                "errors": [
                    "ps aux | grep {process}",
                    "ps -ef | grep {process}",
                ],
                "category": "processes",
            },
            {
                "correct": "curl -s {url} | jq '.'",
                "errors": [
                    "curl {url} | jq .",
                    "curl -s {url} | jq",
                ],
                "category": "network",
            },
            {
                "correct": "docker ps -q | xargs docker stop",
                "errors": [
                    "docker stop $(docker ps -q)",
                    "docker ps | xargs docker stop",
                ],
                "category": "docker",
            },
            {
                "correct": "git log --oneline | head -10",
                "errors": [
                    "git log | head -10",
                    "git log --oneline | head",
                ],
                "category": "git",
            },
            {
                "correct": "ls -la | awk '{print $9}'",
                "errors": [
                    "ls -la | awk {print $9}",
                    "ls -la | cut -d' ' -f9",
                ],
                "category": "file_operations",
            },
        ],
        "zsh": [
            {
                "correct": "grep -r '{pattern}' **/*.py | sort -u",
                "errors": [
                    "grep -r '{pattern}' . --include='*.py' | sort -u",
                    "find . -name '*.py' | xargs grep '{pattern}' | sort -u",
                ],
                "category": "search",
            },
        ],
        "powershell": [
            {
                "correct": "Get-Process | Where-Object {{$_.CPU -gt 100}} | Sort-Object CPU -Descending",
                "errors": [
                    "Get-Process | Where CPU -gt 100 | Sort CPU",
                    "ps | where CPU -gt 100 | sort CPU",
                ],
                "category": "processes",
            },
            {
                "correct": "Get-ChildItem -Recurse | Where-Object {{$_.Length -gt 1MB}}",
                "errors": [
                    "Get-ChildItem -r | Where Length -gt 1MB",
                    "gci -r | ? Length -gt 1MB",
                ],
                "category": "file_operations",
            },
            {
                "correct": "Get-Content {file} | Select-String '{pattern}'",
                "errors": [
                    "gc {file} | sls '{pattern}'",
                    "cat {file} | grep '{pattern}'",
                ],
                "category": "text_processing",
            },
        ],
        "cmd": [
            {
                "correct": "dir /s /b | findstr \"{pattern}\"",
                "errors": [
                    "dir /s | findstr {pattern}",
                    "dir | findstr \"{pattern}\"",
                ],
                "category": "search",
            },
            {
                "correct": "type {file} | findstr \"{pattern}\"",
                "errors": [
                    "type {file} | find \"{pattern}\"",
                    "findstr \"{pattern}\" {file}",
                ],
                "category": "text_processing",
            },
        ],
        "fish": [
            {
                "correct": "find . -name '*.py' | while read f; echo $f; end",
                "errors": [
                    "find . -name '*.py' | while read f; do echo $f; done",
                    "for f in (find . -name '*.py'); echo $f; end",
                ],
                "category": "search",
            },
        ],
        "tcsh": [
            {
                "correct": "find . -name '*.py' | xargs grep '{pattern}'",
                "errors": [
                    "find . -name *.py | xargs grep {pattern}",
                ],
                "category": "search",
            },
        ],
    }

    # Command chaining patterns
    CHAINING_PATTERNS = {
        "bash": [
            {
                "correct": "mkdir -p {path} && cd {path}",
                "errors": [
                    "mkdir {path}; cd {path}",
                    "mkdir -p {path}; cd {path}",
                    "mkdir {path} && cd {path}",
                ],
                "category": "navigation",
            },
            {
                "correct": "git add . && git commit -m '{message}' && git push",
                "errors": [
                    "git add .; git commit -m '{message}'; git push",
                    "git add . && git commit -m '{message}'",
                    "git add && git commit && git push",
                ],
                "category": "git",
            },
            {
                "correct": "cd {path} && npm install && npm start",
                "errors": [
                    "cd {path}; npm install; npm start",
                    "cd {path} && npm i && npm start",
                ],
                "category": "development",
            },
            {
                "correct": "sudo apt update && sudo apt upgrade -y",
                "errors": [
                    "apt update && apt upgrade",
                    "sudo apt update; sudo apt upgrade -y",
                ],
                "category": "packages",
            },
            {
                "correct": "docker-compose down && docker-compose up -d",
                "errors": [
                    "docker compose down && docker compose up -d",
                    "docker-compose down; docker-compose up -d",
                ],
                "category": "docker",
            },
            {
                "correct": "make clean && make && make install",
                "errors": [
                    "make clean; make; make install",
                    "make clean && make",
                ],
                "category": "development",
            },
            {
                "correct": "test -f {file} && cat {file} || echo 'Not found'",
                "errors": [
                    "if [ -f {file} ]; then cat {file}; else echo 'Not found'; fi",
                    "[ -f {file} ] && cat {file}",
                ],
                "category": "file_operations",
            },
        ],
        "zsh": [
            {
                "correct": "mkdir -p {path} && cd {path}",
                "errors": [
                    "mkdir {path}; cd {path}",
                ],
                "category": "navigation",
            },
        ],
        "powershell": [
            {
                "correct": "New-Item -ItemType Directory -Path {path} -Force; Set-Location {path}",
                "errors": [
                    "mkdir {path}; cd {path}",
                    "md {path} && cd {path}",
                ],
                "category": "navigation",
            },
            {
                "correct": "git add .; git commit -m '{message}'; git push",
                "errors": [
                    "git add . && git commit -m '{message}' && git push",
                ],
                "category": "git",
            },
        ],
        "cmd": [
            {
                "correct": "mkdir {path} && cd {path}",
                "errors": [
                    "mkdir {path} & cd {path}",
                    "md {path} && cd {path}",
                ],
                "category": "navigation",
            },
        ],
        "fish": [
            {
                "correct": "mkdir -p {path}; and cd {path}",
                "errors": [
                    "mkdir -p {path} && cd {path}",
                    "mkdir {path}; cd {path}",
                ],
                "category": "navigation",
            },
        ],
        "tcsh": [
            {
                "correct": "mkdir -p {path} && cd {path}",
                "errors": [
                    "mkdir {path}; cd {path}",
                ],
                "category": "navigation",
            },
        ],
    }

    # Redirection patterns
    REDIRECTION_PATTERNS = {
        "bash": [
            {
                "correct": "command > {file} 2>&1",
                "errors": [
                    "command 2>&1 > {file}",
                    "command > {file} 2>1",
                    "command &> {file}",
                ],
                "category": "redirection",
            },
            {
                "correct": "command >> {file} 2>&1",
                "errors": [
                    "command 2>&1 >> {file}",
                    "command >> {file}",
                ],
                "category": "redirection",
            },
            {
                "correct": "command 2>/dev/null",
                "errors": [
                    "command 2> /dev/null",
                    "command >/dev/null 2>&1",
                ],
                "category": "redirection",
            },
            {
                "correct": "cat << 'EOF'\ncontent\nEOF",
                "errors": [
                    "cat << EOF\ncontent\nEOF",
                    "echo 'content'",
                ],
                "category": "redirection",
            },
            {
                "correct": "tee {file} <<< 'content'",
                "errors": [
                    "echo 'content' | tee {file}",
                    "echo 'content' > {file}",
                ],
                "category": "redirection",
            },
        ],
        "zsh": [
            {
                "correct": "command &> {file}",
                "errors": [
                    "command > {file} 2>&1",
                ],
                "category": "redirection",
            },
        ],
        "powershell": [
            {
                "correct": "command 2>&1 | Out-File {file}",
                "errors": [
                    "command > {file} 2>&1",
                    "command | Out-File {file}",
                ],
                "category": "redirection",
            },
            {
                "correct": "command *> {file}",
                "errors": [
                    "command > {file}",
                    "command 2>&1 > {file}",
                ],
                "category": "redirection",
            },
        ],
        "cmd": [
            {
                "correct": "command > {file} 2>&1",
                "errors": [
                    "command > {file}",
                    "command 2>&1 > {file}",
                ],
                "category": "redirection",
            },
        ],
        "fish": [
            {
                "correct": "command > {file} 2>&1",
                "errors": [
                    "command &> {file}",
                ],
                "category": "redirection",
            },
        ],
        "tcsh": [
            {
                "correct": "command >& {file}",
                "errors": [
                    "command > {file} 2>&1",
                ],
                "category": "redirection",
            },
        ],
    }

    def __init__(self, templates_dir: str, seed: int = 42):
        """Initialize the chained command generator."""
        super().__init__(templates_dir, seed)

    def generate(self, count: int) -> list[TrainingExample]:
        """Generate training examples for chained command corrections."""
        examples = []

        # Calculate count per shell based on weights
        shell_counts = {
            shell: int(count * weight)
            for shell, weight in self.SHELL_WEIGHTS.items()
        }

        # Adjust for rounding
        total = sum(shell_counts.values())
        if total < count:
            shell_counts["bash"] += count - total

        for shell, shell_count in shell_counts.items():
            shell_examples = self._generate_for_shell(shell, shell_count)
            examples.extend(shell_examples)

        # Filter out any null corrections (safety check)
        examples = [
            ex for ex in examples
            if ex.incorrect_command.strip() != ex.correct_command.strip()
        ]

        # Regenerate if we don't have enough (due to filtered null corrections)
        max_retries = 3
        retry = 0
        while len(examples) < count and retry < max_retries:
            deficit = count - len(examples)
            # Generate more with bash as default
            extra = self._generate_for_shell("bash", deficit * 2)
            extra = [
                ex for ex in extra
                if ex.incorrect_command.strip() != ex.correct_command.strip()
            ]
            examples.extend(extra)
            retry += 1

        self.rng.shuffle(examples)
        return examples[:count]

    def _generate_for_shell(self, shell: str, count: int) -> list[TrainingExample]:
        """Generate examples for a specific shell."""
        examples = []

        # Get patterns for this shell
        pipe_patterns = self.PIPE_PATTERNS.get(shell, self.PIPE_PATTERNS.get("bash", []))
        chain_patterns = self.CHAINING_PATTERNS.get(shell, self.CHAINING_PATTERNS.get("bash", []))
        redir_patterns = self.REDIRECTION_PATTERNS.get(shell, self.REDIRECTION_PATTERNS.get("bash", []))

        for _ in range(count):
            error_type = self._select_error_type()

            if error_type == "pipe" and pipe_patterns:
                pattern = self.rng.choice(pipe_patterns)
            elif error_type == "chaining" and chain_patterns:
                pattern = self.rng.choice(chain_patterns)
            elif error_type == "redirection" and redir_patterns:
                pattern = self.rng.choice(redir_patterns)
            else:
                # Fallback to any available pattern
                all_patterns = pipe_patterns + chain_patterns + redir_patterns
                if not all_patterns:
                    continue
                pattern = self.rng.choice(all_patterns)

            example = self._create_example_from_pattern(shell, pattern, error_type)
            if example:
                examples.append(example)

        return examples

    def _select_error_type(self) -> str:
        """Select an error type based on distribution."""
        types = list(self.ERROR_DISTRIBUTION.keys())
        weights = list(self.ERROR_DISTRIBUTION.values())
        return self.rng.choices(types, weights=weights, k=1)[0]

    def _create_example_from_pattern(
        self, shell: str, pattern: dict[str, Any], error_type: str
    ) -> TrainingExample | None:
        """Create a training example from a pattern."""
        correct_template = pattern.get("correct", "")
        errors = pattern.get("errors", [])
        category = pattern.get("category", "chained")

        if not correct_template:
            return None

        # Extract variables first for consistency
        variables = self._extract_variables(correct_template)

        # Expand correct template with fixed variables
        correct_cmd = self.expand_template(correct_template, variables)

        # Select an error - use SAME variable values
        if errors and self.rng.random() < 0.7:
            error_template = self.rng.choice(errors)
            # Apply same variables to error template
            incorrect_cmd = self.expand_template(error_template, variables)
        else:
            # Generate synthetic error on already-expanded command
            incorrect_cmd = self._generate_synthetic_error(correct_cmd, error_type, shell)

        # CRITICAL: Validate that incorrect != correct (avoid null corrections)
        if incorrect_cmd.strip() == correct_cmd.strip():
            # Try synthetic error first
            incorrect_cmd = self._generate_synthetic_error(correct_cmd, error_type, shell)

            # If still equal, force a typo
            if incorrect_cmd.strip() == correct_cmd.strip():
                incorrect_cmd = self.generate_typo(correct_cmd, typo_rate=1.0)

            # Final check - skip if still equal
            if incorrect_cmd.strip() == correct_cmd.strip():
                return None

        return TrainingExample(
            shell=shell,
            incorrect_command=incorrect_cmd,
            correct_command=correct_cmd,
            category=category,
            error_type=error_type,
            source="chained_pattern",
        )

    def _extract_variables(self, template: str) -> dict[str, str]:
        """Extract variable placeholders and generate consistent values."""
        import re

        variables = {}
        placeholders = re.findall(r"\{(\w+)\}", template)

        for placeholder in placeholders:
            if placeholder not in variables:
                variables[placeholder] = get_random_variable(placeholder, self.rng)

        return variables

    def _generate_synthetic_error(self, correct: str, error_type: str, shell: str) -> str:
        """Generate a synthetic error for the command."""
        if error_type == "pipe":
            # Common pipe errors
            if " | " in correct:
                # Remove a pipe segment
                parts = correct.split(" | ")
                if len(parts) > 2:
                    idx = self.rng.randint(1, len(parts) - 1)
                    parts.pop(idx)
                    return " | ".join(parts)

        elif error_type == "chaining":
            # Common chaining errors
            if " && " in correct:
                # Replace && with ;
                return correct.replace(" && ", "; ", 1)
            elif " || " in correct:
                return correct.replace(" || ", " | ", 1)

        elif error_type == "redirection":
            # Common redirection errors
            if "2>&1" in correct:
                return correct.replace("2>&1", "2>1", 1)
            elif ">>" in correct:
                return correct.replace(">>", ">", 1)

        # Fallback to typo
        return self.generate_typo(correct)


if __name__ == "__main__":
    import sys

    templates_dir = sys.argv[1] if len(sys.argv) > 1 else "src/data_generation/templates"

    generator = ChainedCommandGenerator(templates_dir)
    examples = generator.generate(50)

    print(f"Generated {len(examples)} examples")
    print("\nSample examples:")
    for example in examples[:5]:
        print(f"\nShell: {example.shell}")
        print(f"Error type: {example.error_type}")
        print(f"Incorrect: {example.incorrect_command}")
        print(f"Correct: {example.correct_command}")
