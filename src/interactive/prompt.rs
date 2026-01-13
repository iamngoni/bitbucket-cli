//
//  bitbucket-cli
//  interactive/prompt.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Interactive Prompts Module
//!
//! This module provides various interactive prompt functions for gathering user
//! input in the terminal. It wraps the `dialoguer` crate to offer a consistent
//! interface for text input, password entry, confirmations, and editor integration.
//!
//! # Overview
//!
//! The module provides the following prompt types:
//! - **Text Input** - Single-line text entry with optional defaults
//! - **Password Input** - Masked text entry with optional confirmation
//! - **Confirmation** - Yes/no prompts with optional defaults
//! - **Editor** - Multi-line text editing using external editors
//!
//! # Example
//!
//! ```no_run
//! use bitbucket_cli::interactive::prompt::{prompt_input, prompt_confirm, prompt_password};
//!
//! // Get username
//! let username = prompt_input("Enter username:").unwrap();
//!
//! // Get password (masked)
//! let password = prompt_password("Enter password:").unwrap();
//!
//! // Confirm action
//! if prompt_confirm("Save credentials?").unwrap() {
//!     println!("Credentials saved!");
//! }
//! ```

use anyhow::Result;
use dialoguer::{Confirm, Editor, Input, Password};

/// Prompts the user for text input.
///
/// Displays a prompt message and waits for the user to enter text.
/// The input cannot be empty - the prompt will repeat until valid input is provided.
///
/// # Parameters
///
/// * `message` - The prompt message displayed to the user
///
/// # Returns
///
/// Returns `Ok(String)` containing the user's input on success.
/// Returns `Err` if the terminal interaction fails (e.g., stdin closed).
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::prompt::prompt_input;
///
/// let repo_name = prompt_input("Repository name:").unwrap();
/// println!("Creating repository: {}", repo_name);
/// ```
///
/// # Notes
///
/// - The prompt will block until the user provides input and presses Enter
/// - Use [`prompt_input_optional`] if empty input should be allowed
/// - Use [`prompt_input_with_default`] to provide a default value
pub fn prompt_input(message: &str) -> Result<String> {
    let input: String = Input::new().with_prompt(message).interact_text()?;
    Ok(input)
}

/// Prompts the user for text input with a default value.
///
/// Displays a prompt message with a pre-filled default value. The user can
/// accept the default by pressing Enter or type a new value.
///
/// # Parameters
///
/// * `message` - The prompt message displayed to the user
/// * `default` - The default value shown to the user (used if Enter is pressed)
///
/// # Returns
///
/// Returns `Ok(String)` containing either the user's input or the default value.
/// Returns `Err` if the terminal interaction fails.
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::prompt::prompt_input_with_default;
///
/// let branch = prompt_input_with_default("Branch name:", "main").unwrap();
/// println!("Using branch: {}", branch);
/// ```
///
/// # Notes
///
/// - The default value is displayed in the prompt (e.g., "Branch name: [main]")
/// - Useful for configuration values that have sensible defaults
pub fn prompt_input_with_default(message: &str, default: &str) -> Result<String> {
    let input: String = Input::new()
        .with_prompt(message)
        .default(default.to_string())
        .interact_text()?;
    Ok(input)
}

/// Prompts the user for optional text input.
///
/// Displays a prompt message and allows the user to provide input or skip
/// by pressing Enter without typing anything.
///
/// # Parameters
///
/// * `message` - The prompt message displayed to the user
///
/// # Returns
///
/// Returns `Ok(Some(String))` if the user provided non-empty input.
/// Returns `Ok(None)` if the user pressed Enter without input.
/// Returns `Err` if the terminal interaction fails.
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::prompt::prompt_input_optional;
///
/// let description = prompt_input_optional("Description (optional):").unwrap();
/// match description {
///     Some(desc) => println!("Description: {}", desc),
///     None => println!("No description provided"),
/// }
/// ```
///
/// # Notes
///
/// - Unlike [`prompt_input`], this allows empty input
/// - Empty strings are converted to `None` for cleaner handling
pub fn prompt_input_optional(message: &str) -> Result<Option<String>> {
    let input: String = Input::new()
        .with_prompt(message)
        .allow_empty(true)
        .interact_text()?;
    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input))
    }
}

