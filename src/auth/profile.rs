//
//  bitbucket-cli
//  auth/profile.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # Authentication Profile Management Module
//!
//! This module provides multi-account and multi-host authentication profile
//! management for the Bitbucket CLI. It enables users to configure and switch
//! between different Bitbucket accounts (Cloud and Server/DC).
//!
//! ## Use Cases
//!
//! - Managing multiple Bitbucket Cloud accounts (personal, work)
//! - Connecting to multiple Bitbucket Server/DC instances
//! - Switching between accounts without re-authenticating
//! - Configuring a default profile for quick access
//!
//! ## Profile Structure
//!
//! Each profile contains:
//! - **Name**: Unique identifier for the profile (e.g., "work", "personal")
//! - **Host**: The Bitbucket instance URL
//! - **Username**: Optional username for display/identification
//! - **Auth Type**: The authentication method used
//! - **Default**: Whether this is the default profile
//!
//! ## Storage
//!
//! Profiles are stored in the CLI configuration file (typically at
//! `~/.config/bitbucket-cli/config.toml` or platform equivalent). Actual
//! credentials are stored separately in the system keyring.
//!
//! ## Example
//!
//! ```rust,no_run
//! use bitbucket_cli::auth::{Profile, ProfileManager, AuthType};
//! use std::path::Path;
//!
//! fn setup_profiles() -> anyhow::Result<()> {
//!     let mut manager = ProfileManager::new();
//!
//!     // Add a work profile
//!     manager.add(Profile {
//!         name: "work".to_string(),
//!         host: "https://git.company.com".to_string(),
//!         username: Some("jsmith".to_string()),
//!         auth_type: AuthType::Pat,
//!         default: true,
//!     });
//!
//!     // Add a personal profile
//!     manager.add(Profile {
//!         name: "personal".to_string(),
//!         host: "https://bitbucket.org".to_string(),
//!         username: Some("john_smith".to_string()),
//!         auth_type: AuthType::OAuth,
//!         default: false,
//!     });
//!
//!     // Save profiles
//!     manager.save(Path::new("~/.config/bitbucket-cli/config.toml"))?;
//!
//!     Ok(())
//! }
//! ```

use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Represents an authentication profile for a Bitbucket account.
///
/// A profile encapsulates all metadata needed to identify and authenticate
/// with a specific Bitbucket instance. The actual credentials are stored
/// separately in the system keyring, referenced by the profile's host.
///
/// # Fields
///
/// - `name`: Unique identifier for this profile (e.g., "work", "personal").
/// - `host`: The Bitbucket instance URL (Cloud or Server/DC).
/// - `username`: Optional username for display purposes.
/// - `auth_type`: The authentication method used with this profile.
/// - `default`: Whether this profile should be used by default.
///
/// # Serialization
///
/// This struct implements `Serialize` and `Deserialize` for persistent storage
/// in configuration files (TOML, JSON, YAML).
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::auth::{Profile, AuthType};
///
/// // Create a profile for Bitbucket Cloud
/// let cloud_profile = Profile {
///     name: "personal".to_string(),
///     host: "https://bitbucket.org".to_string(),
///     username: Some("user@example.com".to_string()),
///     auth_type: AuthType::OAuth,
///     default: true,
/// };
///
/// // Create a profile for Bitbucket Server
/// let server_profile = Profile {
///     name: "work".to_string(),
///     host: "https://bitbucket.company.com".to_string(),
///     username: Some("jsmith".to_string()),
///     auth_type: AuthType::Pat,
///     default: false,
/// };
/// ```
///
/// # Notes
///
/// - Profile names must be unique within a [`ProfileManager`].
/// - The `host` field is used to look up credentials in the keyring.
/// - Only one profile should have `default: true` at a time.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    /// Unique identifier for this profile.
    ///
    /// This name is used in CLI commands to select a specific profile,
    /// e.g., `bitbucket --profile work pr list`.
    pub name: String,

    /// The Bitbucket instance URL.
    ///
    /// For Bitbucket Cloud, use `https://bitbucket.org`.
    /// For Server/DC, use the instance's base URL (e.g., `https://git.company.com`).
    pub host: String,

    /// Optional username associated with this profile.
    ///
    /// This is primarily for display purposes and user identification.
    /// For OAuth authentication, this may be the Atlassian account email.
    pub username: Option<String>,

    /// The authentication method used with this profile.
    ///
    /// Determines how credentials are validated and applied to requests.
    pub auth_type: AuthType,

    /// Whether this is the default profile.
    ///
    /// The default profile is used when no specific profile is specified
    /// in CLI commands.
    #[serde(default)]
    pub default: bool,
}

