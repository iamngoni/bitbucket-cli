//
//  bitbucket-cli
//  cli/repo.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Repository commands for the Bitbucket CLI.
//!
//! This module provides comprehensive repository management functionality including:
//! - Listing repositories in a workspace or project
//! - Viewing repository details
//! - Creating new repositories
//! - Cloning repositories
//! - Forking repositories
//! - Deleting repositories
//! - Archive/unarchive operations
//! - Repository settings management

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use console::style;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::process::Command;

use super::GlobalOptions;
use crate::api::cloud::repositories as cloud_repos;
use crate::api::common::{PaginatedResponse, ServerPaginatedResponse};
use crate::api::format_api_error;
use crate::api::server::repositories as server_repos;
use crate::auth::KeyringStore;
use crate::config::{is_cloud_host, Config};
use crate::context::{ContextResolver, HostType, RepoContext};
use crate::output::{print_field, print_header, OutputFormat, OutputWriter, TableOutput};

/// Manage repositories
#[derive(Args, Debug)]
pub struct RepoCommand {
    #[command(subcommand)]
    pub command: RepoSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum RepoSubcommand {
    /// List repositories
    #[command(visible_alias = "ls")]
    List(ListArgs),

    /// View repository details
    View(ViewArgs),

    /// Create a new repository
    Create(CreateArgs),

    /// Clone a repository
    Clone(CloneArgs),

    /// Fork a repository
    Fork(ForkArgs),

    /// Delete a repository
    Delete(DeleteArgs),

    /// Archive a repository
    Archive,

    /// Unarchive a repository
    Unarchive,

    /// Rename a repository
    Rename(RenameArgs),

    /// Sync a fork with upstream
    Sync(SyncArgs),

    /// Edit repository settings
    Edit(EditArgs),

    /// Open repository in browser
    Browse,

    /// View repository contributors
    Credits,
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter by workspace (Cloud)
    #[arg(long, short = 'w')]
    pub workspace: Option<String>,

    /// Filter by project
    #[arg(long, short = 'p')]
    pub project: Option<String>,

    /// Maximum number of repositories to list
    #[arg(long, short = 'L', default_value = "30")]
    pub limit: u32,

    /// Filter by primary language
    #[arg(long)]
    pub language: Option<String>,

    /// Filter by visibility
    #[arg(long, value_parser = ["public", "private"])]
    pub visibility: Option<String>,

    /// Include archived repositories
    #[arg(long)]
    pub archived: bool,
}

#[derive(Args, Debug)]
pub struct ViewArgs {
    /// Repository to view (WORKSPACE/REPO or PROJECT/REPO)
    pub repo: Option<String>,

    /// Open in browser
    #[arg(long, short = 'w')]
    pub web: bool,
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Repository name
    pub name: Option<String>,

    /// Make the repository public
    #[arg(long, conflicts_with = "private")]
    pub public: bool,

    /// Make the repository private
    #[arg(long, conflicts_with = "public")]
    pub private: bool,

    /// Repository description
    #[arg(long, short = 'd')]
    pub description: Option<String>,

    /// Clone the repository after creation
    #[arg(long, short = 'c')]
    pub clone: bool,

    /// Target workspace (Cloud)
    #[arg(long, short = 'w')]
    pub workspace: Option<String>,

    /// Target project
    #[arg(long, short = 'p')]
    pub project: Option<String>,

    /// Use a template repository
    #[arg(long)]
    pub template: Option<String>,
}

#[derive(Args, Debug)]
pub struct CloneArgs {
    /// Repository to clone (WORKSPACE/REPO, PROJECT/REPO, or URL)
    pub repo: String,

    /// Directory to clone into
    pub directory: Option<String>,

    /// Additional git clone arguments
    #[arg(last = true)]
    pub git_args: Vec<String>,
}

#[derive(Args, Debug)]
pub struct ForkArgs {
    /// Repository to fork
    pub repo: Option<String>,

    /// Target workspace for the fork
    #[arg(long, short = 'w')]
    pub workspace: Option<String>,

    /// Clone the fork after creation
    #[arg(long, short = 'c')]
    pub clone: bool,

    /// Name for the fork remote
    #[arg(long, default_value = "fork")]
    pub remote_name: String,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// Repository to delete
    pub repo: Option<String>,

    /// Skip confirmation prompt
    #[arg(long)]
    pub confirm: bool,
}

#[derive(Args, Debug)]
pub struct RenameArgs {
    /// New name for the repository
    pub new_name: String,
}

#[derive(Args, Debug)]
pub struct SyncArgs {
    /// Branch to sync
    #[arg(long, short = 'b')]
    pub branch: Option<String>,

