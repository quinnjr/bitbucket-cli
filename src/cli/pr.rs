use anyhow::{Context, Result};
use clap::{Subcommand, ValueEnum};
use colored::Colorize;
use tabled::{Table, Tabled};

use crate::api::BitbucketClient;
use crate::models::{
    BranchInfo, CreatePullRequestRequest, MergePullRequestRequest, MergeStrategy,
    PullRequestBranchRef, PullRequestState,
};

#[derive(Subcommand)]
pub enum PrCommands {
    /// List pull requests
    List {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Filter by state
        #[arg(short, long, value_enum)]
        state: Option<PrState>,

        /// Number of results
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// View pull request details
    View {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,

        /// Open in browser
        #[arg(short, long)]
        web: bool,
    },

    /// Create a new pull request
    Create {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Title of the pull request
        #[arg(short, long)]
        title: String,

        /// Source branch
        #[arg(short, long)]
        source: String,

        /// Destination branch (defaults to main branch)
        #[arg(short, long)]
        destination: Option<String>,

        /// Description of the pull request
        #[arg(short = 'b', long)]
        body: Option<String>,

        /// Close source branch after merge
        #[arg(long)]
        close_source_branch: bool,
    },

    /// Merge a pull request
    Merge {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,

        /// Merge strategy
        #[arg(short, long, value_enum, default_value = "merge-commit")]
        strategy: MergeStrategyArg,

        /// Commit message
        #[arg(short, long)]
        message: Option<String>,

        /// Close source branch
        #[arg(long)]
        close_source_branch: bool,
    },

    /// Approve a pull request
    Approve {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,
    },

    /// Decline a pull request
    Decline {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,
    },

    /// Checkout a pull request branch locally
    Checkout {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,
    },

    /// View pull request diff
    Diff {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,
    },

    /// Add a comment to a pull request
    Comment {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,

        /// Comment text
        #[arg(short, long)]
        body: String,
    },

    /// List comments on a pull request
    ListComments {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,

        /// Number of results
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// View a specific comment on a pull request
    ViewComment {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        #[arg(value_name = "PR_ID")]
        id: u64,

        /// Comment ID
        comment_id: u64,
    },

    /// List pipelines for the PR's head commit
    Pipelines {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,

        /// Maximum recent pipelines to scan for matches (capped at 100)
        #[arg(short, long, default_value = "100")]
        scan_limit: u32,
    },
}

#[derive(ValueEnum, Clone)]
pub enum PrState {
    Open,
    Merged,
    Declined,
    Superseded,
}

impl From<PrState> for PullRequestState {
    fn from(state: PrState) -> Self {
        match state {
            PrState::Open => PullRequestState::Open,
            PrState::Merged => PullRequestState::Merged,
            PrState::Declined => PullRequestState::Declined,
            PrState::Superseded => PullRequestState::Superseded,
        }
    }
}

#[derive(ValueEnum, Clone)]
pub enum MergeStrategyArg {
    MergeCommit,
    Squash,
    FastForward,
}

impl From<MergeStrategyArg> for MergeStrategy {
    fn from(strategy: MergeStrategyArg) -> Self {
        match strategy {
            MergeStrategyArg::MergeCommit => MergeStrategy::MergeCommit,
            MergeStrategyArg::Squash => MergeStrategy::Squash,
            MergeStrategyArg::FastForward => MergeStrategy::FastForward,
        }
    }
}

#[derive(Tabled)]
struct PrRow {
    #[tabled(rename = "ID")]
    id: u64,
    #[tabled(rename = "TITLE")]
    title: String,
    #[tabled(rename = "AUTHOR")]
    author: String,
    #[tabled(rename = "STATE")]
    state: String,
    #[tabled(rename = "UPDATED")]
    updated: String,
}

#[derive(Tabled)]
struct PipelineRow {
    #[tabled(rename = "#")]
    build: u64,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "BRANCH")]
    branch: String,
    #[tabled(rename = "COMMIT")]
    commit: String,
    #[tabled(rename = "TRIGGERED")]
    triggered: String,
    #[tabled(rename = "DURATION")]
    duration: String,
}

#[derive(Tabled)]
struct CommentRow {
    #[tabled(rename = "ID")]
    id: u64,
    #[tabled(rename = "AUTHOR")]
    author: String,
    #[tabled(rename = "CREATED")]
    created: String,
    #[tabled(rename = "TYPE")]
    comment_type: String,
    #[tabled(rename = "CONTENT")]
    content: String,
}

impl PrCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            PrCommands::List { repo, state, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let prs = client
                    .list_pull_requests(
                        &workspace,
                        &repo_slug,
                        state.map(|s| s.into()),
                        None,
                        Some(limit),
                    )
                    .await?;

                if prs.values.is_empty() {
                    println!("No pull requests found");
                    return Ok(());
                }

                let rows: Vec<PrRow> = prs
                    .values
                    .iter()
                    .map(|pr| PrRow {
                        id: pr.id,
                        title: pr.title.chars().take(50).collect(),
                        author: pr.author.display_name.clone(),
                        state: format_state(&pr.state),
                        updated: pr.updated_on.format("%Y-%m-%d").to_string(),
                    })
                    .collect();

