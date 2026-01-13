//
//  bitbucket-cli
//  cli/extension.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! CLI extension commands
//!
//! This module provides commands for managing CLI extensions.
//! Extensions are external executables that extend the CLI's functionality.
//!
//! ## Examples
//!
//! ```bash
//! # List installed extensions
//! bb extension list
//!
//! # Install an extension
//! bb extension install owner/bb-lint
//!
//! # Upgrade an extension
//! bb extension upgrade lint
//!
//! # Create a new extension
//! bb extension create my-tool --precompiled rust
//!
//! # Execute an extension
//! bb extension exec lint -- --fix
//! ```

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use console::style;
use serde::Serialize;

use crate::extension::ExtensionManager;

use super::GlobalOptions;

/// Manage CLI extensions
#[derive(Args, Debug)]
pub struct ExtensionCommand {
    #[command(subcommand)]
    pub command: ExtensionSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ExtensionSubcommand {
    /// List installed extensions
    #[command(visible_alias = "ls")]
    List,

    /// Install an extension
    Install(InstallArgs),

    /// Upgrade extensions
    Upgrade(UpgradeArgs),

    /// Remove an extension
    Remove(RemoveArgs),

    /// Create a new extension
    Create(CreateArgs),

    /// Browse available extensions
    Browse,

    /// Execute an extension
    Exec(ExecArgs),
}

#[derive(Args, Debug)]
pub struct InstallArgs {
    /// Extension repository (e.g., owner/bb-extension)
    pub repo: String,

    /// Pin to specific version
    #[arg(long)]
    pub pin: Option<String>,
}

#[derive(Args, Debug)]
pub struct UpgradeArgs {
    /// Extension to upgrade (or --all)
    pub extension: Option<String>,

    /// Upgrade all extensions
    #[arg(long)]
    pub all: bool,
}

#[derive(Args, Debug)]
pub struct RemoveArgs {
    /// Extension to remove
    pub extension: String,
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Extension name
    pub name: String,

    /// Create precompiled extension
    #[arg(long, value_parser = ["go", "rust"])]
    pub precompiled: Option<String>,
}

#[derive(Args, Debug)]
pub struct ExecArgs {
    /// Extension name
    pub extension: String,

    /// Arguments to pass to extension
    #[arg(trailing_var_arg = true)]
    pub args: Vec<String>,
}

// Output types

#[derive(Debug, Serialize)]
struct ExtensionListItem {
    name: String,
    path: String,
    precompiled: bool,
    source: Option<String>,
    pinned_version: Option<String>,
}

impl ExtensionCommand {
    pub async fn run(&self, global: &GlobalOptions) -> Result<()> {
        match &self.command {
            ExtensionSubcommand::List => self.list(global).await,
            ExtensionSubcommand::Install(args) => self.install(args, global).await,
            ExtensionSubcommand::Upgrade(args) => self.upgrade(args, global).await,
            ExtensionSubcommand::Remove(args) => self.remove(args, global).await,
            ExtensionSubcommand::Create(args) => self.create(args, global).await,
            ExtensionSubcommand::Browse => self.browse(global).await,
            ExtensionSubcommand::Exec(args) => self.exec(args, global).await,
        }
    }

    /// List installed extensions
    async fn list(&self, global: &GlobalOptions) -> Result<()> {
        let manager = ExtensionManager::new()?;
        let extensions = manager.list()?;

        if extensions.is_empty() {
            if global.json {
                println!("[]");
            } else {
                println!("No extensions installed.");
                println!();
                println!("Install extensions with:");
                println!("  bb extension install <owner/repo>");
                println!();
                println!("Create a new extension with:");
                println!("  bb extension create <name>");
            }
            return Ok(());
        }

        let items: Vec<ExtensionListItem> = extensions
            .into_iter()
            .map(|e| ExtensionListItem {
                name: e.name,
                path: e.path.display().to_string(),
                precompiled: e.precompiled,
                source: e.source,
                pinned_version: e.pinned_version,
            })
            .collect();

        if global.json {
            println!("{}", serde_json::to_string_pretty(&items)?);
        } else {
            println!();
            println!("{}", style("Installed Extensions").bold());
            println!("{}", "-".repeat(70));
            println!(
                "{} {} {}",
                style(format!("{:<20}", "NAME")).bold(),
                style(format!("{:<40}", "PATH")).bold(),
                style("TYPE").bold()
            );
            println!("{}", "-".repeat(70));

            for item in &items {
                let type_str = if item.precompiled { "binary" } else { "script" };
                println!(
                    "{:<20} {:<40} {}",
                    style(&item.name).cyan(),
                    truncate(&item.path, 38),
                    type_str
                );
            }

            println!();
            println!("{} extension(s) installed", items.len());
        }

        Ok(())
    }

