# Bitbucket CLI ("bb") Specification

> A comprehensive command-line interface for Bitbucket Cloud and Server/Data Center

## Executive Summary

`bb` is a Rust-based CLI that brings repository management, pull requests, pipelines, and team collaboration directly to the terminal. It supports both Bitbucket Cloud and Bitbucket Server/Data Center with a unified command interface.

### Design Principles

1. **gh Familiarity**: Mirror GitHub CLI's intuitive verb-noun structure
2. **Platform Parity**: First-class support for both Cloud and Server/DC
3. **Dual Output**: Rich interactive experience + scriptable JSON output
4. **Bitbucket Native**: Leverage unique features (Pipelines, Workspaces)
5. **Rust Performance**: Fast, cross-platform, single binary

---

## Command Reference

### Top-Level Commands

```
bb
├── auth          # Authentication management
├── repo          # Repository operations
├── pr            # Pull request operations
├── issue         # Issue tracker operations
├── pipeline      # CI/CD pipeline operations (Cloud)
├── workspace     # Workspace management (Cloud)
├── project       # Project management (Server/DC primary)
├── browse        # Open resources in browser
├── api           # Direct API calls
├── config        # CLI configuration
├── alias         # Command aliases
├── extension     # CLI extensions
├── webhook       # Webhook management
├── deploy        # Deployment operations
├── artifact      # Build artifact operations
├── secret        # Secret/variable management
├── ssh-key       # SSH key management
├── completion    # Shell completion scripts
├── version       # Version information
└── help          # Help documentation
```

---

## `bb auth` - Authentication Management

Manage authentication for Bitbucket Cloud and Server/Data Center.

### Commands

```bash
bb auth login         # Authenticate with Bitbucket
    --cloud           # Force Cloud authentication
    --server          # Force Server/DC authentication
    --host <host>     # Specify host for Server/DC
    --with-token      # Read token from stdin
    --scopes <scopes> # OAuth scopes (Cloud)
    --web             # Open browser for OAuth

bb auth logout        # Remove authentication
    --host <host>     # Specific host
    --all             # All hosts

bb auth status        # View authentication status
    --show-token      # Display token (masked)

bb auth refresh       # Refresh OAuth tokens

bb auth switch        # Switch between profiles
    --profile <name>

bb auth token         # Print current token
    --host <host>

bb auth setup-git     # Configure git credential helper
```

### Authentication Methods

| Platform | Method | Description |
|----------|--------|-------------|
| Cloud | OAuth 2.0 | Browser-based OAuth flow |
| Cloud | App Password | Username + app password (deprecated) |
| Server/DC | Personal Access Token | Token-based authentication |
| Server/DC | Basic Auth | Username + password |

---

## `bb repo` - Repository Operations

Manage repositories across workspaces and projects.

### Commands

```bash
bb repo list          # List repositories
    --workspace <ws>  # Filter by workspace (Cloud)
    --project <proj>  # Filter by project
    --limit <n>       # Limit results
    --language <lang> # Filter by language
    --visibility <v>  # public/private
    --archived        # Include archived
    --json            # JSON output

bb repo view          # View repository details
    --web             # Open in browser
    --json

bb repo create        # Create new repository
    --public/--private
    --description <desc>
    --clone           # Clone after creation
    --workspace <ws>  # Target workspace (Cloud)
    --project <proj>  # Target project
    --template <repo>

bb repo clone         # Clone repository
    -- <git-flags>    # Pass-through to git

bb repo fork          # Fork repository
    --workspace <ws>
    --clone
    --remote-name <name>

bb repo delete        # Delete repository
    --confirm

bb repo archive       # Archive repository
bb repo unarchive     # Unarchive repository
bb repo rename        # Rename repository

bb repo sync          # Sync fork with upstream
    --branch <branch>
    --force

bb repo edit          # Edit repository settings
    --description <desc>
    --default-branch <br>
    --enable-issues
    --enable-wiki
    --visibility <v>

bb repo browse        # Open repo in browser
bb repo credits       # View contributors
```

---

## `bb pr` - Pull Request Operations

Complete pull request workflow from creation to merge.

### Commands

