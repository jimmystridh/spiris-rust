//! Currencies API endpoint.

use crate::types::Currency;

crate::define_endpoint! {
    /// Currencies endpoint for accessing available currencies.
    CurrenciesEndpoint, "/currencies", Currency,
    caps: [list]
}
