use crate::bus::{EventBus, SystemEvent};
use tracing::{debug, error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

pub fn init() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // We default to pretty logs for CLI usage.
    // In the future, we can switch to JSON based on an env var or flag.
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .pretty();

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}

pub async fn start_event_logger(bus: &EventBus) {
    let mut rx = bus.subscribe_raw();

    tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            match event {
                SystemEvent::MessageReceived { role, content } => {
                    info!(target: "bus", event = "message_received", role = %role, content = %content);
                }
                SystemEvent::ToolExecuted { tool, result } => {
                    info!(target: "bus", event = "tool_executed", tool = %tool, result_preview = %&result[..std::cmp::min(result.len(), 50)]);
                }
                SystemEvent::PermissionRequest {
                    operation,
                    tool_name,
                    call_id,
                } => {
                    info!(target: "bus", event = "permission_request", operation = %operation, tool = %tool_name, call_id = %call_id);
                }
                SystemEvent::Error { message } => {
                    error!(target: "bus", event = "error", message = %message);
                }
                SystemEvent::AgentStateChanged { session_id, state } => {
                    debug!(target: "bus", event = "state_change", session_id = %session_id, state = %state);
                }
                SystemEvent::Shutdown => {
                    info!(target: "bus", event = "shutdown");
                }
            }
        }
    });
}
