//! Banks API endpoint.

use crate::types::{Bank, ForeignPaymentCode, PaginatedResponse};

crate::define_endpoint! {
    /// Banks endpoint for accessing bank information.
    BanksEndpoint, "/banks", Bank,
    caps: [list],
    extra: {
        /// List all foreign payment codes.
        pub async fn list_foreign_payment_codes(
            &self,
        ) -> crate::error::Result<PaginatedResponse<ForeignPaymentCode>> {
            self.client.get("/foreignpaymentcodes").await
        }
    }
}
