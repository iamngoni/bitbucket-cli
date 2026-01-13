//
//  bitbucket-cli
//  cli/pipeline.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Pipeline commands for Bitbucket Pipelines (Cloud only).
//!
//! This module implements CI/CD pipeline management commands for Bitbucket Cloud.
//! Pipelines are not available for Bitbucket Server/Data Center.

use std::process::Command as ProcessCommand;
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use clap::{Args, Subcommand};
use console::style;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::api::cloud::pipelines::{
    Pipeline, PipelineStep, PipelineVariable, TriggerPipelineRequest, TriggerSelector,
    TriggerTarget,
};
use crate::api::common::PaginatedResponse;
use crate::auth::KeyringStore;
use crate::config::Config;
use crate::context::{ContextResolver, HostType, RepoContext};
use crate::output::{OutputFormat, OutputWriter, TableOutput};

use super::GlobalOptions;

/// Manage pipelines (Cloud only).
///
/// Bitbucket Pipelines is a CI/CD service built into Bitbucket Cloud.
/// This command group provides operations for listing, triggering,
/// monitoring, and managing pipeline runs.
#[derive(Args, Debug)]
pub struct PipelineCommand {
    #[command(subcommand)]
    pub command: PipelineSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum PipelineSubcommand {
    /// List pipeline runs
    #[command(visible_alias = "ls")]
    List(ListArgs),

    /// View pipeline details
    View(ViewArgs),

    /// Trigger a pipeline run
    Run(RunArgs),

    /// Stop a running pipeline
    Stop(StopArgs),

    /// Rerun a pipeline
    Rerun(RerunArgs),

    /// View pipeline logs
    Logs(LogsArgs),

    /// Watch pipeline progress
    Watch(WatchArgs),

    /// Enable pipelines for repository
    Enable,

    /// Disable pipelines for repository
    Disable,

    /// View or edit pipeline configuration
    Config(ConfigArgs),

    /// Manage build caches
    Cache(CacheCommand),

    /// Manage scheduled pipelines
    Schedule(ScheduleCommand),

    /// Manage self-hosted runners
    Runner(RunnerCommand),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter by branch
    #[arg(long, short = 'b')]
    pub branch: Option<String>,

    /// Filter by status
    #[arg(long, short = 's', value_parser = ["pending", "in_progress", "successful", "failed", "stopped"])]
    pub status: Option<String>,

    /// Filter by trigger type
    #[arg(long, short = 't', value_parser = ["push", "manual", "schedule"])]
    pub trigger: Option<String>,

    /// Maximum number of pipelines to list
    #[arg(long, short = 'L', default_value = "20")]
    pub limit: u32,
}

#[derive(Args, Debug)]
pub struct ViewArgs {
    /// Pipeline UUID or build number
    pub id: String,

    /// Open in browser
    #[arg(long, short = 'w')]
    pub web: bool,
}

#[derive(Args, Debug)]
pub struct RunArgs {
    /// Branch to run pipeline on
    #[arg(long, short = 'b')]
    pub branch: Option<String>,

    /// Custom pipeline to run
    #[arg(long, short = 'c')]
    pub custom: Option<String>,

    /// Set pipeline variable (key=value)
    #[arg(long, short = 'v', action = clap::ArgAction::Append)]
    pub variable: Vec<String>,

    /// Watch pipeline after triggering
    #[arg(long, short = 'w')]
    pub watch: bool,
}

#[derive(Args, Debug)]
pub struct StopArgs {
    /// Pipeline UUID or build number
    pub id: String,
}

#[derive(Args, Debug)]
pub struct RerunArgs {
    /// Pipeline UUID or build number
    pub id: String,

    /// Only rerun failed steps
    #[arg(long)]
    pub failed_only: bool,
}

#[derive(Args, Debug)]
pub struct LogsArgs {
    /// Pipeline UUID or build number
    pub id: String,

    /// Specific step name
    #[arg(long, short = 's')]
    pub step: Option<String>,

    /// Follow logs in real-time
    #[arg(long, short = 'f')]
    pub follow: bool,

    /// Show only failed steps
    #[arg(long)]
    pub failed: bool,
}

#[derive(Args, Debug)]
pub struct WatchArgs {
    /// Pipeline UUID or build number
    pub id: String,

    /// Exit with pipeline's exit status
    #[arg(long)]
    pub exit_status: bool,

    /// Refresh interval in seconds
    #[arg(long, short = 'i', default_value = "3")]
    pub interval: u32,
}

#[derive(Args, Debug)]
pub struct ConfigArgs {
    /// Validate the pipeline configuration
    #[arg(long, short = 'v')]
    pub validate: bool,

    /// Open configuration in editor
    #[arg(long, short = 'e')]
    pub edit: bool,
}

#[derive(Args, Debug)]
pub struct CacheCommand {
    #[command(subcommand)]
    pub command: CacheSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum CacheSubcommand {
    /// List caches
    List,
    /// Delete a specific cache
    Delete { name: String },
    /// Clear all caches
    Clear,
}

#[derive(Args, Debug)]
pub struct ScheduleCommand {
    #[command(subcommand)]
    pub command: ScheduleSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ScheduleSubcommand {
    /// List scheduled pipelines
    List,
    /// Create a scheduled pipeline
    Create(ScheduleCreateArgs),
    /// Delete a scheduled pipeline
    Delete { id: String },
    /// Pause a scheduled pipeline
    Pause { id: String },
    /// Resume a scheduled pipeline
    Resume { id: String },
}

#[derive(Args, Debug)]
pub struct ScheduleCreateArgs {
    /// Cron expression
    #[arg(long)]
    pub cron: String,

    /// Branch to run on
    #[arg(long, short = 'b')]
    pub branch: String,

