//
//  bitbucket-cli
//  config/hosts.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # Host Configuration Module
//!
//! This module provides utilities for working with Bitbucket host configurations,
//! including constants for known hosts and functions for host detection and normalization.
//!
//! ## Overview
//!
//! The Bitbucket CLI supports both Bitbucket Cloud (bitbucket.org) and self-hosted
//! Bitbucket Server/Data Center instances. This module provides:
//!
//! - Constants for well-known Bitbucket Cloud hostnames
//! - Functions to detect whether a host is Bitbucket Cloud
//! - Default configuration generators for Bitbucket Cloud
//! - Host URL normalization utilities
//!
//! ## Bitbucket Cloud vs Server/Data Center
//!
//! | Feature | Cloud | Server/DC |
//! |---------|-------|-----------|
//! | Hostname | `bitbucket.org` | Custom (e.g., `bitbucket.company.com`) |
//! | API Base | `api.bitbucket.org` | Same as web hostname |
//! | API Version | 2.0 | 1.0 |
//! | Organization | Workspaces | Projects |
//!
//! ## Usage
//!
//! ```rust
//! use bitbucket_cli::config::{is_cloud_host, normalize_host, cloud_host_config};
//!
//! // Check if a host is Bitbucket Cloud
//! assert!(is_cloud_host("bitbucket.org"));
//! assert!(!is_cloud_host("bitbucket.company.com"));
//!
//! // Normalize host URLs for consistent storage
//! let host = normalize_host("https://BITBUCKET.ORG/");
//! assert_eq!(host, "bitbucket.org");
//!
//! // Get default Cloud configuration
//! let config = cloud_host_config();
//! assert_eq!(config.api_version, Some("2.0".to_string()));
//! ```

use super::HostConfig;

/// The primary hostname for Bitbucket Cloud.
///
/// This is the main web interface hostname where users interact with
/// repositories, pull requests, and other Bitbucket features.
///
/// # Value
///
/// `"bitbucket.org"`
///
/// # Examples
///
/// ```rust
/// use bitbucket_cli::config::BITBUCKET_CLOUD;
///
/// let url = format!("https://{}/workspace/repo", BITBUCKET_CLOUD);
/// assert_eq!(url, "https://bitbucket.org/workspace/repo");
/// ```
///
/// # Notes
///
/// - Always use this constant instead of hardcoding the hostname
/// - For API calls, use [`BITBUCKET_API`] instead
pub const BITBUCKET_CLOUD: &str = "bitbucket.org";

/// The API hostname for Bitbucket Cloud.
///
/// This is the dedicated API endpoint hostname used for REST API calls
/// to Bitbucket Cloud. It's separate from the web interface hostname.
///
/// # Value
///
/// `"api.bitbucket.org"`
///
/// # Examples
///
/// ```rust
/// use bitbucket_cli::config::BITBUCKET_API;
///
/// let api_url = format!("https://{}/2.0/repositories", BITBUCKET_API);
/// assert_eq!(api_url, "https://api.bitbucket.org/2.0/repositories");
/// ```
///
/// # Notes
///
/// - REST API 2.0 endpoints are served from this hostname
/// - The API version (2.0) is part of the URL path, not the hostname
/// - For Bitbucket Server/DC, the API uses the same hostname as the web interface
pub const BITBUCKET_API: &str = "api.bitbucket.org";

/// Checks if a hostname corresponds to Bitbucket Cloud.
///
/// Determines whether the given hostname is one of the known Bitbucket Cloud
/// hostnames. This is used to apply Cloud-specific behavior and defaults.
///
/// # Parameters
///
/// * `host` - The hostname to check (should be normalized, without protocol)
///
/// # Returns
///
/// - `true` - The host is Bitbucket Cloud (bitbucket.org or api.bitbucket.org)
/// - `false` - The host is not Bitbucket Cloud (likely Server/DC)
///
/// # Examples
///
/// ## Identifying Cloud Hosts
///
/// ```rust
/// use bitbucket_cli::config::is_cloud_host;
///
/// // Cloud hosts
/// assert!(is_cloud_host("bitbucket.org"));
/// assert!(is_cloud_host("api.bitbucket.org"));
///
/// // Server/DC hosts
/// assert!(!is_cloud_host("bitbucket.company.com"));
/// assert!(!is_cloud_host("git.internal.net"));
/// ```
///
/// ## Conditional Logic Based on Host Type
///
/// ```rust
/// use bitbucket_cli::config::{is_cloud_host, cloud_host_config, HostConfig};
///
/// fn get_host_config(host: &str) -> HostConfig {
///     if is_cloud_host(host) {
///         cloud_host_config()
///     } else {
///         // Server/DC defaults
///         HostConfig {
///             host: host.to_string(),
///             api_version: Some("1.0".to_string()),
///             ..Default::default()
///         }
///     }
/// }
/// ```
///
/// # Notes
///
/// - Comparison is exact; use [`normalize_host`] first for user input
/// - Only matches the two known Cloud hostnames; subdomains return `false`
/// - Case-sensitive; input should be lowercase (use [`normalize_host`])
pub fn is_cloud_host(host: &str) -> bool {
    host == BITBUCKET_CLOUD || host == BITBUCKET_API
}

