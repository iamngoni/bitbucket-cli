//
//  bitbucket-cli
//  output/json.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # JSON Output Formatting
//!
//! This module provides utilities for serializing data to JSON format,
//! designed for scripting and automation use cases where machine-readable
//! output is required.
//!
//! ## Features
//!
//! - Pretty-printed JSON output for readability
//! - Compact JSON output for minimal size
//! - JSON Lines (NDJSON) format for streaming
//! - Basic jq-style field extraction
//! - Writer-based output for flexible destinations
//!
//! ## Output Formats
//!
//! | Function | Description | Use Case |
//! |----------|-------------|----------|
//! | [`write_json`] | Pretty-printed JSON | Human-readable output |
//! | [`write_json_compact`] | Minified JSON | Piping to other tools |
//! | [`write_json_lines`] | One JSON object per line | Streaming processing |
//!
//! ## Example
//!
//! ```rust,ignore
//! use bitbucket_cli::output::json::{write_json, write_json_lines, apply_jq_filter};
//! use serde::Serialize;
//!
//! #[derive(Serialize)]
//! struct Repo { name: String }
//!
//! let repo = Repo { name: "my-repo".into() };
//!
//! // Pretty-printed output
//! write_json(&repo)?;
//!
//! // JSON lines for multiple items
//! let repos = vec![repo];
//! write_json_lines(&repos)?;
//!
//! // Extract a field
//! let json = r#"{"name": "test"}"#;
//! let name = apply_jq_filter(json, ".name")?;
//! ```
//!
//! ## Notes
//!
//! JSON output is ideal for:
//! - Piping to `jq` for further processing
//! - Parsing in shell scripts with tools like `jq` or `python -m json.tool`
//! - Integration with other CLI tools and automation pipelines

use serde::Serialize;
use std::io::{self, Write};

