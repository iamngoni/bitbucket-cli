//
//  bitbucket-cli
//  alias/mod.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Command Alias System
//!
//! This module provides a command alias system that allows users to create
//! shortcuts for commonly used commands. Aliases can be either direct command
//! expansions or shell commands that are executed through the system shell.
//!
//! # Overview
//!
//! The alias system consists of three main components:
//!
//! - [`AliasConfig`]: Stores the collection of defined aliases
//! - [`AliasEntry`]: Represents a single alias with its expansion and type
//! - [`AliasManager`]: Handles CRUD operations and persistence for aliases
//!
//! # Features
//!
//! - Create command aliases that expand to longer commands
//! - Create shell aliases that execute through the system shell
//! - Import/export aliases for sharing configurations
//! - Automatic circular reference detection
//! - Reserved command protection
//!
//! # Example
//!
//! ```rust,no_run
//! use std::path::PathBuf;
//! use bitbucket_cli::alias::AliasManager;
//!
//! // Create an alias manager
//! let mut manager = AliasManager::new(PathBuf::from("~/.config/bb/aliases.toml"))?;
//!
//! // Create a command alias
//! manager.set("prs", "pr list --state open", false)?;
//!
//! // Create a shell alias
//! manager.set("branch", "git rev-parse --abbrev-ref HEAD", true)?;
//!
//! // Expand an alias
//! if let Some(expanded) = manager.expand("prs") {
//!     println!("Expanded to: {}", expanded.expansion);
//! }
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! # Configuration Format
//!
//! Aliases are stored in TOML format:
//!
//! ```toml
//! [aliases.prs]
//! expansion = "pr list --state open"
//! shell = false
//!
//! [aliases.branch]
//! expansion = "git rev-parse --abbrev-ref HEAD"
//! shell = true
//! ```

use std::collections::HashMap;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Configuration container for all defined command aliases.
///
/// This struct holds a collection of [`AliasEntry`] instances indexed by their
/// alias names. It supports serialization to and from TOML format for persistent
/// storage.
///
/// # Fields
///
/// * `aliases` - A map of alias names to their corresponding [`AliasEntry`] definitions
///
/// # Example
///
/// ```rust
/// use std::collections::HashMap;
/// use bitbucket_cli::alias::{AliasConfig, AliasEntry};
///
/// let mut config = AliasConfig::default();
/// config.aliases.insert(
///     "prs".to_string(),
///     AliasEntry::command("pr list --state open".to_string())
/// );
/// ```
///
/// # Serialization
///
/// The configuration is serialized to TOML format where each alias becomes
/// a subsection under `[aliases]`:
///
/// ```toml
/// [aliases.prs]
/// expansion = "pr list --state open"
/// shell = false
/// ```
///
/// # Notes
///
/// - Implements `Default` for easy initialization of empty configurations
/// - The `aliases` field defaults to an empty HashMap if not present in config
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AliasConfig {
    /// Command aliases mapping alias names to their expansions.
    ///
    /// Each key is the short alias name (e.g., "prs") and the value
    /// is an [`AliasEntry`] containing the expansion and execution type.
    #[serde(default)]
    pub aliases: HashMap<String, AliasEntry>,
}

/// Represents a single command alias definition.
///
/// An alias entry contains the command expansion string and a flag indicating
/// whether the command should be executed through the system shell.
///
/// # Fields
///
/// * `expansion` - The full command string that the alias expands to
/// * `shell` - Whether this alias should be executed as a shell command
///
/// # Alias Types
///
/// There are two types of aliases:
///
/// 1. **Command aliases** (`shell = false`): The expansion is parsed into
///    arguments and passed to the CLI. Additional arguments are appended.
///
/// 2. **Shell aliases** (`shell = true`): The expansion is passed directly
///    to the system shell for execution. Additional arguments are ignored.
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::alias::AliasEntry;
///
/// // Command alias - expands to CLI arguments
/// let cmd_alias = AliasEntry::command("pr list --state open".to_string());
/// assert!(!cmd_alias.shell);
///
/// // Shell alias - runs through system shell
/// let shell_alias = AliasEntry::shell_command("git status".to_string());
/// assert!(shell_alias.shell);
/// ```
///
/// # Notes
///
/// - The `shell` field defaults to `false` during deserialization
/// - Command aliases allow argument appending, shell aliases do not
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasEntry {
    /// The command expansion string.
    ///
    /// For command aliases, this is parsed into individual arguments.
    /// For shell aliases, this is passed as-is to the shell.
    pub expansion: String,

    /// Whether this alias should be executed through the system shell.
    ///
    /// When `true`, the expansion is passed to the shell with `-c` flag.
    /// When `false`, the expansion is parsed into arguments for the CLI.
    #[serde(default)]
    pub shell: bool,
}

