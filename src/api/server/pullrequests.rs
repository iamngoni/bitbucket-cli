//
//  bitbucket-cli
//  api/server/pullrequests.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # Bitbucket Server/DC Pull Request API
//!
//! This module provides types and structures for working with pull requests
//! in Bitbucket Server/Data Center. Pull requests enable code review workflows
//! by proposing changes from one branch to another.
//!
//! ## Pull Request Workflow
//!
//! In Bitbucket Server/DC, pull requests:
//! - Propose merging changes from a source branch (`fromRef`) to a target branch (`toRef`)
//! - Support multiple reviewers who can approve or request changes
//! - Track participants (author, reviewers, and other commenters)
//! - Can be merged, declined, or reopened
//!
//! ## API Endpoints
//!
//! Pull request operations use these endpoints:
//! ```text
//! GET/POST /rest/api/1.0/projects/{projectKey}/repos/{repoSlug}/pull-requests
//! GET/PUT/DELETE /rest/api/1.0/projects/{projectKey}/repos/{repoSlug}/pull-requests/{pullRequestId}
//! POST /rest/api/1.0/projects/{projectKey}/repos/{repoSlug}/pull-requests/{pullRequestId}/merge
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use bitbucket_cli::api::server::pullrequests::{CreatePullRequestRequest, RefSpec};
//!
//! let request = CreatePullRequestRequest {
//!     title: "Add new feature".to_string(),
//!     description: Some("This PR adds the new widget feature".to_string()),
//!     from_ref: RefSpec {
//!         id: "refs/heads/feature/widget".to_string(),
//!         repository: RepositorySpec {
//!             slug: "my-repo".to_string(),
//!             project: ProjectSpec { key: "PROJ".to_string() },
//!         },
//!     },
//!     to_ref: RefSpec {
//!         id: "refs/heads/main".to_string(),
//!         repository: RepositorySpec {
//!             slug: "my-repo".to_string(),
//!             project: ProjectSpec { key: "PROJ".to_string() },
//!         },
//!     },
//!     reviewers: vec![],
//! };
//! ```
//!
//! ## Notes
//!
//! - Timestamps (`created_date`, `updated_date`) are Unix milliseconds
//! - Branch IDs use the full ref path format: "refs/heads/branch-name"
//! - Reviewer approval status is tracked per-participant

use serde::{Deserialize, Serialize};

/// Represents a pull request in Bitbucket Server/Data Center.
///
/// A pull request is a proposal to merge changes from one branch (source) into
/// another branch (target). It tracks the state of the review process, including
/// approvals from reviewers and the overall open/closed status.
///
/// # Fields
///
/// * `id` - Unique numeric identifier for the pull request
/// * `title` - Short summary of the changes being proposed
/// * `description` - Detailed explanation of the changes (optional)
/// * `state` - Current state (OPEN, MERGED, DECLINED)
/// * `open` - Whether the PR is currently open for review
/// * `closed` - Whether the PR has been closed (merged or declined)
/// * `created_date` - Unix timestamp (milliseconds) when the PR was created
/// * `updated_date` - Unix timestamp (milliseconds) of the last update
/// * `from_ref` - Source branch reference (the changes to merge)
/// * `to_ref` - Target branch reference (where changes will be merged)
/// * `author` - User who created the pull request
/// * `reviewers` - Users assigned to review the pull request
/// * `participants` - All users who have interacted with the PR
///
/// # Example
///
/// ```rust,ignore
/// let pr: PullRequest = client.get_pull_request("PROJ", "repo", 42).await?;
///
/// println!("PR #{}: {}", pr.id, pr.title);
/// println!("State: {} (open: {}, closed: {})", pr.state, pr.open, pr.closed);
/// println!("From: {} -> To: {}", pr.from_ref.display_id, pr.to_ref.display_id);
///
/// // Check approval status
/// let approved_count = pr.reviewers.iter().filter(|r| r.approved).count();
/// println!("Approvals: {}/{}", approved_count, pr.reviewers.len());
/// ```
///
/// # Notes
///
/// - `state` can be "OPEN", "MERGED", or "DECLINED"
/// - The `open` and `closed` fields provide quick boolean checks
/// - Timestamps are in Unix milliseconds, not seconds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// Unique numeric identifier for this pull request.
    pub id: u64,

    /// Short summary title describing the changes.
    pub title: String,

    /// Detailed description of the changes being proposed.
    /// May contain Markdown formatting.
    #[serde(default)]
    pub description: Option<String>,

    /// Current state of the pull request.
    /// Values: "OPEN", "MERGED", "DECLINED".
    pub state: String,

    /// Whether the pull request is currently open for review.
    pub open: bool,

    /// Whether the pull request has been closed (merged or declined).
    pub closed: bool,

    /// Unix timestamp in milliseconds when the PR was created.
    #[serde(rename = "createdDate")]
    pub created_date: u64,

    /// Unix timestamp in milliseconds of the last update.
    #[serde(rename = "updatedDate")]
    pub updated_date: u64,

    /// Source branch reference containing the changes to merge.
    #[serde(rename = "fromRef")]
    pub from_ref: PrRef,

    /// Target branch reference where changes will be merged.
    #[serde(rename = "toRef")]
    pub to_ref: PrRef,

    /// The user who created this pull request.
    pub author: PrParticipant,

    /// Users assigned to review this pull request.
    #[serde(default)]
    pub reviewers: Vec<PrParticipant>,

    /// All users who have participated in this pull request.
    /// Includes the author, reviewers, and commenters.
    #[serde(default)]
    pub participants: Vec<PrParticipant>,
}

