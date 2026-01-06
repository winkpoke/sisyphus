use anyhow::Result;
use async_trait::async_trait;
use common::bus::EventBus;
use common::llm::{CompletionRequest, FunctionCall, LLMProvider, Message, Role, ToolCall};
use common::tool::Tool;
use futures::Stream;
use sisyphus_core::agent::config::{AgentConfig, PermissionLevel};
use sisyphus_core::agent::Agent;
use sisyphus_core::session::{Session, SessionStatus};
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
                content: Some("I understand you denied.".to_string()),
                tool_calls: None,
                tool_call_id: None,
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

struct MockTool {
    name: String,
}

#[async_trait]
impl Tool for MockTool {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        "A mock tool"
    }

    fn schema(&self) -> serde_json::Value {
        serde_json::json!({})
    }

    async fn execute(&self, _args: serde_json::Value) -> Result<String> {
        Ok(format!("Executed {}", self.name))
    }
}

#[tokio::test]
async fn test_deny_tool_execution() {
    let tool_calls = vec![ToolCall {
        id: "call_1".to_string(),
        function: FunctionCall {
            name: "ask_tool".to_string(),
            arguments: "{}".to_string(),
        },
        kind: "function".to_string(),
    }];

    let initial_response = Message {
        role: Role::Assistant,
        content: Some("Thinking...".to_string()),
        tool_calls: Some(tool_calls),
        tool_call_id: None,
    };

    let provider = Box::new(MockProvider::new(vec![initial_response]));
    let bus = Arc::new(EventBus::new(100));

    let mut config = AgentConfig::default();
    config
        .permissions
        .overrides
        .insert("ask_tool".to_string(), PermissionLevel::Ask);

    let mut agent = Agent::new(provider, bus, config, PathBuf::from("."));
    agent.register_tool(Box::new(MockTool {
        name: "ask_tool".to_string(),
    }));

    let mut session = Session::new(None);

    // 1. Start chat
    let result = agent.chat(&mut session, "Do it".to_string()).await.unwrap();

    // Expect "Permission required"
    assert!(result
        .output
        .as_ref()
        .unwrap()
        .contains("Permission required"));

    // Check pending
    assert_eq!(session.pending_approvals.len(), 1);
    assert!(session.pending_approvals.contains_key("call_1"));

    // 2. Deny call_1
    let res = agent
        .resolve_approval(&mut session, "call_1", false)
        .await
        .unwrap();

    println!("Result after deny: {}", res);

    let history = session.history();
    for msg in history {
        println!("{:?}: {:?}", msg.role, msg.content);
    }
}

#[tokio::test]
async fn test_resolve_approval_error_resets_session_status() {
    // Setup
    let provider = Box::new(MockProvider::new(vec![]));
    let bus = Arc::new(EventBus::new(100));
    let config = AgentConfig::default();
    let agent = Agent::new(provider, bus, config, PathBuf::from("."));

    let mut session = Session::new(None);

    // Ensure initial status is Idle
    assert_eq!(session.status, SessionStatus::Idle);

    // Call resolve_approval with an invalid call_id to force an error
    let result = agent
        .resolve_approval(&mut session, "invalid_id", true)
        .await;

    // Expect an error
    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err().to_string(),
        "No pending approval found for call_id: invalid_id"
    );

    // CRITICAL ASSERTION: Status must be Idle after error
    assert_eq!(
        session.status,
        SessionStatus::Idle,
        "Session status should be Idle after error"
    );

    // Ensure we can still use the session
    let chat_result = agent.chat(&mut session, "Hello".to_string()).await;
    assert!(chat_result.is_ok(), "Should be able to chat after error");
}