/// Creates a default configuration for Bitbucket Cloud.
///
/// Returns a [`HostConfig`] pre-populated with appropriate defaults
/// for Bitbucket Cloud, including the correct API version and hostname.
///
/// # Returns
///
/// A `HostConfig` with:
/// - `host`: `"bitbucket.org"`
/// - `user`: `None` (to be filled during authentication)
/// - `default_workspace`: `None` (to be set by user)
/// - `default_project`: `None` (not used for Cloud)
/// - `api_version`: `Some("2.0")`
///
/// # Examples
///
/// ## Getting Cloud Defaults
///
/// ```rust
/// use bitbucket_cli::config::cloud_host_config;
///
/// let config = cloud_host_config();
/// assert_eq!(config.host, "bitbucket.org");
/// assert_eq!(config.api_version, Some("2.0".to_string()));
/// assert!(config.user.is_none());
/// ```
///
/// ## Customizing Cloud Configuration
///
/// ```rust
/// use bitbucket_cli::config::cloud_host_config;
///
/// let mut config = cloud_host_config();
/// config.user = Some("myusername".to_string());
/// config.default_workspace = Some("myworkspace".to_string());
/// ```
///
/// # Notes
///
/// - User and workspace should be set after authentication
/// - The API version is pre-set to "2.0" for the REST API v2
/// - `default_project` is not applicable for Cloud (use `default_workspace`)
pub fn cloud_host_config() -> HostConfig {
    HostConfig {
        host: BITBUCKET_CLOUD.to_string(),
        user: None,
        default_workspace: None,
        default_project: None,
        api_version: Some("2.0".to_string()),
    }
}

/// Normalizes a host URL to a consistent hostname format.
///
/// Processes a host string by removing protocol prefixes, trailing slashes,
/// and converting to lowercase. This ensures consistent hostname storage
/// and comparison regardless of how the user inputs the URL.
///
/// # Parameters
///
/// * `host` - The host string to normalize (may include protocol and path)
///
/// # Returns
///
/// A normalized hostname string with:
/// - Protocol prefix removed (`https://`, `http://`)
/// - Trailing slash removed
/// - Converted to lowercase
/// - Leading/trailing whitespace trimmed
///
/// # Examples
///
/// ## Basic Normalization
///
/// ```rust
/// use bitbucket_cli::config::normalize_host;
///
/// // Remove HTTPS prefix
/// assert_eq!(normalize_host("https://bitbucket.org"), "bitbucket.org");
///
/// // Remove HTTP prefix
/// assert_eq!(normalize_host("http://bitbucket.org"), "bitbucket.org");
///
/// // Remove trailing slash
/// assert_eq!(normalize_host("bitbucket.org/"), "bitbucket.org");
///
/// // Lowercase conversion
/// assert_eq!(normalize_host("BITBUCKET.ORG"), "bitbucket.org");
///
/// // Combined
/// assert_eq!(normalize_host("  HTTPS://BitBucket.Org/  "), "bitbucket.org");
/// ```
///
/// ## Use with Host Configuration
///
/// ```rust
/// use bitbucket_cli::config::{normalize_host, is_cloud_host};
///
/// // User might input URL in various formats
/// let user_input = "https://BITBUCKET.ORG/";
/// let normalized = normalize_host(user_input);
///
/// // Normalized hostname works correctly with other functions
/// assert!(is_cloud_host(&normalized));
/// ```
///
/// ## Storing Normalized Hostnames
///
/// ```rust
/// use std::collections::HashMap;
/// use bitbucket_cli::config::{normalize_host, HostConfig};
///
/// let mut hosts: HashMap<String, HostConfig> = HashMap::new();
///
/// // Always normalize before storing
/// let host = normalize_host("https://git.company.com/");
/// hosts.insert(host.clone(), HostConfig {
///     host: host,
///     ..Default::default()
/// });
/// ```
///
/// # Notes
///
/// - Does not validate that the result is a valid hostname
/// - Does not remove paths beyond the trailing slash (e.g., "/path" would remain)
/// - Does not handle port numbers specially (they remain in the output)
/// - The order of operations is: trim -> remove https -> remove http -> remove trailing slash -> lowercase
pub fn normalize_host(host: &str) -> String {
    let host = host.trim();
    let host = host.strip_prefix("https://").unwrap_or(host);
    let host = host.strip_prefix("http://").unwrap_or(host);
    let host = host.strip_suffix('/').unwrap_or(host);
    host.to_lowercase()
}
