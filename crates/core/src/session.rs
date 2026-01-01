use common::llm::Message;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub history: Vec<Message>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            history: Vec::new(),
        }
    }
    
    pub fn add_message(&mut self, message: Message) {
        self.history.push(message);
    }
}
