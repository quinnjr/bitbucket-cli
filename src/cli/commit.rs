use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use tabled::{Table, Tabled};

use super::{output_json, parse_repo, print_json};
use crate::api::BitbucketClient;
use crate::cli::pagination::effective_limit;
use crate::models::CommitAuthor;

/// Width of the truncated commit-message column in list tables.
const MSG_WIDTH: usize = 50;

#[derive(Subcommand)]
pub enum CommitCommands {
    /// List commits in a repository
    List {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Branch, tag, or commit hash to list history from (default: the repository's main branch)
        #[arg(short = 'r', long = "ref", value_name = "REF")]
        git_ref: Option<String>,

        /// Number of results per page (max 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,

        /// Page number to fetch
        #[arg(long)]
        page: Option<u32>,
    },

    /// View a single commit
    View {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Commit hash (full or abbreviated)
        hash: String,
    },

    /// Show the raw diff for a commit or revision spec
    Diff {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// A commit (diffed against its first parent) or a source..destination revision spec
        spec: String,
    },
}

#[derive(Tabled)]
struct CommitRow {
    #[tabled(rename = "HASH")]
    hash: String,
    #[tabled(rename = "AUTHOR")]
    author: String,
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "MESSAGE")]
    message: String,
}

impl CommitCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            CommitCommands::List {
                repo,
                git_ref,
                limit,
                page,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let commits = client
                    .list_commits(
                        &workspace,
                        &repo_slug,
                        git_ref.as_deref(),
                        page,
                        Some(effective_limit(limit)),
                    )
                    .await?;

                if output_json() {
                    return print_json(&commits.values);
                }

                if commits.values.is_empty() {
                    println!("No commits found in {}", repo);
                    return Ok(());
                }

                let rows: Vec<CommitRow> = commits
                    .values
                    .iter()
                    .map(|c| CommitRow {
                        hash: short_hash(&c.hash),
                        author: author_label(c.author.as_ref()),
                        date: c
                            .date
                            .map(|d| d.format("%Y-%m-%d").to_string())
                            .unwrap_or_default(),
                        message: first_line_truncated(
                            c.message.as_deref().unwrap_or(""),
                            MSG_WIDTH,
                        ),
                    })
                    .collect();

                println!("{}", Table::new(rows));

                if commits.next.is_some() {
                    println!(
                        "\n{} More commits available. Use --limit or --page to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            CommitCommands::View { repo, hash } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let commit = client.get_commit(&workspace, &repo_slug, &hash).await?;

                if output_json() {
                    return print_json(&commit);
                }

                println!("{}", commit.hash.bold());
                println!("{}", "─".repeat(50));
                println!(
                    "{} {}",
                    "Author:".dimmed(),
                    author_label(commit.author.as_ref())
                );

                if let Some(date) = commit.date {
                    println!("{} {}", "Date:".dimmed(), date.format("%Y-%m-%d %H:%M:%S"));
                }

                if let Some(message) = &commit.message {
                    println!();
                    println!("{}", message.trim_end());
                }

                Ok(())
            }

            CommitCommands::Diff { repo, spec } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let diff = client.get_diff(&workspace, &repo_slug, &spec).await?;

                print!("{}", diff);

                Ok(())
            }
        }
    }
}

/// Abbreviate a commit hash to the conventional 7 characters.
pub(crate) fn short_hash(hash: &str) -> String {
    hash.chars().take(7).collect()
}

/// First line of a commit message, truncated to at most `max` characters.
pub(crate) fn first_line_truncated(message: &str, max: usize) -> String {
    message
        .lines()
        .next()
        .unwrap_or("")
        .chars()
        .take(max)
        .collect()
}

/// Human-readable author: the account display name when the author maps to a
/// Bitbucket user, otherwise the raw `Name <email>` signature, otherwise "-".
pub(crate) fn author_label(author: Option<&CommitAuthor>) -> String {
    author
        .and_then(|a| {
            a.user
                .as_ref()
                .map(|u| u.display_name.clone())
                .or_else(|| a.raw.clone())
        })
        .unwrap_or_else(|| "-".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_hash_abbreviates_to_seven_chars() {
        assert_eq!(short_hash("deadbeefcafe1234"), "deadbee");
    }

    #[test]
    fn short_hash_keeps_already_short_input() {
        assert_eq!(short_hash("abc12"), "abc12");
    }

    #[test]
    fn first_line_truncated_takes_first_line() {
        assert_eq!(
            first_line_truncated("feat: add login\n\nLong body here", 60),
            "feat: add login"
        );
    }

    #[test]
    fn first_line_truncated_caps_length() {
        assert_eq!(first_line_truncated("abcdefghij", 4), "abcd");
    }

    #[test]
    fn first_line_truncated_handles_empty_message() {
        assert_eq!(first_line_truncated("", 60), "");
    }

    #[test]
    fn author_label_prefers_user_display_name_over_raw() {
        let author = CommitAuthor {
            raw: Some("Jo <jo@example.com>".to_string()),
            user: Some(crate::models::User {
                uuid: "{user-uuid}".to_string(),
                username: None,
                display_name: "Jo Bloggs".to_string(),
                account_id: None,
                user_type: "user".to_string(),
                links: None,
            }),
        };
        assert_eq!(author_label(Some(&author)), "Jo Bloggs");
    }

    #[test]
    fn author_label_falls_back_to_raw_then_dash() {
        let raw_only = CommitAuthor {
            raw: Some("Jo <jo@example.com>".to_string()),
            user: None,
        };
        assert_eq!(author_label(Some(&raw_only)), "Jo <jo@example.com>");

        let empty = CommitAuthor {
            raw: None,
            user: None,
        };
        assert_eq!(author_label(Some(&empty)), "-");
        assert_eq!(author_label(None), "-");
    }
}
