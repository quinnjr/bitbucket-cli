use anyhow::Result;

use super::BitbucketClient;
use super::webhooks::encode_path_segment;
use crate::models::{Paginated, User};

impl BitbucketClient {
    /// List a repository's default reviewers
    pub async fn list_default_reviewers(
        &self,
        workspace: &str,
        repo_slug: &str,
        pagelen: Option<u32>,
    ) -> Result<Paginated<User>> {
        let path = format!(
            "/repositories/{}/{}/default-reviewers",
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

    /// Add a user to a repository's default reviewers.
    ///
    /// `username` may be a username, an account ID, or a brace-wrapped UUID.
    /// The endpoint is a bodyless PUT; an empty JSON object is sent because
    /// the client's PUT helper always attaches a JSON body.
    pub async fn add_default_reviewer(
        &self,
        workspace: &str,
        repo_slug: &str,
        username: &str,
    ) -> Result<User> {
        let path = format!(
            "/repositories/{}/{}/default-reviewers/{}",
            workspace,
            repo_slug,
            encode_path_segment(username)
        );
        self.put(&path, &serde_json::json!({})).await
    }

    /// Remove a user from a repository's default reviewers
    pub async fn remove_default_reviewer(
        &self,
        workspace: &str,
        repo_slug: &str,
        username: &str,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/default-reviewers/{}",
            workspace,
            repo_slug,
            encode_path_segment(username)
        );
        self.delete(&path).await
    }
}
