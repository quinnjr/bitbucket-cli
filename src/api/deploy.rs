use anyhow::Result;

use super::BitbucketClient;
use crate::models::{
    AddDeployKeyRequest, CreateEnvironmentRequest, DeployKey, Deployment, Environment, Paginated,
};

impl BitbucketClient {
    // -----------------------------------------------------------------------
    // Deploy keys
    // -----------------------------------------------------------------------

    /// List deploy keys for a repository
    pub async fn list_deploy_keys(
        &self,
        workspace: &str,
        repo_slug: &str,
    ) -> Result<Paginated<DeployKey>> {
        let path = format!("/repositories/{}/{}/deploy-keys", workspace, repo_slug);
        self.get(&path).await
    }

    /// Add a deploy key to a repository
    pub async fn add_deploy_key(
        &self,
        workspace: &str,
        repo_slug: &str,
        request: &AddDeployKeyRequest,
    ) -> Result<DeployKey> {
        let path = format!("/repositories/{}/{}/deploy-keys", workspace, repo_slug);
        self.post(&path, request).await
    }

    /// Delete a deploy key from a repository
    pub async fn delete_deploy_key(
        &self,
        workspace: &str,
        repo_slug: &str,
        key_id: u64,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/deploy-keys/{}",
            workspace, repo_slug, key_id
        );
        self.delete(&path).await
    }

    // -----------------------------------------------------------------------
    // Deployment environments
    // -----------------------------------------------------------------------

    /// List deployment environments for a repository
    pub async fn list_environments(
        &self,
        workspace: &str,
        repo_slug: &str,
    ) -> Result<Paginated<Environment>> {
        let path = format!("/repositories/{}/{}/environments/", workspace, repo_slug);
        self.get(&path).await
    }

    /// Create a deployment environment
    pub async fn create_environment(
        &self,
        workspace: &str,
        repo_slug: &str,
        request: &CreateEnvironmentRequest,
    ) -> Result<Environment> {
        let path = format!("/repositories/{}/{}/environments/", workspace, repo_slug);
        self.post(&path, request).await
    }

    /// Delete a deployment environment
    pub async fn delete_environment(
        &self,
        workspace: &str,
        repo_slug: &str,
        environment_uuid: &str,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/environments/{}",
            workspace, repo_slug, environment_uuid
        );
        self.delete(&path).await
    }

    // -----------------------------------------------------------------------
    // Deployments
    // -----------------------------------------------------------------------

    /// List deployments for a repository
    pub async fn list_deployments(
        &self,
        workspace: &str,
        repo_slug: &str,
        pagelen: Option<u32>,
    ) -> Result<Paginated<Deployment>> {
        let mut query = Vec::new();

        if let Some(len) = pagelen {
            query.push(("pagelen", len.to_string()));
        }

        let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let path = format!("/repositories/{}/{}/deployments/", workspace, repo_slug);
        self.get_with_query(&path, &query_refs).await
    }

    /// Get a specific deployment
    pub async fn get_deployment(
        &self,
        workspace: &str,
        repo_slug: &str,
        deployment_uuid: &str,
    ) -> Result<Deployment> {
        let path = format!(
            "/repositories/{}/{}/deployments/{}",
            workspace, repo_slug, deployment_uuid
        );
        self.get(&path).await
    }
}
