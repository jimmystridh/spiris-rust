//! Supplier labels API endpoint.

use crate::types::SupplierLabel;

crate::define_endpoint! {
    /// Supplier labels endpoint for managing supplier categorization.
    SupplierLabelsEndpoint, "/supplierlabels", SupplierLabel,
    caps: [list, get, create, update, delete]
}
