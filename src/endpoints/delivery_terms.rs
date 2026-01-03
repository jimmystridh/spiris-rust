//! Delivery terms API endpoint.

use crate::types::DeliveryTerm;

crate::define_endpoint! {
    /// Delivery terms endpoint for managing delivery terms.
    DeliveryTermsEndpoint, "/deliveryterms", DeliveryTerm,
    caps: [list, get]
}
