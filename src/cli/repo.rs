use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Subcommand;
use colored::Colorize;
use tabled::{Table, Tabled};

use super::parse_repo;
use crate::api::{BitbucketClient, download_url};
use crate::cli::pagination::{effective_limit, format_size};
use crate::models::{CreateRepositoryRequest, UpdateRepositoryRequest};

#[derive(Subcommand)]
pub enum RepoCommands {
    /// List repositories in a workspace
    List {
        /// Workspace slug (defaults to --workspace or the configured default workspace)
        workspace: Option<String>,

        /// Number of results per page (max 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,

        /// Filter by the authenticated user's role on the repository
        #[arg(long, value_parser = ["member", "contributor", "admin", "owner"])]
        role: Option<String>,

        /// Filter with a Bitbucket query (BBQL) expression, e.g. 'language="rust"'
        #[arg(short, long)]
        query: Option<String>,

        /// Sort by a field, e.g. 'name' or '-updated_on' for descending
        #[arg(long)]
        sort: Option<String>,

        /// Page number to fetch
        #[arg(long)]
        page: Option<u32>,
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

        /// Primary language
        #[arg(short, long)]
        language: Option<String>,

        /// Enable or disable the issue tracker (default: enabled)
        #[arg(long, value_name = "true|false")]
        issues: Option<bool>,

        /// Enable or disable the wiki (default: disabled)
        #[arg(long, value_name = "true|false")]
        wiki: Option<bool>,

        /// Website URL to show on the repository
        #[arg(long)]
        website: Option<String>,

        /// Main branch name
        #[arg(long)]
        main_branch: Option<String>,
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

        /// Website URL to show on the repository
        #[arg(long)]
        website: Option<String>,

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

    /// List users watching a repository
    Watchers {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Maximum number of watchers to list (max 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// List forks of a repository
    Forks {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Maximum number of forks to list (max 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// Manage repository downloads (uploaded file artifacts)
    Download {
        #[command(subcommand)]
        command: DownloadCommands,
    },

    /// Manage repository branches
    Branch {
        #[command(subcommand)]
        command: super::branch::BranchCommands,
    },

    /// Manage repository tags
    Tag {
        #[command(subcommand)]
        command: super::tag::TagCommands,
    },

    /// Work with repository commits
    Commit {
        #[command(subcommand)]
        command: super::commit::CommitCommands,
    },

    /// Browse repository files and file contents
    File {
        #[command(subcommand)]
        command: super::file::FileCommands,
    },

    /// Manage repository webhooks
    Webhook {
        #[command(subcommand)]
        command: super::webhook::WebhookCommands,
    },

    /// Manage branch restrictions (branch permissions)
    BranchRestriction {
        #[command(subcommand)]
        command: super::branch_restriction::BranchRestrictionCommands,
    },

    /// Manage default reviewers
    Reviewer {
        #[command(subcommand)]
        command: super::reviewer::ReviewerCommands,
    },

    /// Inspect repository user and group permissions
    Permission {
        #[command(subcommand)]
        command: super::permission::PermissionCommands,
    },

    /// Manage repository deploy keys
    DeployKey {
        #[command(subcommand)]
        command: super::deploy::DeployKeyCommands,
    },

    /// Manage deployment environments
    Environment {
        #[command(subcommand)]
        command: super::deploy::EnvironmentCommands,
    },

    /// View repository deployments
    Deployment {
        #[command(subcommand)]
        command: super::deploy::DeploymentCommands,
    },

    /// Manage the repository branching model
    BranchingModel {
        #[command(subcommand)]
        command: super::branching_model::BranchingModelCommands,
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

    /// Download an artifact from the repository's downloads area
    Get {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Name of the artifact to download
        name: String,

        /// Output path (defaults to the artifact name in the current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
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

#[derive(Tabled)]
struct WatcherRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "USERNAME")]
    username: String,
}

/// Map repositories into table rows shared by `repo list` and `repo forks`.
fn repo_rows(repos: &[crate::models::Repository]) -> Vec<RepoRow> {
    repos
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
        .collect()
}

impl RepoCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            RepoCommands::List {
                workspace,
                limit,
                role,
                query,
                sort,
                page,
            } => {
                let workspace = super::resolve_workspace(workspace)?;
                let limit = effective_limit(limit);
                let client = BitbucketClient::from_stored().await?;
                let repos = client
                    .list_repositories_filtered(
                        &workspace,
                        role.as_deref(),
                        query.as_deref(),
                        sort.as_deref(),
                        page,
                        Some(limit),
                    )
                    .await?;

                if super::output_json() {
                    return super::print_json(&repos.values);
                }

                if repos.values.is_empty() {
                    println!("No repositories found in workspace '{}'", workspace);
                    return Ok(());
                }

                let table = Table::new(repo_rows(&repos.values)).to_string();
                println!("{}", table);

                if repos.next.is_some() {
                    println!(
                        "\n{} More repositories available. Use --page {} to see the next page.",
                        "ℹ".blue(),
                        page.unwrap_or(1) + 1
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

                if super::output_json() {
                    return super::print_json(&repository);
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
                language,
                issues,
                wiki,
                website,
                main_branch,
            } => {
                let (workspace, name) = super::resolve_create_target(workspace, name)?;

                let client = BitbucketClient::from_stored().await?;

                let slug = name.to_lowercase().replace(' ', "-");

                let request = build_create_request(CreateFlags {
                    name,
                    description,
                    public,
                    project,
                    fork_policy,
                    language,
                    issues,
                    wiki,
                    website,
                    main_branch,
                });

                let repository = client
                    .create_repository(&workspace, &slug, &request)
                    .await?;

                if super::output_json() {
                    return super::print_json(&repository);
                }

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
                website,
                main_branch,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                let request = build_update_request(UpdateFlags {
                    name,
                    description,
                    private,
                    public,
                    language,
                    fork_policy,
                    issues,
                    wiki,
                    website,
                    main_branch,
                });

                if request.is_empty() {
                    anyhow::bail!(
                        "Nothing to update. Pass at least one option, e.g. --description or --private."
                    );
                }

                let client = BitbucketClient::from_stored().await?;
                let updated = client
                    .update_repository(&workspace, &repo_slug, &request)
                    .await?;

                if super::output_json() {
                    return super::print_json(&updated);
                }

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

                if super::output_json() {
                    return super::print_json(&updated);
                }

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

                if super::output_json() {
                    return super::print_json(&forked);
                }

                println!("{} Forked to {}", "✓".green(), forked.full_name.cyan());

                Ok(())
            }

            RepoCommands::Delete { repo, yes } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                if !yes
                    && !confirm_or_abort(format!(
                        "Are you sure you want to delete {}? This cannot be undone!",
                        repo.red()
                    ))?
                {
                    return Ok(());
                }

                let client = BitbucketClient::from_stored().await?;
                client.delete_repository(&workspace, &repo_slug).await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!("{} Deleted repository {}", "✓".green(), repo);

                Ok(())
            }

            RepoCommands::Watchers { repo, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let limit = effective_limit(limit);
                let client = BitbucketClient::from_stored().await?;
                let watchers = client
                    .list_watchers(&workspace, &repo_slug, Some(limit))
                    .await?;

                if super::output_json() {
                    return super::print_json(&watchers.values);
                }

                if watchers.values.is_empty() {
                    println!("No watchers found for {}", repo);
                    return Ok(());
                }

                let rows: Vec<WatcherRow> = watchers
                    .values
                    .iter()
                    .map(|u| WatcherRow {
                        name: u.display_name.clone(),
                        username: u.username.clone().unwrap_or_else(|| "-".to_string()),
                    })
                    .collect();

                println!("{}", Table::new(rows));

                if watchers.next.is_some() {
                    println!(
                        "\n{} More watchers available. Use --limit to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            RepoCommands::Forks { repo, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let limit = effective_limit(limit);
                let client = BitbucketClient::from_stored().await?;
                let forks = client
                    .list_forks(&workspace, &repo_slug, Some(limit))
                    .await?;

                if super::output_json() {
                    return super::print_json(&forks.values);
                }

                if forks.values.is_empty() {
                    println!("No forks found for {}", repo);
                    return Ok(());
                }

                println!("{}", Table::new(repo_rows(&forks.values)));

                if forks.next.is_some() {
                    println!(
                        "\n{} More forks available. Use --limit to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            RepoCommands::Download { command } => command.run().await,
            RepoCommands::Branch { command } => command.run().await,
            RepoCommands::Tag { command } => command.run().await,
            RepoCommands::Commit { command } => command.run().await,
            RepoCommands::File { command } => command.run().await,
            RepoCommands::Webhook { command } => command.run().await,
            RepoCommands::BranchRestriction { command } => command.run().await,
            RepoCommands::Reviewer { command } => command.run().await,
            RepoCommands::Permission { command } => command.run().await,
            RepoCommands::DeployKey { command } => command.run().await,
            RepoCommands::Environment { command } => command.run().await,
            RepoCommands::Deployment { command } => command.run().await,
            RepoCommands::BranchingModel { command } => command.run().await,
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

                // Bitbucket keys downloads by filename, so two inputs with the
                // same basename would silently overwrite each other server-side.
                let mut seen = HashSet::new();
                for (name, _) in &uploads {
                    if !seen.insert(name.as_str()) {
                        anyhow::bail!(
                            "Duplicate upload name '{}': multiple input files share this filename; Bitbucket stores downloads by filename, so one would overwrite the other",
                            name
                        );
                    }
                }

                let client = BitbucketClient::from_stored().await?;
                client
                    .upload_downloads(&workspace, &repo_slug, &uploads)
                    .await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

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

                if super::output_json() {
                    return super::print_json(&downloads);
                }

                if downloads.is_empty() {
                    println!("No downloads found in {}", repo);
                    return Ok(());
                }

                let rows: Vec<DownloadRow> = downloads
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

            DownloadCommands::Get { repo, name, output } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let bytes = client
                    .get_download(&workspace, &repo_slug, &name)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to download '{}' from {}/{}",
                            name, workspace, repo_slug
                        )
                    })?;

                let output = output.unwrap_or_else(|| PathBuf::from(&name));
                std::fs::write(&output, &bytes)
                    .with_context(|| format!("Failed to write '{}'", output.display()))?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({
                        "ok": true,
                        "path": output.display().to_string(),
                        "bytes": bytes.len(),
                    }));
                }

                println!(
                    "{} Downloaded {} to {} ({} bytes)",
                    "✓".green(),
                    name.cyan(),
                    output.display(),
                    bytes.len()
                );

                Ok(())
            }

            DownloadCommands::Delete { repo, name, yes } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                if !yes
                    && !confirm_or_abort(format!("Delete download {} from {}?", name.red(), repo))?
                {
                    return Ok(());
                }

                let client = BitbucketClient::from_stored().await?;
                client
                    .delete_download(&workspace, &repo_slug, &name)
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to delete download '{}' from {}/{}",
                            name, workspace, repo_slug
                        )
                    })?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!("{} Deleted download {}", "✓".green(), name);

                Ok(())
            }
        }
    }
}

