#
#  bitbucket-cli
#  packaging/chocolatey/tools/chocolateyuninstall.ps1
#
#  Created by Ngonidzashe Mangudya on 2026/01/12.
#  Copyright (c) 2025 IAMNGONI. All rights reserved.
#

$ErrorActionPreference = 'Stop'

$packageName = 'bb'
$toolsDir = "$(Split-Path -Parent $MyInvocation.MyCommand.Definition)"

# Remove the executable
$exePath = Join-Path $toolsDir "bb.exe"
if (Test-Path $exePath) {
    Remove-Item -Path $exePath -Force
    Write-Host "Removed bb.exe" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "bb (Bitbucket CLI) has been uninstalled." -ForegroundColor Yellow
Write-Host ""
Write-Host "Note: Your configuration files remain at:"
Write-Host "  $env:APPDATA\bb\"
Write-Host ""
Write-Host "To completely remove all data, delete this folder manually."
Write-Host ""
