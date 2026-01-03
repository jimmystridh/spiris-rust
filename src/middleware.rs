//! Middleware support for request/response interception.
//!
//! This module provides a lightweight middleware pattern for intercepting
//! API requests and responses. Middleware can be used for logging, metrics,
//! custom headers, caching, and more.
//!
//! # Example
//!
//! ```
//! use spiris::middleware::{Middleware, RequestContext, ResponseContext};
//! use spiris::error::Result;
//!
//! struct MyLoggingMiddleware;
//!
//! impl Middleware for MyLoggingMiddleware {
//!     fn on_request(&self, ctx: &mut RequestContext) -> Result<()> {
//!         println!("Request: {} {}", ctx.method, ctx.url);
//!         Ok(())
//!     }
//!
//!     fn on_response(&self, ctx: &ResponseContext) {
//!         println!("Response: {} ({}ms)", ctx.status, ctx.duration.as_millis());
//!     }
//! }
//! ```

use crate::error::Result;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Context provided to middleware before a request is sent.
#[derive(Debug)]
pub struct RequestContext {
    /// HTTP method (GET, POST, PUT, DELETE).
    pub method: String,
    /// Full URL being requested.
    pub url: String,
    /// Request headers that can be modified.
    pub headers: HashMap<String, String>,
    /// Optional request body (for POST/PUT).
    pub body: Option<String>,
    /// Custom data that can be passed between on_request and on_response.
    pub extensions: HashMap<String, String>,
}

impl RequestContext {
    pub(crate) fn new(method: &str, url: &str) -> Self {
        Self {
            method: method.to_string(),
            url: url.to_string(),
            headers: HashMap::new(),
            body: None,
            extensions: HashMap::new(),
        }
    }

    /// Add a custom header to the request.
    pub fn add_header(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.headers.insert(key.into(), value.into());
    }

    /// Store custom data for use in on_response.
    pub fn set_extension(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.extensions.insert(key.into(), value.into());
    }
}

/// Context provided to middleware after a response is received.
#[derive(Debug)]
pub struct ResponseContext {
    /// HTTP method that was used.
    pub method: String,
    /// URL that was requested.
    pub url: String,
    /// HTTP status code.
    pub status: u16,
    /// Time taken for the request.
    pub duration: Duration,
    /// Whether the request was successful (2xx status).
    pub success: bool,
    /// Error message if the request failed.
    pub error: Option<String>,
    /// Extensions from the request context.
    pub extensions: HashMap<String, String>,
}

impl ResponseContext {
    pub(crate) fn new(
        method: String,
        url: String,
        status: u16,
        duration: Duration,
        extensions: HashMap<String, String>,
    ) -> Self {
        Self {
            method,
            url,
            status,
            success: (200..300).contains(&status),
            duration,
            error: None,
            extensions,
        }
    }

    pub(crate) fn with_error(
        method: String,
        url: String,
        duration: Duration,
        error: String,
        extensions: HashMap<String, String>,
    ) -> Self {
        Self {
            method,
            url,
            status: 0,
            success: false,
            duration,
            error: Some(error),
            extensions,
        }
    }
}

/// Trait for implementing request/response middleware.
///
/// Middleware can intercept requests before they are sent and responses
/// after they are received. This is useful for logging, metrics collection,
/// adding custom headers, and more.
///
/// # Example
///
/// ```
/// use spiris::middleware::{Middleware, RequestContext, ResponseContext};
/// use spiris::error::Result;
///
/// struct TimingMiddleware;
///
/// impl Middleware for TimingMiddleware {
///     fn on_request(&self, ctx: &mut RequestContext) -> Result<()> {
///         // Store request ID for correlation
///         ctx.set_extension("request_id", "12345");
///         Ok(())
///     }
///
///     fn on_response(&self, ctx: &ResponseContext) {
///         println!(
///             "[{}] {} {} - {}ms",
///             ctx.extensions.get("request_id").unwrap_or(&"unknown".to_string()),
///             ctx.method,
///             ctx.url,
///             ctx.duration.as_millis()
///         );
///     }
/// }
/// ```
pub trait Middleware: Send + Sync {
    /// Called before a request is sent.
    ///
    /// Return `Ok(())` to continue with the request, or an error to abort.
    /// You can modify the request context to add headers or store data.
    fn on_request(&self, ctx: &mut RequestContext) -> Result<()> {
        let _ = ctx;
        Ok(())
    }

