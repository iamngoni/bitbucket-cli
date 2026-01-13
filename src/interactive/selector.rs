//
//  bitbucket-cli
//  interactive/selector.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Interactive Selectors Module
//!
//! This module provides interactive selection components for choosing items
//! from lists in the terminal. It wraps the `dialoguer` crate to offer
//! single-select, multi-select, and fuzzy search selection interfaces.
//!
//! # Overview
//!
//! The module provides the following selector types:
//! - **Single Select** - Choose one item from a list using arrow keys
//! - **Fuzzy Select** - Choose one item with real-time fuzzy search filtering
//! - **Multi Select** - Choose multiple items with checkbox toggles
//! - **Paginated Selector** - Navigate large lists with pagination controls
//!
//! # Example
//!
//! ```no_run
//! use bitbucket_cli::interactive::selector::{select, fuzzy_select, multi_select};
//!
//! let repos = vec!["repo1", "repo2", "repo3"];
//!
//! // Simple selection
//! let idx = select("Choose repository:", &repos).unwrap();
//! println!("Selected: {}", repos[idx]);
//!
//! // Fuzzy search selection
//! let idx = fuzzy_select("Search repositories:", &repos).unwrap();
//!
//! // Multi-selection
//! let indices = multi_select("Select repositories to clone:", &repos).unwrap();
//! for idx in indices {
//!     println!("Will clone: {}", repos[idx]);
//! }
//! ```

use anyhow::Result;
use dialoguer::{FuzzySelect, MultiSelect, Select};

/// Prompts the user to select a single item from a list.
///
/// Displays a list of items and allows the user to navigate with arrow keys
/// and confirm selection with Enter. The list can be scrolled if it exceeds
/// the terminal height.
///
/// # Type Parameters
///
/// * `T` - The item type, which must implement `ToString` for display
///
/// # Parameters
///
/// * `message` - The prompt message displayed above the selection list
/// * `items` - A slice of items to choose from
///
/// # Returns
///
/// Returns `Ok(usize)` containing the zero-based index of the selected item.
/// Returns `Err` if the terminal interaction fails or is cancelled (e.g., Ctrl+C).
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::selector::select;
///
/// let branches = vec!["main", "develop", "feature/auth"];
/// let idx = select("Select branch:", &branches).unwrap();
/// println!("Checking out: {}", branches[idx]);
/// ```
///
/// # Notes
///
/// - Use arrow keys (Up/Down) to navigate
/// - Press Enter to confirm selection
/// - Press Escape or Ctrl+C to cancel
/// - For large lists, consider [`fuzzy_select`] for faster navigation
pub fn select<T: ToString>(message: &str, items: &[T]) -> Result<usize> {
    let selection = Select::new().with_prompt(message).items(items).interact()?;
    Ok(selection)
}

/// Prompts the user to select a single item with a pre-selected default.
///
/// Similar to [`select`], but with a cursor starting at the specified
/// default index, making it faster to accept common choices.
///
/// # Type Parameters
///
/// * `T` - The item type, which must implement `ToString` for display
///
/// # Parameters
///
/// * `message` - The prompt message displayed above the selection list
/// * `items` - A slice of items to choose from
/// * `default` - The zero-based index of the pre-selected item
///
/// # Returns
///
/// Returns `Ok(usize)` containing the zero-based index of the selected item.
/// Returns `Err` if the terminal interaction fails or the default index is out of bounds.
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::selector::select_with_default;
///
/// let envs = vec!["development", "staging", "production"];
/// // Pre-select "development" (index 0)
/// let idx = select_with_default("Select environment:", &envs, 0).unwrap();
/// ```
///
/// # Notes
///
/// - The default item is highlighted when the prompt appears
/// - Pressing Enter immediately selects the default
/// - The `default` index must be within bounds of `items`
pub fn select_with_default<T: ToString>(
    message: &str,
    items: &[T],
    default: usize,
) -> Result<usize> {
    let selection = Select::new()
        .with_prompt(message)
        .items(items)
        .default(default)
        .interact()?;
    Ok(selection)
}