    /// Pipeline name
    #[arg(long, short = 'p')]
    pub pipeline: Option<String>,
}

#[derive(Args, Debug)]
pub struct RunnerCommand {
    #[command(subcommand)]
    pub command: RunnerSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum RunnerSubcommand {
    /// List runners
    List,
    /// Register a new runner
    Register,
    /// Remove a runner
    Remove { id: String },
}

// Display types for pipelines
#[derive(Debug, Serialize)]
struct PipelineListItem {
    build_number: u64,
    state: String,
    result: Option<String>,
    branch: String,
    created_on: String,
    duration_seconds: Option<u64>,
}

impl TableOutput for PipelineListItem {
    fn print_table(&self, color: bool) {
        let status = if let Some(ref result) = self.result {
            let status_str = format!("{} ({})", self.state, result);
            if color {
                match result.to_uppercase().as_str() {
                    "SUCCESSFUL" => style(status_str).green().to_string(),
                    "FAILED" => style(status_str).red().to_string(),
                    "STOPPED" => style(status_str).yellow().to_string(),
                    _ => status_str,
                }
            } else {
                status_str
            }
        } else if color && self.state.to_uppercase() == "IN_PROGRESS" {
            style(&self.state).cyan().to_string()
        } else {
            self.state.clone()
        };

        let duration = self
            .duration_seconds
            .map(format_duration)
            .unwrap_or_else(|| "-".to_string());

        println!(
            "#{:<6} {:20} {:20} {:15} {}",
            self.build_number,
            status,
            self.branch,
            format_timestamp(&self.created_on),
            duration
        );
    }

    fn print_markdown(&self) {
        println!(
            "| #{} | {} | {} | {} | {} |",
            self.build_number,
            self.result.as_ref().unwrap_or(&self.state),
            self.branch,
            format_timestamp(&self.created_on),
            self.duration_seconds
                .map(format_duration)
                .unwrap_or_else(|| "-".to_string())
        );
    }
}

// Cache types
#[derive(Debug, Deserialize, Serialize)]
struct PipelineCache {
    uuid: String,
    name: String,
    #[serde(default)]
    created_on: Option<String>,
    #[serde(default)]
    file_size_bytes: Option<u64>,
}

impl TableOutput for PipelineCache {
    fn print_table(&self, _color: bool) {
        println!(
            "{:30} {:15} {}",
            self.name,
            self.file_size_bytes
                .map(format_bytes)
                .unwrap_or_else(|| "-".to_string()),
            self.created_on
                .as_ref()
                .map(|c| format_timestamp(c))
                .unwrap_or_else(|| "-".to_string())
        );
    }

    fn print_markdown(&self) {
        println!(
            "| {} | {} | {} |",
            self.name,
            self.file_size_bytes
                .map(format_bytes)
                .unwrap_or_else(|| "-".to_string()),
            self.created_on
                .as_ref()
                .map(|c| format_timestamp(c))
                .unwrap_or_else(|| "-".to_string())
        );
    }
}

// Schedule types
#[derive(Debug, Deserialize, Serialize)]
struct PipelineSchedule {
    uuid: String,
    enabled: bool,
    cron_pattern: String,
    target: PipelineScheduleTarget,
    #[serde(default)]
    created_on: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PipelineScheduleTarget {
    #[serde(default)]
    ref_name: Option<String>,
    #[serde(default)]
    ref_type: Option<String>,
    #[serde(default)]
    selector: Option<ScheduleSelector>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ScheduleSelector {
    #[serde(default)]
    pattern: Option<String>,
}

impl TableOutput for PipelineSchedule {
    fn print_table(&self, color: bool) {
        let enabled = if self.enabled {
            if color {
                style("Yes").green().to_string()
            } else {
                "Yes".to_string()
            }
        } else if color {
            style("No").dim().to_string()
        } else {
            "No".to_string()
        };

        let pipeline = self
            .target
            .selector
            .as_ref()
            .and_then(|s| s.pattern.clone())
            .unwrap_or_else(|| "default".to_string());

        println!(
            "{:12} {:8} {:20} {:20} {}",
            truncate_uuid(&self.uuid),
            enabled,
            self.cron_pattern,
            self.target.ref_name.as_deref().unwrap_or("-"),
            pipeline
        );
    }

    fn print_markdown(&self) {
        let pipeline = self
            .target
            .selector
            .as_ref()
            .and_then(|s| s.pattern.clone())
            .unwrap_or_else(|| "default".to_string());

        println!(
            "| {} | {} | {} | {} | {} |",
            truncate_uuid(&self.uuid),
            if self.enabled { "Yes" } else { "No" },
            self.cron_pattern,
            self.target.ref_name.as_deref().unwrap_or("-"),
            pipeline
        );
    }
}

// Runner types
#[derive(Debug, Deserialize, Serialize)]
struct PipelineRunner {
    uuid: String,
    name: String,
    #[serde(default)]
    state: Option<RunnerState>,
    #[serde(default)]
    labels: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RunnerState {
    status: String,
}

impl TableOutput for PipelineRunner {
    fn print_table(&self, color: bool) {
        let status = self
            .state
            .as_ref()
            .map(|s| s.status.clone())
            .unwrap_or_else(|| "UNKNOWN".to_string());

        let status_display = if color {
            match status.to_uppercase().as_str() {
                "ONLINE" => style(&status).green().to_string(),
                "OFFLINE" => style(&status).red().to_string(),
                _ => status.clone(),
            }
        } else {
            status.clone()
        };

        println!(
            "{:12} {:30} {:15} {}",
            truncate_uuid(&self.uuid),
            self.name,
            status_display,
            self.labels.join(", ")
        );
    }

    fn print_markdown(&self) {
        let status = self
            .state
            .as_ref()
            .map(|s| s.status.clone())
            .unwrap_or_else(|| "UNKNOWN".to_string());

        println!(
            "| {} | {} | {} | {} |",
            truncate_uuid(&self.uuid),
            self.name,
            status,
            self.labels.join(", ")
        );
    }
}

// Repository pipelines configuration
#[derive(Debug, Deserialize)]
struct RepositoryPipelinesConfig {
    enabled: bool,
}

#[derive(Debug, Serialize)]
struct UpdatePipelinesConfig {
    enabled: bool,
}

// Create schedule request
#[derive(Debug, Serialize)]
struct CreateScheduleRequest {
    #[serde(rename = "type")]
    type_field: String,
    enabled: bool,
    cron_pattern: String,
    target: CreateScheduleTarget,
}

#[derive(Debug, Serialize)]
struct CreateScheduleTarget {
    #[serde(rename = "type")]
    type_field: String,
    ref_type: String,
    ref_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    selector: Option<CreateScheduleSelector>,
}

#[derive(Debug, Serialize)]
struct CreateScheduleSelector {
    #[serde(rename = "type")]
    type_field: String,
    pattern: String,
}

impl PipelineCommand {
    pub async fn run(&self, global: &GlobalOptions) -> Result<()> {
        match &self.command {
            PipelineSubcommand::List(args) => self.list(args, global).await,
            PipelineSubcommand::View(args) => self.view(args, global).await,
            PipelineSubcommand::Run(args) => self.run_pipeline(args, global).await,
            PipelineSubcommand::Stop(args) => self.stop(args, global).await,
            PipelineSubcommand::Rerun(args) => self.rerun(args, global).await,
            PipelineSubcommand::Logs(args) => self.logs(args, global).await,
            PipelineSubcommand::Watch(args) => self.watch(args, global).await,
            PipelineSubcommand::Enable => self.enable(global).await,
            PipelineSubcommand::Disable => self.disable(global).await,
            PipelineSubcommand::Config(args) => self.config(args, global).await,
            PipelineSubcommand::Cache(cache) => self.cache(&cache.command, global).await,
            PipelineSubcommand::Schedule(schedule) => {
                self.schedule(&schedule.command, global).await
            }
            PipelineSubcommand::Runner(runner) => self.runner(&runner.command, global).await,
        }
    }

    /// Get output format from global options
    fn get_format(&self, global: &GlobalOptions) -> OutputFormat {
        if global.json {
            OutputFormat::Json
        } else {
            OutputFormat::Table
        }
    }

    /// Get repository context, ensuring Cloud-only
    fn get_cloud_context(&self, global: &GlobalOptions) -> Result<RepoContext> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        let context = resolver.resolve(global)?;

        if context.host_type != HostType::Cloud {
            anyhow::bail!(
                "Pipelines are only available for Bitbucket Cloud. \
                 Bitbucket Server/Data Center does not support Pipelines."
            );
        }

        Ok(context)
    }

    /// Get authentication token for Cloud
    fn get_token(&self, context: &RepoContext) -> Result<String> {
        let keyring = KeyringStore::new();
        keyring.get(&context.host)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Not authenticated for {}. Run 'bb auth login' first.",
                context.host
            )
        })
    }

