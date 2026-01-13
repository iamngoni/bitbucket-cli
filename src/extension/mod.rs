//
//  bitbucket-cli
//  extension/mod.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! # CLI Extension System
//!
//! This module provides a flexible extension system for the Bitbucket CLI,
//! allowing users to extend functionality through external executables.
//!
//! ## Overview
//!
//! Extensions are external executables that follow the naming convention `bb-<extension-name>`.
//! When a user runs `bb <extension-name>`, the CLI locates and executes the corresponding
//! extension executable, passing any additional arguments.
//!
//! ## Extension Discovery
//!
//! Extensions are discovered in two locations:
//! - The dedicated extension directory (`~/.local/share/bb/extensions` on Linux)
//! - Any directory in the system `PATH`
//!
//! ## Supported Extension Types
//!
//! Extensions can be created in multiple languages:
//! - **Shell scripts**: Simple bash scripts for quick automation
//! - **Go**: Compiled Go binaries for cross-platform support
//! - **Rust**: Compiled Rust binaries for performance-critical extensions
//!
//! ## Example
//!
//! ```no_run
//! use bitbucket_cli::extension::ExtensionManager;
//!
//! // Create an extension manager
//! let manager = ExtensionManager::new().expect("Failed to create manager");
//!
//! // List all installed extensions
//! let extensions = manager.list().expect("Failed to list extensions");
//! for ext in extensions {
//!     println!("Found extension: {}", ext.name);
//! }
//!
//! // Find and execute a specific extension
//! if let Some(ext) = manager.find("my-extension").expect("Failed to find") {
//!     let exit_code = ext.execute(&["arg1".to_string()]).expect("Failed to execute");
//!     println!("Extension exited with code: {}", exit_code);
//! }
//! ```

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

/// Represents metadata and state for a CLI extension.
///
/// An `Extension` encapsulates all information about an installed extension,
/// including its location, origin, and version information. Extensions are
/// external executables that extend the CLI's functionality.
///
/// # Fields
///
/// * `name` - The extension name without the `bb-` prefix (e.g., "lint" for "bb-lint")
/// * `path` - Absolute path to the extension executable
/// * `precompiled` - Whether this is a precompiled binary (vs. interpreted script)
/// * `source` - The source repository URL if the extension was installed from a repo
/// * `pinned_version` - Specific version to use if the extension is pinned
///
/// # Example
///
/// ```no_run
/// use std::path::PathBuf;
/// use bitbucket_cli::extension::Extension;
///
/// // Create an extension from a discovered executable
/// let path = PathBuf::from("/usr/local/bin/bb-lint");
/// if let Some(ext) = Extension::from_path(path) {
///     println!("Found extension: {}", ext.name);
///
///     // Execute the extension
///     let exit_code = ext.execute(&["--fix".to_string()]).unwrap();
/// }
/// ```
///
/// # Notes
///
/// - Extension names must start with `bb-` to be recognized
/// - The `precompiled` field defaults to `true` when created via `from_path`
/// - Source and version information is only populated for extensions installed
///   via the extension manager
#[derive(Debug, Clone)]
pub struct Extension {
    /// The extension name without the `bb-` prefix.
    ///
    /// For an executable named `bb-lint`, this would be `"lint"`.
    pub name: String,

    /// Absolute path to the extension executable.
    ///
    /// This path is used when executing the extension via `Command`.
    pub path: PathBuf,

    /// Indicates whether this is a precompiled binary.
    ///
    /// - `true` for compiled executables (Go, Rust, C, etc.)
    /// - `false` for interpreted scripts (shell, Python, etc.)
    pub precompiled: bool,

    /// The source repository URL if known.
    ///
    /// This is populated when an extension is installed from a repository
    /// and is used for upgrade operations.
    pub source: Option<String>,

    /// The pinned version string if the extension is version-locked.
    ///
    /// When set, upgrade operations will target this specific version
    /// rather than the latest available.
    pub pinned_version: Option<String>,
}

