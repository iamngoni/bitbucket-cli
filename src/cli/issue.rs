//
//  bitbucket-cli
//  cli/issue.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Issue commands (Cloud only)
//!
//! This module provides commands for managing issues in Bitbucket Cloud repositories.
//! Issues track bugs, features, and tasks. Note: Issues are a Bitbucket Cloud feature
//! only - for Server/Data Center, use Jira integration.

use std::fs;

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use console::style;
use serde::{Deserialize, Serialize};

use crate::api::client::BitbucketClient;
use crate::api::cloud::issues::{
    CreateIssueRequest, Issue, IssueComment, IssueContentInput, UserUuid,
};
use crate::api::common::PaginatedResponse;
use crate::auth::{AuthCredential, KeyringStore};
use crate::config::Config;
use crate::context::{ContextResolver, HostType, RepoContext};
use crate::interactive::{prompt_confirm_with_default, prompt_input, prompt_input_optional};
use crate::output::{OutputFormat, OutputWriter, TableOutput};
use crate::util::open_browser;

use super::GlobalOptions;

/// Manage issues (Cloud only)
#[derive(Args, Debug)]
pub struct IssueCommand {
    #[command(subcommand)]
    pub command: IssueSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum IssueSubcommand {
    /// List issues
    #[command(visible_alias = "ls")]
    List(ListArgs),

    /// View an issue
    View(ViewArgs),

    /// Create a new issue
    Create(CreateArgs),

    /// Edit an issue
    Edit(EditArgs),

    /// Close an issue
    Close(CloseArgs),

    /// Reopen an issue
    Reopen(ReopenArgs),

    /// Comment on an issue
    Comment(CommentArgs),

    /// Delete an issue
    Delete(DeleteArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter by state
    #[arg(long, short = 's', value_parser = ["open", "new", "resolved", "closed", "on hold", "invalid", "duplicate", "wontfix"])]
    pub state: Option<String>,

    /// Filter by assignee
    #[arg(long, short = 'a')]
    pub assignee: Option<String>,

    /// Filter by reporter
    #[arg(long)]
    pub reporter: Option<String>,

    /// Filter by priority
    #[arg(long, short = 'p', value_parser = ["trivial", "minor", "major", "critical", "blocker"])]
    pub priority: Option<String>,

    /// Filter by kind
    #[arg(long, short = 'k', value_parser = ["bug", "enhancement", "proposal", "task"])]
    pub kind: Option<String>,

    /// Search in title
    #[arg(long, short = 'S')]
    pub search: Option<String>,

    /// Maximum number of issues to list
    #[arg(long, short = 'l', default_value = "25")]
    pub limit: u32,
}

#[derive(Args, Debug)]
pub struct ViewArgs {
    /// Issue ID
    pub id: u32,

    /// Open in browser
    #[arg(long, short = 'w')]
    pub web: bool,

    /// Include comments
    #[arg(long, short = 'c')]
    pub comments: bool,
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Issue title
    #[arg(long, short = 't')]
    pub title: Option<String>,

    /// Issue body/description
    #[arg(long, short = 'b')]
    pub body: Option<String>,

    /// Read body from file
    #[arg(long, short = 'F')]
    pub body_file: Option<String>,

    /// Assignee username
    #[arg(long, short = 'a')]
    pub assignee: Option<String>,

    /// Priority
    #[arg(long, short = 'p', value_parser = ["trivial", "minor", "major", "critical", "blocker"])]
    pub priority: Option<String>,

    /// Kind
    #[arg(long, short = 'k', value_parser = ["bug", "enhancement", "proposal", "task"])]
    pub kind: Option<String>,

    /// Open in browser after creation
    #[arg(long, short = 'w')]
    pub web: bool,
}

#[derive(Args, Debug)]
pub struct EditArgs {
    /// Issue ID
    pub id: u32,

    /// New title
    #[arg(long, short = 't')]
    pub title: Option<String>,

    /// New body/description
    #[arg(long, short = 'b')]
    pub body: Option<String>,

    /// New assignee
    #[arg(long, short = 'a')]
    pub assignee: Option<String>,

    /// New state
    #[arg(long, short = 's', value_parser = ["open", "new", "resolved", "closed", "on hold", "invalid", "duplicate", "wontfix"])]
    pub state: Option<String>,

    /// New priority
    #[arg(long, short = 'p', value_parser = ["trivial", "minor", "major", "critical", "blocker"])]
    pub priority: Option<String>,

    /// New kind
    #[arg(long, short = 'k', value_parser = ["bug", "enhancement", "proposal", "task"])]
    pub kind: Option<String>,
}

#[derive(Args, Debug)]
pub struct CloseArgs {
    /// Issue ID
    pub id: u32,

    /// Close reason/comment
    #[arg(long, short = 'r')]
    pub reason: Option<String>,
}

#[derive(Args, Debug)]
pub struct ReopenArgs {
    /// Issue ID
    pub id: u32,
}

#[derive(Args, Debug)]
pub struct CommentArgs {
    /// Issue ID
    pub id: u32,

    /// Comment body
    #[arg(long, short = 'b')]
    pub body: Option<String>,

    /// Read body from file
    #[arg(long, short = 'F')]
    pub body_file: Option<String>,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// Issue ID
    pub id: u32,

    /// Skip confirmation prompt
    #[arg(long)]
    pub confirm: bool,
}

// Display types

#[derive(Debug, Serialize)]
struct IssueListItem {
    id: u64,
    title: String,
    state: String,
    priority: String,
    kind: String,
    reporter: String,
    assignee: Option<String>,
    votes: u32,
}

impl TableOutput for IssueListItem {
    fn print_table(&self, color: bool) {
        let id_display = if color {
            style(format!("#{}", self.id)).cyan().bold().to_string()
        } else {
            format!("#{}", self.id)
        };

        let state_display = if color {
            match self.state.as_str() {
                "open" | "new" => style(&self.state).green().to_string(),
                "resolved" | "closed" => style(&self.state).dim().to_string(),
                "on hold" => style(&self.state).yellow().to_string(),
                _ => style(&self.state).red().to_string(),
            }
        } else {
            self.state.clone()
        };

        let priority_display = if color {
            match self.priority.as_str() {
                "blocker" | "critical" => style(&self.priority).red().bold().to_string(),
                "major" => style(&self.priority).yellow().to_string(),
                "minor" => style(&self.priority).dim().to_string(),
                "trivial" => style(&self.priority).dim().to_string(),
                _ => self.priority.clone(),
            }
        } else {
            self.priority.clone()
        };

        let assignee = self.assignee.as_deref().unwrap_or("-");

        println!(
            "{:<8} {:<12} {:<10} {:<10} {}",
            id_display, state_display, priority_display, assignee, self.title
        );
    }

    fn print_markdown(&self) {
        let assignee = self.assignee.as_deref().unwrap_or("-");
        println!(
            "| #{} | {} | {} | {} | {} | {} |",
            self.id, self.state, self.priority, self.kind, assignee, self.title
        );
    }
}

#[derive(Debug, Serialize)]
struct IssueDetail {
    id: u64,
    title: String,
    content: Option<String>,
    state: String,
    priority: String,
    kind: String,
    reporter: String,
    assignee: Option<String>,
    votes: u32,
    created_on: String,
    updated_on: String,
    url: String,
}

impl TableOutput for IssueDetail {
    fn print_table(&self, color: bool) {
        let title = if color {
            style(&self.title).bold().to_string()
        } else {
            self.title.clone()
        };

        let state_display = if color {
            match self.state.as_str() {
                "open" | "new" => style(&self.state).green().to_string(),
                "resolved" | "closed" => style(&self.state).dim().to_string(),
                _ => style(&self.state).yellow().to_string(),
            }
        } else {
            self.state.clone()
        };

        println!(
            "{} #{}",
            if color {
                style("Issue").cyan().bold().to_string()
            } else {
                "Issue".to_string()
            },
            self.id
        );
        println!();
        println!("  {}", title);
        println!();
        println!("  State:    {}", state_display);
        println!("  Priority: {}", self.priority);
        println!("  Kind:     {}", self.kind);
        println!("  Reporter: {}", self.reporter);
        println!(
            "  Assignee: {}",
            self.assignee.as_deref().unwrap_or("Unassigned")
        );
        println!("  Votes:    {}", self.votes);
        println!();
        println!("  Created:  {}", self.created_on);
        println!("  Updated:  {}", self.updated_on);

        if let Some(content) = &self.content {
            println!();
            println!("{}", if color { style("Description").bold().to_string() } else { "Description".to_string() });
            println!("{}", "-".repeat(60));
            println!("{}", content);
        }

        println!();
        println!("  URL: {}", self.url);
    }

    fn print_markdown(&self) {
        println!("# Issue #{}: {}", self.id, self.title);
        println!();
        println!("| Property | Value |");
        println!("|----------|-------|");
        println!("| State | {} |", self.state);
        println!("| Priority | {} |", self.priority);
        println!("| Kind | {} |", self.kind);
        println!("| Reporter | {} |", self.reporter);
        println!(
            "| Assignee | {} |",
            self.assignee.as_deref().unwrap_or("Unassigned")
        );
        println!("| Votes | {} |", self.votes);

        if let Some(content) = &self.content {
            println!();
            println!("## Description");
            println!();
            println!("{}", content);
        }
    }
}

#[derive(Debug, Serialize)]
struct CommentDisplay {
    id: u64,
    author: String,
    content: String,
    created_on: String,
}

impl TableOutput for CommentDisplay {
    fn print_table(&self, color: bool) {
        let author = if color {
            style(&self.author).cyan().bold().to_string()
        } else {
            self.author.clone()
        };

        println!("{} commented on {}:", author, self.created_on);
        println!();
        for line in self.content.lines() {
            println!("  {}", line);
        }
        println!();
    }

    fn print_markdown(&self) {
        println!("### {} ({})", self.author, self.created_on);
        println!();
        println!("{}", self.content);
        println!();
    }
}

/// Issue update request
#[derive(Debug, Clone, Serialize)]
struct UpdateIssueRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<IssueContentInput>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assignee: Option<UserUuid>,
}

/// Comment request
#[derive(Debug, Clone, Serialize)]
struct CreateCommentRequest {
    content: IssueContentInput,
}

impl IssueCommand {
    pub async fn run(&self, global: &GlobalOptions) -> Result<()> {
        match &self.command {
            IssueSubcommand::List(args) => self.list(args, global).await,
            IssueSubcommand::View(args) => self.view(args, global).await,
            IssueSubcommand::Create(args) => self.create(args, global).await,
            IssueSubcommand::Edit(args) => self.edit(args, global).await,
            IssueSubcommand::Close(args) => self.close(args, global).await,
            IssueSubcommand::Reopen(args) => self.reopen(args, global).await,
            IssueSubcommand::Comment(args) => self.comment(args, global).await,
            IssueSubcommand::Delete(args) => self.delete(args, global).await,
        }
    }

