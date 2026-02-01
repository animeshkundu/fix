#!/bin/bash
# fix/wit installer - Installs both binaries
# Usage: curl -fsSL https://animeshkundu.github.io/fix/install.sh | sh
# Usage: curl -fsSL https://animeshkundu.github.io/fix/install.sh | sh -s fix  (fix as primary)

set -e

REPO="animeshkundu/fix"
# Default to 'wit' as primary, 'fix' as secondary
# First argument can override primary binary
PRIMARY="${1:-wit}"
if [ "$PRIMARY" = "wit" ]; then
  SECONDARY="fix"
else
  SECONDARY="wit"
fi
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

# Check if Rust is installed
has_rust() {
  command -v cargo >/dev/null 2>&1 && command -v rustc >/dev/null 2>&1
}

# Install Rust via rustup
install_rust() {
  info "Installing Rust via rustup..."
  if curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y; then
    # Source cargo env for current session
    if [ -f "$HOME/.cargo/env" ]; then
      . "$HOME/.cargo/env"
    fi
    success "Rust installed successfully"
    return 0
  else
    warn "Failed to install Rust"
    return 1
  fi
}

# Install build dependencies based on OS
install_build_deps() {
  local os="$1"
  info "Installing build dependencies..."

  case "$os" in
    linux)
      # Detect package manager and install deps
      if command -v apt-get >/dev/null 2>&1; then
        info "Using apt-get to install dependencies..."
        sudo apt-get update
        sudo apt-get install -y build-essential cmake pkg-config libssl-dev libclang-dev git
      elif command -v dnf >/dev/null 2>&1; then
        info "Using dnf to install dependencies..."
        sudo dnf install -y gcc gcc-c++ cmake pkgconfig openssl-devel clang-devel git
      elif command -v yum >/dev/null 2>&1; then
        info "Using yum to install dependencies..."
        sudo yum install -y gcc gcc-c++ cmake pkgconfig openssl-devel clang-devel git
      elif command -v pacman >/dev/null 2>&1; then
        info "Using pacman to install dependencies..."
        sudo pacman -Sy --noconfirm base-devel cmake openssl clang git
      elif command -v apk >/dev/null 2>&1; then
        info "Using apk to install dependencies..."
        sudo apk add --no-cache build-base cmake pkgconfig openssl-dev clang-dev git
      else
        warn "Could not detect package manager. Please install manually:"
        echo "  - build-essential/base-devel (C/C++ compiler)"
        echo "  - cmake"
        echo "  - pkg-config"
        echo "  - libssl-dev/openssl-devel"
        echo "  - libclang-dev/clang-devel"
        return 1
      fi
      ;;
    macos)
      # macOS: use Homebrew or Xcode
      if ! command -v xcode-select >/dev/null 2>&1 || ! xcode-select -p >/dev/null 2>&1; then
        info "Installing Xcode command line tools..."
        xcode-select --install 2>/dev/null || true
        # Wait for installation
        echo "Please complete Xcode installation and re-run the installer."
        return 1
      fi
      # cmake is typically available via Homebrew or not needed on macOS
      if command -v brew >/dev/null 2>&1; then
        brew install cmake pkg-config openssl 2>/dev/null || true
      fi
      ;;
  esac

  success "Build dependencies installed"
  return 0
}

# Build a single binary from source
build_binary_from_source() {
  local binary="$1"
  local os="$2"
  info "Building ${binary} from source..."

  # Install Rust if needed (only check once)
  if ! has_rust; then
    if ! install_rust; then
      error "Failed to install Rust. Please install manually: https://rustup.rs"
    fi
  fi

  # Build using cargo install
  info "Building ${binary} with cargo (this may take a few minutes)..."

  # Determine features based on platform
  local features=""
  if [ "$os" = "macos" ]; then
    features="--features metal"
  fi

  if cargo install --git "https://github.com/${REPO}" ${binary} $features; then
    # cargo installs to ~/.cargo/bin, copy to our install dir
    if [ -f "$HOME/.cargo/bin/${binary}" ]; then
      mkdir -p "$INSTALL_DIR"
      cp "$HOME/.cargo/bin/${binary}" "${INSTALL_DIR}/${binary}"
      chmod +x "${INSTALL_DIR}/${binary}"
      success "Built and installed ${binary} to ${INSTALL_DIR}/${binary}"
      return 0
    fi
  fi

  warn "Build of ${binary} from source failed."
  return 1
}

