//
//  bitbucket-cli
//  cli/auth.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Authentication commands for the Bitbucket CLI.
//!
//! This module provides commands for managing authentication with Bitbucket
//! Cloud and Server/Data Center instances.

use anyhow::{Context, Result};
use clap::{Args, Subcommand};

use crate::auth::{
    get_cloud_username, oauth_login, read_token_from_stdin, refresh_oauth_token,
    validate_cloud_token, validate_token, KeyringStore, OAuthConfig, PersonalAccessToken,
    DEFAULT_CLIENT_ID, DEFAULT_CLIENT_SECRET,
};
use crate::config::{Config, HostConfig};
use crate::interactive::{prompt_confirm_with_default, prompt_input, prompt_password};

use super::GlobalOptions;

/// Authenticate with Bitbucket.
///
/// This command group provides subcommands for managing authentication
/// with Bitbucket Cloud and Server/Data Center instances.
#[derive(Args, Debug)]
pub struct AuthCommand {
    #[command(subcommand)]
    pub command: AuthSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum AuthSubcommand {
    /// Log in to Bitbucket
    Login(LoginArgs),

    /// Log out of Bitbucket
    Logout(LogoutArgs),

    /// View authentication status
    Status(StatusArgs),

    /// Refresh OAuth token
    Refresh,

    /// Switch authentication profile
    Switch(SwitchArgs),

    /// Print the authentication token
    Token(TokenArgs),

    /// Configure git to use bb as credential helper
    #[command(name = "setup-git")]
    SetupGit,
}

#[derive(Args, Debug)]
pub struct LoginArgs {
    /// Authenticate with Bitbucket Server/Data Center (self-hosted)
    #[arg(long, visible_alias = "self-hosted")]
    pub server: bool,

    /// The hostname of the Bitbucket instance (for Server/DC)
    #[arg(long, short = 'H')]
    pub host: Option<String>,

    /// Read token from standard input
    #[arg(long)]
    pub with_token: bool,

    /// OAuth scopes to request (Cloud only)
    #[arg(long, value_delimiter = ',')]
    pub scopes: Option<Vec<String>>,

    /// Open browser for authentication
    #[arg(long)]
    pub web: bool,
}

#[derive(Args, Debug)]
pub struct LogoutArgs {
    /// The hostname to log out from
    #[arg(long, short = 'H')]
    pub host: Option<String>,

    /// Log out of all accounts
    #[arg(long)]
    pub all: bool,
}

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Show the authentication token (masked)
    #[arg(long, short = 't')]
    pub show_token: bool,
}

#[derive(Args, Debug)]
pub struct SwitchArgs {
    /// Profile name to switch to
    #[arg(long, short = 'p')]
    pub profile: Option<String>,
}

#[derive(Args, Debug)]
pub struct TokenArgs {
    /// The hostname for which to print the token
    #[arg(long, short = 'H')]
    pub host: Option<String>,
}

impl AuthCommand {
    pub async fn run(&self, _global: &GlobalOptions) -> Result<()> {
        match &self.command {
            AuthSubcommand::Login(args) => login(args).await,
            AuthSubcommand::Logout(args) => logout(args).await,
            AuthSubcommand::Status(args) => status(args).await,
            AuthSubcommand::Refresh => refresh().await,
            AuthSubcommand::Switch(args) => switch(args).await,
            AuthSubcommand::Token(args) => token(args).await,
            AuthSubcommand::SetupGit => setup_git().await,
        }
    }
}

/// Performs the login flow.
async fn login(args: &LoginArgs) -> Result<()> {
    // Determine if we're logging into Cloud or Server/DC
    // Default to Cloud unless --server/--self-hosted is specified or a non-Cloud host is provided
    let is_server = args.server
        || args.host.as_ref().map_or(false, |h| {
            h != "bitbucket.org" && h != "api.bitbucket.org"
        });

    if is_server {
        login_server(args).await
    } else {
        login_cloud(args).await
    }
}

