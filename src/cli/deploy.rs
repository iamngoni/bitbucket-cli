//
//  bitbucket-cli
//  cli/deploy.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Deployment management commands (Cloud only)
//!
//! This module provides commands for managing deployments and environments.
//! Deployments track releases to different environments (test, staging, production).
//!
//! ## Examples
//!
//! ```bash
//! # List deployments
//! bb deploy list
//!
//! # List environments
//! bb deploy environment list
//!
//! # Create an environment
//! bb deploy environment create staging --type staging
//!
//! # View deployment details
//! bb deploy view <uuid>
//! ```

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use console::style;
use serde::{Deserialize, Serialize};

use crate::api::common::PaginatedResponse;
use crate::api::BitbucketClient;
use crate::auth::{AuthCredential, KeyringStore};
use crate::config::Config;
use crate::context::{ContextResolver, HostType, RepoContext};
use crate::output::{OutputWriter, TableOutput};

use super::GlobalOptions;

/// Manage deployments (Cloud only)
#[derive(Args, Debug)]
pub struct DeployCommand {
    #[command(subcommand)]
    pub command: DeploySubcommand,
}

#[derive(Subcommand, Debug)]
pub enum DeploySubcommand {
    /// List deployments
    #[command(visible_alias = "ls")]
    List(ListArgs),

    /// View deployment details
    View(ViewArgs),

    /// Promote a deployment to another environment
    Promote(PromoteArgs),

    /// Manage deployment environments
    Environment(EnvironmentCommand),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter by environment
    #[arg(long, short = 'e')]
    pub environment: Option<String>,

    /// Filter by status (pending, in_progress, completed, failed)
    #[arg(long, short = 's')]
    pub status: Option<String>,

    /// Maximum number to show
    #[arg(long, short = 'l', default_value = "25")]
    pub limit: usize,
}

#[derive(Args, Debug)]
pub struct ViewArgs {
    /// Deployment UUID
    pub uuid: String,
}

#[derive(Args, Debug)]
pub struct PromoteArgs {
    /// Deployment UUID to promote
    pub uuid: String,

    /// Target environment
    #[arg(long, short = 'e')]
    pub environment: String,
}

#[derive(Args, Debug)]
pub struct EnvironmentCommand {
    #[command(subcommand)]
    pub command: EnvironmentSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum EnvironmentSubcommand {
    /// List environments
    #[command(visible_alias = "ls")]
    List,

    /// View environment details
    View(EnvironmentViewArgs),

    /// Create an environment
    Create(EnvironmentCreateArgs),

    /// Edit an environment
    Edit(EnvironmentEditArgs),

    /// Delete an environment
    Delete(EnvironmentDeleteArgs),
}

#[derive(Args, Debug)]
pub struct EnvironmentViewArgs {
    /// Environment name or UUID
    pub name: String,
}

#[derive(Args, Debug)]
pub struct EnvironmentCreateArgs {
    /// Environment name
    pub name: String,

    /// Environment type (test, staging, production)
    #[arg(long, short = 't', value_parser = ["test", "staging", "production"])]
    pub r#type: String,
}

#[derive(Args, Debug)]
pub struct EnvironmentEditArgs {
    /// Environment name or UUID
    pub name: String,

    /// New name
    #[arg(long)]
    pub new_name: Option<String>,

    /// New type
    #[arg(long, short = 't', value_parser = ["test", "staging", "production"])]
    pub r#type: Option<String>,
}

#[derive(Args, Debug)]
pub struct EnvironmentDeleteArgs {
    /// Environment name or UUID
    pub name: String,