/// Prompts the user to select an item using fuzzy search.
///
/// Displays a searchable list where the user can type to filter items
/// in real-time. Matching is fuzzy, meaning partial and out-of-order
/// character matches are supported.
///
/// # Type Parameters
///
/// * `T` - The item type, which must implement `ToString` for display and search
///
/// # Parameters
///
/// * `message` - The prompt message displayed above the search input
/// * `items` - A slice of items to search and choose from
///
/// # Returns
///
/// Returns `Ok(usize)` containing the zero-based index of the selected item.
/// Returns `Err` if the terminal interaction fails or is cancelled.
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::selector::fuzzy_select;
///
/// let repositories = vec![
///     "company/frontend-app",
///     "company/backend-api",
///     "company/shared-libs",
///     "personal/dotfiles",
/// ];
///
/// // User can type "front" or "app" to quickly find "company/frontend-app"
/// let idx = fuzzy_select("Search repositories:", &repositories).unwrap();
/// println!("Selected: {}", repositories[idx]);
/// ```
///
/// # Notes
///
/// - Start typing to filter the list
/// - Fuzzy matching allows for typos and partial matches
/// - Use arrow keys to navigate filtered results
/// - Ideal for large lists where scrolling would be slow
pub fn fuzzy_select<T: ToString>(message: &str, items: &[T]) -> Result<usize> {
    let selection = FuzzySelect::new()
        .with_prompt(message)
        .items(items)
        .interact()?;
    Ok(selection)
}

/// Prompts the user to select an item using fuzzy search with a default.
///
/// Combines fuzzy search with a pre-selected default item for cases
/// where a common choice should be quickly accessible.
///
/// # Type Parameters
///
/// * `T` - The item type, which must implement `ToString` for display and search
///
/// # Parameters
///
/// * `message` - The prompt message displayed above the search input
/// * `items` - A slice of items to search and choose from
/// * `default` - The zero-based index of the pre-selected item
///
/// # Returns
///
/// Returns `Ok(usize)` containing the zero-based index of the selected item.
/// Returns `Err` if the terminal interaction fails or the default is invalid.
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::selector::fuzzy_select_with_default;
///
/// let templates = vec!["rust", "python", "javascript", "go", "java"];
/// // Pre-select "rust" but allow searching
/// let idx = fuzzy_select_with_default("Select template:", &templates, 0).unwrap();
/// ```
///
/// # Notes
///
/// - The default is shown initially but typing starts a search
/// - Press Enter without typing to accept the default
pub fn fuzzy_select_with_default<T: ToString>(
    message: &str,
    items: &[T],
    default: usize,
) -> Result<usize> {
    let selection = FuzzySelect::new()
        .with_prompt(message)
        .items(items)
        .default(default)
        .interact()?;
    Ok(selection)
}

/// Prompts the user to select multiple items using checkboxes.
///
/// Displays a list with checkboxes where the user can toggle items
/// on/off and confirm the selection. All checked items are returned.
///
/// # Type Parameters
///
/// * `T` - The item type, which must implement `ToString` for display
///
/// # Parameters
///
/// * `message` - The prompt message displayed above the checkbox list
/// * `items` - A slice of items to choose from
///
/// # Returns
///
/// Returns `Ok(Vec<usize>)` containing the zero-based indices of all selected items.
/// The vector may be empty if no items were selected.
/// Returns `Err` if the terminal interaction fails.
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::selector::multi_select;
///
/// let files = vec!["README.md", "src/main.rs", "Cargo.toml", ".gitignore"];
/// let selected = multi_select("Select files to stage:", &files).unwrap();
///
/// for idx in &selected {
///     println!("Staging: {}", files[*idx]);
/// }
/// ```
///
/// # Notes
///
/// - Use arrow keys to navigate
/// - Press Space to toggle selection on/off
/// - Press Enter to confirm selection
/// - Press 'a' to toggle all items
pub fn multi_select<T: ToString>(message: &str, items: &[T]) -> Result<Vec<usize>> {
    let selections = MultiSelect::new()
        .with_prompt(message)
        .items(items)
        .interact()?;
    Ok(selections)
}

