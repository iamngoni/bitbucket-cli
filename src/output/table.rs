//
//  bitbucket-cli
//  output/table.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # Table Output Formatting
//!
//! This module provides utilities for creating and formatting tabular output
//! in the terminal. It uses the `comfy_table` crate for rendering Unicode
//! tables with dynamic content arrangement.
//!
//! ## Features
//!
//! - Builder pattern for constructing tables with headers and rows
//! - Automatic color detection and application
//! - Status-aware formatting with semantic colors
//! - String truncation for long values
//! - Boolean formatting as human-readable Yes/No
//!
//! ## Example
//!
//! ```rust,ignore
//! use bitbucket_cli::output::table::TableBuilder;
//!
//! TableBuilder::new()
//!     .headers(["ID", "Name", "Status"])
//!     .row(["1", "my-repo", "active"])
//!     .row(["2", "other-repo", "archived"])
//!     .print();
//! ```
//!
//! ## Notes
//!
//! Tables are rendered using UTF-8 box-drawing characters for a clean,
//! modern appearance. Content is dynamically arranged to fit the terminal width.

use comfy_table::{presets::UTF8_FULL, Cell, Color, ContentArrangement, Table};

/// Creates a new styled table with default settings.
///
/// The table is configured with:
/// - UTF-8 full border preset for clean visual appearance
/// - Dynamic content arrangement to fit terminal width
///
/// # Returns
///
/// A new [`Table`] instance with default styling applied.
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::table::create_table;
///
/// let mut table = create_table();
/// table.set_header(vec!["Column 1", "Column 2"]);
/// table.add_row(vec!["Value 1", "Value 2"]);
/// println!("{}", table);
/// ```
///
/// # Notes
///
/// For most use cases, prefer using [`TableBuilder`] which provides
/// a more ergonomic builder pattern interface.
pub fn create_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .set_content_arrangement(ContentArrangement::Dynamic);
    table
}

/// A builder for constructing formatted tables with a fluent API.
///
/// `TableBuilder` provides a convenient builder pattern for creating
/// tables with headers, rows, and optional color support.
///
/// # Features
///
/// - Fluent builder API for chaining operations
/// - Automatic header styling with cyan color when enabled
/// - Support for generic iterators for headers and row data
/// - Optional color toggle for non-terminal output
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::table::TableBuilder;
///
/// // Basic usage
/// TableBuilder::new()
///     .headers(["Name", "Value"])
///     .row(["foo", "1"])
///     .row(["bar", "2"])
///     .print();
///
/// // With color disabled
/// TableBuilder::new()
///     .color(false)
///     .headers(["Name", "Value"])
///     .rows(vec![
///         vec!["foo", "1"],
///         vec!["bar", "2"],
///     ])
///     .print();
/// ```
///
/// # Notes
///
/// The builder automatically detects terminal color support on creation.
/// Use the [`color`](TableBuilder::color) method to override this detection.
pub struct TableBuilder {
    table: Table,
    headers: Vec<String>,
    color: bool,
}

