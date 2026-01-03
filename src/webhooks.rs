//! Webhook support for receiving event notifications.
//!
//! This module provides infrastructure for handling webhook callbacks.
//! While the Visma eAccounting API currently uses a polling-based model,
//! this module provides utilities that can be used with third-party
//! integrations or future API webhook support.
//!
//! # Features
//!
//! - Webhook payload parsing and validation
//! - HMAC signature verification
//! - Event type handling
//! - Typed event payloads
//!
//! # Example
//!
//! ```
//! use spiris::webhooks::{WebhookHandler, WebhookEvent, WebhookConfig};
//!
//! // Create a webhook handler with a signing secret
//! let config = WebhookConfig::new("your_webhook_secret");
//! let handler = WebhookHandler::new(config);
//!
//! // In your webhook endpoint handler:
//! // let event = handler.verify_and_parse(payload, signature)?;
//! // match event.event_type.as_str() {
//! //     "invoice.created" => { /* handle invoice created */ }
//! //     "customer.updated" => { /* handle customer updated */ }
//! //     _ => { /* unknown event */ }
//! // }
//! ```

use crate::error::{Error, Result};
use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Configuration for webhook handling.
#[derive(Debug, Clone)]
pub struct WebhookConfig {
    /// Secret key used for HMAC signature verification.
    pub signing_secret: String,

    /// Expected signature header name.
    pub signature_header: String,

    /// Timestamp tolerance in seconds (to prevent replay attacks).
    pub timestamp_tolerance_secs: u64,
}

impl WebhookConfig {
    /// Create a new webhook configuration with a signing secret.
    pub fn new(signing_secret: impl Into<String>) -> Self {
        Self {
            signing_secret: signing_secret.into(),
            signature_header: "X-Webhook-Signature".to_string(),
            timestamp_tolerance_secs: 300, // 5 minutes
        }
    }

    /// Set the expected signature header name.
    pub fn signature_header(mut self, header: impl Into<String>) -> Self {
        self.signature_header = header.into();
        self
    }

    /// Set the timestamp tolerance for replay attack prevention.
    pub fn timestamp_tolerance_secs(mut self, secs: u64) -> Self {
        self.timestamp_tolerance_secs = secs;
        self
    }
}

/// A webhook event received from the API.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookEvent {
    /// Unique identifier for this event.
    pub id: String,

    /// Type of event (e.g., "invoice.created", "customer.updated").
    pub event_type: String,

    /// Timestamp when the event occurred (Unix timestamp).
    pub timestamp: i64,

    /// The resource type that triggered the event.
    #[serde(default)]
    pub resource_type: Option<String>,

    /// The ID of the resource that triggered the event.
    #[serde(default)]
    pub resource_id: Option<String>,

    /// The raw payload data.
    #[serde(default)]
    pub data: serde_json::Value,
}

impl WebhookEvent {
    /// Check if this event is of a specific type.
    pub fn is_type(&self, event_type: &str) -> bool {
        self.event_type == event_type
    }

    /// Get the event category (e.g., "invoice" from "invoice.created").
    pub fn category(&self) -> Option<&str> {
        self.event_type.split('.').next()
    }

    /// Get the event action (e.g., "created" from "invoice.created").
    pub fn action(&self) -> Option<&str> {
        self.event_type.split('.').nth(1)
    }

    /// Try to deserialize the data payload into a specific type.
    pub fn data_as<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        serde_json::from_value(self.data.clone())
            .map_err(|e| Error::InvalidRequest(format!("Failed to parse webhook data: {}", e)))
    }
}

/// Handler for processing and validating webhook requests.
#[derive(Debug, Clone)]
pub struct WebhookHandler {
    config: WebhookConfig,
}

impl WebhookHandler {
    /// Create a new webhook handler with the given configuration.
    pub fn new(config: WebhookConfig) -> Self {
        Self { config }
    }

    /// Verify the webhook signature and parse the payload.
    ///
    /// # Arguments
    ///
    /// * `payload` - The raw request body
    /// * `signature` - The signature from the webhook header
    ///
    /// # Returns
    ///
    /// The parsed webhook event if the signature is valid.
    pub fn verify_and_parse(&self, payload: &[u8], signature: &str) -> Result<WebhookEvent> {
        self.verify_signature(payload, signature)?;
        self.parse_payload(payload)
    }