/// Prompts the user to select multiple items with pre-selected defaults.
///
/// Similar to [`multi_select`], but with some items pre-checked based
/// on a boolean array indicating the initial state of each checkbox.
///
/// # Type Parameters
///
/// * `T` - The item type, which must implement `ToString` for display
///
/// # Parameters
///
/// * `message` - The prompt message displayed above the checkbox list
/// * `items` - A slice of items to choose from
/// * `defaults` - A slice of booleans indicating which items should be pre-selected;
///   must have the same length as `items`
///
/// # Returns
///
/// Returns `Ok(Vec<usize>)` containing the zero-based indices of all selected items.
/// Returns `Err` if the terminal interaction fails.
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::selector::multi_select_with_defaults;
///
/// let reviewers = vec!["alice", "bob", "charlie", "diana"];
/// // Pre-select alice and charlie
/// let defaults = vec![true, false, true, false];
///
/// let selected = multi_select_with_defaults(
///     "Select reviewers:",
///     &reviewers,
///     &defaults
/// ).unwrap();
///
/// println!("Reviewers: {:?}", selected.iter().map(|&i| reviewers[i]).collect::<Vec<_>>());
/// ```
///
/// # Notes
///
/// - The `defaults` slice must match the length of `items`
/// - Pre-selected items appear checked when the prompt opens
/// - User can toggle any item regardless of default state
pub fn multi_select_with_defaults<T: ToString>(
    message: &str,
    items: &[T],
    defaults: &[bool],
) -> Result<Vec<usize>> {
    let selections = MultiSelect::new()
        .with_prompt(message)
        .items(items)
        .defaults(defaults)
        .interact()?;
    Ok(selections)
}

/// A paginated selector for navigating large lists.
///
/// When dealing with lists too large to display at once, this struct
/// provides pagination controls to navigate through pages of items
/// and select from the current page.
///
/// # Type Parameters
///
/// * `T` - The item type, which must implement `ToString` and `Clone`
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::interactive::selector::PaginatedSelector;
///
/// let repos: Vec<String> = (1..=100).map(|i| format!("repo-{}", i)).collect();
/// let mut selector = PaginatedSelector::new(repos.clone(), 10);
///
/// println!("Page 1 of {}", selector.total_pages());
/// let idx = selector.select("Choose repository:").unwrap();
/// println!("Selected: {}", repos[idx]);
///
/// // Navigate to next page
/// if selector.next_page() {
///     println!("Now on page 2");
/// }
/// ```
///
/// # Notes
///
/// - Items are divided into pages of `page_size` items each
/// - The last page may have fewer items than `page_size`
/// - Selection returns the global index, not the page-local index
pub struct PaginatedSelector<T> {
    /// The complete list of items to select from.
    items: Vec<T>,

    /// The number of items to display per page.
    page_size: usize,

    /// The zero-based index of the current page.
    current_page: usize,
}

impl<T: ToString + Clone> PaginatedSelector<T> {
    /// Creates a new paginated selector.
    ///
    /// Initializes the selector with the given items and page size,
    /// starting at the first page (page 0).
    ///
    /// # Parameters
    ///
    /// * `items` - The complete list of items to paginate
    /// * `page_size` - The maximum number of items to display per page
    ///
    /// # Returns
    ///
    /// Returns a new `PaginatedSelector` instance positioned at the first page.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bitbucket_cli::interactive::selector::PaginatedSelector;
    ///
    /// let items = vec!["a", "b", "c", "d", "e"];
    /// let selector = PaginatedSelector::new(items, 2);
    /// // Creates 3 pages: [a,b], [c,d], [e]
    /// ```
    ///
    /// # Notes
    ///
    /// - A page size of 0 will cause division by zero errors
    /// - Recommended page sizes are 5-20 depending on terminal height
    pub fn new(items: Vec<T>, page_size: usize) -> Self {
        Self {
            items,
            page_size,
            current_page: 0,
        }
    }

