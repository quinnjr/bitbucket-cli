//! Models for refs (branches and tags), commits, and source browsing.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// The refs/commits API returns the same author shape as the pull request
// commit payload; reuse it instead of redefining a colliding `CommitAuthor`
// (models are glob re-exported from `crate::models`).
use super::pr::CommitAuthor;

/// A branch ref as returned by the `refs/branches` API, including its target
/// commit. Distinct from [`super::repo::Branch`], which is the bare
/// `{ name }` shape embedded in repository payloads.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchDetail {
    pub name: String,
    pub target: Option<CommitSummary>,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub ref_type: Option<String>,
}

/// A tag ref as returned by the `refs/tags` API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagDetail {
    pub name: String,
    /// Annotation message, for annotated tags.
    pub message: Option<String>,
    pub target: Option<CommitSummary>,
    /// Tag creation date, for annotated tags.
    pub date: Option<DateTime<Utc>>,
}

/// A commit as returned by the `commits` / `commit/{hash}` endpoints and
/// embedded as the `target` of branch and tag refs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitSummary {
    pub hash: String,
    pub message: Option<String>,
    pub date: Option<DateTime<Utc>>,
    pub author: Option<CommitAuthor>,
}

/// A file or directory entry from the `src/{ref}/{path}` listing endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEntry {
    pub path: String,
    /// `commit_file` or `commit_directory`.
    #[serde(rename = "type")]
    pub entry_type: String,
    /// File size in bytes; absent for directories.
    pub size: Option<u64>,
}

/// One entry from the `filehistory/{ref}/{path}` endpoint: a commit that
/// touched the file, plus the path the file had at that commit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileHistoryEntry {
    pub commit: Option<CommitSummary>,
    pub path: Option<String>,
}

/// The `target` object of a ref-creation request: the commit to point at.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefTarget {
    pub hash: String,
}

/// Request body for `POST refs/branches`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CreateBranchRequest {
    pub name: String,
    pub target: RefTarget,
}

/// Request body for `POST refs/tags`. `message` is omitted from the payload
/// when unset so Bitbucket creates a lightweight tag.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CreateTagRequest {
    pub name: String,
    pub target: RefTarget,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_branch_request_serializes_name_and_target_hash() {
        let request = CreateBranchRequest {
            name: "feature/login".to_string(),
            target: RefTarget {
                hash: "a1b2c3d".to_string(),
            },
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "name": "feature/login",
                "target": { "hash": "a1b2c3d" }
            })
        );
    }

    #[test]
    fn create_tag_request_omits_unset_message() {
        let request = CreateTagRequest {
            name: "v1.0.0".to_string(),
            target: RefTarget {
                hash: "a1b2c3d".to_string(),
            },
            message: None,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "name": "v1.0.0",
                "target": { "hash": "a1b2c3d" }
            })
        );
    }

    #[test]
    fn create_tag_request_includes_message_when_set() {
        let request = CreateTagRequest {
            name: "v1.0.0".to_string(),
            target: RefTarget {
                hash: "a1b2c3d".to_string(),
            },
            message: Some("First release".to_string()),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["message"], serde_json::json!("First release"));
    }

    #[test]
    fn branch_detail_maps_type_field() {
        let branch: BranchDetail = serde_json::from_str(
            r#"{
                "name": "main",
                "type": "branch",
                "target": {
                    "hash": "deadbeefcafe",
                    "message": "Initial commit\n",
                    "date": "2024-01-01T00:00:00+00:00",
                    "author": { "raw": "Jo <jo@example.com>" }
                }
            }"#,
        )
        .unwrap();

        assert_eq!(branch.ref_type.as_deref(), Some("branch"));
        let target = branch.target.unwrap();
        assert_eq!(target.hash, "deadbeefcafe");
        assert_eq!(
            target.author.unwrap().raw.as_deref(),
            Some("Jo <jo@example.com>")
        );
    }

    #[test]
    fn source_entry_maps_type_field() {
        let entry: SourceEntry = serde_json::from_str(
            r#"{ "path": "src/main.rs", "type": "commit_file", "size": 1024 }"#,
        )
        .unwrap();

        assert_eq!(entry.entry_type, "commit_file");
        assert_eq!(entry.size, Some(1024));
    }
}
