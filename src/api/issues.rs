use std::path::PathBuf;

use anyhow::{Context, Result};
use reqwest::multipart::{Form, Part};

use super::BitbucketClient;
use crate::api::util::urlencode_segment;
use crate::models::{
    CreateIssueCommentRequest, CreateIssueRequest, Issue, IssueAttachment, IssueChange,
    IssueComment, IssueContentRequest, IssueKind, IssueMetaItem, IssuePriority, IssueState,
    Paginated, UserAccountId,
};

/// Fields of an issue that can be changed with
/// [`BitbucketClient::update_issue_full`]. Unset fields are left untouched.
#[derive(Default)]
pub struct IssueUpdate<'a> {
    pub title: Option<&'a str>,
    /// Raw markdown body for the issue description.
    pub content: Option<&'a str>,
    pub kind: Option<IssueKind>,
    pub priority: Option<IssuePriority>,
    /// Atlassian account id of the new assignee.
    pub assignee: Option<&'a str>,
    pub state: Option<IssueState>,
}

impl IssueUpdate<'_> {
    /// True when no field is set, i.e. the update would be a no-op.
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.content.is_none()
            && self.kind.is_none()
            && self.priority.is_none()
            && self.assignee.is_none()
            && self.state.is_none()
    }
}

/// Filters and paging options for [`BitbucketClient::list_issues_filtered`].
#[derive(Default)]
pub(crate) struct IssueListFilters<'a> {
    pub state: Option<IssueState>,
    pub kind: Option<IssueKind>,
    pub priority: Option<IssuePriority>,
    /// Atlassian account id of the assignee to filter by.
    pub assignee: Option<&'a str>,
    /// Atlassian account id of the reporter to filter by.
    pub reporter: Option<&'a str>,
    /// Raw BBQL query; when set it wins over the individual filters above.
    pub query: Option<&'a str>,
    /// Sort field, e.g. `-updated_on` for most recently updated first.
    pub sort: Option<&'a str>,
    pub page: Option<u32>,
    pub pagelen: Option<u32>,
}

impl BitbucketClient {
    /// List issues for a repository
    pub async fn list_issues(
        &self,
        workspace: &str,
        repo_slug: &str,
        state: Option<IssueState>,
        page: Option<u32>,
        pagelen: Option<u32>,
    ) -> Result<Paginated<Issue>> {
        self.list_issues_filtered(
            workspace,
            repo_slug,
            &IssueListFilters {
                state,
                page,
                pagelen,
                ..Default::default()
            },
        )
        .await
    }

    /// List issues for a repository with the full set of filters, an optional
    /// raw BBQL query, and sorting.
    pub(crate) async fn list_issues_filtered(
        &self,
        workspace: &str,
        repo_slug: &str,
        filters: &IssueListFilters<'_>,
    ) -> Result<Paginated<Issue>> {
        let query = issue_list_query_full(filters);
        let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let path = format!("/repositories/{}/{}/issues", workspace, repo_slug);
        self.get_with_query(&path, &query_refs).await
    }

    /// Get a specific issue
    pub async fn get_issue(
        &self,
        workspace: &str,
        repo_slug: &str,
        issue_id: u64,
    ) -> Result<Issue> {
        let path = format!(
            "/repositories/{}/{}/issues/{}",
            workspace, repo_slug, issue_id
        );
        self.get(&path).await
    }

    /// Create a new issue
    pub async fn create_issue(
        &self,
        workspace: &str,
        repo_slug: &str,
        request: &CreateIssueRequest,
    ) -> Result<Issue> {
        let path = format!("/repositories/{}/{}/issues", workspace, repo_slug);
        self.post(&path, request).await
    }

    /// Update an issue's title, description and/or state.
    ///
    /// Convenience wrapper over [`Self::update_issue_full`] kept for the
    /// existing close/reopen callers.
    pub async fn update_issue(
        &self,
        workspace: &str,
        repo_slug: &str,
        issue_id: u64,
        title: Option<&str>,
        content: Option<&str>,
        state: Option<IssueState>,
    ) -> Result<Issue> {
        self.update_issue_full(
            workspace,
            repo_slug,
            issue_id,
            &IssueUpdate {
                title,
                content,
                state,
                ..Default::default()
            },
        )
        .await
    }

