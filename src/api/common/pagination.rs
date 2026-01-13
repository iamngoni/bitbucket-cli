//
//  bitbucket-cli
//  api/common/pagination.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Pagination Types for Bitbucket API Responses
//!
//! This module provides pagination structures for handling multi-page API responses
//! from both Bitbucket Cloud and Bitbucket Server/Data Center. Each platform uses
//! a different pagination strategy, and these types abstract those differences.
//!
//! # Overview
//!
//! | Type | Platform | Strategy |
//! |------|----------|----------|
//! | [`PaginatedResponse`] | Cloud | URL-based (next/previous links) |
//! | [`ServerPaginatedResponse`] | Server | Offset-based (start index) |
//!
//! # Cloud vs Server Pagination
//!
//! **Bitbucket Cloud** uses cursor-based pagination with `next` and `previous` URLs:
//! - Iterate by following the `next` URL until it's `None`
//! - More resilient to data changes during iteration
//!
//! **Bitbucket Server** uses offset-based pagination with `start` and `limit`:
//! - Request pages using `start` parameter (0-indexed)
//! - Check `isLastPage` or `nextPageStart` to determine if more pages exist
//!
//! # Example
//!
//! ```rust
//! use bitbucket_cli::api::common::{PaginatedResponse, ServerPaginatedResponse};
//!
//! // Cloud pagination
//! fn fetch_all_cloud_repos<T>(initial: PaginatedResponse<T>) -> Vec<T> {
//!     let mut all_items = initial.values;
//!     let mut response = initial;
//!
//!     while response.has_next() {
//!         // Fetch next page using response.next_url()
//!         // response = fetch(response.next_url().unwrap());
//!         // all_items.extend(response.values);
//!         break; // Simplified for example
//!     }
//!     all_items
//! }
//!
//! // Server pagination
//! fn fetch_all_server_repos<T>(initial: ServerPaginatedResponse<T>) -> Vec<T> {
//!     let mut all_items = initial.values;
//!     let mut response = initial;
//!
//!     while response.has_next() {
//!         // Fetch next page using response.next_start()
//!         // let start = response.next_start().unwrap();
//!         // response = fetch_with_start(start);
//!         // all_items.extend(response.values);
//!         break; // Simplified for example
//!     }
//!     all_items
//! }
//! ```
//!
//! # Notes
//!
//! - Both types implement `Clone` for easy state management during pagination
//! - Default values are used for optional fields to handle partial responses
//! - The `values` field is always present, even if empty

use serde::{Deserialize, Serialize};

/// Paginated response from Bitbucket Cloud API.
///
/// `PaginatedResponse` represents a single page of results from the Bitbucket Cloud
/// REST API (v2.0). Cloud uses URL-based pagination where each response includes
/// links to the next and previous pages.
///
/// # Type Parameters
///
/// - `T` - The type of items contained in the `values` array
///
/// # Fields
///
/// | Field | Type | Description |
/// |-------|------|-------------|
/// | `values` | `Vec<T>` | Array of items in the current page |
/// | `page` | `Option<u32>` | Current page number (1-indexed) |
/// | `pagelen` | `Option<u32>` | Number of items per page |
/// | `size` | `Option<u32>` | Total number of items across all pages |
/// | `next` | `Option<String>` | URL to fetch the next page |
/// | `previous` | `Option<String>` | URL to fetch the previous page |
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::api::common::PaginatedResponse;
/// use serde::Deserialize;
///
/// #[derive(Clone, Deserialize)]
/// struct Repository {
///     slug: String,
///     name: String,
/// }
///
/// // Parse a paginated response
/// let json = r#"{
///     "values": [{"slug": "repo1", "name": "Repository 1"}],
///     "page": 1,
///     "pagelen": 10,
///     "size": 25,
///     "next": "https://api.bitbucket.org/2.0/repositories?page=2"
/// }"#;
///
/// let response: PaginatedResponse<Repository> = serde_json::from_str(json).unwrap();
///
/// println!("Fetched {} of {} repositories", response.values.len(), response.size.unwrap_or(0));
///
/// if response.has_next() {
///     println!("Next page: {}", response.next_url().unwrap());
/// }
/// ```
///
/// # Notes
///
/// - The `size` field may not always be present for performance reasons
/// - Use [`has_next()`](Self::has_next) and [`next_url()`](Self::next_url) for iteration
/// - Page numbers are 1-indexed (first page is page 1)
/// - Default `pagelen` is typically 10 or 25 depending on the endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    /// Array of items in the current page.
    ///
    /// Contains the actual data for this page of results.
    /// May be empty if the query returned no results.
    pub values: Vec<T>,

    /// Current page number (1-indexed).
    ///
    /// The first page is page 1. This field may be absent in some
    /// API responses, particularly when using cursor-based pagination.
    #[serde(default)]
    pub page: Option<u32>,

    /// Number of items per page.
    ///
    /// This is the maximum number of items that can be returned in
    /// a single response. The actual number of items may be less.
    /// Common values are 10, 25, 50, or 100.
    #[serde(default)]
    pub pagelen: Option<u32>,

    /// Total number of items across all pages.
    ///
    /// When present, this indicates the total count of items matching
    /// the query. May be omitted for performance on large result sets.
    #[serde(default)]
    pub size: Option<u32>,

    /// URL to fetch the next page of results.
    ///
    /// When `None`, there are no more pages to fetch.
    /// This is a complete URL that can be used directly.
    #[serde(default)]
    pub next: Option<String>,

    /// URL to fetch the previous page of results.
    ///
    /// When `None`, this is the first page.
    /// This is a complete URL that can be used directly.
    #[serde(default)]
    pub previous: Option<String>,
}

