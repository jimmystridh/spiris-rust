//! Terms of payment API endpoint.

use crate::types::TermsOfPayment;

crate::define_endpoint! {
    /// Terms of payment endpoint for managing payment terms.
    TermsOfPaymentEndpoint, "/termsofpayments", TermsOfPayment,
    caps: [list, get, create, update, delete]
}
