use crate::session::Session;
use common::llm::{LLMProvider, CompletionRequest, Message, Role, ToolDefinition, ToolFunctionDefinition};
use common::bus::{EventBus, SystemEvent};
use common::tool::Tool;
use std::sync::Arc;
use anyhow::{Result, anyhow};
use std::collections::HashMap;

const MAX_TURNS: u32 = 1000;

pub struct Agent {
    provider: Box<dyn LLMProvider>,
    session: Session,
    bus: Arc<EventBus>,
    tools: HashMap<String, Box<dyn Tool>>,
}

impl Agent {
    pub fn new(provider: Box<dyn LLMProvider>, bus: Arc<EventBus>) -> Self {
        Self {
            provider,
            session: Session::new(),
            bus,
            tools: HashMap::new(),
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
                        Err(e) => format!("Error executing tool: {}", e),
                    }
                },
                Err(e) => format!("Error parsing arguments: {}", e),
            }
        } else {
            format!("Tool not found: {}", tool_name)
        }
    }

    pub async fn chat(&mut self, input: String) -> Result<String> {
        let user_msg = Message {
            role: Role::User,
            content: Some(input.clone()),
            tool_calls: None,
            tool_call_id: None,
        };
        self.session.add_message(user_msg);
        
        self.bus.publish(SystemEvent::MessageReceived { 
            content: input, 
            role: "user".to_string() 
        });

        let mut current_turn = 0;

        loop {
            if current_turn >= MAX_TURNS {
                return Err(anyhow!("Max turns reached"));
            }
            current_turn += 1;

            let req = CompletionRequest {
                messages: self.session.history.clone(),
                temperature: None,
                max_tokens: None,
                tools: self.get_tool_definitions(),
            };

            let response_msg = self.provider.complete(req).await?;
            self.session.add_message(response_msg.clone());
            
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
                    self.session.add_message(tool_msg);
                }
            } else {
                return Ok(response_msg.content.unwrap_or_default());
            }
        }
    }
}
