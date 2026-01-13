//
//  bitbucket-cli
//  util/mod.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # Utility Module
//!
//! This module provides common utility functions used throughout the Bitbucket CLI
//! application. It includes functions for time formatting, string manipulation,
//! size conversions, browser interaction, and pager support.
//!
//! ## Categories
//!
//! - **Time Utilities**: [`format_time`], [`format_duration`], [`format_relative_time`]
//! - **String Utilities**: [`slugify`], [`truncate`]
//! - **Size Utilities**: [`parse_size`], [`format_size`]
//! - **System Utilities**: [`open_browser`], [`get_pager`], [`page_output`]
//!
//! ## Example
//!
//! ```rust
//! use bitbucket_cli::util::{format_relative_time, slugify, format_size};
//!
//! // Format timestamps for display
//! let relative = format_relative_time(1704067200); // "X days ago"
//!
//! // Create URL-safe slugs
//! let slug = slugify("My Feature Branch"); // "my-feature-branch"
//!
//! // Format file sizes
//! let size = format_size(1536); // "1.5 KB"
//! ```

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use chrono::{DateTime, Local};

/// Formats a Unix timestamp into a human-readable local datetime string.
///
/// Converts a Unix timestamp (seconds since epoch) into a formatted string
/// showing the date and time in the user's local timezone.
///
/// # Parameters
///
/// * `timestamp` - A Unix timestamp in seconds since the Unix epoch (1970-01-01 00:00:00 UTC).
///
/// # Returns
///
/// A `String` containing the formatted datetime in "YYYY-MM-DD HH:MM:SS" format,
/// or "Unknown" if the timestamp cannot be converted.
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::util::format_time;
///
/// let timestamp = 1704067200; // 2024-01-01 00:00:00 UTC
/// let formatted = format_time(timestamp);
/// // Returns something like "2024-01-01 02:00:00" depending on local timezone
/// ```
///
/// # Notes
///
/// - The output is always in the local timezone of the system running the CLI.
/// - Invalid timestamps (e.g., those that cannot be represented) return "Unknown".
/// - The format is fixed and designed for consistent display in CLI output.
pub fn format_time(timestamp: i64) -> String {
    if let Some(dt) = DateTime::from_timestamp(timestamp, 0) {
        let local: DateTime<Local> = dt.into();
        local.format("%Y-%m-%d %H:%M:%S").to_string()
    } else {
        "Unknown".to_string()
    }
}

/// Formats a duration into a human-readable compact string.
///
/// Converts a [`Duration`] into a concise, human-friendly format that
/// automatically selects the most appropriate time units based on the
/// magnitude of the duration.
///
/// # Parameters
///
/// * `duration` - A [`std::time::Duration`] representing the time span to format.
///
/// # Returns
///
/// A `String` containing the formatted duration using the most appropriate
/// units for readability.
///
/// # Example
///
/// ```rust
/// use std::time::Duration;
/// use bitbucket_cli::util::format_duration;
///
/// assert_eq!(format_duration(Duration::from_secs(45)), "45s");
/// assert_eq!(format_duration(Duration::from_secs(125)), "2m 5s");
/// assert_eq!(format_duration(Duration::from_secs(3665)), "1h 1m");
/// assert_eq!(format_duration(Duration::from_secs(90000)), "1d 1h");
/// ```
///
/// # Notes
///
/// - For durations under 60 seconds, shows only seconds (e.g., "45s").
/// - For durations under 1 hour, shows minutes and seconds (e.g., "2m 5s").
/// - For durations under 1 day, shows hours and minutes (e.g., "1h 1m").
/// - For longer durations, shows days and hours (e.g., "1d 1h").
/// - Nanosecond precision is ignored; only whole seconds are considered.
/// - Useful for displaying pipeline execution times, sync durations, etc.
pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();

    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m {}s", secs / 60, secs % 60)
    } else if secs < 86400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        format!("{}d {}h", secs / 86400, (secs % 86400) / 3600)
    }
}

