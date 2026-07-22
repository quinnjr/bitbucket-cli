use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use tabled::{Table, Tabled};

use super::{confirm_or_abort, resolve_workspace};
use crate::api::BitbucketClient;
use crate::cli::pagination::{MAX_LIMIT, effective_limit};
use crate::models::{
    CreateProjectRequest, ProjectDetail, UpdateProjectRequest, Workspace, WorkspaceMembership,
    WorkspacePermission,
};

#[derive(Subcommand)]
pub enum WorkspaceCommands {
    /// List workspaces you have access to
    List {
        /// Filter by your role in the workspace
        #[arg(long, value_parser = ["member", "collaborator", "owner"])]
        role: Option<String>,

        /// Filter with a Bitbucket query (BBQL) expression, e.g. 'name~"acme"'
        #[arg(short, long)]
        query: Option<String>,

        /// Sort by a field, e.g. 'name' or '-created_on' for descending
        #[arg(long)]
        sort: Option<String>,

        /// Number of results per page (max 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,

        /// Fetch every page instead of a single page of results
        #[arg(long, conflicts_with_all = ["role", "query", "sort", "limit"])]
        all: bool,
    },

    /// View workspace details
    View {
        /// Workspace slug (defaults to --workspace or the configured default workspace)
        workspace: Option<String>,
    },

    /// List members of a workspace
    Members {
        /// Workspace slug (defaults to --workspace or the configured default workspace)
        workspace: Option<String>,

        /// Number of results per page (max 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// List user permissions on a workspace or one of its repositories
    Permissions {
        /// Workspace slug (defaults to --workspace or the configured default workspace)
        workspace: Option<String>,

        /// Show permissions on this repository instead of the workspace
        #[arg(long)]
        repo: Option<String>,

        /// Number of results per page (max 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// Manage projects in a workspace
    Project {
        #[command(subcommand)]
        command: ProjectCommands,
    },
}

#[derive(Subcommand)]
pub enum ProjectCommands {
    /// List projects in a workspace
    List {
        /// Workspace slug (defaults to --workspace or the configured default workspace)
        workspace: Option<String>,

        /// Filter with a Bitbucket query (BBQL) expression, e.g. 'name~"api"'
        #[arg(short, long)]
        query: Option<String>,

        /// Sort by a field, e.g. 'name' or '-updated_on' for descending
        #[arg(long)]
        sort: Option<String>,

        /// Number of results per page (max 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,

        /// Page number to fetch
        #[arg(long)]
        page: Option<u32>,
    },

    /// View project details
    View {
        /// Project key, e.g. PROJ
        key: String,

        /// Workspace slug (defaults to --workspace or the configured default workspace)
        #[arg(long)]
        workspace: Option<String>,
    },

    /// Create a new project
    Create {
        /// Project key, e.g. PROJ
        key: String,

        /// Project name
        name: String,

        /// Project description
        #[arg(short, long)]
        description: Option<String>,

        /// Make the project private (default)
        #[arg(long, conflicts_with = "public")]
        private: bool,

        /// Make the project public
        #[arg(long)]
        public: bool,

        /// Workspace slug (defaults to --workspace or the configured default workspace)
        #[arg(long)]
        workspace: Option<String>,
    },

    /// Update project settings
    Edit {
        /// Project key, e.g. PROJ
        key: String,

        /// New project name
        #[arg(long)]
        name: Option<String>,

        /// New description
        #[arg(short, long)]
        description: Option<String>,

        /// Make the project private
        #[arg(long, conflicts_with = "public")]
        private: bool,

        /// Make the project public
        #[arg(long)]
        public: bool,

        /// Workspace slug (defaults to --workspace or the configured default workspace)
        #[arg(long)]
        workspace: Option<String>,
    },

    /// Delete a project
    Delete {
        /// Project key, e.g. PROJ
        key: String,

        /// Workspace slug (defaults to --workspace or the configured default workspace)
        #[arg(long)]
        workspace: Option<String>,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Tabled)]
struct WorkspaceRow {
    #[tabled(rename = "SLUG")]
    slug: String,
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "PRIVATE")]
    private: String,
    #[tabled(rename = "CREATED")]
    created: String,
}

#[derive(Tabled)]
struct MemberRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "USERNAME")]
    username: String,
    #[tabled(rename = "UUID")]
    uuid: String,
}

#[derive(Tabled)]
struct PermissionRow {
    #[tabled(rename = "USER")]
    user: String,
    #[tabled(rename = "PERMISSION")]
    permission: String,
}

#[derive(Tabled)]
struct ProjectRow {
    #[tabled(rename = "KEY")]
    key: String,
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "PRIVATE")]
    private: String,
    #[tabled(rename = "UPDATED")]
    updated: String,
}

/// Render an optional privacy flag as Yes/No/-
fn privacy_label(is_private: Option<bool>) -> String {
    match is_private {
        Some(true) => "Yes",
        Some(false) => "No",
        None => "-",
    }
    .to_string()
}

