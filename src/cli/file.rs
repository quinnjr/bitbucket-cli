use std::io::Write;

use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use tabled::{Table, Tabled};

use super::commit::{author_label, first_line_truncated, short_hash};
use super::{output_json, parse_repo, print_json};
use crate::api::BitbucketClient;
use crate::cli::pagination::{effective_limit, format_size};

/// Width of the truncated commit-message column in list tables.
const MSG_WIDTH: usize = 50;

#[derive(Subcommand)]
pub enum FileCommands {
    /// List files and directories in a repository
    Ls {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Directory path to list (default: repository root)
        #[arg(default_value = "")]
        path: String,

        /// Branch, tag, or commit hash (default: the repository's main branch)
        #[arg(short = 'r', long = "ref", value_name = "REF")]
        git_ref: Option<String>,

        /// Number of results per page (max 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// Print the raw contents of a file
    Cat {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// File path within the repository
        path: String,

        /// Branch, tag, or commit hash (default: the repository's main branch)
        #[arg(short = 'r', long = "ref", value_name = "REF")]
        git_ref: Option<String>,
    },

    /// List the commits that touched a file
    History {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// File path within the repository
        path: String,

        /// Branch, tag, or commit hash to start from (default: the repository's main branch)
        #[arg(short = 'r', long = "ref", value_name = "REF")]
        git_ref: Option<String>,

        /// Number of results per page (max 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },
}

#[derive(Tabled)]
struct SourceRow {
    #[tabled(rename = "TYPE")]
    entry_type: String,
    #[tabled(rename = "SIZE")]
    size: String,
    #[tabled(rename = "PATH")]
    path: String,
}

#[derive(Tabled)]
struct HistoryRow {
    #[tabled(rename = "COMMIT")]
    commit: String,
    #[tabled(rename = "AUTHOR")]
    author: String,
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "MESSAGE")]
    message: String,
}

impl FileCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            FileCommands::Ls {
                repo,
                path,
                git_ref,
                limit,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let git_ref = resolve_ref(&client, &workspace, &repo_slug, git_ref).await?;

                let entries = client
                    .list_source(
                        &workspace,
                        &repo_slug,
                        &git_ref,
                        &path,
                        None,
                        Some(effective_limit(limit)),
                    )
                    .await?;

                if output_json() {
                    return print_json(&entries.values);
                }

                if entries.values.is_empty() {
                    println!("No entries found at '{}' in {} ({})", path, repo, git_ref);
                    return Ok(());
                }

                let rows: Vec<SourceRow> = entries
                    .values
                    .iter()
                    .map(|e| SourceRow {
                        entry_type: entry_type_label(&e.entry_type).to_string(),
                        size: e.size.map(format_size).unwrap_or_else(|| "-".to_string()),
                        path: e.path.clone(),
                    })
                    .collect();

                println!("{}", Table::new(rows));

                if entries.next.is_some() {
                    println!(
                        "\n{} More entries available. Use --limit to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            FileCommands::Cat {
                repo,
                path,
                git_ref,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let git_ref = resolve_ref(&client, &workspace, &repo_slug, git_ref).await?;

                let content = client
                    .get_file_raw(&workspace, &repo_slug, &git_ref, &path)
                    .await?;

                // Emit the file verbatim: no added trailing newline, and no
                // table/color decoration, so output is pipe-friendly.
                print!("{}", content);
                std::io::stdout().flush()?;

                Ok(())
            }

            FileCommands::History {
                repo,
                path,
                git_ref,
                limit,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let git_ref = resolve_ref(&client, &workspace, &repo_slug, git_ref).await?;

                let history = client
                    .get_file_history(
                        &workspace,
                        &repo_slug,
                        &git_ref,
                        &path,
                        Some(effective_limit(limit)),
                    )
                    .await?;

                if output_json() {
                    return print_json(&history.values);
                }

                if history.values.is_empty() {
                    println!("No history found for '{}' in {} ({})", path, repo, git_ref);
                    return Ok(());
                }

                let rows: Vec<HistoryRow> = history
                    .values
                    .iter()
                    .map(|entry| {
                        let commit = entry.commit.as_ref();
                        HistoryRow {
                            commit: commit.map(|c| short_hash(&c.hash)).unwrap_or_default(),
                            author: commit
                                .map(|c| author_label(c.author.as_ref()))
                                .unwrap_or_else(|| "-".to_string()),
                            date: commit
                                .and_then(|c| c.date)
                                .map(|d| d.format("%Y-%m-%d").to_string())
                                .unwrap_or_default(),
                            message: commit
                                .and_then(|c| c.message.as_deref())
                                .map(|m| first_line_truncated(m, MSG_WIDTH))
                                .unwrap_or_default(),
                        }
                    })
                    .collect();

                println!("{}", Table::new(rows));

                if history.next.is_some() {
                    println!(
                        "\n{} More history available. Use --limit to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }
        }
    }
}

/// Resolve the ref to browse: an explicit --ref wins; otherwise the
/// repository's configured main branch, falling back to "main".
async fn resolve_ref(
    client: &BitbucketClient,
    workspace: &str,
    repo_slug: &str,
    git_ref: Option<String>,
) -> Result<String> {
    if let Some(r) = git_ref {
        return Ok(r);
    }

    let repository = client.get_repository(workspace, repo_slug).await?;
    Ok(repository
        .mainbranch
        .map(|b| b.name)
        .unwrap_or_else(|| "main".to_string()))
}

/// Map Bitbucket's source entry types to short labels for the TYPE column.
fn entry_type_label(entry_type: &str) -> &str {
    match entry_type {
        "commit_file" => "file",
        "commit_directory" => "dir",
        other => other,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_type_label_maps_known_types() {
        assert_eq!(entry_type_label("commit_file"), "file");
        assert_eq!(entry_type_label("commit_directory"), "dir");
    }

    #[test]
    fn entry_type_label_passes_through_unknown_types() {
        assert_eq!(entry_type_label("commit_submodule"), "commit_submodule");
    }
}
