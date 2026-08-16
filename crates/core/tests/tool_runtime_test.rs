//! Integration tests for selective parallel tool execution
//! (`add-parallel-tool-runtime` change).
//!
//! These exercise the full path: model tool-calls → `process_tool_batch`
//! permission phasing → `ToolCallRuntime` concurrency gate → deterministic
//! transcript ordering.

use anyhow::Result;
use async_trait::async_trait;
use common::bus::{EventBus, SystemEvent};
use common::llm::{CompletionRequest, FunctionCall, LLMProvider, Message, Role, ToolCall};
use common::tool::{ExecutionMode, Tool};
use common::bus::SystemEventEnvelope;
use futures::Stream;
use serde_json::{json, Value};
use sisyphus_core::agent::config::{AgentConfig, AgentPermissions, PermissionMode};
use sisyphus_core::agent::Agent;
use sisyphus_core::session::Session;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Mock tool with configurable execution mode, artificial latency, and a
/// shared concurrency tracker (records peak simultaneous executions).
struct TimedTool {
    name: String,
    mode: ExecutionMode,
    delay: Duration,
    active: Arc<AtomicUsize>,
    peak: Arc<AtomicUsize>,
    calls: Arc<Mutex<Vec<String>>>,
}

impl TimedTool {
    fn new(
        name: &str,
        mode: ExecutionMode,
        delay_ms: u64,
        tracker: &Tracker,
    ) -> Self {
        Self {
            name: name.to_string(),
            mode,
            delay: Duration::from_millis(delay_ms),
            active: tracker.active.clone(),
            peak: tracker.peak.clone(),
            calls: tracker.calls.clone(),
        }
    }
}

#[derive(Default)]
struct Tracker {
    active: Arc<AtomicUsize>,
    peak: Arc<AtomicUsize>,
    calls: Arc<Mutex<Vec<String>>>,
}

impl Tracker {
    fn peak(&self) -> usize {
        self.peak.load(Ordering::SeqCst)
    }
    fn calls(&self) -> Vec<String> {
        self.calls.lock().unwrap().clone()
    }
}

#[async_trait]
impl Tool for TimedTool {
    fn name(&self) -> &str {
        &self.name
    }
    fn description(&self) -> &str {
        "Timed mock tool"
    }
    fn schema(&self) -> Value {
        json!({})
    }
    fn execution_mode(&self) -> ExecutionMode {
        self.mode
    }
    async fn execute(&self, _args: Value) -> Result<String> {
        let now = self.active.fetch_add(1, Ordering::SeqCst) + 1;
        self.peak.fetch_max(now, Ordering::SeqCst);
        self.calls.lock().unwrap().push(self.name.clone());
        tokio::time::sleep(self.delay).await;
        self.active.fetch_sub(1, Ordering::SeqCst);
        Ok(format!("{} done", self.name))
    }
}

struct MockProvider {
    responses: Mutex<Vec<Message>>,
}