# Download Windows binary for WSL fallback
download_windows_binary() {
  local binary="$1"
  local latest
  latest=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
  local win_url="https://github.com/${REPO}/releases/download/${latest}/${binary}-x86_64-pc-windows-msvc.zip"

  info "Downloading Windows binary for WSL (${binary})..."
  local tmpdir=$(mktemp -d)

  if curl -fsSL "$win_url" -o "${tmpdir}/${binary}.zip"; then
    # Try multiple extraction methods
    if command -v unzip >/dev/null 2>&1; then
      unzip -q "${tmpdir}/${binary}.zip" -d "${tmpdir}"
    elif command -v 7z >/dev/null 2>&1; then
      7z x -o"${tmpdir}" "${tmpdir}/${binary}.zip" >/dev/null
    elif command -v powershell.exe >/dev/null 2>&1; then
      powershell.exe -Command "Expand-Archive -Path '${tmpdir}/${binary}.zip' -DestinationPath '${tmpdir}'" 2>/dev/null
    else
      warn "No zip extraction tool found. Install unzip: sudo apt install unzip"
      rm -rf "${tmpdir}"
      return 1
    fi

    mv "${tmpdir}/${binary}.exe" "${INSTALL_DIR}/${binary}.exe"
    chmod +x "${INSTALL_DIR}/${binary}.exe"
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

# Get download URL for a specific binary and target
get_download_url() {
  local binary="$1"
  local os="$2"
  local arch="$3"
  local target=""

  case "${os}-${arch}" in
    macos-aarch64) target="aarch64-apple-darwin" ;;
    macos-x86_64) target="x86_64-apple-darwin" ;;
    linux-x86_64) target="x86_64-unknown-linux-gnu" ;;
    linux-aarch64) target="aarch64-unknown-linux-gnu" ;;
    *)
      # No pre-built binary for this platform
      return 1
      ;;
  esac

  # Get latest release tag
  local latest
  latest=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" 2>/dev/null | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')

  if [ -z "$latest" ]; then
    return 1
  fi

  echo "https://github.com/${REPO}/releases/download/${latest}/${binary}-${target}.tar.gz"
}

# Install a single binary (download or build)
install_binary() {
  local binary="$1"
  local os="$2"
  local arch="$3"
  local from_source="$4"

  info "Installing ${binary}..."

  if [ "$from_source" = true ]; then
    if build_binary_from_source "$binary" "$os"; then
      return 0
    fi
    return 1
  fi

  # Try downloading pre-built binary
  local url
  url=$(get_download_url "$binary" "$os" "$arch" 2>/dev/null) || true

  if [ -n "$url" ]; then
    info "Downloading ${binary} from: ${url}"

    # Create temp directory
    local tmpdir=$(mktemp -d)

    # Download and extract
    if curl -fsSL "$url" 2>/dev/null | tar -xz -C "$tmpdir" 2>/dev/null; then
      # Create install directory if needed
      mkdir -p "$INSTALL_DIR"

      # Install binary
      if [ -f "${tmpdir}/${binary}" ]; then
        mv "${tmpdir}/${binary}" "${INSTALL_DIR}/${binary}"
        chmod +x "${INSTALL_DIR}/${binary}"
        rm -rf "$tmpdir"
        success "Installed ${binary} to ${INSTALL_DIR}/${binary}"
        return 0
      fi
    fi
    rm -rf "$tmpdir"
  fi

  # Download failed, try build from source
  warn "Pre-built ${binary} not available or download failed."
  echo ""
  echo "Would you like to build ${binary} from source? [y/N]"
  read -r response
  if [ "$response" = "y" ] || [ "$response" = "Y" ]; then
    # Install build deps if not done yet
    if [ "$BUILD_DEPS_INSTALLED" != "true" ]; then
      if install_build_deps "$os"; then
        BUILD_DEPS_INSTALLED=true
      fi
    fi
    if build_binary_from_source "$binary" "$os"; then
      return 0
    fi
  fi

  return 1
}

