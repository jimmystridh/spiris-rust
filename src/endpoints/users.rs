//! Users API endpoint.

use crate::types::User;

crate::define_endpoint! {
    /// Users endpoint for accessing user information.
    UsersEndpoint, "/users", User,
    caps: [list, get]
}