impl Extension {
    /// Creates a new `Extension` instance from an executable path.
    ///
    /// This constructor validates that the path refers to a valid extension
    /// (filename starts with `bb-`) and extracts the extension name.
    ///
    /// # Parameters
    ///
    /// * `path` - The path to a potential extension executable
    ///
    /// # Returns
    ///
    /// * `Some(Extension)` - If the path is a valid extension (starts with `bb-`)
    /// * `None` - If the path is not a valid extension or has no filename
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    /// use bitbucket_cli::extension::Extension;
    ///
    /// // Valid extension path
    /// let ext = Extension::from_path(PathBuf::from("/usr/bin/bb-lint"));
    /// assert!(ext.is_some());
    /// assert_eq!(ext.unwrap().name, "lint");
    ///
    /// // Invalid path (missing bb- prefix)
    /// let ext = Extension::from_path(PathBuf::from("/usr/bin/my-tool"));
    /// assert!(ext.is_none());
    /// ```
    ///
    /// # Notes
    ///
    /// - The `precompiled` field is set to `true` by default
    /// - The `source` and `pinned_version` fields are set to `None`
    /// - The path is stored as-is without canonicalization
    pub fn from_path(path: PathBuf) -> Option<Self> {
        let name = path.file_name()?.to_str()?;
        if !name.starts_with("bb-") {
            return None;
        }

        let name = name.strip_prefix("bb-")?.to_string();

        Some(Self {
            name,
            path,
            precompiled: true,
            source: None,
            pinned_version: None,
        })
    }

    /// Executes the extension with the provided arguments.
    ///
    /// This method spawns a new process for the extension executable,
    /// passing the given arguments and waiting for completion.
    ///
    /// # Parameters
    ///
    /// * `args` - Command-line arguments to pass to the extension
    ///
    /// # Returns
    ///
    /// * `Ok(i32)` - The exit code from the extension process
    /// * `Err` - If the extension could not be executed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use std::path::PathBuf;
    /// use bitbucket_cli::extension::Extension;
    ///
    /// let ext = Extension::from_path(PathBuf::from("/usr/bin/bb-lint")).unwrap();
    ///
    /// // Execute with arguments
    /// let exit_code = ext.execute(&[
    ///     "--fix".to_string(),
    ///     "src/".to_string(),
    /// ]).expect("Failed to execute");
    ///
    /// if exit_code == 0 {
    ///     println!("Extension completed successfully");
    /// } else {
    ///     println!("Extension failed with code: {}", exit_code);
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - The extension inherits the current process's environment
    /// - Standard I/O streams are inherited (stdin, stdout, stderr)
    /// - If the process is terminated by a signal, exit code 1 is returned
    pub fn execute(&self, args: &[String]) -> Result<i32> {
        let status = Command::new(&self.path)
            .args(args)
            .status()
            .with_context(|| format!("Failed to execute extension: {}", self.name))?;

        Ok(status.code().unwrap_or(1))
    }
}

/// Manages the lifecycle of CLI extensions.
///
/// The `ExtensionManager` provides a centralized interface for discovering,
/// installing, updating, and removing CLI extensions. It handles both
/// locally installed extensions and those available in the system PATH.
///
/// # Responsibilities
///
/// - **Discovery**: Finding installed extensions in the extension directory and PATH
/// - **Installation**: Downloading and installing extensions from repositories
/// - **Removal**: Uninstalling extensions from the extension directory
/// - **Upgrade**: Updating installed extensions to newer versions
/// - **Scaffolding**: Creating new extension projects from templates
///
/// # Example
///
/// ```no_run
/// use bitbucket_cli::extension::ExtensionManager;
///
/// // Create a new extension manager
/// let manager = ExtensionManager::new().expect("Failed to create manager");
///
/// // List all installed extensions
/// for ext in manager.list().expect("Failed to list") {
///     println!("{}: {}", ext.name, ext.path.display());
/// }
///
/// // Find a specific extension
/// if let Some(ext) = manager.find("lint").expect("Failed to find") {
///     println!("Found lint extension at: {}", ext.path.display());
/// }
///
/// // Create a new extension scaffold
/// let project_dir = manager.create("my-ext", Some("rust"))
///     .expect("Failed to create scaffold");
/// println!("Created project at: {}", project_dir.display());
/// ```
///
/// # Notes
///
/// - The default extension directory is platform-specific:
///   - Linux: `~/.local/share/bb/extensions`
///   - macOS: `~/Library/Application Support/com.bitbucket.bb/extensions`
///   - Windows: `%APPDATA%\bitbucket\bb\data\extensions`
/// - Extensions in PATH take lower priority than those in the extension directory
/// - The manager does not require the extension directory to exist until installation
pub struct ExtensionManager {
    /// Directory where extensions are installed.
    ///
    /// This is the primary location for managed extensions. Extensions
    /// installed via `install()` are placed here.
    extension_dir: PathBuf,
}

