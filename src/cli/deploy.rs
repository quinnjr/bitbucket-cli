use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Subcommand, ValueEnum};
use colored::Colorize;
use tabled::{Table, Tabled};

use super::{confirm_or_abort, output_json, parse_repo, print_json};
use crate::api::BitbucketClient;
use crate::cli::pagination::{MAX_LIMIT, effective_limit as capped_limit};
use crate::models::{
    AddDeployKeyRequest, CreateEnvironmentRequest, Deployment, Environment, EnvironmentType,
};

/// How many characters of SSH key material to show in list tables.
const KEY_DISPLAY_CHARS: usize = 40;

/// Commands for managing repository deploy keys.
#[derive(Subcommand)]
pub enum DeployKeyCommands {
    /// List deploy keys for a repository
    List {
        /// Repository in format workspace/repo-slug
        repo: String,
    },

    /// Add a deploy key to a repository
    Add {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// SSH public key material (e.g. "ssh-ed25519 AAAA... user@host")
        #[arg(
            long,
            conflicts_with = "key_file",
            required_unless_present = "key_file"
        )]
        key: Option<String>,

        /// Read the SSH public key from a file
        #[arg(long, value_name = "PATH", required_unless_present = "key")]
        key_file: Option<PathBuf>,

        /// Label for the deploy key
        #[arg(long)]
        label: String,
    },

    /// Delete a deploy key from a repository
    Delete {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Deploy key ID (see `deploy-key list`)
        id: u64,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

/// Commands for managing deployment environments.
#[derive(Subcommand)]
pub enum EnvironmentCommands {
    /// List deployment environments for a repository
    List {
        /// Repository in format workspace/repo-slug
        repo: String,
    },

    /// Create a deployment environment
    Create {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Environment name
        name: String,

        /// Environment type
        #[arg(
            long = "type",
            value_name = "TYPE",
            value_enum,
            ignore_case = true,
            default_value = "test"
        )]
        environment_type: EnvironmentTypeArg,
    },

    /// Delete a deployment environment
    Delete {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Environment UUID (see `environment list`)
        uuid: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

/// Commands for inspecting deployments.
#[derive(Subcommand)]
pub enum DeploymentCommands {
    /// List deployments for a repository
    List {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Number of results (max 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// View deployment details
    View {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Deployment UUID
        uuid: String,
    },
}

/// Environment category accepted by `environment create --type`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum EnvironmentTypeArg {
    Test,
    Staging,
    Production,
}

impl EnvironmentTypeArg {
    /// The type name the Bitbucket API expects for this environment category.
    fn api_name(self) -> &'static str {
        match self {
            EnvironmentTypeArg::Test => "Test",
            EnvironmentTypeArg::Staging => "Staging",
            EnvironmentTypeArg::Production => "Production",
        }
    }
}

#[derive(Tabled)]
struct DeployKeyRow {
    #[tabled(rename = "ID")]
    id: String,
    #[tabled(rename = "LABEL")]
    label: String,
    #[tabled(rename = "KEY")]
    key: String,
}

impl DeployKeyCommands {
    /// Execute the deploy-key subcommand.
    pub async fn run(self) -> Result<()> {
        match self {
            DeployKeyCommands::List { repo } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let keys = client.list_deploy_keys(&workspace, &repo_slug).await?;

                if output_json() {
                    return print_json(&keys.values);
                }

                if keys.values.is_empty() {
                    println!("No deploy keys found in {}", repo);
                    return Ok(());
                }

                let rows: Vec<DeployKeyRow> = keys
                    .values
                    .iter()
                    .map(|k| DeployKeyRow {
                        id: k.id.map(|id| id.to_string()).unwrap_or_else(|| "-".into()),
                        label: k.label.clone().unwrap_or_default(),
                        key: truncate_chars(k.key.as_deref().unwrap_or("-"), KEY_DISPLAY_CHARS),
                    })
                    .collect();

                println!("{}", Table::new(rows));

                Ok(())
            }

            DeployKeyCommands::Add {
                repo,
                key,
                key_file,
                label,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                // clap enforces exactly one key source; the bail arms are a
                // safety net in case this is ever called programmatically.
                let material = match (key, key_file) {
                    (Some(key), None) => key,
                    (None, Some(path)) => std::fs::read_to_string(&path)
                        .with_context(|| format!("Failed to read key file '{}'", path.display()))?,
                    (Some(_), Some(_)) => {
                        anyhow::bail!("Pass either --key or --key-file, not both")
                    }
                    (None, None) => anyhow::bail!("Pass one of --key or --key-file"),
                };
                let key = normalize_key(&material)?;

                let request = AddDeployKeyRequest { key, label };

                let client = BitbucketClient::from_stored().await?;
                let added = client
                    .add_deploy_key(&workspace, &repo_slug, &request)
                    .await?;

                if output_json() {
                    return print_json(&added);
                }

                println!(
                    "{} Added deploy key {} to {}",
                    "✓".green(),
                    added.label.as_deref().unwrap_or(&request.label).cyan(),
                    repo.cyan()
                );

                if let Some(id) = added.id {
                    println!("{} {}", "ID:".dimmed(), id);
                }

                Ok(())
            }

            DeployKeyCommands::Delete { repo, id, yes } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                if !yes
                    && !confirm_or_abort(format!(
                        "Delete deploy key {} from {}?",
                        id.to_string().red(),
                        repo
                    ))?
                {
                    return Ok(());
                }

                let client = BitbucketClient::from_stored().await?;
                client.delete_deploy_key(&workspace, &repo_slug, id).await?;

                if output_json() {
                    return print_json(&serde_json::json!({ "ok": true }));
                }

                println!("{} Deleted deploy key {}", "✓".green(), id);

                Ok(())
            }
        }
    }
}

