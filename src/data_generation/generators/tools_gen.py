"""
Tools Generator (DS4).

Generates training examples for top 100 CLI tools corrections.
Covers version control, package managers, containers, cloud CLIs,
build tools, and more.
"""
from __future__ import annotations

from typing import Any

from .base_generator import BaseGenerator, TrainingExample


class ToolsGenerator(BaseGenerator):
    """Generator for top 100 CLI tools correction examples."""

    # Maximum percentage of single-char corrections allowed
    MAX_SINGLE_CHAR_PERCENTAGE = 0.05

    # Tool categories with commands and common errors
    TOOLS = {
        # Version Control (git, gh, svn, hg)
        "version_control": {
            "git": [
                {"correct": "git status", "errors": ["gti status", "git stauts", "git stats"]},
                {"correct": "git add .", "errors": ["git add", "git add -a", "gti add ."]},
                {"correct": "git commit -m '{message}'", "errors": ["git commit '{message}'", "git comit -m '{message}'", "git commit -m {message}"]},
                {"correct": "git push origin {branch}", "errors": ["git push", "git psuh origin {branch}", "git push orgin {branch}"]},
                {"correct": "git pull origin {branch}", "errors": ["git pull", "git pul origin {branch}", "git pull orign {branch}"]},
                {"correct": "git checkout {branch}", "errors": ["git checkotu {branch}", "git checkout", "git ckeckout {branch}"]},
                {"correct": "git checkout -b {branch}", "errors": ["git checkout -B {branch}", "git checkout -b"]},
                {"correct": "git merge {branch}", "errors": ["git merg {branch}", "git merge"]},
                {"correct": "git rebase {branch}", "errors": ["git reabse {branch}", "git rebase"]},
                {"correct": "git stash", "errors": ["git stach", "git stas"]},
                {"correct": "git stash pop", "errors": ["git stash apply", "git stash ppo"]},
                {"correct": "git log --oneline", "errors": ["git log --online", "git log -oneline"]},
                {"correct": "git diff", "errors": ["git dif", "git diff."]},
                {"correct": "git clone {url}", "errors": ["git clone", "git cloen {url}"]},
                {"correct": "git branch -d {branch}", "errors": ["git branch -D {branch}", "git branch -d"]},
                {"correct": "git remote -v", "errors": ["git remote", "git remote -V"]},
                {"correct": "git fetch --all", "errors": ["git fetch", "git fecth --all"]},
                {"correct": "git reset --hard HEAD", "errors": ["git reset --hard", "git reset HEAD"]},
                {"correct": "git cherry-pick {commit}", "errors": ["git cherry-pick", "git cherrypick {commit}"]},
            ],
            "gh": [
                {"correct": "gh pr create", "errors": ["gh pr creat", "gh create pr"]},
                {"correct": "gh pr list", "errors": ["gh pr ls", "gh prs"]},
                {"correct": "gh issue create", "errors": ["gh issue creat", "gh create issue"]},
                {"correct": "gh repo clone {repo}", "errors": ["gh clone {repo}", "gh repo clone"]},
            ],
        },

        # Package Managers
        "package_managers": {
            "npm": [
                {"correct": "npm install", "errors": ["npm instal", "npm i", "npn install"]},
                {"correct": "npm install {package}", "errors": ["npm instal {package}", "npm add {package}"]},
                {"correct": "npm install --save-dev {package}", "errors": ["npm install -D {package}", "npm i --save-dev {package}"]},
                {"correct": "npm run {script}", "errors": ["npm {script}", "npm run"]},
                {"correct": "npm start", "errors": ["npm strart", "npm star"]},
                {"correct": "npm test", "errors": ["npm tets", "npm tests"]},
                {"correct": "npm update", "errors": ["npm updaet", "npm upgrade"]},
                {"correct": "npm uninstall {package}", "errors": ["npm remove {package}", "npm uninstal {package}"]},
            ],
            "yarn": [
                {"correct": "yarn install", "errors": ["yarn instal", "yarn"]},
                {"correct": "yarn add {package}", "errors": ["yarn install {package}", "yarn ad {package}"]},
                {"correct": "yarn add --dev {package}", "errors": ["yarn add -D {package}", "yarn add -dev {package}"]},
                {"correct": "yarn run {script}", "errors": ["yarn {script}"]},
            ],
            "pip": [
                {"correct": "pip install {package}", "errors": ["pip instal {package}", "pip istall {package}"]},
                {"correct": "pip install -r requirements.txt", "errors": ["pip install requirements.txt", "pip install -r requirements"]},
                {"correct": "pip install --upgrade {package}", "errors": ["pip upgrade {package}", "pip install -U {package}"]},
                {"correct": "pip uninstall {package}", "errors": ["pip remove {package}", "pip uninstal {package}"]},
                {"correct": "pip freeze", "errors": ["pip freez", "pip list --format=freeze"]},
                {"correct": "pip list", "errors": ["pip ls", "pip lsit"]},
            ],
            "cargo": [
                {"correct": "cargo build", "errors": ["cargo biuld", "cargo buidl"]},
                {"correct": "cargo run", "errors": ["cargo rnu", "cargo exec"]},
                {"correct": "cargo test", "errors": ["cargo tets", "cargo tests"]},
                {"correct": "cargo add {package}", "errors": ["cargo install {package}", "cargo ad {package}"]},
                {"correct": "cargo update", "errors": ["cargo updaet", "cargo upgrade"]},
            ],
            "brew": [
                {"correct": "brew install {package}", "errors": ["brew instal {package}", "brew add {package}"]},
                {"correct": "brew update", "errors": ["brew updaet", "brew refresh"]},
                {"correct": "brew upgrade", "errors": ["brew upgarde", "brew update --all"]},
                {"correct": "brew uninstall {package}", "errors": ["brew remove {package}", "brew uninstal {package}"]},
                {"correct": "brew list", "errors": ["brew ls", "brew lsit"]},
                {"correct": "brew search {package}", "errors": ["brew find {package}", "brew serach {package}"]},
            ],
            "apt": [
                {"correct": "sudo apt update", "errors": ["apt update", "sudo apt updaet"]},
                {"correct": "sudo apt upgrade -y", "errors": ["apt upgrade", "sudo apt upgrade"]},
                {"correct": "sudo apt install {package}", "errors": ["apt install {package}", "sudo apt instal {package}"]},
                {"correct": "sudo apt remove {package}", "errors": ["apt remove {package}", "sudo apt uninstall {package}"]},
                {"correct": "apt search {package}", "errors": ["apt find {package}", "apt serach {package}"]},
            ],
            "pacman": [
                {"correct": "sudo pacman -Syu", "errors": ["pacman -Syu", "sudo pacman -syu"]},
                {"correct": "sudo pacman -S {package}", "errors": ["pacman -S {package}", "sudo pacman -s {package}"]},
                {"correct": "sudo pacman -R {package}", "errors": ["pacman -R {package}", "sudo pacman -r {package}"]},
                {"correct": "pacman -Ss {package}", "errors": ["pacman -ss {package}", "pacman search {package}"]},
            ],
        },

        # Containers
        "containers": {
            "docker": [
                {"correct": "docker ps", "errors": ["docker pa", "docker ls", "dcoker ps"]},
                {"correct": "docker ps -a", "errors": ["docker ps --all", "docker pa -a"]},
                {"correct": "docker images", "errors": ["docker image", "docker imgs"]},
                {"correct": "docker run -it {image}", "errors": ["docker run {image}", "docker run -i -t {image}"]},
                {"correct": "docker exec -it {container} bash", "errors": ["docker exec {container} bash", "docker exec -it {container}"]},
                {"correct": "docker stop {container}", "errors": ["docker stopp {container}", "docker kill {container}"]},
                {"correct": "docker rm {container}", "errors": ["docker remove {container}", "docker delete {container}"]},
                {"correct": "docker rmi {image}", "errors": ["docker rm {image}", "docker image rm {image}"]},
                {"correct": "docker build -t {image} .", "errors": ["docker build {image} .", "docker build -t {image}"]},
                {"correct": "docker pull {image}", "errors": ["docker pul {image}", "docker get {image}"]},
                {"correct": "docker push {image}", "errors": ["docker psuh {image}", "docker upload {image}"]},
                {"correct": "docker logs {container}", "errors": ["docker log {container}", "docker logs"]},
                {"correct": "docker-compose up -d", "errors": ["docker compose up -d", "docker-compose up"]},
                {"correct": "docker-compose down", "errors": ["docker compose down", "docker-compose stop"]},
                {"correct": "docker system prune -a", "errors": ["docker prune", "docker system prune"]},
            ],
            "kubectl": [
                {"correct": "kubectl get pods", "errors": ["kubectl get pod", "kubeclt get pods", "kubectl pods"]},
                {"correct": "kubectl get pods -A", "errors": ["kubectl get pods --all-namespaces", "kubectl get pods -a"]},
                {"correct": "kubectl describe pod {pod}", "errors": ["kubectl describe {pod}", "kubectl desc pod {pod}"]},
                {"correct": "kubectl logs {pod}", "errors": ["kubectl log {pod}", "kubeclt logs {pod}"]},
                {"correct": "kubectl exec -it {pod} -- bash", "errors": ["kubectl exec {pod} bash", "kubectl exec -it {pod} bash"]},
                {"correct": "kubectl apply -f {file}", "errors": ["kubectl apply {file}", "kubectl create -f {file}"]},
                {"correct": "kubectl delete pod {pod}", "errors": ["kubectl delete {pod}", "kubectl remove pod {pod}"]},
                {"correct": "kubectl get services", "errors": ["kubectl get svc", "kubectl get service"]},
                {"correct": "kubectl get deployments", "errors": ["kubectl get deploy", "kubectl get deployment"]},
                {"correct": "kubectl scale deployment {name} --replicas={n}", "errors": ["kubectl scale {name} --replicas={n}"]},
            ],
            "podman": [
                {"correct": "podman ps", "errors": ["podman pa", "podman ls"]},
                {"correct": "podman run -it {image}", "errors": ["podman run {image}"]},
            ],
        },

        # Cloud CLIs
        "cloud": {
            "aws": [
                {"correct": "aws s3 ls", "errors": ["aws s3 list", "aws s3ls"]},
                {"correct": "aws s3 cp {src} {dst}", "errors": ["aws s3 copy {src} {dst}"]},
                {"correct": "aws ec2 describe-instances", "errors": ["aws ec2 list-instances", "aws ec2 describe-instance"]},
                {"correct": "aws configure", "errors": ["aws config", "aws setup"]},
                {"correct": "aws sts get-caller-identity", "errors": ["aws whoami", "aws sts get-identity"]},
            ],
            "az": [
                {"correct": "az login", "errors": ["az signin", "az auth"]},
                {"correct": "az account list", "errors": ["az accounts", "az account ls"]},
                {"correct": "az vm list", "errors": ["az vms", "az vm ls"]},
                {"correct": "az group list", "errors": ["az groups", "az group ls"]},
            ],
            "gcloud": [
                {"correct": "gcloud auth login", "errors": ["gcloud login", "gcloud signin"]},
                {"correct": "gcloud config set project {project}", "errors": ["gcloud set project {project}"]},
                {"correct": "gcloud compute instances list", "errors": ["gcloud instances list", "gcloud compute list"]},
            ],
            "terraform": [
                {"correct": "terraform init", "errors": ["tf init", "terraform initialize"]},
                {"correct": "terraform plan", "errors": ["tf plan", "terraform preview"]},
                {"correct": "terraform apply", "errors": ["tf apply", "terraform deploy"]},
                {"correct": "terraform destroy", "errors": ["tf destroy", "terraform delete"]},
                {"correct": "terraform fmt", "errors": ["terraform format", "tf fmt"]},
                {"correct": "terraform validate", "errors": ["terraform check", "tf validate"]},
            ],
        },

        # Build Tools
        "build": {
            "make": [
                {"correct": "make", "errors": ["mak", "mae"]},
                {"correct": "make clean", "errors": ["make clen", "make clear"]},
                {"correct": "make install", "errors": ["make instal", "make setup"]},
                {"correct": "make -j{n}", "errors": ["make -j {n}", "make -J{n}"]},
            ],
            "cmake": [
                {"correct": "cmake ..", "errors": ["cmake", "cmake -"]},
                {"correct": "cmake --build .", "errors": ["cmake build", "cmake --build"]},
                {"correct": "cmake -DCMAKE_BUILD_TYPE=Release ..", "errors": ["cmake -DCMAKE_BUILD_TYPE Release .."]},
            ],
            "gradle": [
                {"correct": "gradle build", "errors": ["gradle biuld", "./gradlew build"]},
                {"correct": "./gradlew build", "errors": ["gradlew build", "gradle build"]},
                {"correct": "gradle test", "errors": ["gradle tets", "./gradlew test"]},
                {"correct": "gradle clean", "errors": ["gradle clen", "./gradlew clean"]},
            ],
            "maven": [
                {"correct": "mvn clean install", "errors": ["mvn clean instal", "maven clean install"]},
                {"correct": "mvn package", "errors": ["mvn pack", "maven package"]},
                {"correct": "mvn test", "errors": ["mvn tets", "maven test"]},
                {"correct": "mvn compile", "errors": ["mvn complie", "maven compile"]},
            ],
        },

        # Network Tools
        "network": {
            "curl": [
                {"correct": "curl -X GET {url}", "errors": ["curl {url}", "crul -X GET {url}"]},
                {"correct": "curl -X POST -d '{data}' {url}", "errors": ["curl -X POST {url} -d '{data}'"]},
                {"correct": "curl -O {url}", "errors": ["curl -o {url}", "curl --output {url}"]},
                {"correct": "curl -s {url}", "errors": ["curl --silent {url}", "curl -S {url}"]},
                {"correct": "curl -H 'Content-Type: application/json' {url}", "errors": ["curl -H Content-Type: application/json {url}"]},
            ],
            "wget": [
                {"correct": "wget {url}", "errors": ["wegt {url}", "wget -O {url}"]},
                {"correct": "wget -O {file} {url}", "errors": ["wget {url} -O {file}"]},
                {"correct": "wget -r {url}", "errors": ["wget --recursive {url}"]},
            ],
            "ssh": [
                {"correct": "ssh {user}@{host}", "errors": ["ssh {host}", "shh {user}@{host}"]},
                {"correct": "ssh -i {file} {user}@{host}", "errors": ["ssh {user}@{host} -i {file}"]},
                {"correct": "ssh-keygen -t rsa -b 4096", "errors": ["ssh-keygen", "ssh-keygen -t rsa"]},
            ],
            "scp": [
                {"correct": "scp {src} {user}@{host}:{dst}", "errors": ["scp {src} {host}:{dst}"]},
                {"correct": "scp -r {src} {user}@{host}:{dst}", "errors": ["scp {src} {user}@{host}:{dst}"]},
            ],
            "rsync": [
                {"correct": "rsync -avz {src} {dst}", "errors": ["rsync {src} {dst}", "rsync -av {src} {dst}"]},
                {"correct": "rsync -avz --delete {src} {dst}", "errors": ["rsync -avz {src} {dst} --delete"]},
            ],
        },

        # Text Processing
        "text": {
            "grep": [
                {"correct": "grep '{pattern}' {file}", "errors": ["grep {pattern} {file}", "gerp '{pattern}' {file}"]},
                {"correct": "grep -r '{pattern}' .", "errors": ["grep -R '{pattern}' .", "grep '{pattern}' . -r"]},
                {"correct": "grep -i '{pattern}' {file}", "errors": ["grep -I '{pattern}' {file}"]},
                {"correct": "grep -v '{pattern}' {file}", "errors": ["grep --invert-match '{pattern}' {file}"]},
                {"correct": "grep -n '{pattern}' {file}", "errors": ["grep -N '{pattern}' {file}"]},
            ],
            "sed": [
                {"correct": "sed -i 's/{old}/{new}/g' {file}", "errors": ["sed 's/{old}/{new}/g' {file}", "sed -i 's/{old}/{new}/' {file}"]},
                {"correct": "sed -n '{n}p' {file}", "errors": ["sed '{n}p' {file}"]},
            ],
            "awk": [
                {"correct": "awk '{{print $1}}' {file}", "errors": ["awk {print $1} {file}", "awk '{{print}}' {file}"]},
                {"correct": "awk -F',' '{{print $1}}' {file}", "errors": ["awk -F, '{{print $1}}' {file}"]},
            ],
            "jq": [
                {"correct": "jq '.' {file}", "errors": ["jq . {file}", "jq '{file}'"]},
                {"correct": "jq '.{key}' {file}", "errors": ["jq {key} {file}"]},
                {"correct": "jq -r '.{key}' {file}", "errors": ["jq '.{key}' {file} -r"]},
            ],
        },

        # Language Runtimes
        "languages": {
            "python": [
                {"correct": "python {file}", "errors": ["pyhton {file}", "pytohn {file}"]},
                {"correct": "python -m venv {path}", "errors": ["python -m venv", "python venv {path}"]},
                {"correct": "python -m pip install {package}", "errors": ["python -m pip instal {package}"]},
                {"correct": "python -c '{code}'", "errors": ["python -c {code}"]},
            ],
            "node": [
                {"correct": "node {file}", "errors": ["nod {file}", "nodee {file}"]},
                {"correct": "node -e '{code}'", "errors": ["node -e {code}"]},
            ],
            "go": [
                {"correct": "go run {file}", "errors": ["go rnu {file}", "go exec {file}"]},
                {"correct": "go build", "errors": ["go biuld", "go compile"]},
                {"correct": "go test", "errors": ["go tets", "go tests"]},
                {"correct": "go mod init {module}", "errors": ["go mod init", "go init {module}"]},
                {"correct": "go mod tidy", "errors": ["go mod clean", "go tidy"]},
            ],
            "java": [
                {"correct": "java {class}", "errors": ["jav {class}", "java {class}.class"]},
                {"correct": "javac {file}", "errors": ["javac {file}.java", "java -c {file}"]},
            ],
        },

        # System Tools
        "system": {
            "systemctl": [
                {"correct": "sudo systemctl start {service}", "errors": ["systemctl start {service}", "sudo systemctl strart {service}"]},
                {"correct": "sudo systemctl stop {service}", "errors": ["systemctl stop {service}", "sudo systemctl stopp {service}"]},
                {"correct": "sudo systemctl restart {service}", "errors": ["systemctl restart {service}", "sudo systemctl restar {service}"]},
                {"correct": "systemctl status {service}", "errors": ["systemctl stauts {service}", "systemctl stat {service}"]},
                {"correct": "sudo systemctl enable {service}", "errors": ["systemctl enable {service}"]},
            ],
            "journalctl": [
                {"correct": "journalctl -u {service}", "errors": ["journalctl {service}", "journalctl -U {service}"]},
                {"correct": "journalctl -f", "errors": ["journalctl --follow", "journalctl -F"]},
            ],
        },
    }

    def __init__(self, templates_dir: str, seed: int = 42):
        """Initialize the tools generator."""
        super().__init__(templates_dir, seed)

    def generate(self, count: int) -> list[TrainingExample]:
        """Generate training examples for top 100 tools."""
        examples = []

        # Flatten all tools into a single list with weights
        all_tools = []
        for category, tools in self.TOOLS.items():
            for tool_name, commands in tools.items():
                for cmd in commands:
                    all_tools.append((category, tool_name, cmd))

        # Generate examples with shell distribution
        shell_counts = {
            shell: int(count * weight)
            for shell, weight in self.SHELL_WEIGHTS.items()
        }

        total = sum(shell_counts.values())
        if total < count:
            shell_counts["bash"] += count - total

        for shell, shell_count in shell_counts.items():
            for _ in range(shell_count):
                category, tool_name, cmd = self.rng.choice(all_tools)
                example = self._create_example(shell, category, tool_name, cmd)
                if example:
                    examples.append(example)

        # Filter out null corrections
        examples = [
            ex for ex in examples
            if ex is not None and ex.incorrect_command.strip() != ex.correct_command.strip()
        ]

        # Regenerate if we don't have enough
        max_retries = 3
        retry = 0
        while len(examples) < count and retry < max_retries:
            deficit = count - len(examples)
            for _ in range(deficit * 2):
                category, tool_name, cmd = self.rng.choice(all_tools)
                example = self._create_example("bash", category, tool_name, cmd)
                if example and example.incorrect_command.strip() != example.correct_command.strip():
                    examples.append(example)
            retry += 1

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

    def _create_example(
        self, shell: str, category: str, tool_name: str, cmd: dict[str, Any]
    ) -> TrainingExample | None:
        """Create a training example from a tool command."""
        correct_template = cmd.get("correct", "")
        errors = cmd.get("errors", [])

        if not correct_template:
            return None

        # First extract and fix variable values for consistency
        variables = self._extract_variables(correct_template)

        # Expand correct template with fixed variables
        correct = self.expand_template(correct_template, variables)

        # Select or generate error - use SAME variable values
        if errors and self.rng.random() < 0.7:
            error_template = self.rng.choice(errors)
            # Apply same variables to error template
            incorrect = self.expand_template(error_template, variables)
        else:
            # Generate typo on the already-expanded correct command
            incorrect = self.generate_typo(correct, typo_rate=1.0)

        # Check for single-char correction and try to generate multi-char error
        max_retries = 3
        retry = 0
        while self._is_single_char_correction(incorrect, correct) and retry < max_retries:
            # Try removing a space (guaranteed multi-char)
            incorrect = self.remove_space(correct)
            if incorrect == correct or self._is_single_char_correction(incorrect, correct):
                # Try a different predefined error
                if errors:
                    incorrect = self.expand_template(self.rng.choice(errors), variables)
                else:
                    incorrect = self.generate_typo(correct, typo_rate=1.0)
            retry += 1

        # Final null correction check
        if incorrect.strip() == correct.strip():
            return None

        return TrainingExample(
            shell=shell,
            incorrect_command=incorrect,
            correct_command=correct,
            category=f"tools_{category}",
            error_type="tool_specific",
            source="tools_pattern",
            metadata={"tool": tool_name},
        )

    def _extract_variables(self, template: str) -> dict[str, str]:
        """Extract variable placeholders and generate consistent values."""
        import re
        from .base_generator import get_random_variable

        variables = {}
        placeholders = re.findall(r"\{(\w+)\}", template)

        for placeholder in placeholders:
            if placeholder not in variables:
                variables[placeholder] = get_random_variable(placeholder, self.rng)

        return variables

    def _is_single_char_correction(self, incorrect: str, correct: str) -> bool:
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


if __name__ == "__main__":
    import sys

    templates_dir = sys.argv[1] if len(sys.argv) > 1 else "src/data_generation/templates"

    generator = ToolsGenerator(templates_dir)
    examples = generator.generate(100)

    print(f"Generated {len(examples)} examples")
    print("\nSample examples:")
    for example in examples[:10]:
        print(f"\nShell: {example.shell}")
        print(f"Category: {example.category}")
        print(f"Incorrect: {example.incorrect_command}")
        print(f"Correct: {example.correct_command}")
