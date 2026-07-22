use anyhow::Result;
use clap::Subcommand;
use colored::Colorize;
use tabled::{Table, Tabled};

use super::{confirm_or_abort, output_json, parse_repo, print_json};
use crate::api::BitbucketClient;

#[derive(Subcommand)]
pub enum PermissionCommands {
    /// List users with explicit permissions on a repository
    ListUsers {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Number of results per page (max 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// List groups with explicit permissions on a repository
    ListGroups {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Number of results per page (max 100)
        #[arg(short, long, default_value = "25")]
        limit: u32,
    },

    /// Grant or update a user's permission
    GrantUser {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Account ID or brace-wrapped UUID of the user
        user: String,

        /// Permission level: read, write, or admin
        #[arg(short, long)]
        permission: String,
    },

    /// Revoke a user's explicit permission
    RevokeUser {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Account ID or brace-wrapped UUID of the user
        user: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },

    /// Grant or update a group's permission
    GrantGroup {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Slug of the workspace group
        group: String,

        /// Permission level: read, write, or admin
        #[arg(short, long)]
        permission: String,
    },

    /// Revoke a group's explicit permission
    RevokeGroup {
        /// Repository in format workspace/repo-slug
        repo: String,

        /// Slug of the workspace group
        group: String,

        /// Skip confirmation prompt
        #[arg(short, long)]
        yes: bool,
    },
}

#[derive(Tabled)]
struct UserPermissionRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "UUID")]
    uuid: String,
    #[tabled(rename = "PERMISSION")]
    permission: String,
}

#[derive(Tabled)]
struct GroupPermissionRow {
    #[tabled(rename = "NAME")]
    name: String,
    #[tabled(rename = "SLUG")]
    slug: String,
    #[tabled(rename = "PERMISSION")]
    permission: String,
}

impl PermissionCommands {
    pub async fn run(self) -> Result<()> {
        match self {
            PermissionCommands::ListUsers { repo, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let permissions = client
                    .list_user_permissions(&workspace, &repo_slug, Some(limit.clamp(1, 100)))
                    .await?;

                if output_json() {
                    return print_json(&permissions.values);
                }

                if permissions.values.is_empty() {
                    println!("No explicit user permissions found in {}", repo);
                    return Ok(());
                }

                let rows: Vec<UserPermissionRow> = permissions
                    .values
                    .iter()
                    .map(|p| UserPermissionRow {
                        name: p
                            .user
                            .as_ref()
                            .map(|u| u.display_name.clone())
                            .unwrap_or_default(),
                        uuid: p.user.as_ref().map(|u| u.uuid.clone()).unwrap_or_default(),
                        permission: p.permission.clone(),
                    })
                    .collect();

                println!("{}", Table::new(rows));

                if permissions.next.is_some() {
                    println!(
                        "\n{} More user permissions available. Use --limit to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            PermissionCommands::ListGroups { repo, limit } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;
                let permissions = client
                    .list_group_permissions(&workspace, &repo_slug, Some(limit.clamp(1, 100)))
                    .await?;

                if output_json() {
                    return print_json(&permissions.values);
                }

                if permissions.values.is_empty() {
                    println!("No explicit group permissions found in {}", repo);
                    return Ok(());
                }

                let rows: Vec<GroupPermissionRow> = permissions
                    .values
                    .iter()
                    .map(|p| GroupPermissionRow {
                        name: p
                            .group
                            .as_ref()
                            .and_then(|g| g.name.clone())
                            .unwrap_or_default(),
                        slug: p
                            .group
                            .as_ref()
                            .and_then(|g| g.slug.clone())
                            .unwrap_or_default(),
                        permission: p.permission.clone(),
                    })
                    .collect();

                println!("{}", Table::new(rows));

                if permissions.next.is_some() {
                    println!(
                        "\n{} More group permissions available. Use --limit to see more.",
                        "ℹ".blue()
                    );
                }

                Ok(())
            }

            PermissionCommands::GrantUser {
                repo,
                user,
                permission,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let granted = client
                    .grant_user_permission(&workspace, &repo_slug, &user, &permission)
                    .await?;

                if output_json() {
                    return print_json(&granted);
                }

                let label = granted
                    .user
                    .as_ref()
                    .map(|u| u.display_name.clone())
                    .unwrap_or(user);

                println!(
                    "{} Granted {} permission to {} on {}",
                    "✓".green(),
                    granted.permission.cyan(),
                    label.cyan(),
                    repo
                );

                Ok(())
            }

            PermissionCommands::RevokeUser { repo, user, yes } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                if !yes
                    && !confirm_or_abort(format!(
                        "Revoke {}'s explicit permission on {}?",
                        user.red(),
                        repo
                    ))?
                {
                    return Ok(());
                }

                let client = BitbucketClient::from_stored().await?;
                client
                    .revoke_user_permission(&workspace, &repo_slug, &user)
                    .await?;

                if output_json() {
                    return print_json(&serde_json::json!({ "ok": true }));
                }

                println!("{} Revoked permission for {}", "✓".green(), user);

                Ok(())
            }

            PermissionCommands::GrantGroup {
                repo,
                group,
                permission,
            } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;
                let client = BitbucketClient::from_stored().await?;

                let granted = client
                    .grant_group_permission(&workspace, &repo_slug, &group, &permission)
                    .await?;

                if output_json() {
                    return print_json(&granted);
                }

                let label = granted
                    .group
                    .as_ref()
                    .and_then(|g| g.name.clone().or_else(|| g.slug.clone()))
                    .unwrap_or(group);

                println!(
                    "{} Granted {} permission to group {} on {}",
                    "✓".green(),
                    granted.permission.cyan(),
                    label.cyan(),
                    repo
                );

                Ok(())
            }

            PermissionCommands::RevokeGroup { repo, group, yes } => {
                let (workspace, repo_slug) = parse_repo(&repo)?;

                if !yes
                    && !confirm_or_abort(format!(
                        "Revoke group {}'s explicit permission on {}?",
                        group.red(),
                        repo
                    ))?
                {
                    return Ok(());
                }

                let client = BitbucketClient::from_stored().await?;
                client
                    .revoke_group_permission(&workspace, &repo_slug, &group)
                    .await?;

                if output_json() {
                    return print_json(&serde_json::json!({ "ok": true }));
                }

                println!("{} Revoked permission for group {}", "✓".green(), group);

                Ok(())
            }
        }
    }
}
