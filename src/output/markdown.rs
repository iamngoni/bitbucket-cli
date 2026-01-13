//
//  bitbucket-cli
//  output/markdown.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # Markdown Output Formatting
//!
//! This module provides utilities for generating and rendering Markdown-formatted
//! output. It uses the `termimad` crate for terminal-friendly markdown rendering.
//!
//! ## Features
//!
//! - Terminal-aware markdown rendering with `termimad`
//! - Helper functions for common markdown elements
//! - Table generation with proper markdown syntax
//! - Text formatting (bold, italic, code)
//! - List generation (bullet and numbered)
//!
//! ## Supported Elements
//!
//! | Function | Element | Output |
//! |----------|---------|--------|
//! | [`md_header`] | Headers | `# Title` |
//! | [`md_bold`] | Bold text | `**text**` |
//! | [`md_italic`] | Italic text | `*text*` |
//! | [`md_inline_code`] | Inline code | `` `code` `` |
//! | [`md_code_block`] | Code blocks | ```` ```lang ... ``` ```` |
//! | [`md_list`] | Bullet lists | `- item` |
//! | [`md_numbered_list`] | Numbered lists | `1. item` |
//! | [`md_link`] | Links | `[text](url)` |
//! | [`md_table`] | Tables | `\| col \| col \|` |
//!
//! ## Example
//!
//! ```rust,ignore
//! use bitbucket_cli::output::markdown::*;
//!
//! // Generate a markdown document
//! let doc = format!(
//!     "{}\n\n{}\n\n{}",
//!     md_header(1, "Repository Report"),
//!     md_bold("Summary"),
//!     md_list(&["10 open PRs", "5 issues", "2 pending reviews"])
//! );
//!
//! // Render for terminal display
//! print_markdown(&doc);
//! ```
//!
//! ## Notes
//!
//! The markdown output is valid CommonMark and can be used with any markdown
//! processor. Terminal rendering applies ANSI styling for enhanced readability.

/// Renders markdown text for terminal display.
///
/// Uses the `termimad` crate to convert markdown to terminal-friendly output
/// with ANSI color codes and formatting.
///
/// # Parameters
///
/// * `text` - Markdown-formatted text to render
///
/// # Returns
///
/// A string with ANSI formatting codes for terminal display.
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::markdown::render_markdown;
///
/// let md = "# Hello\n\nThis is **bold** and *italic* text.";
/// let rendered = render_markdown(md);
/// println!("{}", rendered);
/// ```
///
/// # Notes
///
/// The output is optimized for terminal display. For raw markdown output
/// without rendering, use the formatting functions directly.
pub fn render_markdown(text: &str) -> String {
    // Use termimad for markdown rendering
    termimad::text(text).to_string()
}

/// Prints markdown text to the terminal with formatting.
///
/// This is a convenience function that calls [`render_markdown`] and
/// prints the result to stdout.
///
/// # Parameters
///
/// * `text` - Markdown-formatted text to print
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::markdown::print_markdown;
///
/// print_markdown("# Welcome\n\nThis is a **CLI** application.");
/// ```
///
/// # Notes
///
/// The output includes ANSI color codes. When piping to a file or another
/// program, consider using the raw markdown string instead.
pub fn print_markdown(text: &str) {
    println!("{}", render_markdown(text));
}

/// Creates a markdown header at the specified level.
///
/// # Parameters
///
/// * `level` - The header level (1-6)
/// * `text` - The header text
///
/// # Returns
///
/// A markdown header string with the appropriate number of `#` characters.
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::markdown::md_header;
///
/// assert_eq!(md_header(1, "Title"), "# Title");
/// assert_eq!(md_header(2, "Section"), "## Section");
/// assert_eq!(md_header(3, "Subsection"), "### Subsection");
/// ```
///
/// # Notes
///
/// Valid header levels are 1-6. Levels outside this range will still
/// produce output with the corresponding number of `#` characters, but
/// may not render correctly in all markdown processors.
pub fn md_header(level: u8, text: &str) -> String {
    let prefix = "#".repeat(level as usize);
    format!("{} {}", prefix, text)
}