    /// Returns the total number of pages.
    ///
    /// Calculates how many pages are needed to display all items
    /// given the configured page size.
    ///
    /// # Returns
    ///
    /// Returns the total number of pages (always at least 1 for non-empty lists).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bitbucket_cli::interactive::selector::PaginatedSelector;
    ///
    /// let items: Vec<i32> = (1..=25).collect();
    /// let selector = PaginatedSelector::new(items, 10);
    /// assert_eq!(selector.total_pages(), 3); // [1-10], [11-20], [21-25]
    /// ```
    pub fn total_pages(&self) -> usize {
        self.items.len().div_ceil(self.page_size)
    }

    /// Returns a slice of items for the current page.
    ///
    /// Gets the subset of items that should be displayed on the
    /// current page.
    ///
    /// # Returns
    ///
    /// Returns a slice containing the items for the current page.
    /// The slice may be shorter than `page_size` on the last page.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bitbucket_cli::interactive::selector::PaginatedSelector;
    ///
    /// let items = vec!["a", "b", "c", "d", "e"];
    /// let selector = PaginatedSelector::new(items, 2);
    /// assert_eq!(selector.current_items(), &["a", "b"]);
    /// ```
    pub fn current_items(&self) -> &[T] {
        let start = self.current_page * self.page_size;
        let end = (start + self.page_size).min(self.items.len());
        &self.items[start..end]
    }

    /// Advances to the next page.
    ///
    /// Increments the current page index if not already on the last page.
    ///
    /// # Returns
    ///
    /// Returns `true` if navigation succeeded (moved to next page).
    /// Returns `false` if already on the last page (no change made).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bitbucket_cli::interactive::selector::PaginatedSelector;
    ///
    /// let items = vec!["a", "b", "c"];
    /// let mut selector = PaginatedSelector::new(items, 2);
    ///
    /// assert!(selector.next_page());  // Now on page 2 (last page)
    /// assert!(!selector.next_page()); // Already on last page
    /// ```
    pub fn next_page(&mut self) -> bool {
        if self.current_page < self.total_pages() - 1 {
            self.current_page += 1;
            true
        } else {
            false
        }
    }

    /// Returns to the previous page.
    ///
    /// Decrements the current page index if not already on the first page.
    ///
    /// # Returns
    ///
    /// Returns `true` if navigation succeeded (moved to previous page).
    /// Returns `false` if already on the first page (no change made).
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bitbucket_cli::interactive::selector::PaginatedSelector;
    ///
    /// let items = vec!["a", "b", "c"];
    /// let mut selector = PaginatedSelector::new(items, 2);
    ///
    /// assert!(!selector.prev_page()); // Already on first page
    /// selector.next_page();
    /// assert!(selector.prev_page());  // Back to first page
    /// ```
    pub fn prev_page(&mut self) -> bool {
        if self.current_page > 0 {
            self.current_page -= 1;
            true
        } else {
            false
        }
    }

    /// Prompts the user to select an item from the current page.
    ///
    /// Displays the items on the current page and returns the global
    /// index of the selected item (accounting for pagination offset).
    ///
    /// # Parameters
    ///
    /// * `message` - The prompt message displayed above the selection list
    ///
    /// # Returns
    ///
    /// Returns `Ok(usize)` containing the global zero-based index in the
    /// original items list (not the page-local index).
    /// Returns `Err` if the terminal interaction fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bitbucket_cli::interactive::selector::PaginatedSelector;
    ///
    /// let items = vec!["a", "b", "c", "d", "e"];
    /// let mut selector = PaginatedSelector::new(items.clone(), 2);
    ///
    /// selector.next_page(); // Page 2: ["c", "d"]
    ///
    /// // If user selects first item on page 2 ("c")
    /// let idx = selector.select("Choose:").unwrap();
    /// // idx == 2 (global index of "c"), not 0
    /// assert_eq!(items[idx], "c");
    /// ```
    ///
    /// # Notes
    ///
    /// - The returned index can be used directly with the original items list
    /// - Pagination navigation (next/prev) should be handled separately by the caller
    pub fn select(&self, message: &str) -> Result<usize> {
        let local_idx = select(message, self.current_items())?;
        Ok(self.current_page * self.page_size + local_idx)
    }
}
