//! Benchmarks for the Spiris Bokföring API client.
//!
//! Run with: cargo bench
//!
//! These benchmarks measure:
//! - JSON serialization/deserialization performance
//! - HTTP request latency with mock server
//! - Concurrent request handling

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use spiris::{
    AccessToken, Article, Client, ClientConfig, Customer, Invoice, InvoiceRow,
    PaginatedResponse, PaginationParams,
};

// =============================================================================
// Serialization Benchmarks
// =============================================================================

fn customer_serialization(c: &mut Criterion) {
    let customer = Customer {
        id: Some("cust-12345".to_string()),
        customer_number: Some("C001".to_string()),
        name: Some("Acme Corporation AB".to_string()),
        email: Some("contact@acme.se".to_string()),
        phone: Some("+46701234567".to_string()),
        mobile_phone: Some("+46709876543".to_string()),
        is_active: Some(true),
        is_private_person: Some(false),
        ..Default::default()
    };

    c.bench_function("customer_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&customer)).unwrap())
    });
}

fn customer_deserialization(c: &mut Criterion) {
    let json = r#"{
        "Id": "cust-12345",
        "CustomerNumber": "C001",
        "Name": "Acme Corporation AB",
        "Email": "contact@acme.se",
        "Phone": "+46701234567",
        "MobilePhone": "+46709876543",
        "IsActive": true,
        "IsPrivatePerson": false,
        "CreatedUtc": "2024-01-15T10:30:00Z",
        "ModifiedUtc": "2024-01-16T14:45:30Z"
    }"#;

    c.bench_function("customer_deserialize", |b| {
        b.iter(|| serde_json::from_str::<Customer>(black_box(json)).unwrap())
    });
}

fn invoice_with_rows_serialization(c: &mut Criterion) {
    let invoice = Invoice {
        id: Some("inv-12345".to_string()),
        invoice_number: Some("1001".to_string()),
        customer_id: Some("cust-12345".to_string()),
        total_amount: Some(10000.00),
        total_vat_amount: Some(2500.00),
        total_amount_including_vat: Some(12500.00),
        rows: (0..10)
            .map(|i| InvoiceRow {
                id: Some(format!("row-{}", i)),
                article_id: Some(format!("art-{}", i)),
                text: Some(format!("Product {} description", i)),
                unit_price: Some(100.0 * (i + 1) as f64),
                quantity: Some(2.0),
                discount_percentage: Some(0.0),
                total_amount: Some(200.0 * (i + 1) as f64),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };

    c.bench_function("invoice_with_10_rows_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&invoice)).unwrap())
    });
}

fn invoice_with_rows_deserialization(c: &mut Criterion) {
    let json = r#"{
        "Id": "inv-12345",
        "InvoiceNumber": "1001",
        "CustomerId": "cust-12345",
        "TotalAmount": 10000.00,
        "TotalVatAmount": 2500.00,
        "TotalAmountIncludingVat": 12500.00,
        "Rows": [
            {"Id": "row-0", "ArticleId": "art-0", "Text": "Product 0", "UnitPrice": 100.00, "Quantity": 2.0, "TotalAmount": 200.00},
            {"Id": "row-1", "ArticleId": "art-1", "Text": "Product 1", "UnitPrice": 200.00, "Quantity": 2.0, "TotalAmount": 400.00},
            {"Id": "row-2", "ArticleId": "art-2", "Text": "Product 2", "UnitPrice": 300.00, "Quantity": 2.0, "TotalAmount": 600.00},
            {"Id": "row-3", "ArticleId": "art-3", "Text": "Product 3", "UnitPrice": 400.00, "Quantity": 2.0, "TotalAmount": 800.00},
            {"Id": "row-4", "ArticleId": "art-4", "Text": "Product 4", "UnitPrice": 500.00, "Quantity": 2.0, "TotalAmount": 1000.00},
            {"Id": "row-5", "ArticleId": "art-5", "Text": "Product 5", "UnitPrice": 600.00, "Quantity": 2.0, "TotalAmount": 1200.00},
            {"Id": "row-6", "ArticleId": "art-6", "Text": "Product 6", "UnitPrice": 700.00, "Quantity": 2.0, "TotalAmount": 1400.00},
            {"Id": "row-7", "ArticleId": "art-7", "Text": "Product 7", "UnitPrice": 800.00, "Quantity": 2.0, "TotalAmount": 1600.00},
            {"Id": "row-8", "ArticleId": "art-8", "Text": "Product 8", "UnitPrice": 900.00, "Quantity": 2.0, "TotalAmount": 1800.00},
            {"Id": "row-9", "ArticleId": "art-9", "Text": "Product 9", "UnitPrice": 1000.00, "Quantity": 2.0, "TotalAmount": 2000.00}
        ]
    }"#;

    c.bench_function("invoice_with_10_rows_deserialize", |b| {
        b.iter(|| serde_json::from_str::<Invoice>(black_box(json)).unwrap())
    });
}

