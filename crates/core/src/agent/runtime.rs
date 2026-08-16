//! Selective parallel tool-call runtime (`add-parallel-tool-runtime`).
//!
//! Executes a batch of permission-cleared tool calls with selective
//! parallelism:
//!
//! - [`ExecutionMode::Parallel`] tools acquire a **read** lock on the gate and
//!   may run concurrently with each other.
//! - [`ExecutionMode::Sequential`] tools acquire a **write** lock, so no other
//!   tool (parallel or sequential) executes while one is in flight.
//!
//! Determinism: results are returned in the same order as the input batch,
//! regardless of completion order. The session transcript therefore stays
//! stable even when tools race.
//!
//! Starvation: the gate is a fair (FIFO) async RwLock, so a sequential tool
//! waiting on the write lock is not starved by a stream of parallel readers.

use common::tool::ExecutionMode;
use futures::future::join_all;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A permission-cleared tool call scheduled for execution.
#[derive(Debug, Clone)]
pub struct ScheduledCall {
    /// Position of the call in the originating `tool_calls` list; results are
    /// reported against this index.
    pub index: usize,
    /// Tool name (for mode lookup and diagnostics).
    pub tool_name: String,
    /// Raw JSON arguments string, exactly as provided by the model.
    pub args: String,
}

/// The result of executing one scheduled call.
#[derive(Debug, Clone, PartialEq)]
pub struct CallOutcome {
    /// Position of the call in the originating `tool_calls` list.
    pub index: usize,
    /// The tool result (or error/mapping text) exactly as it should be
    /// recorded in the transcript.
    pub result: String,
}

/// Shared concurrency gate. Parallel tools hold read guards; sequential tools
/// hold the single write guard.
#[derive(Debug, Clone, Default)]
pub struct ToolCallRuntime {
    gate: Arc<RwLock<()>>,
}

impl ToolCallRuntime {
    pub fn new() -> Self {
        Self::default()
    }

