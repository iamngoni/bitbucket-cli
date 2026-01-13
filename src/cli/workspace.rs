//
//  bitbucket-cli
//  cli/workspace.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Workspace commands (Cloud only)
//!
//! Workspaces are a Bitbucket Cloud concept for organizing repositories,
//! projects, and team members. This module provides commands to list,
//! view, and manage workspaces.

use anyhow::Result;
use clap::{Args, Subcommand};
use console::style;
use serde::{Deserialize, Serialize};

use crate::api::client::BitbucketClient;
use crate::api::common::PaginatedResponse;
use crate::api::cloud::workspaces::{Workspace, WorkspaceMember};
use crate::auth::{AuthCredential, KeyringStore};
use crate::config::Config;
use crate::output::{OutputFormat, OutputWriter, TableOutput};
use crate::util::open_browser;

use super::GlobalOptions;

/// Manage workspaces (Cloud only)
#[derive(Args, Debug)]
pub struct WorkspaceCommand {
    #[command(subcommand)]
    pub command: WorkspaceSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum WorkspaceSubcommand {
    /// List workspaces
    #[command(visible_alias = "ls")]
    List(ListArgs),

    /// View workspace details
    View(ViewArgs),

    /// List workspace members
    Members(MembersArgs),

    /// List projects in workspace
    Projects(ProjectsArgs),

    /// Switch default workspace
    Switch(SwitchArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Maximum number of workspaces to list
    #[arg(long, short = 'l', default_value = "25")]
    pub limit: u32,
}

#[derive(Args, Debug)]
pub struct ViewArgs {
    /// Workspace slug
    pub workspace: Option<String>,

    /// Open in browser
    #[arg(long, short = 'w')]
    pub web: bool,
}

#[derive(Args, Debug)]
pub struct MembersArgs {
    /// Workspace slug
    pub workspace: Option<String>,

    /// Filter by role
    #[arg(long, short = 'r', value_parser = ["owner", "collaborator", "member"])]
    pub role: Option<String>,

    /// Maximum number of members to list
    #[arg(long, short = 'l', default_value = "50")]
    pub limit: u32,
}

#[derive(Args, Debug)]
pub struct ProjectsArgs {
    /// Workspace slug
    pub workspace: Option<String>,

    /// Maximum number of projects to list
    #[arg(long, short = 'l', default_value = "25")]
    pub limit: u32,
}

#[derive(Args, Debug)]
pub struct SwitchArgs {
    /// Workspace to switch to
    pub workspace: String,
}

// API Response Types

/// Workspace membership with permission level
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspacePermission {
    permission: String,
    workspace: Workspace,
}

/// Project in a workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
struct WorkspaceProject {
    uuid: String,
    key: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    is_private: bool,
    #[serde(default)]
    created_on: Option<String>,
}

// Display Types

#[derive(Debug, Serialize)]
struct WorkspaceListItem {
    slug: String,
    name: String,
    permission: String,
    is_private: bool,
}

impl TableOutput for WorkspaceListItem {
    fn print_table(&self, color: bool) {
        let privacy = if self.is_private { "private" } else { "public" };
        let privacy_display = if color {
            if self.is_private {
                style(privacy).yellow().to_string()
            } else {
                style(privacy).green().to_string()
            }
        } else {
            privacy.to_string()
        };

        let permission_display = if color {
            match self.permission.to_lowercase().as_str() {
                "owner" => style(&self.permission).cyan().bold().to_string(),
                "admin" => style(&self.permission).magenta().to_string(),
                _ => self.permission.clone(),
            }
        } else {
            self.permission.clone()
        };

        println!(
            "{:<25} {:<30} {:<12} {}",
            self.slug, self.name, permission_display, privacy_display
        );
    }

    fn print_markdown(&self) {
        let privacy = if self.is_private { "Private" } else { "Public" };
        println!(
            "| {} | {} | {} | {} |",
            self.slug, self.name, self.permission, privacy
        );
    }
}

#[derive(Debug, Serialize)]
struct WorkspaceDetail {
    uuid: String,
    slug: String,
    name: String,
    is_private: bool,
    created_on: String,
}

impl TableOutput for WorkspaceDetail {
    fn print_table(&self, color: bool) {
        let title = if color {
            style(&self.name).bold().to_string()
        } else {
            self.name.clone()
        };

        let privacy = if self.is_private { "Private" } else { "Public" };
        let privacy_display = if color {
            if self.is_private {
                style(privacy).yellow().to_string()
            } else {
                style(privacy).green().to_string()
            }
        } else {
            privacy.to_string()
        };

        println!("{}", title);
        println!();
        println!("  Slug:       {}", self.slug);
        println!("  UUID:       {}", self.uuid);
        println!("  Visibility: {}", privacy_display);
        println!("  Created:    {}", self.created_on);
        println!();
        println!("  URL: https://bitbucket.org/{}/", self.slug);
    }

    fn print_markdown(&self) {
        let privacy = if self.is_private { "Private" } else { "Public" };
        println!("# {}", self.name);
        println!();
        println!("| Property | Value |");
        println!("|----------|-------|");
        println!("| Slug | {} |", self.slug);
        println!("| UUID | {} |", self.uuid);
        println!("| Visibility | {} |", privacy);
        println!("| Created | {} |", self.created_on);
    }
}

#[derive(Debug, Serialize)]
struct MemberListItem {
    display_name: String,
    nickname: Option<String>,
    account_id: Option<String>,
    uuid: String,
}

impl TableOutput for MemberListItem {
    fn print_table(&self, color: bool) {
        let name = if color {
            style(&self.display_name).bold().to_string()
        } else {
            self.display_name.clone()
        };

        let nickname = self.nickname.as_deref().unwrap_or("-");

        println!(
            "{:<30} {:<20} {}",
            name, nickname, self.uuid
        );
    }

    fn print_markdown(&self) {
        let nickname = self.nickname.as_deref().unwrap_or("-");
        println!(
            "| {} | {} | {} |",
            self.display_name, nickname, self.uuid
        );
    }
}

#[derive(Debug, Serialize)]
struct ProjectListItem {
    key: String,
    name: String,
    description: Option<String>,
    is_private: bool,
}

impl TableOutput for ProjectListItem {
    fn print_table(&self, color: bool) {
        let key_display = if color {
            style(&self.key).cyan().bold().to_string()
        } else {
            self.key.clone()
        };

        let privacy = if self.is_private { "private" } else { "public" };
        let privacy_display = if color {
            if self.is_private {
                style(privacy).yellow().to_string()
            } else {
                style(privacy).green().to_string()
            }
        } else {
            privacy.to_string()
        };

        let description = self.description.as_deref().unwrap_or("-");
        let desc_truncated = if description.len() > 40 {
            format!("{}...", &description[..37])
        } else {
            description.to_string()
        };

        println!(
            "{:<10} {:<25} {:<10} {}",
            key_display, self.name, privacy_display, desc_truncated
        );
    }

    fn print_markdown(&self) {
        let privacy = if self.is_private { "Private" } else { "Public" };
        let description = self.description.as_deref().unwrap_or("-");
        println!(
            "| {} | {} | {} | {} |",
            self.key, self.name, privacy, description
        );
    }
}

impl WorkspaceCommand {
    pub async fn run(&self, global: &GlobalOptions) -> Result<()> {
        match &self.command {
            WorkspaceSubcommand::List(args) => self.list(args, global).await,
            WorkspaceSubcommand::View(args) => self.view(args, global).await,
            WorkspaceSubcommand::Members(args) => self.members(args, global).await,
            WorkspaceSubcommand::Projects(args) => self.projects(args, global).await,
            WorkspaceSubcommand::Switch(args) => self.switch(args, global).await,
        }
    }

    fn get_format(&self, global: &GlobalOptions) -> OutputFormat {
        if global.json {
            OutputFormat::Json
        } else {
            OutputFormat::Table
        }
    }

    fn get_client(&self) -> Result<BitbucketClient> {
        // Workspaces are Cloud-only
        let keyring = KeyringStore::new();
        let host = "bitbucket.org";

        let token = keyring.get(host)?
            .ok_or_else(|| anyhow::anyhow!(
                "Not authenticated with Bitbucket Cloud. Run 'bb auth login --cloud' first."
            ))?;

        let client = BitbucketClient::cloud()?
            .with_auth(AuthCredential::PersonalAccessToken { token });

        Ok(client)
    }

    /// List workspaces the user has access to
    async fn list(&self, args: &ListArgs, global: &GlobalOptions) -> Result<()> {
        let client = self.get_client()?;

        // Get workspaces the user has permission to access
        let url = format!(
            "/user/permissions/workspaces?pagelen={}",
            args.limit
        );

        let response: PaginatedResponse<WorkspacePermission> = client.get(&url).await?;

        let items: Vec<WorkspaceListItem> = response.values.into_iter().map(|wp| {
            WorkspaceListItem {
                slug: wp.workspace.slug,
                name: wp.workspace.name,
                permission: wp.permission,
                is_private: wp.workspace.is_private,
            }
        }).collect();

        if items.is_empty() {
            println!("No workspaces found.");
            return Ok(());
        }

        let writer = OutputWriter::new(self.get_format(global));

        if !global.json {
            println!();
            println!(
                "{} {} {} {}",
                style(format!("{:<25}", "SLUG")).bold(),
                style(format!("{:<30}", "NAME")).bold(),
                style(format!("{:<12}", "PERMISSION")).bold(),
                style("VISIBILITY").bold()
            );
            println!("{}", "-".repeat(80));
        }

        writer.write_list(&items)?;

        if !global.json {
            println!();
            println!("Showing {} workspace(s)", items.len());
        }

        Ok(())
    }

    /// View workspace details
    async fn view(&self, args: &ViewArgs, global: &GlobalOptions) -> Result<()> {
        let workspace_slug = args.workspace.as_ref()
            .or(global.workspace.as_ref())
            .ok_or_else(|| anyhow::anyhow!(
                "Workspace required. Specify with argument or use --workspace flag."
            ))?;

        if args.web {
            let url = format!("https://bitbucket.org/{}/", workspace_slug);
            println!("Opening {} in browser...", url);
            open_browser(&url)?;
            return Ok(());
        }

        let client = self.get_client()?;
        let url = format!("/workspaces/{}", workspace_slug);

        let workspace: Workspace = client.get(&url).await?;

        let detail = WorkspaceDetail {
            uuid: workspace.uuid,
            slug: workspace.slug,
            name: workspace.name,
            is_private: workspace.is_private,
            created_on: workspace.created_on,
        };

        let writer = OutputWriter::new(self.get_format(global));
        writer.write(&detail)?;

        Ok(())
    }

    /// List workspace members
    async fn members(&self, args: &MembersArgs, global: &GlobalOptions) -> Result<()> {
        let workspace_slug = args.workspace.as_ref()
            .or(global.workspace.as_ref())
            .ok_or_else(|| anyhow::anyhow!(
                "Workspace required. Specify with argument or use --workspace flag."
            ))?;

        let client = self.get_client()?;

        let mut url = format!(
            "/workspaces/{}/members?pagelen={}",
            workspace_slug, args.limit
        );

        // Role filter if specified
        if let Some(role) = &args.role {
            url = format!("{}&q=permission=\"{}\"", url, role);
        }

        let response: PaginatedResponse<WorkspaceMember> = client.get(&url).await?;

        let items: Vec<MemberListItem> = response.values.into_iter().map(|m| {
            MemberListItem {
                display_name: m.user.display_name,
                nickname: m.user.nickname,
                account_id: m.user.account_id,
                uuid: m.user.uuid,
            }
        }).collect();

        if items.is_empty() {
            println!("No members found.");
            return Ok(());
        }

        let writer = OutputWriter::new(self.get_format(global));

        if !global.json {
            println!();
            println!("Members of workspace '{}':", workspace_slug);
            println!();
            println!(
                "{} {} {}",
                style(format!("{:<30}", "NAME")).bold(),
                style(format!("{:<20}", "NICKNAME")).bold(),
                style("UUID").bold()
            );
            println!("{}", "-".repeat(70));
        }

        writer.write_list(&items)?;

        if !global.json {
            println!();
            println!("Showing {} member(s)", items.len());
        }

        Ok(())
    }

    /// List projects in workspace
    async fn projects(&self, args: &ProjectsArgs, global: &GlobalOptions) -> Result<()> {
        let workspace_slug = args.workspace.as_ref()
            .or(global.workspace.as_ref())
            .ok_or_else(|| anyhow::anyhow!(
                "Workspace required. Specify with argument or use --workspace flag."
            ))?;

        let client = self.get_client()?;

        let url = format!(
            "/workspaces/{}/projects?pagelen={}",
            workspace_slug, args.limit
        );

        let response: PaginatedResponse<WorkspaceProject> = client.get(&url).await?;

        let items: Vec<ProjectListItem> = response.values.into_iter().map(|p| {
            ProjectListItem {
                key: p.key,
                name: p.name,
                description: p.description,
                is_private: p.is_private,
            }
        }).collect();

        if items.is_empty() {
            println!("No projects found in workspace '{}'.", workspace_slug);
            return Ok(());
        }

        let writer = OutputWriter::new(self.get_format(global));

        if !global.json {
            println!();
            println!("Projects in workspace '{}':", workspace_slug);
            println!();
            println!(
                "{} {} {} {}",
                style(format!("{:<10}", "KEY")).bold(),
                style(format!("{:<25}", "NAME")).bold(),
                style(format!("{:<10}", "VISIBILITY")).bold(),
                style("DESCRIPTION").bold()
            );
            println!("{}", "-".repeat(80));
        }

        writer.write_list(&items)?;

        if !global.json {
            println!();
            println!("Showing {} project(s)", items.len());
        }

        Ok(())
    }

    /// Switch default workspace
    async fn switch(&self, args: &SwitchArgs, global: &GlobalOptions) -> Result<()> {
        // First verify the workspace exists and we have access
        let client = self.get_client()?;
        let url = format!("/workspaces/{}", args.workspace);

        let workspace: Workspace = client.get(&url).await
            .map_err(|_| anyhow::anyhow!(
                "Workspace '{}' not found or you don't have access to it.",
                args.workspace
            ))?;

        // Update config
        let mut config = Config::load()?;

        let host = "bitbucket.org";
        let host_config = config.hosts.entry(host.to_string()).or_default();
        host_config.host = host.to_string();
        host_config.default_workspace = Some(workspace.slug.clone());

        config.save()?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "workspace": workspace.slug,
                "name": workspace.name,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!();
            println!(
                "{} Switched default workspace",
                style("✓").green()
            );
            println!();
            println!("  Workspace: {} ({})", workspace.name, workspace.slug);
            println!();
            println!("Future commands will use '{}' as the default workspace.", workspace.slug);
        }

        Ok(())
    }
}
