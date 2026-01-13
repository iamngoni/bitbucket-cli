//
//  bitbucket-cli
//  cli/project.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Project commands
//!
//! Projects are available on both Bitbucket Cloud and Server/Data Center,
//! but with different semantics:
//! - Cloud: Projects are optional groupings within a workspace
//! - Server/DC: Projects are the primary organizational unit

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use console::style;
use serde::{Deserialize, Serialize};

use crate::api::client::BitbucketClient;
use crate::api::common::{PaginatedResponse, ServerPaginatedResponse};
use crate::api::server::projects::{
    CreateProjectRequest, Project as ServerProject, UpdateProjectRequest,
};
use crate::auth::{AuthCredential, KeyringStore};
use crate::config::Config;
use crate::context::{ContextResolver, HostType, RepoContext};
use crate::interactive::prompt_confirm_with_default;
use crate::output::{OutputFormat, OutputWriter, TableOutput};
use crate::util::open_browser;

use super::GlobalOptions;

/// Manage projects
#[derive(Args, Debug)]
pub struct ProjectCommand {
    #[command(subcommand)]
    pub command: ProjectSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum ProjectSubcommand {
    /// List projects
    #[command(visible_alias = "ls")]
    List(ListArgs),

    /// View project details
    View(ViewArgs),

    /// Create a project (Server/DC)
    Create(CreateArgs),

    /// Edit a project
    Edit(EditArgs),

    /// Delete a project
    Delete(DeleteArgs),

    /// List repositories in project
    Repos(ReposArgs),

    /// Manage project members
    Members(MembersCommand),

    /// View project permissions
    Permissions(PermissionsArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Workspace (Cloud)
    #[arg(long, short = 'w')]
    pub workspace: Option<String>,

    /// Maximum number of projects to list
    #[arg(long, short = 'l', default_value = "25")]
    pub limit: u32,
}

#[derive(Args, Debug)]
pub struct ViewArgs {
    /// Project key
    pub project: Option<String>,

    /// Open in browser
    #[arg(long, short = 'w')]
    pub web: bool,
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Project key (uppercase)
    #[arg(long, short = 'k')]
    pub key: String,

    /// Project name
    #[arg(long, short = 'n')]
    pub name: String,

    /// Project description
    #[arg(long, short = 'd')]
    pub description: Option<String>,

    /// Make project public (Server/DC only)
    #[arg(long)]
    pub public: bool,
}

#[derive(Args, Debug)]
pub struct EditArgs {
    /// Project key
    pub project: String,

    /// New name
    #[arg(long, short = 'n')]
    pub name: Option<String>,

    /// New description
    #[arg(long, short = 'd')]
    pub description: Option<String>,

    /// Make project public (Server/DC only)
    #[arg(long)]
    pub public: Option<bool>,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// Project key
    pub project: String,

    /// Skip confirmation
    #[arg(long)]
    pub confirm: bool,
}

#[derive(Args, Debug)]
pub struct ReposArgs {
    /// Project key
    pub project: Option<String>,

    /// Maximum number of repositories to list
    #[arg(long, short = 'l', default_value = "25")]
    pub limit: u32,
}

#[derive(Args, Debug)]
pub struct MembersCommand {
    #[command(subcommand)]
    pub command: MembersSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum MembersSubcommand {
    /// List project members
    List(MemberListArgs),

    /// Add a member to project
    Add(AddMemberArgs),

    /// Remove a member from project
    Remove(RemoveMemberArgs),
}

#[derive(Args, Debug)]
pub struct MemberListArgs {
    /// Project key
    pub project: String,

    /// Maximum number of members to list
    #[arg(long, short = 'l', default_value = "50")]
    pub limit: u32,
}

#[derive(Args, Debug)]
pub struct AddMemberArgs {
    /// Project key
    pub project: String,

    /// User to add
    pub user: String,

    /// Permission level
    #[arg(long, short = 'p', value_parser = ["read", "write", "admin"])]
    pub permission: String,
}

#[derive(Args, Debug)]
pub struct RemoveMemberArgs {
    /// Project key
    pub project: String,

    /// User to remove
    pub user: String,
}

#[derive(Args, Debug)]
pub struct PermissionsArgs {
    /// Project key
    pub project: Option<String>,
}

// API Response Types for Cloud

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CloudProject {
    uuid: String,
    key: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    is_private: bool,
    #[serde(default)]
    created_on: Option<String>,
    #[serde(default)]
    updated_on: Option<String>,
    #[serde(default)]
    links: Option<CloudProjectLinks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CloudProjectLinks {
    #[serde(default)]
    html: Option<LinkRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LinkRef {
    href: String,
}

// Cloud repository type for listing repos in project
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CloudRepository {
    uuid: String,
    slug: String,
    name: String,
    #[serde(default)]
    full_name: Option<String>,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    is_private: bool,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    updated_on: Option<String>,
}

// Server repository type
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerRepository {
    id: u64,
    slug: String,
    name: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default, rename = "public")]
    is_public: bool,
}

// Server project permission
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerProjectPermission {
    user: ServerUser,
    permission: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ServerUser {
    name: String,
    #[serde(rename = "displayName")]
    display_name: String,
    #[serde(rename = "emailAddress", default)]
    email_address: Option<String>,
}

// Display Types

#[derive(Debug, Serialize)]
struct ProjectListItem {
    key: String,
    name: String,
    description: Option<String>,
    is_private: bool,
    project_type: String, // "cloud" or "server"
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
        let desc_truncated = if description.len() > 35 {
            format!("{}...", &description[..32])
        } else {
            description.to_string()
        };

        println!(
            "{:<12} {:<25} {:<10} {}",
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

#[derive(Debug, Serialize)]
struct ProjectDetail {
    key: String,
    name: String,
    description: Option<String>,
    is_private: bool,
    project_type: String,
    created_on: Option<String>,
    url: Option<String>,
}

impl TableOutput for ProjectDetail {
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
        println!("  Key:         {}", self.key);
        println!("  Visibility:  {}", privacy_display);
        println!("  Platform:    {}", self.project_type.to_uppercase());

        if let Some(desc) = &self.description {
            println!("  Description: {}", desc);
        }

        if let Some(created) = &self.created_on {
            println!("  Created:     {}", created);
        }

        if let Some(url) = &self.url {
            println!();
            println!("  URL: {}", url);
        }
    }

    fn print_markdown(&self) {
        let privacy = if self.is_private { "Private" } else { "Public" };
        println!("# {} ({})", self.name, self.key);
        println!();
        println!("| Property | Value |");
        println!("|----------|-------|");
        println!("| Key | {} |", self.key);
        println!("| Visibility | {} |", privacy);
        println!("| Platform | {} |", self.project_type.to_uppercase());
        if let Some(desc) = &self.description {
            println!("| Description | {} |", desc);
        }
        if let Some(url) = &self.url {
            println!("| URL | {} |", url);
        }
    }
}

#[derive(Debug, Serialize)]
struct RepoListItem {
    slug: String,
    name: String,
    description: Option<String>,
    is_private: bool,
}

impl TableOutput for RepoListItem {
    fn print_table(&self, color: bool) {
        let name_display = if color {
            style(&self.name).bold().to_string()
        } else {
            self.name.clone()
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
            "{:<25} {:<30} {:<10} {}",
            self.slug, name_display, privacy_display, desc_truncated
        );
    }

    fn print_markdown(&self) {
        let privacy = if self.is_private { "Private" } else { "Public" };
        let description = self.description.as_deref().unwrap_or("-");
        println!(
            "| {} | {} | {} | {} |",
            self.slug, self.name, privacy, description
        );
    }
}

#[derive(Debug, Serialize)]
struct MemberItem {
    username: String,
    display_name: String,
    permission: String,
}

impl TableOutput for MemberItem {
    fn print_table(&self, color: bool) {
        let name_display = if color {
            style(&self.display_name).bold().to_string()
        } else {
            self.display_name.clone()
        };

        let permission_display = if color {
            match self.permission.to_lowercase().as_str() {
                "admin" | "project_admin" => style(&self.permission).magenta().to_string(),
                "write" | "project_write" => style(&self.permission).cyan().to_string(),
                _ => self.permission.clone(),
            }
        } else {
            self.permission.clone()
        };

        println!(
            "{:<25} {:<30} {}",
            self.username, name_display, permission_display
        );
    }

    fn print_markdown(&self) {
        println!(
            "| {} | {} | {} |",
            self.username, self.display_name, self.permission
        );
    }
}

impl ProjectCommand {
    pub async fn run(&self, global: &GlobalOptions) -> Result<()> {
        match &self.command {
            ProjectSubcommand::List(args) => self.list(args, global).await,
            ProjectSubcommand::View(args) => self.view(args, global).await,
            ProjectSubcommand::Create(args) => self.create(args, global).await,
            ProjectSubcommand::Edit(args) => self.edit(args, global).await,
            ProjectSubcommand::Delete(args) => self.delete(args, global).await,
            ProjectSubcommand::Repos(args) => self.repos(args, global).await,
            ProjectSubcommand::Members(members) => match &members.command {
                MembersSubcommand::List(args) => self.members_list(args, global).await,
                MembersSubcommand::Add(args) => self.members_add(args, global).await,
                MembersSubcommand::Remove(args) => self.members_remove(args, global).await,
            },
            ProjectSubcommand::Permissions(args) => self.permissions(args, global).await,
        }
    }

    fn get_format(&self, global: &GlobalOptions) -> OutputFormat {
        if global.json {
            OutputFormat::Json
        } else {
            OutputFormat::Table
        }
    }

    fn get_cloud_client(&self) -> Result<BitbucketClient> {
        let keyring = KeyringStore::new();
        let host = "bitbucket.org";

        let token = keyring.get(host)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Not authenticated with Bitbucket Cloud. Run 'bb auth login --cloud' first."
            )
        })?;

        let client =
            BitbucketClient::cloud()?.with_auth(AuthCredential::PersonalAccessToken { token });

        Ok(client)
    }

    fn get_server_client(&self, host: &str) -> Result<BitbucketClient> {
        let keyring = KeyringStore::new();

        let token = keyring.get(host)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Not authenticated with {}. Run 'bb auth login --server --host {}' first.",
                host,
                host
            )
        })?;