impl TableBuilder {
    /// Creates a new table builder with default settings.
    ///
    /// The builder is initialized with:
    /// - An empty table with UTF-8 borders
    /// - No headers
    /// - Color support auto-detected from terminal
    ///
    /// # Returns
    ///
    /// A new `TableBuilder` instance ready for configuration.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::table::TableBuilder;
    ///
    /// let builder = TableBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            table: create_table(),
            headers: Vec::new(),
            color: console::colors_enabled(),
        }
    }

    /// Sets whether color output is enabled.
    ///
    /// By default, color support is auto-detected based on terminal
    /// capabilities. Use this method to override the detection.
    ///
    /// # Parameters
    ///
    /// * `enabled` - `true` to enable colors, `false` to disable
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::table::TableBuilder;
    ///
    /// // Force colors off for piped output
    /// TableBuilder::new()
    ///     .color(false)
    ///     .headers(["Name"])
    ///     .print();
    /// ```
    pub fn color(mut self, enabled: bool) -> Self {
        self.color = enabled;
        self
    }

    /// Sets the table headers.
    ///
    /// Headers are displayed in cyan when color is enabled.
    /// This method accepts any iterator of string-like items.
    ///
    /// # Parameters
    ///
    /// * `headers` - An iterator of items that can be converted to `String`
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::table::TableBuilder;
    ///
    /// // Using an array
    /// TableBuilder::new()
    ///     .headers(["ID", "Name", "Status"])
    ///     .print();
    ///
    /// // Using a Vec
    /// let headers = vec!["ID", "Name", "Status"];
    /// TableBuilder::new()
    ///     .headers(headers)
    ///     .print();
    /// ```
    ///
    /// # Notes
    ///
    /// Headers should be set before adding rows for correct table structure.
    pub fn headers<I, S>(mut self, headers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.headers = headers.into_iter().map(|s| s.into()).collect();
        if self.color {
            let header_cells: Vec<Cell> = self
                .headers
                .iter()
                .map(|h| Cell::new(h).fg(Color::Cyan))
                .collect();
            self.table.set_header(header_cells);
        } else {
            self.table.set_header(&self.headers);
        }
        self
    }

    /// Adds a single row to the table.
    ///
    /// # Parameters
    ///
    /// * `cells` - An iterator of items that can be converted to `String`
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::table::TableBuilder;
    ///
    /// TableBuilder::new()
    ///     .headers(["Name", "Value"])
    ///     .row(["foo", "1"])
    ///     .row(["bar", "2"])
    ///     .row(["baz", "3"])
    ///     .print();
    /// ```
    ///
    /// # Notes
    ///
    /// The number of cells should match the number of headers for
    /// proper table alignment.
    pub fn row<I, S>(mut self, cells: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let row: Vec<String> = cells.into_iter().map(|s| s.into()).collect();
        self.table.add_row(row);
        self
    }

    /// Adds multiple rows to the table at once.
    ///
    /// This is a convenience method for adding several rows in one call.
    ///
    /// # Parameters
    ///
    /// * `rows` - An iterator of rows, where each row is an iterator of cells
    ///
    /// # Returns
    ///
    /// The builder instance for method chaining.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::table::TableBuilder;
    ///
    /// let data = vec![
    ///     vec!["repo1", "active"],
    ///     vec!["repo2", "archived"],
    ///     vec!["repo3", "active"],
    /// ];
    ///
    /// TableBuilder::new()
    ///     .headers(["Repository", "Status"])
    ///     .rows(data)
    ///     .print();
    /// ```
    pub fn rows<I, R, S>(mut self, rows: I) -> Self
    where
        I: IntoIterator<Item = R>,
        R: IntoIterator<Item = S>,
        S: Into<String>,
    {
        for row in rows {
            let row: Vec<String> = row.into_iter().map(|s| s.into()).collect();
            self.table.add_row(row);
        }
        self
    }

    /// Builds and prints the table to stdout.
    ///
    /// This is a terminal operation that consumes the builder.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::table::TableBuilder;
    ///
    /// TableBuilder::new()
    ///     .headers(["Name"])
    ///     .row(["Value"])
    ///     .print();
    /// ```
    pub fn print(self) {
        println!("{}", self.table);
    }

    /// Builds and returns the underlying table.
    ///
    /// Use this when you need direct access to the `comfy_table::Table`
    /// for further customization or non-stdout output.
    ///
    /// # Returns
    ///
    /// The constructed [`Table`] instance.
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// use bitbucket_cli::output::table::TableBuilder;
    ///
    /// let table = TableBuilder::new()
    ///     .headers(["Name"])
    ///     .row(["Value"])
    ///     .build();
    ///
    /// // Custom handling of the table
    /// let output = format!("{}", table);
    /// ```
    pub fn build(self) -> Table {
        self.table
    }
}

