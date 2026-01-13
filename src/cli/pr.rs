//
//  bitbucket-cli
//  cli/pr.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use console::style;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::process::Command;

use super::GlobalOptions;
use crate::api::cloud::pullrequests as cloud_prs;
use crate::api::common::{PaginatedResponse, ServerPaginatedResponse};
use crate::api::format_api_error;
use crate::api::server::pullrequests as server_prs;
use crate::auth::KeyringStore;
use crate::config::Config;
use crate::context::{ContextResolver, HostType, RepoContext};
use crate::output::{print_field, print_header, OutputFormat, OutputWriter, TableOutput};

/// Manage pull requests
#[derive(Args, Debug)]
pub struct PrCommand {
    #[command(subcommand)]
    pub command: PrSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum PrSubcommand {
    /// List pull requests
    #[command(visible_alias = "ls")]
    List(ListArgs),

    /// View a pull request
    View(ViewArgs),

    /// Create a pull request
    Create(CreateArgs),

    /// Check out a pull request locally
    #[command(visible_alias = "co")]
    Checkout(CheckoutArgs),

    /// View pull request diff
    Diff(DiffArgs),

    /// Merge a pull request
    Merge(MergeArgs),

    /// Close/decline a pull request
    Close(CloseArgs),

    /// Reopen a declined pull request
    Reopen(ReopenArgs),

    /// Approve a pull request
    Approve(ApproveArgs),

    /// Request changes on a pull request
    #[command(name = "request-changes")]
    RequestChanges(RequestChangesArgs),

    /// Remove your approval from a pull request
    Unapprove(UnapproveArgs),

    /// Submit a review
    Review(ReviewArgs),

    /// Add a comment to a pull request
    Comment(CommentArgs),

    /// List comments on a pull request
    Comments(CommentsArgs),

    /// Edit a pull request
    Edit(EditArgs),

    /// Mark a pull request as ready for review
    Ready(ReadyArgs),

    /// View build status and checks
    Checks(ChecksArgs),
}

#[derive(Args, Debug)]
pub struct ListArgs {
    /// Filter by state
    #[arg(long, short = 's', value_parser = ["open", "merged", "declined", "superseded"])]
    pub state: Option<String>,

    /// Filter by author
    #[arg(long, short = 'a')]
    pub author: Option<String>,

    /// Filter by reviewer
    #[arg(long)]
    pub reviewer: Option<String>,

    /// Filter by assignee
    #[arg(long)]
    pub assignee: Option<String>,

    /// Filter by target branch
    #[arg(long, short = 'B')]
    pub base: Option<String>,

    /// Filter by source branch
    #[arg(long, short = 'H')]
    pub head: Option<String>,

    /// Filter by label
    #[arg(long, short = 'l')]
    pub label: Option<String>,

    /// Maximum number of PRs to list
    #[arg(long, short = 'L', default_value = "30")]
    pub limit: u32,

    /// Search in title and description
    #[arg(long, short = 'S')]
    pub search: Option<String>,
}

#[derive(Args, Debug)]
pub struct ViewArgs {
    /// Pull request number
    pub number: Option<u32>,

    /// Open in browser
    #[arg(long, short = 'w')]
    pub web: bool,

    /// Include comments
    #[arg(long, short = 'c')]
    pub comments: bool,
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Pull request title
    #[arg(long, short = 't')]
    pub title: Option<String>,

    /// Pull request body/description
    #[arg(long, short = 'b')]
    pub body: Option<String>,

    /// Read body from file
    #[arg(long, short = 'F')]
    pub body_file: Option<String>,

    /// Target branch
    #[arg(long, short = 'B')]
    pub base: Option<String>,

    /// Source branch
    #[arg(long, short = 'H')]
    pub head: Option<String>,

    /// Create as draft
    #[arg(long, short = 'd')]
    pub draft: bool,

    /// Add reviewers
    #[arg(long, short = 'r', action = clap::ArgAction::Append)]
    pub reviewer: Vec<String>,

    /// Add assignees
    #[arg(long, short = 'a', action = clap::ArgAction::Append)]
    pub assignee: Vec<String>,

    /// Add labels
    #[arg(long, short = 'l', action = clap::ArgAction::Append)]
    pub label: Vec<String>,

    /// Open in browser after creation
    #[arg(long, short = 'w')]
    pub web: bool,

    /// Auto-fill title and body from commits
    #[arg(long, short = 'f')]
    pub fill: bool,
}

#[derive(Args, Debug)]
pub struct CheckoutArgs {
    /// Pull request number
    pub number: u32,
}

#[derive(Args, Debug)]
pub struct DiffArgs {
    /// Pull request number
    pub number: Option<u32>,

    /// Show diff stats only
    #[arg(long)]
    pub stat: bool,

    /// Show full patch
    #[arg(long)]
    pub patch: bool,
}

#[derive(Args, Debug)]
pub struct MergeArgs {
    /// Pull request number
    pub number: Option<u32>,

    /// Merge commit strategy
    #[arg(long, short = 'm', conflicts_with_all = ["squash", "rebase"])]
    pub merge: bool,

    /// Squash and merge
    #[arg(long, short = 's', conflicts_with_all = ["merge", "rebase"])]
    pub squash: bool,

    /// Rebase and merge
    #[arg(long, short = 'r', conflicts_with_all = ["merge", "squash"])]
    pub rebase: bool,

    /// Delete source branch after merge
    #[arg(long, short = 'd')]
    pub delete_branch: bool,

    /// Enable auto-merge when requirements are met
    #[arg(long)]
    pub auto: bool,

    /// Custom merge commit message
    #[arg(long, short = 'M')]
    pub message: Option<String>,
}

#[derive(Args, Debug)]
pub struct CloseArgs {
    /// Pull request number
    pub number: Option<u32>,
}

#[derive(Args, Debug)]
pub struct ReopenArgs {
    /// Pull request number
    pub number: u32,
}

#[derive(Args, Debug)]
pub struct ApproveArgs {
    /// Pull request number
    pub number: Option<u32>,
}

#[derive(Args, Debug)]
pub struct RequestChangesArgs {
    /// Pull request number
    pub number: Option<u32>,

