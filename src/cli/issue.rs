use std::collections::HashSet;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::{Subcommand, ValueEnum};
use colored::Colorize;
use tabled::{Table, Tabled};

use super::{confirm_or_abort, parse_repo};
use crate::api::BitbucketClient;
use crate::api::issues::{IssueListFilters, IssueUpdate};
use crate::cli::pagination::{MAX_LIMIT, effective_limit as capped_limit};
use crate::models::{
    ComponentName, CreateIssueRequest, IssueAttachment, IssueChange, IssueComment,
    IssueContentRequest, IssueKind, IssueMetaItem, IssuePriority, IssueState, MilestoneName,
    UserAccountId, VersionName,
};

#[derive(Subcommand)]
pub enum IssueCommands {
    /// List issues
    List {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Filter by state
        #[arg(short, long, value_enum)]
        state: Option<IssueStateArg>,

        /// Filter by kind
        #[arg(short, long, value_enum)]
        kind: Option<IssueKindArg>,

        /// Filter by priority
        #[arg(short, long, value_enum)]
        priority: Option<IssuePriorityArg>,

        /// Filter by assignee account id
        #[arg(long)]
        assignee: Option<String>,

        /// Filter by reporter account id
        #[arg(long)]
        reporter: Option<String>,

        /// Raw BBQL query (overrides the individual filter flags)
        #[arg(short, long)]
        query: Option<String>,

        /// Sort field, e.g. "-updated_on"
        #[arg(long)]
        sort: Option<String>,

        /// Page number
        #[arg(long)]
        page: Option<u32>,

        /// Number of results (capped at 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// View issue details
    View {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue ID
        id: u64,

        /// Also show the issue's comments
        #[arg(short, long)]
        comments: bool,

        /// Open in browser
        #[arg(short, long)]
        web: bool,
    },

    /// Create a new issue
    #[command(disable_version_flag = true)]
    Create {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue title
        #[arg(short, long)]
        title: String,

        /// Issue description
        #[arg(short = 'b', long)]
        body: Option<String>,

        /// Issue type
        #[arg(short, long, value_enum, default_value = "bug")]
        kind: IssueKindArg,

        /// Issue priority
        #[arg(short, long, value_enum, default_value = "major")]
        priority: IssuePriorityArg,

        /// Assignee account id
        #[arg(short, long)]
        assignee: Option<String>,

        /// Component name
        #[arg(short, long)]
        component: Option<String>,

        /// Milestone name
        #[arg(short, long)]
        milestone: Option<String>,

        /// Version name
        #[arg(long)]
        version: Option<String>,
    },

    /// Edit an issue's fields
    Edit {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue ID
        id: u64,

        /// New title
        #[arg(short, long)]
        title: Option<String>,

        /// New description
        #[arg(short = 'b', long)]
        body: Option<String>,

        /// New kind
        #[arg(short, long, value_enum)]
        kind: Option<IssueKindArg>,

        /// New priority
        #[arg(short, long, value_enum)]
        priority: Option<IssuePriorityArg>,

        /// New assignee account id
        #[arg(short, long)]
        assignee: Option<String>,

        /// New state
        #[arg(short, long, value_enum)]
        state: Option<IssueStateArg>,
    },

    /// Delete an issue
    Delete {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue ID
        id: u64,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Add a comment to an issue
    Comment {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue ID
        id: u64,

        /// Comment text
        #[arg(short, long)]
        body: String,
    },

    /// List comments on an issue
    Comments {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue ID
        id: u64,

        /// Number of results (capped at 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// Edit a comment on an issue
    EditComment {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue ID
        id: u64,

        /// Comment ID
        comment_id: u64,

        /// New comment text
        #[arg(short, long)]
        body: String,
    },

    /// Delete a comment from an issue
    DeleteComment {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue ID
        id: u64,

        /// Comment ID
        comment_id: u64,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// List the change log of an issue
    Changes {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue ID
        id: u64,

        /// Number of results (capped at 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// Vote for an issue
    Vote {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue ID
        id: u64,
    },

    /// Remove your vote from an issue
    Unvote {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue ID
        id: u64,
    },

    /// Watch an issue
    Watch {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue ID
        id: u64,
    },

    /// Stop watching an issue
    Unwatch {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue ID
        id: u64,
    },

    /// List the components defined in the issue tracker
    Components {
        /// Repository in format workspace/repo-slug
        repo: String,
    },

    /// List the milestones defined in the issue tracker
    Milestones {
        /// Repository in format workspace/repo-slug
        repo: String,
    },

    /// List the versions defined in the issue tracker
    Versions {
        /// Repository in format workspace/repo-slug
        repo: String,
    },

    /// Manage files attached to an issue
    Attachment {
        #[command(subcommand)]
        command: AttachmentCommands,
    },

    /// Close an issue
    Close {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue ID
        id: u64,
    },

    /// Reopen an issue
    Reopen {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue ID
        id: u64,
    },
}

#[derive(Subcommand)]
pub enum AttachmentCommands {
    /// List files attached to an issue
    List {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue ID
        id: u64,
    },

    /// Attach one or more files to an issue
    Add {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue ID
        id: u64,

        /// Files to attach
        #[arg(required = true)]
        files: Vec<PathBuf>,
    },

    /// Delete an attachment from an issue
    Delete {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue ID
        id: u64,

        /// Attachment path (the filename shown by `issue attachment list`)
        path: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(ValueEnum, Clone)]
pub enum IssueStateArg {
    New,
    Open,
    Resolved,
    OnHold,
    Invalid,
    Duplicate,
    Wontfix,
    Closed,
}

impl From<IssueStateArg> for IssueState {
    fn from(state: IssueStateArg) -> Self {
        match state {
            IssueStateArg::New => IssueState::New,
            IssueStateArg::Open => IssueState::Open,
            IssueStateArg::Resolved => IssueState::Resolved,
            IssueStateArg::OnHold => IssueState::OnHold,
            IssueStateArg::Invalid => IssueState::Invalid,
            IssueStateArg::Duplicate => IssueState::Duplicate,
            IssueStateArg::Wontfix => IssueState::Wontfix,
            IssueStateArg::Closed => IssueState::Closed,
        }
    }
}

#[derive(ValueEnum, Clone)]
pub enum IssueKindArg {
    Bug,
    Enhancement,
    Proposal,
    Task,
}

impl From<IssueKindArg> for IssueKind {
    fn from(kind: IssueKindArg) -> Self {
        match kind {
            IssueKindArg::Bug => IssueKind::Bug,
            IssueKindArg::Enhancement => IssueKind::Enhancement,
            IssueKindArg::Proposal => IssueKind::Proposal,
            IssueKindArg::Task => IssueKind::Task,
        }
    }
}

#[derive(ValueEnum, Clone)]
pub enum IssuePriorityArg {
    Trivial,
    Minor,
    Major,
    Critical,
    Blocker,
}

impl From<IssuePriorityArg> for IssuePriority {
    fn from(priority: IssuePriorityArg) -> Self {
        match priority {
            IssuePriorityArg::Trivial => IssuePriority::Trivial,
            IssuePriorityArg::Minor => IssuePriority::Minor,
            IssuePriorityArg::Major => IssuePriority::Major,
            IssuePriorityArg::Critical => IssuePriority::Critical,
            IssuePriorityArg::Blocker => IssuePriority::Blocker,
        }
    }
}

#[derive(Tabled)]
struct IssueRow {
    #[tabled(rename = "ID")]
    id: u64,
    #[tabled(rename = "TITLE")]
    title: String,
    #[tabled(rename = "STATE")]
    state: String,
    #[tabled(rename = "KIND")]
    kind: String,
    #[tabled(rename = "PRIORITY")]
    priority: String,
}

#[derive(Tabled)]
struct CommentRow {
    #[tabled(rename = "ID")]
    id: u64,
    #[tabled(rename = "AUTHOR")]
    author: String,
    #[tabled(rename = "CREATED")]
    created: String,
    #[tabled(rename = "COMMENT")]
    comment: String,
}

#[derive(Tabled)]
struct ChangeRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "USER")]
    user: String,
    #[tabled(rename = "DATE")]
    date: String,
    #[tabled(rename = "CHANGED")]
    changed: String,
}

#[derive(Tabled)]
struct MetaRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "NAME")]
    name: String,
}