/// Branch reference within a pull request context.
///
/// Contains information about a branch involved in a pull request, including
/// the full ref path, display name, latest commit, and the repository it
/// belongs to.
///
/// # Fields
///
/// * `id` - Full ref path (e.g., "refs/heads/main")
/// * `display_id` - Short display name (e.g., "main")
/// * `latest_commit` - SHA of the latest commit on this branch
/// * `repository` - Reference to the repository containing this branch
///
/// # Example
///
/// ```rust,ignore
/// let from_branch = &pr.from_ref;
/// println!("Source: {} ({})", from_branch.display_id, from_branch.id);
///
/// if let Some(commit) = &from_branch.latest_commit {
///     println!("Latest commit: {}", commit);
/// }
/// ```
///
/// # Notes
///
/// - The `id` is the full Git ref path used by Git
/// - The `display_id` is the human-readable short form
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrRef {
    /// Full Git ref path for the branch.
    /// Example: "refs/heads/feature/my-feature".
    pub id: String,

    /// Human-readable short name for the branch.
    /// Example: "feature/my-feature".
    #[serde(rename = "displayId")]
    pub display_id: String,

    /// SHA hash of the latest commit on this branch.
    /// May be `None` if the ref information is not fully populated.
    #[serde(rename = "latestCommit")]
    #[serde(default)]
    pub latest_commit: Option<String>,

    /// Reference to the repository containing this branch.
    pub repository: RepositoryRef,
}

/// Lightweight reference to a repository.
///
/// Used within pull request contexts to identify which repository a branch
/// belongs to. Contains essential identification fields only.
///
/// # Fields
///
/// * `id` - Unique numeric identifier for the repository
/// * `slug` - URL-safe identifier used in API paths
/// * `name` - Human-readable display name
/// * `project` - Reference to the parent project
///
/// # Example
///
/// ```rust,ignore
/// let repo = &pr.from_ref.repository;
/// println!("Repository: {}/{}", repo.project.key, repo.slug);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryRef {
    /// Unique numeric identifier for the repository.
    pub id: u64,

    /// URL-safe identifier used in API endpoints.
    pub slug: String,

    /// Human-readable display name of the repository.
    pub name: String,

    /// Reference to the project containing this repository.
    pub project: ProjectRef,
}

/// Lightweight reference to a project.
///
/// Used within repository references to identify the parent project.
/// Contains only the essential identification fields.
///
/// # Fields
///
/// * `id` - Unique numeric identifier for the project
/// * `key` - Short uppercase key used in URLs
/// * `name` - Human-readable display name
///
/// # Example
///
/// ```rust,ignore
/// let project = &pr.from_ref.repository.project;
/// println!("Project: {} ({})", project.name, project.key);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRef {
    /// Unique numeric identifier for the project.
    pub id: u64,

    /// Short uppercase key used in URLs and API paths.
    /// Example: "PROJ", "DEV".
    pub key: String,

    /// Human-readable display name of the project.
    pub name: String,
}

