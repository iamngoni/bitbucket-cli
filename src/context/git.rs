//
//  bitbucket-cli
//  context/git.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # Git Repository Operations
//!
//! This module provides low-level git repository operations using the `git2` library.
//! It wraps common git operations needed for repository context detection.
//!
//! ## Overview
//!
//! The [`GitContext`] struct provides a safe interface for:
//! - Opening git repositories (current directory or specific path)
//! - Querying branch information
//! - Accessing remote URLs
//! - Discovering repository metadata
//!
//! ## Usage
//!
//! ```rust,no_run
//! use bitbucket_cli::context::GitContext;
//!
//! // Open repository in current directory
//! if let Ok(git) = GitContext::open() {
//!     println!("Current branch: {}", git.current_branch().unwrap());
//!
//!     if let Some(url) = git.origin_url().unwrap() {
//!         println!("Origin URL: {}", url);
//!     }
//! }
//! ```
//!
//! ## Notes
//!
//! - Uses libgit2 via the `git2` crate for reliable cross-platform support
//! - The `discover` function walks up directories to find the repository root
//! - All methods return `Result` types for proper error handling

use anyhow::Result;
use git2::Repository;

/// Provides access to git repository information and operations.
///
/// `GitContext` wraps a `git2::Repository` to provide convenient methods
/// for querying repository state. It is designed for read-only operations
/// needed during context resolution.
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::context::GitContext;
/// use std::path::Path;
///
/// // Open from current directory
/// let ctx = GitContext::open()?;
///
/// // Or open from a specific path
/// let ctx = GitContext::open_at(Path::new("/path/to/repo"))?;
///
/// // Query repository information
/// println!("Branch: {}", ctx.current_branch()?);
/// println!("Remotes: {:?}", ctx.remote_names()?);
/// # Ok::<(), anyhow::Error>(())
/// ```
///
/// # Notes
///
/// - The struct holds an open repository handle; create new instances as needed
/// - Methods may fail if the repository state changes during use
/// - Bare repositories are not fully supported (no working directory)
pub struct GitContext {
    /// The underlying git2 repository handle
    repo: Repository,
}

impl GitContext {
    /// Opens the git repository containing the current working directory.
    ///
    /// This method uses git's repository discovery mechanism, which walks up
    /// the directory tree from the current directory until it finds a `.git`
    /// directory or file.
    ///
    /// # Returns
    ///
    /// - `Ok(GitContext)` if a repository is found
    /// - `Err` if no repository is found or an error occurs
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::context::GitContext;
    ///
    /// match GitContext::open() {
    ///     Ok(ctx) => println!("Found repository at {:?}", ctx.root_dir()),
    ///     Err(e) => eprintln!("Not in a git repository: {}", e),
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - Works from any subdirectory within the repository
    /// - Also finds repositories with detached worktrees
    /// - Returns an error for bare repositories without a working directory
    pub fn open() -> Result<Self> {
        let repo = Repository::discover(".")?;
        Ok(Self { repo })
    }

    /// Opens a git repository at a specific filesystem path.
    ///
    /// Unlike [`open`](#method.open), this method opens the repository directly
    /// at the specified path without discovery. The path should point to either
    /// the repository root or the `.git` directory.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to the repository root directory
    ///
    /// # Returns
    ///
    /// - `Ok(GitContext)` if the repository is successfully opened
    /// - `Err` if the path is not a valid repository
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::context::GitContext;
    /// use std::path::Path;
    ///
    /// let path = Path::new("/home/user/projects/my-repo");
    /// let ctx = GitContext::open_at(path)?;
    /// println!("Opened repository: {}", ctx.current_branch()?);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Notes
    ///
    /// - Does not walk up directories; path must be exact
    /// - Useful when you know the exact repository location
    /// - Returns an error if the path doesn't contain a valid repository
    pub fn open_at(path: &std::path::Path) -> Result<Self> {
        let repo = Repository::open(path)?;
        Ok(Self { repo })
    }