    /// Update any combination of an issue's editable fields; unset fields are
    /// omitted from the request body and left unchanged server-side.
    pub async fn update_issue_full(
        &self,
        workspace: &str,
        repo_slug: &str,
        issue_id: u64,
        update: &IssueUpdate<'_>,
    ) -> Result<Issue> {
        #[derive(serde::Serialize)]
        struct UpdateRequest<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            title: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            content: Option<IssueContentRequest>,
            #[serde(skip_serializing_if = "Option::is_none")]
            kind: Option<&'a IssueKind>,
            #[serde(skip_serializing_if = "Option::is_none")]
            priority: Option<&'a IssuePriority>,
            #[serde(skip_serializing_if = "Option::is_none")]
            assignee: Option<UserAccountId>,
            #[serde(skip_serializing_if = "Option::is_none")]
            state: Option<&'a IssueState>,
        }

        let request = UpdateRequest {
            title: update.title,
            content: update
                .content
                .map(|c| IssueContentRequest { raw: c.to_string() }),
            kind: update.kind.as_ref(),
            priority: update.priority.as_ref(),
            assignee: update.assignee.map(|a| UserAccountId {
                account_id: a.to_string(),
            }),
            state: update.state.as_ref(),
        };

        let path = format!(
            "/repositories/{}/{}/issues/{}",
            workspace, repo_slug, issue_id
        );
        self.put(&path, &request).await
    }

    /// Delete an issue
    pub async fn delete_issue(
        &self,
        workspace: &str,
        repo_slug: &str,
        issue_id: u64,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/issues/{}",
            workspace, repo_slug, issue_id
        );
        self.delete(&path).await
    }

    /// List comments on an issue
    pub async fn list_issue_comments(
        &self,
        workspace: &str,
        repo_slug: &str,
        issue_id: u64,
        pagelen: Option<u32>,
    ) -> Result<Paginated<IssueComment>> {
        let query: Vec<(&str, String)> = pagelen
            .map(|len| ("pagelen", len.to_string()))
            .into_iter()
            .collect();
        let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let path = format!(
            "/repositories/{}/{}/issues/{}/comments",
            workspace, repo_slug, issue_id
        );
        self.get_with_query(&path, &query_refs).await
    }

    /// Add a comment to an issue
    pub async fn add_issue_comment(
        &self,
        workspace: &str,
        repo_slug: &str,
        issue_id: u64,
        content: &str,
    ) -> Result<IssueComment> {
        let request = CreateIssueCommentRequest {
            content: IssueContentRequest {
                raw: content.to_string(),
            },
        };

        let path = format!(
            "/repositories/{}/{}/issues/{}/comments",
            workspace, repo_slug, issue_id
        );
        self.post(&path, &request).await
    }

    /// Update the body of an existing comment on an issue.
    pub async fn update_issue_comment(
        &self,
        workspace: &str,
        repo_slug: &str,
        issue_id: u64,
        comment_id: u64,
        content: &str,
    ) -> Result<IssueComment> {
        let request = CreateIssueCommentRequest {
            content: IssueContentRequest {
                raw: content.to_string(),
            },
        };

        let path = format!(
            "/repositories/{}/{}/issues/{}/comments/{}",
            workspace, repo_slug, issue_id, comment_id
        );
        self.put(&path, &request).await
    }

    /// Delete a comment from an issue.
    pub async fn delete_issue_comment(
        &self,
        workspace: &str,
        repo_slug: &str,
        issue_id: u64,
        comment_id: u64,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/issues/{}/comments/{}",
            workspace, repo_slug, issue_id, comment_id
        );
        self.delete(&path).await
    }

    /// List entries from an issue's change log (state transitions,
    /// reassignments, ...).
    pub async fn list_issue_changes(
        &self,
        workspace: &str,
        repo_slug: &str,
        issue_id: u64,
        pagelen: Option<u32>,
    ) -> Result<Paginated<IssueChange>> {
        let query: Vec<(&str, String)> = pagelen
            .map(|len| ("pagelen", len.to_string()))
            .into_iter()
            .collect();
        let query_refs: Vec<(&str, &str)> = query.iter().map(|(k, v)| (*k, v.as_str())).collect();

        let path = format!(
            "/repositories/{}/{}/issues/{}/changes",
            workspace, repo_slug, issue_id
        );
        self.get_with_query(&path, &query_refs).await
    }

    /// List the components defined in a repository's issue tracker,
    /// following pagination until every page has been fetched.
    pub async fn list_issue_components(
        &self,
        workspace: &str,
        repo_slug: &str,
    ) -> Result<Vec<IssueMetaItem>> {
        let path = format!("/repositories/{}/{}/components", workspace, repo_slug);
        self.get_all_pages::<IssueMetaItem>(&path).await
    }

    /// List the milestones defined in a repository's issue tracker,
    /// following pagination until every page has been fetched.
    pub async fn list_issue_milestones(
        &self,
        workspace: &str,
        repo_slug: &str,
    ) -> Result<Vec<IssueMetaItem>> {
        let path = format!("/repositories/{}/{}/milestones", workspace, repo_slug);
        self.get_all_pages::<IssueMetaItem>(&path).await
    }

    /// List the versions defined in a repository's issue tracker,
    /// following pagination until every page has been fetched.
    pub async fn list_issue_versions(
        &self,
        workspace: &str,
        repo_slug: &str,
    ) -> Result<Vec<IssueMetaItem>> {
        let path = format!("/repositories/{}/{}/versions", workspace, repo_slug);
        self.get_all_pages::<IssueMetaItem>(&path).await
    }

    /// List the files attached to an issue, following pagination until every
    /// page has been fetched.
    pub async fn list_issue_attachments(
        &self,
        workspace: &str,
        repo_slug: &str,
        issue_id: u64,
    ) -> Result<Vec<IssueAttachment>> {
        let path = format!(
            "/repositories/{}/{}/issues/{}/attachments",
            workspace, repo_slug, issue_id
        );
        self.get_all_pages::<IssueAttachment>(&path).await
    }

    /// Attach one or more files to an issue via a multipart upload.
    ///
    /// Follows the repository downloads upload convention: every file goes
    /// under the "files" form field, keyed server-side by filename.
    pub async fn upload_issue_attachments(
        &self,
        workspace: &str,
        repo_slug: &str,
        issue_id: u64,
        files: &[(String, PathBuf)],
    ) -> Result<()> {
        let mut form = Form::new();

        for (upload_name, path) in files {
            // Bodies are buffered in memory, matching upload_downloads:
            // streaming would require the reqwest `stream` feature +
            // tokio-util; acceptable for CLI-sized attachments.
            let bytes = std::fs::read(path)
                .with_context(|| format!("Failed to read file '{}'", path.display()))?;
            let part = Part::bytes(bytes).file_name(upload_name.clone());
            form = form.part("files", part);
        }

        let path = format!(
            "/repositories/{}/{}/issues/{}/attachments",
            workspace, repo_slug, issue_id
        );
        self.post_multipart(&path, form).await
    }

    /// Delete an attachment from an issue by its path (usually the filename
    /// as returned by the attachments list).
    pub async fn delete_issue_attachment(
        &self,
        workspace: &str,
        repo_slug: &str,
        issue_id: u64,
        attachment_path: &str,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/issues/{}/attachments/{}",
            workspace,
            repo_slug,
            issue_id,
            urlencode_segment(attachment_path)
        );
        self.delete(&path).await
    }

    /// Vote for an issue
    pub async fn vote_issue(&self, workspace: &str, repo_slug: &str, issue_id: u64) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/issues/{}/vote",
            workspace, repo_slug, issue_id
        );
        self.put::<serde_json::Value, _>(&path, &serde_json::json!({}))
            .await?;
        Ok(())
    }

    /// Remove vote from an issue
    pub async fn unvote_issue(
        &self,
        workspace: &str,
        repo_slug: &str,
        issue_id: u64,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/issues/{}/vote",
            workspace, repo_slug, issue_id
        );
        self.delete(&path).await
    }

    /// Watch an issue
    pub async fn watch_issue(&self, workspace: &str, repo_slug: &str, issue_id: u64) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/issues/{}/watch",
            workspace, repo_slug, issue_id
        );
        self.put::<serde_json::Value, _>(&path, &serde_json::json!({}))
            .await?;
        Ok(())
    }

    /// Unwatch an issue
    pub async fn unwatch_issue(
        &self,
        workspace: &str,
        repo_slug: &str,
        issue_id: u64,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/issues/{}/watch",
            workspace, repo_slug, issue_id
        );
        self.delete(&path).await
    }
}

