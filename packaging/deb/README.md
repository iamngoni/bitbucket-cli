# Debian/Ubuntu Packaging

## Building .deb Package

The project uses `cargo-deb` for building Debian packages.

### Prerequisites

```bash
# Install cargo-deb
cargo install cargo-deb
```

### Build

```bash
# Build the .deb package
cargo deb

# The package will be in target/debian/
ls target/debian/*.deb
```

## Distribution Options

### Option 1: GitHub Releases (Simplest)

Users download the .deb directly from GitHub releases:

```bash
# Download the latest release
curl -LO https://github.com/iamngoni/bitbucket-cli/releases/latest/download/bb_0.1.0_amd64.deb

# Install
sudo dpkg -i bb_0.1.0_amd64.deb
```

### Option 2: Launchpad PPA (Recommended for wide distribution)

1. Create a Launchpad account: https://launchpad.net/

2. Create a PPA:
   - Go to your Launchpad profile
   - Click "Create a new PPA"
   - Name it something like "bb-cli"

3. Upload source packages:
   ```bash
   # Install dput
   sudo apt install dput

   # Configure dput for your PPA
   # Add to ~/.dput.cf:
   # [bb-ppa]
   # fqdn = ppa.launchpad.net
   # method = ftp
   # incoming = ~iamngoni/ubuntu/bb-cli/
   # login = anonymous

   # Upload (requires GPG signing)
   dput bb-ppa bb_0.1.0_source.changes
   ```

4. Users can then install with:
   ```bash
   sudo add-apt-repository ppa:iamngoni/bb-cli
   sudo apt update
   sudo apt install bb
   ```

### Option 3: Self-hosted APT Repository

Create your own APT repository using GitHub Pages or a web server:

1. Generate GPG key for signing:
   ```bash
   gpg --full-generate-key
   ```

2. Create repository structure:
   ```bash
   mkdir -p apt-repo/pool/main/b/bb
   mkdir -p apt-repo/dists/stable/main/binary-amd64

   # Copy .deb files
   cp target/debian/*.deb apt-repo/pool/main/b/bb/
   ```

3. Generate Packages index:
   ```bash
   cd apt-repo
   dpkg-scanpackages pool/ | gzip -9c > dists/stable/main/binary-amd64/Packages.gz
   ```

4. Generate Release file and sign it:
   ```bash
   cd dists/stable
   apt-ftparchive release . > Release
   gpg --armor --sign -o Release.gpg Release
   gpg --armor --clearsign -o InRelease Release
   ```

5. Users add your repository:
   ```bash
   curl -fsSL https://yoursite.com/apt-repo/KEY.gpg | sudo apt-key add -
   echo "deb https://yoursite.com/apt-repo stable main" | sudo tee /etc/apt/sources.list.d/bb.list
   sudo apt update
   sudo apt install bb
   ```

## GitHub Actions Integration

The release workflow automatically builds .deb packages and includes them in GitHub releases.
See `.github/workflows/release.yml` for details.
