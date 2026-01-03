//! Article labels API endpoint.

use crate::types::ArticleLabel;

crate::define_endpoint! {
    /// Article labels endpoint for managing article categorization.
    ArticleLabelsEndpoint, "/articlelabels", ArticleLabel,
    caps: [list, get, create, update, delete]
}
