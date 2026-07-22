use anyhow::Result;

use super::BitbucketClient;
use super::webhooks::encode_path_segment;
use crate::models::{GroupPermission, Paginated, UserPermission};

impl BitbucketClient {
    /// List users with explicit permissions on a repository
    pub async fn list_user_permissions(
        &self,
        workspace: &str,
        repo_slug: &str,
        pagelen: Option<u32>,
    ) -> Result<Paginated<UserPermission>> {
        let path = format!(
            "/repositories/{}/{}/permissions-config/users",
            workspace, repo_slug
        );

        match pagelen {
            Some(len) => {
                self.get_with_query(&path, &[("pagelen", len.to_string().as_str())])
                    .await
            }
            None => self.get(&path).await,
        }
    }

    /// List groups with explicit permissions on a repository
    pub async fn list_group_permissions(
        &self,
        workspace: &str,
        repo_slug: &str,
        pagelen: Option<u32>,
    ) -> Result<Paginated<GroupPermission>> {
        let path = format!(
            "/repositories/{}/{}/permissions-config/groups",
            workspace, repo_slug
        );

        match pagelen {
            Some(len) => {
                self.get_with_query(&path, &[("pagelen", len.to_string().as_str())])
                    .await
            }
            None => self.get(&path).await,
        }
    }

    /// Grant or update a user's explicit permission on a repository.
    ///
    /// `selected_user` may be an account ID or a brace-wrapped UUID;
    /// `permission` is `read`, `write`, or `admin`.
    pub async fn grant_user_permission(
        &self,
        workspace: &str,
        repo_slug: &str,
        selected_user: &str,
        permission: &str,
    ) -> Result<UserPermission> {
        let path = format!(
            "/repositories/{}/{}/permissions-config/users/{}",
            workspace,
            repo_slug,
            encode_path_segment(selected_user)
        );
        self.put(&path, &serde_json::json!({ "permission": permission }))
            .await
    }

    /// Revoke a user's explicit permission on a repository
    pub async fn revoke_user_permission(
        &self,
        workspace: &str,
        repo_slug: &str,
        selected_user: &str,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/permissions-config/users/{}",
            workspace,
            repo_slug,
            encode_path_segment(selected_user)
        );
        self.delete(&path).await
    }

    /// Grant or update a group's explicit permission on a repository.
    ///
    /// `group_slug` is the workspace group's slug; `permission` is `read`,
    /// `write`, or `admin`.
    pub async fn grant_group_permission(
        &self,
        workspace: &str,
        repo_slug: &str,
        group_slug: &str,
        permission: &str,
    ) -> Result<GroupPermission> {
        let path = format!(
            "/repositories/{}/{}/permissions-config/groups/{}",
            workspace,
            repo_slug,
            encode_path_segment(group_slug)
        );
        self.put(&path, &serde_json::json!({ "permission": permission }))
            .await
    }

    /// Revoke a group's explicit permission on a repository
    pub async fn revoke_group_permission(
        &self,
        workspace: &str,
        repo_slug: &str,
        group_slug: &str,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/permissions-config/groups/{}",
            workspace,
            repo_slug,
            encode_path_segment(group_slug)
        );
        self.delete(&path).await
    }
}
