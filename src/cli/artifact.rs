//
//  bitbucket-cli
//  cli/artifact.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Build artifact management commands (Cloud only)
//!
//! This module provides commands for managing pipeline build artifacts.
//! Artifacts are files produced during pipeline runs that can be downloaded.
//!
//! ## Examples
//!
//! ```bash
//! # List artifacts from latest pipeline
//! bb artifact list
//!
//! # List artifacts from specific pipeline
//! bb artifact list --pipeline 123
//!
//! # Download an artifact
//! bb artifact download build.zip --pipeline 123 --dir ./downloads
//!
//! # Delete an artifact
//! bb artifact delete <artifact-id>
//! ```

use std::fs;
use std::io::Write;
use std::path::Path;

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};

use crate::api::common::PaginatedResponse;
use crate::api::BitbucketClient;
use crate::auth::{AuthCredential, KeyringStore};
use crate::config::Config;
use crate::context::{ContextResolver, HostType, RepoContext};
use crate::output::OutputWriter;

use super::GlobalOptions;

/// Manage build artifacts (Cloud only)
#[derive(Args, Debug)]
pub struct ArtifactCommand {
    #[command(subcommand)]
    pub command: ArtifactSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ArtifactSubcommand {
    /// List artifacts
    #[command(visible_alias = "ls")]
    List(ListArgs),

    /// Download an artifact
    Download(DownloadArgs),

    /// Delete an artifact
    Delete(DeleteArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter by pipeline build number
    #[arg(long, short = 'p')]
    pub pipeline: Option<u64>,

    /// Maximum number to show
    #[arg(long, short = 'l', default_value = "25")]
    pub limit: usize,
}

#[derive(Args, Debug)]
pub struct DownloadArgs {
    /// Artifact name or pattern
    pub name: String,

    /// Pipeline build number
    #[arg(long, short = 'p')]
    pub pipeline: u64,

    /// Step name
    #[arg(long, short = 's')]
    pub step: Option<String>,

    /// Output directory
    #[arg(long, short = 'd', default_value = ".")]
    pub dir: String,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// Artifact name
    pub name: String,

    /// Pipeline build number
    #[arg(long, short = 'p')]
    pub pipeline: u64,

    /// Skip confirmation
    #[arg(long, short = 'y')]
    pub confirm: bool,
}

// API response types

#[derive(Debug, Deserialize)]
struct PipelineArtifact {
    name: String,
    #[serde(default)]
    size: Option<u64>,
    #[serde(default)]
    step_uuid: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Pipeline {
    build_number: u64,
    state: PipelineState,
    #[serde(default)]
    created_on: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PipelineState {
    name: String,
}

// Output types

#[derive(Debug, Serialize)]
struct ArtifactListItem {
    name: String,
    pipeline: u64,
    size: Option<u64>,
    step: Option<String>,
}

impl crate::output::TableOutput for ArtifactListItem {
    fn print_table(&self, _color: bool) {
        let size_str = self
            .size
            .map(|s| format_bytes(s))
            .unwrap_or_else(|| "-".to_string());
        let step_str = self
            .step
            .as_ref()
            .map(|s| truncate(s, 15))
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<40} {:<15} {}",
            truncate(&self.name, 38),
            size_str,
            step_str
        );
    }

    fn print_markdown(&self) {
        println!(
            "| {} | {} | {} | {} |",
            self.name,
            self.pipeline,
            self.size.unwrap_or(0),
            self.step.as_deref().unwrap_or("-")
        );
    }
}

impl ArtifactCommand {
    pub async fn run(&self, global: &GlobalOptions) -> Result<()> {
        match &self.command {
            ArtifactSubcommand::List(args) => self.list(args, global).await,
            ArtifactSubcommand::Download(args) => self.download(args, global).await,
            ArtifactSubcommand::Delete(args) => self.delete(args, global).await,
        }
    }

    fn resolve_context(&self, global: &GlobalOptions) -> Result<RepoContext> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        resolver.resolve(global)
    }

    fn get_client(&self, ctx: &RepoContext) -> Result<BitbucketClient> {
        if !matches!(ctx.host_type, HostType::Cloud) {
            bail!("Artifacts are only available for Bitbucket Cloud.");
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

    /// List artifacts
    async fn list(&self, args: &ListArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        // Get pipeline number to list artifacts from
        let pipeline_num = if let Some(num) = args.pipeline {
            num
        } else {
            // Get latest successful pipeline
            let pipelines_url = format!(
                "/repositories/{}/{}/pipelines?sort=-build_number&pagelen=1",
                ctx.owner, ctx.repo_slug
            );

            let pipelines: PaginatedResponse<Pipeline> = client.get(&pipelines_url).await?;

            let latest = pipelines
                .values
                .first()
                .ok_or_else(|| anyhow::anyhow!("No pipelines found"))?;

            latest.build_number
        };

        // List artifacts for this pipeline
        let url = format!(
            "/repositories/{}/{}/pipelines/{}/artifacts?pagelen={}",
            ctx.owner, ctx.repo_slug, pipeline_num, args.limit
        );

        let response: PaginatedResponse<PipelineArtifact> = client.get(&url).await?;

        let items: Vec<ArtifactListItem> = response
            .values
            .into_iter()
            .map(|a| ArtifactListItem {
                name: a.name,
                pipeline: pipeline_num,
                size: a.size,
                step: a.step_uuid,
            })
            .collect();

        if items.is_empty() {
            println!("No artifacts found for pipeline #{}.", pipeline_num);
            return Ok(());
        }

        let writer = OutputWriter::new(self.get_format(global));

        if !global.json {
            println!();
            println!(
                "{}",
                style(format!("Artifacts from Pipeline #{}", pipeline_num)).bold()
            );
            println!("{}", "-".repeat(70));
            println!(
                "{} {} {}",
                style(format!("{:<40}", "NAME")).bold(),
                style(format!("{:<15}", "SIZE")).bold(),
                style("STEP").bold()
            );
            println!("{}", "-".repeat(70));

            for item in &items {
                let size_str = item
                    .size
                    .map(|s| format_bytes(s))
                    .unwrap_or_else(|| "-".to_string());

                let step_str = item
                    .step
                    .as_ref()
                    .map(|s| truncate(s, 15))
                    .unwrap_or_else(|| "-".to_string());

                println!(
                    "{:<40} {:<15} {}",
                    truncate(&item.name, 38),
                    size_str,
                    step_str
                );
            }

            println!();
            println!("Showing {} artifact(s)", items.len());
        } else {
            writer.write_list(&items)?;
        }

        Ok(())
    }

    /// Download an artifact
    async fn download(&self, args: &DownloadArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        // Ensure output directory exists
        let output_dir = Path::new(&args.dir);
        if !output_dir.exists() {
            fs::create_dir_all(output_dir)?;
        }

        println!(
            "{} Downloading artifact '{}' from pipeline #{}...",
            style("→").cyan(),
            args.name,
            args.pipeline
        );

        // Get download URL
        let url = format!(
            "/repositories/{}/{}/pipelines/{}/artifacts/{}/download",
            ctx.owner, ctx.repo_slug, args.pipeline, args.name
        );

        // Make the download request
        let keyring = KeyringStore::new();
        let token = keyring
            .get(&ctx.host)?
            .ok_or_else(|| anyhow::anyhow!("Not authenticated"))?;

        let http_client = reqwest::Client::new();
        let response = http_client
            .get(format!("https://api.bitbucket.org/2.0{}", url))
            .bearer_auth(&token)
            .send()
            .await?;

        if !response.status().is_success() {
            bail!("Failed to download artifact: {}", response.status());
        }

        let total_size = response.content_length().unwrap_or(0);
        let output_path = output_dir.join(&args.name);

        let pb = ProgressBar::new(total_size);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap()
                .progress_chars("#>-"),
        );

        let bytes = response.bytes().await?;
        let mut file = fs::File::create(&output_path)?;
        file.write_all(&bytes)?;
        pb.finish_with_message("Downloaded");

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "name": args.name,
                "pipeline": args.pipeline,
                "path": output_path.display().to_string(),
                "size": bytes.len(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!();
            println!(
                "{} Downloaded to {}",
                style("✓").green(),
                output_path.display()
            );
            println!("  Size: {}", format_bytes(bytes.len() as u64));
        }

        Ok(())
    }

    /// Delete an artifact
    async fn delete(&self, args: &DeleteArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        // Confirm deletion
        if !args.confirm && !global.no_prompt {
            use dialoguer::Confirm;
            let confirmed = Confirm::new()
                .with_prompt(format!(
                    "Delete artifact '{}' from pipeline #{}?",
                    args.name, args.pipeline
                ))
                .default(false)
                .interact()?;

            if !confirmed {
                println!("{} Cancelled.", style("!").yellow());
                return Ok(());
            }
        }

        let url = format!(
            "/repositories/{}/{}/pipelines/{}/artifacts/{}",
            ctx.owner, ctx.repo_slug, args.pipeline, args.name
        );

        client.delete(&url).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "name": args.name,
                "pipeline": args.pipeline,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Deleted artifact '{}' from pipeline #{}",
                style("✓").green(),
                style(&args.name).cyan(),
                args.pipeline
            );
        }

        Ok(())
    }
}

/// Format bytes to human-readable string
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
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