    /// Create HTTP client
    fn create_client(&self) -> Result<Client> {
        Client::builder()
            .user_agent(format!("bb/{}", crate::VERSION))
            .build()
            .context("Failed to create HTTP client")
    }

    /// List pipeline runs
    async fn list(&self, args: &ListArgs, global: &GlobalOptions) -> Result<()> {
        let context = self.get_cloud_context(global)?;
        let token = self.get_token(&context)?;
        let client = self.create_client()?;

        // Build URL with query parameters
        let url = format!(
            "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines/?sort=-created_on&pagelen={}",
            context.owner, context.repo_slug, args.limit
        );

        let response = client
            .get(&url)
            .bearer_auth(&token)
            .send()
            .await
            .context("Failed to fetch pipelines")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error ({}): {}", status, body);
        }

        let paginated: PaginatedResponse<Pipeline> = response
            .json()
            .await
            .context("Failed to parse pipelines response")?;

        // Filter results
        let items: Vec<PipelineListItem> = paginated
            .values
            .into_iter()
            .filter(|p| {
                // Filter by branch
                if let Some(ref branch) = args.branch {
                    if let Some(ref ref_name) = p.target.ref_name {
                        if ref_name != branch {
                            return false;
                        }
                    }
                }
                // Filter by status
                if let Some(ref status) = args.status {
                    let matches = match status.as_str() {
                        "pending" => p.state.state_type == "pipeline_state_pending",
                        "in_progress" => p.state.state_type == "pipeline_state_in_progress",
                        "successful" => p
                            .state
                            .result
                            .as_ref()
                            .map(|r| r.result_type == "pipeline_state_completed_successful")
                            .unwrap_or(false),
                        "failed" => p
                            .state
                            .result
                            .as_ref()
                            .map(|r| r.result_type == "pipeline_state_completed_failed")
                            .unwrap_or(false),
                        "stopped" => p
                            .state
                            .result
                            .as_ref()
                            .map(|r| r.result_type == "pipeline_state_completed_stopped")
                            .unwrap_or(false),
                        _ => true,
                    };
                    if !matches {
                        return false;
                    }
                }
                true
            })
            .take(args.limit as usize)
            .map(|p| PipelineListItem {
                build_number: p.build_number,
                state: p.state.name.clone(),
                result: p.state.result.as_ref().map(|r| r.name.clone()),
                branch: p.target.ref_name.clone().unwrap_or_else(|| "-".to_string()),
                created_on: p.created_on.clone(),
                duration_seconds: p.duration_in_seconds,
            })
            .collect();

        if items.is_empty() {
            println!("No pipelines found");
            return Ok(());
        }

        let format = self.get_format(global);
        let writer = OutputWriter::new(format);

        if !global.json {
            println!(
                "{:<7} {:20} {:20} {:15} DURATION",
                "#", "STATUS", "BRANCH", "CREATED"
            );
            println!("{}", "-".repeat(80));
        }

        writer.write_list(&items)?;

        Ok(())
    }

    /// View pipeline details
    async fn view(&self, args: &ViewArgs, global: &GlobalOptions) -> Result<()> {
        let context = self.get_cloud_context(global)?;
        let token = self.get_token(&context)?;
        let client = self.create_client()?;

        // Open in browser if requested
        if args.web {
            let url = format!(
                "https://bitbucket.org/{}/{}/pipelines/results/{}",
                context.owner, context.repo_slug, args.id
            );
            println!("Opening {} in browser...", url);
            webbrowser::open(&url)?;
            return Ok(());
        }

        // Fetch pipeline details
        let pipeline_url = format!(
            "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines/{}",
            context.owner,
            context.repo_slug,
            format_pipeline_id(&args.id)
        );

        let response = client
            .get(&pipeline_url)
            .bearer_auth(&token)
            .send()
            .await
            .context("Failed to fetch pipeline")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error ({}): {}", status, body);
        }

        let pipeline: Pipeline = response
            .json()
            .await
            .context("Failed to parse pipeline response")?;

        // Fetch steps
        let steps_url = format!(
            "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines/{}/steps/",
            context.owner,
            context.repo_slug,
            format_pipeline_id(&args.id)
        );

