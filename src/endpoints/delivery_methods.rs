//! Delivery methods API endpoint.

use crate::types::DeliveryMethod;

crate::define_endpoint! {
    /// Delivery methods endpoint for managing shipping methods.
    DeliveryMethodsEndpoint, "/deliverymethods", DeliveryMethod,
    caps: [list, get]
}
