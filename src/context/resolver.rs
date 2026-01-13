//
//  bitbucket-cli
//  context/resolver.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # Repository Context Resolver
//!
//! This module handles the resolution of repository context from various sources,
//! including git remote URLs and CLI arguments. It supports multiple URL formats
//! for both Bitbucket Cloud and Bitbucket Server/Data Center.
//!
//! ## Overview
//!
//! The [`ContextResolver`] is responsible for:
//! - Parsing git remote URLs in SSH and HTTPS formats
//! - Handling Bitbucket Server-specific URL patterns (SCM paths)
//! - Resolving repository information from CLI flags
//! - Determining host type (Cloud vs Server)
//!
//! ## Supported URL Formats
//!
//! ### Bitbucket Cloud
//! - SSH: `git@bitbucket.org:workspace/repo.git`
//! - HTTPS: `https://bitbucket.org/workspace/repo.git`
//!
//! ### Bitbucket Server/Data Center
//! - SSH: `ssh://git@server:7999/PROJECT/repo.git`
//! - HTTPS SCM: `https://server/scm/PROJECT/repo.git`
//! - HTTPS: `https://server/PROJECT/repo.git`
//!
//! ## Resolution Priority
//!
//! When resolving context, the following priority order is used:
//! 1. CLI flags (`--repo` option)
//! 2. Environment variables (if implemented)
//! 3. Git remote URL (origin)
//! 4. Configuration file defaults
//!
//! ## Example
//!
//! ```rust,no_run
//! use bitbucket_cli::context::ContextResolver;
//! use bitbucket_cli::config::Config;
//! use bitbucket_cli::cli::GlobalOptions;
//!
//! let config = Config::default();
//! let resolver = ContextResolver::new(config);
//!
//! // Parse a remote URL directly
//! let ctx = resolver.parse_remote_url("git@bitbucket.org:myworkspace/my-repo.git")?;
//! println!("Repository: {}/{}", ctx.owner, ctx.repo_slug);
//! # Ok::<(), anyhow::Error>(())
//! ```

use anyhow::Result;
use regex::Regex;
use once_cell::sync::Lazy;

use super::{GitContext, HostType, RepoContext};
use crate::cli::GlobalOptions;
use crate::config::{is_cloud_host, Config};

/// Regular expression pattern for parsing standard SSH remote URLs.
///
/// Matches URLs in the format: `git@host:owner/repo.git`
///
/// # Capture Groups
/// 1. Host (e.g., "bitbucket.org")
/// 2. Owner/workspace (e.g., "myworkspace")
/// 3. Repository slug (e.g., "my-repo")
///
/// # Examples of Matched URLs
/// - `git@bitbucket.org:workspace/repo.git`
/// - `git@bitbucket.org:workspace/repo` (without .git suffix)
/// - `git@github.com:owner/repo.git`
static SSH_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^git@([^:]+):(.+)/(.+?)(?:\.git)?$").unwrap()
});

/// Regular expression pattern for parsing HTTPS remote URLs.
///
/// Matches URLs in the format: `https://host/owner/repo.git`
///
/// # Capture Groups
/// 1. Host (e.g., "bitbucket.org")
/// 2. Owner/workspace (e.g., "myworkspace")
/// 3. Repository slug (e.g., "my-repo")
///
/// # Examples of Matched URLs
/// - `https://bitbucket.org/workspace/repo.git`
/// - `http://bitbucket.org/workspace/repo` (also matches HTTP)
static HTTPS_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^https?://([^/]+)/(.+)/(.+?)(?:\.git)?$").unwrap()
});

/// Regular expression pattern for Bitbucket Server SSH URLs.
///
/// Matches URLs in the format: `ssh://git@host:port/project/repo.git`
///
/// # Capture Groups
/// 1. Host (e.g., "bitbucket.company.com")
/// 2. Project key (e.g., "PROJ")
/// 3. Repository slug (e.g., "my-repo")
///
/// # Examples of Matched URLs
/// - `ssh://git@bitbucket.company.com:7999/PROJ/repo.git`
/// - `ssh://git@server/PROJECT/repo.git` (without port)
///
/// # Notes
/// - The port number is optional and not captured
/// - This pattern is specific to Bitbucket Server/Data Center
static SERVER_SSH_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^ssh://git@([^:/]+)(?::\d+)?/(.+)/(.+?)(?:\.git)?$").unwrap()
});

