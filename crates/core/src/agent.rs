use crate::session::Session;
use common::llm::{LLMProvider, CompletionRequest, Message, Role};
use common::bus::{EventBus, SystemEvent};
use std::sync::Arc;
use anyhow::Result;

pub struct Agent {
    provider: Box<dyn LLMProvider>,
    session: Session,
    bus: Arc<EventBus>,
}

impl Agent {
    pub fn new(provider: Box<dyn LLMProvider>, bus: Arc<EventBus>) -> Self {
        Self {
            provider,
            session: Session::new(),
            bus,
        }
    }

    pub async fn chat(&mut self, input: String) -> Result<String> {
        let user_msg = Message {
            role: Role::User,
            content: input.clone(),
        };
        self.session.add_message(user_msg.clone());
        
        self.bus.publish(SystemEvent::MessageReceived { 
            content: input, 
            role: "user".to_string() 
        });

        let req = CompletionRequest {
            messages: self.session.history.clone(),
            temperature: None,
            max_tokens: None,
        };

        // Emit thinking event?
        
        let response = self.provider.complete(req).await?;
        
        let assistant_msg = Message {
            role: Role::Assistant,
            content: response.clone(),
        };
        self.session.add_message(assistant_msg);
        
        self.bus.publish(SystemEvent::MessageReceived { 
            content: response.clone(), 
            role: "assistant".to_string() 
        });

        Ok(response)
    }
}