impl ExtensionManager {
    /// Creates a new `ExtensionManager` instance.
    ///
    /// Initializes the manager with the platform-appropriate extension directory.
    /// The directory is determined using the `directories` crate for cross-platform
    /// compatibility.
    ///
    /// # Returns
    ///
    /// * `Ok(ExtensionManager)` - A configured extension manager
    /// * `Err` - If the extension directory path cannot be determined
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bitbucket_cli::extension::ExtensionManager;
    ///
    /// let manager = ExtensionManager::new().expect("Failed to create manager");
    /// ```
    ///
    /// # Notes
    ///
    /// - Does not create the extension directory; it is created on first install
    /// - Uses `directories::ProjectDirs` for platform-specific paths
    pub fn new() -> Result<Self> {
        let extension_dir = get_extension_dir()?;
        Ok(Self { extension_dir })
    }

    /// Lists all installed extensions.
    ///
    /// Discovers extensions from two sources:
    /// 1. The managed extension directory
    /// 2. Directories in the system PATH
    ///
    /// Extensions in the managed directory take priority; duplicates from PATH
    /// are excluded.
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<Extension>)` - List of discovered extensions (may be empty)
    /// * `Err` - If directory enumeration fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bitbucket_cli::extension::ExtensionManager;
    ///
    /// let manager = ExtensionManager::new().unwrap();
    /// let extensions = manager.list().expect("Failed to list extensions");
    ///
    /// println!("Found {} extensions:", extensions.len());
    /// for ext in &extensions {
    ///     println!("  - {} ({})", ext.name, ext.path.display());
    /// }
    /// ```
    ///
    /// # Notes
    ///
    /// - Returns an empty vector if no extensions are found
    /// - Silently skips directories that cannot be read
    /// - Does not validate that executables are actually runnable
    pub fn list(&self) -> Result<Vec<Extension>> {
        let mut extensions = Vec::new();

        if !self.extension_dir.exists() {
            return Ok(extensions);
        }

        for entry in std::fs::read_dir(&self.extension_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(ext) = Extension::from_path(path) {
                extensions.push(ext);
            }
        }

        // Also check PATH for bb-* executables
        if let Ok(path_var) = std::env::var("PATH") {
            for dir in std::env::split_paths(&path_var) {
                if let Ok(entries) = std::fs::read_dir(&dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if let Some(ext) = Extension::from_path(path) {
                            // Avoid duplicates
                            if !extensions.iter().any(|e| e.name == ext.name) {
                                extensions.push(ext);
                            }
                        }
                    }
                }
            }
        }

        Ok(extensions)
    }

    /// Finds an extension by name.
    ///
    /// Searches through all installed extensions to find one matching the
    /// given name (without the `bb-` prefix).
    ///
    /// # Parameters
    ///
    /// * `name` - The extension name without the `bb-` prefix
    ///
    /// # Returns
    ///
    /// * `Ok(Some(Extension))` - If an extension with the given name is found
    /// * `Ok(None)` - If no extension matches the name
    /// * `Err` - If listing extensions fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bitbucket_cli::extension::ExtensionManager;
    ///
    /// let manager = ExtensionManager::new().unwrap();
    ///
    /// match manager.find("lint").expect("Failed to search") {
    ///     Some(ext) => {
    ///         println!("Found: {}", ext.path.display());
    ///         ext.execute(&[]).expect("Failed to run");
    ///     }
    ///     None => println!("Extension 'lint' not found"),
    /// }
    /// ```
    pub fn find(&self, name: &str) -> Result<Option<Extension>> {
        let extensions = self.list()?;
        Ok(extensions.into_iter().find(|e| e.name == name))
    }

    /// Installs an extension from a repository.
    ///
    /// Downloads and installs an extension from a Bitbucket repository.
    /// The repository can be specified in several formats.
    ///
    /// # Parameters
    ///
    /// * `repo` - Repository URL or owner/repo identifier
    /// * `_pin` - Optional version to pin the extension to (currently unused)
    ///
    /// # Returns
    ///
    /// * `Ok(Extension)` - The newly installed extension
    /// * `Err` - If installation fails or is not yet implemented
    ///
    /// # Supported Repository Formats
    ///
    /// - `owner/repo` - Simple owner/repository format
    /// - `https://bitbucket.org/owner/repo` - Full HTTPS URL
    /// - `git@bitbucket.org:owner/repo.git` - SSH URL (partial support)
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bitbucket_cli::extension::ExtensionManager;
    ///
    /// let manager = ExtensionManager::new().unwrap();
    ///
    /// // Install from owner/repo format
    /// let ext = manager.install("myteam/bb-lint", None)
    ///     .expect("Failed to install");
    ///
    /// // Install with version pinning
    /// let ext = manager.install("myteam/bb-format", Some("v1.2.0"))
    ///     .expect("Failed to install");
    /// ```
    ///
    /// # Notes
    ///
    /// - Creates the extension directory if it does not exist
    /// - Currently returns an error as installation is not yet implemented
    /// - Future implementation will support downloading releases and building from source
    pub fn install(&self, repo: &str, _pin: Option<&str>) -> Result<Extension> {
        // Create extension directory if it doesn't exist
        std::fs::create_dir_all(&self.extension_dir)?;

        // Parse repository URL
        let (_owner, repo_name) = parse_repo_url(repo)?;
        let name = repo_name.strip_prefix("bb-").unwrap_or(&repo_name);

        // TODO: Actually download and install the extension
        // This would involve:
        // 1. Clone the repository or download release binary
        // 2. Build if necessary
        // 3. Copy to extension directory

        anyhow::bail!(
            "Extension installation from {} not yet implemented. \
            Extension name would be: {}",
            repo,
            name
        )
    }

    /// Removes an installed extension.
    ///
    /// Deletes the extension executable from the managed extension directory.
    /// This only removes extensions installed via the extension manager; it
    /// does not affect extensions in the system PATH.
    ///
    /// # Parameters
    ///
    /// * `name` - The extension name without the `bb-` prefix
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the extension was successfully removed
    /// * `Err` - If the extension is not found or removal fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bitbucket_cli::extension::ExtensionManager;
    ///
    /// let manager = ExtensionManager::new().unwrap();
    ///
    /// // Remove an extension
    /// manager.remove("lint").expect("Failed to remove extension");
    /// println!("Extension removed successfully");
    /// ```
    ///
    /// # Notes
    ///
    /// - Only removes extensions from the managed extension directory
    /// - Does not remove associated configuration or data files
    /// - Returns an error if the extension exists only in PATH
    pub fn remove(&self, name: &str) -> Result<()> {
        let ext_path = self.extension_dir.join(format!("bb-{}", name));

        if ext_path.exists() {
            std::fs::remove_file(&ext_path)?;
            Ok(())
        } else {
            anyhow::bail!("Extension not found: {}", name)
        }
    }

    /// Upgrades an installed extension to the latest version.
    ///
    /// Updates an extension by fetching the latest version from its source
    /// repository. Requires the extension to have source information recorded.
    ///
    /// # Parameters
    ///
    /// * `name` - The extension name without the `bb-` prefix
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the upgrade succeeds
    /// * `Err` - If the extension is not found, has no source, or upgrade fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bitbucket_cli::extension::ExtensionManager;
    ///
    /// let manager = ExtensionManager::new().unwrap();
    ///
    /// // Upgrade a specific extension
    /// manager.upgrade("lint").expect("Failed to upgrade");
    /// ```
    ///
    /// # Notes
    ///
    /// - Currently returns an error as upgrade is not yet implemented
    /// - Requires the extension to have been installed with source tracking
    /// - Pinned versions will be upgraded to the pinned version, not latest
    pub fn upgrade(&self, name: &str) -> Result<()> {
        let ext = self.find(name)?
            .ok_or_else(|| anyhow::anyhow!("Extension not found: {}", name))?;

        if ext.source.is_none() {
            anyhow::bail!("Cannot upgrade extension without source information: {}", name);
        }

        // TODO: Implement upgrade logic
        anyhow::bail!("Extension upgrade not yet implemented")
    }

    /// Creates a new extension project scaffold.
    ///
    /// Generates a starter project structure for developing a new extension.
    /// Supports multiple programming languages with appropriate boilerplate.
    ///
    /// # Parameters
    ///
    /// * `name` - The extension name without the `bb-` prefix
    /// * `precompiled` - Optional language for compiled extensions:
    ///   - `Some("go")` - Create a Go extension project
    ///   - `Some("rust")` - Create a Rust extension project
    ///   - `None` - Create a shell script extension
    ///
    /// # Returns
    ///
    /// * `Ok(PathBuf)` - Path to the created project directory
    /// * `Err` - If the directory exists or creation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bitbucket_cli::extension::ExtensionManager;
    ///
    /// let manager = ExtensionManager::new().unwrap();
    ///
    /// // Create a shell script extension
    /// let dir = manager.create("my-tool", None)
    ///     .expect("Failed to create");
    /// println!("Created shell extension at: {}", dir.display());
    ///
    /// // Create a Rust extension
    /// let dir = manager.create("fast-tool", Some("rust"))
    ///     .expect("Failed to create");
    /// println!("Created Rust extension at: {}", dir.display());
    ///
    /// // Create a Go extension
    /// let dir = manager.create("go-tool", Some("go"))
    ///     .expect("Failed to create");
    /// println!("Created Go extension at: {}", dir.display());
    /// ```
    ///
    /// # Generated Structure
    ///
    /// **Shell (default):**
    /// ```text
    /// bb-<name>/
    /// └── bb-<name>  (executable script)
    /// ```
    ///
    /// **Rust:**
    /// ```text
    /// bb-<name>/
    /// ├── Cargo.toml
    /// └── src/
    ///     └── main.rs
    /// ```
    ///
    /// **Go:**
    /// ```text
    /// bb-<name>/
    /// ├── go.mod
    /// └── main.go
    /// ```
    ///
    /// # Notes
    ///
    /// - Creates the project in the current working directory
    /// - The directory name will be `bb-<name>`
    /// - Returns an error if the directory already exists
    /// - Shell scripts are created with executable permissions on Unix
    pub fn create(&self, name: &str, precompiled: Option<&str>) -> Result<PathBuf> {
        let project_dir = std::env::current_dir()?.join(format!("bb-{}", name));

        if project_dir.exists() {
            anyhow::bail!("Directory already exists: {}", project_dir.display());
        }

        std::fs::create_dir_all(&project_dir)?;

        // Create basic extension structure based on language
        match precompiled {
            Some("go") => create_go_extension(&project_dir, name)?,
            Some("rust") => create_rust_extension(&project_dir, name)?,
            Some(lang) => anyhow::bail!("Unsupported language: {}", lang),
            None => create_shell_extension(&project_dir, name)?,
        }

        Ok(project_dir)
    }
}