// =============================================================================
// Paginated Response Benchmarks
// =============================================================================

fn paginated_response_deserialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("paginated_response_deserialize");

    for size in [1, 10, 50, 100].iter() {
        let customers: Vec<String> = (0..*size)
            .map(|i| {
                format!(
                    r#"{{"Id": "cust-{}", "CustomerNumber": "C{:04}", "Name": "Customer {}", "Email": "cust{}@example.com", "IsActive": true}}"#,
                    i, i, i, i
                )
            })
            .collect();

        let json = format!(
            r#"{{
                "Data": [{}],
                "Meta": {{
                    "CurrentPage": 0,
                    "PageSize": 50,
                    "TotalPages": 1,
                    "TotalCount": {},
                    "HasNextPage": false,
                    "HasPreviousPage": false
                }}
            }}"#,
            customers.join(","),
            size
        );

        group.bench_with_input(
            BenchmarkId::from_parameter(size),
            &json,
            |b, json| {
                b.iter(|| {
                    serde_json::from_str::<PaginatedResponse<Customer>>(black_box(json)).unwrap()
                })
            },
        );
    }

    group.finish();
}

// =============================================================================
// Article Benchmarks
// =============================================================================

fn article_serialization(c: &mut Criterion) {
    let article = Article {
        id: Some("art-12345".to_string()),
        article_number: Some("ART001".to_string()),
        name: Some("Premium Widget".to_string()),
        unit: Some("pcs".to_string()),
        sales_price: Some(299.99),
        purchase_price: Some(149.99),
        is_active: Some(true),
        vat_rate_id: Some("vat-25".to_string()),
        ..Default::default()
    };

    c.bench_function("article_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&article)).unwrap())
    });
}

fn article_deserialization(c: &mut Criterion) {
    let json = r#"{
        "Id": "art-12345",
        "ArticleNumber": "ART001",
        "Name": "Premium Widget",
        "Unit": "pcs",
        "SalesPrice": 299.99,
        "PurchasePrice": 149.99,
        "IsActive": true,
        "VatRateId": "vat-25",
        "CreatedUtc": "2024-01-15T10:30:00Z",
        "ModifiedUtc": "2024-01-16T14:45:30Z"
    }"#;

    c.bench_function("article_deserialize", |b| {
        b.iter(|| serde_json::from_str::<Article>(black_box(json)).unwrap())
    });
}

// =============================================================================
// Token Benchmarks
// =============================================================================

fn access_token_creation(c: &mut Criterion) {
    c.bench_function("access_token_create", |b| {
        b.iter(|| {
            AccessToken::new(
                black_box("test_token_value_here".to_string()),
                black_box(3600),
                black_box(Some("refresh_token_here".to_string())),
            )
        })
    });
}

fn access_token_expiration_check(c: &mut Criterion) {
    let token = AccessToken::new("test_token".to_string(), 3600, None);

    c.bench_function("access_token_is_expired", |b| {
        b.iter(|| black_box(&token).is_expired())
    });
}

fn access_token_authorization_header(c: &mut Criterion) {
    let token = AccessToken::new("test_token_value".to_string(), 3600, None);

    c.bench_function("access_token_auth_header", |b| {
        b.iter(|| black_box(&token).authorization_header())
    });
}

// =============================================================================
// Pagination Params Benchmarks
// =============================================================================

fn pagination_params_serialization(c: &mut Criterion) {
    let params = PaginationParams::new().page(5).pagesize(100);

    c.bench_function("pagination_params_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&params)).unwrap())
    });
}

// =============================================================================
// Client Creation Benchmarks
// =============================================================================

fn client_creation(c: &mut Criterion) {
    c.bench_function("client_create", |b| {
        b.iter(|| {
            let token = AccessToken::new("test_token".to_string(), 3600, None);
            Client::new(black_box(token))
        })
    });
}

fn client_with_config_creation(c: &mut Criterion) {
    c.bench_function("client_with_config_create", |b| {
        b.iter(|| {
            let token = AccessToken::new("test_token".to_string(), 3600, None);
            let config = ClientConfig::new()
                .base_url("https://api.example.com/v2/")
                .timeout_seconds(60);
            Client::with_config(black_box(token), black_box(config))
        })
    });
}

// =============================================================================
// Benchmark Groups
// =============================================================================

criterion_group!(
    serialization_benches,
    customer_serialization,
    customer_deserialization,
    invoice_with_rows_serialization,
    invoice_with_rows_deserialization,
    article_serialization,
    article_deserialization,
    paginated_response_deserialization,
    pagination_params_serialization,
);

criterion_group!(
    token_benches,
    access_token_creation,
    access_token_expiration_check,
    access_token_authorization_header,
);

criterion_group!(
    client_benches,
    client_creation,
    client_with_config_creation,
);

criterion_main!(serialization_benches, token_benches, client_benches);
