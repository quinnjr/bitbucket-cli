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

    /// List comments on a pull request
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

        let mut comments: Vec<PullRequestComment> = Vec::new();
        let mut page: Paginated<PullRequestComment> = self
            .get_with_query(&path, &[("sort", "-created_on"), ("pagelen", "100")])
            .await?;

        loop {
            comments.extend(page.values);

            if comments.len() >= max_items {
                comments.truncate(max_items);
                break;
            }

            match page.next {
                Some(next_url) => page = self.get_absolute(&next_url).await?,
                None => break,
            }
        }

        Ok(comments)
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
