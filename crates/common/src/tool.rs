use anyhow::Result;
use async_trait::async_trait;
use serde_json::Value;

/// How the tool-call runtime may schedule this tool.
///
/// - `Parallel`: read-only tools that may execute concurrently with other
///   Parallel tools.
/// - `Sequential` (default): tools that mutate workspace state, spawn
///   processes, or touch stateful external systems. They execute
///   exclusively — no other tool may run while one is in flight.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    Parallel,
    Sequential,
}

#[async_trait]
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    // We return a serde_json::Value representing the JSON Schema of arguments
    fn schema(&self) -> Value;
    async fn execute(&self, args: Value) -> Result<String>;

    /// Execution mode used by the tool-call runtime. Tools MUST override this
    /// to `Parallel` only when they are read-only and safe to run
    /// concurrently; the conservative default is `Sequential`.
    fn execution_mode(&self) -> ExecutionMode {
        ExecutionMode::Sequential
    }
}