impl Default for ExtensionManager {
    /// Creates a default `ExtensionManager` with fallback behavior.
    ///
    /// Attempts to create a manager with the standard extension directory.
    /// If that fails, falls back to a temporary directory.
    ///
    /// # Returns
    ///
    /// An `ExtensionManager` instance, using either:
    /// - The platform-specific extension directory (preferred)
    /// - `/tmp/bb-extensions` as a fallback
    ///
    /// # Example
    ///
    /// ```no_run
    /// use bitbucket_cli::extension::ExtensionManager;
    ///
    /// // Use default() when you don't need to handle initialization errors
    /// let manager = ExtensionManager::default();
    /// ```
    ///
    /// # Notes
    ///
    /// - Prefer `new()` when you need to handle initialization errors
    /// - The fallback directory is not persistent across reboots on most systems
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            extension_dir: PathBuf::from("/tmp/bb-extensions"),
        })
    }
}

/// Returns the platform-specific extension directory path.
///
/// Determines the appropriate directory for storing extensions based on
/// the current platform using the `directories` crate.
///
/// # Returns
///
/// * `Ok(PathBuf)` - The extension directory path
/// * `Err` - If the platform does not support standard directories
///
/// # Platform-Specific Paths
///
/// | Platform | Path |
/// |----------|------|
/// | Linux    | `~/.local/share/bb/extensions` |
/// | macOS    | `~/Library/Application Support/com.bitbucket.bb/extensions` |
/// | Windows  | `%APPDATA%\bitbucket\bb\data\extensions` |
///
/// # Notes
///
/// - This function does not create the directory
/// - The directory may not exist until the first extension is installed
fn get_extension_dir() -> Result<PathBuf> {
    let base = directories::ProjectDirs::from("com", "bitbucket", "bb")
        .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))?;

    Ok(base.data_dir().join("extensions"))
}

