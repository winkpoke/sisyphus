pub mod config;
pub mod permission;
pub mod prompt;
pub mod registry;
pub mod runtime;

use self::config::AgentConfig;
use self::prompt::SystemPromptBuilder;
use crate::command::builtins::{CdCommand, HelpCommand};
use crate::command::loader::CommandLoader;
use crate::command::parser::parse_command;
use crate::command::{CommandContext, CommandOutcome, CommandRegistry, CommandType};
use crate::session::context::DefaultTokenEstimator;
use crate::session::{PendingApproval, Session, SessionStatus};
use crate::template::{SystemPromptContext, TemplateContext, TemplateEngine};
use anyhow::{anyhow, Result};
use common::bus::{EventBus, SystemEvent};
use common::llm::{
    CompletionRequest, LLMProvider, Message, ReasoningConfig, ReasoningExposure, ReasoningMode,
    ReasoningStorage, Role, ToolCall, ToolDefinition, ToolFunctionDefinition,
};
use common::tool::Tool;
use runtime::{ScheduledCall, ToolCallRuntime};
use rust_i18n::t;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

const MAX_TURNS: u32 = 1000;

pub struct Agent {
    provider: Box<dyn LLMProvider>,
    bus: Arc<EventBus>,
    tools: HashMap<String, Box<dyn Tool>>,
    config: AgentConfig,
    commands: CommandRegistry,
    workspace_root: PathBuf,
    prompt_builder: SystemPromptBuilder,
    reasoning_config: ReasoningConfig,
    tool_runtime: ToolCallRuntime,
}

impl Agent {
    pub fn new(
        provider: Box<dyn LLMProvider>,
        bus: Arc<EventBus>,
        config: AgentConfig,
        workspace_root: PathBuf,
    ) -> Self {
        let mut agent = Self {
            provider,
            bus,
            tools: HashMap::new(),
            config: config.clone(),
            commands: CommandRegistry::new(),
            workspace_root: workspace_root.clone(),
            prompt_builder: SystemPromptBuilder::new(),
            reasoning_config: ReasoningConfig::with_defaults(),
            tool_runtime: ToolCallRuntime::new(),
        };
        agent.register_builtins();
        // Legacy edit/bash/skill fields are ignored when rules are present.
        permission::warn_legacy_ignored(&agent.config.permissions);
        let cmd_path = agent.config.get_command_path();
        if let Ok(commands) = CommandLoader::load_from_dir(cmd_path) {
            for (name, config) in commands {
                agent.commands.register_custom(&name, config);
            }
        }
        match SystemPromptBuilder::from_config(&agent.workspace_root, &agent.config) {
            Ok(builder) => agent.prompt_builder = builder,
            Err(e) => {
                tracing::warn!(
                    "Failed to load system prompt template: {}. Using default.",
                    e
                );
                agent.prompt_builder = SystemPromptBuilder::new();
            }
        }
        agent
    }

    fn register_builtins(&mut self) {
        self.commands.register_builtin(Box::new(HelpCommand));
        self.commands.register_builtin(Box::new(CdCommand));
    }

    pub fn list_commands(&self) -> Vec<crate::command::CommandInfo> {
        self.commands.list()
    }

    pub fn config(&self) -> &AgentConfig {
        &self.config
    }

    pub fn model_name(&self) -> String {
        self.provider.model()
    }