impl MockProvider {
    fn new(responses: Vec<Message>) -> Self {
        Self {
            responses: Mutex::new(responses),
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

fn tool_call(id: &str, tool: &str) -> ToolCall {
    ToolCall {
        id: id.to_string(),
        function: FunctionCall {
            name: tool.to_string(),
            arguments: "{}".to_string(),
        },
        kind: "function".to_string(),
    }
}

fn response_with_calls(ids_and_tools: &[(&str, &str)]) -> Message {
    Message {
        role: Role::Assistant,
        content: None,
        tool_calls: Some(
            ids_and_tools
                .iter()
                .map(|(id, tool)| tool_call(id, tool))
                .collect(),
        ),
        tool_call_id: None,
        reasoning_summary: None,
        reasoning_raw: None,
    }
}

/// Bypass mode with no deny rules: every tool runs without prompting, so
/// tests observe pure execution behavior.
fn bypass_config() -> AgentConfig {
    AgentConfig {
        permissions: AgentPermissions {
            mode: PermissionMode::BypassPermissions,
            ..AgentPermissions::default()
        },
        ..AgentConfig::default()
    }
}

async fn chat_and_get_history(agent: &Agent, session: &mut Session) -> Vec<Message> {
    let _ = agent.chat(session, "test".to_string()).await.unwrap();
    session.history()
}

/// Three Parallel tools in one batch overlap in time, and their results are
/// appended to the transcript in the original tool_calls order even though
/// they complete in a different order.
#[tokio::test]
async fn parallel_tools_overlap_and_transcript_is_ordered() {
    let tracker = Tracker::default();
    // First call sleeps longest → completes last; transcript must not care.
    let config = bypass_config();
    let provider = Box::new(MockProvider::new(vec![response_with_calls(&[
        ("c1", "p_slow"),
        ("c2", "p_mid"),
        ("c3", "p_fast"),
    ])]));
    let mut agent = Agent::new(provider, Arc::new(EventBus::new(32)), config, PathBuf::from("."));
    for tool in [
        Box::new(TimedTool::new("p_slow", ExecutionMode::Parallel, 120, &tracker)) as Box<dyn Tool>,
        Box::new(TimedTool::new("p_mid", ExecutionMode::Parallel, 60, &tracker)),
        Box::new(TimedTool::new("p_fast", ExecutionMode::Parallel, 10, &tracker)),
    ] {
        agent.register_tool(tool);
    }

    let mut session = Session::new(None);
    let history = chat_and_get_history(&agent, &mut session).await;

    assert!(
        tracker.peak() >= 2,
        "parallel tools must overlap (peak={})",
        tracker.peak()
    );

    let tool_ids: Vec<&str> = history
        .iter()
        .filter(|m| m.role == Role::Tool)
        .map(|m| m.tool_call_id.as_deref().unwrap_or_default())
        .collect();
    assert_eq!(
        tool_ids,
        vec!["c1", "c2", "c3"],
        "tool results must appear in original tool_calls order"
    );
}

/// Sequential tools never overlap — even without any parallel tools in the
/// batch, exclusivity is enforced (peak concurrency == 1).
#[tokio::test]
async fn sequential_tools_execute_exclusively() {
    let tracker = Tracker::default();
    let config = AgentConfig {
        permissions: AgentPermissions {
            mode: PermissionMode::BypassPermissions,
            ..AgentPermissions::default()
        },
        ..AgentConfig::default()
    };
    let provider = Box::new(MockProvider::new(vec![response_with_calls(&[
        ("c1", "s1"),
        ("c2", "s2"),
        ("c3", "s3"),
    ])]));
    let mut agent = Agent::new(provider, Arc::new(EventBus::new(32)), config, PathBuf::from("."));
    for tool in [
        Box::new(TimedTool::new("s1", ExecutionMode::Sequential, 40, &tracker)) as Box<dyn Tool>,
        Box::new(TimedTool::new("s2", ExecutionMode::Sequential, 40, &tracker)),
        Box::new(TimedTool::new("s3", ExecutionMode::Sequential, 40, &tracker)),
    ] {
        agent.register_tool(tool);
    }

    let mut session = Session::new(None);
    let history = chat_and_get_history(&agent, &mut session).await;

    assert_eq!(tracker.peak(), 1, "sequential tools must not overlap");
    let tool_ids: Vec<&str> = history
        .iter()
        .filter(|m| m.role == Role::Tool)
        .map(|m| m.tool_call_id.as_deref().unwrap_or_default())
        .collect();
    assert_eq!(tool_ids, vec!["c1", "c2", "c3"]);
}

/// A Sequential tool in a mixed batch blocks all other tools while it runs:
/// the parallel pair may overlap each other, but nothing overlaps the writer.
#[tokio::test]
async fn sequential_tool_blocks_parallel_tools_in_mixed_batch() {
    let tracker = Tracker::default();
    let config = AgentConfig {
        permissions: AgentPermissions {
            mode: PermissionMode::BypassPermissions,
            ..AgentPermissions::default()
        },
        ..AgentConfig::default()
    };
    let provider = Box::new(MockProvider::new(vec![response_with_calls(&[
        ("c1", "p1"),
        ("c2", "p2"),
        ("c3", "writer"),
        ("c4", "p3"),
    ])]));
    let mut agent = Agent::new(provider, Arc::new(EventBus::new(32)), config, PathBuf::from("."));
    for tool in [
        Box::new(TimedTool::new("p1", ExecutionMode::Parallel, 60, &tracker)) as Box<dyn Tool>,
        Box::new(TimedTool::new("p2", ExecutionMode::Parallel, 60, &tracker)),
        Box::new(TimedTool::new("writer", ExecutionMode::Sequential, 60, &tracker)),
        Box::new(TimedTool::new("p3", ExecutionMode::Parallel, 10, &tracker)),
    ] {
        agent.register_tool(tool);
    }

    let mut session = Session::new(None);
    let history = chat_and_get_history(&agent, &mut session).await;

    // Peak may be 2 (the parallel pair) but never 3+: the writer runs alone.
    assert!(
        tracker.peak() <= 2,
        "sequential tool must block all others (peak={})",
        tracker.peak()
    );
    let executed = tracker.calls();
    assert!(executed.contains(&"writer".to_string()));
    assert_eq!(executed.len(), 4, "all four calls must execute");

    let tool_ids: Vec<&str> = history
        .iter()
        .filter(|m| m.role == Role::Tool)
        .map(|m| m.tool_call_id.as_deref().unwrap_or_default())
        .collect();
    assert_eq!(tool_ids, vec!["c1", "c2", "c3", "c4"]);
}

/// An Ask-gated call parks the batch: earlier allowed calls still execute,
/// later allowed calls must NOT execute until the approval resolves.
#[tokio::test]
async fn ask_parks_batch_and_skips_later_calls() {
    let tracker = Tracker::default();
    let bus = Arc::new(EventBus::new(32));
    let mut rx = bus.subscribe_raw();

    // allow p1/p3 explicitly; p2 falls through to Default-mode Ask.
    let config = AgentConfig {
        permissions: AgentPermissions {
            mode: PermissionMode::Default,
            allow: vec!["p1".to_string(), "p3".to_string()],
            ..AgentPermissions::default()
        },
        ..AgentConfig::default()
    };
    let provider = Box::new(MockProvider::new(vec![response_with_calls(&[
        ("c1", "p1"),
        ("c2", "p2"),
        ("c3", "p3"),
    ])]));
    let mut agent = Agent::new(provider, bus.clone(), config, PathBuf::from("."));
    for tool in [
        Box::new(TimedTool::new("p1", ExecutionMode::Parallel, 30, &tracker)) as Box<dyn Tool>,
        Box::new(TimedTool::new("p2", ExecutionMode::Parallel, 30, &tracker)),
        Box::new(TimedTool::new("p3", ExecutionMode::Parallel, 30, &tracker)),
    ] {
        agent.register_tool(tool);
    }

    let mut session = Session::new(None);
    let outcome = agent.chat(&mut session, "test".to_string()).await.unwrap();

    // The turn stops with the permission-required message.
    assert!(format!("{:?}", outcome).contains("Permission required"));

    // c2 is parked for approval; c3 is parked in the pending batch.
    assert!(session.pending_approvals.contains_key("c2"));
    assert_eq!(session.pending_batch.len(), 1);
    assert_eq!(session.pending_batch[0].id, "c3");

    // Only p1 executed; p2/p3 never ran.
    assert_eq!(tracker.calls(), vec!["p1".to_string()]);

    // The PermissionRequest was emitted (after c1's ToolExecuted).
    let mut events = Vec::new();
    while let Ok(SystemEventEnvelope { event, .. }) = rx.try_recv() {
        events.push(event);
    }
    assert!(events
        .iter()
        .any(|e| matches!(e, SystemEvent::PermissionRequest { call_id, .. } if call_id == "c2")));
}
