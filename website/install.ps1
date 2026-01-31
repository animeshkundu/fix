# fix installer for Windows
# Usage: iwr -useb https://animeshkundu.github.io/fix/install.ps1 | iex

$ErrorActionPreference = 'Stop'

$repo = "animeshkundu/fix"
$binary = "fix"

function Write-Info { param($msg) Write-Host "==> " -ForegroundColor Blue -NoNewline; Write-Host $msg }
function Write-Success { param($msg) Write-Host "==> " -ForegroundColor Green -NoNewline; Write-Host $msg }
function Write-Err { param($msg) Write-Host "error: " -ForegroundColor Red -NoNewline; Write-Host $msg }

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

if (-not $asset) {
    Write-Err "No binary found for $target"
    Write-Host "Available assets:"
    $release.assets | ForEach-Object { Write-Host "  - $($_.name)" }
    exit 1
}

$downloadUrl = $asset.browser_download_url
Write-Info "Downloading from: $downloadUrl"

# Create install directory
$installDir = "$env:LOCALAPPDATA\$binary"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

# Download and extract
$tempFile = "$env:TEMP\$assetName"
try {
    Invoke-WebRequest -Uri $downloadUrl -OutFile $tempFile -UseBasicParsing
    Expand-Archive -Path $tempFile -DestinationPath $installDir -Force
    Remove-Item $tempFile -Force
} catch {
    Write-Err "Download failed: $_"
    exit 1
}

$exePath = "$installDir\$binary.exe"
if (-not (Test-Path $exePath)) {
    Write-Err "Binary not found after extraction"
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

# Download default model
$modelUrl = "https://huggingface.co/animeshkundu/cmd-correct/resolve/main/qwen3-correct-0.6B.gguf"
$modelPath = "$installDir\qwen3-correct-0.6B.gguf"

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
            Write-Host "Correcting: $lastCmd -> $corrected" -ForegroundColor Cyan
            [Microsoft.PowerShell.PSConsoleReadLine]::Insert($corrected)
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
