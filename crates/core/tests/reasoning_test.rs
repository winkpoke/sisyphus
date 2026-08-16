use anyhow::Result;
use async_trait::async_trait;
use common::bus::{EventBus, SystemEvent};
use common::llm::{
    CompletionRequest, FunctionCall, LLMProvider, Message, ReasoningConfig, ReasoningExposure,
    ReasoningMode, Role, ToolCall,
};
use futures::Stream;
use sisyphus_core::agent::config::AgentConfig;
use sisyphus_core::agent::Agent;
use sisyphus_core::session::{PendingApproval, Session};
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

/// Mock provider that returns canned responses and records every
/// `CompletionRequest` it receives (so tests can inspect the reasoning
/// configuration the agent chose).
struct MockProvider {
    responses: Arc<Mutex<Vec<Message>>>,
    requests: Arc<Mutex<Vec<CompletionRequest>>>,
}

impl MockProvider {
    /// Returns the provider plus a shared handle to the captured requests.
    fn new(responses: Vec<Message>) -> (Self, Arc<Mutex<Vec<CompletionRequest>>>) {
        let requests: Arc<Mutex<Vec<CompletionRequest>>> = Arc::new(Mutex::new(Vec::new()));
        (
            Self {
                responses: Arc::new(Mutex::new(responses)),
                requests: requests.clone(),
            },
            requests,
        )
    }
}

fn assistant_message(content: &str) -> Message {
    Message {
        role: Role::Assistant,
        content: Some(content.to_string()),
        tool_calls: None,
        tool_call_id: None,
        reasoning_summary: None,
        reasoning_raw: None,
    }
}

fn reasoning_response(summary: Option<&str>, raw: Option<&str>) -> Message {
    Message {
        role: Role::Assistant,
        content: Some("Final answer".to_string()),
        tool_calls: None,
        tool_call_id: None,
        reasoning_summary: summary.map(|s| s.to_string()),
        reasoning_raw: raw.map(|s| s.to_string()),
    }
}

/// Seed the session context with a completed tool exchange so it contains a
/// Tool-role message (mirrors a turn where tool usage has begun).
fn seed_tool_exchange(session: &mut Session) {
    session
        .add_message(Message {
            role: Role::User,
            content: Some("read the file".to_string()),
            tool_calls: None,
            tool_call_id: None,
            reasoning_summary: None,
            reasoning_raw: None,
        })
        .unwrap();
    session
        .add_message(Message {
            role: Role::Assistant,
            content: None,
            tool_calls: Some(vec![ToolCall {
                id: "call-1".to_string(),
                function: FunctionCall {
                    name: "read_file".to_string(),
                    arguments: "{}".to_string(),
                },
                kind: "function".to_string(),
            }]),
            tool_call_id: None,
            reasoning_summary: None,
            reasoning_raw: None,
        })
        .unwrap();
    session
        .add_message(Message {
            role: Role::Tool,
            content: Some("file contents".to_string()),
            tool_calls: None,
            tool_call_id: Some("call-1".to_string()),
            reasoning_summary: None,
            reasoning_raw: None,
        })
        .unwrap();
}

#[async_trait]
impl LLMProvider for MockProvider {
    fn model(&self) -> String {
        "mock-model".to_string()
    }

    async fn complete(&self, request: CompletionRequest) -> Result<Message> {
        self.requests.lock().unwrap().push(request);
        let mut responses = self.responses.lock().unwrap();
        if !responses.is_empty() {
            Ok(responses.remove(0))
        } else {
            Ok(assistant_message("Done"))
        }
    }

    async fn stream(
        &self,
        _request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        unimplemented!()
    }
}

fn drain_events(
    rx: &mut tokio::sync::broadcast::Receiver<common::bus::EventEnvelope<SystemEvent>>,
) -> Vec<SystemEvent> {
    let mut events = Vec::new();
    while let Ok(envelope) = rx.try_recv() {
        events.push(envelope.event);
    }
    events
}

