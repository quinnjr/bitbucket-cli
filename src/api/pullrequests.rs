use anyhow::Result;

use super::BitbucketClient;
use crate::models::{
    CreatePullRequestRequest, MergePullRequestRequest, Paginated, PullRequest, PullRequestComment,
    PullRequestState,
};

impl BitbucketClient {
    /// List pull requests for a repository
    pub async fn list_pull_requests(
        &self,
        workspace: &str,
        repo_slug: &str,
        state: Option<PullRequestState>,
        page: Option<u32>,
        pagelen: Option<u32>,
    ) -> Result<Paginated<PullRequest>> {
        let mut query = Vec::new();

        if let Some(s) = state {
            query.push(("state", s.to_string()));
        }
        if let Some(p) = page {
            query.push(("page", p.to_string()));
        }
        if let Some(len) = pagelen {
            query.push(("pagelen", len.to_string()));
        }

        let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();

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

    /// Update a pull request
    pub async fn update_pull_request(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
        title: Option<&str>,
        description: Option<&str>,
    ) -> Result<PullRequest> {
        #[derive(serde::Serialize)]
        struct UpdateRequest {
            #[serde(skip_serializing_if = "Option::is_none")]
            title: Option<String>,
            #[serde(skip_serializing_if = "Option::is_none")]
            description: Option<String>,
        }

        let request = UpdateRequest {
            title: title.map(|t| t.to_string()),
            description: description.map(|d| d.to_string()),
        };

        let path = format!(
            "/repositories/{}/{}/pullrequests/{}",
            workspace, repo_slug, pr_id
        );
        self.put(&path, &request).await
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

    /// Add a comment to a pull request
    pub async fn add_pr_comment(
        &self,
        workspace: &str,
        repo_slug: &str,
        pr_id: u64,
        content: &str,
    ) -> Result<PullRequestComment> {
        #[derive(serde::Serialize)]
        struct CommentRequest {
            content: ContentRequest,
        }

        #[derive(serde::Serialize)]
        struct ContentRequest {
            raw: String,
        }

        let request = CommentRequest {
            content: ContentRequest {
                raw: content.to_string(),
            },
        };

        let path = format!(
            "/repositories/{}/{}/pullrequests/{}/comments",
            workspace, repo_slug, pr_id
        );
        self.post(&path, &request).await
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