/// Performs Cloud OAuth login.
async fn login_cloud(args: &LoginArgs) -> Result<()> {
    let keyring = KeyringStore::new();
    let host = "bitbucket.org";

    // Check if already logged in
    if let Some(existing_token) = keyring.get(host)? {
        if validate_cloud_token(&existing_token).await? {
            let username = get_cloud_username(&existing_token).await?;
            if let Some(user) = username {
                println!("Already logged in to {} as {}", host, user);
                if !prompt_confirm_with_default("Re-authenticate?", false)? {
                    return Ok(());
                }
            }
        }
    }

    let token = if args.with_token {
        // Read token from stdin
        println!("Paste your access token:");
        let token = read_token_from_stdin()?;
        if !validate_token(&token) {
            anyhow::bail!("Invalid token format");
        }

        // Validate the token
        println!("Validating token...");
        if !validate_cloud_token(&token).await? {
            anyhow::bail!("Token is invalid or expired");
        }

        token
    } else {
        // OAuth flow
        let scopes = args.scopes.clone().unwrap_or_else(|| {
            vec![
                "repository".to_string(),
                "repository:write".to_string(),
                "pullrequest".to_string(),
                "pullrequest:write".to_string(),
                "account".to_string(),
                "pipeline".to_string(),
                "webhook".to_string(),
            ]
        });

        let config = OAuthConfig {
            client_id: DEFAULT_CLIENT_ID.to_string(),
            client_secret: Some(DEFAULT_CLIENT_SECRET.to_string()),
            scopes,
            ..Default::default()
        };

        let tokens = oauth_login(&config).await?;

        // Store refresh token separately if we have one
        if let Some(ref refresh) = tokens.refresh_token {
            keyring.store(&format!("{}.refresh", host), refresh)?;
        }

        tokens.access_token
    };

    // Store the token
    keyring.store(host, &token)?;

    // Get username and update config
    let username = get_cloud_username(&token).await?;
    let mut config = Config::load()?;

    config.hosts.insert(
        host.to_string(),
        HostConfig {
            host: host.to_string(),
            user: username.clone(),
            default_workspace: None,
            default_project: None,
            api_version: Some("2.0".to_string()),
        },
    );
    config.save()?;

    if let Some(user) = username {
        println!("Logged in to {} as {}", host, user);
    } else {
        println!("Logged in to {}", host);
    }

    Ok(())
}

/// Performs Server/DC PAT login.
async fn login_server(args: &LoginArgs) -> Result<()> {
    let keyring = KeyringStore::new();

    // Get the host
    let host = if let Some(ref h) = args.host {
        h.clone()
    } else {
        prompt_input("Bitbucket Server hostname (e.g., bitbucket.company.com):")?
    };

    // Normalize host (add https:// if needed, remove trailing slash)
    let host = normalize_host(&host);
    let host_key = extract_host_key(&host);

    // Check if already logged in
    if let Some(existing_token) = keyring.get(&host_key)? {
        let pat = PersonalAccessToken::new(existing_token.clone(), host.clone());
        if pat.validate().await? {
            let username = pat.get_username().await?;
            if let Some(user) = username {
                println!("Already logged in to {} as {}", host_key, user);
            } else {
                println!("Already logged in to {}", host_key);
            }
            if !prompt_confirm_with_default("Re-authenticate?", false)? {
                return Ok(());
            }
        }
    }

    // Get the token
    let token = if args.with_token {
        println!("Paste your Personal Access Token:");
        read_token_from_stdin()?
    } else {
        println!();
        println!("To create a Personal Access Token:");
        println!("  1. Go to {}/plugins/servlet/access-tokens/manage", host);
        println!("  2. Click 'Create token'");
        println!("  3. Give it a name and select permissions");
        println!("  4. Copy the generated token");
        println!();
        prompt_password("Personal Access Token:")?
    };

    if !validate_token(&token) {
        anyhow::bail!("Invalid token format");
    }

    // Validate the token
    println!("Validating token...");
    let pat = PersonalAccessToken::new(token.clone(), host.clone());
    if !pat.validate().await? {
        anyhow::bail!("Token is invalid or the server is unreachable");
    }

    // Store the token
    keyring.store(&host_key, &token)?;

    // Get username and update config
    let username = pat.get_username().await?;
    let mut config = Config::load()?;

    config.hosts.insert(
        host_key.clone(),
        HostConfig {
            host: host_key.clone(),
            user: username.clone(),
            default_workspace: None,
            default_project: None,
            api_version: Some("1.0".to_string()),
        },
    );
    config.save()?;

    if let Some(user) = username {
        println!("Logged in to {} as {}", host_key, user);
    } else {
        println!("Logged in to {}", host_key);
    }

    Ok(())
}

