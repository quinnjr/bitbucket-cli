use anyhow::{Context, Result};
use clap::{Subcommand, ValueEnum};
use colored::Colorize;
use tabled::{Table, Tabled};

use super::parse_repo;
use crate::api::BitbucketClient;
use crate::cli::pagination::effective_limit;
use crate::models::{
    BranchInfo, CreatePullRequestRequest, MergePullRequestRequest, MergeStrategy, PrActivity,
    PullRequest, PullRequestBranchRef, PullRequestComment, PullRequestState,
    UpdatePullRequestRequest, UserRef,
};

#[derive(Subcommand)]
pub enum PrCommands {
    /// List pull requests
    List {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Filter by state (repeatable to match several states)
        #[arg(short, long, value_enum)]
        state: Vec<PrState>,

        /// Filter with a Bitbucket query (BBQL) expression
        #[arg(short = 'q', long)]
        query: Option<String>,

        /// Sort field (e.g. -updated_on)
        #[arg(long)]
        sort: Option<String>,

        /// Page number to fetch
        #[arg(long)]
        page: Option<u32>,

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

        /// Comma-separated reviewer account IDs, UUIDs, or usernames
        #[arg(long)]
        reviewers: Option<String>,
    },

    /// Edit an existing pull request
    Edit {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,

        /// New title
        #[arg(short, long)]
        title: Option<String>,

        /// New description
        #[arg(short = 'b', long)]
        body: Option<String>,

        /// Comma-separated reviewer account IDs, UUIDs, or usernames (replaces the reviewer list)
        #[arg(long)]
        reviewers: Option<String>,

        /// New destination branch
        #[arg(short, long)]
        destination: Option<String>,
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

    /// Remove your approval from a pull request
    Unapprove {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,
    },

    /// Request changes on a pull request
    RequestChanges {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,
    },

    /// Withdraw your request for changes on a pull request
    UnrequestChanges {
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

        /// File path for an inline comment
        #[arg(long)]
        path: Option<String>,

        /// Line number for an inline comment (requires --path)
        #[arg(long, requires = "path")]
        line: Option<u32>,

        /// Parent comment ID to reply to
        #[arg(long)]
        parent: Option<u64>,
    },

    /// Edit a comment on a pull request
    EditComment {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        #[arg(value_name = "PR_ID")]
        id: u64,

        /// Comment ID
        comment_id: u64,

        /// New comment text
        #[arg(short, long)]
        body: String,
    },

    /// Delete a comment on a pull request
    DeleteComment {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        #[arg(value_name = "PR_ID")]
        id: u64,

        /// Comment ID
        comment_id: u64,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Resolve a comment thread on a pull request
    ResolveComment {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        #[arg(value_name = "PR_ID")]
        id: u64,

        /// Comment ID
        comment_id: u64,
    },

    /// Reopen a resolved comment thread on a pull request
    UnresolveComment {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        #[arg(value_name = "PR_ID")]
        id: u64,

        /// Comment ID
        comment_id: u64,
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

    /// List commits on a pull request
    Commits {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,

        /// Number of results
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// Show build statuses for a pull request
    Statuses {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,
    },

    /// Show the per-file change summary for a pull request
    Diffstat {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,

        /// Number of results
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// Manage tasks on a pull request
    Task {
        #[command(subcommand)]
        command: TaskCommands,
    },

    /// Show the activity feed for a pull request
    Activity {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,

        /// Number of results
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// Print the patch (mbox-style) for a pull request
    Patch {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,
    },
}

/// Subcommands for `pr task`
#[derive(Subcommand)]
pub enum TaskCommands {
    /// List tasks on a pull request
    List {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,
    },

    /// Add a task to a pull request
    Add {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        id: u64,

        /// Task text
        #[arg(short, long)]
        body: String,
    },

    /// Mark a task as resolved
    Resolve {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        #[arg(value_name = "PR_ID")]
        id: u64,

        /// Task ID
        task_id: u64,
    },

    /// Reopen a resolved task
    Reopen {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        #[arg(value_name = "PR_ID")]
        id: u64,

        /// Task ID
        task_id: u64,
    },

    /// Delete a task from a pull request
    Delete {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pull request ID
        #[arg(value_name = "PR_ID")]
        id: u64,

        /// Task ID
        task_id: u64,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
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
struct CommitRow {
    #[tabled(rename = "HASH")]
    hash: String,
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "MESSAGE")]
    message: String,
}

#[derive(Tabled)]
struct StatusRow {
    #[tabled(rename = "KEY")]
    key: String,
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "STATE")]
    state: String,
    #[tabled(rename = "URL")]
    url: String,
}

#[derive(Tabled)]
struct DiffstatRow {
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "FILE")]
    file: String,
    #[tabled(rename = "+")]
    added: String,
    #[tabled(rename = "-")]
    removed: String,
}

#[derive(Tabled)]
struct TaskRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "STATE")]
    state: String,
    #[tabled(rename = "TASK")]
    content: String,
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
            PrCommands::List {
                repo,
                state,
                query,
                sort,
                page,
                limit,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let states: Vec<PullRequestState> = state.into_iter().map(Into::into).collect();
                let prs = client
                    .list_pull_requests_filtered(
                        &workspace,
                        &repo_slug,
                        crate::api::pullrequests::PrListFilters {
                            states: &states,
                            query: query.as_deref(),
                            sort: sort.as_deref(),
                            page,
                            pagelen: Some(limit.clamp(1, 100)),
                        },
                    )
                    .await?;

                if super::output_json() {
                    return super::print_json(&prs.values);
                }

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

                if prs.next.is_some() {
                    println!(
                        "{} More pull requests available; use --page/--limit to see more.",
                        "ℹ".blue()
                    );
                }

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

                if super::output_json() {
                    return super::print_json(&pr);
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
                reviewers,
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
                    reviewers: reviewers.as_deref().map(parse_reviewers),
                };

                let pr = client
                    .create_pull_request(&workspace, &repo_slug, &request)
                    .await?;

                if super::output_json() {
                    return super::print_json(&pr);
                }

                println!("{} Created pull request #{}", "✓".green(), pr.id);

                if let Some(links) = &pr.links {
                    if let Some(html) = &links.html {
                        println!("{} {}", "URL:".dimmed(), html.href.cyan());
                    }
                }

                Ok(())
            }

            PrCommands::Edit {
                repo,
                id,
                title,
                body,
                reviewers,
                destination,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                let request = UpdatePullRequestRequest {
                    title,
                    description: body,
                    reviewers: reviewers.as_deref().map(parse_reviewers),
                    destination: destination.map(|d| PullRequestBranchRef {
                        branch: BranchInfo { name: d },
                    }),
                };

                if request.is_empty() {
                    anyhow::bail!(
                        "Nothing to edit: pass at least one of --title, --body, --reviewers, or --destination"
                    );
                }

                let client = BitbucketClient::from_stored().await?;
                let pr = client
                    .update_pull_request(&workspace, &repo_slug, id, &request)
                    .await?;

                if super::output_json() {
                    return super::print_json(&pr);
                }

                println!("{} Updated pull request #{}", "✓".green(), pr.id);

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

                if super::output_json() {
                    return super::print_json(&pr);
                }

                println!("{} Merged pull request #{}", "✓".green(), pr.id);

                Ok(())
            }

            PrCommands::Approve { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                client
                    .approve_pull_request(&workspace, &repo_slug, id)
                    .await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!("{} Approved pull request #{}", "✓".green(), id);

                Ok(())
            }

            PrCommands::Unapprove { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                client
                    .unapprove_pull_request(&workspace, &repo_slug, id)
                    .await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!("{} Removed approval from pull request #{}", "✓".green(), id);

                Ok(())
            }

            PrCommands::RequestChanges { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                client
                    .request_pr_changes(&workspace, &repo_slug, id)
                    .await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!("{} Requested changes on pull request #{}", "✓".green(), id);

                Ok(())
            }

            PrCommands::UnrequestChanges { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                client
                    .unrequest_pr_changes(&workspace, &repo_slug, id)
                    .await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!(
                    "{} Withdrew request for changes on pull request #{}",
                    "✓".green(),
                    id
                );

                Ok(())
            }

            PrCommands::Decline { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let pr = client
                    .decline_pull_request(&workspace, &repo_slug, id)
                    .await?;

                if super::output_json() {
                    return super::print_json(&pr);
                }

                println!("{} Declined pull request #{}", "✓".green(), id);

                Ok(())
            }

            PrCommands::Checkout { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let pr = client.get_pull_request(&workspace, &repo_slug, id).await?;
                checkout_pr_branch(&pr, &workspace, &repo_slug).await
            }

            PrCommands::Diff { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let diff = client.get_pr_diff(&workspace, &repo_slug, id).await?;
                println!("{}", diff);

                Ok(())
            }

            PrCommands::Comment {
                repo,
                id,
                body,
                path,
                line,
                parent,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let comment = client
                    .add_pr_comment_full(
                        &workspace,
                        &repo_slug,
                        id,
                        crate::api::pullrequests::PrCommentInput {
                            content: &body,
                            path: path.as_deref(),
                            line,
                            parent,
                        },
                    )
                    .await?;

                if super::output_json() {
                    return super::print_json(&comment);
                }

                println!("{} Added comment to pull request #{}", "✓".green(), id);

                Ok(())
            }

            PrCommands::EditComment {
                repo,
                id,
                comment_id,
                body,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let comment = client
                    .update_pr_comment(&workspace, &repo_slug, id, comment_id, &body)
                    .await?;

                if super::output_json() {
                    return super::print_json(&comment);
                }

                println!(
                    "{} Updated comment #{} on pull request #{}",
                    "✓".green(),
                    comment_id,
                    id
                );

                Ok(())
            }

            PrCommands::DeleteComment {
                repo,
                id,
                comment_id,
                yes,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                if !yes
                    && !super::confirm_or_abort(format!(
                        "Delete comment #{} on PR #{}?",
                        comment_id, id
                    ))?
                {
                    return Ok(());
                }

                let client = BitbucketClient::from_stored().await?;
                client
                    .delete_pr_comment(&workspace, &repo_slug, id, comment_id)
                    .await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!(
                    "{} Deleted comment #{} from pull request #{}",
                    "✓".green(),
                    comment_id,
                    id
                );

                Ok(())
            }

            PrCommands::ResolveComment {
                repo,
                id,
                comment_id,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                client
                    .resolve_pr_comment(&workspace, &repo_slug, id, comment_id)
                    .await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!(
                    "{} Resolved comment thread #{} on pull request #{}",
                    "✓".green(),
                    comment_id,
                    id
                );

                Ok(())
            }

            PrCommands::UnresolveComment {
                repo,
                id,
                comment_id,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                client
                    .unresolve_pr_comment(&workspace, &repo_slug, id, comment_id)
                    .await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!(
                    "{} Reopened comment thread #{} on pull request #{}",
                    "✓".green(),
                    comment_id,
                    id
                );

                Ok(())
            }

            PrCommands::ListComments { repo, id, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let comments = client
                    .list_recent_pr_comments(&workspace, &repo_slug, id, limit as usize)
                    .await?;

                if super::output_json() {
                    return super::print_json(&newest_comments_chronological(
                        comments,
                        limit as usize,
                    ));
                }

                if comments.is_empty() {
                    println!("No comments found");
                    return Ok(());
                }

                let hit_limit = comments.len() >= limit as usize;
                let values = newest_comments_chronological(comments, limit as usize);

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

                if hit_limit {
                    println!(
                        "Showing the {} most recent comments. Use --limit to see more.",
                        limit
                    );
                }

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

                if super::output_json() {
                    return super::print_json(&pipelines);
                }

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

                if super::output_json() {
                    return super::print_json(&comment);
                }

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

            PrCommands::Commits { repo, id, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let commits = client
                    .list_pr_commits(&workspace, &repo_slug, id, Some(limit.clamp(1, 100)))
                    .await?;

                if super::output_json() {
                    return super::print_json(&commits.values);
                }

                if commits.values.is_empty() {
                    println!("No commits found");
                    return Ok(());
                }

                let rows: Vec<CommitRow> = commits
                    .values
                    .iter()
                    .map(|c| CommitRow {
                        hash: c.hash.chars().take(12).collect(),
                        date: c
                            .date
                            .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
                            .unwrap_or_else(|| "-".to_string()),
                        message: first_line_truncated(c.message.as_deref().unwrap_or(""), 60),
                    })
                    .collect();

                println!("{}", Table::new(rows));

                if commits.next.is_some() {
                    println!(
                        "{} More commits available; use --limit to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            PrCommands::Statuses { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let statuses = client.list_pr_statuses(&workspace, &repo_slug, id).await?;

                if super::output_json() {
                    return super::print_json(&statuses);
                }

                if statuses.is_empty() {
                    println!("No build statuses found");
                    return Ok(());
                }

                let rows: Vec<StatusRow> = statuses
                    .iter()
                    .map(|s| StatusRow {
                        key: s.key.clone().unwrap_or_else(|| "-".to_string()),
                        name: s.name.clone().unwrap_or_else(|| "-".to_string()),
                        state: format_commit_status_state(s.state.as_deref().unwrap_or("-")),
                        url: s.url.clone().unwrap_or_else(|| "-".to_string()),
                    })
                    .collect();

                println!("{}", Table::new(rows));

                Ok(())
            }

            PrCommands::Diffstat { repo, id, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let limit = effective_limit(limit);
                let client = BitbucketClient::from_stored().await?;

                let diffstat = client
                    .get_pr_diffstat(&workspace, &repo_slug, id, Some(limit))
                    .await?;

                if super::output_json() {
                    return super::print_json(&diffstat.values);
                }

                if diffstat.values.is_empty() {
                    println!("No changes found");
                    return Ok(());
                }

                let rows: Vec<DiffstatRow> = diffstat
                    .values
                    .iter()
                    .map(|e| {
                        let file = e
                            .new
                            .as_ref()
                            .and_then(|f| f.path.clone())
                            .or_else(|| e.old.as_ref().and_then(|f| f.path.clone()))
                            .unwrap_or_else(|| "-".to_string());

                        DiffstatRow {
                            status: e.status.clone().unwrap_or_else(|| "-".to_string()),
                            file,
                            added: e
                                .lines_added
                                .map(|n| n.to_string())
                                .unwrap_or_else(|| "-".to_string()),
                            removed: e
                                .lines_removed
                                .map(|n| n.to_string())
                                .unwrap_or_else(|| "-".to_string()),
                        }
                    })
                    .collect();

                println!("{}", Table::new(rows));

                let total_added: u64 = diffstat.values.iter().filter_map(|e| e.lines_added).sum();
                let total_removed: u64 =
                    diffstat.values.iter().filter_map(|e| e.lines_removed).sum();
                println!(
                    "{} files changed, {} insertions(+), {} deletions(-)",
                    diffstat.values.len(),
                    total_added.to_string().green(),
                    total_removed.to_string().red()
                );

                if diffstat.next.is_some() {
                    println!(
                        "{} More changed files available; use --limit to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            PrCommands::Task { command } => command.run().await,

            PrCommands::Activity { repo, id, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let activity = client
                    .list_pr_activity(&workspace, &repo_slug, id, Some(limit.clamp(1, 100)))
                    .await?;

                if super::output_json() {
                    return super::print_json(&activity.values);
                }

                if activity.values.is_empty() {
                    println!("No activity found");
                    return Ok(());
                }

                for entry in &activity.values {
                    println!("{}", format_activity_line(entry));
                }

                if activity.next.is_some() {
                    println!(
                        "{} More activity available; use --limit to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            PrCommands::Patch { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let patch = client.get_pr_patch(&workspace, &repo_slug, id).await?;
                println!("{}", patch);

                Ok(())
            }
        }
    }
}

impl TaskCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            TaskCommands::List { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let tasks = client.list_pr_tasks(&workspace, &repo_slug, id).await?;

                if super::output_json() {
                    return super::print_json(&tasks);
                }

                if tasks.is_empty() {
                    println!("No tasks found");
                    return Ok(());
                }

                let rows: Vec<TaskRow> = tasks
                    .iter()
                    .map(|t| TaskRow {
                        id: t
                            .id
                            .map(|i| i.to_string())
                            .unwrap_or_else(|| "-".to_string()),
                        state: match t.state.as_deref() {
                            Some("RESOLVED") => "RESOLVED".green().to_string(),
                            Some("UNRESOLVED") => "UNRESOLVED".yellow().to_string(),
                            Some(other) => other.to_string(),
                            None => "-".to_string(),
                        },
                        content: first_line_truncated(
                            t.content
                                .as_ref()
                                .and_then(|c| c.raw.as_deref())
                                .unwrap_or(""),
                            60,
                        ),
                    })
                    .collect();

                println!("{}", Table::new(rows));

                Ok(())
            }

            TaskCommands::Add { repo, id, body } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let task = client
                    .add_pr_task(&workspace, &repo_slug, id, &body)
                    .await?;

                if super::output_json() {
                    return super::print_json(&task);
                }

                match task.id {
                    Some(task_id) => println!(
                        "{} Added task #{} to pull request #{}",
                        "✓".green(),
                        task_id,
                        id
                    ),
                    None => println!("{} Added task to pull request #{}", "✓".green(), id),
                }

                Ok(())
            }

            TaskCommands::Resolve { repo, id, task_id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let task = client
                    .update_pr_task(&workspace, &repo_slug, id, task_id, None, Some("RESOLVED"))
                    .await?;

                if super::output_json() {
                    return super::print_json(&task);
                }

                println!(
                    "{} Resolved task #{} on pull request #{}",
                    "✓".green(),
                    task_id,
                    id
                );

                Ok(())
            }

            TaskCommands::Reopen { repo, id, task_id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let task = client
                    .update_pr_task(
                        &workspace,
                        &repo_slug,
                        id,
                        task_id,
                        None,
                        Some("UNRESOLVED"),
                    )
                    .await?;

                if super::output_json() {
                    return super::print_json(&task);
                }

                println!(
                    "{} Reopened task #{} on pull request #{}",
                    "✓".green(),
                    task_id,
                    id
                );

                Ok(())
            }

            TaskCommands::Delete {
                repo,
                id,
                task_id,
                yes,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                if !yes
                    && !super::confirm_or_abort(format!("Delete task #{} on PR #{}?", task_id, id))?
                {
                    return Ok(());
                }

                let client = BitbucketClient::from_stored().await?;
                client
                    .delete_pr_task(&workspace, &repo_slug, id, task_id)
                    .await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!(
                    "{} Deleted task #{} from pull request #{}",
                    "✓".green(),
                    task_id,
                    id
                );

                Ok(())
            }
        }
    }
}

fn format_state(state: &PullRequestState) -> String {
    match state {
        PullRequestState::Open => "OPEN".green().to_string(),
        PullRequestState::Merged => "MERGED".purple().to_string(),
        PullRequestState::Declined => "DECLINED".red().to_string(),
        PullRequestState::Superseded => "SUPERSEDED".yellow().to_string(),
    }
}

fn format_commit_status_state(state: &str) -> String {
    match state {
        "SUCCESSFUL" => state.green().to_string(),
        "FAILED" => state.red().to_string(),
        "INPROGRESS" => state.yellow().to_string(),
        "STOPPED" => state.dimmed().to_string(),
        _ => state.to_string(),
    }
}

/// Parse a comma-separated list of reviewer tokens into reviewer references,
/// trimming whitespace and skipping empty segments. Each token is classified
/// as an account ID / UUID or a username by [`reviewer_ref`].
fn parse_reviewers(spec: &str) -> Vec<UserRef> {
    spec.split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(reviewer_ref)
        .collect()
}

/// Classify a reviewer token and build the matching [`UserRef`].
///
/// Post-GDPR Bitbucket ignores `username` in request bodies, so a token that
/// looks like an account ID or UUID — it starts with `{`, contains `:`, or
/// matches the `8-4-4-4-12` hex-dash UUID shape — is placed in `uuid`.
/// Everything else is treated as a `username`.
fn reviewer_ref(token: &str) -> UserRef {
    if looks_like_account_id(token) {
        UserRef {
            uuid: Some(token.to_string()),
            username: None,
        }
    } else {
        UserRef {
            uuid: None,
            username: Some(token.to_string()),
        }
    }
}

/// True when `token` looks like a Bitbucket account ID or UUID rather than a
/// plain username.
fn looks_like_account_id(token: &str) -> bool {
    token.starts_with('{') || token.contains(':') || is_uuid(token)
}

/// True when `token` matches the canonical `8-4-4-4-12` hex-dash UUID shape.
fn is_uuid(token: &str) -> bool {
    const GROUPS: [usize; 5] = [8, 4, 4, 4, 12];

    let mut parts = token.split('-');
    for &len in &GROUPS {
        match parts.next() {
            Some(part) if part.len() == len && part.bytes().all(|b| b.is_ascii_hexdigit()) => {}
            _ => return false,
        }
    }
    parts.next().is_none()
}

/// True when a git remote URL points at `workspace/repo_slug`.
///
/// Matches case-insensitively and tolerates a trailing `.git` suffix or `/`,
/// covering https (`https://host/ws/slug`), scp-style ssh
/// (`git@host:ws/slug`), and `ssh://` URLs.
fn remote_matches_repo(url: &str, workspace: &str, repo_slug: &str) -> bool {
    let lowered = url.trim().to_lowercase();
    let trimmed = lowered.trim_end_matches('/');
    let trimmed = trimmed.strip_suffix(".git").unwrap_or(trimmed);

    let expected = format!("{}/{}", workspace.to_lowercase(), repo_slug.to_lowercase());
    trimmed.ends_with(&format!("/{}", expected)) || trimmed.ends_with(&format!(":{}", expected))
}

/// Fetch and check out the source branch of `pr` from `origin`.
///
/// Refuses to touch the working tree unless `origin` points at the PR's
/// repository, and bails for PRs from forks (whose branches don't live on
/// `origin`). After checkout, fast-forwards a stale local branch to the freshly
/// fetched remote tip when possible.
async fn checkout_pr_branch(pr: &PullRequest, workspace: &str, repo_slug: &str) -> Result<()> {
    let branch = &pr.source.branch.name;

    // Make sure `origin` actually points at the repo the PR
    // belongs to before touching the working tree.
    let output = std::process::Command::new("git")
        .args(["remote", "get-url", "origin"])
        .output()
        .context("Failed to read the 'origin' remote URL")?;

    if !output.status.success() {
        anyhow::bail!(
            "Could not determine the 'origin' remote URL — is the current directory a git repository with an 'origin' remote?"
        );
    }

    let origin_url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if !remote_matches_repo(&origin_url, workspace, repo_slug) {
        anyhow::bail!(
            "The 'origin' remote ({}) does not point at {}/{}",
            origin_url,
            workspace,
            repo_slug
        );
    }

    // PRs from forks live in a different repository; fetching
    // `origin` would not bring their branch down.
    if let Some(source_repo) = &pr.source.repository {
        let target = format!("{}/{}", workspace, repo_slug);
        if !source_repo.full_name.eq_ignore_ascii_case(&target) {
            anyhow::bail!(
                "PR #{} comes from fork '{}' — add the fork as a remote and fetch manually",
                pr.id,
                source_repo.full_name
            );
        }
    }

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
        // The local branch may be stale; fast-forward it to the
        // freshly fetched remote tip when possible.
        let ff_status = std::process::Command::new("git")
            .args(["merge", "--ff-only", &format!("origin/{}", branch)])
            .status()
            .context("Failed to fast-forward branch")?;

        if ff_status.success() {
            println!("{} Checked out branch {}", "✓".green(), branch);
        } else {
            println!(
                "{} Checked out branch {}, but it has diverged from origin/{} and could not be fast-forwarded; merge or rebase manually.",
                "⚠".yellow(),
                branch,
                branch
            );
        }
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

/// Return the first line of `text`, truncated to at most `max_chars` characters.
fn first_line_truncated(text: &str, max_chars: usize) -> String {
    text.lines()
        .next()
        .unwrap_or("")
        .chars()
        .take(max_chars)
        .collect()
}

/// Extract a `YYYY-MM-DD HH:MM` timestamp from a JSON string field.
fn activity_date(value: &serde_json::Value, pointer: &str) -> String {
    value
        .pointer(pointer)
        .and_then(|v| v.as_str())
        .map(|d| d.chars().take(16).collect::<String>().replace('T', " "))
        .unwrap_or_else(|| "-".to_string())
}

/// Render one PR activity entry as a single typed line.
fn format_activity_line(entry: &PrActivity) -> String {
    fn str_at<'a>(value: &'a serde_json::Value, pointer: &'a str) -> Option<&'a str> {
        value.pointer(pointer).and_then(|v| v.as_str())
    }

    if let Some(comment) = &entry.comment {
        format!(
            "[{}] comment: {} — {}",
            activity_date(comment, "/created_on"),
            str_at(comment, "/user/display_name").unwrap_or("unknown"),
            first_line_truncated(str_at(comment, "/content/raw").unwrap_or(""), 60)
        )
    } else if let Some(approval) = &entry.approval {
        format!(
            "[{}] approval: approved by {}",
            activity_date(approval, "/date"),
            str_at(approval, "/user/display_name").unwrap_or("unknown")
        )
    } else if let Some(changes_requested) = &entry.changes_requested {
        format!(
            "[{}] changes-requested: by {}",
            activity_date(changes_requested, "/date"),
            str_at(changes_requested, "/user/display_name").unwrap_or("unknown")
        )
    } else if let Some(update) = &entry.update {
        format!(
            "[{}] update: {} by {}",
            activity_date(update, "/date"),
            str_at(update, "/state").unwrap_or("updated"),
            str_at(update, "/author/display_name").unwrap_or("unknown")
        )
    } else {
        "unrecognized activity entry".to_string()
    }
}

/// Keep the newest `limit` comments and return them in chronological
/// (oldest -> newest) display order.
fn newest_comments_chronological(
    mut comments: Vec<PullRequestComment>,
    limit: usize,
) -> Vec<PullRequestComment> {
    comments.sort_by_key(|c| std::cmp::Reverse(c.created_on));
    comments.truncate(limit);
    comments.reverse();
    comments
}

#[cfg(test)]
mod tests {
    use super::{
        first_line_truncated, format_activity_line, format_commit_status_state,
        newest_comments_chronological, parse_reviewers, remote_matches_repo, reviewer_ref,
    };
    use crate::models::{CommentContent, PrActivity, PullRequestComment, User};
    use chrono::{DateTime, Utc};

    fn ts(s: &str) -> DateTime<Utc> {
        s.parse().unwrap()
    }

    fn comment(id: u64, created_on: DateTime<Utc>) -> PullRequestComment {
        PullRequestComment {
            id,
            content: CommentContent {
                raw: format!("comment {}", id),
                markup: None,
                html: None,
            },
            user: User {
                uuid: "{user-uuid}".to_string(),
                username: None,
                display_name: "Test User".to_string(),
                account_id: None,
                user_type: "user".to_string(),
                links: None,
            },
            created_on,
            updated_on: None,
            deleted: None,
            inline: None,
            parent: None,
            links: None,
        }
    }

    #[test]
    fn keeps_the_newest_comments_in_ascending_display_order() {
        let t1 = ts("2024-01-01T00:00:00Z");
        let t2 = ts("2024-01-02T00:00:00Z");
        let t3 = ts("2024-01-03T00:00:00Z");
        let input = vec![comment(3, t3), comment(1, t1), comment(2, t2)];

        let result = newest_comments_chronological(input, 2);

        let times: Vec<DateTime<Utc>> = result.iter().map(|c| c.created_on).collect();
        assert_eq!(times, vec![t2, t3]);
    }

    #[test]
    fn limit_larger_than_input_returns_all_in_ascending_order() {
        let t1 = ts("2024-01-01T00:00:00Z");
        let t2 = ts("2024-01-02T00:00:00Z");
        let t3 = ts("2024-01-03T00:00:00Z");
        let input = vec![comment(3, t3), comment(1, t1), comment(2, t2)];

        let result = newest_comments_chronological(input, 10);

        let times: Vec<DateTime<Utc>> = result.iter().map(|c| c.created_on).collect();
        assert_eq!(times, vec![t1, t2, t3]);
    }

    #[test]
    fn remote_matches_repo_accepts_https_urls() {
        assert!(remote_matches_repo(
            "https://bitbucket.org/acme/my-repo",
            "acme",
            "my-repo"
        ));
        assert!(remote_matches_repo(
            "https://user@bitbucket.org/acme/my-repo.git",
            "acme",
            "my-repo"
        ));
    }

    #[test]
    fn remote_matches_repo_accepts_ssh_urls() {
        assert!(remote_matches_repo(
            "git@bitbucket.org:acme/my-repo.git",
            "acme",
            "my-repo"
        ));
        assert!(remote_matches_repo(
            "ssh://git@bitbucket.org/acme/my-repo.git",
            "acme",
            "my-repo"
        ));
    }

    #[test]
    fn remote_matches_repo_tolerates_case_and_trailing_slash() {
        assert!(remote_matches_repo(
            "https://bitbucket.org/Acme/My-Repo.git/",
            "acme",
            "my-repo"
        ));
        assert!(remote_matches_repo(
            "git@bitbucket.org:ACME/MY-REPO",
            "Acme",
            "My-Repo"
        ));
    }

    #[test]
    fn remote_matches_repo_rejects_other_repos() {
        assert!(!remote_matches_repo(
            "https://bitbucket.org/acme/other-repo.git",
            "acme",
            "my-repo"
        ));
        assert!(!remote_matches_repo(
            "git@bitbucket.org:someone-else/my-repo.git",
            "acme",
            "my-repo"
        ));
        // A workspace that merely ends with the expected name must not match.
        assert!(!remote_matches_repo(
            "https://bitbucket.org/other-acme/my-repo",
            "acme",
            "my-repo"
        ));
        assert!(!remote_matches_repo("", "acme", "my-repo"));
    }

    #[test]
    fn parse_reviewers_splits_trims_and_skips_empty_segments() {
        let reviewers = parse_reviewers(" alice , bob,,charlie ");

        let usernames: Vec<Option<String>> = reviewers.iter().map(|r| r.username.clone()).collect();
        assert_eq!(
            usernames,
            vec![
                Some("alice".to_string()),
                Some("bob".to_string()),
                Some("charlie".to_string())
            ]
        );
        assert!(reviewers.iter().all(|r| r.uuid.is_none()));
    }

    #[test]
    fn reviewer_ref_classifies_uuid_and_username_tokens() {
        // Brace-wrapped UUID -> uuid
        let braced = reviewer_ref("{4d9e8f2a-1234-5678-9abc-def012345678}");
        assert_eq!(
            braced.uuid.as_deref(),
            Some("{4d9e8f2a-1234-5678-9abc-def012345678}")
        );
        assert!(braced.username.is_none());

        // Bare UUID (8-4-4-4-12 hex-dash) -> uuid
        let uuid = reviewer_ref("4d9e8f2a-1234-5678-9abc-def012345678");
        assert_eq!(
            uuid.uuid.as_deref(),
            Some("4d9e8f2a-1234-5678-9abc-def012345678")
        );
        assert!(uuid.username.is_none());

        // Account ID (contains a colon) -> uuid
        let account = reviewer_ref("557058:e5f8d5c1-0000-1111-2222-333344445555");
        assert_eq!(
            account.uuid.as_deref(),
            Some("557058:e5f8d5c1-0000-1111-2222-333344445555")
        );
        assert!(account.username.is_none());

        // Plain username -> username
        let username = reviewer_ref("alice");
        assert_eq!(username.username.as_deref(), Some("alice"));
        assert!(username.uuid.is_none());

        // A dashed but non-hex/non-UUID-shaped token stays a username.
        let dashed = reviewer_ref("my-cool-user");
        assert_eq!(dashed.username.as_deref(), Some("my-cool-user"));
        assert!(dashed.uuid.is_none());
    }

    #[test]
    fn format_commit_status_state_preserves_state_text() {
        for state in ["SUCCESSFUL", "FAILED", "INPROGRESS", "STOPPED"] {
            assert!(
                format_commit_status_state(state).contains(state),
                "state text {state} should be preserved",
            );
        }
    }

    #[test]
    fn first_line_truncated_keeps_only_the_first_line() {
        assert_eq!(first_line_truncated("one\ntwo", 60), "one");
        assert_eq!(first_line_truncated("abcdef", 3), "abc");
        assert_eq!(first_line_truncated("", 10), "");
    }

    fn empty_activity() -> PrActivity {
        PrActivity {
            update: None,
            approval: None,
            comment: None,
            changes_requested: None,
        }
    }

    #[test]
    fn format_activity_line_renders_each_entry_type() {
        let update = PrActivity {
            update: Some(serde_json::json!({
                "date": "2024-05-01T10:20:30+00:00",
                "state": "OPEN",
                "author": { "display_name": "Alice" }
            })),
            ..empty_activity()
        };
        assert_eq!(
            format_activity_line(&update),
            "[2024-05-01 10:20] update: OPEN by Alice"
        );

        let approval = PrActivity {
            approval: Some(serde_json::json!({
                "date": "2024-05-02T11:00:00+00:00",
                "user": { "display_name": "Bob" }
            })),
            ..empty_activity()
        };
        assert_eq!(
            format_activity_line(&approval),
            "[2024-05-02 11:00] approval: approved by Bob"
        );

        let comment = PrActivity {
            comment: Some(serde_json::json!({
                "created_on": "2024-05-03T09:15:00+00:00",
                "user": { "display_name": "Carol" },
                "content": { "raw": "Looks good\nSecond line ignored" }
            })),
            ..empty_activity()
        };
        assert_eq!(
            format_activity_line(&comment),
            "[2024-05-03 09:15] comment: Carol — Looks good"
        );

        let changes_requested = PrActivity {
            changes_requested: Some(serde_json::json!({
                "date": "2024-05-04T08:00:00+00:00",
                "user": { "display_name": "Dave" }
            })),
            ..empty_activity()
        };
        assert_eq!(
            format_activity_line(&changes_requested),
            "[2024-05-04 08:00] changes-requested: by Dave"
        );

        assert_eq!(
            format_activity_line(&empty_activity()),
            "unrecognized activity entry"
        );
    }
}