/// Represents a participant in a pull request.
///
/// A participant is any user involved in a pull request, including the author,
/// assigned reviewers, and users who have commented. The `role` field indicates
/// the type of participation.
///
/// # Fields
///
/// * `user` - The user who is participating
/// * `role` - Role in the PR (AUTHOR, REVIEWER, PARTICIPANT)
/// * `approved` - Whether this participant has approved the PR
/// * `status` - Review status (APPROVED, UNAPPROVED, NEEDS_WORK)
///
/// # Example
///
/// ```rust,ignore
/// for reviewer in &pr.reviewers {
///     let status = if reviewer.approved { "approved" } else { "pending" };
///     println!("{}: {}", reviewer.user.display_name, status);
/// }
/// ```
///
/// # Notes
///
/// - `approved` is a quick boolean check; `status` provides more detail
/// - Status values: "APPROVED", "UNAPPROVED", "NEEDS_WORK"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrParticipant {
    /// The user who is participating.
    pub user: User,

    /// Role of this participant in the pull request.
    /// Values: "AUTHOR", "REVIEWER", "PARTICIPANT".
    pub role: String,

    /// Whether this participant has approved the pull request.
    #[serde(default)]
    pub approved: bool,

    /// Detailed review status from this participant.
    /// Values: "APPROVED", "UNAPPROVED", "NEEDS_WORK".
    #[serde(default)]
    pub status: Option<String>,
}

/// Represents a user in Bitbucket Server/Data Center.
///
/// Contains user identification and contact information. Used throughout
/// the API to represent authors, reviewers, and other participants.
///
/// # Fields
///
/// * `id` - Unique numeric identifier for the user
/// * `name` - Username (login name)
/// * `display_name` - Full display name
/// * `email_address` - Email address (may be hidden by privacy settings)
/// * `slug` - URL-safe version of the username
///
/// # Example
///
/// ```rust,ignore
/// let author = &pr.author.user;
/// println!("Author: {} (@{})", author.display_name, author.name);
///
/// if let Some(email) = &author.email_address {
///     println!("Email: {}", email);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// Unique numeric identifier for the user.
    pub id: u64,

    /// Username (login name) for the user.
    pub name: String,

    /// Full display name of the user.
    #[serde(rename = "displayName")]
    pub display_name: String,

    /// Email address of the user.
    /// May be `None` if hidden by privacy settings.
    #[serde(rename = "emailAddress")]
    #[serde(default)]
    pub email_address: Option<String>,

    /// URL-safe version of the username.
    /// May differ from `name` if special characters are present.
    #[serde(default)]
    pub slug: Option<String>,
}

/// Request body for creating a new pull request.
///
/// Used when sending a POST request to create a new pull request. Requires
/// specifying the source and target branches, along with title and optional
/// reviewers.
///
/// # Fields
///
/// * `title` - Required title summarizing the changes
/// * `description` - Optional detailed description
/// * `from_ref` - Source branch specification (where changes come from)
/// * `to_ref` - Target branch specification (where changes will merge)
/// * `reviewers` - Optional list of users to assign as reviewers
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::api::server::pullrequests::*;
///
/// let request = CreatePullRequestRequest {
///     title: "Add user authentication".to_string(),
///     description: Some("Implements OAuth2 authentication flow".to_string()),
///     from_ref: RefSpec {
///         id: "refs/heads/feature/auth".to_string(),
///         repository: RepositorySpec {
///             slug: "my-app".to_string(),
///             project: ProjectSpec { key: "PROJ".to_string() },
///         },
///     },
///     to_ref: RefSpec {
///         id: "refs/heads/main".to_string(),
///         repository: RepositorySpec {
///             slug: "my-app".to_string(),
///             project: ProjectSpec { key: "PROJ".to_string() },
///         },
///     },
///     reviewers: vec![
///         UserRef { user: UserName { name: "jsmith".to_string() } },
///     ],
/// };
/// ```
///
/// # Notes
///
/// - Branch IDs should use full ref paths: "refs/heads/branch-name"
/// - Empty reviewers list is valid; reviewers can be added later
#[derive(Debug, Clone, Serialize)]
pub struct CreatePullRequestRequest {
    /// Title summarizing the changes in this pull request.
    pub title: String,

    /// Optional detailed description of the changes.
    /// Supports Markdown formatting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Source branch specification (where changes come from).
    #[serde(rename = "fromRef")]
    pub from_ref: RefSpec,