impl AliasEntry {
    /// Creates a new alias entry with the specified expansion and shell flag.
    ///
    /// This is the primary constructor for creating alias entries with full
    /// control over all parameters.
    ///
    /// # Parameters
    ///
    /// * `expansion` - The command string that the alias expands to
    /// * `shell` - Whether to execute through the system shell
    ///
    /// # Returns
    ///
    /// A new [`AliasEntry`] instance with the specified configuration.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::alias::AliasEntry;
    ///
    /// // Create a command alias
    /// let entry = AliasEntry::new("pr list".to_string(), false);
    ///
    /// // Create a shell alias
    /// let shell_entry = AliasEntry::new("echo hello".to_string(), true);
    /// ```
    pub fn new(expansion: String, shell: bool) -> Self {
        Self { expansion, shell }
    }

    /// Creates a command alias (non-shell).
    ///
    /// This is a convenience constructor for creating aliases that are
    /// parsed and executed as CLI commands rather than shell commands.
    ///
    /// # Parameters
    ///
    /// * `expansion` - The CLI command string that the alias expands to
    ///
    /// # Returns
    ///
    /// A new [`AliasEntry`] with `shell` set to `false`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::alias::AliasEntry;
    ///
    /// let alias = AliasEntry::command("pr list --state open".to_string());
    /// assert_eq!(alias.expansion, "pr list --state open");
    /// assert!(!alias.shell);
    /// ```
    ///
    /// # Notes
    ///
    /// Command aliases support argument appending. For example, if the alias
    /// "prs" expands to "pr list --state open", running "prs --author me"
    /// will execute "pr list --state open --author me".
    pub fn command(expansion: String) -> Self {
        Self::new(expansion, false)
    }

    /// Creates a shell alias.
    ///
    /// This is a convenience constructor for creating aliases that are
    /// executed directly through the system shell.
    ///
    /// # Parameters
    ///
    /// * `expansion` - The shell command string to execute
    ///
    /// # Returns
    ///
    /// A new [`AliasEntry`] with `shell` set to `true`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::alias::AliasEntry;
    ///
    /// let alias = AliasEntry::shell_command("git rev-parse --abbrev-ref HEAD".to_string());
    /// assert_eq!(alias.expansion, "git rev-parse --abbrev-ref HEAD");
    /// assert!(alias.shell);
    /// ```
    ///
    /// # Notes
    ///
    /// - Shell aliases do not support argument appending
    /// - The command is passed to the shell with the `-c` flag
    /// - Be cautious with user input to avoid shell injection vulnerabilities
    pub fn shell_command(expansion: String) -> Self {
        Self::new(expansion, true)
    }
}

/// Manages command alias storage, retrieval, and persistence.
///
/// The `AliasManager` provides a complete interface for managing command
/// aliases including CRUD operations, import/export functionality, and
/// automatic configuration persistence.
///
/// # Fields
///
/// * `config` - The current alias configuration (private)
/// * `config_path` - Path to the configuration file (private)
///
/// # Persistence
///
/// All modifications to aliases are automatically persisted to the
/// configuration file in TOML format.
///
/// # Example
///
/// ```rust,no_run
/// use std::path::PathBuf;
/// use bitbucket_cli::alias::AliasManager;
///
/// let mut manager = AliasManager::new(PathBuf::from("~/.config/bb/aliases.toml"))?;
///
/// // Create aliases
/// manager.set("prs", "pr list --state open", false)?;
/// manager.set("branch", "git rev-parse --abbrev-ref HEAD", true)?;
///
/// // List all aliases
/// for (name, entry) in manager.list() {
///     println!("{} -> {}", name, entry.expansion);
/// }
///
/// // Delete an alias
/// manager.delete("prs")?;
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Notes
///
/// - Configuration is loaded lazily from disk on creation
/// - If the config file doesn't exist, an empty configuration is used
/// - Parent directories are created automatically when saving
pub struct AliasManager {
    /// The current alias configuration.
    config: AliasConfig,
    /// Path to the configuration file for persistence.
    config_path: PathBuf,
}

