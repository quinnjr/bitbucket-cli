use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use tabled::{Table, Tabled};

use crate::api::{BitbucketClient, download_url, upload_name_for};
use crate::models::CreateRepositoryRequest;

#[derive(Subcommand)]
pub enum RepoCommands {
    /// List repositories in a workspace
    List {
        /// Workspace slug
        workspace: String,

        /// Number of results per page
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// View repository details
    View {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Open in browser
        #[arg(short, long)]
        web: bool,
    },

    /// Clone a repository
    Clone {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Directory to clone into
        #[arg(short, long)]
        dir: Option<String>,
    },

    /// Create a new repository
    Create {
        /// Workspace slug
        workspace: String,

        /// Repository name
        name: String,

        /// Repository description
        #[arg(short, long)]
        description: Option<String>,

        /// Make repository public
        #[arg(long)]
        public: bool,

        /// Project key to add repository to
        #[arg(short, long)]
        project: Option<String>,

        /// Fork policy: allow_forks, no_public_forks, no_forks (default: allow_forks when --public, no_public_forks otherwise)
        #[arg(long)]
        fork_policy: Option<String>,
    },

    /// Fork a repository
    Fork {
        /// Repository to fork in format workspace/repo-slug
        repo: String,

        /// Workspace to fork into
        #[arg(short, long)]
        workspace: Option<String>,

        /// New repository name
        #[arg(short, long)]
        name: Option<String>,
    },

    /// Delete a repository
    Delete {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Manage repository downloads (uploaded file artifacts)
    Download {
        #[command(subcommand)]
        command: DownloadCommands,
    },
}

#[derive(Subcommand)]
pub enum DownloadCommands {
    /// Upload one or more files to the repository's downloads area
    Upload {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// File(s) to upload. Uploading a file whose name already exists replaces it.
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },

    /// List artifacts in the repository's downloads area
    List {
        /// Repository in format workspace/repo-slug
        repo: String,
    },

    /// Delete an artifact from the repository's downloads area
    Delete {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Name of the artifact to delete
        name: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Tabled)]
struct RepoRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "DESCRIPTION")]
    description: String,
    #[tabled(rename = "PRIVATE")]
    private: String,
    #[tabled(rename = "UPDATED")]
    updated: String,
}

impl RepoCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            RepoCommands::List { workspace, limit } => {
                let client = BitbucketClient::from_stored().await?;
                let repos = client
                    .list_repositories(&workspace, None, Some(limit))
                    .await?;

                if repos.values.is_empty() {
                    println!("No repositories found in workspace '{}'", workspace);
                    return Ok(());
                }

                let rows: Vec<RepoRow> = repos
                    .values
                    .iter()
                    .map(|r| RepoRow {
                        name: r.full_name.clone(),
                        description: r
                            .description
                            .clone()
                            .unwrap_or_default()
                            .chars()
                            .take(40)
                            .collect::<String>(),
                        private: if r.is_private.unwrap_or(false) {
                            "Yes"
                        } else {
                            "No"
                        }
                        .to_string(),
                        updated: r
                            .updated_on
                            .map(|d| d.format("%Y-%m-%d").to_string())
                            .unwrap_or_default(),
                    })
                    .collect();

                let table = Table::new(rows).to_string();
                println!("{}", table);

