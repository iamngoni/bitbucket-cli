//
//  bitbucket-cli
//  cli/webhook.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Webhook management commands
//!
//! This module provides commands for managing repository webhooks.
//! Webhooks allow external services to receive notifications about events.
//!
//! ## Examples
//!
//! ```bash
//! # List webhooks
//! bb webhook list
//!
//! # Create a webhook
//! bb webhook create --url https://example.com/hook --events repo:push,pullrequest:created
//!
//! # View webhook details
//! bb webhook view <uuid>
//!
//! # Delete a webhook
//! bb webhook delete <uuid>
//! ```

use anyhow::{bail, Result};
use clap::{Args, Subcommand};
use console::style;
use serde::{Deserialize, Serialize};

use crate::api::BitbucketClient;
use crate::api::common::PaginatedResponse;
use crate::auth::{AuthCredential, KeyringStore};
use crate::config::Config;
use crate::context::{ContextResolver, HostType, RepoContext};
use crate::output::{OutputWriter, TableOutput};

use super::GlobalOptions;

/// Manage repository webhooks
#[derive(Args, Debug)]
pub struct WebhookCommand {
    #[command(subcommand)]
    pub command: WebhookSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum WebhookSubcommand {
    /// List webhooks
    #[command(visible_alias = "ls")]
    List,

    /// View webhook details
    View(ViewArgs),

    /// Create a new webhook
    Create(CreateArgs),

    /// Edit an existing webhook
    Edit(EditArgs),

    /// Delete a webhook
    Delete(DeleteArgs),

    /// View webhook deliveries (Cloud only)
    Deliveries(DeliveriesArgs),

    /// Test a webhook
    Test(TestArgs),
}

#[derive(Args, Debug)]
pub struct ViewArgs {
    /// Webhook UUID
    pub uuid: String,
}

#[derive(Args, Debug)]
pub struct CreateArgs {
    /// Webhook URL
    #[arg(long, short = 'u')]
    pub url: String,

    /// Description
    #[arg(long, short = 'd')]
    pub description: Option<String>,

    /// Events to listen for (comma-separated)
    #[arg(long, short = 'e', value_delimiter = ',')]
    pub events: Vec<String>,

    /// Secret for webhook signature verification
    #[arg(long, short = 's')]
    pub secret: Option<String>,

    /// Create as active (default: true)
    #[arg(long, default_value = "true")]
    pub active: bool,

    /// Skip SSL verification
    #[arg(long)]
    pub skip_cert_verification: bool,
}

#[derive(Args, Debug)]
pub struct EditArgs {
    /// Webhook UUID
    pub uuid: String,

    /// New URL
    #[arg(long, short = 'u')]
    pub url: Option<String>,

    /// New description
    #[arg(long, short = 'd')]
    pub description: Option<String>,

    /// Add events to listen for
    #[arg(long, value_delimiter = ',')]
    pub add_event: Vec<String>,

    /// Remove events
    #[arg(long, value_delimiter = ',')]
    pub remove_event: Vec<String>,

    /// New secret
    #[arg(long, short = 's')]
    pub secret: Option<String>,

    /// Set active state
    #[arg(long)]
    pub active: Option<bool>,
}

#[derive(Args, Debug)]
pub struct DeleteArgs {
    /// Webhook UUID
    pub uuid: String,

    /// Skip confirmation
    #[arg(long, short = 'y')]
    pub confirm: bool,
}

#[derive(Args, Debug)]
pub struct DeliveriesArgs {
    /// Webhook UUID
    pub uuid: String,

