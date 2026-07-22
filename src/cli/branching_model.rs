use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use tabled::{Table, Tabled};

use super::{output_json, parse_repo, print_json};
use crate::api::BitbucketClient;
use crate::models::{BranchingModelBranchSettings, ModelBranch, UpdateBranchingModelRequest};

/// Commands for viewing and configuring a repository's branching model.
#[derive(Subcommand)]
pub enum BranchingModelCommands {
    /// View the repository's effective branching model
    View {
        /// Repository in format workspace/repo-slug
        repo: String,
    },

    /// Update the repository's branching model settings
    Set {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Branch to use as the development branch
        #[arg(long, value_name = "BRANCH")]
        development: Option<String>,

        /// Branch to use as the production branch (also enables it)
        #[arg(long, value_name = "BRANCH")]
        production: Option<String>,
    },
}

#[derive(Tabled)]
struct BranchTypeRow {
    #[tabled(rename = "KIND")]
    kind: String,
    #[tabled(rename = "PREFIX")]
    prefix: String,
}

impl BranchingModelCommands {
    /// Execute the branching-model subcommand.
    pub async fn run(self) -> Result<()> {
        match self {
            BranchingModelCommands::View { repo } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let model = client.get_branching_model(&workspace, &repo_slug).await?;

                if output_json() {
                    return print_json(&model);
                }

                println!("{}", format!("Branching model for {}", repo).bold());
                println!("{}", "─".repeat(50));

                println!(
                    "{} {}",
                    "Development:".dimmed(),
                    describe_model_branch(model.development.as_ref())
                );
                println!(
                    "{} {}",
                    "Production:".dimmed(),
                    describe_model_branch(model.production.as_ref())
                );

                match model.branch_types.as_deref() {
                    Some(branch_types) if !branch_types.is_empty() => {
                        println!();
                        println!("{}", "Branch types:".bold());

                        let rows: Vec<BranchTypeRow> = branch_types
                            .iter()
                            .map(|t| BranchTypeRow {
                                kind: t.kind.clone().unwrap_or_else(|| "-".into()),
                                prefix: t.prefix.clone().unwrap_or_else(|| "-".into()),
                            })
                            .collect();

                        println!("{}", Table::new(rows));
                    }
                    _ => {
                        println!();
                        println!("No branch types configured");
                    }
                }

                Ok(())
            }

            BranchingModelCommands::Set {
                repo,
                development,
                production,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                let request = build_branching_model_update(development, production);
                if request.is_empty() {
                    anyhow::bail!("Nothing to update. Pass --development and/or --production.");
                }

                let client = BitbucketClient::from_stored().await?;
                let updated = client
                    .update_branching_model_settings(&workspace, &repo_slug, &request)
                    .await?;

                if output_json() {
                    return print_json(&updated);
                }

                println!(
                    "{} Updated branching model for {}",
                    "✓".green(),
                    repo.cyan()
                );
                println!(
                    "{} {}",
                    "Development:".dimmed(),
                    describe_model_branch(updated.development.as_ref())
                );
                println!(
                    "{} {}",
                    "Production:".dimmed(),
                    describe_model_branch(updated.production.as_ref())
                );

                Ok(())
            }
        }
    }
}

/// Build the settings update for `branching-model set` from its flags. A
/// named branch implies `use_mainbranch: false` (the API ignores the name
/// otherwise), and naming a production branch also enables it.
fn build_branching_model_update(
    development: Option<String>,
    production: Option<String>,
) -> UpdateBranchingModelRequest {
    UpdateBranchingModelRequest {
        development: development.map(|name| BranchingModelBranchSettings {
            name: Some(name),
            use_mainbranch: Some(false),
            enabled: None,
        }),
        production: production.map(|name| BranchingModelBranchSettings {
            name: Some(name),
            use_mainbranch: Some(false),
            enabled: Some(true),
        }),
    }
}

/// One-line description of a branching model branch: the resolved branch name
/// when the model is effective, else the configured name, marking branches
/// that track the repository's main branch.
fn describe_model_branch(branch: Option<&ModelBranch>) -> String {
    let Some(branch) = branch else {
        return "(not set)".to_string();
    };

    let name = branch
        .branch
        .as_ref()
        .map(|b| b.name.as_str())
        .or(branch.name.as_deref());

    match (name, branch.use_mainbranch.unwrap_or(false)) {
        (Some(name), true) => format!("{} (main branch)", name),
        (Some(name), false) => name.to_string(),
        (None, true) => "(main branch)".to_string(),
        (None, false) => "-".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Branch;

    #[test]
    fn build_update_with_no_flags_is_empty() {
        assert!(build_branching_model_update(None, None).is_empty());
    }

    #[test]
    fn build_update_sets_development_branch_by_name() {
        let request = build_branching_model_update(Some("develop".into()), None);

        let development = request.development.expect("development should be set");
        assert_eq!(development.name.as_deref(), Some("develop"));
        assert_eq!(development.use_mainbranch, Some(false));
        assert_eq!(development.enabled, None);
        assert!(request.production.is_none());
    }

    #[test]
    fn build_update_enables_production_branch() {
        let request = build_branching_model_update(None, Some("release".into()));

        let production = request.production.expect("production should be set");
        assert_eq!(production.name.as_deref(), Some("release"));
        assert_eq!(production.use_mainbranch, Some(false));
        assert_eq!(production.enabled, Some(true));
        assert!(request.development.is_none());
    }

    fn model_branch(
        name: Option<&str>,
        resolved: Option<&str>,
        use_mainbranch: Option<bool>,
    ) -> ModelBranch {
        ModelBranch {
            name: name.map(String::from),
            branch: resolved.map(|n| Branch {
                name: n.to_string(),
                branch_type: None,
            }),
            use_mainbranch,
        }
    }

    #[test]
    fn describe_model_branch_prefers_resolved_branch_name() {
        let branch = model_branch(Some("development"), Some("main"), Some(true));
        assert_eq!(describe_model_branch(Some(&branch)), "main (main branch)");
    }

    #[test]
    fn describe_model_branch_uses_configured_name_without_resolution() {
        let branch = model_branch(Some("develop"), None, Some(false));
        assert_eq!(describe_model_branch(Some(&branch)), "develop");
    }

    #[test]
    fn describe_model_branch_handles_unnamed_mainbranch_tracking() {
        let branch = model_branch(None, None, Some(true));
        assert_eq!(describe_model_branch(Some(&branch)), "(main branch)");
    }

    #[test]
    fn describe_model_branch_handles_missing_branch() {
        assert_eq!(describe_model_branch(None), "(not set)");
        let empty = model_branch(None, None, None);
        assert_eq!(describe_model_branch(Some(&empty)), "-");
    }
}