    /// Comment explaining requested changes
    #[arg(long, short = 'b')]
    pub body: Option<String>,
}

#[derive(Args, Debug)]
pub struct UnapproveArgs {
    /// Pull request number
    pub number: Option<u32>,
}

#[derive(Args, Debug)]
pub struct ReviewArgs {
    /// Pull request number
    pub number: Option<u32>,

    /// Approve the pull request
    #[arg(long, short = 'a')]
    pub approve: bool,

    /// Request changes
    #[arg(long, short = 'r')]
    pub request_changes: bool,

    /// Leave a comment only
    #[arg(long, short = 'c')]
    pub comment: bool,

    /// Review body
    #[arg(long, short = 'b')]
    pub body: Option<String>,
}

#[derive(Args, Debug)]
pub struct CommentArgs {
    /// Pull request number
    pub number: Option<u32>,

    /// Comment body
    #[arg(long, short = 'b')]
    pub body: Option<String>,

    /// Read body from file
    #[arg(long, short = 'F')]
    pub body_file: Option<String>,

    /// Edit your last comment
    #[arg(long)]
    pub edit_last: bool,

    /// Reply to a specific comment
    #[arg(long)]
    pub reply_to: Option<u32>,
}

#[derive(Args, Debug)]
pub struct CommentsArgs {
    /// Pull request number
    pub number: Option<u32>,
}

#[derive(Args, Debug)]
pub struct EditArgs {
    /// Pull request number
    pub number: Option<u32>,

    /// New title
    #[arg(long, short = 't')]
    pub title: Option<String>,

    /// New body
    #[arg(long, short = 'b')]
    pub body: Option<String>,

    /// New target branch
    #[arg(long, short = 'B')]
    pub base: Option<String>,

    /// Add reviewer
    #[arg(long)]
    pub add_reviewer: Option<String>,
}

#[derive(Args, Debug)]
pub struct ReadyArgs {
    /// Pull request number
    pub number: Option<u32>,
}

#[derive(Args, Debug)]
pub struct ChecksArgs {
    /// Pull request number
    pub number: Option<u32>,

    /// Watch until all checks complete
    #[arg(long, short = 'w')]
    pub watch: bool,
}

/// Display format for PR in list output
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PrListItem {
    id: u64,
    title: String,
    state: String,
    author: String,
    source_branch: String,
    destination_branch: String,
    updated_on: String,
}

impl TableOutput for PrListItem {
    fn print_table(&self, color: bool) {
        let state_styled = if color {
            match self.state.as_str() {
                "OPEN" => style("OPEN").green().to_string(),
                "MERGED" => style("MERGED").magenta().to_string(),
                "DECLINED" => style("DECLINED").red().to_string(),
                "SUPERSEDED" => style("SUPERSEDED").yellow().to_string(),
                _ => self.state.clone(),
            }
        } else {
            self.state.clone()
        };

        // Truncate title smartly
        let title_truncated = truncate_string(&self.title, 45);

        // Truncate author name
        let author_truncated = truncate_string(&self.author, 18);

        // Truncate branch names and format with arrow
        let source = truncate_string(&self.source_branch, 20);
        let dest = truncate_string(&self.destination_branch, 15);
        let branches = format!("{} → {}", source, dest);

        // Use bold for PR ID
        let id_styled = if color {
            style(format!("#{}", self.id)).cyan().to_string()
        } else {
            format!("#{}", self.id)
        };

        println!(
            "{:<7} {:<10} {:<45}  {:<18}  {}",
            id_styled, state_styled, title_truncated, author_truncated, branches
        );
    }

    fn print_markdown(&self) {
        println!(
            "- **#{}** [{}] {} (by @{}) `{}` → `{}`",
            self.id,
            self.state,
            self.title,
            self.author,
            self.source_branch,
            self.destination_branch
        );
    }
}

/// Truncate a string to max length, adding "…" if truncated
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        let truncated: String = s.chars().take(max_len - 1).collect();
        format!("{}…", truncated)
    } else {
        s.to_string()
    }
}

/// Display format for PR detail view
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PrDetail {
    id: u64,
    title: String,
    description: Option<String>,
    state: String,
    author: String,
    source_branch: String,
    destination_branch: String,
    reviewers: Vec<String>,
    approvals: u32,
    comment_count: u32,
    created_on: String,
    updated_on: String,
    web_url: String,
}

impl TableOutput for PrDetail {
    fn print_table(&self, color: bool) {
        let state_styled = if color {
            match self.state.as_str() {
                "OPEN" => style(&self.state).green().bold().to_string(),
                "MERGED" => style(&self.state).magenta().bold().to_string(),
                "DECLINED" => style(&self.state).red().bold().to_string(),
                "SUPERSEDED" => style(&self.state).yellow().bold().to_string(),
                _ => self.state.clone(),
            }
        } else {
            self.state.clone()
        };

        print_header(&format!("PR #{}: {}", self.id, self.title));
        println!();

        print_field("State", &state_styled, color);
        print_field("Author", &self.author, color);
        println!();

        print_field("Source", &self.source_branch, color);
        print_field("Destination", &self.destination_branch, color);
        println!();

        if !self.reviewers.is_empty() {
            print_field("Reviewers", &self.reviewers.join(", "), color);
        }
        print_field("Approvals", &self.approvals.to_string(), color);
        print_field("Comments", &self.comment_count.to_string(), color);
        println!();

        if let Some(desc) = &self.description {
            if !desc.is_empty() {
                println!("Description:");
                println!("{}", desc);
                println!();
            }
        }

        print_field("Created", &self.created_on, color);
        print_field("Updated", &self.updated_on, color);
        print_field("URL", &self.web_url, color);
    }

    fn print_markdown(&self) {
        println!("# PR #{}: {}", self.id, self.title);
        println!();
        println!("**State**: {} | **Author**: @{}", self.state, self.author);
        println!();
        println!("`{}` -> `{}`", self.source_branch, self.destination_branch);
        println!();

        if !self.reviewers.is_empty() {
            println!("**Reviewers**: {}", self.reviewers.join(", "));
        }
        println!(
            "**Approvals**: {} | **Comments**: {}",
            self.approvals, self.comment_count
        );
        println!();

        if let Some(desc) = &self.description {
            if !desc.is_empty() {
                println!("## Description");
                println!();
                println!("{}", desc);
                println!();
            }
        }

        println!("[View in browser]({})", self.web_url);
    }
}

