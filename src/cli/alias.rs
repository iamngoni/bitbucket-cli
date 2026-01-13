//
//  bitbucket-cli
//  cli/alias.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Alias commands
//!
//! This module provides commands for managing command aliases.
//! Aliases allow users to create shortcuts for frequently used commands.
//!
//! ## Examples
//!
//! ```bash
//! # Create an alias
//! bb alias set co "pr checkout"
//!
//! # Use the alias
//! bb co 123  # expands to: bb pr checkout 123
//!
//! # List aliases
//! bb alias list
//!
//! # Delete an alias
//! bb alias delete co
//! ```

use std::collections::HashMap;
use std::fs;
use std::path::Path;

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use console::style;
use serde::{Deserialize, Serialize};

use crate::config::Config;

use super::GlobalOptions;

/// Manage command aliases
#[derive(Args, Debug)]
pub struct AliasCommand {
    #[command(subcommand)]
    pub command: AliasSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum AliasSubcommand {
    /// Set an alias
    Set(SetArgs),

    /// Delete an alias
    Delete(DeleteArgs),

    /// List all aliases
    #[command(visible_alias = "ls")]
    List,

    /// Import aliases from file
    Import(ImportArgs),

    /// Export aliases to file
    Export(ExportArgs),
}

#[derive(Args, Debug)]
pub struct SetArgs {
    /// Alias name
    pub alias: String,

    /// Command expansion
    pub expansion: String,

    /// Create shell alias (wraps in shell execution)
    #[arg(long, short = 's')]
    pub shell: bool,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// Alias name to delete
    pub alias: String,
}

#[derive(Args, Debug)]
pub struct ImportArgs {
    /// File to import from (YAML, JSON, or TOML)
    pub file: String,

    /// Overwrite existing aliases
    #[arg(long)]
    pub overwrite: bool,
}

#[derive(Args, Debug)]
pub struct ExportArgs {
    /// File to export to
    pub file: String,

    /// Export format (json, yaml, toml)
    #[arg(long, short = 'f', default_value = "yaml")]
    pub format: String,
}

/// Alias definition for import/export
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AliasFile {
    aliases: HashMap<String, String>,
}

impl AliasCommand {
    pub async fn run(&self, global: &GlobalOptions) -> Result<()> {
        match &self.command {
            AliasSubcommand::Set(args) => self.set(args, global).await,
            AliasSubcommand::Delete(args) => self.delete(args, global).await,
            AliasSubcommand::List => self.list(global).await,
            AliasSubcommand::Import(args) => self.import(args, global).await,
            AliasSubcommand::Export(args) => self.export(args, global).await,
        }
    }

    /// Set an alias
    async fn set(&self, args: &SetArgs, global: &GlobalOptions) -> Result<()> {
        // Validate alias name
        if args.alias.is_empty() {
            bail!("Alias name cannot be empty");
        }

        if args.alias.contains(' ') || args.alias.contains('\t') {
            bail!("Alias name cannot contain whitespace");
        }

        // Reserved commands that can't be aliased
        let reserved = [
            "auth",
            "repo",
            "pr",
            "issue",
            "pipeline",
            "workspace",
            "project",
            "browse",
            "api",
            "config",
            "alias",
            "extension",
            "webhook",
            "deploy",
            "artifact",
            "secret",
            "ssh-key",
            "completion",
            "version",
            "help",
        ];

        if reserved.contains(&args.alias.as_str()) {
            bail!(
                "Cannot create alias '{}' - this is a reserved command name",
                args.alias
            );
        }

        let mut config = Config::load()?;

        // Handle shell aliases (prefix with !)
        let expansion = if args.shell {
            format!("!{}", args.expansion)
        } else {
            args.expansion.clone()
        };

        // Check if alias already exists
        let existed = config.aliases.contains_key(&args.alias);
        config.aliases.insert(args.alias.clone(), expansion.clone());
        config.save()?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "alias": args.alias,
                "expansion": expansion,
                "shell": args.shell,
                "updated": existed,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            let action = if existed { "Updated" } else { "Created" };
            let alias_type = if args.shell { "shell alias" } else { "alias" };
            println!(
                "{} {} {}: {} -> {}",
                style("✓").green(),
                action,
                alias_type,
                style(&args.alias).cyan().bold(),
                expansion
            );

            if args.shell {
                println!();
                println!(
                    "  {} Shell aliases execute in a subshell.",
                    style("Note:").yellow()
                );
            }
        }

        Ok(())
    }

