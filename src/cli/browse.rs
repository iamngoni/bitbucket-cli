//
//  bitbucket-cli
//  cli/browse.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Browse command - open resources in browser
//!
//! This command provides quick access to various Bitbucket web pages
//! including repository views, settings, issues, pull requests, pipelines,
//! and more. It supports both Bitbucket Cloud and Server/Data Center.

use anyhow::{bail, Result};
use clap::Args;
use console::style;

use crate::config::Config;
use crate::context::{ContextResolver, HostType, RepoContext};
use crate::util::open_browser;

use super::GlobalOptions;

/// Open repository in browser
#[derive(Args, Debug)]
pub struct BrowseCommand {
    /// Path to open (file or directory)
    pub path: Option<String>,

    /// Branch name
    #[arg(long, short = 'b')]
    pub branch: Option<String>,

    /// Commit SHA
    #[arg(long, short = 'c')]
    pub commit: Option<String>,

    /// Open repository settings
    #[arg(long)]
    pub settings: bool,

    /// Open issues page
    #[arg(long)]
    pub issues: bool,

    /// Open pull requests page
    #[arg(long)]
    pub prs: bool,

    /// Open pipelines page
    #[arg(long)]
    pub pipelines: bool,

    /// Open wiki page
    #[arg(long)]
    pub wiki: bool,

    /// Open projects page
    #[arg(long)]
    pub projects: bool,

    /// Open branches page
    #[arg(long)]
    pub branches: bool,

    /// Open commits page
    #[arg(long)]
    pub commits: bool,

    /// Open downloads page
    #[arg(long)]
    pub downloads: bool,

    /// Print URL instead of opening browser
    #[arg(long, short = 'p')]
    pub print: bool,
}

impl BrowseCommand {
    pub async fn run(&self, global: &GlobalOptions) -> Result<()> {
        let ctx = self.resolve_context(global)?;
        let url = self.build_url(&ctx)?;

        if self.print {
            println!("{}", url);
        } else {
            let target = self.describe_target();
            println!("{} Opening {} in browser...", style("→").cyan(), target);
            open_browser(&url)?;
        }

        Ok(())
    }

    fn resolve_context(&self, global: &GlobalOptions) -> Result<RepoContext> {
        let config = Config::load()?;
        let resolver = ContextResolver::new(config);
        resolver.resolve(global).map_err(|_| {
            anyhow::anyhow!(
                "Could not determine repository. Run from a git repository or specify --repo."
            )
        })
    }

    fn describe_target(&self) -> &str {
        if self.settings {
            "settings"
        } else if self.issues {
            "issues"
        } else if self.prs {
            "pull requests"
        } else if self.pipelines {
            "pipelines"
        } else if self.wiki {
            "wiki"
        } else if self.projects {
            "projects"
        } else if self.branches {
            "branches"
        } else if self.commits {
            "commits"
        } else if self.downloads {
            "downloads"
        } else if self.path.is_some() {
            "file"
        } else {
            "repository"
        }
    }

    fn build_url(&self, ctx: &RepoContext) -> Result<String> {
        match ctx.host_type {
            HostType::Cloud => self.build_cloud_url(ctx),
            HostType::Server => self.build_server_url(ctx),
        }
    }

    /// Build URL for Bitbucket Cloud
    fn build_cloud_url(&self, ctx: &RepoContext) -> Result<String> {
        let base = format!("https://bitbucket.org/{}/{}", ctx.owner, ctx.repo_slug);

        // Handle special pages
        if self.settings {
            return Ok(format!("{}/admin", base));
        }

        if self.issues {
            return Ok(format!("{}/issues", base));
        }

        if self.prs {
            return Ok(format!("{}/pull-requests", base));
        }

        if self.pipelines {
            return Ok(format!("{}/pipelines", base));
        }

        if self.wiki {
            return Ok(format!("{}/wiki", base));
        }

        if self.projects {
            // Projects page is at workspace level
            return Ok(format!(
                "https://bitbucket.org/{}/workspace/projects",
                ctx.owner
            ));
        }

        if self.branches {
            return Ok(format!("{}/branches", base));
        }

        if self.commits {
            if let Some(branch) = &self.branch {
                return Ok(format!("{}/commits/branch/{}", base, branch));
            }
            return Ok(format!("{}/commits", base));
        }

        if self.downloads {
            return Ok(format!("{}/downloads", base));
        }

        // Handle commit view
        if let Some(commit) = &self.commit {
            return Ok(format!("{}/commits/{}", base, commit));
        }

        // Handle file/path view
        if let Some(path) = &self.path {
            let ref_spec = self.branch.as_deref().unwrap_or("HEAD");
            return Ok(format!("{}/src/{}/{}", base, ref_spec, path));
        }

        // Handle branch view (source browser at branch)
        if let Some(branch) = &self.branch {
            return Ok(format!("{}/src/{}", base, branch));
        }

        // Default: repository home
        Ok(base)
    }

    /// Build URL for Bitbucket Server/Data Center
    fn build_server_url(&self, ctx: &RepoContext) -> Result<String> {
        let base = format!(
            "https://{}/projects/{}/repos/{}",
            ctx.host, ctx.owner, ctx.repo_slug
        );

        // Handle special pages
        if self.settings {
            return Ok(format!("{}/settings", base));
        }

        if self.issues {
            bail!("Issues are not available on Bitbucket Server. Use Jira integration instead.");
        }

        if self.prs {
            return Ok(format!("{}/pull-requests", base));
        }

        if self.pipelines {
            bail!("Pipelines are a Bitbucket Cloud feature. Bitbucket Server uses external CI/CD.");
        }

        if self.wiki {
            bail!("Wiki is not available on Bitbucket Server in the same way as Cloud.");
        }

        if self.projects {
            // Projects page at server level
            return Ok(format!("https://{}/projects/{}", ctx.host, ctx.owner));
        }

        if self.branches {
            return Ok(format!("{}/branches", base));
        }

        if self.commits {
            if let Some(branch) = &self.branch {
                return Ok(format!("{}/commits?until=refs/heads/{}", base, branch));
            }
            return Ok(format!("{}/commits", base));
        }

        if self.downloads {
            bail!("Downloads page is not available on Bitbucket Server.");
        }

        // Handle commit view
        if let Some(commit) = &self.commit {
            return Ok(format!("{}/commits/{}", base, commit));
        }

        // Handle file/path view
        if let Some(path) = &self.path {
            let mut url = format!("{}/browse/{}", base, path);
            if let Some(branch) = &self.branch {
                url = format!("{}?at=refs/heads/{}", url, branch);
            }
            return Ok(url);
        }

        // Handle branch view
        if let Some(branch) = &self.branch {
            return Ok(format!("{}/browse?at=refs/heads/{}", base, branch));
        }

        // Default: repository browse page
        Ok(format!("{}/browse", base))
    }
}