/// Performs logout.
async fn logout(args: &LogoutArgs) -> Result<()> {
    let keyring = KeyringStore::new();
    let mut config = Config::load()?;

    if args.all {
        // Log out of all hosts
        let hosts: Vec<String> = config.hosts.keys().cloned().collect();
        if hosts.is_empty() {
            println!("Not logged in to any hosts");
            return Ok(());
        }

        for host in &hosts {
            keyring.delete(host)?;
            keyring.delete(&format!("{}.refresh", host))?;
            config.hosts.remove(host);
        }
        config.save()?;

        println!("Logged out of {} host(s)", hosts.len());
    } else {
        let host = if let Some(ref h) = args.host {
            h.clone()
        } else {
            // Use default host or ask
            if config.hosts.len() == 1 {
                config.hosts.keys().next().unwrap().clone()
            } else if config.hosts.is_empty() {
                println!("Not logged in to any hosts");
                return Ok(());
            } else {
                // List hosts and ask which to log out from
                println!("Logged in to:");
                for (i, host) in config.hosts.keys().enumerate() {
                    println!("  {}) {}", i + 1, host);
                }
                let choice = prompt_input("Select host to log out from:")?;
                let idx: usize = choice.trim().parse().context("Invalid selection")?;
                config
                    .hosts
                    .keys()
                    .nth(idx - 1)
                    .cloned()
                    .unwrap_or_default()
            }
        };

        if host.is_empty() {
            anyhow::bail!("No host specified");
        }

        let host_key = extract_host_key(&host);
        keyring.delete(&host_key)?;
        keyring.delete(&format!("{}.refresh", host_key))?;
        config.hosts.remove(&host_key);
        config.save()?;

        println!("Logged out of {}", host_key);
    }

    Ok(())
}

/// Shows authentication status.
async fn status(args: &StatusArgs) -> Result<()> {
    let keyring = KeyringStore::new();
    let config = Config::load()?;

    if config.hosts.is_empty() {
        println!("Not logged in to any Bitbucket hosts");
        println!();
        println!("Run 'bb auth login' to authenticate");
        return Ok(());
    }

    for (host, host_config) in &config.hosts {
        let token = keyring.get(host)?;
        let is_valid = if let Some(ref t) = token {
            if host == "bitbucket.org" {
                validate_cloud_token(t).await.unwrap_or(false)
            } else {
                let pat = PersonalAccessToken::new(t.clone(), format!("https://{}", host));
                pat.validate().await.unwrap_or(false)
            }
        } else {
            false
        };

        println!("{}", host);
        if let Some(ref user) = host_config.user {
            println!("  Logged in as: {}", user);
        }
        println!(
            "  Status: {}",
            if is_valid {
                "Active"
            } else {
                "Invalid/Expired"
            }
        );

        if args.show_token {
            if let Some(ref t) = token {
                let masked = mask_token(t);
                println!("  Token: {}", masked);
            }
        }

        if let Some(ref api_version) = host_config.api_version {
            println!("  API Version: {}", api_version);
        }
        println!();
    }

    Ok(())
}

/// Refreshes OAuth token.
async fn refresh() -> Result<()> {
    let keyring = KeyringStore::new();
    let host = "bitbucket.org";

    // Get refresh token
    let refresh_token = keyring.get(&format!("{}.refresh", host))?.ok_or_else(|| {
        anyhow::anyhow!("No refresh token found. Please re-authenticate with 'bb auth login'")
    })?;

    println!("Refreshing token...");
    let tokens = refresh_oauth_token(&refresh_token, None, None).await?;

    // Store new tokens
    keyring.store(host, &tokens.access_token)?;
    if let Some(ref new_refresh) = tokens.refresh_token {
        keyring.store(&format!("{}.refresh", host), new_refresh)?;
    }

    println!("Token refreshed successfully");

    // Show expiration if available
    if let Some(expires_in) = tokens.expires_in {
        let hours = expires_in / 3600;
        let minutes = (expires_in % 3600) / 60;
        println!("New token expires in {} hours {} minutes", hours, minutes);
    }

    Ok(())
}

