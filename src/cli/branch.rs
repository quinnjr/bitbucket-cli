use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use tabled::{Table, Tabled};

use super::commit::{author_label, first_line_truncated, short_hash};
use super::{confirm_or_abort, output_json, parse_repo, print_json};
use crate::api::BitbucketClient;
use crate::cli::pagination::effective_limit;
use crate::models::BranchDetail;

/// Width of the truncated commit-message column in list tables.
const MSG_WIDTH: usize = 50;

#[derive(Subcommand)]
pub enum BranchCommands {
    /// List branches in a repository
    List {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Filter expression, e.g. 'name ~ "feat"'
        #[arg(short = 'q', long)]
        query: Option<String>,

        /// Sort field, prefix with '-' for descending, e.g. -target.date
        #[arg(long)]
        sort: Option<String>,

        /// Number of results per page (max 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,

        /// Page number to fetch
        #[arg(long)]
        page: Option<u32>,
    },

    /// Create a branch
    Create {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Name of the branch to create
        name: String,

        /// Commit hash or existing branch name to branch from (a 7-40 char hex value is treated as a commit hash)
        #[arg(long, value_name = "HASH_OR_BRANCH")]
        from: String,
    },

    /// View a single branch
    View {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Branch name
        name: String,
    },

    /// Delete a branch
    Delete {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Branch name
        name: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Tabled)]
struct BranchRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "COMMIT")]
    commit: String,
    #[tabled(rename = "AUTHOR")]
    author: String,
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "MESSAGE")]
    message: String,
}

impl From<&BranchDetail> for BranchRow {
    fn from(branch: &BranchDetail) -> Self {
        let target = branch.target.as_ref();
        BranchRow {
            name: branch.name.clone(),
            commit: target.map(|t| short_hash(&t.hash)).unwrap_or_default(),
            author: target
                .map(|t| author_label(t.author.as_ref()))
                .unwrap_or_else(|| "-".to_string()),
            date: target
                .and_then(|t| t.date)
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
            message: target
                .and_then(|t| t.message.as_deref())
                .map(|m| first_line_truncated(m, MSG_WIDTH))
                .unwrap_or_default(),
        }
    }
}

impl BranchCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            BranchCommands::List {
                repo,
                query,
                sort,
                limit,
                page,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let branches = client
                    .list_branches_filtered(
                        &workspace,
                        &repo_slug,
                        query.as_deref(),
                        sort.as_deref(),
                        page,
                        Some(effective_limit(limit)),
                    )
                    .await?;

                if output_json() {
                    return print_json(&branches.values);
                }

                if branches.values.is_empty() {
                    println!("No branches found in {}", repo);
                    return Ok(());
                }

                let rows: Vec<BranchRow> = branches.values.iter().map(BranchRow::from).collect();
                println!("{}", Table::new(rows));

                if branches.next.is_some() {
                    println!(
                        "\n{} More branches available. Use --limit or --page to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            BranchCommands::Create { repo, name, from } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                // --from accepts either a commit hash or a branch name; the
                // API only takes a hash, so resolve branch names first.
                let target_hash = if looks_like_hash(&from) {
                    from
                } else {
                    let source = client
                        .get_branch(&workspace, &repo_slug, &from)
                        .await
                        .with_context(|| format!("Failed to resolve source branch '{}'", from))?;
                    source
                        .target
                        .map(|t| t.hash)
                        .with_context(|| format!("Branch '{}' has no target commit", from))?
                };

                let branch = client
                    .create_branch(&workspace, &repo_slug, &name, &target_hash)
                    .await?;

                if output_json() {
                    return print_json(&branch);
                }

                println!(
                    "{} Created branch {} at {}",
                    "✓".green(),
                    branch.name.cyan(),
                    short_hash(&target_hash)
                );

                Ok(())
            }

            BranchCommands::View { repo, name } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let branch = client.get_branch(&workspace, &repo_slug, &name).await?;

                if output_json() {
                    return print_json(&branch);
                }

                println!("{}", branch.name.bold());
                println!("{}", "─".repeat(50));

                if let Some(target) = &branch.target {
                    println!("{} {}", "Commit:".dimmed(), target.hash);
                    println!(
                        "{} {}",
                        "Author:".dimmed(),
                        author_label(target.author.as_ref())
                    );

                    if let Some(date) = target.date {
                        println!("{} {}", "Date:".dimmed(), date.format("%Y-%m-%d %H:%M:%S"));
                    }

                    if let Some(message) = &target.message {
                        println!();
                        println!("{}", message.trim_end());
                    }
                }

                Ok(())
            }

            BranchCommands::Delete { repo, name, yes } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                if !yes
                    && !confirm_or_abort(format!("Delete branch {} from {}?", name.red(), repo))?
                {
                    return Ok(());
                }

                let client = BitbucketClient::from_stored().await?;
                client.delete_branch(&workspace, &repo_slug, &name).await?;

                if output_json() {
                    return print_json(&serde_json::json!({ "ok": true }));
                }

                println!("{} Deleted branch {}", "✓".green(), name);

                Ok(())
            }
        }
    }
}

/// Whether `s` is plausibly an abbreviated or full commit hash:
/// 7 to 40 hexadecimal characters.
fn looks_like_hash(s: &str) -> bool {
    (7..=40).contains(&s.len()) && s.chars().all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::looks_like_hash;

    #[test]
    fn looks_like_hash_accepts_abbreviated_and_full_hashes() {
        assert!(looks_like_hash("deadbee"));
        assert!(looks_like_hash("DEADBEE1"));
        assert!(looks_like_hash(&"a".repeat(40)));
    }

    #[test]
    fn looks_like_hash_rejects_wrong_lengths() {
        assert!(!looks_like_hash("abc123")); // 6 chars: too short
        assert!(!looks_like_hash(&"a".repeat(41))); // too long
        assert!(!looks_like_hash(""));
    }

    #[test]
    fn looks_like_hash_rejects_non_hex_names() {
        assert!(!looks_like_hash("develop"));
        assert!(!looks_like_hash("feature/x"));
        assert!(!looks_like_hash("release-1.0"));
    }
}
