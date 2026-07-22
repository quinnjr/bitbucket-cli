use anyhow::Result;

use super::BitbucketClient;
use crate::models::{
    CommentRef, CommitStatus, CreatePullRequestRequest, DiffStat, InlineComment,
    MergePullRequestRequest, Paginated, PrActivity, PrCommit, PrTask, PullRequest,
    PullRequestComment, PullRequestState, UpdatePullRequestRequest,
};

/// Server-side filters for [`BitbucketClient::list_pull_requests_filtered`].
#[derive(Default)]
pub(crate) struct PrListFilters<'a> {
    pub states: &'a [PullRequestState],
    pub query: Option<&'a str>,
    pub sort: Option<&'a str>,
    pub page: Option<u32>,
    pub pagelen: Option<u32>,
}

/// Input for [`BitbucketClient::add_pr_comment_full`]: the comment body plus
/// optional inline placement and reply target.
pub(crate) struct PrCommentInput<'a> {
    pub content: &'a str,
    pub path: Option<&'a str>,
    pub line: Option<u32>,
    pub parent: Option<u64>,
}

impl BitbucketClient {
    /// List pull requests for a repository, filtered by at most one state.
    ///
    /// Thin wrapper over [`Self::list_pull_requests_filtered`] kept for
    /// callers (e.g. the TUI) that only need the simple form.
    pub async fn list_pull_requests(
        &self,
        workspace: &str,
        repo_slug: &str,
        state: Option<PullRequestState>,
        page: Option<u32>,
        pagelen: Option<u32>,
    ) -> Result<Paginated<PullRequest>> {
        let states: Vec<PullRequestState> = state.into_iter().collect();
        self.list_pull_requests_filtered(
            workspace,
            repo_slug,
            PrListFilters {
                states: &states,
                page,
                pagelen,
                ..Default::default()
            },
        )
        .await
    }

    /// List pull requests with full filtering: multiple `state` values, a
    /// BBQL `q` expression, a `sort` field, and explicit paging.
    pub(crate) async fn list_pull_requests_filtered(
        &self,
        workspace: &str,
        repo_slug: &str,
        filters: PrListFilters<'_>,
    ) -> Result<Paginated<PullRequest>> {
        let mut params = Vec::new();

        for state in filters.states {
            params.push(("state", state.to_string()));
        }
        if let Some(q) = filters.query {
            params.push(("q", q.to_string()));
        }
        if let Some(s) = filters.sort {
            params.push(("sort", s.to_string()));
        }
        if let Some(p) = filters.page {
            params.push(("page", p.to_string()));
        }
        if let Some(len) = filters.pagelen {
            params.push(("pagelen", len.to_string()));
        }

        let query_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let path = format!("/repositories/{}/{}/pullrequests", workspace, repo_slug);
        self.get_with_query(&path, &query_refs).await
    }