#[derive(Tabled)]
struct EnvironmentRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "TYPE")]
    environment_type: String,
    #[tabled(rename = "UUID")]
    uuid: String,
}

impl EnvironmentCommands {
    /// Execute the environment subcommand.
    pub async fn run(self) -> Result<()> {
        match self {
            EnvironmentCommands::List { repo } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let environments = client.list_environments(&workspace, &repo_slug).await?;

                if output_json() {
                    return print_json(&environments.values);
                }

                if environments.values.is_empty() {
                    println!("No environments found in {}", repo);
                    return Ok(());
                }

                let rows: Vec<EnvironmentRow> = environments
                    .values
                    .iter()
                    .map(|e| EnvironmentRow {
                        name: e.name.clone().unwrap_or_default(),
                        environment_type: e
                            .environment_type
                            .as_ref()
                            .and_then(|t| t.name.clone())
                            .unwrap_or_else(|| "-".into()),
                        uuid: e.uuid.clone().unwrap_or_else(|| "-".into()),
                    })
                    .collect();

                println!("{}", Table::new(rows));

                Ok(())
            }

            EnvironmentCommands::Create {
                repo,
                name,
                environment_type,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                let request = CreateEnvironmentRequest {
                    name,
                    environment_type: EnvironmentType {
                        name: Some(environment_type.api_name().to_string()),
                    },
                };

                let client = BitbucketClient::from_stored().await?;
                let created = client
                    .create_environment(&workspace, &repo_slug, &request)
                    .await?;

                if output_json() {
                    return print_json(&created);
                }

                println!(
                    "{} Created {} environment {}",
                    "✓".green(),
                    environment_type.api_name(),
                    created.name.as_deref().unwrap_or(&request.name).cyan()
                );

                if let Some(uuid) = &created.uuid {
                    println!("{} {}", "UUID:".dimmed(), uuid);
                }

                Ok(())
            }

            EnvironmentCommands::Delete { repo, uuid, yes } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let uuid = ensure_braced(&uuid);

                if !yes
                    && !confirm_or_abort(format!(
                        "Delete environment {} from {}?",
                        uuid.red(),
                        repo
                    ))?
                {
                    return Ok(());
                }

                let client = BitbucketClient::from_stored().await?;
                client
                    .delete_environment(&workspace, &repo_slug, &uuid)
                    .await?;

                if output_json() {
                    return print_json(&serde_json::json!({ "ok": true }));
                }

                println!("{} Deleted environment {}", "✓".green(), uuid);

                Ok(())
            }
        }
    }
}

#[derive(Tabled)]
struct DeploymentRow {
    #[tabled(rename = "UUID")]
    uuid: String,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "ENVIRONMENT")]
    environment: String,
}

