use anyhow::{Context, Result};
use clap::Subcommand;
use colored::{ColoredString, Colorize};
use indicatif::{ProgressBar, ProgressStyle};
use tabled::{Table, Tabled};

use super::{confirm_or_abort, parse_repo, resolve_workspace};
use crate::api::BitbucketClient;
use crate::cli::pagination::effective_limit;
use crate::models::{
    Pipeline, PipelineResultName, PipelineStateName, PipelineStep, PipelineVariable,
    PipelineVariableInput, TriggerPipelineRequest, TriggerPipelineSelector,
};

#[derive(Subcommand)]
pub enum PipelineCommands {
    /// List pipelines
    List {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Number of results (capped at 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,

        /// Filter by status (PENDING, IN_PROGRESS, COMPLETED, ...)
        #[arg(long)]
        status: Option<String>,

        /// Filter by target branch name
        #[arg(long)]
        target_branch: Option<String>,

        /// Sort order (default: -created_on, most recent first)
        #[arg(long)]
        sort: Option<String>,

        /// Page number to fetch
        #[arg(long)]
        page: Option<u32>,
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

        /// Show only one step: a 1-based index or a step UUID prefix
        #[arg(long)]
        step: Option<String>,

        /// Print complete step logs instead of the first 50 lines (implies --logs)
        #[arg(long)]
        full_logs: bool,
    },

    /// Trigger a new pipeline
    Trigger {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Branch to run pipeline on
        #[arg(short, long, default_value = "main")]
        branch: String,

        /// Commit hash to run the pipeline on (takes precedence over --branch)
        #[arg(long)]
        commit: Option<String>,

        /// Custom pipeline name (from bitbucket-pipelines.yml)
        #[arg(short, long)]
        pipeline: Option<String>,

        /// Pipeline variable as KEY=VALUE (repeatable)
        #[arg(long, value_name = "KEY=VALUE")]
        var: Vec<String>,

        /// Secured pipeline variable as KEY=VALUE, masked in logs (repeatable)
        #[arg(long, value_name = "KEY=VALUE")]
        secured_var: Vec<String>,

        /// Wait for pipeline to complete
        #[arg(short, long)]
        wait: bool,
    },

    /// Stop a running pipeline
    Stop {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Pipeline build number
        #[arg(short, long, required_unless_present = "uuid")]
        build: Option<u64>,

        /// Pipeline UUID (skips the build-number lookup)
        #[arg(long, conflicts_with = "build")]
        uuid: Option<String>,
    },

    /// Re-run a pipeline by triggering an equivalent one
    Rerun {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Build number of the pipeline to re-run
        #[arg(short, long)]
        build: u64,
    },

    /// View or update the repository pipelines configuration
    Config {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Enable pipelines for the repository
        #[arg(long, conflicts_with = "disable")]
        enable: bool,

        /// Disable pipelines for the repository
        #[arg(long)]
        disable: bool,

        /// Set the next pipeline build number
        #[arg(long)]
        next_build_number: Option<u64>,
    },

    /// Manage repository pipeline variables
    Variable {
        #[command(subcommand)]
        command: VariableCommands,
    },

    /// Manage workspace-level pipeline variables
    WorkspaceVariable {
        #[command(subcommand)]
        command: WorkspaceVariableCommands,
    },

    /// Manage pipeline schedules
    Schedule {
        #[command(subcommand)]
        command: ScheduleCommands,
    },

    /// Manage pipeline dependency caches
    Cache {
        #[command(subcommand)]
        command: CacheCommands,
    },
}

#[derive(Subcommand)]
pub enum VariableCommands {
    /// List pipeline variables
    List {
        /// Repository in format workspace/repo-slug
        repo: String,
    },

    /// Create or update a pipeline variable
    Set {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Variable name
        key: String,

        /// Variable value
        value: String,

        /// Mask the value in logs and the UI
        #[arg(long)]
        secured: bool,
    },

