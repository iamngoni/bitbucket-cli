//
//  bitbucket-cli
//  cli/completion.rs
//
//  Created by Ngonidzashe Mangudya on 2026/01/12.
//  Copyright (c) 2025 IAMNGONI. All rights reserved.
//

//! Shell completion commands

use anyhow::Result;
use clap::{Args, CommandFactory, Subcommand};
use clap_complete::{generate, Shell};

use super::{Cli, GlobalOptions};

/// Generate shell completion scripts
#[derive(Args, Debug)]
pub struct CompletionCommand {
    #[command(subcommand)]
    pub command: CompletionSubcommand,
}

#[derive(Subcommand, Debug)]
pub enum CompletionSubcommand {
    /// Generate Bash completions
    Bash,

    /// Generate Zsh completions
    Zsh,

    /// Generate Fish completions
    Fish,

    /// Generate PowerShell completions
    Powershell,
}

impl CompletionCommand {
    pub async fn run(&self, _global: &GlobalOptions) -> Result<()> {
        let mut cmd = Cli::command();
        let name = "bb";

        match &self.command {
            CompletionSubcommand::Bash => {
                generate(Shell::Bash, &mut cmd, name, &mut std::io::stdout());
            }
            CompletionSubcommand::Zsh => {
                generate(Shell::Zsh, &mut cmd, name, &mut std::io::stdout());
            }
            CompletionSubcommand::Fish => {
                generate(Shell::Fish, &mut cmd, name, &mut std::io::stdout());
            }
            CompletionSubcommand::Powershell => {
                generate(Shell::PowerShell, &mut cmd, name, &mut std::io::stdout());
            }
        }

        Ok(())
    }
}