/// Parses a repository URL or identifier into owner and repository name.
///
/// Supports multiple repository reference formats commonly used with
/// Bitbucket repositories.
///
/// # Parameters
///
/// * `url` - The repository URL or identifier to parse
///
/// # Returns
///
/// * `Ok((owner, repo))` - Tuple containing the owner and repository name
/// * `Err` - If the URL format is not recognized
///
/// # Supported Formats
///
/// | Format | Example | Owner | Repo |
/// |--------|---------|-------|------|
/// | Simple | `myteam/bb-lint` | `myteam` | `bb-lint` |
/// | HTTPS  | `https://bitbucket.org/myteam/bb-lint` | `myteam` | `bb-lint` |
/// | HTTPS with .git | `https://bitbucket.org/myteam/bb-lint.git` | `myteam` | `bb-lint` |
///
/// # Example
///
/// ```ignore
/// let (owner, repo) = parse_repo_url("myteam/bb-lint")?;
/// assert_eq!(owner, "myteam");
/// assert_eq!(repo, "bb-lint");
///
/// let (owner, repo) = parse_repo_url("https://bitbucket.org/myteam/bb-lint.git")?;
/// assert_eq!(owner, "myteam");
/// assert_eq!(repo, "bb-lint");
/// ```
///
/// # Notes
///
/// - Whitespace is trimmed from the input
/// - The `.git` suffix is automatically stripped from URLs
/// - SSH URLs (`git@bitbucket.org:owner/repo.git`) are not fully supported
fn parse_repo_url(url: &str) -> Result<(String, String)> {
    // Handle various formats:
    // - owner/repo
    // - https://bitbucket.org/owner/repo
    // - git@bitbucket.org:owner/repo.git

    let url = url.trim();

    if url.contains('/') && !url.contains(':') && !url.contains("://") {
        // Simple owner/repo format
        let parts: Vec<&str> = url.split('/').collect();
        if parts.len() == 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    // Try to extract from URL
    let url = url.strip_suffix(".git").unwrap_or(url);

    if let Some(rest) = url.strip_prefix("https://bitbucket.org/") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() >= 2 {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
    }

    anyhow::bail!("Could not parse repository URL: {}", url)
}

/// Creates a shell script extension scaffold.
///
/// Generates a basic bash script extension with boilerplate code including
/// error handling and a placeholder for the extension logic.
///
/// # Parameters
///
/// * `dir` - The directory where the extension script will be created
/// * `name` - The extension name without the `bb-` prefix
///
/// # Returns
///
/// * `Ok(())` - If the script was created successfully
/// * `Err` - If file creation or permission setting fails
///
/// # Generated Structure
///
/// Creates a single executable file named `bb-<name>` containing:
/// - Shebang line for bash
/// - Comment header with usage information
/// - `set -e` for error handling
/// - Placeholder echo statement
///
/// # Example
///
/// ```ignore
/// use std::path::Path;
///
/// create_shell_extension(Path::new("/tmp/bb-hello"), "hello")?;
/// // Creates /tmp/bb-hello/bb-hello with:
/// // #!/bin/bash
/// // # bb-hello - A bb CLI extension
/// // ...
/// ```
///
/// # Notes
///
/// - On Unix systems, the script is made executable (mode 0o755)
/// - On Windows, the script will need manual permission adjustment
fn create_shell_extension(dir: &Path, name: &str) -> Result<()> {
    let script = format!(
        r#"#!/bin/bash
# bb-{name} - A bb CLI extension
#
# Usage: bb {name} [options]

set -e

# Your extension code here
echo "Hello from bb-{name}!"
"#,
        name = name
    );

    let script_path = dir.join(format!("bb-{}", name));
    std::fs::write(&script_path, script)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&script_path)?.permissions();
        perms.set_mode(0o755);
        std::fs::set_permissions(&script_path, perms)?;
    }

    Ok(())
}

