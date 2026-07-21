use anyhow::Result;

use super::BitbucketClient;
use crate::models::Workspace;

impl BitbucketClient {
    /// List all workspaces the authenticated user has access to,
    /// following pagination until every page has been fetched
    pub async fn list_workspaces(&self) -> Result<Vec<Workspace>> {
        self.get_all_pages("/workspaces").await
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{Paginated, Workspace};

    #[test]
    fn workspace_list_payload_deserializes() {
        let payload = r#"{
            "pagelen": 10,
            "page": 1,
            "size": 2,
            "values": [
                {
                    "uuid": "{a1b2c3d4-0000-1111-2222-333344445555}",
                    "slug": "my-workspace",
                    "name": "My Workspace",
                    "type": "workspace",
                    "is_private": true,
                    "created_on": "2020-04-07T21:38:29.542346+00:00",
                    "links": {
                        "html": { "href": "https://bitbucket.org/my-workspace/" },
                        "avatar": { "href": "https://bitbucket.org/account/my-workspace/avatar/" }
                    }
                },
                {
                    "uuid": "{b2c3d4e5-0000-1111-2222-333344445555}",
                    "slug": "other",
                    "name": "Other",
                    "type": "workspace"
                }
            ]
        }"#;

        let page: Paginated<Workspace> = serde_json::from_str(payload).unwrap();
        assert_eq!(page.values.len(), 2);
        assert_eq!(page.values[0].slug, "my-workspace");
        assert_eq!(page.values[0].is_private, Some(true));
        assert!(page.values[0].created_on.is_some());
        assert_eq!(page.values[1].is_private, None);
        assert!(page.next.is_none());
    }
}
