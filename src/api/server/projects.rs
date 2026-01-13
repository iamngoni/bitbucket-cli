//
//  bitbucket-cli
//  api/server/projects.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # Bitbucket Server/DC Project API
//!
//! This module provides types and structures for working with projects
//! in Bitbucket Server/Data Center. Projects are containers that group
//! related repositories together and provide shared access control.
//!
//! ## Project Structure
//!
//! In Bitbucket Server/DC, projects:
//! - Have a unique key (short uppercase identifier used in URLs)
//! - Contain one or more repositories
//! - Define access permissions for all contained repositories
//! - Can be public or private
//! - Have a type indicating their purpose (NORMAL or PERSONAL)
//!
//! ## API Endpoints
//!
//! Project operations use these endpoints:
//! ```text
//! GET/POST /rest/api/1.0/projects
//! GET/PUT/DELETE /rest/api/1.0/projects/{projectKey}
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use bitbucket_cli::api::server::projects::{Project, CreateProjectRequest};
//!
//! // Create a new project
//! let request = CreateProjectRequest {
//!     key: "MYPROJ".to_string(),
//!     name: "My Project".to_string(),
//!     description: Some("A sample project for demonstration".to_string()),
//!     is_public: Some(false),
//! };
//!
//! let project: Project = client.create_project(request).await?;
//! println!("Created project: {} ({})", project.name, project.key);
//! ```
//!
//! ## Notes
//!
//! - Project keys must be unique across the Bitbucket instance
//! - Personal projects have type "PERSONAL" and are prefixed with "~"
//! - Normal projects have type "NORMAL"

use serde::{Deserialize, Serialize};

/// Represents a project in Bitbucket Server/Data Center.
///
/// A project is a container for repositories that provides shared access
/// control and organization. Projects are identified by a unique key that
/// appears in URLs and API paths.
///
/// # Fields
///
/// * `id` - Unique numeric identifier for the project
/// * `key` - Short uppercase key used in URLs (e.g., "PROJ", "DEV")
/// * `name` - Human-readable display name of the project
/// * `description` - Optional description of the project's purpose
/// * `is_public` - Whether the project is publicly accessible
/// * `project_type` - Type of project (NORMAL or PERSONAL)
/// * `links` - Collection of URLs for accessing the project
///
/// # Example
///
/// ```rust,ignore
/// let project: Project = client.get_project("PROJ").await?;
///
/// println!("Project: {} ({})", project.name, project.key);
/// println!("Type: {}", project.project_type);
/// println!("Public: {}", project.is_public);
///
/// if let Some(desc) = &project.description {
///     println!("Description: {}", desc);
/// }
///
/// // Get web UI URL
/// if let Some(link) = project.links.self_link.first() {
///     println!("URL: {}", link.href);
/// }
/// ```
///
/// # Notes
///
/// - Personal projects have keys starting with "~" (e.g., "~jsmith")
/// - The `project_type` distinguishes normal and personal projects
/// - Project keys are case-insensitive but typically uppercase
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    /// Unique numeric identifier assigned by Bitbucket Server.
    pub id: u64,

    /// Short uppercase key used in URLs and API paths.
    /// Must be unique across the Bitbucket instance.
    /// Example: "PROJ", "DEV", "INFRA", or "~username" for personal projects.
    pub key: String,

    /// Human-readable display name of the project.
    pub name: String,

    /// Optional description explaining the project's purpose.
    /// May be `None` if no description was provided.
    #[serde(default)]
    pub description: Option<String>,

    /// Whether the project is publicly accessible.
    /// Public projects can be viewed by unauthenticated users.
    /// Defaults to `false` if not specified in the API response.
    #[serde(rename = "public")]
    #[serde(default)]
    pub is_public: bool,

    /// Type of the project.
    /// Values: "NORMAL" for regular projects, "PERSONAL" for user projects.
    #[serde(rename = "type")]
    pub project_type: String,

    /// Collection of links for accessing the project.
    pub links: ProjectLinks,
}