    /// Target branch specification (where changes will merge).
    #[serde(rename = "toRef")]
    pub to_ref: RefSpec,

    /// List of users to assign as reviewers.
    /// Empty list is valid; omitted from JSON if empty.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reviewers: Vec<UserRef>,
}

/// Branch reference specification for pull request creation.
///
/// Specifies a branch by its full ref path and the repository it belongs to.
/// Used in [`CreatePullRequestRequest`] to identify source and target branches.
///
/// # Fields
///
/// * `id` - Full Git ref path (e.g., "refs/heads/main")
/// * `repository` - Repository containing this branch
///
/// # Example
///
/// ```rust,ignore
/// let from_ref = RefSpec {
///     id: "refs/heads/feature/new-feature".to_string(),
///     repository: RepositorySpec {
///         slug: "my-repo".to_string(),
///         project: ProjectSpec { key: "PROJ".to_string() },
///     },
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct RefSpec {
    /// Full Git ref path for the branch.
    /// Must use format: "refs/heads/<branch-name>".
    pub id: String,

    /// Repository containing this branch.
    pub repository: RepositorySpec,
}

/// Repository specification for pull request creation.
///
/// Identifies a repository by its slug and parent project. Used within
/// [`RefSpec`] to fully qualify a branch reference.
///
/// # Fields
///
/// * `slug` - URL-safe identifier for the repository
/// * `project` - Project containing the repository
///
/// # Example
///
/// ```rust,ignore
/// let repo_spec = RepositorySpec {
///     slug: "my-repo".to_string(),
///     project: ProjectSpec { key: "PROJ".to_string() },
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct RepositorySpec {
    /// URL-safe identifier for the repository.
    pub slug: String,

    /// Project containing this repository.
    pub project: ProjectSpec,
}

/// Project specification for pull request creation.
///
/// Identifies a project by its unique key. Used within [`RepositorySpec`]
/// to fully qualify a repository reference.
///
/// # Fields
///
/// * `key` - Short uppercase project key (e.g., "PROJ", "DEV")
///
/// # Example
///
/// ```rust,ignore
/// let project_spec = ProjectSpec { key: "PROJ".to_string() };
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct ProjectSpec {
    /// Short uppercase key identifying the project.
    /// Example: "PROJ", "DEV", "INFRA".
    pub key: String,
}

/// User reference for adding reviewers to a pull request.
///
/// Wraps a [`UserName`] to match the API's expected JSON structure for
/// specifying reviewers when creating or updating a pull request.
///
/// # Fields
///
/// * `user` - The user name specification
///
/// # Example
///
/// ```rust,ignore
/// let reviewer = UserRef {
///     user: UserName { name: "jsmith".to_string() },
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct UserRef {
    /// User name specification for the reviewer.
    pub user: UserName,
}

/// User name reference for identifying users.
///
/// Contains just the username string, used when only the user's login name
/// is needed (such as when adding reviewers).
///
/// # Fields
///
/// * `name` - Username (login name) of the user
///
/// # Example
///
/// ```rust,ignore
/// let user_name = UserName { name: "jsmith".to_string() };
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct UserName {
    /// Username (login name) of the user.
    pub name: String,
}

/// Request body for merging a pull request.
///
/// Used when sending a POST request to merge a pull request. Both fields
/// are optional; if not provided, the server uses default values.
///
/// # Fields
///
/// * `version` - Expected version number for optimistic locking
/// * `message` - Custom merge commit message
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::api::server::pullrequests::MergePullRequestRequest;
///
/// // Simple merge with defaults
/// let request = MergePullRequestRequest {
///     version: None,
///     message: None,
/// };
///
/// // Merge with custom message and version check
/// let request = MergePullRequestRequest {
///     version: Some(5),
///     message: Some("Merge feature branch".to_string()),
/// };
/// ```
///
/// # Notes
///
/// - The `version` field enables optimistic locking to prevent concurrent merges
/// - If `version` is provided and doesn't match, the merge will fail
/// - The `message` field overrides the default merge commit message
#[derive(Debug, Clone, Serialize)]
pub struct MergePullRequestRequest {
    /// Expected version number for optimistic locking.
    /// The merge will fail if the PR has been updated since this version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<u64>,

    /// Custom message for the merge commit.
    /// If not provided, a default message is generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}
