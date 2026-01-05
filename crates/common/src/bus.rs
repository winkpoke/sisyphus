use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SystemEventKind {
    AgentStateChanged,
    MessageReceived,
    ToolExecuted,
    PermissionRequest,
    Error,
    Shutdown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

impl SystemEvent {
    pub fn kind(&self) -> SystemEventKind {
        match self {
            SystemEvent::AgentStateChanged { .. } => SystemEventKind::AgentStateChanged,
            SystemEvent::MessageReceived { .. } => SystemEventKind::MessageReceived,
            SystemEvent::ToolExecuted { .. } => SystemEventKind::ToolExecuted,
            SystemEvent::PermissionRequest { .. } => SystemEventKind::PermissionRequest,
            SystemEvent::Error { .. } => SystemEventKind::Error,
            SystemEvent::Shutdown => SystemEventKind::Shutdown,
        }
    }
}

pub struct Subscription {
    handle: JoinHandle<()>,
}

impl Subscription {
    pub fn abort(&self) {
        self.handle.abort();
    }
}

impl Drop for Subscription {
    fn drop(&mut self) {
        self.handle.abort();
    }
}

pub struct EventBus {
    global_tx: broadcast::Sender<SystemEvent>,
    topic_txs: HashMap<SystemEventKind, broadcast::Sender<SystemEvent>>,
}

impl EventBus {
    pub fn new(capacity: usize) -> Self {
        let (global_tx, _) = broadcast::channel(capacity);
        let mut topic_txs = HashMap::new();

        let kinds = [
            SystemEventKind::AgentStateChanged,
            SystemEventKind::MessageReceived,
            SystemEventKind::ToolExecuted,
            SystemEventKind::PermissionRequest,
            SystemEventKind::Error,
            SystemEventKind::Shutdown,
        ];

        for kind in kinds {
            let (tx, _) = broadcast::channel(capacity);
            topic_txs.insert(kind, tx);
        }

        Self {
            global_tx,
            topic_txs,
        }
    }

    pub fn publish(&self, event: SystemEvent) -> usize {
        let mut count = 0;

        // Publish to specific topic channel
        let kind = event.kind();
        if let Some(tx) = self.topic_txs.get(&kind) {
            if let Ok(n) = tx.send(event.clone()) {
                count += n;
            }
        }

        // Publish to global channel
        // Ignore errors if no active subscribers
        if let Ok(n) = self.global_tx.send(event) {
            count += n;
        }

        count
    }

    /// Returns a raw broadcast receiver for low-level handling (e.g. in loops or tests).
    /// This subscribes to the global channel, receiving all events.
    pub fn subscribe_raw(&self) -> broadcast::Receiver<SystemEvent> {
        self.global_tx.subscribe()
    }

    /// Subscribes to all events with a callback.
    /// Returns a Subscription handle that cancels the subscription when dropped.
    pub fn subscribe_all<F>(&self, mut callback: F) -> Subscription
    where
        F: FnMut(SystemEvent) + Send + 'static,
    {
        let mut rx = self.global_tx.subscribe();
        let handle = tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        callback(event);
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
        Subscription { handle }
    }

    /// Subscribes to events of a specific kind with a callback.
    /// Returns a Subscription handle that cancels the subscription when dropped.
    pub fn subscribe<F>(&self, kind: SystemEventKind, mut callback: F) -> Subscription
    where
        F: FnMut(SystemEvent) + Send + 'static,
    {
        let tx = self
            .topic_txs
            .get(&kind)
            .expect("EventBus initialized with all SystemEventKind variants");
        let mut rx = tx.subscribe();
        let handle = tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        callback(event);
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
        Subscription { handle }
    }

    /// Subscribes to the next event of a specific kind with a callback, then unsubscribes.
    /// Returns a Subscription handle that cancels the subscription when dropped.
    pub fn subscribe_once<F>(&self, kind: SystemEventKind, callback: F) -> Subscription
    where
        F: FnOnce(SystemEvent) + Send + 'static,
    {
        let tx = self
            .topic_txs
            .get(&kind)
            .expect("EventBus initialized with all SystemEventKind variants");
        let mut rx = tx.subscribe();
        let mut callback_opt = Some(callback);
        let handle = tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(event) => {
                        if let Some(cb) = callback_opt.take() {
                            cb(event);
                        }
                        break;
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
        Subscription { handle }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_subscribe_specific() {
        let bus = EventBus::new(100);
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);

        let _sub = bus.subscribe(SystemEventKind::MessageReceived, move |event| {
            tx.try_send(event).unwrap();
        });

        // Publish correct event
        bus.publish(SystemEvent::MessageReceived {
            content: "hello".to_string(),
            role: "user".to_string(),
        });

        // Publish incorrect event
        bus.publish(SystemEvent::Error {
            message: "oops".to_string(),
        });

        // Wait a bit
        sleep(Duration::from_millis(50)).await;

        // Check what we got
        let event = rx.recv().await.unwrap();
        assert_eq!(event.kind(), SystemEventKind::MessageReceived);

        // Should be empty now
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_subscribe_all() {
        let bus = EventBus::new(100);
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);

        let _sub = bus.subscribe_all(move |event| {
            tx.try_send(event).unwrap();
        });

        bus.publish(SystemEvent::MessageReceived {
            content: "hello".to_string(),
            role: "user".to_string(),
        });
        bus.publish(SystemEvent::Error {
            message: "oops".to_string(),
        });

        sleep(Duration::from_millis(50)).await;

        let event1 = rx.recv().await.unwrap();
        let event2 = rx.recv().await.unwrap();

        assert_eq!(event1.kind(), SystemEventKind::MessageReceived);
        assert_eq!(event2.kind(), SystemEventKind::Error);
    }
}
