use serde::{Deserialize, Serialize};

use super::user::User;

/// A repository webhook subscription.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Webhook {
    /// Server-assigned identifier, a UUID wrapped in braces (e.g. `{1a2b-...}`).
    pub uuid: Option<String>,
    /// Destination URL the events are delivered to.
    pub url: String,
    pub description: Option<String>,
    pub active: Option<bool>,
    /// Event keys this hook subscribes to (e.g. `repo:push`).
    #[serde(default)]
    pub events: Vec<String>,
}

/// Body for creating (POST) or replacing (PUT) a webhook subscription.
///
/// Bitbucket's PUT replaces the whole subscription, so updates send a full
/// body built by merging the current hook with the changed fields.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct CreateWebhookRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub url: String,
    pub active: bool,
    pub events: Vec<String>,
}

/// A branch restriction rule on a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchRestriction {
    pub id: Option<u64>,
    /// Rule kind (e.g. `push`, `force`, `delete`, `require_approvals_to_merge`).
    pub kind: String,
    /// Branch pattern the rule applies to (glob, e.g. `main` or `release/*`).
    pub pattern: Option<String>,
    /// How `pattern` is interpreted (`glob` or `branching_model`).
    pub branch_match_kind: Option<String>,
    /// Numeric parameter for kinds that need one (e.g. required approval count).
    pub value: Option<u64>,
    /// Users exempt from the rule (for kinds like `push`).
    pub users: Option<Vec<User>>,
}

/// A user's explicit permission on a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPermission {
    pub user: Option<User>,
    /// Permission level: `read`, `write`, or `admin`.
    pub permission: String,
}

/// A group's explicit permission on a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupPermission {
    pub group: Option<GroupRef>,
    /// Permission level: `read`, `write`, or `admin`.
    pub permission: String,
}

/// Minimal reference to a workspace group as embedded in permission entries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRef {
    pub name: Option<String>,
    pub slug: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::{CreateWebhookRequest, Webhook};

    #[test]
    fn create_webhook_request_omits_unset_description() {
        let request = CreateWebhookRequest {
            description: None,
            url: "https://example.com/hook".to_string(),
            active: true,
            events: vec!["repo:push".to_string()],
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "url": "https://example.com/hook",
                "active": true,
                "events": ["repo:push"],
            })
        );
    }

    #[test]
    fn create_webhook_request_includes_description_when_set() {
        let request = CreateWebhookRequest {
            description: Some("CI trigger".to_string()),
            url: "https://example.com/hook".to_string(),
            active: false,
            events: vec![],
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["description"], "CI trigger");
        assert_eq!(json["active"], false);
    }

    #[test]
    fn webhook_deserializes_without_events() {
        let webhook: Webhook =
            serde_json::from_str(r#"{ "url": "https://example.com/hook" }"#).unwrap();
        assert!(webhook.events.is_empty());
        assert_eq!(webhook.uuid, None);
    }
}