/// Regular expression pattern for Bitbucket Server SCM URLs.
///
/// Matches URLs in the format: `https://host/scm/project/repo.git`
///
/// # Capture Groups
/// 1. Host (e.g., "bitbucket.company.com")
/// 2. Project key (e.g., "PROJ")
/// 3. Repository slug (e.g., "my-repo")
///
/// # Examples of Matched URLs
/// - `https://bitbucket.company.com/scm/PROJ/repo.git`
/// - `http://server/scm/PROJECT/repo` (also matches HTTP)
///
/// # Notes
/// - The `/scm/` path component is specific to Bitbucket Server
/// - This is the most common HTTPS format for Server installations
static SERVER_SCM_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"^https?://([^/]+)/scm/(.+)/(.+?)(?:\.git)?$").unwrap()
});

/// Resolves repository context from various sources.
///
/// The `ContextResolver` is the primary mechanism for determining which
/// Bitbucket repository the CLI should operate on. It combines information
/// from CLI arguments, git remotes, and configuration to produce a
/// [`RepoContext`].
///
/// # Resolution Strategy
///
/// The resolver follows this priority order:
/// 1. **CLI flags**: If `--repo` is provided, use it directly
/// 2. **Git remote**: Parse the "origin" remote URL
/// 3. **Error**: If neither is available, return an error
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::context::ContextResolver;
/// use bitbucket_cli::config::Config;
/// use bitbucket_cli::cli::GlobalOptions;
///
/// let config = Config::default();
/// let resolver = ContextResolver::new(config);
///
/// // Resolve from current directory's git remote
/// let options = GlobalOptions::default();
/// let ctx = resolver.resolve(&options)?;
///
/// // Or parse a URL directly
/// let ctx = resolver.parse_remote_url("git@bitbucket.org:workspace/repo.git")?;
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Notes
///
/// - The resolver holds a `Config` reference for host type detection
/// - URL parsing is stateless; the config is only used for host classification
/// - Unknown URL formats result in an error
#[allow(dead_code)]
pub struct ContextResolver {
    /// Configuration for host type detection and defaults
    config: Config,
}

impl ContextResolver {
    /// Creates a new `ContextResolver` with the given configuration.
    ///
    /// The configuration is used to determine host types when the URL alone
    /// is ambiguous (e.g., differentiating between Cloud and Server based
    /// on hostname patterns).
    ///
    /// # Arguments
    ///
    /// * `config` - The CLI configuration containing host settings
    ///
    /// # Returns
    ///
    /// A new `ContextResolver` instance.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::context::ContextResolver;
    /// use bitbucket_cli::config::Config;
    ///
    /// let config = Config::default();
    /// let resolver = ContextResolver::new(config);
    /// ```
    ///
    /// # Notes
    ///
    /// - The resolver takes ownership of the config
    /// - A default config uses "bitbucket.org" as the Cloud host indicator
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Resolves repository context from CLI options and git remote.
    ///
    /// This is the main entry point for context resolution. It attempts to
    /// determine the target repository using the following priority:
    ///
    /// 1. If `--repo` option is specified, parse it directly
    /// 2. If in a git repository, parse the origin remote URL
    /// 3. Return an error with usage instructions
    ///
    /// # Arguments
    ///
    /// * `options` - The global CLI options containing repository overrides
    ///
    /// # Returns
    ///
    /// - `Ok(RepoContext)` if the repository can be determined
    /// - `Err` with a helpful message if resolution fails
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::context::ContextResolver;
    /// use bitbucket_cli::config::Config;
    /// use bitbucket_cli::cli::GlobalOptions;
    ///
    /// let resolver = ContextResolver::new(Config::default());
    ///
    /// // With explicit repo flag
    /// let mut options = GlobalOptions::default();
    /// options.repo = Some("myworkspace/my-repo".to_string());
    /// let ctx = resolver.resolve(&options)?;
    ///
    /// // Or relying on git remote detection
    /// let ctx = resolver.resolve(&GlobalOptions::default())?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Notes
    ///
    /// - The `--repo` flag takes precedence over git remote detection
    /// - The `--host` flag can override the default host for `--repo`
    /// - Returns a clear error message guiding users on how to specify a repo
    pub fn resolve(&self, options: &GlobalOptions) -> Result<RepoContext> {
        // Priority: CLI flags > environment > git remote > config

        // If repo is specified directly
        if let Some(repo) = &options.repo {
            return self.parse_repo_arg(repo, options);
        }

        // Try to get from git remote
        if let Ok(git) = GitContext::open() {
            if let Some(url) = git.origin_url()? {
                return self.parse_remote_url(&url);
            }
        }

        anyhow::bail!("Could not determine repository. Use --repo to specify, or run from within a git repository.")
    }

