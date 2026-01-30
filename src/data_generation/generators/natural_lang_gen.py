"""
Natural Language Generator (DS3).

Generates training examples for natural language to command translations including:
- Imperative partial NL (40%)
- Question-like (20%)
- Mixed NL + command (25%)
- Intent-based (15%)
"""
from __future__ import annotations

from typing import Any

from .base_generator import BaseGenerator, TrainingExample


class NaturalLanguageGenerator(BaseGenerator):
    """Generator for natural language to command translation examples."""

    # Error type distribution
    NL_TYPE_DISTRIBUTION = {
        "imperative": 0.40,
        "question": 0.20,
        "mixed": 0.25,
        "intent": 0.15,
    }

    # Natural language patterns mapped to commands per shell
    NL_PATTERNS = {
        "bash": {
            "imperative": [
                ("list all files", "ls -la"),
                ("list files", "ls"),
                ("show hidden files", "ls -la"),
                ("list files sorted by size", "ls -lS"),
                ("list files sorted by date", "ls -lt"),
                ("show current directory", "pwd"),
                ("go to home", "cd ~"),
                ("go up one directory", "cd .."),
                ("go to parent", "cd .."),
                ("create directory {path}", "mkdir -p {path}"),
                ("make folder {path}", "mkdir -p {path}"),
                ("delete file {file}", "rm {file}"),
                ("remove {file}", "rm {file}"),
                ("delete folder {path}", "rm -rf {path}"),
                ("copy {src} to {dst}", "cp {src} {dst}"),
                ("copy directory {src} to {dst}", "cp -r {src} {dst}"),
                ("move {src} to {dst}", "mv {src} {dst}"),
                ("rename {old} to {new}", "mv {old} {new}"),
                ("show {file}", "cat {file}"),
                ("print {file}", "cat {file}"),
                ("display {file} contents", "cat {file}"),
                ("edit {file}", "nano {file}"),
                ("open {file} in vim", "vim {file}"),
                ("find files named {pattern}", "find . -name '{pattern}'"),
                ("search for {pattern}", "grep -r '{pattern}' ."),
                ("search {pattern} in {file}", "grep '{pattern}' {file}"),
                ("show running processes", "ps aux"),
                ("list processes", "ps aux"),
                ("kill process {pid}", "kill -9 {pid}"),
                ("stop {process}", "pkill {process}"),
                ("show disk space", "df -h"),
                ("check disk usage", "df -h"),
                ("show memory usage", "free -h"),
                ("check memory", "free -h"),
                ("show system info", "uname -a"),
                ("download {url}", "curl -O {url}"),
                ("fetch {url}", "curl {url}"),
                ("ping {host}", "ping -c 4 {host}"),
                ("check if {host} is up", "ping -c 4 {host}"),
                ("connect to {host}", "ssh {user}@{host}"),
                ("ssh to {host}", "ssh {user}@{host}"),
                ("compress {dir}", "tar -czf {dir}.tar.gz {dir}"),
                ("zip {dir}", "zip -r {dir}.zip {dir}"),
                ("extract {file}", "tar -xzf {file}"),
                ("unzip {file}", "unzip {file}"),
                ("make {file} executable", "chmod +x {file}"),
                ("change owner of {file} to {user}", "chown {user} {file}"),
                ("count lines in {file}", "wc -l {file}"),
                ("sort {file}", "sort {file}"),
                ("remove duplicates from {file}", "sort -u {file}"),
                ("show first {n} lines of {file}", "head -n {n} {file}"),
                ("show last {n} lines of {file}", "tail -n {n} {file}"),
                ("follow {file}", "tail -f {file}"),
                ("watch {file} for changes", "tail -f {file}"),
            ],
            "question": [
                ("what's my ip", "ip addr show"),
                ("what is my ip address", "ip addr show"),
                ("how much disk space", "df -h"),
                ("how much memory", "free -h"),
                ("what's running", "ps aux"),
                ("what processes are running", "ps aux"),
                ("where am i", "pwd"),
                ("what directory", "pwd"),
                ("who am i", "whoami"),
                ("what user am i", "whoami"),
                ("what's the date", "date"),
                ("what time is it", "date"),
                ("what's in this directory", "ls -la"),
                ("what's the hostname", "hostname"),
                ("what version of linux", "uname -r"),
                ("what kernel version", "uname -r"),
                ("is {process} running", "pgrep {process}"),
                ("what's listening on port {n}", "ss -tulpn | grep {n}"),
                ("what's using port {n}", "lsof -i :{n}"),
            ],
            "mixed": [
                ("grep for {pattern} in {file}", "grep '{pattern}' {file}"),
                ("grep {pattern} recursively", "grep -r '{pattern}' ."),
                ("find large files", "find . -type f -size +100M"),
                ("find files modified today", "find . -mtime 0"),
                ("find empty files", "find . -empty"),
                ("cat {file} and grep {pattern}", "grep '{pattern}' {file}"),
                ("ls and sort by size", "ls -lS"),
                ("ps and filter {process}", "ps aux | grep {process}"),
                ("curl {url} and save", "curl -O {url}"),
                ("wget {url}", "wget {url}"),
                ("tar extract {file}", "tar -xzf {file}"),
                ("chmod 755 {file}", "chmod 755 {file}"),
                ("chown {user}:{group} {file}", "chown {user}:{group} {file}"),
            ],
            "intent": [
                ("clean up docker", "docker system prune -a"),
                ("remove unused docker images", "docker image prune -a"),
                ("update system", "sudo apt update && sudo apt upgrade -y"),
                ("update packages", "sudo apt update && sudo apt upgrade -y"),
                ("install {package}", "sudo apt install {package}"),
                ("uninstall {package}", "sudo apt remove {package}"),
                ("restart {service}", "sudo systemctl restart {service}"),
                ("start {service}", "sudo systemctl start {service}"),
                ("stop {service}", "sudo systemctl stop {service}"),
                ("check {service} status", "sudo systemctl status {service}"),
                ("enable {service}", "sudo systemctl enable {service}"),
                ("clear terminal", "clear"),
                ("clear screen", "clear"),
                ("exit terminal", "exit"),
                ("logout", "exit"),
                ("reboot", "sudo reboot"),
                ("shutdown", "sudo shutdown -h now"),
                ("backup {src} to {dst}", "rsync -av {src} {dst}"),
                ("sync {src} to {dst}", "rsync -av {src} {dst}"),
            ],
        },
        "zsh": {
            # Mostly same as bash, with some zsh-specific
            "imperative": [
                ("list all files", "ls -la"),
                ("list all python files", "ls **/*.py"),
                ("show current directory", "pwd"),
                ("create directory {path}", "mkdir -p {path}"),
                ("find files named {pattern}", "ls **/{pattern}"),
            ],
            "question": [
                ("what's my ip", "ip addr show"),
                ("how much disk space", "df -h"),
            ],
            "mixed": [
                ("grep for {pattern} in python files", "grep -r '{pattern}' **/*.py"),
            ],
            "intent": [
                ("update system", "sudo apt update && sudo apt upgrade -y"),
            ],
        },
        "powershell": {
            "imperative": [
                ("list all files", "Get-ChildItem"),
                ("list files", "Get-ChildItem"),
                ("show hidden files", "Get-ChildItem -Force"),
                ("show current directory", "Get-Location"),
                ("go to home", "Set-Location ~"),
                ("go up one directory", "Set-Location .."),
                ("create directory {path}", "New-Item -ItemType Directory -Path {path}"),
                ("make folder {path}", "New-Item -ItemType Directory -Path {path}"),
                ("delete file {file}", "Remove-Item {file}"),
                ("remove {file}", "Remove-Item {file}"),
                ("delete folder {path}", "Remove-Item {path} -Recurse -Force"),
                ("copy {src} to {dst}", "Copy-Item {src} -Destination {dst}"),
                ("move {src} to {dst}", "Move-Item {src} -Destination {dst}"),
                ("show {file}", "Get-Content {file}"),
                ("print {file}", "Get-Content {file}"),
                ("show running processes", "Get-Process"),
                ("list processes", "Get-Process"),
                ("kill process {name}", "Stop-Process -Name {name}"),
                ("stop {process}", "Stop-Process -Name {process}"),
                ("show services", "Get-Service"),
                ("restart {service}", "Restart-Service {service}"),
            ],
            "question": [
                ("what's my ip", "Get-NetIPAddress"),
                ("how much disk space", "Get-Volume"),
                ("what's running", "Get-Process"),
                ("where am i", "Get-Location"),
                ("who am i", "$env:USERNAME"),
                ("what computer name", "$env:COMPUTERNAME"),
            ],
            "mixed": [
                ("find {pattern} in files", "Get-ChildItem -Recurse | Select-String '{pattern}'"),
                ("grep for {pattern}", "Select-String -Path * -Pattern '{pattern}'"),
                ("filter processes by cpu", "Get-Process | Sort-Object CPU -Descending"),
            ],
            "intent": [
                ("install {package}", "winget install {package}"),
                ("update system", "winget upgrade --all"),
                ("clear terminal", "Clear-Host"),
                ("exit terminal", "exit"),
            ],
        },
        "cmd": {
            "imperative": [
                ("list all files", "dir /a"),
                ("list files", "dir"),
                ("show current directory", "cd"),
                ("go up one directory", "cd .."),
                ("create directory {path}", "mkdir {path}"),
                ("delete file {file}", "del {file}"),
                ("delete folder {path}", "rmdir /s /q {path}"),
                ("copy {src} to {dst}", "copy {src} {dst}"),
                ("move {src} to {dst}", "move {src} {dst}"),
                ("show {file}", "type {file}"),
                ("show running processes", "tasklist"),
                ("kill process {name}", "taskkill /im {name} /f"),
            ],
            "question": [
                ("what's my ip", "ipconfig"),
                ("what's running", "tasklist"),
                ("where am i", "cd"),
                ("who am i", "whoami"),
            ],
            "mixed": [
                ("find {pattern} in files", "findstr /s \"{pattern}\" *.*"),
            ],
            "intent": [
                ("clear screen", "cls"),
                ("reboot", "shutdown /r /t 0"),
                ("shutdown", "shutdown /s /t 0"),
            ],
        },
        "fish": {
            "imperative": [
                ("list all files", "ls -la"),
                ("show current directory", "pwd"),
                ("set variable {var} to {value}", "set {var} {value}"),
                ("export {var}", "set -x {var} {value}"),
            ],
            "question": [
                ("what's my ip", "ip addr show"),
            ],
            "mixed": [
                ("calculate {expression}", "math '{expression}'"),
            ],
            "intent": [
                ("update completions", "fish_update_completions"),
            ],
        },
        "tcsh": {
            "imperative": [
                ("list all files", "ls -la"),
                ("set environment variable {var}", "setenv {var} {value}"),
            ],
            "question": [
                ("what's my ip", "ifconfig"),
            ],
            "mixed": [],
            "intent": [],
        },
    }

    # Git patterns (common across shells)
    GIT_PATTERNS = [
        ("check git status", "git status"),
        ("what changed", "git status"),
        ("show changes", "git diff"),
        ("stage all changes", "git add ."),
        ("add all files", "git add ."),
        ("commit with message {message}", "git commit -m '{message}'"),
        ("save changes as {message}", "git commit -m '{message}'"),
        ("push changes", "git push"),
        ("push to {branch}", "git push origin {branch}"),
        ("pull latest", "git pull"),
        ("pull from {branch}", "git pull origin {branch}"),
        ("switch to {branch}", "git checkout {branch}"),
        ("create branch {branch}", "git checkout -b {branch}"),
        ("new branch {branch}", "git checkout -b {branch}"),
        ("merge {branch}", "git merge {branch}"),
        ("show commit history", "git log --oneline"),
        ("git history", "git log --oneline"),
        ("stash changes", "git stash"),
        ("apply stash", "git stash pop"),
        ("clone {url}", "git clone {url}"),
        ("download repo {url}", "git clone {url}"),
        ("undo last commit", "git reset --soft HEAD~1"),
        ("discard changes", "git checkout ."),
    ]

    # Docker patterns (common across shells)
    DOCKER_PATTERNS = [
        ("list containers", "docker ps"),
        ("show all containers", "docker ps -a"),
        ("list images", "docker images"),
        ("run {image}", "docker run -it {image}"),
        ("start {container}", "docker start {container}"),
        ("stop {container}", "docker stop {container}"),
        ("remove container {container}", "docker rm {container}"),
        ("remove image {image}", "docker rmi {image}"),
        ("shell into {container}", "docker exec -it {container} bash"),
        ("enter container {container}", "docker exec -it {container} bash"),
        ("show {container} logs", "docker logs {container}"),
        ("build docker image", "docker build -t {image} ."),
        ("docker compose up", "docker-compose up -d"),
        ("start services", "docker-compose up -d"),
        ("stop services", "docker-compose down"),
        ("clean up docker", "docker system prune -a"),
    ]

    # Kubernetes patterns (common across shells)
    K8S_PATTERNS = [
        ("check pod status", "kubectl get pods"),
        ("list all pods", "kubectl get pods -A"),
        ("show pods in all namespaces", "kubectl get pods --all-namespaces"),
        ("what pods are running", "kubectl get pods"),
        ("describe pod {pod}", "kubectl describe pod {pod}"),
        ("get pod details {pod}", "kubectl describe pod {pod}"),
        ("show pod logs {pod}", "kubectl logs {pod}"),
        ("tail pod logs {pod}", "kubectl logs -f {pod}"),
        ("follow logs for {pod}", "kubectl logs -f {pod}"),
        ("exec into pod {pod}", "kubectl exec -it {pod} -- bash"),
        ("shell into pod {pod}", "kubectl exec -it {pod} -- bash"),
        ("restart deployment {name}", "kubectl rollout restart deployment/{name}"),
        ("scale deployment {name} to {n}", "kubectl scale deployment/{name} --replicas={n}"),
        ("scale {name} to {n} replicas", "kubectl scale deployment/{name} --replicas={n}"),
        ("get deployments", "kubectl get deployments"),
        ("list deployments", "kubectl get deployments"),
        ("show services", "kubectl get services"),
        ("list services", "kubectl get svc"),
        ("apply config {file}", "kubectl apply -f {file}"),
        ("deploy {file}", "kubectl apply -f {file}"),
        ("delete pod {pod}", "kubectl delete pod {pod}"),
        ("port forward {pod} {n}", "kubectl port-forward {pod} {n}:{n}"),
        ("forward port {n} to pod {pod}", "kubectl port-forward {pod} {n}:{n}"),
        ("get configmaps", "kubectl get configmaps"),
        ("get secrets", "kubectl get secrets"),
        ("describe service {name}", "kubectl describe svc {name}"),
        ("check cluster info", "kubectl cluster-info"),
        ("get nodes", "kubectl get nodes"),
        ("show node status", "kubectl get nodes"),
    ]

    # Multi-step workflow patterns
    WORKFLOW_PATTERNS = [
        ("deploy to production", "git pull && npm run build && npm run deploy"),
        ("setup dev environment", "npm install && npm run setup && npm start"),
        ("clean and rebuild", "npm run clean && npm run build"),
        ("run tests then push", "npm test && git push"),
        ("build and deploy", "docker build -t {image} . && docker push {image}"),
        ("backup database", "pg_dump -U postgres {name} > {name}_backup.sql"),
        ("restore database", "psql -U postgres {name} < {file}"),
        ("update and upgrade", "sudo apt update && sudo apt upgrade -y"),
        ("clean npm cache", "npm cache clean --force && rm -rf node_modules && npm install"),
        ("reset git branch", "git fetch origin && git reset --hard origin/{branch}"),
        ("deploy container", "docker-compose down && docker-compose pull && docker-compose up -d"),
        ("full stack deploy", "git pull && npm install && npm run build && pm2 restart all"),
        ("restart services", "sudo systemctl stop {service} && sudo systemctl start {service}"),
    ]

    # Debugging/monitoring patterns
    DEBUG_PATTERNS = [
        ("check open ports", "ss -tulpn"),
        ("list listening ports", "netstat -tlnp"),
        ("monitor memory", "watch -n 1 free -h"),
        ("watch disk space", "watch -n 1 df -h"),
        ("monitor cpu", "htop"),
        ("check load average", "uptime"),
        ("show system logs", "journalctl -xe"),
        ("check recent logs", "journalctl -n 100"),
        ("debug network to {host}", "ping -c 4 {host} && traceroute {host}"),
        ("test dns {host}", "nslookup {host}"),
        ("check dns resolution {host}", "dig {host}"),
        ("show firewall rules", "sudo iptables -L"),
        ("check failed logins", "grep 'Failed' /var/log/auth.log | tail -20"),
        ("show last logins", "last -10"),
        ("check disk io", "iostat -x 1 5"),
        ("monitor network traffic", "iftop"),
        ("trace system calls", "strace -p {pid}"),
        ("profile process {pid}", "top -p {pid}"),
    ]

    def __init__(self, templates_dir: str, seed: int = 42):
        """Initialize the natural language generator."""
        super().__init__(templates_dir, seed)

    def generate(self, count: int) -> list[TrainingExample]:
        """Generate training examples for natural language translations."""
        examples = []

        # Calculate count per shell based on weights
        shell_counts = {
            shell: int(count * weight)
            for shell, weight in self.SHELL_WEIGHTS.items()
        }

        total = sum(shell_counts.values())
        if total < count:
            shell_counts["bash"] += count - total

        for shell, shell_count in shell_counts.items():
            shell_examples = self._generate_for_shell(shell, shell_count)
            examples.extend(shell_examples)

        # Filter out any null corrections (NL input should never equal command output)
        examples = [
            ex for ex in examples
            if ex.incorrect_command.strip() != ex.correct_command.strip()
        ]

        self.rng.shuffle(examples)
        return examples[:count]

    def _generate_for_shell(self, shell: str, count: int) -> list[TrainingExample]:
        """Generate examples for a specific shell."""
        examples = []
        patterns = self.NL_PATTERNS.get(shell, self.NL_PATTERNS.get("bash", {}))

        for _ in range(count):
            nl_type = self._select_nl_type()

            # Get patterns for this type
            type_patterns = patterns.get(nl_type, [])

            # Add git, docker, k8s, workflow, and debug patterns for relevant types
            if nl_type in ["imperative", "intent"] and shell in ["bash", "zsh", "powershell"]:
                if self.rng.random() < 0.25:
                    type_patterns = self.GIT_PATTERNS + type_patterns
                if self.rng.random() < 0.15:
                    type_patterns = self.DOCKER_PATTERNS + type_patterns
                if self.rng.random() < 0.20:
                    type_patterns = self.K8S_PATTERNS + type_patterns
                if self.rng.random() < 0.15:
                    type_patterns = self.WORKFLOW_PATTERNS + type_patterns
                if self.rng.random() < 0.10:
                    type_patterns = self.DEBUG_PATTERNS + type_patterns

            if not type_patterns:
                # Fallback to imperative
                type_patterns = patterns.get("imperative", [])

            if not type_patterns:
                continue

            nl_text, command = self.rng.choice(type_patterns)

            # Extract variables first, then expand both with same values
            variables = self._extract_variables(nl_text + " " + command)
            nl_text = self.expand_template(nl_text, variables)
            command = self.expand_template(command, variables)

            # Add variations to natural language
            nl_text = self._add_nl_variation(nl_text, nl_type)

            examples.append(TrainingExample(
                shell=shell,
                incorrect_command=nl_text,
                correct_command=command,
                category="natural_language",
                error_type=nl_type,
                source="nl_pattern",
            ))

        return examples

    def _select_nl_type(self) -> str:
        """Select a natural language type based on distribution."""
        types = list(self.NL_TYPE_DISTRIBUTION.keys())
        weights = list(self.NL_TYPE_DISTRIBUTION.values())
        return self.rng.choices(types, weights=weights, k=1)[0]

    def _extract_variables(self, text: str) -> dict[str, str]:
        """Extract variable placeholders and generate consistent values."""
        import re
        from .base_generator import get_random_variable

        variables = {}
        placeholders = re.findall(r"\{(\w+)\}", text)

        for placeholder in placeholders:
            if placeholder not in variables:
                variables[placeholder] = get_random_variable(placeholder, self.rng)

        return variables

    def _add_nl_variation(self, text: str, nl_type: str) -> str:
        """Add variations to natural language input."""
        variations = {
            "imperative": [
                lambda t: t,  # Keep as is
                lambda t: "please " + t,
                lambda t: "can you " + t,
                lambda t: t + " please",
                lambda t: "I want to " + t,
                lambda t: "I need to " + t,
            ],
            "question": [
                lambda t: t,
                lambda t: t + "?",
                lambda t: "can you tell me " + t,
                lambda t: "I want to know " + t,
            ],
            "mixed": [
                lambda t: t,
                lambda t: "do a " + t,
                lambda t: "run " + t,
            ],
            "intent": [
                lambda t: t,
                lambda t: "I want to " + t,
                lambda t: "help me " + t,
                lambda t: "please " + t,
            ],
        }

        type_variations = variations.get(nl_type, [lambda t: t])
        variation = self.rng.choice(type_variations)
        return variation(text)


if __name__ == "__main__":
    import sys

    templates_dir = sys.argv[1] if len(sys.argv) > 1 else "src/data_generation/templates"

    generator = NaturalLanguageGenerator(templates_dir)
    examples = generator.generate(50)

    print(f"Generated {len(examples)} examples")
    print("\nSample examples:")
    for example in examples[:10]:
        print(f"\nShell: {example.shell}")
        print(f"Type: {example.error_type}")
        print(f"Input: {example.incorrect_command}")
        print(f"Output: {example.correct_command}")
