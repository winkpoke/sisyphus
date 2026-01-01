use common::llm::Message;
use serde::{Deserialize, Serialize};

pub mod manager;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    Idle,
    Busy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub history: Vec<Message>,
    pub status: SessionStatus,
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            history: Vec::new(),
            status: SessionStatus::Idle,
        }
    }
    
    pub fn add_message(&mut self, message: Message) {
        self.history.push(message);
    }
}