# Download model for a specific binary
download_model() {
  local binary="$1"
  local model_name=""
  local model_size=""

  if [ "$binary" = "wit" ]; then
    model_name="qwen3-wit-1.7B.gguf"
    model_size="~1GB"
  else
    model_name="qwen3-correct-0.6B.gguf"
    model_size="~378MB"
  fi

  local model_url="https://huggingface.co/animeshkundu/cmd-correct/resolve/main/${model_name}"
  local model_dir="${HOME}/.config/fix"
  local model_path="${model_dir}/${model_name}"

  if [ -f "$model_path" ]; then
    info "Model already exists at ${model_path}"
  else
    info "Downloading ${binary} model (${model_size})..."
    mkdir -p "$model_dir"
    if curl -fSL --progress-bar "$model_url" -o "$model_path"; then
      success "Model downloaded to ${model_path}"
    else
      warn "Model download failed. You can retry with: ${binary} --update"
    fi
  fi
}

# Test a binary installation
test_binary() {
  local binary="$1"
  local os="$2"

  info "Testing ${binary}..."
  local test_output

  if [ "$binary" = "wit" ]; then
    # Test wit with --show-config
    test_output=$("${INSTALL_DIR}/${binary}" --show-config 2>&1) || true
    if echo "$test_output" | grep -qE "(Configuration:|Wit model:)"; then
      success "Test passed! ${binary} --show-config works"
      return 0
    fi
  else
    # Test fix with command correction
    test_output=$("${INSTALL_DIR}/${binary}" "gti status" 2>&1) || true
    if [ "$test_output" = "git status" ]; then
      success "Test passed! ${binary}: 'gti status' → 'git status'"
      return 0
    fi
  fi

  warn "Test for ${binary} produced unexpected output: ${test_output}"

  # Try WSL fallback if applicable
  if is_wsl; then
    info "WSL detected. Trying Windows binary for ${binary}..."
    if download_windows_binary "$binary"; then
      if [ "$binary" = "wit" ]; then
        test_output=$("${INSTALL_DIR}/${binary}.exe" --show-config 2>&1) || true
        if echo "$test_output" | grep -qE "(Configuration:|Wit model:)"; then
          success "Windows binary works for ${binary}!"
          rm -f "${INSTALL_DIR}/${binary}"
          return 0
        fi
      else
        test_output=$("${INSTALL_DIR}/${binary}.exe" "gti status" 2>&1) || true
        if [ "$test_output" = "git status" ]; then
          success "Windows binary works for ${binary}!"
          rm -f "${INSTALL_DIR}/${binary}"
          return 0
        fi
      fi
    fi
  fi

  return 1
}

# Shell integration configuration (parameterized for binary name)
configure_shell_integration() {
  local binary="$1"
  local shell_name=$(basename "$SHELL")
  local config_file=""
  local marker="${binary} - AI-powered"

  case "$shell_name" in
    bash) config_file="$HOME/.bashrc" ;;
    zsh)  config_file="$HOME/.zshrc" ;;
    fish) config_file="$HOME/.config/fish/functions/${binary}.fish" ;;
    tcsh) config_file="$HOME/.tcshrc" ;;
    *)    return ;;
  esac

  # Check if already configured for this binary
  if [ -f "$config_file" ] && grep -q "${marker}" "$config_file" 2>/dev/null; then
    info "Shell integration for ${binary} already configured in $config_file"
    return
  fi

  info "Configuring shell integration for ${binary} in $config_file..."

  case "$shell_name" in
    bash)
      cat >> "$config_file" <<BASH_FUNC

# ${binary} - AI-powered shell command corrector
${binary}() {
    if [[ -n "\$1" ]]; then
        command ${binary} "\$@"
    else
        local cmd=\$(fc -ln -1 | sed 's/^[[:space:]]*//')
        local corrected=\$(command ${binary} "\$cmd" 2>/dev/null)
        if [[ -n "\$corrected" && "\$corrected" != "\$cmd" ]]; then
            echo "Correcting: \$cmd → \$corrected"
            read -e -i "\$corrected" -p "» " final_cmd
            [[ -n "\$final_cmd" ]] && eval "\$final_cmd"
        else
            echo "No correction needed"
        fi
    fi
}
BASH_FUNC
      ;;
    zsh)
      cat >> "$config_file" <<ZSH_FUNC

