//
//  bitbucket-cli
//  api/cloud/pullrequests.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Cloud pull request API types and data structures.
//!
//! This module provides types for managing pull requests in Bitbucket Cloud,
//! including creating, reviewing, and merging pull requests.
//!
//! # Overview
//!
//! Pull requests are the primary mechanism for code review in Bitbucket.
//! They represent a request to merge changes from one branch into another,
//! and provide a forum for discussion, review, and approval.
//!
//! # Pull Request Lifecycle
//!
//! 1. **OPEN** - Initial state when created
//! 2. **MERGED** - Successfully merged into the destination branch
//! 3. **DECLINED** - Rejected and closed without merging
//! 4. **SUPERSEDED** - Replaced by another pull request
//!
//! # Example
//!
//! ```rust,no_run
//! use bitbucket_cli::api::cloud::pullrequests::{
//!     CreatePullRequestRequest, BranchSpec, BranchName
//! };
//!
//! // Create a pull request from feature branch to main
//! let request = CreatePullRequestRequest {
//!     title: "Add user authentication".to_string(),
//!     description: Some("Implements OAuth2 login flow".to_string()),
//!     source: BranchSpec {
//!         branch: BranchName { name: "feature/auth".to_string() },
//!     },
//!     destination: BranchSpec {
//!         branch: BranchName { name: "main".to_string() },
//!     },
//!     reviewers: vec![],
//!     close_source_branch: Some(true),
//! };
//! ```
//!
//! # Notes
//!
//! - Pull requests are numbered sequentially within each repository
//! - Reviewers can be added at creation time or later
//! - The source branch can optionally be deleted upon merge

use serde::{Deserialize, Serialize};

use super::repositories::Branch;
use crate::api::common::UserRef;

/// Represents a pull request in Bitbucket Cloud.
///
/// A pull request contains all metadata about a proposed change, including
/// the source and destination branches, reviewers, and current state.
///
/// # Fields
///
/// * `id` - Unique numeric identifier within the repository
/// * `title` - Short summary of the changes
/// * `description` - Optional detailed description (supports Markdown)
/// * `state` - Current state (`OPEN`, `MERGED`, `DECLINED`, `SUPERSEDED`)
/// * `author` - The user who created the pull request
/// * `source` - The branch containing the changes
/// * `destination` - The branch to merge into
/// * `reviewers` - Users assigned to review
/// * `participants` - All users who have participated
/// * `created_on` - ISO 8601 creation timestamp
/// * `updated_on` - ISO 8601 last update timestamp
/// * `closed_by` - User who merged or declined (if applicable)
/// * `merge_commit` - The merge commit (if merged)
/// * `comment_count` - Number of comments on the PR
/// * `task_count` - Number of tasks/todos in the PR
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pullrequests::PullRequest;
///
/// fn display_pr(pr: &PullRequest) {
///     println!("PR #{}: {}", pr.id, pr.title);
///     println!("  State: {}", pr.state);
///     println!("  {} -> {}", pr.source.branch.name, pr.destination.branch.name);
///     println!("  Reviewers: {}", pr.reviewers.len());
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    /// Unique numeric identifier within the repository.
    pub id: u64,

    /// Short summary of the changes (typically a single line).
    pub title: String,

    /// Optional detailed description of the changes. Supports Markdown formatting.
    #[serde(default)]
    pub description: Option<String>,

    /// Current state of the pull request.
    /// Possible values: `OPEN`, `MERGED`, `DECLINED`, `SUPERSEDED`.
    pub state: String,

    /// The user who created this pull request.
    pub author: UserRef,

    /// The branch containing the proposed changes.
    pub source: PrBranchRef,

    /// The target branch that changes will be merged into.
    pub destination: PrBranchRef,

    /// List of users assigned to review this pull request.
    #[serde(default)]
    pub reviewers: Vec<UserRef>,

    /// All users who have participated in this pull request
    /// (commented, approved, or made changes).
    #[serde(default)]
    pub participants: Vec<Participant>,

    /// ISO 8601 timestamp indicating when the pull request was created.
    pub created_on: String,

    /// ISO 8601 timestamp indicating when the pull request was last updated.
    pub updated_on: String,

    /// The user who merged or declined this pull request (if applicable).
    #[serde(default)]
    pub closed_by: Option<UserRef>,

    /// Reference to the merge commit, if the pull request has been merged.
    #[serde(default)]
    pub merge_commit: Option<CommitRef>,

    /// Total number of comments on this pull request.
    #[serde(default)]
    pub comment_count: u32,

    /// Total number of tasks/todos on this pull request.
    #[serde(default)]
    pub task_count: u32,
}