#[derive(Tabled)]
struct AttachmentRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "LINK")]
    link: String,
}

impl IssueCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            IssueCommands::List {
                repo,
                state,
                kind,
                priority,
                assignee,
                reporter,
                query,
                sort,
                page,
                limit,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                let capped = capped_limit(limit);
                if capped != limit && !super::output_json() {
                    println!(
                        "{} Limit capped at {} (the Bitbucket API maximum)",
                        "ℹ".blue(),
                        MAX_LIMIT
                    );
                }

                let client = BitbucketClient::from_stored().await?;

                let filters = IssueListFilters {
                    state: state.map(Into::into),
                    kind: kind.map(Into::into),
                    priority: priority.map(Into::into),
                    assignee: assignee.as_deref(),
                    reporter: reporter.as_deref(),
                    query: query.as_deref(),
                    sort: sort.as_deref(),
                    page,
                    pagelen: Some(capped),
                };

                let issues = client
                    .list_issues_filtered(&workspace, &repo_slug, &filters)
                    .await?;

                if super::output_json() {
                    return super::print_json(&issues.values);
                }

                if issues.values.is_empty() {
                    println!("No issues found");
                    return Ok(());
                }

                let rows: Vec<IssueRow> = issues
                    .values
                    .iter()
                    .map(|issue| IssueRow {
                        id: issue.id,
                        title: issue.title.chars().take(50).collect(),
                        state: format_state(&issue.state),
                        kind: format!("{}", issue.kind),
                        priority: format_priority(&issue.priority),
                    })
                    .collect();

