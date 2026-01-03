//! Endpoint definition macros to reduce boilerplate.
//!
//! This module provides macros for defining API endpoints with consistent
//! implementations of common CRUD operations.

/// Define an API endpoint with specified capabilities.
///
/// This macro generates endpoint structs with the specified operations.
/// Operations are enabled using capability flags.
///
/// # Capabilities
///
/// - `list` - List all items with pagination
/// - `get` - Get a single item by ID
/// - `create` - Create a new item
/// - `update` - Update an existing item
/// - `delete` - Delete an item
/// - `search` - Search with query parameters
/// - `stream` - Paginated streaming (requires `stream` feature)
///
/// # Example
///
/// ```ignore
/// // Full CRUD endpoint with search and streaming
/// define_endpoint! {
///     CustomersEndpoint, "/customers", Customer,
///     caps: [list, get, create, update, delete, search, stream]
/// }
///
/// // Read-only endpoint
/// define_endpoint! {
///     VatCodesEndpoint, "/vatcodes", VatCode,
///     caps: [list, get]
/// }
/// ```
#[macro_export]
macro_rules! define_endpoint {
    // Main entry point
    (
        $(#[$outer:meta])*
        $endpoint:ident, $path:literal, $type:ty,
        caps: [$($cap:ident),* $(,)?]
        $(, extra: { $($extra:tt)* })?
    ) => {
        $(#[$outer])*
        pub struct $endpoint<'a> {
            client: &'a $crate::client::Client,
        }

        impl<'a> $endpoint<'a> {
            /// Create a new endpoint instance.
            pub(crate) fn new(client: &'a $crate::client::Client) -> Self {
                Self { client }
            }

            $crate::__endpoint_impl!($path, $type, $($cap),*);

            $($($extra)*)?
        }
    };
}

/// Internal macro for implementing endpoint capabilities.
#[macro_export]
#[doc(hidden)]
macro_rules! __endpoint_impl {
    // Base case - no more capabilities
    ($path:literal, $type:ty,) => {};

    // list capability
    ($path:literal, $type:ty, list $(, $rest:ident)*) => {
        /// List all items with optional pagination.
        pub async fn list(
            &self,
            params: Option<$crate::types::PaginationParams>,
        ) -> $crate::error::Result<$crate::types::PaginatedResponse<$type>> {
            if let Some(params) = params {
                self.client.get_with_params($path, &params).await
            } else {
                self.client.get($path).await
            }
        }

        $crate::__endpoint_impl!($path, $type, $($rest),*);
    };

    // get capability
    ($path:literal, $type:ty, get $(, $rest:ident)*) => {
        /// Get a specific item by ID.
        pub async fn get(&self, id: &str) -> $crate::error::Result<$type> {
            self.client.get(&format!(concat!($path, "/{}"), id)).await
        }

        $crate::__endpoint_impl!($path, $type, $($rest),*);
    };

    // create capability
    ($path:literal, $type:ty, create $(, $rest:ident)*) => {
        /// Create a new item.
        pub async fn create(&self, item: &$type) -> $crate::error::Result<$type> {
            self.client.post($path, item).await
        }

        $crate::__endpoint_impl!($path, $type, $($rest),*);
    };

    // update capability
    ($path:literal, $type:ty, update $(, $rest:ident)*) => {
        /// Update an existing item.
        pub async fn update(&self, id: &str, item: &$type) -> $crate::error::Result<$type> {
            self.client.put(&format!(concat!($path, "/{}"), id), item).await
        }

        $crate::__endpoint_impl!($path, $type, $($rest),*);
    };

    // delete capability
    ($path:literal, $type:ty, delete $(, $rest:ident)*) => {
        /// Delete an item.
        pub async fn delete(&self, id: &str) -> $crate::error::Result<()> {
            self.client.delete(&format!(concat!($path, "/{}"), id)).await
        }

        $crate::__endpoint_impl!($path, $type, $($rest),*);
    };

    // search capability
    ($path:literal, $type:ty, search $(, $rest:ident)*) => {
        /// Search items with custom query parameters.
        pub async fn search(
            &self,
            query: $crate::types::QueryParams,
            pagination: Option<$crate::types::PaginationParams>,
        ) -> $crate::error::Result<$crate::types::PaginatedResponse<$type>> {
            #[derive(serde::Serialize)]
            struct CombinedParams {
                #[serde(flatten)]
                query: $crate::types::QueryParams,
                #[serde(flatten)]
                pagination: Option<$crate::types::PaginationParams>,
            }

            let params = CombinedParams { query, pagination };
            self.client.get_with_params($path, &params).await
        }

        $crate::__endpoint_impl!($path, $type, $($rest),*);
    };

    // stream capability
    ($path:literal, $type:ty, stream $(, $rest:ident)*) => {
        /// Stream all items, automatically paginating through results.
        ///
        /// Requires the `stream` feature.
        #[cfg(feature = "stream")]
        pub fn list_stream(&self) -> impl futures::Stream<Item = $crate::error::Result<$type>> + '_ {
            self.list_stream_with_page_size($crate::pagination::DEFAULT_PAGE_SIZE)
        }

        /// Stream all items with a custom page size.
        ///
        /// Requires the `stream` feature.
        #[cfg(feature = "stream")]
        pub fn list_stream_with_page_size(
            &self,
            page_size: u32,
        ) -> impl futures::Stream<Item = $crate::error::Result<$type>> + '_ {
            $crate::paginated_stream!(page_size, |params| self.list(Some(params)))
        }

        $crate::__endpoint_impl!($path, $type, $($rest),*);
    };

    // Handle trailing comma
    ($path:literal, $type:ty, $cap:ident,) => {
        $crate::__endpoint_impl!($path, $type, $cap);
    };
}

// Macro compilation is tested via the actual endpoint implementations
// in src/endpoints/. Those tests cover all macro variants.
