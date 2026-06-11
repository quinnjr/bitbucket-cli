use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::user::{Link, User, Workspace};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub uuid: String,
    pub name: String,
    pub full_name: String,
    pub slug: Option<String>,
    pub description: Option<String>,
    pub is_private: Option<bool>,
    pub scm: Option<String>,
    pub owner: Option<User>,
    pub workspace: Option<Workspace>,
    pub project: Option<Project>,
    pub created_on: Option<DateTime<Utc>>,
    pub updated_on: Option<DateTime<Utc>>,
    pub size: Option<u64>,
    pub language: Option<String>,
    pub has_issues: Option<bool>,
    pub has_wiki: Option<bool>,
    pub fork_policy: Option<String>,
    pub mainbranch: Option<Branch>,
    pub links: Option<RepositoryLinks>,
    #[serde(rename = "type")]
    pub repo_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryLinks {
    #[serde(rename = "self")]
    pub self_link: Option<Link>,
    pub html: Option<Link>,
    pub avatar: Option<Link>,
    pub clone: Option<Vec<CloneLink>>,
    pub pullrequests: Option<Link>,
    pub commits: Option<Link>,
    pub forks: Option<Link>,
    pub watchers: Option<Link>,
    pub branches: Option<Link>,
    pub tags: Option<Link>,
    pub downloads: Option<Link>,
    pub source: Option<Link>,
    pub issues: Option<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloneLink {
    pub href: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Branch {
    pub name: String,
    #[serde(rename = "type")]
    pub branch_type: Option<String>,
}

/// A repository download artifact (the "Downloads" area of a repo).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Download {
    pub name: String,
    pub size: Option<u64>,
    /// Number of times this artifact has been downloaded.
    pub downloads: Option<u64>,
    pub links: Option<DownloadLinks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadLinks {
    #[serde(rename = "self")]
    pub self_link: Option<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub uuid: String,
    pub key: String,
    pub name: String,
    #[serde(rename = "type")]
    pub project_type: String,
    pub links: Option<ProjectLinks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectLinks {
    pub html: Option<Link>,
    pub avatar: Option<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateRepositoryRequest {
    pub scm: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub is_private: Option<bool>,
    pub project: Option<ProjectKey>,
    pub fork_policy: Option<String>,
    pub language: Option<String>,
    pub has_issues: Option<bool>,
    pub has_wiki: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectKey {
    pub key: String,
}

impl Default for CreateRepositoryRequest {
    fn default() -> Self {
        Self {
            scm: "git".to_string(),
            name: None,
            description: None,
            is_private: Some(true),
            project: None,
            fork_policy: Some("no_public_forks".to_string()),
            language: None,
            has_issues: Some(true),
            has_wiki: Some(false),
        }
    }
}
