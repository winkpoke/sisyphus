use chrono::{DateTime, Utc};
use common::llm::{Message, ToolCall};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub mod context;

pub mod manager;

use context::{Context, ContextError, ContextLimits, RenderedContext, TokenEstimator};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PendingApproval {
    pub call_id: String,
    pub tool_name: String,
    pub args: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    Idle,
    Busy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Session {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub agent_id: Option<String>,
    context: Context,
    pub status: SessionStatus,
    #[serde(default)]
    pub pending_approvals: HashMap<String, PendingApproval>,
    #[serde(default)]
    pub pending_batch: Vec<ToolCall>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub agent_id: Option<String>,
    pub message_count: usize,
    pub status: SessionStatus,
}

impl Default for Session {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Session {
    pub fn new(agent_id: Option<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            created_at: Utc::now(),
            agent_id,
            context: Context::new(),
            status: SessionStatus::Idle,
            pending_approvals: HashMap::new(),
            pending_batch: Vec::new(),
        }
    }

    pub fn summary(&self) -> SessionSummary {
        SessionSummary {
            id: self.id.clone(),
            created_at: self.created_at,
            agent_id: self.agent_id.clone(),
            message_count: self.context.message_count(),
            status: self.status.clone(),
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

    pub fn estimate_tokens(&self) -> u32 {
        // Use default estimator and no limits to get total tokens
        let estimator = context::DefaultTokenEstimator;
        if let Ok(rendered) = self.context.render(&[], None, &estimator) {
            rendered.estimated_prompt_tokens
        } else {
            0
        }
    }
}