/// Specifies the authentication method for a profile.
///
/// Each variant corresponds to a different authentication mechanism
/// supported by Bitbucket Cloud or Server/Data Center.
///
/// # Variants
///
/// - `OAuth`: OAuth 2.0 token authentication (recommended for Cloud).
/// - `AppPassword`: App password authentication (deprecated for Cloud).
/// - `Pat`: Personal Access Token (recommended for Server/DC).
/// - `Basic`: HTTP Basic authentication (legacy).
///
/// # Serialization
///
/// Variants are serialized as lowercase strings for configuration files:
/// - `OAuth` -> `"oauth"`
/// - `AppPassword` -> `"apppassword"`
/// - `Pat` -> `"pat"`
/// - `Basic` -> `"basic"`
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::auth::AuthType;
///
/// let auth_type = AuthType::OAuth;
///
/// // In configuration file (TOML):
/// // auth_type = "oauth"
/// ```
///
/// # Notes
///
/// - Choose `OAuth` for Bitbucket Cloud when possible.
/// - Choose `Pat` for Bitbucket Server/Data Center.
/// - `AppPassword` is deprecated; migrate to OAuth for Cloud.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AuthType {
    /// OAuth 2.0 authentication for Bitbucket Cloud.
    ///
    /// Provides secure, scoped access with refresh capability.
    OAuth,

    /// App password authentication for Bitbucket Cloud (deprecated).
    ///
    /// Use OAuth instead for better security and granular permissions.
    AppPassword,

    /// Personal Access Token for Bitbucket Server/Data Center.
    ///
    /// The recommended authentication method for self-hosted instances.
    Pat,

    /// HTTP Basic authentication with username and password.
    ///
    /// Legacy method; use OAuth or PAT instead when available.
    Basic,
}

/// Manages multiple authentication profiles.
///
/// This struct provides methods to add, remove, and query authentication
/// profiles, as well as persist them to and load them from configuration files.
///
/// # Features
///
/// - Store multiple profiles for different accounts/hosts
/// - Set and retrieve a default profile
/// - Look up profiles by name or host
/// - Persist profiles to configuration files
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::auth::{Profile, ProfileManager, AuthType};
///
/// let mut manager = ProfileManager::new();
///
/// // Add profiles
/// manager.add(Profile {
///     name: "cloud".to_string(),
///     host: "https://bitbucket.org".to_string(),
///     username: None,
///     auth_type: AuthType::OAuth,
///     default: true,
/// });
///
/// // Retrieve the default profile
/// if let Some(profile) = manager.default_profile() {
///     println!("Using profile: {}", profile.name);
/// }
///
/// // List all profiles
/// for profile in manager.list() {
///     println!("Profile: {} -> {}", profile.name, profile.host);
/// }
/// ```
///
/// # Notes
///
/// - Profile names must be unique; adding a profile with an existing name
///   will replace the previous profile.
/// - When a default profile is removed, no new default is automatically set.
pub struct ProfileManager {
    /// Map of profile names to profiles.
    profiles: HashMap<String, Profile>,

    /// Name of the current default profile, if set.
    default_profile: Option<String>,
}