impl DeploymentCommands {
    /// Execute the deployment subcommand.
    pub async fn run(self) -> Result<()> {
        match self {
            DeploymentCommands::List { repo, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                let capped = capped_limit(limit);
                if capped != limit && !output_json() {
                    println!(
                        "{} Limit capped at {} (the Bitbucket API maximum)",
                        "ℹ".blue(),
                        MAX_LIMIT
                    );
                }

                let client = BitbucketClient::from_stored().await?;
                let deployments = client
                    .list_deployments(&workspace, &repo_slug, Some(capped))
                    .await?;

                if output_json() {
                    return print_json(&deployments.values);
                }

                if deployments.values.is_empty() {
                    println!("No deployments found in {}", repo);
                    return Ok(());
                }

                let rows: Vec<DeploymentRow> = deployments
                    .values
                    .iter()
                    .map(|d| DeploymentRow {
                        uuid: d.uuid.clone().unwrap_or_else(|| "-".into()),
                        status: color_status(&deployment_status(d)),
                        environment: environment_label(d.environment.as_ref()),
                    })
                    .collect();

                println!("{}", Table::new(rows));

                if deployments.next.is_some() {
                    println!(
                        "\n{} More deployments available. Use --limit to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            DeploymentCommands::View { repo, uuid } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let uuid = ensure_braced(&uuid);

                let client = BitbucketClient::from_stored().await?;
                let deployment = client.get_deployment(&workspace, &repo_slug, &uuid).await?;

                if output_json() {
                    return print_json(&deployment);
                }

                println!(
                    "{}",
                    format!("Deployment {}", deployment.uuid.as_deref().unwrap_or(&uuid)).bold()
                );
                println!("{}", "─".repeat(60));

                println!(
                    "{} {}",
                    "Status:".dimmed(),
                    color_status(&deployment_status(&deployment))
                );
                println!(
                    "{} {}",
                    "Environment:".dimmed(),
                    environment_label(deployment.environment.as_ref())
                );

                if let Some(summary) = deployment.release.as_ref().and_then(release_summary) {
                    println!("{} {}", "Release:".dimmed(), summary);
                }

                Ok(())
            }
        }
    }
}

/// Trim surrounding whitespace from SSH key material (public keys are a
/// single line; files usually end with a newline), rejecting empty input.
fn normalize_key(material: &str) -> Result<String> {
    let key = material.trim();
    if key.is_empty() {
        anyhow::bail!("The provided SSH key is empty");
    }
    Ok(key.to_string())
}

/// Truncate a string to at most `max` characters, appending an ellipsis when
/// anything was cut off.
fn truncate_chars(s: &str, max: usize) -> String {
    let mut out: String = s.chars().take(max).collect();
    if s.chars().count() > max {
        out.push('…');
    }
    out
}

/// Wrap a UUID in the braces Bitbucket expects (`{...}`) unless it already
/// has them, so users can paste UUIDs with or without braces.
fn ensure_braced(uuid: &str) -> String {
    let uuid = uuid.trim();
    if uuid.starts_with('{') && uuid.ends_with('}') && uuid.len() >= 2 {
        uuid.to_string()
    } else {
        format!("{{{}}}", uuid)
    }
}

/// The most specific status string available for a deployment: the nested
/// status name when present, then the state name, then "-".
fn deployment_status(deployment: &Deployment) -> String {
    deployment
        .state
        .as_ref()
        .and_then(|state| {
            state
                .status
                .as_ref()
                .and_then(|status| status.get("name"))
                .and_then(|name| name.as_str())
                .map(str::to_string)
                .or_else(|| state.name.clone())
        })
        .unwrap_or_else(|| "-".to_string())
}

/// Human-readable label for a deployment's environment: its name when known,
/// falling back to the UUID, then "-".
fn environment_label(environment: Option<&Environment>) -> String {
    environment
        .and_then(|e| e.name.clone().or_else(|| e.uuid.clone()))
        .unwrap_or_else(|| "-".to_string())
}

/// Short summary of a deployment release: its name and, when present, the
/// abbreviated commit hash it points at.
fn release_summary(release: &serde_json::Value) -> Option<String> {
    let name = release.get("name").and_then(|v| v.as_str());
    let commit = release
        .get("commit")
        .and_then(|c| c.get("hash"))
        .and_then(|h| h.as_str())
        .map(|h| h.chars().take(12).collect::<String>());

    match (name, commit) {
        (Some(name), Some(commit)) => Some(format!("{} ({})", name, commit)),
        (Some(name), None) => Some(name.to_string()),
        (None, Some(commit)) => Some(commit),
        (None, None) => None,
    }
}

/// Colorize a deployment status string for terminal output.
fn color_status(status: &str) -> String {
    match status {
        "COMPLETED" | "SUCCESSFUL" => status.green().to_string(),
        "FAILED" | "ERROR" | "STOPPED" => status.red().to_string(),
        "IN_PROGRESS" => status.blue().to_string(),
        "PENDING" | "UNDEPLOYED" => status.dimmed().to_string(),
        _ => status.normal().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;
    use crate::models::DeploymentState;

    #[test]
    fn normalize_key_trims_surrounding_whitespace() {
        let key = normalize_key("  ssh-ed25519 AAAA host\n").unwrap();
        assert_eq!(key, "ssh-ed25519 AAAA host");
    }

    #[test]
    fn normalize_key_rejects_empty_input() {
        assert!(normalize_key("").is_err());
        assert!(normalize_key("  \n\t ").is_err());
    }

    #[test]
    fn truncate_chars_leaves_short_strings_alone() {
        assert_eq!(truncate_chars("short", 40), "short");
        assert_eq!(truncate_chars("1234567890", 10), "1234567890");
    }

    #[test]
    fn truncate_chars_cuts_long_strings_with_ellipsis() {
        assert_eq!(truncate_chars("abcdef", 3), "abc…");
    }

    #[test]
    fn truncate_chars_counts_characters_not_bytes() {
        assert_eq!(truncate_chars("ééééé", 3), "ééé…");
    }

    #[test]
    fn ensure_braced_wraps_bare_uuids() {
        assert_eq!(ensure_braced("abc-123"), "{abc-123}");
    }

    #[test]
    fn ensure_braced_keeps_existing_braces() {
        assert_eq!(ensure_braced("{abc-123}"), "{abc-123}");
    }

    #[test]
    fn ensure_braced_trims_whitespace() {
        assert_eq!(ensure_braced(" abc "), "{abc}");
        assert_eq!(ensure_braced(" {abc} "), "{abc}");
    }

    #[test]
    fn environment_type_arg_maps_to_api_names() {
        assert_eq!(EnvironmentTypeArg::Test.api_name(), "Test");
        assert_eq!(EnvironmentTypeArg::Staging.api_name(), "Staging");
        assert_eq!(EnvironmentTypeArg::Production.api_name(), "Production");
    }

    fn deployment_with_state(state: Option<DeploymentState>) -> Deployment {
        Deployment {
            uuid: None,
            state,
            environment: None,
            release: None,
        }
    }

    #[test]
    fn deployment_status_prefers_nested_status_name() {
        let deployment = deployment_with_state(Some(DeploymentState {
            name: Some("COMPLETED".into()),
            status: Some(json!({ "name": "SUCCESSFUL" })),
        }));
        assert_eq!(deployment_status(&deployment), "SUCCESSFUL");
    }

    #[test]
    fn deployment_status_falls_back_to_state_name() {
        let deployment = deployment_with_state(Some(DeploymentState {
            name: Some("IN_PROGRESS".into()),
            status: None,
        }));
        assert_eq!(deployment_status(&deployment), "IN_PROGRESS");
    }

    #[test]
    fn deployment_status_defaults_to_dash() {
        assert_eq!(deployment_status(&deployment_with_state(None)), "-");
    }

    #[test]
    fn environment_label_prefers_name_over_uuid() {
        let environment = Environment {
            uuid: Some("{u}".into()),
            name: Some("Production".into()),
            environment_type: None,
        };
        assert_eq!(environment_label(Some(&environment)), "Production");

        let unnamed = Environment {
            uuid: Some("{u}".into()),
            name: None,
            environment_type: None,
        };
        assert_eq!(environment_label(Some(&unnamed)), "{u}");

        assert_eq!(environment_label(None), "-");
    }

    #[test]
    fn release_summary_combines_name_and_short_hash() {
        let release = json!({
            "name": "v1.2.3",
            "commit": { "hash": "0123456789abcdef0123" }
        });
        assert_eq!(release_summary(&release).unwrap(), "v1.2.3 (0123456789ab)");
    }

    #[test]
    fn release_summary_handles_partial_data() {
        assert_eq!(release_summary(&json!({ "name": "v1" })).unwrap(), "v1");
        assert_eq!(
            release_summary(&json!({ "commit": { "hash": "abc123" } })).unwrap(),
            "abc123"
        );
        assert!(release_summary(&json!({})).is_none());
    }
}
