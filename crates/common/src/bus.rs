use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::AtomicU64;
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
    DirectoryChanged,
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
        #[serde(skip_serializing_if = "Option::is_none")]
        kind: Option<String>,
    },
    ToolExecuted {
        tool: String,
        result: String,
    },
    PermissionRequest {
        operation: String,
        tool_name: String,
        call_id: String,
        /// The specific rule that triggered the Ask, if any (mode-fallback
        /// asks carry `None`).
        #[serde(default)]
        matched_rule: Option<String>,
        /// Name of the active [`crate::agent::config::PermissionMode`].
        #[serde(default)]
        mode: String,
    },
    Error {
        message: String,
    },
    Shutdown,
    DirectoryChanged {
        path: String,
    },
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
            SystemEvent::DirectoryChanged { .. } => SystemEventKind::DirectoryChanged,
        }
    }
}

/// Wrapper for system events with stable metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventEnvelope<T> {
    /// Monotonically increasing identifier (process-local)
    pub id: u64,
    /// Unix epoch timestamp in milliseconds (UTC)
    pub timestamp_ms: i64,
    /// The actual event payload
    pub event: T,
}

/// Event envelope for SystemEvent
pub type SystemEventEnvelope = EventEnvelope<SystemEvent>;

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
    global_tx: broadcast::Sender<SystemEventEnvelope>,
    topic_txs: HashMap<SystemEventKind, broadcast::Sender<SystemEventEnvelope>>,
    next_id: AtomicU64,
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
            SystemEventKind::DirectoryChanged,
        ];

        for kind in kinds {
            let (tx, _) = broadcast::channel(capacity);
            topic_txs.insert(kind, tx);
        }

        Self {
            global_tx,
            topic_txs,
            next_id: AtomicU64::new(0),
        }
    }

    pub fn publish(&self, event: SystemEvent) -> usize {
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let timestamp_ms = chrono::Utc::now().timestamp_millis();

        let envelope = SystemEventEnvelope {
            id,
            timestamp_ms,
            event,
        };

        let mut count = 0;

        let kind = envelope.event.kind();
        if let Some(Ok(n)) = self.topic_txs.get(&kind).map(|tx| tx.send(envelope.clone())) {
            count += n;
        }

        if let Ok(n) = self.global_tx.send(envelope) {
            count += n;
        }

        count
    }

    pub fn subscribe_raw(&self) -> broadcast::Receiver<SystemEventEnvelope> {
        self.global_tx.subscribe()
    }

    pub fn subscribe_all<F>(&self, mut callback: F) -> Subscription
    where
        F: FnMut(SystemEventEnvelope) + Send + 'static,
    {
        let mut rx = self.global_tx.subscribe();
        let handle = tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(envelope) => {
                        callback(envelope);
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
        Subscription { handle }
    }

    pub fn subscribe<F>(&self, kind: SystemEventKind, mut callback: F) -> Subscription
    where
        F: FnMut(SystemEventEnvelope) + Send + 'static,
    {
        let Some(tx) = self.topic_txs.get(&kind) else {
            tracing::error!(?kind, "EventBus missing topic channel; subscription is a no-op");
            return Subscription {
                handle: tokio::spawn(async {}),
            };
        };
        let mut rx = tx.subscribe();
        let handle = tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(envelope) => {
                        callback(envelope);
                    }
                    Err(broadcast::error::RecvError::Lagged(_)) => continue,
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
        });
        Subscription { handle }
    }

    pub fn subscribe_once<F>(&self, kind: SystemEventKind, callback: F) -> Subscription
    where
        F: FnOnce(SystemEventEnvelope) + Send + 'static,
    {
        let Some(tx) = self.topic_txs.get(&kind) else {
            tracing::error!(?kind, "EventBus missing topic channel; subscription is a no-op");
            return Subscription {
                handle: tokio::spawn(async {}),
            };
        };
        let mut rx = tx.subscribe();
        let mut callback_opt = Some(callback);
        let handle = tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(envelope) => {
                        if let Some(cb) = callback_opt.take() {
                            cb(envelope);
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

        let _sub = bus.subscribe(SystemEventKind::MessageReceived, move |envelope| {
            tx.try_send(envelope).unwrap();
        });

        bus.publish(SystemEvent::MessageReceived {
            content: "hello".to_string(),
            role: "user".to_string(),
            kind: None,
        });

        bus.publish(SystemEvent::Error {
            message: "oops".to_string(),
        });

        sleep(Duration::from_millis(50)).await;

        let envelope = rx.recv().await.unwrap();
        assert_eq!(envelope.event.kind(), SystemEventKind::MessageReceived);

        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn test_subscribe_all() {
        let bus = EventBus::new(100);
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);

        let _sub = bus.subscribe_all(move |envelope| {
            tx.try_send(envelope).unwrap();
        });

        bus.publish(SystemEvent::MessageReceived {
            content: "hello".to_string(),
            role: "user".to_string(),
            kind: None,
        });
        bus.publish(SystemEvent::Error {
            message: "oops".to_string(),
        });

        sleep(Duration::from_millis(50)).await;

        let envelope1 = rx.recv().await.unwrap();
        let envelope2 = rx.recv().await.unwrap();

        assert_eq!(envelope1.event.kind(), SystemEventKind::MessageReceived);
        assert_eq!(envelope2.event.kind(), SystemEventKind::Error);
    }

    #[tokio::test]
    async fn test_envelope_monotonic_ids() {
        let bus = EventBus::new(100);
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);

        let _sub = bus.subscribe_all(move |envelope| {
            tx.try_send(envelope).unwrap();
        });

        bus.publish(SystemEvent::MessageReceived {
            content: "hello".to_string(),
            role: "user".to_string(),
            kind: None,
        });
        bus.publish(SystemEvent::Error {
            message: "second".to_string(),
        });
        bus.publish(SystemEvent::Shutdown);

        sleep(Duration::from_millis(50)).await;

        let env1 = rx.recv().await.unwrap();
        let env2 = rx.recv().await.unwrap();
        let env3 = rx.recv().await.unwrap();

        assert_eq!(env1.id, 0);
        assert_eq!(env2.id, 1);
        assert_eq!(env3.id, 2);
    }

    #[tokio::test]
    async fn test_envelope_serialization() {
        let envelope = SystemEventEnvelope {
            id: 42,
            timestamp_ms: 1768631000000,
            event: SystemEvent::MessageReceived {
                content: "test".to_string(),
                role: "user".to_string(),
                kind: None,
            },
        };

        let json = serde_json::to_string(&envelope).unwrap();
        let deserialized: SystemEventEnvelope = serde_json::from_str(&json).unwrap();

        assert_eq!(envelope.id, deserialized.id);
        assert_eq!(envelope.timestamp_ms, deserialized.timestamp_ms);
        assert_eq!(envelope.event, deserialized.event);
    }

    #[tokio::test]
    async fn test_envelope_timestamp_in_utc() {
        let bus = EventBus::new(100);
        let (tx, mut rx) = tokio::sync::mpsc::channel(10);

        let _sub = bus.subscribe_all(move |envelope| {
            tx.try_send(envelope).unwrap();
        });

        let expected_min_timestamp = chrono::Utc::now().timestamp_millis();
        bus.publish(SystemEvent::Shutdown);

        sleep(Duration::from_millis(50)).await;

        let envelope = rx.recv().await.unwrap();
        let expected_max_timestamp = chrono::Utc::now().timestamp_millis();

        assert!(envelope.timestamp_ms >= expected_min_timestamp);
        assert!(envelope.timestamp_ms <= expected_max_timestamp);
    }
}
