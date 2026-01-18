use crate::bus::{EventBus, SystemEvent};
use std::fs::File;
use tracing::{debug, error, info};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Log output destination
#[derive(Debug, Clone)]
pub enum LogOutput {
    /// Log to stderr
    Stderr,
    /// Log to file at specified path
    File(String),
    /// Discard all logs (for interactive modes)
    Null,
}

/// Logging configuration
pub struct LogConfig {
    pub default_level: &'static str,
    pub output: LogOutput,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            default_level: "info",
            output: LogOutput::Stderr,
        }
    }
}

/// Initialize logging with flexible configuration
///
/// # Arguments
///
/// * `config` - LogConfig specifying level and output destination
pub fn init(config: LogConfig) -> anyhow::Result<()> {
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(config.default_level));

    let fmt_layer = fmt::layer()
        .with_target(false)
        .with_thread_ids(true)
        .with_line_number(true)
        .pretty();

    match config.output {
        LogOutput::Stderr => {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer.with_writer(std::io::stderr))
                .try_init()
                .map_err(|e| anyhow::anyhow!("Failed to initialize logging: {}", e))?;
        }
        LogOutput::File(path) => {
            let file = File::options().append(true).create(true).open(path)?;
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer.with_writer(file))
                .try_init()
                .map_err(|e| anyhow::anyhow!("Failed to initialize logging: {}", e))?;
        }
        LogOutput::Null => {
            tracing_subscriber::registry()
                .with(filter)
                .with(fmt_layer.with_writer(std::io::sink))
                .try_init()
                .map_err(|e| anyhow::anyhow!("Failed to initialize logging: {}", e))?;
        }
    }

    Ok(())
}

/// Initialize logging with default info level to stderr (backward compatibility)
pub fn init_with_defaults(default_level: &'static str) {
    init(LogConfig {
        default_level,
        output: LogOutput::Stderr,
    })
    .expect("Failed to initialize logging");
}

/// Initialize logging to info level (backward compatibility)
pub fn init_legacy() {
    init_with_defaults("info");
}

pub async fn start_event_logger(bus: &EventBus) {
    let sub = bus.subscribe_all(|envelope| match envelope.event {
        SystemEvent::MessageReceived { role, content, .. } => {
            info!(
                target: "bus",
                event_id = envelope.id,
                timestamp_ms = envelope.timestamp_ms,
                event = "message_received",
                role = %role,
                content = %content
            );
        }
        SystemEvent::ToolExecuted { tool, result } => {
            info!(
                target: "bus",
                event_id = envelope.id,
                timestamp_ms = envelope.timestamp_ms,
                event = "tool_executed",
                tool = %tool,
                result_preview = %result.chars().take(50).collect::<String>()
            );
        }
        SystemEvent::PermissionRequest {
            operation,
            tool_name,
            call_id,
        } => {
            info!(
                target: "bus",
                event_id = envelope.id,
                timestamp_ms = envelope.timestamp_ms,
                event = "permission_request",
                operation = %operation,
                tool = %tool_name,
                call_id = %call_id
            );
        }
        SystemEvent::Error { message } => {
            error!(
                target: "bus",
                event_id = envelope.id,
                timestamp_ms = envelope.timestamp_ms,
                event = "error",
                message = %message
            );
        }
        SystemEvent::AgentStateChanged { session_id, state } => {
            debug!(
                target: "bus",
                event_id = envelope.id,
                timestamp_ms = envelope.timestamp_ms,
                event = "state_change",
                session_id = %session_id,
                state = %state
            );
        }
        SystemEvent::Shutdown => {
            info!(
                target: "bus",
                event_id = envelope.id,
                timestamp_ms = envelope.timestamp_ms,
                event = "shutdown"
            );
        }
        SystemEvent::DirectoryChanged { path } => {
            info!(
                target: "bus",
                event_id = envelope.id,
                timestamp_ms = envelope.timestamp_ms,
                event = "directory_changed",
                path = %path
            );
        }
    });

    std::mem::forget(sub);
}