/// Build the query parameters for listing issues.
///
/// Kept with its original three-argument shape so the original tests keep
/// covering the delegation path; the full builder lives in
/// [`issue_list_query_full`], which production code reaches through
/// [`BitbucketClient::list_issues_filtered`].
#[cfg(test)]
fn issue_list_query(
    state: Option<IssueState>,
    page: Option<u32>,
    pagelen: Option<u32>,
) -> Vec<(&'static str, String)> {
    issue_list_query_full(&IssueListFilters {
        state,
        page,
        pagelen,
        ..Default::default()
    })
}

/// Build the full set of query parameters for listing issues.
///
/// A raw BBQL `query` wins over the individual filters; otherwise the
/// filters are AND-ed together into a single `q` expression.
fn issue_list_query_full(filters: &IssueListFilters<'_>) -> Vec<(&'static str, String)> {
    let mut query = Vec::new();

    let q = match filters.query {
        Some(raw) => Some(raw.to_string()),
        None => issue_query_filter(
            filters.state.as_ref(),
            filters.kind.as_ref(),
            filters.priority.as_ref(),
            filters.assignee,
            filters.reporter,
        ),
    };

    if let Some(q) = q {
        query.push(("q", q));
    }
    if let Some(sort) = filters.sort {
        query.push(("sort", sort.to_string()));
    }
    if let Some(p) = filters.page {
        query.push(("page", p.to_string()));
    }
    if let Some(len) = filters.pagelen {
        query.push(("pagelen", len.to_string()));
    }

    query
}