                let table = Table::new(rows).to_string();
                println!("{}", table);

                if issues.next.is_some() {
                    println!(
                        "{} More issues available; use --page/--limit to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            IssueCommands::View {
                repo,
                id,
                comments,
                web,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let issue = client.get_issue(&workspace, &repo_slug, id).await?;

                // In json mode, skip the browser and fall through to emit the
                // issue JSON below.
                if web && !super::output_json() {
                    if let Some(links) = &issue.links {
                        if let Some(html) = &links.html {
                            open::that(&html.href)?;
                            println!("Opened {} in browser", html.href.cyan());
                            return Ok(());
                        }
                    }
                    anyhow::bail!("Could not find issue URL");
                }

                if super::output_json() {
                    if comments {
                        let thread = client
                            .list_issue_comments(&workspace, &repo_slug, id, Some(MAX_LIMIT))
                            .await?;
                        return super::print_json(&serde_json::json!({
                            "issue": issue,
                            "comments": thread.values,
                        }));
                    }
                    return super::print_json(&issue);
                }

                println!(
                    "{} {} #{}",
                    format_state(&issue.state),
                    issue.title.bold(),
                    issue.id
                );
                println!("{}", "─".repeat(60));

                println!("{} {}", "Kind:".dimmed(), issue.kind);
                println!(
                    "{} {}",
                    "Priority:".dimmed(),
                    format_priority(&issue.priority)
                );

                if let Some(reporter) = &issue.reporter {
                    println!("{} {}", "Reporter:".dimmed(), reporter.display_name);
                }

                if let Some(assignee) = &issue.assignee {
                    println!("{} {}", "Assignee:".dimmed(), assignee.display_name);
                }

                println!(
                    "{} {}",
                    "Created:".dimmed(),
                    issue.created_on.format("%Y-%m-%d %H:%M")
                );

                if let Some(updated) = issue.updated_on {
                    println!(
                        "{} {}",
                        "Updated:".dimmed(),
                        updated.format("%Y-%m-%d %H:%M")
                    );
                }

                if let Some(votes) = issue.votes {
                    if votes > 0 {
                        println!("{} {}", "Votes:".dimmed(), votes);
                    }
                }

                if let Some(content) = &issue.content {
                    if let Some(raw) = &content.raw {
                        if !raw.is_empty() {
                            println!();
                            println!("{}", raw);
                        }
                    }
                }

                if let Some(links) = &issue.links {
                    if let Some(html) = &links.html {
                        println!();
                        println!("{} {}", "URL:".dimmed(), html.href.cyan());
                    }
                }

                if comments {
                    let thread = client
                        .list_issue_comments(&workspace, &repo_slug, id, Some(MAX_LIMIT))
                        .await?;

                    println!();
                    if thread.values.is_empty() {
                        println!("No comments");
                    } else {
                        println!("{}", format!("Comments ({})", thread.values.len()).bold());
                        for comment in &thread.values {
                            println!("{}", "─".repeat(60));
                            print_comment(comment);
                        }
                        if thread.next.is_some() {
                            println!(
                                "{} More comments available; use 'issue comments' to page through them.",
                                "ℹ".blue()
                            );
                        }
                    }
                }

                Ok(())
            }

            IssueCommands::Create {
                repo,
                title,
                body,
                kind,
                priority,
                assignee,
                component,
                milestone,
                version,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let request = CreateIssueRequest {
                    title,
                    content: body.map(|b| IssueContentRequest { raw: b }),
                    kind: Some(kind.into()),
                    priority: Some(priority.into()),
                    assignee: assignee.map(|account_id| UserAccountId { account_id }),
                    component: component.map(|name| ComponentName { name }),
                    milestone: milestone.map(|name| MilestoneName { name }),
                    version: version.map(|name| VersionName { name }),
                };

                let issue = client
                    .create_issue(&workspace, &repo_slug, &request)
                    .await?;

                if super::output_json() {
                    return super::print_json(&issue);
                }

                println!("{} Created issue #{}", "✓".green(), issue.id);

                if let Some(links) = &issue.links {
                    if let Some(html) = &links.html {
                        println!("{} {}", "URL:".dimmed(), html.href.cyan());
                    }
                }

                Ok(())
            }

            IssueCommands::Edit {
                repo,
                id,
                title,
                body,
                kind,
                priority,
                assignee,
                state,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                let update = IssueUpdate {
                    title: title.as_deref(),
                    content: body.as_deref(),
                    kind: kind.map(Into::into),
                    priority: priority.map(Into::into),
                    assignee: assignee.as_deref(),
                    state: state.map(Into::into),
                };

                if update.is_empty() {
                    anyhow::bail!(
                        "Nothing to edit: pass at least one of --title, --body, --kind, --priority, --assignee or --state"
                    );
                }

                let client = BitbucketClient::from_stored().await?;
                let issue = client
                    .update_issue_full(&workspace, &repo_slug, id, &update)
                    .await?;

                if super::output_json() {
                    return super::print_json(&issue);
                }

                println!("{} Updated issue #{}", "✓".green(), issue.id);

                Ok(())
            }

            IssueCommands::Delete { repo, id, yes } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                if !yes
                    && !confirm_or_abort(format!(
                        "Delete issue {} from {}?",
                        format!("#{}", id).red(),
                        repo
                    ))?
                {
                    return Ok(());
                }

                let client = BitbucketClient::from_stored().await?;
                client.delete_issue(&workspace, &repo_slug, id).await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!("{} Deleted issue #{}", "✓".green(), id);

                Ok(())
            }