impl Default for ProfileManager {
    /// Creates an empty [`ProfileManager`] with no profiles.
    ///
    /// This is equivalent to calling [`ProfileManager::new()`].
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileManager {
    /// Creates a new empty profile manager.
    ///
    /// # Returns
    ///
    /// Returns a new [`ProfileManager`] with no profiles configured.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::auth::ProfileManager;
    ///
    /// let manager = ProfileManager::new();
    /// assert!(manager.list().count() == 0);
    /// ```
    pub fn new() -> Self {
        Self {
            profiles: HashMap::new(),
            default_profile: None,
        }
    }

    /// Loads profiles from a configuration file.
    ///
    /// Reads the profile configuration from the specified path and creates
    /// a [`ProfileManager`] populated with the stored profiles.
    ///
    /// # Parameters
    ///
    /// - `config_path`: Path to the configuration file.
    ///
    /// # Returns
    ///
    /// Returns `Ok(ProfileManager)` with loaded profiles on success.
    /// Returns `Err` if the file cannot be read or parsed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    /// use bitbucket_cli::auth::ProfileManager;
    ///
    /// fn load_config() -> anyhow::Result<ProfileManager> {
    ///     let config_path = Path::new("~/.config/bitbucket-cli/config.toml");
    ///     let manager = ProfileManager::load(config_path)?;
    ///
    ///     println!("Loaded {} profiles", manager.list().count());
    ///     Ok(manager)
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - **Not yet implemented**: Currently returns an empty manager.
    /// - Will support TOML format configuration files.
    /// - Missing files should return an empty manager, not an error.
    pub fn load(_config_path: &std::path::Path) -> Result<Self> {
        // TODO: Implement profile loading
        Ok(Self::new())
    }

    /// Saves profiles to a configuration file.
    ///
    /// Persists all profiles to the specified configuration file path.
    /// The file is created if it doesn't exist, or overwritten if it does.
    ///
    /// # Parameters
    ///
    /// - `config_path`: Path where the configuration should be saved.
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success.
    /// Returns `Err` if the file cannot be written.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::path::Path;
    /// use bitbucket_cli::auth::{Profile, ProfileManager, AuthType};
    ///
    /// fn save_config() -> anyhow::Result<()> {
    ///     let mut manager = ProfileManager::new();
    ///     manager.add(Profile {
    ///         name: "default".to_string(),
    ///         host: "https://bitbucket.org".to_string(),
    ///         username: None,
    ///         auth_type: AuthType::OAuth,
    ///         default: true,
    ///     });
    ///
    ///     manager.save(Path::new("~/.config/bitbucket-cli/config.toml"))?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - **Not yet implemented**: Currently a no-op.
    /// - Will create parent directories if they don't exist.
    /// - File permissions should be restricted (0600 on Unix).
    pub fn save(&self, _config_path: &std::path::Path) -> Result<()> {
        // TODO: Implement profile saving
        Ok(())
    }

    /// Adds a profile to the manager.
    ///
    /// If a profile with the same name already exists, it will be replaced.
    /// If the new profile has `default: true`, it becomes the default profile.
    ///
    /// # Parameters
    ///
    /// - `profile`: The [`Profile`] to add.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::auth::{Profile, ProfileManager, AuthType};
    ///
    /// let mut manager = ProfileManager::new();
    ///
    /// manager.add(Profile {
    ///     name: "work".to_string(),
    ///     host: "https://git.company.com".to_string(),
    ///     username: Some("jsmith".to_string()),
    ///     auth_type: AuthType::Pat,
    ///     default: true,
    /// });
    ///
    /// assert!(manager.get("work").is_some());
    /// assert!(manager.default_profile().is_some());
    /// ```
    ///
    /// # Notes
    ///
    /// - Adding a profile with `default: true` changes the default profile.
    /// - The previous default profile's `default` field is not updated.
    pub fn add(&mut self, profile: Profile) {
        if profile.default {
            self.default_profile = Some(profile.name.clone());
        }
        self.profiles.insert(profile.name.clone(), profile);
    }

    /// Removes a profile from the manager.
    ///
    /// If the removed profile was the default, the default is cleared
    /// (no automatic fallback to another profile).
    ///
    /// # Parameters
    ///
    /// - `name`: The name of the profile to remove.
    ///
    /// # Returns
    ///
    /// Returns `Some(Profile)` if the profile was found and removed.
    /// Returns `None` if no profile with that name exists.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::auth::{Profile, ProfileManager, AuthType};
    ///
    /// let mut manager = ProfileManager::new();
    /// manager.add(Profile {
    ///     name: "temp".to_string(),
    ///     host: "https://bitbucket.org".to_string(),
    ///     username: None,
    ///     auth_type: AuthType::OAuth,
    ///     default: false,
    /// });
    ///
    /// let removed = manager.remove("temp");
    /// assert!(removed.is_some());
    /// assert!(manager.get("temp").is_none());
    /// ```
    ///
    /// # Notes
    ///
    /// - Removing the default profile clears the default setting.
    /// - This does not delete associated credentials from the keyring.
    pub fn remove(&mut self, name: &str) -> Option<Profile> {
        if self.default_profile.as_deref() == Some(name) {
            self.default_profile = None;
        }
        self.profiles.remove(name)
    }

    /// Retrieves a profile by name.
    ///
    /// # Parameters
    ///
    /// - `name`: The name of the profile to retrieve.
    ///
    /// # Returns
    ///
    /// Returns `Some(&Profile)` if found, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::auth::{Profile, ProfileManager, AuthType};
    ///
    /// let mut manager = ProfileManager::new();
    /// manager.add(Profile {
    ///     name: "work".to_string(),
    ///     host: "https://git.company.com".to_string(),
    ///     username: None,
    ///     auth_type: AuthType::Pat,
    ///     default: false,
    /// });
    ///
    /// if let Some(profile) = manager.get("work") {
    ///     println!("Host: {}", profile.host);
    /// }
    /// ```
    pub fn get(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }

    /// Returns the default profile, if one is set.
    ///
    /// The default profile is used when no specific profile is specified
    /// in CLI commands.
    ///
    /// # Returns
    ///
    /// Returns `Some(&Profile)` if a default is set, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::auth::{Profile, ProfileManager, AuthType};
    ///
    /// let mut manager = ProfileManager::new();
    /// manager.add(Profile {
    ///     name: "default".to_string(),
    ///     host: "https://bitbucket.org".to_string(),
    ///     username: None,
    ///     auth_type: AuthType::OAuth,
    ///     default: true,
    /// });
    ///
    /// let default = manager.default_profile().unwrap();
    /// assert_eq!(default.name, "default");
    /// ```
    ///
    /// # Notes
    ///
    /// - Returns `None` if no profiles exist or no default is set.
    /// - A profile marked with `default: true` when added becomes the default.
    pub fn default_profile(&self) -> Option<&Profile> {
        self.default_profile
            .as_ref()
            .and_then(|name| self.profiles.get(name))
    }

    /// Sets the default profile by name.
    ///
    /// # Parameters
    ///
    /// - `name`: The name of the profile to set as default.
    ///
    /// # Returns
    ///
    /// Returns `true` if the profile exists and was set as default.
    /// Returns `false` if no profile with that name exists.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::auth::{Profile, ProfileManager, AuthType};
    ///
    /// let mut manager = ProfileManager::new();
    /// manager.add(Profile {
    ///     name: "work".to_string(),
    ///     host: "https://git.company.com".to_string(),
    ///     username: None,
    ///     auth_type: AuthType::Pat,
    ///     default: false,
    /// });
    ///
    /// assert!(manager.set_default("work"));
    /// assert!(!manager.set_default("nonexistent"));
    /// ```
    ///
    /// # Notes
    ///
    /// - This does not modify the profile's `default` field.
    /// - Only one profile can be the default at a time.
    pub fn set_default(&mut self, name: &str) -> bool {
        if self.profiles.contains_key(name) {
            self.default_profile = Some(name.to_string());
            true
        } else {
            false
        }
    }

    /// Returns an iterator over all profiles.
    ///
    /// # Returns
    ///
    /// Returns an iterator yielding references to all stored profiles.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::auth::{Profile, ProfileManager, AuthType};
    ///
    /// let mut manager = ProfileManager::new();
    /// manager.add(Profile {
    ///     name: "profile1".to_string(),
    ///     host: "https://host1.com".to_string(),
    ///     username: None,
    ///     auth_type: AuthType::OAuth,
    ///     default: false,
    /// });
    /// manager.add(Profile {
    ///     name: "profile2".to_string(),
    ///     host: "https://host2.com".to_string(),
    ///     username: None,
    ///     auth_type: AuthType::Pat,
    ///     default: false,
    /// });
    ///
    /// for profile in manager.list() {
    ///     println!("{}: {}", profile.name, profile.host);
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - The iteration order is not guaranteed (HashMap-based storage).
    /// - Returns an empty iterator if no profiles are configured.
    pub fn list(&self) -> impl Iterator<Item = &Profile> {
        self.profiles.values()
    }

    /// Finds a profile by its host URL.
    ///
    /// Useful when you know the host but not the profile name.
    ///
    /// # Parameters
    ///
    /// - `host`: The Bitbucket host URL to search for.
    ///
    /// # Returns
    ///
    /// Returns `Some(&Profile)` if a profile with matching host is found.
    /// Returns `None` if no profile matches.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::auth::{Profile, ProfileManager, AuthType};
    ///
    /// let mut manager = ProfileManager::new();
    /// manager.add(Profile {
    ///     name: "cloud".to_string(),
    ///     host: "https://bitbucket.org".to_string(),
    ///     username: None,
    ///     auth_type: AuthType::OAuth,
    ///     default: false,
    /// });
    ///
    /// if let Some(profile) = manager.for_host("https://bitbucket.org") {
    ///     println!("Found profile: {}", profile.name);
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - Performs exact string matching on the host field.
    /// - If multiple profiles share the same host, returns the first match.
    /// - Host comparison is case-sensitive.
    pub fn for_host(&self, host: &str) -> Option<&Profile> {
        self.profiles.values().find(|p| p.host == host)
    }
}
