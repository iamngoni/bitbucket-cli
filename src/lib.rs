//
//  bitbucket-cli
//  lib.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # Bitbucket CLI Library
//!
//! A comprehensive command-line interface library for interacting with Bitbucket Cloud
//! and Bitbucket Server/Data Center platforms.
//!
//! ## Overview
//!
//! This library provides the core functionality for the `bb` CLI tool, enabling developers
//! to manage repositories, pull requests, pipelines, and other Bitbucket resources directly
//! from the terminal.
//!
//! ## Features
//!
//! - **Multi-Platform Support**: First-class support for both Bitbucket Cloud and Server/DC
//! - **Repository Management**: Clone, create, fork, and manage repositories
//! - **Pull Request Workflows**: Create, review, merge, and manage PRs
//! - **CI/CD Integration**: Trigger and monitor Bitbucket Pipelines (Cloud)
//! - **Secure Authentication**: OAuth 2.0, Personal Access Tokens, and secure credential storage
//! - **Interactive & Scriptable**: Rich terminal UI with JSON output for automation
//!
//! ## Module Structure
//!
//! - [`cli`]: Command-line interface definitions using clap
//! - [`api`]: HTTP clients for Bitbucket Cloud and Server/DC APIs
//! - [`auth`]: Authentication management (OAuth, PAT, keychain storage)
//! - [`config`]: Configuration file management
//! - [`context`]: Git repository context detection
//! - [`output`]: Output formatting (Table, JSON, Markdown)
//! - [`interactive`]: Interactive prompts and selectors
//! - [`extension`]: CLI extension system
//! - [`alias`]: Command alias management
//! - [`util`]: Utility functions
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use bitbucket_cli::{Config, RepoContext};
//!
//! // Load configuration
//! let config = Config::load().expect("Failed to load config");
//!
//! // Detect repository context from current directory
//! let context = RepoContext::detect().expect("Not in a git repository");
//! println!("Current repo: {}/{}", context.workspace, context.repo);
//! ```
//!
//! ## Platform Differences
//!
//! | Feature | Cloud | Server/DC |
//! |---------|-------|-----------|
//! | Pipelines | Yes | No |
//! | Workspaces | Yes | No |
//! | Projects | Limited | Yes |
//! | OAuth 2.0 | Yes | Limited |

/// Command-line interface definitions.
///
/// Contains all CLI commands, arguments, and subcommands defined using the clap derive API.
/// Each command module handles parsing and execution of its respective functionality.
pub mod cli;

/// API client implementations for Bitbucket platforms.
///
/// This module provides HTTP clients for interacting with:
/// - Bitbucket Cloud API v2.0
/// - Bitbucket Server/Data Center API v1.0
///
/// The clients handle authentication, request building, pagination, and error handling.
pub mod api;

/// Authentication and credential management.
///
/// Handles multiple authentication methods:
/// - OAuth 2.0 with PKCE (Bitbucket Cloud)
/// - Personal Access Tokens (Server/DC)
/// - Secure credential storage via system keychain
/// - Multi-profile management for different hosts
pub mod auth;

/// Configuration file management.
///
/// Manages the CLI's configuration stored in platform-specific locations:
/// - Linux: `~/.config/bb/config.toml`
/// - macOS: `~/Library/Application Support/bb/config.toml`
/// - Windows: `%APPDATA%\bb\config.toml`
pub mod config;

/// Git repository context detection.
///
/// Detects the current repository's Bitbucket context by parsing git remotes,
/// identifying the platform (Cloud vs Server/DC), and extracting workspace/project
/// and repository information.
pub mod context;

/// Output formatting for different modes.
///
/// Provides formatters for:
/// - Table format: Human-readable tables for interactive use
/// - JSON format: Structured output for scripting and automation
/// - Markdown format: Documentation-friendly output
pub mod output;

/// Interactive terminal UI components.
///
/// Provides interactive prompts, selectors, and input helpers for:
/// - Text input with validation
/// - Password input (masked)
/// - Single and multi-select menus
/// - Fuzzy search selection
/// - Editor integration for multiline text
pub mod interactive;

/// CLI extension system.
///
/// Allows extending the CLI with external executables following the `bb-<name>`
/// naming convention. Extensions can be written in any language and are
/// discovered from the system PATH or the extensions directory.
pub mod extension;