/// The `repo create` command-line flags, grouped for translation into a
/// `CreateRepositoryRequest`.
#[derive(Default)]
struct CreateFlags {
    name: String,
    description: Option<String>,
    public: bool,
    project: Option<String>,
    fork_policy: Option<String>,
    language: Option<String>,
    issues: Option<bool>,
    wiki: Option<bool>,
    website: Option<String>,
    main_branch: Option<String>,
}

/// Build the creation request for `repo create` from its command-line flags.
///
/// An explicit --fork-policy wins; otherwise public repositories default to
/// `allow_forks` and private ones to `no_public_forks`. --issues and --wiki
/// override the request defaults (issues on, wiki off).
fn build_create_request(flags: CreateFlags) -> CreateRepositoryRequest {
    let defaults = CreateRepositoryRequest::default();

    let fork_policy = flags.fork_policy.unwrap_or_else(|| {
        if flags.public {
            "allow_forks".to_string()
        } else {
            "no_public_forks".to_string()
        }
    });

    CreateRepositoryRequest {
        scm: "git".to_string(),
        name: Some(flags.name),
        description: flags.description,
        is_private: Some(!flags.public),
        project: flags.project.map(|key| crate::models::ProjectKey { key }),
        fork_policy: Some(fork_policy),
        language: flags.language,
        has_issues: flags.issues.or(defaults.has_issues),
        has_wiki: flags.wiki.or(defaults.has_wiki),
        website: flags.website,
        mainbranch: flags.main_branch.map(|name| crate::models::Branch {
            name,
            branch_type: None,
        }),
    }
}

