use anyhow::Result;

use super::BitbucketClient;
use crate::models::{CreateRepositoryRequest, Paginated, Repository, UpdateRepositoryRequest};

impl BitbucketClient {
    /// List repositories for a workspace
    pub async fn list_repositories(
        &self,
        workspace: &str,
        page: Option<u32>,
        pagelen: Option<u32>,
    ) -> Result<Paginated<Repository>> {
        self.list_repositories_filtered(workspace, None, None, None, page, pagelen)
            .await
    }

    /// List repositories for a workspace with optional filters.
    ///
    /// `role` restricts results to repositories where the caller has at least
    /// that role (member, contributor, admin, or owner), `query` is a
    /// Bitbucket query (BBQL) expression passed as the `q` parameter, and
    /// `sort` names a field to sort by (prefix with `-` for descending).
    pub async fn list_repositories_filtered(
        &self,
        workspace: &str,
        role: Option<&str>,
        query: Option<&str>,
        sort: Option<&str>,
        page: Option<u32>,
        pagelen: Option<u32>,
    ) -> Result<Paginated<Repository>> {
        let mut params = Vec::new();

        if let Some(role) = role {
            params.push(("role", role.to_string()));
        }
        if let Some(q) = query {
            params.push(("q", q.to_string()));
        }
        if let Some(sort) = sort {
            params.push(("sort", sort.to_string()));
        }
        if let Some(p) = page {
            params.push(("page", p.to_string()));
        }
        if let Some(len) = pagelen {
            params.push(("pagelen", len.to_string()));
        }

        let query_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let path = format!("/repositories/{}", workspace);
        self.get_with_query(&path, &query_refs).await
    }

    /// Get a specific repository
    pub async fn get_repository(&self, workspace: &str, repo_slug: &str) -> Result<Repository> {
        let path = format!("/repositories/{}/{}", workspace, repo_slug);
        self.get(&path).await
    }

    /// Create a new repository
    pub async fn create_repository(
        &self,
        workspace: &str,
        repo_slug: &str,
        request: &CreateRepositoryRequest,
    ) -> Result<Repository> {
        let path = format!("/repositories/{}/{}", workspace, repo_slug);
        self.put(&path, request).await
    }

    /// Update settings on an existing repository
    pub async fn update_repository(
        &self,
        workspace: &str,
        repo_slug: &str,
        request: &UpdateRepositoryRequest,
    ) -> Result<Repository> {
        let path = format!("/repositories/{}/{}", workspace, repo_slug);
        self.put(&path, request).await
    }

    /// Delete a repository
    pub async fn delete_repository(&self, workspace: &str, repo_slug: &str) -> Result<()> {
        let path = format!("/repositories/{}/{}", workspace, repo_slug);
        self.delete(&path).await
    }

    /// Fork a repository
    pub async fn fork_repository(
        &self,
        workspace: &str,
        repo_slug: &str,
        new_workspace: Option<&str>,
        new_name: Option<&str>,
    ) -> Result<Repository> {
        #[derive(serde::Serialize)]
        struct ForkRequest {
            #[serde(skip_serializing_if = "Option::is_none")]
            workspace: Option<WorkspaceRef>,
            #[serde(skip_serializing_if = "Option::is_none")]
            name: Option<String>,
        }

        #[derive(serde::Serialize)]
        struct WorkspaceRef {
            slug: String,
        }

        let request = ForkRequest {
            workspace: new_workspace.map(|w| WorkspaceRef {
                slug: w.to_string(),
            }),
            name: new_name.map(|n| n.to_string()),
        };

        let path = format!("/repositories/{}/{}/forks", workspace, repo_slug);
        self.post(&path, &request).await
    }

    /// List users watching a repository
    pub async fn list_watchers(
        &self,
        workspace: &str,
        repo_slug: &str,
        pagelen: Option<u32>,
    ) -> Result<Paginated<crate::models::User>> {
        let path = format!("/repositories/{}/{}/watchers", workspace, repo_slug);
        match pagelen {
            Some(len) => {
                let len = len.to_string();
                self.get_with_query(&path, &[("pagelen", len.as_str())])
                    .await
            }
            None => self.get(&path).await,
        }
    }

    /// List forks of a repository
    pub async fn list_forks(
        &self,
        workspace: &str,
        repo_slug: &str,
        pagelen: Option<u32>,
    ) -> Result<Paginated<Repository>> {
        let path = format!("/repositories/{}/{}/forks", workspace, repo_slug);
        match pagelen {
            Some(len) => {
                let len = len.to_string();
                self.get_with_query(&path, &[("pagelen", len.as_str())])
                    .await
            }
            None => self.get(&path).await,
        }
    }
}
