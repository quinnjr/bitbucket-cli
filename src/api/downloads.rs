use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use reqwest::multipart::{Form, Part};

use super::BitbucketClient;
use crate::models::{Download, Paginated};

impl BitbucketClient {
    /// Upload one or more files to a repository's downloads area.
    ///
    /// Uploading a file whose name matches an existing artifact replaces it.
    pub async fn upload_downloads(
        &self,
        workspace: &str,
        repo_slug: &str,
        files: &[(String, PathBuf)],
    ) -> Result<()> {
        let mut form = Form::new();

        for (upload_name, path) in files {
            let bytes = std::fs::read(path)
                .with_context(|| format!("Failed to read file '{}'", path.display()))?;
            let part = Part::bytes(bytes).file_name(upload_name.clone());
            // Bitbucket expects every file under the "files" field.
            form = form.part("files", part);
        }

        let path = format!("/repositories/{}/{}/downloads", workspace, repo_slug);
        self.post_multipart(&path, form).await
    }

    /// List the artifacts in a repository's downloads area.
    pub async fn list_downloads(
        &self,
        workspace: &str,
        repo_slug: &str,
    ) -> Result<Paginated<Download>> {
        let path = format!("/repositories/{}/{}/downloads", workspace, repo_slug);
        self.get(&path).await
    }

    /// Delete a single artifact from a repository's downloads area.
    pub async fn delete_download(
        &self,
        workspace: &str,
        repo_slug: &str,
        filename: &str,
    ) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/downloads/{}",
            workspace,
            repo_slug,
            urlencode_segment(filename)
        );
        self.delete(&path).await
    }
}

/// Build the public, browser-facing URL for a download artifact.
///
/// This is the URL to embed in markdown (e.g. `![alt](url)`); it is rendered
/// for anyone with read access to the repository.
pub fn download_url(workspace: &str, repo_slug: &str, name: &str) -> String {
    format!(
        "https://bitbucket.org/{}/{}/downloads/{}",
        workspace,
        repo_slug,
        urlencode_segment(name)
    )
}

/// Extract the file name (final path component) from a path, for use as the
/// uploaded artifact name.
pub fn upload_name_for(path: &Path) -> Result<String> {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|s| s.to_string())
        .with_context(|| format!("Could not determine a file name for '{}'", path.display()))
}

/// Percent-encode the characters that are unsafe in a single URL path segment.
/// Intentionally minimal — file names are mostly `[A-Za-z0-9._-]` plus spaces.
fn urlencode_segment(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_url_is_browser_facing_host() {
        assert_eq!(
            download_url("acme", "widgets", "shot.png"),
            "https://bitbucket.org/acme/widgets/downloads/shot.png"
        );
    }

    #[test]
    fn download_url_encodes_spaces_and_specials() {
        assert_eq!(
            download_url("acme", "widgets", "my shot (1).png"),
            "https://bitbucket.org/acme/widgets/downloads/my%20shot%20%281%29.png"
        );
    }

    #[test]
    fn upload_name_takes_final_component() {
        let p = PathBuf::from("/tmp/screenshots/login.png");
        assert_eq!(upload_name_for(&p).unwrap(), "login.png");
    }

    #[test]
    fn urlencode_leaves_safe_chars_untouched() {
        assert_eq!(urlencode_segment("a-b_c.d~1.png"), "a-b_c.d~1.png");
    }
}