    /// Verify only the webhook signature without parsing.
    ///
    /// # Arguments
    ///
    /// * `payload` - The raw request body
    /// * `signature` - The signature from the webhook header
    pub fn verify_signature(&self, payload: &[u8], signature: &str) -> Result<()> {
        let mut mac = HmacSha256::new_from_slice(self.config.signing_secret.as_bytes())
            .map_err(|e| Error::InvalidRequest(format!("Invalid signing secret: {}", e)))?;

        mac.update(payload);

        // Decode the provided signature (expected to be hex-encoded)
        let provided_sig = hex::decode(signature.trim())
            .map_err(|_| Error::AuthError("Invalid signature format".to_string()))?;

        mac.verify_slice(&provided_sig)
            .map_err(|_| Error::AuthError("Invalid webhook signature".to_string()))?;

        Ok(())
    }

    /// Parse the webhook payload without signature verification.
    ///
    /// Use this only if you've already verified the signature through other means.
    pub fn parse_payload(&self, payload: &[u8]) -> Result<WebhookEvent> {
        serde_json::from_slice(payload)
            .map_err(|e| Error::InvalidRequest(format!("Failed to parse webhook payload: {}", e)))
    }

    /// Get the expected signature header name.
    pub fn signature_header(&self) -> &str {
        &self.config.signature_header
    }
}

/// Common webhook event types.
pub mod event_types {
    // Invoice events
    pub const INVOICE_CREATED: &str = "invoice.created";
    pub const INVOICE_UPDATED: &str = "invoice.updated";
    pub const INVOICE_DELETED: &str = "invoice.deleted";
    pub const INVOICE_SENT: &str = "invoice.sent";
    pub const INVOICE_PAID: &str = "invoice.paid";

    // Customer events
    pub const CUSTOMER_CREATED: &str = "customer.created";
    pub const CUSTOMER_UPDATED: &str = "customer.updated";
    pub const CUSTOMER_DELETED: &str = "customer.deleted";

    // Article events
    pub const ARTICLE_CREATED: &str = "article.created";
    pub const ARTICLE_UPDATED: &str = "article.updated";
    pub const ARTICLE_DELETED: &str = "article.deleted";

    // Supplier events
    pub const SUPPLIER_CREATED: &str = "supplier.created";
    pub const SUPPLIER_UPDATED: &str = "supplier.updated";
    pub const SUPPLIER_DELETED: &str = "supplier.deleted";

    // Payment events
    pub const PAYMENT_RECEIVED: &str = "payment.received";
    pub const PAYMENT_SENT: &str = "payment.sent";
}

/// Builder for creating webhook events (useful for testing).
#[derive(Debug, Default)]
pub struct WebhookEventBuilder {
    id: Option<String>,
    event_type: Option<String>,
    timestamp: Option<i64>,
    resource_type: Option<String>,
    resource_id: Option<String>,
    data: Option<serde_json::Value>,
}

impl WebhookEventBuilder {
    /// Create a new event builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the event ID.
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the event type.
    pub fn event_type(mut self, event_type: impl Into<String>) -> Self {
        self.event_type = Some(event_type.into());
        self
    }

    /// Set the timestamp.
    pub fn timestamp(mut self, timestamp: i64) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Set the resource type.
    pub fn resource_type(mut self, resource_type: impl Into<String>) -> Self {
        self.resource_type = Some(resource_type.into());
        self
    }

    /// Set the resource ID.
    pub fn resource_id(mut self, resource_id: impl Into<String>) -> Self {
        self.resource_id = Some(resource_id.into());
        self
    }

    /// Set the data payload.
    pub fn data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    /// Build the webhook event.
    pub fn build(self) -> WebhookEvent {
        WebhookEvent {
            id: self.id.unwrap_or_else(uuid_v4),
            event_type: self.event_type.unwrap_or_else(|| "unknown".to_string()),
            timestamp: self.timestamp.unwrap_or_else(|| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64
            }),
            resource_type: self.resource_type,
            resource_id: self.resource_id,
            data: self.data.unwrap_or(serde_json::Value::Null),
        }
    }
}

/// Generate a simple UUID v4 (for event IDs in testing).
fn uuid_v4() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    format!("{:032x}", timestamp)
}

/// Utility for creating test webhook payloads with valid signatures.
#[derive(Debug)]
pub struct WebhookTestHelper {
    secret: String,
}

