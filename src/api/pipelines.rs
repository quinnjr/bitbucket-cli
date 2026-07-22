use anyhow::Result;

use super::BitbucketClient;
use crate::models::{
    Paginated, Pipeline, PipelineCache, PipelineSchedule, PipelineStep, PipelineVariable,
    PipelineVariableInput, PipelinesConfig, TriggerPipelineRequest,
};

/// Server-side filters for [`BitbucketClient::list_pipelines_filtered`].
#[derive(Default)]
pub(crate) struct PipelineListFilters<'a> {
    pub page: Option<u32>,
    pub pagelen: Option<u32>,
    pub status: Option<&'a str>,
    pub target_branch: Option<&'a str>,
    pub sort: Option<&'a str>,
}

impl BitbucketClient {
    /// List pipelines for a repository, sorted by most recent first.
    ///
    /// Thin wrapper over [`Self::list_pipelines_filtered`] kept for the TUI
    /// and the build-number/commit lookups, which need no server-side filters.
    pub async fn list_pipelines(
        &self,
        workspace: &str,
        repo_slug: &str,
        page: Option<u32>,
        pagelen: Option<u32>,
    ) -> Result<Paginated<Pipeline>> {
        self.list_pipelines_filtered(
            workspace,
            repo_slug,
            PipelineListFilters {
                page,
                pagelen,
                ..Default::default()
            },
        )
        .await
    }

    /// List pipelines with optional server-side filters.
    ///
    /// `status` filters on pipeline state (e.g. `PENDING`, `IN_PROGRESS`,
    /// `COMPLETED`), `target_branch` on the target branch name, and `sort`
    /// overrides the default `-created_on` (most recent first) ordering.
    pub(crate) async fn list_pipelines_filtered(
        &self,
        workspace: &str,
        repo_slug: &str,
        filters: PipelineListFilters<'_>,
    ) -> Result<Paginated<Pipeline>> {
        let query = pipeline_list_params(&filters);
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

        self.get_text(&path, None).await
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
        // Search for the pipeline with the given build number, newest first,
        // scanning at most 50 pages of 100 pipelines each.
        find_pipeline_by_build_number(
            |page| self.list_pipelines(workspace, repo_slug, Some(page), Some(100)),
            build_number,
            50,
        )
        .await
    }

    // ---- Repository pipeline variables -----------------------------------

    /// List all pipeline variables configured on a repository.
    pub async fn list_pipeline_variables(
        &self,
        workspace: &str,
        repo_slug: &str,
    ) -> Result<Vec<PipelineVariable>> {
        let path = format!(
            "/repositories/{}/{}/pipelines_config/variables/",
            workspace, repo_slug
        );
        self.get_all_pages(&path).await
    }

    /// Create a new pipeline variable on a repository.
    pub async fn create_pipeline_variable(
        &self,
        workspace: &str,
        repo_slug: &str,
        variable: &PipelineVariableInput,
    ) -> Result<PipelineVariable> {
        let path = format!(
            "/repositories/{}/{}/pipelines_config/variables/",
            workspace, repo_slug
        );
        self.post(&path, variable).await
    }

    /// Update an existing pipeline variable on a repository.
    pub async fn update_pipeline_variable(
        &self,
        workspace: &str,
        repo_slug: &str,
        variable_uuid: &str,
        variable: &PipelineVariableInput,
    ) -> Result<PipelineVariable> {
        let path = format!(
            "/repositories/{}/{}/pipelines_config/variables/{}",
            workspace, repo_slug, variable_uuid
        );
        self.put(&path, variable).await
    }