/// Creates a Go extension project scaffold.
///
/// Generates a complete Go module structure with a main.go entry point
/// and go.mod file configured for the extension.
///
/// # Parameters
///
/// * `dir` - The directory where the Go project will be created
/// * `name` - The extension name without the `bb-` prefix
///
/// # Returns
///
/// * `Ok(())` - If all files were created successfully
/// * `Err` - If file creation fails
///
/// # Generated Structure
///
/// ```text
/// bb-<name>/
/// ├── go.mod     # Go module definition (Go 1.21+)
/// └── main.go    # Entry point with argument handling
/// ```
///
/// # Generated Code Features
///
/// The generated `main.go` includes:
/// - Package declaration
/// - Required imports (`fmt`, `os`)
/// - Main function with greeting and argument display
///
/// # Example
///
/// ```ignore
/// use std::path::Path;
///
/// create_go_extension(Path::new("/tmp/bb-mytool"), "mytool")?;
/// // Creates:
/// // - /tmp/bb-mytool/go.mod
/// // - /tmp/bb-mytool/main.go
/// ```
///
/// # Notes
///
/// - Targets Go 1.21 as the minimum version
/// - The module name matches the extension name (`bb-<name>`)
/// - Build with `go build` to create the extension binary
fn create_go_extension(dir: &Path, name: &str) -> Result<()> {
    let main_go = format!(
        r#"package main

import (
	"fmt"
	"os"
)

func main() {{
	fmt.Printf("Hello from bb-{name}!\n")
	fmt.Printf("Args: %v\n", os.Args[1:])
}}
"#,
        name = name
    );

    std::fs::write(dir.join("main.go"), main_go)?;

    let go_mod = format!(
        r#"module bb-{name}

go 1.21
"#,
        name = name
    );

    std::fs::write(dir.join("go.mod"), go_mod)?;

    Ok(())
}

