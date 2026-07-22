use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::user::{User, Workspace};

/// A Bitbucket project as returned by the workspace projects endpoints.
///
/// Named `ProjectDetail` to avoid colliding with the lightweight
/// [`Project`](super::repo::Project) reference embedded in repositories.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectDetail {
    pub uuid: Option<String>,
    pub key: String,
    pub name: String,
    pub description: Option<String>,
    pub is_private: Option<bool>,
    pub created_on: Option<DateTime<Utc>>,
    pub updated_on: Option<DateTime<Utc>>,
}

/// Body for creating a new project in a workspace.
#[derive(Debug, Clone, Serialize)]
#[non_exhaustive]
pub struct CreateProjectRequest {
    pub name: String,
    pub key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_private: Option<bool>,
}

/// Partial update for an existing project. Unset fields are omitted from the
/// request body so the corresponding settings are left untouched.
#[derive(Debug, Clone, Default, Serialize)]
#[non_exhaustive]
pub struct UpdateProjectRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_private: Option<bool>,
}

impl UpdateProjectRequest {
    /// True when no field is set, i.e. the update would be a no-op.
    pub fn is_empty(&self) -> bool {
        self.name.is_none() && self.description.is_none() && self.is_private.is_none()
    }
}

/// A user's membership in a workspace, as returned by
/// `GET /workspaces/{workspace}/members`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceMembership {
    pub user: Option<User>,
    pub workspace: Option<Workspace>,
}

/// A permission grant on a workspace (or one of its repositories), as
/// returned by `GET /workspaces/{workspace}/permissions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacePermission {
    pub user: Option<User>,
    pub permission: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{CreateProjectRequest, ProjectDetail, UpdateProjectRequest};

    #[test]
    fn project_detail_deserializes_with_minimal_fields() {
        let payload = r#"{ "key": "PROJ", "name": "My Project" }"#;
        let project: ProjectDetail = serde_json::from_str(payload).unwrap();
        assert_eq!(project.key, "PROJ");
        assert_eq!(project.name, "My Project");
        assert!(project.uuid.is_none());
        assert!(project.is_private.is_none());
    }

    #[test]
    fn create_request_omits_unset_options() {
        let request = CreateProjectRequest {
            name: "My Project".to_string(),
            key: "PROJ".to_string(),
            description: None,
            is_private: Some(true),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "name": "My Project", "key": "PROJ", "is_private": true })
        );
    }

    #[test]
    fn update_request_omits_unset_fields() {
        let request = UpdateProjectRequest {
            description: Some("new description".to_string()),
            ..Default::default()
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            json,
            serde_json::json!({ "description": "new description" })
        );
    }

    #[test]
    fn update_request_is_empty() {
        assert!(UpdateProjectRequest::default().is_empty());

        assert!(
            !UpdateProjectRequest {
                name: Some("Renamed".to_string()),
                ..Default::default()
            }
            .is_empty()
        );
        assert!(
            !UpdateProjectRequest {
                description: Some(String::new()),
                ..Default::default()
            }
            .is_empty()
        );
        assert!(
            !UpdateProjectRequest {
                is_private: Some(false),
                ..Default::default()
            }
            .is_empty()
        );
    }
}