```bash
bb pr list            # List pull requests
    --state <state>   # open/merged/declined/superseded
    --author <user>
    --reviewer <user>
    --assignee <user>
    --base <branch>   # Target branch
    --head <branch>   # Source branch
    --label <label>
    --limit <n>
    --search <query>
    --json

bb pr view <id>       # View pull request details
    --web
    --comments
    --json

bb pr create          # Create pull request
    --title <title>
    --body <body>
    --body-file <file>
    --base <branch>
    --head <branch>
    --draft
    --reviewer <user>
    --assignee <user>
    --label <label>
    --web
    --fill            # Auto-fill from commits

bb pr checkout <id>   # Check out PR branch locally

bb pr diff <id>       # View PR diff
    --stat
    --patch

bb pr merge <id>      # Merge pull request
    --merge           # Merge commit
    --squash          # Squash and merge
    --rebase          # Rebase and merge
    --delete-branch
    --auto            # Enable auto-merge
    --message <msg>

bb pr close <id>      # Decline/close PR
bb pr reopen <id>     # Reopen declined PR

bb pr approve <id>    # Approve pull request
bb pr request-changes <id>
    --body <msg>
bb pr unapprove <id>  # Remove approval

bb pr review <id>     # Submit review
    --approve
    --request-changes
    --comment
    --body <msg>

bb pr comment <id>    # Add comment
    --body <msg>
    --body-file <file>
    --edit-last
    --reply-to <id>

bb pr comments <id>   # List comments
    --json

bb pr edit <id>       # Edit PR details
    --title <title>
    --body <body>
    --base <branch>
    --add-reviewer <user>

bb pr ready <id>      # Mark as ready for review

bb pr checks <id>     # View build/policy status
    --watch
    --json
```

---

## `bb issue` - Issue Tracker Operations

Manage issues in Bitbucket Cloud repositories.

### Commands

```bash
bb issue list         # List issues
    --state <state>   # open/new/resolved/closed
    --assignee <user>
    --author <user>
    --label <label>
    --milestone <ms>
    --priority <p>
    --kind <kind>     # bug/enhancement/proposal/task
    --search <query>
    --limit <n>
    --json

bb issue view <id>    # View issue details
    --web
    --comments
    --json

bb issue create       # Create new issue
    --title <title>
    --body <body>
    --body-file <file>
    --assignee <user>
    --label <label>
    --milestone <ms>
    --priority <p>
    --kind <kind>
    --web

bb issue edit <id>    # Edit issue
    --title <title>
    --body <body>
    --assignee <user>
    --add-label <label>
    --remove-label <label>

bb issue close <id>   # Close issue
    --reason <reason>

bb issue reopen <id>  # Reopen issue

bb issue comment <id> # Add comment
    --body <msg>
    --body-file <file>

bb issue delete <id>  # Delete issue
    --confirm

# Jira Integration
bb issue jira link <id> <jira-key>      # Link to Jira
bb issue jira unlink <id>               # Remove Jira link
bb issue jira transition <key> <status> # Transition Jira
bb issue jira comment <key> --body <msg>
```

---

## `bb pipeline` - CI/CD Pipeline Operations

Manage Bitbucket Pipelines (Cloud only).

### Commands

```bash
bb pipeline list      # List pipeline runs
    --branch <branch>
    --status <status>
    --trigger <trigger>  # push/manual/schedule
    --limit <n>
    --json

bb pipeline view <id> # View pipeline details
    --web
    --json

bb pipeline run       # Trigger pipeline
    --branch <branch>
    --custom <name>   # Custom pipeline name
    --variable <k=v>
    --watch

bb pipeline stop <id> # Stop running pipeline

bb pipeline rerun <id>
    --failed-only     # Rerun failed steps only

bb pipeline logs <id> # View pipeline logs
    --step <step>
    --follow          # Stream logs
    --failed

bb pipeline watch <id>
    --exit-status
    --interval <secs>

bb pipeline enable    # Enable pipelines for repo
bb pipeline disable   # Disable pipelines

bb pipeline config    # View/edit pipeline config
    --validate
    --edit

# Cache management
bb pipeline cache list
bb pipeline cache delete <name>
bb pipeline cache clear

# Schedule management
bb pipeline schedule list
bb pipeline schedule create --cron <expr> --branch <br>
bb pipeline schedule delete <id>
bb pipeline schedule pause <id>
bb pipeline schedule resume <id>

# Runner management
bb pipeline runner list
bb pipeline runner register
bb pipeline runner remove <id>
```

---

## `bb workspace` - Workspace Management (Cloud)

Manage Bitbucket Cloud workspaces.

### Commands

```bash
bb workspace list     # List workspaces
    --json

bb workspace view <ws>
    --web
    --json

bb workspace members <ws>
    --role <role>
    --json

bb workspace projects <ws>

bb workspace switch <ws>  # Set default workspace
```

---

## `bb project` - Project Management

Manage projects (primary for Server/DC, also available on Cloud).

### Commands