impl AliasManager {
    /// Creates a new alias manager with the specified configuration file path.
    ///
    /// If the configuration file exists, it will be loaded and parsed.
    /// Otherwise, an empty configuration will be initialized.
    ///
    /// # Parameters
    ///
    /// * `config_path` - Path to the TOML configuration file
    ///
    /// # Returns
    ///
    /// * `Ok(AliasManager)` - A new manager instance
    /// * `Err` - If the config file exists but cannot be read or parsed
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    /// use bitbucket_cli::alias::AliasManager;
    ///
    /// // Load existing configuration
    /// let manager = AliasManager::new(PathBuf::from("~/.config/bb/aliases.toml"))?;
    ///
    /// // Start fresh if file doesn't exist
    /// let manager = AliasManager::new(PathBuf::from("/tmp/new-aliases.toml"))?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The config file exists but cannot be read
    /// - The config file contains invalid TOML syntax
    /// - The config file TOML structure doesn't match expected schema
    pub fn new(config_path: PathBuf) -> Result<Self> {
        let config = if config_path.exists() {
            let content = std::fs::read_to_string(&config_path)?;
            toml::from_str(&content)?
        } else {
            AliasConfig::default()
        };

        Ok(Self { config, config_path })
    }

    /// Returns a reference to all defined aliases.
    ///
    /// # Returns
    ///
    /// A reference to the HashMap containing all alias name to entry mappings.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    /// use bitbucket_cli::alias::AliasManager;
    ///
    /// let manager = AliasManager::new(PathBuf::from("aliases.toml"))?;
    ///
    /// for (name, entry) in manager.list() {
    ///     let alias_type = if entry.shell { "shell" } else { "command" };
    ///     println!("{} ({}) -> {}", name, alias_type, entry.expansion);
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn list(&self) -> &HashMap<String, AliasEntry> {
        &self.config.aliases
    }

    /// Retrieves a specific alias by name.
    ///
    /// # Parameters
    ///
    /// * `name` - The alias name to look up
    ///
    /// # Returns
    ///
    /// * `Some(&AliasEntry)` - The alias entry if found
    /// * `None` - If no alias exists with the given name
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    /// use bitbucket_cli::alias::AliasManager;
    ///
    /// let manager = AliasManager::new(PathBuf::from("aliases.toml"))?;
    ///
    /// if let Some(entry) = manager.get("prs") {
    ///     println!("prs expands to: {}", entry.expansion);
    /// } else {
    ///     println!("Alias 'prs' not found");
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn get(&self, name: &str) -> Option<&AliasEntry> {
        self.config.aliases.get(name)
    }

    /// Checks whether an alias with the given name exists.
    ///
    /// # Parameters
    ///
    /// * `name` - The alias name to check
    ///
    /// # Returns
    ///
    /// `true` if an alias with this name exists, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    /// use bitbucket_cli::alias::AliasManager;
    ///
    /// let manager = AliasManager::new(PathBuf::from("aliases.toml"))?;
    ///
    /// if manager.exists("prs") {
    ///     println!("Alias 'prs' is already defined");
    /// } else {
    ///     println!("Alias 'prs' is available");
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn exists(&self, name: &str) -> bool {
        self.config.aliases.contains_key(name)
    }