                if repos.next.is_some() {
                    println!(
                        "\n{} More repositories available. Use --limit to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            RepoCommands::View { repo, web } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let repository = client.get_repository(&workspace, &repo_slug).await?;

                if web {
                    if let Some(links) = &repository.links {
                        if let Some(html) = &links.html {
                            open::that(&html.href)?;
                            println!("Opened {} in browser", html.href.cyan());
                            return Ok(());
                        }
                    }
                    anyhow::bail!("Could not find repository URL");
                }

                println!("{}", repository.full_name.bold());
                println!("{}", "─".repeat(50));

                if let Some(desc) = &repository.description {
                    if !desc.is_empty() {
                        println!("{}", desc);
                        println!();
                    }
                }

                println!(
                    "{} {}",
                    "Private:".dimmed(),
                    if repository.is_private.unwrap_or(false) {
                        "Yes"
                    } else {
                        "No"
                    }
                );
                println!(
                    "{} {}",
                    "SCM:".dimmed(),
                    repository.scm.as_deref().unwrap_or("unknown")
                );

                if let Some(lang) = &repository.language {
                    if !lang.is_empty() {
                        println!("{} {}", "Language:".dimmed(), lang);
                    }
                }

                if let Some(branch) = &repository.mainbranch {
                    println!("{} {}", "Main branch:".dimmed(), branch.name);
                }

                if let Some(size) = repository.size {
                    let size_mb = size as f64 / (1024.0 * 1024.0);
                    println!("{} {:.2} MB", "Size:".dimmed(), size_mb);
                }

                if let Some(created) = repository.created_on {
                    println!("{} {}", "Created:".dimmed(), created.format("%Y-%m-%d"));
                }

                if let Some(updated) = repository.updated_on {
                    println!("{} {}", "Updated:".dimmed(), updated.format("%Y-%m-%d"));
                }

                if let Some(links) = &repository.links {
                    println!();
                    if let Some(html) = &links.html {
                        println!("{} {}", "Web:".dimmed(), html.href.cyan());
                    }
                    if let Some(clone_links) = &links.clone {
                        for link in clone_links {
                            println!("{} {} ({})", "Clone:".dimmed(), link.href, link.name);
                        }
                    }
                }

                Ok(())
            }

            RepoCommands::Clone { repo, dir } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let repository = client.get_repository(&workspace, &repo_slug).await?;

                let clone_url = repository
                    .links
                    .as_ref()
                    .and_then(|l| l.clone.as_ref())
                    .and_then(|links| links.iter().find(|l| l.name == "ssh" || l.name == "https"))
                    .map(|l| &l.href)
                    .context("Could not find clone URL")?;

                let target_dir = dir.unwrap_or_else(|| repo_slug.clone());

                println!("Cloning {} into {}...", repo.cyan(), target_dir);

                let status = std::process::Command::new("git")
                    .args(["clone", clone_url, &target_dir])
                    .status()
                    .context("Failed to run git clone")?;

                if status.success() {
                    println!("{} Successfully cloned repository", "✓".green());
                } else {
                    anyhow::bail!("git clone failed");
                }

                Ok(())
            }

            RepoCommands::Create {
                workspace,
                name,
                description,
                public,
                project,
                fork_policy,
            } => {
                let client = BitbucketClient::from_stored().await?;

                let slug = name.to_lowercase().replace(' ', "-");

                let resolved_fork_policy = fork_policy.unwrap_or_else(|| {
                    if public {
                        "allow_forks".to_string()
                    } else {
                        "no_public_forks".to_string()
                    }
                });

                let request = CreateRepositoryRequest {
                    scm: "git".to_string(),
                    name: Some(name.clone()),
                    description,
                    is_private: Some(!public),
                    project: project.map(|key| crate::models::ProjectKey { key }),
                    fork_policy: Some(resolved_fork_policy),
                    ..Default::default()
                };

                let repository = client
                    .create_repository(&workspace, &slug, &request)
                    .await?;

                println!(
                    "{} Created repository {}",
                    "✓".green(),
                    repository.full_name.cyan()
                );

                if let Some(links) = &repository.links {
                    if let Some(html) = &links.html {
                        println!("{} {}", "URL:".dimmed(), html.href);
                    }
                }

                Ok(())
            }

            RepoCommands::Fork {
                repo,
                workspace,
                name,
            } => {
                let (src_workspace, src_repo) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let forked = client
                    .fork_repository(
                        &src_workspace,
                        &src_repo,
                        workspace.as_deref(),
                        name.as_deref(),
                    )
                    .await?;

                println!("{} Forked to {}", "✓".green(), forked.full_name.cyan());

                Ok(())
            }

            RepoCommands::Delete { repo, yes } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                if !yes {
                    use dialoguer::Confirm;
                    let confirmed = Confirm::new()
                        .with_prompt(format!(
                            "Are you sure you want to delete {}? This cannot be undone!",
                            repo.red()
                        ))
                        .default(false)
                        .interact()?;

                    if !confirmed {
                        println!("Aborted");
                        return Ok(());
                    }
                }

