//
//  bitbucket-cli
//  cli/config.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! CLI configuration commands
//!
//! This module provides commands for managing the CLI configuration,
//! including getting, setting, and listing configuration values.
//! Configuration can be set globally or per-host.

use std::process::Command;

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use console::style;

use crate::config::Config;

use super::GlobalOptions;

/// Valid core configuration keys
const VALID_CORE_KEYS: &[&str] = &[
    "editor",
    "pager",
    "browser",
    "git_protocol",
    "prompt",
];

/// Valid host configuration keys
const VALID_HOST_KEYS: &[&str] = &[
    "user",
    "default_workspace",
    "default_project",
    "api_version",
];

/// Manage CLI configuration
#[derive(Args, Debug)]
pub struct ConfigCommand {
    #[command(subcommand)]
    pub command: ConfigSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ConfigSubcommand {
    /// Get a configuration value
    Get(GetArgs),

    /// Set a configuration value
    Set(SetArgs),

    /// Unset a configuration value
    Unset(UnsetArgs),

    /// List all configuration values
    #[command(visible_alias = "ls")]
    List(ListArgs),

    /// Open configuration in editor
    Edit(EditArgs),

    /// Show configuration file path
    Path,
}

#[derive(Args, Debug)]
pub struct GetArgs {
    /// Configuration key
    pub key: String,

    /// Get value for specific host
    #[arg(long, short = 'H')]
    pub host: Option<String>,
}

#[derive(Args, Debug)]
pub struct SetArgs {
    /// Configuration key
    pub key: String,

    /// Configuration value
    pub value: String,

    /// Set value for specific host
    #[arg(long, short = 'H')]
    pub host: Option<String>,
}

#[derive(Args, Debug)]
pub struct UnsetArgs {
    /// Configuration key
    pub key: String,

    /// Unset value for specific host
    #[arg(long, short = 'H')]
    pub host: Option<String>,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// List configuration for specific host
    #[arg(long, short = 'H')]
    pub host: Option<String>,
}

#[derive(Args, Debug)]
pub struct EditArgs {
    /// Edit configuration for specific host
    #[arg(long, short = 'H')]
    pub host: Option<String>,
}

impl ConfigCommand {
    pub async fn run(&self, global: &GlobalOptions) -> Result<()> {
        match &self.command {
            ConfigSubcommand::Get(args) => self.get(args, global).await,
            ConfigSubcommand::Set(args) => self.set(args, global).await,
            ConfigSubcommand::Unset(args) => self.unset(args, global).await,
            ConfigSubcommand::List(args) => self.list(args, global).await,
            ConfigSubcommand::Edit(args) => self.edit(args, global).await,
            ConfigSubcommand::Path => self.path(global).await,
        }
    }

