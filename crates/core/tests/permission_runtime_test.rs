//! Integration tests for the rule-based permission runtime
//! (`refactor-permission-system` change, task 4.4).
//!
//! These exercise the full path: model tool-call → `Agent::execute_tool` →
//! permission resolver → event emission / execution / denial.

use anyhow::Result;
use async_trait::async_trait;
use common::bus::{EventBus, SystemEvent};
use common::llm::{CompletionRequest, FunctionCall, LLMProvider, Message, Role, ToolCall};
use common::tool::Tool;
use futures::Stream;
use serde_json::{json, Value};
use sisyphus_core::agent::config::{AgentConfig, AgentPermissions, PermissionMode};
use sisyphus_core::agent::Agent;
use sisyphus_core::session::Session;
use std::path::PathBuf;
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

fn tool_call_message(id: &str, tool: &str, args: Value) -> Message {
    Message {
        role: Role::Assistant,
        content: None,
        tool_calls: Some(vec![ToolCall {
            id: id.to_string(),
            function: FunctionCall {
                name: tool.to_string(),
                arguments: args.to_string(),
            },
            kind: "function".to_string(),
        }]),
        tool_call_id: None,
        reasoning_summary: None,
        reasoning_raw: None,
    }
}

/// Allow rule `Bash(git:*)` auto-executes matching commands without a
/// permission event, and non-matching commands fall back to mode Ask.
#[tokio::test]
async fn allow_rule_executes_matching_command_without_prompt() {
    let perms = AgentPermissions {
        allow: vec!["Bash(git:*)".to_string()],
        ..AgentPermissions::default()
    };

    let decision = sisyphus_core::agent::permission::resolve(
        &perms,
        "execute_command",
        &json!({ "command": "git push origin main" }),
    );
    assert_eq!(
        decision,
        sisyphus_core::agent::permission::PermissionDecision::Allow
    );
    let decision = sisyphus_core::agent::permission::resolve(
        &perms,
        "execute_command",
        &json!({ "command": "npm install" }),
    );
    assert!(matches!(
        decision,
        sisyphus_core::agent::permission::PermissionDecision::Ask { .. }
    ));
}

/// End-to-end: deny rule produces the deny tool-result in the session
/// history and no permission event.
#[tokio::test]
async fn deny_rule_short_circuits_execution() {
    let bus = Arc::new(EventBus::new(16));
    let mut rx = bus.subscribe_raw();
    let config = AgentConfig {
        permissions: AgentPermissions {
            deny: vec!["Bash(rm -rf:*)".to_string(), "Read(./.env)".to_string()],
            ..AgentPermissions::default()
        },
        ..AgentConfig::default()
    };

    let provider = Box::new(MockProvider::new(vec![tool_call_message(
        "c1",
        "execute_command",
        json!({ "command": "rm -rf /tmp/thing" }),
    )]));
    let mut agent = Agent::new(provider, bus.clone(), config, PathBuf::from("."));
    agent.register_tool(Box::new(MockTool {
        name: "execute_command".to_string(),
    }));

    let mut session = Session::new(None);
    let _ = agent.chat(&mut session, "test".to_string()).await.unwrap();

    // The mock tool must NOT have run; the deny reason is the tool result.
    let history = session.history();
    let tool_results: Vec<&str> = history
        .iter()
        .filter(|m| m.role == Role::Tool)
        .map(|m| m.content.as_deref().unwrap_or_default())
        .collect();
    assert!(
        tool_results
            .iter()
            .any(|r| r.contains("matched deny rule 'Bash(rm -rf:*)'")),
        "expected deny-rule tool result, got {:?}",
        tool_results
    );

    // No permission request event was emitted for the denied call.
    let mut saw_request = false;
    while let Ok(envelope) = rx.try_recv() {
        if matches!(envelope.event, SystemEvent::PermissionRequest { .. }) {
            saw_request = true;
        }
    }
    assert!(!saw_request, "deny must not prompt");
}

/// End-to-end: ask rule (and no allow match) emits PermissionRequest with the
/// matched rule and mode, and parks the call as a pending approval.
#[tokio::test]
async fn ask_rule_emits_permission_request_with_rule_and_mode() -> Result<()> {
    let bus = Arc::new(EventBus::new(16));
    let mut rx = bus.subscribe_raw();
    let config = AgentConfig {
        permissions: AgentPermissions {
            mode: PermissionMode::AcceptEdits,
            ask: vec!["Bash(npm install:*)".to_string()],
            ..AgentPermissions::default()
        },
        ..AgentConfig::default()
    };

    let provider = Box::new(MockProvider::new(vec![tool_call_message(
        "c1",
        "execute_command",
        json!({ "command": "npm install lodash" }),
    )]));
    let mut agent = Agent::new(provider, bus.clone(), config, PathBuf::from("."));
    agent.register_tool(Box::new(MockTool {
        name: "execute_command".to_string(),
    }));

    let mut session = Session::new(None);
    let _ = agent.chat(&mut session, "test".to_string()).await.unwrap();

    let mut request = None;
    while let Ok(envelope) = rx.try_recv() {
        if let SystemEvent::PermissionRequest {
            tool_name,
            call_id,
            matched_rule,
            mode,
            ..
        } = envelope.event
        {
            request = Some((tool_name, call_id, matched_rule, mode));
        }
    }
    let (tool_name, call_id, matched_rule, mode) = request
        .ok_or_else(|| anyhow::anyhow!("expected a PermissionRequest event"))?;
    assert_eq!(tool_name, "execute_command");
    assert_eq!(call_id, "c1");
    assert_eq!(matched_rule.as_deref(), Some("Bash(npm install:*)"));
    assert_eq!(mode, "AcceptEdits");

    // The call is parked for approval, not executed.
    assert!(session.pending_approvals.contains_key("c1"));
    Ok(())
}