    pub fn register_tool(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    pub fn register_read_only_tools(&mut self, tools: Vec<Box<dyn Tool>>) -> anyhow::Result<()> {
        const ALLOWED_TOOLS: &[&str] = &["read_file", "glob", "grep"];

        for tool in &tools {
            if !ALLOWED_TOOLS.contains(&tool.name()) {
                return Err(anyhow!(
                    "Tool '{}' not allowed in read-only agent",
                    tool.name()
                ));
            }
        }

        for tool in tools {
            self.register_tool(tool);
        }
        Ok(())
    }

    /// Replace the agent's reasoning configuration (mode/effort/exposure/storage).
    pub fn set_reasoning_config(&mut self, config: ReasoningConfig) {
        self.reasoning_config = config;
    }

    fn should_enable_reasoning(&self, session: &Session) -> ReasoningConfig {
        let mode = self.reasoning_config.mode.clone();

        if mode == ReasoningMode::Off {
            return ReasoningConfig {
                mode: ReasoningMode::Off,
                effort: self.reasoning_config.effort.clone(),
                expose: self.reasoning_config.expose.clone(),
                store: self.reasoning_config.store.clone(),
            };
        }

        if mode == ReasoningMode::On {
            return self.reasoning_config.clone();
        }

        if mode == ReasoningMode::Auto {
            let has_tool_messages = session.has_tool_messages();

            let has_pending_approval_or_batch =
                !session.pending_approvals.is_empty() || !session.pending_batch.is_empty();

            if has_tool_messages || has_pending_approval_or_batch {
                return self.reasoning_config.clone();
            }
        }

        ReasoningConfig {
            mode: ReasoningMode::Off,
            effort: self.reasoning_config.effort.clone(),
            expose: self.reasoning_config.expose.clone(),
            store: self.reasoning_config.store.clone(),
        }
    }

    fn get_tool_definitions(&self) -> Option<Vec<ToolDefinition>> {
        if self.tools.is_empty() {
            None
        } else {
            Some(
                self.tools
                    .values()
                    .map(|t| ToolDefinition {
                        kind: "function".to_string(),
                        function: ToolFunctionDefinition {
                            name: t.name().to_string(),
                            description: t.description().to_string(),
                            parameters: t.schema(),
                        },
                    })
                    .collect(),
            )
        }
    }

    /// Publish the PermissionRequest event for an Ask-gated call.
    fn emit_permission_request(
        &self,
        tool_name: &str,
        call_id: &str,
        matched_rule: Option<String>,
    ) {
        let mode = serde_json::to_value(self.config.permissions.mode)
            .ok()
            .and_then(|v| v.as_str().map(str::to_string))
            .unwrap_or_default();
        self.bus.publish(SystemEvent::PermissionRequest {
            operation: "tool_execution".to_string(),
            tool_name: tool_name.to_string(),
            call_id: call_id.to_string(),
            matched_rule,
            mode,
        });
    }

    /// Execute a permission-cleared tool call and map every failure mode to a
    /// transcript-ready string.
    async fn run_tool(&self, tool_name: &str, args_str: &str) -> String {
        if let Some(tool) = self.tools.get(tool_name) {
            match serde_json::from_str::<serde_json::Value>(args_str) {
                Ok(args) => match tool.execute(args).await {
                    Ok(output) => output,
                    Err(e) => t!("tool_exec_error", err = e).to_string(),
                },
                Err(e) => t!("tool_args_error", err = e).to_string(),
            }
        } else {
            t!("tool_not_found", name = tool_name).to_string()
        }
    }

    fn update_session_status(&self, session: &mut Session, status: SessionStatus) {
        let state_str = match status {
            SessionStatus::Idle => "Idle",
            SessionStatus::Busy => "Busy",
        };
        session.status = status;
        self.bus.publish(SystemEvent::AgentStateChanged {
            session_id: session.id.clone(),
            state: state_str.to_string(),
        });
    }

    pub async fn chat(&self, session: &mut Session, input: String) -> Result<CommandOutcome> {
        if session.status == SessionStatus::Busy {
            return Err(anyhow!("Session is busy"));
        }
        self.update_session_status(session, SessionStatus::Busy);

        let mut input = input;
        let mut depth = 0;
        const MAX_DEPTH: usize = 10;

        while input.starts_with('/') {
            let (cmd_name, parts, raw_args) = match parse_command(&input) {
                Ok((name, args, raw)) => (name, args, raw),
                Err(e) => {
                    self.update_session_status(session, SessionStatus::Idle);
                    return Err(e);
                }
            };

            let lookup_name = cmd_name.strip_prefix('/').unwrap_or(&cmd_name);
            if let Some(command) = self.commands.get(lookup_name) {
                match command {
                    CommandType::Builtin(cmd) => {
                        let ctx = CommandContext {
                            session_id: session.id.clone(),
                            event_bus: self.bus.clone(),
                            registry: &self.commands,
                        };
                        let res = cmd.execute(&ctx, parts).await;
                        self.update_session_status(session, SessionStatus::Idle);
                        return res;
                    }
                    CommandType::Custom(config) => {
                        depth += 1;
                        if depth > MAX_DEPTH {
                            self.update_session_status(session, SessionStatus::Idle);
                            return Err(anyhow!(t!("command_recursion_limit")));
                        }

                        let cwd = std::env::current_dir()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|_| "Unknown".to_string());
                        let workspace_root = self.workspace_root.to_string_lossy().to_string();

                        let template_context = TemplateContext::new(
                            raw_args.clone(),
                            parts.clone(),
                            cmd_name.clone(),
                            cwd,
                            workspace_root,
                        );

                        let engine = TemplateEngine::new();
                        match engine.render(&config.template, &template_context) {
                            Ok(rendered) => input = rendered,
                            Err(e) => {
                                self.update_session_status(session, SessionStatus::Idle);
                                return Err(e);
                            }
                        }
                        continue;
                    }
                    CommandType::Remote(_) => {
                        self.update_session_status(session, SessionStatus::Idle);
                        return Err(anyhow!("Remote commands cannot be executed directly"));
                    }
                }
            } else {
                self.update_session_status(session, SessionStatus::Idle);
                return Err(anyhow!(t!("command_not_found", name = cmd_name)));
            }
        }

        let result = self.process_turn(session, input).await;

        self.update_session_status(session, SessionStatus::Idle);
        match result {
            Ok(output) => Ok(CommandOutcome {
                output: Some(output),
            }),
            Err(e) => Err(e),
        }
    }