// ---------------------------------------------------------------------------
// Summary emission (task 7.1)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_reasoning_event_publishing() {
    let reasoning_content = "This is the reasoning content.";
    let response = reasoning_response(Some("Thought for 1s"), Some(reasoning_content));

    let (provider, _requests) = MockProvider::new(vec![response]);
    let bus = Arc::new(EventBus::new(100));
    let mut rx = bus.subscribe_raw();

    let config = AgentConfig::default();
    let agent = Agent::new(Box::new(provider), bus.clone(), config, PathBuf::from("."));
    let mut session = Session::new(None);

    let _ = agent.chat(&mut session, "Hello".to_string()).await.unwrap();

    // Check events
    let mut found_reasoning = false;
    let mut found_answer = false;

    for event in drain_events(&mut rx) {
        if let SystemEvent::MessageReceived { content, kind, .. } = event {
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

    assert!(
        found_reasoning,
        "Should have received reasoning event with raw content"
    );
    assert!(found_answer, "Should have received final answer event");
}

#[tokio::test]
async fn test_summary_emitted_as_system_event_when_provider_returns_summary() {
    let response = reasoning_response(Some("Short summary"), None);

    let (provider, _requests) = MockProvider::new(vec![response]);
    let bus = Arc::new(EventBus::new(100));
    let mut rx = bus.subscribe_raw();

    let agent = Agent::new(
        Box::new(provider),
        bus.clone(),
        AgentConfig::default(),
        PathBuf::from("."),
    );
    let mut session = Session::new(None);

    let _ = agent.chat(&mut session, "Hello".to_string()).await.unwrap();

    let events = drain_events(&mut rx);
    let summary_events: Vec<_> = events
        .iter()
        .filter_map(|e| match e {
            SystemEvent::MessageReceived {
                content,
                role,
                kind: Some(k),
            } if k == "reasoning_summary" => Some((content.clone(), role.clone())),
            _ => None,
        })
        .collect();

    assert_eq!(
        summary_events,
        vec![("Short summary".to_string(), "system".to_string())],
        "summary must be published as a single system event with kind=reasoning_summary"
    );
}

#[tokio::test]
async fn test_summary_not_emitted_when_exposure_none() {
    let response = reasoning_response(Some("Secret summary"), Some("raw chain of thought"));

    let (provider, _requests) = MockProvider::new(vec![response]);
    let bus = Arc::new(EventBus::new(100));
    let mut rx = bus.subscribe_raw();

    let mut agent = Agent::new(
        Box::new(provider),
        bus.clone(),
        AgentConfig::default(),
        PathBuf::from("."),
    );
    let mut reasoning = ReasoningConfig::with_defaults();
    reasoning.expose = ReasoningExposure::None;
    agent.set_reasoning_config(reasoning);

    let mut session = Session::new(None);
    let _ = agent.chat(&mut session, "Hello".to_string()).await.unwrap();

    let events = drain_events(&mut rx);
    let has_reasoning = events.iter().any(|e| {
        matches!(
            e,
            SystemEvent::MessageReceived { kind: Some(k), .. } if k == "reasoning_summary"
        )
    });
    assert!(
        !has_reasoning,
        "no reasoning summary event may be published when expose=none"
    );
}

// ---------------------------------------------------------------------------
// Reasoning is not persisted into session context by default (task 7.1)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_reasoning_not_stored_in_session_context() {
    let response = reasoning_response(Some("Thought for 1s"), Some("raw chain of thought"));

    let (provider, _requests) = MockProvider::new(vec![response]);
    let bus = Arc::new(EventBus::new(100));

    let agent = Agent::new(
        Box::new(provider),
        bus.clone(),
        AgentConfig::default(),
        PathBuf::from("."),
    );
    let mut session = Session::new(None);

    let _ = agent.chat(&mut session, "Hello".to_string()).await.unwrap();

    let history = session.history();
    let stored: Vec<&Message> = history
        .iter()
        .filter(|m| m.role == Role::Assistant)
        .collect();
    assert!(!stored.is_empty(), "assistant response should be stored");
    for msg in stored {
        assert!(
            msg.reasoning_raw.is_none(),
            "raw reasoning must never be persisted into the session context"
        );
        assert!(
            msg.reasoning_summary.is_none(),
            "with store=none (default) the summary must not be persisted either"
        );
    }
}