/// Formats a Unix timestamp as a human-readable relative time string.
///
/// Converts a Unix timestamp into a natural language description of how long
/// ago that time occurred, relative to the current system time.
///
/// # Parameters
///
/// * `timestamp` - A Unix timestamp in seconds since the Unix epoch (1970-01-01 00:00:00 UTC).
///
/// # Returns
///
/// A `String` containing a human-readable relative time description such as
/// "just now", "5 minutes ago", "2 hours ago", "3 days ago", etc.
///
/// # Example
///
/// ```rust
/// use std::time::{SystemTime, UNIX_EPOCH};
/// use bitbucket_cli::util::format_relative_time;
///
/// // Get current time and subtract 2 hours
/// let now = SystemTime::now()
///     .duration_since(UNIX_EPOCH)
///     .unwrap()
///     .as_secs() as i64;
/// let two_hours_ago = now - 7200;
///
/// let relative = format_relative_time(two_hours_ago);
/// assert_eq!(relative, "2 hours ago");
/// ```
///
/// # Notes
///
/// - Returns "just now" for timestamps within the last 60 seconds.
/// - Returns "in the future" for timestamps ahead of the current time.
/// - Uses proper singular/plural forms (e.g., "1 minute ago" vs "2 minutes ago").
/// - Time ranges and their labels:
///   - < 60 seconds: "just now"
///   - < 1 hour: "X minute(s) ago"
///   - < 1 day: "X hour(s) ago"
///   - < 1 week: "X day(s) ago"
///   - < 30 days: "X week(s) ago"
///   - < 1 year: "X month(s) ago"
///   - >= 1 year: "X year(s) ago"
/// - Month and year calculations use approximate values (30 days/month, 365 days/year).
pub fn format_relative_time(timestamp: i64) -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);

    let diff = now - timestamp;

    if diff < 0 {
        return "in the future".to_string();
    }

    let diff = diff as u64;

    if diff < 60 {
        "just now".to_string()
    } else if diff < 3600 {
        let mins = diff / 60;
        format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
    } else if diff < 86400 {
        let hours = diff / 3600;
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if diff < 604800 {
        let days = diff / 86400;
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else if diff < 2592000 {
        let weeks = diff / 604800;
        format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" })
    } else if diff < 31536000 {
        let months = diff / 2592000;
        format!("{} month{} ago", months, if months == 1 { "" } else { "s" })
    } else {
        let years = diff / 31536000;
        format!("{} year{} ago", years, if years == 1 { "" } else { "s" })
    }
}

/// Converts a string into a URL-safe slug format.
///
/// Transforms an arbitrary string into a slug suitable for use in URLs,
/// file names, or identifiers. The resulting slug contains only lowercase
/// alphanumeric characters separated by single hyphens.
///
/// # Parameters
///
/// * `s` - The input string to convert into a slug.
///
/// # Returns
///
/// A `String` containing the slugified version of the input.
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::util::slugify;
///
/// assert_eq!(slugify("Hello World"), "hello-world");
/// assert_eq!(slugify("My Feature Branch!"), "my-feature-branch");
/// assert_eq!(slugify("foo--bar"), "foo-bar");
/// assert_eq!(slugify("test_123"), "test-123");
/// assert_eq!(slugify("  Multiple   Spaces  "), "multiple-spaces");
/// ```
///
/// # Notes
///
/// - Converts all characters to lowercase.
/// - Replaces all non-alphanumeric characters with hyphens.
/// - Collapses multiple consecutive hyphens into a single hyphen.
/// - Removes leading and trailing hyphens.
/// - Useful for creating branch names, repository slugs, or URL paths.
/// - Empty strings or strings with no alphanumeric characters return an empty string.
pub fn slugify(s: &str) -> String {
    s.to_lowercase()
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '-' })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

/// Truncates a string to a maximum length, adding an ellipsis if needed.
///
/// Shortens a string to fit within a specified character limit while adding
/// "..." to indicate truncation occurred. Useful for displaying long content
/// in constrained CLI output columns.
///
/// # Parameters
///
/// * `s` - The input string to truncate.
/// * `max_len` - The maximum allowed length of the output string, including the ellipsis.
///
/// # Returns
///
/// A `String` that is at most `max_len` characters long. If truncation was
/// necessary, the string ends with "...".
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::util::truncate;
///
/// assert_eq!(truncate("hello", 10), "hello");
/// assert_eq!(truncate("hello world", 8), "hello...");
/// assert_eq!(truncate("a very long description", 15), "a very long ...");
/// assert_eq!(truncate("short", 3), "sho"); // No room for ellipsis
/// ```
///
/// # Notes
///
/// - If the string is already within `max_len`, it is returned unchanged.
/// - When `max_len` is greater than 3, the ellipsis "..." is appended after truncation.
/// - When `max_len` is 3 or less, the string is simply cut without ellipsis (no room).
/// - This function operates on bytes, not grapheme clusters, which may cause issues
///   with multi-byte UTF-8 characters at truncation boundaries.
/// - The ellipsis always counts as 3 characters toward the `max_len` limit.
pub fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else if max_len > 3 {
        format!("{}...", &s[..max_len - 3])
    } else {
        s[..max_len].to_string()
    }
}

