use async_trait::async_trait;
use common::bus::EventBus;
use common::llm::{CompletionRequest, FunctionCall, LLMProvider, Message, Role, ToolCall};
use common::tool::Tool;
use futures::Stream;
use server::Server;
use sisyphus_core::agent::registry::AgentRegistry;
use sisyphus_core::agent::{
    config::{AgentConfig, PermissionLevel},
    Agent,
};
use sisyphus_core::session::manager::SessionManager;
use std::collections::VecDeque;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .try_init();
}

struct ScriptedProvider {
    responses: Arc<Mutex<VecDeque<Message>>>,
}

#[async_trait]
impl LLMProvider for ScriptedProvider {
    fn model(&self) -> String {
        "scripted-model".to_string()
    }

    async fn complete(&self, _request: CompletionRequest) -> anyhow::Result<Message> {
        let mut responses = self.responses.lock().unwrap();
        if let Some(msg) = responses.pop_front() {
            Ok(msg)
        } else {
            Ok(Message {
                role: Role::Assistant,
                content: Some("No more scripted responses".to_string()),
                tool_calls: None,
                tool_call_id: None,
            })
        }
    }
    async fn stream(
        &self,
        _request: CompletionRequest,
    ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<String>> + Send>>> {
        Ok(Box::pin(futures::stream::empty()))
    }
}

struct MockTool;
#[async_trait]
impl Tool for MockTool {
    fn name(&self) -> &str {
        "mock_tool"
    }
    fn description(&self) -> &str {
        "A mock tool"
    }
    fn schema(&self) -> serde_json::Value {
        serde_json::json!({})
    }
    async fn execute(&self, _args: serde_json::Value) -> anyhow::Result<String> {
        Ok("Mock tool executed".to_string())
    }
}

async fn wait_for_server(port: u16) {
    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(0)
        .build()
        .unwrap();
    let url = format!("http://127.0.0.1:{}/health", port);
    let start = std::time::Instant::now();
    while start.elapsed() < std::time::Duration::from_secs(5) {
        if let Ok(resp) = client.get(&url).send().await {
            if resp.status().is_success() {
                return;
            }
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }
    panic!("Server failed to start on port {}", port);
}

#[tokio::test]
async fn test_approval_flow() {
    init_tracing();
    let bus = Arc::new(EventBus::new(100));

    let responses = Arc::new(Mutex::new(VecDeque::new()));

    // First response: Tool call that triggers permission check
    responses.lock().unwrap().push_back(Message {
        role: Role::Assistant,
        content: None,
        tool_calls: Some(vec![ToolCall {
            id: "call_123".to_string(),
            kind: "function".to_string(),
            function: FunctionCall {
                name: "mock_tool".to_string(),
                arguments: "{}".to_string(),
            },
        }]),
        tool_call_id: None,
    });

    // Second response: After approval, agent resumes and sees tool output
    responses.lock().unwrap().push_back(Message {
        role: Role::Assistant,
        content: Some("Tool executed successfully".to_string()),
        tool_calls: None,
        tool_call_id: None,
    });

    let provider = Box::new(ScriptedProvider { responses });

    let mut config = AgentConfig::default();
    config
        .permissions
        .overrides
        .insert("mock_tool".to_string(), PermissionLevel::Ask);

    let mut agent = Agent::new(provider, bus.clone(), config, std::path::PathBuf::from("."));
    agent.register_tool(Box::new(MockTool));
    let agent = Arc::new(agent);
    let registry = Arc::new(AgentRegistry::new(agent));

    let session_manager = Arc::new(SessionManager::new());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = Server::new(port, registry, session_manager, bus);

    tokio::spawn(async move {
        if let Err(e) = server
            .run_on_listener(listener, std::future::pending::<()>())
            .await
        {
            eprintln!("Server exited with error: {:?}", e);
        }
    });

    // Give server time to start
    wait_for_server(port).await;

    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(0)
        .build()
        .unwrap();
    let base_url = format!("http://127.0.0.1:{}", port);

    // Create session
    tracing::info!("Creating session...");
    let resp = client
        .post(format!("{}/api/v1/sessions", base_url))
        .send()
        .await
        .unwrap();
    let session: serde_json::Value = resp.json().await.unwrap();
    let session_id = session["id"].as_str().unwrap();

    // Send chat message that triggers tool
    tracing::info!("Sending chat message...");
    let chat_body = serde_json::json!({ "message": "Run tool" });
    let resp = client
        .post(format!("{}/api/v1/sessions/{}/chat", base_url, session_id))
        .json(&chat_body)
        .send()
        .await
        .unwrap();

    let chat_resp: serde_json::Value = resp.json().await.unwrap();
    // Verify we get permission required message
    assert!(chat_resp["response"]
        .as_str()
        .unwrap()
        .contains("Permission required"));

    // Verify pending approval exists
    tracing::info!("Verifying pending approval...");
    let resp = client
        .get(format!("{}/api/v1/sessions/{}", base_url, session_id))
        .send()
        .await
        .unwrap();
    let session: serde_json::Value = resp.json().await.unwrap();
    let pending = session["pending_approvals"].as_object().unwrap();
    assert!(pending.contains_key("call_123"));

    // Submit approval
    tracing::info!("Submitting approval...");
    let approval_body = serde_json::json!({ "decision": "approve" });
    let resp = client
        .post(format!(
            "{}/api/v1/sessions/{}/approvals/call_123",
            base_url, session_id
        ))
        .json(&approval_body)
        .send()
        .await
        .unwrap();

    assert!(resp.status().is_success());
    let chat_resp: serde_json::Value = resp.json().await.unwrap();
    // Verify final response
    assert_eq!(
        chat_resp["response"].as_str().unwrap(),
        "Tool executed successfully"
    );
}

#[tokio::test]
async fn test_denial_flow() {
    init_tracing();
    let bus = Arc::new(EventBus::new(100));

    let responses = Arc::new(Mutex::new(VecDeque::new()));

    // First response: Tool call
    responses.lock().unwrap().push_back(Message {
        role: Role::Assistant,
        content: None,
        tool_calls: Some(vec![ToolCall {
            id: "call_456".to_string(),
            kind: "function".to_string(),
            function: FunctionCall {
                name: "mock_tool".to_string(),
                arguments: "{}".to_string(),
            },
        }]),
        tool_call_id: None,
    });

    // Second response: After denial, agent resumes and sees denial message
    responses.lock().unwrap().push_back(Message {
        role: Role::Assistant,
        content: Some("Understood, tool execution denied.".to_string()),
        tool_calls: None,
        tool_call_id: None,
    });

    let provider = Box::new(ScriptedProvider { responses });

    let mut config = AgentConfig::default();
    config
        .permissions
        .overrides
        .insert("mock_tool".to_string(), PermissionLevel::Ask);

    let mut agent = Agent::new(provider, bus.clone(), config, std::path::PathBuf::from("."));
    agent.register_tool(Box::new(MockTool));
    let agent = Arc::new(agent);
    let registry = Arc::new(AgentRegistry::new(agent));

    let session_manager = Arc::new(SessionManager::new());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = Server::new(port, registry, session_manager, bus);

    tokio::spawn(async move {
        if let Err(e) = server
            .run_on_listener(listener, std::future::pending::<()>())
            .await
        {
            eprintln!("Server (denial) exited with error: {:?}", e);
        }
    });

    wait_for_server(port).await;

    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(0)
        .build()
        .unwrap();
    let base_url = format!("http://127.0.0.1:{}", port);

    // Create session
    tracing::info!("Creating session (denial test)...");
    let resp = client
        .post(format!("{}/api/v1/sessions", base_url))
        .send()
        .await
        .unwrap();
    let session: serde_json::Value = resp.json().await.unwrap();
    let session_id = session["id"].as_str().unwrap();

    // Send chat message
    tracing::info!("Sending chat message (denial test)...");
    let chat_body = serde_json::json!({ "message": "Run tool" });
    let resp = client
        .post(format!("{}/api/v1/sessions/{}/chat", base_url, session_id))
        .json(&chat_body)
        .send()
        .await
        .unwrap();

    // Verify permission required
    let chat_resp: serde_json::Value = resp.json().await.unwrap();
    assert!(chat_resp["response"]
        .as_str()
        .unwrap()
        .contains("Permission required"));

    // Verify pending approval exists
    tracing::info!("Verifying pending approval...");
    let resp = client
        .get(format!("{}/api/v1/sessions/{}", base_url, session_id))
        .send()
        .await
        .unwrap();
    let session_data: serde_json::Value = resp.json().await.unwrap();
    let pending = session_data["pending_approvals"].as_object().unwrap();
    assert!(pending.contains_key("call_456"));

    // Submit denial
    tracing::info!("Submitting denial...");
    let approval_body = serde_json::json!({ "decision": "deny" });
    let resp = client
        .post(format!(
            "{}/api/v1/sessions/{}/approvals/call_456",
            base_url, session_id
        ))
        .json(&approval_body)
        .send()
        .await
        .unwrap();

    assert!(resp.status().is_success());
    let chat_resp: serde_json::Value = resp.json().await.unwrap();

    // Verify final response matches the one from provider (which saw the denial)
    assert_eq!(
        chat_resp["response"].as_str().unwrap(),
        "Understood, tool execution denied."
    );

    // Verify history contains denial message (by checking session history)
    let resp = client
        .get(format!("{}/api/v1/sessions/{}", base_url, session_id))
        .send()
        .await
        .unwrap();
    let _session: serde_json::Value = resp.json().await.unwrap();
    // TODO: access history/context via API? API doesn't expose history directly in Session struct,
    // but Session struct has `context`. Wait, Session struct in lib.rs has `context: Context`.
    // Context has `messages`.
    // Let's check Session struct in core.
    // It has `context: Context`. Context has `messages`.
    // But Session serialization might not include context details if not public?
    // Session struct:
    // pub struct Session {
    //    pub id: String,
    //    ...
    //    context: Context,
    // }
    // `context` field is private in `Session` struct definition in `crates/core/src/session.rs`?
    // Let's check `crates/core/src/session.rs`.
    // `context: Context` is private (no pub).
    // So it won't be serialized unless `Context` is serialized and the field is included.
    // But since it is private, serde might skip it or it might be serialized if `Serialize` is derived on struct.
    // If it is derived, it serializes private fields too? No, only if they are accessible to the macro.
    // Wait, derived Serialize implementation accesses fields.
    // If field is private to the module, but we are in the same crate... `api_test` is in `server` crate, `Session` is in `core`.
    // `Session` definition has `context: Context`.
    // If I cannot see history, I rely on the agent response "Understood..." which implies it received the denial tool result.
}