    /// Delete a pipeline variable by key
    Delete {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Variable name
        key: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum WorkspaceVariableCommands {
    /// List workspace pipeline variables
    List {
        /// Workspace (defaults to --workspace or the configured default)
        workspace: Option<String>,
    },

    /// Create or update a workspace pipeline variable
    Set {
        /// Variable name
        key: String,

        /// Variable value
        value: String,

        /// Workspace (defaults to --workspace or the configured default)
        workspace: Option<String>,

        /// Mask the value in logs and the UI
        #[arg(long)]
        secured: bool,
    },

    /// Delete a workspace pipeline variable by key
    Delete {
        /// Variable name
        key: String,

        /// Workspace (defaults to --workspace or the configured default)
        workspace: Option<String>,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum ScheduleCommands {
    /// List pipeline schedules
    List {
        /// Repository in format workspace/repo-slug
        repo: String,
    },

    /// Create a pipeline schedule
    Create {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Branch to run the scheduled pipeline on
        #[arg(long)]
        branch: String,

        /// Cron pattern (e.g. "0 0 12 * * ? *")
        #[arg(long)]
        cron: String,
    },

    /// Delete a pipeline schedule by UUID
    Delete {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Schedule UUID (with or without braces)
        uuid: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Subcommand)]
pub enum CacheCommands {
    /// List pipeline dependency caches
    List {
        /// Repository in format workspace/repo-slug
        repo: String,
    },

    /// Delete pipeline dependency caches by name
    Delete {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Cache name (as shown by 'cache list')
        name: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
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

#[derive(Tabled)]
struct VariableRow {
    #[tabled(rename = "KEY")]
    key: String,
    #[tabled(rename = "VALUE")]
    value: String,
    #[tabled(rename = "SECURED")]
    secured: String,
    #[tabled(rename = "UUID")]
    uuid: String,
}

#[derive(Tabled)]
struct ScheduleRow {
    #[tabled(rename = "ENABLED")]
    enabled: String,
    #[tabled(rename = "CRON")]
    cron: String,
    #[tabled(rename = "BRANCH")]
    branch: String,
    #[tabled(rename = "UUID")]
    uuid: String,
}

#[derive(Tabled)]
struct CacheRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "PATH")]
    path: String,
    #[tabled(rename = "SIZE")]
    size: String,
    #[tabled(rename = "UUID")]
    uuid: String,
}

impl PipelineCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            PipelineCommands::List {
                repo,
                limit,
                status,
                target_branch,
                sort,
                page,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let limit = effective_limit(limit);

                let pipelines = client
                    .list_pipelines_filtered(
                        &workspace,
                        &repo_slug,
                        crate::api::pipelines::PipelineListFilters {
                            page,
                            pagelen: Some(limit),
                            status: status.as_deref(),
                            target_branch: target_branch.as_deref(),
                            sort: sort.as_deref(),
                        },
                    )
                    .await?;

                if super::output_json() {
                    return super::print_json(&pipelines.values);
                }

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

            PipelineCommands::View {
                repo,
                build,
                logs,
                step,
                full_logs,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let show_logs = logs || full_logs;

                let pipeline = client
                    .get_pipeline_by_build_number(&workspace, &repo_slug, build)
                    .await?;

                if super::output_json() && !show_logs {
                    let steps = client
                        .list_pipeline_steps(&workspace, &repo_slug, &pipeline.uuid)
                        .await?;

                    let steps = match &step {
                        Some(selector) => {
                            let (index, by_uuid) = select_step(&steps.values, selector)?;
                            let chosen = &steps.values[index];
                            let detail = if by_uuid {
                                client
                                    .get_pipeline_step(
                                        &workspace,
                                        &repo_slug,
                                        &pipeline.uuid,
                                        &chosen.uuid,
                                    )
                                    .await?
                            } else {
                                chosen.clone()
                            };
                            vec![detail]
                        }
                        None => steps.values,
                    };

                    return super::print_json(&serde_json::json!({
                        "pipeline": pipeline,
                        "steps": steps,
                    }));
                }

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

                let steps = client
                    .list_pipeline_steps(&workspace, &repo_slug, &pipeline.uuid)
                    .await?;

                if let Some(selector) = step {
                    let (index, by_uuid) = select_step(&steps.values, &selector)?;
                    let chosen = &steps.values[index];

                    // A UUID selection points at one concrete step, so use the
                    // dedicated step endpoint for the freshest detail.
                    let detail = if by_uuid {
                        client
                            .get_pipeline_step(&workspace, &repo_slug, &pipeline.uuid, &chosen.uuid)
                            .await?
                    } else {
                        chosen.clone()
                    };

                    print_step_detail(&detail);

                    if show_logs {
                        // A single step was explicitly requested, so a failed
                        // log fetch is worth surfacing rather than swallowing.
                        if let Err(e) = print_step_log(
                            &client,
                            &workspace,
                            &repo_slug,
                            &pipeline.uuid,
                            &detail.uuid,
                            full_logs,
                        )
                        .await
                        {
                            eprintln!(
                                "{} could not fetch log for step {}: {}",
                                "⚠".yellow(),
                                detail.name.as_deref().unwrap_or(&detail.uuid),
                                e
                            );
                        }
                    }

                    return Ok(());
                }

                if !steps.values.is_empty() {
                    println!();
                    println!("{}", "Steps:".bold());

                    for step in &steps.values {
                        let name = step.name.as_deref().unwrap_or("Step");
                        println!("  {} {}", step_status_icon(step), name);

                        if show_logs {
                            // Iterating over every step: logs for pending or
                            // in-progress steps may not exist yet, so ignore
                            // fetch errors here.
                            let _ = print_step_log(
                                &client,
                                &workspace,
                                &repo_slug,
                                &pipeline.uuid,
                                &step.uuid,
                                full_logs,
                            )
                            .await;
                        }
                    }
                }

                Ok(())
            }

            PipelineCommands::Trigger {
                repo,
                branch,
                commit,
                pipeline,
                var,
                secured_var,
                wait,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let mut variables = Vec::with_capacity(var.len() + secured_var.len());
                for spec in &var {
                    let (key, value) = parse_var(spec)?;
                    variables.push(PipelineVariableInput {
                        key,
                        value,
                        secured: false,
                    });
                }
                for spec in &secured_var {
                    let (key, value) = parse_var(spec)?;
                    variables.push(PipelineVariableInput {
                        key,
                        value,
                        secured: true,
                    });
                }
                let variable_count = variables.len();

                let mut request = match &commit {
                    Some(hash) => TriggerPipelineRequest::for_commit(hash),
                    None => TriggerPipelineRequest::for_branch(&branch),
                };
                if let Some(pipeline_name) = &pipeline {
                    request = request.with_pipeline(pipeline_name);
                }
                request = request.with_variables(variables);

                let triggered = client
                    .trigger_pipeline(&workspace, &repo_slug, &request)
                    .await?;

                if super::output_json() {
                    if wait {
                        let done = wait_for_pipeline_silently(
                            &client,
                            &workspace,
                            &repo_slug,
                            &triggered.uuid,
                        )
                        .await?;
                        return super::print_json(&done);
                    }
                    return super::print_json(&triggered);
                }

                let target_desc = match &commit {
                    Some(hash) => format!("commit {}", hash.cyan()),
                    None => format!("branch {}", branch.cyan()),
                };
                let variable_note = if variable_count > 0 {
                    format!(" with {} variable(s)", variable_count)
                } else {
                    String::new()
                };
                println!(
                    "{} Triggered pipeline #{} on {}{}",
                    "✓".green(),
                    triggered.build_number,
                    target_desc,
                    variable_note
                );

                if wait {
                    println!();
                    wait_for_pipeline(&client, &workspace, &repo_slug, &triggered.uuid).await?;
                }

                Ok(())
            }

            PipelineCommands::Stop { repo, build, uuid } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                if let Some(uuid) = uuid {
                    let uuid = ensure_braced_uuid(&uuid);
                    client.stop_pipeline(&workspace, &repo_slug, &uuid).await?;
                    if super::output_json() {
                        return super::print_json(&serde_json::json!({"ok": true}));
                    }
                    println!("{} Stopped pipeline {}", "✓".green(), uuid);
                    return Ok(());
                }

                // clap guarantees --build when --uuid is absent.
                let build = build.expect("clap enforces --build or --uuid");
                let pipeline = client
                    .get_pipeline_by_build_number(&workspace, &repo_slug, build)
                    .await?;

                client
                    .stop_pipeline(&workspace, &repo_slug, &pipeline.uuid)
                    .await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!("{} Stopped pipeline #{}", "✓".green(), build);

                Ok(())
            }

            PipelineCommands::Rerun { repo, build } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let original = client
                    .get_pipeline_by_build_number(&workspace, &repo_slug, build)
                    .await?;

                // Bitbucket has no rerun endpoint, so re-trigger an equivalent
                // pipeline from the original's target.
                let Some(ref_name) = original.target.ref_name.clone() else {
                    anyhow::bail!(
                        "Pipeline #{} has no branch to re-run from (target type '{}'). \
                         Bitbucket has no rerun API, so only pipelines that ran against \
                         a branch can be re-triggered. Use 'pipeline trigger' instead.",
                        build,
                        original.target.target_type
                    );
                };

                let mut request = TriggerPipelineRequest::for_branch(&ref_name);
                if let Some(selector) = &original.target.selector {
                    if let Some(pattern) = &selector.pattern {
                        request.target.selector = Some(TriggerPipelineSelector {
                            selector_type: selector.selector_type.clone(),
                            pattern: pattern.clone(),
                        });
                    }
                }

                let new = client
                    .trigger_pipeline(&workspace, &repo_slug, &request)
                    .await?;

                if super::output_json() {
                    return super::print_json(&new);
                }

                println!(
                    "{} Re-ran pipeline #{} as #{} on branch {}",
                    "✓".green(),
                    build,
                    new.build_number,
                    ref_name.cyan()
                );

                Ok(())
            }

            PipelineCommands::Config {
                repo,
                enable,
                disable,
                next_build_number,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                if super::output_json() {
                    let mut config = None;
                    if enable || disable {
                        config = Some(
                            client
                                .update_pipelines_config_enabled(&workspace, &repo_slug, enable)
                                .await?,
                        );
                    }
                    if let Some(next) = next_build_number {
                        client
                            .set_pipelines_config_build_number(&workspace, &repo_slug, next)
                            .await?;
                    }
                    return match config {
                        Some(config) => super::print_json(&config),
                        None if next_build_number.is_some() => {
                            super::print_json(&serde_json::json!({"ok": true}))
                        }
                        None => super::print_json(
                            &client.get_pipelines_config(&workspace, &repo_slug).await?,
                        ),
                    };
                }

                let mut changed = false;

                if enable || disable {
                    let config = client
                        .update_pipelines_config_enabled(&workspace, &repo_slug, enable)
                        .await?;
                    let state = if config.enabled.unwrap_or(enable) {
                        "enabled".green()
                    } else {
                        "disabled".red()
                    };
                    println!("{} Pipelines {} for {}", "✓".green(), state, repo);
                    changed = true;
                }

                if let Some(next) = next_build_number {
                    client
                        .set_pipelines_config_build_number(&workspace, &repo_slug, next)
                        .await?;
                    println!("{} Next build number set to {}", "✓".green(), next);
                    changed = true;
                }

                if !changed {
                    let config = client.get_pipelines_config(&workspace, &repo_slug).await?;
                    match config.enabled {
                        Some(true) => println!("Pipelines are {} for {}", "enabled".green(), repo),
                        Some(false) => println!("Pipelines are {} for {}", "disabled".red(), repo),
                        None => println!("Pipelines enabled state is unknown for {}", repo),
                    }
                }

                Ok(())
            }

            PipelineCommands::Variable { command } => command.run().await,
            PipelineCommands::WorkspaceVariable { command } => command.run().await,
            PipelineCommands::Schedule { command } => command.run().await,
            PipelineCommands::Cache { command } => command.run().await,
        }
    }
}