/// Opens a URL in the user's default web browser.
///
/// Launches the system's default browser application to display the specified
/// URL. This function uses platform-specific commands to ensure cross-platform
/// compatibility.
///
/// # Parameters
///
/// * `url` - The URL to open in the browser. Should be a fully qualified URL
///   including the protocol (e.g., "https://bitbucket.org/repo").
///
/// # Returns
///
/// Returns `Ok(())` if the browser was successfully launched, or an error if
/// the command failed to spawn.
///
/// # Errors
///
/// Returns an error if:
/// - The browser command fails to execute.
/// - The system command is not available.
/// - The URL is malformed (behavior depends on browser).
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::util::open_browser;
///
/// // Open a pull request in the browser
/// let pr_url = "https://bitbucket.org/myworkspace/myrepo/pull-requests/42";
/// if let Err(e) = open_browser(pr_url) {
///     eprintln!("Failed to open browser: {}", e);
/// }
/// ```
///
/// # Notes
///
/// - On macOS: Uses the `open` command.
/// - On Linux: Uses `xdg-open` (requires xdg-utils package).
/// - On Windows: Uses `cmd /c start` command.
/// - The function spawns the browser process and returns immediately without
///   waiting for the browser to close.
/// - URL validation is not performed; invalid URLs may cause browser-specific
///   error pages.
pub fn open_browser(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open").arg(url).spawn()?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open").arg(url).spawn()?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/c", "start", "", url])
            .spawn()?;
    }

    Ok(())
}

/// Retrieves the user's preferred pager command from environment variables.
///
/// Checks for a pager program to use when displaying long output. The function
/// first checks for a Bitbucket CLI-specific pager setting, then falls back to
/// the standard PAGER environment variable.
///
/// # Returns
///
/// Returns `Some(String)` containing the pager command if one is configured,
/// or `None` if no pager is set.
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::util::get_pager;
///
/// // Check if a pager is configured
/// match get_pager() {
///     Some(pager) => println!("Using pager: {}", pager),
///     None => println!("No pager configured, output will be printed directly"),
/// }
/// ```
///
/// # Notes
///
/// - First checks the `BB_PAGER` environment variable (Bitbucket CLI-specific).
/// - Falls back to the standard `PAGER` environment variable.
/// - Common pager values include "less", "more", "less -R" (with color support).
/// - Returns `None` if neither environment variable is set.
/// - Used internally by [`page_output`] to determine how to display long content.
pub fn get_pager() -> Option<String> {
    std::env::var("BB_PAGER")
        .ok()
        .or_else(|| std::env::var("PAGER").ok())
}

/// Displays content through the user's configured pager or prints directly.
///
/// Pipes the provided content through a pager program (like `less` or `more`)
/// if one is configured, otherwise prints the content directly to stdout.
/// This provides a better user experience for viewing long output.
///
/// # Parameters
///
/// * `content` - The text content to display.
///
/// # Returns
///
/// Returns `Ok(())` if the content was successfully displayed, or an error
/// if the pager process failed to execute or write.
///
/// # Errors
///
/// Returns an error if:
/// - The pager command fails to spawn.
/// - Writing to the pager's stdin fails.
/// - The pager process terminates with an error.
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::util::page_output;
///
/// // Display a long list of pull requests
/// let long_content = "PR #1: Feature A\nPR #2: Bug fix B\n...";
/// page_output(long_content)?;
/// ```
///
/// # Notes
///
/// - Uses [`get_pager`] to determine the pager command.
/// - If no pager is configured, content is printed directly to stdout.
/// - The function blocks until the pager is closed by the user.
/// - Useful for displaying long lists, diffs, or other scrollable content.
/// - Set `BB_PAGER=less -R` to support ANSI color codes in paged output.
/// - Set `BB_PAGER=cat` or unset both pager variables to disable paging.
pub fn page_output(content: &str) -> Result<()> {
    if let Some(pager) = get_pager() {
        use std::io::Write;
        use std::process::{Command, Stdio};

        let mut child = Command::new(&pager)
            .stdin(Stdio::piped())
            .spawn()?;

        if let Some(stdin) = child.stdin.as_mut() {
            stdin.write_all(content.as_bytes())?;
        }

        child.wait()?;
    } else {
        print!("{}", content);
    }

    Ok(())
}

