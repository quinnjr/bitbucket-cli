use anyhow::Result;

use super::BitbucketClient;
use crate::api::util::urlencode_segment;
use crate::models::{
    BranchDetail, CreateBranchRequest, CreateTagRequest, Paginated, RefTarget, TagDetail,
};

impl BitbucketClient {
    /// List branches in a repository, optionally filtered and sorted.
    ///
    /// `query` is a Bitbucket filter expression (e.g. `name ~ "feat"`) passed
    /// as `q`; `sort` is a field name, prefix with `-` for descending
    /// (e.g. `-target.date`).
    pub async fn list_branches_filtered(
        &self,
        workspace: &str,
        repo_slug: &str,
        query: Option<&str>,
        sort: Option<&str>,
        page: Option<u32>,
        pagelen: Option<u32>,
    ) -> Result<Paginated<BranchDetail>> {
        let path = format!("/repositories/{}/{}/refs/branches", workspace, repo_slug);
        self.get_ref_page(&path, query, sort, page, pagelen).await
    }

    /// Create a branch pointing at `target_hash`.
    pub async fn create_branch(
        &self,
        workspace: &str,
        repo_slug: &str,
        name: &str,
        target_hash: &str,
    ) -> Result<BranchDetail> {
        let path = format!("/repositories/{}/{}/refs/branches", workspace, repo_slug);
        let request = CreateBranchRequest {
            name: name.to_string(),
            target: RefTarget {
                hash: target_hash.to_string(),
            },
        };
        self.post(&path, &request).await
    }

    /// Get a single branch by name.
    pub async fn get_branch(
        &self,
        workspace: &str,
        repo_slug: &str,
        name: &str,
    ) -> Result<BranchDetail> {
        let path = format!(
            "/repositories/{}/{}/refs/branches/{}",
            workspace,
            repo_slug,
            urlencode_segment(name)
        );
        self.get(&path).await
    }

    /// Delete a branch by name.
    pub async fn delete_branch(&self, workspace: &str, repo_slug: &str, name: &str) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/refs/branches/{}",
            workspace,
            repo_slug,
            urlencode_segment(name)
        );
        self.delete(&path).await
    }

    /// List tags in a repository, optionally filtered and sorted.
    ///
    /// `query` and `sort` behave as in [`Self::list_branches_filtered`].
    pub async fn list_tags_filtered(
        &self,
        workspace: &str,
        repo_slug: &str,
        query: Option<&str>,
        sort: Option<&str>,
        page: Option<u32>,
        pagelen: Option<u32>,
    ) -> Result<Paginated<TagDetail>> {
        let path = format!("/repositories/{}/{}/refs/tags", workspace, repo_slug);
        self.get_ref_page(&path, query, sort, page, pagelen).await
    }

    /// Create a tag pointing at `target_hash`. Passing a `message` creates an
    /// annotated tag; omitting it creates a lightweight tag.
    pub async fn create_tag(
        &self,
        workspace: &str,
        repo_slug: &str,
        name: &str,
        target_hash: &str,
        message: Option<&str>,
    ) -> Result<TagDetail> {
        let path = format!("/repositories/{}/{}/refs/tags", workspace, repo_slug);
        let request = CreateTagRequest {
            name: name.to_string(),
            target: RefTarget {
                hash: target_hash.to_string(),
            },
            message: message.map(String::from),
        };
        self.post(&path, &request).await
    }

    /// Get a single tag by name.
    pub async fn get_tag(&self, workspace: &str, repo_slug: &str, name: &str) -> Result<TagDetail> {
        let path = format!(
            "/repositories/{}/{}/refs/tags/{}",
            workspace,
            repo_slug,
            urlencode_segment(name)
        );
        self.get(&path).await
    }

    /// Delete a tag by name.
    pub async fn delete_tag(&self, workspace: &str, repo_slug: &str, name: &str) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/refs/tags/{}",
            workspace,
            repo_slug,
            urlencode_segment(name)
        );
        self.delete(&path).await
    }

    /// Shared GET for both ref-listing endpoints: attaches the optional
    /// `q` / `sort` / `page` / `pagelen` query parameters.
    async fn get_ref_page<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        query: Option<&str>,
        sort: Option<&str>,
        page: Option<u32>,
        pagelen: Option<u32>,
    ) -> Result<Paginated<T>> {
        let mut params = Vec::new();

        if let Some(q) = query {
            params.push(("q", q.to_string()));
        }
        if let Some(s) = sort {
            params.push(("sort", s.to_string()));
        }
        if let Some(p) = page {
            params.push(("page", p.to_string()));
        }
        if let Some(len) = pagelen {
            params.push(("pagelen", len.to_string()));
        }

        let param_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.get_with_query(path, &param_refs).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urlencode_leaves_safe_chars_untouched() {
        assert_eq!(urlencode_segment("v1.0.0-rc_2~x"), "v1.0.0-rc_2~x");
    }

    #[test]
    fn urlencode_encodes_slash_in_branch_names() {
        assert_eq!(urlencode_segment("feature/login"), "feature%2Flogin");
    }

    #[test]
    fn urlencode_encodes_spaces_and_specials() {
        assert_eq!(urlencode_segment("wip branch#1"), "wip%20branch%231");
    }
}
