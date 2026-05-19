use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use tabled::{Table, Tabled};

use crate::api::BitbucketClient;
use crate::models::{PipelineResultName, PipelineStateName, TriggerPipelineRequest};

#[derive(Subcommand)]
pub enum PipelineCommands {
    /// List pipelines
    List {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Number of results
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// View pipeline details
    View {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pipeline build number
        #[arg(short, long)]
        build: u64,

        /// Show step logs
        #[arg(short, long)]
        logs: bool,
    },

    /// Trigger a new pipeline
    Trigger {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Branch to run pipeline on
        #[arg(short, long, default_value = "main")]
        branch: String,

        /// Custom pipeline name (from bitbucket-pipelines.yml)
        #[arg(short, long)]
        pipeline: Option<String>,

        /// Wait for pipeline to complete
        #[arg(short, long)]
        wait: bool,
    },

    /// Stop a running pipeline
    Stop {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pipeline build number
        #[arg(short, long)]
        build: u64,
    },
}

#[derive(Tabled)]
struct PipelineRow {
    #[tabled(rename = "#")]
    build: u64,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "BRANCH")]
    branch: String,
    #[tabled(rename = "TRIGGERED")]
    triggered: String,
    #[tabled(rename = "DURATION")]
    duration: String,
}

impl PipelineCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            PipelineCommands::List { repo, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let pipelines = client
                    .list_pipelines(&workspace, &repo_slug, None, Some(limit))
                    .await?;

                if pipelines.values.is_empty() {
                    println!("No pipelines found");
                    return Ok(());
                }

                let rows: Vec<PipelineRow> = pipelines
                    .values
                    .iter()
                    .map(|p| {
                        let duration = if let Some(seconds) = p.build_seconds_used {
                            format_duration(seconds)
                        } else if p.state.name == PipelineStateName::InProgress {
                            "running...".to_string()
                        } else {
                            "-".to_string()
                        };

                        PipelineRow {
                            build: p.build_number,
                            status: format_status(
                                &p.state.name,
                                p.state.result.as_ref().map(|r| &r.name),
                            ),
                            branch: p.target.ref_name.clone().unwrap_or_else(|| "-".to_string()),
                            triggered: p.created_on.format("%Y-%m-%d %H:%M").to_string(),
                            duration,
                        }
                    })
                    .collect();

                let table = Table::new(rows).to_string();
                println!("{}", table);