    /// Get a specific pull request
    pub async fn get_pull_request(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
    ) -> Result<PullRequest> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}",
            workspace, repo_slug, pr_id
        );
        self.get(&path).await
    }

    /// Create a new pull request
    pub async fn create_pull_request(
        &self,
        workspace: &str,
        repo_slug: &str,
        request: &CreatePullRequestRequest,
    ) -> Result<PullRequest> {
        let path = format!("/repositories/{}/{}/pullrequests", workspace, repo_slug);
        self.post(&path, request).await
    }

    /// Update a pull request; only the fields set on `request` are changed.
    pub async fn update_pull_request(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
        request: &UpdatePullRequestRequest,
    ) -> Result<PullRequest> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}",
            workspace, repo_slug, pr_id
        );
        self.put(&path, request).await
    }

    /// Merge a pull request
    pub async fn merge_pull_request(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
        request: Option<&MergePullRequestRequest>,
    ) -> Result<PullRequest> {
        let default_request = MergePullRequestRequest::default();
        let request = request.unwrap_or(&default_request);

        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/merge",
            workspace, repo_slug, pr_id
        );
        self.post(&path, request).await
    }

    /// Approve a pull request
    pub async fn approve_pull_request(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/approve",
            workspace, repo_slug, pr_id
        );
        self.post_no_response(&path, &serde_json::json!({})).await
    }

    /// Unapprove a pull request
    pub async fn unapprove_pull_request(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/approve",
            workspace, repo_slug, pr_id
        );
        self.delete(&path).await
    }

    /// Request changes on a pull request
    pub async fn request_pr_changes(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/request-changes",
            workspace, repo_slug, pr_id
        );
        self.post_no_response(&path, &serde_json::json!({})).await
    }

    /// Withdraw a previous request for changes on a pull request
    pub async fn unrequest_pr_changes(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/request-changes",
            workspace, repo_slug, pr_id
        );
        self.delete(&path).await
    }

    /// Decline a pull request
    pub async fn decline_pull_request(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
    ) -> Result<PullRequest> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/decline",
            workspace, repo_slug, pr_id
        );
        self.post(&path, &serde_json::json!({})).await
    }

    /// List the most recent comments on a pull request, newest first.
    ///
    /// Requests pages sorted by `-created_on` and stops following the
    /// pagination `next` links as soon as `max_items` comments have been
    /// collected (or the pages are exhausted), so large PRs don't require
    /// downloading their entire comment history.
    pub async fn list_recent_pr_comments(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
        max_items: usize,
    ) -> Result<Vec<PullRequestComment>> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/comments",
            workspace, repo_slug, pr_id
        );

        let first_page: Paginated<PullRequestComment> = self
            .get_with_query(&path, &[("sort", "-created_on"), ("pagelen", "100")])
            .await?;

        collect_limited(first_page, max_items, |next_url| async move {
            self.get_absolute(&next_url).await
        })
        .await
    }

    /// Get a specific comment on a pull request
    pub async fn get_pr_comment(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
        comment_id: u64,
    ) -> Result<PullRequestComment> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/comments/{}",
            workspace, repo_slug, pr_id, comment_id
        );
        self.get(&path).await
    }

    /// Add a comment to a pull request, optionally inline (`path` + `line`)
    /// and/or as a reply to `parent`.
    pub(crate) async fn add_pr_comment_full(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
        input: PrCommentInput<'_>,
    ) -> Result<PullRequestComment> {
        #[derive(serde::Serialize)]
        struct CommentRequest {
            content: ContentRequest,
            #[serde(skip_serializing_if = "Option::is_none")]
            inline: Option<InlineComment>,
            #[serde(skip_serializing_if = "Option::is_none")]
            parent: Option<CommentRef>,
        }

        #[derive(serde::Serialize)]
        struct ContentRequest {
            raw: String,
        }

        let request = CommentRequest {
            content: ContentRequest {
                raw: input.content.to_string(),
            },
            inline: input.path.map(|p| InlineComment {
                from: None,
                to: input.line,
                path: p.to_string(),
            }),
            parent: input.parent.map(|id| CommentRef { id }),
        };

        let api_path = format!(
            "/repositories/{}/{}/pullrequests/{}/comments",
            workspace, repo_slug, pr_id
        );
        self.post(&api_path, &request).await
    }

    /// Update the content of an existing pull request comment
    pub async fn update_pr_comment(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
        comment_id: u64,
        content: &str,
    ) -> Result<PullRequestComment> {
        let request = serde_json::json!({ "content": { "raw": content } });

        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/comments/{}",
            workspace, repo_slug, pr_id, comment_id
        );
        self.put(&path, &request).await
    }

    /// Delete a pull request comment
    pub async fn delete_pr_comment(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
        comment_id: u64,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/comments/{}",
            workspace, repo_slug, pr_id, comment_id
        );
        self.delete(&path).await
    }

    /// Resolve a pull request comment thread
    pub async fn resolve_pr_comment(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
        comment_id: u64,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/comments/{}/resolve",
            workspace, repo_slug, pr_id, comment_id
        );
        self.post_no_response(&path, &serde_json::json!({})).await
    }

    /// Reopen (unresolve) a pull request comment thread
    pub async fn unresolve_pr_comment(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
        comment_id: u64,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/comments/{}/resolve",
            workspace, repo_slug, pr_id, comment_id
        );
        self.delete(&path).await
    }

    /// Get the diff for a pull request
    pub async fn get_pr_diff(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
    ) -> Result<String> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/diff",
            workspace, repo_slug, pr_id
        );

        self.get_text(&path, Some("text/plain")).await
    }

    /// Get the patch (mbox-style) for a pull request
    pub async fn get_pr_patch(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
    ) -> Result<String> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/patch",
            workspace, repo_slug, pr_id
        );

        self.get_text(&path, None).await
    }

    /// List one page of commits on a pull request
    pub async fn list_pr_commits(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
        pagelen: Option<u32>,
    ) -> Result<Paginated<PrCommit>> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/commits",
            workspace, repo_slug, pr_id
        );

        match pagelen {
            Some(len) => {
                let len = len.to_string();
                self.get_with_query(&path, &[("pagelen", len.as_str())])
                    .await
            }
            None => self.get(&path).await,
        }
    }

    /// List all commit statuses (builds) for a pull request's head commit
    pub async fn list_pr_statuses(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
    ) -> Result<Vec<CommitStatus>> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/statuses",
            workspace, repo_slug, pr_id
        );
        self.get_all_pages(&path).await
    }

    /// Get one page of the diffstat (per-file change summary) for a pull request
    pub async fn get_pr_diffstat(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
        pagelen: Option<u32>,
    ) -> Result<Paginated<DiffStat>> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/diffstat",
            workspace, repo_slug, pr_id
        );

        match pagelen {
            Some(len) => {
                let len = len.to_string();
                self.get_with_query(&path, &[("pagelen", len.as_str())])
                    .await
            }
            None => self.get(&path).await,
        }
    }

    /// List all tasks on a pull request
    pub async fn list_pr_tasks(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
    ) -> Result<Vec<PrTask>> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/tasks",
            workspace, repo_slug, pr_id
        );
        self.get_all_pages(&path).await
    }

    /// Create a task on a pull request
    pub async fn add_pr_task(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
        content: &str,
    ) -> Result<PrTask> {
        let request = serde_json::json!({ "content": { "raw": content } });

        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/tasks",
            workspace, repo_slug, pr_id
        );
        self.post(&path, &request).await
    }

    /// Update a pull request task's content and/or state
    /// (`state` is `"RESOLVED"` or `"UNRESOLVED"`).
    pub async fn update_pr_task(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
        task_id: u64,
        content: Option<&str>,
        state: Option<&str>,
    ) -> Result<PrTask> {
        #[derive(serde::Serialize)]
        struct TaskUpdateRequest {
            #[serde(skip_serializing_if = "Option::is_none")]
            content: Option<TaskContentRequest>,
            #[serde(skip_serializing_if = "Option::is_none")]
            state: Option<String>,
        }

        #[derive(serde::Serialize)]
        struct TaskContentRequest {
            raw: String,
        }

        let request = TaskUpdateRequest {
            content: content.map(|c| TaskContentRequest { raw: c.to_string() }),
            state: state.map(|s| s.to_string()),
        };

        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/tasks/{}",
            workspace, repo_slug, pr_id, task_id
        );
        self.put(&path, &request).await
    }

    /// Delete a pull request task
    pub async fn delete_pr_task(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
        task_id: u64,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/tasks/{}",
            workspace, repo_slug, pr_id, task_id
        );
        self.delete(&path).await
    }

    /// List one page of the activity feed for a pull request
    pub async fn list_pr_activity(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
        pagelen: Option<u32>,
    ) -> Result<Paginated<PrActivity>> {
        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/activity",
            workspace, repo_slug, pr_id
        );

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

