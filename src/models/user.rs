use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub uuid: String,
    pub username: Option<String>,
    pub display_name: String,
    pub account_id: Option<String>,
    #[serde(rename = "type")]
    pub user_type: String,
    pub links: Option<UserLinks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserLinks {
    #[serde(rename = "self")]
    pub self_link: Option<Link>,
    pub html: Option<Link>,
    pub avatar: Option<Link>,
}

/// An email address associated with the authenticated user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserEmail {
    pub email: Option<String>,
    pub is_primary: Option<bool>,
    pub is_confirmed: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub href: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workspace {
    pub uuid: String,
    pub slug: String,
    pub name: String,
    #[serde(rename = "type")]
    pub workspace_type: String,
    pub is_private: Option<bool>,
    pub created_on: Option<DateTime<Utc>>,
    pub links: Option<WorkspaceLinks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceLinks {
    pub html: Option<Link>,
    pub avatar: Option<Link>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Paginated<T> {
    pub size: Option<u32>,
    pub page: Option<u32>,
    pub pagelen: Option<u32>,
    pub next: Option<String>,
    pub previous: Option<String>,
    pub values: Vec<T>,
}