/// Prompts the user for password input with masked display.
///
/// Displays a prompt message and accepts password input without echoing
/// the characters to the terminal. Each typed character is hidden or
/// replaced with a masking character.
///
/// # Parameters
///
/// * `message` - The prompt message displayed to the user
///
/// # Returns
///
/// Returns `Ok(String)` containing the entered password.
/// Returns `Err` if the terminal interaction fails.
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::prompt::prompt_password;
///
/// let token = prompt_password("API Token:").unwrap();
/// // Token is now stored securely, not visible in terminal history
/// ```
///
/// # Notes
///
/// - Characters are not echoed to prevent shoulder surfing
/// - Use [`prompt_password_confirm`] when setting new passwords
/// - The password is returned as a plain `String` - handle securely
pub fn prompt_password(message: &str) -> Result<String> {
    let password = Password::new().with_prompt(message).interact()?;
    Ok(password)
}

/// Prompts the user for password input with confirmation.
///
/// Displays a prompt message for password entry, then asks the user to
/// confirm by entering the same password again. The prompt repeats if
/// the passwords do not match.
///
/// # Parameters
///
/// * `message` - The prompt message displayed to the user
///
/// # Returns
///
/// Returns `Ok(String)` containing the confirmed password.
/// Returns `Err` if the terminal interaction fails or is cancelled.
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::prompt::prompt_password_confirm;
///
/// let new_password = prompt_password_confirm("Set new password:").unwrap();
/// println!("Password set successfully");
/// ```
///
/// # Notes
///
/// - Both entries must match exactly
/// - If passwords don't match, an error message is shown and the prompt repeats
/// - Ideal for account creation or password change flows
pub fn prompt_password_confirm(message: &str) -> Result<String> {
    let password = Password::new()
        .with_prompt(message)
        .with_confirmation("Confirm password", "Passwords do not match")
        .interact()?;
    Ok(password)
}

/// Prompts the user for a yes/no confirmation.
///
/// Displays a message and waits for the user to confirm (y/yes) or
/// deny (n/no). The response is case-insensitive.
///
/// # Parameters
///
/// * `message` - The confirmation message displayed to the user
///
/// # Returns
///
/// Returns `Ok(true)` if the user confirmed (y/yes).
/// Returns `Ok(false)` if the user denied (n/no).
/// Returns `Err` if the terminal interaction fails.
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::prompt::prompt_confirm;
///
/// if prompt_confirm("Delete this repository?").unwrap() {
///     println!("Repository deleted");
/// } else {
///     println!("Operation cancelled");
/// }
/// ```
///
/// # Notes
///
/// - Use [`prompt_confirm_with_default`] if a default choice is preferred
/// - Without a default, the user must explicitly type y/yes or n/no
pub fn prompt_confirm(message: &str) -> Result<bool> {
    let confirmed = Confirm::new().with_prompt(message).interact()?;
    Ok(confirmed)
}

/// Prompts the user for a yes/no confirmation with a default value.
///
/// Displays a message with a default choice. The user can accept the
/// default by pressing Enter, or explicitly choose yes/no.
///
/// # Parameters
///
/// * `message` - The confirmation message displayed to the user
/// * `default` - The default value if the user presses Enter (`true` for yes, `false` for no)
///
/// # Returns
///
/// Returns `Ok(true)` if the user confirmed or accepted a `true` default.
/// Returns `Ok(false)` if the user denied or accepted a `false` default.
/// Returns `Err` if the terminal interaction fails.
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::prompt::prompt_confirm_with_default;
///
/// // Default to yes - shown as [Y/n]
/// let proceed = prompt_confirm_with_default("Continue?", true).unwrap();
///
/// // Default to no - shown as [y/N]
/// let risky = prompt_confirm_with_default("Force push?", false).unwrap();
/// ```
///
/// # Notes
///
/// - The default is shown in the prompt (e.g., "[Y/n]" or "[y/N]")
/// - Pressing Enter without input accepts the default
pub fn prompt_confirm_with_default(message: &str, default: bool) -> Result<bool> {
    let confirmed = Confirm::new()
        .with_prompt(message)
        .default(default)
        .interact()?;
    Ok(confirmed)
}