impl<T> PaginatedResponse<T> {
    /// Checks if there are more pages of results available.
    ///
    /// Returns `true` if a `next` URL is present, indicating that additional
    /// pages can be fetched. Use this method to control pagination loops.
    ///
    /// # Returns
    ///
    /// - `true` - More pages are available; call [`next_url()`](Self::next_url) to get the URL
    /// - `false` - This is the last page; no more results available
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::api::common::PaginatedResponse;
    ///
    /// fn process_all_pages<T: Clone>(mut response: PaginatedResponse<T>) {
    ///     loop {
    ///         for item in &response.values {
    ///             // Process each item
    ///         }
    ///
    ///         if !response.has_next() {
    ///             break;
    ///         }
    ///
    ///         // Fetch next page using response.next_url().unwrap()
    ///         break; // Simplified for example
    ///     }
    /// }
    /// ```
    pub fn has_next(&self) -> bool {
        self.next.is_some()
    }

    /// Returns the URL for the next page of results.
    ///
    /// Provides access to the `next` URL as a string slice, avoiding
    /// the need to clone the URL string when only reading is required.
    ///
    /// # Returns
    ///
    /// - `Some(&str)` - The URL to fetch the next page
    /// - `None` - No next page available (this is the last page)
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::api::common::PaginatedResponse;
    ///
    /// fn fetch_next<T>(response: &PaginatedResponse<T>) {
    ///     if let Some(url) = response.next_url() {
    ///         println!("Fetching next page from: {}", url);
    ///         // Make HTTP request to `url`
    ///     } else {
    ///         println!("No more pages available");
    ///     }
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - Returns a borrowed string slice to avoid unnecessary allocation
    /// - Use [`has_next()`](Self::has_next) for simple boolean checks
    pub fn next_url(&self) -> Option<&str> {
        self.next.as_deref()
    }
}

