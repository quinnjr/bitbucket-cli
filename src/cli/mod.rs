pub mod auth;
pub mod branch;
pub mod branch_restriction;
pub mod branching_model;
pub mod commit;
pub mod deploy;
pub mod file;
pub mod issue;
pub mod pagination;
pub mod permission;
pub mod pipeline;
pub mod pr;
pub mod repo;
pub mod reviewer;
pub mod tag;
pub mod user;
pub mod webhook;
pub mod workspace;

use std::sync::OnceLock;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bitbucket")]
#[command(author = "quinnjr")]
#[command(version)]
#[command(about = "A command-line interface for Bitbucket Cloud", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Workspace to use when omitted from arguments (overrides config default)
    #[arg(long, global = true)]
    pub workspace: Option<String>,

    /// Output format for command results
    #[arg(long, global = true, value_enum, default_value_t = OutputFormat::Table)]
    pub output: OutputFormat,
}

/// How command results are rendered.
#[derive(Clone, Copy, PartialEq, Eq, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    /// Human-readable tables and text
    Table,
    /// Raw JSON for scripting
    Json,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Manage authentication with Bitbucket
    Auth {
        #[command(subcommand)]
        command: auth::AuthCommands,
    },

    /// Manage repositories
    Repo {
        #[command(subcommand)]
        command: repo::RepoCommands,
    },

    /// Manage pull requests
    Pr {
        #[command(subcommand)]
        command: pr::PrCommands,
    },

    /// Manage issues
    Issue {
        #[command(subcommand)]
        command: issue::IssueCommands,
    },

    /// Manage pipelines
    Pipeline {
        #[command(subcommand)]
        command: pipeline::PipelineCommands,
    },

    /// Manage workspaces
    Workspace {
        #[command(subcommand)]
        command: workspace::WorkspaceCommands,
    },

    /// Inspect Bitbucket user accounts
    User {
        #[command(subcommand)]
        command: user::UserCommands,
    },

    /// Launch interactive TUI
    Tui,
}

static OUTPUT_FORMAT: OnceLock<OutputFormat> = OnceLock::new();

/// Record the global `--output` choice for this process.
///
/// `main` must call this once before dispatching a command; later calls are
/// no-ops (set-once cell).
pub fn set_output_format(format: OutputFormat) {
    let _ = OUTPUT_FORMAT.set(format);
}

/// True when `--output json` was passed; handlers should emit raw JSON.
pub fn output_json() -> bool {
    OUTPUT_FORMAT.get().copied() == Some(OutputFormat::Json)
}

/// Serialize a value as pretty JSON to stdout.
pub(crate) fn print_json<T: serde::Serialize>(value: &T) -> anyhow::Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

static WORKSPACE_OVERRIDE: OnceLock<Option<String>> = OnceLock::new();

/// Record the global `--workspace` override for this process.
///
/// `main` must call this once before dispatching a command; the value is
/// stored in a set-once cell, so any later call is a no-op. Commands that
/// run before it is set fall back to the config file alone.
pub fn set_workspace_override(workspace: Option<String>) {
    let _ = WORKSPACE_OVERRIDE.set(workspace);
}

fn workspace_override() -> Option<String> {
    WORKSPACE_OVERRIDE.get().cloned().flatten()
}

fn fallback_workspace() -> Option<String> {
    workspace_override().or_else(|| {
        crate::config::Config::load()
            .ok()
            .and_then(|c| c.default_workspace().map(String::from))
    })
}

/// Resolve a workspace given optionally on the command line, falling back to
/// the `--workspace` override recorded by [`set_workspace_override`] and then
/// to `default_workspace` from the config file (read from disk on each call).
pub fn resolve_workspace(explicit: Option<String>) -> anyhow::Result<String> {
    resolve_workspace_with(explicit, fallback_workspace())
}

fn resolve_workspace_with(
    explicit: Option<String>,
    fallback: Option<String>,
) -> anyhow::Result<String> {
    explicit
        .filter(|w| !w.is_empty())
        .or(fallback.filter(|w| !w.is_empty()))
        .ok_or_else(|| {
            anyhow::anyhow!(
                "No workspace specified. Pass a workspace argument, use --workspace, or set a default workspace in the config."
            )
        })
}

/// Ask the user to confirm a destructive action; on decline, print "Aborted"
/// and return Ok(false).
pub(crate) fn confirm_or_abort(prompt: String) -> anyhow::Result<bool> {
    use dialoguer::Confirm;
    let confirmed = Confirm::new()
        .with_prompt(prompt)
        .default(false)
        .interact()?;

    if !confirmed {
        // stderr so an aborted run never corrupts `--output json` on stdout
        eprintln!("Aborted");
    }

    Ok(confirmed)
}

/// Parse `workspace/repo-slug`, or a bare `repo-slug` resolved against
/// --workspace / the configured default workspace.
///
/// Resolution order for a bare slug: the process-wide override recorded by
/// [`set_workspace_override`], then `default_workspace` from the config file
/// (read from disk on each call).
pub fn parse_repo(repo: &str) -> anyhow::Result<(String, String)> {
    let fallback = fallback_workspace();
    parse_repo_with(repo, fallback.as_deref())
}

/// Resolve `repo create`'s positional pair: an explicit second argument makes
/// the first the workspace; otherwise the first is `workspace/name` or a bare
/// name resolved like any repo argument.
pub fn resolve_create_target(
    first: String,
    second: Option<String>,
) -> anyhow::Result<(String, String)> {
    let fallback = fallback_workspace();
    resolve_create_target_with(first, second, fallback.as_deref())
}