    /// Execute a batch of scheduled calls, returning outcomes **in input
    /// order**.
    ///
    /// `mode_of` classifies each call; `exec` runs one call (already
    /// permission-cleared) and returns its result string. Executors are
    /// awaited concurrently subject to the gate.
    pub async fn execute<F, Fut>(
        &self,
        calls: Vec<ScheduledCall>,
        mode_of: impl Fn(&ScheduledCall) -> ExecutionMode + Send + Sync,
        exec: F,
    ) -> Vec<CallOutcome>
    where
        F: Fn(ScheduledCall) -> Fut + Send + Sync,
        Fut: Future<Output = String> + Send,
    {
        let gate = self.gate.clone();
        let futs = calls.into_iter().map(|call| {
            let gate = gate.clone();
            let mode = mode_of(&call);
            let exec = &exec;
            async move {
                let result = match mode {
                    ExecutionMode::Parallel => {
                        let _guard = gate.read().await;
                        exec(call.clone()).await
                    }
                    ExecutionMode::Sequential => {
                        let _guard = gate.write().await;
                        exec(call.clone()).await
                    }
                };
                (call.index, result)
            }
        });

        // join_all yields futures' outputs in input order, which is the
        // original `tool_calls` order — deterministic transcripts.
        join_all(futs)
            .await
            .into_iter()
            .map(|(index, result)| CallOutcome { index, result })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    fn call(index: usize, tool: &str) -> ScheduledCall {
        ScheduledCall {
            index,
            tool_name: tool.to_string(),
            args: "{}".to_string(),
        }
    }

    /// Shared test harness: records peak concurrent executions while
    /// exercising the runtime with a configurable executor.
    struct ConcurrencyTracker {
        active: AtomicUsize,
        peak: AtomicUsize,
    }

    impl ConcurrencyTracker {
        fn new() -> Self {
            Self {
                active: AtomicUsize::new(0),
                peak: AtomicUsize::new(0),
            }
        }

        async fn run(&self, delay_ms: u64) -> String {
            let now = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            self.peak.fetch_max(now, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            self.active.fetch_sub(1, Ordering::SeqCst);
            format!("done after {delay_ms}ms")
        }

        fn peak(&self) -> usize {
            self.peak.load(Ordering::SeqCst)
        }
    }

    #[tokio::test]
    async fn parallel_tools_run_concurrently() {
        let tracker = std::sync::Arc::new(ConcurrencyTracker::new());
        let t = tracker.clone();
        let runtime = ToolCallRuntime::new();

        let outcomes = runtime
            .execute(
                vec![call(0, "glob"), call(1, "grep"), call(2, "glob")],
                |_| ExecutionMode::Parallel,
                move |c| {
                    let t = t.clone();
                    async move { t.run(50).await + &format!(" #{}", c.index) }
                },
            )
            .await;

        assert_eq!(outcomes.len(), 3);
        assert!(
            tracker.peak() >= 2,
            "parallel tools should overlap (peak={})",
            tracker.peak()
        );
    }

    #[tokio::test]
    async fn sequential_tools_execute_exclusively() {
        let tracker = std::sync::Arc::new(ConcurrencyTracker::new());
        let t = tracker.clone();
        let runtime = ToolCallRuntime::new();

        runtime
            .execute(
                vec![call(0, "write"), call(1, "write"), call(2, "write")],
                |_| ExecutionMode::Sequential,
                move |c| {
                    let t = t.clone();
                    async move { t.run(30).await + &format!(" #{}", c.index) }
                },
            )
            .await;

        assert_eq!(tracker.peak(), 1, "sequential tools must never overlap");
    }

    #[tokio::test]
    async fn sequential_blocks_parallel_tools() {
        let tracker = std::sync::Arc::new(ConcurrencyTracker::new());
        let t = tracker.clone();
        let runtime = ToolCallRuntime::new();

        let modes = [
            ExecutionMode::Parallel,
            ExecutionMode::Parallel,
            ExecutionMode::Sequential,
            ExecutionMode::Parallel,
        ];
        runtime
            .execute(
                vec![
                    call(0, "glob"),
                    call(1, "grep"),
                    call(2, "write"),
                    call(3, "glob"),
                ],
                |c| modes[c.index],
                move |c| {
                    let t = t.clone();
                    async move { t.run(30).await + &format!(" #{}", c.index) }
                },
            )
            .await;

        // The write-lock holder runs while nothing else does; parallel calls
        // may overlap only with each other, never with the sequential one.
        assert_eq!(
            tracker.peak(),
            2,
            "peak should be the parallel pair, never 3+"
        );
    }

    #[tokio::test]
    async fn results_preserve_input_order_regardless_of_completion() {
        let runtime = ToolCallRuntime::new();

        // Call 0 sleeps longest so it completes last; results must still be
        // returned in input order.
        let outcomes = runtime
            .execute(
                vec![call(0, "glob"), call(1, "grep"), call(2, "glob")],
                |_| ExecutionMode::Parallel,
                |c| async move {
                    let delay = match c.index {
                        0 => 80,
                        1 => 40,
                        _ => 10,
                    };
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                    format!("result-{}", c.index)
                },
            )
            .await;

        let results: Vec<(usize, String)> =
            outcomes.into_iter().map(|o| (o.index, o.result)).collect();
        assert_eq!(
            results,
            vec![
                (0, "result-0".to_string()),
                (1, "result-1".to_string()),
                (2, "result-2".to_string()),
            ]
        );
    }

    #[tokio::test]
    async fn empty_batch_is_a_noop() {
        let runtime = ToolCallRuntime::new();
        let outcomes = runtime
            .execute(
                vec![],
                |_| ExecutionMode::Sequential,
                |_| async { unreachable!() },
            )
            .await;
        assert!(outcomes.is_empty());
    }
}