/// Paginated response from Bitbucket Server/Data Center API.
///
/// `ServerPaginatedResponse` represents a single page of results from Bitbucket
/// Server or Data Center REST APIs. Server uses offset-based pagination with
/// explicit `start` and `limit` parameters.
///
/// # Type Parameters
///
/// - `T` - The type of items contained in the `values` array
///
/// # Fields
///
/// | Field | Type | Description |
/// |-------|------|-------------|
/// | `values` | `Vec<T>` | Array of items in the current page |
/// | `size` | `u32` | Number of items in the current page |
/// | `limit` | `u32` | Maximum items per page (requested) |
/// | `is_last_page` | `bool` | Whether this is the final page |
/// | `next_page_start` | `Option<u32>` | Start index for the next page |
/// | `start` | `u32` | Start index of the current page |
///
/// # Pagination Strategy
///
/// To iterate through all pages:
/// 1. Make initial request with `start=0` and desired `limit`
/// 2. Check [`has_next()`](Self::has_next) to see if more pages exist
/// 3. Use [`next_start()`](Self::next_start) to get the `start` value for the next request
/// 4. Repeat until `has_next()` returns `false`
///
/// # Example
///
/// ```rust
/// use bitbucket_cli::api::common::ServerPaginatedResponse;
/// use serde::Deserialize;
///
/// #[derive(Clone, Deserialize)]
/// struct Project {
///     key: String,
///     name: String,
/// }
///
/// // Parse a Server paginated response
/// let json = r#"{
///     "values": [{"key": "PROJ", "name": "My Project"}],
///     "size": 1,
///     "limit": 25,
///     "isLastPage": false,
///     "nextPageStart": 25,
///     "start": 0
/// }"#;
///
/// let response: ServerPaginatedResponse<Project> = serde_json::from_str(json).unwrap();
///
/// println!("Fetched {} projects (starting at {})", response.size, response.start);
///
/// if response.has_next() {
///     println!("Next page starts at: {}", response.next_start().unwrap());
/// }
/// ```
///
/// # Notes
///
/// - The `start` parameter is 0-indexed (first item is at index 0)
/// - The `limit` field reflects the requested page size, not the actual count
/// - The `size` field indicates how many items are in the current page
/// - When `is_last_page` is `true`, `next_page_start` will be `None`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerPaginatedResponse<T> {
    /// Array of items in the current page.
    ///
    /// Contains the actual data for this page of results.
    /// The length of this array equals the `size` field.
    pub values: Vec<T>,

    /// Number of items in the current page.
    ///
    /// This indicates how many items were returned in this response.
    /// Will be less than or equal to `limit`.
    #[serde(default)]
    pub size: u32,

    /// Maximum items per page (as requested).
    ///
    /// The page size limit that was used for this request.
    /// Typical values are 25, 50, or 100.
    #[serde(default)]
    pub limit: u32,

    /// Indicates whether this is the last page of results.
    ///
    /// When `true`, there are no more items to fetch.
    /// When `false`, use [`next_start()`](Self::next_start) to get the next page.
    #[serde(default, rename = "isLastPage")]
    pub is_last_page: bool,

    /// Start index for the next page of results.
    ///
    /// Use this value as the `start` parameter in the next API request.
    /// Will be `None` when `is_last_page` is `true`.
    #[serde(default, rename = "nextPageStart")]
    pub next_page_start: Option<u32>,

    /// Start index of the current page (0-indexed).
    ///
    /// The offset of the first item in this page.
    /// For the first page, this is always 0.
    #[serde(default)]
    pub start: u32,
}

impl<T> ServerPaginatedResponse<T> {
    /// Checks if there are more pages of results available.
    ///
    /// Returns `true` if this is not the last page, indicating that additional
    /// pages can be fetched using the [`next_start()`](Self::next_start) value.
    ///
    /// # Returns
    ///
    /// - `true` - More pages are available
    /// - `false` - This is the last page; no more results available
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::api::common::ServerPaginatedResponse;
    ///
    /// fn fetch_all_items<T: Clone>(mut response: ServerPaginatedResponse<T>) -> Vec<T> {
    ///     let mut all_items = response.values.clone();
    ///
    ///     while response.has_next() {
    ///         let next_start = response.next_start().unwrap();
    ///         // response = fetch_page(next_start);
    ///         // all_items.extend(response.values.clone());
    ///         break; // Simplified for example
    ///     }
    ///
    ///     all_items
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - This is the inverse of `is_last_page`
    /// - Prefer this method over checking `is_last_page` directly for clarity
    pub fn has_next(&self) -> bool {
        !self.is_last_page
    }

    /// Returns the start index for the next page of results.
    ///
    /// Provides the value to use as the `start` query parameter when
    /// requesting the next page from the Server API.
    ///
    /// # Returns
    ///
    /// - `Some(u32)` - The start index for the next page
    /// - `None` - No next page available (this is the last page)
    ///
    /// # Example
    ///
    /// ```rust
    /// use bitbucket_cli::api::common::ServerPaginatedResponse;
    ///
    /// fn build_next_url<T>(base_url: &str, response: &ServerPaginatedResponse<T>) -> Option<String> {
    ///     response.next_start().map(|start| {
    ///         format!("{}?start={}&limit={}", base_url, start, response.limit)
    ///     })
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - The returned value should be used as the `start` query parameter
    /// - Remember to also include the `limit` parameter in the next request
    /// - Returns `None` when [`has_next()`](Self::has_next) would return `false`
    pub fn next_start(&self) -> Option<u32> {
        self.next_page_start
    }
}