/// Combine individual issue filters into a single BBQL `q` expression, or
/// `None` when no filter is set.
///
/// Every value is quoted, since some states (e.g. "on hold") contain spaces.
/// Assignee and reporter filter on `*.account_id`, matching the account ids
/// the rest of the CLI accepts.
fn issue_query_filter(
    state: Option<&IssueState>,
    kind: Option<&IssueKind>,
    priority: Option<&IssuePriority>,
    assignee: Option<&str>,
    reporter: Option<&str>,
) -> Option<String> {
    let mut parts = Vec::new();

    if let Some(s) = state {
        parts.push(format!("state=\"{}\"", s));
    }
    if let Some(k) = kind {
        parts.push(format!("kind=\"{}\"", k));
    }
    if let Some(p) = priority {
        parts.push(format!("priority=\"{}\"", p));
    }
    if let Some(a) = assignee {
        parts.push(format!("assignee.account_id={}", bbql_quote(a)));
    }
    if let Some(r) = reporter {
        parts.push(format!("reporter.account_id={}", bbql_quote(r)));
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" AND "))
    }
}

/// Quote a free-text value for use inside a BBQL `q=` expression: wrap it in
/// double quotes, backslash-escaping any `\` (first) then any `"` so the value
/// cannot break out of the quoted string and inject BBQL. Used for
/// user-supplied `--assignee`/`--reporter` values; state/kind/priority come
/// from enums and are already safe.
fn bbql_quote(s: &str) -> String {
    let escaped = s.replace('\\', "\\\\").replace('"', "\\\"");
    format!("\"{}\"", escaped)
}

#[cfg(test)]
mod tests {
    use super::{
        IssueListFilters, bbql_quote, issue_list_query, issue_list_query_full, issue_query_filter,
        urlencode_segment,
    };
    use crate::models::{IssueKind, IssuePriority, IssueState};

    #[test]
    fn open_state_builds_quoted_bbql_filter() {
        assert_eq!(
            issue_list_query(Some(IssueState::Open), None, None),
            vec![("q", "state=\"open\"".to_string())]
        );
    }

