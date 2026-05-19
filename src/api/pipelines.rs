use anyhow::Result;

use super::BitbucketClient;
use crate::models::{Paginated, Pipeline, PipelineStep, TriggerPipelineRequest};

impl BitbucketClient {
    /// List pipelines for a repository
    pub async fn list_pipelines(
        &self,
        workspace: &str,
        repo_slug: &str,
        page: Option<u32>,
        pagelen: Option<u32>,
    ) -> Result<Paginated<Pipeline>> {
        let mut query = Vec::new();

        // Sort by created_on descending to get most recent first
        query.push(("sort", "-created_on".to_string()));

        if let Some(p) = page {
            query.push(("page", p.to_string()));
        }
        if let Some(len) = pagelen {
            query.push(("pagelen", len.to_string()));
        }

        let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let path = format!("/repositories/{}/{}/pipelines", workspace, repo_slug);
        self.get_with_query(&path, &query_refs).await
    }

    /// Get a specific pipeline
    pub async fn get_pipeline(
        &self,
        workspace: &str,
        repo_slug: &str,
        pipeline_uuid: &str,
    ) -> Result<Pipeline> {
        let path = format!(
            "/repositories/{}/{}/pipelines/{}",
            workspace, repo_slug, pipeline_uuid
        );
        self.get(&path).await
    }

    /// Trigger a new pipeline
    pub async fn trigger_pipeline(
        &self,
        workspace: &str,
        repo_slug: &str,
        request: &TriggerPipelineRequest,
    ) -> Result<Pipeline> {
        let path = format!("/repositories/{}/{}/pipelines", workspace, repo_slug);
        self.post(&path, request).await
    }

    /// Stop a running pipeline
    pub async fn stop_pipeline(
        &self,
        workspace: &str,
        repo_slug: &str,
        pipeline_uuid: &str,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/pipelines/{}/stopPipeline",
            workspace, repo_slug, pipeline_uuid
        );
        self.post_no_response(&path, &serde_json::json!({})).await
    }

    /// List steps for a pipeline
    pub async fn list_pipeline_steps(
        &self,
        workspace: &str,
        repo_slug: &str,
        pipeline_uuid: &str,
    ) -> Result<Paginated<PipelineStep>> {
        let path = format!(
            "/repositories/{}/{}/pipelines/{}/steps",
            workspace, repo_slug, pipeline_uuid
        );
        self.get(&path).await
    }

    /// Get a specific pipeline step
    pub async fn get_pipeline_step(
        &self,
        workspace: &str,
        repo_slug: &str,
        pipeline_uuid: &str,
        step_uuid: &str,
    ) -> Result<PipelineStep> {
        let path = format!(
            "/repositories/{}/{}/pipelines/{}/steps/{}",
            workspace, repo_slug, pipeline_uuid, step_uuid
        );
        self.get(&path).await
    }

    /// Get pipeline step log
    pub async fn get_step_log(
        &self,
        workspace: &str,
        repo_slug: &str,
        pipeline_uuid: &str,
        step_uuid: &str,
    ) -> Result<String> {
        let path = format!(
            "/repositories/{}/{}/pipelines/{}/steps/{}/log",
            workspace, repo_slug, pipeline_uuid, step_uuid
        );

        let response = reqwest::Client::new()
            .get(self.url(&path))
            .header("Authorization", self.auth_header())
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.text().await?)
        } else {
            anyhow::bail!("Failed to get step log: {}", response.status())
        }
    }

    /// List pipelines whose target commit matches `commit_hash`, newest first.
    ///
    /// Bitbucket's pipelines endpoint does not expose a server-side filter on
    /// `target.commit.hash` (both the `?target.commit.hash=` query param and
    /// the BBQL `q=` filter are silently ignored as of the 2.0 API), so this
    /// fetches the most recent pipelines and filters client-side. `scan_limit`
    /// bounds how many of the repo's recent pipelines are scanned and is
    /// capped at Bitbucket's pagelen ceiling of 100; on repos that churn more
    /// than 100 pipelines between the commit landing and this call, matches
    /// older than that window will be missed.
    pub async fn list_pipelines_for_commit(
        &self,
        workspace: &str,
        repo_slug: &str,
        commit_hash: &str,
        scan_limit: u32,
    ) -> Result<Vec<Pipeline>> {
        let pagelen = scan_limit.clamp(1, 100);
        let pipelines = self
            .list_pipelines(workspace, repo_slug, None, Some(pagelen))
            .await?;
        Ok(pipelines
            .values
            .into_iter()
            .filter(|p| {
                p.target
                    .commit
                    .as_ref()
                    .is_some_and(|c| commit_hashes_match(&c.hash, commit_hash))
            })
            .collect())
    }

    /// Get pipeline by build number
    pub async fn get_pipeline_by_build_number(
        &self,
        workspace: &str,
        repo_slug: &str,
        build_number: u64,
    ) -> Result<Pipeline> {
        // Search for the pipeline with the given build number
        let pipelines = self
            .list_pipelines(workspace, repo_slug, Some(1), Some(100))
            .await?;

        pipelines
            .values
            .into_iter()
            .find(|p| p.build_number == build_number)
            .ok_or_else(|| anyhow::anyhow!("Pipeline #{} not found", build_number))
    }
}

/// Compare two git commit hashes that may differ in length.
///
/// Bitbucket's pull-request API returns a 12-char short hash while the
/// pipelines API returns the full 40-char hash. Match by treating the
/// shorter of the two as a prefix of the longer.
fn commit_hashes_match(a: &str, b: &str) -> bool {
    let (long, short) = if a.len() >= b.len() { (a, b) } else { (b, a) };
    !short.is_empty() && long.starts_with(short)
}

#[cfg(test)]
mod tests {
    use super::commit_hashes_match;

    #[test]
    fn exact_equality_matches() {
        assert!(commit_hashes_match("abc123", "abc123"));
    }

    #[test]
    fn short_pr_hash_matches_full_pipeline_hash() {
        assert!(commit_hashes_match(
            "975f24a99dab12345abc678def910ghi112233jk",
            "975f24a99dab"
        ));
    }

    #[test]
    fn full_pr_hash_matches_short_pipeline_hash() {
        assert!(commit_hashes_match(
            "975f24a99dab",
            "975f24a99dab12345abc678def910ghi112233jk"
        ));
    }

    #[test]
    fn different_commits_do_not_match() {
        assert!(!commit_hashes_match("abc123def", "abc999def"));
    }

    #[test]
    fn empty_hash_never_matches() {
        // Guard against false positives if the API returns a missing hash
        // that somehow deserializes to an empty string.
        assert!(!commit_hashes_match("", "abc123"));
        assert!(!commit_hashes_match("abc123", ""));
        assert!(!commit_hashes_match("", ""));
    }
}
