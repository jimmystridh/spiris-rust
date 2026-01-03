//! Example: Create a new customer
//!
//! Run with:
//! ```
//! cargo run --example create_customer
//! ```

use spiris::{AccessToken, Address, Client, Customer};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // In a real application, you would obtain this token through OAuth2
    let access_token = std::env::var("SPIRIS_ACCESS_TOKEN")
        .expect("Please set SPIRIS_ACCESS_TOKEN environment variable");

    // Create an access token
    let token = AccessToken::new(access_token, 3600, None);

    // Create the API client
    let client = Client::new(token);

    // Create a new customer
    let new_customer = Customer {
        name: Some("Acme Corporation".to_string()),
        email: Some("contact@acme.com".to_string()),
        phone: Some("+46123456789".to_string()),
        website: Some("https://acme.com".to_string()),
        invoice_address: Some(Address {
            address1: Some("123 Main Street".to_string()),
            city: Some("Stockholm".to_string()),
            postal_code: Some("11122".to_string()),
            country_code: Some("SE".to_string()),
            ..Default::default()
        }),
        is_active: Some(true),
        is_private_person: Some(false),
        payment_terms_in_days: Some(30),
        ..Default::default()
    };

    println!("Creating customer...");
    let created = client.customers().create(&new_customer).await?;

    println!("\nCustomer created successfully!");
    println!("ID: {:?}", created.id);
    println!("Name: {:?}", created.name);
    println!("Email: {:?}", created.email);
    println!("Customer Number: {:?}", created.customer_number);

    Ok(())
}
