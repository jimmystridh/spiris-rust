//! Countries API endpoint.

use crate::types::Country;

crate::define_endpoint! {
    /// Countries endpoint for accessing available countries.
    CountriesEndpoint, "/countries", Country,
    caps: [list, get]
}
