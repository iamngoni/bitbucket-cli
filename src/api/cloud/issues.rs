//
//  bitbucket-cli
//  api/cloud/issues.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Cloud issues API types and data structures.
//!
//! This module provides types for managing issues in Bitbucket Cloud repositories.
//! Issues provide a way to track bugs, feature requests, and tasks associated
//! with a repository.
//!
//! # Overview
//!
//! Bitbucket issues are a lightweight issue tracking system built into
//! repositories. They support states, priorities, kinds, and can be
//! assigned to users.
//!
//! # Issue States
//!
//! * `new` - Newly created, not yet triaged
//! * `open` - Acknowledged and being worked on
//! * `resolved` - Fixed or addressed
//! * `on hold` - Temporarily paused
//! * `invalid` - Not a valid issue
//! * `duplicate` - Duplicate of another issue
//! * `wontfix` - Will not be fixed
//! * `closed` - Completed
//!
//! # Issue Priorities
//!
//! * `trivial` - Minor cosmetic issues
//! * `minor` - Small bugs, minor features
//! * `major` - Important bugs, significant features
//! * `critical` - Severe bugs requiring immediate attention
//! * `blocker` - Blocks other work, must be fixed ASAP
//!
//! # Issue Kinds
//!
//! * `bug` - Something isn't working correctly
//! * `enhancement` - Request for new functionality
//! * `proposal` - Idea for discussion
//! * `task` - Work item to be completed
//!
//! # Example
//!
//! ```rust,no_run
//! use bitbucket_cli::api::cloud::issues::{
//!     CreateIssueRequest, IssueContentInput
//! };
//!
//! // Create a bug report
//! let request = CreateIssueRequest {
//!     title: "Login button not responding".to_string(),
//!     content: Some(IssueContentInput {
//!         raw: "When clicking the login button, nothing happens.\n\nSteps to reproduce:\n1. Go to login page\n2. Enter credentials\n3. Click Login".to_string(),
//!     }),
//!     state: Some("open".to_string()),
//!     priority: Some("major".to_string()),
//!     kind: Some("bug".to_string()),
//!     assignee: None,
//! };
//! ```
//!
//! # Notes
//!
//! - Issues must be enabled for the repository
//! - Issue numbers are sequential within each repository
//! - Content supports Markdown formatting

use serde::{Deserialize, Serialize};

use crate::api::common::UserRef;

/// Represents an issue in a Bitbucket Cloud repository.
///
/// Issues track bugs, features, and tasks. Each issue has a unique ID
/// within its repository and includes metadata like state, priority,
/// and assignment.
///
/// # Fields
///
/// * `id` - Unique numeric identifier within the repository
/// * `title` - Short summary of the issue
/// * `content` - Optional detailed description with formatting
/// * `state` - Current state of the issue
/// * `priority` - Priority level of the issue
/// * `kind` - Type/category of the issue
/// * `reporter` - The user who created the issue
/// * `assignee` - Optional user assigned to work on the issue
/// * `created_on` - ISO 8601 timestamp of creation
/// * `updated_on` - ISO 8601 timestamp of last update
/// * `votes` - Number of votes/upvotes on the issue
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::issues::Issue;
///
/// fn display_issue(issue: &Issue) {
///     println!("#{}: {} [{}]", issue.id, issue.title, issue.state);
///     println!("  Priority: {} | Kind: {}", issue.priority, issue.kind);
///     println!("  Reporter: {}", issue.reporter.display_name);
///     if let Some(ref assignee) = issue.assignee {
///         println!("  Assignee: {}", assignee.display_name);
///     }
///     println!("  Votes: {}", issue.votes);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    /// Unique numeric identifier within the repository.
    pub id: u64,

    /// Short summary of the issue (typically a single line).
    pub title: String,

    /// Optional detailed description with multiple format representations.
    #[serde(default)]
    pub content: Option<IssueContent>,

    /// Current state of the issue.
    /// Possible values: `new`, `open`, `resolved`, `on hold`,
    /// `invalid`, `duplicate`, `wontfix`, `closed`.
    pub state: String,

    /// Priority level of the issue.
    /// Possible values: `trivial`, `minor`, `major`, `critical`, `blocker`.
    pub priority: String,

    /// Type/category of the issue.
    /// Possible values: `bug`, `enhancement`, `proposal`, `task`.
    pub kind: String,

    /// The user who created this issue.
    pub reporter: UserRef,

    /// The user assigned to work on this issue (if any).
    #[serde(default)]
    pub assignee: Option<UserRef>,

    /// ISO 8601 timestamp indicating when the issue was created.
    pub created_on: String,

    /// ISO 8601 timestamp indicating when the issue was last updated.
    pub updated_on: String,

    /// Number of votes/upvotes on this issue.
    #[serde(default)]
    pub votes: u32,
}

