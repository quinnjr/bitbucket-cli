pub mod auth;
pub mod issue;
pub mod pipeline;
pub mod pr;
pub mod repo;
pub mod workspace;

use std::sync::OnceLock;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bitbucket")]
#[command(author = "Pegasus Heavy Industries")]
#[command(version)]
#[command(about = "A command-line interface for Bitbucket Cloud", long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Workspace to use when omitted from arguments (overrides config default)
    #[arg(long, global = true)]
    pub workspace: Option<String>,
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

    /// Launch interactive TUI
    Tui,
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