```bash
bb project list       # List projects
    --workspace <ws>  # Workspace (Cloud)
    --json

bb project view <proj>
    --web
    --json

bb project create     # Create project (Server/DC)
    --key <key>
    --name <name>
    --description <desc>
    --avatar <file>

bb project edit <proj>
    --name <name>
    --description <desc>

bb project delete <proj>
    --confirm

bb project repos <proj>

bb project members list <proj>
bb project members add <proj> <user> --permission <perm>
bb project members remove <proj> <user>

bb project permissions <proj>
```

---

## `bb browse` - Browser Navigation

Open resources in the web browser.

### Commands

```bash
bb browse [path]      # Open repo in browser
    --branch <br>
    --commit <sha>
    --settings
    --issues
    --prs
    --pipelines
    --wiki
    --projects
```

---

## `bb api` - Direct API Access

Make direct API calls for advanced use cases.

### Commands

```bash
bb api <endpoint>     # Make API request
    -X, --method <method>
    -H, --header <header>
    -F, --field <key=value>
    --raw-field <key=value>
    -f, --input <file>
    --jq <query>
    --paginate
    --hostname <host>
    --cache <duration>
    -i, --include
    --silent
```

### Examples

```bash
# List repositories in a workspace
bb api /2.0/repositories/myworkspace

# Get pull request details
bb api /2.0/repositories/myworkspace/myrepo/pullrequests/123

# Create a repository (with JSON body)
bb api /2.0/repositories/myworkspace/newrepo -X POST \
  -F name=newrepo -F is_private=true
```

---

## `bb config` - CLI Configuration

Manage CLI configuration.

### Commands

```bash
bb config get <key>
bb config set <key> <value>
bb config unset <key>
bb config list
    --host <host>
bb config edit
    --host <host>
```

### Configuration Keys

| Key | Description | Default |
|-----|-------------|---------|
| `editor` | Preferred text editor | `$EDITOR` |
| `pager` | Pager for long output | `less` |
| `browser` | Web browser | System default |
| `git_protocol` | Git protocol (https/ssh) | `https` |
| `prompt` | Enable interactive prompts | `enabled` |
| `default_workspace` | Default Cloud workspace | - |
| `default_host` | Default Server host | `bitbucket.org` |

---

## `bb alias` - Command Aliases

Create shortcuts for common commands.

### Commands

```bash
bb alias set <alias> <expansion>
    --shell           # Shell command alias
bb alias delete <alias>
bb alias list
bb alias import <file>
```

### Examples

```bash
# Create PR checkout shortcut
bb alias set co "pr checkout"

# Create complex alias
bb alias set prs "pr list --state open --reviewer @me"

# Shell alias
bb alias set --shell pbcopy "bb pr view --json url | jq -r .url | pbcopy"
```

---

## `bb extension` - CLI Extensions

Manage CLI extensions.

### Commands

```bash
bb extension list
bb extension install <repo>
    --pin <version>
bb extension upgrade <ext>
    --all
bb extension remove <ext>
bb extension create <name>
    --precompiled <lang>
bb extension browse
bb extension exec <ext>
```

---

## `bb webhook` - Webhook Management

Manage repository webhooks.

### Commands

```bash
bb webhook list
    --json

bb webhook create
    --url <url>
    --events <events>
    --secret <secret>
    --active/--inactive

bb webhook edit <id>
    --url <url>
    --add-event <event>
    --remove-event <event>

bb webhook delete <id>

bb webhook deliveries <id>
    --json

bb webhook test <id>
```

---

## `bb deploy` - Deployment Operations (Cloud)

Manage deployments and environments.

### Commands

```bash
bb deploy list        # List deployments
    --environment <env>
    --status <status>
    --json

bb deploy view <id>   # View deployment details

bb deploy create      # Create deployment
    --environment <env>
    --pipeline <id>
    --step <step>

bb deploy promote <id>
    --environment <env>

# Environment management
bb deploy environment list
bb deploy environment view <name>
bb deploy environment create <name> --type <t>
bb deploy environment edit <name>
bb deploy environment delete <name>
```

---

## `bb artifact` - Build Artifact Operations

Manage pipeline artifacts (Cloud).

### Commands

```bash
bb artifact list      # List artifacts
    --pipeline <id>
    --json

bb artifact download <name>
    --pipeline <id>
    --step <step>
    --dir <output-dir>

bb artifact delete <id>
```

---

## `bb secret` - Secrets & Variables Management

Manage repository and deployment secrets.

### Commands