impl Default for TableBuilder {
    /// Creates a default `TableBuilder` instance.
    ///
    /// Equivalent to calling [`TableBuilder::new()`].
    ///
    /// # Returns
    ///
    /// A new `TableBuilder` with default settings.
    fn default() -> Self {
        Self::new()
    }
}

/// Formats a status string with semantic colors.
///
/// Different status values are colored based on their semantic meaning:
/// - **Green**: open, active, running, in_progress
/// - **Blue**: merged, completed, passed, successful
/// - **Red**: declined, closed, failed, error
/// - **Yellow**: draft, pending, waiting
///
/// # Parameters
///
/// * `status` - The status string to format
/// * `color` - Whether to apply color formatting
///
/// # Returns
///
/// The formatted status string, with ANSI color codes if `color` is `true`.
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::table::format_status;
///
/// // With color
/// let colored = format_status("open", true);
/// println!("{}", colored);  // Prints "open" in green
///
/// // Without color
/// let plain = format_status("open", false);
/// assert_eq!(plain, "open");
/// ```
///
/// # Notes
///
/// Status matching is case-insensitive. Unknown status values are
/// returned without color formatting.
pub fn format_status(status: &str, color: bool) -> String {
    if !color {
        return status.to_string();
    }

    use console::style;
    match status.to_lowercase().as_str() {
        "open" | "active" | "running" | "in_progress" => {
            style(status).green().to_string()
        }
        "merged" | "completed" | "passed" | "successful" => {
            style(status).blue().to_string()
        }
        "declined" | "closed" | "failed" | "error" => {
            style(status).red().to_string()
        }
        "draft" | "pending" | "waiting" => {
            style(status).yellow().to_string()
        }
        _ => status.to_string(),
    }
}

/// Formats a boolean value as a human-readable Yes/No string.
///
/// # Parameters
///
/// * `value` - The boolean value to format
/// * `color` - Whether to apply color formatting
///
/// # Returns
///
/// - `"Yes"` (green if colored) for `true`
/// - `"No"` (dimmed if colored) for `false`
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::table::format_bool;
///
/// // With color
/// println!("Private: {}", format_bool(true, true));   // "Yes" in green
/// println!("Archived: {}", format_bool(false, true)); // "No" dimmed
///
/// // Without color
/// assert_eq!(format_bool(true, false), "Yes");
/// assert_eq!(format_bool(false, false), "No");
/// ```
///
/// # Notes
///
/// This provides a more user-friendly representation of boolean values
/// compared to "true"/"false".
pub fn format_bool(value: bool, color: bool) -> String {
    if color {
        use console::style;
        if value {
            style("Yes").green().to_string()
        } else {
            style("No").dim().to_string()
        }
    } else if value {
        "Yes".to_string()
    } else {
        "No".to_string()
    }
}

/// Truncates a string to a maximum length with ellipsis.
///
/// If the string exceeds `max_len` characters, it is truncated and
/// "..." is appended. The total length of the result will be `max_len`.
///
/// # Parameters
///
/// * `s` - The string to truncate
/// * `max_len` - The maximum length of the result (including ellipsis)
///
/// # Returns
///
/// The original string if shorter than `max_len`, otherwise a truncated
/// string with "..." suffix.
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::table::truncate;
///
/// assert_eq!(truncate("hello", 10), "hello");
/// assert_eq!(truncate("hello world", 8), "hello...");
/// assert_eq!(truncate("hi", 3), "hi");
/// assert_eq!(truncate("hello", 3), "hel");  // No room for ellipsis
/// ```
///
/// # Notes
///
/// - If `max_len <= 3`, no ellipsis is added (not enough room)
/// - This function operates on bytes, not grapheme clusters, so results
///   may be unexpected with non-ASCII strings
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s[..max_len].to_string()
    }
}
