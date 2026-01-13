//
//  bitbucket-cli
//  cli/secret.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Secret and variable management commands
//!
//! This module provides commands for managing pipeline variables and secrets.
//! Secrets can be set at workspace, repository, or environment level.
//!
//! ## Examples
//!
//! ```bash
//! # List repository secrets
//! bb secret list
//!
//! # List workspace secrets
//! bb secret list --workspace myworkspace
//!
//! # Set a secret
//! bb secret set API_KEY --body "secret-value" --secured
//!
//! # Set from file
//! bb secret set CERTIFICATE --body-file ./cert.pem
//!
//! # Delete a secret
//! bb secret delete API_KEY
//!
//! # Sync from .env file
//! bb secret sync --file .env
//! ```

use std::fs;
use std::io::{self, BufRead};

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use console::style;
use serde::{Deserialize, Serialize};

use crate::api::common::PaginatedResponse;
use crate::api::BitbucketClient;
use crate::auth::{AuthCredential, KeyringStore};
use crate::config::Config;
use crate::context::{ContextResolver, HostType, RepoContext};
use crate::output::OutputWriter;

use super::GlobalOptions;

/// Manage secrets and pipeline variables
#[derive(Args, Debug)]
pub struct SecretCommand {
    #[command(subcommand)]
    pub command: SecretSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum SecretSubcommand {
    /// List secrets/variables
    #[command(visible_alias = "ls")]
    List(ListArgs),

    /// Set a secret/variable
    Set(SetArgs),

    /// Delete a secret/variable
    Delete(DeleteArgs),

    /// Sync secrets from .env file
    Sync(SyncArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// List workspace-level variables
    #[arg(long, short = 'w')]
    pub workspace: Option<String>,

    /// List repository-level variables
    #[arg(long, short = 'r')]
    pub repo: bool,

    /// List deployment environment variables
    #[arg(long, short = 'e')]
    pub environment: Option<String>,
}

#[derive(Args, Debug)]
pub struct SetArgs {
    /// Variable name
    pub name: String,

    /// Variable value (prompts if omitted)
    #[arg(long, short = 'b')]
    pub body: Option<String>,

    /// Read value from file
    #[arg(long, short = 'f')]
    pub body_file: Option<String>,

    /// Set at workspace level
    #[arg(long, short = 'w')]
    pub workspace: Option<String>,

    /// Set for specific deployment environment
    #[arg(long, short = 'e')]
    pub environment: Option<String>,

    /// Mark as secured (hidden in logs)
    #[arg(long, short = 's')]
    pub secured: bool,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// Variable name
    pub name: String,

    /// Delete from workspace level
    #[arg(long, short = 'w')]
    pub workspace: Option<String>,

    /// Delete from specific deployment environment
    #[arg(long, short = 'e')]
    pub environment: Option<String>,

    /// Skip confirmation
    #[arg(long, short = 'y')]
    pub confirm: bool,
}

#[derive(Args, Debug)]
pub struct SyncArgs {
    /// Path to .env file
    #[arg(long, short = 'f', default_value = ".env")]
    pub file: String,

    /// Mark all as secured
    #[arg(long, short = 's')]
    pub secured: bool,

    /// Delete variables not in file
    #[arg(long)]
    pub delete: bool,

    /// Skip confirmation
    #[arg(long, short = 'y')]
    pub confirm: bool,
}

// API response types

#[derive(Debug, Deserialize)]
struct PipelineVariable {
    uuid: String,
    key: String,
    #[serde(default)]
    value: Option<String>,
    secured: bool,
}

// Output types

#[derive(Debug, Serialize)]
struct VariableListItem {
    uuid: String,
    key: String,
    value: Option<String>,
    secured: bool,
}

impl crate::output::TableOutput for VariableListItem {
    fn print_table(&self, _color: bool) {
        let secured_str = if self.secured {
            style("yes").yellow().to_string()
        } else {
            style("no").dim().to_string()
        };
        let value_str = if self.secured {
            "********".to_string()
        } else {
            self.value.clone().unwrap_or_else(|| "-".to_string())
        };
        println!("{:<30} {:<10} {}", &self.key, secured_str, value_str);
    }

    fn print_markdown(&self) {
        println!(
            "| {} | {} | {} |",
            self.key,
            self.secured,
            self.value.as_deref().unwrap_or("-")
        );
    }
}

impl SecretCommand {
    pub async fn run(&self, global: &GlobalOptions) -> Result<()> {
        match &self.command {
            SecretSubcommand::List(args) => self.list(args, global).await,
            SecretSubcommand::Set(args) => self.set(args, global).await,
            SecretSubcommand::Delete(args) => self.delete(args, global).await,
            SecretSubcommand::Sync(args) => self.sync(args, global).await,
        }
    }

    fn resolve_context(&self, global: &GlobalOptions) -> Result<RepoContext> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        resolver.resolve(global)
    }

    fn get_client(&self, ctx: &RepoContext) -> Result<BitbucketClient> {
        let client = match ctx.host_type {
            HostType::Cloud => BitbucketClient::cloud()?,
            HostType::Server => BitbucketClient::server(&ctx.host)?,
        };

        let keyring = KeyringStore::new();
        let token = keyring
            .get(&ctx.host)?
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

    /// List variables
    async fn list(&self, args: &ListArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;

        if !matches!(ctx.host_type, HostType::Cloud) {
            bail!("Pipeline variables are only available for Bitbucket Cloud.");
        }

        let client = self.get_client(&ctx)?;

        // Determine the URL based on scope
        let url = if let Some(workspace) = &args.workspace {
            format!("/workspaces/{}/pipelines-config/variables", workspace)
        } else if let Some(env) = &args.environment {
            format!(
                "/repositories/{}/{}/deployments_config/environments/{}/variables",
                ctx.owner, ctx.repo_slug, env
            )
        } else {
            format!(
                "/repositories/{}/{}/pipelines_config/variables",
                ctx.owner, ctx.repo_slug
            )
        };

        let response: PaginatedResponse<PipelineVariable> = client.get(&url).await?;

        let items: Vec<VariableListItem> = response
            .values
            .into_iter()
            .map(|v| VariableListItem {
                uuid: v.uuid,
                key: v.key,
                value: v.value,
                secured: v.secured,
            })
            .collect();

        if items.is_empty() {
            println!("No variables found.");
            return Ok(());
        }

        let writer = OutputWriter::new(self.get_format(global));

        if !global.json {
            let scope = if args.workspace.is_some() {
                "Workspace"
            } else if args.environment.is_some() {
                "Environment"
            } else {
                "Repository"
            };

            println!();
            println!("{}", style(format!("{} Variables", scope)).bold());
            println!("{}", "-".repeat(60));
            println!(
                "{} {} {}",
                style(format!("{:<30}", "KEY")).bold(),
                style(format!("{:<10}", "SECURED")).bold(),
                style("VALUE").bold()
            );
            println!("{}", "-".repeat(60));

            for item in &items {
                let secured_str = if item.secured {
                    style("yes").yellow().to_string()
                } else {
                    style("no").dim().to_string()
                };

                let value_str = if item.secured {
                    "********".to_string()
                } else {
                    item.value.clone().unwrap_or_else(|| "-".to_string())
                };

                println!("{:<30} {:<10} {}", item.key, secured_str, value_str);
            }

            println!();
            println!("Showing {} variable(s)", items.len());
        } else {
            writer.write_list(&items)?;
        }

        Ok(())
    }

    /// Set a variable
    async fn set(&self, args: &SetArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;

        if !matches!(ctx.host_type, HostType::Cloud) {
            bail!("Pipeline variables are only available for Bitbucket Cloud.");
        }

        let client = self.get_client(&ctx)?;

        // Get the value
        let value = if let Some(body) = &args.body {
            body.clone()
        } else if let Some(file) = &args.body_file {
            fs::read_to_string(file)?
        } else {
            // Prompt for value
            use dialoguer::Password;
            Password::new().with_prompt("Enter value").interact()?
        };

        // Determine the URL based on scope
        let url = if let Some(workspace) = &args.workspace {
            format!("/workspaces/{}/pipelines-config/variables", workspace)
        } else if let Some(env) = &args.environment {
            format!(
                "/repositories/{}/{}/deployments_config/environments/{}/variables",
                ctx.owner, ctx.repo_slug, env
            )
        } else {
            format!(
                "/repositories/{}/{}/pipelines_config/variables",
                ctx.owner, ctx.repo_slug
            )
        };

        // Check if variable exists (to decide POST vs PUT)
        let existing = self.find_variable(&client, &url, &args.name).await?;

        let body = serde_json::json!({
            "key": args.name,
            "value": value,
            "secured": args.secured,
        });

        if let Some(existing_var) = existing {
            // Update existing variable
            let update_url = format!("{}/{}", url, existing_var.uuid);
            let _: PipelineVariable = client.put(&update_url, &body).await?;

            if global.json {
                let result = serde_json::json!({
                    "success": true,
                    "action": "updated",
                    "key": args.name,
                    "secured": args.secured,
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!(
                    "{} Updated variable {}",
                    style("✓").green(),
                    style(&args.name).cyan()
                );
            }
        } else {
            // Create new variable
            let _: PipelineVariable = client.post(&url, &body).await?;

            if global.json {
                let result = serde_json::json!({
                    "success": true,
                    "action": "created",
                    "key": args.name,
                    "secured": args.secured,
                });
                println!("{}", serde_json::to_string_pretty(&result)?);
            } else {
                println!(
                    "{} Created variable {}",
                    style("✓").green(),
                    style(&args.name).cyan()
                );
            }
        }

        Ok(())
    }

    /// Delete a variable
    async fn delete(&self, args: &DeleteArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;

        if !matches!(ctx.host_type, HostType::Cloud) {
            bail!("Pipeline variables are only available for Bitbucket Cloud.");
        }

        let client = self.get_client(&ctx)?;

        // Determine the URL based on scope
        let base_url = if let Some(workspace) = &args.workspace {
            format!("/workspaces/{}/pipelines-config/variables", workspace)
        } else if let Some(env) = &args.environment {
            format!(
                "/repositories/{}/{}/deployments_config/environments/{}/variables",
                ctx.owner, ctx.repo_slug, env
            )
        } else {
            format!(
                "/repositories/{}/{}/pipelines_config/variables",
                ctx.owner, ctx.repo_slug
            )
        };

        // Find the variable
        let existing = self.find_variable(&client, &base_url, &args.name).await?;

        let var = match existing {
            Some(v) => v,
            None => bail!("Variable '{}' not found", args.name),
        };

        // Confirm deletion
        if !args.confirm && !global.no_prompt {
            use dialoguer::Confirm;
            let confirmed = Confirm::new()
                .with_prompt(format!("Delete variable '{}'?", args.name))
                .default(false)
                .interact()?;

            if !confirmed {
                println!("{} Cancelled.", style("!").yellow());
                return Ok(());
            }
        }

        let delete_url = format!("{}/{}", base_url, var.uuid);
        client.delete(&delete_url).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "key": args.name,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Deleted variable {}",
                style("✓").green(),
                style(&args.name).cyan()
            );
        }

        Ok(())
    }

    /// Sync variables from .env file
    async fn sync(&self, args: &SyncArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;

        if !matches!(ctx.host_type, HostType::Cloud) {
            bail!("Pipeline variables are only available for Bitbucket Cloud.");
        }

        // Parse .env file
        let file = fs::File::open(&args.file)?;
        let reader = io::BufReader::new(file);
        let mut env_vars: Vec<(String, String)> = Vec::new();

        for line in reader.lines() {
            let line = line?;
            let line = line.trim();

            // Skip empty lines and comments
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            // Parse KEY=VALUE
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = value
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .to_string();
                env_vars.push((key, value));
            }
        }

        if env_vars.is_empty() {
            println!("No variables found in {}", args.file);
            return Ok(());
        }

        println!("Found {} variable(s) in {}", env_vars.len(), args.file);

        // Confirm sync
        if !args.confirm && !global.no_prompt {
            use dialoguer::Confirm;
            let confirmed = Confirm::new()
                .with_prompt("Sync these variables to the repository?")
                .default(false)
                .interact()?;

            if !confirmed {
                println!("{} Cancelled.", style("!").yellow());
                return Ok(());
            }
        }

        let client = self.get_client(&ctx)?;
        let base_url = format!(
            "/repositories/{}/{}/pipelines_config/variables",
            ctx.owner, ctx.repo_slug
        );

        let mut created = 0;
        let mut updated = 0;

        for (key, value) in env_vars {
            // Check if variable exists
            let existing = self.find_variable(&client, &base_url, &key).await?;

            let body = serde_json::json!({
                "key": key,
                "value": value,
                "secured": args.secured,
            });

            if let Some(existing_var) = existing {
                // Update
                let update_url = format!("{}/{}", base_url, existing_var.uuid);
                let _: PipelineVariable = client.put(&update_url, &body).await?;
                updated += 1;
                println!("  {} Updated {}", style("↻").yellow(), key);
            } else {
                // Create
                let _: PipelineVariable = client.post(&base_url, &body).await?;
                created += 1;
                println!("  {} Created {}", style("+").green(), key);
            }
        }

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "created": created,
                "updated": updated,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!();
            println!(
                "{} Sync complete: {} created, {} updated",
                style("✓").green(),
                created,
                updated
            );
        }

        Ok(())
    }

    /// Helper to find a variable by key
    async fn find_variable(
        &self,
        client: &BitbucketClient,
        base_url: &str,
        key: &str,
    ) -> Result<Option<PipelineVariable>> {
        let response: PaginatedResponse<PipelineVariable> = client.get(base_url).await?;

        Ok(response.values.into_iter().find(|v| v.key == key))
    }
}
