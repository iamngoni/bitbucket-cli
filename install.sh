#!/bin/sh
#
#  bitbucket-cli
#  install.sh
#
#  Created by Ngonidzashe Mangudya on 2026/01/12.
#  Copyright (c) 2025 IAMNGONI. All rights reserved.
#
#  Universal install script for bb (Bitbucket CLI)
#  Usage: curl -fsSL https://raw.githubusercontent.com/iamngoni/bitbucket-cli/master/install.sh | sh

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
REPO="iamngoni/bitbucket-cli"
BINARY_NAME="bb"
INSTALL_DIR="${BB_INSTALL_DIR:-/usr/local/bin}"

# Print colored message
info() {
    printf "${BLUE}[INFO]${NC} %s\n" "$1"
}

success() {
    printf "${GREEN}[SUCCESS]${NC} %s\n" "$1"
}

warn() {
    printf "${YELLOW}[WARN]${NC} %s\n" "$1"
}

error() {
    printf "${RED}[ERROR]${NC} %s\n" "$1"
    exit 1
}

# Detect OS and architecture
detect_platform() {
    OS="$(uname -s)"
    ARCH="$(uname -m)"

    case "$OS" in
        Linux*)
            OS="linux"
            ;;
        Darwin*)
            OS="darwin"
            ;;
        MINGW*|MSYS*|CYGWIN*)
            error "Please use the PowerShell installer for Windows: install.ps1"
            ;;
        *)
            error "Unsupported operating system: $OS"
            ;;
    esac

    case "$ARCH" in
        x86_64|amd64)
            ARCH="x86_64"
            ;;
        aarch64|arm64)
            ARCH="aarch64"
            ;;
        *)
            error "Unsupported architecture: $ARCH"
            ;;
    esac

    PLATFORM="${OS}-${ARCH}"
    info "Detected platform: $PLATFORM"
}

# Get the latest release version
get_latest_version() {
    if command -v curl >/dev/null 2>&1; then
        VERSION=$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    elif command -v wget >/dev/null 2>&1; then
        VERSION=$(wget -qO- "https://api.github.com/repos/${REPO}/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
    else
        error "Neither curl nor wget found. Please install one of them."
    fi

    if [ -z "$VERSION" ]; then
        error "Failed to get latest version. Please check your internet connection."
    fi

    info "Latest version: $VERSION"
}

# Download and install
download_and_install() {
    DOWNLOAD_URL="https://github.com/${REPO}/releases/download/${VERSION}/bb-${PLATFORM}.tar.gz"
    CHECKSUM_URL="${DOWNLOAD_URL}.sha256"

    info "Downloading from: $DOWNLOAD_URL"

    # Create temporary directory
    TMP_DIR=$(mktemp -d)
    trap 'rm -rf "$TMP_DIR"' EXIT

    # Download binary
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL "$DOWNLOAD_URL" -o "$TMP_DIR/bb.tar.gz"
        curl -fsSL "$CHECKSUM_URL" -o "$TMP_DIR/bb.tar.gz.sha256" 2>/dev/null || true
    else
        wget -q "$DOWNLOAD_URL" -O "$TMP_DIR/bb.tar.gz"
        wget -q "$CHECKSUM_URL" -O "$TMP_DIR/bb.tar.gz.sha256" 2>/dev/null || true
    fi

    # Verify checksum if available
    if [ -f "$TMP_DIR/bb.tar.gz.sha256" ]; then
        info "Verifying checksum..."
        cd "$TMP_DIR"
        if command -v sha256sum >/dev/null 2>&1; then
            sha256sum -c bb.tar.gz.sha256 >/dev/null 2>&1 || error "Checksum verification failed!"
        elif command -v shasum >/dev/null 2>&1; then
            EXPECTED=$(cat bb.tar.gz.sha256 | awk '{print $1}')
            ACTUAL=$(shasum -a 256 bb.tar.gz | awk '{print $1}')
            [ "$EXPECTED" = "$ACTUAL" ] || error "Checksum verification failed!"
        else
            warn "No checksum tool found, skipping verification"
        fi
        success "Checksum verified"
    fi

    # Extract
    info "Extracting..."
    tar -xzf "$TMP_DIR/bb.tar.gz" -C "$TMP_DIR"

    # Find the binary
    BINARY=$(find "$TMP_DIR" -name "bb-*" -type f | head -1)
    if [ -z "$BINARY" ]; then
        BINARY="$TMP_DIR/$BINARY_NAME"
    fi

    if [ ! -f "$BINARY" ]; then
        error "Binary not found in archive"
    fi

    # Install
    info "Installing to $INSTALL_DIR..."

    # Check if we need sudo
    if [ -w "$INSTALL_DIR" ]; then
        mv "$BINARY" "$INSTALL_DIR/$BINARY_NAME"
        chmod +x "$INSTALL_DIR/$BINARY_NAME"
    else
        if command -v sudo >/dev/null 2>&1; then
            sudo mv "$BINARY" "$INSTALL_DIR/$BINARY_NAME"
            sudo chmod +x "$INSTALL_DIR/$BINARY_NAME"
        else
            error "Cannot write to $INSTALL_DIR and sudo is not available. Try: BB_INSTALL_DIR=~/.local/bin $0"
        fi
    fi

    success "Installed $BINARY_NAME to $INSTALL_DIR/$BINARY_NAME"
}

# Verify installation
verify_installation() {
    if command -v "$BINARY_NAME" >/dev/null 2>&1; then
        INSTALLED_VERSION=$("$BINARY_NAME" --version 2>/dev/null | head -1)
        success "Installation verified: $INSTALLED_VERSION"
    else
        warn "$BINARY_NAME is installed but not in PATH"
        warn "Add $INSTALL_DIR to your PATH:"
        warn "  export PATH=\"\$PATH:$INSTALL_DIR\""
    fi
}

# Print next steps
print_next_steps() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    printf "${GREEN}bb (Bitbucket CLI) has been installed!${NC}\n"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""
    echo "Get started:"
    echo "  bb auth login    # Authenticate with Bitbucket"
    echo "  bb repo list     # List your repositories"
    echo "  bb --help        # View all commands"
    echo ""
    echo "Enable shell completions:"
    echo "  # Bash"
    echo "  bb completion bash >> ~/.bashrc"
    echo ""
    echo "  # Zsh"
    echo "  bb completion zsh >> ~/.zshrc"
    echo ""
    echo "  # Fish"
    echo "  bb completion fish > ~/.config/fish/completions/bb.fish"
    echo ""
    echo "Documentation: https://github.com/iamngoni/bitbucket-cli"
    echo ""
}

# Main
main() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  bb (Bitbucket CLI) Installer"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo ""

    detect_platform
    get_latest_version
    download_and_install
    verify_installation
    print_next_steps
}

main