    /// Delete a pipeline variable from a repository.
    pub async fn delete_pipeline_variable(
        &self,
        workspace: &str,
        repo_slug: &str,
        variable_uuid: &str,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/pipelines_config/variables/{}",
            workspace, repo_slug, variable_uuid
        );
        self.delete(&path).await
    }

    // ---- Workspace pipeline variables -------------------------------------

    /// List all workspace-level pipeline variables.
    ///
    /// Note the workspace endpoint uses `pipelines-config` (hyphen), unlike
    /// the repository endpoint's `pipelines_config`.
    pub async fn list_workspace_pipeline_variables(
        &self,
        workspace: &str,
    ) -> Result<Vec<PipelineVariable>> {
        let path = format!("/workspaces/{}/pipelines-config/variables/", workspace);
        self.get_all_pages(&path).await
    }

    /// Create a new workspace-level pipeline variable.
    pub async fn create_workspace_pipeline_variable(
        &self,
        workspace: &str,
        variable: &PipelineVariableInput,
    ) -> Result<PipelineVariable> {
        let path = format!("/workspaces/{}/pipelines-config/variables/", workspace);
        self.post(&path, variable).await
    }

    /// Update an existing workspace-level pipeline variable.
    pub async fn update_workspace_pipeline_variable(
        &self,
        workspace: &str,
        variable_uuid: &str,
        variable: &PipelineVariableInput,
    ) -> Result<PipelineVariable> {
        let path = format!(
            "/workspaces/{}/pipelines-config/variables/{}",
            workspace, variable_uuid
        );
        self.put(&path, variable).await
    }

    /// Delete a workspace-level pipeline variable.
    pub async fn delete_workspace_pipeline_variable(
        &self,
        workspace: &str,
        variable_uuid: &str,
    ) -> Result<()> {
        let path = format!(
            "/workspaces/{}/pipelines-config/variables/{}",
            workspace, variable_uuid
        );
        self.delete(&path).await
    }

    // ---- Pipeline schedules ------------------------------------------------

    /// List all pipeline schedules configured on a repository.
    pub async fn list_pipeline_schedules(
        &self,
        workspace: &str,
        repo_slug: &str,
    ) -> Result<Vec<PipelineSchedule>> {
        let path = format!(
            "/repositories/{}/{}/pipelines_config/schedules/",
            workspace, repo_slug
        );
        self.get_all_pages(&path).await
    }

    /// Create a pipeline schedule running `branch` on `cron_pattern`.
    pub async fn create_pipeline_schedule(
        &self,
        workspace: &str,
        repo_slug: &str,
        branch: &str,
        cron_pattern: &str,
    ) -> Result<PipelineSchedule> {
        let path = format!(
            "/repositories/{}/{}/pipelines_config/schedules/",
            workspace, repo_slug
        );
        let body = serde_json::json!({
            "enabled": true,
            "target": {
                "type": "pipeline_ref_target",
                "ref_type": "branch",
                "ref_name": branch,
                "selector": {
                    "type": "branches",
                    "pattern": branch,
                },
            },
            "cron_pattern": cron_pattern,
        });
        self.post(&path, &body).await
    }

    /// Delete a pipeline schedule from a repository.
    pub async fn delete_pipeline_schedule(
        &self,
        workspace: &str,
        repo_slug: &str,
        schedule_uuid: &str,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/pipelines_config/schedules/{}",
            workspace, repo_slug, schedule_uuid
        );
        self.delete(&path).await
    }

    // ---- Pipeline caches ----------------------------------------------------

    /// List dependency caches stored for a repository's pipelines.
    ///
    /// Note the caches endpoint uses `pipelines-config` (hyphen), unlike the
    /// variables/schedules endpoints' `pipelines_config`.
    pub async fn list_pipeline_caches(
        &self,
        workspace: &str,
        repo_slug: &str,
    ) -> Result<Vec<PipelineCache>> {
        let path = format!(
            "/repositories/{}/{}/pipelines-config/caches/",
            workspace, repo_slug
        );
        self.get_all_pages(&path).await
    }

    /// Delete a pipeline dependency cache by UUID.
    pub async fn delete_pipeline_cache(
        &self,
        workspace: &str,
        repo_slug: &str,
        cache_uuid: &str,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/pipelines-config/caches/{}",
            workspace, repo_slug, cache_uuid
        );
        self.delete(&path).await
    }

    // ---- Pipelines configuration ---------------------------------------------

    /// Get the repository's pipelines configuration (enabled state).
    pub async fn get_pipelines_config(
        &self,
        workspace: &str,
        repo_slug: &str,
    ) -> Result<PipelinesConfig> {
        let path = format!("/repositories/{}/{}/pipelines_config", workspace, repo_slug);
        self.get(&path).await
    }

    /// Enable or disable pipelines on a repository.
    pub async fn update_pipelines_config_enabled(
        &self,
        workspace: &str,
        repo_slug: &str,
        enabled: bool,
    ) -> Result<PipelinesConfig> {
        let path = format!("/repositories/{}/{}/pipelines_config", workspace, repo_slug);
        self.put(&path, &serde_json::json!({ "enabled": enabled }))
            .await
    }

    /// Set the next build number for a repository's pipelines.
    pub async fn set_pipelines_config_build_number(
        &self,
        workspace: &str,
        repo_slug: &str,
        next: u64,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/pipelines_config/build_number",
            workspace, repo_slug
        );
        let _: serde_json::Value = self
            .put(&path, &serde_json::json!({ "next": next }))
            .await?;
        Ok(())
    }
}