    /// Maximum number of deliveries to show
    #[arg(long, short = 'l', default_value = "10")]
    pub limit: usize,
}

#[derive(Args, Debug)]
pub struct TestArgs {
    /// Webhook UUID
    pub uuid: String,
}

// API response types

#[derive(Debug, Deserialize)]
struct Webhook {
    uuid: String,
    url: String,
    description: Option<String>,
    active: bool,
    events: Vec<String>,
    #[serde(default)]
    skip_cert_verification: bool,
    created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WebhookDelivery {
    uuid: String,
    event: String,
    triggered_at: String,
    response_status_code: Option<i32>,
    success: bool,
}

// Output types

#[derive(Debug, Serialize)]
struct WebhookListItem {
    uuid: String,
    url: String,
    description: Option<String>,
    active: bool,
    events: Vec<String>,
}

#[derive(Debug, Serialize)]
struct WebhookDetail {
    uuid: String,
    url: String,
    description: Option<String>,
    active: bool,
    events: Vec<String>,
    skip_cert_verification: bool,
    created_at: Option<String>,
}

#[derive(Debug, Serialize)]
struct DeliveryListItem {
    uuid: String,
    event: String,
    triggered_at: String,
    status_code: Option<i32>,
    success: bool,
}

impl TableOutput for WebhookListItem {
    fn print_table(&self, _color: bool) {
        let status = if self.active {
            style("active").green().to_string()
        } else {
            style("inactive").red().to_string()
        };
        println!(
            "{:<38} {:<10} {}",
            &self.uuid,
            status,
            truncate(&self.url, 40)
        );
    }

    fn print_markdown(&self) {
        println!("| {} | {} | {} |", self.uuid, self.active, self.url);
    }
}

impl TableOutput for DeliveryListItem {
    fn print_table(&self, _color: bool) {
        let status = if self.success {
            style("success").green().to_string()
        } else {
            style("failed").red().to_string()
        };
        let code = self.status_code.map(|c| c.to_string()).unwrap_or_else(|| "-".to_string());
        println!(
            "{:<38} {:<25} {:<10} {}",
            &self.uuid,
            &self.event,
            status,
            code
        );
    }

    fn print_markdown(&self) {
        println!("| {} | {} | {} | {} |", self.uuid, self.event, self.success, self.status_code.unwrap_or(0));
    }
}

impl WebhookCommand {
    pub async fn run(&self, global: &GlobalOptions) -> Result<()> {
        match &self.command {
            WebhookSubcommand::List => self.list(global).await,
            WebhookSubcommand::View(args) => self.view(args, global).await,
            WebhookSubcommand::Create(args) => self.create(args, global).await,
            WebhookSubcommand::Edit(args) => self.edit(args, global).await,
            WebhookSubcommand::Delete(args) => self.delete(args, global).await,
            WebhookSubcommand::Deliveries(args) => self.deliveries(args, global).await,
            WebhookSubcommand::Test(args) => self.test(args, global).await,
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
        let token = keyring.get(&ctx.host)?
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

    /// List webhooks
    async fn list(&self, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        let url = match ctx.host_type {
            HostType::Cloud => format!(
                "/repositories/{}/{}/hooks",
                ctx.owner, ctx.repo_slug
            ),
            HostType::Server => format!(
                "/projects/{}/repos/{}/webhooks",
                ctx.owner, ctx.repo_slug
            ),
        };

        let response: PaginatedResponse<Webhook> = client.get(&url).await?;

        let items: Vec<WebhookListItem> = response
            .values
            .into_iter()
            .map(|hook| WebhookListItem {
                uuid: hook.uuid,
                url: hook.url,
                description: hook.description,
                active: hook.active,
                events: hook.events,
            })
            .collect();

        if items.is_empty() {
            println!("No webhooks found.");
            return Ok(());
        }

        let writer = OutputWriter::new(self.get_format(global));

        if !global.json {
            println!();
            println!(
                "{} {} {} {}",
                style(format!("{:<38}", "UUID")).bold(),
                style(format!("{:<8}", "ACTIVE")).bold(),
                style(format!("{:<30}", "URL")).bold(),
                style("EVENTS").bold()
            );
            println!("{}", "-".repeat(100));

            for item in &items {
                item.print_table();
            }

            println!();
            println!("Showing {} webhook(s)", items.len());
        } else {
            writer.write_list(&items)?;
        }

        Ok(())
    }

    /// View webhook details
    async fn view(&self, args: &ViewArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        let url = match ctx.host_type {
            HostType::Cloud => format!(
                "/repositories/{}/{}/hooks/{}",
                ctx.owner, ctx.repo_slug, args.uuid
            ),
            HostType::Server => format!(
                "/projects/{}/repos/{}/webhooks/{}",
                ctx.owner, ctx.repo_slug, args.uuid
            ),
        };

        let hook: Webhook = client.get(&url).await?;

        let detail = WebhookDetail {
            uuid: hook.uuid,
            url: hook.url,
            description: hook.description,
            active: hook.active,
            events: hook.events,
            skip_cert_verification: hook.skip_cert_verification,
            created_at: hook.created_at,
        };

        let writer = OutputWriter::new(self.get_format(global));
        writer.write(&detail)?;

        Ok(())
    }

    /// Create a new webhook
    async fn create(&self, args: &CreateArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        if args.events.is_empty() {
            bail!("At least one event is required. Use --events to specify events.");
        }

        let url = match ctx.host_type {
            HostType::Cloud => format!(
                "/repositories/{}/{}/hooks",
                ctx.owner, ctx.repo_slug
            ),
            HostType::Server => format!(
                "/projects/{}/repos/{}/webhooks",
                ctx.owner, ctx.repo_slug
            ),
        };

        let body = serde_json::json!({
            "description": args.description.clone().unwrap_or_default(),
            "url": args.url,
            "active": args.active,
            "events": args.events,
            "skip_cert_verification": args.skip_cert_verification,
            "secret": args.secret,
        });

        let hook: Webhook = client.post(&url, &body).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "uuid": hook.uuid,
                "url": hook.url,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Created webhook {}",
                style("✓").green(),
                style(&hook.uuid).cyan()
            );
            println!("  URL: {}", hook.url);
            println!("  Events: {}", args.events.join(", "));
        }

        Ok(())
    }

