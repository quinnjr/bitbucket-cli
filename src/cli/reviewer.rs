use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use tabled::{Table, Tabled};

use super::{confirm_or_abort, output_json, parse_repo, print_json};
use crate::api::BitbucketClient;

#[derive(Subcommand)]
pub enum ReviewerCommands {
    /// List a repository's default reviewers
    List {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Number of results per page (max 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// Add a user to the default reviewers
    Add {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Username, account ID, or brace-wrapped UUID of the user
        username: String,
    },

    /// Remove a user from the default reviewers
    Remove {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Username, account ID, or brace-wrapped UUID of the user
        username: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Tabled)]
struct ReviewerRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "USERNAME")]
    username: String,
    #[tabled(rename = "UUID")]
    uuid: String,
}

impl ReviewerCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            ReviewerCommands::List { repo, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let reviewers = client
                    .list_default_reviewers(&workspace, &repo_slug, Some(limit.clamp(1, 100)))
                    .await?;

                if output_json() {
                    return print_json(&reviewers.values);
                }

                if reviewers.values.is_empty() {
                    println!("No default reviewers found in {}", repo);
                    return Ok(());
                }

                let rows: Vec<ReviewerRow> = reviewers
                    .values
                    .iter()
                    .map(|u| ReviewerRow {
                        name: u.display_name.clone(),
                        username: u.username.clone().unwrap_or_else(|| "-".to_string()),
                        uuid: u.uuid.clone(),
                    })
                    .collect();

                println!("{}", Table::new(rows));

                if reviewers.next.is_some() {
                    println!(
                        "\n{} More default reviewers available. Use --limit to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            ReviewerCommands::Add { repo, username } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let user = client
                    .add_default_reviewer(&workspace, &repo_slug, &username)
                    .await?;

                if output_json() {
                    return print_json(&user);
                }

                println!(
                    "{} Added {} as a default reviewer on {}",
                    "✓".green(),
                    user.display_name.cyan(),
                    repo
                );

                Ok(())
            }

            ReviewerCommands::Remove {
                repo,
                username,
                yes,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                if !yes
                    && !confirm_or_abort(format!(
                        "Remove {} from the default reviewers of {}?",
                        username.red(),
                        repo
                    ))?
                {
                    return Ok(());
                }

                let client = BitbucketClient::from_stored().await?;
                client
                    .remove_default_reviewer(&workspace, &repo_slug, &username)
                    .await?;

                if output_json() {
                    return print_json(&serde_json::json!({ "ok": true }));
                }

                println!("{} Removed default reviewer {}", "✓".green(), username);

                Ok(())
            }
        }
    }
}
