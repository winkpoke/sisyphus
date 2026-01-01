use common::llm::{LLMProvider, CompletionRequest};
use async_trait::async_trait;
use anyhow::Result;
use futures::Stream;
use std::pin::Pin;

pub struct MockProvider;

impl MockProvider {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl LLMProvider for MockProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<String> {
        let last_msg = request.messages.last().map(|m| m.content.as_str()).unwrap_or("");
        Ok(format!("Mock response to: {}", last_msg))
    }

    async fn stream(&self, _request: CompletionRequest) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        Ok(Box::pin(futures::stream::iter(vec![
            Ok("Mock ".to_string()),
            Ok("streaming ".to_string()),
            Ok("response".to_string()),
        ])))
    }
}