impl VariableCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            VariableCommands::List { repo } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let variables = client
                    .list_pipeline_variables(&workspace, &repo_slug)
                    .await?;

                if super::output_json() {
                    return super::print_json(&masked_variables(&variables));
                }

                if variables.is_empty() {
                    println!("No pipeline variables found");
                    return Ok(());
                }

                let table = Table::new(variable_rows(&variables)).to_string();
                println!("{}", table);

                Ok(())
            }

            VariableCommands::Set {
                repo,
                key,
                value,
                secured,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let input = PipelineVariableInput {
                    key: key.clone(),
                    value,
                    secured,
                };

                let variables = client
                    .list_pipeline_variables(&workspace, &repo_slug)
                    .await?;
                match find_variable_uuid(&variables, &key) {
                    Some(uuid) => {
                        let variable = client
                            .update_pipeline_variable(&workspace, &repo_slug, &uuid, &input)
                            .await
                            .with_context(|| {
                                format!(
                                    "updating variable '{}' (it may have been deleted \
                                     concurrently — re-run to create it)",
                                    key
                                )
                            })?;
                        if super::output_json() {
                            return super::print_json(&variable);
                        }
                        println!("{} Updated pipeline variable {}", "✓".green(), key.cyan());
                    }
                    None => {
                        let variable = client
                            .create_pipeline_variable(&workspace, &repo_slug, &input)
                            .await?;
                        if super::output_json() {
                            return super::print_json(&variable);
                        }
                        println!("{} Created pipeline variable {}", "✓".green(), key.cyan());
                    }
                }

                Ok(())
            }

            VariableCommands::Delete { repo, key, yes } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let variables = client
                    .list_pipeline_variables(&workspace, &repo_slug)
                    .await?;
                let uuid = find_variable_uuid(&variables, &key).ok_or_else(|| {
                    anyhow::anyhow!("No pipeline variable named '{}' in {}", key, repo)
                })?;

                if !yes && !confirm_or_abort(format!("Delete pipeline variable {}?", key.red()))? {
                    return Ok(());
                }

                client
                    .delete_pipeline_variable(&workspace, &repo_slug, &uuid)
                    .await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!("{} Deleted pipeline variable {}", "✓".green(), key);

                Ok(())
            }
        }
    }
}