            IssueCommands::Comment { repo, id, body } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let comment = client
                    .add_issue_comment(&workspace, &repo_slug, id, &body)
                    .await?;

                if super::output_json() {
                    return super::print_json(&comment);
                }

                println!("{} Added comment to issue #{}", "✓".green(), id);

                Ok(())
            }

            IssueCommands::Comments { repo, id, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                let capped = capped_limit(limit);
                if capped != limit && !super::output_json() {
                    println!(
                        "{} Limit capped at {} (the Bitbucket API maximum)",
                        "ℹ".blue(),
                        MAX_LIMIT
                    );
                }

                let client = BitbucketClient::from_stored().await?;
                let comments = client
                    .list_issue_comments(&workspace, &repo_slug, id, Some(capped))
                    .await?;

                if super::output_json() {
                    return super::print_json(&comments.values);
                }

                if comments.values.is_empty() {
                    println!("No comments found on issue #{}", id);
                    return Ok(());
                }

                let rows: Vec<CommentRow> = comments
                    .values
                    .iter()
                    .map(|comment| CommentRow {
                        id: comment.id,
                        author: comment.user.display_name.clone(),
                        created: comment.created_on.format("%Y-%m-%d %H:%M").to_string(),
                        comment: summarize_text(comment.content.raw.as_deref()),
                    })
                    .collect();

                let table = Table::new(rows).to_string();
                println!("{}", table);

                if comments.next.is_some() {
                    println!(
                        "{} More comments available; use --limit to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            IssueCommands::EditComment {
                repo,
                id,
                comment_id,
                body,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let comment = client
                    .update_issue_comment(&workspace, &repo_slug, id, comment_id, &body)
                    .await?;

                if super::output_json() {
                    return super::print_json(&comment);
                }

                println!(
                    "{} Updated comment {} on issue #{}",
                    "✓".green(),
                    comment_id,
                    id
                );

                Ok(())
            }

            IssueCommands::DeleteComment {
                repo,
                id,
                comment_id,
                yes,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                if !yes
                    && !confirm_or_abort(format!(
                        "Delete comment {} from issue #{} in {}?",
                        comment_id.to_string().red(),
                        id,
                        repo
                    ))?
                {
                    return Ok(());
                }

                let client = BitbucketClient::from_stored().await?;
                client
                    .delete_issue_comment(&workspace, &repo_slug, id, comment_id)
                    .await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!(
                    "{} Deleted comment {} from issue #{}",
                    "✓".green(),
                    comment_id,
                    id
                );

                Ok(())
            }

            IssueCommands::Changes { repo, id, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                let capped = capped_limit(limit);
                if capped != limit && !super::output_json() {
                    println!(
                        "{} Limit capped at {} (the Bitbucket API maximum)",
                        "ℹ".blue(),
                        MAX_LIMIT
                    );
                }

                let client = BitbucketClient::from_stored().await?;
                let changes = client
                    .list_issue_changes(&workspace, &repo_slug, id, Some(capped))
                    .await?;

                if super::output_json() {
                    return super::print_json(&changes.values);
                }

                if changes.values.is_empty() {
                    println!("No changes found for issue #{}", id);
                    return Ok(());
                }

                let rows: Vec<ChangeRow> = changes.values.iter().map(change_row).collect();

                let table = Table::new(rows).to_string();
                println!("{}", table);

                if changes.next.is_some() {
                    println!(
                        "{} More changes available; use --limit to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            IssueCommands::Vote { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                client.vote_issue(&workspace, &repo_slug, id).await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!("{} Voted for issue #{}", "✓".green(), id);

                Ok(())
            }

            IssueCommands::Unvote { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                client.unvote_issue(&workspace, &repo_slug, id).await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!("{} Removed vote from issue #{}", "✓".green(), id);

                Ok(())
            }

            IssueCommands::Watch { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                client.watch_issue(&workspace, &repo_slug, id).await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!("{} Watching issue #{}", "✓".green(), id);

                Ok(())
            }

            IssueCommands::Unwatch { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                client.unwatch_issue(&workspace, &repo_slug, id).await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!("{} Stopped watching issue #{}", "✓".green(), id);

                Ok(())
            }

            IssueCommands::Components { repo } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let items = client.list_issue_components(&workspace, &repo_slug).await?;

                if super::output_json() {
                    return super::print_json(&items);
                }

                print_meta_items(&items, "components");

                Ok(())
            }

            IssueCommands::Milestones { repo } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let items = client.list_issue_milestones(&workspace, &repo_slug).await?;

                if super::output_json() {
                    return super::print_json(&items);
                }

                print_meta_items(&items, "milestones");

                Ok(())
            }

            IssueCommands::Versions { repo } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let items = client.list_issue_versions(&workspace, &repo_slug).await?;

                if super::output_json() {
                    return super::print_json(&items);
                }

                print_meta_items(&items, "versions");

                Ok(())
            }

            IssueCommands::Attachment { command } => command.run().await,

            IssueCommands::Close { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let issue = client
                    .update_issue(
                        &workspace,
                        &repo_slug,
                        id,
                        None,
                        None,
                        Some(IssueState::Closed),
                    )
                    .await?;

                if super::output_json() {
                    return super::print_json(&issue);
                }

                println!("{} Closed issue #{}", "✓".green(), id);

                Ok(())
            }

            IssueCommands::Reopen { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let issue = client
                    .update_issue(
                        &workspace,
                        &repo_slug,
                        id,
                        None,
                        None,
                        Some(IssueState::Open),
                    )
                    .await?;

                if super::output_json() {
                    return super::print_json(&issue);
                }

                println!("{} Reopened issue #{}", "✓".green(), id);

                Ok(())
            }
        }
    }
}