    fn get_format(&self, global: &GlobalOptions) -> OutputFormat {
        if global.json {
            OutputFormat::Json
        } else {
            OutputFormat::Table
        }
    }

    fn resolve_context(&self, global: &GlobalOptions) -> Result<RepoContext> {
        let config = Config::load()?;
        let resolver = ContextResolver::new(config);
        resolver.resolve(global)
    }

    fn get_client(&self, ctx: &RepoContext) -> Result<BitbucketClient> {
        // Issues are Cloud-only
        if ctx.host_type != HostType::Cloud {
            bail!("Issues are only available on Bitbucket Cloud. For Server/Data Center, use Jira integration.");
        }

        let keyring = KeyringStore::new();
        let token = keyring.get(&ctx.host)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Not authenticated with {}. Run 'bb auth login' first.",
                ctx.host
            )
        })?;

        let client = BitbucketClient::cloud()?
            .with_auth(AuthCredential::PersonalAccessToken { token });

        Ok(client)
    }

    /// List issues
    async fn list(&self, args: &ListArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        // Build query parameters
        let mut query_parts = Vec::new();

        if let Some(state) = &args.state {
            query_parts.push(format!("state=\"{}\"", state));
        }

        if let Some(priority) = &args.priority {
            query_parts.push(format!("priority=\"{}\"", priority));
        }

        if let Some(kind) = &args.kind {
            query_parts.push(format!("kind=\"{}\"", kind));
        }

        if let Some(assignee) = &args.assignee {
            query_parts.push(format!("assignee.username=\"{}\"", assignee));
        }

        if let Some(reporter) = &args.reporter {
            query_parts.push(format!("reporter.username=\"{}\"", reporter));
        }

        if let Some(search) = &args.search {
            query_parts.push(format!("title~\"{}\"", search));
        }

        let mut url = format!(
            "/repositories/{}/{}/issues?pagelen={}",
            ctx.owner, ctx.repo_slug, args.limit
        );

        if !query_parts.is_empty() {
            url = format!("{}&q={}", url, query_parts.join(" AND "));
        }

        let response: PaginatedResponse<Issue> = client.get(&url).await?;

        let items: Vec<IssueListItem> = response
            .values
            .into_iter()
            .map(|issue| IssueListItem {
                id: issue.id,
                title: truncate(&issue.title, 50),
                state: issue.state,
                priority: issue.priority,
                kind: issue.kind,
                reporter: issue.reporter.name.clone(),
                assignee: issue.assignee.map(|a| a.name.clone()),
                votes: issue.votes,
            })
            .collect();

        if items.is_empty() {
            println!("No issues found.");
            return Ok(());
        }

        let writer = OutputWriter::new(self.get_format(global));

        if !global.json {
            println!();
            println!(
                "{} {} {} {} {}",
                style(format!("{:<8}", "ID")).bold(),
                style(format!("{:<12}", "STATE")).bold(),
                style(format!("{:<10}", "PRIORITY")).bold(),
                style(format!("{:<10}", "ASSIGNEE")).bold(),
                style("TITLE").bold()
            );
            println!("{}", "-".repeat(80));
        }

        writer.write_list(&items)?;

        if !global.json {
            println!();
            println!("Showing {} issue(s)", items.len());
        }

        Ok(())
    }

    /// View an issue
    async fn view(&self, args: &ViewArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;

        let url = format!(
            "https://bitbucket.org/{}/{}/issues/{}",
            ctx.owner, ctx.repo_slug, args.id
        );

        if args.web {
            println!("{} Opening issue #{} in browser...", style("→").cyan(), args.id);
            open_browser(&url)?;
            return Ok(());
        }

        let client = self.get_client(&ctx)?;

        let api_url = format!(
            "/repositories/{}/{}/issues/{}",
            ctx.owner, ctx.repo_slug, args.id
        );

        let issue: Issue = client.get(&api_url).await?;

        let detail = IssueDetail {
            id: issue.id,
            title: issue.title,
            content: issue.content.map(|c| c.raw),
            state: issue.state,
            priority: issue.priority,
            kind: issue.kind,
            reporter: issue.reporter.name.clone(),
            assignee: issue.assignee.map(|a| a.name.clone()),
            votes: issue.votes,
            created_on: format_timestamp(&issue.created_on),
            updated_on: format_timestamp(&issue.updated_on),
            url: url.clone(),
        };

        let writer = OutputWriter::new(self.get_format(global));
        writer.write(&detail)?;

        // Show comments if requested
        if args.comments && !global.json {
            let comments_url = format!(
                "/repositories/{}/{}/issues/{}/comments",
                ctx.owner, ctx.repo_slug, args.id
            );

            let comments_response: PaginatedResponse<IssueComment> =
                client.get(&comments_url).await?;

            if !comments_response.values.is_empty() {
                println!();
                println!("{}", style("Comments").bold());
                println!("{}", "-".repeat(60));

                for comment in comments_response.values {
                    let display = CommentDisplay {
                        id: comment.id,
                        author: comment.user.name.clone(),
                        content: comment.content.raw,
                        created_on: format_timestamp(&comment.created_on),
                    };
                    display.print_table(true);
                }
            }
        }

        Ok(())
    }

    /// Create an issue
    async fn create(&self, args: &CreateArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        // Get title
        let title = if let Some(t) = &args.title {
            t.clone()
        } else if !global.no_prompt {
            prompt_input("Issue title")?
        } else {
            bail!("Issue title is required. Use --title or run interactively.");
        };

        // Get body
        let body = if let Some(file) = &args.body_file {
            Some(fs::read_to_string(file)?)
        } else if let Some(b) = &args.body {
            Some(b.clone())
        } else if !global.no_prompt {
            prompt_input_optional("Issue description (optional)")?
        } else {
            None
        };

        let request = CreateIssueRequest {
            title: title.clone(),
            content: body.map(|b| IssueContentInput { raw: b }),
            state: Some("new".to_string()),
            priority: args.priority.clone(),
            kind: args.kind.clone(),
            assignee: None, // Would need to look up user UUID
        };

        let url = format!(
            "/repositories/{}/{}/issues",
            ctx.owner, ctx.repo_slug
        );

        let issue: Issue = client.post(&url, &request).await?;

        let issue_url = format!(
            "https://bitbucket.org/{}/{}/issues/{}",
            ctx.owner, ctx.repo_slug, issue.id
        );

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "id": issue.id,
                "title": issue.title,
                "state": issue.state,
                "url": issue_url,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!();
            println!(
                "{} Created issue #{}",
                style("✓").green(),
                style(issue.id).cyan().bold()
            );
            println!();
            println!("  Title: {}", issue.title);
            println!("  State: {}", issue.state);
            println!();
            println!("  URL: {}", issue_url);
        }

        if args.web {
            open_browser(&issue_url)?;
        }

        Ok(())
    }

    /// Edit an issue
    async fn edit(&self, args: &EditArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        let request = UpdateIssueRequest {
            title: args.title.clone(),
            content: args.body.clone().map(|b| IssueContentInput { raw: b }),
            state: args.state.clone(),
            priority: args.priority.clone(),
            kind: args.kind.clone(),
            assignee: None,
        };

        let url = format!(
            "/repositories/{}/{}/issues/{}",
            ctx.owner, ctx.repo_slug, args.id
        );

        let issue: Issue = client.put(&url, &request).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "id": issue.id,
                "title": issue.title,
                "state": issue.state,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Updated issue #{}",
                style("✓").green(),
                style(issue.id).cyan().bold()
            );
        }

        Ok(())
    }

    /// Close an issue
    async fn close(&self, args: &CloseArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        // Add comment if reason provided
        if let Some(reason) = &args.reason {
            let comment_url = format!(
                "/repositories/{}/{}/issues/{}/comments",
                ctx.owner, ctx.repo_slug, args.id
            );
            let comment = CreateCommentRequest {
                content: IssueContentInput {
                    raw: format!("Closing: {}", reason),
                },
            };
            let _: IssueComment = client.post(&comment_url, &comment).await?;
        }

        let request = UpdateIssueRequest {
            title: None,
            content: None,
            state: Some("closed".to_string()),
            priority: None,
            kind: None,
            assignee: None,
        };

        let url = format!(
            "/repositories/{}/{}/issues/{}",
            ctx.owner, ctx.repo_slug, args.id
        );

        let issue: Issue = client.put(&url, &request).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "id": issue.id,
                "state": issue.state,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Closed issue #{}",
                style("✓").green(),
                style(issue.id).cyan().bold()
            );
        }

        Ok(())
    }

    /// Reopen an issue
    async fn reopen(&self, args: &ReopenArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        let request = UpdateIssueRequest {
            title: None,
            content: None,
            state: Some("open".to_string()),
            priority: None,
            kind: None,
            assignee: None,
        };

        let url = format!(
            "/repositories/{}/{}/issues/{}",
            ctx.owner, ctx.repo_slug, args.id
        );

        let issue: Issue = client.put(&url, &request).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "id": issue.id,
                "state": issue.state,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Reopened issue #{}",
                style("✓").green(),
                style(issue.id).cyan().bold()
            );
        }

        Ok(())
    }

    /// Comment on an issue
    async fn comment(&self, args: &CommentArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        // Get body
        let body = if let Some(file) = &args.body_file {
            fs::read_to_string(file)?
        } else if let Some(b) = &args.body {
            b.clone()
        } else if !global.no_prompt {
            prompt_input("Comment")?
        } else {
            bail!("Comment body is required. Use --body or run interactively.");
        };

        let request = CreateCommentRequest {
            content: IssueContentInput { raw: body },
        };

        let url = format!(
            "/repositories/{}/{}/issues/{}/comments",
            ctx.owner, ctx.repo_slug, args.id
        );

        let comment: IssueComment = client.post(&url, &request).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "issue_id": args.id,
                "comment_id": comment.id,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Added comment to issue #{}",
                style("✓").green(),
                style(args.id).cyan().bold()
            );
        }

        Ok(())
    }

    /// Delete an issue
    async fn delete(&self, args: &DeleteArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        if !args.confirm && !global.no_prompt {
            let confirmed = prompt_confirm_with_default(
                &format!("Are you sure you want to delete issue #{}?", args.id),
                false,
            )?;

            if !confirmed {
                println!("Cancelled.");
                return Ok(());
            }
        }

        let url = format!(
            "/repositories/{}/{}/issues/{}",
            ctx.owner, ctx.repo_slug, args.id
        );

        client.delete(&url).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "id": args.id,
                "deleted": true,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Deleted issue #{}",
                style("✓").green(),
                style(args.id).cyan().bold()
            );
        }

        Ok(())
    }
}

// Helper functions

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn format_timestamp(ts: &str) -> String {
    // Parse ISO 8601 and format nicely
    chrono::DateTime::parse_from_rfc3339(ts)
        .map(|dt| dt.format("%Y-%m-%d %H:%M").to_string())
        .unwrap_or_else(|_| ts.to_string())
}
