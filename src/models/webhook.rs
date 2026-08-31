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
    /// Whether a signing secret is configured on this hook. Bitbucket returns
    /// this flag on reads but never the secret value itself.
    #[serde(default)]
    pub secret_set: Option<bool>,
}

/// How a webhook subscription's signing secret should be handled when sending
/// a create/replace body to Bitbucket.
///
/// Bitbucket never returns the plaintext secret on reads, so the update flow
/// (which does a GET-then-PUT merge) must distinguish "leave the existing
/// secret untouched" from "clear it". Omitting the `secret` field entirely
/// preserves the current secret; sending `null` clears it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum SecretUpdate {
    /// Do not send the `secret` field; keep whatever secret is configured.
    #[default]
    Unchanged,
    /// Send `"secret": null` to remove the configured secret.
    Clear,
    /// Send `"secret": "<value>"` to set/replace the secret.
    Set(String),
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
    /// Signing secret handling. Serialized as follows:
    /// - `Unchanged` -> field omitted (keeps the existing secret)
    /// - `Clear`     -> `"secret": null` (removes the secret)
    /// - `Set(v)`    -> `"secret": "v"` (sets/replaces the secret)
    #[serde(
        default,
        skip_deserializing,
        skip_serializing_if = "SecretUpdate::is_unchanged",
        serialize_with = "serialize_secret"
    )]
    pub secret: SecretUpdate,
}

impl SecretUpdate {
    fn is_unchanged(&self) -> bool {
        matches!(self, SecretUpdate::Unchanged)
    }
}

/// Serialize the `secret` field. Only called when the value is not
/// `Unchanged` (that case is skipped via `skip_serializing_if`), so this maps
/// `Clear` to JSON `null` and `Set(v)` to the string value.
fn serialize_secret<S>(secret: &SecretUpdate, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match secret {
        SecretUpdate::Clear => serializer.serialize_none(),
        SecretUpdate::Set(value) => serializer.serialize_some(value),
        // Unreachable because `skip_serializing_if` filters this variant out
        // before serialization. Keep the match total (serialize as null rather
        // than panicking in a serializer) but assert the invariant in debug.
        SecretUpdate::Unchanged => {
            debug_assert!(false, "Unchanged should be skipped by skip_serializing_if");
            serializer.serialize_none()
        }
    }
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
    use super::{CreateWebhookRequest, SecretUpdate, Webhook};

    #[test]
    fn create_webhook_request_omits_unset_description() {
        let request = CreateWebhookRequest {
            description: None,
            url: "https://example.com/hook".to_string(),
            active: true,
            events: vec!["repo:push".to_string()],
            secret: SecretUpdate::Unchanged,
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
            secret: SecretUpdate::Unchanged,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["description"], "CI trigger");
        assert_eq!(json["active"], false);
    }

    #[test]
    fn create_webhook_request_omits_secret_when_unchanged() {
        let request = CreateWebhookRequest {
            description: None,
            url: "https://example.com/hook".to_string(),
            active: true,
            events: vec!["repo:push".to_string()],
            secret: SecretUpdate::Unchanged,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert!(
            json.get("secret").is_none(),
            "secret must be omitted when unchanged, got {json}"
        );
    }

    #[test]
    fn create_webhook_request_sends_secret_string_when_set() {
        let request = CreateWebhookRequest {
            description: None,
            url: "https://example.com/hook".to_string(),
            active: true,
            events: vec!["repo:push".to_string()],
            secret: SecretUpdate::Set("s3cr3t".to_string()),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["secret"], "s3cr3t");
    }

    #[test]
    fn create_webhook_request_sends_null_secret_when_cleared() {
        let request = CreateWebhookRequest {
            description: None,
            url: "https://example.com/hook".to_string(),
            active: true,
            events: vec!["repo:push".to_string()],
            secret: SecretUpdate::Clear,
        };

        let json = serde_json::to_value(&request).unwrap();
        assert!(
            json.get("secret").is_some(),
            "cleared secret must send the field, got {json}"
        );
        assert!(
            json["secret"].is_null(),
            "cleared secret must serialize to null, got {}",
            json["secret"]
        );
    }

    #[test]
    fn webhook_deserializes_without_events() {
        let webhook: Webhook =
            serde_json::from_str(r#"{ "url": "https://example.com/hook" }"#).unwrap();
        assert!(webhook.events.is_empty());
        assert_eq!(webhook.uuid, None);
        assert_eq!(webhook.secret_set, None);
    }

    #[test]
    fn webhook_deserializes_secret_set_flag() {
        let webhook: Webhook =
            serde_json::from_str(r#"{ "url": "https://example.com/hook", "secret_set": true }"#)
                .unwrap();
        assert_eq!(webhook.secret_set, Some(true));
    }

    #[test]
    fn webhook_never_reexposes_returned_secret() {
        // Bitbucket create/update responses may echo a plaintext `secret`.
        // The read model must drop it so it can never leak back out through
        // `--output json`. This guards against a future `#[serde(flatten)]`
        // catch-all silently re-surfacing the value.
        let webhook: Webhook = serde_json::from_str(
            r#"{ "url": "https://example.com/hook", "secret": "leaked", "secret_set": true }"#,
        )
        .unwrap();

        let json = serde_json::to_value(&webhook).unwrap();
        assert!(
            json.get("secret").is_none(),
            "the plaintext secret must never be re-serialized, got {json}"
        );
    }
}