/// Assemble the query parameters for a filtered pipeline listing.
///
/// `sort` defaults to `-created_on` (most recent first) unless overridden;
/// `status` is upper-cased to match Bitbucket's enum, and the target-branch
/// filter is sent under the `target.branch` key the API expects.
fn pipeline_list_params(filters: &PipelineListFilters<'_>) -> Vec<(&'static str, String)> {
    let mut query = Vec::new();

    query.push(("sort", filters.sort.unwrap_or("-created_on").to_string()));

    if let Some(status) = filters.status {
        query.push(("status", status.to_uppercase()));
    }
    if let Some(branch) = filters.target_branch {
        query.push(("target.branch", branch.to_string()));
    }
    if let Some(p) = filters.page {
        query.push(("page", p.to_string()));
    }
    if let Some(len) = filters.pagelen {
        query.push(("pagelen", len.to_string()));
    }

    query
}

/// Walk pages of pipelines, newest first, looking for a matching build number.
///
/// Fetches pages `1..=max_pages` via `fetch_page` and returns as soon as a
/// pipeline with `build_number` is found. Stops early when a page has no
/// `next` link; errors if the page cap is reached without a match so a bogus
/// build number can't trigger an unbounded page walk.
async fn find_pipeline_by_build_number<F, Fut>(
    mut fetch_page: F,
    build_number: u64,
    max_pages: u32,
) -> Result<Pipeline>
where
    F: FnMut(u32) -> Fut,
    Fut: std::future::Future<Output = Result<Paginated<Pipeline>>>,
{
    let mut pipelines_seen: usize = 0;

    for page in 1..=max_pages {
        let pipelines = fetch_page(page).await?;
        let has_next = pipelines.next.is_some();
        pipelines_seen += pipelines.values.len();

        if let Some(pipeline) = pipelines
            .values
            .into_iter()
            .find(|p| p.build_number == build_number)
        {
            return Ok(pipeline);
        }

        if !has_next {
            return Err(anyhow::anyhow!("Pipeline #{} not found", build_number));
        }
    }

    Err(anyhow::anyhow!(
        "Pipeline #{} not found (searched the {} most recent pipelines)",
        build_number,
        pipelines_seen
    ))
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
    use std::cell::Cell;
    use std::future::{Ready, ready};

    use chrono::Utc;

    use super::{
        PipelineListFilters, commit_hashes_match, find_pipeline_by_build_number,
        pipeline_list_params,
    };
    use crate::models::{Paginated, Pipeline, PipelineState, PipelineStateName, PipelineTarget};

    fn param<'a>(params: &'a [(&'static str, String)], key: &str) -> Option<&'a str> {
        params
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| v.as_str())
    }

    #[test]
    fn params_default_sort_is_created_on_descending() {
        let params = pipeline_list_params(&PipelineListFilters::default());
        assert_eq!(param(&params, "sort"), Some("-created_on"));
    }

    #[test]
    fn params_sort_override_wins() {
        let filters = PipelineListFilters {
            sort: Some("created_on"),
            ..Default::default()
        };
        assert_eq!(
            param(&pipeline_list_params(&filters), "sort"),
            Some("created_on")
        );
    }

    #[test]
    fn params_status_is_uppercased() {
        let filters = PipelineListFilters {
            status: Some("in_progress"),
            ..Default::default()
        };
        assert_eq!(
            param(&pipeline_list_params(&filters), "status"),
            Some("IN_PROGRESS")
        );
    }

    #[test]
    fn params_target_branch_uses_dotted_key() {
        let filters = PipelineListFilters {
            target_branch: Some("main"),
            ..Default::default()
        };
        let params = pipeline_list_params(&filters);
        assert_eq!(param(&params, "target.branch"), Some("main"));
        // Absent when no branch filter is set.
        assert_eq!(
            param(
                &pipeline_list_params(&PipelineListFilters::default()),
                "target.branch"
            ),
            None
        );
    }

    fn pipeline(build_number: u64) -> Pipeline {
        Pipeline {
            uuid: format!("{{pipeline-{}}}", build_number),
            build_number,
            creator: None,
            repository: None,
            target: PipelineTarget {
                target_type: "pipeline_ref_target".to_string(),
                ref_type: None,
                ref_name: None,
                selector: None,
                commit: None,
            },
            trigger: None,
            state: PipelineState {
                name: PipelineStateName::Completed,
                state_type: "pipeline_state_completed".to_string(),
                result: None,
                stage: None,
            },
            created_on: Utc::now(),
            completed_on: None,
            build_seconds_used: None,
            links: None,
        }
    }

    fn page(values: Vec<Pipeline>, has_next: bool) -> Paginated<Pipeline> {
        Paginated {
            size: None,
            page: None,
            pagelen: None,
            next: has_next.then(|| "https://api.bitbucket.org/next".to_string()),
            previous: None,
            values,
        }
    }

    type PageFuture = Ready<anyhow::Result<Paginated<Pipeline>>>;

    #[tokio::test]
    async fn finds_pipeline_on_second_page_with_exactly_two_fetches() {
        let calls = Cell::new(0u32);
        let fetch = |page_number: u32| -> PageFuture {
            calls.set(calls.get() + 1);
            ready(Ok(match page_number {
                1 => page(vec![pipeline(100), pipeline(99)], true),
                2 => page(vec![pipeline(98), pipeline(42)], true),
                other => panic!("unexpected page fetch: {}", other),
            }))
        };

        let found = find_pipeline_by_build_number(fetch, 42, 50)
            .await
            .expect("pipeline should be found on page 2");

        assert_eq!(found.build_number, 42);
        assert_eq!(calls.get(), 2);
    }

    #[tokio::test]
    async fn stops_after_one_fetch_when_pages_are_exhausted() {
        let calls = Cell::new(0u32);
        let fetch = |_page_number: u32| -> PageFuture {
            calls.set(calls.get() + 1);
            ready(Ok(page(vec![pipeline(100), pipeline(99)], false)))
        };

        let err = find_pipeline_by_build_number(fetch, 42, 50)
            .await
            .expect_err("pipeline should not be found");

        assert_eq!(err.to_string(), "Pipeline #42 not found");
        assert_eq!(calls.get(), 1);
    }

    #[tokio::test]
    async fn enforces_page_cap_when_next_is_always_present() {
        let calls = Cell::new(0u32);
        let fetch = |_page_number: u32| -> PageFuture {
            calls.set(calls.get() + 1);
            ready(Ok(page(vec![pipeline(100), pipeline(99)], true)))
        };

        let err = find_pipeline_by_build_number(fetch, 42, 3)
            .await
            .expect_err("search should stop at the page cap");

        assert_eq!(
            err.to_string(),
            "Pipeline #42 not found (searched the 6 most recent pipelines)"
        );
        assert_eq!(calls.get(), 3);
    }

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
