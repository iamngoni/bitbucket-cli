//
//  bitbucket-cli
//  config/mod.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # Configuration Module
//!
//! This module provides comprehensive configuration management for the Bitbucket CLI.
//! It handles loading, saving, and accessing configuration settings from TOML files
//! stored in platform-specific directories.
//!
//! ## Overview
//!
//! The configuration system is organized into three main components:
//!
//! - **Core Configuration**: General CLI settings like editor, pager, browser preferences
//! - **Host Configuration**: Per-host settings for different Bitbucket instances
//! - **Aliases**: Custom command shortcuts defined by the user
//!
//! ## Configuration File Location
//!
//! Configuration files are stored in platform-specific directories:
//!
//! - **Linux**: `~/.config/bb/config.toml`
//! - **macOS**: `~/Library/Application Support/bb/config.toml`
//! - **Windows**: `C:\Users\<User>\AppData\Roaming\bb\config.toml`
//!
//! ## Example Configuration File
//!
//! ```toml
//! [core]
//! editor = "vim"
//! pager = "less"
//! browser = "firefox"
//! git_protocol = "ssh"
//! prompt = "enabled"
//!
//! [hosts.bitbucket.org]
//! host = "bitbucket.org"
//! user = "myusername"
//! default_workspace = "myworkspace"
//! api_version = "2.0"
//!
//! [aliases]
//! co = "pr checkout"
//! pv = "pr view"
//! ```
//!
//! ## Usage
//!
//! ```rust,no_run
//! use bitbucket_cli::config::Config;
//!
//! // Load configuration from default location
//! let config = Config::load()?;
//!
//! // Access core settings
//! if let Some(editor) = config.get("editor") {
//!     println!("Using editor: {}", editor);
//! }
//!
//! // Modify and save configuration
//! let mut config = Config::load()?;
//! config.set("git_protocol".to_string(), "ssh".to_string());
//! config.save()?;
//! ```
//!
//! ## Submodules
//!
//! - [`file`]: Low-level configuration file I/O operations
//! - [`hosts`]: Host-specific configuration and utilities

mod file;
mod hosts;

pub use file::*;
pub use hosts::*;

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};

/// Global configuration container for the Bitbucket CLI.
///
/// This struct represents the complete configuration state, including core settings,
/// host-specific configurations, and user-defined command aliases. It is serialized
/// to and from TOML format for persistent storage.
///
/// # Fields
///
/// * `core` - Core CLI configuration options (editor, pager, git protocol, etc.)
/// * `hosts` - Map of hostname to host-specific configuration
/// * `aliases` - Map of alias name to command expansion
///
/// # Examples
///
/// ## Creating a Default Configuration
///
/// ```rust
/// use bitbucket_cli::config::Config;
///
/// let config = Config::default();
/// assert_eq!(config.core.git_protocol, "https");
/// ```
///
/// ## Loading from Disk
///
/// ```rust,no_run
/// use bitbucket_cli::config::Config;
///
/// let config = Config::load()?;
/// println!("Git protocol: {}", config.core.git_protocol);
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Notes
///
/// - All fields use `#[serde(default)]` to ensure graceful handling of missing keys
/// - The configuration file is created automatically on first save if it doesn't exist
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Config {
    /// Core CLI configuration options.
    ///
    /// Contains general settings that apply across all operations,
    /// such as preferred text editor, pager, and git protocol.
    #[serde(default)]
    pub core: CoreConfig,

    /// Host-specific configuration map.
    ///
    /// Keys are hostnames (e.g., "bitbucket.org"), values are the
    /// corresponding [`HostConfig`] settings for that host.
    #[serde(default)]
    pub hosts: HashMap<String, HostConfig>,

    /// Command aliases map.
    ///
    /// Keys are alias names, values are the command expansions.
    /// For example: `{"co": "pr checkout"}` allows `bb co` to expand to `bb pr checkout`.
    #[serde(default)]
    pub aliases: HashMap<String, String>,
}