# ${binary} - AI-powered shell command corrector
${binary}() {
    if [[ -n "\$1" ]]; then
        command ${binary} "\$@"
    else
        local cmd=\$(fc -ln -1 | sed 's/^[[:space:]]*//')
        local corrected=\$(command ${binary} "\$cmd" 2>/dev/null)
        if [[ -n "\$corrected" && "\$corrected" != "\$cmd" ]]; then
            echo "Correcting: \$cmd → \$corrected"
            print -z "\$corrected"
        else
            echo "No correction needed"
        fi
    fi
}
ZSH_FUNC
      ;;
    fish)
      mkdir -p "$(dirname "$config_file")"
      cat > "$config_file" <<FISH_FUNC
# ${binary} - AI-powered shell command corrector
function ${binary} --description 'Fix the last command'
    if test (count \$argv) -gt 0
        command ${binary} \$argv
    else
        set -l cmd (string trim (history --max=1))
        set -l corrected (command ${binary} "\$cmd" 2>/dev/null)
        if test -n "\$corrected" -a "\$corrected" != "\$cmd"
            echo "Correcting: \$cmd → \$corrected"
            commandline -r "\$corrected"
            commandline -f repaint
        else
            echo "No correction needed"
        end
    end
end
FISH_FUNC
      ;;
    tcsh)
      cat >> "$config_file" <<TCSH_FUNC

# ${binary} - AI-powered shell command corrector
alias ${binary}last 'set _cmd = \`history -h 1\` && set _fix = \`${binary} "\$_cmd"\` && echo "Correcting: \$_cmd -> \$_fix" && eval "\$_fix"'
TCSH_FUNC
      ;;
  esac

  success "Shell integration for ${binary} configured"
  echo "  Restart your shell or run: source $config_file"
}

main() {
  info "Installing wit and fix CLI tools..."
  info "Primary: ${PRIMARY}, Secondary: ${SECONDARY}"

  # Check for --from-source flag
  local from_source=false
  for arg in "$@"; do
    if [ "$arg" = "--from-source" ]; then
      from_source=true
    fi
  done

  local os=$(detect_os)
  local arch=$(detect_arch)

  info "Detected: ${os} (${arch})"

  if [ "$os" = "windows" ]; then
    error "Windows installation via script is not supported. Please download from GitHub Releases."
  fi

  # Track build deps installation
  BUILD_DEPS_INSTALLED=false

  # Install build deps upfront if building from source
  if [ "$from_source" = true ]; then
    if install_build_deps "$os"; then
      BUILD_DEPS_INSTALLED=true
    fi
  fi

  # Install both binaries
  local primary_installed=false
  local secondary_installed=false

  if install_binary "$PRIMARY" "$os" "$arch" "$from_source"; then
    primary_installed=true
  fi

  if install_binary "$SECONDARY" "$os" "$arch" "$from_source"; then
    secondary_installed=true
  fi

  if [ "$primary_installed" = false ]; then
    error "Failed to install primary binary (${PRIMARY}). Please try building from source manually."
  fi

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

  # Download model for primary binary only
  download_model "$PRIMARY"

  # Test installations
  if [ "$primary_installed" = true ]; then
    test_binary "$PRIMARY" "$os" || true
  fi
  if [ "$secondary_installed" = true ]; then
    test_binary "$SECONDARY" "$os" || true
  fi

  # Configure shell integration for primary binary
  configure_shell_integration "$PRIMARY"

  success "Installation complete!"
  echo ""
  echo "Installed binaries:"
  [ "$primary_installed" = true ] && echo "  - ${PRIMARY} (primary, with shell integration)"
  [ "$secondary_installed" = true ] && echo "  - ${SECONDARY}"
  echo ""
  echo "Run '${PRIMARY} --help' to get started."
  if [ "$secondary_installed" = true ]; then
    echo "Run '${SECONDARY} --help' for the alternative corrector."
  fi
}

main "$@"