    /// Called after a response is received (or an error occurs).
    ///
    /// This is always called, even if the request failed.
    fn on_response(&self, ctx: &ResponseContext) {
        let _ = ctx;
    }

    /// Optional name for debugging/logging purposes.
    fn name(&self) -> &'static str {
        "unnamed"
    }
}

/// A stack of middleware that processes requests in order.
#[derive(Default, Clone)]
pub struct MiddlewareStack {
    middlewares: Vec<Arc<dyn Middleware>>,
}

impl MiddlewareStack {
    /// Create a new empty middleware stack.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add middleware to the stack.
    ///
    /// Middleware is executed in the order it is added for requests,
    /// and in reverse order for responses.
    pub fn push<M: Middleware + 'static>(&mut self, middleware: M) {
        self.middlewares.push(Arc::new(middleware));
    }

    /// Add middleware and return self for chaining.
    pub fn with<M: Middleware + 'static>(mut self, middleware: M) -> Self {
        self.push(middleware);
        self
    }

    /// Execute all middleware on_request handlers.
    pub(crate) fn process_request(&self, ctx: &mut RequestContext) -> Result<()> {
        for middleware in &self.middlewares {
            middleware.on_request(ctx)?;
        }
        Ok(())
    }

    /// Execute all middleware on_response handlers (in reverse order).
    pub(crate) fn process_response(&self, ctx: &ResponseContext) {
        for middleware in self.middlewares.iter().rev() {
            middleware.on_response(ctx);
        }
    }

    /// Check if the stack is empty.
    pub fn is_empty(&self) -> bool {
        self.middlewares.is_empty()
    }

    /// Get the number of middlewares in the stack.
    pub fn len(&self) -> usize {
        self.middlewares.len()
    }
}

impl std::fmt::Debug for MiddlewareStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MiddlewareStack")
            .field("count", &self.middlewares.len())
            .finish()
    }
}

/// A request timing tracker for use with middleware.
#[derive(Debug)]
pub struct RequestTimer {
    start: Instant,
}

impl RequestTimer {
    /// Start a new timer.
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Get the elapsed duration.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

// ============================================================================
// Built-in Middleware Implementations
// ============================================================================

/// Middleware that logs all requests and responses.
///
/// # Example
///
/// ```no_run
/// use spiris::middleware::LoggingMiddleware;
/// use spiris::ClientConfig;
///
/// let config = ClientConfig::new()
///     .middleware(LoggingMiddleware::new());
/// ```
#[derive(Debug, Clone)]
pub struct LoggingMiddleware {
    log_bodies: bool,
}

impl LoggingMiddleware {
    /// Create a new logging middleware.
    pub fn new() -> Self {
        Self { log_bodies: false }
    }

