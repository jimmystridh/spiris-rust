//! Articles/Products API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{Article, PaginatedResponse, PaginationParams, QueryParams};

/// Articles endpoint for managing products and services.
///
/// # Example
///
/// ```no_run
/// use visma_eaccounting::{Client, AccessToken};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let token = AccessToken::new("your_token".to_string(), 3600, None);
///     let client = Client::new(token);
///     let articles = client.articles();
///
///     // List all articles
///     let all_articles = articles.list(None).await?;
///     println!("Found {} articles", all_articles.data.len());
///
///     Ok(())
/// }
/// ```
pub struct ArticlesEndpoint<'a> {
    client: &'a Client,
}

impl<'a> ArticlesEndpoint<'a> {
    /// Create a new articles endpoint.
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    /// List all articles with optional pagination.
    ///
    /// # Arguments
    ///
    /// * `params` - Optional pagination parameters
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use visma_eaccounting::{Client, PaginationParams};
    /// # async fn example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let params = PaginationParams::new().page(0).pagesize(100);
    /// let articles = client.articles().list(Some(params)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list(
        &self,
        params: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Article>> {
        if let Some(params) = params {
            self.client.get_with_params("/articles", &params).await
        } else {
            self.client.get("/articles").await
        }
    }

    /// Get a specific article by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The article ID (GUID)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use visma_eaccounting::Client;
    /// # async fn example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let article = client.articles().get("article-id-123").await?;
    /// println!("Article: {}", article.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get(&self, id: &str) -> Result<Article> {
        let path = format!("/articles/{}", id);
        self.client.get(&path).await
    }

    /// Create a new article.
    ///
    /// # Arguments
    ///
    /// * `article` - The article data to create
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use visma_eaccounting::{Client, Article};
    /// # async fn example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let new_article = Article {
    ///     name: "Consulting Hour".to_string(),
    ///     unit: Some("hours".to_string()),
    ///     sales_price: Some(1000.0),
    ///     ..Default::default()
    /// };
    /// let created = client.articles().create(&new_article).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create(&self, article: &Article) -> Result<Article> {
        self.client.post("/articles", article).await
    }

    /// Update an existing article.
    ///
    /// # Arguments
    ///
    /// * `id` - The article ID (GUID)
    /// * `article` - The updated article data
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use visma_eaccounting::Client;
    /// # async fn example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let mut article = client.articles().get("article-id-123").await?;
    /// article.sales_price = Some(1200.0);
    /// let updated = client.articles().update("article-id-123", &article).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update(&self, id: &str, article: &Article) -> Result<Article> {
        let path = format!("/articles/{}", id);
        self.client.put(&path, article).await
    }

    /// Delete an article.
    ///
    /// # Arguments
    ///
    /// * `id` - The article ID (GUID)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use visma_eaccounting::Client;
    /// # async fn example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// client.articles().delete("article-id-123").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete(&self, id: &str) -> Result<()> {
        let path = format!("/articles/{}", id);
        self.client.delete(&path).await
    }

    /// Search articles with custom query parameters.
    ///
    /// # Arguments
    ///
    /// * `query` - Query parameters for filtering and field selection
    /// * `pagination` - Optional pagination parameters
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use visma_eaccounting::{Client, QueryParams};
    /// # async fn example(client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    /// let query = QueryParams::new()
    ///     .filter("IsActive eq true")
    ///     .select("Id,Name,SalesPrice");
    /// let articles = client.articles().search(query, None).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search(
        &self,
        query: QueryParams,
        pagination: Option<PaginationParams>,
    ) -> Result<PaginatedResponse<Article>> {
        #[derive(serde::Serialize)]
        struct CombinedParams {
            #[serde(flatten)]
            query: QueryParams,
            #[serde(flatten)]
            pagination: Option<PaginationParams>,
        }

        let params = CombinedParams { query, pagination };
        self.client.get_with_params("/articles", &params).await
    }
}
