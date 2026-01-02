//! Attachments API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{Attachment, AttachmentLink, PaginatedResponse, PaginationParams};

pub struct AttachmentsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> AttachmentsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Attachment>> {
        if let Some(params) = params {
            self.client.get_with_params("/attachments", &params).await
        } else {
            self.client.get("/attachments").await
        }
    }

    pub async fn get(&self, id: &str) -> Result<Attachment> {
        self.client.get(&format!("/attachments/{}", id)).await
    }

    pub async fn delete(&self, id: &str) -> Result<()> {
        self.client.delete(&format!("/attachments/{}", id)).await
    }

    pub async fn get_content(&self, id: &str) -> Result<Vec<u8>> {
        self.client
            .get_bytes(&format!("/attachments/{}/content", id))
            .await
    }

    pub async fn create_link(&self, link: &AttachmentLink) -> Result<AttachmentLink> {
        self.client.post("/attachmentlinks", link).await
    }

    pub async fn delete_link(&self, id: &str) -> Result<()> {
        self.client
            .delete(&format!("/attachmentlinks/{}", id))
            .await
    }
}