    pub async fn resolve_approval(
        &self,
        session: &mut Session,
        call_id: &str,
        approved: bool,
    ) -> Result<String> {
        if session.status == SessionStatus::Busy {
            return Err(anyhow!("Session is busy"));
        }
        self.update_session_status(session, SessionStatus::Busy);

        let result = self
            .resolve_approval_inner(session, call_id, approved)
            .await;

        self.update_session_status(session, SessionStatus::Idle);
        result
    }

    async fn resolve_approval_inner(
        &self,
        session: &mut Session,
        call_id: &str,
        approved: bool,
    ) -> Result<String> {
        let approval = session
            .pending_approvals
            .remove(call_id)
            .ok_or_else(|| anyhow!("No pending approval found for call_id: {}", call_id))?;

        let result = if approved {
            if let Some(tool) = self.tools.get(&approval.tool_name) {
                match serde_json::from_str::<serde_json::Value>(&approval.args) {
                    Ok(args) => match tool.execute(args).await {
                        Ok(output) => output,
                        Err(e) => t!("tool_exec_error", err = e).to_string(),
                    },
                    Err(e) => t!("tool_args_error", err = e).to_string(),
                }
            } else {
                t!("tool_not_found", name = approval.tool_name).to_string()
            }
        } else {
            "User denied permission to execute this tool.".to_string()
        };

        self.bus.publish(SystemEvent::ToolExecuted {
            tool: approval.tool_name.clone(),
            result: result.clone(),
        });

        let tool_msg = Message {
            role: Role::Tool,
            content: Some(result.clone()),
            tool_calls: None,
            tool_call_id: Some(call_id.to_string()),
            reasoning_summary: None,
            reasoning_raw: None,
        };
        session.add_message(tool_msg)?;

        if !session.pending_approvals.is_empty() {
            return Ok(result);
        }

        // Resume batch execution if any
        if !session.pending_batch.is_empty() {
            let batch = std::mem::take(&mut session.pending_batch);
            if let Some(msg) = self.process_tool_batch(session, batch).await? {
                return Ok(msg);
            }
        }

        self.run_turn_loop(session, 0).await
    }