    /// Returns the name of the currently checked out branch.
    ///
    /// Retrieves the short name of the current HEAD reference. For a branch
    /// named `refs/heads/feature/login`, this returns `"feature/login"`.
    ///
    /// # Returns
    ///
    /// - `Ok(String)` containing the branch name
    /// - `Err` if HEAD cannot be resolved (e.g., unborn repository)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::context::GitContext;
    ///
    /// let ctx = GitContext::open()?;
    /// let branch = ctx.current_branch()?;
    /// println!("You are on branch: {}", branch);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Notes
    ///
    /// - Returns `"HEAD"` if in a detached HEAD state
    /// - The shorthand strips the `refs/heads/` prefix for branches
    /// - May return tag names or commit SHAs in detached states
    pub fn current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;
        let name = head.shorthand().unwrap_or("HEAD");
        Ok(name.to_string())
    }

    /// Retrieves the URL for a specific remote by name.
    ///
    /// Looks up a remote by its name and returns its configured URL, if any.
    /// Remotes can have different URLs for fetching and pushing; this returns
    /// the fetch URL.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the remote (e.g., "origin", "upstream")
    ///
    /// # Returns
    ///
    /// - `Ok(Some(String))` if the remote exists and has a URL
    /// - `Ok(None)` if the remote doesn't exist
    /// - `Err` only for unexpected git errors
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::context::GitContext;
    ///
    /// let ctx = GitContext::open()?;
    ///
    /// // Check for upstream remote
    /// if let Some(url) = ctx.remote_url("upstream")? {
    ///     println!("Upstream remote: {}", url);
    /// } else {
    ///     println!("No upstream remote configured");
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Notes
    ///
    /// - Returns `None` for non-existent remotes (not an error)
    /// - The URL may be SSH, HTTPS, or any git-supported protocol
    /// - Push URLs (if different) are not returned by this method
    pub fn remote_url(&self, name: &str) -> Result<Option<String>> {
        match self.repo.find_remote(name) {
            Ok(remote) => Ok(remote.url().map(|s| s.to_string())),
            Err(_) => Ok(None),
        }
    }

    /// Retrieves the URL of the "origin" remote.
    ///
    /// Convenience method that calls [`remote_url`](#method.remote_url) with
    /// `"origin"` as the remote name. The origin remote is conventionally the
    /// primary upstream repository.
    ///
    /// # Returns
    ///
    /// - `Ok(Some(String))` if origin remote exists and has a URL
    /// - `Ok(None)` if no origin remote is configured
    /// - `Err` only for unexpected git errors
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::context::GitContext;
    ///
    /// let ctx = GitContext::open()?;
    ///
    /// match ctx.origin_url()? {
    ///     Some(url) => println!("Origin: {}", url),
    ///     None => println!("No origin remote configured"),
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Notes
    ///
    /// - "origin" is the default name for the clone source
    /// - Some workflows use different naming conventions
    /// - Use [`remote_url`](#method.remote_url) for other remote names
    pub fn origin_url(&self) -> Result<Option<String>> {
        self.remote_url("origin")
    }

    /// Returns a list of all configured remote names.
    ///
    /// Retrieves the names of all remotes configured in the repository.
    /// This is useful for discovering available remotes or iterating over
    /// all remote URLs.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<String>)` containing all remote names
    /// - `Err` if remotes cannot be listed
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::context::GitContext;
    ///
    /// let ctx = GitContext::open()?;
    /// let remotes = ctx.remote_names()?;
    ///
    /// println!("Configured remotes:");
    /// for name in &remotes {
    ///     if let Some(url) = ctx.remote_url(name)? {
    ///         println!("  {} -> {}", name, url);
    ///     }
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Notes
    ///
    /// - Returns an empty vector if no remotes are configured
    /// - Remote names are typically ASCII strings like "origin", "upstream"
    /// - The order of remotes is not guaranteed
    pub fn remote_names(&self) -> Result<Vec<String>> {
        let remotes = self.repo.remotes()?;
        Ok(remotes
            .iter()
            .flatten()
            .map(|s| s.to_string())
            .collect())
    }

    /// Returns the working directory path of the repository.
    ///
    /// The working directory is the root folder containing the repository's
    /// files (as opposed to the `.git` directory which contains git metadata).
    ///
    /// # Returns
    ///
    /// - `Some(&Path)` pointing to the working directory
    /// - `None` if this is a bare repository
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::context::GitContext;
    ///
    /// let ctx = GitContext::open()?;
    ///
    /// if let Some(root) = ctx.root_dir() {
    ///     println!("Repository root: {}", root.display());
    /// } else {
    ///     println!("This is a bare repository");
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Notes
    ///
    /// - Bare repositories (used on servers) have no working directory
    /// - The path is absolute and canonicalized
    /// - Useful for finding configuration files relative to repo root
    pub fn root_dir(&self) -> Option<&std::path::Path> {
        self.repo.workdir()
    }

    /// Checks if this context represents a valid git repository.
    ///
    /// Since `GitContext` can only be created from a valid repository, this
    /// method always returns `true`. It exists for API consistency and to
    /// make intent clear in code that checks repository validity.
    ///
    /// # Returns
    ///
    /// Always returns `true`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use bitbucket_cli::context::GitContext;
    ///
    /// let ctx = GitContext::open()?;
    /// assert!(ctx.is_git_repo()); // Always true if we get here
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Notes
    ///
    /// - For checking before opening, use [`is_in_git_repo`] instead
    /// - This method exists for semantic clarity in conditional logic
    /// - The repository handle may become invalid if `.git` is deleted
    pub fn is_git_repo(&self) -> bool {
        true // If we got here, we have a valid repo
    }
}

/// Checks if the current working directory is inside a git repository.
///
/// This is a convenience function that attempts to open a repository and
/// returns whether the operation succeeded. It's useful for quick checks
/// without needing the repository context.
///
/// # Returns
///
/// - `true` if the current directory is within a git repository
/// - `false` if not in a repository or an error occurs
///
/// # Example
///
/// ```rust,no_run
/// use bitbucket_cli::context::is_in_git_repo;
///
/// if is_in_git_repo() {
///     println!("This is a git repository");
/// } else {
///     println!("Not in a git repository");
/// }
/// ```
///
/// # Notes
///
/// - This function does not distinguish between "not a repo" and other errors
/// - For detailed error information, use `GitContext::open()` directly
/// - The check uses git's directory discovery mechanism
pub fn is_in_git_repo() -> bool {
    GitContext::open().is_ok()
}
