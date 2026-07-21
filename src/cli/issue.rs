use anyhow::Result;
use clap::{Subcommand, ValueEnum};
use colored::Colorize;
use tabled::{Table, Tabled};

use super::parse_repo;
use crate::api::BitbucketClient;
use crate::models::{
    CreateIssueRequest, IssueContentRequest, IssueKind, IssuePriority, IssueState,
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

        /// Number of results
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// View issue details
    View {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Issue ID
        id: u64,

        /// Open in browser
        #[arg(short, long)]
        web: bool,
    },

    /// Create a new issue
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

impl IssueCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            IssueCommands::List { repo, state, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let issues = client
                    .list_issues(
                        &workspace,
                        &repo_slug,
                        state.map(|s| s.into()),
                        None,
                        Some(limit),
                    )
                    .await?;

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

                Ok(())
            }

            IssueCommands::View { repo, id, web } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let issue = client.get_issue(&workspace, &repo_slug, id).await?;

                if web {
                    if let Some(links) = &issue.links {
                        if let Some(html) = &links.html {
                            open::that(&html.href)?;
                            println!("Opened {} in browser", html.href.cyan());
                            return Ok(());
                        }
                    }
                    anyhow::bail!("Could not find issue URL");
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

                Ok(())
            }

            IssueCommands::Create {
                repo,
                title,
                body,
                kind,
                priority,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let request = CreateIssueRequest {
                    title,
                    content: body.map(|b| IssueContentRequest { raw: b }),
                    kind: Some(kind.into()),
                    priority: Some(priority.into()),
                    assignee: None,
                    component: None,
                    milestone: None,
                    version: None,
                };

                let issue = client
                    .create_issue(&workspace, &repo_slug, &request)
                    .await?;

                println!("{} Created issue #{}", "✓".green(), issue.id);

                if let Some(links) = &issue.links {
                    if let Some(html) = &links.html {
                        println!("{} {}", "URL:".dimmed(), html.href.cyan());
                    }
                }

                Ok(())
            }

            IssueCommands::Comment { repo, id, body } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                client
                    .add_issue_comment(&workspace, &repo_slug, id, &body)
                    .await?;

                println!("{} Added comment to issue #{}", "✓".green(), id);

                Ok(())
            }

            IssueCommands::Close { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                client
                    .update_issue(
                        &workspace,
                        &repo_slug,
                        id,
                        None,
                        None,
                        Some(IssueState::Closed),
                    )
                    .await?;

                println!("{} Closed issue #{}", "✓".green(), id);

                Ok(())
            }

            IssueCommands::Reopen { repo, id } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                client
                    .update_issue(
                        &workspace,
                        &repo_slug,
                        id,
                        None,
                        None,
                        Some(IssueState::Open),
                    )
                    .await?;

                println!("{} Reopened issue #{}", "✓".green(), id);

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