/// Display format for PR comment
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PrComment {
    id: u64,
    author: String,
    content: String,
    created_on: String,
}

impl TableOutput for PrComment {
    fn print_table(&self, color: bool) {
        let author_styled = if color {
            style(&self.author).cyan().to_string()
        } else {
            self.author.clone()
        };

        println!("@{} ({}):", author_styled, self.created_on);
        println!("  {}", self.content);
        println!();
    }

    fn print_markdown(&self) {
        println!("**@{}** - {}", self.author, self.created_on);
        println!();
        println!("> {}", self.content);
        println!();
    }
}

impl PrCommand {
    pub async fn run(&self, global: &GlobalOptions) -> Result<()> {
        match &self.command {
            PrSubcommand::List(args) => self.list(args, global).await,
            PrSubcommand::View(args) => self.view(args, global).await,
            PrSubcommand::Create(args) => self.create(args, global).await,
            PrSubcommand::Checkout(args) => self.checkout(args, global).await,
            PrSubcommand::Diff(args) => self.diff(args, global).await,
            PrSubcommand::Merge(args) => self.merge(args, global).await,
            PrSubcommand::Close(args) => self.close(args, global).await,
            PrSubcommand::Reopen(args) => self.reopen(args, global).await,
            PrSubcommand::Approve(args) => self.approve(args, global).await,
            PrSubcommand::RequestChanges(args) => self.request_changes(args, global).await,
            PrSubcommand::Unapprove(args) => self.unapprove(args, global).await,
            PrSubcommand::Review(args) => self.review(args, global).await,
            PrSubcommand::Comment(args) => self.comment(args, global).await,
            PrSubcommand::Comments(args) => self.comments(args, global).await,
            PrSubcommand::Edit(args) => self.edit(args, global).await,
            PrSubcommand::Ready(args) => self.ready(args, global).await,
            PrSubcommand::Checks(args) => self.checks(args, global).await,
        }
    }

    /// List pull requests
    async fn list(&self, args: &ListArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

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
            let mut url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pullrequests?pagelen={}",
                context.owner, context.repo_slug, args.limit
            );

            // State filter
            let state = args.state.as_deref().unwrap_or("OPEN").to_uppercase();
            url.push_str(&format!("&state={}", state));

            // Build query parameters
            let mut q_parts: Vec<String> = Vec::new();

            if let Some(author) = &args.author {
                q_parts.push(format!("author.username=\"{}\"", author));
            }

            if let Some(base) = &args.base {
                q_parts.push(format!("destination.branch.name=\"{}\"", base));
            }

            if let Some(head) = &args.head {
                q_parts.push(format!("source.branch.name=\"{}\"", head));
            }

