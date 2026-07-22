use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use tabled::{Table, Tabled};

use super::{confirm_or_abort, output_json, parse_repo, print_json};
use crate::api::BitbucketClient;

#[derive(Subcommand)]
pub enum BranchRestrictionCommands {
    /// List branch restriction rules on a repository
    List {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Number of results per page (max 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// Create a branch restriction rule
    Create {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Rule kind (e.g. push, force, delete, require_approvals_to_merge)
        #[arg(long)]
        kind: String,

        /// Branch pattern the rule applies to (glob, e.g. main or release/*)
        #[arg(long)]
        pattern: String,

        /// Numeric parameter for kinds that need one (e.g. required approval count)
        #[arg(long, value_name = "N")]
        value: Option<u64>,
    },

    /// Delete a branch restriction rule
    Delete {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Rule id (shown by `branch-restriction list`)
        id: u64,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Tabled)]
struct RestrictionRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "KIND")]
    kind: String,
    #[tabled(rename = "PATTERN")]
    pattern: String,
    #[tabled(rename = "VALUE")]
    value: String,
}

impl BranchRestrictionCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            BranchRestrictionCommands::List { repo, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let restrictions = client
                    .list_branch_restrictions(&workspace, &repo_slug, Some(limit.clamp(1, 100)))
                    .await?;

                if output_json() {
                    return print_json(&restrictions.values);
                }

                if restrictions.values.is_empty() {
                    println!("No branch restrictions found in {}", repo);
                    return Ok(());
                }

                let rows: Vec<RestrictionRow> = restrictions
                    .values
                    .iter()
                    .map(|r| RestrictionRow {
                        id: r.id.map(|id| id.to_string()).unwrap_or_default(),
                        kind: r.kind.clone(),
                        pattern: r.pattern.clone().unwrap_or_default(),
                        value: r
                            .value
                            .map(|v| v.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    })
                    .collect();

                println!("{}", Table::new(rows));

                if restrictions.next.is_some() {
                    println!(
                        "\n{} More branch restrictions available. Use --limit to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            BranchRestrictionCommands::Create {
                repo,
                kind,
                pattern,
                value,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let restriction = client
                    .create_branch_restriction(&workspace, &repo_slug, &kind, &pattern, value)
                    .await?;

                if output_json() {
                    return print_json(&restriction);
                }

                let id_label = restriction
                    .id
                    .map(|id| format!(" (id {})", id))
                    .unwrap_or_default();

                println!(
                    "{} Created {} restriction on {} for pattern {}{}",
                    "✓".green(),
                    restriction.kind.cyan(),
                    repo,
                    restriction.pattern.as_deref().unwrap_or(&pattern).cyan(),
                    id_label
                );

                Ok(())
            }

            BranchRestrictionCommands::Delete { repo, id, yes } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                if !yes
                    && !confirm_or_abort(format!(
                        "Delete branch restriction {} from {}?",
                        id.to_string().red(),
                        repo
                    ))?
                {
                    return Ok(());
                }

                let client = BitbucketClient::from_stored().await?;
                client
                    .delete_branch_restriction(&workspace, &repo_slug, id)
                    .await?;

                if output_json() {
                    return print_json(&serde_json::json!({ "ok": true }));
                }

                println!("{} Deleted branch restriction {}", "✓".green(), id);

                Ok(())
            }
        }
    }
}