/// Command alias management.
///
/// Allows users to create shortcuts for frequently used commands.
/// Aliases can expand to full command strings or execute shell commands.
pub mod alias;

/// Utility functions and helpers.
///
/// Common utilities used throughout the codebase including:
/// - Time formatting (absolute and relative)
/// - Size formatting (bytes to human-readable)
/// - String manipulation (truncation, slugification)
/// - Browser and pager integration
pub mod util;

/// Re-export of the main CLI struct for convenient access.
///
/// The [`Cli`] struct represents the root command and is the entry point
/// for parsing command-line arguments.
///
/// # Example
///
/// ```rust,no_run
/// use clap::Parser;
/// use bitbucket_cli::Cli;
///
/// let cli = Cli::parse();
/// // Handle cli.command...
/// ```
pub use cli::Cli;

/// Re-export of the configuration struct.
///
/// The [`Config`] struct provides access to the user's CLI configuration,
/// including authentication profiles, default settings, and preferences.
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::Config;
///
/// let config = Config::load().expect("Failed to load config");
/// if let Some(workspace) = config.get("default_workspace") {
///     println!("Default workspace: {}", workspace);
/// }
/// ```
pub use config::Config;

/// Re-export of the repository context struct.
///
/// The [`RepoContext`] struct contains information about the current
/// git repository's connection to Bitbucket, including the platform,
/// workspace/project, and repository name.
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::RepoContext;
///
/// if let Ok(context) = RepoContext::detect() {
///     println!("Repository: {}/{}", context.workspace, context.repo);
///     println!("Platform: {:?}", context.platform);
/// }
/// ```
pub use context::RepoContext;

/// Application name constant.
///
/// The name of the CLI binary, used for display purposes and configuration paths.
///
/// # Value
///
/// `"bb"`
pub const APP_NAME: &str = "bb";

/// Application version constant.
///
/// The current version of the CLI, automatically derived from Cargo.toml
/// at compile time using the `CARGO_PKG_VERSION` environment variable.
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::VERSION;
///
/// println!("bb version {}", VERSION);
/// ```
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Exit codes for the CLI.
///
/// Standardized exit codes following Unix conventions, allowing scripts
/// to programmatically detect the outcome of CLI operations.
///
/// # Exit Code Ranges
///
/// - `0`: Success
/// - `1-3`: General errors and usage issues
/// - `4-7`: Authentication-related issues
/// - `8-15`: Resource-related issues
/// - `16-31`: Operation-related issues
/// - `32+`: External service issues
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::exit_codes;
/// use std::process;
///
/// // Exit with authentication error
/// process::exit(exit_codes::AUTH_ERROR);
/// ```
pub mod exit_codes {
    /// Successful execution.
    ///
    /// The command completed without errors.
    ///
    /// # Value
    ///
    /// `0`
    pub const SUCCESS: i32 = 0;

    /// General error.
    ///
    /// An unspecified error occurred during execution.
    /// Check stderr for details.
    ///
    /// # Value
    ///
    /// `1`
    pub const ERROR: i32 = 1;

    /// Invalid usage or arguments.
    ///
    /// The command was invoked with invalid arguments or options.
    /// Use `--help` to see correct usage.
    ///
    /// # Value
    ///
    /// `2`
    pub const USAGE: i32 = 2;

    /// Authentication required or failed.
    ///
    /// The user is not authenticated or the authentication token is invalid.
    /// Run `bb auth login` to authenticate.
    ///
    /// # Value
    ///
    /// `4`
    pub const AUTH_ERROR: i32 = 4;

    /// Resource not found.
    ///
    /// The requested resource (repository, pull request, etc.) does not exist
    /// or the user does not have permission to access it.
    ///
    /// # Value
    ///
    /// `8`
    pub const NOT_FOUND: i32 = 8;

    /// Operation cancelled by user.
    ///
    /// The user cancelled the operation, typically by pressing Ctrl+C
    /// or declining a confirmation prompt.
    ///
    /// # Value
    ///
    /// `16`
    pub const CANCELLED: i32 = 16;

    /// API rate limit exceeded.
    ///
    /// The Bitbucket API rate limit has been exceeded.
    /// Wait before retrying or use a different authentication method.
    ///
    /// # Value
    ///
    /// `32`
    pub const RATE_LIMIT: i32 = 32;
}
