use anyhow::Result;

use super::BitbucketClient;
use crate::api::util::urlencode_segment;
use crate::models::{Paginated, User, UserEmail};

impl BitbucketClient {
    /// Fetch the profile of the currently authenticated user.
    pub async fn get_current_user(&self) -> Result<User> {
        self.get("/user").await
    }

    /// Fetch a user's public profile.
    ///
    /// `selected` may be an account ID or a UUID (including surrounding
    /// braces, which are percent-encoded for the URL path).
    pub async fn get_user(&self, selected: &str) -> Result<User> {
        let path = format!("/users/{}", urlencode_segment(selected));
        self.get(&path).await
    }

    /// List the email addresses associated with the authenticated user.
    pub async fn list_user_emails(&self) -> Result<Paginated<UserEmail>> {
        self.get("/user/emails").await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Paginated, UserEmail};

    #[test]
    fn urlencode_encodes_uuid_braces() {
        assert_eq!(
            urlencode_segment("{a1b2c3d4-0000-1111-2222-333344445555}"),
            "%7Ba1b2c3d4-0000-1111-2222-333344445555%7D"
        );
    }

    #[test]
    fn urlencode_leaves_account_ids_untouched() {
        assert_eq!(
            urlencode_segment("557058:c0b72ad0-1234-5678-9abc-def012345678"),
            "557058%3Ac0b72ad0-1234-5678-9abc-def012345678"
        );
    }

    #[test]
    fn user_emails_payload_deserializes() {
        let payload = r#"{
            "pagelen": 10,
            "page": 1,
            "size": 2,
            "values": [
                {
                    "email": "primary@example.com",
                    "is_primary": true,
                    "is_confirmed": true,
                    "type": "email"
                },
                {
                    "email": "alt@example.com",
                    "is_primary": false,
                    "is_confirmed": false,
                    "type": "email"
                }
            ]
        }"#;

        let page: Paginated<UserEmail> = serde_json::from_str(payload).unwrap();
        assert_eq!(page.values.len(), 2);
        assert_eq!(page.values[0].email.as_deref(), Some("primary@example.com"));
        assert_eq!(page.values[0].is_primary, Some(true));
        assert_eq!(page.values[1].is_confirmed, Some(false));
    }
}
