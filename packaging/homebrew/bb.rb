#
#  bitbucket-cli
#  packaging/homebrew/bb.rb
#
#  Created by Ngonidzashe Mangudya on 2026/01/12.
#  Copyright (c) 2025 IAMNGONI. All rights reserved.
#

# Homebrew Formula for bb (Bitbucket CLI)
#
# To use this formula:
# 1. Create a tap repository: https://github.com/iamngoni/homebrew-tap
# 2. Copy this file to Formula/bb.rb in that repository
# 3. Users can then install with: brew install iamngoni/tap/bb
#
# Or submit to homebrew-core for wider distribution.

class Bb < Formula
  desc "Command-line interface for Bitbucket Cloud and Server/Data Center"
  homepage "https://github.com/iamngoni/bitbucket-cli"
  version "0.1.0"
  license "MIT"

  on_macos do
    on_intel do
      url "https://github.com/iamngoni/bitbucket-cli/releases/download/v#{version}/bb-darwin-x86_64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_DARWIN_X86_64"
    end

    on_arm do
      url "https://github.com/iamngoni/bitbucket-cli/releases/download/v#{version}/bb-darwin-aarch64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_DARWIN_AARCH64"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/iamngoni/bitbucket-cli/releases/download/v#{version}/bb-linux-x86_64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_X86_64"
    end

    on_arm do
      url "https://github.com/iamngoni/bitbucket-cli/releases/download/v#{version}/bb-linux-aarch64.tar.gz"
      sha256 "PLACEHOLDER_SHA256_LINUX_AARCH64"
    end
  end

  def install
    if OS.mac?
      if Hardware::CPU.arm?
        bin.install "bb-darwin-aarch64" => "bb"
      else
        bin.install "bb-darwin-x86_64" => "bb"
      end
    elsif OS.linux?
      if Hardware::CPU.arm?
        bin.install "bb-linux-aarch64" => "bb"
      else
        bin.install "bb-linux-x86_64" => "bb"
      end
    end

    # Generate shell completions
    generate_completions_from_executable(bin/"bb", "completion")
  end

  test do
    assert_match "bb version #{version}", shell_output("#{bin}/bb version")
  end
end