impl AttachmentCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            AttachmentCommands::List { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let attachments = client
                    .list_issue_attachments(&workspace, &repo_slug, id)
                    .await?;

                if super::output_json() {
                    return super::print_json(&attachments);
                }

                if attachments.is_empty() {
                    println!("No attachments found on issue #{}", id);
                    return Ok(());
                }

                let rows: Vec<AttachmentRow> = attachments
                    .iter()
                    .map(|attachment| AttachmentRow {
                        name: attachment.name.clone().unwrap_or_else(|| "-".to_string()),
                        link: attachment_link(attachment),
                    })
                    .collect();

                let table = Table::new(rows).to_string();
                println!("{}", table);

                Ok(())
            }

            AttachmentCommands::Add { repo, id, files } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                // Resolve each path to (upload-name, path) up front so a bad
                // path fails before we open a network connection.
                let uploads: Vec<(String, PathBuf)> = files
                    .iter()
                    .map(|p| Ok((attachment_name_for(p)?, p.clone())))
                    .collect::<Result<_>>()?;

                // Bitbucket keys attachments by filename, so two inputs with
                // the same basename would silently overwrite each other
                // server-side.
                let mut seen = HashSet::new();
                for (name, _) in &uploads {
                    if !seen.insert(name.as_str()) {
                        anyhow::bail!(
                            "Duplicate attachment name '{}': multiple input files share this filename; Bitbucket stores attachments by filename, so one would overwrite the other",
                            name
                        );
                    }
                }

                let client = BitbucketClient::from_stored().await?;
                client
                    .upload_issue_attachments(&workspace, &repo_slug, id, &uploads)
                    .await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                for (name, _) in &uploads {
                    println!("{} Attached {} to issue #{}", "✓".green(), name.cyan(), id);
                }

                Ok(())
            }

            AttachmentCommands::Delete {
                repo,
                id,
                path,
                yes,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                if !yes
                    && !confirm_or_abort(format!(
                        "Delete attachment {} from issue #{} in {}?",
                        path.red(),
                        id,
                        repo
                    ))?
                {
                    return Ok(());
                }

                let client = BitbucketClient::from_stored().await?;
                client
                    .delete_issue_attachment(&workspace, &repo_slug, id, &path)
                    .await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!(
                    "{} Deleted attachment {} from issue #{}",
                    "✓".green(),
                    path,
                    id
                );

                Ok(())
            }
        }
    }
}

