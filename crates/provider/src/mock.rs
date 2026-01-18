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

impl Default for MockProvider {
    fn default() -> Self {
        Self::new()
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
            reasoning_summary: None,
            reasoning_raw: None,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_provider_complete() {
        let provider = MockProvider::new();

        let request = CompletionRequest {
            messages: vec![common::llm::Message {
                role: common::llm::Role::User,
                content: Some("Hello".to_string()),
                tool_calls: None,
                tool_call_id: None,
                reasoning_summary: None,
                reasoning_raw: None,
            }],
            temperature: None,
            max_tokens: None,
            tools: None,
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.complete(request).await;

        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(message.role, common::llm::Role::Assistant);
        assert_eq!(message.content, Some("Mock response to: Hello".to_string()));
        assert!(message.tool_calls.is_none());
    }

    #[tokio::test]
    async fn test_mock_provider_complete_empty_messages() {
        let provider = MockProvider::new();

        let request = CompletionRequest {
            messages: vec![],
            temperature: None,
            max_tokens: None,
            tools: None,
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.complete(request).await;

        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(message.content, Some("Mock response to: ".to_string()));
    }

    #[tokio::test]
    async fn test_mock_provider_stream() {
        use futures::StreamExt;

        let provider = MockProvider::new();

        let request = CompletionRequest {
            messages: vec![],
            temperature: None,
            max_tokens: None,
            tools: None,
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.stream(request).await;

        assert!(result.is_ok());
        let stream = result.unwrap();

        let items: Vec<_> = stream.collect().await;
        assert_eq!(items.len(), 3);
    }

    #[tokio::test]
    async fn test_mock_provider_model() {
        let provider = MockProvider::new();

        assert_eq!(provider.model(), "mock-model");
    }

    #[tokio::test]
    async fn test_mock_provider_default() {
        let provider = MockProvider::default();

        let request = CompletionRequest {
            messages: vec![],
            temperature: None,
            max_tokens: None,
            tools: None,
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.complete(request).await;

        assert!(result.is_ok());
    }
}