    /// Enable logging of request/response bodies.
    pub fn with_bodies(mut self) -> Self {
        self.log_bodies = true;
        self
    }
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for LoggingMiddleware {
    fn on_request(&self, ctx: &mut RequestContext) -> Result<()> {
        if self.log_bodies {
            if let Some(ref body) = ctx.body {
                println!("[API] → {} {} body={}", ctx.method, ctx.url, body);
            } else {
                println!("[API] → {} {}", ctx.method, ctx.url);
            }
        } else {
            println!("[API] → {} {}", ctx.method, ctx.url);
        }
        Ok(())
    }

    fn on_response(&self, ctx: &ResponseContext) {
        if ctx.success {
            println!(
                "[API] ← {} {} {} ({}ms)",
                ctx.method,
                ctx.url,
                ctx.status,
                ctx.duration.as_millis()
            );
        } else if let Some(ref error) = ctx.error {
            println!(
                "[API] ✗ {} {} error={} ({}ms)",
                ctx.method,
                ctx.url,
                error,
                ctx.duration.as_millis()
            );
        } else {
            println!(
                "[API] ✗ {} {} {} ({}ms)",
                ctx.method,
                ctx.url,
                ctx.status,
                ctx.duration.as_millis()
            );
        }
    }

    fn name(&self) -> &'static str {
        "logging"
    }
}

/// Middleware that adds custom headers to all requests.
///
/// # Example
///
/// ```
/// use spiris::middleware::HeadersMiddleware;
///
/// let middleware = HeadersMiddleware::new()
///     .add("X-Custom-Header", "value")
///     .add("X-Request-Source", "my-app");
/// ```
#[derive(Debug, Clone, Default)]
pub struct HeadersMiddleware {
    headers: HashMap<String, String>,
}

impl HeadersMiddleware {
    /// Create a new headers middleware.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a header to be included in all requests.
    pub fn add(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

impl Middleware for HeadersMiddleware {
    fn on_request(&self, ctx: &mut RequestContext) -> Result<()> {
        for (key, value) in &self.headers {
            ctx.add_header(key.clone(), value.clone());
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "headers"
    }
}

/// Middleware that collects request metrics.
///
/// This middleware tracks request counts, success/failure rates, and timing.
/// Metrics can be retrieved using the `metrics()` method.
///
/// # Example
///
/// ```
/// use spiris::middleware::MetricsMiddleware;
/// use std::sync::Arc;
///
/// let metrics = Arc::new(MetricsMiddleware::new());
///
/// // After some requests...
/// // let stats = metrics.metrics();
/// // println!("Total requests: {}", stats.total_requests);
/// ```
#[derive(Debug)]
pub struct MetricsMiddleware {
    metrics: std::sync::RwLock<Metrics>,
}

/// Collected metrics from API requests.
#[derive(Debug, Clone, Default)]
pub struct Metrics {
    /// Total number of requests made.
    pub total_requests: u64,
    /// Number of successful requests (2xx status).
    pub successful_requests: u64,
    /// Number of failed requests.
    pub failed_requests: u64,
    /// Total time spent on requests.
    pub total_duration: Duration,
    /// Requests by HTTP method.
    pub requests_by_method: HashMap<String, u64>,
    /// Requests by status code.
    pub requests_by_status: HashMap<u16, u64>,
}

impl MetricsMiddleware {
    /// Create a new metrics middleware.
    pub fn new() -> Self {
        Self {
            metrics: std::sync::RwLock::new(Metrics::default()),
        }
    }

    /// Get a snapshot of the current metrics.
    pub fn metrics(&self) -> Metrics {
        self.metrics.read().unwrap().clone()
    }

    /// Reset all metrics to zero.
    pub fn reset(&self) {
        let mut metrics = self.metrics.write().unwrap();
        *metrics = Metrics::default();
    }
}

impl Default for MetricsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl Middleware for MetricsMiddleware {
    fn on_response(&self, ctx: &ResponseContext) {
        let mut metrics = self.metrics.write().unwrap();
        metrics.total_requests += 1;
        metrics.total_duration += ctx.duration;

        if ctx.success {
            metrics.successful_requests += 1;
        } else {
            metrics.failed_requests += 1;
        }

        *metrics
            .requests_by_method
            .entry(ctx.method.clone())
            .or_insert(0) += 1;

        if ctx.status > 0 {
            *metrics.requests_by_status.entry(ctx.status).or_insert(0) += 1;
        }
    }

    fn name(&self) -> &'static str {
        "metrics"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_request_context_creation() {
        let ctx = RequestContext::new("GET", "https://api.example.com/test");
        assert_eq!(ctx.method, "GET");
        assert_eq!(ctx.url, "https://api.example.com/test");
        assert!(ctx.headers.is_empty());
        assert!(ctx.body.is_none());
    }

    #[test]
    fn test_request_context_add_header() {
        let mut ctx = RequestContext::new("POST", "https://api.example.com/test");
        ctx.add_header("X-Custom", "value");
        assert_eq!(ctx.headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_request_context_extensions() {
        let mut ctx = RequestContext::new("GET", "https://api.example.com/test");
        ctx.set_extension("request_id", "12345");
        assert_eq!(
            ctx.extensions.get("request_id"),
            Some(&"12345".to_string())
        );
    }

    #[test]
    fn test_response_context_success() {
        let ctx = ResponseContext::new(
            "GET".to_string(),
            "https://api.example.com/test".to_string(),
            200,
            Duration::from_millis(100),
            HashMap::new(),
        );
        assert!(ctx.success);
        assert_eq!(ctx.status, 200);
        assert!(ctx.error.is_none());
    }

    #[test]
    fn test_response_context_failure() {
        let ctx = ResponseContext::new(
            "GET".to_string(),
            "https://api.example.com/test".to_string(),
            404,
            Duration::from_millis(50),
            HashMap::new(),
        );
        assert!(!ctx.success);
        assert_eq!(ctx.status, 404);
    }

    #[test]
    fn test_response_context_with_error() {
        let ctx = ResponseContext::with_error(
            "GET".to_string(),
            "https://api.example.com/test".to_string(),
            Duration::from_millis(10),
            "Connection refused".to_string(),
            HashMap::new(),
        );
        assert!(!ctx.success);
        assert_eq!(ctx.status, 0);
        assert_eq!(ctx.error, Some("Connection refused".to_string()));
    }

    #[test]
    fn test_middleware_stack_empty() {
        let stack = MiddlewareStack::new();
        assert!(stack.is_empty());
        assert_eq!(stack.len(), 0);
    }

    #[test]
    fn test_middleware_stack_push() {
        let mut stack = MiddlewareStack::new();
        stack.push(LoggingMiddleware::new());
        assert!(!stack.is_empty());
        assert_eq!(stack.len(), 1);
    }

    #[test]
    fn test_middleware_stack_with() {
        let stack = MiddlewareStack::new()
            .with(LoggingMiddleware::new())
            .with(HeadersMiddleware::new());
        assert_eq!(stack.len(), 2);
    }

    #[test]
    fn test_headers_middleware() {
        let middleware = HeadersMiddleware::new()
            .add("X-Api-Key", "secret")
            .add("X-Client", "test");

        let mut ctx = RequestContext::new("GET", "https://api.example.com");
        middleware.on_request(&mut ctx).unwrap();

        assert_eq!(ctx.headers.get("X-Api-Key"), Some(&"secret".to_string()));
        assert_eq!(ctx.headers.get("X-Client"), Some(&"test".to_string()));
    }

    #[test]
    fn test_metrics_middleware() {
        let middleware = MetricsMiddleware::new();

        // Simulate successful response
        let ctx1 = ResponseContext::new(
            "GET".to_string(),
            "https://api.example.com/test".to_string(),
            200,
            Duration::from_millis(100),
            HashMap::new(),
        );
        middleware.on_response(&ctx1);

        // Simulate failed response
        let ctx2 = ResponseContext::new(
            "POST".to_string(),
            "https://api.example.com/test".to_string(),
            400,
            Duration::from_millis(50),
            HashMap::new(),
        );
        middleware.on_response(&ctx2);

        let metrics = middleware.metrics();
        assert_eq!(metrics.total_requests, 2);
        assert_eq!(metrics.successful_requests, 1);
        assert_eq!(metrics.failed_requests, 1);
        assert_eq!(metrics.total_duration, Duration::from_millis(150));
        assert_eq!(metrics.requests_by_method.get("GET"), Some(&1));
        assert_eq!(metrics.requests_by_method.get("POST"), Some(&1));
        assert_eq!(metrics.requests_by_status.get(&200), Some(&1));
        assert_eq!(metrics.requests_by_status.get(&400), Some(&1));
    }

    #[test]
    fn test_metrics_middleware_reset() {
        let middleware = MetricsMiddleware::new();

        let ctx = ResponseContext::new(
            "GET".to_string(),
            "https://api.example.com/test".to_string(),
            200,
            Duration::from_millis(100),
            HashMap::new(),
        );
        middleware.on_response(&ctx);

        assert_eq!(middleware.metrics().total_requests, 1);

        middleware.reset();

        assert_eq!(middleware.metrics().total_requests, 0);
    }

    #[test]
    fn test_request_timer() {
        let timer = RequestTimer::start();
        std::thread::sleep(Duration::from_millis(10));
        let elapsed = timer.elapsed();
        assert!(elapsed >= Duration::from_millis(10));
    }

    #[test]
    fn test_middleware_stack_execution_order() {
        let stack = MiddlewareStack::new()
            .with(LoggingMiddleware::new())
            .with(HeadersMiddleware::new().add("X-Test", "value"));

        let mut ctx = RequestContext::new("GET", "https://api.example.com");
        stack.process_request(&mut ctx).unwrap();

        // Headers middleware should have added the header
        assert_eq!(ctx.headers.get("X-Test"), Some(&"value".to_string()));
    }
}