    /// Edit a webhook
    async fn edit(&self, args: &EditArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        // First, get the current webhook
        let get_url = match ctx.host_type {
            HostType::Cloud => format!(
                "/repositories/{}/{}/hooks/{}",
                ctx.owner, ctx.repo_slug, args.uuid
            ),
            HostType::Server => format!(
                "/projects/{}/repos/{}/webhooks/{}",
                ctx.owner, ctx.repo_slug, args.uuid
            ),
        };

        let current: Webhook = client.get(&get_url).await?;

        // Build updated events list
        let mut events: Vec<String> = current.events;
        for event in &args.add_event {
            if !events.contains(event) {
                events.push(event.clone());
            }
        }
        for event in &args.remove_event {
            events.retain(|e| e != event);
        }

        // Build update body
        let body = serde_json::json!({
            "description": args.description.clone().or(current.description),
            "url": args.url.clone().unwrap_or(current.url),
            "active": args.active.unwrap_or(current.active),
            "events": events,
            "secret": args.secret,
        });

        let hook: Webhook = client.put(&get_url, &body).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "uuid": hook.uuid,
                "url": hook.url,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Updated webhook {}",
                style("✓").green(),
                style(&hook.uuid).cyan()
            );
        }

        Ok(())
    }

    /// Delete a webhook
    async fn delete(&self, args: &DeleteArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let client = self.get_client(&ctx)?;

        // Confirm deletion
        if !args.confirm && !global.no_prompt {
            use dialoguer::Confirm;
            let confirmed = Confirm::new()
                .with_prompt(format!("Delete webhook {}?", args.uuid))
                .default(false)
                .interact()?;

            if !confirmed {
                println!("{} Cancelled.", style("!").yellow());
                return Ok(());
            }
        }

        let url = match ctx.host_type {
            HostType::Cloud => format!(
                "/repositories/{}/{}/hooks/{}",
                ctx.owner, ctx.repo_slug, args.uuid
            ),
            HostType::Server => format!(
                "/projects/{}/repos/{}/webhooks/{}",
                ctx.owner, ctx.repo_slug, args.uuid
            ),
        };

        client.delete(&url).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "uuid": args.uuid,
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Deleted webhook {}",
                style("✓").green(),
                style(&args.uuid).cyan()
            );
        }

        Ok(())
    }

    /// View webhook deliveries (Cloud only)
    async fn deliveries(&self, args: &DeliveriesArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;

        if !matches!(ctx.host_type, HostType::Cloud) {
            bail!("Webhook deliveries are only available for Bitbucket Cloud.");
        }

        let client = self.get_client(&ctx)?;

        let url = format!(
            "/repositories/{}/{}/hooks/{}/deliveries?pagelen={}",
            ctx.owner, ctx.repo_slug, args.uuid, args.limit
        );

        let response: PaginatedResponse<WebhookDelivery> = client.get(&url).await?;

        let items: Vec<DeliveryListItem> = response
            .values
            .into_iter()
            .map(|d| DeliveryListItem {
                uuid: d.uuid,
                event: d.event,
                triggered_at: d.triggered_at,
                status_code: d.response_status_code,
                success: d.success,
            })
            .collect();

        if items.is_empty() {
            println!("No deliveries found.");
            return Ok(());
        }

        let writer = OutputWriter::new(self.get_format(global));

        if !global.json {
            println!();
            println!(
                "{} {} {} {} {}",
                style(format!("{:<38}", "UUID")).bold(),
                style(format!("{:<25}", "EVENT")).bold(),
                style(format!("{:<20}", "TRIGGERED")).bold(),
                style(format!("{:<6}", "STATUS")).bold(),
                style("SUCCESS").bold()
            );
            println!("{}", "-".repeat(100));

            for item in &items {
                item.print_table();
            }

            println!();
            println!("Showing {} delivery/deliveries", items.len());
        } else {
            writer.write_list(&items)?;
        }

        Ok(())
    }

    /// Test a webhook
    async fn test(&self, args: &TestArgs, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;

        if !matches!(ctx.host_type, HostType::Cloud) {
            bail!("Webhook testing is only available for Bitbucket Cloud.");
        }

        let client = self.get_client(&ctx)?;

        // Trigger a test ping
        let url = format!(
            "/repositories/{}/{}/hooks/{}/test",
            ctx.owner, ctx.repo_slug, args.uuid
        );

        let _: serde_json::Value = client.post(&url, &serde_json::json!({})).await?;

        if global.json {
            let result = serde_json::json!({
                "success": true,
                "uuid": args.uuid,
                "message": "Test payload sent",
            });
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!(
                "{} Test payload sent to webhook {}",
                style("✓").green(),
                style(&args.uuid).cyan()
            );
        }

        Ok(())
    }
}