    /// Install an extension
    async fn install(&self, args: &InstallArgs, global: &GlobalOptions) -> Result<()> {
        let manager = ExtensionManager::new()?;

        if !global.json {
            println!(
                "{} Installing extension from {}...",
                style("→").cyan(),
                args.repo
            );
            if let Some(version) = &args.pin {
                println!("  Pinned to version: {}", version);
            }
        }

        match manager.install(&args.repo, args.pin.as_deref()) {
            Ok(ext) => {
                if global.json {
                    let result = serde_json::json!({
                        "success": true,
                        "name": ext.name,
                        "path": ext.path.display().to_string(),
                    });
                    println!("{}", serde_json::to_string_pretty(&result)?);
                } else {
                    println!(
                        "{} Installed extension '{}'",
                        style("✓").green(),
                        style(&ext.name).cyan()
                    );
                    println!("  Path: {}", ext.path.display());
                }
                Ok(())
            }
            Err(e) => {
                if global.json {
                    let result = serde_json::json!({
                        "success": false,
                        "error": e.to_string(),
                    });
                    println!("{}", serde_json::to_string_pretty(&result)?);
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Upgrade extensions
    async fn upgrade(&self, args: &UpgradeArgs, global: &GlobalOptions) -> Result<()> {
        let manager = ExtensionManager::new()?;

        if args.all {
            // Upgrade all extensions
            let extensions = manager.list()?;

            if extensions.is_empty() {
                if !global.json {
                    println!("No extensions installed.");
                }
                return Ok(());
            }

            if !global.json {
                println!(
                    "{} Upgrading {} extension(s)...",
                    style("→").cyan(),
                    extensions.len()
                );
            }

            let mut results = Vec::new();
            for ext in &extensions {
                let result = manager.upgrade(&ext.name);
                results.push(serde_json::json!({
                    "name": ext.name,
                    "success": result.is_ok(),
                    "error": result.err().map(|e| e.to_string()),
                }));

                if !global.json {
                    match &results.last().unwrap()["success"].as_bool() {
                        Some(true) => {
                            println!("  {} Upgraded {}", style("✓").green(), ext.name);
                        }
                        _ => {
                            println!(
                                "  {} Failed to upgrade {}: {}",
                                style("✗").red(),
                                ext.name,
                                results.last().unwrap()["error"]
                                    .as_str()
                                    .unwrap_or("Unknown error")
                            );
                        }
                    }
                }
            }

            if global.json {
                println!("{}", serde_json::to_string_pretty(&results)?);
            }
        } else if let Some(name) = &args.extension {
            // Upgrade specific extension
            if !global.json {
                println!("{} Upgrading extension '{}'...", style("→").cyan(), name);
            }

            match manager.upgrade(name) {
                Ok(()) => {
                    if global.json {
                        let result = serde_json::json!({
                            "success": true,
                            "name": name,
                        });
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        println!("{} Upgraded '{}'", style("✓").green(), style(name).cyan());
                    }
                }
                Err(e) => {
                    if global.json {
                        let result = serde_json::json!({
                            "success": false,
                            "name": name,
                            "error": e.to_string(),
                        });
                        println!("{}", serde_json::to_string_pretty(&result)?);
                    } else {
                        bail!("Failed to upgrade '{}': {}", name, e);
                    }
                }
            }
        } else {
            bail!("Specify an extension name or use --all to upgrade all extensions");
        }

        Ok(())
    }

    /// Remove an extension
    async fn remove(&self, args: &RemoveArgs, global: &GlobalOptions) -> Result<()> {
        let manager = ExtensionManager::new()?;

        // Confirm removal
        if !global.no_prompt {
            use dialoguer::Confirm;
            let confirmed = Confirm::new()
                .with_prompt(format!("Remove extension '{}'?", args.extension))
                .default(false)
                .interact()?;

            if !confirmed {
                println!("{} Cancelled.", style("!").yellow());
                return Ok(());
            }
        }

        if !global.json {
            println!(
                "{} Removing extension '{}'...",
                style("→").cyan(),
                args.extension
            );
        }

        match manager.remove(&args.extension) {
            Ok(()) => {
                if global.json {
                    let result = serde_json::json!({
                        "success": true,
                        "name": args.extension,
                    });
                    println!("{}", serde_json::to_string_pretty(&result)?);
                } else {
                    println!(
                        "{} Removed extension '{}'",
                        style("✓").green(),
                        style(&args.extension).cyan()
                    );
                }
                Ok(())
            }
            Err(e) => {
                if global.json {
                    let result = serde_json::json!({
                        "success": false,
                        "name": args.extension,
                        "error": e.to_string(),
                    });
                    println!("{}", serde_json::to_string_pretty(&result)?);
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Create a new extension
    async fn create(&self, args: &CreateArgs, global: &GlobalOptions) -> Result<()> {
        let manager = ExtensionManager::new()?;

        if !global.json {
            let lang_str = args
                .precompiled
                .as_ref()
                .map(|l| format!(" ({} project)", l))
                .unwrap_or_else(|| " (shell script)".to_string());
            println!(
                "{} Creating extension 'bb-{}'{}...",
                style("→").cyan(),
                args.name,
                lang_str
            );
        }

        match manager.create(&args.name, args.precompiled.as_deref()) {
            Ok(path) => {
                if global.json {
                    let result = serde_json::json!({
                        "success": true,
                        "name": args.name,
                        "path": path.display().to_string(),
                        "type": args.precompiled.as_deref().unwrap_or("shell"),
                    });
                    println!("{}", serde_json::to_string_pretty(&result)?);
                } else {
                    println!(
                        "{} Created extension project at {}",
                        style("✓").green(),
                        style(path.display().to_string()).cyan()
                    );
                    println!();

                    match args.precompiled.as_deref() {
                        Some("rust") => {
                            println!("To build and install:");
                            println!("  cd bb-{}", args.name);
                            println!("  cargo build --release");
                            println!(
                                "  cp target/release/bb-{} ~/.local/share/bb/extensions/",
                                args.name
                            );
                        }
                        Some("go") => {
                            println!("To build and install:");
                            println!("  cd bb-{}", args.name);
                            println!("  go build");
                            println!("  cp bb-{} ~/.local/share/bb/extensions/", args.name);
                        }
                        _ => {
                            println!("To install:");
                            println!(
                                "  cp bb-{}/bb-{} ~/.local/share/bb/extensions/",
                                args.name, args.name
                            );
                        }
                    }
                }
                Ok(())
            }
            Err(e) => {
                if global.json {
                    let result = serde_json::json!({
                        "success": false,
                        "name": args.name,
                        "error": e.to_string(),
                    });
                    println!("{}", serde_json::to_string_pretty(&result)?);
                    Ok(())
                } else {
                    Err(e)
                }
            }
        }
    }

    /// Browse available extensions
    async fn browse(&self, global: &GlobalOptions) -> Result<()> {
        // Open the Bitbucket extension marketplace/topic page
        let url = "https://bitbucket.org/topics/bb-extension";

        if global.json {
            let result = serde_json::json!({
                "action": "browse",
                "url": url,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("{} Opening extension browser...", style("→").cyan());

            // Try to open in browser
            if let Err(e) = webbrowser::open(url) {
                println!("{} Could not open browser: {}", style("!").yellow(), e);
                println!("Browse extensions at: {}", url);
            } else {
                println!("{} Opened {} in browser", style("✓").green(), url);
            }
        }

        Ok(())
    }

    /// Execute an extension
    async fn exec(&self, args: &ExecArgs, global: &GlobalOptions) -> Result<()> {
        let manager = ExtensionManager::new()?;

        let ext = manager
            .find(&args.extension)?
            .ok_or_else(|| anyhow::anyhow!("Extension not found: {}", args.extension))?;

        if global.json {
            // In JSON mode, just report what would be executed
            let result = serde_json::json!({
                "action": "exec",
                "extension": args.extension,
                "path": ext.path.display().to_string(),
                "args": args.args,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
            return Ok(());
        }

        // Execute the extension
        let exit_code = ext.execute(&args.args)?;

        if exit_code != 0 {
            bail!(
                "Extension '{}' exited with code {}",
                args.extension,
                exit_code
            );
        }

        Ok(())
    }
}

/// Truncate string to max length with ellipsis
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() > max_len {
        format!("{}...", &s[..max_len - 3])
    } else {
        s.to_string()
    }
}
