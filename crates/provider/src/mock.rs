use anyhow::Result;
use async_trait::async_trait;
use common::llm::{CompletionRequest, LLMProvider, Message, Role};
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
    async fn complete(&self, request: CompletionRequest) -> Result<Message> {
        let last_msg = request
            .messages
            .last()
            .and_then(|m| m.content.clone())
            .unwrap_or_default();
        Ok(Message {
            role: Role::Assistant,
            content: Some(format!("Mock response to: {}", last_msg)),
            tool_calls: None,
            tool_call_id: None,
        })
    }

    async fn stream(
        &self,
        _request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        Ok(Box::pin(futures::stream::iter(vec![
            Ok("Mock ".to_string()),
            Ok("streaming ".to_string()),
            Ok("response".to_string()),
        ])))
    }

    fn model(&self) -> String {
        "mock-model".to_string()
    }
}