/// Accumulate paginated values until `max_items` have been collected.
///
/// Starts from an already-fetched `first_page`, follows pagination `next`
/// links via `fetch_next` only while fewer than `max_items` values have been
/// collected (or until a page has no `next` link), and truncates the result
/// to at most `max_items`.
async fn collect_limited<T, F, Fut>(
    first_page: Paginated<T>,
    max_items: usize,
    mut fetch_next: F,
) -> Result<Vec<T>>
where
    F: FnMut(String) -> Fut,
    Fut: std::future::Future<Output = Result<Paginated<T>>>,
{
    let mut items: Vec<T> = Vec::new();
    let mut page = first_page;

    loop {
        items.extend(page.values);

        if items.len() >= max_items {
            items.truncate(max_items);
            break;
        }

        match page.next {
            Some(next_url) => page = fetch_next(next_url).await?,
            None => break,
        }
    }

    Ok(items)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::future::{Ready, ready};

    use super::collect_limited;
    use crate::models::Paginated;

    fn page(values: Vec<u32>, has_next: bool) -> Paginated<u32> {
        Paginated {
            size: None,
            page: None,
            pagelen: None,
            next: has_next.then(|| "https://api.bitbucket.org/next".to_string()),
            previous: None,
            values,
        }
    }

    type PageFuture = Ready<anyhow::Result<Paginated<u32>>>;

    #[tokio::test]
    async fn follows_next_links_until_exhausted_when_under_max() {
        let calls = Cell::new(0u32);
        let fetch = |url: String| -> PageFuture {
            calls.set(calls.get() + 1);
            assert_eq!(url, "https://api.bitbucket.org/next");
            ready(Ok(page(vec![3], false)))
        };

        let items = collect_limited(page(vec![1, 2], true), 10, fetch)
            .await
            .expect("collection should succeed");

        assert_eq!(items, vec![1, 2, 3]);
        assert_eq!(calls.get(), 1);
    }

    #[tokio::test]
    async fn truncates_to_max_and_stops_fetching_once_reached() {
        let calls = Cell::new(0u32);
        let fetch = |_url: String| -> PageFuture {
            calls.set(calls.get() + 1);
            // This page still advertises a `next` link, but the collection
            // reaches `max_items` here, so no further fetch may happen.
            ready(Ok(page(vec![3, 4], true)))
        };

        let items = collect_limited(page(vec![1, 2], true), 3, fetch)
            .await
            .expect("collection should succeed");

        assert_eq!(items, vec![1, 2, 3]);
        assert_eq!(calls.get(), 1);
    }

    #[tokio::test]
    async fn exactly_max_on_first_page_makes_zero_fetches() {
        let calls = Cell::new(0u32);
        let fetch = |_url: String| -> PageFuture {
            calls.set(calls.get() + 1);
            panic!("no page beyond the first should be fetched");
        };

        let items = collect_limited(page(vec![1, 2, 3], true), 3, fetch)
            .await
            .expect("collection should succeed");

        assert_eq!(items, vec![1, 2, 3]);
        assert_eq!(calls.get(), 0);
    }
}