/// Creates a Rust extension project scaffold.
///
/// Generates a complete Cargo project structure with a main.rs entry point
/// and Cargo.toml configured for the extension binary.
///
/// # Parameters
///
/// * `dir` - The directory where the Rust project will be created
/// * `name` - The extension name without the `bb-` prefix
///
/// # Returns
///
/// * `Ok(())` - If all files and directories were created successfully
/// * `Err` - If directory or file creation fails
///
/// # Generated Structure
///
/// ```text
/// bb-<name>/
/// ├── Cargo.toml    # Cargo manifest with binary configuration
/// └── src/
///     └── main.rs   # Entry point with argument handling
/// ```
///
/// # Generated Code Features
///
/// The generated `main.rs` includes:
/// - Main function with greeting message
/// - Command-line argument collection and display
/// - Conditional output (only shows args if provided)
///
/// The generated `Cargo.toml` includes:
/// - Package metadata (name, version, edition)
/// - Binary target configuration pointing to `src/main.rs`
///
/// # Example
///
/// ```ignore
/// use std::path::Path;
///
/// create_rust_extension(Path::new("/tmp/bb-fasttool"), "fasttool")?;
/// // Creates:
/// // - /tmp/bb-fasttool/Cargo.toml
/// // - /tmp/bb-fasttool/src/main.rs
/// ```
///
/// # Notes
///
/// - Uses Rust 2021 edition
/// - The binary name matches the extension name (`bb-<name>`)
/// - Build with `cargo build --release` for optimized binary
/// - The resulting binary will be at `target/release/bb-<name>`
fn create_rust_extension(dir: &Path, name: &str) -> Result<()> {
    let main_rs = format!(
        r#"fn main() {{
    println!("Hello from bb-{name}!");

    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {{
        println!("Args: {{:?}}", args);
    }}
}}
"#,
        name = name
    );

    std::fs::create_dir_all(dir.join("src"))?;
    std::fs::write(dir.join("src/main.rs"), main_rs)?;

    let cargo_toml = format!(
        r#"[package]
name = "bb-{name}"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "bb-{name}"
path = "src/main.rs"
"#,
        name = name
    );

    std::fs::write(dir.join("Cargo.toml"), cargo_toml)?;

    Ok(())
}
