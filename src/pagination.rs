//! Pagination stream support for iterating through paginated API responses.
//!
//! This module provides utilities for automatically fetching
//! and iterating through all pages of a paginated API response.
//!
//! # Feature Flag
//!
//! This module is only available when the `stream` feature is enabled:
//!
//! ```toml
//! [dependencies]
//! spiris = { version = "0.1", features = ["stream"] }
//! ```
//!
//! # Example
//!
//! ```ignore
//! use futures::StreamExt;
//! use tokio::pin;
//! use spiris::{Client, AccessToken};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let token = AccessToken::new("token".to_string(), 3600, None);
//! let client = Client::new(token);
//!
//! // Stream all customers, automatically fetching pages as needed
//! let stream = client.customers().list_stream();
//! pin!(stream);
//! while let Some(result) = stream.next().await {
//!     let customer = result?;
//!     println!("Customer: {:?}", customer.name);
//! }
//!
//! // Or collect all into a Vec
//! use futures::TryStreamExt;
//! let all_customers: Vec<_> = client.customers()
//!     .list_stream()
//!     .try_collect()
//!     .await?;
//! # Ok(())
//! # }
//! ```

use crate::error::Result;
use crate::types::{PaginatedResponse, PaginationParams};
use futures::Stream;
use std::future::Future;

/// Default page size for pagination streams.
pub const DEFAULT_PAGE_SIZE: u32 = 50;

/// Creates a stream that automatically paginates through API responses.
///
/// This is a macro-based stream that yields items one at a time from paginated
/// API responses, automatically fetching the next page when needed.
///
/// # Type Parameters
///
/// * `T` - The type of items in the paginated response
/// * `F` - A function that takes pagination parameters and returns a future
/// * `Fut` - The future type returned by `F`
///
/// # Arguments
///
/// * `page_size` - Number of items per page
/// * `fetch` - A closure that fetches a page given `PaginationParams`
#[macro_export]
macro_rules! paginated_stream {
    ($page_size:expr, $fetch:expr) => {{
        async_stream::try_stream! {
            let mut current_page = 0u32;
            let fetch_fn = $fetch;

            loop {
                let params = $crate::PaginationParams::new()
                    .page(current_page)
                    .pagesize($page_size);

                let response = fetch_fn(params).await?;

                for item in response.data {
                    yield item;
                }

                if !response.meta.has_next_page {
                    break;
                }

                current_page += 1;
            }
        }
    }};
}

/// Creates a stream that automatically paginates through API responses.
///
/// This function takes a fetch function that retrieves a page of data and
/// returns a stream that yields individual items, automatically fetching
/// the next page when needed.
///
/// # Type Parameters
///
/// * `T` - The type of items in the paginated response
/// * `F` - A function that takes pagination parameters and returns a future
/// * `Fut` - The future type returned by `F`
///
/// # Arguments
///
/// * `page_size` - Number of items per page (default: 50)
/// * `fetch` - A function that fetches a page given `PaginationParams`
pub fn paginated_stream<T, F, Fut>(page_size: u32, fetch: F) -> impl Stream<Item = Result<T>>
where
    T: 'static,
    F: Fn(PaginationParams) -> Fut + 'static,
    Fut: Future<Output = Result<PaginatedResponse<T>>>,
{
    async_stream::try_stream! {
        let mut current_page = 0u32;

        loop {
            let params = PaginationParams::new()
                .page(current_page)
                .pagesize(page_size);

            let response = fetch(params).await?;

            for item in response.data {
                yield item;
            }

            if !response.meta.has_next_page {
                break;
            }

            current_page += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ResponseMetadata;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_paginated_stream_single_page() {
        let items = vec!["a", "b", "c"];
        let stream = paginated_stream(50, move |_params| {
            let items = items.clone();
            async move {
                Ok(PaginatedResponse {
                    data: items,
                    meta: ResponseMetadata {
                        current_page: 0,
                        page_size: 50,
                        total_pages: 1,
                        total_count: 3,
                        has_next_page: false,
                        has_previous_page: false,
                    },
                })
            }
        });

        let results: Vec<Result<&str>> = stream.collect().await;
        assert_eq!(results.len(), 3);
        assert_eq!(results[0].as_ref().unwrap(), &"a");
        assert_eq!(results[1].as_ref().unwrap(), &"b");
        assert_eq!(results[2].as_ref().unwrap(), &"c");
    }

    #[tokio::test]
    async fn test_paginated_stream_multiple_pages() {
        use std::sync::atomic::{AtomicU32, Ordering};
        use std::sync::Arc;

        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();

        let stream = paginated_stream(2, move |params| {
            let count = call_count_clone.fetch_add(1, Ordering::SeqCst);
            async move {
                match count {
                    0 => Ok(PaginatedResponse {
                        data: vec![1, 2],
                        meta: ResponseMetadata {
                            current_page: params.page.unwrap_or(0),
                            page_size: 2,
                            total_pages: 2,
                            total_count: 4,
                            has_next_page: true,
                            has_previous_page: false,
                        },
                    }),
                    1 => Ok(PaginatedResponse {
                        data: vec![3, 4],
                        meta: ResponseMetadata {
                            current_page: params.page.unwrap_or(0),
                            page_size: 2,
                            total_pages: 2,
                            total_count: 4,
                            has_next_page: false,
                            has_previous_page: true,
                        },
                    }),
                    _ => panic!("Too many calls"),
                }
            }
        });

        let results: Vec<Result<i32>> = stream.collect().await;
        assert_eq!(results.len(), 4);
        assert_eq!(*results[0].as_ref().unwrap(), 1);
        assert_eq!(*results[1].as_ref().unwrap(), 2);
        assert_eq!(*results[2].as_ref().unwrap(), 3);
        assert_eq!(*results[3].as_ref().unwrap(), 4);
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_paginated_stream_empty() {
        let stream = paginated_stream(50, |_params| async move {
            Ok(PaginatedResponse::<String> {
                data: vec![],
                meta: ResponseMetadata {
                    current_page: 0,
                    page_size: 50,
                    total_pages: 0,
                    total_count: 0,
                    has_next_page: false,
                    has_previous_page: false,
                },
            })
        });

        let results: Vec<Result<String>> = stream.collect().await;
        assert!(results.is_empty());
    }

    #[tokio::test]
    async fn test_paginated_stream_error_propagation() {
        use crate::error::Error;

        let stream = paginated_stream(50, |_params| async move {
            Err::<PaginatedResponse<i32>, _>(Error::NotFound("not found".to_string()))
        });

        let results: Vec<Result<i32>> = stream.collect().await;
        assert_eq!(results.len(), 1);
        assert!(results[0].is_err());
    }
}
