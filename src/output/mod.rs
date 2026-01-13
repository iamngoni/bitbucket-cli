//
//  bitbucket-cli
//  output/mod.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # Output Module
//!
//! This module provides comprehensive output formatting capabilities for the Bitbucket CLI,
//! supporting multiple output formats to accommodate different use cases:
//!
//! - **Table format**: Human-readable tabular output for interactive terminal use
//! - **JSON format**: Machine-readable JSON output for scripting and automation
//! - **Markdown format**: Formatted markdown output for documentation and reports
//!
//! ## Architecture
//!
//! The module is organized into three submodules:
//! - [`table`]: Table formatting utilities using `comfy_table`
//! - [`json`]: JSON serialization utilities using `serde_json`
//! - [`markdown`]: Markdown formatting and rendering using `termimad`
//!
//! ## Core Components
//!
//! - [`OutputFormat`]: Enum representing the available output formats
//! - [`OutputWriter`]: Main entry point for writing formatted output
//! - [`TableOutput`]: Trait for types that can be rendered as tables or markdown
//!
//! ## Example
//!
//! ```rust,ignore
//! use bitbucket_cli::output::{OutputWriter, OutputFormat, TableOutput};
//!
//! // Create a writer for JSON output
//! let writer = OutputWriter::new(OutputFormat::Json);
//!
//! // Write a serializable value
//! writer.write(&my_data)?;
//!
//! // Write status messages
//! writer.write_success("Operation completed successfully");
//! writer.write_error("Something went wrong");
//! ```

mod table;
mod json;
mod markdown;

pub use table::*;
pub use json::*;
pub use markdown::*;

use serde::Serialize;

/// Represents the available output formats for CLI output.
///
/// The output format determines how data is rendered when written through
/// an [`OutputWriter`]. Each format is optimized for different use cases.
///
/// # Variants
///
/// * `Table` - Human-readable tabular format, best for interactive terminal sessions
/// * `Json` - Machine-readable JSON format, ideal for scripting and piping to other tools
/// * `Markdown` - Formatted markdown, useful for documentation or report generation
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::OutputFormat;
///
/// let format = OutputFormat::Json;
///
/// match format {
///     OutputFormat::Table => println!("Using table format"),
///     OutputFormat::Json => println!("Using JSON format"),
///     OutputFormat::Markdown => println!("Using markdown format"),
/// }
/// ```
///
/// # Notes
///
/// The default output format is [`OutputFormat::Table`], which provides the best
/// experience for interactive terminal use with color support.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OutputFormat {
    /// Human-readable table format with optional color support.
    ///
    /// Tables are rendered using Unicode box-drawing characters and
    /// support dynamic column width adjustment.
    Table,
    /// JSON format for scripting and automation.
    ///
    /// Output is pretty-printed by default for readability.
    /// Use [`write_json_compact`] for minified output.
    Json,
    /// Markdown format for documentation and reports.
    ///
    /// Rendered using `termimad` for terminal-friendly display.
    Markdown,
}

impl Default for OutputFormat {
    /// Returns the default output format.
    ///
    /// # Returns
    ///
    /// Returns [`OutputFormat::Table`] as the default, which provides
    /// the best experience for interactive terminal use.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::OutputFormat;
    ///
    /// let format = OutputFormat::default();
    /// assert_eq!(format, OutputFormat::Table);
    /// ```
    fn default() -> Self {
        Self::Table
    }
}

/// A unified output writer that handles multiple output formats.
///
/// `OutputWriter` is the primary interface for writing formatted output in the CLI.
/// It abstracts away the details of different output formats and provides a consistent
/// API for writing data, status messages, and errors.
///
/// # Features
///
/// * Automatic format detection and rendering
/// * Color support detection and handling
/// * Consistent message styling for errors, warnings, and success messages
/// * Support for both single values and lists
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::{OutputWriter, OutputFormat};
///
/// // Create a table output writer
/// let writer = OutputWriter::table();
///
/// // Write data
/// writer.write(&repository)?;
///
/// // Write status messages
/// writer.write_success("Repository cloned successfully");
/// writer.write_warning("Repository is archived");
/// writer.write_error("Failed to fetch repository");
/// ```
///
/// # Notes
///
/// Color output is automatically detected based on terminal capabilities.
/// Colors are disabled when output is piped or redirected.
pub struct OutputWriter {
    format: OutputFormat,
    color: bool,
}

impl OutputWriter {
    /// Creates a new output writer with the specified format.
    ///
    /// The writer automatically detects whether color output is supported
    /// based on terminal capabilities.
    ///
    /// # Parameters
    ///
    /// * `format` - The [`OutputFormat`] to use for rendering output
    ///
    /// # Returns
    ///
    /// A new `OutputWriter` instance configured with the specified format.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::{OutputWriter, OutputFormat};
    ///
    /// let json_writer = OutputWriter::new(OutputFormat::Json);
    /// let table_writer = OutputWriter::new(OutputFormat::Table);
    /// ```
    pub fn new(format: OutputFormat) -> Self {
        Self {
            format,
            color: console::colors_enabled(),
        }
    }

