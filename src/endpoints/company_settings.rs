//! Company settings API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::CompanySettings;

pub struct CompanySettingsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> CompanySettingsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn get(&self) -> Result<CompanySettings> {
        self.client.get("/companysettings").await
    }

    pub async fn update(&self, settings: &CompanySettings) -> Result<CompanySettings> {
        self.client.put("/companysettings", settings).await
    }
}