    /// Process a batch of tool calls with selective parallelism and
    /// deterministic ordering.
    ///
    /// Phase 1 (sequential, in order): resolve permissions. Deny results are
    /// synthesized without execution; the first Ask parks the current call in
    /// `pending_approvals`, the remaining calls in `pending_batch`, and stops
    /// scheduling — later calls MUST NOT execute behind an unanswered Ask.
    ///
    /// Phase 2: the Allow-marked prefix executes via [`ToolCallRuntime`]
    /// (Parallel tools concurrently, Sequential tools exclusively).
    ///
    /// Phase 3: results are appended to the session in the original
    /// `tool_calls` order regardless of completion order.
    async fn process_tool_batch(
        &self,
        session: &mut Session,
        mut calls: Vec<ToolCall>,
    ) -> Result<Option<String>> {
        enum Slot {
            Denied(String),
            Allowed,
        }

        // Phase 1: permission checks in deterministic order.
        let mut slots: Vec<Slot> = Vec::new();
        let mut scheduled: Vec<ScheduledCall> = Vec::new();
        let mut parked_at: Option<usize> = None;

        for i in 0..calls.len() {
            let call = &calls[i];
            let tool_name = &call.function.name;
            let args: serde_json::Value =
                serde_json::from_str(&call.function.arguments).unwrap_or(serde_json::Value::Null);

            match permission::resolve(&self.config.permissions, tool_name, &args) {
                permission::PermissionDecision::Deny { reason, .. } => {
                    slots.push(Slot::Denied(reason));
                }
                permission::PermissionDecision::Ask { matched_rule } => {
                    self.emit_permission_request(tool_name, &call.id, matched_rule);
                    parked_at = Some(i);
                    break;
                }
                permission::PermissionDecision::Allow => {
                    scheduled.push(ScheduledCall {
                        index: slots.len(),
                        tool_name: tool_name.clone(),
                        args: call.function.arguments.clone(),
                    });
                    slots.push(Slot::Allowed);
                }
            }
        }

        // Phase 2: execute the allowed prefix via the runtime.
        let outcomes: std::collections::HashMap<usize, String> = if scheduled.is_empty() {
            Default::default()
        } else {
            self.tool_runtime
                .execute(
                    scheduled,
                    |call| {
                        self.tools
                            .get(&call.tool_name)
                            .map(|t| t.execution_mode())
                            .unwrap_or(common::tool::ExecutionMode::Sequential)
                    },
                    |call| async move { self.run_tool(&call.tool_name, &call.args).await },
                )
                .await
                .into_iter()
                .map(|o| (o.index, o.result))
                .collect()
        };

        // Phase 3: append results in original tool_calls order.
        for (pos, slot) in slots.iter().enumerate() {
            let (tool_name, result) = match slot {
                Slot::Denied(reason) => (&calls[pos].function.name, reason.clone()),
                Slot::Allowed => {
                    let name = &calls[pos].function.name;
                    (name, outcomes.get(&pos).cloned().unwrap_or_default())
                }
            };

            self.bus.publish(SystemEvent::ToolExecuted {
                tool: tool_name.clone(),
                result: result.clone(),
            });

            let tool_msg = Message {
                role: Role::Tool,
                content: Some(result),
                tool_calls: None,
                tool_call_id: Some(calls[pos].id.clone()),
                reasoning_summary: None,
                reasoning_raw: None,
            };
            session.add_message(tool_msg)?;
        }

        // Park the Ask-gated call and everything after it.
        if let Some(i) = parked_at {
            let call = &calls[i];
            session.pending_approvals.insert(
                call.id.clone(),
                PendingApproval {
                    call_id: call.id.clone(),
                    tool_name: call.function.name.clone(),
                    args: call.function.arguments.clone(),
                },
            );
            session.pending_batch = calls.split_off(i + 1);

            return Ok(Some(
                "Permission required: approve tool execution to continue.".to_string(),
            ));
        }
        Ok(None)
    }

    async fn process_turn(&self, session: &mut Session, input: String) -> Result<String> {
        let user_msg = Message {
            role: Role::User,
            content: Some(input.clone()),
            tool_calls: None,
            tool_call_id: None,
            reasoning_summary: None,
            reasoning_raw: None,
        };
        session.add_message(user_msg)?;

        self.bus.publish(SystemEvent::MessageReceived {
            content: input,
            role: "user".to_string(),
            kind: None,
        });

        self.run_turn_loop(session, 0).await
    }