/// Writes a value as pretty-printed JSON to stdout.
///
/// The output is formatted with indentation and newlines for human
/// readability. Each top-level element starts on its own line.
///
/// # Parameters
///
/// * `value` - Any type implementing [`Serialize`]
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if serialization fails.
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::json::write_json;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Repository {
///     name: String,
///     is_private: bool,
/// }
///
/// let repo = Repository {
///     name: "my-repo".into(),
///     is_private: true,
/// };
///
/// write_json(&repo)?;
/// // Output:
/// // {
/// //   "name": "my-repo",
/// //   "is_private": true
/// // }
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The value cannot be serialized to JSON
/// - stdout is not writable
pub fn write_json<T: Serialize>(value: &T) -> anyhow::Result<()> {
    let json = serde_json::to_string_pretty(value)?;
    println!("{}", json);
    Ok(())
}

/// Writes a value as pretty-printed JSON to a custom writer.
///
/// This function allows writing JSON to any destination implementing
/// [`Write`], such as files, buffers, or network streams.
///
/// # Parameters
///
/// * `writer` - A mutable reference to any type implementing [`Write`]
/// * `value` - Any type implementing [`Serialize`]
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if serialization or writing fails.
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::json::write_json_to;
/// use std::io::Cursor;
///
/// let mut buffer = Cursor::new(Vec::new());
/// write_json_to(&mut buffer, &serde_json::json!({"key": "value"}))?;
///
/// let output = String::from_utf8(buffer.into_inner())?;
/// assert!(output.contains("\"key\": \"value\""));
/// ```
///
/// # Errors
///
/// Returns an error if:
/// - The value cannot be serialized to JSON
/// - Writing to the destination fails
pub fn write_json_to<W: Write, T: Serialize>(writer: &mut W, value: &T) -> anyhow::Result<()> {
    serde_json::to_writer_pretty(&mut *writer, value)?;
    writeln!(writer)?;
    Ok(())
}

/// Writes a value as compact (minified) JSON to stdout.
///
/// The output contains no extra whitespace, making it ideal for
/// piping to other programs or minimizing output size.
///
/// # Parameters
///
/// * `value` - Any type implementing [`Serialize`]
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if serialization fails.
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::json::write_json_compact;
///
/// write_json_compact(&serde_json::json!({
///     "name": "my-repo",
///     "is_private": true
/// }))?;
/// // Output: {"name":"my-repo","is_private":true}
/// ```
///
/// # Notes
///
/// Use this format when:
/// - Output will be parsed by another program
/// - Minimizing data transfer size is important
/// - Pretty-printing is not needed
///
/// # Errors
///
/// Returns an error if the value cannot be serialized to JSON.
pub fn write_json_compact<T: Serialize>(value: &T) -> anyhow::Result<()> {
    let json = serde_json::to_string(value)?;
    println!("{}", json);
    Ok(())
}

/// Writes a list of values as JSON Lines (NDJSON) to stdout.
///
/// JSON Lines is a format where each line contains a valid JSON object.
/// This format is ideal for streaming processing of multiple records.
///
/// # Parameters
///
/// * `values` - A slice of values implementing [`Serialize`]
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if serialization fails.
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::json::write_json_lines;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct Record { id: u32, name: String }
///
/// let records = vec![
///     Record { id: 1, name: "first".into() },
///     Record { id: 2, name: "second".into() },
/// ];
///
/// write_json_lines(&records)?;
/// // Output:
/// // {"id":1,"name":"first"}
/// // {"id":2,"name":"second"}
/// ```
///
/// # Notes
///
/// JSON Lines format is useful for:
/// - Processing with tools like `jq -c`
/// - Streaming large datasets line by line
/// - Log aggregation systems
/// - Data pipelines that process records individually
///
/// Each JSON object is written on its own line without pretty-printing.
///
/// # Errors
///
/// Returns an error if any value cannot be serialized to JSON.
pub fn write_json_lines<T: Serialize>(values: &[T]) -> anyhow::Result<()> {
    let stdout = io::stdout();
    let mut handle = stdout.lock();
    for value in values {
        serde_json::to_writer(&mut handle, value)?;
        writeln!(handle)?;
    }
    Ok(())
}

/// Applies a basic jq-style filter to a JSON string.
///
/// This is a simplified implementation supporting only basic operations:
/// - `.` - Identity filter (returns input unchanged)
/// - `.field` - Simple field extraction
///
/// # Parameters
///
/// * `json` - A valid JSON string to filter
/// * `filter` - A jq-style filter expression
///
/// # Returns
///
/// Returns the filtered JSON as a pretty-printed string.
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::json::apply_jq_filter;
///
/// let json = r#"{"name": "my-repo", "owner": {"username": "admin"}}"#;
///
/// // Identity filter
/// let result = apply_jq_filter(json, ".")?;
/// assert!(result.contains("my-repo"));
///
/// // Field extraction
/// let name = apply_jq_filter(json, ".name")?;
/// assert_eq!(name, "\"my-repo\"");
/// ```
///
/// # Supported Filters
///
/// | Filter | Description | Example |
/// |--------|-------------|---------|
/// | `.` | Identity | Returns input unchanged |
/// | `.field` | Field access | `.name` extracts `"name"` field |
///
/// # Errors
///
/// Returns an error if:
/// - The input is not valid JSON
/// - The filter is not supported
/// - The requested field does not exist
///
/// # Notes
///
/// For complex filtering, consider piping output to the actual `jq` command.
/// This implementation is intentionally limited to avoid introducing a
/// full jq parser dependency.
pub fn apply_jq_filter(json: &str, filter: &str) -> anyhow::Result<String> {
    // This is a placeholder - in a full implementation, you'd integrate
    // with a jq library or subprocess
    if filter == "." {
        return Ok(json.to_string());
    }

    // Basic field extraction for simple cases like ".field"
    if filter.starts_with('.') && !filter.contains(' ') {
        let field = &filter[1..];
        let value: serde_json::Value = serde_json::from_str(json)?;
        if let Some(result) = value.get(field) {
            return Ok(serde_json::to_string_pretty(result)?);
        }
    }

    anyhow::bail!("Complex jq filters not yet implemented. Filter: {}", filter)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_apply_jq_identity() {
        let json = r#"{"name": "test"}"#;
        let result = apply_jq_filter(json, ".").unwrap();
        assert_eq!(result, json);
    }

    #[test]
    fn test_apply_jq_field() {
        let json = r#"{"name": "test", "value": 42}"#;
        let result = apply_jq_filter(json, ".name").unwrap();
        assert_eq!(result, "\"test\"");
    }
}
