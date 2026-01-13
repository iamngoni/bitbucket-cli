//
//  bitbucket-cli
//  cli/ssh_key.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! SSH key management commands
//!
//! This module provides commands for managing SSH keys for your account.
//! SSH keys are used for secure authentication when using Git over SSH.
//!
//! ## Examples
//!
//! ```bash
//! # List SSH keys
//! bb ssh-key list
//!
//! # Add a new SSH key
//! bb ssh-key add --title "Work Laptop" --key-file ~/.ssh/id_ed25519.pub
//!
//! # Delete an SSH key
//! bb ssh-key delete <key-id>
//!
//! # Test SSH connection
//! bb ssh-key test
//! ```

use std::fs;
use std::process::Command;

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use console::style;
use serde::{Deserialize, Serialize};

use crate::api::common::PaginatedResponse;
use crate::api::BitbucketClient;
use crate::auth::{AuthCredential, KeyringStore};
use crate::config::Config;
use crate::context::{ContextResolver, HostType};
use crate::output::OutputWriter;

use super::GlobalOptions;

/// Manage SSH keys
#[derive(Args, Debug)]
pub struct SshKeyCommand {
    #[command(subcommand)]
    pub command: SshKeySubcommand,
}

#[derive(Subcommand, Debug)]
pub enum SshKeySubcommand {
    /// List SSH keys
    #[command(visible_alias = "ls")]
    List,

    /// Add an SSH key
    Add(AddArgs),

    /// Delete an SSH key
    Delete(DeleteArgs),

    /// Test SSH connection
    Test,
}

#[derive(Args, Debug)]
pub struct AddArgs {
    /// Key title/label
    #[arg(long, short = 't')]
    pub title: String,

    /// SSH public key content
    #[arg(long, short = 'k', conflicts_with = "key_file")]
    pub key: Option<String>,

    /// Read key from file
    #[arg(long, short = 'f', conflicts_with = "key")]
    pub key_file: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// Key ID to delete
    pub id: String,

    /// Skip confirmation
    #[arg(long, short = 'y')]
    pub confirm: bool,
}

// API response types

#[derive(Debug, Deserialize)]
struct SshKey {
    uuid: String,
    key: String,
    label: String,
    #[serde(default)]
    created_on: Option<String>,
    #[serde(default)]
    last_used: Option<String>,
}

// Output types

#[derive(Debug, Serialize)]
struct SshKeyListItem {
    uuid: String,
    label: String,
    key_preview: String,
    created_on: Option<String>,
    last_used: Option<String>,
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
struct SshKeyDetail {
    uuid: String,
    label: String,
    key: String,
    created_on: Option<String>,
    last_used: Option<String>,
}

impl crate::output::TableOutput for SshKeyListItem {
    fn print_table(&self, _color: bool) {
        println!(
            "{:<38} {:<20} {}",
            &self.uuid,
            truncate(&self.label, 18),
            truncate(&self.key_preview, 30)
        );
    }

    fn print_markdown(&self) {
        println!("| {} | {} | {} |", self.uuid, self.label, self.key_preview);
    }
}

impl SshKeyCommand {
    pub async fn run(&self, global: &GlobalOptions) -> Result<()> {
        match &self.command {
            SshKeySubcommand::List => self.list(global).await,
            SshKeySubcommand::Add(args) => self.add(args, global).await,
            SshKeySubcommand::Delete(args) => self.delete(args, global).await,
            SshKeySubcommand::Test => self.test(global).await,
        }
    }

    fn get_host(&self, global: &GlobalOptions) -> Result<(String, HostType)> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);

