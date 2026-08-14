pub mod config;
pub mod prompt;
pub mod registry;

use self::config::{AgentConfig, PermissionLevel};
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
    Role, ToolCall, ToolDefinition, ToolFunctionDefinition,
};
use common::tool::Tool;
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
}

enum ToolExecResult {
    Ok(String),
    PermissionRequired(String),
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
        };
        agent.register_builtins();
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

    fn get_permission_level(&self, tool_name: &str) -> PermissionLevel {
        if let Some(level) = self.config.permissions.overrides.get(tool_name) {
            return *level;
        }

        // Map each tool name to its permission category. These names MUST match
        // the `name()` returned by the tool implementation (e.g. `CommandTool`
        // returns "execute_command", not "run_command").
        match tool_name {
            "execute_command" => self.config.permissions.bash,
            "write_file" | "replace_in_file" | "delete_file" => self.config.permissions.edit,
            _ => self.config.permissions.skill,
        }
    }

    async fn execute_tool(&self, tool_name: &str, args_str: &str, call_id: &str) -> ToolExecResult {
        let permission = self.get_permission_level(tool_name);
        match permission {
            PermissionLevel::Deny => {
                return ToolExecResult::Ok(
                    "Permission denied: tool execution is set to Deny.".to_string(),
                );
            }
            PermissionLevel::Ask => {
                self.bus.publish(SystemEvent::PermissionRequest {
                    operation: "tool_execution".to_string(),
                    tool_name: tool_name.to_string(),
                    call_id: call_id.to_string(),
                });
                return ToolExecResult::PermissionRequired(
                    "Permission required: approve tool execution to continue.".to_string(),
                );
            }
            PermissionLevel::Allow => {}
        }

        if let Some(tool) = self.tools.get(tool_name) {
            match serde_json::from_str::<serde_json::Value>(args_str) {
                Ok(args) => match tool.execute(args).await {
                    Ok(output) => ToolExecResult::Ok(output),
                    Err(e) => ToolExecResult::Ok(t!("tool_exec_error", err = e).to_string()),
                },
                Err(e) => ToolExecResult::Ok(t!("tool_args_error", err = e).to_string()),
            }
        } else {
            ToolExecResult::Ok(t!("tool_not_found", name = tool_name).to_string())
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

    async fn process_tool_batch(
        &self,
        session: &mut Session,
        mut calls: Vec<ToolCall>,
    ) -> Result<Option<String>> {
        let mut i = 0;
        while i < calls.len() {
            let call = &calls[i];
            let tool_name = &call.function.name;
            let args_str = &call.function.arguments;

            let execution = self.execute_tool(tool_name, args_str, &call.id).await;

            match execution {
                ToolExecResult::Ok(result) => {
                    self.bus.publish(SystemEvent::ToolExecuted {
                        tool: tool_name.clone(),
                        result: result.clone(),
                    });

                    let tool_msg = Message {
                        role: Role::Tool,
                        content: Some(result),
                        tool_calls: None,
                        tool_call_id: Some(call.id.to_string()),
                        reasoning_summary: None,
                        reasoning_raw: None,
                    };
                    session.add_message(tool_msg)?;
                }
                ToolExecResult::PermissionRequired(msg) => {
                    session.pending_approvals.insert(
                        call.id.clone(),
                        PendingApproval {
                            call_id: call.id.clone(),
                            tool_name: tool_name.clone(),
                            args: args_str.clone(),
                        },
                    );

                    let remaining = calls.split_off(i + 1);
                    session.pending_batch = remaining;

                    return Ok(Some(msg));
                }
            }
            i += 1;
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
                reasoning: self.should_enable_reasoning(&session),
                request_overrides: None,
            };

            let response_msg = self.provider.complete(req).await?;
            session.add_message(response_msg.clone())?;

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
    use super::Agent;
    use crate::agent::config::{AgentConfig, AgentPermissions, PermissionLevel};
    use async_trait::async_trait;
    use common::bus::EventBus;
    use common::llm::{CompletionRequest, LLMProvider, Message};
    use futures::Stream;
    use std::path::PathBuf;
    use std::pin::Pin;
    use std::sync::Arc;

    // A no-op provider is sufficient: permission routing is pure and never
    // invokes the provider.
    struct StubProvider;
    #[async_trait]
    impl LLMProvider for StubProvider {
        fn model(&self) -> String {
            "stub".to_string()
        }
        async fn complete(&self, _req: CompletionRequest) -> anyhow::Result<Message> {
            unreachable!("StubProvider::complete is not invoked by these tests")
        }
        async fn stream(
            &self,
            _req: CompletionRequest,
        ) -> anyhow::Result<Pin<Box<dyn Stream<Item = anyhow::Result<String>> + Send>>> {
            unreachable!("StubProvider::stream is not invoked by these tests")
        }
    }

    fn build_agent(permissions: AgentPermissions) -> Agent {
        Agent::new(
            Box::new(StubProvider),
            Arc::new(EventBus::new(16)),
            AgentConfig {
                permissions,
                ..AgentConfig::default()
            },
            PathBuf::from("."),
        )
    }

    #[test]
    fn bash_permission_gates_execute_command_tool() {
        // Regression: `CommandTool` reports its name as "execute_command", not
        // "run_command". Previously the `bash` level was never applied and the
        // command tool silently fell through to the `skill` level.
        let agent = build_agent(AgentPermissions {
            bash: PermissionLevel::Ask,
            edit: PermissionLevel::Deny,
            skill: PermissionLevel::Allow,
            ..AgentPermissions::default()
        });
        assert_eq!(
            agent.get_permission_level("execute_command"),
            PermissionLevel::Ask
        );
    }

    #[test]
    fn edit_permission_gates_write_tools() {
        let agent = build_agent(AgentPermissions {
            edit: PermissionLevel::Deny,
            ..AgentPermissions::default()
        });
        assert_eq!(agent.get_permission_level("write_file"), PermissionLevel::Deny);
        assert_eq!(
            agent.get_permission_level("replace_in_file"),
            PermissionLevel::Deny
        );
        assert_eq!(agent.get_permission_level("delete_file"), PermissionLevel::Deny);
    }

    #[test]
    fn unknown_tools_fall_back_to_skill() {
        let agent = build_agent(AgentPermissions {
            skill: PermissionLevel::Deny,
            ..AgentPermissions::default()
        });
        assert_eq!(
            agent.get_permission_level("some_skill_tool"),
            PermissionLevel::Deny
        );
    }

    #[test]
    fn per_tool_overrides_take_precedence() {
        use std::collections::HashMap;
        let mut overrides = HashMap::new();
        overrides.insert("execute_command".to_string(), PermissionLevel::Deny);
        let agent = build_agent(AgentPermissions {
            bash: PermissionLevel::Allow,
            overrides,
            ..AgentPermissions::default()
        });
        assert_eq!(
            agent.get_permission_level("execute_command"),
            PermissionLevel::Deny
        );
    }
}
