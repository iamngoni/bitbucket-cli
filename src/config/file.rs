//
//  bitbucket-cli
//  config/file.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # Configuration File I/O Module
//!
//! This module provides low-level file operations for configuration management.
//! It handles reading, writing, and checking the existence of configuration files.
//!
//! ## Overview
//!
//! The functions in this module are designed to be simple, focused utilities
//! that handle the file system operations needed by the higher-level configuration
//! management code. They abstract away common patterns like creating parent
//! directories before writing files.
//!
//! ## Usage
//!
//! These functions are typically used internally by the [`Config`](super::Config)
//! struct, but can be used directly for custom configuration file operations.
//!
//! ```rust,no_run
//! use std::path::Path;
//! use bitbucket_cli::config::{read_config_file, write_config_file, config_exists};
//!
//! let path = Path::new("/path/to/config.toml");
//!
//! // Check if configuration exists
//! if config_exists(path) {
//!     // Read existing configuration
//!     let content = read_config_file(path)?;
//!     println!("Config content: {}", content);
//! } else {
//!     // Write new configuration
//!     write_config_file(path, "[core]\ngit_protocol = \"https\"\n")?;
//! }
//! # Ok::<(), anyhow::Error>(())
//! ```
//!
//! ## Error Handling
//!
//! All I/O operations use `anyhow::Result` for error handling, providing
//! rich error context that can be propagated up the call stack.
//!
//! ## Notes
//!
//! - All functions accept `&Path` to support both `Path` and `PathBuf`
//! - Write operations automatically create parent directories
//! - Read operations return the raw file content as a `String`

use std::path::Path;

use anyhow::Result;

/// Reads the contents of a configuration file.
///
/// Opens the specified file and reads its entire contents into a string.
/// This function is a thin wrapper around `std::fs::read_to_string` that
/// converts the error type to `anyhow::Error`.
///
/// # Parameters
///
/// * `path` - The path to the configuration file to read
///
/// # Returns
///
/// - `Ok(String)` - The complete contents of the file as a UTF-8 string
/// - `Err` - If the file cannot be read
///
/// # Errors
///
/// This function will return an error if:
/// - The file does not exist
/// - The file cannot be opened (permissions, in use, etc.)
/// - The file contains invalid UTF-8 data
/// - An I/O error occurs during reading
///
/// # Examples
///
/// ## Reading an Existing Configuration File
///
/// ```rust,no_run
/// use std::path::Path;
/// use bitbucket_cli::config::read_config_file;
///
/// let path = Path::new("~/.config/bb/config.toml");
/// let content = read_config_file(path)?;
/// println!("Configuration:\n{}", content);
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// ## Handling Missing Files
///
/// ```rust,no_run
/// use std::path::Path;
/// use bitbucket_cli::config::read_config_file;
///
/// let path = Path::new("/nonexistent/config.toml");
/// match read_config_file(path) {
///     Ok(content) => println!("Found: {}", content),
///     Err(e) => println!("File not found or unreadable: {}", e),
/// }
/// ```
///
/// # Notes
///
/// - Use [`config_exists`] to check for file existence before reading
/// - The entire file is read into memory; not suitable for very large files
/// - The returned string includes any trailing newlines from the file
pub fn read_config_file(path: &Path) -> Result<String> {
    Ok(std::fs::read_to_string(path)?)
}

/// Writes content to a configuration file.
///
/// Creates or overwrites the specified file with the given content.
/// Automatically creates any necessary parent directories if they don't exist.
///
/// # Parameters
///
/// * `path` - The path where the configuration file should be written
/// * `content` - The content to write to the file
///
/// # Returns
///
/// - `Ok(())` - The file was written successfully
/// - `Err` - If the file could not be written
///
/// # Errors
///
/// This function will return an error if:
/// - Parent directories cannot be created (permissions, invalid path)
/// - The file cannot be created or opened for writing
/// - An I/O error occurs during writing
/// - The disk is full or a quota is exceeded
///
/// # Examples
///
/// ## Writing a New Configuration File
///
/// ```rust,no_run
/// use std::path::Path;
/// use bitbucket_cli::config::write_config_file;
///
/// let path = Path::new("~/.config/bb/config.toml");
/// let content = r#"
/// [core]
/// editor = "vim"
/// git_protocol = "https"
/// "#;
///
/// write_config_file(path, content)?;
/// println!("Configuration saved!");
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// ## Writing to a Nested Path
///
/// ```rust,no_run
/// use std::path::Path;
/// use bitbucket_cli::config::write_config_file;
///
/// // Parent directories are created automatically
/// let path = Path::new("/deeply/nested/path/config.toml");
/// write_config_file(path, "content")?;
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Notes
///
/// - **Warning**: This function overwrites existing files without warning
/// - Parent directories are created with default permissions
/// - Content is written atomically where supported by the OS
/// - Use [`config_exists`] first if you need to avoid overwriting
pub fn write_config_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, content)?;
    Ok(())
}

/// Checks if a configuration file exists.
///
/// Returns `true` if a file exists at the specified path. This function
/// checks only for file existence, not whether the file is readable or
/// contains valid configuration data.
///
/// # Parameters
///
/// * `path` - The path to check for existence
///
/// # Returns
///
/// - `true` - A file exists at the specified path
/// - `false` - No file exists at the path, or the path is inaccessible
///
/// # Examples
///
/// ## Checking Before Reading
///
/// ```rust,no_run
/// use std::path::Path;
/// use bitbucket_cli::config::{config_exists, read_config_file};
///
/// let path = Path::new("~/.config/bb/config.toml");
///
/// if config_exists(path) {
///     let content = read_config_file(path)?;
///     // Process existing configuration
/// } else {
///     // Use default configuration
///     println!("No configuration file found, using defaults");
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// ## Avoiding Overwrites
///
/// ```rust,no_run
/// use std::path::Path;
/// use bitbucket_cli::config::{config_exists, write_config_file};
///
/// let path = Path::new("~/.config/bb/config.toml");
///
/// if !config_exists(path) {
///     write_config_file(path, "default config content")?;
/// } else {
///     println!("Configuration already exists, not overwriting");
/// }
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Notes
///
/// - This is a thin wrapper around `Path::exists()`
/// - Returns `false` for directories (only checks for files)
/// - May return `false` for files that exist but are not accessible
/// - Does not follow symbolic links on some platforms
pub fn config_exists(path: &Path) -> bool {
    path.exists()
}
