use anyhow::Result;

use super::BitbucketClient;
use crate::api::util::urlencode_segment;
use crate::models::{FileHistoryEntry, Paginated, SourceEntry};

impl BitbucketClient {
    /// List the entries of a directory at `path` (empty string for the
    /// repository root) as of `git_ref` (branch, tag, or commit hash).
    pub async fn list_source(
        &self,
        workspace: &str,
        repo_slug: &str,
        git_ref: &str,
        path: &str,
        page: Option<u32>,
        pagelen: Option<u32>,
    ) -> Result<Paginated<SourceEntry>> {
        let endpoint = source_path(workspace, repo_slug, git_ref, path);

        let mut params = Vec::new();
        if let Some(p) = page {
            params.push(("page", p.to_string()));
        }
        if let Some(len) = pagelen {
            params.push(("pagelen", len.to_string()));
        }

        let param_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.get_with_query(&endpoint, &param_refs).await
    }

    /// Fetch the raw contents of the file at `path` as of `git_ref`.
    ///
    /// This is the same endpoint as [`Self::list_source`]; pointed at a file
    /// instead of a directory, it returns the file body verbatim.
    pub async fn get_file_raw(
        &self,
        workspace: &str,
        repo_slug: &str,
        git_ref: &str,
        path: &str,
    ) -> Result<String> {
        let endpoint = source_path(workspace, repo_slug, git_ref, path);
        self.get_text(&endpoint, None).await
    }

    /// List the commits that touched the file at `path`, starting from
    /// `git_ref`, newest first.
    pub async fn get_file_history(
        &self,
        workspace: &str,
        repo_slug: &str,
        git_ref: &str,
        path: &str,
        pagelen: Option<u32>,
    ) -> Result<Paginated<FileHistoryEntry>> {
        let endpoint = format!(
            "/repositories/{}/{}/filehistory/{}/{}",
            workspace,
            repo_slug,
            urlencode_segment(git_ref),
            urlencode_path(path)
        );

        let mut params = Vec::new();
        if let Some(len) = pagelen {
            params.push(("pagelen", len.to_string()));
        }

        let param_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();
        self.get_with_query(&endpoint, &param_refs).await
    }
}

/// Build the `src/{ref}/{path}` endpoint path. An empty `path` targets the
/// repository root; the trailing slash tells Bitbucket to return a directory
/// listing rather than a ref redirect.
fn source_path(workspace: &str, repo_slug: &str, git_ref: &str, path: &str) -> String {
    let base = format!(
        "/repositories/{}/{}/src/{}",
        workspace,
        repo_slug,
        urlencode_segment(git_ref)
    );

    if path.is_empty() {
        format!("{}/", base)
    } else {
        format!("{}/{}", base, urlencode_path(path))
    }
}

/// Percent-encode a repository file path: each component is encoded as a
/// path segment while the `/` separators are preserved.
fn urlencode_path(path: &str) -> String {
    path.split('/')
        .map(urlencode_segment)
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urlencode_path_preserves_separators() {
        assert_eq!(urlencode_path("src/api/mod.rs"), "src/api/mod.rs");
    }

    #[test]
    fn urlencode_path_encodes_specials_within_segments() {
        assert_eq!(
            urlencode_path("docs/release notes/v1 (draft).md"),
            "docs/release%20notes/v1%20%28draft%29.md"
        );
    }

    #[test]
    fn source_path_uses_trailing_slash_for_root() {
        assert_eq!(
            source_path("acme", "widgets", "main", ""),
            "/repositories/acme/widgets/src/main/"
        );
    }

    #[test]
    fn source_path_appends_encoded_file_path() {
        assert_eq!(
            source_path("acme", "widgets", "feature/x", "src/lib.rs"),
            "/repositories/acme/widgets/src/feature%2Fx/src/lib.rs"
        );
    }
}