impl WebhookTestHelper {
    /// Create a new test helper with the given secret.
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }

    /// Sign a payload and return the signature.
    pub fn sign(&self, payload: &[u8]) -> String {
        let mut mac = HmacSha256::new_from_slice(self.secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(payload);
        hex::encode(mac.finalize().into_bytes())
    }

    /// Create a signed payload from an event.
    pub fn create_signed_payload(&self, event: &WebhookEvent) -> (Vec<u8>, String) {
        let payload = serde_json::to_vec(event).expect("Failed to serialize event");
        let signature = self.sign(&payload);
        (payload, signature)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> WebhookConfig {
        WebhookConfig::new("test_secret_key")
    }

    fn test_handler() -> WebhookHandler {
        WebhookHandler::new(test_config())
    }

    #[test]
    fn test_webhook_config_new() {
        let config = WebhookConfig::new("my_secret");
        assert_eq!(config.signing_secret, "my_secret");
        assert_eq!(config.signature_header, "X-Webhook-Signature");
        assert_eq!(config.timestamp_tolerance_secs, 300);
    }

    #[test]
    fn test_webhook_config_builder() {
        let config = WebhookConfig::new("secret")
            .signature_header("X-Custom-Sig")
            .timestamp_tolerance_secs(600);

        assert_eq!(config.signature_header, "X-Custom-Sig");
        assert_eq!(config.timestamp_tolerance_secs, 600);
    }

    #[test]
    fn test_webhook_event_builder() {
        let event = WebhookEventBuilder::new()
            .id("evt_123")
            .event_type("invoice.created")
            .resource_type("invoice")
            .resource_id("inv_456")
            .timestamp(1234567890)
            .build();

        assert_eq!(event.id, "evt_123");
        assert_eq!(event.event_type, "invoice.created");
        assert_eq!(event.resource_type, Some("invoice".to_string()));
        assert_eq!(event.resource_id, Some("inv_456".to_string()));
        assert_eq!(event.timestamp, 1234567890);
    }

    #[test]
    fn test_webhook_event_category_and_action() {
        let event = WebhookEventBuilder::new()
            .event_type("invoice.created")
            .build();

        assert_eq!(event.category(), Some("invoice"));
        assert_eq!(event.action(), Some("created"));
    }

    #[test]
    fn test_webhook_event_is_type() {
        let event = WebhookEventBuilder::new()
            .event_type("customer.updated")
            .build();

        assert!(event.is_type("customer.updated"));
        assert!(!event.is_type("customer.created"));
    }

    #[test]
    fn test_webhook_test_helper_sign() {
        let helper = WebhookTestHelper::new("secret");
        let payload = b"test payload";
        let signature = helper.sign(payload);

        assert!(!signature.is_empty());
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_verify_valid_signature() {
        let handler = test_handler();
        let helper = WebhookTestHelper::new("test_secret_key");

        let event = WebhookEventBuilder::new()
            .event_type("invoice.created")
            .build();

        let (payload, signature) = helper.create_signed_payload(&event);

        let result = handler.verify_and_parse(&payload, &signature);
        assert!(result.is_ok());

        let parsed = result.unwrap();
        assert_eq!(parsed.event_type, "invoice.created");
    }

    #[test]
    fn test_verify_invalid_signature() {
        let handler = test_handler();

        let payload = b"{\"event_type\":\"test\",\"id\":\"1\",\"timestamp\":0}";
        let bad_signature = "0000000000000000000000000000000000000000000000000000000000000000";

        let result = handler.verify_signature(payload, bad_signature);
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_malformed_signature() {
        let handler = test_handler();
        let payload = b"test";

        let result = handler.verify_signature(payload, "not-hex");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_payload() {
        let handler = test_handler();

        let payload = br#"{
            "id": "evt_123",
            "eventType": "invoice.paid",
            "timestamp": 1234567890,
            "resourceType": "invoice",
            "resourceId": "inv_456",
            "data": {"amount": 1000}
        }"#;

        let event = handler.parse_payload(payload).unwrap();

        assert_eq!(event.id, "evt_123");
        assert_eq!(event.event_type, "invoice.paid");
        assert_eq!(event.timestamp, 1234567890);
        assert_eq!(event.resource_type, Some("invoice".to_string()));
        assert_eq!(event.resource_id, Some("inv_456".to_string()));
        assert_eq!(event.data["amount"], 1000);
    }

    #[test]
    fn test_parse_invalid_payload() {
        let handler = test_handler();
        let result = handler.parse_payload(b"not json");
        assert!(result.is_err());
    }

    #[test]
    fn test_event_data_as() {
        #[derive(Debug, Deserialize, PartialEq)]
        struct InvoiceData {
            amount: i64,
            currency: String,
        }

        let event = WebhookEventBuilder::new()
            .event_type("invoice.paid")
            .data(serde_json::json!({
                "amount": 1000,
                "currency": "SEK"
            }))
            .build();

        let data: InvoiceData = event.data_as().unwrap();
        assert_eq!(data.amount, 1000);
        assert_eq!(data.currency, "SEK");
    }

    #[test]
    fn test_event_types_constants() {
        assert_eq!(event_types::INVOICE_CREATED, "invoice.created");
        assert_eq!(event_types::CUSTOMER_UPDATED, "customer.updated");
        assert_eq!(event_types::PAYMENT_RECEIVED, "payment.received");
    }
}
