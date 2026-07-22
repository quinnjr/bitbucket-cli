use anyhow::Result;

use super::BitbucketClient;
use crate::models::{BranchingModel, UpdateBranchingModelRequest};

impl BitbucketClient {
    /// Get a repository's effective branching model
    pub async fn get_branching_model(
        &self,
        workspace: &str,
        repo_slug: &str,
    ) -> Result<BranchingModel> {
        let path = format!("/repositories/{}/{}/branching-model", workspace, repo_slug);
        self.get(&path).await
    }

    /// Update a repository's branching model settings
    pub async fn update_branching_model_settings(
        &self,
        workspace: &str,
        repo_slug: &str,
        request: &UpdateBranchingModelRequest,
    ) -> Result<BranchingModel> {
        let path = format!(
            "/repositories/{}/{}/branching-model/settings",
            workspace, repo_slug
        );
        self.put(&path, request).await
    }
}