/// The `repo update` command-line flags, grouped for translation into an
/// `UpdateRepositoryRequest`.
#[derive(Default)]
struct UpdateFlags {
    name: Option<String>,
    description: Option<String>,
    private: bool,
    public: bool,
    language: Option<String>,
    fork_policy: Option<String>,
    issues: Option<bool>,
    wiki: Option<bool>,
    website: Option<String>,
    main_branch: Option<String>,
}

/// Build the update request for `repo update` from its command-line flags.
fn build_update_request(flags: UpdateFlags) -> UpdateRepositoryRequest {
    UpdateRepositoryRequest {
        name: flags.name,
        description: flags.description,
        is_private: if flags.private {
            Some(true)
        } else if flags.public {
            Some(false)
        } else {
            None
        },
        language: flags.language,
        fork_policy: flags.fork_policy,
        has_issues: flags.issues,
        has_wiki: flags.wiki,
        website: flags.website,
        project: None,
        mainbranch: flags.main_branch.map(|name| crate::models::Branch {
            name,
            branch_type: None,
        }),
    }
}

use super::confirm_or_abort;

/// Extract the file name (final path component) from a path, for use as the
/// uploaded artifact name.
fn upload_name_for(path: &Path) -> Result<String> {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .with_context(|| format!("Could not determine a file name for '{}'", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upload_name_takes_final_component() {
        let p = PathBuf::from("/tmp/screenshots/login.png");
        assert_eq!(upload_name_for(&p).unwrap(), "login.png");
    }

    fn empty_update_request(private: bool, public: bool) -> UpdateRepositoryRequest {
        build_update_request(UpdateFlags {
            private,
            public,
            ..Default::default()
        })
    }

    #[test]
    fn build_update_request_maps_private_flag() {
        assert_eq!(empty_update_request(true, false).is_private, Some(true));
    }

    #[test]
    fn build_update_request_maps_public_flag() {
        assert_eq!(empty_update_request(false, true).is_private, Some(false));
    }

    #[test]
    fn build_update_request_leaves_privacy_unset_without_flags() {
        assert_eq!(empty_update_request(false, false).is_private, None);
    }

    #[test]
    fn build_update_request_maps_main_branch() {
        let request = build_update_request(UpdateFlags {
            main_branch: Some("develop".to_string()),
            ..Default::default()
        });
        let branch = request.mainbranch.expect("mainbranch should be set");
        assert_eq!(branch.name, "develop");
        assert_eq!(branch.branch_type, None);
    }

    #[test]
    fn build_update_request_maps_website() {
        let request = build_update_request(UpdateFlags {
            website: Some("https://example.com".to_string()),
            ..Default::default()
        });
        assert_eq!(request.website.as_deref(), Some("https://example.com"));
    }

    #[test]
    fn build_create_request_keeps_defaults_without_flags() {
        let request = build_create_request(CreateFlags {
            name: "widgets".to_string(),
            ..Default::default()
        });

        assert_eq!(request.name.as_deref(), Some("widgets"));
        assert_eq!(request.is_private, Some(true));
        assert_eq!(request.fork_policy.as_deref(), Some("no_public_forks"));
        // Defaults from CreateRepositoryRequest::default(): issues on, wiki off.
        assert_eq!(request.has_issues, Some(true));
        assert_eq!(request.has_wiki, Some(false));
        assert_eq!(request.website, None);
        assert!(request.mainbranch.is_none());
    }

    #[test]
    fn build_create_request_flags_override_issue_and_wiki_defaults() {
        let request = build_create_request(CreateFlags {
            name: "widgets".to_string(),
            issues: Some(false),
            wiki: Some(true),
            ..Default::default()
        });

        assert_eq!(request.has_issues, Some(false));
        assert_eq!(request.has_wiki, Some(true));
    }

    #[test]
    fn build_create_request_public_defaults_to_allow_forks() {
        let request = build_create_request(CreateFlags {
            name: "widgets".to_string(),
            public: true,
            ..Default::default()
        });

        assert_eq!(request.is_private, Some(false));
        assert_eq!(request.fork_policy.as_deref(), Some("allow_forks"));
    }

    #[test]
    fn build_create_request_explicit_fork_policy_wins() {
        let request = build_create_request(CreateFlags {
            name: "widgets".to_string(),
            public: true,
            fork_policy: Some("no_forks".to_string()),
            ..Default::default()
        });

        assert_eq!(request.fork_policy.as_deref(), Some("no_forks"));
    }

    #[test]
    fn build_create_request_maps_website_and_main_branch() {
        let request = build_create_request(CreateFlags {
            name: "widgets".to_string(),
            website: Some("https://example.com".to_string()),
            main_branch: Some("trunk".to_string()),
            ..Default::default()
        });

        assert_eq!(request.website.as_deref(), Some("https://example.com"));
        let branch = request.mainbranch.expect("mainbranch should be set");
        assert_eq!(branch.name, "trunk");
        assert_eq!(branch.branch_type, None);
    }
}
