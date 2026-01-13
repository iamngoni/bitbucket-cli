//
//  bitbucket-cli
//  main.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

use anyhow::Result;
use clap::Parser;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use bitbucket_cli::cli::{Cli, Commands};
use bitbucket_cli::exit_codes;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    init_logging();

    // Parse CLI arguments
    let cli = Cli::parse();

    // Execute command
    let result = run(cli).await;

    // Handle result and exit
    match result {
        Ok(()) => std::process::exit(exit_codes::SUCCESS),
        Err(e) => {
            eprintln!("Error: {e:#}");
            std::process::exit(exit_codes::ERROR);
        }
    }
}

/// Initialize logging based on environment
fn init_logging() {
    let filter = EnvFilter::try_from_env("BB_DEBUG")
        .unwrap_or_else(|_| EnvFilter::new("warn"));

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false))
        .with(filter)
        .init();
}

/// Main command dispatcher
async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Commands::Auth(cmd) => cmd.run(&cli.global).await,
        Commands::Repo(cmd) => cmd.run(&cli.global).await,
        Commands::Pr(cmd) => cmd.run(&cli.global).await,
        Commands::Issue(cmd) => cmd.run(&cli.global).await,
        Commands::Pipeline(cmd) => cmd.run(&cli.global).await,
        Commands::Workspace(cmd) => cmd.run(&cli.global).await,
        Commands::Project(cmd) => cmd.run(&cli.global).await,
        Commands::Browse(cmd) => cmd.run(&cli.global).await,
        Commands::Api(cmd) => cmd.run(&cli.global).await,
        Commands::Config(cmd) => cmd.run(&cli.global).await,
        Commands::Alias(cmd) => cmd.run(&cli.global).await,
        Commands::Extension(cmd) => cmd.run(&cli.global).await,
        Commands::Webhook(cmd) => cmd.run(&cli.global).await,
        Commands::Deploy(cmd) => cmd.run(&cli.global).await,
        Commands::Artifact(cmd) => cmd.run(&cli.global).await,
        Commands::Secret(cmd) => cmd.run(&cli.global).await,
        Commands::SshKey(cmd) => cmd.run(&cli.global).await,
        Commands::Completion(cmd) => cmd.run(&cli.global).await,
        Commands::Version => {
            println!("bb version {}", bitbucket_cli::VERSION);
            Ok(())
        }
    }
}