    /// Force sync even if not a fast-forward
    #[arg(long, short = 'f')]
    pub force: bool,
}

#[derive(Args, Debug)]
pub struct EditArgs {
    /// New description
    #[arg(long, short = 'd')]
    pub description: Option<String>,

    /// New default branch
    #[arg(long)]
    pub default_branch: Option<String>,

    /// Enable issues
    #[arg(long)]
    pub enable_issues: bool,

    /// Disable issues
    #[arg(long, conflicts_with = "enable_issues")]
    pub disable_issues: bool,

    /// Enable wiki
    #[arg(long)]
    pub enable_wiki: bool,

    /// Disable wiki
    #[arg(long, conflicts_with = "enable_wiki")]
    pub disable_wiki: bool,

    /// Change visibility
    #[arg(long, value_parser = ["public", "private"])]
    pub visibility: Option<String>,
}

/// Display format for repository in list output
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RepoListItem {
    full_name: String,
    description: Option<String>,
    is_private: bool,
    language: Option<String>,
    updated_on: String,
}

impl TableOutput for RepoListItem {
    fn print_table(&self, color: bool) {
        let visibility = if self.is_private { "private" } else { "public" };
        let visibility_styled = if color {
            if self.is_private {
                style(visibility).yellow().to_string()
            } else {
                style(visibility).green().to_string()
            }
        } else {
            visibility.to_string()
        };

        let lang = self.language.as_deref().unwrap_or("-");
        let desc = self.description.as_deref().unwrap_or("");
        let desc_truncated = if desc.len() > 50 {
            format!("{}...", &desc[..47])
        } else {
            desc.to_string()
        };

        println!(
            "{:<40} {:<10} {:<12} {}",
            self.full_name, visibility_styled, lang, desc_truncated
        );
    }

    fn print_markdown(&self) {
        let visibility = if self.is_private { "private" } else { "public" };
        println!(
            "- **{}** ({}) - {}",
            self.full_name,
            visibility,
            self.description.as_deref().unwrap_or("No description")
        );
    }
}

/// Display format for repository detail view
#[derive(Debug, Clone, Serialize, Deserialize)]
struct RepoDetail {
    full_name: String,
    description: Option<String>,
    is_private: bool,
    language: Option<String>,
    default_branch: Option<String>,
    clone_url_ssh: Option<String>,
    clone_url_https: Option<String>,
    web_url: String,
    created_on: String,
    updated_on: String,
}

impl TableOutput for RepoDetail {
    fn print_table(&self, color: bool) {
        print_header(&self.full_name);
        println!();

        if let Some(desc) = &self.description {
            if !desc.is_empty() {
                print_field("Description", desc, color);
            }
        }

        let visibility = if self.is_private { "private" } else { "public" };
        print_field("Visibility", visibility, color);

        if let Some(lang) = &self.language {
            print_field("Language", lang, color);
        }

        if let Some(branch) = &self.default_branch {
            print_field("Default branch", branch, color);
        }

        println!();
        print_field("Web URL", &self.web_url, color);

        if let Some(ssh) = &self.clone_url_ssh {
            print_field("SSH URL", ssh, color);
        }

        if let Some(https) = &self.clone_url_https {
            print_field("HTTPS URL", https, color);
        }

        println!();
        print_field("Created", &self.created_on, color);
        print_field("Updated", &self.updated_on, color);
    }

    fn print_markdown(&self) {
        println!("# {}", self.full_name);
        println!();

        if let Some(desc) = &self.description {
            if !desc.is_empty() {
                println!("{}", desc);
                println!();
            }
        }

        println!("## Details");
        println!();
        let visibility = if self.is_private { "Private" } else { "Public" };
        println!("- **Visibility**: {}", visibility);

        if let Some(lang) = &self.language {
            println!("- **Language**: {}", lang);
        }

        if let Some(branch) = &self.default_branch {
            println!("- **Default Branch**: {}", branch);
        }

        println!();
        println!("## Clone URLs");
        println!();

        if let Some(ssh) = &self.clone_url_ssh {
            println!("```");
            println!("git clone {}", ssh);
            println!("```");
        }

        if let Some(https) = &self.clone_url_https {
            println!("```");
            println!("git clone {}", https);
            println!("```");
        }
    }
}

impl RepoCommand {
    pub async fn run(&self, global: &GlobalOptions) -> Result<()> {
        match &self.command {
            RepoSubcommand::List(args) => self.list(args, global).await,
            RepoSubcommand::View(args) => self.view(args, global).await,
            RepoSubcommand::Create(args) => self.create(args, global).await,
            RepoSubcommand::Clone(args) => self.clone_repo(args, global).await,
            RepoSubcommand::Fork(args) => self.fork(args, global).await,
            RepoSubcommand::Delete(args) => self.delete(args, global).await,
            RepoSubcommand::Archive => self.archive(global).await,
            RepoSubcommand::Unarchive => self.unarchive(global).await,
            RepoSubcommand::Rename(args) => self.rename(args, global).await,
            RepoSubcommand::Sync(args) => self.sync(args, global).await,
            RepoSubcommand::Edit(args) => self.edit(args, global).await,
            RepoSubcommand::Browse => self.browse(global).await,
            RepoSubcommand::Credits => self.credits(global).await,
        }
    }

