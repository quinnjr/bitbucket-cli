use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use tabled::{Table, Tabled};

use super::parse_repo;
use crate::api::BitbucketClient;
use crate::models::{CreateRepositoryRequest, UpdateRepositoryRequest};

#[derive(Subcommand)]
pub enum RepoCommands {
    /// List repositories in a workspace
    List {
        /// Workspace slug (defaults to --workspace or the configured default workspace)
        workspace: Option<String>,

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
        /// Workspace slug, `workspace/name`, or just the repository name
        /// when --workspace or a default workspace supplies the workspace
        #[arg(value_name = "WORKSPACE_OR_NAME")]
        workspace: String,

        /// Repository name (omit when the first argument already names the repository)
        name: Option<String>,

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

    /// Update repository settings
    Update {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// New repository name (also changes the repository slug)
        #[arg(long)]
        name: Option<String>,

        /// New description
        #[arg(short, long)]
        description: Option<String>,

        /// Make repository private
        #[arg(long, conflicts_with = "public")]
        private: bool,

        /// Make repository public
        #[arg(long)]
        public: bool,

        /// Primary language
        #[arg(short, long)]
        language: Option<String>,

        /// Fork policy: allow_forks, no_public_forks, no_forks
        #[arg(long)]
        fork_policy: Option<String>,

        /// Enable or disable the issue tracker
        #[arg(long, value_name = "true|false")]
        issues: Option<bool>,

        /// Enable or disable the wiki
        #[arg(long, value_name = "true|false")]
        wiki: Option<bool>,

        /// Main branch name
        #[arg(long)]
        main_branch: Option<String>,
    },

    /// Move a repository to a different project in its workspace
    Move {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Destination project key
        project: String,
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
                let workspace = super::resolve_workspace(workspace)?;
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
                let (workspace, name) = match name {
                    Some(name) => (workspace, name),
                    None => parse_repo(&workspace)?,
                };

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

            RepoCommands::Update {
                repo,
                name,
                description,
                private,
                public,
                language,
                fork_policy,
                issues,
                wiki,
                main_branch,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                let request = UpdateRepositoryRequest {
                    name,
                    description,
                    is_private: if private {
                        Some(true)
                    } else if public {
                        Some(false)
                    } else {
                        None
                    },
                    language,
                    fork_policy,
                    has_issues: issues,
                    has_wiki: wiki,
                    project: None,
                    mainbranch: main_branch.map(|name| crate::models::Branch {
                        name,
                        branch_type: None,
                    }),
                };

                if request.is_empty() {
                    anyhow::bail!(
                        "Nothing to update. Pass at least one option, e.g. --description or --private."
                    );
                }

                let client = BitbucketClient::from_stored().await?;
                let updated = client
                    .update_repository(&workspace, &repo_slug, &request)
                    .await?;

                println!(
                    "{} Updated repository {}",
                    "✓".green(),
                    updated.full_name.cyan()
                );

                if let Some(new_slug) = &updated.slug {
                    if new_slug != &repo_slug {
                        println!(
                            "{} Repository slug changed: {} → {}",
                            "ℹ".blue(),
                            repo_slug,
                            new_slug.cyan()
                        );
                    }
                }

                Ok(())
            }

            RepoCommands::Move { repo, project } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                let request = UpdateRepositoryRequest {
                    project: Some(crate::models::ProjectKey {
                        key: project.clone(),
                    }),
                    ..Default::default()
                };

                let client = BitbucketClient::from_stored().await?;
                let updated = client
                    .update_repository(&workspace, &repo_slug, &request)
                    .await?;

                let project_label = updated
                    .project
                    .as_ref()
                    .map(|p| format!("{} ({})", p.name, p.key))
                    .unwrap_or(project);

                println!(
                    "{} Moved {} to project {}",
                    "✓".green(),
                    updated.full_name.cyan(),
                    project_label.cyan()
                );

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
        }
    }
}
