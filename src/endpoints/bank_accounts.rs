//! Bank accounts API endpoint.

use crate::types::BankAccount;

crate::define_endpoint! {
    /// Bank accounts endpoint for managing payment accounts.
    BankAccountsEndpoint, "/bankaccounts", BankAccount,
    caps: [list, get, create, update, delete]
}
