use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use futures::Stream;
use std::pin::Pin;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub role: Role,
    pub content: String,
}

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<String>;
    
    // For object safety with streaming, we return a pinned box stream.
    // The stream yields chunks of content (Strings).
    async fn stream(&self, request: CompletionRequest) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>>;
}