/// Creates a markdown bullet list from an array of items.
///
/// # Parameters
///
/// * `items` - A slice of string items to list
///
/// # Returns
///
/// A markdown-formatted bullet list with each item prefixed by `- `.
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::markdown::md_list;
///
/// let list = md_list(&["First item", "Second item", "Third item"]);
/// assert_eq!(list, "- First item\n- Second item\n- Third item");
/// ```
///
/// # Notes
///
/// Items are separated by newlines. The function does not add a trailing
/// newline after the last item.
pub fn md_list(items: &[&str]) -> String {
    items
        .iter()
        .map(|item| format!("- {}", item))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Creates a markdown numbered list from an array of items.
///
/// # Parameters
///
/// * `items` - A slice of string items to list
///
/// # Returns
///
/// A markdown-formatted numbered list with items numbered starting from 1.
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::markdown::md_numbered_list;
///
/// let list = md_numbered_list(&["Clone repo", "Install deps", "Run tests"]);
/// assert_eq!(list, "1. Clone repo\n2. Install deps\n3. Run tests");
/// ```
///
/// # Notes
///
/// Numbers are assigned sequentially starting from 1. Many markdown
/// processors will auto-number lists regardless of the actual numbers used.
pub fn md_numbered_list(items: &[&str]) -> String {
    items
        .iter()
        .enumerate()
        .map(|(i, item)| format!("{}. {}", i + 1, item))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Creates a markdown fenced code block with syntax highlighting.
///
/// # Parameters
///
/// * `code` - The code content to display
/// * `language` - The language identifier for syntax highlighting (e.g., "rust", "json")
///
/// # Returns
///
/// A markdown fenced code block with the specified language.
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::markdown::md_code_block;
///
/// let block = md_code_block("fn main() {\n    println!(\"Hello!\");\n}", "rust");
/// // Output:
/// // ```rust
/// // fn main() {
/// //     println!("Hello!");
/// // }
/// // ```
/// ```
///
/// # Notes
///
/// Common language identifiers include:
/// - `rust`, `python`, `javascript`, `typescript`
/// - `json`, `yaml`, `toml`
/// - `bash`, `shell`, `sh`
/// - `sql`, `html`, `css`
///
/// Use an empty string for no syntax highlighting.
pub fn md_code_block(code: &str, language: &str) -> String {
    format!("```{}\n{}\n```", language, code)
}

/// Formats text as inline code.
///
/// # Parameters
///
/// * `text` - The text to format as code
///
/// # Returns
///
/// The text wrapped in backticks.
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::markdown::md_inline_code;
///
/// let code = md_inline_code("git status");
/// assert_eq!(code, "`git status`");
///
/// println!("Run {} to see changes", md_inline_code("git diff"));
/// ```
///
/// # Notes
///
/// Inline code is rendered in a monospace font. For multi-line code,
/// use [`md_code_block`] instead.
pub fn md_inline_code(text: &str) -> String {
    format!("`{}`", text)
}

/// Formats text as bold.
///
/// # Parameters
///
/// * `text` - The text to make bold
///
/// # Returns
///
/// The text wrapped in double asterisks.
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::markdown::md_bold;
///
/// let bold = md_bold("Important");
/// assert_eq!(bold, "**Important**");
/// ```
pub fn md_bold(text: &str) -> String {
    format!("**{}**", text)
}

/// Formats text as italic.
///
/// # Parameters
///
/// * `text` - The text to italicize
///
/// # Returns
///
/// The text wrapped in single asterisks.
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::markdown::md_italic;
///
/// let italic = md_italic("emphasis");
/// assert_eq!(italic, "*emphasis*");
/// ```
pub fn md_italic(text: &str) -> String {
    format!("*{}*", text)
}

/// Creates a markdown link.
///
/// # Parameters
///
/// * `text` - The link display text
/// * `url` - The URL to link to
///
/// # Returns
///
/// A markdown-formatted link.
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::markdown::md_link;
///
/// let link = md_link("Bitbucket", "https://bitbucket.org");
/// assert_eq!(link, "[Bitbucket](https://bitbucket.org)");
///
/// let pr_link = md_link("PR #42", "https://bitbucket.org/workspace/repo/pull-requests/42");
/// ```
///
/// # Notes
///
/// The URL should be properly encoded. Special characters in the text
/// or URL are not escaped automatically.
pub fn md_link(text: &str, url: &str) -> String {
    format!("[{}]({})", text, url)
}

/// Creates a markdown table from headers and rows.
///
/// Generates a properly formatted markdown table with a header row,
/// separator row, and data rows.
///
/// # Parameters
///
/// * `headers` - A slice of column headers
/// * `rows` - A slice of row vectors, where each vector contains cell values
///
/// # Returns
///
/// A markdown-formatted table string.
///
/// # Example
///
/// ```rust,ignore
/// use bitbucket_cli::output::markdown::md_table;
///
/// let table = md_table(
///     &["Name", "Status", "Branch"],
///     &[
///         vec!["PR #1", "Open", "feature/login"],
///         vec!["PR #2", "Merged", "fix/bug-123"],
///     ],
/// );
/// // Output:
/// // | Name | Status | Branch |
/// // | --- | --- | --- |
/// // | PR #1 | Open | feature/login |
/// // | PR #2 | Merged | fix/bug-123 |
/// ```
///
/// # Notes
///
/// - Cell values should not contain pipe characters (`|`) as they are
///   used as column delimiters
/// - All rows should have the same number of cells as headers
/// - The separator row uses `---` for each column
pub fn md_table(headers: &[&str], rows: &[Vec<&str>]) -> String {
    let mut result = String::new();

    // Header row
    result.push_str("| ");
    result.push_str(&headers.join(" | "));
    result.push_str(" |\n");

    // Separator row
    result.push_str("| ");
    result.push_str(&headers.iter().map(|_| "---").collect::<Vec<_>>().join(" | "));
    result.push_str(" |\n");

    // Data rows
    for row in rows {
        result.push_str("| ");
        result.push_str(&row.join(" | "));
        result.push_str(" |\n");
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_md_header() {
        assert_eq!(md_header(1, "Title"), "# Title");
        assert_eq!(md_header(2, "Section"), "## Section");
    }

    #[test]
    fn test_md_list() {
        let result = md_list(&["item1", "item2"]);
        assert_eq!(result, "- item1\n- item2");
    }

    #[test]
    fn test_md_table() {
        let result = md_table(
            &["Name", "Value"],
            &[vec!["foo", "1"], vec!["bar", "2"]],
        );
        assert!(result.contains("| Name | Value |"));
        assert!(result.contains("| foo | 1 |"));
    }
}