/// Switches authentication profile.
async fn switch(args: &SwitchArgs) -> Result<()> {
    let config = Config::load()?;

    if config.hosts.is_empty() {
        println!("No authentication profiles configured");
        println!("Run 'bb auth login' to authenticate");
        return Ok(());
    }

    if let Some(ref profile_name) = args.profile {
        // Switch to specific profile
        if !config.hosts.contains_key(profile_name) {
            anyhow::bail!("Profile '{}' not found", profile_name);
        }

        // For now, we just show the profile - in a full implementation,
        // we'd set this as the default profile
        let host_config = config.hosts.get(profile_name).unwrap();
        println!("Switched to profile: {}", profile_name);
        if let Some(ref user) = host_config.user {
            println!("  User: {}", user);
        }
    } else {
        // List available profiles
        println!("Available authentication profiles:");
        println!();
        for (host, host_config) in &config.hosts {
            println!("  {}", host);
            if let Some(ref user) = host_config.user {
                println!("    User: {}", user);
            }
        }
        println!();
        println!("Use 'bb auth switch --profile <name>' to switch profiles");
    }

    Ok(())
}

/// Prints the authentication token.
async fn token(args: &TokenArgs) -> Result<()> {
    let keyring = KeyringStore::new();
    let config = Config::load()?;

    let host = if let Some(ref h) = args.host {
        extract_host_key(h)
    } else if config.hosts.contains_key("bitbucket.org") {
        "bitbucket.org".to_string()
    } else if config.hosts.len() == 1 {
        config.hosts.keys().next().unwrap().clone()
    } else {
        anyhow::bail!("Multiple hosts configured. Specify one with --host");
    };

    let token = keyring
        .get(&host)?
        .ok_or_else(|| anyhow::anyhow!("No token found for {}", host))?;

    // Print just the token (useful for piping to other commands)
    println!("{}", token);

    Ok(())
}

/// Sets up git credential helper.
async fn setup_git() -> Result<()> {
    use std::process::Command;

    println!("Setting up git to use bb as credential helper...");

    // Configure git to use bb as the credential helper for Bitbucket
    let result = Command::new("git")
        .args([
            "config",
            "--global",
            "credential.https://bitbucket.org.helper",
            "!bb auth git-credential",
        ])
        .status();

    match result {
        Ok(status) if status.success() => {
            println!("Git credential helper configured for bitbucket.org");
        }
        Ok(status) => {
            eprintln!("Warning: git config command exited with status {}", status);
        }
        Err(e) => {
            eprintln!("Warning: Failed to run git config: {}", e);
        }
    }

    println!();
    println!("You can also manually add this to your ~/.gitconfig:");
    println!();
    println!("[credential \"https://bitbucket.org\"]");
    println!("    helper = !bb auth git-credential");
    println!();
    println!("For Bitbucket Server, add:");
    println!();
    println!("[credential \"https://your-server.com\"]");
    println!("    helper = !bb auth git-credential");

    Ok(())
}

// Helper functions

/// Normalizes a host URL.
fn normalize_host(host: &str) -> String {
    let host = host.trim();

    // Add https:// if no protocol specified
    let host = if !host.starts_with("http://") && !host.starts_with("https://") {
        format!("https://{}", host)
    } else {
        host.to_string()
    };

    // Remove trailing slash
    host.trim_end_matches('/').to_string()
}

/// Extracts the host key (hostname without protocol) for storage.
fn extract_host_key(host: &str) -> String {
    host.trim()
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/')
        .to_string()
}

/// Masks a token for display (shows first and last 4 characters).
fn mask_token(token: &str) -> String {
    if token.len() <= 8 {
        "*".repeat(token.len())
    } else {
        format!("{}...{}", &token[..4], &token[token.len() - 4..])
    }
}