    async fn run_turn_loop(&self, session: &mut Session, start_turn: u32) -> Result<String> {
        let mut current_turn = start_turn;

        let agents_file = self.workspace_root.join("AGENTS.md");
        let custom_rules = tokio::fs::read_to_string(agents_file).await.ok();

        let context = SystemPromptContext::capture(
            Some(&self.workspace_root),
            self.config.instructions.clone(),
            custom_rules,
        );

        loop {
            if current_turn >= MAX_TURNS {
                return Err(anyhow!(t!("max_turns_reached")));
            }
            current_turn += 1;

            let system_prompt = self
                .prompt_builder
                .build_from_context(&context)
                .map_err(|e| anyhow!("Failed to render system prompt: {}", e))?;
            let system_msg = Message {
                role: Role::System,
                content: Some(system_prompt),
                tool_calls: None,
                tool_call_id: None,
                reasoning_summary: None,
                reasoning_raw: None,
            };

            let rendered = session
                .render_context(
                    &[system_msg],
                    self.config.context_limits,
                    &DefaultTokenEstimator,
                )
                .map_err(|e| anyhow!(e))?;

            if rendered.dropped_turns > 0 {
                tracing::warn!(
                    "Context compaction triggered: dropped {} turns to stay within limits",
                    rendered.dropped_turns
                );
            }

            let req = CompletionRequest {
                messages: rendered.messages,
                temperature: None,
                max_tokens: None,
                tools: self.get_tool_definitions(),
                reasoning: self.should_enable_reasoning(session),
                request_overrides: None,
            };

            let response_msg = self.provider.complete(req).await?;

            // Raw reasoning is never persisted into the session context;
            // summaries are only stored when explicitly configured.
            let mut stored_msg = response_msg.clone();
            stored_msg.reasoning_raw = None;
            if self.reasoning_config.store != ReasoningStorage::Summary {
                stored_msg.reasoning_summary = None;
            }
            session.add_message(stored_msg)?;

            if let Some(content) = &response_msg.content {
                self.bus.publish(SystemEvent::MessageReceived {
                    content: content.clone(),
                    role: "assistant".to_string(),
                    kind: None,
                });
            }

            if self.reasoning_config.expose == ReasoningExposure::Summary {
                if let Some(reasoning_content) = &response_msg.reasoning_raw {
                    self.bus.publish(SystemEvent::MessageReceived {
                        content: reasoning_content.clone(),
                        role: "system".to_string(),
                        kind: Some("reasoning_summary".to_string()),
                    });
                } else if let Some(reasoning_summary) = &response_msg.reasoning_summary {
                    self.bus.publish(SystemEvent::MessageReceived {
                        content: reasoning_summary.clone(),
                        role: "system".to_string(),
                        kind: Some("reasoning_summary".to_string()),
                    });
                }
            }

            if let Some(tool_calls) = &response_msg.tool_calls {
                if tool_calls.is_empty() {
                    return Ok(response_msg.content.unwrap_or_default());
                }

                if let Some(msg) = self.process_tool_batch(session, tool_calls.clone()).await? {
                    return Ok(msg);
                }
            } else {
                return Ok(response_msg.content.unwrap_or_default());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::agent::config::{AgentPermissions, PermissionLevel};
    use crate::agent::permission::{self, PermissionDecision};

    fn legacy_resolve(
        permissions: &AgentPermissions,
        tool_name: &str,
    ) -> PermissionDecision {
        permission::resolve(permissions, tool_name, &serde_json::Value::Null)
    }

    #[test]
    fn bash_permission_gates_execute_command_tool() {
        // Regression: `CommandTool` reports its name as "execute_command", not
        // "run_command". Previously the `bash` level was never applied and the
        // command tool silently fell through to the `skill` level.
        let perms = AgentPermissions {
            bash: PermissionLevel::Ask,
            edit: PermissionLevel::Deny,
            skill: PermissionLevel::Allow,
            ..AgentPermissions::default()
        };
        assert!(matches!(
            legacy_resolve(&perms, "execute_command"),
            PermissionDecision::Ask { .. }
        ));
    }

    #[test]
    fn edit_permission_gates_write_tools() {
        let perms = AgentPermissions {
            edit: PermissionLevel::Deny,
            ..AgentPermissions::default()
        };
        assert!(matches!(
            legacy_resolve(&perms, "write_file"),
            PermissionDecision::Deny { .. }
        ));
        assert!(matches!(
            legacy_resolve(&perms, "replace_in_file"),
            PermissionDecision::Deny { .. }
        ));
        assert!(matches!(
            legacy_resolve(&perms, "delete_file"),
            PermissionDecision::Deny { .. }
        ));
    }

    #[test]
    fn unknown_tools_fall_back_to_skill() {
        let perms = AgentPermissions {
            skill: PermissionLevel::Deny,
            ..AgentPermissions::default()
        };
        assert!(matches!(
            legacy_resolve(&perms, "some_skill_tool"),
            PermissionDecision::Deny { .. }
        ));
    }

    #[test]
    fn per_tool_overrides_take_precedence() {
        let perms = AgentPermissions {
            bash: PermissionLevel::Allow,
            overrides: [("execute_command".to_string(), PermissionLevel::Deny)]
                .into_iter()
                .collect(),
            ..AgentPermissions::default()
        };
        assert!(matches!(
            legacy_resolve(&perms, "execute_command"),
            PermissionDecision::Deny { .. }
        ));
    }
}
