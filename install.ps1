#
#  bitbucket-cli
#  install.ps1
#
#  Created by Ngonidzashe Mangudya on 2026/01/12.
#  Copyright (c) 2025 IAMNGONI. All rights reserved.
#
#  PowerShell install script for bb (Bitbucket CLI)
#  Usage: iwr -useb https://raw.githubusercontent.com/iamngoni/bitbucket-cli/master/install.ps1 | iex

$ErrorActionPreference = 'Stop'

# Configuration
$Repo = "iamngoni/bitbucket-cli"
$BinaryName = "bb"
$InstallDir = if ($env:BB_INSTALL_DIR) { $env:BB_INSTALL_DIR } else { "$env:LOCALAPPDATA\Programs\bb" }

function Write-Info {
    param([string]$Message)
    Write-Host "[INFO] " -ForegroundColor Blue -NoNewline
    Write-Host $Message
}

function Write-Success {
    param([string]$Message)
    Write-Host "[SUCCESS] " -ForegroundColor Green -NoNewline
    Write-Host $Message
}

function Write-Warn {
    param([string]$Message)
    Write-Host "[WARN] " -ForegroundColor Yellow -NoNewline
    Write-Host $Message
}

function Write-Error {
    param([string]$Message)
    Write-Host "[ERROR] " -ForegroundColor Red -NoNewline
    Write-Host $Message
    exit 1
}

function Get-LatestVersion {
    Write-Info "Fetching latest version..."

    try {
        $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest" -UseBasicParsing
        $version = $release.tag_name
        Write-Info "Latest version: $version"
        return $version
    }
    catch {
        Write-Error "Failed to get latest version: $_"
    }
}

function Install-Bb {
    param([string]$Version)

    $Platform = "windows-x86_64"
    $DownloadUrl = "https://github.com/$Repo/releases/download/$Version/bb-$Platform.zip"
    $ChecksumUrl = "$DownloadUrl.sha256"

    Write-Info "Downloading from: $DownloadUrl"

    # Create temporary directory
    $TmpDir = New-Item -ItemType Directory -Path ([System.IO.Path]::GetTempPath()) -Name "bb-install-$([guid]::NewGuid())" -Force

    try {
        # Download binary
        $ZipPath = Join-Path $TmpDir "bb.zip"
        Invoke-WebRequest -Uri $DownloadUrl -OutFile $ZipPath -UseBasicParsing

        # Download and verify checksum
        try {
            $ChecksumPath = Join-Path $TmpDir "bb.zip.sha256"
            Invoke-WebRequest -Uri $ChecksumUrl -OutFile $ChecksumPath -UseBasicParsing

            Write-Info "Verifying checksum..."
            $ExpectedHash = (Get-Content $ChecksumPath).Split()[0].ToLower()
            $ActualHash = (Get-FileHash -Path $ZipPath -Algorithm SHA256).Hash.ToLower()

            if ($ExpectedHash -ne $ActualHash) {
                Write-Error "Checksum verification failed!"
            }
            Write-Success "Checksum verified"
        }
        catch {
            Write-Warn "Could not verify checksum: $_"
        }

        # Extract
        Write-Info "Extracting..."
        Expand-Archive -Path $ZipPath -DestinationPath $TmpDir -Force

        # Create install directory
        if (-not (Test-Path $InstallDir)) {
            New-Item -ItemType Directory -Path $InstallDir -Force | Out-Null
        }

        # Find and install binary
        $Binary = Get-ChildItem -Path $TmpDir -Filter "*.exe" -Recurse | Select-Object -First 1
        if (-not $Binary) {
            Write-Error "Binary not found in archive"
        }

        $DestPath = Join-Path $InstallDir "$BinaryName.exe"
        Copy-Item -Path $Binary.FullName -Destination $DestPath -Force

        Write-Success "Installed $BinaryName to $DestPath"
    }
    finally {
        # Cleanup
        Remove-Item -Path $TmpDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

function Add-ToPath {
    $UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")

    if ($UserPath -notlike "*$InstallDir*") {
        Write-Info "Adding $InstallDir to PATH..."
        [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$InstallDir", "User")
        $env:PATH = "$env:PATH;$InstallDir"
        Write-Success "Added to PATH"
    }
    else {
        Write-Info "$InstallDir is already in PATH"
    }
}

function Test-Installation {
    $BbPath = Join-Path $InstallDir "$BinaryName.exe"

    if (Test-Path $BbPath) {
        try {
            $Version = & $BbPath --version 2>&1 | Select-Object -First 1
            Write-Success "Installation verified: $Version"
        }
        catch {
            Write-Warn "Binary installed but could not verify version"
        }
    }
    else {
        Write-Error "Installation failed - binary not found"
    }
}

function Show-NextSteps {
    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host "  bb (Bitbucket CLI) has been installed!" -ForegroundColor Green
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Get started:"
    Write-Host "  bb auth login    # Authenticate with Bitbucket"
    Write-Host "  bb repo list     # List your repositories"
    Write-Host "  bb --help        # View all commands"
    Write-Host ""
    Write-Host "Enable PowerShell completions:"
    Write-Host "  bb completion powershell | Out-String | Invoke-Expression"
    Write-Host ""
    Write-Host "To make completions permanent, add to your profile:"
    Write-Host "  bb completion powershell >> `$PROFILE"
    Write-Host ""
    Write-Host "Documentation: https://github.com/iamngoni/bitbucket-cli"
    Write-Host ""
    Write-Host "Note: You may need to restart your terminal for PATH changes to take effect."
    Write-Host ""
}

# Main
Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host "  bb (Bitbucket CLI) Installer" -ForegroundColor White
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host ""

$Version = Get-LatestVersion
Install-Bb -Version $Version
Add-ToPath
Test-Installation
Show-NextSteps