impl WorkspaceVariableCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            WorkspaceVariableCommands::List { workspace } => {
                let workspace = resolve_workspace(workspace)?;
                let client = BitbucketClient::from_stored().await?;

                let variables = client.list_workspace_pipeline_variables(&workspace).await?;

                if super::output_json() {
                    return super::print_json(&masked_variables(&variables));
                }

                if variables.is_empty() {
                    println!("No workspace pipeline variables found");
                    return Ok(());
                }

                let table = Table::new(variable_rows(&variables)).to_string();
                println!("{}", table);

                Ok(())
            }

            WorkspaceVariableCommands::Set {
                key,
                value,
                workspace,
                secured,
            } => {
                let workspace = resolve_workspace(workspace)?;
                let client = BitbucketClient::from_stored().await?;

                let input = PipelineVariableInput {
                    key: key.clone(),
                    value,
                    secured,
                };

                let variables = client.list_workspace_pipeline_variables(&workspace).await?;
                match find_variable_uuid(&variables, &key) {
                    Some(uuid) => {
                        let variable = client
                            .update_workspace_pipeline_variable(&workspace, &uuid, &input)
                            .await
                            .with_context(|| {
                                format!(
                                    "updating variable '{}' (it may have been deleted \
                                     concurrently — re-run to create it)",
                                    key
                                )
                            })?;
                        if super::output_json() {
                            return super::print_json(&variable);
                        }
                        println!(
                            "{} Updated workspace pipeline variable {}",
                            "✓".green(),
                            key.cyan()
                        );
                    }
                    None => {
                        let variable = client
                            .create_workspace_pipeline_variable(&workspace, &input)
                            .await?;
                        if super::output_json() {
                            return super::print_json(&variable);
                        }
                        println!(
                            "{} Created workspace pipeline variable {}",
                            "✓".green(),
                            key.cyan()
                        );
                    }
                }

                Ok(())
            }

            WorkspaceVariableCommands::Delete {
                key,
                workspace,
                yes,
            } => {
                let workspace = resolve_workspace(workspace)?;
                let client = BitbucketClient::from_stored().await?;

                let variables = client.list_workspace_pipeline_variables(&workspace).await?;
                let uuid = find_variable_uuid(&variables, &key).ok_or_else(|| {
                    anyhow::anyhow!(
                        "No workspace pipeline variable named '{}' in {}",
                        key,
                        workspace
                    )
                })?;

                if !yes
                    && !confirm_or_abort(format!(
                        "Delete workspace pipeline variable {}?",
                        key.red()
                    ))?
                {
                    return Ok(());
                }

                client
                    .delete_workspace_pipeline_variable(&workspace, &uuid)
                    .await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!(
                    "{} Deleted workspace pipeline variable {}",
                    "✓".green(),
                    key
                );

                Ok(())
            }
        }
    }
}

