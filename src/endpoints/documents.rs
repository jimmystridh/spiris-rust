//! Documents API endpoint.

use crate::types::Document;

crate::define_endpoint! {
    /// Documents endpoint for accessing document information.
    DocumentsEndpoint, "/documents", Document,
    caps: [get]
}