    /// Creates or updates an alias.
    ///
    /// This method validates the alias name, checks for circular references,
    /// and persists the new alias to the configuration file.
    ///
    /// # Parameters
    ///
    /// * `name` - The alias name (must not be empty, contain whitespace, or be reserved)
    /// * `expansion` - The command string the alias expands to
    /// * `shell` - Whether this should be a shell alias
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the alias was successfully created/updated
    /// * `Err` - If validation fails or the config cannot be saved
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    /// use bitbucket_cli::alias::AliasManager;
    ///
    /// let mut manager = AliasManager::new(PathBuf::from("aliases.toml"))?;
    ///
    /// // Create a command alias
    /// manager.set("prs", "pr list --state open", false)?;
    ///
    /// // Create a shell alias
    /// manager.set("branch", "git rev-parse --abbrev-ref HEAD", true)?;
    ///
    /// // Update an existing alias
    /// manager.set("prs", "pr list --state merged", false)?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The alias name is empty
    /// - The alias name contains whitespace
    /// - The alias name is a reserved command (help, version, alias, etc.)
    /// - The alias would create a circular reference
    /// - The configuration file cannot be written
    ///
    /// # Notes
    ///
    /// Reserved commands that cannot be aliased: `help`, `version`, `alias`,
    /// `extension`, `config`, `completion`
    pub fn set(&mut self, name: &str, expansion: &str, shell: bool) -> Result<()> {
        // Validate alias name
        validate_alias_name(name)?;

        // Check for circular references (simple check)
        if !shell && expansion.starts_with(name) {
            anyhow::bail!("Alias would create a circular reference: {}", name);
        }

        self.config.aliases.insert(
            name.to_string(),
            AliasEntry::new(expansion.to_string(), shell),
        );
        self.save()
    }

