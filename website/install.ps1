# fix/wit installer for Windows - Installs both binaries
# Usage: iwr -useb https://animeshkundu.github.io/fix/install.ps1 | iex
# Usage: iwr -useb https://animeshkundu.github.io/fix/install.ps1 | iex -args fix  (fix as primary)

$ErrorActionPreference = 'Stop'

$repo = "animeshkundu/fix"
# Default to 'wit' as primary, 'fix' as secondary
$primary = if ($args.Count -gt 0) { $args[0] } else { "wit" }
$secondary = if ($primary -eq "wit") { "fix" } else { "wit" }

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
    $vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vsWhere) {
        $installPath = & $vsWhere -latest -property installationPath 2>$null
        if ($installPath) {
            return $true
        }
    }
    return $false
}

# Build a single binary from source
function Build-BinaryFromSource {
    param($binaryName, $installDir)

    Write-Info "Building $binaryName from source..."

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

    Write-Info "Building $binaryName with cargo (this may take several minutes)..."
    try {
        $env:PATH = "$env:USERPROFILE\.cargo\bin;$env:PATH"
        & cargo install --git "https://github.com/$repo" $binaryName

        # Copy from cargo bin to install dir
        $cargoBin = "$env:USERPROFILE\.cargo\bin\$binaryName.exe"
        if (Test-Path $cargoBin) {
            if (-not (Test-Path $installDir)) {
                New-Item -ItemType Directory -Path $installDir -Force | Out-Null
            }
            Copy-Item $cargoBin -Destination "$installDir\$binaryName.exe" -Force
            Write-Success "Built and installed $binaryName to $installDir\$binaryName.exe"
            return $true
        }
    } catch {
        Write-Err "Build of $binaryName failed: $_"
    }
    return $false
}

# Install a single binary (download or build)
function Install-Binary {
    param($binaryName, $installDir)

    Write-Info "Installing $binaryName..."

    # Try downloading pre-built binary
    try {
        $release = Invoke-RestMethod "https://api.github.com/repos/$repo/releases/latest"
        $version = $release.tag_name
    } catch {
        Write-Warn "Failed to fetch latest release for $binaryName"
        return (Invoke-BuildFromSource $binaryName $installDir)
    }

    $assetName = "$binaryName-$target.zip"
    $asset = $release.assets | Where-Object { $_.name -eq $assetName }

    if (-not $asset) {
        Write-Warn "No pre-built $binaryName binary found for $target"
        return (Invoke-BuildFromSource $binaryName $installDir)
    }

    $downloadUrl = $asset.browser_download_url
    Write-Info "Downloading $binaryName from: $downloadUrl"

    # Download and extract
    $tempFile = "$env:TEMP\$assetName"
    try {
        Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -UseBasicParsing
        Expand-Archive -Path $tempFile -DestinationPath $installDir -Force
        Remove-Item $tempFile -Force
        Write-Success "Installed $binaryName to $installDir\$binaryName.exe"
        return $true
    } catch {
        Write-Warn "Download of $binaryName failed: $_"
        return (Invoke-BuildFromSource $binaryName $installDir)
    }
}

# Offer to build from source
function Invoke-BuildFromSource {
    param($binaryName, $installDir)

    Write-Host ""
    Write-Host "Would you like to build $binaryName from source? [y/N]" -ForegroundColor Yellow
    $response = Read-Host
    if ($response -eq "y" -or $response -eq "Y") {
        return (Build-BinaryFromSource $binaryName $installDir)
    } else {
        Write-Host ""
        Write-Host "You can build manually later with:"
        Write-Host "  cargo install --git https://github.com/$repo $binaryName"
        return $false
    }
}

# Download model for a specific binary
function Download-Model {
    param($binaryName)

    if ($binaryName -eq "wit") {
        $modelName = "qwen3-wit-1.7B.gguf"
        $modelSize = "~1GB"
    } else {
        $modelName = "qwen3-correct-0.6B.gguf"
        $modelSize = "~378MB"
    }

    $modelUrl = "https://huggingface.co/animeshkundu/cmd-correct/resolve/main/$modelName"
    $modelDir = "$env:APPDATA\fix"
    if (-not (Test-Path $modelDir)) {
        New-Item -ItemType Directory -Path $modelDir -Force | Out-Null
    }
    $modelPath = "$modelDir\$modelName"

    if (Test-Path $modelPath) {
        Write-Info "Model already exists at $modelPath"
    } else {
        Write-Info "Downloading $binaryName model ($modelSize)..."
        try {
            Invoke-WebRequest -Uri $modelUrl -OutFile $modelPath -UseBasicParsing
            Write-Success "Model downloaded to $modelPath"
        } catch {
            Write-Warn "Model download failed: $_"
            Write-Host "You can retry with: $binaryName --update"
        }
    }
}