    /// Delete an alias
    async fn delete(&self, args: &DeleteArgs, global: &GlobalOptions) -> Result<()> {
        let mut config = Config::load()?;

        if !config.aliases.contains_key(&args.alias) {
            if global.json {
                let result = serde_json::json!({
                    "success": false,
                    "error": format!("Alias '{}' not found", args.alias),
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                bail!("Alias '{}' not found", args.alias);
            }
            return Ok(());
        }

        let expansion = config.aliases.remove(&args.alias);
        config.save()?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "alias": args.alias,
                "expansion": expansion,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Deleted alias: {}",
                style("✓").green(),
                style(&args.alias).cyan().bold()
            );
        }

        Ok(())
    }

    /// List all aliases
    async fn list(&self, global: &GlobalOptions) -> Result<()> {
        let config = Config::load()?;

        if global.json {
            let result = serde_json::json!({
                "aliases": config.aliases,
                "count": config.aliases.len(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
            return Ok(());
        }

        if config.aliases.is_empty() {
            println!("No aliases configured.");
            println!();
            println!("Create an alias with:");
            println!("  bb alias set <name> <expansion>");
            println!();
            println!("Examples:");
            println!("  bb alias set co \"pr checkout\"");
            println!("  bb alias set pv \"pr view\"");
            println!("  bb alias set myrepos \"repo list --limit 10\"");
            return Ok(());
        }

        println!();
        println!("{}", style("Configured Aliases").bold());
        println!("{}", "-".repeat(60));

        // Sort aliases for consistent output
        let mut aliases: Vec<_> = config.aliases.iter().collect();
        aliases.sort_by(|a, b| a.0.cmp(b.0));

        for (alias, expansion) in aliases {
            let is_shell = expansion.starts_with('!');
            let display_expansion = if is_shell { &expansion[1..] } else { expansion };

            if is_shell {
                println!(
                    "  {} = {} {}",
                    style(alias).cyan().bold(),
                    display_expansion,
                    style("(shell)").dim()
                );
            } else {
                println!(
                    "  {} = bb {}",
                    style(alias).cyan().bold(),
                    display_expansion
                );
            }
        }

        println!();
        println!("Total: {} alias(es)", config.aliases.len());

        Ok(())
    }

    /// Import aliases from file
    async fn import(&self, args: &ImportArgs, global: &GlobalOptions) -> Result<()> {
        let path = Path::new(&args.file);

        if !path.exists() {
            bail!("File not found: {}", args.file);
        }

        let content = fs::read_to_string(path)?;

        // Detect format from extension or content
        let aliases: HashMap<String, String> = if args.file.ends_with(".json") {
            let file: AliasFile = serde_json::from_str(&content)?;
            file.aliases
        } else if args.file.ends_with(".toml") {
            let file: AliasFile = toml::from_str(&content)?;
            file.aliases
        } else if args.file.ends_with(".yaml") || args.file.ends_with(".yml") {
            let file: AliasFile = serde_yaml::from_str(&content)?;
            file.aliases
        } else {
            // Try to auto-detect
            if content.trim().starts_with('{') {
                let file: AliasFile = serde_json::from_str(&content)?;
                file.aliases
            } else if content.contains('[') && content.contains(']') {
                let file: AliasFile = toml::from_str(&content)?;
                file.aliases
            } else {
                let file: AliasFile = serde_yaml::from_str(&content)?;
                file.aliases
            }
        };

        let mut config = Config::load()?;
        let mut imported = 0;
        let mut skipped = 0;

        for (alias, expansion) in aliases {
            if config.aliases.contains_key(&alias) && !args.overwrite {
                skipped += 1;
                continue;
            }
            config.aliases.insert(alias, expansion);
            imported += 1;
        }

        config.save()?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "imported": imported,
                "skipped": skipped,
                "total": config.aliases.len(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Imported {} alias(es) from {}",
                style("✓").green(),
                imported,
                args.file
            );
            if skipped > 0 {
                println!(
                    "  {} {} existing alias(es) skipped (use --overwrite to replace)",
                    style("!").yellow(),
                    skipped
                );
            }
        }

        Ok(())
    }

    /// Export aliases to file
    async fn export(&self, args: &ExportArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load()?;

        if config.aliases.is_empty() {
            bail!("No aliases to export");
        }

        let file = AliasFile {
            aliases: config.aliases.clone(),
        };

        let content = match args.format.as_str() {
            "json" => serde_json::to_string_pretty(&file)?,
            "toml" => toml::to_string_pretty(&file)?,
            "yaml" | "yml" => serde_yaml::to_string(&file)?,
            _ => bail!(
                "Unsupported format: {}. Use json, yaml, or toml",
                args.format
            ),
        };

        fs::write(&args.file, &content)?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "file": args.file,
                "format": args.format,
                "count": config.aliases.len(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Exported {} alias(es) to {}",
                style("✓").green(),
                config.aliases.len(),
                args.file
            );
        }

        Ok(())
    }
}
