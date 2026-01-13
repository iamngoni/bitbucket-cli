# Chocolatey Packaging

## Building the Package

### Prerequisites

1. Install Chocolatey: https://chocolatey.org/install
2. Install the Chocolatey package tools:
   ```powershell
   choco install chocolatey-package-builder -y
   ```

### Build

```powershell
cd packaging/chocolatey

# Update the checksum in chocolateyinstall.ps1 with the actual SHA256
# You can get it from the release assets or calculate it:
# (Get-FileHash -Path bb-windows-x86_64.zip -Algorithm SHA256).Hash

# Pack the package
choco pack bb.nuspec

# This creates bb.0.1.0.nupkg
```

### Test Locally

```powershell
# Install from local package
choco install bb -source .

# Test
bb --version
bb --help

# Uninstall
choco uninstall bb
```

## Publishing to Chocolatey Community Repository

### First-time Setup

1. Create a Chocolatey account: https://community.chocolatey.org/account/Register

2. Get your API key from: https://community.chocolatey.org/account

3. Set your API key:
   ```powershell
   choco apikey --key YOUR_API_KEY --source https://push.chocolatey.org/
   ```

### Publishing

```powershell
# Push to Chocolatey
choco push bb.0.1.0.nupkg --source https://push.chocolatey.org/
```

The package will go through moderation before being publicly available.
This typically takes 1-7 days.

## Automated Publishing via GitHub Actions

The release workflow includes a step to publish to Chocolatey.
To enable it:

1. Add your Chocolatey API key as a GitHub secret named `CHOCOLATEY_API_KEY`

2. Uncomment and add this job to `.github/workflows/release.yml`:

```yaml
  publish-chocolatey:
    name: Publish to Chocolatey
    runs-on: windows-latest
    needs: release
    steps:
      - uses: actions/checkout@v4

      - name: Download Windows artifact
        uses: actions/download-artifact@v4
        with:
          name: bb-windows-x86_64
          path: artifacts

      - name: Update checksums
        shell: pwsh
        run: |
          $hash = (Get-FileHash -Path artifacts/bb-windows-x86_64.zip -Algorithm SHA256).Hash
          $content = Get-Content packaging/chocolatey/tools/chocolateyinstall.ps1
          $content = $content -replace 'PLACEHOLDER_SHA256_WINDOWS_X86_64', $hash.ToLower()
          Set-Content -Path packaging/chocolatey/tools/chocolateyinstall.ps1 -Value $content

      - name: Pack and push
        shell: pwsh
        run: |
          cd packaging/chocolatey
          choco pack bb.nuspec
          choco push bb.*.nupkg --source https://push.chocolatey.org/ --api-key ${{ secrets.CHOCOLATEY_API_KEY }}
```

## Version Updates

When releasing a new version:

1. Update `version` in `bb.nuspec`
2. Update `$version` in `tools/chocolateyinstall.ps1`
3. Update the checksum in `tools/chocolateyinstall.ps1`
