# aig installer for Windows — downloads and installs the latest release.
#
# Usage:
#   irm https://raw.githubusercontent.com/saschb2b/ai-git/main/scripts/install.ps1 | iex
#

$ErrorActionPreference = "Stop"

$Repo = "saschb2b/ai-git"
$Asset = "aig-x86_64-windows.zip"
$Url = "https://github.com/$Repo/releases/latest/download/$Asset"
$InstallDir = "$env:USERPROFILE\.aig\bin"

Write-Host ""
Write-Host "  aig installer" -ForegroundColor White
Write-Host "  Version Control for the AI Age" -ForegroundColor DarkGray
Write-Host ""

# --- Download ---

Write-Host "> Downloading $Asset..." -ForegroundColor Cyan
$TmpDir = New-TemporaryFile | ForEach-Object { Remove-Item $_; New-Item -ItemType Directory -Path $_ }
$ZipPath = Join-Path $TmpDir $Asset

try {
    Invoke-WebRequest -Uri $Url -OutFile $ZipPath -UseBasicParsing
} catch {
    Write-Host "error: download failed - check https://github.com/$Repo/releases" -ForegroundColor Red
    exit 1
}

# --- Extract ---

Write-Host "> Extracting..." -ForegroundColor Cyan
Expand-Archive -Path $ZipPath -DestinationPath $TmpDir -Force

# --- Install ---

if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
}

Move-Item -Path (Join-Path $TmpDir "aig.exe") -Destination (Join-Path $InstallDir "aig.exe") -Force
Write-Host "> Installed to $InstallDir\aig.exe" -ForegroundColor Green

# --- Add to PATH if needed ---

$CurrentPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($CurrentPath -notlike "*$InstallDir*") {
    Write-Host "> Adding $InstallDir to your PATH..." -ForegroundColor Cyan
    [Environment]::SetEnvironmentVariable("Path", "$InstallDir;$CurrentPath", "User")
    $env:PATH = "$InstallDir;$env:PATH"
    Write-Host "> PATH updated (restart your terminal for it to take effect)" -ForegroundColor Green
}

# --- Clean up ---

Remove-Item -Recurse -Force $TmpDir

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
