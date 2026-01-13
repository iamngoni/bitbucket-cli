//
//  bitbucket-cli
//  context/mod.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # Repository Context Module
//!
//! This module provides repository context resolution for the Bitbucket CLI.
//! It automatically detects repository information from git remotes and CLI
//! arguments, supporting both Bitbucket Cloud and Bitbucket Server/Data Center.
//!
//! ## Overview
//!
//! The context module is responsible for:
//! - Detecting the current git repository and extracting remote URLs
//! - Parsing various git remote URL formats (SSH, HTTPS, SCM)
//! - Determining whether the repository is hosted on Bitbucket Cloud or Server
//! - Providing repository metadata for API calls and web URLs
//!
//! ## Architecture
//!
//! The module consists of three main components:
//! - [`GitContext`]: Low-level git repository operations
//! - [`ContextResolver`]: URL parsing and context resolution logic
//! - [`RepoContext`]: The resolved repository information
//!
//! ## Example
//!
//! ```rust,no_run
//! use bitbucket_cli::context::{ContextResolver, RepoContext};
//! use bitbucket_cli::config::Config;
//! use bitbucket_cli::cli::GlobalOptions;
//!
//! let config = Config::default();
//! let resolver = ContextResolver::new(config);
//! let options = GlobalOptions::default();
//!
//! match resolver.resolve(&options) {
//!     Ok(ctx) => {
//!         println!("Repository: {}", ctx.full_name());
//!         println!("Web URL: {}", ctx.web_url());
//!     }
//!     Err(e) => eprintln!("Failed to resolve context: {}", e),
//! }
//! ```

mod git;
mod resolver;

pub use git::*;
pub use resolver::*;

/// Repository context containing all resolved information about a Bitbucket repository.
///
/// This struct holds the essential metadata needed to interact with a Bitbucket
/// repository, including host information, ownership details, and the repository
/// identifier. It supports both Bitbucket Cloud and Bitbucket Server/Data Center.
///
/// # Fields
///
/// * `host` - The hostname of the Bitbucket instance (e.g., "bitbucket.org" for Cloud
///   or "bitbucket.company.com" for Server)
/// * `host_type` - Indicates whether this is a Cloud or Server instance
/// * `owner` - The workspace name (Cloud) or project key (Server)
/// * `repo_slug` - The URL-friendly repository identifier
/// * `default_branch` - The repository's default branch, if known
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::context::{RepoContext, HostType};
///
/// let ctx = RepoContext {
///     host: "bitbucket.org".to_string(),
///     host_type: HostType::Cloud,
///     owner: "myworkspace".to_string(),
///     repo_slug: "my-repo".to_string(),
///     default_branch: Some("main".to_string()),
/// };
///
/// assert_eq!(ctx.full_name(), "myworkspace/my-repo");
/// ```
///
/// # Notes
///
/// - The `repo_slug` is typically a URL-safe version of the repository name
/// - For Cloud, `owner` represents the workspace; for Server, it's the project key
/// - The `default_branch` may be `None` if not yet fetched from the API
#[derive(Debug, Clone)]
pub struct RepoContext {
    /// Host (e.g., "bitbucket.org" or "bitbucket.company.com")
    pub host: String,
    /// Host type (Cloud or Server)
    pub host_type: HostType,
    /// Owner/namespace (workspace for Cloud, project for Server)
    pub owner: String,
    /// Repository slug
    pub repo_slug: String,
    /// Default branch
    pub default_branch: Option<String>,
}

/// Represents the type of Bitbucket host.
///
/// Bitbucket exists in two main deployment models:
/// - **Cloud**: The SaaS offering at bitbucket.org
/// - **Server/Data Center**: Self-hosted enterprise installations
///
/// The host type affects URL construction, API endpoints, and authentication
/// mechanisms used by the CLI.
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::context::HostType;
///
/// let cloud = HostType::Cloud;
/// let server = HostType::Server;
///
/// assert_ne!(cloud, server);
/// ```
///
/// # Notes
///
/// - Bitbucket Server and Data Center are treated identically by this enum
/// - The host type is typically auto-detected from the remote URL
/// - Cloud uses workspace-based organization; Server uses project-based
#[derive(Debug, Clone, PartialEq)]
pub enum HostType {
    /// Bitbucket Cloud (bitbucket.org)
    ///
    /// The SaaS version of Bitbucket hosted at bitbucket.org. Uses workspaces
    /// for organization and the 2.0 REST API.
    Cloud,
    /// Bitbucket Server/Data Center
    ///
    /// Self-hosted enterprise version of Bitbucket. Uses projects for
    /// organization and the 1.0 REST API.
    Server,
}

