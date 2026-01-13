//
//  bitbucket-cli
//  interactive/mod.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Interactive UI Components Module
//!
//! This module provides interactive prompts and selectors for user input in the
//! Bitbucket CLI application. It wraps the `dialoguer` and `console` crates to
//! offer a consistent, user-friendly interface for terminal interactions.
//!
//! # Overview
//!
//! The interactive module is divided into two submodules:
//! - [`prompt`] - Text input, password entry, confirmations, and editor integration
//! - [`selector`] - Single and multi-item selection with optional fuzzy search
//!
//! # Environment Variables
//!
//! The module respects the following environment variables:
//! - `BB_NO_PROMPT` - When set, disables interactive prompts
//! - `BB_FORCE_TTY` - When set, forces TTY behavior even in non-interactive contexts
//!
//! # Example
//!
//! ```no_run
//! use bitbucket_cli::interactive::{is_interactive, prompt_confirm, select};
//!
//! if is_interactive() {
//!     let confirmed = prompt_confirm("Proceed with operation?").unwrap();
//!     if confirmed {
//!         let choice = select("Select repository:", &["repo1", "repo2"]).unwrap();
//!         println!("Selected index: {}", choice);
//!     }
//! }
//! ```

mod prompt;
mod selector;

pub use prompt::*;
pub use selector::*;

use console::Term;

/// Checks if the application is running in an interactive terminal.
///
/// This function determines whether user interaction is possible by verifying
/// that stdout is connected to a terminal (TTY) and that interactive prompts
/// have not been explicitly disabled.
///
/// # Returns
///
/// Returns `true` if both conditions are met:
/// - Standard output is connected to a terminal
/// - The `BB_NO_PROMPT` environment variable is not set
///
/// Returns `false` otherwise.
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::is_interactive;
///
/// if is_interactive() {
///     println!("Running interactively - can prompt user");
/// } else {
///     println!("Non-interactive mode - using defaults");
/// }
/// ```
///
/// # Notes
///
/// - This check is commonly used before displaying prompts to avoid hanging
///   in CI/CD pipelines or when output is redirected
/// - Set `BB_NO_PROMPT=1` to disable interactive prompts in scripts
pub fn is_interactive() -> bool {
    Term::stdout().is_term() && std::env::var("BB_NO_PROMPT").is_err()
}

/// Checks if TTY behavior should be forced.
///
/// This function allows users to override the automatic terminal detection
/// and force the application to behave as if it's running in an interactive
/// terminal, even when it might not be.
///
/// # Returns
///
/// Returns `true` if the `BB_FORCE_TTY` environment variable is set (to any value).
/// Returns `false` otherwise.
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::force_tty;
///
/// if force_tty() {
///     println!("TTY mode forced by user");
/// }
/// ```
///
/// # Notes
///
/// - Useful for debugging or when terminal detection fails
/// - Set `BB_FORCE_TTY=1` to enable forced TTY mode
/// - This can cause issues if stdout is actually redirected to a file
pub fn force_tty() -> bool {
    std::env::var("BB_FORCE_TTY").is_ok()
}

/// Determines the effective interactive mode for the application.
///
/// This is the primary function to check before displaying any interactive
/// UI elements. It combines both automatic detection and manual override.
///
/// # Returns
///
/// Returns `true` if either:
/// - TTY mode is forced via `BB_FORCE_TTY` environment variable, OR
/// - The application is running in an interactive terminal (see [`is_interactive`])
///
/// Returns `false` only when both conditions are false.
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::{interactive_mode, prompt_input};
///
/// fn get_username() -> String {
///     if interactive_mode() {
///         prompt_input("Enter username:").unwrap()
///     } else {
///         std::env::var("BB_USERNAME").unwrap_or_else(|_| "default".to_string())
///     }
/// }
/// ```
///
/// # Notes
///
/// - Prefer this function over [`is_interactive`] for most use cases
/// - The force override takes precedence over automatic detection
pub fn interactive_mode() -> bool {
    force_tty() || is_interactive()
}