    /// Parses a repository argument in "owner/repo" format.
    ///
    /// This method handles the `--repo` CLI flag, which accepts repository
    /// identifiers in the format `WORKSPACE/REPO` (Cloud) or `PROJECT/REPO`
    /// (Server).
    ///
    /// # Arguments
    ///
    /// * `repo` - The repository string in "owner/repo" format
    /// * `options` - Global options containing the optional host override
    ///
    /// # Returns
    ///
    /// - `Ok(RepoContext)` if the format is valid
    /// - `Err` if the format is invalid (not exactly two parts)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::context::ContextResolver;
    /// use bitbucket_cli::config::Config;
    /// use bitbucket_cli::cli::GlobalOptions;
    ///
    /// let resolver = ContextResolver::new(Config::default());
    /// let options = GlobalOptions::default();
    ///
    /// // This would be called internally by resolve()
    /// // let ctx = resolver.parse_repo_arg("myworkspace/my-repo", &options)?;
    /// ```
    ///
    /// # Notes
    ///
    /// - Defaults to Bitbucket Cloud if no host is specified
    /// - The `--host` flag can override the host
    /// - Host type is determined by the [`is_cloud_host`] function
    fn parse_repo_arg(&self, repo: &str, options: &GlobalOptions) -> Result<RepoContext> {
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid repository format. Expected WORKSPACE/REPO or PROJECT/REPO");
        }

        let owner = parts[0].to_string();
        let repo_slug = parts[1].to_string();

        // Determine host type from options or default to Cloud
        let host = options.host.clone().unwrap_or_else(|| "bitbucket.org".to_string());
        let host_type = if is_cloud_host(&host) {
            HostType::Cloud
        } else {
            HostType::Server
        };