```bash
bb secret list
    --workspace <ws>
    --repo
    --environment <env>
    --json

bb secret set <name>
    --body <value>
    --body-file <file>
    --workspace <ws>
    --environment <env>
    --secured

bb secret delete <name>
    --workspace/--repo/--environment

bb secret sync
    --file <path>    # Sync from .env file
```

---

## `bb ssh-key` - SSH Key Management

Manage SSH keys for authentication.

### Commands

```bash
bb ssh-key list
    --json

bb ssh-key add
    --title <title>
    --key <key>
    --key-file <file>

bb ssh-key delete <id>
bb ssh-key test
```

---

## `bb completion` - Shell Completion

Generate shell completion scripts.

### Commands

```bash
bb completion bash
bb completion zsh
bb completion fish
bb completion powershell
```

### Installation

```bash
# Bash
bb completion bash > /etc/bash_completion.d/bb

# Zsh
bb completion zsh > ~/.zsh/completion/_bb

# Fish
bb completion fish > ~/.config/fish/completions/bb.fish
```

---

## Platform Capability Matrix

| Feature | Cloud | Server/DC | Notes |
|---------|-------|-----------|-------|
| **Authentication** |
| OAuth 2.0 | Yes | Limited | Cloud primary |
| App Passwords | Deprecated | N/A | |
| Personal Access Tokens | N/A | Yes | Server/DC primary |
| **Repositories** |
| CRUD Operations | Yes | Yes | Full parity |
| Forking | Yes | Yes | |
| Branch Restrictions | Yes | Yes | Different APIs |
| Default Reviewers | Yes | Yes | |
| **Organization** |
| Workspaces | Yes | N/A | Cloud-only |
| Projects | Limited | Yes | Server/DC primary |
| **Pull Requests** |
| Full Lifecycle | Yes | Yes | Full parity |
| Code Review | Yes | Yes | |
| Merge Strategies | Yes | Yes | |
| Draft PRs | Yes | Yes | |
| Auto-merge | Yes | Limited | |
| **CI/CD** |
| Pipelines | Yes | N/A | Cloud-only |
| Artifacts | Yes | N/A | |
| Deployments | Yes | N/A | |
| Build Status | Yes | Yes | Different APIs |
| **Issues** |
| Native Issues | Yes | N/A | Cloud-only |
| Jira Integration | Yes | Yes | |
| **Admin** |
| Webhooks | Yes | Yes | |
| Secrets/Variables | Yes | Yes | |
| Permissions | Yes | Yes | |

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `BB_TOKEN` | Authentication token (overrides keyring) |
| `BB_HOST` | Default host |
| `BB_WORKSPACE` | Default workspace (Cloud) |
| `BB_PROJECT` | Default project (Server/DC) |
| `BB_REPO` | Default repository |
| `BB_EDITOR` | Preferred editor |
| `BB_PAGER` | Pager for output |
| `BB_BROWSER` | Browser for web links |
| `BB_NO_PROMPT` | Disable interactive prompts |
| `BB_FORCE_TTY` | Force TTY behavior |
| `BB_DEBUG` | Enable debug logging |
| `NO_COLOR` | Disable color output (standard) |

---

## Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error |
| `2` | Invalid usage |
| `4` | Authentication required/failed |
| `8` | Resource not found |
| `16` | Operation cancelled |
| `32` | API rate limit exceeded |

---

## Configuration Files

| Platform | Location |
|----------|----------|
| Linux | `~/.config/bb/config.toml` |
| macOS | `~/Library/Application Support/bb/config.toml` |
| Windows | `%APPDATA%\bb\config.toml` |

### Example Configuration

```toml
[core]
editor = "vim"
pager = "less"
browser = "open"
git_protocol = "ssh"
prompt = "enabled"

[hosts.bitbucket.org]
user = "myusername"
default_workspace = "myworkspace"

[hosts."bitbucket.company.com"]
user = "jdoe"
default_project = "PROJ"

[aliases]
co = "pr checkout"
prs = "pr list --state open"
```

---

## API Reference

### Cloud API (v2.0)

Base URL: `https://api.bitbucket.org/2.0`

Common endpoints:
- `/repositories/{workspace}/{repo_slug}`
- `/repositories/{workspace}/{repo_slug}/pullrequests`
- `/repositories/{workspace}/{repo_slug}/pipelines`
- `/workspaces/{workspace}`

### Server/DC API (v1.0)

Base URL: `https://{host}/rest/api/1.0`

Common endpoints:
- `/projects/{projectKey}/repos/{repositorySlug}`
- `/projects/{projectKey}/repos/{repositorySlug}/pull-requests`
- `/projects`

---

## License

MIT License
