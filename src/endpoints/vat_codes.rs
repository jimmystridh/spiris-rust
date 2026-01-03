//! VAT codes API endpoint.

use crate::types::VatCode;

crate::define_endpoint! {
    /// VAT codes endpoint for managing tax rates.
    VatCodesEndpoint, "/vatcodes", VatCode,
    caps: [list, get]
}