    #[test]
    fn on_hold_state_keeps_two_word_value_quoted() {
        assert_eq!(
            issue_list_query(Some(IssueState::OnHold), None, None),
            vec![("q", "state=\"on hold\"".to_string())]
        );
    }

    #[test]
    fn no_state_omits_q_pair() {
        let query = issue_list_query(None, None, None);
        assert!(query.is_empty());
    }

    #[test]
    fn page_and_pagelen_pass_through_as_strings() {
        assert_eq!(
            issue_list_query(None, Some(2), Some(50)),
            vec![("page", "2".to_string()), ("pagelen", "50".to_string())]
        );
    }

    #[test]
    fn state_page_and_pagelen_combine_in_order() {
        assert_eq!(
            issue_list_query(Some(IssueState::Open), Some(3), Some(25)),
            vec![
                ("q", "state=\"open\"".to_string()),
                ("page", "3".to_string()),
                ("pagelen", "25".to_string()),
            ]
        );
    }

    #[test]
    fn query_filter_is_none_without_filters() {
        assert_eq!(issue_query_filter(None, None, None, None, None), None);
    }

    #[test]
    fn query_filter_quotes_on_hold_state() {
        assert_eq!(
            issue_query_filter(Some(&IssueState::OnHold), None, None, None, None).as_deref(),
            Some("state=\"on hold\"")
        );
    }

    #[test]
    fn query_filter_ands_all_filters_in_order() {
        assert_eq!(
            issue_query_filter(
                Some(&IssueState::Open),
                Some(&IssueKind::Bug),
                Some(&IssuePriority::Major),
                Some("acc-123"),
                Some("acc-456"),
            )
            .as_deref(),
            Some(
                "state=\"open\" AND kind=\"bug\" AND priority=\"major\" AND \
                 assignee.account_id=\"acc-123\" AND reporter.account_id=\"acc-456\""
            )
        );
    }

    #[test]
    fn query_filter_handles_people_filters_alone() {
        assert_eq!(
            issue_query_filter(None, None, None, Some("acc-123"), None).as_deref(),
            Some("assignee.account_id=\"acc-123\"")
        );
        assert_eq!(
            issue_query_filter(None, None, None, None, Some("acc-456")).as_deref(),
            Some("reporter.account_id=\"acc-456\"")
        );
    }

    #[test]
    fn full_query_raw_query_wins_over_filters() {
        let filters = IssueListFilters {
            state: Some(IssueState::Open),
            kind: Some(IssueKind::Bug),
            query: Some("kind=\"task\""),
            ..Default::default()
        };
        assert_eq!(
            issue_list_query_full(&filters),
            vec![("q", "kind=\"task\"".to_string())]
        );
    }

    #[test]
    fn full_query_includes_sort_page_and_pagelen_in_order() {
        let filters = IssueListFilters {
            state: Some(IssueState::Open),
            sort: Some("-updated_on"),
            page: Some(2),
            pagelen: Some(50),
            ..Default::default()
        };
        assert_eq!(
            issue_list_query_full(&filters),
            vec![
                ("q", "state=\"open\"".to_string()),
                ("sort", "-updated_on".to_string()),
                ("page", "2".to_string()),
                ("pagelen", "50".to_string()),
            ]
        );
    }

    #[test]
    fn bbql_quote_escapes_backslash_and_double_quote() {
        // A value with both a `"` (would close the string) and a `\` (would
        // escape the next char) must come back fully escaped and wrapped.
        assert_eq!(bbql_quote(r#"a"b\c"#), r#""a\"b\\c""#);
        // A plain value is just wrapped in quotes.
        assert_eq!(bbql_quote("acc-123"), r#""acc-123""#);
        // An injection attempt cannot break out of the quoted string.
        assert_eq!(
            bbql_quote(r#"x" OR state="open"#),
            r#""x\" OR state=\"open""#
        );
    }

    #[test]
    fn urlencode_encodes_spaces_and_leaves_safe_chars() {
        assert_eq!(urlencode_segment("shot 1.png"), "shot%201.png");
        assert_eq!(urlencode_segment("a-b_c.d~1"), "a-b_c.d~1");
    }
}