fn workspace_rows(workspaces: &[Workspace]) -> Vec<WorkspaceRow> {
    workspaces
        .iter()
        .map(|w| WorkspaceRow {
            slug: w.slug.clone(),
            name: w.name.clone(),
            private: privacy_label(w.is_private),
            created: w
                .created_on
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
        })
        .collect()
}

fn member_rows(members: &[WorkspaceMembership]) -> Vec<MemberRow> {
    members
        .iter()
        .map(|m| MemberRow {
            name: m
                .user
                .as_ref()
                .map(|u| u.display_name.clone())
                .unwrap_or_else(|| "-".to_string()),
            username: m
                .user
                .as_ref()
                .and_then(|u| u.username.clone())
                .unwrap_or_else(|| "-".to_string()),
            uuid: m
                .user
                .as_ref()
                .map(|u| u.uuid.clone())
                .unwrap_or_else(|| "-".to_string()),
        })
        .collect()
}

fn permission_rows(permissions: &[WorkspacePermission]) -> Vec<PermissionRow> {
    permissions
        .iter()
        .map(|p| PermissionRow {
            user: p
                .user
                .as_ref()
                .map(|u| u.display_name.clone())
                .unwrap_or_else(|| "-".to_string()),
            permission: p.permission.clone().unwrap_or_else(|| "-".to_string()),
        })
        .collect()
}

fn project_rows(projects: &[ProjectDetail]) -> Vec<ProjectRow> {
    projects
        .iter()
        .map(|p| ProjectRow {
            key: p.key.clone(),
            name: p.name.clone(),
            private: privacy_label(p.is_private),
            updated: p
                .updated_on
                .map(|d| d.format("%Y-%m-%d").to_string())
                .unwrap_or_default(),
        })
        .collect()
}

impl WorkspaceCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            WorkspaceCommands::List {
                role,
                query,
                sort,
                limit,
                all,
            } => {
                let client = BitbucketClient::from_stored().await?;

                if all {
                    let workspaces = client.list_workspaces().await?;

                    if super::output_json() {
                        return super::print_json(&workspaces);
                    }

                    if workspaces.is_empty() {
                        println!("No workspaces found");
                        return Ok(());
                    }

                    let table = Table::new(workspace_rows(&workspaces)).to_string();
                    println!("{}", table);

                    return Ok(());
                }

                let limit = effective_limit(limit);
                let page = client
                    .list_workspaces_filtered(
                        role.as_deref(),
                        query.as_deref(),
                        sort.as_deref(),
                        None,
                        Some(limit),
                    )
                    .await?;

                if super::output_json() {
                    return super::print_json(&page.values);
                }

                if page.values.is_empty() {
                    println!("No workspaces found");
                    return Ok(());
                }

                let table = Table::new(workspace_rows(&page.values)).to_string();
                println!("{}", table);

                if page.next.is_some() {
                    println!(
                        "\n{} More workspaces available. Use --all to fetch every workspace.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            WorkspaceCommands::View { workspace } => {
                let workspace = resolve_workspace(workspace)?;
                let client = BitbucketClient::from_stored().await?;
                let workspace = client.get_workspace(&workspace).await?;

                if super::output_json() {
                    return super::print_json(&workspace);
                }

                println!("{}", workspace.name.bold());
                println!("{}", "─".repeat(50));
                println!("{} {}", "Slug:".bold(), workspace.slug);
                println!("{} {}", "UUID:".bold(), workspace.uuid);
                println!(
                    "{} {}",
                    "Private:".bold(),
                    privacy_label(workspace.is_private)
                );
                println!(
                    "{} {}",
                    "Created:".bold(),
                    workspace
                        .created_on
                        .map(|d| d.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| "-".to_string())
                );

                Ok(())
            }

            WorkspaceCommands::Members { workspace, limit } => {
                let workspace = resolve_workspace(workspace)?;
                let limit = effective_limit(limit);
                let client = BitbucketClient::from_stored().await?;
                let members = client
                    .list_workspace_members(&workspace, Some(limit))
                    .await?;

                if super::output_json() {
                    return super::print_json(&members.values);
                }

                if members.values.is_empty() {
                    println!("No members found in workspace '{}'", workspace);
                    return Ok(());
                }

                let table = Table::new(member_rows(&members.values)).to_string();
                println!("{}", table);

                if members.next.is_some() {
                    println!(
                        "\n{} More members available. Increase --limit (max {}) to see more.",
                        "ℹ".blue(),
                        MAX_LIMIT
                    );
                }

                Ok(())
            }

            WorkspaceCommands::Permissions {
                workspace,
                repo,
                limit,
            } => {
                let workspace = resolve_workspace(workspace)?;
                let limit = effective_limit(limit);
                let client = BitbucketClient::from_stored().await?;
                let permissions = client
                    .list_workspace_permissions(&workspace, repo.as_deref(), None, Some(limit))
                    .await?;

                if super::output_json() {
                    return super::print_json(&permissions.values);
                }

                if permissions.values.is_empty() {
                    match &repo {
                        Some(repo) => {
                            println!(
                                "No permissions found on repository '{}/{}'",
                                workspace, repo
                            )
                        }
                        None => println!("No permissions found in workspace '{}'", workspace),
                    }
                    return Ok(());
                }

                let table = Table::new(permission_rows(&permissions.values)).to_string();
                println!("{}", table);

                if permissions.next.is_some() {
                    println!(
                        "\n{} More permissions available. Increase --limit (max {}) to see more.",
                        "ℹ".blue(),
                        MAX_LIMIT
                    );
                }

                Ok(())
            }

            WorkspaceCommands::Project { command } => command.run().await,
        }
    }
}

