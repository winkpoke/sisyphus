pub mod config;
pub mod prompt;

use self::config::{AgentConfig, PermissionLevel};
use self::prompt::SystemPromptBuilder;
use crate::command::loader::CommandLoader;
use crate::command::parser::parse_command;
use crate::command::{
    builtins, CommandContext, CommandEffect, CommandOutcome, CommandRegistry, CommandType,
};
use crate::session::context::DefaultTokenEstimator;
use crate::session::{PendingApproval, Session, SessionStatus};
use anyhow::{anyhow, Result};
use chrono::Utc;
use common::bus::{EventBus, SystemEvent};
use common::llm::{
    CompletionRequest, LLMProvider, Message, Role, ToolCall, ToolDefinition, ToolFunctionDefinition,
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
            config,
            commands: CommandRegistry::new(),
            workspace_root,
        };
        agent.register_builtins();
        // Load custom commands from .sisyphus/command or config
        let cmd_path = agent.config.get_command_path();
        if let Ok(commands) = CommandLoader::load_from_dir(cmd_path) {
            for (name, config) in commands {
                agent.commands.register_custom(&name, config);
            }
        }
        agent
    }

    fn register_builtins(&mut self) {
        self.commands
            .register_builtin(Box::new(builtins::HelpCommand));
        self.commands
            .register_builtin(Box::new(builtins::ExitCommand));
        self.commands
            .register_builtin(Box::new(builtins::QuitCommand));
        self.commands
            .register_builtin(Box::new(builtins::NewSessionCommand));
        self.commands
            .register_builtin(Box::new(builtins::ClearHistoryCommand));
    }

    pub fn list_commands(&self) -> Vec<crate::command::CommandInfo> {
        self.commands.list()
    }

    pub fn register_tool(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
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

    fn get_permission_level(&self, tool_name: &str) -> &PermissionLevel {
        if let Some(level) = self.config.permissions.overrides.get(tool_name) {
            return level;
        }

        match tool_name {
            "run_command" => &self.config.permissions.bash,
            "write_file" | "replace_in_file" | "delete_file" => &self.config.permissions.edit,
            _ => &self.config.permissions.skill,
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

    pub async fn chat(&self, session: &mut Session, input: String) -> Result<CommandOutcome> {
        if session.status == SessionStatus::Busy {
            return Err(anyhow!("Session is busy"));
        }
        session.status = SessionStatus::Busy;

        let mut input = input;
        let mut depth = 0;
        const MAX_DEPTH: usize = 10;

        while input.starts_with('/') {
            let (cmd_name, parts, raw_args) = match parse_command(&input) {
                Ok((name, args, raw)) => (name, args, raw),
                Err(e) => {
                    session.status = SessionStatus::Idle;
                    return Err(e);
                }
            };

            if let Some(command) = self.commands.get(&cmd_name) {
                match command {
                    CommandType::Builtin(cmd) => {
                        let ctx = CommandContext {
                            session_id: session.id.clone(),
                            event_bus: self.bus.clone(),
                            registry: &self.commands,
                        };
                        let res = cmd.execute(&ctx, parts).await;

                        // Handle command effects
                        if let Ok(outcome) = &res {
                            match outcome.effect {
                                CommandEffect::ClearHistory => {
                                    session.clear_context();
                                }
                                CommandEffect::NewSession => {
                                    session.clear_context();
                                    session.id = uuid::Uuid::new_v4().to_string();
                                    session.created_at = Utc::now();
                                }
                                _ => {}
                            }
                        }

                        session.status = SessionStatus::Idle;
                        return res;
                    }
                    CommandType::Custom(config) => {
                        depth += 1;
                        if depth > MAX_DEPTH {
                            session.status = SessionStatus::Idle;
                            return Err(anyhow!(t!("command_recursion_limit")));
                        }
                        // For custom commands, we use the raw_args directly.
                        if config.template.contains("{{args}}") {
                            input = config.template.replace("{{args}}", &raw_args);
                        } else {
                            input = config.template.clone();
                        }
                        continue;
                    }
                }
            } else {
                session.status = SessionStatus::Idle;
                return Err(anyhow!(t!("command_not_found", name = cmd_name)));
            }
        }

        let result = self.process_turn(session, input).await;

        session.status = SessionStatus::Idle;
        match result {
            Ok(output) => Ok(CommandOutcome {
                output: Some(output),
                effect: CommandEffect::None,
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
        session.status = SessionStatus::Busy;

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
        };
        session.add_message(tool_msg)?;

        if !session.pending_approvals.is_empty() {
            session.status = SessionStatus::Idle;
            return Ok(result);
        }

        // Resume batch execution if any
        if !session.pending_batch.is_empty() {
            let batch = std::mem::take(&mut session.pending_batch);
            if let Some(msg) = self.process_tool_batch(session, batch).await? {
                session.status = SessionStatus::Idle;
                return Ok(msg);
            }
        }

        let output = self.run_turn_loop(session, 0).await;

        session.status = SessionStatus::Idle;
        output
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
                        tool_call_id: Some(call.id.clone()),
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
        };
        session.add_message(user_msg)?;

        self.bus.publish(SystemEvent::MessageReceived {
            content: input,
            role: "user".to_string(),
        });

        self.run_turn_loop(session, 0).await
    }

    async fn run_turn_loop(&self, session: &mut Session, start_turn: u32) -> Result<String> {
        let mut current_turn = start_turn;
        let snapshot = SystemPromptBuilder::snapshot(Some(&self.workspace_root)).await;

        loop {
            if current_turn >= MAX_TURNS {
                return Err(anyhow!(t!("max_turns_reached")));
            }
            current_turn += 1;

            let system_prompt = SystemPromptBuilder::build(&self.config, &snapshot);
            let system_msg = Message {
                role: Role::System,
                content: Some(system_prompt),
                tool_calls: None,
                tool_call_id: None,
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
            };

            let response_msg = self.provider.complete(req).await?;
            session.add_message(response_msg.clone())?;

            if let Some(content) = &response_msg.content {
                self.bus.publish(SystemEvent::MessageReceived {
                    content: content.clone(),
                    role: "assistant".to_string(),
                });
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
