#!/bin/bash
# fix installer
# Usage: curl -fsSL https://animeshkundu.github.io/fix/install.sh | sh

set -e

REPO="animeshkundu/fix"
BINARY="fix"
INSTALL_DIR="${HOME}/.local/bin"

# Colors (if terminal supports it)
if [ -t 1 ]; then
  RED='\033[0;31m'
  GREEN='\033[0;32m'
  YELLOW='\033[0;33m'
  BLUE='\033[0;34m'
  NC='\033[0m'
else
  RED=''
  GREEN=''
  YELLOW=''
  BLUE=''
  NC=''
fi

info() {
  printf "${BLUE}==>${NC} %s\n" "$1"
}

success() {
  printf "${GREEN}==>${NC} %s\n" "$1"
}

warn() {
  printf "${YELLOW}warning:${NC} %s\n" "$1"
}

error() {
  printf "${RED}error:${NC} %s\n" "$1" >&2
  exit 1
}

# Detect WSL
is_wsl() {
  grep -qiE "(microsoft|wsl)" /proc/version 2>/dev/null
}

# Check for NVIDIA GPU
has_nvidia_gpu() {
  command -v lspci >/dev/null 2>&1 && lspci 2>/dev/null | grep -qi nvidia
}

# Download Windows binary for WSL fallback
download_windows_binary() {
  local latest
  latest=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
  local win_url="https://github.com/${REPO}/releases/download/${latest}/${BINARY}-x86_64-pc-windows-msvc.zip"

  info "Downloading Windows binary for WSL..."
  local tmpdir=$(mktemp -d)

  if curl -fsSL "$win_url" -o "${tmpdir}/fix.zip"; then
    # Try multiple extraction methods
    if command -v unzip >/dev/null 2>&1; then
      unzip -q "${tmpdir}/fix.zip" -d "${tmpdir}"
    elif command -v 7z >/dev/null 2>&1; then
      7z x -o"${tmpdir}" "${tmpdir}/fix.zip" >/dev/null
    elif command -v powershell.exe >/dev/null 2>&1; then
      powershell.exe -Command "Expand-Archive -Path '${tmpdir}/fix.zip' -DestinationPath '${tmpdir}'" 2>/dev/null
    else
      warn "No zip extraction tool found. Install unzip: sudo apt install unzip"
      rm -rf "${tmpdir}"
      return 1
    fi

    mv "${tmpdir}/${BINARY}.exe" "${INSTALL_DIR}/${BINARY}.exe"
    chmod +x "${INSTALL_DIR}/${BINARY}.exe"
    rm -rf "${tmpdir}"
    return 0
  else
    rm -rf "${tmpdir}"
    return 1
  fi
}

# Detect OS
detect_os() {
  case "$(uname -s)" in
    Darwin) echo "macos" ;;
    Linux) echo "linux" ;;
    MINGW*|MSYS*|CYGWIN*) echo "windows" ;;
    *) error "Unsupported operating system: $(uname -s)" ;;
  esac
}

# Detect architecture
detect_arch() {
  case "$(uname -m)" in
    x86_64|amd64) echo "x86_64" ;;
    arm64|aarch64) echo "aarch64" ;;
    *) error "Unsupported architecture: $(uname -m)" ;;
  esac
}

# Get download URL for target
get_download_url() {
  local os="$1"
  local arch="$2"
  local target=""

  case "${os}-${arch}" in
    macos-aarch64) target="aarch64-apple-darwin" ;;
    macos-x86_64) target="x86_64-apple-darwin" ;;
    linux-x86_64) target="x86_64-unknown-linux-gnu" ;;
    linux-aarch64) target="aarch64-unknown-linux-gnu" ;;
    *) error "No binary available for ${os}-${arch}" ;;
  esac

  # Get latest release tag
  local latest
  latest=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

  if [ -z "$latest" ]; then
    error "Failed to fetch latest release"
  fi

  echo "https://github.com/${REPO}/releases/download/${latest}/${BINARY}-${target}.tar.gz"
}

