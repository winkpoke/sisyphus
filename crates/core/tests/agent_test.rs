use anyhow::Result;
use async_trait::async_trait;
use common::bus::EventBus;
use common::llm::{CompletionRequest, FunctionCall, LLMProvider, Message, Role, ToolCall};
use common::tool::Tool;
use futures::Stream;
use sisyphus_core::agent::config::{AgentConfig, PermissionLevel};
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
async fn test_sequential_tool_execution() {
    let tool_calls = vec![
        ToolCall {
            id: "call_1".to_string(),
            function: FunctionCall {
                name: "allow_tool".to_string(),
                arguments: "{}".to_string(),
            },
            kind: "function".to_string(),
        },
        ToolCall {
            id: "call_2".to_string(),
            function: FunctionCall {
                name: "ask_tool".to_string(),
                arguments: "{}".to_string(),
            },
            kind: "function".to_string(),
        },
        ToolCall {
            id: "call_3".to_string(),
            function: FunctionCall {
                name: "allow_tool_2".to_string(),
                arguments: "{}".to_string(),
            },
            kind: "function".to_string(),
        },
    ];

    let initial_response = Message {
        role: Role::Assistant,
        content: Some("Thinking...".to_string()),
        tool_calls: Some(tool_calls),
        tool_call_id: None,
        reasoning_summary: None,
        reasoning_raw: None,
    };

    let provider = Box::new(MockProvider::new(vec![initial_response]));
    let bus = Arc::new(EventBus::new(100));

    let mut config = AgentConfig::default();
    config
        .permissions
        .overrides
        .insert("ask_tool".to_string(), PermissionLevel::Ask);
    config
        .permissions
        .overrides
        .insert("allow_tool".to_string(), PermissionLevel::Allow);
    config
        .permissions
        .overrides
        .insert("allow_tool_2".to_string(), PermissionLevel::Allow);

    let mut agent = Agent::new(provider, bus, config, PathBuf::from("."));
    agent.register_tool(Box::new(MockTool {
        name: "allow_tool".to_string(),
    }));
    agent.register_tool(Box::new(MockTool {
        name: "ask_tool".to_string(),
    }));
    agent.register_tool(Box::new(MockTool {
        name: "allow_tool_2".to_string(),
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

    // Check history:
    // User: Do it
    // Assistant: Thinking...
    // Tool: Executed allow_tool (call_1)
    // (call_2 is pending, call_3 is held)

    let history = session.history();
    // 0: User, 1: Assistant, 2: Tool(allow_tool)
    assert_eq!(history.len(), 3);
    assert_eq!(history[2].role, Role::Tool);
    assert_eq!(history[2].content.as_deref(), Some("Executed allow_tool"));

    // Check pending
    assert_eq!(session.pending_approvals.len(), 1);
    assert!(session.pending_approvals.contains_key("call_2"));

    assert_eq!(session.pending_batch.len(), 1);
    assert_eq!(session.pending_batch[0].id, "call_3");

    // 2. Approve call_2
    let res = agent
        .resolve_approval(&mut session, "call_2", true)
        .await
        .unwrap();

    // After approval:
    // Tool: Executed ask_tool (call_2)
    // Then resume batch -> Tool: Executed allow_tool_2 (call_3)
    // Then run_turn_loop -> "Done"

    assert_eq!(res, "Done");

    let history = session.history();
    // 3: Tool(ask_tool), 4: Tool(allow_tool_2), 5: Assistant(Done)
    assert_eq!(history.len(), 6);
    assert_eq!(history[3].content.as_deref(), Some("Executed ask_tool"));
    assert_eq!(history[4].content.as_deref(), Some("Executed allow_tool_2"));
    assert_eq!(history[5].content.as_deref(), Some("Done"));

    assert!(session.pending_approvals.is_empty());
    assert!(session.pending_batch.is_empty());
}
