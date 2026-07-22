use bitbucket_cli::{cli, tui};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use cli::{Cli, Commands};

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    cli::set_workspace_override(cli.workspace.clone());
    cli::set_output_format(cli.output);

    let result = match cli.command {
        Commands::Auth { command } => command.run().await,
        Commands::Repo { command } => command.run().await,
        Commands::Pr { command } => command.run().await,
        Commands::Issue { command } => command.run().await,
        Commands::Pipeline { command } => command.run().await,
        Commands::Workspace { command } => command.run().await,
        Commands::User { command } => command.run().await,
        Commands::Tui => tui::run_tui(cli.workspace).await,
    };

    if let Err(e) = result {
        // Error text can include remote-controlled strings (API messages,
        // OAuth callback parameters); strip control characters so they
        // cannot smuggle terminal escape sequences.
        let message: String = e
            .to_string()
            .chars()
            .filter(|c| !c.is_control() || *c == '\n')
            .collect();
        eprintln!("{} {}", "Error:".red().bold(), message);
        std::process::exit(1);
    }

    Ok(())
}
