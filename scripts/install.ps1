# aig installer for Windows — downloads and installs the latest release.
#
# Usage:
#   irm https://raw.githubusercontent.com/saschb2b/ai-git/main/scripts/install.ps1 | iex
#

$ErrorActionPreference = "Stop"

# Force TLS 1.2 (required for GitHub)
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$Repo = "saschb2b/ai-git"
$Asset = "aig-x86_64-windows.zip"
$Url = "https://github.com/$Repo/releases/latest/download/$Asset"
$InstallDir = Join-Path $env:USERPROFILE ".aig\bin"

Write-Host ""
Write-Host "  aig installer" -ForegroundColor White
Write-Host "  Version Control for the AI Age" -ForegroundColor DarkGray
Write-Host ""

# --- Download ---

Write-Host "> Downloading $Asset..." -ForegroundColor Cyan
$TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) "aig-install-$([System.Guid]::NewGuid().ToString('N').Substring(0,8))"
New-Item -ItemType Directory -Path $TmpDir -Force | Out-Null
$ZipPath = Join-Path $TmpDir $Asset

try {
    # Use WebClient — handles GitHub's double redirect better than Invoke-WebRequest
    $wc = New-Object System.Net.WebClient
    $wc.DownloadFile($Url, $ZipPath)
} catch {
    Write-Host "error: download failed" -ForegroundColor Red
    Write-Host "  $_" -ForegroundColor Red
    Write-Host "  Check https://github.com/$Repo/releases for manual download" -ForegroundColor DarkGray
    Read-Host "Press Enter to exit"
    exit 1
}

# --- Extract ---

Write-Host "> Extracting..." -ForegroundColor Cyan
Expand-Archive -Path $ZipPath -DestinationPath $TmpDir -Force

# --- Install ---

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

$ExeSrc = Join-Path $TmpDir "aig.exe"
$ExeDst = Join-Path $InstallDir "aig.exe"

if (-not (Test-Path $ExeSrc)) {
    Write-Host "error: aig.exe not found in archive" -ForegroundColor Red
    Read-Host "Press Enter to exit"
    exit 1
}

Move-Item -Path $ExeSrc -Destination $ExeDst -Force
Write-Host "> Installed to $ExeDst" -ForegroundColor Green

# --- Add to PATH if needed ---

$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($CurrentPath -notlike "*$InstallDir*") {
    Write-Host "> Adding $InstallDir to your PATH..." -ForegroundColor Cyan
    [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$CurrentPath", "User")
    $env:PATH = "$InstallDir;$env:PATH"
    Write-Host "> PATH updated (restart your terminal for it to take effect)" -ForegroundColor Green
}

# --- Clean up ---

Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue

# --- Done ---

Write-Host ""
Write-Host "> aig is ready!" -ForegroundColor Green
Write-Host ""
Write-Host "  Get started:"
Write-Host "    cd your-project" -ForegroundColor Cyan
Write-Host "    aig init --import" -ForegroundColor Cyan
Write-Host "    aig log" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Docs: https://saschb2b.github.io/ai-git/guide/getting-started" -ForegroundColor DarkGray
Write-Host ""
