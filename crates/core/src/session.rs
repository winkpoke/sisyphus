use common::llm::Message;
use serde::{Deserialize, Serialize};

pub mod context;

pub mod manager;

use context::{Context, ContextError, ContextLimits, RenderedContext, TokenEstimator};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    Idle,
    Busy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    context: Context,
    pub status: SessionStatus,
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            context: Context::new(),
            status: SessionStatus::Idle,
        }
    }

    pub fn add_message(&mut self, message: Message) -> anyhow::Result<()> {
        self.context.push_message(message).map_err(Into::into)
    }

    pub fn clear_context(&mut self) {
        self.context.clear();
    }

    pub fn history(&self) -> Vec<Message> {
        self.context.linear_messages()
    }

    pub fn render_context(
        &self,
        system_messages: &[Message],
        limits: Option<ContextLimits>,
        estimator: &dyn TokenEstimator,
    ) -> Result<RenderedContext, ContextError> {
        self.context.render(system_messages, limits, estimator)
    }
}