/// Content of an issue in multiple format representations.
///
/// Bitbucket provides issue content in multiple formats to support
/// different rendering contexts.
///
/// # Fields
///
/// * `raw` - The raw content as entered by the user (typically Markdown)
/// * `markup` - The markup type used (e.g., `markdown`)
/// * `html` - Pre-rendered HTML version of the content
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::issues::IssueContent;
///
/// fn display_content(content: &IssueContent) {
///     // Use HTML if available, otherwise raw
///     if let Some(ref html) = content.html {
///         println!("HTML: {}", html);
///     } else {
///         println!("Raw: {}", content.raw);
///     }
/// }
/// ```
///
/// # Notes
///
/// - The `raw` field contains the original user input
/// - The `html` field may be pre-rendered for display
/// - The `markup` field indicates the format of `raw`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueContent {
    /// The raw content as entered by the user.
    /// Typically in Markdown format.
    pub raw: String,

    /// The markup type used for the raw content (e.g., `markdown`).
    #[serde(default)]
    pub markup: Option<String>,

    /// Pre-rendered HTML version of the content.
    #[serde(default)]
    pub html: Option<String>,
}

/// Request payload for creating a new issue.
///
/// Used when making POST requests to create an issue in a repository.
/// Only the `title` field is required.
///
/// # Fields
///
/// * `title` - Required title for the issue
/// * `content` - Optional detailed description
/// * `state` - Optional initial state (defaults to `new`)
/// * `priority` - Optional priority level (defaults to `major`)
/// * `kind` - Optional issue kind (defaults to `bug`)
/// * `assignee` - Optional user to assign the issue to
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::issues::{
///     CreateIssueRequest, IssueContentInput, UserUuid
/// };
///
/// // Create a minimal issue
/// let minimal = CreateIssueRequest {
///     title: "Quick bug report".to_string(),
///     content: None,
///     state: None,
///     priority: None,
///     kind: None,
///     assignee: None,
/// };
///
/// // Create a fully specified issue
/// let detailed = CreateIssueRequest {
///     title: "Implement user authentication".to_string(),
///     content: Some(IssueContentInput {
///         raw: "## Requirements\n\n- OAuth2 support\n- Remember me option".to_string(),
///     }),
///     state: Some("open".to_string()),
///     priority: Some("major".to_string()),
///     kind: Some("enhancement".to_string()),
///     assignee: Some(UserUuid {
///         uuid: "{user-uuid}".to_string(),
///     }),
/// };
/// ```
///
/// # Notes
///
/// - Issues must be enabled for the repository
/// - Default values are applied for optional fields
/// - Content supports Markdown formatting
#[derive(Debug, Clone, Serialize)]
pub struct CreateIssueRequest {
    /// The title of the issue (required).
    pub title: String,

    /// Optional detailed description of the issue.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<IssueContentInput>,

    /// Optional initial state. Defaults to `new` if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// Optional priority level. Defaults to `major` if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<String>,

    /// Optional issue kind. Defaults to `bug` if not specified.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,

    /// Optional user to assign the issue to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<UserUuid>,
}

/// Input format for issue content when creating or updating issues.
///
/// Provides the raw content that will be processed by Bitbucket.
///
/// # Fields
///
/// * `raw` - The content in Markdown format
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::issues::IssueContentInput;
///
/// let content = IssueContentInput {
///     raw: "## Bug Description\n\nSteps to reproduce:\n1. Step one\n2. Step two".to_string(),
/// };
/// ```
///
/// # Notes
///
/// - Content is interpreted as Markdown
/// - Bitbucket will render the content to HTML for display
#[derive(Debug, Clone, Serialize)]
pub struct IssueContentInput {
    /// The raw content in Markdown format.
    pub raw: String,
}

/// User identification by UUID for issue assignment.
///
/// Used when specifying users in issue creation or update requests.
///
/// # Fields
///
/// * `uuid` - The unique identifier for the user
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::issues::UserUuid;
///
/// let assignee = UserUuid {
///     uuid: "{123e4567-e89b-12d3-a456-426614174000}".to_string(),
/// };
/// ```
///
/// # Notes
///
/// - UUIDs include curly braces as returned by the Bitbucket API
/// - The user must be a member of the repository's workspace
#[derive(Debug, Clone, Serialize)]
pub struct UserUuid {
    /// The unique identifier for the user (includes curly braces).
    pub uuid: String,
}

/// Represents a comment on an issue.
///
/// Comments allow users to discuss and provide updates on issues.
///
/// # Fields
///
/// * `id` - Unique numeric identifier for the comment
/// * `content` - The comment content in multiple formats
/// * `user` - The user who wrote the comment
/// * `created_on` - ISO 8601 timestamp of creation
/// * `updated_on` - ISO 8601 timestamp of last update
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::issues::IssueComment;
///
/// fn display_comment(comment: &IssueComment) {
///     println!("{} commented:", comment.user.display_name);
///     println!("  {}", comment.content.raw);
///     println!("  ({})", comment.created_on);
/// }
/// ```
///
/// # Notes
///
/// - Comments are ordered chronologically
/// - Comment content supports Markdown formatting
/// - Comments can be edited after creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueComment {
    /// Unique numeric identifier for this comment.
    pub id: u64,

    /// The content of the comment in multiple format representations.
    pub content: IssueContent,

    /// The user who wrote this comment.
    pub user: UserRef,

    /// ISO 8601 timestamp indicating when the comment was created.
    pub created_on: String,

    /// ISO 8601 timestamp indicating when the comment was last updated.
    pub updated_on: String,
}
