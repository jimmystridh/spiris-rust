//! Example: List all customers with pagination
//!
//! Run with:
//! ```
//! cargo run --example list_customers
//! ```

use spiris::{AccessToken, Client, PaginationParams};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // In a real application, you would obtain this token through OAuth2
    let access_token = std::env::var("SPIRIS_ACCESS_TOKEN")
        .expect("Please set SPIRIS_ACCESS_TOKEN environment variable");

    // Create an access token (expires in 1 hour)
    let token = AccessToken::new(access_token, 3600, None);

    // Create the API client
    let client = Client::new(token);

    // List customers with pagination
    let params = PaginationParams::new().page(0).pagesize(50);

    println!("Fetching customers...");
    let response = client.customers().list(Some(params)).await?;

    println!("\nFound {} total customers", response.meta.total_count);
    println!(
        "Page {} of {}",
        response.meta.current_page + 1,
        response.meta.total_pages
    );
    println!("\nCustomers:");
    println!("{:-<80}", "");

    for customer in response.data {
        println!(
            "ID: {:?}\nName: {:?}\nEmail: {:?}\n",
            customer.id, customer.name, customer.email
        );
    }

    Ok(())
}
