//
//  bitbucket-cli
//  api/server/repositories.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # Bitbucket Server/DC Repository API
//!
//! This module provides types and structures for working with repositories
//! in Bitbucket Server/Data Center. Repositories are the core unit of code
//! storage and are always associated with a project.
//!
//! ## Repository Structure
//!
//! In Bitbucket Server/DC, repositories:
//! - Belong to exactly one project (identified by project key)
//! - Have a unique slug within their project
//! - Support Git as the primary SCM
//! - Can be public or private
//! - May or may not allow forking
//!
//! ## API Endpoint
//!
//! Repository operations use the endpoint:
//! ```text
//! GET/POST /rest/api/1.0/projects/{projectKey}/repos
//! GET/PUT/DELETE /rest/api/1.0/projects/{projectKey}/repos/{repoSlug}
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use bitbucket_cli::api::server::repositories::{Repository, CreateRepositoryRequest};
//!
//! // Create a new repository request
//! let request = CreateRepositoryRequest {
//!     name: "my-new-repo".to_string(),
//!     description: Some("A sample repository".to_string()),
//!     ..Default::default()
//! };
//!
//! // Repository response from API
//! let repo: Repository = client.create_repository("PROJECT", request).await?;
//! println!("Created: {} with ID {}", repo.name, repo.id);
//! ```

use serde::{Deserialize, Serialize};

/// Represents a repository in Bitbucket Server/Data Center.
///
/// This struct contains the complete information about a repository as returned
/// by the Bitbucket Server REST API. Repositories are the primary containers
/// for source code and are always associated with a project.
///
/// # Fields
///
/// * `id` - Unique numeric identifier for the repository
/// * `slug` - URL-safe identifier used in API paths and clone URLs
/// * `name` - Human-readable display name of the repository
/// * `description` - Optional description of the repository's purpose
/// * `project` - Reference to the parent project containing this repository
/// * `scm_id` - Source control management type (typically "git")
/// * `state` - Current state of the repository (e.g., "AVAILABLE", "INITIALISING")
/// * `status_message` - Optional message about the current state
/// * `forkable` - Whether the repository allows forking
/// * `is_public` - Whether the repository is publicly accessible
/// * `links` - Collection of URLs for accessing the repository
///
/// # Example
///
/// ```rust,ignore
/// let repo: Repository = serde_json::from_str(json_response)?;
///
/// println!("Repository: {} ({})", repo.name, repo.slug);
/// println!("Project: {}", repo.project.key);
/// println!("Public: {}", repo.is_public);
///
/// // Get clone URLs
/// for link in &repo.links.clone {
///     println!("{}: {}", link.name, link.href);
/// }
/// ```
///
/// # Notes
///
/// - The `slug` is derived from the name but may differ (lowercase, hyphenated)
/// - The `state` field indicates if the repository is ready for use
/// - Clone links typically include both SSH and HTTPS options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Unique numeric identifier assigned by Bitbucket Server.
    pub id: u64,

    /// URL-safe identifier used in API endpoints and clone URLs.
    /// Typically a lowercase, hyphenated version of the name.
    pub slug: String,

    /// Human-readable display name of the repository.
    pub name: String,

    /// Optional description explaining the repository's purpose.
    /// May be `None` if no description was provided during creation.
    #[serde(default)]
    pub description: Option<String>,

    /// Reference to the project that contains this repository.
    pub project: ProjectRef,

    /// Source control management identifier (typically "git").
    #[serde(rename = "scmId")]
    pub scm_id: String,

    /// Current state of the repository.
    /// Common values: "AVAILABLE", "INITIALISING", "INITIALISATION_FAILED".
    pub state: String,

    /// Optional message providing additional context about the state.
    /// Usually present when state is not "AVAILABLE".
    #[serde(rename = "statusMessage")]
    #[serde(default)]
    pub status_message: Option<String>,

    /// Whether the repository allows forking.
    /// Defaults to `false` if not specified in the API response.
    #[serde(default)]
    pub forkable: bool,

    /// Whether the repository is publicly accessible.
    /// Defaults to `false` if not specified in the API response.
    #[serde(rename = "public")]
    #[serde(default)]
    pub is_public: bool,

    /// Collection of links for accessing the repository.
    pub links: RepositoryLinks,
}

/// Reference to a project within a repository context.
///
/// This is a lightweight representation of a project containing only the
/// essential identification fields. Used when a repository needs to reference
/// its parent project without including full project details.
///
/// # Fields
///
/// * `id` - Unique numeric identifier for the project
/// * `key` - Short uppercase key used in URLs (e.g., "PROJ", "DEV")
/// * `name` - Human-readable display name of the project
/// * `is_public` - Whether the project is publicly accessible
///
/// # Example
///
/// ```rust,ignore
/// let project_ref = &repository.project;
/// println!("Project: {} ({})", project_ref.name, project_ref.key);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRef {
    /// Unique numeric identifier for the project.
    pub id: u64,

    /// Short uppercase key used in URLs and API paths.
    /// Example: "PROJ", "DEV", "INFRA".
    pub key: String,

    /// Human-readable display name of the project.
    pub name: String,

    /// Whether the project is publicly accessible.
    /// Defaults to `false` if not specified.
    #[serde(rename = "public")]
    #[serde(default)]
    pub is_public: bool,
}

