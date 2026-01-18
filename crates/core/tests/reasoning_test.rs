use anyhow::Result;
use async_trait::async_trait;
use common::bus::{EventBus, SystemEvent};
use common::llm::{CompletionRequest, LLMProvider, Message, Role};
use futures::Stream;
use sisyphus_core::agent::config::AgentConfig;
use sisyphus_core::agent::Agent;
use sisyphus_core::session::Session;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

struct MockProvider {
    responses: Arc<Mutex<Vec<Message>>>,
}

impl MockProvider {
    fn new(responses: Vec<Message>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
        }
    }
}

#[async_trait]
impl LLMProvider for MockProvider {
    fn model(&self) -> String {
        "mock-model".to_string()
    }

    async fn complete(&self, _request: CompletionRequest) -> Result<Message> {
        let mut responses = self.responses.lock().unwrap();
        if !responses.is_empty() {
            Ok(responses.remove(0))
        } else {
            Ok(Message {
                role: Role::Assistant,
                content: Some("Done".to_string()),
                tool_calls: None,
                tool_call_id: None,
                reasoning_summary: None,
                reasoning_raw: None,
            })
        }
    }

    async fn stream(
        &self,
        _request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        unimplemented!()
    }
}

#[tokio::test]
async fn test_reasoning_event_publishing() {
    let reasoning_content = "This is the reasoning content.";
    let response = Message {
        role: Role::Assistant,
        content: Some("Final answer".to_string()),
        tool_calls: None,
        tool_call_id: None,
        reasoning_summary: Some("Thought for 1s".to_string()),
        reasoning_raw: Some(reasoning_content.to_string()),
    };

    let provider = Box::new(MockProvider::new(vec![response]));
    let bus = Arc::new(EventBus::new(100));
    let mut rx = bus.subscribe_raw();

    let config = AgentConfig::default();
    let agent = Agent::new(provider, bus.clone(), config, PathBuf::from("."));
    let mut session = Session::new(None);

    let _ = agent.chat(&mut session, "Hello".to_string()).await.unwrap();

    // Check events
    let mut found_reasoning = false;
    let mut found_answer = false;

    // We expect multiple events. We'll drain the channel.
    while let Ok(envelope) = rx.try_recv() {
        if let SystemEvent::MessageReceived { content, kind, .. } = envelope.event {
            if let Some(k) = kind {
                if k == "reasoning_summary" && content == reasoning_content {
                    found_reasoning = true;
                }
            } else {
                if content == "Final answer" {
                    found_answer = true;
                }
            }
        }
    }

    assert!(found_reasoning, "Should have received reasoning event with raw content");
    assert!(found_answer, "Should have received final answer event");
}