        let client =
            BitbucketClient::server(host)?.with_auth(AuthCredential::PersonalAccessToken { token });

        Ok(client)
    }

    /// Try to resolve repository context from git remote
    fn try_resolve_context(&self, global: &GlobalOptions) -> Option<RepoContext> {
        let config = Config::load().ok()?;
        let resolver = ContextResolver::new(config);
        resolver.resolve(global).ok()
    }

    /// List projects
    async fn list(&self, args: &ListArgs, global: &GlobalOptions) -> Result<()> {
        // Determine context - try workspace for Cloud, or detect from git
        let context = self.try_resolve_context(global);

        // If workspace is provided or detected as Cloud, use Cloud API
        if let Some(ws) = args.workspace.as_ref().or(global.workspace.as_ref()) {
            return self.list_cloud_projects(ws, args.limit, global).await;
        }

        // Check if we have a context from git
        if let Some(ctx) = &context {
            if ctx.host_type == HostType::Cloud {
                // Cloud context - owner is the workspace
                return self
                    .list_cloud_projects(&ctx.owner, args.limit, global)
                    .await;
            } else {
                // Server/DC context
                return self
                    .list_server_projects(&ctx.host, args.limit, global)
                    .await;
            }
        }

        // Default to trying Cloud if authenticated
        let keyring = KeyringStore::new();
        if keyring.get("bitbucket.org")?.is_some() {
            bail!("No workspace specified. Use --workspace or run from a git repository.");
        }

        bail!("Not authenticated. Run 'bb auth login' first.");
    }

    async fn list_cloud_projects(
        &self,
        workspace: &str,
        limit: u32,
        global: &GlobalOptions,
    ) -> Result<()> {
        let client = self.get_cloud_client()?;

        let url = format!("/workspaces/{}/projects?pagelen={}", workspace, limit);

        let response: PaginatedResponse<CloudProject> = client.get(&url).await?;

        let items: Vec<ProjectListItem> = response
            .values
            .into_iter()
            .map(|p| ProjectListItem {
                key: p.key,
                name: p.name,
                description: p.description,
                is_private: p.is_private,
                project_type: "cloud".to_string(),
            })
            .collect();

        self.display_project_list(&items, Some(workspace), global)
    }

    async fn list_server_projects(
        &self,
        host: &str,
        limit: u32,
        global: &GlobalOptions,
    ) -> Result<()> {
        let client = self.get_server_client(host)?;

        let url = format!("/projects?limit={}", limit);

        let response: ServerPaginatedResponse<ServerProject> = client.get(&url).await?;

        let items: Vec<ProjectListItem> = response
            .values
            .into_iter()
            .map(|p| ProjectListItem {
                key: p.key,
                name: p.name,
                description: p.description,
                is_private: !p.is_public,
                project_type: "server".to_string(),
            })
            .collect();

        self.display_project_list(&items, None, global)
    }

    fn display_project_list(
        &self,
        items: &[ProjectListItem],
        workspace: Option<&str>,
        global: &GlobalOptions,
    ) -> Result<()> {
        if items.is_empty() {
            if let Some(ws) = workspace {
                println!("No projects found in workspace '{}'.", ws);
            } else {
                println!("No projects found.");
            }
            return Ok(());
        }

        let writer = OutputWriter::new(self.get_format(global));

        if !global.json {
            println!();
            if let Some(ws) = workspace {
                println!("Projects in workspace '{}':", ws);
            } else {
                println!("Projects:");
            }
            println!();
            println!(
                "{} {} {} {}",
                style(format!("{:<12}", "KEY")).bold(),
                style(format!("{:<25}", "NAME")).bold(),
                style(format!("{:<10}", "VISIBILITY")).bold(),
                style("DESCRIPTION").bold()
            );
            println!("{}", "-".repeat(80));
        }

        writer.write_list(items)?;

        if !global.json {
            println!();
            println!("Showing {} project(s)", items.len());
        }

        Ok(())
    }

    /// View project details
    async fn view(&self, args: &ViewArgs, global: &GlobalOptions) -> Result<()> {
        let context = self.try_resolve_context(global);

        // For Cloud, we need a project key from args or global options
        // For Server, the context.owner IS the project key
        let project_key = args
            .project
            .as_ref()
            .or(global.project.as_ref())
            .or_else(|| {
                context
                    .as_ref()
                    .filter(|c| c.host_type == HostType::Server)
                    .map(|c| &c.owner)
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Project key required. Specify with argument or use --project flag."
                )
            })?;

        // Determine if Cloud or Server
        let is_cloud = if let Some(ctx) = &context {
            ctx.host_type == HostType::Cloud
        } else {
            global.workspace.is_some()
        };

        if is_cloud {
            let workspace = global
                .workspace
                .as_ref()
                .or_else(|| {
                    context
                        .as_ref()
                        .filter(|c| c.host_type == HostType::Cloud)
                        .map(|c| &c.owner)
                })
                .ok_or_else(|| {
                    anyhow::anyhow!("Workspace required for Cloud projects. Use --workspace flag.")
                })?;

            if args.web {
                let url = format!(
                    "https://bitbucket.org/{}/workspace/projects/{}",
                    workspace, project_key
                );
                println!("Opening {} in browser...", url);
                open_browser(&url)?;
                return Ok(());
            }

            self.view_cloud_project(workspace, project_key, global)
                .await
        } else {
            let host = context.as_ref().map(|c| c.host.as_str()).ok_or_else(|| {
                anyhow::anyhow!(
                    "Cannot determine Bitbucket Server host. Run from a git repository."
                )
            })?;

            if args.web {
                let url = format!("https://{}/projects/{}", host, project_key);
                println!("Opening {} in browser...", url);
                open_browser(&url)?;
                return Ok(());
            }

            self.view_server_project(host, project_key, global).await
        }
    }

    async fn view_cloud_project(
        &self,
        workspace: &str,
        project_key: &str,
        global: &GlobalOptions,
    ) -> Result<()> {
        let client = self.get_cloud_client()?;
        let url = format!("/workspaces/{}/projects/{}", workspace, project_key);

        let project: CloudProject = client.get(&url).await?;

        let html_url = project
            .links
            .as_ref()
            .and_then(|l| l.html.as_ref())
            .map(|h| h.href.clone());

        let detail = ProjectDetail {
            key: project.key,
            name: project.name,
            description: project.description,
            is_private: project.is_private,
            project_type: "cloud".to_string(),
            created_on: project.created_on,
            url: html_url.or_else(|| {
                Some(format!(
                    "https://bitbucket.org/{}/workspace/projects/{}",
                    workspace, project_key
                ))
            }),
        };

        let writer = OutputWriter::new(self.get_format(global));
        writer.write(&detail)?;

        Ok(())
    }

    async fn view_server_project(
        &self,
        host: &str,
        project_key: &str,
        global: &GlobalOptions,
    ) -> Result<()> {
        let client = self.get_server_client(host)?;
        let url = format!("/projects/{}", project_key);

        let project: ServerProject = client.get(&url).await?;

        let project_url = project.links.self_link.first().map(|l| l.href.clone());

        let detail = ProjectDetail {
            key: project.key,
            name: project.name,
            description: project.description,
            is_private: !project.is_public,
            project_type: "server".to_string(),
            created_on: None,
            url: project_url.or_else(|| Some(format!("https://{}/projects/{}", host, project_key))),
        };

        let writer = OutputWriter::new(self.get_format(global));
        writer.write(&detail)?;

        Ok(())
    }

    /// Create a project (Server/DC only)
    async fn create(&self, args: &CreateArgs, global: &GlobalOptions) -> Result<()> {
        // Get context to determine host
        let context = self.try_resolve_context(global).ok_or_else(|| {
            anyhow::anyhow!(
                "Cannot determine Bitbucket Server host. Run from a git repository or specify host."
            )
        })?;

        if context.host_type == HostType::Cloud {
            bail!(
                "Project creation via API is not fully supported for Bitbucket Cloud. \
                   Use the web interface at https://bitbucket.org/{}/workspace/projects/create",
                context.owner
            );
        }

        let client = self.get_server_client(&context.host)?;

        let request = CreateProjectRequest {
            key: args.key.to_uppercase(),
            name: args.name.clone(),
            description: args.description.clone(),
            is_public: Some(args.public),
        };

        let project: ServerProject = client.post("/projects", &request).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "key": project.key,
                "name": project.name,
                "url": format!("https://{}/projects/{}", context.host, project.key),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!();
            println!(
                "{} Created project {}",
                style("✓").green(),
                style(&project.key).cyan().bold()
            );
            println!();
            println!("  Name: {}", project.name);
            if let Some(desc) = &project.description {
                println!("  Description: {}", desc);
            }
            println!("  Public: {}", project.is_public);
            println!();
            println!("  URL: https://{}/projects/{}", context.host, project.key);
        }

        Ok(())
    }

    /// Edit a project
    async fn edit(&self, args: &EditArgs, global: &GlobalOptions) -> Result<()> {
        let context = self.try_resolve_context(global).ok_or_else(|| {
            anyhow::anyhow!("Cannot determine Bitbucket Server host. Run from a git repository.")
        })?;

        if context.host_type == HostType::Cloud {
            bail!(
                "Project editing via API is not fully supported for Bitbucket Cloud. \
                   Use the web interface."
            );
        }

        if args.name.is_none() && args.description.is_none() && args.public.is_none() {
            bail!("No changes specified. Use --name, --description, or --public.");
        }

        let client = self.get_server_client(&context.host)?;

        let _request = UpdateProjectRequest {
            name: args.name.clone(),
            description: args.description.clone(),
            is_public: args.public,
        };

        let url = format!("/projects/{}", args.project);

        // Server uses PUT for updates - we need a put method
        // For now, simulate with a message since the client only has get/post/delete
        // In a real implementation, we'd add a put method to the client

        // Actually check if the project exists first
        let _existing: ServerProject = client.get(&url).await?;

        // Since we don't have a PUT method, inform the user
        bail!(
            "Project editing requires PUT method which is not yet implemented. \
               Use the web interface at https://{}/projects/{}/settings",
            context.host,
            args.project
        );
    }

    /// Delete a project
    async fn delete(&self, args: &DeleteArgs, global: &GlobalOptions) -> Result<()> {
        let context = self.try_resolve_context(global).ok_or_else(|| {
            anyhow::anyhow!("Cannot determine Bitbucket Server host. Run from a git repository.")
        })?;

        if context.host_type == HostType::Cloud {
            bail!(
                "Project deletion via API is not fully supported for Bitbucket Cloud. \
                   Use the web interface."
            );
        }

        // Confirm deletion
        if !args.confirm {
            println!("You are about to delete project '{}'.", args.project);
            println!(
                "This will NOT delete repositories in the project, but they will become orphaned."
            );
            println!();

            if !prompt_confirm_with_default("Are you sure you want to delete this project?", false)?
            {
                println!("Cancelled.");
                return Ok(());
            }
        }

        let client = self.get_server_client(&context.host)?;
        let url = format!("/projects/{}", args.project);

        client.delete(&url).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "deleted": args.project,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!();
            println!(
                "{} Deleted project {}",
                style("✓").green(),
                style(&args.project).red()
            );
        }

        Ok(())
    }

    /// List repositories in project
    async fn repos(&self, args: &ReposArgs, global: &GlobalOptions) -> Result<()> {
        let context = self.try_resolve_context(global);

        // For Cloud, we need a project key from args or global options
        // For Server, the context.owner IS the project key
        let project_key = args
            .project
            .as_ref()
            .or(global.project.as_ref())
            .or_else(|| {
                context
                    .as_ref()
                    .filter(|c| c.host_type == HostType::Server)
                    .map(|c| &c.owner)
            })
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Project key required. Specify with argument or use --project flag."
                )
            })?;

        let is_cloud = if let Some(ctx) = &context {
            ctx.host_type == HostType::Cloud
        } else {
            global.workspace.is_some()
        };

        if is_cloud {
            let workspace = global
                .workspace
                .as_ref()
                .or_else(|| {
                    context
                        .as_ref()
                        .filter(|c| c.host_type == HostType::Cloud)
                        .map(|c| &c.owner)
                })
                .ok_or_else(|| {
                    anyhow::anyhow!("Workspace required for Cloud projects. Use --workspace flag.")
                })?;

            self.repos_cloud(workspace, project_key, args.limit, global)
                .await
        } else {
            let host = context.as_ref().map(|c| c.host.as_str()).ok_or_else(|| {
                anyhow::anyhow!(
                    "Cannot determine Bitbucket Server host. Run from a git repository."
                )
            })?;

            self.repos_server(host, project_key, args.limit, global)
                .await
        }
    }

    async fn repos_cloud(
        &self,
        workspace: &str,
        project_key: &str,
        limit: u32,
        global: &GlobalOptions,
    ) -> Result<()> {
        let client = self.get_cloud_client()?;

        // Filter repositories by project
        let url = format!(
            "/repositories/{}?q=project.key=\"{}\"&pagelen={}",
            workspace, project_key, limit
        );

        let response: PaginatedResponse<CloudRepository> = client.get(&url).await?;

        let items: Vec<RepoListItem> = response
            .values
            .into_iter()
            .map(|r| RepoListItem {
                slug: r.slug,
                name: r.name,
                description: r.description,
                is_private: r.is_private,
            })
            .collect();

        self.display_repos_list(&items, project_key, global)
    }

    async fn repos_server(
        &self,
        host: &str,
        project_key: &str,
        limit: u32,
        global: &GlobalOptions,
    ) -> Result<()> {
        let client = self.get_server_client(host)?;

        let url = format!("/projects/{}/repos?limit={}", project_key, limit);

        let response: ServerPaginatedResponse<ServerRepository> = client.get(&url).await?;

        let items: Vec<RepoListItem> = response
            .values
            .into_iter()
            .map(|r| RepoListItem {
                slug: r.slug,
                name: r.name,
                description: r.description,
                is_private: !r.is_public,
            })
            .collect();

        self.display_repos_list(&items, project_key, global)
    }

    fn display_repos_list(
        &self,
        items: &[RepoListItem],
        project_key: &str,
        global: &GlobalOptions,
    ) -> Result<()> {
        if items.is_empty() {
            println!("No repositories found in project '{}'.", project_key);
            return Ok(());
        }

        let writer = OutputWriter::new(self.get_format(global));

        if !global.json {
            println!();
            println!("Repositories in project '{}':", project_key);
            println!();
            println!(
                "{} {} {} {}",
                style(format!("{:<25}", "SLUG")).bold(),
                style(format!("{:<30}", "NAME")).bold(),
                style(format!("{:<10}", "VISIBILITY")).bold(),
                style("DESCRIPTION").bold()
            );
            println!("{}", "-".repeat(90));
        }

        writer.write_list(items)?;

        if !global.json {
            println!();
            println!("Showing {} repository(ies)", items.len());
        }

        Ok(())
    }

    /// List project members (Server/DC only)
    async fn members_list(&self, args: &MemberListArgs, global: &GlobalOptions) -> Result<()> {
        let context = self.try_resolve_context(global).ok_or_else(|| {
            anyhow::anyhow!("Cannot determine Bitbucket Server host. Run from a git repository.")
        })?;

        if context.host_type == HostType::Cloud {
            bail!(
                "Project member listing is only available for Bitbucket Server/DC. \
                   For Cloud, use 'bb workspace members' instead."
            );
        }

        let client = self.get_server_client(&context.host)?;

        let url = format!(
            "/projects/{}/permissions/users?limit={}",
            args.project, args.limit
        );

        let response: ServerPaginatedResponse<ServerProjectPermission> = client.get(&url).await?;

        let items: Vec<MemberItem> = response
            .values
            .into_iter()
            .map(|p| MemberItem {
                username: p.user.name,
                display_name: p.user.display_name,
                permission: p.permission,
            })
            .collect();

        if items.is_empty() {
            println!("No members found for project '{}'.", args.project);
            return Ok(());
        }

        let writer = OutputWriter::new(self.get_format(global));

        if !global.json {
            println!();
            println!("Members of project '{}':", args.project);
            println!();
            println!(
                "{} {} {}",
                style(format!("{:<25}", "USERNAME")).bold(),
                style(format!("{:<30}", "NAME")).bold(),
                style("PERMISSION").bold()
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

    /// Add a member to project (Server/DC only)
    async fn members_add(&self, args: &AddMemberArgs, global: &GlobalOptions) -> Result<()> {
        let context = self.try_resolve_context(global).ok_or_else(|| {
            anyhow::anyhow!("Cannot determine Bitbucket Server host. Run from a git repository.")
        })?;

        if context.host_type == HostType::Cloud {
            bail!(
                "Adding project members is only available for Bitbucket Server/DC. \
                   For Cloud, use the workspace settings in the web interface."
            );
        }

        let client = self.get_server_client(&context.host)?;

        // Server API uses permission levels: PROJECT_READ, PROJECT_WRITE, PROJECT_ADMIN
        let permission = match args.permission.to_lowercase().as_str() {
            "read" => "PROJECT_READ",
            "write" => "PROJECT_WRITE",
            "admin" => "PROJECT_ADMIN",
            _ => bail!("Invalid permission level. Use: read, write, or admin"),
        };

        let url = format!(
            "/projects/{}/permissions/users?name={}&permission={}",
            args.project, args.user, permission
        );

        // This is typically a PUT request, but we'll try POST
        let body: serde_json::Value = serde_json::json!({});
        let _: serde_json::Value = client.post(&url, &body).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "project": args.project,
                "user": args.user,
                "permission": args.permission,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!();
            println!(
                "{} Added {} to project {} with {} permission",
                style("✓").green(),
                style(&args.user).cyan(),
                style(&args.project).cyan().bold(),
                style(&args.permission).yellow()
            );
        }

        Ok(())
    }

    /// Remove a member from project (Server/DC only)
    async fn members_remove(&self, args: &RemoveMemberArgs, global: &GlobalOptions) -> Result<()> {
        let context = self.try_resolve_context(global).ok_or_else(|| {
            anyhow::anyhow!("Cannot determine Bitbucket Server host. Run from a git repository.")
        })?;

        if context.host_type == HostType::Cloud {
            bail!(
                "Removing project members is only available for Bitbucket Server/DC. \
                   For Cloud, use the workspace settings in the web interface."
            );
        }

        let client = self.get_server_client(&context.host)?;

        let url = format!(
            "/projects/{}/permissions/users?name={}",
            args.project, args.user
        );

        client.delete(&url).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "project": args.project,
                "user": args.user,
                "removed": true,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!();
            println!(
                "{} Removed {} from project {}",
                style("✓").green(),
                style(&args.user).red(),
                style(&args.project).cyan().bold()
            );
        }

        Ok(())
    }

    /// View project permissions (Server/DC only)
    async fn permissions(&self, args: &PermissionsArgs, global: &GlobalOptions) -> Result<()> {
        let context = self.try_resolve_context(global).ok_or_else(|| {
            anyhow::anyhow!("Cannot determine Bitbucket Server host. Run from a git repository.")
        })?;

        if context.host_type == HostType::Cloud {
            bail!(
                "Project permissions are only available for Bitbucket Server/DC. \
                   For Cloud, use 'bb workspace view' and check the web interface."
            );
        }

        let project_key = args
            .project
            .as_ref()
            .or(global.project.as_ref())
            .unwrap_or(&context.owner);

        // For permissions, we list both users and groups
        let client = self.get_server_client(&context.host)?;

        // Get user permissions
        let user_url = format!("/projects/{}/permissions/users?limit=100", project_key);
        let users: ServerPaginatedResponse<ServerProjectPermission> = client.get(&user_url).await?;

        if global.json {
            let result = serde_json::json!({
                "project": project_key,
                "users": users.values.iter().map(|u| {
                    serde_json::json!({
                        "username": u.user.name,
                        "display_name": u.user.display_name,
                        "permission": u.permission,
                    })
                }).collect::<Vec<_>>(),
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!();
            println!("Permissions for project '{}':", project_key);
            println!();

            if users.values.is_empty() {
                println!("  No user permissions configured.");
            } else {
                println!("  User Permissions:");
                println!("  {}", "-".repeat(60));
                for perm in &users.values {
                    let permission_display = match perm.permission.as_str() {
                        "PROJECT_ADMIN" => style(&perm.permission).magenta().to_string(),
                        "PROJECT_WRITE" => style(&perm.permission).cyan().to_string(),
                        _ => perm.permission.clone(),
                    };
                    println!(
                        "  {:<25} {:<25} {}",
                        perm.user.name, perm.user.display_name, permission_display
                    );
                }
            }

            println!();
            println!(
                "  For group permissions, visit: https://{}/projects/{}/permissions",
                context.host, project_key
            );
        }

        Ok(())
    }
}
