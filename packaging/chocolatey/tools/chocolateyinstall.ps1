#
#  bitbucket-cli
#  packaging/chocolatey/tools/chocolateyinstall.ps1
#
#  Created by Ngonidzashe Mangudya on 2026/01/12.
#  Copyright (c) 2025 IAMNGONI. All rights reserved.
#

$ErrorActionPreference = 'Stop'

$packageName = 'bb'
$version = '0.1.0'

# URL to the release zip file
$url64 = "https://github.com/iamngoni/bitbucket-cli/releases/download/v$version/bb-windows-x86_64.zip"

# SHA256 checksum (update with each release)
$checksum64 = 'PLACEHOLDER_SHA256_WINDOWS_X86_64'

$packageArgs = @{
    packageName    = $packageName
    unzipLocation  = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
    url64bit       = $url64
    checksum64     = $checksum64
    checksumType64 = 'sha256'
}

Install-ChocolateyZipPackage @packageArgs

# Create shim for the executable
$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"
$exePath = Join-Path $toolsDir "bb-windows-x86_64.exe"
$shimPath = Join-Path $toolsDir "bb.exe"

# Rename to bb.exe for easier access
if (Test-Path $exePath) {
    Move-Item -Path $exePath -Destination $shimPath -Force
}

Write-Host ""
Write-Host "bb (Bitbucket CLI) has been installed!" -ForegroundColor Green
Write-Host ""
Write-Host "Get started with:"
Write-Host "  bb auth login    # Authenticate with Bitbucket"
Write-Host "  bb --help        # View all commands"
Write-Host ""