    /// List repositories
    async fn list(&self, args: &ListArgs, global: &GlobalOptions) -> Result<()> {
        let _config = Config::load().unwrap_or_default();
        let keyring = KeyringStore::new();

        // Determine host and type
        let host = global
            .host
            .clone()
            .or_else(|| args.workspace.as_ref().map(|_| "bitbucket.org".to_string()))
            .unwrap_or_else(|| "bitbucket.org".to_string());

        let is_cloud = is_cloud_host(&host);

        // Get authentication token
        let token = keyring.get(&host)?.ok_or_else(|| {
            anyhow::anyhow!("Not authenticated for {}. Run 'bb auth login' first.", host)
        })?;

        let client = Client::builder()
            .user_agent(format!("bb/{}", crate::VERSION))
            .build()?;

        let output = OutputWriter::new(if global.json {
            OutputFormat::Json
        } else {
            OutputFormat::Table
        });

        if is_cloud {
            // Bitbucket Cloud
            let workspace = args
                .workspace
                .as_ref()
                .or(global.workspace.as_ref())
                .ok_or_else(|| {
                    anyhow::anyhow!("Workspace required. Use --workspace or set default.")
                })?;

            let mut url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}?pagelen={}",
                workspace, args.limit
            );

            // Add filters
            let mut q_parts: Vec<String> = Vec::new();

            if let Some(lang) = &args.language {
                q_parts.push(format!("language=\"{}\"", lang));
            }

            if let Some(visibility) = &args.visibility {
                let is_private = visibility == "private";
                q_parts.push(format!("is_private={}", is_private));
            }

            if let Some(project) = &args.project {
                q_parts.push(format!("project.key=\"{}\"", project));
            }

            if !q_parts.is_empty() {
                url.push_str(&format!("&q={}", q_parts.join(" AND ")));
            }

            let response = client
                .get(&url)
                .bearer_auth(&token)
                .send()
                .await
                .context("Failed to connect to Bitbucket Cloud")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(format_api_error(status, &text));
            }

            let repos: PaginatedResponse<cloud_repos::Repository> = response.json().await?;

            // Convert to list items
            let items: Vec<RepoListItem> = repos
                .values
                .into_iter()
                .map(|r| RepoListItem {
                    full_name: r.full_name,
                    description: r.description,
                    is_private: r.is_private,
                    language: r.language,
                    updated_on: r.updated_on,
                })
                .collect();

