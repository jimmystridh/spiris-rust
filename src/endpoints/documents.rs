//! Documents API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::Document;

pub struct DocumentsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> DocumentsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn get(&self, id: &str) -> Result<Document> {
        self.client.get(&format!("/documents/{}", id)).await
    }
}
