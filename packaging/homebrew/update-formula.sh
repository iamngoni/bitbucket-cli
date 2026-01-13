#!/bin/bash
#
#  bitbucket-cli
#  packaging/homebrew/update-formula.sh
#
#  Created by Ngonidzashe Mangudya on 2026/01/12.
#  Copyright (c) 2025 IAMNGONI. All rights reserved.
#

# Update Homebrew formula with new version and SHA256 hashes
# Usage: ./update-formula.sh <version>
# Example: ./update-formula.sh 0.1.0

set -e

VERSION="${1:-}"
if [ -z "$VERSION" ]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 0.1.0"
    exit 1
fi

REPO="iamngoni/bitbucket-cli"
BASE_URL="https://github.com/${REPO}/releases/download/v${VERSION}"

echo "Downloading release assets for v${VERSION}..."

# Download and calculate SHA256 for each platform
declare -A PLATFORMS=(
    ["darwin-x86_64"]="PLACEHOLDER_SHA256_DARWIN_X86_64"
    ["darwin-aarch64"]="PLACEHOLDER_SHA256_DARWIN_AARCH64"
    ["linux-x86_64"]="PLACEHOLDER_SHA256_LINUX_X86_64"
    ["linux-aarch64"]="PLACEHOLDER_SHA256_LINUX_AARCH64"
)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
FORMULA_FILE="${SCRIPT_DIR}/bb.rb"

# Create a temporary copy
cp "$FORMULA_FILE" "${FORMULA_FILE}.tmp"

# Update version
sed -i.bak "s/version \".*\"/version \"${VERSION}\"/" "${FORMULA_FILE}.tmp"

for platform in "${!PLATFORMS[@]}"; do
    url="${BASE_URL}/bb-${platform}.tar.gz"
    echo "Fetching SHA256 for ${platform}..."

    # Download the SHA256 file or calculate from tarball
    sha256_url="${url}.sha256"
    sha256=$(curl -sL "$sha256_url" | awk '{print $1}' 2>/dev/null || \
             curl -sL "$url" | sha256sum | awk '{print $1}')

    if [ -n "$sha256" ]; then
        placeholder="${PLATFORMS[$platform]}"
        sed -i.bak "s/${placeholder}/${sha256}/" "${FORMULA_FILE}.tmp"
        echo "  ${platform}: ${sha256}"
    else
        echo "  Warning: Could not get SHA256 for ${platform}"
    fi
done

# Clean up and replace
rm -f "${FORMULA_FILE}.tmp.bak"
mv "${FORMULA_FILE}.tmp" "$FORMULA_FILE"

echo ""
echo "Formula updated successfully!"
echo "Don't forget to commit and push the changes to your Homebrew tap."