fn resolve_create_target_with(
    first: String,
    second: Option<String>,
    fallback_workspace: Option<&str>,
) -> anyhow::Result<(String, String)> {
    match second {
        Some(name) => {
            if first.contains('/') {
                anyhow::bail!("Pass either 'workspace/name' or '<workspace> <name>', not both");
            }
            Ok((first, name))
        }
        None => parse_repo_with(&first, fallback_workspace),
    }
}

fn parse_repo_with(
    repo: &str,
    fallback_workspace: Option<&str>,
) -> anyhow::Result<(String, String)> {
    if repo.contains('/') {
        let parts: Vec<&str> = repo.split('/').collect();
        if parts.len() == 2 && !parts[0].is_empty() && !parts[1].is_empty() {
            return Ok((parts[0].to_string(), parts[1].to_string()));
        }
        anyhow::bail!(
            "Invalid repository format. Expected 'workspace/repo-slug', got '{}'",
            repo
        );
    }

    if repo.is_empty() {
        anyhow::bail!(
            "Invalid repository format. Expected 'workspace/repo-slug', got '{}'",
            repo
        );
    }

    match fallback_workspace {
        Some(workspace) if !workspace.is_empty() => Ok((workspace.to_string(), repo.to_string())),
        _ => anyhow::bail!(
            "No workspace specified. Use 'workspace/repo-slug', pass --workspace, or set a default workspace in the config."
        ),
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    #[test]
    fn cli_parser_is_valid() {
        super::Cli::command().debug_assert();
    }

    #[test]
    fn parse_repo_with_accepts_workspace_and_slug() {
        let (workspace, slug) = super::parse_repo_with("ws/slug", None).unwrap();
        assert_eq!(workspace, "ws");
        assert_eq!(slug, "slug");
    }

    #[test]
    fn parse_repo_with_rejects_wrong_arity() {
        assert!(super::parse_repo_with("a/b/c", None).is_err());
    }

    #[test]
    fn parse_repo_with_rejects_empty_components() {
        assert!(super::parse_repo_with("ws/", Some("fallback")).is_err());
        assert!(super::parse_repo_with("/repo", Some("fallback")).is_err());
        assert!(super::parse_repo_with("/", Some("fallback")).is_err());
        assert!(super::parse_repo_with("", Some("fallback")).is_err());
    }

    #[test]
    fn parse_repo_with_resolves_bare_slug_against_fallback() {
        let (workspace, slug) = super::parse_repo_with("slug", Some("ws")).unwrap();
        assert_eq!(workspace, "ws");
        assert_eq!(slug, "slug");
    }

    #[test]
    fn parse_repo_with_rejects_bare_slug_without_fallback() {
        assert!(super::parse_repo_with("slug", None).is_err());
        assert!(super::parse_repo_with("slug", Some("")).is_err());
    }

    #[test]
    fn resolve_create_target_with_takes_explicit_workspace_and_name() {
        let (workspace, name) =
            super::resolve_create_target_with("ws".into(), Some("name".into()), None).unwrap();
        assert_eq!(workspace, "ws");
        assert_eq!(name, "name");
    }

    #[test]
    fn resolve_create_target_with_splits_slashed_first_argument() {
        let (workspace, name) =
            super::resolve_create_target_with("ws/name".into(), None, None).unwrap();
        assert_eq!(workspace, "ws");
        assert_eq!(name, "name");
    }

    #[test]
    fn resolve_create_target_with_resolves_bare_name_against_fallback() {
        let (workspace, name) =
            super::resolve_create_target_with("name".into(), None, Some("acme")).unwrap();
        assert_eq!(workspace, "acme");
        assert_eq!(name, "name");
    }

    #[test]
    fn resolve_create_target_with_rejects_bare_name_without_fallback() {
        assert!(super::resolve_create_target_with("name".into(), None, None).is_err());
    }

    #[test]
    fn resolve_create_target_with_rejects_slashed_workspace_plus_explicit_name() {
        assert!(
            super::resolve_create_target_with("ws/x".into(), Some("name".into()), None).is_err()
        );
    }

    #[test]
    fn resolve_workspace_with_prefers_explicit_over_fallback() {
        let ws = super::resolve_workspace_with(Some("cli-ws".into()), Some("cfg-ws".into()));
        assert_eq!(ws.unwrap(), "cli-ws");
    }

    #[test]
    fn resolve_workspace_with_uses_fallback_when_explicit_missing_or_empty() {
        assert_eq!(
            super::resolve_workspace_with(None, Some("cfg-ws".into())).unwrap(),
            "cfg-ws"
        );
        assert_eq!(
            super::resolve_workspace_with(Some(String::new()), Some("cfg-ws".into())).unwrap(),
            "cfg-ws"
        );
    }

    #[test]
    fn resolve_workspace_with_errors_when_nothing_available() {
        assert!(super::resolve_workspace_with(None, None).is_err());
        assert!(super::resolve_workspace_with(None, Some(String::new())).is_err());
    }

    // The only test allowed to touch the process-wide override: OnceLock is
    // set-once, so a second test setting a different value could not pass.
    #[test]
    fn parse_repo_resolves_bare_slug_via_workspace_override() {
        super::set_workspace_override(Some("override-ws".to_string()));
        let (workspace, slug) = super::parse_repo("slug").unwrap();
        assert_eq!(workspace, "override-ws");
        assert_eq!(slug, "slug");
    }
}
