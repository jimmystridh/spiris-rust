//! Projects API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{PaginatedResponse, PaginationParams, Project, QueryParams};

pub struct ProjectsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> ProjectsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Project>> {
        if let Some(params) = params {
            self.client.get_with_params("/projects", &params).await
        } else {
            self.client.get("/projects").await
        }
    }

    pub async fn get(&self, id: &str) -> Result<Project> {
        self.client.get(&format!("/projects/{}", id)).await
    }

    pub async fn create(&self, project: &Project) -> Result<Project> {
        self.client.post("/projects", project).await
    }

    pub async fn update(&self, id: &str, project: &Project) -> Result<Project> {
        self.client.put(&format!("/projects/{}", id), project).await
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("/projects/{}", id)).await
    }

    pub async fn search(
        &self,
        query: QueryParams,
        pagination: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Project>> {
        #[derive(serde::Serialize)]
        struct CombinedParams {
            #[serde(flatten)]
            query: QueryParams,
            #[serde(flatten)]
            pagination: Option<PaginationParams>,
        }
        self.client
            .get_with_params("/projects", &CombinedParams { query, pagination })
            .await
    }
}