    /// Skip confirmation
    #[arg(long, short = 'y')]
    pub confirm: bool,
}

// API response types

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct Deployment {
    uuid: String,
    state: DeploymentState,
    environment: DeploymentEnvironment,
    release: Option<DeploymentRelease>,
    #[serde(default)]
    deployable: Option<DeploymentDeployable>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeploymentState {
    name: String,
    #[serde(default)]
    started_on: Option<String>,
    #[serde(default)]
    completed_on: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeploymentEnvironment {
    uuid: String,
    name: String,
    #[serde(default)]
    environment_type: Option<EnvironmentType>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeploymentRelease {
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    commit: Option<DeploymentCommit>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeploymentCommit {
    hash: String,
    #[serde(default)]
    message: Option<String>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeploymentDeployable {
    #[serde(default)]
    pipeline: Option<DeploymentPipeline>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct DeploymentPipeline {
    uuid: String,
}

#[derive(Debug, Deserialize)]
struct Environment {
    uuid: String,
    name: String,
    #[serde(default)]
    environment_type: Option<EnvironmentType>,
    #[serde(default)]
    rank: Option<i32>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct EnvironmentType {
    name: String,
    #[serde(default)]
    rank: Option<i32>,
}

// Output types

#[derive(Debug, Serialize)]
struct DeploymentListItem {
    uuid: String,
    environment: String,
    state: String,
    release: Option<String>,
    commit: Option<String>,
    started_on: Option<String>,
}

#[derive(Debug, Serialize)]
struct DeploymentDetail {
    uuid: String,
    environment: String,
    environment_type: Option<String>,
    state: String,
    release: Option<String>,
    commit: Option<String>,
    started_on: Option<String>,
    completed_on: Option<String>,
}

#[derive(Debug, Serialize)]
struct EnvironmentListItem {
    uuid: String,
    name: String,
    r#type: Option<String>,
    rank: Option<i32>,
}

impl TableOutput for DeploymentListItem {
    fn print_table(&self, _color: bool) {
        let state_styled = match self.state.as_str() {
            "SUCCESSFUL" => style(&self.state).green().to_string(),
            "FAILED" => style(&self.state).red().to_string(),
            "IN_PROGRESS" => style(&self.state).yellow().to_string(),
            _ => self.state.clone(),
        };
        println!(
            "{:<38} {:<15} {:<12} {}",
            &self.uuid,
            truncate(&self.environment, 13),
            state_styled,
            self.release.as_deref().unwrap_or("-")
        );
    }

    fn print_markdown(&self) {
        println!(
            "| {} | {} | {} | {} |",
            self.uuid,
            self.environment,
            self.state,
            self.release.as_deref().unwrap_or("-")
        );
    }
}

impl DeployCommand {
    pub async fn run(&self, global: &GlobalOptions) -> Result<()> {
        match &self.command {
            DeploySubcommand::List(args) => self.list(args, global).await,
            DeploySubcommand::View(args) => self.view(args, global).await,
            DeploySubcommand::Promote(args) => self.promote(args, global).await,
            DeploySubcommand::Environment(env_cmd) => self.environment(env_cmd, global).await,
        }
    }

    fn resolve_context(&self, global: &GlobalOptions) -> Result<RepoContext> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        resolver.resolve(global)
    }

    fn get_client(&self, ctx: &RepoContext) -> Result<BitbucketClient> {
        if !matches!(ctx.host_type, HostType::Cloud) {
            bail!("Deployments are only available for Bitbucket Cloud.");
        }

        let client = BitbucketClient::cloud()?;

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

    /// List deployments
    async fn list(&self, args: &ListArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        let mut url = format!(
            "/repositories/{}/{}/deployments?pagelen={}",
            ctx.owner, ctx.repo_slug, args.limit
        );

        if let Some(env) = &args.environment {
            url = format!("{}&environment={}", url, env);
        }

        let response: PaginatedResponse<Deployment> = client.get(&url).await?;

        let items: Vec<DeploymentListItem> = response
            .values
            .into_iter()
            .filter(|d| {
                args.status
                    .as_ref()
                    .is_none_or(|s| d.state.name.to_lowercase() == s.to_lowercase())
            })
            .map(|d| DeploymentListItem {
                uuid: d.uuid,
                environment: d.environment.name,
                state: d.state.name,
                release: d.release.as_ref().and_then(|r| r.name.clone()),
                commit: d
                    .release
                    .as_ref()
                    .and_then(|r| r.commit.as_ref().map(|c| c.hash[..8].to_string())),
                started_on: d.state.started_on,
            })
            .collect();

        if items.is_empty() {
            println!("No deployments found.");
            return Ok(());
        }

        let writer = OutputWriter::new(self.get_format(global));

        if !global.json {
            println!();
            println!("{}", style("Deployments").bold());
            println!("{}", "-".repeat(100));
            println!(
                "{} {} {} {} {} {}",
                style(format!("{:<38}", "UUID")).bold(),
                style(format!("{:<15}", "ENVIRONMENT")).bold(),
                style(format!("{:<12}", "STATE")).bold(),
                style(format!("{:<15}", "RELEASE")).bold(),
                style(format!("{:<10}", "COMMIT")).bold(),
                style("STARTED").bold()
            );
            println!("{}", "-".repeat(100));

            for item in &items {
                let state_styled = match item.state.as_str() {
                    "COMPLETED" | "completed" => style(&item.state).green().to_string(),
                    "FAILED" | "failed" => style(&item.state).red().to_string(),
                    "IN_PROGRESS" | "in_progress" => style(&item.state).yellow().to_string(),
                    _ => item.state.clone(),
                };

                println!(
                    "{:<38} {:<15} {:<12} {:<15} {:<10} {}",
                    &item.uuid,
                    &item.environment,
                    state_styled,
                    item.release.as_deref().unwrap_or("-"),
                    item.commit.as_deref().unwrap_or("-"),
                    item.started_on
                        .as_deref()
                        .map(|s| &s[..std::cmp::min(19, s.len())])
                        .unwrap_or("-")
                );
            }

            println!();
            println!("Showing {} deployment(s)", items.len());
        } else {
            writer.write_list(&items)?;
        }

        Ok(())
    }

    /// View deployment details
    async fn view(&self, args: &ViewArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        let url = format!(
            "/repositories/{}/{}/deployments/{}",
            ctx.owner, ctx.repo_slug, args.uuid
        );

        let d: Deployment = client.get(&url).await?;

        let detail = DeploymentDetail {
            uuid: d.uuid,
            environment: d.environment.name,
            environment_type: d.environment.environment_type.map(|t| t.name),
            state: d.state.name,
            release: d.release.as_ref().and_then(|r| r.name.clone()),
            commit: d
                .release
                .as_ref()
                .and_then(|r| r.commit.as_ref().map(|c| c.hash.clone())),
            started_on: d.state.started_on,
            completed_on: d.state.completed_on,
        };

        let writer = OutputWriter::new(self.get_format(global));
        writer.write(&detail)?;

        Ok(())
    }

    /// Promote a deployment to another environment
    async fn promote(&self, args: &PromoteArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        // Get current deployment
        let get_url = format!(
            "/repositories/{}/{}/deployments/{}",
            ctx.owner, ctx.repo_slug, args.uuid
        );

        let current: Deployment = client.get(&get_url).await?;

        // Find target environment UUID
        let envs_url = format!("/repositories/{}/{}/environments", ctx.owner, ctx.repo_slug);

        let envs: PaginatedResponse<Environment> = client.get(&envs_url).await?;

        let target_env = envs
            .values
            .iter()
            .find(|e| e.name.to_lowercase() == args.environment.to_lowercase())
            .ok_or_else(|| anyhow::anyhow!("Environment '{}' not found", args.environment))?;

        // Create promotion - this creates a new deployment pointing to the same release
        let url = format!("/repositories/{}/{}/deployments", ctx.owner, ctx.repo_slug);

        let body = serde_json::json!({
            "environment": {
                "uuid": target_env.uuid
            },
            "release": current.release.map(|r| serde_json::json!({
                "name": r.name,
                "commit": r.commit.map(|c| serde_json::json!({
                    "hash": c.hash
                }))
            }))
        });

        let new_deployment: Deployment = client.post(&url, &body).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "uuid": new_deployment.uuid,
                "environment": new_deployment.environment.name,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Promoted deployment to {}",
                style("✓").green(),
                style(&args.environment).cyan()
            );
            println!("  New deployment UUID: {}", new_deployment.uuid);
        }

        Ok(())
    }

    /// Handle environment subcommands
    async fn environment(&self, cmd: &EnvironmentCommand, global: &GlobalOptions) -> Result<()> {
        match &cmd.command {
            EnvironmentSubcommand::List => self.env_list(global).await,
            EnvironmentSubcommand::View(args) => self.env_view(args, global).await,
            EnvironmentSubcommand::Create(args) => self.env_create(args, global).await,
            EnvironmentSubcommand::Edit(args) => self.env_edit(args, global).await,
            EnvironmentSubcommand::Delete(args) => self.env_delete(args, global).await,
        }
    }

    /// List environments
    async fn env_list(&self, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        let url = format!("/repositories/{}/{}/environments", ctx.owner, ctx.repo_slug);

        let response: PaginatedResponse<Environment> = client.get(&url).await?;

        let items: Vec<EnvironmentListItem> = response
            .values
            .into_iter()
            .map(|e| EnvironmentListItem {
                uuid: e.uuid,
                name: e.name,
                r#type: e.environment_type.map(|t| t.name),
                rank: e.rank,
            })
            .collect();

        if items.is_empty() {
            println!("No environments found.");
            println!();
            println!("Create an environment with:");
            println!("  bb deploy environment create <name> --type <test|staging|production>");
            return Ok(());
        }

        let writer = OutputWriter::new(self.get_format(global));

        if !global.json {
            println!();
            println!("{}", style("Environments").bold());
            println!("{}", "-".repeat(70));
            println!(
                "{} {} {} {}",
                style(format!("{:<38}", "UUID")).bold(),
                style(format!("{:<15}", "NAME")).bold(),
                style(format!("{:<12}", "TYPE")).bold(),
                style("RANK").bold()
            );
            println!("{}", "-".repeat(70));

            for item in &items {
                let type_str = item.r#type.as_deref().unwrap_or("-");
                let type_styled = match type_str {
                    "Production" | "production" => style(type_str).red().to_string(),
                    "Staging" | "staging" => style(type_str).yellow().to_string(),
                    "Test" | "test" => style(type_str).cyan().to_string(),
                    _ => type_str.to_string(),
                };

                println!(
                    "{:<38} {:<15} {:<12} {}",
                    &item.uuid,
                    &item.name,
                    type_styled,
                    item.rank.map_or("-".to_string(), |r| r.to_string())
                );
            }

            println!();
            println!("Showing {} environment(s)", items.len());
        } else {
            writer.write_list(&items)?;
        }

        Ok(())
    }

    /// View environment details
    async fn env_view(&self, args: &EnvironmentViewArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        let url = format!(
            "/repositories/{}/{}/environments/{}",
            ctx.owner, ctx.repo_slug, args.name
        );

        let env: Environment = client.get(&url).await?;

        let detail = EnvironmentListItem {
            uuid: env.uuid,
            name: env.name,
            r#type: env.environment_type.map(|t| t.name),
            rank: env.rank,
        };

        let writer = OutputWriter::new(self.get_format(global));
        writer.write(&detail)?;

        Ok(())
    }

    /// Create an environment
    async fn env_create(&self, args: &EnvironmentCreateArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        let url = format!("/repositories/{}/{}/environments", ctx.owner, ctx.repo_slug);

        let body = serde_json::json!({
            "name": args.name,
            "environment_type": {
                "name": args.r#type
            }
        });

        let env: Environment = client.post(&url, &body).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "uuid": env.uuid,
                "name": env.name,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Created environment {}",
                style("✓").green(),
                style(&args.name).cyan()
            );
            println!("  UUID: {}", env.uuid);
            println!("  Type: {}", args.r#type);
        }

        Ok(())
    }

    /// Edit an environment
    async fn env_edit(&self, args: &EnvironmentEditArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        // Get current environment
        let get_url = format!(
            "/repositories/{}/{}/environments/{}",
            ctx.owner, ctx.repo_slug, args.name
        );

        let current: Environment = client.get(&get_url).await?;

        // Build update body
        let mut body = serde_json::json!({
            "name": args.new_name.clone().unwrap_or(current.name),
        });

        if let Some(env_type) = &args.r#type {
            body["environment_type"] = serde_json::json!({
                "name": env_type
            });
        } else if let Some(current_type) = current.environment_type {
            body["environment_type"] = serde_json::json!({
                "name": current_type.name
            });
        }

        let env: Environment = client.put(&get_url, &body).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "uuid": env.uuid,
                "name": env.name,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Updated environment {}",
                style("✓").green(),
                style(&env.name).cyan()
            );
        }

        Ok(())
    }

    /// Delete an environment
    async fn env_delete(&self, args: &EnvironmentDeleteArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        // Confirm deletion
        if !args.confirm && !global.no_prompt {
            use dialoguer::Confirm;
            let confirmed = Confirm::new()
                .with_prompt(format!("Delete environment '{}'?", args.name))
                .default(false)
                .interact()?;

            if !confirmed {
                println!("{} Cancelled.", style("!").yellow());
                return Ok(());
            }
        }

        let url = format!(
            "/repositories/{}/{}/environments/{}",
            ctx.owner, ctx.repo_slug, args.name
        );

        client.delete(&url).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "name": args.name,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Deleted environment {}",
                style("✓").green(),
                style(&args.name).cyan()
            );
        }

        Ok(())
    }
}