/// Opens the user's preferred editor for multiline text input.
///
/// Launches an external text editor for composing or editing text.
/// The editor preference is determined by [`get_editor`].
///
/// # Parameters
///
/// * `initial` - Optional initial content to populate the editor with
///
/// # Returns
///
/// Returns `Ok(Some(String))` containing the edited text if the user saved and closed.
/// Returns `Ok(None)` if the user closed without saving or the editor returned empty.
/// Returns `Err` if the editor fails to launch or encounters an error.
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::prompt::prompt_editor;
///
/// // New content
/// let body = prompt_editor(None).unwrap();
///
/// // Edit existing content
/// let updated = prompt_editor(Some("Existing description")).unwrap();
/// ```
///
/// # Notes
///
/// - The editor is determined by `BB_EDITOR`, `EDITOR`, `VISUAL`, or defaults to `nano`
/// - A temporary file is created and opened in the editor
/// - Use [`prompt_editor_with_command`] to specify a custom editor
pub fn prompt_editor(initial: Option<&str>) -> Result<Option<String>> {
    let editor = Editor::new();
    let result = editor.edit(initial.unwrap_or(""))?;
    Ok(result)
}

/// Opens a specific editor for multiline text input.
///
/// Launches the specified text editor for composing or editing text,
/// overriding the default editor detection.
///
/// # Parameters
///
/// * `command` - The editor command to execute (e.g., "vim", "code --wait", "nano")
/// * `initial` - Optional initial content to populate the editor with
///
/// # Returns
///
/// Returns `Ok(Some(String))` containing the edited text if the user saved and closed.
/// Returns `Ok(None)` if the user closed without saving or the editor returned empty.
/// Returns `Err` if the editor fails to launch or encounters an error.
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::prompt::prompt_editor_with_command;
///
/// // Use VS Code with wait flag
/// let content = prompt_editor_with_command("code --wait", Some("Initial text")).unwrap();
///
/// // Use vim
/// let message = prompt_editor_with_command("vim", None).unwrap();
/// ```
///
/// # Notes
///
/// - The command can include arguments (e.g., "code --wait" for VS Code)
/// - Ensure the editor blocks until the file is saved (some GUI editors need special flags)
/// - If the editor command is not found, an error is returned
pub fn prompt_editor_with_command(command: &str, initial: Option<&str>) -> Result<Option<String>> {
    let mut editor = Editor::new();
    editor.executable(command);
    let result = editor.edit(initial.unwrap_or(""))?;
    Ok(result)
}

/// Returns the user's preferred text editor.
///
/// Determines the editor to use for multiline input by checking environment
/// variables in order of priority.
///
/// # Returns
///
/// Returns the editor command string, determined by checking (in order):
/// 1. `BB_EDITOR` - Bitbucket CLI specific editor preference
/// 2. `EDITOR` - Standard Unix editor environment variable
/// 3. `VISUAL` - Alternative Unix editor environment variable
/// 4. `"nano"` - Default fallback editor
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::prompt::get_editor;
///
/// let editor = get_editor();
/// println!("Using editor: {}", editor);
/// ```
///
/// # Notes
///
/// - `BB_EDITOR` takes precedence to allow CLI-specific customization
/// - The returned string may include arguments (e.g., "code --wait")
/// - Falls back to `nano` as it's commonly available on Unix systems
pub fn get_editor() -> String {
    std::env::var("BB_EDITOR")
        .or_else(|_| std::env::var("EDITOR"))
        .or_else(|_| std::env::var("VISUAL"))
        .unwrap_or_else(|_| "nano".to_string())
}