        let steps: Vec<PipelineStep> = match client.get(&steps_url).bearer_auth(&token).send().await
        {
            Ok(resp) if resp.status().is_success() => {
                let paginated: PaginatedResponse<PipelineStep> =
                    resp.json().await.unwrap_or_else(|_| PaginatedResponse {
                        values: vec![],
                        next: None,
                        previous: None,
                        size: None,
                        page: None,
                        pagelen: None,
                    });
                paginated.values
            }
            _ => vec![],
        };

        if global.json {
            let output = serde_json::json!({
                "uuid": pipeline.uuid,
                "build_number": pipeline.build_number,
                "state": pipeline.state.name,
                "result": pipeline.state.result.as_ref().map(|r| &r.name),
                "target": {
                    "ref_type": pipeline.target.ref_type,
                    "ref_name": pipeline.target.ref_name,
                },
                "created_on": pipeline.created_on,
                "completed_on": pipeline.completed_on,
                "duration_seconds": pipeline.duration_in_seconds,
                "steps": steps.iter().map(|s| {
                    serde_json::json!({
                        "uuid": s.uuid,
                        "name": s.name,
                        "state": s.state.name,
                        "result": s.state.result.as_ref().map(|r| &r.name),
                        "duration_seconds": s.duration_in_seconds,
                    })
                }).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            // Display pipeline info
            let status = if let Some(ref result) = pipeline.state.result {
                format!("{} ({})", pipeline.state.name, result.name)
            } else {
                pipeline.state.name.clone()
            };

            println!("Pipeline #{}", pipeline.build_number);
            println!("Status:     {}", status);
            println!(
                "Branch:     {}",
                pipeline
                    .target
                    .ref_name
                    .clone()
                    .unwrap_or_else(|| "-".to_string())
            );
            println!("Created:    {}", format_timestamp(&pipeline.created_on));

            if let Some(ref completed) = pipeline.completed_on {
                println!("Completed:  {}", format_timestamp(completed));
            }

            if let Some(duration) = pipeline.duration_in_seconds {
                println!("Duration:   {}", format_duration(duration));
            }

            println!("\nSteps:");
            for step in &steps {
                let step_status = if let Some(ref result) = step.state.result {
                    let symbol = match result.result_type.as_str() {
                        "pipeline_state_completed_successful" => style("[OK]").green(),
                        "pipeline_state_completed_failed" => style("[FAILED]").red(),
                        "pipeline_state_completed_stopped" => style("[STOPPED]").yellow(),
                        _ => style("[?]").dim(),
                    };
                    format!("{} {}", symbol, result.name)
                } else {
                    format!("[{}]", step.state.name)
                };

                let duration_str = step
                    .duration_in_seconds
                    .map(|d| format!(" ({})", format_duration(d)))
                    .unwrap_or_default();

                println!("  {} - {}{}", step.name, step_status, duration_str);
            }

            println!(
                "\nView in browser: https://bitbucket.org/{}/{}/pipelines/results/{}",
                context.owner, context.repo_slug, pipeline.build_number
            );
        }

        Ok(())
    }

    /// Trigger a new pipeline run
    async fn run_pipeline(&self, args: &RunArgs, global: &GlobalOptions) -> Result<()> {
        let context = self.get_cloud_context(global)?;
        let token = self.get_token(&context)?;
        let client = self.create_client()?;

        // Get branch (default to current branch)
        let branch = if let Some(ref b) = args.branch {
            b.clone()
        } else {
            get_current_branch()?
        };

        // Parse variables
        let variables: Vec<PipelineVariable> = args
            .variable
            .iter()
            .filter_map(|v| {
                let parts: Vec<&str> = v.splitn(2, '=').collect();
                if parts.len() == 2 {
                    Some(PipelineVariable {
                        key: parts[0].to_string(),
                        value: parts[1].to_string(),
                        secured: false,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Build request
        let selector = args.custom.as_ref().map(|c| TriggerSelector {
            selector_type: "custom".to_string(),
            pattern: c.clone(),
        });

        let request = TriggerPipelineRequest {
            target: TriggerTarget {
                target_type: "pipeline_ref_target".to_string(),
                ref_type: "branch".to_string(),
                ref_name: branch.clone(),
                selector,
            },
            variables,
        };

        let url = format!(
            "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines/",
            context.owner, context.repo_slug
        );

        let response = client
            .post(&url)
            .bearer_auth(&token)
            .json(&request)
            .send()
            .await
            .context("Failed to trigger pipeline")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error ({}): {}", status, body);
        }

        let pipeline: Pipeline = response
            .json()
            .await
            .context("Failed to parse pipeline response")?;

        if global.json {
            let output = serde_json::json!({
                "build_number": pipeline.build_number,
                "uuid": pipeline.uuid,
                "branch": branch,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!(
                "Triggered pipeline #{} on branch '{}'",
                pipeline.build_number, branch
            );
            println!(
                "View at: https://bitbucket.org/{}/{}/pipelines/results/{}",
                context.owner, context.repo_slug, pipeline.build_number
            );

            // Watch if requested
            if args.watch {
                println!("\nWatching pipeline progress...\n");
                self.watch_pipeline(&context, &token, &pipeline.uuid, 3, false)
                    .await?;
            }
        }

        Ok(())
    }

    /// Stop a running pipeline
    async fn stop(&self, args: &StopArgs, global: &GlobalOptions) -> Result<()> {
        let context = self.get_cloud_context(global)?;
        let token = self.get_token(&context)?;
        let client = self.create_client()?;

        let url = format!(
            "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines/{}/stopPipeline",
            context.owner,
            context.repo_slug,
            format_pipeline_id(&args.id)
        );

        let response = client
            .post(&url)
            .bearer_auth(&token)
            .send()
            .await
            .context("Failed to stop pipeline")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error ({}): {}", status, body);
        }

        if global.json {
            println!(r#"{{"stopped": true, "pipeline": "{}"}}"#, args.id);
        } else {
            println!("Pipeline {} has been stopped", args.id);
        }

        Ok(())
    }

    /// Rerun a pipeline
    async fn rerun(&self, args: &RerunArgs, global: &GlobalOptions) -> Result<()> {
        let context = self.get_cloud_context(global)?;
        let token = self.get_token(&context)?;
        let client = self.create_client()?;

        // First, get the original pipeline to extract its target
        let pipeline_url = format!(
            "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines/{}",
            context.owner,
            context.repo_slug,
            format_pipeline_id(&args.id)
        );

        let response = client
            .get(&pipeline_url)
            .bearer_auth(&token)
            .send()
            .await
            .context("Failed to fetch pipeline")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error ({}): {}", status, body);
        }

        let pipeline: Pipeline = response
            .json()
            .await
            .context("Failed to parse pipeline response")?;

        // Trigger a new pipeline with the same target
        let ref_name =
            pipeline.target.ref_name.clone().ok_or_else(|| {
                anyhow::anyhow!("Could not determine branch from original pipeline")
            })?;

        let selector = pipeline.target.selector.as_ref().map(|s| TriggerSelector {
            selector_type: s.selector_type.clone(),
            pattern: s.pattern.clone().unwrap_or_default(),
        });

        let request = TriggerPipelineRequest {
            target: TriggerTarget {
                target_type: "pipeline_ref_target".to_string(),
                ref_type: pipeline
                    .target
                    .ref_type
                    .clone()
                    .unwrap_or_else(|| "branch".to_string()),
                ref_name,
                selector,
            },
            variables: vec![],
        };

        let url = format!(
            "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines/",
            context.owner, context.repo_slug
        );

        let response = client
            .post(&url)
            .bearer_auth(&token)
            .json(&request)
            .send()
            .await
            .context("Failed to trigger pipeline rerun")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error ({}): {}", status, body);
        }

        let new_pipeline: Pipeline = response
            .json()
            .await
            .context("Failed to parse pipeline response")?;

        if global.json {
            let output = serde_json::json!({
                "original_pipeline": args.id,
                "new_build_number": new_pipeline.build_number,
                "uuid": new_pipeline.uuid,
            });
            println!("{}", serde_json::to_string_pretty(&output)?);
        } else {
            println!(
                "Rerunning pipeline {} as #{}",
                args.id, new_pipeline.build_number
            );
            println!(
                "View at: https://bitbucket.org/{}/{}/pipelines/results/{}",
                context.owner, context.repo_slug, new_pipeline.build_number
            );
        }

        Ok(())
    }

    /// View pipeline logs
    async fn logs(&self, args: &LogsArgs, global: &GlobalOptions) -> Result<()> {
        let context = self.get_cloud_context(global)?;
        let token = self.get_token(&context)?;
        let client = self.create_client()?;

        // First, get the steps
        let steps_url = format!(
            "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines/{}/steps/",
            context.owner,
            context.repo_slug,
            format_pipeline_id(&args.id)
        );

        let response = client
            .get(&steps_url)
            .bearer_auth(&token)
            .send()
            .await
            .context("Failed to fetch pipeline steps")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error ({}): {}", status, body);
        }

        let paginated: PaginatedResponse<PipelineStep> = response
            .json()
            .await
            .context("Failed to parse steps response")?;

        let steps: Vec<PipelineStep> = paginated
            .values
            .into_iter()
            .filter(|s| {
                // Filter by step name if specified
                if let Some(ref step_name) = args.step {
                    return s.name.to_lowercase().contains(&step_name.to_lowercase());
                }
                // Filter by failed if requested
                if args.failed {
                    return s
                        .state
                        .result
                        .as_ref()
                        .map(|r| r.result_type == "pipeline_state_completed_failed")
                        .unwrap_or(false);
                }
                true
            })
            .collect();

        if steps.is_empty() {
            println!("No matching steps found");
            return Ok(());
        }

        // Fetch logs for each step
        for step in &steps {
            let log_url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines/{}/steps/{}/log",
                context.owner,
                context.repo_slug,
                format_pipeline_id(&args.id),
                step.uuid.trim_matches(|c| c == '{' || c == '}')
            );

            if !global.json {
                println!("\n=== Step: {} ===\n", step.name);
            }

            match client.get(&log_url).bearer_auth(&token).send().await {
                Ok(resp) if resp.status().is_success() => {
                    let log_text = resp
                        .text()
                        .await
                        .unwrap_or_else(|_| "(no logs)".to_string());
                    if global.json {
                        let output = serde_json::json!({
                            "step": step.name,
                            "log": log_text,
                        });
                        println!("{}", serde_json::to_string(&output)?);
                    } else {
                        println!("{}", log_text);
                    }
                }
                Ok(resp) => {
                    if !global.json {
                        println!("(logs not available: {})", resp.status());
                    }
                }
                Err(e) => {
                    if !global.json {
                        println!("(failed to fetch logs: {})", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Watch pipeline progress
    async fn watch(&self, args: &WatchArgs, global: &GlobalOptions) -> Result<()> {
        let context = self.get_cloud_context(global)?;
        let token = self.get_token(&context)?;

        self.watch_pipeline(&context, &token, &args.id, args.interval, args.exit_status)
            .await
    }

    /// Internal watch implementation
    async fn watch_pipeline(
        &self,
        context: &RepoContext,
        token: &str,
        id: &str,
        interval: u32,
        exit_status: bool,
    ) -> Result<()> {
        let client = self.create_client()?;

        loop {
            // Clear screen for better display
            print!("\x1B[2J\x1B[1;1H");

            let pipeline_url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines/{}",
                context.owner,
                context.repo_slug,
                format_pipeline_id(id)
            );

            let response = client
                .get(&pipeline_url)
                .bearer_auth(token)
                .send()
                .await
                .context("Failed to fetch pipeline")?;

            if !response.status().is_success() {
                let status = response.status();
                let body = response.text().await.unwrap_or_default();
                anyhow::bail!("API error ({}): {}", status, body);
            }

            let pipeline: Pipeline = response
                .json()
                .await
                .context("Failed to parse pipeline response")?;

            // Fetch steps
            let steps_url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines/{}/steps/",
                context.owner,
                context.repo_slug,
                format_pipeline_id(id)
            );

            let steps: Vec<PipelineStep> =
                match client.get(&steps_url).bearer_auth(token).send().await {
                    Ok(resp) if resp.status().is_success() => {
                        let paginated: PaginatedResponse<PipelineStep> =
                            resp.json().await.unwrap_or_else(|_| PaginatedResponse {
                                values: vec![],
                                next: None,
                                previous: None,
                                size: None,
                                page: None,
                                pagelen: None,
                            });
                        paginated.values
                    }
                    _ => vec![],
                };

            // Display status
            let status = if let Some(ref result) = pipeline.state.result {
                format!("{} ({})", pipeline.state.name, result.name)
            } else {
                pipeline.state.name.clone()
            };

            println!("Pipeline #{} - {}", pipeline.build_number, status);
            println!(
                "Branch: {}",
                pipeline
                    .target
                    .ref_name
                    .clone()
                    .unwrap_or_else(|| "-".to_string())
            );
            println!();

            for step in &steps {
                let symbol = match step.state.state_type.as_str() {
                    "pipeline_state_pending" => style("[ ]").dim(),
                    "pipeline_state_in_progress" => style("[*]").cyan(),
                    "pipeline_state_completed" => {
                        match step.state.result.as_ref().map(|r| r.result_type.as_str()) {
                            Some("pipeline_state_completed_successful") => style("[OK]").green(),
                            Some("pipeline_state_completed_failed") => style("[FAILED]").red(),
                            Some("pipeline_state_completed_stopped") => style("[STOPPED]").yellow(),
                            _ => style("[?]").dim(),
                        }
                    }
                    _ => style("[?]").dim(),
                };

                let duration = step
                    .duration_in_seconds
                    .map(|d| format!(" ({})", format_duration(d)))
                    .unwrap_or_default();

                println!("  {} {}{}", symbol, step.name, duration);
            }

            // Check if pipeline is complete
            if pipeline.state.state_type == "pipeline_state_completed" {
                println!("\nPipeline completed!");

                if let Some(duration) = pipeline.duration_in_seconds {
                    println!("Total duration: {}", format_duration(duration));
                }

                if exit_status {
                    let success = pipeline
                        .state
                        .result
                        .as_ref()
                        .map(|r| r.result_type == "pipeline_state_completed_successful")
                        .unwrap_or(false);
                    if !success {
                        std::process::exit(1);
                    }
                }

                break;
            }

            // Wait before next refresh
            thread::sleep(Duration::from_secs(interval as u64));
        }

        Ok(())
    }

    /// Enable pipelines for the repository
    async fn enable(&self, global: &GlobalOptions) -> Result<()> {
        let context = self.get_cloud_context(global)?;
        let token = self.get_token(&context)?;
        let client = self.create_client()?;

        let url = format!(
            "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines_config",
            context.owner, context.repo_slug
        );

        let response = client
            .put(&url)
            .bearer_auth(&token)
            .json(&UpdatePipelinesConfig { enabled: true })
            .send()
            .await
            .context("Failed to enable pipelines")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error ({}): {}", status, body);
        }

        if global.json {
            println!(r#"{{"enabled": true}}"#);
        } else {
            println!(
                "Pipelines have been enabled for {}/{}",
                context.owner, context.repo_slug
            );
            println!("\nNext steps:");
            println!("  1. Add a bitbucket-pipelines.yml file to your repository");
            println!("  2. Push changes to trigger your first pipeline");
        }

        Ok(())
    }

    /// Disable pipelines for the repository
    async fn disable(&self, global: &GlobalOptions) -> Result<()> {
        let context = self.get_cloud_context(global)?;
        let token = self.get_token(&context)?;
        let client = self.create_client()?;

        let url = format!(
            "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines_config",
            context.owner, context.repo_slug
        );

        let response = client
            .put(&url)
            .bearer_auth(&token)
            .json(&UpdatePipelinesConfig { enabled: false })
            .send()
            .await
            .context("Failed to disable pipelines")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("API error ({}): {}", status, body);
        }

        if global.json {
            println!(r#"{{"enabled": false}}"#);
        } else {
            println!(
                "Pipelines have been disabled for {}/{}",
                context.owner, context.repo_slug
            );
        }

        Ok(())
    }

    /// View or edit pipeline configuration
    async fn config(&self, args: &ConfigArgs, global: &GlobalOptions) -> Result<()> {
        let context = self.get_cloud_context(global)?;

        let config_file = "bitbucket-pipelines.yml";

        if args.validate {
            // Read the file and validate
            let content = std::fs::read_to_string(config_file)
                .context("Failed to read bitbucket-pipelines.yml. Make sure the file exists.")?;

            // Basic check - try to detect common YAML issues
            // We just check for valid structure, not Bitbucket-specific schema
            let has_image_or_pipelines =
                content.contains("image:") || content.contains("pipelines:");
            let balanced_quotes = content.matches('"').count() % 2 == 0;
            let balanced_single_quotes = content.matches('\'').count() % 2 == 0;

            if !has_image_or_pipelines {
                println!("Warning: File doesn't contain 'image:' or 'pipelines:' sections");
            }

            if !balanced_quotes || !balanced_single_quotes {
                anyhow::bail!("Invalid YAML: Unbalanced quotes detected");
            }

            if global.json {
                println!(r#"{{"valid": true, "file": "{}"}}"#, config_file);
            } else {
                println!("bitbucket-pipelines.yml appears to be valid");
                println!("\nNote: Full pipeline validation happens when the pipeline runs.");
                println!("For detailed validation, push to Bitbucket and check the Pipeline tab.");
            }
        } else if args.edit {
            // Open in editor
            let editor = std::env::var("EDITOR")
                .or_else(|_| std::env::var("VISUAL"))
                .unwrap_or_else(|_| "vi".to_string());

            let status = ProcessCommand::new(&editor)
                .arg(config_file)
                .status()
                .context("Failed to open editor")?;

            if !status.success() {
                anyhow::bail!("Editor exited with error");
            }
        } else {
            // Show current configuration status
            let token = self.get_token(&context)?;
            let client = self.create_client()?;

            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines_config",
                context.owner, context.repo_slug
            );

            let response = client
                .get(&url)
                .bearer_auth(&token)
                .send()
                .await
                .context("Failed to fetch pipelines config")?;

            let enabled = if response.status().is_success() {
                let config: RepositoryPipelinesConfig = response
                    .json()
                    .await
                    .context("Failed to parse config response")?;
                config.enabled
            } else {
                false
            };

            let local_exists = std::path::Path::new(config_file).exists();

            if global.json {
                let output = serde_json::json!({
                    "enabled": enabled,
                    "local_config_exists": local_exists,
                });
                println!("{}", serde_json::to_string_pretty(&output)?);
            } else {
                println!(
                    "Pipelines configuration for {}/{}:",
                    context.owner, context.repo_slug
                );
                println!("  Enabled: {}", if enabled { "Yes" } else { "No" });

                if local_exists {
                    println!("\nLocal configuration file: {}", config_file);
                    println!("  Use --edit to open in editor");
                    println!("  Use --validate to check syntax");
                } else {
                    println!("\nNo local bitbucket-pipelines.yml found");
                    println!("Create one to define your CI/CD workflow");
                }
            }
        }

        Ok(())
    }

    /// Handle cache subcommands
    async fn cache(&self, command: &CacheSubcommand, global: &GlobalOptions) -> Result<()> {
        let context = self.get_cloud_context(global)?;
        let token = self.get_token(&context)?;
        let client = self.create_client()?;

        match command {
            CacheSubcommand::List => {
                let url = format!(
                    "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines-config/caches/",
                    context.owner, context.repo_slug
                );

                let response = client
                    .get(&url)
                    .bearer_auth(&token)
                    .send()
                    .await
                    .context("Failed to fetch caches")?;

                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    anyhow::bail!("API error ({}): {}", status, body);
                }

                let paginated: PaginatedResponse<PipelineCache> = response
                    .json()
                    .await
                    .context("Failed to parse caches response")?;

                if paginated.values.is_empty() {
                    println!("No caches found");
                    return Ok(());
                }

                let format = self.get_format(global);
                let writer = OutputWriter::new(format);

                if !global.json {
                    println!("{:30} {:15} CREATED", "NAME", "SIZE");
                    println!("{}", "-".repeat(60));
                }

                writer.write_list(&paginated.values)?;
            }

            CacheSubcommand::Delete { name } => {
                let url = format!(
                    "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines-config/caches/{}",
                    context.owner, context.repo_slug, name
                );

                let response = client
                    .delete(&url)
                    .bearer_auth(&token)
                    .send()
                    .await
                    .context("Failed to delete cache")?;

                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    anyhow::bail!("API error ({}): {}", status, body);
                }

                if global.json {
                    println!(r#"{{"deleted": true, "name": "{}"}}"#, name);
                } else {
                    println!("Cache '{}' deleted", name);
                }
            }

            CacheSubcommand::Clear => {
                // List and delete all caches
                let url = format!(
                    "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines-config/caches/",
                    context.owner, context.repo_slug
                );

                let response = client
                    .get(&url)
                    .bearer_auth(&token)
                    .send()
                    .await
                    .context("Failed to fetch caches")?;

                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    anyhow::bail!("API error ({}): {}", status, body);
                }

                let paginated: PaginatedResponse<PipelineCache> = response
                    .json()
                    .await
                    .context("Failed to parse caches response")?;

                if paginated.values.is_empty() {
                    if global.json {
                        println!(r#"{{"cleared": 0}}"#);
                    } else {
                        println!("No caches to clear");
                    }
                    return Ok(());
                }

                let mut deleted = 0;
                for cache in paginated.values {
                    let delete_url = format!(
                        "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines-config/caches/{}",
                        context.owner, context.repo_slug, cache.name
                    );

                    if let Ok(resp) = client.delete(&delete_url).bearer_auth(&token).send().await {
                        if resp.status().is_success() {
                            deleted += 1;
                        }
                    }
                }

                if global.json {
                    println!(r#"{{"cleared": {}}}"#, deleted);
                } else {
                    println!("Cleared {} cache(s)", deleted);
                }
            }
        }

        Ok(())
    }

    /// Handle schedule subcommands
    async fn schedule(&self, command: &ScheduleSubcommand, global: &GlobalOptions) -> Result<()> {
        let context = self.get_cloud_context(global)?;
        let token = self.get_token(&context)?;
        let client = self.create_client()?;

        match command {
            ScheduleSubcommand::List => {
                let url = format!(
                    "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines_config/schedules/",
                    context.owner, context.repo_slug
                );

                let response = client
                    .get(&url)
                    .bearer_auth(&token)
                    .send()
                    .await
                    .context("Failed to fetch schedules")?;

                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    anyhow::bail!("API error ({}): {}", status, body);
                }

                let paginated: PaginatedResponse<PipelineSchedule> = response
                    .json()
                    .await
                    .context("Failed to parse schedules response")?;

                if paginated.values.is_empty() {
                    println!("No scheduled pipelines found");
                    return Ok(());
                }

                let format = self.get_format(global);
                let writer = OutputWriter::new(format);

                if !global.json {
                    println!(
                        "{:12} {:8} {:20} {:20} PIPELINE",
                        "ID", "ENABLED", "CRON", "BRANCH"
                    );
                    println!("{}", "-".repeat(80));
                }

                writer.write_list(&paginated.values)?;
            }

            ScheduleSubcommand::Create(args) => {
                let selector = args.pipeline.as_ref().map(|p| CreateScheduleSelector {
                    type_field: "custom".to_string(),
                    pattern: p.clone(),
                });

                let request = CreateScheduleRequest {
                    type_field: "pipeline_schedule".to_string(),
                    enabled: true,
                    cron_pattern: args.cron.clone(),
                    target: CreateScheduleTarget {
                        type_field: "pipeline_ref_target".to_string(),
                        ref_type: "branch".to_string(),
                        ref_name: args.branch.clone(),
                        selector,
                    },
                };

                let url = format!(
                    "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines_config/schedules/",
                    context.owner, context.repo_slug
                );

                let response = client
                    .post(&url)
                    .bearer_auth(&token)
                    .json(&request)
                    .send()
                    .await
                    .context("Failed to create schedule")?;

                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    anyhow::bail!("API error ({}): {}", status, body);
                }

                let schedule: PipelineSchedule = response
                    .json()
                    .await
                    .context("Failed to parse schedule response")?;

                if global.json {
                    println!("{}", serde_json::to_string_pretty(&schedule)?);
                } else {
                    println!("Created schedule: {}", truncate_uuid(&schedule.uuid));
                    println!("  Cron: {}", args.cron);
                    println!("  Branch: {}", args.branch);
                    if let Some(ref pipeline) = args.pipeline {
                        println!("  Pipeline: {}", pipeline);
                    }
                }
            }

            ScheduleSubcommand::Delete { id } => {
                let url = format!(
                    "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines_config/schedules/{}",
                    context.owner, context.repo_slug, id
                );

                let response = client
                    .delete(&url)
                    .bearer_auth(&token)
                    .send()
                    .await
                    .context("Failed to delete schedule")?;

                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    anyhow::bail!("API error ({}): {}", status, body);
                }

                if global.json {
                    println!(r#"{{"deleted": true}}"#);
                } else {
                    println!("Schedule deleted");
                }
            }

            ScheduleSubcommand::Pause { id } => {
                let url = format!(
                    "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines_config/schedules/{}",
                    context.owner, context.repo_slug, id
                );

                let response = client
                    .put(&url)
                    .bearer_auth(&token)
                    .json(&serde_json::json!({ "enabled": false }))
                    .send()
                    .await
                    .context("Failed to pause schedule")?;

                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    anyhow::bail!("API error ({}): {}", status, body);
                }

                if global.json {
                    println!(r#"{{"paused": true}}"#);
                } else {
                    println!("Schedule paused");
                }
            }

            ScheduleSubcommand::Resume { id } => {
                let url = format!(
                    "https://api.bitbucket.org/2.0/repositories/{}/{}/pipelines_config/schedules/{}",
                    context.owner, context.repo_slug, id
                );

                let response = client
                    .put(&url)
                    .bearer_auth(&token)
                    .json(&serde_json::json!({ "enabled": true }))
                    .send()
                    .await
                    .context("Failed to resume schedule")?;

                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    anyhow::bail!("API error ({}): {}", status, body);
                }

                if global.json {
                    println!(r#"{{"resumed": true}}"#);
                } else {
                    println!("Schedule resumed");
                }
            }
        }

        Ok(())
    }

    /// Handle runner subcommands
    async fn runner(&self, command: &RunnerSubcommand, global: &GlobalOptions) -> Result<()> {
        let context = self.get_cloud_context(global)?;
        let token = self.get_token(&context)?;
        let client = self.create_client()?;

        match command {
            RunnerSubcommand::List => {
                // List workspace runners
                let url = format!(
                    "https://api.bitbucket.org/2.0/workspaces/{}/pipelines-config/runners/",
                    context.owner
                );

                let response = client
                    .get(&url)
                    .bearer_auth(&token)
                    .send()
                    .await
                    .context("Failed to fetch runners")?;

                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    anyhow::bail!("API error ({}): {}", status, body);
                }

                let paginated: PaginatedResponse<PipelineRunner> = response
                    .json()
                    .await
                    .context("Failed to parse runners response")?;

                if paginated.values.is_empty() {
                    println!("No runners found");
                    return Ok(());
                }

                let format = self.get_format(global);
                let writer = OutputWriter::new(format);

                if !global.json {
                    println!("{:12} {:30} {:15} LABELS", "ID", "NAME", "STATUS");
                    println!("{}", "-".repeat(80));
                }

                writer.write_list(&paginated.values)?;
            }

            RunnerSubcommand::Register => {
                if global.json {
                    let output = serde_json::json!({
                        "message": "Manual registration required",
                        "documentation_url": "https://support.atlassian.com/bitbucket-cloud/docs/runners/"
                    });
                    println!("{}", serde_json::to_string_pretty(&output)?);
                } else {
                    println!("To register a self-hosted runner:");
                    println!();
                    println!("1. Go to your workspace settings in Bitbucket");
                    println!("2. Navigate to Settings > Pipelines > Runners");
                    println!("3. Click 'Add runner'");
                    println!("4. Follow the instructions to download and configure the runner");
                    println!();
                    println!("For more information, visit:");
                    println!("https://support.atlassian.com/bitbucket-cloud/docs/runners/");
                }
            }

            RunnerSubcommand::Remove { id } => {
                let url = format!(
                    "https://api.bitbucket.org/2.0/workspaces/{}/pipelines-config/runners/{}",
                    context.owner, id
                );

                let response = client
                    .delete(&url)
                    .bearer_auth(&token)
                    .send()
                    .await
                    .context("Failed to remove runner")?;

                if !response.status().is_success() {
                    let status = response.status();
                    let body = response.text().await.unwrap_or_default();
                    anyhow::bail!("API error ({}): {}", status, body);
                }

                if global.json {
                    println!(r#"{{"removed": true}}"#);
                } else {
                    println!("Runner removed");
                }
            }
        }

        Ok(())
    }
}

// Helper functions

fn get_current_branch() -> Result<String> {
    let output = ProcessCommand::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .context("Failed to run git command")?;

    if !output.status.success() {
        anyhow::bail!("Not in a git repository or no commits yet");
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn format_pipeline_id(id: &str) -> String {
    // If it looks like a build number or already wrapped in braces, return as-is
    if id.chars().all(|c| c.is_ascii_digit()) || (id.starts_with('{') && id.ends_with('}')) {
        id.to_string()
    } else {
        format!("{{{}}}", id)
    }
}

fn format_timestamp(ts: &str) -> String {
    if let Ok(dt) = DateTime::parse_from_rfc3339(ts) {
        let utc: DateTime<Utc> = dt.into();
        let now = Utc::now();
        let diff = now.signed_duration_since(utc);

        if diff.num_minutes() < 1 {
            "just now".to_string()
        } else if diff.num_minutes() < 60 {
            format!("{} min ago", diff.num_minutes())
        } else if diff.num_hours() < 24 {
            format!("{} hours ago", diff.num_hours())
        } else if diff.num_days() < 7 {
            format!("{} days ago", diff.num_days())
        } else {
            utc.format("%Y-%m-%d %H:%M").to_string()
        }
    } else {
        ts.to_string()
    }
}

fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    }
}

fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes < KB {
        format!("{} B", bytes)
    } else if bytes < MB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else if bytes < GB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    }
}

fn truncate_uuid(uuid: &str) -> String {
    let trimmed = uuid.trim_matches(|c| c == '{' || c == '}');
    if trimmed.len() > 8 {
        format!("{}...", &trimmed[..8])
    } else {
        trimmed.to_string()
    }
}