// Implement Display for list items
impl WebhookListItem {
    pub fn print_table(&self) {
        let active_str = if self.active {
            style("active").green().to_string()
        } else {
            style("inactive").dim().to_string()
        };

        let url_truncated = if self.url.len() > 28 {
            format!("{}...", &self.url[..25])
        } else {
            self.url.clone()
        };

        let events_str = if self.events.len() > 2 {
            format!("{}, +{}", self.events[..2].join(", "), self.events.len() - 2)
        } else {
            self.events.join(", ")
        };

        println!(
            "{:<38} {:<8} {:<30} {}",
            self.uuid,
            active_str,
            url_truncated,
            events_str
        );
    }
}

impl DeliveryListItem {
    pub fn print_table(&self) {
        let success_str = if self.success {
            style("yes").green().to_string()
        } else {
            style("no").red().to_string()
        };

        let status_str = self.status_code
            .map(|s| s.to_string())
            .unwrap_or_else(|| "-".to_string());

        println!(
            "{:<38} {:<25} {:<20} {:<6} {}",
            self.uuid,
            self.event,
            &self.triggered_at[..std::cmp::min(19, self.triggered_at.len())],
            status_str,
            success_str
        );
    }
}

// TableOutput implementations

impl TableOutput for WebhookDetail {
    fn print_table(&self, _color: bool) {
        println!();
        println!("{}", style("Webhook Details").bold());
        println!("{}", "-".repeat(60));
        println!("UUID:                 {}", style(&self.uuid).cyan());
        println!("URL:                  {}", self.url);
        println!("Description:          {}", self.description.as_deref().unwrap_or("-"));
        println!("Active:               {}", if self.active { style("yes").green() } else { style("no").dim() });
        println!("Events:               {}", self.events.join(", "));
        println!("Skip cert verify:     {}", self.skip_cert_verification);
        println!("Created:              {}", self.created_at.as_deref().unwrap_or("-"));
        println!();
    }

    fn print_markdown(&self) {
        println!("# Webhook: {}", self.uuid);
        println!();
        println!("- **URL**: {}", self.url);
        println!("- **Active**: {}", self.active);
        println!("- **Events**: {}", self.events.join(", "));
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
