use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::user::{Link, User};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: u64,
    pub title: String,
    pub content: Option<IssueContent>,
    pub reporter: Option<User>,
    pub assignee: Option<User>,
    pub state: IssueState,
    pub kind: IssueKind,
    pub priority: IssuePriority,
    pub milestone: Option<Milestone>,
    pub component: Option<Component>,
    pub version: Option<Version>,
    pub votes: Option<u32>,
    pub watches: Option<u32>,
    pub created_on: DateTime<Utc>,
    pub updated_on: Option<DateTime<Utc>>,
    pub edited_on: Option<DateTime<Utc>>,
    pub links: Option<IssueLinks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueContent {
    pub raw: Option<String>,
    pub markup: Option<String>,
    pub html: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IssueState {
    New,
    Open,
    Resolved,
    #[serde(rename = "on hold")]
    OnHold,
    Invalid,
    Duplicate,
    Wontfix,
    Closed,
}

impl std::fmt::Display for IssueState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueState::New => write!(f, "new"),
            IssueState::Open => write!(f, "open"),
            IssueState::Resolved => write!(f, "resolved"),
            IssueState::OnHold => write!(f, "on hold"),
            IssueState::Invalid => write!(f, "invalid"),
            IssueState::Duplicate => write!(f, "duplicate"),
            IssueState::Wontfix => write!(f, "wontfix"),
            IssueState::Closed => write!(f, "closed"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IssueKind {
    Bug,
    Enhancement,
    Proposal,
    Task,
}

impl std::fmt::Display for IssueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueKind::Bug => write!(f, "bug"),
            IssueKind::Enhancement => write!(f, "enhancement"),
            IssueKind::Proposal => write!(f, "proposal"),
            IssueKind::Task => write!(f, "task"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum IssuePriority {
    Trivial,
    Minor,
    Major,
    Critical,
    Blocker,
}

impl std::fmt::Display for IssuePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssuePriority::Trivial => write!(f, "trivial"),
            IssuePriority::Minor => write!(f, "minor"),
            IssuePriority::Major => write!(f, "major"),
            IssuePriority::Critical => write!(f, "critical"),
            IssuePriority::Blocker => write!(f, "blocker"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueLinks {
    #[serde(rename = "self")]
    pub self_link: Option<Link>,
    pub html: Option<Link>,
    pub comments: Option<Link>,
    pub attachments: Option<Link>,
    pub watch: Option<Link>,
    pub vote: Option<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CreateIssueRequest {
    pub title: String,
    pub content: Option<IssueContentRequest>,
    pub kind: Option<IssueKind>,
    pub priority: Option<IssuePriority>,
    pub assignee: Option<UserAccountId>,
    pub component: Option<ComponentName>,
    pub milestone: Option<MilestoneName>,
    pub version: Option<VersionName>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueContentRequest {
    pub raw: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserAccountId {
    pub account_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentName {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneName {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionName {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueComment {
    pub id: u64,
    pub content: IssueContent,
    pub user: User,
    pub created_on: DateTime<Utc>,
    pub updated_on: Option<DateTime<Utc>>,
    pub links: Option<IssueCommentLinks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueCommentLinks {
    #[serde(rename = "self")]
    pub self_link: Option<Link>,
    pub html: Option<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateIssueCommentRequest {
    pub content: IssueContentRequest,
}

/// A single entry in an issue's change log (state transitions, reassignments,
/// priority bumps, ...).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueChange {
    pub id: Option<u64>,
    pub user: Option<User>,
    pub message: Option<IssueContent>,
    pub created_on: Option<DateTime<Utc>>,
    /// Map of changed field name to `{old, new}` values. Kept as raw JSON
    /// because the API emits arbitrary field names here.
    pub changes: Option<serde_json::Value>,
}

/// Shared shape for issue tracker metadata: components, milestones and
/// versions all serialize as an id plus a name.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueMetaItem {
    pub id: Option<u64>,
    pub name: Option<String>,
}

/// A file attached to an issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueAttachment {
    pub name: Option<String>,
    /// Kept as raw JSON: the API serves `links.self.href` for attachments as
    /// either a string or an array of strings.
    pub links: Option<serde_json::Value>,
}