fn format_state(state: &IssueState) -> String {
    match state {
        IssueState::New => "NEW".cyan().to_string(),
        IssueState::Open => "OPEN".green().to_string(),
        IssueState::Resolved => "RESOLVED".blue().to_string(),
        IssueState::OnHold => "ON HOLD".yellow().to_string(),
        IssueState::Invalid => "INVALID".dimmed().to_string(),
        IssueState::Duplicate => "DUPLICATE".dimmed().to_string(),
        IssueState::Wontfix => "WONTFIX".dimmed().to_string(),
        IssueState::Closed => "CLOSED".purple().to_string(),
    }
}

fn format_priority(priority: &IssuePriority) -> String {
    match priority {
        IssuePriority::Trivial => "trivial".dimmed().to_string(),
        IssuePriority::Minor => "minor".normal().to_string(),
        IssuePriority::Major => "major".yellow().to_string(),
        IssuePriority::Critical => "critical".red().to_string(),
        IssuePriority::Blocker => "blocker".red().bold().to_string(),
    }
}

/// Print a single comment in thread style: header line, then the raw body.
fn print_comment(comment: &IssueComment) {
    println!(
        "{} {} {}",
        format!("#{}", comment.id).dimmed(),
        comment.user.display_name.bold(),
        comment
            .created_on
            .format("%Y-%m-%d %H:%M")
            .to_string()
            .dimmed()
    );
    if let Some(raw) = &comment.content.raw {
        if !raw.is_empty() {
            println!("{}", raw);
        }
    }
}