                Ok(())
            }

            PipelineCommands::View { repo, build, logs } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let pipeline = client
                    .get_pipeline_by_build_number(&workspace, &repo_slug, build)
                    .await?;

                println!(
                    "{} Pipeline #{} - {}",
                    format_status(
                        &pipeline.state.name,
                        pipeline.state.result.as_ref().map(|r| &r.name)
                    ),
                    pipeline.build_number,
                    pipeline.target.ref_name.as_deref().unwrap_or("unknown")
                );
                println!("{}", "─".repeat(60));

                if let Some(creator) = &pipeline.creator {
                    println!("{} {}", "Triggered by:".dimmed(), creator.display_name);
                }

                if let Some(trigger) = &pipeline.trigger {
                    println!("{} {}", "Trigger type:".dimmed(), trigger.trigger_type);
                }

                println!(
                    "{} {}",
                    "Started:".dimmed(),
                    pipeline.created_on.format("%Y-%m-%d %H:%M:%S")
                );

                if let Some(completed) = pipeline.completed_on {
                    println!(
                        "{} {}",
                        "Completed:".dimmed(),
                        completed.format("%Y-%m-%d %H:%M:%S")
                    );
                }

                if let Some(seconds) = pipeline.build_seconds_used {
                    println!("{} {}", "Duration:".dimmed(), format_duration(seconds));
                }

                // Show pipeline steps
                let steps = client
                    .list_pipeline_steps(&workspace, &repo_slug, &pipeline.uuid)
                    .await?;

                if !steps.values.is_empty() {
                    println!();
                    println!("{}", "Steps:".bold());

                    for step in &steps.values {
                        let status = step
                            .state
                            .as_ref()
                            .map(|s| s.name.as_str())
                            .unwrap_or("unknown");

                        let status_icon = match status {
                            "COMPLETED" => {
                                let result = step
                                    .state
                                    .as_ref()
                                    .and_then(|s| s.result.as_ref())
                                    .map(|r| r.name.as_str())
                                    .unwrap_or("");
                                match result {
                                    "SUCCESSFUL" => "✓".green(),
                                    "FAILED" => "✗".red(),
                                    _ => "○".normal(),
                                }
                            }
                            "IN_PROGRESS" => "◉".blue(),
                            "PENDING" => "○".dimmed(),
                            _ => "○".normal(),
                        };

                        let name = step.name.as_deref().unwrap_or("Step");
                        println!("  {} {}", status_icon, name);

                        if logs {
                            // Fetch and display step log
                            match client
                                .get_step_log(&workspace, &repo_slug, &pipeline.uuid, &step.uuid)
                                .await
                            {
                                Ok(log) => {
                                    if !log.is_empty() {
                                        println!();
                                        for line in log.lines().take(50) {
                                            println!("    {}", line.dimmed());
                                        }
                                        if log.lines().count() > 50 {
                                            println!("    {} ... (truncated)", "".dimmed());
                                        }
                                        println!();
                                    }
                                }
                                Err(_) => {
                                    // Log might not be available yet
                                }
                            }
                        }
                    }
                }

                Ok(())
            }

            PipelineCommands::Trigger {
                repo,
                branch,
                pipeline,
                wait,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let request = if let Some(pipeline_name) = pipeline {
                    TriggerPipelineRequest::for_branch_with_pipeline(&branch, &pipeline_name)
                } else {
                    TriggerPipelineRequest::for_branch(&branch)
                };

                let triggered = client
                    .trigger_pipeline(&workspace, &repo_slug, &request)
                    .await?;

                println!(
                    "{} Triggered pipeline #{} on branch {}",
                    "✓".green(),
                    triggered.build_number,
                    branch.cyan()
                );

                if wait {
                    println!();
                    let pb = ProgressBar::new_spinner();
                    pb.set_style(
                        ProgressStyle::default_spinner()
                            .template("{spinner:.blue} {msg}")
                            .unwrap(),
                    );
                    pb.set_message("Waiting for pipeline to complete...");

                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

                        let current = client
                            .get_pipeline(&workspace, &repo_slug, &triggered.uuid)
                            .await?;

                        match current.state.name {
                            PipelineStateName::Completed => {
                                pb.finish_and_clear();

                                if let Some(result) = &current.state.result {
                                    match result.name {
                                        PipelineResultName::Successful => {
                                            println!(
                                                "{} Pipeline #{} completed successfully!",
                                                "✓".green(),
                                                current.build_number
                                            );
                                        }
                                        PipelineResultName::Failed => {
                                            println!(
                                                "{} Pipeline #{} failed",
                                                "✗".red(),
                                                current.build_number
                                            );
                                        }
                                        _ => {
                                            println!(
                                                "Pipeline #{} completed with status: {:?}",
                                                current.build_number, result.name
                                            );
                                        }
                                    }
                                }
                                break;
                            }
                            PipelineStateName::Halted => {
                                pb.finish_and_clear();
                                println!(
                                    "{} Pipeline #{} was halted",
                                    "⚠".yellow(),
                                    current.build_number
                                );
                                break;
                            }
                            _ => {
                                pb.tick();
                            }
                        }
                    }
                }

                Ok(())
            }

            PipelineCommands::Stop { repo, build } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let pipeline = client
                    .get_pipeline_by_build_number(&workspace, &repo_slug, build)
                    .await?;

                client
                    .stop_pipeline(&workspace, &repo_slug, &pipeline.uuid)
                    .await?;

                println!("{} Stopped pipeline #{}", "✓".green(), build);

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

pub(crate) fn format_status(
    state: &PipelineStateName,
    result: Option<&PipelineResultName>,
) -> String {
    match state {
        PipelineStateName::Pending => "PENDING".yellow().to_string(),
        PipelineStateName::InProgress => "RUNNING".blue().to_string(),
        PipelineStateName::Paused => "PAUSED".yellow().to_string(),
        PipelineStateName::Halted => "HALTED".red().to_string(),
        PipelineStateName::Completed => {
            if let Some(result) = result {
                match result {
                    PipelineResultName::Successful => "SUCCESS".green().to_string(),
                    PipelineResultName::Failed => "FAILED".red().to_string(),
                    PipelineResultName::Error => "ERROR".red().to_string(),
                    PipelineResultName::Stopped => "STOPPED".yellow().to_string(),
                    PipelineResultName::Expired => "EXPIRED".dimmed().to_string(),
                }
            } else {
                "COMPLETED".normal().to_string()
            }
        }
    }
}

pub(crate) fn format_duration(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!("{}h {}m", seconds / 3600, (seconds % 3600) / 60)
    }
}