impl RepoContext {
    /// Returns the full repository path in the format "owner/repo".
    ///
    /// This method constructs the canonical repository identifier by combining
    /// the owner (workspace or project) with the repository slug.
    ///
    /// # Returns
    ///
    /// A `String` containing the full repository path.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::context::{RepoContext, HostType};
    ///
    /// let ctx = RepoContext {
    ///     host: "bitbucket.org".to_string(),
    ///     host_type: HostType::Cloud,
    ///     owner: "atlassian".to_string(),
    ///     repo_slug: "python-bitbucket".to_string(),
    ///     default_branch: None,
    /// };
    ///
    /// assert_eq!(ctx.full_name(), "atlassian/python-bitbucket");
    /// ```
    ///
    /// # Notes
    ///
    /// - The format is consistent across Cloud and Server
    /// - This value is suitable for display and logging purposes
    pub fn full_name(&self) -> String {
        format!("{}/{}", self.owner, self.repo_slug)
    }

    /// Returns the web URL for viewing the repository in a browser.
    ///
    /// Constructs the appropriate URL based on whether this is a Cloud or
    /// Server repository. The URL points to the repository's main page.
    ///
    /// # Returns
    ///
    /// A `String` containing the full web URL.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::context::{RepoContext, HostType};
    ///
    /// // Cloud repository
    /// let cloud_ctx = RepoContext {
    ///     host: "bitbucket.org".to_string(),
    ///     host_type: HostType::Cloud,
    ///     owner: "myworkspace".to_string(),
    ///     repo_slug: "my-repo".to_string(),
    ///     default_branch: None,
    /// };
    /// assert_eq!(
    ///     cloud_ctx.web_url(),
    ///     "https://bitbucket.org/myworkspace/my-repo"
    /// );
    ///
    /// // Server repository
    /// let server_ctx = RepoContext {
    ///     host: "bitbucket.company.com".to_string(),
    ///     host_type: HostType::Server,
    ///     owner: "PROJ".to_string(),
    ///     repo_slug: "my-repo".to_string(),
    ///     default_branch: None,
    /// };
    /// assert_eq!(
    ///     server_ctx.web_url(),
    ///     "https://bitbucket.company.com/projects/PROJ/repos/my-repo"
    /// );
    /// ```
    ///
    /// # Notes
    ///
    /// - Cloud URLs follow the pattern: `https://bitbucket.org/{workspace}/{repo}`
    /// - Server URLs follow the pattern: `https://{host}/projects/{project}/repos/{repo}`
    /// - Always uses HTTPS for security
    pub fn web_url(&self) -> String {
        match self.host_type {
            HostType::Cloud => {
                format!("https://bitbucket.org/{}/{}", self.owner, self.repo_slug)
            }
            HostType::Server => {
                format!(
                    "https://{}/projects/{}/repos/{}",
                    self.host, self.owner, self.repo_slug
                )
            }
        }
    }

    /// Returns the base API URL for making REST API calls to this repository.
    ///
    /// Constructs the appropriate API endpoint based on the host type. Cloud
    /// uses the 2.0 API while Server uses the 1.0 API.
    ///
    /// # Returns
    ///
    /// A `String` containing the base API URL for repository operations.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::context::{RepoContext, HostType};
    ///
    /// // Cloud repository
    /// let cloud_ctx = RepoContext {
    ///     host: "bitbucket.org".to_string(),
    ///     host_type: HostType::Cloud,
    ///     owner: "myworkspace".to_string(),
    ///     repo_slug: "my-repo".to_string(),
    ///     default_branch: None,
    /// };
    /// assert_eq!(
    ///     cloud_ctx.api_url(),
    ///     "https://api.bitbucket.org/2.0/repositories/myworkspace/my-repo"
    /// );
    ///
    /// // Server repository
    /// let server_ctx = RepoContext {
    ///     host: "bitbucket.company.com".to_string(),
    ///     host_type: HostType::Server,
    ///     owner: "PROJ".to_string(),
    ///     repo_slug: "my-repo".to_string(),
    ///     default_branch: None,
    /// };
    /// assert_eq!(
    ///     server_ctx.api_url(),
    ///     "https://bitbucket.company.com/rest/api/1.0/projects/PROJ/repos/my-repo"
    /// );
    /// ```
    ///
    /// # Notes
    ///
    /// - Cloud uses `api.bitbucket.org` with the 2.0 API version
    /// - Server uses the same host with `/rest/api/1.0/` path prefix
    /// - Additional path segments should be appended for specific API endpoints
    pub fn api_url(&self) -> String {
        match self.host_type {
            HostType::Cloud => {
                format!(
                    "https://api.bitbucket.org/2.0/repositories/{}/{}",
                    self.owner, self.repo_slug
                )
            }
            HostType::Server => {
                format!(
                    "https://{}/rest/api/1.0/projects/{}/repos/{}",
                    self.host, self.owner, self.repo_slug
                )
            }
        }
    }
}
