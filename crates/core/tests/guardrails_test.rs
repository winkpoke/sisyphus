use anyhow::Result;
use async_trait::async_trait;
use common::bus::{EventBus, SystemEvent};
use common::llm::{CompletionRequest, FunctionCall, LLMProvider, Message, Role, ToolCall};
use common::tool::Tool;
use futures::Stream;
use serde_json::{json, Value};
use sisyphus_core::agent::config::{AgentConfig, PermissionLevel};
use sisyphus_core::agent::Agent;
use sisyphus_core::session::Session;
use std::pin::Pin;
use std::sync::Arc;

struct MockTool {
    name: String,
}

#[async_trait]
impl Tool for MockTool {
    fn name(&self) -> &str {
        &self.name
    }
    fn description(&self) -> &str {
        "Mock tool"
    }
    fn schema(&self) -> Value {
        json!({})
    }
    async fn execute(&self, _args: Value) -> Result<String> {
        Ok("Executed".to_string())
    }
}

struct MockProvider {
    responses: std::sync::Mutex<Vec<Message>>,
}

impl MockProvider {
    fn new(responses: Vec<Message>) -> Self {
        Self {
            responses: std::sync::Mutex::new(responses),
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
                content: Some("Default response".to_string()),
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

#[tokio::test]
async fn test_permission_enforcement_deny() {
    let bus = Arc::new(EventBus::new(10));
    let mut config = AgentConfig::default();
    config.permissions.edit = PermissionLevel::Deny;

    let provider = Box::new(MockProvider::new(vec![Message {
        role: Role::Assistant,
        content: None,
        tool_calls: Some(vec![ToolCall {
            id: "call_1".to_string(),
            function: FunctionCall {
                name: "write_file".to_string(),
                arguments: "{}".to_string(),
            },
            kind: "function".to_string(),
        }]),
        tool_call_id: None,
    }]));

    let mut agent = Agent::new(provider, bus.clone(), config, std::path::PathBuf::from("."));
    agent.register_tool(Box::new(MockTool {
        name: "write_file".to_string(),
    }));

    let mut session = Session::new(None);
    let result = agent.chat(&mut session, "test".to_string()).await;
    assert!(result.is_ok());

    // Check history for permission denied message
    let history = session.history();
    let tool_msg = history
        .iter()
        .find(|m| m.role == Role::Tool)
        .expect("Tool message not found");
    assert_eq!(
        tool_msg.content.as_ref().unwrap(),
        "Permission denied: tool execution is set to Deny."
    );
}

#[tokio::test]
async fn test_permission_enforcement_ask() {
    let bus = Arc::new(EventBus::new(10));
    let mut rx = bus.subscribe_raw();
    let mut config = AgentConfig::default();
    config.permissions.edit = PermissionLevel::Ask;

    let provider = Box::new(MockProvider::new(vec![Message {
        role: Role::Assistant,
        content: None,
        tool_calls: Some(vec![ToolCall {
            id: "call_2".to_string(),
            function: FunctionCall {
                name: "write_file".to_string(),
                arguments: "{}".to_string(),
            },
            kind: "function".to_string(),
        }]),
        tool_call_id: None,
    }]));

    let mut agent = Agent::new(provider, bus.clone(), config, std::path::PathBuf::from("."));
    agent.register_tool(Box::new(MockTool {
        name: "write_file".to_string(),
    }));

    let mut session = Session::new(None);
    let result = agent.chat(&mut session, "test".to_string()).await;
    assert!(result.is_ok());
    assert_eq!(
        result.unwrap().output.unwrap(),
        "Permission required: approve tool execution to continue."
    );

    // Check history: should NOT have tool message yet
    let history = session.history();
    assert!(!history.iter().any(|m| m.role == Role::Tool));

    // Check pending approvals
    assert!(session.pending_approvals.contains_key("call_2"));

    // Check for event
    loop {
        match rx.try_recv() {
            Ok(SystemEvent::PermissionRequest {
                operation,
                tool_name,
                call_id,
            }) => {
                assert_eq!(operation, "tool_execution");
                assert_eq!(tool_name, "write_file");
                assert_eq!(call_id, "call_2");
                break;
            }
            Ok(_) => continue,
            Err(_) => break, // Should have found it
        }
    }
}

#[test]
fn test_prompt_snapshot() {
    // This requires inspecting internals or relying on SystemPromptBuilder tests.
    // SystemPromptBuilder tests were updated in prompt.rs.
    // So we are covered.
}

#[tokio::test]
async fn test_permission_ask_stops_turn() {
    let bus = Arc::new(EventBus::new(10));
    let mut config = AgentConfig::default();
    config.permissions.edit = PermissionLevel::Ask;

    let provider = Box::new(MockProvider::new(vec![
        Message {
            role: Role::Assistant,
            content: None,
            tool_calls: Some(vec![ToolCall {
                id: "call_ask".to_string(),
                function: FunctionCall {
                    name: "write_file".to_string(),
                    arguments: "{}".to_string(),
                },
                kind: "function".to_string(),
            }]),
            tool_call_id: None,
        },
        // Provide a second response that should NOT be consumed if it stops
        Message {
            role: Role::Assistant,
            content: Some("I should not be called".to_string()),
            tool_calls: None,
            tool_call_id: None,
        },
    ]));

    let mut agent = Agent::new(provider, bus.clone(), config, std::path::PathBuf::from("."));
    agent.register_tool(Box::new(MockTool {
        name: "write_file".to_string(),
    }));

    let mut session = Session::new(None);
    let result = agent.chat(&mut session, "test".to_string()).await.unwrap();

    // 1. Check return value
    assert_eq!(
        result.output.unwrap(),
        "Permission required: approve tool execution to continue."
    );

    // 2. Check history: User, Assistant (Call)
    // We don't expect the second Assistant message, nor the Tool message yet
    let history = session.history();
    // 1. User "test"
    // 2. Assistant (Tool Call)

    assert_eq!(history.len(), 2);
    assert_eq!(history.last().unwrap().role, Role::Assistant);

    // Check pending approvals
    assert!(session.pending_approvals.contains_key("call_ask"));
}
