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
        let mut query = Vec::new();

        if let Some(p) = page {
            query.push(("page", p.to_string()));
        }
        if let Some(len) = pagelen {
            query.push(("pagelen", len.to_string()));
        }

        let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();

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

    /// List repository branches
    pub async fn list_branches(
        &self,
        workspace: &str,
        repo_slug: &str,
    ) -> Result<Paginated<crate::models::Branch>> {
        let path = format!("/repositories/{}/{}/refs/branches", workspace, repo_slug);
        self.get(&path).await
    }

    /// Get the main branch
    pub async fn get_main_branch(
        &self,
        workspace: &str,
        repo_slug: &str,
    ) -> Result<crate::models::Branch> {
        let path = format!("/repositories/{}/{}/main-branch", workspace, repo_slug);
        self.get(&path).await
    }
}
