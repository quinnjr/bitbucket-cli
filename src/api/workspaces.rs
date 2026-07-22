use anyhow::Result;

use super::BitbucketClient;
use crate::models::{Paginated, Workspace, WorkspaceMembership, WorkspacePermission};

impl BitbucketClient {
    /// List all workspaces the authenticated user has access to,
    /// following pagination until every page has been fetched
    pub async fn list_workspaces(&self) -> Result<Vec<Workspace>> {
        self.get_all_pages("/workspaces").await
    }

    /// List a single page of workspaces with optional filters.
    ///
    /// `role` restricts results to workspaces where the caller has at least
    /// that role (member, collaborator, or owner), `query` is a Bitbucket
    /// query (BBQL) expression passed as the `q` parameter, and `sort` names
    /// a field to sort by (prefix with `-` for descending).
    pub async fn list_workspaces_filtered(
        &self,
        role: Option<&str>,
        query: Option<&str>,
        sort: Option<&str>,
        page: Option<u32>,
        pagelen: Option<u32>,
    ) -> Result<Paginated<Workspace>> {
        let mut params = Vec::new();

        if let Some(role) = role {
            params.push(("role", role.to_string()));
        }
        if let Some(q) = query {
            params.push(("q", q.to_string()));
        }
        if let Some(sort) = sort {
            params.push(("sort", sort.to_string()));
        }
        if let Some(p) = page {
            params.push(("page", p.to_string()));
        }
        if let Some(len) = pagelen {
            params.push(("pagelen", len.to_string()));
        }

        let query_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        self.get_with_query("/workspaces", &query_refs).await
    }

    /// Get a single workspace by slug or UUID
    pub async fn get_workspace(&self, workspace: &str) -> Result<Workspace> {
        let path = format!("/workspaces/{}", workspace);
        self.get(&path).await
    }

    /// List a page of members of a workspace
    pub async fn list_workspace_members(
        &self,
        workspace: &str,
        pagelen: Option<u32>,
    ) -> Result<Paginated<WorkspaceMembership>> {
        let path = format!("/workspaces/{}/members", workspace);
        match pagelen {
            Some(len) => {
                let len = len.to_string();
                self.get_with_query(&path, &[("pagelen", len.as_str())])
                    .await
            }
            None => self.get(&path).await,
        }
    }

    /// List a page of user permissions on a workspace, or — when `repo_slug`
    /// is given — on a single repository in that workspace.
    pub async fn list_workspace_permissions(
        &self,
        workspace: &str,
        repo_slug: Option<&str>,
        page: Option<u32>,
        pagelen: Option<u32>,
    ) -> Result<Paginated<WorkspacePermission>> {
        let path = match repo_slug {
            Some(repo) => format!(
                "/workspaces/{}/permissions/repositories/{}",
                workspace, repo
            ),
            None => format!("/workspaces/{}/permissions", workspace),
        };

        let mut params = Vec::new();
        if let Some(p) = page {
            params.push(("page", p.to_string()));
        }
        if let Some(len) = pagelen {
            params.push(("pagelen", len.to_string()));
        }

        let query_refs: Vec<(&str, &str)> = params.iter().map(|(k, v)| (*k, v.as_str())).collect();

        self.get_with_query(&path, &query_refs).await
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{Paginated, Workspace, WorkspaceMembership, WorkspacePermission};

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

    #[test]
    fn workspace_members_payload_deserializes() {
        let payload = r#"{
            "pagelen": 50,
            "page": 1,
            "size": 1,
            "values": [
                {
                    "type": "workspace_membership",
                    "user": {
                        "uuid": "{c3d4e5f6-0000-1111-2222-333344445555}",
                        "display_name": "Jane Doe",
                        "nickname": "jdoe",
                        "type": "user"
                    },
                    "workspace": {
                        "uuid": "{a1b2c3d4-0000-1111-2222-333344445555}",
                        "slug": "my-workspace",
                        "name": "My Workspace",
                        "type": "workspace"
                    }
                }
            ]
        }"#;

        let page: Paginated<WorkspaceMembership> = serde_json::from_str(payload).unwrap();
        assert_eq!(page.values.len(), 1);
        let member = &page.values[0];
        assert_eq!(member.user.as_ref().unwrap().display_name, "Jane Doe");
        assert_eq!(member.workspace.as_ref().unwrap().slug, "my-workspace");
    }

    #[test]
    fn workspace_permissions_payload_deserializes() {
        let payload = r#"{
            "pagelen": 50,
            "page": 1,
            "size": 1,
            "values": [
                {
                    "type": "workspace_membership",
                    "permission": "owner",
                    "user": {
                        "uuid": "{c3d4e5f6-0000-1111-2222-333344445555}",
                        "display_name": "Jane Doe",
                        "type": "user"
                    }
                }
            ]
        }"#;

        let page: Paginated<WorkspacePermission> = serde_json::from_str(payload).unwrap();
        assert_eq!(page.values.len(), 1);
        assert_eq!(page.values[0].permission.as_deref(), Some("owner"));
        assert_eq!(
            page.values[0].user.as_ref().unwrap().display_name,
            "Jane Doe"
        );
    }
}
