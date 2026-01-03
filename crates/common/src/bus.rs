use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum SystemEvent {
    AgentStateChanged {
        session_id: String,
        state: String,
    },
    MessageReceived {
        content: String,
        role: String,
    },
    ToolExecuted {
        tool: String,
        result: String,
    },
    PermissionRequest {
        operation: String,
        tool_name: String,
        call_id: String,
    },
    Error {
        message: String,
    },
    Shutdown,
}

pub struct EventBus {
    tx: broadcast::Sender<SystemEvent>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);
        Self { tx }
    }

    pub fn publish(&self, event: SystemEvent) -> usize {
        // Ignore errors if no active subscribers
        self.tx.send(event).unwrap_or(0)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<SystemEvent> {
        self.tx.subscribe()
    }
}
