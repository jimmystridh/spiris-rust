//! Allocation periods API endpoint.

use crate::types::AllocationPeriod;

crate::define_endpoint! {
    /// Allocation periods endpoint for managing accounting periods.
    AllocationPeriodsEndpoint, "/allocationperiods", AllocationPeriod,
    caps: [list, get, create]
}
