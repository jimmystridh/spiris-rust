//! Message threads API endpoint.

use crate::client::Client;
use crate::error::Result;
use crate::types::{Message, MessageThread};

pub struct MessagesEndpoint<'a> {
    client: &'a Client,
}

impl<'a> MessagesEndpoint<'a> {
    pub(crate) fn new(client: &'a Client) -> Self {
        Self { client }
    }

    pub async fn get_thread(&self, id: &str) -> Result<MessageThread> {
        self.client.get(&format!("/messagethreads/{}", id)).await
    }

    pub async fn update_thread(&self, id: &str, thread: &MessageThread) -> Result<MessageThread> {
        self.client
            .put(&format!("/messagethreads/{}", id), thread)
            .await
    }

    pub async fn add_message(&self, thread_id: &str, message: &Message) -> Result<Message> {
        self.client
            .post(&format!("/messagethreads/{}", thread_id), message)
            .await
    }
}