    /// Creates a new output writer configured for JSON output.
    ///
    /// This is a convenience constructor equivalent to
    /// `OutputWriter::new(OutputFormat::Json)`.
    ///
    /// # Returns
    ///
    /// A new `OutputWriter` instance configured for JSON output.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::OutputWriter;
    ///
    /// let writer = OutputWriter::json();
    /// writer.write(&data)?;  // Outputs pretty-printed JSON
    /// ```
    pub fn json() -> Self {
        Self::new(OutputFormat::Json)
    }

    /// Creates a new output writer configured for table output.
    ///
    /// This is a convenience constructor equivalent to
    /// `OutputWriter::new(OutputFormat::Table)`.
    ///
    /// # Returns
    ///
    /// A new `OutputWriter` instance configured for table output.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::OutputWriter;
    ///
    /// let writer = OutputWriter::table();
    /// writer.write(&repository)?;  // Outputs a formatted table
    /// ```
    pub fn table() -> Self {
        Self::new(OutputFormat::Table)
    }

    /// Checks if color output is enabled.
    ///
    /// Color support is automatically detected during construction based on
    /// terminal capabilities and environment variables.
    ///
    /// # Returns
    ///
    /// `true` if color output is enabled, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::OutputWriter;
    ///
    /// let writer = OutputWriter::table();
    /// if writer.color_enabled() {
    ///     println!("Colors are supported!");
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// Colors are typically disabled when:
    /// - Output is piped to another program
    /// - The `NO_COLOR` environment variable is set
    /// - The terminal does not support ANSI colors
    pub fn color_enabled(&self) -> bool {
        self.color
    }

    /// Returns the output format configured for this writer.
    ///
    /// # Returns
    ///
    /// The [`OutputFormat`] that this writer uses for rendering.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::{OutputWriter, OutputFormat};
    ///
    /// let writer = OutputWriter::json();
    /// assert_eq!(writer.format(), OutputFormat::Json);
    /// ```
    pub fn format(&self) -> OutputFormat {
        self.format
    }

    /// Writes a value to stdout using the configured output format.
    ///
    /// The value must implement both [`Serialize`] (for JSON output) and
    /// [`TableOutput`] (for table and markdown output).
    ///
    /// # Parameters
    ///
    /// * `value` - The value to write, must implement `Serialize` and `TableOutput`
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if serialization fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::OutputWriter;
    /// use serde::Serialize;
    ///
    /// #[derive(Serialize)]
    /// struct Repository {
    ///     name: String,
    ///     slug: String,
    /// }
    ///
    /// impl TableOutput for Repository {
    ///     fn print_table(&self, color: bool) {
    ///         println!("Name: {}", self.name);
    ///     }
    ///     fn print_markdown(&self) {
    ///         println!("## {}", self.name);
    ///     }
    /// }
    ///
    /// let writer = OutputWriter::table();
    /// writer.write(&Repository { name: "my-repo".into(), slug: "my-repo".into() })?;
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails (only applicable for JSON format).
    pub fn write<T: Serialize + TableOutput>(&self, value: &T) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(value)?;
                println!("{}", json);
            }
            OutputFormat::Table => {
                value.print_table(self.color);
            }
            OutputFormat::Markdown => {
                value.print_markdown();
            }
        }
        Ok(())
    }

    /// Writes a list of values to stdout using the configured output format.
    ///
    /// For JSON format, the entire list is serialized as a JSON array.
    /// For table and markdown formats, each value is rendered individually.
    ///
    /// # Parameters
    ///
    /// * `values` - A slice of values to write
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if serialization fails.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::OutputWriter;
    ///
    /// let repositories = vec![repo1, repo2, repo3];
    /// let writer = OutputWriter::json();
    /// writer.write_list(&repositories)?;  // Outputs a JSON array
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if JSON serialization fails (only applicable for JSON format).
    pub fn write_list<T: Serialize + TableOutput>(&self, values: &[T]) -> anyhow::Result<()> {
        match self.format {
            OutputFormat::Json => {
                let json = serde_json::to_string_pretty(values)?;
                println!("{}", json);
            }
            OutputFormat::Table => {
                for value in values {
                    value.print_table(self.color);
                }
            }
            OutputFormat::Markdown => {
                for value in values {
                    value.print_markdown();
                }
            }
        }
        Ok(())
    }

    /// Writes an error message to stderr.
    ///
    /// The message is prefixed with "error:" and styled in red when
    /// color output is enabled.
    ///
    /// # Parameters
    ///
    /// * `msg` - The error message to display
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::OutputWriter;
    ///
    /// let writer = OutputWriter::table();
    /// writer.write_error("Failed to connect to Bitbucket API");
    /// // Output: error: Failed to connect to Bitbucket API
    /// ```
    ///
    /// # Notes
    ///
    /// Error messages are always written to stderr, regardless of output format.
    pub fn write_error(&self, msg: &str) {
        use console::style;
        if self.color {
            eprintln!("{} {}", style("error:").red().bold(), msg);
        } else {
            eprintln!("error: {}", msg);
        }
    }

    /// Writes a warning message to stderr.
    ///
    /// The message is prefixed with "warning:" and styled in yellow when
    /// color output is enabled.
    ///
    /// # Parameters
    ///
    /// * `msg` - The warning message to display
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::OutputWriter;
    ///
    /// let writer = OutputWriter::table();
    /// writer.write_warning("Repository is archived and read-only");
    /// // Output: warning: Repository is archived and read-only
    /// ```
    ///
    /// # Notes
    ///
    /// Warning messages are written to stderr to separate them from normal output.
    pub fn write_warning(&self, msg: &str) {
        use console::style;
        if self.color {
            eprintln!("{} {}", style("warning:").yellow().bold(), msg);
        } else {
            eprintln!("warning: {}", msg);
        }
    }

    /// Writes an informational message to stdout.
    ///
    /// The message is printed without any prefix or styling.
    ///
    /// # Parameters
    ///
    /// * `msg` - The informational message to display
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::OutputWriter;
    ///
    /// let writer = OutputWriter::table();
    /// writer.write_info("Fetching repository information...");
    /// ```
    pub fn write_info(&self, msg: &str) {
        println!("{}", msg);
    }

    /// Writes a success message to stdout.
    ///
    /// The message is prefixed with a green checkmark when color output
    /// is enabled.
    ///
    /// # Parameters
    ///
    /// * `msg` - The success message to display
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::OutputWriter;
    ///
    /// let writer = OutputWriter::table();
    /// writer.write_success("Pull request created successfully");
    /// // Output: ✓ Pull request created successfully
    /// ```
    pub fn write_success(&self, msg: &str) {
        use console::style;
        if self.color {
            println!("{} {}", style("✓").green().bold(), msg);
        } else {
            println!("✓ {}", msg);
        }
    }
}