        if let Ok(ctx) = resolver.resolve(global) {
            Ok((ctx.host, ctx.host_type))
        } else if let Some(host) = &global.host {
            let host_type = if host == "bitbucket.org" {
                HostType::Cloud
            } else {
                HostType::Server
            };
            Ok((host.clone(), host_type))
        } else {
            Ok(("bitbucket.org".to_string(), HostType::Cloud))
        }
    }

    fn get_client(&self, host: &str, host_type: &HostType) -> Result<BitbucketClient> {
        let client = match host_type {
            HostType::Cloud => BitbucketClient::cloud()?,
            HostType::Server => BitbucketClient::server(host)?,
        };

        let keyring = KeyringStore::new();
        let token = keyring
            .get(host)?
            .ok_or_else(|| anyhow::anyhow!("Not authenticated. Run 'bb auth login' first."))?;

        Ok(client.with_auth(AuthCredential::OAuth {
            access_token: token,
            refresh_token: None,
            expires_at: None,
        }))
    }

    fn get_format(&self, global: &GlobalOptions) -> crate::output::OutputFormat {
        if global.json {
            crate::output::OutputFormat::Json
        } else {
            crate::output::OutputFormat::Table
        }
    }

    /// List SSH keys
    async fn list(&self, global: &GlobalOptions) -> Result<()> {
        let (host, host_type) = self.get_host(global)?;
        let client = self.get_client(&host, &host_type)?;

        let url = match host_type {
            HostType::Cloud => "/user/ssh-keys".to_string(),
            HostType::Server => "/ssh/1.0/keys".to_string(),
        };

        let response: PaginatedResponse<SshKey> = client.get(&url).await?;

        let items: Vec<SshKeyListItem> = response
            .values
            .into_iter()
            .map(|key| {
                // Create key preview (first part + ...  + last part)
                let key_preview = if key.key.len() > 50 {
                    let parts: Vec<&str> = key.key.split_whitespace().collect();
                    if parts.len() >= 2 {
                        format!("{} {}...", parts[0], &parts[1][..20])
                    } else {
                        format!("{}...", &key.key[..40])
                    }
                } else {
                    key.key.clone()
                };

                SshKeyListItem {
                    uuid: key.uuid,
                    label: key.label,
                    key_preview,
                    created_on: key.created_on,
                    last_used: key.last_used,
                }
            })
            .collect();

        if items.is_empty() {
            println!("No SSH keys found.");
            println!();
            println!("Add an SSH key with:");
            println!("  bb ssh-key add --title \"My Key\" --key-file ~/.ssh/id_ed25519.pub");
            return Ok(());
        }

        let writer = OutputWriter::new(self.get_format(global));

        if !global.json {
            println!();
            println!("{}", style("SSH Keys").bold());
            println!("{}", "-".repeat(80));
            println!(
                "{} {} {}",
                style(format!("{:<38}", "UUID")).bold(),
                style(format!("{:<20}", "LABEL")).bold(),
                style("KEY").bold()
            );
            println!("{}", "-".repeat(80));

            for item in &items {
                println!(
                    "{:<38} {:<20} {}",
                    &item.uuid,
                    truncate(&item.label, 18),
                    truncate(&item.key_preview, 30)
                );
            }

            println!();
            println!("Showing {} SSH key(s)", items.len());
        } else {
            writer.write_list(&items)?;
        }

        Ok(())
    }

    /// Add an SSH key
    async fn add(&self, args: &AddArgs, global: &GlobalOptions) -> Result<()> {
        let (host, host_type) = self.get_host(global)?;
        let client = self.get_client(&host, &host_type)?;

        // Get the key content
        let key_content = if let Some(key) = &args.key {
            key.clone()
        } else if let Some(key_file) = &args.key_file {
            fs::read_to_string(key_file)?.trim().to_string()
        } else {
            bail!("Either --key or --key-file is required");
        };

        // Validate key format
        if !key_content.starts_with("ssh-") && !key_content.starts_with("ecdsa-") {
            bail!("Invalid SSH public key format. Key should start with 'ssh-' or 'ecdsa-'");
        }

        let url = match host_type {
            HostType::Cloud => "/user/ssh-keys".to_string(),
            HostType::Server => "/ssh/1.0/keys".to_string(),
        };

        let body = serde_json::json!({
            "key": key_content,
            "label": args.title,
        });

        let key: SshKey = client.post(&url, &body).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "uuid": key.uuid,
                "label": key.label,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Added SSH key {}",
                style("✓").green(),
                style(&args.title).cyan()
            );
            println!("  UUID: {}", key.uuid);
        }

        Ok(())
    }

    /// Delete an SSH key
    async fn delete(&self, args: &DeleteArgs, global: &GlobalOptions) -> Result<()> {
        let (host, host_type) = self.get_host(global)?;
        let client = self.get_client(&host, &host_type)?;

        // Confirm deletion
        if !args.confirm && !global.no_prompt {
            use dialoguer::Confirm;
            let confirmed = Confirm::new()
                .with_prompt(format!("Delete SSH key {}?", args.id))
                .default(false)
                .interact()?;

            if !confirmed {
                println!("{} Cancelled.", style("!").yellow());
                return Ok(());
            }
        }

        let url = match host_type {
            HostType::Cloud => format!("/user/ssh-keys/{}", args.id),
            HostType::Server => format!("/ssh/1.0/keys/{}", args.id),
        };

        client.delete(&url).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "uuid": args.id,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Deleted SSH key {}",
                style("✓").green(),
                style(&args.id).cyan()
            );
        }

        Ok(())
    }

    /// Test SSH connection
    async fn test(&self, global: &GlobalOptions) -> Result<()> {
        let (host, host_type) = self.get_host(global)?;

        let ssh_host = match host_type {
            HostType::Cloud => "git@bitbucket.org",
            HostType::Server => &format!("git@{}", host),
        };

        println!(
            "{} Testing SSH connection to {}...",
            style("→").cyan(),
            ssh_host
        );
        println!();

        // Run ssh -T to test connection
        let output = Command::new("ssh")
            .arg("-T")
            .arg("-o")
            .arg("StrictHostKeyChecking=no")
            .arg("-o")
            .arg("BatchMode=yes")
            .arg(ssh_host)
            .output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        // Bitbucket returns exit code 0 for successful auth
        // and prints authentication message to stderr
        let combined_output = format!("{}{}", stdout, stderr);

        if global.json {
            let result = serde_json::json!({
                "success": output.status.success() || combined_output.contains("logged in as"),
                "host": ssh_host,
                "output": combined_output.trim(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else if combined_output.contains("logged in as") {
            println!("{} SSH connection successful!", style("✓").green());
            // Extract username if present
            if let Some(start) = combined_output.find("logged in as") {
                let user_part = &combined_output[start..];
                if let Some(end) = user_part.find('.').or_else(|| user_part.find('\n')) {
                    println!("  {}", &user_part[..end]);
                }
            }
        } else if combined_output.contains("Permission denied") {
            println!(
                "{} SSH connection failed: Permission denied",
                style("✗").red()
            );
            println!();
            println!("Make sure you have:");
            println!("  1. Generated an SSH key pair");
            println!("  2. Added the public key to your Bitbucket account");
            println!();
            println!("Add a key with:");
            println!("  bb ssh-key add --title \"My Key\" --key-file ~/.ssh/id_ed25519.pub");
        } else {
            println!("{} SSH connection status unknown", style("?").yellow());
            if !combined_output.is_empty() {
                println!();
                println!("Output: {}", combined_output.trim());
            }
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
