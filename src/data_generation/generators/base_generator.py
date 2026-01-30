from __future__ import annotations

"""
Base generator class for creating training data.

Provides common functionality for all dataset generators.
"""

import json
import random
import string
from abc import ABC, abstractmethod
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Dict, List, Optional, Union

import yaml


@dataclass
class TrainingExample:
    """A single training example for the command correction model."""

    shell: str
    incorrect_command: str
    correct_command: str
    category: str = "unknown"
    error_type: str = "unknown"
    source: str = "generated"
    metadata: dict[str, Any] = field(default_factory=dict)

    def to_chat_format(self) -> dict[str, Any]:
        """Convert to ChatML format for training."""
        return {
            "messages": [
                {
                    "role": "system",
                    "content": f"You are a shell command corrector for {self.shell}. Output only the corrected command.",
                },
                {"role": "user", "content": self.incorrect_command},
                {"role": "assistant", "content": self.correct_command},
            ]
        }

    def to_dict(self) -> dict[str, Any]:
        """Convert to dictionary for analysis."""
        return {
            "shell": self.shell,
            "incorrect_command": self.incorrect_command,
            "correct_command": self.correct_command,
            "category": self.category,
            "error_type": self.error_type,
            "source": self.source,
            "metadata": self.metadata,
        }


class BaseGenerator(ABC):
    """Abstract base class for training data generators."""

    # Shell weights for data distribution
    SHELL_WEIGHTS = {
        "bash": 0.35,
        "zsh": 0.25,
        "powershell": 0.20,
        "cmd": 0.12,
        "fish": 0.05,
        "tcsh": 0.03,
    }

    # Common keyboard adjacency for typo generation
    KEYBOARD_ADJACENT = {
        "a": "qwsz",
        "b": "vghn",
        "c": "xdfv",
        "d": "serfcx",
        "e": "wsdr",
        "f": "drtgcv",
        "g": "ftyhbv",
        "h": "gyujnb",
        "i": "ujko",
        "j": "huiknm",
        "k": "jiolm",
        "l": "kop",
        "m": "njk",
        "n": "bhjm",
        "o": "iklp",
        "p": "ol",
        "q": "wa",
        "r": "edft",
        "s": "awedxz",
        "t": "rfgy",
        "u": "yhji",
        "v": "cfgb",
        "w": "qase",
        "x": "zsdc",
        "y": "tghu",
        "z": "asx",
    }

    def __init__(self, templates_dir: str | Path, seed: int = 42):
        """Initialize generator with templates directory."""
        self.templates_dir = Path(templates_dir)
        self.templates: dict[str, dict] = {}
        self.rng = random.Random(seed)
        self._load_templates()

    def _load_templates(self) -> None:
        """Load shell-specific templates from YAML files."""
        for shell in self.SHELL_WEIGHTS:
            template_file = self.templates_dir / f"{shell}.yaml"
            if template_file.exists():
                with open(template_file, encoding="utf-8") as f:
                    self.templates[shell] = yaml.safe_load(f)

    def select_shell(self) -> str:
        """Select a shell based on configured weights."""
        shells = list(self.SHELL_WEIGHTS.keys())
        weights = list(self.SHELL_WEIGHTS.values())
        return self.rng.choices(shells, weights=weights, k=1)[0]

    def generate_typo(self, text: str, typo_rate: float = 0.15) -> str:
        """Generate a realistic typo in the given text."""
        if not text or self.rng.random() > typo_rate:
            return text

        words = text.split()
        if not words:
            return text

        # Select a word to modify (prefer longer words)
        word_weights = [len(w) for w in words]
        idx = self.rng.choices(range(len(words)), weights=word_weights, k=1)[0]
        word = words[idx]

        if len(word) < 2:
            return text

        # Choose typo type
        typo_type = self.rng.choice(
            ["swap", "delete", "insert", "adjacent", "double"]
        )

        if typo_type == "swap" and len(word) >= 3:
            # Swap two adjacent characters
            pos = self.rng.randint(0, len(word) - 2)
            word = word[:pos] + word[pos + 1] + word[pos] + word[pos + 2 :]

        elif typo_type == "delete" and len(word) >= 3:
            # Delete a character
            pos = self.rng.randint(1, len(word) - 1)
            word = word[:pos] + word[pos + 1 :]

        elif typo_type == "insert":
            # Insert a random character
            pos = self.rng.randint(1, len(word) - 1)
            char = self.rng.choice(string.ascii_lowercase)
            word = word[:pos] + char + word[pos:]

        elif typo_type == "adjacent":
            # Replace with keyboard-adjacent character
            pos = self.rng.randint(0, len(word) - 1)
            char = word[pos].lower()
            if char in self.KEYBOARD_ADJACENT:
                adjacent = self.KEYBOARD_ADJACENT[char]
                new_char = self.rng.choice(adjacent)
                word = word[:pos] + new_char + word[pos + 1 :]

        elif typo_type == "double" and len(word) >= 2:
            # Double a character
            pos = self.rng.randint(0, len(word) - 1)
            word = word[:pos] + word[pos] + word[pos:]

        words[idx] = word
        return " ".join(words)

    def remove_space(self, text: str) -> str:
        """Remove a space from the text (common error)."""
        parts = text.split()
        if len(parts) < 2:
            return text

        # Join two random adjacent parts
        idx = self.rng.randint(0, len(parts) - 2)
        parts[idx] = parts[idx] + parts[idx + 1]
        parts.pop(idx + 1)
        return " ".join(parts)

    def wrong_flag(self, text: str) -> str:
        """Introduce a wrong flag (single dash vs double dash)."""
        if " --" in text and self.rng.random() > 0.5:
            return text.replace(" --", " -", 1)
        elif " -" in text and " --" not in text:
            return text.replace(" -", " --", 1)
        return text

    def get_template_entries(self, shell: str, category: str) -> list[dict]:
        """Get template entries for a shell and category."""
        if shell not in self.templates:
            return []

        template = self.templates[shell]
        if category in template:
            entries = template[category]
            return entries if isinstance(entries, list) else []
        return []

    def expand_template(self, template: str, variables: dict[str, str]) -> str:
        """Expand a template string with variables."""
        result = template
        for key, value in variables.items():
            result = result.replace(f"{{{key}}}", value)
        return result

    @abstractmethod
    def generate(self, count: int) -> list[TrainingExample]:
        """Generate training examples."""
        pass

    def save_jsonl(self, examples: list[TrainingExample], output_path: str | Path) -> None:
        """Save examples to JSONL file in chat format."""
        output_path = Path(output_path)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        with open(output_path, "w", encoding="utf-8") as f:
            for example in examples:
                f.write(json.dumps(example.to_chat_format(), ensure_ascii=False) + "\n")

    def save_analysis_jsonl(
        self, examples: list[TrainingExample], output_path: str | Path
    ) -> None:
        """Save examples to JSONL file with full metadata for analysis."""
        output_path = Path(output_path)
        output_path.parent.mkdir(parents=True, exist_ok=True)

        with open(output_path, "w", encoding="utf-8") as f:
            for example in examples:
                f.write(json.dumps(example.to_dict(), ensure_ascii=False) + "\n")