/// A trait for types that can be rendered as table or markdown output.
///
/// Types implementing this trait can be written through an [`OutputWriter`]
/// and will be properly formatted for table and markdown output formats.
/// For JSON output, types must also implement [`Serialize`].
///
/// # Required Methods
///
/// * [`print_table`](TableOutput::print_table) - Renders the type as a table row or section
/// * [`print_markdown`](TableOutput::print_markdown) - Renders the type as markdown
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::{TableOutput, print_field, print_header};
///
/// struct PullRequest {
///     id: u64,
///     title: String,
///     state: String,
/// }
///
/// impl TableOutput for PullRequest {
///     fn print_table(&self, color: bool) {
///         print_header(&format!("PR #{}", self.id));
///         print_field("Title", &self.title, color);
///         print_field("State", &self.state, color);
///         println!();
///     }
///
///     fn print_markdown(&self) {
///         println!("## PR #{}", self.id);
///         println!("- **Title**: {}", self.title);
///         println!("- **State**: {}", self.state);
///     }
/// }
/// ```
///
/// # Notes
///
/// Implementations should be mindful of terminal width and use appropriate
/// truncation for long values.
pub trait TableOutput {
    /// Renders the type as a table row or section.
    ///
    /// # Parameters
    ///
    /// * `color` - Whether color output is enabled
    ///
    /// # Notes
    ///
    /// Implementations should use the `color` parameter to conditionally
    /// apply styling. Use helper functions like [`format_status`] and
    /// [`format_bool`] for consistent styling.
    fn print_table(&self, color: bool);

    /// Renders the type as markdown.
    ///
    /// # Notes
    ///
    /// The markdown output should be valid markdown that can be rendered
    /// by standard markdown processors. Use helper functions like
    /// [`md_header`], [`md_bold`], and [`md_table`] for formatting.
    fn print_markdown(&self);
}

/// Prints a styled header with an underline.
///
/// The header text is printed in bold, followed by a dashed underline
/// of the same length.
///
/// # Parameters
///
/// * `text` - The header text to print
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::print_header;
///
/// print_header("Repository Details");
/// // Output:
/// // Repository Details
/// // ------------------
/// ```
///
/// # Notes
///
/// The underline uses ASCII dashes for maximum terminal compatibility.
pub fn print_header(text: &str) {
    use console::style;
    println!("{}", style(text).bold());
    println!("{}", "-".repeat(text.len()));
}

/// Prints a key-value pair with optional styling.
///
/// The key is dimmed when color is enabled to provide visual separation
/// from the value.
///
/// # Parameters
///
/// * `key` - The field name/label
/// * `value` - The field value
/// * `color` - Whether to apply color styling
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::print_field;
///
/// print_field("Name", "my-repository", true);
/// print_field("Owner", "team-workspace", true);
/// print_field("Private", "Yes", true);
/// // Output:
/// // Name: my-repository
/// // Owner: team-workspace
/// // Private: Yes
/// ```
///
/// # Notes
///
/// This function is commonly used in [`TableOutput::print_table`]
/// implementations for rendering object fields.
pub fn print_field(key: &str, value: &str, color: bool) {
    use console::style;
    if color {
        println!("{}: {}", style(key).dim(), value);
    } else {
        println!("{}: {}", key, value);
    }
}