/// Core configuration options for the Bitbucket CLI.
///
/// This struct contains general settings that affect CLI behavior across
/// all commands and operations. These settings control external tool
/// preferences and protocol choices.
///
/// # Fields
///
/// * `editor` - Preferred text editor for composing messages
/// * `pager` - Pager program for viewing long output
/// * `browser` - Web browser for opening URLs
/// * `git_protocol` - Git protocol preference ("https" or "ssh")
/// * `prompt` - Interactive prompt behavior ("enabled" or "disabled")
///
/// # Default Values
///
/// | Field | Default |
/// |-------|---------|
/// | `editor` | `None` (uses `$EDITOR` or system default) |
/// | `pager` | `None` (uses `$PAGER` or system default) |
/// | `browser` | `None` (uses system default) |
/// | `git_protocol` | `"https"` |
/// | `prompt` | `"enabled"` |
///
/// # Examples
///
/// ```rust
/// use bitbucket_cli::config::CoreConfig;
///
/// let core = CoreConfig::default();
/// assert_eq!(core.git_protocol, "https");
/// assert_eq!(core.prompt, "enabled");
/// assert!(core.editor.is_none());
/// ```
///
/// # Notes
///
/// - When `editor`, `pager`, or `browser` are `None`, the CLI will attempt
///   to use environment variables (`$EDITOR`, `$PAGER`, `$BROWSER`) or
///   platform-specific defaults
/// - The `git_protocol` affects how repository URLs are constructed when
///   cloning or adding remotes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    /// Preferred text editor for composing commit messages, PR descriptions, etc.
    ///
    /// If `None`, falls back to `$EDITOR` environment variable or system default.
    ///
    /// # Examples
    ///
    /// Common values: `"vim"`, `"nano"`, `"code --wait"`, `"subl -w"`
    #[serde(default)]
    pub editor: Option<String>,

    /// Pager program for displaying long output.
    ///
    /// If `None`, falls back to `$PAGER` environment variable or system default.
    ///
    /// # Examples
    ///
    /// Common values: `"less"`, `"more"`, `"bat"`
    #[serde(default)]
    pub pager: Option<String>,

    /// Web browser for opening URLs.
    ///
    /// If `None`, uses the system default browser via platform-specific mechanisms.
    ///
    /// # Examples
    ///
    /// Common values: `"firefox"`, `"google-chrome"`, `"open"` (macOS)
    #[serde(default)]
    pub browser: Option<String>,

    /// Git protocol preference for repository operations.
    ///
    /// Determines the URL scheme used when cloning repositories or adding remotes.
    ///
    /// # Valid Values
    ///
    /// - `"https"` - Use HTTPS URLs (default, works through firewalls)
    /// - `"ssh"` - Use SSH URLs (requires SSH key setup)
    ///
    /// # Examples
    ///
    /// With `"https"`: `https://bitbucket.org/workspace/repo.git`
    /// With `"ssh"`: `git@bitbucket.org:workspace/repo.git`
    #[serde(default = "default_git_protocol")]
    pub git_protocol: String,

    /// Interactive prompt behavior setting.
    ///
    /// Controls whether the CLI prompts for user input in interactive scenarios.
    ///
    /// # Valid Values
    ///
    /// - `"enabled"` - Show interactive prompts (default)
    /// - `"disabled"` - Suppress prompts, use defaults or fail
    ///
    /// # Notes
    ///
    /// Setting to `"disabled"` is useful for scripting and CI/CD environments.
    #[serde(default = "default_prompt")]
    pub prompt: String,
}

/// Returns the default git protocol value.
///
/// # Returns
///
/// Returns `"https"` as the default protocol, which is the most universally
/// compatible option that works through firewalls and doesn't require SSH key setup.
///
/// # Notes
///
/// This function is used as the serde default for [`CoreConfig::git_protocol`].
fn default_git_protocol() -> String {
    "https".to_string()
}

/// Returns the default prompt setting value.
///
/// # Returns
///
/// Returns `"enabled"` to enable interactive prompts by default, providing
/// a user-friendly experience for interactive terminal usage.
///
/// # Notes
///
/// This function is used as the serde default for [`CoreConfig::prompt`].
fn default_prompt() -> String {
    "enabled".to_string()
}

