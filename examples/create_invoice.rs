//! Example: Create a customer invoice
//!
//! Run with:
//! ```
//! cargo run --example create_invoice
//! ```

use chrono::Utc;
use visma_eaccounting::{AccessToken, Client, Invoice, InvoiceRow};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // In a real application, you would obtain this token through OAuth2
    let access_token = std::env::var("VISMA_ACCESS_TOKEN")
        .expect("Please set VISMA_ACCESS_TOKEN environment variable");

    // You also need a customer ID to create an invoice
    let customer_id = std::env::var("VISMA_CUSTOMER_ID")
        .expect("Please set VISMA_CUSTOMER_ID environment variable");

    // Create an access token
    let token = AccessToken::new(access_token, 3600, None);

    // Create the API client
    let client = Client::new(token);

    // Create a new invoice
    let new_invoice = Invoice {
        customer_id: Some(customer_id),
        invoice_date: Some(Utc::now()),
        currency_code: Some("SEK".to_string()),
        remarks: Some("Thank you for your business!".to_string()),
        rows: vec![
            InvoiceRow {
                text: Some("Consulting services - Project A".to_string()),
                unit_price: Some(1200.0),
                quantity: Some(40.0), // 40 hours
                discount_percentage: Some(0.0),
                ..Default::default()
            },
            InvoiceRow {
                text: Some("Development services - Module B".to_string()),
                unit_price: Some(1500.0),
                quantity: Some(20.0), // 20 hours
                discount_percentage: Some(10.0), // 10% discount
                ..Default::default()
            },
        ],
        ..Default::default()
    };

    println!("Creating invoice...");
    let created = client.invoices().create(&new_invoice).await?;

    println!("\nInvoice created successfully!");
    println!("ID: {:?}", created.id);
    println!("Invoice Number: {:?}", created.invoice_number);
    println!("Total Amount: {:?}", created.total_amount);
    println!("Total VAT: {:?}", created.total_vat_amount);
    println!(
        "Total Including VAT: {:?}",
        created.total_amount_including_vat
    );

    Ok(())
}
