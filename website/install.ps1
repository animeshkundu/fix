# fix installer for Windows
# Usage: iwr -useb https://animeshkundu.github.io/fix/install.ps1 | iex

$ErrorActionPreference = 'Stop'

$repo = "animeshkundu/fix"
$binary = "fix"

function Write-Info { param($msg) Write-Host "==> " -ForegroundColor Blue -NoNewline; Write-Host $msg }
function Write-Success { param($msg) Write-Host "==> " -ForegroundColor Green -NoNewline; Write-Host $msg }
function Write-Warn { param($msg) Write-Host "warning: " -ForegroundColor Yellow -NoNewline; Write-Host $msg }
function Write-Err { param($msg) Write-Host "error: " -ForegroundColor Red -NoNewline; Write-Host $msg }

# Check if Rust is installed
function Test-Rust {
    try {
        $null = & cargo --version 2>&1
        $null = & rustc --version 2>&1
        return $true
    } catch {
        return $false
    }
}

# Install Rust via rustup
function Install-Rust {
    Write-Info "Installing Rust via rustup..."
    try {
        $rustupInit = "$env:TEMP\rustup-init.exe"
        Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile $rustupInit -UseBasicParsing
        & $rustupInit -y --default-toolchain stable
        Remove-Item $rustupInit -Force -ErrorAction SilentlyContinue

        # Add cargo to current session PATH
        $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"

        Write-Success "Rust installed successfully"
        return $true
    } catch {
        Write-Err "Failed to install Rust: $_"
        return $false
    }
}

# Check for Visual Studio Build Tools
function Test-BuildTools {
    # Check for cl.exe in common locations
    $vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vsWhere) {
        $installPath = & $vsWhere -latest -property installationPath 2>$null
        if ($installPath) {
            return $true
        }
    }
    return $false
}

# Build from source
function Build-FromSource {
    Write-Info "Building $binary from source..."

    # Check/install Rust
    if (-not (Test-Rust)) {
        if (-not (Install-Rust)) {
            Write-Err "Could not install Rust. Please install manually from https://rustup.rs"
            return $false
        }
    }

    # Check for Visual Studio Build Tools
    if (-not (Test-BuildTools)) {
        Write-Warn "Visual Studio Build Tools not found."
        Write-Host ""
        Write-Host "To build from source on Windows, you need:"
        Write-Host "  1. Visual Studio Build Tools (or full Visual Studio)"
        Write-Host "     Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/"
        Write-Host "     Select 'Desktop development with C++' workload"
        Write-Host ""
        Write-Host "  2. CMake (usually included with VS Build Tools)"
        Write-Host ""
        Write-Host "After installing, run this installer again."
        return $false
    }

    Write-Info "Building with cargo (this may take several minutes)..."
    try {
        $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
        & cargo install --git "https://github.com/$repo" fix

        # Copy from cargo bin to install dir
        $cargoBin = "$env:USERPROFILE\.cargo\bin\fix.exe"
        if (Test-Path $cargoBin) {
            if (-not (Test-Path $installDir)) {
                New-Item -ItemType Directory -Path $installDir -Force | Out-Null
            }
            Copy-Item $cargoBin -Destination "$installDir\$binary.exe" -Force
            Write-Success "Built and installed $binary to $installDir\$binary.exe"
            return $true
        }
    } catch {
        Write-Err "Build failed: $_"
    }
    return $false
}

# Offer to build from source
function Invoke-BuildFromSource {
    Write-Host ""
    Write-Host "Would you like to build from source? [y/N]" -ForegroundColor Yellow
    $response = Read-Host
    if ($response -eq "y" -or $response -eq "Y") {
        return Build-FromSource
    } else {
        Write-Host ""
        Write-Host "You can build manually later with:"
        Write-Host "  cargo install --git https://github.com/$repo fix"
        return $false
    }
}

# Detect architecture
$arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "i686" }
$target = "$arch-pc-windows-msvc"

Write-Info "Installing $binary..."
Write-Info "Detected: Windows ($arch)"

# Get latest release
try {
    $release = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest"
    $version = $release.tag_name
    Write-Info "Latest version: $version"
} catch {
    Write-Err "Failed to fetch latest release"
    exit 1
}

# Find the right asset
$assetName = "$binary-$target.zip"
$asset = $release.assets | Where-Object { $_.name -eq $assetName }

# Create install directory
$installDir = "$env:LOCALAPPDATA\$binary"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

$exePath = "$installDir\$binary.exe"
$binaryInstalled = $false

if (-not $asset) {
    Write-Warn "No pre-built binary found for $target"
    if (Invoke-BuildFromSource) {
        $binaryInstalled = $true
    }
} else {
    $downloadUrl = $asset.browser_download_url
    Write-Info "Downloading from: $downloadUrl"

    # Download and extract
    $tempFile = "$env:TEMP\$assetName"
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -UseBasicParsing
        Expand-Archive -Path $tempFile -DestinationPath $installDir -Force
        Remove-Item $tempFile -Force
        $binaryInstalled = $true
    } catch {
        Write-Warn "Download failed: $_"
        Write-Host ""
        if (Invoke-BuildFromSource) {
            $binaryInstalled = $true
        }
    }
}

