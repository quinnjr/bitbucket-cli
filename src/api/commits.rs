use anyhow::Result;

use super::BitbucketClient;
use crate::api::util::urlencode_segment;
use crate::models::{CommitSummary, Paginated};

impl BitbucketClient {
    /// List commits in a repository, newest first.
    ///
    /// With `revision` set, lists the history reachable from that branch,
    /// tag, or commit hash; otherwise lists from the default branch.
    pub async fn list_commits(
        &self,
        workspace: &str,
        repo_slug: &str,
        revision: Option<&str>,
        page: Option<u32>,
        pagelen: Option<u32>,
    ) -> Result<Paginated<CommitSummary>> {
        let path = match revision {
            Some(rev) => format!(
                "/repositories/{}/{}/commits/{}",
                workspace,
                repo_slug,
                urlencode_segment(rev)
            ),
            None => format!("/repositories/{}/{}/commits", workspace, repo_slug),
        };

        let mut params = Vec::new();
        if let Some(p) = page {
            params.push(("page", p.to_string()));
        }
        if let Some(len) = pagelen {
            params.push(("pagelen", len.to_string()));
        }

        let param_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.get_with_query(&path, &param_refs).await
    }

    /// Get a single commit by hash.
    pub async fn get_commit(
        &self,
        workspace: &str,
        repo_slug: &str,
        hash: &str,
    ) -> Result<CommitSummary> {
        let path = format!(
            "/repositories/{}/{}/commit/{}",
            workspace,
            repo_slug,
            urlencode_segment(hash)
        );
        self.get(&path).await
    }

    /// Get a raw unified diff for a commit or revision spec.
    ///
    /// `spec` is either a single revision (diff against its first parent) or
    /// a `source..destination` pair of revisions.
    pub async fn get_diff(&self, workspace: &str, repo_slug: &str, spec: &str) -> Result<String> {
        let path = format!(
            "/repositories/{}/{}/diff/{}",
            workspace,
            repo_slug,
            urlencode_segment(spec)
        );
        self.get_text(&path, None).await
    }
}