// TableOutput implementations

impl TableOutput for DeploymentDetail {
    fn print_table(&self, _color: bool) {
        println!();
        println!("{}", style("Deployment Details").bold());
        println!("{}", "-".repeat(60));
        println!("UUID:                 {}", style(&self.uuid).cyan());
        println!("Environment:          {}", self.environment);
        println!(
            "Type:                 {}",
            self.environment_type.as_deref().unwrap_or("-")
        );
        println!(
            "State:                {}",
            match self.state.as_str() {
                "COMPLETED" | "completed" => style(&self.state).green().to_string(),
                "FAILED" | "failed" => style(&self.state).red().to_string(),
                _ => self.state.clone(),
            }
        );
        println!(
            "Release:              {}",
            self.release.as_deref().unwrap_or("-")
        );
        println!(
            "Commit:               {}",
            self.commit.as_deref().unwrap_or("-")
        );
        println!(
            "Started:              {}",
            self.started_on.as_deref().unwrap_or("-")
        );
        println!(
            "Completed:            {}",
            self.completed_on.as_deref().unwrap_or("-")
        );
        println!();
    }

    fn print_markdown(&self) {
        println!("# Deployment: {}", self.uuid);
        println!();
        println!("- **Environment**: {}", self.environment);
        println!("- **State**: {}", self.state);
        println!("- **Release**: {}", self.release.as_deref().unwrap_or("-"));
    }
}

impl TableOutput for EnvironmentListItem {
    fn print_table(&self, _color: bool) {
        println!();
        println!("{}", style("Environment Details").bold());
        println!("{}", "-".repeat(60));
        println!("UUID:                 {}", style(&self.uuid).cyan());
        println!("Name:                 {}", self.name);
        println!(
            "Type:                 {}",
            self.r#type.as_deref().unwrap_or("-")
        );
        println!(
            "Rank:                 {}",
            self.rank.map_or("-".to_string(), |r| r.to_string())
        );
        println!();
    }

    fn print_markdown(&self) {
        println!("# Environment: {}", self.name);
        println!();
        println!("- **UUID**: {}", self.uuid);
        println!("- **Type**: {}", self.r#type.as_deref().unwrap_or("-"));
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