if (-not $binaryInstalled) {
    Write-Err "Installation failed. Please try building from source manually."
    exit 1
}

if (-not (Test-Path $exePath)) {
    Write-Err "Binary not found at expected location: $exePath"
    exit 1
}

Write-Success "Installed to $exePath"

# Add to PATH if not already there
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($userPath -notlike "*$installDir*") {
    Write-Info "Adding to PATH..."
    [Environment]::SetEnvironmentVariable("PATH", "$userPath;$installDir", "User")
    $env:PATH = "$env:PATH;$installDir"
    Write-Success "Added $installDir to PATH"
    Write-Host ""
    Write-Host "NOTE: Restart your terminal for PATH changes to take effect." -ForegroundColor Yellow
} else {
    Write-Info "$installDir is already in PATH"
}

# Download default model (to APPDATA, where CLI expects it)
$modelUrl = "https://huggingface.co/animeshkundu/cmd-correct/resolve/main/qwen3-correct-0.6B.gguf"
$modelDir = "$env:APPDATA\fix"
if (-not (Test-Path $modelDir)) {
    New-Item -ItemType Directory -Path $modelDir -Force | Out-Null
}
$modelPath = "$modelDir\qwen3-correct-0.6B.gguf"

if (Test-Path $modelPath) {
    Write-Info "Model already exists at $modelPath"
} else {
    Write-Info "Downloading default model (~378MB)..."
    try {
        Invoke-WebRequest -Uri $modelUrl -OutFile $modelPath -UseBasicParsing
        Write-Success "Model downloaded to $modelPath"
    } catch {
        Write-Host "warning: " -ForegroundColor Yellow -NoNewline
        Write-Host "Model download failed: $_"
        Write-Host "You can retry with: $binary --update"
    }
}

# Verify installation with test command
Write-Info "Testing installation..."
try {
    $testOutput = & $exePath "gti status" 2>&1 | Out-String
    $testOutput = $testOutput.Trim()
    if ($testOutput -eq "git status") {
        Write-Success "Test passed! 'gti status' -> 'git status'"
    } else {
        Write-Host "warning: " -ForegroundColor Yellow -NoNewline
        Write-Host "Test produced: $testOutput"
        Write-Host ""
        Write-Host "If this doesn't look right, try:"
        Write-Host "  - Ensure GPU drivers are up to date"
        Write-Host "  - Run '$binary --verbose gti status' for debug output"
    }
} catch {
    Write-Host "warning: " -ForegroundColor Yellow -NoNewline
    Write-Host "Test failed: $_"
}

# Configure shell integration
$profilePath = $PROFILE.CurrentUserCurrentHost
$fixFunction = @'

# fix - AI-powered shell command corrector
function fix {
    param([Parameter(ValueFromRemainingArguments=$true)]$args)
    $fixPath = "$env:LOCALAPPDATA\fix\fix.exe"
    if ($args) {
        & $fixPath @args
    } else {
        $lastCmd = (Get-History -Count 1).CommandLine
        $corrected = & $fixPath $lastCmd 2>$null
        if ($corrected -and $corrected -ne $lastCmd) {
            Write-Host "Correcting: " -NoNewline
            Write-Host $lastCmd -ForegroundColor Red
            Write-Host "       to: " -NoNewline
            Write-Host $corrected -ForegroundColor Green
            $response = Read-Host "Run? [Y/n]"
            if ($response -ne "n" -and $response -ne "N") {
                Invoke-Expression $corrected
            }
        } else {
            Write-Host "No correction needed"
        }
    }
}
'@

# Check if already configured
$alreadyConfigured = $false
if (Test-Path $profilePath) {
    $profileContent = Get-Content $profilePath -Raw -ErrorAction SilentlyContinue
    if ($profileContent -match "fix - AI-powered shell command corrector") {
        $alreadyConfigured = $true
        Write-Info "Shell integration already configured in $profilePath"
    }
}

if (-not $alreadyConfigured) {
    Write-Info "Configuring shell integration in $profilePath..."

    # Create profile directory if needed
    $profileDir = Split-Path $profilePath -Parent
    if (-not (Test-Path $profileDir)) {
        New-Item -ItemType Directory -Path $profileDir -Force | Out-Null
    }

    # Append to profile
    Add-Content -Path $profilePath -Value $fixFunction
    Write-Success "Shell integration configured"
    Write-Host "  Restart PowerShell to use the 'fix' function."
}

Write-Host ""
Write-Success "Installation complete!"
Write-Host ""
Write-Host "Run '$binary --help' to get started."