/// Flatten a possibly multi-line text into a single-line table cell,
/// truncated to 60 characters. Empty or missing text renders as "-".
fn summarize_text(raw: Option<&str>) -> String {
    let flattened = raw
        .unwrap_or("")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    if flattened.is_empty() {
        return "-".to_string();
    }

    flattened.chars().take(60).collect()
}

/// Build a table row for one issue change-log entry: who, when, and the
/// names of the fields that changed.
fn change_row(change: &IssueChange) -> ChangeRow {
    ChangeRow {
        id: change
            .id
            .map(|v| v.to_string())
            .unwrap_or_else(|| "-".to_string()),
        user: change
            .user
            .as_ref()
            .map(|u| u.display_name.clone())
            .unwrap_or_else(|| "-".to_string()),
        date: change
            .created_on
            .map(|d| d.format("%Y-%m-%d %H:%M").to_string())
            .unwrap_or_else(|| "-".to_string()),
        changed: changed_keys(change),
    }
}

/// The comma-separated names of the fields touched by a change-log entry.
fn changed_keys(change: &IssueChange) -> String {
    let keys = change
        .changes
        .as_ref()
        .and_then(|v| v.as_object())
        .map(|obj| obj.keys().cloned().collect::<Vec<_>>().join(", "))
        .unwrap_or_default();

    if keys.is_empty() {
        "-".to_string()
    } else {
        keys
    }
}

/// Print components/milestones/versions as an id + name table.
fn print_meta_items(items: &[IssueMetaItem], noun: &str) {
    if items.is_empty() {
        println!("No {} found", noun);
        return;
    }

    let rows: Vec<MetaRow> = items
        .iter()
        .map(|item| MetaRow {
            id: item
                .id
                .map(|v| v.to_string())
                .unwrap_or_else(|| "-".to_string()),
            name: item.name.clone().unwrap_or_else(|| "-".to_string()),
        })
        .collect();

    println!("{}", Table::new(rows));
}