impl ScheduleCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            ScheduleCommands::List { repo } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let schedules = client
                    .list_pipeline_schedules(&workspace, &repo_slug)
                    .await?;

                if super::output_json() {
                    return super::print_json(&schedules);
                }

                if schedules.is_empty() {
                    println!("No pipeline schedules found");
                    return Ok(());
                }

                let rows: Vec<ScheduleRow> = schedules
                    .iter()
                    .map(|s| ScheduleRow {
                        enabled: if s.enabled.unwrap_or(false) {
                            "yes".to_string()
                        } else {
                            "no".to_string()
                        },
                        cron: s.cron_pattern.clone().unwrap_or_else(|| "-".to_string()),
                        branch: schedule_branch(s.target.as_ref()),
                        uuid: s.uuid.clone(),
                    })
                    .collect();

                let table = Table::new(rows).to_string();
                println!("{}", table);

                Ok(())
            }

            ScheduleCommands::Create { repo, branch, cron } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let schedule = client
                    .create_pipeline_schedule(&workspace, &repo_slug, &branch, &cron)
                    .await?;

                if super::output_json() {
                    return super::print_json(&schedule);
                }

                println!(
                    "{} Created schedule {} on branch {} ({})",
                    "✓".green(),
                    schedule.uuid,
                    branch.cyan(),
                    schedule.cron_pattern.as_deref().unwrap_or(&cron)
                );

                Ok(())
            }

            ScheduleCommands::Delete { repo, uuid, yes } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let uuid = ensure_braced_uuid(&uuid);

                if !yes && !confirm_or_abort(format!("Delete pipeline schedule {}?", uuid.red()))? {
                    return Ok(());
                }

                client
                    .delete_pipeline_schedule(&workspace, &repo_slug, &uuid)
                    .await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!("{} Deleted pipeline schedule {}", "✓".green(), uuid);

                Ok(())
            }
        }
    }
}

impl CacheCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            CacheCommands::List { repo } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let caches = client.list_pipeline_caches(&workspace, &repo_slug).await?;

                if super::output_json() {
                    return super::print_json(&caches);
                }

                if caches.is_empty() {
                    println!("No pipeline caches found");
                    return Ok(());
                }

                let rows: Vec<CacheRow> = caches
                    .iter()
                    .map(|c| CacheRow {
                        name: c.name.clone().unwrap_or_else(|| "-".to_string()),
                        path: c.path.clone().unwrap_or_else(|| "-".to_string()),
                        size: c
                            .file_size_bytes
                            .map(format_bytes)
                            .unwrap_or_else(|| "-".to_string()),
                        uuid: c.uuid.clone(),
                    })
                    .collect();

                let table = Table::new(rows).to_string();
                println!("{}", table);

                Ok(())
            }

            CacheCommands::Delete { repo, name, yes } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let caches = client.list_pipeline_caches(&workspace, &repo_slug).await?;
                let matches: Vec<_> = caches
                    .into_iter()
                    .filter(|c| c.name.as_deref() == Some(name.as_str()))
                    .collect();

                if matches.is_empty() {
                    anyhow::bail!("No pipeline cache named '{}' in {}", name, repo);
                }

                if !yes
                    && !confirm_or_abort(format!(
                        "Delete {} pipeline cache(s) named {}?",
                        matches.len(),
                        name.red()
                    ))?
                {
                    return Ok(());
                }

                let total = matches.len();
                let json = super::output_json();
                let mut deleted = 0usize;
                let mut failures: Vec<(String, anyhow::Error)> = Vec::new();

                // Delete every match rather than aborting on the first error, so
                // a single bad cache doesn't leave the rest stranded.
                for cache in &matches {
                    match client
                        .delete_pipeline_cache(&workspace, &repo_slug, &cache.uuid)
                        .await
                    {
                        Ok(()) => {
                            deleted += 1;
                            if !json {
                                println!("{} Deleted pipeline cache {}", "✓".green(), cache.uuid);
                            }
                        }
                        Err(e) => failures.push((cache.uuid.clone(), e)),
                    }
                }

                let failed = failures.len();

                if failed > 0 {
                    for (uuid, e) in &failures {
                        eprintln!(
                            "{} could not delete pipeline cache {}: {}",
                            "⚠".yellow(),
                            uuid,
                            e
                        );
                    }
                    anyhow::bail!(
                        "Deleted {} of {} pipeline cache(s) named '{}' ({} failed)",
                        deleted,
                        total,
                        name,
                        failed
                    );
                }

                if json {
                    return super::print_json(&serde_json::json!({"ok": true, "deleted": deleted}));
                }

                println!(
                    "{} Deleted {} of {} pipeline cache(s) named {}",
                    "✓".green(),
                    deleted,
                    total,
                    name
                );

                Ok(())
            }
        }
    }
}

/// Poll a pipeline every 5 seconds until it reaches a terminal state (same
/// states as [`wait_for_pipeline`]) without any terminal output, and return
/// the final pipeline for JSON rendering.
async fn wait_for_pipeline_silently(
    client: &BitbucketClient,
    workspace: &str,
    repo_slug: &str,
    pipeline_uuid: &str,
) -> Result<Pipeline> {
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        let current = client
            .get_pipeline(workspace, repo_slug, pipeline_uuid)
            .await?;

        match current.state.name {
            PipelineStateName::Completed
            | PipelineStateName::Halted
            | PipelineStateName::Paused => return Ok(current),
            _ => {}
        }
    }
}