# Test a binary installation
function Test-Binary {
    param($binaryName, $exePath)

    Write-Info "Testing $binaryName..."
    try {
        if ($binaryName -eq "wit") {
            $testOutput = & $exePath "--show-config" 2>&1 | Out-String
            $testOutput = $testOutput.Trim()
            if ($testOutput -match "(Configuration:|Wit model:)") {
                Write-Success "Test passed! $binaryName --show-config works"
                return $true
            }
        } else {
            $testOutput = & $exePath "gti status" 2>&1 | Out-String
            $testOutput = $testOutput.Trim()
            if ($testOutput -eq "git status") {
                Write-Success "Test passed! ${binaryName}: 'gti status' -> 'git status'"
                return $true
            }
        }
        Write-Warn "Test for $binaryName produced: $testOutput"
        return $false
    } catch {
        Write-Warn "Test for $binaryName failed: $_"
        return $false
    }
}

# Configure shell integration for a specific binary
function Configure-ShellIntegration {
    param($binaryName)

    $profilePath = $PROFILE.CurrentUserCurrentHost
    $marker = "$binaryName - AI-powered shell command corrector"

    # Generate the function for this binary using literal here-string
    $shellFunction = @'

# __BINARY_NAME__ - AI-powered shell command corrector
function __BINARY_NAME__ {
    param([Parameter(ValueFromRemainingArguments=$true)]$args)
    $binPath = "$env:LOCALAPPDATA\fix\__BINARY_NAME__.exe"
    if ($args) {
        & $binPath @args
    } else {
        $lastCmd = (Get-History -Count 1).CommandLine
        $corrected = & $binPath $lastCmd 2>$null
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
    # Replace placeholder with actual binary name
    $shellFunction = $shellFunction -replace '__BINARY_NAME__', $binaryName

    # Check if already configured
    $alreadyConfigured = $false
    if (Test-Path $profilePath) {
        $profileContent = Get-Content $profilePath -Raw -ErrorAction SilentlyContinue
        if ($profileContent -match [regex]::Escape($marker)) {
            $alreadyConfigured = $true
            Write-Info "Shell integration for $binaryName already configured in $profilePath"
        }
    }

    if (-not $alreadyConfigured) {
        Write-Info "Configuring shell integration for $binaryName in $profilePath..."

        # Create profile directory if needed
        $profileDir = Split-Path $profilePath -Parent
        if (-not (Test-Path $profileDir)) {
            New-Item -ItemType Directory -Path $profileDir -Force | Out-Null
        }

        # Append to profile
        Add-Content -Path $profilePath -Value $shellFunction
        Write-Success "Shell integration for $binaryName configured"
        Write-Host "  Restart PowerShell to use the '$binaryName' function."
    }
}

# Main installation
Write-Info "Installing wit and fix CLI tools..."
Write-Info "Primary: $primary, Secondary: $secondary"

# Detect architecture
$arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "i686" }
$target = "$arch-pc-windows-msvc"
Write-Info "Detected: Windows ($arch)"

# Install directory (shared for both binaries)
$installDir = "$env:LOCALAPPDATA\fix"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

# Install both binaries
$primaryInstalled = Install-Binary $primary $installDir
$secondaryInstalled = Install-Binary $secondary $installDir

if (-not $primaryInstalled) {
    Write-Err "Failed to install primary binary ($primary). Please try building from source manually."
    exit 1
}

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

# Download model for primary binary only
Download-Model $primary

# Test installations
if ($primaryInstalled) {
    Test-Binary $primary "$installDir\$primary.exe" | Out-Null
}
if ($secondaryInstalled) {
    Test-Binary $secondary "$installDir\$secondary.exe" | Out-Null
}

# Configure shell integration for primary binary
Configure-ShellIntegration $primary

Write-Host ""
Write-Success "Installation complete!"
Write-Host ""
Write-Host "Installed binaries:"
if ($primaryInstalled) { Write-Host "  - $primary (primary, with shell integration)" }
if ($secondaryInstalled) { Write-Host "  - $secondary" }
Write-Host ""
Write-Host "Run '$primary --help' to get started."
if ($secondaryInstalled) {
    Write-Host "Run '$secondary --help' for the alternative corrector."
}
