# bb - Bitbucket CLI

[![CI](https://github.com/iamngoni/bitbucket-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/iamngoni/bitbucket-cli/actions/workflows/ci.yml)
[![Release](https://github.com/iamngoni/bitbucket-cli/actions/workflows/release.yml/badge.svg)](https://github.com/iamngoni/bitbucket-cli/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A powerful command-line interface for Bitbucket that brings repository management, pull requests, pipelines, and team collaboration directly to the terminal. Supports both **Bitbucket Cloud** and **Bitbucket Server/Data Center**.

## Features

- **Repository Management** - Clone, create, fork, list, and manage repositories
- **Pull Requests** - Create, review, merge, approve, and manage PRs with full workflow support
- **Pipelines** - Trigger, monitor, and view CI/CD pipeline logs (Bitbucket Cloud)
- **Issues** - Create and manage issues with Jira integration
- **Workspaces & Projects** - Navigate and manage organizational structures
- **Interactive & Scriptable** - Rich terminal UI with JSON output for automation
- **Multi-Platform** - First-class support for both Cloud and Server/Data Center
- **Secure** - Credentials stored securely in system keychain

## Installation

### Quick Install (Recommended)

**macOS/Linux:**
```bash
curl -fsSL https://raw.githubusercontent.com/iamngoni/bitbucket-cli/master/install.sh | sh
```

**Windows (PowerShell):**
```powershell
iwr -useb https://raw.githubusercontent.com/iamngoni/bitbucket-cli/master/install.ps1 | iex
```

### Package Managers

**Homebrew (macOS/Linux):**
```bash
brew install iamngoni/tap/bb
```

**Debian/Ubuntu:**
```bash
# Download the latest .deb package
curl -LO https://github.com/iamngoni/bitbucket-cli/releases/latest/download/bb_amd64.deb
sudo dpkg -i bb_amd64.deb
```

**Chocolatey (Windows):**
```powershell
choco install bb
```

### From Source

```bash
# Requires Rust 1.75+
cargo install --git https://github.com/iamngoni/bitbucket-cli
```

### Manual Download

Download the appropriate binary for your platform from the [Releases](https://github.com/iamngoni/bitbucket-cli/releases) page.

| Platform | Architecture | Download |
|----------|--------------|----------|
| Linux | x86_64 | `bb-linux-x86_64.tar.gz` |
| Linux | ARM64 | `bb-linux-aarch64.tar.gz` |
| macOS | Intel | `bb-darwin-x86_64.tar.gz` |
| macOS | Apple Silicon | `bb-darwin-aarch64.tar.gz` |
| Windows | x86_64 | `bb-windows-x86_64.zip` |

## Quick Start

### 1. Authenticate

**Bitbucket Cloud:**
```bash
bb auth login --cloud
```

**Bitbucket Server/Data Center:**
```bash
bb auth login --server --host bitbucket.yourcompany.com
```

### 2. Work with Repositories

```bash
# List your repositories
bb repo list

# Clone a repository
bb repo clone myworkspace/myrepo

# View repository details
bb repo view
```

### 3. Manage Pull Requests

```bash
# List open PRs
bb pr list

# Create a new PR
bb pr create --title "My feature" --base main

# Review and approve
bb pr approve 123

# Merge
bb pr merge 123
```

### 4. Monitor Pipelines (Cloud)

```bash
# List pipeline runs
bb pipeline list

# Trigger a pipeline
bb pipeline run --branch main

# View logs
bb pipeline logs 12345 --follow
```

## Command Reference

```
bb auth          Authentication management
bb repo          Repository operations
bb pr            Pull request operations
bb issue         Issue tracker operations
bb pipeline      CI/CD pipeline operations (Cloud)
bb workspace     Workspace management (Cloud)
bb project       Project management (Server/DC)
bb browse        Open resources in browser
bb api           Direct API calls
bb config        CLI configuration
bb alias         Command aliases
bb extension     CLI extensions
bb completion    Shell completions
```

For detailed help on any command:
```bash
bb <command> --help
```

## Configuration

Configuration is stored in:
- **Linux**: `~/.config/bb/config.toml`
- **macOS**: `~/Library/Application Support/bb/config.toml`
- **Windows**: `%APPDATA%\bb\config.toml`

### Environment Variables

| Variable | Description |
|----------|-------------|
| `BB_TOKEN` | Authentication token (overrides keychain) |
| `BB_HOST` | Default host for Server/DC |
| `BB_WORKSPACE` | Default workspace (Cloud) |
| `BB_PAGER` | Pager for output (default: system pager) |
| `BB_EDITOR` | Editor for text input |
| `BB_NO_PROMPT` | Disable interactive prompts |
| `NO_COLOR` | Disable colored output |

## Shell Completions

```bash
# Bash
bb completion bash >> ~/.bashrc

# Zsh
bb completion zsh >> ~/.zshrc

# Fish
bb completion fish > ~/.config/fish/completions/bb.fish

# PowerShell
bb completion powershell >> $PROFILE
```

## Aliases

Create shortcuts for frequently used commands:

```bash
# Create an alias
bb alias set prs "pr list --state open"

# Use the alias
bb prs

# List all aliases
bb alias list
```

## JSON Output

All commands support `--json` for scriptable output:

```bash
# Get PR data as JSON
bb pr list --json

# Use with jq
bb pr view 123 --json | jq '.title'
```

## Platform Differences

| Feature | Cloud | Server/DC |
|---------|-------|-----------|
| Pipelines | Yes | No |
| Workspaces | Yes | No |
| Projects | Limited | Yes |
| OAuth 2.0 | Yes | Limited |
| Personal Access Tokens | No | Yes |
| Native Issues | Yes | No |
| Jira Integration | Yes | Yes |

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

```bash
# Clone the repository
git clone https://github.com/iamngoni/bitbucket-cli
cd bitbucket-cli

# Build
cargo build

# Run tests
cargo test

# Run the CLI
cargo run -- --help
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Author

**Ngonidzashe Mangudya** - [@iamngoni](https://github.com/iamngoni)

## Co-Author

[Claude Code](https://claude.ai/code)

---

*Inspired by [GitHub CLI (gh)](https://cli.github.com)*
