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

Write-Host ""
Write-Success "Installation complete!"
Write-Host ""
Write-Host "Run '$binary --help' to get started."