impl Default for CoreConfig {
    /// Creates a new `CoreConfig` with default values.
    ///
    /// # Returns
    ///
    /// A `CoreConfig` instance with:
    /// - `editor`: `None`
    /// - `pager`: `None`
    /// - `browser`: `None`
    /// - `git_protocol`: `"https"`
    /// - `prompt`: `"enabled"`
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bitbucket_cli::config::CoreConfig;
    ///
    /// let config = CoreConfig::default();
    /// assert_eq!(config.git_protocol, "https");
    /// ```
    fn default() -> Self {
        Self {
            editor: None,
            pager: None,
            browser: None,
            git_protocol: default_git_protocol(),
            prompt: default_prompt(),
        }
    }
}

/// Host-specific configuration for a Bitbucket instance.
///
/// This struct contains settings specific to a particular Bitbucket host,
/// whether it's Bitbucket Cloud (bitbucket.org) or a self-hosted Bitbucket
/// Server/Data Center instance.
///
/// # Fields
///
/// * `host` - The hostname of the Bitbucket instance
/// * `user` - The authenticated username for this host
/// * `default_workspace` - Default workspace for Bitbucket Cloud operations
/// * `default_project` - Default project for Bitbucket Server/DC operations
/// * `api_version` - API version to use (e.g., "2.0" for Cloud)
///
/// # Cloud vs Server/Data Center
///
/// | Field | Cloud | Server/DC |
/// |-------|-------|-----------|
/// | `default_workspace` | Used | Not applicable |
/// | `default_project` | Not applicable | Used |
/// | `api_version` | Typically "2.0" | Typically "1.0" |
///
/// # Examples
///
/// ## Bitbucket Cloud Configuration
///
/// ```rust
/// use bitbucket_cli::config::HostConfig;
///
/// let cloud_config = HostConfig {
///     host: "bitbucket.org".to_string(),
///     user: Some("myusername".to_string()),
///     default_workspace: Some("myworkspace".to_string()),
///     default_project: None,
///     api_version: Some("2.0".to_string()),
/// };
/// ```
///
/// ## Bitbucket Server Configuration
///
/// ```rust
/// use bitbucket_cli::config::HostConfig;
///
/// let server_config = HostConfig {
///     host: "bitbucket.mycompany.com".to_string(),
///     user: Some("john.doe".to_string()),
///     default_workspace: None,
///     default_project: Some("MYPROJ".to_string()),
///     api_version: Some("1.0".to_string()),
/// };
/// ```
///
/// # Notes
///
/// - The `host` field should be the normalized hostname without protocol prefix
/// - Use [`normalize_host`] to ensure consistent hostname formatting
/// - Authentication credentials (tokens) are stored separately in the keyring
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HostConfig {
    /// The hostname of the Bitbucket instance.
    ///
    /// Should be the bare hostname without protocol (e.g., "bitbucket.org"
    /// not "https://bitbucket.org").
    #[serde(default)]
    pub host: String,

    /// The authenticated username for this host.
    ///
    /// This is the Bitbucket username, not email address. For Bitbucket Cloud,
    /// this corresponds to the username shown in account settings.
    #[serde(default)]
    pub user: Option<String>,

    /// Default workspace for Bitbucket Cloud operations.
    ///
    /// When specified, commands that require a workspace will use this value
    /// unless explicitly overridden. Only applicable to Bitbucket Cloud.
    ///
    /// # Notes
    ///
    /// Workspaces are a Bitbucket Cloud concept. For Server/DC, use `default_project`.
    #[serde(default)]
    pub default_workspace: Option<String>,

    /// Default project for Bitbucket Server/Data Center operations.
    ///
    /// When specified, commands that require a project will use this value
    /// unless explicitly overridden. Only applicable to Bitbucket Server/DC.
    ///
    /// # Notes
    ///
    /// Projects are typically uppercase keys like "PROJ" or "MYTEAM".
    #[serde(default)]
    pub default_project: Option<String>,

    /// API version to use for this host.
    ///
    /// Determines which version of the Bitbucket API to use for requests.
    ///
    /// # Common Values
    ///
    /// - `"2.0"` - Bitbucket Cloud REST API v2
    /// - `"1.0"` - Bitbucket Server/DC REST API v1
    #[serde(default)]
    pub api_version: Option<String>,
}

