//
//  bitbucket-cli
//  api/cloud/repositories.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Cloud repository API types and data structures.
//!
//! This module provides types for interacting with Bitbucket Cloud repositories,
//! including repository metadata, branches, owners, and related references.
//!
//! # Overview
//!
//! Repositories are the core resource in Bitbucket. They contain the source code,
//! pull requests, issues, and pipelines for a project. Each repository belongs to
//! a workspace and optionally to a project within that workspace.
//!
//! # Example
//!
//! ```rust,no_run
//! use bitbucket_cli::api::cloud::repositories::{Repository, CreateRepositoryRequest, ProjectKey};
//!
//! // Create a repository in a specific project
//! let request = CreateRepositoryRequest {
//!     name: "backend-service".to_string(),
//!     description: Some("Main backend microservice".to_string()),
//!     is_private: Some(true),
//!     project: Some(ProjectKey { key: "BACKEND".to_string() }),
//!     language: Some("rust".to_string()),
//! };
//! ```
//!
//! # Notes
//!
//! - Repository slugs are URL-safe versions of repository names
//! - The `full_name` field follows the format `{workspace}/{repo_slug}`
//! - Private repositories require authentication for all operations

use serde::{Deserialize, Serialize};

/// Represents a Bitbucket Cloud repository.
///
/// A repository contains all the source code, history, branches, and associated
/// metadata for a project. Repositories belong to a workspace and may optionally
/// be organized within a project.
///
/// # Fields
///
/// * `uuid` - Unique identifier for the repository (includes curly braces)
/// * `name` - Human-readable name of the repository
/// * `full_name` - Full path in format `{workspace_slug}/{repo_slug}`
/// * `slug` - URL-safe identifier derived from the name
/// * `description` - Optional description of the repository
/// * `is_private` - Whether the repository is private (default: false)
/// * `language` - Primary programming language detected in the repository
/// * `mainbranch` - Reference to the main/default branch
/// * `owner` - The user or team that owns the repository
/// * `workspace` - The workspace containing this repository
/// * `project` - Optional project within the workspace
/// * `created_on` - ISO 8601 timestamp of creation
/// * `updated_on` - ISO 8601 timestamp of last update
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::repositories::Repository;
///
/// fn display_repo(repo: &Repository) {
///     println!("Repository: {}", repo.full_name);
///     println!("  Private: {}", repo.is_private);
///     if let Some(ref lang) = repo.language {
///         println!("  Language: {}", lang);
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Unique identifier for the repository (e.g., `{123e4567-e89b-...}`).
    pub uuid: String,

    /// Human-readable name of the repository.
    pub name: String,

    /// Full path in format `{workspace_slug}/{repo_slug}`.
    pub full_name: String,

    /// URL-safe identifier derived from the repository name.
    pub slug: String,

    /// Optional description of the repository's purpose.
    #[serde(default)]
    pub description: Option<String>,

    /// Whether the repository is private. Defaults to `false` if not specified.
    #[serde(default)]
    pub is_private: bool,

    /// Primary programming language detected in the repository.
    #[serde(default)]
    pub language: Option<String>,

    /// Reference to the main/default branch of the repository.
    #[serde(default)]
    pub mainbranch: Option<Branch>,

    /// The user or team that owns this repository.
    pub owner: Owner,

    /// The workspace containing this repository.
    pub workspace: WorkspaceRef,

    /// Optional project within the workspace that organizes this repository.
    #[serde(default)]
    pub project: Option<ProjectRef>,

    /// ISO 8601 timestamp indicating when the repository was created.
    pub created_on: String,

    /// ISO 8601 timestamp indicating when the repository was last updated.
    pub updated_on: String,
}

/// Represents a branch within a repository.
///
/// Branches are used to isolate development work and represent different
/// lines of development within a repository.
///
/// # Fields
///
/// * `name` - The name of the branch (e.g., `main`, `develop`, `feature/login`)
/// * `branch_type` - Optional type classification (e.g., `branch`, `named_branch`)
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::repositories::Branch;
///
/// let main_branch = Branch {
///     name: "main".to_string(),
///     branch_type: Some("branch".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    /// The name of the branch.
    pub name: String,

    /// Optional type classification for the branch.
    #[serde(rename = "type")]
    pub branch_type: Option<String>,
}