impl ProjectCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            ProjectCommands::List {
                workspace,
                query,
                sort,
                limit,
                page,
            } => {
                let workspace = resolve_workspace(workspace)?;
                let limit = effective_limit(limit);
                let client = BitbucketClient::from_stored().await?;
                let projects = client
                    .list_projects(
                        &workspace,
                        query.as_deref(),
                        sort.as_deref(),
                        page,
                        Some(limit),
                    )
                    .await?;

                if super::output_json() {
                    return super::print_json(&projects.values);
                }

                if projects.values.is_empty() {
                    println!("No projects found in workspace '{}'", workspace);
                    return Ok(());
                }

                let table = Table::new(project_rows(&projects.values)).to_string();
                println!("{}", table);

                if projects.next.is_some() {
                    println!(
                        "\n{} More projects available. Use --page {} to see the next page.",
                        "ℹ".blue(),
                        page.unwrap_or(1) + 1
                    );
                }

                Ok(())
            }

            ProjectCommands::View { key, workspace } => {
                let workspace = resolve_workspace(workspace)?;
                let client = BitbucketClient::from_stored().await?;
                let project = client.get_project(&workspace, &key).await?;

                if super::output_json() {
                    return super::print_json(&project);
                }

                println!("{} {}", project.key.bold(), project.name);
                println!("{}", "─".repeat(50));

                if let Some(desc) = &project.description {
                    if !desc.is_empty() {
                        println!("{}", desc);
                        println!();
                    }
                }

                println!(
                    "{} {}",
                    "UUID:".bold(),
                    project.uuid.as_deref().unwrap_or("-")
                );
                println!(
                    "{} {}",
                    "Private:".bold(),
                    privacy_label(project.is_private)
                );
                println!(
                    "{} {}",
                    "Created:".bold(),
                    project
                        .created_on
                        .map(|d| d.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| "-".to_string())
                );
                println!(
                    "{} {}",
                    "Updated:".bold(),
                    project
                        .updated_on
                        .map(|d| d.format("%Y-%m-%d").to_string())
                        .unwrap_or_else(|| "-".to_string())
                );

                Ok(())
            }

            ProjectCommands::Create {
                key,
                name,
                description,
                private: _,
                public,
                workspace,
            } => {
                let workspace = resolve_workspace(workspace)?;
                let client = BitbucketClient::from_stored().await?;

                let request = CreateProjectRequest {
                    name,
                    key,
                    description,
                    // Private unless --public was given (private is the default).
                    is_private: Some(!public),
                };

                let project = client.create_project(&workspace, &request).await?;

                if super::output_json() {
                    return super::print_json(&project);
                }

                println!(
                    "{} Created project {} in workspace {}",
                    "✓".green(),
                    project.key.cyan(),
                    workspace
                );

                Ok(())
            }

            ProjectCommands::Edit {
                key,
                name,
                description,
                private,
                public,
                workspace,
            } => {
                let workspace = resolve_workspace(workspace)?;

                let request = UpdateProjectRequest {
                    name,
                    description,
                    is_private: if private {
                        Some(true)
                    } else if public {
                        Some(false)
                    } else {
                        None
                    },
                };

                if request.is_empty() {
                    anyhow::bail!(
                        "No changes specified. Pass at least one of --name, --description, --private, or --public."
                    );
                }

                let client = BitbucketClient::from_stored().await?;
                let project = client.update_project(&workspace, &key, &request).await?;

                if super::output_json() {
                    return super::print_json(&project);
                }

                println!("{} Updated project {}", "✓".green(), project.key.cyan());

                Ok(())
            }

            ProjectCommands::Delete {
                key,
                workspace,
                yes,
            } => {
                let workspace = resolve_workspace(workspace)?;

                if !yes
                    && !confirm_or_abort(format!(
                        "Are you sure you want to delete project {} from workspace {}? This cannot be undone!",
                        key.red(),
                        workspace
                    ))?
                {
                    return Ok(());
                }

                let client = BitbucketClient::from_stored().await?;
                client.delete_project(&workspace, &key).await?;

                if super::output_json() {
                    return super::print_json(&serde_json::json!({"ok": true}));
                }

                println!("{} Deleted project {}", "✓".green(), key);

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::privacy_label;

    #[test]
    fn privacy_label_renders_all_states() {
        assert_eq!(privacy_label(Some(true)), "Yes");
        assert_eq!(privacy_label(Some(false)), "No");
        assert_eq!(privacy_label(None), "-");
    }
}