/// Poll a pipeline every 5 seconds until it reaches a state where waiting is
/// pointless: completed, halted, or paused (paused pipelines never progress
/// without a manual resume in the Bitbucket UI, so treat them as terminal).
async fn wait_for_pipeline(
    client: &BitbucketClient,
    workspace: &str,
    repo_slug: &str,
    pipeline_uuid: &str,
) -> Result<()> {
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
            .get_pipeline(workspace, repo_slug, pipeline_uuid)
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
                            println!("{} Pipeline #{} failed", "✗".red(), current.build_number);
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
            PipelineStateName::Paused => {
                pb.finish_and_clear();
                println!(
                    "{} Pipeline #{} is PAUSED and requires manual resume",
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

    Ok(())
}

/// Fetch and print a step's log, indented and dimmed. Without `full`, output
/// is truncated to the first 50 lines.
///
/// Returns any fetch error to the caller instead of swallowing it. An empty
/// log is treated as success (logs may simply not exist yet for pending or
/// in-progress steps): callers iterating over every step ignore the error,
/// while a caller that explicitly asked for one step surfaces it.
async fn print_step_log(
    client: &BitbucketClient,
    workspace: &str,
    repo_slug: &str,
    pipeline_uuid: &str,
    step_uuid: &str,
    full: bool,
) -> Result<()> {
    let log = client
        .get_step_log(workspace, repo_slug, pipeline_uuid, step_uuid)
        .await?;

    if log.is_empty() {
        return Ok(());
    }

    println!();
    if full {
        for line in log.lines() {
            println!("    {}", line.dimmed());
        }
    } else {
        let (shown, truncated) = take_log_lines(&log, 50);
        for line in shown {
            println!("    {}", line.dimmed());
        }
        if truncated {
            println!(
                "    {}",
                "... (truncated; use --full-logs for the complete log)".dimmed()
            );
        }
    }
    println!();

    Ok(())
}

/// Print the detail block for a single pipeline step.
fn print_step_detail(step: &PipelineStep) {
    println!();
    println!("{}", "Step:".bold());
    println!(
        "  {} {}",
        step_status_icon(step),
        step.name.as_deref().unwrap_or("Step")
    );
    println!("    {} {}", "UUID:".dimmed(), step.uuid);

    if let Some(state) = &step.state {
        let result = state
            .result
            .as_ref()
            .map(|r| format!(" ({})", r.name))
            .unwrap_or_default();
        println!("    {} {}{}", "State:".dimmed(), state.name, result);
    }

    if let Some(started) = step.started_on {
        println!(
            "    {} {}",
            "Started:".dimmed(),
            started.format("%Y-%m-%d %H:%M:%S")
        );
    }

    if let Some(completed) = step.completed_on {
        println!(
            "    {} {}",
            "Completed:".dimmed(),
            completed.format("%Y-%m-%d %H:%M:%S")
        );
    }

    if let Some(image) = &step.image {
        println!("    {} {}", "Image:".dimmed(), image.name);
    }

    if let Some(commands) = &step.script_commands {
        println!("    {} {}", "Script commands:".dimmed(), commands.len());
    }
}

/// Status icon for a pipeline step based on its state and result.
fn step_status_icon(step: &PipelineStep) -> ColoredString {
    let status = step
        .state
        .as_ref()
        .map(|s| s.name.as_str())
        .unwrap_or("unknown");

    match status {
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
    }
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

/// Split a `KEY=VALUE` argument on the FIRST '=' so values may contain '='.
fn parse_var(spec: &str) -> Result<(String, String)> {
    let (key, value) = spec
        .split_once('=')
        .ok_or_else(|| anyhow::anyhow!("Invalid variable '{}': expected KEY=VALUE", spec))?;

    if key.is_empty() {
        anyhow::bail!("Invalid variable '{}': key must not be empty", spec);
    }

    Ok((key.to_string(), value.to_string()))
}

/// Resolve a `--step` selector against a pipeline's steps.
///
/// A selector that parses as a number is a 1-based index; anything else is
/// matched as a case-insensitive UUID prefix (braces optional). Returns the
/// zero-based index of the step and whether it was selected by UUID.
fn select_step(steps: &[PipelineStep], selector: &str) -> Result<(usize, bool)> {
    if let Ok(index) = selector.parse::<usize>() {
        if index == 0 || index > steps.len() {
            anyhow::bail!(
                "Step index {} out of range (pipeline has {} step(s))",
                index,
                steps.len()
            );
        }
        return Ok((index - 1, false));
    }

    let clean = |s: &str| {
        s.trim_start_matches('{')
            .trim_end_matches('}')
            .to_lowercase()
    };
    let needle = clean(selector);
    if needle.is_empty() {
        anyhow::bail!("--step must be a 1-based index or a step UUID prefix");
    }

    let matches: Vec<usize> = steps
        .iter()
        .enumerate()
        .filter(|(_, s)| clean(&s.uuid).starts_with(&needle))
        .map(|(i, _)| i)
        .collect();

    match matches.as_slice() {
        [] => anyhow::bail!("No step matches '{}'", selector),
        [only] => Ok((*only, true)),
        _ => anyhow::bail!(
            "Step UUID prefix '{}' is ambiguous ({} steps match)",
            selector,
            matches.len()
        ),
    }
}

/// Wrap a UUID in braces if the user omitted them; Bitbucket API paths
/// require the braced form.
fn ensure_braced_uuid(uuid: &str) -> String {
    let uuid = uuid.trim();
    let trimmed = uuid.trim_start_matches('{').trim_end_matches('}');
    format!("{{{}}}", trimmed)
}

/// Find the UUID of the variable whose key matches exactly.
fn find_variable_uuid(variables: &[PipelineVariable], key: &str) -> Option<String> {
    variables
        .iter()
        .find(|v| v.key.as_deref() == Some(key))
        .and_then(|v| v.uuid.clone())
}

/// Return a copy of `variables` with secured values nulled out.
///
/// The API already omits `value` for secured variables, but this defends the
/// JSON path so a future/API quirk can't leak a secret that the table view
/// would have masked.
fn masked_variables(variables: &[PipelineVariable]) -> Vec<PipelineVariable> {
    variables
        .iter()
        .map(|v| {
            let mut masked = v.clone();
            if masked.secured == Some(true) {
                masked.value = None;
            }
            masked
        })
        .collect()
}

/// Table rows for repository or workspace pipeline variables. Secured values
/// are never returned by the API, so they render masked.
fn variable_rows(variables: &[PipelineVariable]) -> Vec<VariableRow> {
    variables
        .iter()
        .map(|v| {
            let secured = v.secured.unwrap_or(false);
            VariableRow {
                key: v.key.clone().unwrap_or_else(|| "-".to_string()),
                value: if secured {
                    "••••••".to_string()
                } else {
                    v.value.clone().unwrap_or_else(|| "-".to_string())
                },
                secured: if secured {
                    "yes".to_string()
                } else {
                    "no".to_string()
                },
                uuid: v.uuid.clone().unwrap_or_else(|| "-".to_string()),
            }
        })
        .collect()
}

/// Extract the branch name from a schedule's raw target object.
fn schedule_branch(target: Option<&serde_json::Value>) -> String {
    target
        .and_then(|t| t.get("ref_name"))
        .and_then(|v| v.as_str())
        .unwrap_or("-")
        .to_string()
}

/// Human-readable byte size using binary units.
fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let mut value = bytes as f64;
    let mut unit = 0;

    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{} B", bytes)
    } else {
        format!("{:.1} {}", value, UNITS[unit])
    }
}

/// Take at most `max` display lines from a log in a single pass, reporting
/// whether any lines were left out.
fn take_log_lines(log: &str, max: usize) -> (Vec<&str>, bool) {
    let mut lines = log.lines();
    let shown: Vec<&str> = lines.by_ref().take(max).collect();
    let truncated = lines.next().is_some();
    (shown, truncated)
}

#[cfg(test)]
mod tests {
    use crate::models::{PipelineStep, PipelineStepResult, PipelineStepState, PipelineVariable};

    use super::{
        ensure_braced_uuid, find_variable_uuid, format_bytes, parse_var, schedule_branch,
        select_step, step_status_icon, take_log_lines, variable_rows,
    };

    fn log_with_lines(n: usize) -> String {
        (0..n).map(|i| format!("line {}\n", i)).collect()
    }

    #[test]
    fn under_the_limit_is_not_truncated() {
        let log = log_with_lines(49);
        let (shown, truncated) = take_log_lines(&log, 50);
        assert_eq!(shown.len(), 49);
        assert!(!truncated);
    }

    #[test]
    fn exactly_the_limit_is_not_truncated() {
        let log = log_with_lines(50);
        let (shown, truncated) = take_log_lines(&log, 50);
        assert_eq!(shown.len(), 50);
        assert!(!truncated);
    }

    #[test]
    fn one_over_the_limit_is_truncated() {
        let log = log_with_lines(51);
        let (shown, truncated) = take_log_lines(&log, 50);
        assert_eq!(shown.len(), 50);
        assert!(truncated);
    }

    #[test]
    fn parse_var_splits_on_first_equals() {
        let (key, value) = parse_var("KEY=a=b=c").unwrap();
        assert_eq!(key, "KEY");
        assert_eq!(value, "a=b=c");
    }

    #[test]
    fn parse_var_accepts_empty_value() {
        let (key, value) = parse_var("KEY=").unwrap();
        assert_eq!(key, "KEY");
        assert_eq!(value, "");
    }

    #[test]
    fn parse_var_rejects_missing_equals() {
        let err = parse_var("KEYVALUE").unwrap_err();
        assert!(err.to_string().contains("expected KEY=VALUE"));
    }

    #[test]
    fn parse_var_rejects_empty_key() {
        let err = parse_var("=value").unwrap_err();
        assert!(err.to_string().contains("key must not be empty"));
    }

    fn step(uuid: &str) -> PipelineStep {
        PipelineStep {
            uuid: uuid.to_string(),
            name: Some("Step".to_string()),
            started_on: None,
            completed_on: None,
            state: None,
            image: None,
            setup_commands: None,
            script_commands: None,
            links: None,
        }
    }

    #[test]
    fn select_step_by_one_based_index() {
        let steps = [step("{aaa-111}"), step("{bbb-222}")];
        let (index, by_uuid) = select_step(&steps, "2").unwrap();
        assert_eq!(index, 1);
        assert!(!by_uuid);
    }

    #[test]
    fn select_step_rejects_index_zero() {
        let steps = [step("{aaa-111}")];
        assert!(select_step(&steps, "0").is_err());
    }

    #[test]
    fn select_step_rejects_index_out_of_range() {
        let steps = [step("{aaa-111}")];
        assert!(select_step(&steps, "2").is_err());
    }

    #[test]
    fn select_step_by_uuid_prefix_without_braces() {
        let steps = [step("{aaa-111}"), step("{bbb-222}")];
        let (index, by_uuid) = select_step(&steps, "bbb").unwrap();
        assert_eq!(index, 1);
        assert!(by_uuid);
    }

    #[test]
    fn select_step_by_full_braced_uuid() {
        let steps = [step("{aaa-111}"), step("{bbb-222}")];
        let (index, by_uuid) = select_step(&steps, "{bbb-222}").unwrap();
        assert_eq!(index, 1);
        assert!(by_uuid);
    }

    #[test]
    fn select_step_uuid_prefix_is_case_insensitive() {
        let steps = [step("{ABC-123}")];
        let (index, _) = select_step(&steps, "abc").unwrap();
        assert_eq!(index, 0);
    }

    #[test]
    fn select_step_rejects_ambiguous_prefix() {
        let steps = [step("{abc-111}"), step("{abc-222}")];
        let err = select_step(&steps, "abc").unwrap_err();
        assert!(err.to_string().contains("ambiguous"));
    }

    #[test]
    fn select_step_rejects_unknown_prefix() {
        let steps = [step("{aaa-111}")];
        assert!(select_step(&steps, "zzz").is_err());
    }

    #[test]
    fn ensure_braced_uuid_wraps_bare_uuids() {
        assert_eq!(ensure_braced_uuid("abc-123"), "{abc-123}");
    }

    #[test]
    fn ensure_braced_uuid_keeps_braced_uuids() {
        assert_eq!(ensure_braced_uuid("{abc-123}"), "{abc-123}");
    }

    #[test]
    fn find_variable_uuid_matches_key_exactly() {
        let variables = vec![
            crate::models::PipelineVariable {
                uuid: Some("{u1}".to_string()),
                key: Some("FOO".to_string()),
                value: None,
                secured: None,
            },
            crate::models::PipelineVariable {
                uuid: Some("{u2}".to_string()),
                key: Some("FOOBAR".to_string()),
                value: None,
                secured: None,
            },
        ];

        assert_eq!(
            find_variable_uuid(&variables, "FOO").as_deref(),
            Some("{u1}")
        );
        assert_eq!(find_variable_uuid(&variables, "BAR"), None);
    }

    #[test]
    fn schedule_branch_reads_ref_name_from_target() {
        let target = serde_json::json!({"type": "pipeline_ref_target", "ref_name": "main"});
        assert_eq!(schedule_branch(Some(&target)), "main");
    }

    #[test]
    fn schedule_branch_falls_back_to_dash() {
        assert_eq!(schedule_branch(None), "-");
        let target = serde_json::json!({"type": "pipeline_ref_target"});
        assert_eq!(schedule_branch(Some(&target)), "-");
    }

    #[test]
    fn format_bytes_uses_binary_units() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2048), "2.0 KiB");
        assert_eq!(format_bytes(5 * 1024 * 1024), "5.0 MiB");
    }

    fn step_with_state(state: &str, result: Option<&str>) -> PipelineStep {
        PipelineStep {
            uuid: "{step}".to_string(),
            name: Some("Step".to_string()),
            started_on: None,
            completed_on: None,
            state: Some(PipelineStepState {
                name: state.to_string(),
                state_type: "pipeline_step_state".to_string(),
                result: result.map(|r| PipelineStepResult {
                    name: r.to_string(),
                    result_type: "pipeline_step_result".to_string(),
                }),
            }),
            image: None,
            setup_commands: None,
            script_commands: None,
            links: None,
        }
    }

    #[test]
    fn step_status_icon_per_state() {
        // Assert on the underlying glyph via `contains` so the result holds
        // whether or not ANSI colouring is active in the test environment.
        assert!(
            step_status_icon(&step_with_state("COMPLETED", Some("SUCCESSFUL")))
                .to_string()
                .contains('✓')
        );
        assert!(
            step_status_icon(&step_with_state("COMPLETED", Some("FAILED")))
                .to_string()
                .contains('✗')
        );
        assert!(
            step_status_icon(&step_with_state("COMPLETED", Some("STOPPED")))
                .to_string()
                .contains('○')
        );
        assert!(
            step_status_icon(&step_with_state("IN_PROGRESS", None))
                .to_string()
                .contains('◉')
        );
        assert!(
            step_status_icon(&step_with_state("PENDING", None))
                .to_string()
                .contains('○')
        );

        // No state at all falls back to the neutral marker.
        let mut stateless = step_with_state("PENDING", None);
        stateless.state = None;
        assert!(step_status_icon(&stateless).to_string().contains('○'));
    }

    fn variable(key: &str, value: Option<&str>, secured: Option<bool>) -> PipelineVariable {
        PipelineVariable {
            uuid: Some("{u}".to_string()),
            key: Some(key.to_string()),
            value: value.map(|v| v.to_string()),
            secured,
        }
    }

    #[test]
    fn variable_rows_masks_secured_values() {
        let vars = vec![
            variable("PLAIN", Some("hello"), Some(false)),
            variable("SECRET", Some("shh"), Some(true)),
        ];
        let rows = variable_rows(&vars);

        assert_eq!(rows[0].value, "hello");
        assert_eq!(rows[0].secured, "no");

        assert_eq!(rows[1].value, "••••••");
        assert_eq!(rows[1].secured, "yes");
    }
}
