//
//  bitbucket-cli
//  cli/mod.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! CLI command definitions using clap derive macros

mod auth;
mod repo;
mod pr;
mod issue;
mod pipeline;
mod workspace;
mod project;
mod browse;
mod api;
mod config;
mod alias;
mod extension;
mod webhook;
mod deploy;
mod artifact;
mod secret;
mod ssh_key;
mod completion;

pub use auth::AuthCommand;
pub use repo::RepoCommand;
pub use pr::PrCommand;
pub use issue::IssueCommand;
pub use pipeline::PipelineCommand;
pub use workspace::WorkspaceCommand;
pub use project::ProjectCommand;
pub use browse::BrowseCommand;
pub use api::ApiCommand;
pub use config::ConfigCommand;
pub use alias::AliasCommand;
pub use extension::ExtensionCommand;
pub use webhook::WebhookCommand;
pub use deploy::DeployCommand;
pub use artifact::ArtifactCommand;
pub use secret::SecretCommand;
pub use ssh_key::SshKeyCommand;
pub use completion::CompletionCommand;

use clap::{Parser, Subcommand};

/// Bitbucket CLI - Work with Bitbucket from the command line
#[derive(Parser, Debug)]
#[command(
    name = "bb",
    version,
    about = "Work with Bitbucket from the command line",
    long_about = "bb is a CLI for Bitbucket Cloud and Server/Data Center.\n\n\
                  It brings pull requests, repositories, pipelines, and more to your terminal.",
    propagate_version = true,
    after_help = "Use 'bb <command> --help' for more information about a command."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[command(flatten)]
    pub global: GlobalOptions,
}

/// Global options available to all commands
#[derive(Parser, Debug, Clone, Default)]
pub struct GlobalOptions {
    /// Repository in WORKSPACE/REPO or PROJECT/REPO format
    #[arg(long, short = 'R', global = true, env = "BB_REPO")]
    pub repo: Option<String>,

    /// Workspace for the operation (Cloud)
    #[arg(long, short = 'w', global = true, env = "BB_WORKSPACE")]
    pub workspace: Option<String>,

    /// Project key for the operation (Server/DC)
    #[arg(long, short = 'p', global = true, env = "BB_PROJECT")]
    pub project: Option<String>,

    /// Bitbucket host (for Server/DC)
    #[arg(long, global = true, env = "BB_HOST")]
    pub host: Option<String>,

    /// Output format as JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Disable interactive prompts
    #[arg(long, global = true, env = "BB_NO_PROMPT")]
    pub no_prompt: bool,
}

/// Top-level commands
#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Authenticate with Bitbucket
    #[command(visible_alias = "login")]
    Auth(AuthCommand),

    /// Manage repositories
    #[command(visible_alias = "r")]
    Repo(RepoCommand),

    /// Manage pull requests
    Pr(PrCommand),

    /// Manage issues
    Issue(IssueCommand),

    /// Manage pipelines (Cloud)
    Pipeline(PipelineCommand),

    /// Manage workspaces (Cloud)
    #[command(visible_alias = "ws")]
    Workspace(WorkspaceCommand),

    /// Manage projects
    #[command(visible_alias = "proj")]
    Project(ProjectCommand),

    /// Open in browser
    Browse(BrowseCommand),

    /// Make API requests
    Api(ApiCommand),

    /// Manage CLI configuration
    Config(ConfigCommand),

    /// Manage command aliases
    Alias(AliasCommand),

    /// Manage CLI extensions
    #[command(visible_alias = "ext")]
    Extension(ExtensionCommand),

    /// Manage webhooks
    Webhook(WebhookCommand),

    /// Manage deployments (Cloud)
    Deploy(DeployCommand),

    /// Manage build artifacts (Cloud)
    Artifact(ArtifactCommand),

    /// Manage secrets and variables
    Secret(SecretCommand),

    /// Manage SSH keys
    #[command(name = "ssh-key")]
    SshKey(SshKeyCommand),

    /// Generate shell completion scripts
    Completion(CompletionCommand),

    /// Print version information
    Version,
}