/// Collection of links associated with a repository.
///
/// Contains URLs for cloning the repository and accessing it via the web UI.
/// The Bitbucket Server API returns links as arrays to support multiple
/// protocols (SSH, HTTPS) and formats.
///
/// # Fields
///
/// * `clone` - List of clone URLs (typically SSH and HTTPS)
/// * `self_link` - List of self-referential web UI URLs
///
/// # Example
///
/// ```rust,ignore
/// // Find the SSH clone URL
/// let ssh_url = repo.links.clone
///     .iter()
///     .find(|link| link.name == "ssh")
///     .map(|link| &link.href);
///
/// // Find the HTTPS clone URL
/// let https_url = repo.links.clone
///     .iter()
///     .find(|link| link.name == "http")
///     .map(|link| &link.href);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryLinks {
    /// List of clone URLs for the repository.
    /// Typically contains entries for "ssh" and "http" protocols.
    #[serde(default)]
    pub clone: Vec<CloneLink>,

    /// Self-referential links to the repository in the web UI.
    #[serde(default, rename = "self")]
    pub self_link: Vec<SelfLink>,
}

/// Represents a clone URL for a repository.
///
/// Each clone link provides a URL for cloning the repository using a specific
/// protocol. The `name` field identifies the protocol type.
///
/// # Fields
///
/// * `href` - The full clone URL
/// * `name` - Protocol identifier (e.g., "ssh", "http")
///
/// # Example
///
/// ```rust,ignore
/// for clone_link in &repo.links.clone {
///     match clone_link.name.as_str() {
///         "ssh" => println!("SSH: {}", clone_link.href),
///         "http" => println!("HTTPS: {}", clone_link.href),
///         _ => println!("Other: {}", clone_link.href),
///     }
/// }
/// ```
///
/// # Notes
///
/// - SSH URLs typically start with `ssh://` or use `git@` format
/// - HTTP URLs are usually HTTPS in production environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneLink {
    /// The full clone URL for this protocol.
    /// Example: "ssh://git@bitbucket.example.com:7999/proj/repo.git"
    pub href: String,

    /// Protocol identifier for this clone URL.
    /// Common values: "ssh", "http".
    pub name: String,
}

/// Self-referential link to a resource.
///
/// Used in the Bitbucket Server API to provide a URL back to the resource
/// in the web UI. Typically used for navigation and reference purposes.
///
/// # Fields
///
/// * `href` - The full URL to the resource
///
/// # Example
///
/// ```rust,ignore
/// if let Some(link) = repo.links.self_link.first() {
///     println!("View in browser: {}", link.href);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfLink {
    /// The full URL to view this resource in the web UI.
    pub href: String,
}

/// Request body for creating a new repository.
///
/// Used when sending a POST request to create a new repository within a project.
/// Only the `name` field is strictly required; other fields have sensible defaults.
///
/// # Fields
///
/// * `name` - Required name for the new repository
/// * `description` - Optional description of the repository
/// * `scm_id` - Source control type (defaults to "git")
/// * `forkable` - Whether to allow forking (defaults to `true`)
/// * `is_public` - Whether the repository should be public (defaults to `false`)
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::api::server::repositories::CreateRepositoryRequest;
///
/// // Minimal request
/// let request = CreateRepositoryRequest {
///     name: "my-repo".to_string(),
///     ..Default::default()
/// };
///
/// // Full request
/// let request = CreateRepositoryRequest {
///     name: "my-repo".to_string(),
///     description: Some("Repository for my project".to_string()),
///     scm_id: "git".to_string(),
///     forkable: Some(true),
///     is_public: Some(false),
/// };
/// ```
///
/// # Notes
///
/// - The repository slug will be derived from the name automatically
/// - Optional fields use `skip_serializing_if` to omit `None` values from JSON
#[derive(Debug, Clone, Serialize)]
pub struct CreateRepositoryRequest {
    /// Name for the new repository.
    /// The slug will be derived from this name.
    pub name: String,

    /// Optional description of the repository.
    /// Omitted from the request if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Source control management type.
    /// Currently only "git" is supported by Bitbucket Server.
    #[serde(rename = "scmId")]
    pub scm_id: String,

    /// Whether the repository should allow forking.
    /// Defaults to `true` when using `Default::default()`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub forkable: Option<bool>,

    /// Whether the repository should be publicly accessible.
    /// Defaults to `false` when using `Default::default()`.
    #[serde(rename = "public")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
}

impl Default for CreateRepositoryRequest {
    /// Creates a default repository creation request.
    ///
    /// # Default Values
    ///
    /// * `name` - Empty string (must be set before use)
    /// * `description` - `None`
    /// * `scm_id` - "git"
    /// * `forkable` - `Some(true)`
    /// * `is_public` - `Some(false)`
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let mut request = CreateRepositoryRequest::default();
    /// request.name = "my-repo".to_string();
    /// ```
    fn default() -> Self {
        Self {
            name: String::new(),
            description: None,
            scm_id: "git".to_string(),
            forkable: Some(true),
            is_public: Some(false),
        }
    }
}