impl Config {
    /// Loads configuration from the default location.
    ///
    /// Reads and parses the configuration file from the platform-specific
    /// configuration directory. If the file doesn't exist, returns a default
    /// configuration.
    ///
    /// # Returns
    ///
    /// - `Ok(Config)` - The loaded or default configuration
    /// - `Err` - If the file exists but cannot be read or parsed
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The configuration file exists but cannot be read (permissions, I/O error)
    /// - The configuration file contains invalid TOML syntax
    /// - The TOML structure doesn't match the expected schema
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::config::Config;
    ///
    /// let config = Config::load()?;
    /// println!("Git protocol: {}", config.core.git_protocol);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Notes
    ///
    /// - A missing configuration file is not an error; defaults are used
    /// - The configuration path is determined by [`Config::config_path`]
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            Ok(toml::from_str(&content)?)
        } else {
            Ok(Self::default())
        }
    }

    /// Saves the configuration to the default location.
    ///
    /// Serializes the configuration to TOML format and writes it to the
    /// platform-specific configuration directory. Creates the directory
    /// structure if it doesn't exist.
    ///
    /// # Returns
    ///
    /// - `Ok(())` - The configuration was saved successfully
    /// - `Err` - If the configuration could not be saved
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The parent directory cannot be created
    /// - The file cannot be written (permissions, disk full, etc.)
    /// - TOML serialization fails (should not happen with valid data)
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::config::Config;
    ///
    /// let mut config = Config::load()?;
    /// config.set("git_protocol", "ssh".to_string());
    /// config.save()?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Notes
    ///
    /// - Uses pretty TOML formatting for human readability
    /// - Overwrites the existing configuration file completely
    /// - Parent directories are created with default permissions
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Returns the path to the configuration file.
    ///
    /// Determines the platform-specific configuration directory and returns
    /// the full path to the `config.toml` file.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)` - The path to the configuration file
    /// - `Err` - If the configuration directory cannot be determined
    ///
    /// # Platform-Specific Paths
    ///
    /// | Platform | Path |
    /// |----------|------|
    /// | Linux | `~/.config/bb/config.toml` |
    /// | macOS | `~/Library/Application Support/bb/config.toml` |
    /// | Windows | `C:\Users\<User>\AppData\Roaming\bb\config.toml` |
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::config::Config;
    ///
    /// let path = Config::config_path()?;
    /// println!("Config file: {}", path.display());
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Notes
    ///
    /// - The file may not exist; this only returns where it would be
    /// - Uses the `directories` crate for cross-platform path resolution
    pub fn config_path() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("", "", "bb")
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;
        Ok(dirs.config_dir().join("config.toml"))
    }

    /// Returns the path to the data directory.
    ///
    /// Determines the platform-specific data directory for storing
    /// application data such as caches, logs, or other persistent data.
    ///
    /// # Returns
    ///
    /// - `Ok(PathBuf)` - The path to the data directory
    /// - `Err` - If the data directory cannot be determined
    ///
    /// # Platform-Specific Paths
    ///
    /// | Platform | Path |
    /// |----------|------|
    /// | Linux | `~/.local/share/bb/` |
    /// | macOS | `~/Library/Application Support/bb/` |
    /// | Windows | `C:\Users\<User>\AppData\Roaming\bb\` |
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::config::Config;
    ///
    /// let data_dir = Config::data_dir()?;
    /// let cache_file = data_dir.join("cache.json");
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Notes
    ///
    /// - The directory may not exist; callers should create it if needed
    /// - Uses the `directories` crate for cross-platform path resolution
    pub fn data_dir() -> Result<PathBuf> {
        let dirs = ProjectDirs::from("", "", "bb")
            .ok_or_else(|| anyhow::anyhow!("Could not determine data directory"))?;
        Ok(dirs.data_dir().to_path_buf())
    }

    /// Returns the configuration for a specific host.
    ///
    /// Looks up host-specific settings in the configuration's hosts map.
    ///
    /// # Parameters
    ///
    /// * `host` - The hostname to look up (e.g., "bitbucket.org")
    ///
    /// # Returns
    ///
    /// - `Some(&HostConfig)` - The configuration for the specified host
    /// - `None` - If no configuration exists for the host
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bitbucket_cli::config::Config;
    ///
    /// let config = Config::default();
    ///
    /// if let Some(host_config) = config.host_config("bitbucket.org") {
    ///     println!("User: {:?}", host_config.user);
    /// } else {
    ///     println!("No configuration for this host");
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - Host lookup is case-sensitive; use [`normalize_host`] for consistent keys
    /// - Returns a reference to avoid cloning large configuration structures
    pub fn host_config(&self, host: &str) -> Option<&HostConfig> {
        self.hosts.get(host)
    }

    /// Gets a core configuration value by key.
    ///
    /// Retrieves the value of a core configuration setting using a string key.
    /// This provides a dynamic way to access configuration values without
    /// knowing the specific field at compile time.
    ///
    /// # Parameters
    ///
    /// * `key` - The configuration key to retrieve
    ///
    /// # Supported Keys
    ///
    /// | Key | Field | Type |
    /// |-----|-------|------|
    /// | `"editor"` | `core.editor` | Optional |
    /// | `"pager"` | `core.pager` | Optional |
    /// | `"browser"` | `core.browser` | Optional |
    /// | `"git_protocol"` | `core.git_protocol` | Required |
    /// | `"prompt"` | `core.prompt` | Required |
    ///
    /// # Returns
    ///
    /// - `Some(String)` - The configuration value if the key exists and has a value
    /// - `None` - If the key is unknown or the optional value is not set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bitbucket_cli::config::Config;
    ///
    /// let config = Config::default();
    ///
    /// // Required fields always return Some
    /// assert_eq!(config.get("git_protocol"), Some("https".to_string()));
    ///
    /// // Optional fields return None if not set
    /// assert_eq!(config.get("editor"), None);
    ///
    /// // Unknown keys return None
    /// assert_eq!(config.get("unknown_key"), None);
    /// ```
    pub fn get(&self, key: &str) -> Option<String> {
        match key {
            "editor" => self.core.editor.clone(),
            "pager" => self.core.pager.clone(),
            "browser" => self.core.browser.clone(),
            "git_protocol" => Some(self.core.git_protocol.clone()),
            "prompt" => Some(self.core.prompt.clone()),
            _ => None,
        }
    }

    /// Sets a core configuration value by key.
    ///
    /// Updates a core configuration setting using a string key. This provides
    /// a dynamic way to modify configuration values, useful for implementing
    /// `bb config set` commands.
    ///
    /// # Parameters
    ///
    /// * `key` - The configuration key to set
    /// * `value` - The new value for the configuration key
    ///
    /// # Supported Keys
    ///
    /// | Key | Field | Notes |
    /// |-----|-------|-------|
    /// | `"editor"` | `core.editor` | Sets to `Some(value)` |
    /// | `"pager"` | `core.pager` | Sets to `Some(value)` |
    /// | `"browser"` | `core.browser` | Sets to `Some(value)` |
    /// | `"git_protocol"` | `core.git_protocol` | Should be "https" or "ssh" |
    /// | `"prompt"` | `core.prompt` | Should be "enabled" or "disabled" |
    ///
    /// # Returns
    ///
    /// - `true` - The value was set successfully
    /// - `false` - The key is unknown and no value was set
    ///
    /// # Examples
    ///
    /// ```rust
    /// use bitbucket_cli::config::Config;
    ///
    /// let mut config = Config::default();
    ///
    /// // Set a known key
    /// assert!(config.set("editor", "vim".to_string()));
    /// assert_eq!(config.get("editor"), Some("vim".to_string()));
    ///
    /// // Unknown keys return false
    /// assert!(!config.set("unknown_key", "value".to_string()));
    /// ```
    ///
    /// # Notes
    ///
    /// - This method does not validate the value; callers should ensure
    ///   appropriate values are provided (e.g., "https" or "ssh" for git_protocol)
    /// - Changes are only persisted when [`Config::save`] is called
    pub fn set(&mut self, key: &str, value: String) -> bool {
        match key {
            "editor" => {
                self.core.editor = Some(value);
                true
            }
            "pager" => {
                self.core.pager = Some(value);
                true
            }
            "browser" => {
                self.core.browser = Some(value);
                true
            }
            "git_protocol" => {
                self.core.git_protocol = value;
                true
            }
            "prompt" => {
                self.core.prompt = value;
                true
            }
            _ => false,
        }
    }
}
