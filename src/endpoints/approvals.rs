//! Approval endpoints API.

use crate::client::Client;
use crate::error::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ApprovalAction {
    pub is_approved: Option<bool>,
    pub comment: Option<String>,
}

pub struct ApprovalsEndpoint<'a> {
    client: &'a Client,
}

impl<'a> ApprovalsEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn approve_vat_report(&self, id: &str, action: &ApprovalAction) -> Result<()> {
        self.client
            .put::<(), _>(&format!("/approval/vatreport/{}", id), action)
            .await?;
        Ok(())
    }

    pub async fn approve_supplier_invoice(&self, id: &str, action: &ApprovalAction) -> Result<()> {
        self.client
            .put::<(), _>(&format!("/approval/supplierinvoice/{}", id), action)
            .await?;
        Ok(())
    }
}
