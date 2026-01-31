#!/bin/sh
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
  if curl -fsSL "$win_url" -o "${tmpdir}/fix.zip" && unzip -q "${tmpdir}/fix.zip" -d "${tmpdir}"; then
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
        warn "Failed to download Windows binary."
        echo ""
        echo "You can manually download from:"
        echo "  https://github.com/${REPO}/releases/latest"
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

  success "Installation complete!"
  echo ""
  echo "Run '${BINARY} --help' to get started."
}

main