main() {
  info "Installing ${BINARY}..."

  local os=$(detect_os)
  local arch=$(detect_arch)

  info "Detected: ${os} (${arch})"

  if [ "$os" = "windows" ]; then
    error "Windows installation via script is not supported. Please download from GitHub Releases."
  fi

  local url=$(get_download_url "$os" "$arch")
  info "Downloading from: ${url}"

  # Create temp directory
  local tmpdir=$(mktemp -d)
  trap "rm -rf ${tmpdir}" EXIT

  # Download and extract
  curl -fsSL "$url" | tar -xz -C "$tmpdir"

  # Create install directory if needed
  mkdir -p "$INSTALL_DIR"

  # Install binary
  mv "${tmpdir}/${BINARY}" "${INSTALL_DIR}/${BINARY}"
  chmod +x "${INSTALL_DIR}/${BINARY}"

  success "Installed ${BINARY} to ${INSTALL_DIR}/${BINARY}"

  # Check if install dir is in PATH
  case ":$PATH:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
      warn "${INSTALL_DIR} is not in your PATH"
      echo ""
      echo "Add it to your shell profile:"
      echo ""
      echo "  # bash"
      echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.bashrc"
      echo ""
      echo "  # zsh"
      echo "  echo 'export PATH=\"\$HOME/.local/bin:\$PATH\"' >> ~/.zshrc"
      echo ""
      ;;
  esac

  # Download default model
  local model_url="https://huggingface.co/animeshkundu/cmd-correct/resolve/main/qwen3-correct-0.6B.gguf"
  local model_dir="${HOME}/.config/fix"
  local model_path="${model_dir}/qwen3-correct-0.6B.gguf"

  if [ -f "$model_path" ]; then
    info "Model already exists at ${model_path}"
  else
    info "Downloading default model (~378MB)..."
    mkdir -p "$model_dir"
    if curl -fSL --progress-bar "$model_url" -o "$model_path"; then
      success "Model downloaded to ${model_path}"
    else
      warn "Model download failed. You can retry with: ${BINARY} --update"
    fi
  fi

  # Verify installation with test command
  info "Testing installation..."
  local test_output
  test_output=$("${INSTALL_DIR}/${BINARY}" "gti status" 2>&1) || true

  if [ "$test_output" = "git status" ]; then
    success "Test passed! 'gti status' → 'git status'"
  else
    warn "Test produced unexpected output: ${test_output}"

    if is_wsl; then
      info "WSL detected. Trying Windows binary instead..."
      if download_windows_binary; then
        # Re-test with Windows binary
        test_output=$("${INSTALL_DIR}/${BINARY}.exe" "gti status" 2>&1) || true
        if [ "$test_output" = "git status" ]; then
          success "Windows binary works! Using fix.exe on WSL."
          rm -f "${INSTALL_DIR}/${BINARY}"  # Remove Linux binary
        else
          warn "Windows binary also failed."
          echo ""
          echo "Debug with: ${BINARY}.exe --verbose gti status"
        fi
      else
        warn "Linux binary failed (likely GLIBC version) and Windows fallback unavailable."
        echo ""
        echo "Options:"
        echo "  1. Install unzip and retry:"
        echo "     sudo apt install unzip && curl -fsSL https://animeshkundu.github.io/fix/install.sh | bash"
        echo ""
        echo "  2. Download Windows binary manually:"
        echo "     https://github.com/${REPO}/releases/latest"
        echo "     Extract fix.exe to ${INSTALL_DIR}/"
        echo ""
        echo "  3. Build from source:"
        echo "     cargo install --git https://github.com/${REPO} fix"
      fi
    else
      # Native Linux
      echo ""
      echo "Troubleshooting:"
      if has_nvidia_gpu; then
        echo "  NVIDIA GPU detected. Ensure drivers are installed:"
        echo "    Ubuntu/Debian: sudo apt install nvidia-driver-535"
        echo "    Fedora: sudo dnf install akmod-nvidia"
      else
        echo "  No GPU detected. CPU inference should still work."
      fi
      echo ""
      echo "  Debug with: ${BINARY} --verbose gti status"
    fi
  fi

  # Configure shell integration
  configure_shell_integration

  success "Installation complete!"
  echo ""
  echo "Run '${BINARY} --help' to get started."
}

# Shell integration configuration
configure_shell_integration() {
  local shell_name=$(basename "$SHELL")
  local config_file=""

  case "$shell_name" in
    bash) config_file="$HOME/.bashrc" ;;
    zsh)  config_file="$HOME/.zshrc" ;;
    fish) config_file="$HOME/.config/fish/functions/fix.fish" ;;
    *)    return ;;
  esac

  # Check if already configured
  if [ -f "$config_file" ] && grep -q "fix - AI-powered shell command corrector" "$config_file" 2>/dev/null; then
    info "Shell integration already configured in $config_file"
    return
  fi

  info "Configuring shell integration in $config_file..."

  case "$shell_name" in
    bash)
      cat >> "$config_file" <<'BASH_FUNC'

# fix - AI-powered shell command corrector
fix() {
    if [[ -n "$1" ]]; then
        command fix "$@"
    else
        local cmd=$(fc -ln -1 | sed 's/^[[:space:]]*//')
        local corrected=$(command fix "$cmd" 2>/dev/null)
        if [[ -n "$corrected" && "$corrected" != "$cmd" ]]; then
            echo "Correcting: $cmd → $corrected"
            read -e -i "$corrected" -p "» " final_cmd
            [[ -n "$final_cmd" ]] && eval "$final_cmd"
        else
            echo "No correction needed"
        fi
    fi
}
BASH_FUNC
      ;;
    zsh)
      cat >> "$config_file" <<'ZSH_FUNC'

# fix - AI-powered shell command corrector
fix() {
    if [[ -n "$1" ]]; then
        command fix "$@"
    else
        local cmd=$(fc -ln -1 | sed 's/^[[:space:]]*//')
        local corrected=$(command fix "$cmd" 2>/dev/null)
        if [[ -n "$corrected" && "$corrected" != "$cmd" ]]; then
            echo "Correcting: $cmd → $corrected"
            print -z "$corrected"
        else
            echo "No correction needed"
        fi
    fi
}
ZSH_FUNC
      ;;
    fish)
      mkdir -p "$(dirname "$config_file")"
      cat > "$config_file" <<'FISH_FUNC'
# fix - AI-powered shell command corrector
function fix --description 'Fix the last command'
    if test (count $argv) -gt 0
        command fix $argv
    else
        set -l cmd (string trim (history --max=1))
        set -l corrected (command fix "$cmd" 2>/dev/null)
        if test -n "$corrected" -a "$corrected" != "$cmd"
            echo "Correcting: $cmd → $corrected"
            commandline -r "$corrected"
            commandline -f repaint
        else
            echo "No correction needed"
        end
    end
end
FISH_FUNC
      ;;
  esac

  success "Shell integration configured"
  echo "  Restart your shell or run: source $config_file"
}

main