/// Branch reference within a pull request context.
///
/// Contains detailed information about a branch involved in a pull request,
/// including the repository it belongs to and the current commit.
///
/// # Fields
///
/// * `branch` - The branch information
/// * `repository` - The repository containing this branch
/// * `commit` - The current commit at the head of the branch
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pullrequests::PrBranchRef;
///
/// fn show_branch(branch_ref: &PrBranchRef) {
///     println!("Branch: {} in {}", branch_ref.branch.name, branch_ref.repository.full_name);
///     if let Some(ref commit) = branch_ref.commit {
///         println!("  HEAD: {}", commit.hash);
///     }
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrBranchRef {
    /// The branch information including name and type.
    pub branch: Branch,

    /// The repository that contains this branch.
    pub repository: RepositoryRef,

    /// The current commit at the head of this branch.
    #[serde(default)]
    pub commit: Option<CommitRef>,
}

/// Minimal repository reference for embedding in other structures.
///
/// Provides essential repository identification without full details.
///
/// # Fields
///
/// * `uuid` - Unique identifier for the repository
/// * `name` - Human-readable name
/// * `full_name` - Full path in format `{workspace}/{repo_slug}`
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pullrequests::RepositoryRef;
///
/// let repo_ref = RepositoryRef {
///     uuid: "{repo-uuid}".to_string(),
///     name: "my-repo".to_string(),
///     full_name: "my-team/my-repo".to_string(),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryRef {
    /// Unique identifier for the repository.
    pub uuid: String,

    /// Human-readable name of the repository.
    pub name: String,

    /// Full path in format `{workspace_slug}/{repo_slug}`.
    pub full_name: String,
}

/// Reference to a Git commit.
///
/// Provides minimal commit information including the hash and optional message.
///
/// # Fields
///
/// * `hash` - The full 40-character SHA-1 hash of the commit
/// * `message` - Optional commit message (may be truncated)
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pullrequests::CommitRef;
///
/// let commit = CommitRef {
///     hash: "abc123def456...".to_string(),
///     message: Some("Fix authentication bug".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitRef {
    /// The full 40-character SHA-1 hash of the commit.
    pub hash: String,

    /// Optional commit message. May be truncated for display purposes.
    #[serde(default)]
    pub message: Option<String>,
}

/// A participant in a pull request with role and approval status.
///
/// Tracks users who have interacted with a pull request in any capacity,
/// including their role and whether they have approved the changes.
///
/// # Fields
///
/// * `user` - Reference to the participating user
/// * `role` - The user's role (`PARTICIPANT`, `REVIEWER`, `AUTHOR`)
/// * `approved` - Whether the user has approved the pull request
/// * `participated_on` - ISO 8601 timestamp of last participation
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pullrequests::Participant;
///
/// fn check_approvals(participants: &[Participant]) -> usize {
///     participants.iter().filter(|p| p.approved).count()
/// }
/// ```
///
/// # Notes
///
/// - A user can have multiple roles (e.g., author who also comments)
/// - The `approved` flag is only meaningful for reviewers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    /// Reference to the participating user.
    pub user: UserRef,

    /// The user's role in this pull request.
    /// Possible values: `PARTICIPANT`, `REVIEWER`, `AUTHOR`.
    pub role: String,

    /// Whether this user has approved the pull request.
    #[serde(default)]
    pub approved: bool,

    /// ISO 8601 timestamp of the user's last participation.
    #[serde(default)]
    pub participated_on: Option<String>,
}