        Ok(RepoContext {
            host,
            host_type,
            owner,
            repo_slug,
            default_branch: None,
        })
    }

    /// Parses a git remote URL to extract repository context.
    ///
    /// This method supports multiple URL formats used by Bitbucket Cloud and
    /// Server. It tries each pattern in sequence until one matches.
    ///
    /// # Supported Formats
    ///
    /// | Format | Example | Host Type |
    /// |--------|---------|-----------|
    /// | SSH | `git@bitbucket.org:workspace/repo.git` | Auto-detect |
    /// | HTTPS | `https://bitbucket.org/workspace/repo.git` | Auto-detect |
    /// | Server SSH | `ssh://git@server:7999/PROJECT/repo.git` | Server |
    /// | Server SCM | `https://server/scm/PROJECT/repo.git` | Server |
    ///
    /// # Arguments
    ///
    /// * `url` - The git remote URL to parse
    ///
    /// # Returns
    ///
    /// - `Ok(RepoContext)` if the URL matches a known pattern
    /// - `Err` if the URL format is not recognized
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::context::ContextResolver;
    /// use bitbucket_cli::config::Config;
    ///
    /// let resolver = ContextResolver::new(Config::default());
    ///
    /// // Parse various URL formats
    /// let ctx = resolver.parse_remote_url("git@bitbucket.org:workspace/repo.git")?;
    /// assert_eq!(ctx.owner, "workspace");
    ///
    /// let ctx = resolver.parse_remote_url("https://bitbucket.org/workspace/repo.git")?;
    /// assert_eq!(ctx.repo_slug, "repo");
    ///
    /// let ctx = resolver.parse_remote_url("https://server/scm/PROJ/repo.git")?;
    /// assert_eq!(ctx.owner, "PROJ");
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Notes
    ///
    /// - The `.git` suffix is optional in all formats
    /// - Server SCM URLs always indicate Bitbucket Server
    /// - Host type for SSH/HTTPS is determined by hostname
    /// - Unknown formats produce a helpful error message
    pub fn parse_remote_url(&self, url: &str) -> Result<RepoContext> {
        // Try Cloud SSH pattern (git@bitbucket.org:workspace/repo.git)
        if let Some(caps) = SSH_PATTERN.captures(url) {
            let host = caps.get(1).unwrap().as_str().to_string();
            let owner = caps.get(2).unwrap().as_str().to_string();
            let repo_slug = caps.get(3).unwrap().as_str().to_string();
            let host_type = if is_cloud_host(&host) {
                HostType::Cloud
            } else {
                HostType::Server
            };
            return Ok(RepoContext {
                host,
                host_type,
                owner,
                repo_slug,
                default_branch: None,
            });
        }

        // Try Server SCM pattern (https://server/scm/PROJECT/repo.git)
        if let Some(caps) = SERVER_SCM_PATTERN.captures(url) {
            let host = caps.get(1).unwrap().as_str().to_string();
            let owner = caps.get(2).unwrap().as_str().to_string();
            let repo_slug = caps.get(3).unwrap().as_str().to_string();
            return Ok(RepoContext {
                host,
                host_type: HostType::Server,
                owner,
                repo_slug,
                default_branch: None,
            });
        }

        // Try Server SSH pattern (ssh://git@server:7999/PROJECT/repo.git)
        if let Some(caps) = SERVER_SSH_PATTERN.captures(url) {
            let host = caps.get(1).unwrap().as_str().to_string();
            let owner = caps.get(2).unwrap().as_str().to_string();
            let repo_slug = caps.get(3).unwrap().as_str().to_string();
            return Ok(RepoContext {
                host,
                host_type: HostType::Server,
                owner,
                repo_slug,
                default_branch: None,
            });
        }

        // Try HTTPS pattern (https://bitbucket.org/workspace/repo.git)
        if let Some(caps) = HTTPS_PATTERN.captures(url) {
            let host = caps.get(1).unwrap().as_str().to_string();
            let owner = caps.get(2).unwrap().as_str().to_string();
            let repo_slug = caps.get(3).unwrap().as_str().to_string();
            let host_type = if is_cloud_host(&host) {
                HostType::Cloud
            } else {
                HostType::Server
            };
            return Ok(RepoContext {
                host,
                host_type,
                owner,
                repo_slug,
                default_branch: None,
            });
        }

        anyhow::bail!("Could not parse remote URL: {}", url)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cloud_ssh() {
        let resolver = ContextResolver::new(Config::default());
        let ctx = resolver.parse_remote_url("git@bitbucket.org:workspace/repo.git").unwrap();
        assert_eq!(ctx.host, "bitbucket.org");
        assert_eq!(ctx.owner, "workspace");
        assert_eq!(ctx.repo_slug, "repo");
        assert_eq!(ctx.host_type, HostType::Cloud);
    }

    #[test]
    fn test_parse_cloud_https() {
        let resolver = ContextResolver::new(Config::default());
        let ctx = resolver.parse_remote_url("https://bitbucket.org/workspace/repo.git").unwrap();
        assert_eq!(ctx.host, "bitbucket.org");
        assert_eq!(ctx.owner, "workspace");
        assert_eq!(ctx.repo_slug, "repo");
        assert_eq!(ctx.host_type, HostType::Cloud);
    }

    #[test]
    fn test_parse_server_scm() {
        let resolver = ContextResolver::new(Config::default());
        let ctx = resolver.parse_remote_url("https://bitbucket.company.com/scm/PROJ/repo.git").unwrap();
        assert_eq!(ctx.host, "bitbucket.company.com");
        assert_eq!(ctx.owner, "PROJ");
        assert_eq!(ctx.repo_slug, "repo");
        assert_eq!(ctx.host_type, HostType::Server);
    }
}