/// Extract the self link of an attachment. The API serves `links.self.href`
/// as either a string or an array of strings, so both shapes are handled.
fn attachment_link(attachment: &IssueAttachment) -> String {
    match attachment
        .links
        .as_ref()
        .and_then(|links| links.pointer("/self/href"))
    {
        Some(serde_json::Value::String(s)) => s.clone(),
        Some(serde_json::Value::Array(hrefs)) => hrefs
            .iter()
            .filter_map(|v| v.as_str())
            .next()
            .map(str::to_string)
            .unwrap_or_else(|| "-".to_string()),
        _ => "-".to_string(),
    }
}

/// Extract the file name (final path component) from a path, for use as the
/// uploaded attachment name.
fn attachment_name_for(path: &Path) -> Result<String> {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .with_context(|| format!("Could not determine a file name for '{}'", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn summarize_text_flattens_newlines_and_truncates() {
        assert_eq!(summarize_text(Some("a\nb\tc")), "a b c");
        assert_eq!(summarize_text(None), "-");
        assert_eq!(summarize_text(Some("   ")), "-");
        let long = "x".repeat(80);
        assert_eq!(summarize_text(Some(&long)).chars().count(), 60);
    }

    #[test]
    fn attachment_link_handles_string_and_array_hrefs() {
        let string_href = IssueAttachment {
            name: Some("a.png".to_string()),
            links: Some(serde_json::json!({"self": {"href": "https://x/a.png"}})),
        };
        assert_eq!(attachment_link(&string_href), "https://x/a.png");

        let array_href = IssueAttachment {
            name: Some("b.png".to_string()),
            links: Some(
                serde_json::json!({"self": {"href": ["https://x/b.png", "https://y/b.png"]}}),
            ),
        };
        assert_eq!(attachment_link(&array_href), "https://x/b.png");

        let missing = IssueAttachment {
            name: None,
            links: None,
        };
        assert_eq!(attachment_link(&missing), "-");
    }

    #[test]
    fn changed_keys_joins_field_names_of_a_change_entry() {
        let change = IssueChange {
            id: Some(1),
            user: None,
            message: None,
            created_on: None,
            changes: Some(serde_json::json!({
                "state": {"old": "new", "new": "open"},
                "assignee": {"old": "", "new": "someone"},
            })),
        };
        assert_eq!(changed_keys(&change), "assignee, state");

        let empty = IssueChange {
            id: None,
            user: None,
            message: None,
            created_on: None,
            changes: None,
        };
        assert_eq!(changed_keys(&empty), "-");
    }

    #[test]
    fn attachment_name_for_extracts_basename_and_errors_without_one() {
        assert_eq!(
            attachment_name_for(Path::new("/some/dir/shot.png")).unwrap(),
            "shot.png"
        );
        assert_eq!(
            attachment_name_for(Path::new("shot.png")).unwrap(),
            "shot.png"
        );
        // A path ending in ".." has no final file-name component.
        assert!(attachment_name_for(Path::new("..")).is_err());
    }

    #[test]
    fn change_row_renders_populated_and_empty_changes() {
        let user = crate::models::User {
            uuid: "u-1".to_string(),
            username: None,
            display_name: "Alice".to_string(),
            account_id: None,
            user_type: "user".to_string(),
            links: None,
        };
        let created: chrono::DateTime<chrono::Utc> = "2024-01-15T10:30:00Z".parse().unwrap();
        let populated = IssueChange {
            id: Some(7),
            user: Some(user),
            message: None,
            created_on: Some(created),
            changes: Some(serde_json::json!({"state": {"old": "new", "new": "open"}})),
        };
        let row = change_row(&populated);
        assert_eq!(row.id, "7");
        assert_eq!(row.user, "Alice");
        assert_eq!(row.date, "2024-01-15 10:30");
        assert_eq!(row.changed, "state");

        let empty = IssueChange {
            id: None,
            user: None,
            message: None,
            created_on: None,
            changes: None,
        };
        let row = change_row(&empty);
        assert_eq!(row.id, "-");
        assert_eq!(row.user, "-");
        assert_eq!(row.date, "-");
        assert_eq!(row.changed, "-");
    }
}