            if items.is_empty() {
                println!("No repositories found in workspace '{}'", workspace);
            } else {
                if !global.json {
                    println!("Repositories in '{}':\n", workspace);
                    println!(
                        "{:<40} {:<10} {:<12} DESCRIPTION",
                        "NAME", "VISIBILITY", "LANGUAGE"
                    );
                    println!("{}", "-".repeat(90));
                }
                output.write_list(&items)?;
            }
        } else {
            // Bitbucket Server/DC
            let project = args
                .project
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Project required for Server/DC. Use --project."))?;

            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos?limit={}",
                host, project, args.limit
            );

            let response = client
                .get(&url)
                .bearer_auth(&token)
                .send()
                .await
                .context("Failed to connect to Bitbucket Server")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(format_api_error(status, &text));
            }

            let repos: ServerPaginatedResponse<server_repos::Repository> = response.json().await?;

            // Convert to list items
            let items: Vec<RepoListItem> = repos
                .values
                .into_iter()
                .map(|r| RepoListItem {
                    full_name: format!("{}/{}", r.project.key, r.slug),
                    description: r.description,
                    is_private: !r.is_public,
                    language: None,              // Server doesn't have language field
                    updated_on: "-".to_string(), // Server doesn't have updated_on in list
                })
                .collect();

            if items.is_empty() {
                println!("No repositories found in project '{}'", project);
            } else {
                if !global.json {
                    println!("Repositories in '{}':\n", project);
                    println!(
                        "{:<40} {:<10} {:<12} DESCRIPTION",
                        "NAME", "VISIBILITY", "LANGUAGE"
                    );
                    println!("{}", "-".repeat(90));
                }
                output.write_list(&items)?;
            }
        }

        Ok(())
    }

    /// View repository details
    async fn view(&self, args: &ViewArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config.clone());

        // Get repo context
        let context = if let Some(repo) = &args.repo {
            resolver.parse_repo_arg_with_host(repo, global)?
        } else {
            resolver.resolve(global)?
        };

        // If web flag, open in browser
        if args.web {
            let url = context.web_url();
            webbrowser::open(&url)?;
            println!("Opened {} in browser", url);
            return Ok(());
        }

        let keyring = KeyringStore::new();
        let token = keyring.get(&context.host)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Not authenticated for {}. Run 'bb auth login' first.",
                context.host
            )
        })?;

        let client = Client::builder()
            .user_agent(format!("bb/{}", crate::VERSION))
            .build()?;

        let output = OutputWriter::new(if global.json {
            OutputFormat::Json
        } else {
            OutputFormat::Table
        });

        let detail = if context.host_type == HostType::Cloud {
            // Bitbucket Cloud
            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}",
                context.owner, context.repo_slug
            );

            let response = client
                .get(&url)
                .bearer_auth(&token)
                .send()
                .await
                .context("Failed to connect to Bitbucket Cloud")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(format_api_error(status, &text));
            }

            let repo: cloud_repos::Repository = response.json().await?;

            RepoDetail {
                full_name: repo.full_name.clone(),
                description: repo.description,
                is_private: repo.is_private,
                language: repo.language,
                default_branch: repo.mainbranch.map(|b| b.name),
                clone_url_ssh: Some(format!("git@bitbucket.org:{}.git", repo.full_name)),
                clone_url_https: Some(format!("https://bitbucket.org/{}.git", repo.full_name)),
                web_url: format!("https://bitbucket.org/{}", repo.full_name),
                created_on: repo.created_on,
                updated_on: repo.updated_on,
            }
        } else {
            // Bitbucket Server/DC
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}",
                context.host, context.owner, context.repo_slug
            );

            let response = client
                .get(&url)
                .bearer_auth(&token)
                .send()
                .await
                .context("Failed to connect to Bitbucket Server")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(format_api_error(status, &text));
            }

            let repo: server_repos::Repository = response.json().await?;

            // Extract clone URLs
            let ssh_url = repo
                .links
                .clone
                .iter()
                .find(|l| l.name == "ssh")
                .map(|l| l.href.clone());
            let https_url = repo
                .links
                .clone
                .iter()
                .find(|l| l.name == "http")
                .map(|l| l.href.clone());
            let web_url = repo
                .links
                .self_link
                .first()
                .map(|l| l.href.clone())
                .unwrap_or_else(|| {
                    format!(
                        "https://{}/projects/{}/repos/{}",
                        context.host, context.owner, context.repo_slug
                    )
                });

            RepoDetail {
                full_name: format!("{}/{}", repo.project.key, repo.slug),
                description: repo.description,
                is_private: !repo.is_public,
                language: None,
                default_branch: None, // Need separate API call for this
                clone_url_ssh: ssh_url,
                clone_url_https: https_url,
                web_url,
                created_on: "-".to_string(),
                updated_on: "-".to_string(),
            }
        };

        output.write(&detail)?;

        Ok(())
    }

    /// Create a new repository
    async fn create(&self, args: &CreateArgs, global: &GlobalOptions) -> Result<()> {
        let _config = Config::load().unwrap_or_default();
        let keyring = KeyringStore::new();

        // Get repository name
        let name = if let Some(n) = &args.name {
            n.clone()
        } else {
            // Interactive mode
            use dialoguer::Input;
            Input::new()
                .with_prompt("Repository name")
                .interact_text()?
        };

        // Determine host
        let host = global
            .host
            .clone()
            .unwrap_or_else(|| "bitbucket.org".to_string());

        let is_cloud = is_cloud_host(&host);

        // Get authentication token
        let token = keyring.get(&host)?.ok_or_else(|| {
            anyhow::anyhow!("Not authenticated for {}. Run 'bb auth login' first.", host)
        })?;

        let client = Client::builder()
            .user_agent(format!("bb/{}", crate::VERSION))
            .build()?;

        let output = OutputWriter::new(if global.json {
            OutputFormat::Json
        } else {
            OutputFormat::Table
        });

        if is_cloud {
            // Bitbucket Cloud
            let workspace = args
                .workspace
                .as_ref()
                .or(global.workspace.as_ref())
                .ok_or_else(|| {
                    anyhow::anyhow!("Workspace required. Use --workspace or set default.")
                })?;

            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}",
                workspace,
                name.to_lowercase().replace(' ', "-")
            );

            let body = cloud_repos::CreateRepositoryRequest {
                name: name.clone(),
                description: args.description.clone(),
                is_private: Some(!args.public),
                project: args
                    .project
                    .as_ref()
                    .map(|p| cloud_repos::ProjectKey { key: p.clone() }),
                language: None,
            };

            let response = client
                .post(&url)
                .bearer_auth(&token)
                .json(&body)
                .send()
                .await
                .context("Failed to connect to Bitbucket Cloud")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to create repository ({}): {}", status, text);
            }

            let repo: cloud_repos::Repository = response.json().await?;

            output.write_success(&format!("Created repository {}", repo.full_name));

            // Clone if requested
            if args.clone {
                let clone_url = format!("git@bitbucket.org:{}.git", repo.full_name);
                self.do_clone(&clone_url, Some(&name), &[])?;
            }
        } else {
            // Bitbucket Server/DC
            let project = args
                .project
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Project required for Server/DC. Use --project."))?;

            let url = format!("https://{}/rest/api/1.0/projects/{}/repos", host, project);

            let body = server_repos::CreateRepositoryRequest {
                name: name.clone(),
                description: args.description.clone(),
                scm_id: "git".to_string(),
                forkable: Some(true),
                is_public: Some(args.public),
            };

            let response = client
                .post(&url)
                .bearer_auth(&token)
                .json(&body)
                .send()
                .await
                .context("Failed to connect to Bitbucket Server")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to create repository ({}): {}", status, text);
            }

            let repo: server_repos::Repository = response.json().await?;

            output.write_success(&format!(
                "Created repository {}/{}",
                repo.project.key, repo.slug
            ));

            // Clone if requested
            if args.clone {
                if let Some(ssh_link) = repo.links.clone.iter().find(|l| l.name == "ssh") {
                    self.do_clone(&ssh_link.href, Some(&name), &[])?;
                }
            }
        }

        Ok(())
    }

    /// Clone a repository
    async fn clone_repo(&self, args: &CloneArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let keyring = KeyringStore::new();

        // Check if it's already a URL
        let clone_url = if args.repo.starts_with("git@")
            || args.repo.starts_with("https://")
            || args.repo.starts_with("ssh://")
        {
            args.repo.clone()
        } else {
            // Parse as WORKSPACE/REPO or PROJECT/REPO
            let resolver = ContextResolver::new(config.clone());
            let context = resolver.parse_repo_arg_with_host(&args.repo, global)?;

            // Get clone URL based on protocol preference
            let use_ssh = config.core.git_protocol != "https";

            if context.host_type == HostType::Cloud {
                if use_ssh {
                    format!(
                        "git@bitbucket.org:{}/{}.git",
                        context.owner, context.repo_slug
                    )
                } else {
                    format!(
                        "https://bitbucket.org/{}/{}.git",
                        context.owner, context.repo_slug
                    )
                }
            } else {
                // For server, we need to fetch the actual clone URL
                let token = keyring.get(&context.host)?.ok_or_else(|| {
                    anyhow::anyhow!(
                        "Not authenticated for {}. Run 'bb auth login' first.",
                        context.host
                    )
                })?;

                let client = Client::builder()
                    .user_agent(format!("bb/{}", crate::VERSION))
                    .build()?;

                let url = format!(
                    "https://{}/rest/api/1.0/projects/{}/repos/{}",
                    context.host, context.owner, context.repo_slug
                );

                let response = client.get(&url).bearer_auth(&token).send().await?;

                if !response.status().is_success() {
                    anyhow::bail!(
                        "Repository not found: {}/{}",
                        context.owner,
                        context.repo_slug
                    );
                }

                let repo: server_repos::Repository = response.json().await?;

                let link_name = if use_ssh { "ssh" } else { "http" };
                repo.links
                    .clone
                    .iter()
                    .find(|l| l.name == link_name)
                    .map(|l| l.href.clone())
                    .ok_or_else(|| anyhow::anyhow!("No {} clone URL available", link_name))?
            }
        };

        self.do_clone(&clone_url, args.directory.as_deref(), &args.git_args)?;

        Ok(())
    }

    /// Execute git clone command
    fn do_clone(&self, url: &str, directory: Option<&str>, extra_args: &[String]) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.arg("clone");

        for arg in extra_args {
            cmd.arg(arg);
        }

        cmd.arg(url);

        if let Some(dir) = directory {
            cmd.arg(dir);
        }

        let status = cmd.status().context("Failed to execute git clone")?;

        if !status.success() {
            anyhow::bail!(
                "git clone failed with exit code: {}",
                status.code().unwrap_or(-1)
            );
        }

        Ok(())
    }

    /// Fork a repository
    async fn fork(&self, args: &ForkArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config.clone());
        let keyring = KeyringStore::new();

        // Get source repo context
        let context = if let Some(repo) = &args.repo {
            resolver.parse_repo_arg_with_host(repo, global)?
        } else {
            resolver.resolve(global)?
        };

        let token = keyring.get(&context.host)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Not authenticated for {}. Run 'bb auth login' first.",
                context.host
            )
        })?;

        let client = Client::builder()
            .user_agent(format!("bb/{}", crate::VERSION))
            .build()?;

        let output = OutputWriter::new(if global.json {
            OutputFormat::Json
        } else {
            OutputFormat::Table
        });

        if context.host_type == HostType::Cloud {
            // Bitbucket Cloud
            let target_workspace = args
                .workspace
                .as_ref()
                .or(global.workspace.as_ref())
                .ok_or_else(|| {
                    anyhow::anyhow!("Target workspace required for fork. Use --workspace.")
                })?;

            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/forks",
                context.owner, context.repo_slug
            );

            #[derive(Serialize)]
            struct ForkRequest {
                workspace: WorkspaceRef,
            }

            #[derive(Serialize)]
            struct WorkspaceRef {
                slug: String,
            }

            let body = ForkRequest {
                workspace: WorkspaceRef {
                    slug: target_workspace.clone(),
                },
            };

            let response = client
                .post(&url)
                .bearer_auth(&token)
                .json(&body)
                .send()
                .await
                .context("Failed to connect to Bitbucket Cloud")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to fork repository ({}): {}", status, text);
            }

            let repo: cloud_repos::Repository = response.json().await?;

            output.write_success(&format!("Forked to {}", repo.full_name));

            // Clone if requested
            if args.clone {
                let clone_url = format!("git@bitbucket.org:{}.git", repo.full_name);
                let dir_name = repo.slug.clone();
                self.do_clone(&clone_url, Some(&dir_name), &[])?;
            }
        } else {
            // Bitbucket Server/DC
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}/forks",
                context.host, context.owner, context.repo_slug
            );

            // For Server, we need to specify target project
            let target_project = args.workspace.as_ref().ok_or_else(|| {
                anyhow::anyhow!("Target project required for Server/DC fork. Use --workspace.")
            })?;

            #[derive(Serialize)]
            struct ForkRequest {
                slug: Option<String>,
                project: ProjectRef,
            }

            #[derive(Serialize)]
            struct ProjectRef {
                key: String,
            }

            let body = ForkRequest {
                slug: None,
                project: ProjectRef {
                    key: target_project.clone(),
                },
            };

            let response = client
                .post(&url)
                .bearer_auth(&token)
                .json(&body)
                .send()
                .await
                .context("Failed to connect to Bitbucket Server")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to fork repository ({}): {}", status, text);
            }

            let repo: server_repos::Repository = response.json().await?;

            output.write_success(&format!("Forked to {}/{}", repo.project.key, repo.slug));

            // Clone if requested
            if args.clone {
                if let Some(ssh_link) = repo.links.clone.iter().find(|l| l.name == "ssh") {
                    self.do_clone(&ssh_link.href, Some(&repo.slug), &[])?;
                }
            }
        }

        Ok(())
    }

    /// Delete a repository
    async fn delete(&self, args: &DeleteArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config.clone());
        let keyring = KeyringStore::new();

        // Get repo context
        let context = if let Some(repo) = &args.repo {
            resolver.parse_repo_arg_with_host(repo, global)?
        } else {
            resolver.resolve(global)?
        };

        // Confirm deletion
        if !args.confirm {
            use dialoguer::Confirm;
            let confirmed = Confirm::new()
                .with_prompt(format!(
                    "Are you sure you want to delete {}? This cannot be undone!",
                    context.full_name()
                ))
                .default(false)
                .interact()?;

            if !confirmed {
                println!("Cancelled.");
                return Ok(());
            }
        }

        let token = keyring.get(&context.host)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Not authenticated for {}. Run 'bb auth login' first.",
                context.host
            )
        })?;

        let client = Client::builder()
            .user_agent(format!("bb/{}", crate::VERSION))
            .build()?;

        let output = OutputWriter::new(if global.json {
            OutputFormat::Json
        } else {
            OutputFormat::Table
        });

        if context.host_type == HostType::Cloud {
            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}",
                context.owner, context.repo_slug
            );

            let response = client
                .delete(&url)
                .bearer_auth(&token)
                .send()
                .await
                .context("Failed to connect to Bitbucket Cloud")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to delete repository ({}): {}", status, text);
            }
        } else {
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}",
                context.host, context.owner, context.repo_slug
            );

            let response = client
                .delete(&url)
                .bearer_auth(&token)
                .send()
                .await
                .context("Failed to connect to Bitbucket Server")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to delete repository ({}): {}", status, text);
            }
        }

        output.write_success(&format!("Deleted repository {}", context.full_name()));

        Ok(())
    }

    /// Archive a repository
    async fn archive(&self, global: &GlobalOptions) -> Result<()> {
        // This is Cloud-only feature currently
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config.clone());
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        if context.host_type != HostType::Cloud {
            anyhow::bail!("Archive is only available for Bitbucket Cloud repositories");
        }

        let _token = keyring
            .get(&context.host)?
            .ok_or_else(|| anyhow::anyhow!("Not authenticated. Run 'bb auth login' first."))?;

        // Archive is done via a PUT to update the repo with archived=true
        // However, Bitbucket Cloud doesn't have a direct archive API
        // We'd need to use the web UI or specific endpoints

        println!("Repository archiving is not yet supported via API.");
        println!("Please archive the repository via the Bitbucket web interface:");
        println!("  {}/admin", context.web_url());

        Ok(())
    }

    /// Unarchive a repository
    async fn unarchive(&self, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config.clone());
        let context = resolver.resolve(global)?;

        println!("Repository unarchiving is not yet supported via API.");
        println!("Please unarchive the repository via the Bitbucket web interface:");
        println!("  {}/admin", context.web_url());

        Ok(())
    }

    /// Rename a repository
    async fn rename(&self, args: &RenameArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config.clone());
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        let token = keyring
            .get(&context.host)?
            .ok_or_else(|| anyhow::anyhow!("Not authenticated. Run 'bb auth login' first."))?;

        let client = Client::builder()
            .user_agent(format!("bb/{}", crate::VERSION))
            .build()?;

        let output = OutputWriter::new(if global.json {
            OutputFormat::Json
        } else {
            OutputFormat::Table
        });

        if context.host_type == HostType::Cloud {
            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}",
                context.owner, context.repo_slug
            );

            #[derive(Serialize)]
            struct UpdateRequest {
                name: String,
            }

            let body = UpdateRequest {
                name: args.new_name.clone(),
            };

            let response = client
                .put(&url)
                .bearer_auth(&token)
                .json(&body)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to rename repository ({}): {}", status, text);
            }

            output.write_success(&format!(
                "Renamed repository to {}/{}",
                context.owner, args.new_name
            ));
        } else {
            // Server uses a different endpoint for renaming
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}",
                context.host, context.owner, context.repo_slug
            );

            #[derive(Serialize)]
            struct UpdateRequest {
                name: String,
            }

            let body = UpdateRequest {
                name: args.new_name.clone(),
            };

            let response = client
                .put(&url)
                .bearer_auth(&token)
                .json(&body)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to rename repository ({}): {}", status, text);
            }

            output.write_success(&format!(
                "Renamed repository to {}/{}",
                context.owner, args.new_name
            ));
        }

        Ok(())
    }

    /// Sync a fork with upstream
    async fn sync(&self, args: &SyncArgs, _global: &GlobalOptions) -> Result<()> {
        let branch = args.branch.as_deref().unwrap_or("main");

        println!("Syncing fork with upstream branch '{}'...", branch);

        // This involves git operations
        let mut cmd = Command::new("git");
        cmd.args(["fetch", "upstream"]);

        let status = cmd.status().context("Failed to fetch upstream")?;
        if !status.success() {
            anyhow::bail!("Failed to fetch upstream. Make sure 'upstream' remote is configured.");
        }

        // Checkout the branch
        let mut cmd = Command::new("git");
        cmd.args(["checkout", branch]);
        let _ = cmd.status();

        // Merge or rebase
        let mut cmd = Command::new("git");
        if args.force {
            cmd.args(["reset", "--hard", &format!("upstream/{}", branch)]);
        } else {
            cmd.args(["merge", &format!("upstream/{}", branch)]);
        }

        let status = cmd.status()?;
        if !status.success() {
            anyhow::bail!("Failed to sync with upstream");
        }

        println!("Successfully synced with upstream/{}", branch);

        Ok(())
    }

    /// Edit repository settings
    async fn edit(&self, args: &EditArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config.clone());
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        let token = keyring
            .get(&context.host)?
            .ok_or_else(|| anyhow::anyhow!("Not authenticated. Run 'bb auth login' first."))?;

        let client = Client::builder()
            .user_agent(format!("bb/{}", crate::VERSION))
            .build()?;

        let output = OutputWriter::new(if global.json {
            OutputFormat::Json
        } else {
            OutputFormat::Table
        });

        if context.host_type == HostType::Cloud {
            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}",
                context.owner, context.repo_slug
            );

            #[derive(Serialize, Default)]
            struct UpdateRequest {
                #[serde(skip_serializing_if = "Option::is_none")]
                description: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none")]
                is_private: Option<bool>,
                #[serde(skip_serializing_if = "Option::is_none")]
                has_issues: Option<bool>,
                #[serde(skip_serializing_if = "Option::is_none")]
                has_wiki: Option<bool>,
            }

            let mut body = UpdateRequest {
                description: args.description.clone(),
                ..Default::default()
            };

            if let Some(vis) = &args.visibility {
                body.is_private = Some(vis == "private");
            }

            if args.enable_issues {
                body.has_issues = Some(true);
            } else if args.disable_issues {
                body.has_issues = Some(false);
            }

            if args.enable_wiki {
                body.has_wiki = Some(true);
            } else if args.disable_wiki {
                body.has_wiki = Some(false);
            }

            let response = client
                .put(&url)
                .bearer_auth(&token)
                .json(&body)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to update repository ({}): {}", status, text);
            }

            output.write_success(&format!("Updated repository {}", context.full_name()));
        } else {
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}",
                context.host, context.owner, context.repo_slug
            );

            #[derive(Serialize, Default)]
            struct UpdateRequest {
                #[serde(skip_serializing_if = "Option::is_none")]
                description: Option<String>,
                #[serde(skip_serializing_if = "Option::is_none")]
                #[serde(rename = "public")]
                is_public: Option<bool>,
            }

            let mut body = UpdateRequest {
                description: args.description.clone(),
                ..Default::default()
            };

            if let Some(vis) = &args.visibility {
                body.is_public = Some(vis == "public");
            }

            let response = client
                .put(&url)
                .bearer_auth(&token)
                .json(&body)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to update repository ({}): {}", status, text);
            }

            output.write_success(&format!("Updated repository {}", context.full_name()));
        }

        Ok(())
    }

    /// Open repository in browser
    async fn browse(&self, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config.clone());
        let context = resolver.resolve(global)?;

        let url = context.web_url();
        webbrowser::open(&url)?;
        println!("Opened {} in browser", url);

        Ok(())
    }

    /// View repository contributors
    async fn credits(&self, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config.clone());
        let _keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        // Get contributors from git log locally
        println!("Contributors to {}:\n", context.full_name());

        let output = Command::new("git")
            .args(["shortlog", "-sn", "--no-merges", "HEAD"])
            .output()
            .context("Failed to get contributors from git")?;

        if output.status.success() {
            let contributors = String::from_utf8_lossy(&output.stdout);
            println!("{}", contributors);
        } else {
            println!("Could not retrieve contributors from git log.");
            println!("Make sure you're in a git repository.");
        }

        Ok(())
    }
}

// Helper method for ContextResolver
impl ContextResolver {
    /// Parse repo argument with host override
    pub fn parse_repo_arg_with_host(
        &self,
        repo: &str,
        options: &GlobalOptions,
    ) -> Result<RepoContext> {
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid repository format. Expected WORKSPACE/REPO or PROJECT/REPO");
        }

        let owner = parts[0].to_string();
        let repo_slug = parts[1].to_string();

        let host = options
            .host
            .clone()
            .unwrap_or_else(|| "bitbucket.org".to_string());
        let host_type = if is_cloud_host(&host) {
            HostType::Cloud
        } else {
            HostType::Server
        };

        Ok(RepoContext {
            host,
            host_type,
            owner,
            repo_slug,
            default_branch: None,
        })
    }
}