            if let Some(search) = &args.search {
                q_parts.push(format!("title~\"{}\"", search));
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

            let prs: PaginatedResponse<cloud_prs::PullRequest> = response.json().await?;

            let items: Vec<PrListItem> = prs
                .values
                .into_iter()
                .map(|pr| PrListItem {
                    id: pr.id,
                    title: pr.title,
                    state: pr.state,
                    author: pr.author.username.unwrap_or(pr.author.name),
                    source_branch: pr.source.branch.name,
                    destination_branch: pr.destination.branch.name,
                    updated_on: pr.updated_on,
                })
                .collect();

            if items.is_empty() {
                println!("No pull requests found");
            } else {
                if !global.json {
                    println!(
                        "Pull requests in {}/{}:\n",
                        context.owner, context.repo_slug
                    );
                    println!(
                        "{:<7} {:<10} {:<45}  {:<18}  {}",
                        style("ID").bold(),
                        style("STATE").bold(),
                        style("TITLE").bold(),
                        style("AUTHOR").bold(),
                        style("BRANCHES").bold()
                    );
                    println!("{}", style("─".repeat(110)).dim());
                }
                output.write_list(&items)?;
            }
        } else {
            // Bitbucket Server/DC
            let state = match args.state.as_deref() {
                Some("open") | None => "OPEN",
                Some("merged") => "MERGED",
                Some("declined") => "DECLINED",
                _ => "ALL",
            };

            let mut url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}/pull-requests?limit={}&state={}",
                context.host, context.owner, context.repo_slug, args.limit, state
            );

            if let Some(base) = &args.base {
                url.push_str(&format!("&at=refs/heads/{}", base));
            }

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

            let prs: ServerPaginatedResponse<server_prs::PullRequest> = response.json().await?;

            let items: Vec<PrListItem> = prs
                .values
                .into_iter()
                .map(|pr| PrListItem {
                    id: pr.id,
                    title: pr.title,
                    state: pr.state,
                    author: pr.author.user.display_name,
                    source_branch: pr.from_ref.display_id,
                    destination_branch: pr.to_ref.display_id,
                    updated_on: format_server_timestamp(pr.updated_date),
                })
                .collect();

            if items.is_empty() {
                println!("No pull requests found");
            } else {
                if !global.json {
                    println!(
                        "Pull requests in {}/{}:\n",
                        context.owner, context.repo_slug
                    );
                    println!(
                        "{:<7} {:<10} {:<45}  {:<18}  {}",
                        style("ID").bold(),
                        style("STATE").bold(),
                        style("TITLE").bold(),
                        style("AUTHOR").bold(),
                        style("BRANCHES").bold()
                    );
                    println!("{}", style("─".repeat(110)).dim());
                }
                output.write_list(&items)?;
            }
        }

        Ok(())
    }

    /// View pull request details
    async fn view(&self, args: &ViewArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        // Get PR number from arg or current branch
        let pr_number = if let Some(num) = args.number {
            num
        } else {
            // Try to find PR for current branch
            self.find_pr_for_current_branch(&context, &keyring).await?
        };

        // If web flag, open in browser
        if args.web {
            let url = if context.host_type == HostType::Cloud {
                format!(
                    "https://bitbucket.org/{}/{}/pull-requests/{}",
                    context.owner, context.repo_slug, pr_number
                )
            } else {
                format!(
                    "https://{}/projects/{}/repos/{}/pull-requests/{}",
                    context.host, context.owner, context.repo_slug, pr_number
                )
            };
            webbrowser::open(&url)?;
            println!("Opened PR #{} in browser", pr_number);
            return Ok(());
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

        let detail = if context.host_type == HostType::Cloud {
            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pullrequests/{}",
                context.owner, context.repo_slug, pr_number
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

            let pr: cloud_prs::PullRequest = response.json().await?;

            let approvals = pr.participants.iter().filter(|p| p.approved).count() as u32;
            let reviewers: Vec<String> = pr
                .reviewers
                .iter()
                .map(|r| r.username.clone().unwrap_or_else(|| r.name.clone()))
                .collect();

            PrDetail {
                id: pr.id,
                title: pr.title,
                description: pr.description,
                state: pr.state,
                author: pr.author.username.unwrap_or(pr.author.name),
                source_branch: pr.source.branch.name,
                destination_branch: pr.destination.branch.name,
                reviewers,
                approvals,
                comment_count: pr.comment_count,
                created_on: pr.created_on,
                updated_on: pr.updated_on,
                web_url: format!(
                    "https://bitbucket.org/{}/{}/pull-requests/{}",
                    context.owner, context.repo_slug, pr_number
                ),
            }
        } else {
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}/pull-requests/{}",
                context.host, context.owner, context.repo_slug, pr_number
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

            let pr: server_prs::PullRequest = response.json().await?;

            let approvals = pr.reviewers.iter().filter(|r| r.approved).count() as u32;
            let reviewers: Vec<String> = pr
                .reviewers
                .iter()
                .map(|r| r.user.display_name.clone())
                .collect();

            PrDetail {
                id: pr.id,
                title: pr.title,
                description: pr.description,
                state: pr.state,
                author: pr.author.user.display_name,
                source_branch: pr.from_ref.display_id,
                destination_branch: pr.to_ref.display_id,
                reviewers,
                approvals,
                comment_count: 0, // Server doesn't return this in PR response
                created_on: format_server_timestamp(pr.created_date),
                updated_on: format_server_timestamp(pr.updated_date),
                web_url: format!(
                    "https://{}/projects/{}/repos/{}/pull-requests/{}",
                    context.host, context.owner, context.repo_slug, pr_number
                ),
            }
        };

        output.write(&detail)?;

        // Show comments if requested
        if args.comments {
            println!("\n--- Comments ---\n");
            self.fetch_and_display_comments(&context, &token, pr_number, global)
                .await?;
        }

        Ok(())
    }

    /// Create a new pull request
    async fn create(&self, args: &CreateArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        let token = keyring.get(&context.host)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Not authenticated for {}. Run 'bb auth login' first.",
                context.host
            )
        })?;

        // Get source branch (current branch or specified)
        let source_branch = if let Some(head) = &args.head {
            head.clone()
        } else {
            self.get_current_branch()?
        };

        // Get destination branch
        let dest_branch = if let Some(base) = &args.base {
            base.clone()
        } else {
            // Default to main/master
            context
                .default_branch
                .clone()
                .unwrap_or_else(|| "main".to_string())
        };

        // Get title
        let title = if let Some(t) = &args.title {
            t.clone()
        } else if args.fill {
            // Auto-fill from first commit
            self.get_first_commit_message(&source_branch, &dest_branch)?
        } else {
            use dialoguer::Input;
            Input::new()
                .with_prompt("Pull request title")
                .interact_text()?
        };

        // Get body/description
        let description = if let Some(body) = &args.body {
            Some(body.clone())
        } else if let Some(file) = &args.body_file {
            Some(std::fs::read_to_string(file)?)
        } else if args.fill {
            self.get_commit_descriptions(&source_branch, &dest_branch)
                .ok()
        } else {
            None
        };

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
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pullrequests",
                context.owner, context.repo_slug
            );

            let reviewers: Vec<cloud_prs::UserUuid> = args
                .reviewer
                .iter()
                .map(|r| cloud_prs::UserUuid { uuid: r.clone() })
                .collect();

            let body = cloud_prs::CreatePullRequestRequest {
                title: title.clone(),
                description,
                source: cloud_prs::BranchSpec {
                    branch: cloud_prs::BranchName {
                        name: source_branch.clone(),
                    },
                },
                destination: cloud_prs::BranchSpec {
                    branch: cloud_prs::BranchName {
                        name: dest_branch.clone(),
                    },
                },
                reviewers,
                close_source_branch: Some(true),
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
                anyhow::bail!("Failed to create pull request ({}): {}", status, text);
            }

            let pr: cloud_prs::PullRequest = response.json().await?;

            output.write_success(&format!(
                "Created PR #{}: {} ({} -> {})",
                pr.id, pr.title, source_branch, dest_branch
            ));

            let pr_url = format!(
                "https://bitbucket.org/{}/{}/pull-requests/{}",
                context.owner, context.repo_slug, pr.id
            );
            println!("View at: {}", pr_url);

            if args.web {
                webbrowser::open(&pr_url)?;
            }
        } else {
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}/pull-requests",
                context.host, context.owner, context.repo_slug
            );

            let reviewers: Vec<server_prs::UserRef> = args
                .reviewer
                .iter()
                .map(|r| server_prs::UserRef {
                    user: server_prs::UserName { name: r.clone() },
                })
                .collect();

            let body = server_prs::CreatePullRequestRequest {
                title: title.clone(),
                description,
                from_ref: server_prs::RefSpec {
                    id: format!("refs/heads/{}", source_branch),
                    repository: server_prs::RepositorySpec {
                        slug: context.repo_slug.clone(),
                        project: server_prs::ProjectSpec {
                            key: context.owner.clone(),
                        },
                    },
                },
                to_ref: server_prs::RefSpec {
                    id: format!("refs/heads/{}", dest_branch),
                    repository: server_prs::RepositorySpec {
                        slug: context.repo_slug.clone(),
                        project: server_prs::ProjectSpec {
                            key: context.owner.clone(),
                        },
                    },
                },
                reviewers,
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
                anyhow::bail!("Failed to create pull request ({}): {}", status, text);
            }

            let pr: server_prs::PullRequest = response.json().await?;

            output.write_success(&format!(
                "Created PR #{}: {} ({} -> {})",
                pr.id, pr.title, source_branch, dest_branch
            ));

            let pr_url = format!(
                "https://{}/projects/{}/repos/{}/pull-requests/{}",
                context.host, context.owner, context.repo_slug, pr.id
            );
            println!("View at: {}", pr_url);

            if args.web {
                webbrowser::open(&pr_url)?;
            }
        }

        Ok(())
    }

    /// Checkout a PR locally
    async fn checkout(&self, args: &CheckoutArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        let token = keyring.get(&context.host)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Not authenticated for {}. Run 'bb auth login' first.",
                context.host
            )
        })?;

        let client = Client::builder()
            .user_agent(format!("bb/{}", crate::VERSION))
            .build()?;

        // Get PR details to find source branch
        let source_branch = if context.host_type == HostType::Cloud {
            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pullrequests/{}",
                context.owner, context.repo_slug, args.number
            );

            let response = client.get(&url).bearer_auth(&token).send().await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(format_api_error(status, &text));
            }

            let pr: cloud_prs::PullRequest = response.json().await?;
            pr.source.branch.name
        } else {
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}/pull-requests/{}",
                context.host, context.owner, context.repo_slug, args.number
            );

            let response = client.get(&url).bearer_auth(&token).send().await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(format_api_error(status, &text));
            }

            let pr: server_prs::PullRequest = response.json().await?;
            pr.from_ref.display_id
        };

        // Fetch and checkout
        println!("Fetching and checking out PR #{}...", args.number);

        let status = Command::new("git")
            .args(["fetch", "origin", &source_branch])
            .status()
            .context("Failed to fetch branch")?;

        if !status.success() {
            anyhow::bail!("Failed to fetch PR branch '{}'", source_branch);
        }

        let status = Command::new("git")
            .args(["checkout", &source_branch])
            .status()
            .context("Failed to checkout branch")?;

        if !status.success() {
            // Try creating a local tracking branch
            let status = Command::new("git")
                .args([
                    "checkout",
                    "-b",
                    &source_branch,
                    &format!("origin/{}", source_branch),
                ])
                .status()?;

            if !status.success() {
                anyhow::bail!("Failed to checkout PR branch '{}'", source_branch);
            }
        }

        println!("Checked out PR #{} branch '{}'", args.number, source_branch);

        Ok(())
    }

    /// View PR diff
    async fn diff(&self, args: &DiffArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        let pr_number = if let Some(num) = args.number {
            num
        } else {
            self.find_pr_for_current_branch(&context, &keyring).await?
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

        if context.host_type == HostType::Cloud {
            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pullrequests/{}/diff",
                context.owner, context.repo_slug, pr_number
            );

            let response = client.get(&url).bearer_auth(&token).send().await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(format_api_error(status, &text));
            }

            let diff = response.text().await?;

            if args.stat {
                self.print_diff_stats(&diff);
            } else {
                println!("{}", diff);
            }
        } else {
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}/pull-requests/{}/diff",
                context.host, context.owner, context.repo_slug, pr_number
            );

            let response = client.get(&url).bearer_auth(&token).send().await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(format_api_error(status, &text));
            }

            let diff = response.text().await?;

            if args.stat {
                self.print_diff_stats(&diff);
            } else {
                println!("{}", diff);
            }
        }

        Ok(())
    }

    /// Merge a pull request
    async fn merge(&self, args: &MergeArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        let pr_number = if let Some(num) = args.number {
            num
        } else {
            self.find_pr_for_current_branch(&context, &keyring).await?
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

        // Determine merge strategy
        let strategy = if args.squash {
            "squash"
        } else if args.rebase {
            "fast_forward"
        } else {
            "merge_commit"
        };

        if context.host_type == HostType::Cloud {
            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pullrequests/{}/merge",
                context.owner, context.repo_slug, pr_number
            );

            let body = cloud_prs::MergePullRequestRequest {
                message: args.message.clone(),
                close_source_branch: Some(args.delete_branch),
                merge_strategy: Some(strategy.to_string()),
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
                anyhow::bail!("Failed to merge PR ({}): {}", status, text);
            }

            output.write_success(&format!("Merged PR #{}", pr_number));
        } else {
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}/pull-requests/{}/merge",
                context.host, context.owner, context.repo_slug, pr_number
            );

            let body = server_prs::MergePullRequestRequest {
                version: None,
                message: args.message.clone(),
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
                anyhow::bail!("Failed to merge PR ({}): {}", status, text);
            }

            output.write_success(&format!("Merged PR #{}", pr_number));

            if args.delete_branch {
                println!(
                    "Note: Delete source branch after merge requires separate action on Server"
                );
            }
        }

        Ok(())
    }

    /// Close/decline a pull request
    async fn close(&self, args: &CloseArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        let pr_number = if let Some(num) = args.number {
            num
        } else {
            self.find_pr_for_current_branch(&context, &keyring).await?
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
            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pullrequests/{}/decline",
                context.owner, context.repo_slug, pr_number
            );

            let response = client
                .post(&url)
                .bearer_auth(&token)
                .send()
                .await
                .context("Failed to connect to Bitbucket Cloud")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to decline PR ({}): {}", status, text);
            }
        } else {
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}/pull-requests/{}/decline",
                context.host, context.owner, context.repo_slug, pr_number
            );

            let response = client
                .post(&url)
                .bearer_auth(&token)
                .send()
                .await
                .context("Failed to connect to Bitbucket Server")?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to decline PR ({}): {}", status, text);
            }
        }

        output.write_success(&format!("Declined PR #{}", pr_number));

        Ok(())
    }

    /// Reopen a declined PR
    async fn reopen(&self, args: &ReopenArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

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
            // Cloud: update PR state
            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pullrequests/{}",
                context.owner, context.repo_slug, args.number
            );

            #[derive(Serialize)]
            struct ReopenRequest {
                state: String,
            }

            let body = ReopenRequest {
                state: "OPEN".to_string(),
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
                anyhow::bail!("Failed to reopen PR ({}): {}", status, text);
            }
        } else {
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}/pull-requests/{}/reopen",
                context.host, context.owner, context.repo_slug, args.number
            );

            let response = client.post(&url).bearer_auth(&token).send().await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to reopen PR ({}): {}", status, text);
            }
        }

        output.write_success(&format!("Reopened PR #{}", args.number));

        Ok(())
    }

    /// Approve a PR
    async fn approve(&self, args: &ApproveArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        let pr_number = if let Some(num) = args.number {
            num
        } else {
            self.find_pr_for_current_branch(&context, &keyring).await?
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
            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pullrequests/{}/approve",
                context.owner, context.repo_slug, pr_number
            );

            let response = client.post(&url).bearer_auth(&token).send().await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to approve PR ({}): {}", status, text);
            }
        } else {
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}/pull-requests/{}/approve",
                context.host, context.owner, context.repo_slug, pr_number
            );

            let response = client.post(&url).bearer_auth(&token).send().await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to approve PR ({}): {}", status, text);
            }
        }

        output.write_success(&format!("Approved PR #{}", pr_number));

        Ok(())
    }

    /// Request changes on a PR
    async fn request_changes(
        &self,
        args: &RequestChangesArgs,
        global: &GlobalOptions,
    ) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        let pr_number = if let Some(num) = args.number {
            num
        } else {
            self.find_pr_for_current_branch(&context, &keyring).await?
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
            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pullrequests/{}/request-changes",
                context.owner, context.repo_slug, pr_number
            );

            let response = client.post(&url).bearer_auth(&token).send().await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to request changes ({}): {}", status, text);
            }

            // Add comment if provided
            if let Some(body) = &args.body {
                self.add_comment(&context, &token, pr_number, body).await?;
            }
        } else {
            // Server uses needs_work status
            // First add comment if provided, then set status
            if let Some(comment_body) = &args.body {
                self.add_comment(&context, &token, pr_number, comment_body)
                    .await?;
            }

            // Note: Server doesn't have a direct request-changes endpoint
            // Status is typically set via the review process
            println!("Note: Request changes functionality is limited on Server/DC. Comment added if provided.");
        }

        output.write_success(&format!("Requested changes on PR #{}", pr_number));

        Ok(())
    }

    /// Remove approval from a PR
    async fn unapprove(&self, args: &UnapproveArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        let pr_number = if let Some(num) = args.number {
            num
        } else {
            self.find_pr_for_current_branch(&context, &keyring).await?
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
            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pullrequests/{}/approve",
                context.owner, context.repo_slug, pr_number
            );

            let response = client.delete(&url).bearer_auth(&token).send().await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to remove approval ({}): {}", status, text);
            }
        } else {
            // Server: remove approval via the same approve endpoint
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}/pull-requests/{}/approve",
                context.host, context.owner, context.repo_slug, pr_number
            );

            let response = client.delete(&url).bearer_auth(&token).send().await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to remove approval ({}): {}", status, text);
            }
        }

        output.write_success(&format!("Removed approval from PR #{}", pr_number));

        Ok(())
    }

    /// Submit a review
    async fn review(&self, args: &ReviewArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        let pr_number = if let Some(num) = args.number {
            num
        } else {
            self.find_pr_for_current_branch(&context, &keyring).await?
        };

        let token = keyring.get(&context.host)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Not authenticated for {}. Run 'bb auth login' first.",
                context.host
            )
        })?;

        let output = OutputWriter::new(if global.json {
            OutputFormat::Json
        } else {
            OutputFormat::Table
        });

        // Add comment if body provided
        if let Some(body) = &args.body {
            self.add_comment(&context, &token, pr_number, body).await?;
        }

        // Perform review action
        if args.approve {
            self.approve(
                &ApproveArgs {
                    number: Some(pr_number),
                },
                global,
            )
            .await?;
        } else if args.request_changes {
            self.request_changes(
                &RequestChangesArgs {
                    number: Some(pr_number),
                    body: None,
                },
                global,
            )
            .await?;
        } else if args.comment && args.body.is_some() {
            output.write_success(&format!("Commented on PR #{}", pr_number));
        }

        Ok(())
    }

    /// Add a comment to a PR
    async fn comment(&self, args: &CommentArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        let pr_number = if let Some(num) = args.number {
            num
        } else {
            self.find_pr_for_current_branch(&context, &keyring).await?
        };

        // Get comment body
        let body = if let Some(b) = &args.body {
            b.clone()
        } else if let Some(file) = &args.body_file {
            std::fs::read_to_string(file)?
        } else {
            use dialoguer::Editor;
            Editor::new()
                .edit("Enter your comment")?
                .ok_or_else(|| anyhow::anyhow!("Comment cannot be empty"))?
        };

        let token = keyring.get(&context.host)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Not authenticated for {}. Run 'bb auth login' first.",
                context.host
            )
        })?;

        self.add_comment(&context, &token, pr_number, &body).await?;

        let output = OutputWriter::new(if global.json {
            OutputFormat::Json
        } else {
            OutputFormat::Table
        });
        output.write_success(&format!("Added comment to PR #{}", pr_number));

        Ok(())
    }

    /// List comments on a PR
    async fn comments(&self, args: &CommentsArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        let pr_number = if let Some(num) = args.number {
            num
        } else {
            self.find_pr_for_current_branch(&context, &keyring).await?
        };

        let token = keyring.get(&context.host)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Not authenticated for {}. Run 'bb auth login' first.",
                context.host
            )
        })?;

        self.fetch_and_display_comments(&context, &token, pr_number, global)
            .await
    }

    /// Edit a PR
    async fn edit(&self, args: &EditArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        let pr_number = if let Some(num) = args.number {
            num
        } else {
            self.find_pr_for_current_branch(&context, &keyring).await?
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

        #[derive(Serialize, Default)]
        struct UpdateRequest {
            #[serde(skip_serializing_if = "Option::is_none")]
            title: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
        }

        let body = UpdateRequest {
            title: args.title.clone(),
            description: args.body.clone(),
        };

        if context.host_type == HostType::Cloud {
            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pullrequests/{}",
                context.owner, context.repo_slug, pr_number
            );

            let response = client
                .put(&url)
                .bearer_auth(&token)
                .json(&body)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to update PR ({}): {}", status, text);
            }
        } else {
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}/pull-requests/{}",
                context.host, context.owner, context.repo_slug, pr_number
            );

            let response = client
                .put(&url)
                .bearer_auth(&token)
                .json(&body)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to update PR ({}): {}", status, text);
            }
        }

        // Add reviewer if specified
        if let Some(reviewer) = &args.add_reviewer {
            self.add_reviewer(&context, &token, pr_number, reviewer)
                .await?;
        }

        output.write_success(&format!("Updated PR #{}", pr_number));

        Ok(())
    }

    /// Mark PR as ready for review
    async fn ready(&self, args: &ReadyArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        let pr_number = if let Some(num) = args.number {
            num
        } else {
            self.find_pr_for_current_branch(&context, &keyring).await?
        };

        let output = OutputWriter::new(if global.json {
            OutputFormat::Json
        } else {
            OutputFormat::Table
        });

        // Bitbucket doesn't have native draft PRs like GitHub
        println!("Note: Bitbucket doesn't have draft PRs. Your PR is already visible and ready for review.");
        output.write_success(&format!("PR #{} is ready for review", pr_number));

        Ok(())
    }

    /// View build checks for a PR
    async fn checks(&self, args: &ChecksArgs, global: &GlobalOptions) -> Result<()> {
        let config = Config::load().unwrap_or_default();
        let resolver = ContextResolver::new(config);
        let keyring = KeyringStore::new();

        let context = resolver.resolve(global)?;

        let pr_number = if let Some(num) = args.number {
            num
        } else {
            self.find_pr_for_current_branch(&context, &keyring).await?
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

        if context.host_type == HostType::Cloud {
            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pullrequests/{}/statuses",
                context.owner, context.repo_slug, pr_number
            );

            let response = client.get(&url).bearer_auth(&token).send().await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(format_api_error(status, &text));
            }

            #[derive(Deserialize)]
            struct StatusResponse {
                values: Vec<BuildStatus>,
            }

            #[derive(Deserialize)]
            struct BuildStatus {
                state: String,
                name: String,
                description: Option<String>,
            }

            let statuses: StatusResponse = response.json().await?;

            if statuses.values.is_empty() {
                println!("No build checks found for PR #{}", pr_number);
            } else {
                println!("Build checks for PR #{}:\n", pr_number);
                for status in &statuses.values {
                    let state_icon = match status.state.as_str() {
                        "SUCCESSFUL" => style("✓").green(),
                        "FAILED" => style("✗").red(),
                        "INPROGRESS" => style("●").yellow(),
                        _ => style("?").dim(),
                    };
                    println!(
                        "{} {} - {}",
                        state_icon,
                        status.name,
                        status.description.as_deref().unwrap_or("")
                    );
                }
            }

            // Watch mode
            if args.watch {
                println!("\nWatching for changes... (Ctrl+C to exit)");
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

                    let response = client.get(&url).bearer_auth(&token).send().await?;

                    let statuses: StatusResponse = response.json().await?;

                    let all_done = statuses
                        .values
                        .iter()
                        .all(|s| s.state == "SUCCESSFUL" || s.state == "FAILED");

                    if all_done {
                        println!("\nAll checks completed!");
                        break;
                    }
                }
            }
        } else {
            // Server build status
            println!("Build checks for Server/DC require checking commit statuses.");
            println!("Use the web interface to view build status at:");
            println!(
                "  https://{}/projects/{}/repos/{}/pull-requests/{}",
                context.host, context.owner, context.repo_slug, pr_number
            );
        }

        Ok(())
    }

    // Helper methods

    /// Get current branch name
    fn get_current_branch(&self) -> Result<String> {
        let output = Command::new("git")
            .args(["rev-parse", "--abbrev-ref", "HEAD"])
            .output()
            .context("Failed to get current branch")?;

        if !output.status.success() {
            anyhow::bail!("Not in a git repository");
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Get first commit message for auto-fill
    fn get_first_commit_message(&self, source: &str, dest: &str) -> Result<String> {
        let output = Command::new("git")
            .args(["log", "--format=%s", "-1", &format!("{}..{}", dest, source)])
            .output()?;

        if output.status.success() && !output.stdout.is_empty() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            // Fallback to current branch name
            Ok(source.to_string())
        }
    }

    /// Get commit descriptions for auto-fill
    fn get_commit_descriptions(&self, source: &str, dest: &str) -> Result<String> {
        let output = Command::new("git")
            .args(["log", "--format=%B", &format!("{}..{}", dest, source)])
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Find PR for current branch
    async fn find_pr_for_current_branch(
        &self,
        context: &RepoContext,
        keyring: &KeyringStore,
    ) -> Result<u32> {
        let current_branch = self.get_current_branch()?;

        let token = keyring.get(&context.host)?.ok_or_else(|| {
            anyhow::anyhow!(
                "Not authenticated for {}. Run 'bb auth login' first.",
                context.host
            )
        })?;

        let client = Client::builder()
            .user_agent(format!("bb/{}", crate::VERSION))
            .build()?;

        if context.host_type == HostType::Cloud {
            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pullrequests?q=source.branch.name=\"{}\"&state=OPEN",
                context.owner, context.repo_slug, current_branch
            );

            let response = client.get(&url).bearer_auth(&token).send().await?;

            if response.status().is_success() {
                let prs: PaginatedResponse<cloud_prs::PullRequest> = response.json().await?;
                if let Some(pr) = prs.values.first() {
                    return Ok(pr.id as u32);
                }
            }
        } else {
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}/pull-requests?state=OPEN",
                context.host, context.owner, context.repo_slug
            );

            let response = client.get(&url).bearer_auth(&token).send().await?;

            if response.status().is_success() {
                let prs: ServerPaginatedResponse<server_prs::PullRequest> = response.json().await?;
                for pr in prs.values {
                    if pr.from_ref.display_id == current_branch {
                        return Ok(pr.id as u32);
                    }
                }
            }
        }

        anyhow::bail!(
            "No open PR found for branch '{}'. Please specify a PR number.",
            current_branch
        )
    }

    /// Print diff statistics
    fn print_diff_stats(&self, diff: &str) {
        let mut files_changed = 0;
        let mut insertions = 0;
        let mut deletions = 0;

        for line in diff.lines() {
            if line.starts_with("diff --git") {
                files_changed += 1;
            } else if line.starts_with('+') && !line.starts_with("+++") {
                insertions += 1;
            } else if line.starts_with('-') && !line.starts_with("---") {
                deletions += 1;
            }
        }

        println!(
            "{} files changed, {} insertions(+), {} deletions(-)",
            files_changed, insertions, deletions
        );
    }

    /// Add a comment to a PR
    async fn add_comment(
        &self,
        context: &RepoContext,
        token: &str,
        pr_number: u32,
        body: &str,
    ) -> Result<()> {
        let client = Client::builder()
            .user_agent(format!("bb/{}", crate::VERSION))
            .build()?;

        if context.host_type == HostType::Cloud {
            let url = format!(
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pullrequests/{}/comments",
                context.owner, context.repo_slug, pr_number
            );

            #[derive(Serialize)]
            struct CommentRequest {
                content: ContentBody,
            }

            #[derive(Serialize)]
            struct ContentBody {
                raw: String,
            }

            let request_body = CommentRequest {
                content: ContentBody {
                    raw: body.to_string(),
                },
            };

            let response = client
                .post(&url)
                .bearer_auth(token)
                .json(&request_body)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to add comment ({}): {}", status, text);
            }
        } else {
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}/pull-requests/{}/comments",
                context.host, context.owner, context.repo_slug, pr_number
            );

            #[derive(Serialize)]
            struct CommentRequest {
                text: String,
            }

            let request_body = CommentRequest {
                text: body.to_string(),
            };

            let response = client
                .post(&url)
                .bearer_auth(token)
                .json(&request_body)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to add comment ({}): {}", status, text);
            }
        }

        Ok(())
    }

    /// Fetch and display comments
    async fn fetch_and_display_comments(
        &self,
        context: &RepoContext,
        token: &str,
        pr_number: u32,
        global: &GlobalOptions,
    ) -> Result<()> {
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
                "https://api.bitbucket.org/2.0/repositories/{}/{}/pullrequests/{}/comments",
                context.owner, context.repo_slug, pr_number
            );

            let response = client.get(&url).bearer_auth(token).send().await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(format_api_error(status, &text));
            }

            #[derive(Deserialize)]
            struct CloudComment {
                id: u64,
                user: crate::api::common::UserRef,
                content: ContentBody,
                created_on: String,
            }

            #[derive(Deserialize)]
            struct ContentBody {
                raw: String,
            }

            #[derive(Deserialize)]
            struct CommentResponse {
                values: Vec<CloudComment>,
            }

            let comments: CommentResponse = response.json().await?;

            let items: Vec<PrComment> = comments
                .values
                .into_iter()
                .map(|c| PrComment {
                    id: c.id,
                    author: c.user.username.unwrap_or(c.user.name),
                    content: c.content.raw,
                    created_on: c.created_on,
                })
                .collect();

            if items.is_empty() {
                println!("No comments on PR #{}", pr_number);
            } else {
                output.write_list(&items)?;
            }
        } else {
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}/pull-requests/{}/activities",
                context.host, context.owner, context.repo_slug, pr_number
            );

            let response = client.get(&url).bearer_auth(token).send().await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                return Err(format_api_error(status, &text));
            }

            #[derive(Deserialize)]
            struct Activity {
                id: u64,
                user: server_prs::User,
                action: String,
                #[serde(default)]
                comment: Option<ServerComment>,
                #[serde(rename = "createdDate")]
                created_date: u64,
            }

            #[derive(Deserialize)]
            struct ServerComment {
                text: String,
            }

            #[derive(Deserialize)]
            struct ActivityResponse {
                values: Vec<Activity>,
            }

            let activities: ActivityResponse = response.json().await?;

            let items: Vec<PrComment> = activities
                .values
                .into_iter()
                .filter(|a| a.action == "COMMENTED" && a.comment.is_some())
                .map(|a| PrComment {
                    id: a.id,
                    author: a.user.display_name,
                    content: a.comment.map(|c| c.text).unwrap_or_default(),
                    created_on: format_server_timestamp(a.created_date),
                })
                .collect();

            if items.is_empty() {
                println!("No comments on PR #{}", pr_number);
            } else {
                output.write_list(&items)?;
            }
        }

        Ok(())
    }

    /// Add a reviewer to a PR
    async fn add_reviewer(
        &self,
        context: &RepoContext,
        token: &str,
        pr_number: u32,
        reviewer: &str,
    ) -> Result<()> {
        let client = Client::builder()
            .user_agent(format!("bb/{}", crate::VERSION))
            .build()?;

        if context.host_type == HostType::Cloud {
            println!("Note: Adding reviewers via CLI is limited on Cloud. Use web interface for full control.");
        } else {
            let url = format!(
                "https://{}/rest/api/1.0/projects/{}/repos/{}/pull-requests/{}/participants",
                context.host, context.owner, context.repo_slug, pr_number
            );

            #[derive(Serialize)]
            struct AddReviewer {
                user: UserRef,
                role: String,
            }

            #[derive(Serialize)]
            struct UserRef {
                name: String,
            }

            let body = AddReviewer {
                user: UserRef {
                    name: reviewer.to_string(),
                },
                role: "REVIEWER".to_string(),
            };

            let response = client
                .post(&url)
                .bearer_auth(token)
                .json(&body)
                .send()
                .await?;

            if !response.status().is_success() {
                let status = response.status();
                let text = response.text().await.unwrap_or_default();
                anyhow::bail!("Failed to add reviewer ({}): {}", status, text);
            }
        }

        Ok(())
    }
}

/// Format Unix timestamp (milliseconds) to readable string
fn format_server_timestamp(ms: u64) -> String {
    use chrono::{DateTime, Utc};

    let secs = (ms / 1000) as i64;
    let nsecs = ((ms % 1000) * 1_000_000) as u32;

    match DateTime::<Utc>::from_timestamp(secs, nsecs) {
        Some(dt) => dt.format("%Y-%m-%d %H:%M:%S").to_string(),
        None => ms.to_string(),
    }
}