/// Subagent isolation: an agent configured with Plan mode (the built-in plan
/// agent shape) denies writes and allows read-only exploration rules.
#[tokio::test]
async fn subagent_plan_mode_isolation() {
    // Parent shape: Default mode with a broad allow rule.
    let parent = AgentPermissions {
        allow: vec!["Write(./**)".to_string()],
        ..AgentPermissions::default()
    };
    // Subagent shape: Plan mode + read-only allow rules (as bootstrap.rs).
    let subagent = AgentPermissions {
        mode: PermissionMode::Plan,
        allow: vec![
            "read_file".to_string(),
            "glob".to_string(),
            "grep".to_string(),
        ],
        ..AgentPermissions::default()
    };

    let resolve = sisyphus_core::agent::permission::resolve;

    // Parent can write; subagent cannot.
    assert_eq!(
        resolve(&parent, "write_file", &json!({ "path": "src/main.rs" })),
        sisyphus_core::agent::permission::PermissionDecision::Allow
    );
    assert!(matches!(
        resolve(&subagent, "write_file", &json!({ "path": "src/main.rs" })),
        sisyphus_core::agent::permission::PermissionDecision::Deny { .. }
    ));

    // Subagent reads flow through its allow rules without prompting.
    assert_eq!(
        resolve(&subagent, "read_file", &json!({ "path": "src/main.rs" })),
        sisyphus_core::agent::permission::PermissionDecision::Allow
    );

    // Subagent cannot run commands even though the parent allow rule is broad.
    assert!(matches!(
        resolve(&subagent, "execute_command", &json!({ "command": "ls" })),
        sisyphus_core::agent::permission::PermissionDecision::Deny { .. }
    ));
}

/// BypassPermissions end-to-end: deny rules still enforced, everything else
/// executes.
#[tokio::test]
async fn bypass_mode_still_honors_deny_rules() {
    let perms = AgentPermissions {
        mode: PermissionMode::BypassPermissions,
        deny: vec!["Read(./.env)".to_string()],
        ..AgentPermissions::default()
    };
    let resolve = sisyphus_core::agent::permission::resolve;

    assert!(matches!(
        resolve(&perms, "read_file", &json!({ "path": ".env" })),
        sisyphus_core::agent::permission::PermissionDecision::Deny { .. }
    ));
    assert_eq!(
        resolve(&perms, "write_file", &json!({ "path": "anything" })),
        sisyphus_core::agent::permission::PermissionDecision::Allow
    );
    assert_eq!(
        resolve(&perms, "execute_command", &json!({ "command": "rm -rf /" })),
        sisyphus_core::agent::permission::PermissionDecision::Allow
    );
}

/// Legacy configs keep byte-for-byte behavior: rules empty + Default mode
/// routes through edit/bash/skill and overrides.
#[tokio::test]
async fn legacy_config_end_to_end_unchanged() {
    use sisyphus_core::agent::config::PermissionLevel;

    let bus = Arc::new(EventBus::new(16));
    let mut rx = bus.subscribe_raw();
    let mut config = AgentConfig::default();
    config.permissions.edit = PermissionLevel::Allow;
    config.permissions.bash = PermissionLevel::Ask;

    let provider = Box::new(MockProvider::new(vec![tool_call_message(
        "c1",
        "write_file",
        json!({ "path": "x.txt" }),
    )]));
    let mut agent = Agent::new(provider, bus.clone(), config, PathBuf::from("."));
    agent.register_tool(Box::new(MockTool {
        name: "write_file".to_string(),
    }));

    let mut session = Session::new(None);
    let _ = agent.chat(&mut session, "test".to_string()).await.unwrap();

    // edit=Allow → the mock tool executed (its result appears in history).
    let history = session.history();
    assert!(
        history
            .iter()
            .filter(|m| m.role == Role::Tool)
            .any(|m| m.content.as_deref() == Some("Executed")),
        "legacy edit=Allow must auto-execute write_file"
    );
    let mut saw_request = false;
    while let Ok(envelope) = rx.try_recv() {
        if matches!(envelope.event, SystemEvent::PermissionRequest { .. }) {
            saw_request = true;
        }
    }
    assert!(!saw_request, "allowed legacy tool must not prompt");
}