    /// Get a configuration value
    async fn get(&self, args: &GetArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load()?;

        if let Some(host) = &args.host {
            // Get host-specific config
            let value = self.get_host_value(&config, host, &args.key)?;

            if global.json {
                let result = serde_json::json!({
                    "key": args.key,
                    "value": value,
                    "host": host,
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else if let Some(v) = value {
                println!("{}", v);
            }
        } else {
            // Get core config
            let value = config.get(&args.key);

            if global.json {
                let result = serde_json::json!({
                    "key": args.key,
                    "value": value,
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else if let Some(v) = value {
                println!("{}", v);
            }
        }

        Ok(())
    }

    /// Set a configuration value
    async fn set(&self, args: &SetArgs, global: &GlobalOptions) -> Result<()> {
        let mut config = Config::load()?;

        if let Some(host) = &args.host {
            // Set host-specific config
            self.set_host_value(&mut config, host, &args.key, &args.value)?;
        } else {
            // Set core config
            if !VALID_CORE_KEYS.contains(&args.key.as_str()) {
                bail!(
                    "Unknown configuration key '{}'. Valid keys: {}",
                    args.key,
                    VALID_CORE_KEYS.join(", ")
                );
            }

            // Validate specific values
            if args.key == "git_protocol" && !["https", "ssh"].contains(&args.value.as_str()) {
                bail!("Invalid value for git_protocol. Valid values: https, ssh");
            }

            if args.key == "prompt" && !["enabled", "disabled"].contains(&args.value.as_str()) {
                bail!("Invalid value for prompt. Valid values: enabled, disabled");
            }

            config.set(&args.key, args.value.clone());
        }

        config.save()?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "key": args.key,
                "value": args.value,
                "host": args.host,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Set {} = {}",
                style("✓").green(),
                style(&args.key).cyan(),
                args.value
            );
        }

        Ok(())
    }

    /// Unset a configuration value
    async fn unset(&self, args: &UnsetArgs, global: &GlobalOptions) -> Result<()> {
        let mut config = Config::load()?;

        if let Some(host) = &args.host {
            // Unset host-specific config
            self.unset_host_value(&mut config, host, &args.key)?;
        } else {
            // Unset core config (set to default/None)
            match args.key.as_str() {
                "editor" => config.core.editor = None,
                "pager" => config.core.pager = None,
                "browser" => config.core.browser = None,
                "git_protocol" => config.core.git_protocol = "https".to_string(),
                "prompt" => config.core.prompt = "enabled".to_string(),
                _ => bail!(
                    "Unknown configuration key '{}'. Valid keys: {}",
                    args.key,
                    VALID_CORE_KEYS.join(", ")
                ),
            }
        }

        config.save()?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "key": args.key,
                "host": args.host,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Unset {}",
                style("✓").green(),
                style(&args.key).cyan()
            );
        }

        Ok(())
    }

    /// List all configuration values
    async fn list(&self, args: &ListArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load()?;

        if global.json {
            let result = if let Some(host) = &args.host {
                if let Some(host_config) = config.hosts.get(host) {
                    serde_json::json!({
                        "host": host,
                        "config": host_config,
                    })
                } else {
                    serde_json::json!({
                        "host": host,
                        "config": null,
                    })
                }
            } else {
                serde_json::json!({
                    "core": {
                        "editor": config.core.editor,
                        "pager": config.core.pager,
                        "browser": config.core.browser,
                        "git_protocol": config.core.git_protocol,
                        "prompt": config.core.prompt,
                    },
                    "hosts": config.hosts,
                    "aliases": config.aliases,
                })
            };
            println!("{}", serde_json::to_string_pretty(&result)?);
            return Ok(());
        }

        if let Some(host) = &args.host {
            // List host-specific config
            println!();
            println!("{}", style(format!("Configuration for host: {}", host)).bold());
            println!("{}", "-".repeat(50));

            if let Some(host_config) = config.hosts.get(host) {
                self.print_kv("user", &host_config.user);
                self.print_kv("default_workspace", &host_config.default_workspace);
                self.print_kv("default_project", &host_config.default_project);
                self.print_kv("api_version", &host_config.api_version);
            } else {
                println!("No configuration for this host.");
            }
        } else {
            // List all config
            println!();
            println!("{}", style("Core Configuration").bold());
            println!("{}", "-".repeat(50));
            self.print_kv("editor", &config.core.editor);
            self.print_kv("pager", &config.core.pager);
            self.print_kv("browser", &config.core.browser);
            self.print_kv_value("git_protocol", &config.core.git_protocol);
            self.print_kv_value("prompt", &config.core.prompt);

            if !config.hosts.is_empty() {
                println!();
                println!("{}", style("Host Configuration").bold());
                println!("{}", "-".repeat(50));
                for (host, host_config) in &config.hosts {
                    println!();
                    println!("  {}", style(host).cyan().bold());
                    if let Some(user) = &host_config.user {
                        println!("    user: {}", user);
                    }
                    if let Some(ws) = &host_config.default_workspace {
                        println!("    default_workspace: {}", ws);
                    }
                    if let Some(proj) = &host_config.default_project {
                        println!("    default_project: {}", proj);
                    }
                    if let Some(api) = &host_config.api_version {
                        println!("    api_version: {}", api);
                    }
                }
            }

            if !config.aliases.is_empty() {
                println!();
                println!("{}", style("Aliases").bold());
                println!("{}", "-".repeat(50));
                for (alias, command) in &config.aliases {
                    println!("  {} = {}", style(alias).cyan(), command);
                }
            }
        }

        println!();
        Ok(())
    }

    /// Open configuration in editor
    async fn edit(&self, args: &EditArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load()?;
        let config_path = Config::config_path()?;

        // Ensure config file exists
        if !config_path.exists() {
            config.save()?;
        }

        // Determine editor
        let editor = config.core.editor
            .or_else(|| std::env::var("EDITOR").ok())
            .or_else(|| std::env::var("VISUAL").ok())
            .unwrap_or_else(|| {
                if cfg!(target_os = "windows") {
                    "notepad".to_string()
                } else {
                    "vi".to_string()
                }
            });

        if global.json {
            let result = serde_json::json!({
                "action": "edit",
                "path": config_path.display().to_string(),
                "editor": editor,
                "host": args.host,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
            return Ok(());
        }

        println!(
            "{} Opening {} in {}...",
            style("→").cyan(),
            config_path.display(),
            editor
        );

        // Parse editor command (handle "code --wait" style editors)
        let parts: Vec<&str> = editor.split_whitespace().collect();
        let (cmd, cmd_args) = parts.split_first()
            .ok_or_else(|| anyhow::anyhow!("Invalid editor command"))?;

        let status = Command::new(cmd)
            .args(cmd_args)
            .arg(&config_path)
            .status()?;

        if !status.success() {
            bail!("Editor exited with non-zero status");
        }

        println!("{} Configuration saved.", style("✓").green());
        Ok(())
    }

    /// Show configuration file path
    async fn path(&self, global: &GlobalOptions) -> Result<()> {
        let config_path = Config::config_path()?;

        if global.json {
            let result = serde_json::json!({
                "path": config_path.display().to_string(),
                "exists": config_path.exists(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("{}", config_path.display());
        }

        Ok(())
    }

    // Helper methods

    fn get_host_value(&self, config: &Config, host: &str, key: &str) -> Result<Option<String>> {
        if !VALID_HOST_KEYS.contains(&key) {
            bail!(
                "Unknown host configuration key '{}'. Valid keys: {}",
                key,
                VALID_HOST_KEYS.join(", ")
            );
        }

        let host_config = config.hosts.get(host);

        Ok(match key {
            "user" => host_config.and_then(|h| h.user.clone()),
            "default_workspace" => host_config.and_then(|h| h.default_workspace.clone()),
            "default_project" => host_config.and_then(|h| h.default_project.clone()),
            "api_version" => host_config.and_then(|h| h.api_version.clone()),
            _ => None,
        })
    }

    fn set_host_value(&self, config: &mut Config, host: &str, key: &str, value: &str) -> Result<()> {
        if !VALID_HOST_KEYS.contains(&key) {
            bail!(
                "Unknown host configuration key '{}'. Valid keys: {}",
                key,
                VALID_HOST_KEYS.join(", ")
            );
        }

        let host_config = config.hosts.entry(host.to_string()).or_default();
        host_config.host = host.to_string();

        match key {
            "user" => host_config.user = Some(value.to_string()),
            "default_workspace" => host_config.default_workspace = Some(value.to_string()),
            "default_project" => host_config.default_project = Some(value.to_string()),
            "api_version" => host_config.api_version = Some(value.to_string()),
            _ => {}
        }

        Ok(())
    }

    fn unset_host_value(&self, config: &mut Config, host: &str, key: &str) -> Result<()> {
        if !VALID_HOST_KEYS.contains(&key) {
            bail!(
                "Unknown host configuration key '{}'. Valid keys: {}",
                key,
                VALID_HOST_KEYS.join(", ")
            );
        }

        if let Some(host_config) = config.hosts.get_mut(host) {
            match key {
                "user" => host_config.user = None,
                "default_workspace" => host_config.default_workspace = None,
                "default_project" => host_config.default_project = None,
                "api_version" => host_config.api_version = None,
                _ => {}
            }
        }

        Ok(())
    }

    fn print_kv(&self, key: &str, value: &Option<String>) {
        let display_value = value.as_deref().unwrap_or("-");
        println!("  {}: {}", style(key).cyan(), display_value);
    }

    fn print_kv_value(&self, key: &str, value: &str) {
        println!("  {}: {}", style(key).cyan(), value);
    }
}
