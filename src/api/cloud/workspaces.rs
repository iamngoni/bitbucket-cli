//
//  bitbucket-cli
//  api/cloud/workspaces.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Cloud workspace API types and data structures.
//!
//! This module provides types for managing Bitbucket Cloud workspaces,
//! which are the top-level organizational unit containing repositories,
//! projects, and members.
//!
//! # Overview
//!
//! Workspaces in Bitbucket Cloud represent a shared space where teams
//! collaborate on repositories. Each workspace has a unique slug that
//! forms part of repository URLs.
//!
//! # Workspace Hierarchy
//!
//! ```text
//! Workspace
//! ├── Projects (optional grouping)
//! │   └── Repositories
//! ├── Repositories (not in projects)
//! └── Members (users with access)
//! ```
//!
//! # Example
//!
//! ```rust,no_run
//! use bitbucket_cli::api::cloud::workspaces::Workspace;
//!
//! fn list_workspaces(workspaces: &[Workspace]) {
//!     for ws in workspaces {
//!         println!("{} ({})", ws.name, ws.slug);
//!         if ws.is_private {
//!             println!("  [Private workspace]");
//!         }
//!     }
//! }
//! ```
//!
//! # Notes
//!
//! - Workspace slugs are globally unique and URL-safe
//! - A user can be a member of multiple workspaces
//! - Workspaces can be private (invitation-only) or public

use serde::{Deserialize, Serialize};

/// Represents a Bitbucket Cloud workspace.
///
/// A workspace is the top-level organizational unit in Bitbucket Cloud.
/// It contains repositories, projects, and members, and provides a shared
/// context for team collaboration.
///
/// # Fields
///
/// * `uuid` - Unique identifier for the workspace
/// * `slug` - URL-safe identifier used in API paths and URLs
/// * `name` - Human-readable name of the workspace
/// * `is_private` - Whether the workspace is private (invitation-only)
/// * `created_on` - ISO 8601 timestamp of creation
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::workspaces::Workspace;
///
/// fn display_workspace(ws: &Workspace) {
///     println!("Workspace: {}", ws.name);
///     println!("  Slug: {}", ws.slug);
///     println!("  UUID: {}", ws.uuid);
///     println!("  Private: {}", ws.is_private);
///     println!("  Created: {}", ws.created_on);
/// }
/// ```
///
/// # Notes
///
/// - The slug is used in URLs: `bitbucket.org/{slug}/{repo}`
/// - Workspace names can contain spaces; slugs cannot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    /// Unique identifier for the workspace (includes curly braces).
    pub uuid: String,

    /// URL-safe identifier used in API paths and repository URLs.
    pub slug: String,

    /// Human-readable name of the workspace.
    pub name: String,

    /// Whether the workspace is private.
    /// Private workspaces are invitation-only.
    #[serde(default)]
    pub is_private: bool,

    /// ISO 8601 timestamp indicating when the workspace was created.
    pub created_on: String,
}

/// Represents a member of a workspace.
///
/// Links a user to a workspace, indicating their membership.
/// Additional permission details may be available through other endpoints.
///
/// # Fields
///
/// * `user` - The user who is a member
/// * `workspace` - The workspace they belong to
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::workspaces::WorkspaceMember;
///
/// fn list_members(members: &[WorkspaceMember]) {
///     for member in members {
///         println!("{} is a member of {}",
///             member.user.display_name,
///             member.workspace.name
///         );
///     }
/// }
/// ```
///
/// # Notes
///
/// - Member permissions are managed separately
/// - A user's access level depends on their role in the workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMember {
    /// The user who is a member of the workspace.
    pub user: WorkspaceUser,

    /// The workspace the user belongs to.
    pub workspace: WorkspaceRef,
}

/// User information in the context of a workspace.
///
/// Provides detailed user information as it relates to workspace membership.
///
/// # Fields
///
/// * `uuid` - Unique identifier for the user
/// * `display_name` - Human-readable name for display
/// * `nickname` - Optional username/handle
/// * `account_id` - Optional Atlassian account identifier
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::api::cloud::workspaces::WorkspaceUser;
///
/// fn display_user(user: &WorkspaceUser) {
///     println!("User: {}", user.display_name);
///     if let Some(ref nick) = user.nickname {
///         println!("  Nickname: {}", nick);
///     }
///     if let Some(ref account_id) = user.account_id {
///         println!("  Account ID: {}", account_id);
///     }
/// }
/// ```
///
/// # Notes
///
/// - The `account_id` is the Atlassian account identifier
/// - The `nickname` is typically the username chosen by the user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceUser {
    /// Unique identifier for the user (includes curly braces).
    pub uuid: String,

    /// Human-readable name for display purposes.
    pub display_name: String,

    /// Optional username or nickname chosen by the user.
    #[serde(default)]
    pub nickname: Option<String>,

    /// Optional Atlassian account identifier for cross-product integration.
    #[serde(default)]
    pub account_id: Option<String>,
}

/// A lightweight reference to a workspace.
///
/// Used when embedding workspace information within other resources
/// without including full workspace details.
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
/// use bitbucket_cli::api::cloud::workspaces::WorkspaceRef;
///
/// let workspace_ref = WorkspaceRef {
///     uuid: "{workspace-uuid}".to_string(),
///     slug: "my-team".to_string(),
///     name: "My Team".to_string(),
/// };
/// ```
///
/// # Notes
///
/// - This is a subset of the full [`Workspace`] struct
/// - Used for embedding in other API responses to reduce payload size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRef {
    /// Unique identifier for the workspace.
    pub uuid: String,

    /// URL-safe identifier for the workspace.
    pub slug: String,

    /// Human-readable name of the workspace.
    pub name: String,
}
