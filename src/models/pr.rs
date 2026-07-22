use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::repo::Repository;
use super::user::{Link, User};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequest {
    pub id: u64,
    pub title: String,
    pub description: Option<String>,
    pub state: PullRequestState,
    pub author: User,
    pub source: PullRequestEndpoint,
    pub destination: PullRequestEndpoint,
    pub merge_commit: Option<Commit>,
    pub close_source_branch: Option<bool>,
    pub closed_by: Option<User>,
    pub reason: Option<String>,
    pub created_on: DateTime<Utc>,
    pub updated_on: DateTime<Utc>,
    pub reviewers: Option<Vec<User>>,
    pub participants: Option<Vec<Participant>>,
    pub links: Option<PullRequestLinks>,
    pub comment_count: Option<u32>,
    pub task_count: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum PullRequestState {
    Open,
    Merged,
    Declined,
    Superseded,
}

impl std::fmt::Display for PullRequestState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PullRequestState::Open => write!(f, "OPEN"),
            PullRequestState::Merged => write!(f, "MERGED"),
            PullRequestState::Declined => write!(f, "DECLINED"),
            PullRequestState::Superseded => write!(f, "SUPERSEDED"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestEndpoint {
    pub branch: BranchInfo,
    pub commit: Option<CommitInfo>,
    pub repository: Option<Repository>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchInfo {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitInfo {
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Commit {
    pub hash: String,
    pub message: Option<String>,
    pub author: Option<CommitAuthor>,
    pub date: Option<DateTime<Utc>>,
    pub links: Option<CommitLinks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitAuthor {
    pub raw: Option<String>,
    pub user: Option<User>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitLinks {
    pub html: Option<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub user: User,
    pub role: ParticipantRole,
    pub approved: bool,
    pub state: Option<ParticipantState>,
    pub participated_on: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum ParticipantRole {
    Participant,
    Reviewer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ParticipantState {
    Approved,
    ChangesRequested,
    #[serde(other)]
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestLinks {
    #[serde(rename = "self")]
    pub self_link: Option<Link>,
    pub html: Option<Link>,
    pub commits: Option<Link>,
    pub approve: Option<Link>,
    pub diff: Option<Link>,
    pub diffstat: Option<Link>,
    pub comments: Option<Link>,
    pub activity: Option<Link>,
    pub merge: Option<Link>,
    pub decline: Option<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CreatePullRequestRequest {
    pub title: String,
    pub source: PullRequestBranchRef,
    pub destination: Option<PullRequestBranchRef>,
    pub description: Option<String>,
    pub close_source_branch: Option<bool>,
    pub reviewers: Option<Vec<UserRef>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestBranchRef {
    pub branch: BranchInfo,
}

/// Lightweight user reference used in request bodies (e.g. reviewers).
///
/// Post-GDPR Bitbucket identifies users by `uuid` (or account ID) and ignores
/// `username`, so prefer setting `uuid`. Unset fields are omitted from the
/// serialized JSON.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRef {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
}

/// Request body for `PUT .../pullrequests/{id}`; every field is optional and
/// omitted from the JSON when unset, so only the provided fields change.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[non_exhaustive]
pub struct UpdatePullRequestRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewers: Option<Vec<UserRef>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination: Option<PullRequestBranchRef>,
}

impl UpdatePullRequestRequest {
    /// True when no field is set, i.e. the update would change nothing.
    pub fn is_empty(&self) -> bool {
        self.title.is_none()
            && self.description.is_none()
            && self.reviewers.is_none()
            && self.destination.is_none()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct MergePullRequestRequest {
    #[serde(rename = "type")]
    pub merge_type: Option<String>,
    pub message: Option<String>,
    pub close_source_branch: Option<bool>,
    pub merge_strategy: Option<MergeStrategy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MergeStrategy {
    MergeCommit,
    Squash,
    FastForward,
}

impl Default for MergePullRequestRequest {
    fn default() -> Self {
        Self {
            merge_type: Some("pullrequest".to_string()),
            message: None,
            close_source_branch: Some(true),
            merge_strategy: Some(MergeStrategy::MergeCommit),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PullRequestComment {
    pub id: u64,
    pub content: CommentContent,
    pub user: User,
    pub created_on: DateTime<Utc>,
    pub updated_on: Option<DateTime<Utc>>,
    pub deleted: Option<bool>,
    pub inline: Option<InlineComment>,
    pub parent: Option<CommentRef>,
    pub links: Option<CommentLinks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentContent {
    pub raw: String,
    pub markup: Option<String>,
    pub html: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineComment {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<u32>,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentRef {
    pub id: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommentLinks {
    #[serde(rename = "self")]
    pub self_link: Option<Link>,
    pub html: Option<Link>,
}

/// A commit as returned by `GET .../pullrequests/{id}/commits`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrCommit {
    pub hash: String,
    pub message: Option<String>,
    pub date: Option<DateTime<Utc>>,
}

/// A build/commit status as returned by `GET .../pullrequests/{id}/statuses`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommitStatus {
    pub key: Option<String>,
    pub name: Option<String>,
    pub state: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
}

/// A per-file entry as returned by `GET .../pullrequests/{id}/diffstat`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStat {
    pub status: Option<String>,
    pub lines_added: Option<u64>,
    pub lines_removed: Option<u64>,
    pub old: Option<DiffStatFile>,
    pub new: Option<DiffStatFile>,
}

/// File reference inside a [`DiffStat`] entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStatFile {
    pub path: Option<String>,
}

/// A task on a pull request (`.../pullrequests/{id}/tasks`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrTask {
    pub id: Option<u64>,
    pub state: Option<String>,
    pub content: Option<PrTaskContent>,
}

/// Content of a [`PrTask`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrTaskContent {
    pub raw: Option<String>,
}

/// One entry from `GET .../pullrequests/{id}/activity`.
///
/// The activity feed mixes several event shapes, so each variant is kept as
/// loose JSON; exactly one of the fields is normally populated per entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrActivity {
    pub update: Option<serde_json::Value>,
    pub approval: Option<serde_json::Value>,
    pub comment: Option<serde_json::Value>,
    pub changes_requested: Option<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::{
        BranchInfo, InlineComment, PullRequestBranchRef, UpdatePullRequestRequest, UserRef,
    };

    #[test]
    fn update_request_is_empty_when_no_field_is_set() {
        assert!(UpdatePullRequestRequest::default().is_empty());
    }

    #[test]
    fn update_request_is_not_empty_when_any_field_is_set() {
        let with_title = UpdatePullRequestRequest {
            title: Some("t".into()),
            ..Default::default()
        };
        let with_description = UpdatePullRequestRequest {
            description: Some("d".into()),
            ..Default::default()
        };
        let with_reviewers = UpdatePullRequestRequest {
            reviewers: Some(vec![UserRef {
                uuid: None,
                username: Some("alice".into()),
            }]),
            ..Default::default()
        };
        let with_destination = UpdatePullRequestRequest {
            destination: Some(PullRequestBranchRef {
                branch: BranchInfo {
                    name: "main".into(),
                },
            }),
            ..Default::default()
        };

        assert!(!with_title.is_empty());
        assert!(!with_description.is_empty());
        assert!(!with_reviewers.is_empty());
        assert!(!with_destination.is_empty());
    }

    #[test]
    fn update_request_serializes_only_set_fields() {
        let request = UpdatePullRequestRequest {
            title: Some("New title".into()),
            destination: Some(PullRequestBranchRef {
                branch: BranchInfo {
                    name: "develop".into(),
                },
            }),
            ..Default::default()
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "title": "New title",
                "destination": { "branch": { "name": "develop" } }
            })
        );
    }

    #[test]
    fn inline_comment_omits_from_when_none() {
        let inline = InlineComment {
            from: None,
            to: Some(3),
            path: "src/main.rs".into(),
        };

        let json = serde_json::to_value(&inline).unwrap();
        assert_eq!(json, serde_json::json!({ "to": 3, "path": "src/main.rs" }));
        assert!(json.get("from").is_none());
    }

    #[test]
    fn user_ref_with_only_uuid_omits_username() {
        let user = UserRef {
            uuid: Some("{account-uuid}".into()),
            username: None,
        };

        let json = serde_json::to_value(&user).unwrap();
        assert_eq!(json, serde_json::json!({ "uuid": "{account-uuid}" }));
        assert!(json.get("username").is_none());
    }
}