// ---------------------------------------------------------------------------
// Deterministic auto mode (spec: agent-core)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn test_auto_mode_stays_off_without_tool_activity() {
    let (provider, requests) = MockProvider::new(vec![assistant_message("answer")]);
    let bus = Arc::new(EventBus::new(100));

    let agent = Agent::new(
        Box::new(provider),
        bus.clone(),
        AgentConfig::default(),
        PathBuf::from("."),
    );
    let mut session = Session::new(None);

    let _ = agent.chat(&mut session, "Hello".to_string()).await.unwrap();

    let requests = requests.lock().unwrap();
    assert!(!requests.is_empty(), "at least one request should be sent");
    assert_eq!(
        requests[0].reasoning.mode,
        ReasoningMode::Off,
        "auto mode must not request reasoning when the session has no tool messages and nothing pending"
    );
}

#[tokio::test]
async fn test_auto_mode_enables_after_tool_usage() {
    let (provider, requests) = MockProvider::new(vec![assistant_message("answer")]);
    let bus = Arc::new(EventBus::new(100));

    let agent = Agent::new(
        Box::new(provider),
        bus.clone(),
        AgentConfig::default(),
        PathBuf::from("."),
    );
    let mut session = Session::new(None);
    seed_tool_exchange(&mut session);

    let _ = agent.chat(&mut session, "Continue".to_string()).await.unwrap();

    let requests = requests.lock().unwrap();
    assert!(!requests.is_empty());
    assert_ne!(
        requests[0].reasoning.mode,
        ReasoningMode::Off,
        "auto mode must request reasoning once the context contains tool messages"
    );
}

#[tokio::test]
async fn test_auto_mode_enables_with_pending_tool_batch() {
    let (provider, requests) = MockProvider::new(vec![assistant_message("answer")]);
    let bus = Arc::new(EventBus::new(100));

    let agent = Agent::new(
        Box::new(provider),
        bus.clone(),
        AgentConfig::default(),
        PathBuf::from("."),
    );
    let mut session = Session::new(None);
    session.pending_batch.push(ToolCall {
        id: "call-9".to_string(),
        function: FunctionCall {
            name: "run_tests".to_string(),
            arguments: "{}".to_string(),
        },
        kind: "function".to_string(),
    });

    let _ = agent.chat(&mut session, "Continue".to_string()).await.unwrap();

    let requests = requests.lock().unwrap();
    assert!(!requests.is_empty());
    assert_ne!(
        requests[0].reasoning.mode,
        ReasoningMode::Off,
        "auto mode must request reasoning when a tool batch is pending"
    );
}

#[tokio::test]
async fn test_auto_mode_enables_with_pending_approval() {
    let (provider, requests) = MockProvider::new(vec![assistant_message("answer")]);
    let bus = Arc::new(EventBus::new(100));

    let agent = Agent::new(
        Box::new(provider),
        bus.clone(),
        AgentConfig::default(),
        PathBuf::from("."),
    );
    let mut session = Session::new(None);
    session.pending_approvals.insert(
        "call-7".to_string(),
        PendingApproval {
            call_id: "call-7".to_string(),
            tool_name: "write_file".to_string(),
            args: "{}".to_string(),
        },
    );

    let _ = agent.chat(&mut session, "Continue".to_string()).await.unwrap();

    let requests = requests.lock().unwrap();
    assert!(!requests.is_empty());
    assert_ne!(
        requests[0].reasoning.mode,
        ReasoningMode::Off,
        "auto mode must request reasoning when an approval is pending"
    );
}

#[tokio::test]
async fn test_explicit_off_mode_never_requests_reasoning() {
    let (provider, requests) = MockProvider::new(vec![assistant_message("answer")]);
    let bus = Arc::new(EventBus::new(100));

    let mut agent = Agent::new(
        Box::new(provider),
        bus.clone(),
        AgentConfig::default(),
        PathBuf::from("."),
    );
    let mut reasoning = ReasoningConfig::with_defaults();
    reasoning.mode = ReasoningMode::Off;
    agent.set_reasoning_config(reasoning);

    let mut session = Session::new(None);
    seed_tool_exchange(&mut session);

    let _ = agent.chat(&mut session, "Continue".to_string()).await.unwrap();

    let requests = requests.lock().unwrap();
    assert!(!requests.is_empty());
    assert_eq!(
        requests[0].reasoning.mode,
        ReasoningMode::Off,
        "mode=off must never request reasoning, even with tool messages present"
    );
}
