//! Customer labels API endpoint.

use crate::types::CustomerLabel;

crate::define_endpoint! {
    /// Customer labels endpoint for managing customer categorization.
    CustomerLabelsEndpoint, "/customerlabels", CustomerLabel,
    caps: [list, get, create, update, delete]
}