                let client = BitbucketClient::from_stored().await?;
                client.delete_repository(&workspace, &repo_slug).await?;

                println!("{} Deleted repository {}", "✓".green(), repo);

                Ok(())
            }

            RepoCommands::Download { command } => command.run().await,
        }
    }
}

#[derive(Tabled)]
struct DownloadRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "SIZE")]
    size: String,
    #[tabled(rename = "DOWNLOADS")]
    downloads: String,
}

impl DownloadCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            DownloadCommands::Upload { repo, files } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                // Resolve each path to (upload-name, path) up front so a bad path
                // fails before we open a network connection.
                let uploads: Vec<(String, PathBuf)> = files
                    .iter()
                    .map(|p| Ok((upload_name_for(p)?, p.clone())))
                    .collect::<Result<_>>()?;

                let client = BitbucketClient::from_stored().await?;
                client
                    .upload_downloads(&workspace, &repo_slug, &uploads)
                    .await?;

                for (name, _) in &uploads {
                    println!(
                        "{} Uploaded {}",
                        "✓".green(),
                        download_url(&workspace, &repo_slug, name).cyan()
                    );
                }

                Ok(())
            }

            DownloadCommands::List { repo } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let downloads = client.list_downloads(&workspace, &repo_slug).await?;

                if downloads.values.is_empty() {
                    println!("No downloads found in {}", repo);
                    return Ok(());
                }

                let rows: Vec<DownloadRow> = downloads
                    .values
                    .iter()
                    .map(|d| DownloadRow {
                        name: d.name.clone(),
                        size: d.size.map(format_size).unwrap_or_else(|| "-".to_string()),
                        downloads: d
                            .downloads
                            .map(|n| n.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                    })
                    .collect();

                println!("{}", Table::new(rows));

                Ok(())
            }

            DownloadCommands::Delete { repo, name, yes } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                if !yes {
                    use dialoguer::Confirm;
                    let confirmed = Confirm::new()
                        .with_prompt(format!("Delete download {} from {}?", name.red(), repo))
                        .default(false)
                        .interact()?;

                    if !confirmed {
                        println!("Aborted");
                        return Ok(());
                    }
                }

                let client = BitbucketClient::from_stored().await?;
                client
                    .delete_download(&workspace, &repo_slug, &name)
                    .await?;

                println!("{} Deleted download {}", "✓".green(), name);

                Ok(())
            }
        }
    }
}

/// Format a byte count into a short human-readable string.
fn format_size(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit = 0;
    while size >= 1024.0 && unit < UNITS.len() - 1 {
        size /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{} {}", bytes, UNITS[unit])
    } else {
        format!("{:.1} {}", size, UNITS[unit])
    }
}

fn parse_repo(repo: &str) -> Result<(String, String)> {
    let parts: Vec<&str> = repo.split('/').collect();
    if parts.len() != 2 {
        anyhow::bail!(
            "Invalid repository format. Expected 'workspace/repo-slug', got '{}'",
            repo
        );
    }
    Ok((parts[0].to_string(), parts[1].to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_repo_splits_workspace_and_slug() {
        let (ws, slug) = parse_repo("acme/widgets").unwrap();
        assert_eq!(ws, "acme");
        assert_eq!(slug, "widgets");
    }

    #[test]
    fn parse_repo_rejects_missing_slug() {
        assert!(parse_repo("acme").is_err());
        assert!(parse_repo("acme/widgets/extra").is_err());
    }

    #[test]
    fn format_size_uses_bytes_below_1k() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1023), "1023 B");
    }

    #[test]
    fn format_size_scales_to_larger_units() {
        assert_eq!(format_size(1024), "1.0 KB");
        assert_eq!(format_size(1536), "1.5 KB");
        assert_eq!(format_size(1024 * 1024), "1.0 MB");
        assert_eq!(format_size(5 * 1024 * 1024 * 1024), "5.0 GB");
    }
}