/// Request payload for creating a new pull request.
///
/// Used when making POST requests to create a pull request.
/// Requires at minimum a title, source branch, and destination branch.
///
/// # Fields
///
/// * `title` - Required title for the pull request
/// * `description` - Optional detailed description (Markdown supported)
/// * `source` - The branch containing changes
/// * `destination` - The target branch for merging
/// * `reviewers` - Optional list of reviewers by UUID
/// * `close_source_branch` - Whether to delete source branch after merge
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pullrequests::{
///     CreatePullRequestRequest, BranchSpec, BranchName, UserUuid
/// };
///
/// let request = CreatePullRequestRequest {
///     title: "Implement feature X".to_string(),
///     description: Some("## Summary\n\nThis PR adds...".to_string()),
///     source: BranchSpec {
///         branch: BranchName { name: "feature/x".to_string() },
///     },
///     destination: BranchSpec {
///         branch: BranchName { name: "main".to_string() },
///     },
///     reviewers: vec![
///         UserUuid { uuid: "{reviewer-uuid}".to_string() },
///     ],
///     close_source_branch: Some(true),
/// };
/// ```
///
/// # Notes
///
/// - The source and destination branches must exist
/// - Cross-repository pull requests require additional setup
#[derive(Debug, Clone, Serialize)]
pub struct CreatePullRequestRequest {
    /// The title of the pull request (required).
    pub title: String,

    /// Optional detailed description. Supports Markdown formatting.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The source branch containing the proposed changes.
    pub source: BranchSpec,

    /// The destination branch that changes will be merged into.
    pub destination: BranchSpec,

    /// List of users to assign as reviewers, identified by UUID.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub reviewers: Vec<UserUuid>,

    /// Whether to delete the source branch after a successful merge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_source_branch: Option<bool>,
}

/// Branch specification for pull request creation.
///
/// A wrapper around branch name used in pull request creation requests.
///
/// # Fields
///
/// * `branch` - The branch name specification
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pullrequests::{BranchSpec, BranchName};
///
/// let spec = BranchSpec {
///     branch: BranchName { name: "develop".to_string() },
/// };
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct BranchSpec {
    /// The branch name specification.
    pub branch: BranchName,
}

/// Branch name wrapper for serialization.
///
/// Used in pull request creation to specify branch names.
///
/// # Fields
///
/// * `name` - The name of the branch
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pullrequests::BranchName;
///
/// let branch = BranchName { name: "feature/login".to_string() };
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct BranchName {
    /// The name of the branch.
    pub name: String,
}

/// User identification by UUID.
///
/// Used when specifying users in API requests where only the UUID is needed.
///
/// # Fields
///
/// * `uuid` - The unique identifier for the user
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pullrequests::UserUuid;
///
/// let user = UserUuid {
///     uuid: "{123e4567-e89b-12d3-a456-426614174000}".to_string(),
/// };
/// ```
///
/// # Notes
///
/// - UUIDs include curly braces as returned by the Bitbucket API
#[derive(Debug, Clone, Serialize)]
pub struct UserUuid {
    /// The unique identifier for the user (includes curly braces).
    pub uuid: String,
}

/// Request payload for merging a pull request.
///
/// Used when making POST requests to merge an approved pull request.
/// All fields are optional and provide customization of the merge behavior.
///
/// # Fields
///
/// * `message` - Optional custom merge commit message
/// * `close_source_branch` - Whether to delete source branch after merge
/// * `merge_strategy` - The merge strategy to use
///
/// # Merge Strategies
///
/// * `merge_commit` - Creates a merge commit (default)
/// * `squash` - Squashes all commits into a single commit
/// * `fast_forward` - Fast-forward merge if possible
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::pullrequests::MergePullRequestRequest;
///
/// // Squash merge with custom message
/// let request = MergePullRequestRequest {
///     message: Some("feat: Add user authentication (#123)".to_string()),
///     close_source_branch: Some(true),
///     merge_strategy: Some("squash".to_string()),
/// };
/// ```
///
/// # Notes
///
/// - The pull request must be approved before merging (if required by settings)
/// - Merge may fail if there are conflicts or failed builds
#[derive(Debug, Clone, Serialize)]
pub struct MergePullRequestRequest {
    /// Optional custom message for the merge commit.
    /// If not provided, a default message is generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,

    /// Whether to delete the source branch after a successful merge.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub close_source_branch: Option<bool>,

    /// The merge strategy to use.
    /// Possible values: `merge_commit`, `squash`, `fast_forward`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_strategy: Option<String>,
}