/// Parses a human-readable size string into bytes.
///
/// Converts a size string with optional unit suffix (KB, MB, GB, B) into
/// the equivalent number of bytes. Useful for parsing user input or
/// configuration values.
///
/// # Parameters
///
/// * `s` - A string representing a size, with an optional unit suffix.
///   Supported units: B (bytes), KB (kilobytes), MB (megabytes), GB (gigabytes).
///
/// # Returns
///
/// Returns `Ok(u64)` containing the size in bytes, or an error if parsing fails.
///
/// # Errors
///
/// Returns an error if:
/// - The numeric portion cannot be parsed as a valid unsigned integer.
/// - The string is empty or contains only whitespace.
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::util::parse_size;
///
/// assert_eq!(parse_size("1024").unwrap(), 1024);
/// assert_eq!(parse_size("1KB").unwrap(), 1024);
/// assert_eq!(parse_size("1 KB").unwrap(), 1024);
/// assert_eq!(parse_size("2MB").unwrap(), 2 * 1024 * 1024);
/// assert_eq!(parse_size("1GB").unwrap(), 1024 * 1024 * 1024);
/// assert_eq!(parse_size("500B").unwrap(), 500);
/// assert_eq!(parse_size("100").unwrap(), 100); // Assumes bytes
/// ```
///
/// # Notes
///
/// - Unit suffixes are case-insensitive ("kb", "KB", "Kb" all work).
/// - Whitespace around the value and between number and unit is trimmed.
/// - Numbers without a unit suffix are treated as bytes.
/// - Uses binary units (1 KB = 1024 bytes, not 1000).
/// - Only integer values are supported; decimals will cause a parse error.
/// - The inverse operation is provided by [`format_size`].
pub fn parse_size(s: &str) -> Result<u64> {
    let s = s.trim().to_uppercase();

    let (num, unit) = if s.ends_with("GB") {
        (&s[..s.len()-2], 1024 * 1024 * 1024)
    } else if s.ends_with("MB") {
        (&s[..s.len()-2], 1024 * 1024)
    } else if s.ends_with("KB") {
        (&s[..s.len()-2], 1024)
    } else if s.ends_with("B") {
        (&s[..s.len()-1], 1)
    } else {
        (s.as_str(), 1)
    };

    let num: u64 = num.trim().parse()?;
    Ok(num * unit)
}

/// Formats a byte count as a human-readable size string.
///
/// Converts a raw byte count into a human-friendly format with the most
/// appropriate unit (B, KB, MB, GB). The output includes one decimal place
/// for units larger than bytes.
///
/// # Parameters
///
/// * `bytes` - The size in bytes to format.
///
/// # Returns
///
/// A `String` containing the formatted size with an appropriate unit suffix.
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::util::format_size;
///
/// assert_eq!(format_size(500), "500 B");
/// assert_eq!(format_size(1024), "1.0 KB");
/// assert_eq!(format_size(1536), "1.5 KB");
/// assert_eq!(format_size(1048576), "1.0 MB");
/// assert_eq!(format_size(1073741824), "1.0 GB");
/// assert_eq!(format_size(1610612736), "1.5 GB");
/// ```
///
/// # Notes
///
/// - Uses binary units (1 KB = 1024 bytes, 1 MB = 1024 KB, 1 GB = 1024 MB).
/// - Values less than 1 KB are shown as whole bytes without decimals.
/// - Larger values are shown with exactly one decimal place.
/// - Useful for displaying file sizes, repository sizes, attachment sizes, etc.
/// - The inverse operation is provided by [`parse_size`].
/// - Does not include TB or larger units; very large values show as many GB.
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.1} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slugify() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("foo--bar"), "foo-bar");
        assert_eq!(slugify("test_123"), "test-123");
    }

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world", 8), "hello...");
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 B");
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
    }

    #[test]
    fn test_parse_size() {
        assert_eq!(parse_size("1024").unwrap(), 1024);
        assert_eq!(parse_size("1KB").unwrap(), 1024);
        assert_eq!(parse_size("1MB").unwrap(), 1024 * 1024);
    }
}
