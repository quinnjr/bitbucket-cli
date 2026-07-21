# Build script for creating Windows MSI installer
# Usage: .\scripts\build-packages.ps1

$ErrorActionPreference = "Stop"

$ProjectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $ProjectRoot

function Build-MSI {
    Write-Host "Building Windows MSI installer..." -ForegroundColor Green
    
    # Check if WiX is installed
    if (!(Get-Command wix -ErrorAction SilentlyContinue)) {
        Write-Host "Installing WiX..." -ForegroundColor Yellow
        dotnet tool install --global wix
        wix extension add -g WixToolset.UI.wixext
    }
    
    # Build the release binary
    Write-Host "Building release binary..." -ForegroundColor Yellow
    cargo build --release --target x86_64-pc-windows-msvc
    
    # Build MSI
    Write-Host "Building MSI package..." -ForegroundColor Yellow
    wix build -arch x64 -o bitbucket-cli.msi packaging\windows\main.wxs
    
    Write-Host "✓ MSI installer built: bitbucket-cli.msi" -ForegroundColor Green
}

Build-MSI
