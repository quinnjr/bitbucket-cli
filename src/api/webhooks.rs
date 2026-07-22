use anyhow::Result;

use super::BitbucketClient;
use crate::models::{CreateWebhookRequest, Paginated, Webhook};

/// Percent-encode a string for use as a single URL path segment.
///
/// Bitbucket identifiers such as webhook UIDs (a UUID wrapped in braces,
/// e.g. `{1a2b-...}`) and permission selectors (`{uuid}`, account IDs with
/// `:`) contain characters that are not valid raw path characters, so
/// everything outside the RFC 3986 "unreserved" set is percent-encoded.
pub(crate) fn encode_path_segment(segment: &str) -> String {
    let mut encoded = String::with_capacity(segment.len());
    for byte in segment.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                encoded.push(byte as char);
            }
            _ => encoded.push_str(&format!("%{:02X}", byte)),
        }
    }
    encoded
}

impl BitbucketClient {
    /// List webhook subscriptions on a repository
    pub async fn list_webhooks(
        &self,
        workspace: &str,
        repo_slug: &str,
        pagelen: Option<u32>,
    ) -> Result<Paginated<Webhook>> {
        let path = format!("/repositories/{}/{}/hooks", workspace, repo_slug);

        match pagelen {
            Some(len) => {
                self.get_with_query(&path, &[("pagelen", len.to_string().as_str())])
                    .await
            }
            None => self.get(&path).await,
        }
    }

    /// Create a webhook subscription on a repository
    pub async fn create_webhook(
        &self,
        workspace: &str,
        repo_slug: &str,
        request: &CreateWebhookRequest,
    ) -> Result<Webhook> {
        let path = format!("/repositories/{}/{}/hooks", workspace, repo_slug);
        self.post(&path, request).await
    }

    /// Get a single webhook subscription by its UID (the brace-wrapped UUID)
    pub async fn get_webhook(
        &self,
        workspace: &str,
        repo_slug: &str,
        uid: &str,
    ) -> Result<Webhook> {
        let path = format!(
            "/repositories/{}/{}/hooks/{}",
            workspace,
            repo_slug,
            encode_path_segment(uid)
        );
        self.get(&path).await
    }

    /// Replace a webhook subscription.
    ///
    /// Bitbucket's PUT replaces the entire subscription, so `request` must
    /// carry the full desired state (see [`CreateWebhookRequest`]).
    pub async fn update_webhook(
        &self,
        workspace: &str,
        repo_slug: &str,
        uid: &str,
        request: &CreateWebhookRequest,
    ) -> Result<Webhook> {
        let path = format!(
            "/repositories/{}/{}/hooks/{}",
            workspace,
            repo_slug,
            encode_path_segment(uid)
        );
        self.put(&path, request).await
    }

    /// Delete a webhook subscription
    pub async fn delete_webhook(&self, workspace: &str, repo_slug: &str, uid: &str) -> Result<()> {
        let path = format!(
            "/repositories/{}/{}/hooks/{}",
            workspace,
            repo_slug,
            encode_path_segment(uid)
        );
        self.delete(&path).await
    }
}

#[cfg(test)]
mod tests {
    use super::encode_path_segment;

    #[test]
    fn encode_path_segment_passes_unreserved_through() {
        assert_eq!(
            encode_path_segment("abc-XYZ_0.9~"),
            "abc-XYZ_0.9~".to_string()
        );
    }

    #[test]
    fn encode_path_segment_encodes_webhook_uid_braces() {
        assert_eq!(
            encode_path_segment("{d3a1e9b0-1234-5678-9abc-def012345678}"),
            "%7Bd3a1e9b0-1234-5678-9abc-def012345678%7D"
        );
    }

    #[test]
    fn encode_path_segment_encodes_reserved_ascii() {
        assert_eq!(encode_path_segment("a/b:c d"), "a%2Fb%3Ac%20d");
    }

    #[test]
    fn encode_path_segment_encodes_utf8_bytes() {
        assert_eq!(encode_path_segment("é"), "%C3%A9");
    }
}
