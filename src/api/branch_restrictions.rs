use anyhow::Result;

use super::BitbucketClient;
use crate::models::{BranchRestriction, Paginated};

impl BitbucketClient {
    /// List branch restriction rules on a repository
    pub async fn list_branch_restrictions(
        &self,
        workspace: &str,
        repo_slug: &str,
        pagelen: Option<u32>,
    ) -> Result<Paginated<BranchRestriction>> {
        let path = format!(
            "/repositories/{}/{}/branch-restrictions",
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

    /// Create a branch restriction rule.
    ///
    /// `value` is the numeric parameter required by some kinds (e.g. the
    /// approval count for `require_approvals_to_merge`); it is omitted from
    /// the request body when `None`.
    pub async fn create_branch_restriction(
        &self,
        workspace: &str,
        repo_slug: &str,
        kind: &str,
        pattern: &str,
        value: Option<u64>,
    ) -> Result<BranchRestriction> {
        let mut body = serde_json::json!({
            "kind": kind,
            "pattern": pattern,
        });
        if let Some(value) = value {
            body["value"] = serde_json::json!(value);
        }

        let path = format!(
            "/repositories/{}/{}/branch-restrictions",
            workspace, repo_slug
        );
        self.post(&path, &body).await
    }

    /// Delete a branch restriction rule by id
    pub async fn delete_branch_restriction(
        &self,
        workspace: &str,
        repo_slug: &str,
        id: u64,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/branch-restrictions/{}",
            workspace, repo_slug, id
        );
        self.delete(&path).await
    }
}
