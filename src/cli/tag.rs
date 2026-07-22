use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use tabled::{Table, Tabled};

use super::commit::{author_label, first_line_truncated, short_hash};
use super::{confirm_or_abort, output_json, parse_repo, print_json};
use crate::api::BitbucketClient;
use crate::cli::pagination::effective_limit;
use crate::models::TagDetail;

/// Width of the truncated commit-message column in list tables.
const MSG_WIDTH: usize = 50;

#[derive(Subcommand)]
pub enum TagCommands {
    /// List tags in a repository
    List {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Filter expression, e.g. 'name ~ "v1"'
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

    /// Create a tag
    Create {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Name of the tag to create
        name: String,

        /// Commit hash the tag points at
        #[arg(long, value_name = "HASH")]
        from: String,

        /// Annotation message (creates an annotated tag)
        #[arg(short = 'm', long)]
        message: Option<String>,
    },

    /// View a single tag
    View {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Tag name
        name: String,
    },

    /// Delete a tag
    Delete {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Tag name
        name: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Tabled)]
struct TagRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "COMMIT")]
    commit: String,
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "MESSAGE")]
    message: String,
}

impl From<&TagDetail> for TagRow {
    fn from(tag: &TagDetail) -> Self {
        let target = tag.target.as_ref();
        TagRow {
            name: tag.name.clone(),
            commit: target.map(|t| short_hash(&t.hash)).unwrap_or_default(),
            // Annotated tags carry their own date; fall back to the target
            // commit's date for lightweight tags.
            date: tag
                .date
                .or_else(|| target.and_then(|t| t.date))
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
            message: tag
                .message
                .as_deref()
                .map(|m| first_line_truncated(m, MSG_WIDTH))
                .unwrap_or_default(),
        }
    }
}

impl TagCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            TagCommands::List {
                repo,
                query,
                sort,
                limit,
                page,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let tags = client
                    .list_tags_filtered(
                        &workspace,
                        &repo_slug,
                        query.as_deref(),
                        sort.as_deref(),
                        page,
                        Some(effective_limit(limit)),
                    )
                    .await?;

                if output_json() {
                    return print_json(&tags.values);
                }

                if tags.values.is_empty() {
                    println!("No tags found in {}", repo);
                    return Ok(());
                }

                let rows: Vec<TagRow> = tags.values.iter().map(TagRow::from).collect();
                println!("{}", Table::new(rows));

                if tags.next.is_some() {
                    println!(
                        "\n{} More tags available. Use --limit or --page to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            TagCommands::Create {
                repo,
                name,
                from,
                message,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let tag = client
                    .create_tag(&workspace, &repo_slug, &name, &from, message.as_deref())
                    .await?;

                if output_json() {
                    return print_json(&tag);
                }

                println!(
                    "{} Created tag {} at {}",
                    "✓".green(),
                    tag.name.cyan(),
                    tag.target
                        .as_ref()
                        .map(|t| short_hash(&t.hash))
                        .unwrap_or_else(|| short_hash(&from))
                );

                Ok(())
            }

            TagCommands::View { repo, name } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let tag = client.get_tag(&workspace, &repo_slug, &name).await?;

                if output_json() {
                    return print_json(&tag);
                }

                println!("{}", tag.name.bold());
                println!("{}", "─".repeat(50));

                if let Some(date) = tag.date {
                    println!(
                        "{} {}",
                        "Tagged:".dimmed(),
                        date.format("%Y-%m-%d %H:%M:%S")
                    );
                }

                if let Some(message) = &tag.message {
                    if !message.trim().is_empty() {
                        println!("{} {}", "Message:".dimmed(), message.trim_end());
                    }
                }

                if let Some(target) = &tag.target {
                    println!();
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

            TagCommands::Delete { repo, name, yes } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                if !yes && !confirm_or_abort(format!("Delete tag {} from {}?", name.red(), repo))? {
                    return Ok(());
                }

                let client = BitbucketClient::from_stored().await?;
                client.delete_tag(&workspace, &repo_slug, &name).await?;

                if output_json() {
                    return print_json(&serde_json::json!({ "ok": true }));
                }

                println!("{} Deleted tag {}", "✓".green(), name);

                Ok(())
            }
        }
    }
}
