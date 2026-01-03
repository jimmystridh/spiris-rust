//! Units API endpoint.

use crate::types::Unit;

crate::define_endpoint! {
    /// Units endpoint for managing measurement units.
    UnitsEndpoint, "/units", Unit,
    caps: [list, get, create, update, delete]
}
