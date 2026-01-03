//! Article account codings API endpoint.

use crate::types::ArticleAccountCoding;

crate::define_endpoint! {
    /// Article account codings endpoint for accessing article accounting mappings.
    ArticleAccountCodingsEndpoint, "/articleaccountcodings", ArticleAccountCoding,
    caps: [list, get]
}
