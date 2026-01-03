//! Real API integration tests.
//!
//! These tests are IGNORED by default and require a real API token to run.
//! They validate that the client works correctly against the actual Spiris/Visma API.
//!
//! To run these tests:
//! 1. Set the SPIRIS_ACCESS_TOKEN environment variable with a valid token
//! 2. Run: cargo test --test real_api_test -- --ignored
//!
//! WARNING: These tests may create/modify real data. Use a test account!

use spiris::{AccessToken, Client, Customer, PaginationParams};
use std::env;

fn get_client() -> Option<Client> {
    let token_str = env::var("SPIRIS_ACCESS_TOKEN").ok()?;
    let token = AccessToken::new(token_str, 3600, None);
    Some(Client::new(token))
}

// =============================================================================
// Read-Only Tests (Safe to run against production)
// =============================================================================

#[tokio::test]
#[ignore = "requires SPIRIS_ACCESS_TOKEN environment variable"]
async fn test_real_api_list_customers() {
    let client = get_client().expect("SPIRIS_ACCESS_TOKEN not set");

    let result = client.customers().list(None).await;

    match result {
        Ok(response) => {
            println!("Successfully fetched {} customers", response.data.len());
            println!("Total count: {}", response.meta.total_count);
            println!(
                "Page: {} of {}",
                response.meta.current_page + 1,
                response.meta.total_pages
            );

            for customer in response.data.iter().take(5) {
                println!(
                    "  - {} ({})",
                    customer.name.as_deref().unwrap_or("N/A"),
                    customer.customer_number.as_deref().unwrap_or("N/A")
                );
            }
        }
        Err(e) => {
            panic!("Failed to list customers: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore = "requires SPIRIS_ACCESS_TOKEN environment variable"]
async fn test_real_api_list_customers_with_pagination() {
    let client = get_client().expect("SPIRIS_ACCESS_TOKEN not set");

    let params = PaginationParams::new().page(0).pagesize(10);
    let result = client.customers().list(Some(params)).await;

    match result {
        Ok(response) => {
            assert!(response.data.len() <= 10, "Page size should be respected");
            println!("Fetched {} customers (page size 10)", response.data.len());
        }
        Err(e) => {
            panic!("Failed to list customers with pagination: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore = "requires SPIRIS_ACCESS_TOKEN environment variable"]
async fn test_real_api_list_articles() {
    let client = get_client().expect("SPIRIS_ACCESS_TOKEN not set");

    let result = client.articles().list(None).await;

    match result {
        Ok(response) => {
            println!("Successfully fetched {} articles", response.data.len());

            for article in response.data.iter().take(5) {
                println!(
                    "  - {} ({}) - {}",
                    article.name.as_deref().unwrap_or("N/A"),
                    article.article_number.as_deref().unwrap_or("N/A"),
                    article
                        .sales_price
                        .map(|p| format!("{:.2}", p))
                        .unwrap_or_else(|| "N/A".to_string())
                );
            }
        }
        Err(e) => {
            panic!("Failed to list articles: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore = "requires SPIRIS_ACCESS_TOKEN environment variable"]
async fn test_real_api_list_invoices() {
    let client = get_client().expect("SPIRIS_ACCESS_TOKEN not set");

    let result = client.invoices().list(None).await;

    match result {
        Ok(response) => {
            println!("Successfully fetched {} invoices", response.data.len());

            for invoice in response.data.iter().take(5) {
                println!(
                    "  - Invoice #{} - {} SEK",
                    invoice.invoice_number.as_deref().unwrap_or("N/A"),
                    invoice
                        .total_amount_including_vat
                        .map(|a| format!("{:.2}", a))
                        .unwrap_or_else(|| "N/A".to_string())
                );
            }
        }
        Err(e) => {
            panic!("Failed to list invoices: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore = "requires SPIRIS_ACCESS_TOKEN environment variable"]
async fn test_real_api_list_suppliers() {
    let client = get_client().expect("SPIRIS_ACCESS_TOKEN not set");

    let result = client.suppliers().list(None).await;

    match result {
        Ok(response) => {
            println!("Successfully fetched {} suppliers", response.data.len());

            for supplier in response.data.iter().take(5) {
                println!(
                    "  - {} ({})",
                    supplier.name.as_deref().unwrap_or("N/A"),
                    supplier.supplier_number.as_deref().unwrap_or("N/A")
                );
            }
        }
        Err(e) => {
            panic!("Failed to list suppliers: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore = "requires SPIRIS_ACCESS_TOKEN environment variable"]
async fn test_real_api_list_vouchers() {
    let client = get_client().expect("SPIRIS_ACCESS_TOKEN not set");

    let result = client.vouchers().list(None).await;

    match result {
        Ok(response) => {
            println!("Successfully fetched {} vouchers", response.data.len());

            for voucher in response.data.iter().take(5) {
                println!(
                    "  - Voucher #{} - {}",
                    voucher.voucher_number.as_deref().unwrap_or("N/A"),
                    voucher.voucher_text.as_deref().unwrap_or("N/A")
                );
            }
        }
        Err(e) => {
            panic!("Failed to list vouchers: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore = "requires SPIRIS_ACCESS_TOKEN environment variable"]
async fn test_real_api_get_company_settings() {
    let client = get_client().expect("SPIRIS_ACCESS_TOKEN not set");

    let result = client.company_settings().get().await;

    match result {
        Ok(settings) => {
            println!(
                "Company: {}",
                settings.company_name.as_deref().unwrap_or("N/A")
            );
            println!(
                "Org Number: {}",
                settings
                    .corporate_identity_number
                    .as_deref()
                    .unwrap_or("N/A")
            );
            println!(
                "Currency: {}",
                settings.currency_code.as_deref().unwrap_or("N/A")
            );
        }
        Err(e) => {
            panic!("Failed to get company settings: {:?}", e);
        }
    }
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires SPIRIS_ACCESS_TOKEN environment variable"]
async fn test_real_api_get_nonexistent_customer() {
    let client = get_client().expect("SPIRIS_ACCESS_TOKEN not set");

    let result = client.customers().get("nonexistent-id-12345").await;

    match result {
        Ok(_) => {
            panic!("Expected NotFound error for nonexistent customer");
        }
        Err(spiris::Error::NotFound(_)) => {
            println!("Correctly received NotFound error");
        }
        Err(e) => {
            panic!("Expected NotFound error, got: {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore = "requires SPIRIS_ACCESS_TOKEN environment variable"]
async fn test_real_api_expired_token() {
    // Create a client with an invalid token
    let token = AccessToken::new("invalid_token_12345".to_string(), 3600, None);
    let client = Client::new(token);

    let result = client.customers().list(None).await;

    match result {
        Ok(_) => {
            panic!("Expected auth error for invalid token");
        }
        Err(spiris::Error::AuthError(_)) => {
            println!("Correctly received AuthError for invalid token");
        }
        Err(e) => {
            // API might return different error codes for invalid tokens
            println!("Received error (expected): {:?}", e);
        }
    }
}

// =============================================================================
// Write Tests (DANGEROUS - Creates/modifies data!)
// =============================================================================

#[tokio::test]
#[ignore = "DANGEROUS: creates real data - requires SPIRIS_ACCESS_TOKEN"]
async fn test_real_api_create_and_delete_customer() {
    let client = get_client().expect("SPIRIS_ACCESS_TOKEN not set");

    // Create a test customer
    let test_customer = Customer {
        name: Some("Test Customer (API Test - Delete Me)".to_string()),
        email: Some("test-delete-me@example.com".to_string()),
        is_active: Some(true),
        is_private_person: Some(false),
        ..Default::default()
    };

    let create_result = client.customers().create(&test_customer).await;

    match create_result {
        Ok(created) => {
            println!(
                "Created customer: {} (ID: {})",
                created.name.as_deref().unwrap_or("N/A"),
                created.id.as_deref().unwrap_or("N/A")
            );

            // Clean up - delete the customer
            if let Some(id) = &created.id {
                let delete_result = client.customers().delete(id).await;
                match delete_result {
                    Ok(()) => {
                        println!("Successfully deleted test customer");
                    }
                    Err(e) => {
                        eprintln!("WARNING: Failed to delete test customer: {:?}", e);
                        eprintln!("Please manually delete customer with ID: {}", id);
                    }
                }
            }
        }
        Err(e) => {
            panic!("Failed to create customer: {:?}", e);
        }
    }
}

// =============================================================================
// Performance Tests
// =============================================================================

#[tokio::test]
#[ignore = "requires SPIRIS_ACCESS_TOKEN environment variable"]
async fn test_real_api_concurrent_requests() {
    let client = get_client().expect("SPIRIS_ACCESS_TOKEN not set");

    let start = std::time::Instant::now();

    // Fire multiple concurrent requests
    let customers = client.customers();
    let articles = client.articles();
    let invoices = client.invoices();

    let (customers_result, articles_result, invoices_result) = tokio::join!(
        customers.list(None),
        articles.list(None),
        invoices.list(None)
    );

    let elapsed = start.elapsed();

    println!("Concurrent requests completed in {:?}", elapsed);
    println!(
        "  Customers: {}",
        customers_result
            .map(|r| format!("{} items", r.data.len()))
            .unwrap_or_else(|e| format!("error: {:?}", e))
    );
    println!(
        "  Articles: {}",
        articles_result
            .map(|r| format!("{} items", r.data.len()))
            .unwrap_or_else(|e| format!("error: {:?}", e))
    );
    println!(
        "  Invoices: {}",
        invoices_result
            .map(|r| format!("{} items", r.data.len()))
            .unwrap_or_else(|e| format!("error: {:?}", e))
    );
}

#[tokio::test]
#[ignore = "requires SPIRIS_ACCESS_TOKEN environment variable"]
async fn test_real_api_pagination_all_pages() {
    let client = get_client().expect("SPIRIS_ACCESS_TOKEN not set");

    let mut page = 0;
    let mut total_fetched = 0;
    let page_size = 50;

    loop {
        let params = PaginationParams::new().page(page).pagesize(page_size);
        let result = client.customers().list(Some(params)).await;

        match result {
            Ok(response) => {
                total_fetched += response.data.len();
                println!(
                    "Page {}: {} items (total so far: {})",
                    page + 1,
                    response.data.len(),
                    total_fetched
                );

                if !response.meta.has_next_page {
                    break;
                }
                page += 1;

                // Safety limit
                if page > 100 {
                    println!("Reached 100 page limit, stopping");
                    break;
                }
            }
            Err(e) => {
                panic!("Failed to fetch page {}: {:?}", page, e);
            }
        }
    }

    println!("Total customers fetched: {}", total_fetched);
}
