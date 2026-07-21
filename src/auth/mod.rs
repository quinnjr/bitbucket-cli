pub mod api_key;
pub mod credential_store;
pub mod file_store;
pub mod keyring_store;
pub mod oauth;

use anyhow::Result;
use base64::Engine;
use serde::{Deserialize, Serialize};

pub use api_key::*;
pub use credential_store::*;
pub use file_store::*;
pub use keyring_store::*;
pub use oauth::*;

/// Credential types for Bitbucket authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Credential {
    /// OAuth 2.0 credentials (preferred method)
    OAuth {
        access_token: String,
        refresh_token: Option<String>,
        expires_at: Option<i64>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        client_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        client_secret: Option<String>,
    },
    /// API Key / HTTP Access Token (for automation/CI)
    ApiKey { username: String, api_key: String },
}

impl Credential {
    /// Get the authorization header value for API requests
    #[inline]
    pub fn auth_header(&self) -> String {
        match self {
            Credential::OAuth { access_token, .. } => {
                let mut result = String::with_capacity(7 + access_token.len());
                result.push_str("Bearer ");
                result.push_str(access_token);
                result
            }
            Credential::ApiKey { username, api_key } => {
                let basic = format!("{}:{}", username, api_key);
                let encoded = base64::engine::general_purpose::STANDARD.encode(basic.as_bytes());
                let mut result = String::with_capacity(6 + encoded.len());
                result.push_str("Basic ");
                result.push_str(&encoded);
                result
            }
        }
    }

    /// Get the credential type name for display
    #[inline]
    pub fn type_name(&self) -> &'static str {
        match self {
            Credential::OAuth { .. } => "OAuth 2.0",
            Credential::ApiKey { .. } => "API Key",
        }
    }

    /// Check if the credential needs refresh
    #[inline]
    pub fn needs_refresh(&self) -> bool {
        match self {
            Credential::OAuth {
                expires_at: Some(expires),
                ..
            } => {
                // Refresh if expiring within 5 minutes (300 seconds)
                *expires < chrono::Utc::now().timestamp() + 300
            }
            _ => false,
        }
    }

    /// Get stored OAuth consumer credentials (client_id, client_secret)
    pub fn oauth_consumer_credentials(&self) -> Option<(&str, &str)> {
        match self {
            Credential::OAuth {
                client_id: Some(id),
                client_secret: Some(secret),
                ..
            } => Some((id, secret)),
            _ => None,
        }
    }

    /// Get username (only available for API key credentials)
    pub fn username(&self) -> Option<&str> {
        match self {
            Credential::ApiKey { username, .. } => Some(username),
            _ => None,
        }
    }
}

/// Authentication manager - uses the platform secret store with file fallback
pub struct AuthManager {
    store: CredentialStore,
}

impl AuthManager {
    pub fn new() -> Result<Self> {
        Ok(Self {
            store: CredentialStore::new()?,
        })
    }

    /// Get stored credentials
    pub fn get_credentials(&self) -> Result<Option<Credential>> {
        self.store.get_credential()
    }

    /// Store credentials
    pub fn store_credentials(&self, credential: &Credential) -> Result<()> {
        self.store.store_credential(credential)
    }

    /// Clear stored credentials
    pub fn clear_credentials(&self) -> Result<()> {
        self.store.delete_credential()
    }

    /// Check if authenticated
    pub fn is_authenticated(&self) -> bool {
        self.get_credentials().map(|c| c.is_some()).unwrap_or(false)
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new().expect("Failed to create auth manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn oauth_credential(expires_at: Option<i64>) -> Credential {
        Credential::OAuth {
            access_token: "test-access-token".to_string(),
            refresh_token: None,
            expires_at,
            client_id: None,
            client_secret: None,
        }
    }

    #[test]
    fn oauth_auth_header_is_bearer_token() {
        let credential = oauth_credential(None);
        assert_eq!(credential.auth_header(), "Bearer test-access-token");
    }

    #[test]
    fn api_key_auth_header_is_basic_base64() {
        let credential = Credential::ApiKey {
            username: "alice".to_string(),
            api_key: "s3cret-key".to_string(),
        };
        let expected = format!(
            "Basic {}",
            base64::engine::general_purpose::STANDARD.encode("alice:s3cret-key")
        );
        assert_eq!(credential.auth_header(), expected);
    }

    #[test]
    fn api_key_never_needs_refresh() {
        let credential = Credential::ApiKey {
            username: "alice".to_string(),
            api_key: "s3cret-key".to_string(),
        };
        assert!(!credential.needs_refresh());
    }

    #[test]
    fn oauth_without_expiry_never_needs_refresh() {
        let credential = oauth_credential(None);
        assert!(!credential.needs_refresh());
    }

    #[test]
    fn oauth_expiring_beyond_five_minutes_does_not_need_refresh() {
        // 10 minutes from now: well past the 5-minute refresh window
        let expires_at = chrono::Utc::now().timestamp() + 600;
        let credential = oauth_credential(Some(expires_at));
        assert!(!credential.needs_refresh());
    }

    #[test]
    fn oauth_expiring_within_five_minutes_needs_refresh() {
        // 1 minute from now: inside the 5-minute refresh window
        let expires_at = chrono::Utc::now().timestamp() + 60;
        let credential = oauth_credential(Some(expires_at));
        assert!(credential.needs_refresh());
    }

    #[test]
    fn oauth_already_expired_needs_refresh() {
        let expires_at = chrono::Utc::now().timestamp() - 600;
        let credential = oauth_credential(Some(expires_at));
        assert!(credential.needs_refresh());
    }
}