    /// Deletes an alias by name.
    ///
    /// If the alias exists, it will be removed and the configuration
    /// will be persisted to disk.
    ///
    /// # Parameters
    ///
    /// * `name` - The alias name to delete
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the alias existed and was deleted
    /// * `Ok(false)` - If no alias with that name existed
    /// * `Err` - If the configuration cannot be saved
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    /// use bitbucket_cli::alias::AliasManager;
    ///
    /// let mut manager = AliasManager::new(PathBuf::from("aliases.toml"))?;
    ///
    /// if manager.delete("prs")? {
    ///     println!("Alias 'prs' deleted");
    /// } else {
    ///     println!("Alias 'prs' not found");
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn delete(&mut self, name: &str) -> Result<bool> {
        if self.config.aliases.remove(name).is_some() {
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Expands an alias name to its full command.
    ///
    /// This method looks up the alias and returns an [`ExpandedAlias`]
    /// containing the expansion and execution type.
    ///
    /// # Parameters
    ///
    /// * `name` - The alias name to expand
    ///
    /// # Returns
    ///
    /// * `Some(ExpandedAlias)` - The expanded alias if found
    /// * `None` - If no alias with that name exists
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    /// use bitbucket_cli::alias::AliasManager;
    ///
    /// let manager = AliasManager::new(PathBuf::from("aliases.toml"))?;
    ///
    /// if let Some(expanded) = manager.expand("prs") {
    ///     if expanded.shell {
    ///         println!("Shell command: {}", expanded.expansion);
    ///     } else {
    ///         let args = expanded.to_args();
    ///         println!("CLI args: {:?}", args);
    ///     }
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn expand(&self, name: &str) -> Option<ExpandedAlias> {
        self.config.aliases.get(name).map(|entry| ExpandedAlias {
            expansion: entry.expansion.clone(),
            shell: entry.shell,
        })
    }

    /// Imports aliases from an external TOML file.
    ///
    /// This method reads aliases from the specified file and merges them
    /// into the current configuration. Existing aliases with the same
    /// names will be overwritten.
    ///
    /// # Parameters
    ///
    /// * `path` - Path to the TOML file to import from
    ///
    /// # Returns
    ///
    /// * `Ok(usize)` - The number of aliases imported
    /// * `Err` - If the file cannot be read/parsed or config cannot be saved
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    /// use bitbucket_cli::alias::AliasManager;
    ///
    /// let mut manager = AliasManager::new(PathBuf::from("aliases.toml"))?;
    ///
    /// let count = manager.import(&PathBuf::from("shared-aliases.toml"))?;
    /// println!("Imported {} aliases", count);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Notes
    ///
    /// - Imported aliases are validated for name conflicts only after import
    /// - Duplicate alias names from the import file overwrite existing aliases
    /// - Changes are persisted immediately after import
    pub fn import(&mut self, path: &PathBuf) -> Result<usize> {
        let content = std::fs::read_to_string(path)?;
        let imported: AliasConfig = toml::from_str(&content)?;

        let count = imported.aliases.len();
        self.config.aliases.extend(imported.aliases);
        self.save()?;

        Ok(count)
    }

    /// Exports all aliases to a TOML file.
    ///
    /// This method writes the current alias configuration to the specified
    /// file in a human-readable TOML format.
    ///
    /// # Parameters
    ///
    /// * `path` - Path to the file to export to
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the export succeeded
    /// * `Err` - If the file cannot be written
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::path::PathBuf;
    /// use bitbucket_cli::alias::AliasManager;
    ///
    /// let manager = AliasManager::new(PathBuf::from("aliases.toml"))?;
    ///
    /// // Export for sharing
    /// manager.export(&PathBuf::from("shared-aliases.toml"))?;
    ///
    /// // Export for backup
    /// manager.export(&PathBuf::from("aliases-backup.toml"))?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Notes
    ///
    /// - The output file is overwritten if it exists
    /// - Parent directories are NOT created automatically
    /// - The file is written in pretty-printed TOML format
    pub fn export(&self, path: &PathBuf) -> Result<()> {
        let content = toml::to_string_pretty(&self.config)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Persists the current configuration to disk.
    ///
    /// This is an internal method called automatically after modifications.
    /// It creates parent directories if they don't exist.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the configuration was saved successfully
    /// * `Err` - If directories cannot be created or file cannot be written
    fn save(&self) -> Result<()> {
        if let Some(parent) = self.config_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(&self.config)?;
        std::fs::write(&self.config_path, content)?;
        Ok(())
    }
}

/// The result of expanding an alias, containing the command and execution type.
///
/// This struct is returned by [`AliasManager::expand`] and provides methods
/// for converting the expansion into executable arguments.
///
/// # Fields
///
/// * `expansion` - The expanded command string
/// * `shell` - Whether this should be executed through the system shell
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::alias::ExpandedAlias;
///
/// // Simulate an expanded command alias
/// let expanded = ExpandedAlias {
///     expansion: "pr list --state open".to_string(),
///     shell: false,
/// };
///
/// let args = expanded.to_args();
/// assert_eq!(args, vec!["pr", "list", "--state", "open"]);
/// ```
///
/// # Notes
///
/// - For shell aliases, `to_args()` returns `["-c", expansion]`
/// - For command aliases, `to_args()` parses the expansion into arguments
#[derive(Debug, Clone)]
pub struct ExpandedAlias {
    /// The expanded command string.
    pub expansion: String,
    /// Whether to run this command through the system shell.
    pub shell: bool,
}

impl ExpandedAlias {
    /// Converts the expansion into a vector of command arguments.
    ///
    /// For shell aliases, this returns `["-c", expansion]` suitable for
    /// passing to a shell. For command aliases, this parses the expansion
    /// into individual arguments using shell-like word splitting.
    ///
    /// # Returns
    ///
    /// A vector of strings representing command arguments.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::alias::ExpandedAlias;
    ///
    /// // Command alias parsing
    /// let cmd = ExpandedAlias {
    ///     expansion: "pr list --state open".to_string(),
    ///     shell: false,
    /// };
    /// assert_eq!(cmd.to_args(), vec!["pr", "list", "--state", "open"]);
    ///
    /// // Shell alias formatting
    /// let shell = ExpandedAlias {
    ///     expansion: "echo hello world".to_string(),
    ///     shell: true,
    /// };
    /// assert_eq!(shell.to_args(), vec!["-c", "echo hello world"]);
    /// ```
    ///
    /// # Notes
    ///
    /// - Shell word splitting respects quoted strings
    /// - If parsing fails, the entire expansion is returned as a single argument
    /// - Shell aliases are not parsed to preserve shell features like pipes
    pub fn to_args(&self) -> Vec<String> {
        if self.shell {
            // Shell commands are passed as-is
            vec!["-c".to_string(), self.expansion.clone()]
        } else {
            // Parse the expansion into arguments
            shell_words::split(&self.expansion).unwrap_or_else(|_| vec![self.expansion.clone()])
        }
    }
}

/// Validates an alias name against naming rules and reserved words.
///
/// # Parameters
///
/// * `name` - The alias name to validate
///
/// # Returns
///
/// * `Ok(())` - If the name is valid
/// * `Err` - If validation fails with a descriptive error message
///
/// # Validation Rules
///
/// An alias name is valid if it:
/// 1. Is not empty
/// 2. Contains no whitespace characters
/// 3. Is not a reserved command name
///
/// # Reserved Commands
///
/// The following command names cannot be aliased:
/// - `help`
/// - `version`
/// - `alias`
/// - `extension`
/// - `config`
/// - `completion`
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::alias::validate_alias_name;
///
/// // Valid names
/// assert!(validate_alias_name("prs").is_ok());
/// assert!(validate_alias_name("my-alias").is_ok());
/// assert!(validate_alias_name("pr2").is_ok());
///
/// // Invalid names
/// assert!(validate_alias_name("").is_err());
/// assert!(validate_alias_name("has space").is_err());
/// assert!(validate_alias_name("help").is_err());
/// ```
fn validate_alias_name(name: &str) -> Result<()> {
    if name.is_empty() {
        anyhow::bail!("Alias name cannot be empty");
    }

    // Check for invalid characters
    if name.contains(char::is_whitespace) {
        anyhow::bail!("Alias name cannot contain whitespace");
    }

    // Reserved commands that cannot be aliased
    const RESERVED: &[&str] = &[
        "help", "version", "alias", "extension", "config", "completion",
    ];

    if RESERVED.contains(&name) {
        anyhow::bail!("Cannot create alias for reserved command: {}", name);
    }

    Ok(())
}

/// Expands aliases in command line arguments.
///
/// This function checks if the first argument is a defined alias and
/// expands it accordingly. For command aliases, remaining arguments
/// are appended. For shell aliases, only the expansion is returned.
///
/// # Parameters
///
/// * `args` - The command line arguments to process
/// * `aliases` - The alias manager to use for expansion
///
/// # Returns
///
/// A vector of strings containing the expanded (or original) arguments.
///
/// # Expansion Behavior
///
/// - **Empty args**: Returns empty vector
/// - **First arg is command alias**: Expansion + remaining args
/// - **First arg is shell alias**: Shell execution args only
/// - **First arg not an alias**: Returns original args unchanged
///
/// # Example
///
/// ```rust,no_run
/// use std::path::PathBuf;
/// use bitbucket_cli::alias::{AliasManager, expand_args};
///
/// let mut manager = AliasManager::new(PathBuf::from("aliases.toml"))?;
/// manager.set("prs", "pr list --state open", false)?;
///
/// // Command alias expansion with additional args
/// let args = vec!["prs".to_string(), "--author".to_string(), "me".to_string()];
/// let expanded = expand_args(&args, &manager);
/// // Result: ["pr", "list", "--state", "open", "--author", "me"]
///
/// // Non-alias passthrough
/// let args = vec!["pr".to_string(), "create".to_string()];
/// let expanded = expand_args(&args, &manager);
/// // Result: ["pr", "create"]
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Notes
///
/// - Only the first argument is checked for alias expansion
/// - Nested aliases are not expanded (no recursive expansion)
/// - Shell aliases replace all arguments with shell invocation format
pub fn expand_args(args: &[String], aliases: &AliasManager) -> Vec<String> {
    if args.is_empty() {
        return args.to_vec();
    }

    let first = &args[0];

    if let Some(expanded) = aliases.expand(first) {
        if expanded.shell {
            // Shell aliases replace everything
            expanded.to_args()
        } else {
            // Command aliases expand and append remaining args
            let mut result = expanded.to_args();
            result.extend(args[1..].iter().cloned());
            result
        }
    } else {
        args.to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alias_entry() {
        let entry = AliasEntry::command("pr list --state open".to_string());
        assert!(!entry.shell);
        assert_eq!(entry.expansion, "pr list --state open");
    }

    #[test]
    fn test_validate_alias_name() {
        assert!(validate_alias_name("prs").is_ok());
        assert!(validate_alias_name("my-alias").is_ok());
        assert!(validate_alias_name("").is_err());
        assert!(validate_alias_name("has space").is_err());
        assert!(validate_alias_name("help").is_err());
    }
}