                let table = Table::new(rows).to_string();
                println!("{}", table);

                Ok(())
            }

            PrCommands::View { repo, id, web } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let pr = client.get_pull_request(&workspace, &repo_slug, id).await?;

                if web {
                    if let Some(links) = &pr.links {
                        if let Some(html) = &links.html {
                            open::that(&html.href)?;
                            println!("Opened {} in browser", html.href.cyan());
                            return Ok(());
                        }
                    }
                    anyhow::bail!("Could not find PR URL");
                }

                println!("{} {} #{}", format_state(&pr.state), pr.title.bold(), pr.id);
                println!("{}", "─".repeat(60));

                println!(
                    "{} {} → {}",
                    "Branches:".dimmed(),
                    pr.source.branch.name.cyan(),
                    pr.destination.branch.name.green()
                );
                println!("{} {}", "Author:".dimmed(), pr.author.display_name);
                println!(
                    "{} {}",
                    "Created:".dimmed(),
                    pr.created_on.format("%Y-%m-%d %H:%M")
                );
                println!(
                    "{} {}",
                    "Updated:".dimmed(),
                    pr.updated_on.format("%Y-%m-%d %H:%M")
                );

                if let Some(count) = pr.comment_count {
                    println!("{} {}", "Comments:".dimmed(), count);
                }

                if let Some(tasks) = pr.task_count {
                    if tasks > 0 {
                        println!("{} {}", "Tasks:".dimmed(), tasks);
                    }
                }

                // Show reviewers/approvals
                if let Some(participants) = &pr.participants {
                    let approvals: Vec<_> = participants
                        .iter()
                        .filter(|p| p.approved)
                        .map(|p| p.user.display_name.clone())
                        .collect();

                    if !approvals.is_empty() {
                        println!(
                            "{} {}",
                            "Approved by:".dimmed(),
                            approvals.join(", ").green()
                        );
                    }
                }

                if let Some(description) = &pr.description {
                    if !description.is_empty() {
                        println!();
                        println!("{}", description);
                    }
                }

                if let Some(links) = &pr.links {
                    if let Some(html) = &links.html {
                        println!();
                        println!("{} {}", "URL:".dimmed(), html.href.cyan());
                    }
                }

                Ok(())
            }

            PrCommands::Create {
                repo,
                title,
                source,
                destination,
                body,
                close_source_branch,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let request = CreatePullRequestRequest {
                    title,
                    source: PullRequestBranchRef {
                        branch: BranchInfo { name: source },
                    },
                    destination: destination.map(|d| PullRequestBranchRef {
                        branch: BranchInfo { name: d },
                    }),
                    description: body,
                    close_source_branch: Some(close_source_branch),
                    reviewers: None,
                };

                let pr = client
                    .create_pull_request(&workspace, &repo_slug, &request)
                    .await?;

                println!("{} Created pull request #{}", "✓".green(), pr.id);

                if let Some(links) = &pr.links {
                    if let Some(html) = &links.html {
                        println!("{} {}", "URL:".dimmed(), html.href.cyan());
                    }
                }

                Ok(())
            }

            PrCommands::Merge {
                repo,
                id,
                strategy,
                message,
                close_source_branch,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let request = MergePullRequestRequest {
                    merge_type: Some("pullrequest".to_string()),
                    message,
                    close_source_branch: Some(close_source_branch),
                    merge_strategy: Some(strategy.into()),
                };

                let pr = client
                    .merge_pull_request(&workspace, &repo_slug, id, Some(&request))
                    .await?;

                println!("{} Merged pull request #{}", "✓".green(), pr.id);

                Ok(())
            }

            PrCommands::Approve { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                client
                    .approve_pull_request(&workspace, &repo_slug, id)
                    .await?;

                println!("{} Approved pull request #{}", "✓".green(), id);

                Ok(())
            }

            PrCommands::Decline { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                client
                    .decline_pull_request(&workspace, &repo_slug, id)
                    .await?;

                println!("{} Declined pull request #{}", "✓".green(), id);

                Ok(())
            }

            PrCommands::Checkout { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let pr = client.get_pull_request(&workspace, &repo_slug, id).await?;
                let branch = &pr.source.branch.name;

                println!("Fetching and checking out branch {}...", branch.cyan());

                // Fetch the branch
                let status = std::process::Command::new("git")
                    .args(["fetch", "origin", branch])
                    .status()
                    .context("Failed to fetch branch")?;

                if !status.success() {
                    anyhow::bail!("git fetch failed");
                }

                // Checkout the branch
                let status = std::process::Command::new("git")
                    .args(["checkout", branch])
                    .status()
                    .context("Failed to checkout branch")?;

                if status.success() {
                    println!("{} Checked out branch {}", "✓".green(), branch);
                } else {
                    // Try creating a tracking branch
                    let status = std::process::Command::new("git")
                        .args(["checkout", "-b", branch, &format!("origin/{}", branch)])
                        .status()
                        .context("Failed to create tracking branch")?;

                    if status.success() {
                        println!("{} Created and checked out branch {}", "✓".green(), branch);
                    } else {
                        anyhow::bail!("git checkout failed");
                    }
                }

                Ok(())
            }

            PrCommands::Diff { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let diff = client.get_pr_diff(&workspace, &repo_slug, id).await?;
                println!("{}", diff);

                Ok(())
            }

            PrCommands::Comment { repo, id, body } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                client
                    .add_pr_comment(&workspace, &repo_slug, id, &body)
                    .await?;

                println!("{} Added comment to pull request #{}", "✓".green(), id);

                Ok(())
            }

            PrCommands::ListComments { repo, id, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let comments = client.list_pr_comments(&workspace, &repo_slug, id).await?;

                let mut values: Vec<_> = comments.values.into_iter().take(limit as usize).collect();

                if values.is_empty() {
                    println!("No comments found");
                    return Ok(());
                }

                values.sort_by_key(|c| c.created_on);

                let rows: Vec<CommentRow> = values
                    .iter()
                    .map(|c| CommentRow {
                        id: c.id,
                        author: c.user.display_name.clone(),
                        created: c.created_on.format("%Y-%m-%d %H:%M").to_string(),
                        comment_type: if c.inline.is_some() {
                            "inline".to_string()
                        } else {
                            "general".to_string()
                        },
                        content: c.content.raw.chars().take(50).collect(),
                    })
                    .collect();

                let table = Table::new(rows).to_string();
                println!("{}", table);

                Ok(())
            }

            PrCommands::Pipelines {
                repo,
                id,
                scan_limit,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let pr = client.get_pull_request(&workspace, &repo_slug, id).await?;
                let head_commit = pr
                    .source
                    .commit
                    .as_ref()
                    .map(|c| c.hash.clone())
                    .context("PR has no head commit hash")?;

                let pipelines = client
                    .list_pipelines_for_commit(&workspace, &repo_slug, &head_commit, scan_limit)
                    .await?;

                if pipelines.is_empty() {
                    println!(
                        "No pipelines found for PR #{} head commit {} (scanned {} most recent).",
                        id,
                        head_commit.chars().take(12).collect::<String>(),
                        scan_limit.clamp(1, 100)
                    );
                    return Ok(());
                }

                let rows: Vec<PipelineRow> = pipelines
                    .iter()
                    .map(|p| {
                        let duration = match (p.build_seconds_used, &p.state.name) {
                            (Some(s), _) => super::pipeline::format_duration(s),
                            (None, crate::models::PipelineStateName::InProgress) => {
                                "running...".to_string()
                            }
                            _ => "-".to_string(),
                        };

                        PipelineRow {
                            build: p.build_number,
                            status: super::pipeline::format_status(
                                &p.state.name,
                                p.state.result.as_ref().map(|r| &r.name),
                            ),
                            branch: p.target.ref_name.clone().unwrap_or_else(|| "-".to_string()),
                            commit: p
                                .target
                                .commit
                                .as_ref()
                                .map(|c| c.hash.chars().take(12).collect())
                                .unwrap_or_else(|| "-".to_string()),
                            triggered: p.created_on.format("%Y-%m-%d %H:%M").to_string(),
                            duration,
                        }
                    })
                    .collect();

                println!("{}", Table::new(rows));

                Ok(())
            }

            PrCommands::ViewComment {
                repo,
                id,
                comment_id,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let comment = client
                    .get_pr_comment(&workspace, &repo_slug, id, comment_id)
                    .await?;

                println!("{} #{} on PR #{}", "Comment".bold(), comment.id, id);
                println!("{}", "─".repeat(60));

                println!("{} {}", "Author:".dimmed(), comment.user.display_name);
                println!(
                    "{} {}",
                    "Created:".dimmed(),
                    comment.created_on.format("%Y-%m-%d %H:%M")
                );

                if let Some(updated) = comment.updated_on {
                    println!(
                        "{} {}",
                        "Updated:".dimmed(),
                        updated.format("%Y-%m-%d %H:%M")
                    );
                }

                if let Some(inline) = &comment.inline {
                    let line = inline.to.or(inline.from);
                    let location = match line {
                        Some(l) => format!("{}:{}", inline.path, l),
                        None => inline.path.clone(),
                    };
                    println!("{} inline", "Type:".dimmed());
                    println!("{} {}", "File:".dimmed(), location.cyan());
                } else {
                    println!("{} general", "Type:".dimmed());
                }

                println!();
                println!("{}", comment.content.raw);

                if let Some(links) = &comment.links {
                    if let Some(html) = &links.html {
                        println!();
                        println!("{} {}", "URL:".dimmed(), html.href.cyan());
                    }
                }

                Ok(())
            }
        }
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

fn format_state(state: &PullRequestState) -> String {
    match state {
        PullRequestState::Open => "OPEN".green().to_string(),
        PullRequestState::Merged => "MERGED".purple().to_string(),
        PullRequestState::Declined => "DECLINED".red().to_string(),
        PullRequestState::Superseded => "SUPERSEDED".yellow().to_string(),
    }
}