# Common variable placeholders for templates (expanded for diversity)
COMMON_VARIABLES = {
    "path": [
        "mydir", "project", "src", "docs", "test", "build", "dist", "tmp",
        "app", "lib", "bin", "config", "scripts", "utils", "core", "api",
        "frontend", "backend", "server", "client", "shared", "common",
        "assets", "static", "public", "private", "internal", "external",
        "/home/user/project", "/var/log", "~/Documents", "./src/components",
        "/opt/app", "/etc/nginx", "/usr/local/bin", "~/projects/myapp",
        "./packages/core", "../shared/utils", "node_modules", "vendor",
        "migrations", "seeds", "fixtures", "templates", "views", "models",
        "controllers", "services", "repositories", "handlers", "middleware",
    ],
    "file": [
        "file.txt", "data.json", "config.yaml", "README.md", "script.py",
        "main.go", "index.js", "app.ts", "Makefile", "Dockerfile", "package.json",
        "requirements.txt", "setup.py", "pyproject.toml", "Cargo.toml", "go.mod",
        "tsconfig.json", "webpack.config.js", ".env", ".gitignore", "LICENSE",
        "CHANGELOG.md", "server.js", "handler.py", "utils.ts", "helpers.rb",
        "database.sql", "schema.prisma", "docker-compose.yml", "nginx.conf",
        "app.py", "main.rs", "lib.rs", "mod.rs", "index.html", "styles.css",
        "test_main.py", "spec.ts", "benchmark.go", "Gemfile", "composer.json",
    ],
    "package": [
        "numpy", "pandas", "requests", "flask", "django", "fastapi", "pytest",
        "react", "express", "lodash", "axios", "moment", "typescript", "webpack",
        "vim", "htop", "curl", "jq", "git", "docker", "kubernetes", "terraform",
        "boto3", "sqlalchemy", "celery", "redis", "pytest-cov", "black", "mypy",
        "eslint", "prettier", "jest", "mocha", "chai", "sinon", "enzyme",
        "tensorflow", "pytorch", "scikit-learn", "matplotlib", "seaborn",
        "gin", "echo", "fiber", "actix-web", "tokio", "serde", "clap",
        "spring-boot", "hibernate", "junit", "mockito", "gradle", "maven",
    ],
    "branch": [
        "main", "master", "develop", "dev", "staging", "production", "release",
        "feature/new-feature", "feature/auth", "feature/api", "feature/ui",
        "feature/payment", "feature/search", "feature/notifications",
        "bugfix/fix-issue", "bugfix/login", "bugfix/memory-leak", "bugfix/typo",
        "hotfix/security", "hotfix/critical", "hotfix/performance",
        "release/v1.0", "release/v2.0", "release/v1.2.3", "release/2024-01",
        "experiment/ml-model", "test/integration", "ci/pipeline", "docs/api",
    ],
    "message": [
        "Initial commit", "Fix bug", "Add feature", "Update docs", "Refactor code",
        "Fix typo", "WIP", "Merge branch", "Hotfix", "Release v1.0",
        "Add unit tests", "Fix linting errors", "Update dependencies",
        "Improve performance", "Add error handling", "Fix security issue",
        "Add logging", "Clean up code", "Remove dead code", "Add comments",
        "Fix CI pipeline", "Update README", "Add Docker support", "Fix build",
        "Implement auth", "Add API endpoint", "Fix database query", "Optimize query",
    ],
    "pattern": [
        "*.py", "*.txt", "*.js", "*.ts", "*.go", "*.rs", "*.java", "*.rb",
        "*.json", "*.yaml", "*.yml", "*.md", "*.html", "*.css", "*.scss",
        "error", "TODO", "FIXME", "BUG", "HACK", "XXX", "NOTE", "WARNING",
        "import", "function", "class", "def ", "const ", "let ", "var ",
        "async", "await", "return", "throw", "catch", "try", "except",
        "password", "secret", "api_key", "token", "debug", "console.log",
    ],
    "url": [
        "https://example.com", "https://api.github.com", "http://localhost:8080",
        "https://google.com", "https://npmjs.com", "https://pypi.org",
        "http://localhost:3000", "http://localhost:5000", "http://127.0.0.1:8000",
        "https://api.stripe.com", "https://s3.amazonaws.com", "https://cdn.example.com",
        "https://raw.githubusercontent.com/user/repo/main/file.txt",
        "https://registry.npmjs.org", "https://hub.docker.com",
    ],
    "host": [
        "localhost", "example.com", "192.168.1.1", "server.local", "prod-server",
        "dev-server", "staging-server", "db-server", "api-server", "web-server",
        "10.0.0.1", "172.16.0.1", "192.168.0.100", "192.168.1.50",
        "myapp.local", "test.example.com", "api.example.com", "db.example.com",
        "ec2-user@aws-instance", "admin@server.company.com",
    ],
    "user": [
        "root", "admin", "user", "deploy", "ubuntu", "ec2-user", "centos",
        "www-data", "nginx", "postgres", "mysql", "redis", "jenkins",
        "circleci", "github-actions", "developer", "devops", "sysadmin",
    ],
    "container": [
        "nginx", "postgres", "redis", "app", "web", "api", "worker", "scheduler",
        "mongo", "mysql", "mariadb", "elasticsearch", "kibana", "grafana",
        "prometheus", "rabbitmq", "kafka", "zookeeper", "consul", "vault",
        "jenkins", "gitlab", "nexus", "sonarqube", "traefik", "haproxy",
    ],
    "image": [
        "nginx:latest", "nginx:1.25", "nginx:alpine", "python:3.11", "python:3.12",
        "python:3.11-slim", "python:3.11-alpine", "node:20", "node:20-alpine",
        "node:18-slim", "postgres:16", "postgres:15-alpine", "mysql:8",
        "redis:7", "redis:alpine", "mongo:7", "ubuntu:22.04", "ubuntu:24.04",
        "alpine:3.19", "debian:bookworm", "golang:1.21", "rust:1.75",
    ],
    "process": [
        "nginx", "python", "node", "java", "docker", "postgres", "mysql", "redis",
        "apache2", "httpd", "gunicorn", "uvicorn", "pm2", "supervisord",
        "systemd", "cron", "sshd", "mongod", "elasticsearch", "kafka",
    ],
    "pid": [str(i) for i in range(1000, 9999, 123)],  # Generate varied PIDs
    "n": ["1", "3", "5", "10", "15", "20", "25", "50", "100", "200", "500", "1000"],
    "size": ["10", "50", "100", "500", "1024", "2048", "4096", "10240"],
    "service": [
        "nginx", "docker", "ssh", "postgresql", "mysql", "redis", "mongodb",
        "elasticsearch", "rabbitmq", "kafka", "apache2", "httpd", "cron",
        "systemd-journald", "NetworkManager", "bluetooth", "cups", "avahi-daemon",
    ],
    "var": [
        "PATH", "HOME", "USER", "PWD", "SHELL", "TERM", "LANG", "LC_ALL",
        "EDITOR", "VISUAL", "PAGER", "TMPDIR", "XDG_CONFIG_HOME",
        "NODE_ENV", "PYTHONPATH", "GOPATH", "JAVA_HOME", "RUST_BACKTRACE",
        "DATABASE_URL", "API_KEY", "SECRET_KEY", "AWS_REGION", "DEBUG",
    ],
    "value": [
        "test", "production", "debug", "/usr/local/bin", "true", "false",
        "development", "staging", "1", "0", "yes", "no", "enabled", "disabled",
        "/opt/app/bin", "~/bin", "/home/user/.local/bin", "info", "warning",
    ],
    "name": [
        "myfunc", "helper", "process_data", "main", "run", "init", "setup",
        "cleanup", "validate", "transform", "parse", "format", "convert",
        "handle_request", "process_event", "send_notification", "update_cache",
        "fetch_data", "save_record", "delete_item", "create_user", "get_config",
    ],
    "old": ["foo", "bar", "old_name", "deprecated", "legacy", "v1", "temp", "bak"],
    "new": ["baz", "qux", "new_name", "updated", "modern", "v2", "current", "latest"],
    "command": ["ls", "cat", "grep", "find", "echo", "sed", "awk", "cut", "sort", "uniq"],
    "data": [
        '{"key": "value"}', '{"name": "test"}', "key=value", '{"id": 1}',
        '{"status": "ok"}', '{"error": null}', "name=test&value=123",
        '{"items": []}', '{"count": 42}', '{"enabled": true}',
    ],
    "drive": ["C", "D", "E", "F", "G"],
    "module": [
        "numpy", "pandas", "PSReadLine", "Az", "AzureRM", "ActiveDirectory",
        "SqlServer", "VMware.PowerCLI", "Pester", "ImportExcel",
    ],
    "cmdlet": [
        "Get-Process", "Get-Service", "Set-Location", "Get-ChildItem", "Get-Content",
        "Set-Content", "New-Item", "Remove-Item", "Copy-Item", "Move-Item",
        "Get-Help", "Get-Command", "Invoke-WebRequest", "ConvertTo-Json",
    ],
    "color": ["red", "green", "blue", "yellow", "cyan", "magenta", "white", "black"],
    "expression": ["1 + 1", "2 * 3", "10 / 2", "5 - 3", "2 ** 8", "100 % 7", "sqrt(16)"],
    "delimiter": [",", ":", ";", "|", "\t", " ", "-", "_", "/", "."],
    "string": [
        "hello world", "test string", "foo,bar,baz", "lorem ipsum", "quick brown fox",
        "sample text", "example data", "test input", "user@email.com", "192.168.1.1",
    ],
    "src": [
        "file.txt", "source/", "./data", "input.json", "backup.tar.gz", "dump.sql",
        "exports/", "downloads/", "~/Documents/report.pdf", "/tmp/data.csv",
    ],
    "dst": [
        "backup/", "output.txt", "./dest", "result.json", "archive/", "processed/",
        "~/backups/", "/mnt/storage/", "s3://bucket/path/", "/var/backup/",
    ],
    "group": ["users", "admin", "www-data", "docker", "wheel", "sudo", "staff", "developers"],
    "content": [
        "Hello World", "Test content", "New line", "Sample text", "Lorem ipsum",
        "Configuration value", "Debug message", "Error log entry", "Status update",
    ],
    "archive": ["backup", "archive", "data", "logs", "exports", "dumps", "snapshots"],
    "dir": [
        "mydir", "project", "backup", "data", "logs", "tmp", "cache", "config",
        "uploads", "downloads", "exports", "imports", "output", "results",
    ],
    "port": ["80", "443", "8080", "3000", "5000", "8000", "5432", "3306", "6379", "27017"],
    "commit": ["HEAD", "HEAD~1", "HEAD~2", "main", "abc123f", "v1.0.0", "origin/main"],
    "repo": ["myrepo", "project", "app", "api", "frontend", "backend", "user/repo"],
    "tag": ["v1.0.0", "v1.0.1", "v2.0.0", "latest", "stable", "beta", "alpha"],
    "key": ["id", "name", "type", "status", "created_at", "updated_at", "value", "data"],
}


def get_random_variable(var_name: str, rng: random.Random) -> str:
    """Get a random value for a variable placeholder."""
    if var_name in COMMON_VARIABLES:
        return rng.choice(COMMON_VARIABLES[var_name])
    return var_name  # Return placeholder name if not found