/// Collection of links associated with a project.
///
/// Contains URLs for accessing the project in the web UI. The Bitbucket Server
/// API returns links as arrays to maintain consistency with other resource types.
///
/// # Fields
///
/// * `self_link` - List of self-referential web UI URLs
///
/// # Example
///
/// ```rust,ignore
/// if let Some(link) = project.links.self_link.first() {
///     println!("View project at: {}", link.href);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLinks {
    /// Self-referential links to the project in the web UI.
    #[serde(default, rename = "self")]
    pub self_link: Vec<SelfLink>,
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
/// if let Some(link) = project.links.self_link.first() {
///     println!("View in browser: {}", link.href);
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfLink {
    /// The full URL to view this resource in the web UI.
    pub href: String,
}

/// Request body for creating a new project.
///
/// Used when sending a POST request to create a new project. The `key` and
/// `name` fields are required; other fields have sensible defaults.
///
/// # Fields
///
/// * `key` - Required unique key for the project (uppercase, no spaces)
/// * `name` - Required human-readable name for the project
/// * `description` - Optional description of the project
/// * `is_public` - Whether the project should be public (defaults to false)
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::api::server::projects::CreateProjectRequest;
///
/// // Minimal request
/// let request = CreateProjectRequest {
///     key: "NEWPROJ".to_string(),
///     name: "New Project".to_string(),
///     description: None,
///     is_public: None,
/// };
///
/// // Full request
/// let request = CreateProjectRequest {
///     key: "DEVTEAM".to_string(),
///     name: "Development Team".to_string(),
///     description: Some("Repositories for the development team".to_string()),
///     is_public: Some(false),
/// };
/// ```
///
/// # Notes
///
/// - Project keys must be unique and typically use uppercase letters
/// - Keys cannot contain spaces or special characters
/// - The key cannot be changed after project creation
/// - Optional fields use `skip_serializing_if` to omit `None` values from JSON
#[derive(Debug, Clone, Serialize)]
pub struct CreateProjectRequest {
    /// Unique key for the new project.
    /// Should be uppercase with no spaces (e.g., "PROJ", "DEVTEAM").
    /// This cannot be changed after creation.
    pub key: String,

    /// Human-readable name for the project.
    pub name: String,

    /// Optional description of the project's purpose.
    /// Omitted from the request if `None`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Whether the project should be publicly accessible.
    /// Defaults to `false` on the server if not specified.
    #[serde(rename = "public")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
}

/// Request body for updating an existing project.
///
/// Used when sending a PUT request to update a project's properties.
/// All fields are optional; only specified fields will be updated.
/// The project key cannot be changed.
///
/// # Fields
///
/// * `name` - New name for the project
/// * `description` - New description for the project
/// * `is_public` - New public visibility setting
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::api::server::projects::UpdateProjectRequest;
///
/// // Update only the description
/// let request = UpdateProjectRequest {
///     name: None,
///     description: Some("Updated project description".to_string()),
///     is_public: None,
/// };
///
/// // Update multiple fields
/// let request = UpdateProjectRequest {
///     name: Some("Renamed Project".to_string()),
///     description: Some("New description".to_string()),
///     is_public: Some(true),
/// };
/// ```
///
/// # Notes
///
/// - Only non-`None` fields will be updated
/// - The project key cannot be changed via this request
/// - Optional fields use `skip_serializing_if` to omit `None` values from JSON
/// - Setting `description` to `Some("")` will clear the description
#[derive(Debug, Clone, Serialize)]
pub struct UpdateProjectRequest {
    /// New name for the project.
    /// If `None`, the name remains unchanged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// New description for the project.
    /// If `None`, the description remains unchanged.
    /// Use `Some("")` to clear the description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// New public visibility setting.
    /// If `None`, the visibility remains unchanged.
    #[serde(rename = "public")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_public: Option<bool>,
}
