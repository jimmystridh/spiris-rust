//! Projects API endpoint.

use crate::types::Project;

crate::define_endpoint! {
    /// Projects endpoint for managing project tracking.
    ProjectsEndpoint, "/projects", Project,
    caps: [list, get, create, update, delete, search]
}