/// Represents the owner of a repository.
///
/// The owner can be either a user or a team. This struct provides
/// identification and display information for the owner entity.
///
/// # Fields
///
/// * `uuid` - Unique identifier for the owner
/// * `display_name` - Human-readable name for display purposes
/// * `username` - Optional username/nickname for the owner
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::repositories::Owner;
///
/// let owner = Owner {
///     uuid: "{abc123...}".to_string(),
///     display_name: "John Doe".to_string(),
///     username: Some("johndoe".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Owner {
    /// Unique identifier for the owner.
    pub uuid: String,

    /// Human-readable display name.
    #[serde(alias = "display_name")]
    pub display_name: String,

    /// Optional username or nickname.
    #[serde(alias = "nickname")]
    pub username: Option<String>,
}

/// A lightweight reference to a workspace.
///
/// Used when embedding workspace information within other resources
/// without including the full workspace details.
///
/// # Fields
///
/// * `uuid` - Unique identifier for the workspace
/// * `slug` - URL-safe identifier for the workspace
/// * `name` - Human-readable name of the workspace
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::repositories::WorkspaceRef;
///
/// let workspace = WorkspaceRef {
///     uuid: "{workspace-uuid}".to_string(),
///     slug: "my-team".to_string(),
///     name: "My Team".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRef {
    /// Unique identifier for the workspace.
    pub uuid: String,

    /// URL-safe identifier for the workspace.
    pub slug: String,

    /// Human-readable name of the workspace.
    pub name: String,
}

/// A lightweight reference to a project within a workspace.
///
/// Projects are used to organize repositories within a workspace.
/// This struct provides minimal project information for embedding
/// within repository responses.
///
/// # Fields
///
/// * `uuid` - Unique identifier for the project
/// * `key` - Short alphanumeric key for the project (e.g., `PROJ`)
/// * `name` - Human-readable name of the project
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::repositories::ProjectRef;
///
/// let project = ProjectRef {
///     uuid: "{project-uuid}".to_string(),
///     key: "BACKEND".to_string(),
///     name: "Backend Services".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRef {
    /// Unique identifier for the project.
    pub uuid: String,

    /// Short alphanumeric key for the project (typically uppercase).
    pub key: String,

    /// Human-readable name of the project.
    pub name: String,
}

/// Request payload for creating a new repository.
///
/// This struct is used when making POST requests to create a new repository
/// in a workspace. Only the `name` field is required; all other fields are optional.
///
/// # Fields
///
/// * `name` - Required name for the new repository
/// * `description` - Optional description of the repository
/// * `is_private` - Whether the repository should be private (default: workspace setting)
/// * `project` - Optional project to organize the repository under
/// * `language` - Optional primary language tag
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::repositories::{CreateRepositoryRequest, ProjectKey};
///
/// // Create a minimal repository
/// let minimal = CreateRepositoryRequest {
///     name: "my-repo".to_string(),
///     description: None,
///     is_private: None,
///     project: None,
///     language: None,
/// };
///
/// // Create a fully specified repository
/// let full = CreateRepositoryRequest {
///     name: "backend-api".to_string(),
///     description: Some("REST API for the backend".to_string()),
///     is_private: Some(true),
///     project: Some(ProjectKey { key: "BACKEND".to_string() }),
///     language: Some("rust".to_string()),
/// };
/// ```
///
/// # Notes
///
/// - The repository slug is automatically generated from the name
/// - If `is_private` is not specified, it inherits from the workspace default
#[derive(Debug, Clone, Serialize)]
pub struct CreateRepositoryRequest {
    /// The name for the new repository. This is required and will be used
    /// to generate the repository slug.
    pub name: String,

    /// Optional description of the repository's purpose.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether the repository should be private. If not specified,
    /// inherits from the workspace default setting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_private: Option<bool>,

    /// Optional project to organize the repository under.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<ProjectKey>,

    /// Optional primary programming language tag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
}

/// Project key for assigning a repository to a project.
///
/// Used in repository creation and update requests to specify
/// which project the repository should belong to.
///
/// # Fields
///
/// * `key` - The short alphanumeric key of the project
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::repositories::ProjectKey;
///
/// let project_key = ProjectKey {
///     key: "FRONTEND".to_string(),
/// };
/// ```
///
/// # Notes
///
/// - Project keys are typically uppercase alphanumeric strings
/// - The project must exist in the workspace before assignment
#[derive(Debug, Clone, Serialize)]
pub struct ProjectKey {
    /// The short alphanumeric key identifying the project.
    pub key: String,
}
