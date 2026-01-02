pub mod config;
pub mod prompt;

use crate::session::{Session, SessionStatus};
use common::llm::{LLMProvider, CompletionRequest, Message, Role, ToolDefinition, ToolFunctionDefinition};
use common::bus::{EventBus, SystemEvent};
use common::tool::Tool;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use std::collections::HashMap;
use rust_i18n::t;
use self::config::AgentConfig;
use self::prompt::SystemPromptBuilder;

const MAX_TURNS: u32 = 1000;

pub struct Agent {
    provider: Box<dyn LLMProvider>,
    bus: Arc<EventBus>,
    tools: HashMap<String, Box<dyn Tool>>,
    config: AgentConfig,
}

impl Agent {
    pub fn new(provider: Box<dyn LLMProvider>, bus: Arc<EventBus>, config: AgentConfig) -> Self {
        Self {
            provider,
            bus,
            tools: HashMap::new(),
            config,
        }
    }

    pub fn register_tool(&mut self, tool: Box<dyn Tool>) {
        self.tools.insert(tool.name().to_string(), tool);
    }

    fn get_tool_definitions(&self) -> Option<Vec<ToolDefinition>> {
        if self.tools.is_empty() {
            None
        } else {
            Some(self.tools.values().map(|t| ToolDefinition {
                kind: "function".to_string(),
                function: ToolFunctionDefinition {
                    name: t.name().to_string(),
                    description: t.description().to_string(),
                    parameters: t.schema(),
                }
            }).collect())
        }
    }

    async fn execute_tool(&self, tool_name: &str, args_str: &str) -> String {
        if let Some(tool) = self.tools.get(tool_name) {
            match serde_json::from_str::<serde_json::Value>(args_str) {
                Ok(args) => {
                    match tool.execute(args).await {
                        Ok(output) => output,
                        Err(e) => t!("tool_exec_error", err = e).to_string(),
                    }
                },
                Err(e) => t!("tool_args_error", err = e).to_string(),
            }
        } else {
            t!("tool_not_found", name = tool_name).to_string()
        }
    }

    pub async fn chat(&self, session: &mut Session, input: String) -> Result<String> {
        if session.status == SessionStatus::Busy {
            return Err(anyhow!("Session is busy"));
        }
        session.status = SessionStatus::Busy;

        let result = self.process_turn(session, input).await;

        session.status = SessionStatus::Idle;
        result
    }

    async fn process_turn(&self, session: &mut Session, input: String) -> Result<String> {
        let user_msg = Message {
            role: Role::User,
            content: Some(input.clone()),
            tool_calls: None,
            tool_call_id: None,
        };
        session.add_message(user_msg);
        
        self.bus.publish(SystemEvent::MessageReceived { 
            content: input, 
            role: "user".to_string() 
        });

        let mut current_turn = 0;

        loop {
            if current_turn >= MAX_TURNS {
                return Err(anyhow!(t!("max_turns_reached")));
            }
            current_turn += 1;

            let system_prompt = SystemPromptBuilder::build(&self.config);
            let system_msg = Message {
                role: Role::System,
                content: Some(system_prompt),
                tool_calls: None,
                tool_call_id: None,
            };

            let mut messages = vec![system_msg];
            messages.extend(session.history.clone());

            let req = CompletionRequest {
                messages,
                temperature: None,
                max_tokens: None,
                tools: self.get_tool_definitions(),
            };

            let response_msg = self.provider.complete(req).await?;
            session.add_message(response_msg.clone());
            
            if let Some(content) = &response_msg.content {
                self.bus.publish(SystemEvent::MessageReceived { 
                    content: content.clone(), 
                    role: "assistant".to_string() 
                });
            }

            if let Some(tool_calls) = &response_msg.tool_calls {
                if tool_calls.is_empty() {
                    return Ok(response_msg.content.unwrap_or_default());
                }

                for call in tool_calls {
                    let tool_name = &call.function.name;
                    let args_str = &call.function.arguments;
                    
                    let result = self.execute_tool(tool_name, args_str).await;

                    self.bus.publish(SystemEvent::ToolExecuted {
                        tool: tool_name.clone(),
                        result: result.clone()
                    });

                    let tool_msg = Message {
                        role: Role::Tool,
                        content: Some(result),
                        tool_calls: None,
                        tool_call_id: Some(call.id.clone()),
                    };
                    session.add_message(tool_msg);
                }
            } else {
                return Ok(response_msg.content.unwrap_or_default());
            }
        }
    }
}
