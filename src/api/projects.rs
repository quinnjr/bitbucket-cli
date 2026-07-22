use anyhow::Result;

use super::BitbucketClient;
use crate::models::{CreateProjectRequest, Paginated, ProjectDetail, UpdateProjectRequest};

impl BitbucketClient {
    /// List a page of projects in a workspace.
    ///
    /// `query` is a Bitbucket query (BBQL) expression passed as the `q`
    /// parameter, and `sort` names a field to sort by (prefix with `-` for
    /// descending).
    pub async fn list_projects(
        &self,
        workspace: &str,
        query: Option<&str>,
        sort: Option<&str>,
        page: Option<u32>,
        pagelen: Option<u32>,
    ) -> Result<Paginated<ProjectDetail>> {
        let mut params = Vec::new();

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

        let path = format!("/workspaces/{}/projects", workspace);
        self.get_with_query(&path, &query_refs).await
    }

    /// Get a single project by its key
    pub async fn get_project(&self, workspace: &str, key: &str) -> Result<ProjectDetail> {
        let path = format!("/workspaces/{}/projects/{}", workspace, key);
        self.get(&path).await
    }

    /// Create a new project in a workspace
    pub async fn create_project(
        &self,
        workspace: &str,
        request: &CreateProjectRequest,
    ) -> Result<ProjectDetail> {
        let path = format!("/workspaces/{}/projects", workspace);
        self.post(&path, request).await
    }

    /// Update settings on an existing project
    pub async fn update_project(
        &self,
        workspace: &str,
        key: &str,
        request: &UpdateProjectRequest,
    ) -> Result<ProjectDetail> {
        let path = format!("/workspaces/{}/projects/{}", workspace, key);
        self.put(&path, request).await
    }

    /// Delete a project. The project must not contain any repositories.
    pub async fn delete_project(&self, workspace: &str, key: &str) -> Result<()> {
        let path = format!("/workspaces/{}/projects/{}", workspace, key);
        self.delete(&path).await
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{Paginated, ProjectDetail};

    #[test]
    fn project_list_payload_deserializes() {
        let payload = r#"{
            "pagelen": 10,
            "page": 1,
            "size": 2,
            "values": [
                {
                    "uuid": "{a1b2c3d4-0000-1111-2222-333344445555}",
                    "key": "PROJ",
                    "name": "My Project",
                    "description": "The main project",
                    "is_private": true,
                    "type": "project",
                    "created_on": "2020-04-07T21:38:29.542346+00:00",
                    "updated_on": "2021-06-01T08:00:00.000000+00:00",
                    "links": {
                        "html": { "href": "https://bitbucket.org/acme/workspace/projects/PROJ" }
                    }
                },
                {
                    "key": "OTHER",
                    "name": "Other",
                    "type": "project"
                }
            ]
        }"#;

        let page: Paginated<ProjectDetail> = serde_json::from_str(payload).unwrap();
        assert_eq!(page.values.len(), 2);
        assert_eq!(page.values[0].key, "PROJ");
        assert_eq!(page.values[0].is_private, Some(true));
        assert!(page.values[0].created_on.is_some());
        assert!(page.values[0].updated_on.is_some());
        assert_eq!(page.values[1].key, "OTHER");
        assert!(page.values[1].uuid.is_none());
        assert!(page.next.is_none());
    }
}
