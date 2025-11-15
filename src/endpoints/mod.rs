//! API endpoint modules.

pub mod customers;
pub mod invoices;
pub mod articles;

pub use customers::CustomersEndpoint;
pub use invoices::InvoicesEndpoint;
pub use articles::ArticlesEndpoint;
