# Tooling System Improvements

**Status**: Proposal | **Created**: 2026-01-10 | **Priority**: High

This document outlines proposed improvements to the Sisyphus tooling system based on comprehensive analysis of the current implementation.

---

## Executive Summary

The current tooling system provides a solid foundation with the `Tool` trait, permission gating, and batch execution. However, there are significant opportunities for improvement in:

- **Safety**: Better error handling, timeouts, and validation
- **Performance**: Parallel execution, caching, streaming
- **UX**: Approval history, dry-run mode, structured errors
- **Observability**: Metrics, telemetry, dependency validation

These improvements maintain backward compatibility while significantly enhancing robustness and user experience.

---

## Current Architecture Overview

### Core Components

```
┌─────────────────────────────────────────────────────────┐
│                     Tool Trait                          │
│  - name() → &str                                        │
│  - description() → &str                                 │
│  - schema() → Value (JSON Schema)                       │
│  - execute(args: Value) → Result<String>                │
└─────────────────────────────────────────────────────────┘
                          ▲
                          │ implements
                          │
        ┌─────────────────┼─────────────────┐
        │                 │                 │
   CommandTool      FS Tools          Grep/Glob
   (Shell)         (Read/Write)      (Search)
        │                 │                 │
        └─────────────────┴─────────────────┘
                          │
        ┌─────────────────▼─────────────────┐
        │             Agent                 │
│  - tools: HashMap<String, Box<dyn Tool>> │
│  - register_tool()                        │
│  - execute_tool() + permission gating     │
│  - process_tool_batch() (sequential)      │
└────────────────────────────────────────────┘
```

### Permission Flow

```
Tool Request → Check Permission Level
    ├─ Deny → Return "Permission denied"
    ├─ Ask → Publish PermissionRequest, queue approval
    └─ Allow → Execute tool
```

### Key Files

| Component | File |
|-----------|------|
| Tool Trait | `crates/common/src/tool.rs` |
| Agent Logic | `crates/core/src/agent.rs` |
| Permission Config | `crates/core/src/agent/config.rs` |
| Session Management | `crates/core/src/session.rs` |
| Built-in Tools | `crates/tools/src/*.rs` |
| Event Bus | `crates/common/src/bus.rs` |

---

## Proposed Improvements

### 1. Tool Metadata & Categorization

**Current State**: Tools only provide name, description, and schema.

**Problem**:
- No way to categorize tools (filesystem, network, shell, database)
- No indication of tool danger level
- Cannot apply context-aware permission rules
- Difficult to discover/filter tools

**Solution**: Add `ToolMetadata` struct

```rust
pub struct ToolMetadata {
    pub category: ToolCategory,
    pub dangerous: bool,
    pub timeout: Duration,
    pub requires_write: bool,
    pub tags: Vec<String>,
}

pub enum ToolCategory {
    FileSystem,
    Shell,
    Network,
    Database,
    Custom,
    MCP,  // Model Context Protocol
}

pub trait Tool: Send + Sync {
    // Existing methods...
    fn metadata(&self) -> ToolMetadata {
        ToolMetadata::default()
    }
}
```

**Benefits**:
- Context-aware permission rules (e.g., `FileSystem` tools in read-only projects)
- Automatic dangerous tool grouping for heightened security
- Better tool discovery and filtering by category/tags
- Per-tool timeout configuration

**Implementation Effort**: Medium

---

### 2. Parallel Tool Execution

**Current State**: `process_tool_batch` executes tools sequentially.

```rust
// Current: Sequential execution
for call in calls {
    let result = self.execute_tool(&call.name, &call.args, &call.id).await?;
    session.add_message(result);
}
```

**Problem**:
- Independent tools (e.g., reading 3 different files) execute one at a time
- Poor resource utilization
- Slower workflows

**Solution**: Parallel execution for independent tools

```rust
async fn process_tool_batch_parallel(
    &self,
    session: &mut Session,
    calls: Vec<ToolCall>,
) -> Result<Option<String>> {
    // Group by permission level (Ask must be sequential)
    let (allow_calls, ask_calls): (Vec<_>, Vec<_>) = calls.into_iter()
        .partition(|call| {
            self.get_permission_level(&call.function.name) == PermissionLevel::Allow
        });

    // Execute Allow calls in parallel
    let allow_results = futures::future::join_all(
        allow_calls.iter().map(|call| {
            self.execute_tool(&call.function.name, &call.function.arguments, &call.id)
        })
    ).await;

    // Add results to session in original order
    for (call, result) in allow_calls.iter().zip(allow_results) {
        match result {
            ToolExecResult::Ok(output) => {
                session.add_message(create_tool_message(call.id.clone(), output));
            }
            ToolExecResult::PermissionRequired(msg) => {
                // Queue remaining and return
                return self.handle_permission_required(session, call, msg, ask_calls);
            }
        }
    }

    // Process Ask calls sequentially
    self.process_ask_calls(session, ask_calls).await
}
```

**Benefits**:
- 3-10x speedup for independent tool calls (e.g., reading multiple files)
- Better CPU and I/O resource utilization
- Maintains permission gating integrity (Ask tools still sequential)

**Implementation Effort**: Medium

---

### 3. Tool Result Caching

**Current State**: Every tool execution runs from scratch.

**Problem**:
- Repeated file reads for the same file in a workflow
- Multiple glob/grep scans for identical patterns
- Wasted compute and token costs

**Solution**: Cache layer for idempotent tools

```rust
pub trait Tool: Send + Sync {
    // Existing methods...

    fn is_cacheable(&self) -> bool {
        false  // Override for caching
    }

    fn cache_key(&self, args: &Value) -> Option<String> {
        if !self.is_cacheable() {
            return None;
        }
        Some(format!("{}:{}", self.name(), serde_json::to_string(args).ok()?))
    }
}

struct ToolCache {
    inner: Arc<Mutex<HashMap<String, CachedResult>>>,
    ttl: Duration,
}

struct CachedResult {
    value: String,
    timestamp: DateTime<Utc>,
}

impl ToolCache {
    pub fn get(&self, key: &str) -> Option<String> {
        let inner = self.inner.lock().ok()?;
        let result = inner.get(key)?;
        if result.timestamp.elapsed() > self.ttl {
            return None;  // Expired
        }
        Some(result.value.clone())
    }

    pub fn put(&self, key: String, value: String) {
        let mut inner = self.inner.lock().unwrap();
        inner.insert(key, CachedResult {
            value,
            timestamp: Utc::now(),
        });
    }
}

// Example: Cacheable ReadFileTool
impl Tool for ReadFileTool {
    fn is_cacheable(&self) -> bool {
        true
    }

    async fn execute(&self, args: Value) -> Result<String> {
        let key = self.cache_key(&args)?;

        // Check cache first
        if let Some(cached) = self.cache.get(&key) {
            return Ok(cached);
        }

        // Execute and cache
        let path = args["path"].as_str().ok_or_else(|| anyhow!("Missing path"))?;
        let content = tokio::fs::read_to_string(path).await?;
        self.cache.put(key, content.clone());

        Ok(content)
    }
}
```

**Benefits**:
- Avoid redundant file reads, glob scans
- Faster repeated operations in workflows
- Reduced token usage and API costs
- Configurable TTL for cache invalidation

**Implementation Effort**: Medium

---

### 4. Approval History & Learning

**Current State**: Every tool execution with `Ask` permission requires user approval.

**Problem**:
- Repetitive workflows require constant approval (e.g., reading 10 files)
- Poor UX for trusted operations
- No way to remember decisions

**Solution**: Remember user decisions with configurable TTL

```rust
pub struct ApprovalHistory {
    approvals: Arc<RwLock<HashMap<String, Vec<ApprovalEntry>>>>,
    // Key: tool_name, session_pattern, or context_hash
    config: ApprovalConfig,
}

pub struct ApprovalConfig {
    pub session_ttl: Duration,      // Remember for session duration
    pub workflow_ttl: Duration,      // Remember across similar workflows
    pub max_entries: usize,
}

pub struct ApprovalEntry {
    tool_name: String,
    args_hash: String,
    decision: bool,
    timestamp: DateTime<Utc>,
    session_id: String,
    context_hash: String,
}

impl ApprovalHistory {
    pub fn get_recent_decision(
        &self,
        tool_name: &str,
        args: &str,
        session_id: &str,
    ) -> Option<bool> {
        let approvals = self.approvals.read().ok()?;
        let entries = approvals.get(tool_name)?;

        let args_hash = self.hash_args(args);

        for entry in entries {
            // Check for exact match first
            if entry.args_hash == args_hash && entry.session_id == session_id {
                if entry.timestamp.elapsed() < self.config.session_ttl {
                    return Some(entry.decision);
                }
            }
        }

        None
    }

    pub fn record_decision(
        &self,
        tool_name: &str,
        args: &str,
        decision: bool,
        session_id: &str,
    ) {
        let mut approvals = self.approvals.write().unwrap();
        let entries = approvals.entry(tool_name.to_string()).or_insert_with(Vec::new);

        entries.push(ApprovalEntry {
            tool_name: tool_name.to_string(),
            args_hash: self.hash_args(args),
            decision,
            timestamp: Utc::now(),
            session_id: session_id.to_string(),
            context_hash: self.compute_context_hash(session_id),
        });

        // Prune old entries
        entries.retain(|e| e.timestamp.elapsed() < self.config.workflow_ttl);
        if entries.len() > self.config.max_entries {
            entries.remove(0);
        }
    }

    fn hash_args(&self, args: &str) -> String {
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(args.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    fn compute_context_hash(&self, session_id: &str) -> String {
        // Hash workspace, agent config, etc.
        format!("session:{}", session_id)
    }
}

// In Agent::execute_tool:
async fn execute_tool(&self, tool_name: &str, args_str: &str, call_id: &str) -> ToolExecResult {
    let permission = self.get_permission_level(tool_name);

    match permission {
        PermissionLevel::Ask => {
            // Check approval history
            if let Some(approved) = self.approval_history.get_recent_decision(
                tool_name, args_str, &session.id
            ) {
                if approved {
                    // Skip permission request
                    return self.execute_now(tool_name, args_str, call_id).await;
                }
            }

            // Publish permission request
            self.bus.publish(SystemEvent::PermissionRequest {
                operation: "tool_execution".to_string(),
                tool_name: tool_name.to_string(),
                call_id: call_id.to_string(),
            });
            return ToolExecResult::PermissionRequired("...".to_string());
        }
        _ => { /* ... */ }
    }
}
```

**Benefits**:
- Reduced approval friction for trusted operations
- Better UX for repetitive workflows
- Configurable approval memory (session vs. workflow)
- Automatic cleanup of old decisions

**Implementation Effort**: Medium

---

### 5. Streaming Tool Output

**Current State**: Tools return `Result<String>` (all-or-nothing, wait for complete result).

**Problem**:
- No real-time feedback for long-running operations
- Timeout issues with large outputs
- Poor UX for downloads, builds, scans

**Solution**: Streaming support for tool output

```rust
pub trait Tool: Send + Sync {
    // Existing sync/async execute
    async fn execute(&self, args: Value) -> Result<String>;

    // New streaming variant
    async fn execute_stream(
        &self,
        args: Value,
        tx: mpsc::Sender<String>,
    ) -> Result<()> {
        // Default implementation: buffer and send
        let result = self.execute(args).await?;
        let _ = tx.send(result).await;
        Ok(())
    }
}

// In Agent:
async fn execute_tool_streaming(
    &self,
    tool_name: &str,
    args: Value,
    call_id: &str,
    stream_tx: mpsc::Sender<String>,
) -> ToolExecResult {
    // Check permissions...
    let permission = self.get_permission_level(tool_name);
    // ... permission logic ...

    if let Some(tool) = self.tools.get(tool_name) {
        tool.execute_stream(args, stream_tx).await?;
        ToolExecResult::Ok("Streaming complete".to_string())
    } else {
        ToolExecResult::Ok("Tool not found".to_string())
    }
}

// Example: Streaming File Tool
async fn execute_stream(
    &self,
    args: Value,
    mut tx: mpsc::Sender<String>,
) -> Result<()> {
    let path = args["path"].as_str().ok_or_else(|| anyhow!("Missing path"))?;

    // Stream large files in chunks
    let file = tokio::fs::File::open(path).await?;
    let reader = tokio::io::BufReader::new(file);
    let mut lines = reader.lines();

    let mut count = 0;
    while let Some(line) = lines.next_line().await? {
        count += 1;
        let _ = tx.send(line).await;

        // Send progress every 100 lines
        if count % 100 == 0 {
            let progress = format!("\n[Progress: {} lines processed]\n", count);
            let _ = tx.send(progress).await;
        }
    }

    Ok(())
}
```

**Benefits**:
- Real-time progress feedback for LLMs (e.g., "reading files 45/100")
- Better UX for long downloads, builds, scans
- Prevents timeouts on large outputs
- Enables incremental processing

**Implementation Effort**: High (requires SSE streaming updates)

---

### 6. Tool Timeout & Cancellation

**Current State**: No timeout mechanism. Tools can hang indefinitely.

**Problem**:
- Runaway processes (infinite loops, network hangs)
- Poor user experience on stuck operations
- Resource exhaustion

**Solution**: Per-tool timeout with cancellation support

```rust
pub struct ToolExecutionConfig {
    pub default_timeout: Duration,
    pub per_tool_timeout: HashMap<String, Duration>,
    pub cancellation_token: CancellationToken,
}

impl Agent {
    async fn execute_with_timeout(
        &self,
        tool: &dyn Tool,
        tool_name: &str,
        args: Value,
    ) -> Result<String> {
        let timeout = self.config.execution_config
            .per_tool_timeout
            .get(tool_name)
            .copied()
            .unwrap_or(self.config.execution_config.default_timeout);

        tokio::select! {
            result = tool.execute(args) => {
                result
            }
            _ = tokio::time::sleep(timeout) => {
                Err(anyhow::anyhow!(
                    "Tool '{}' timed out after {:?}",
                    tool_name, timeout
                ))
            }
        }
    }
}

// Example: Long-running GrepTool with custom timeout
impl GrepTool {
    pub fn with_timeout(sandbox: Arc<SandboxedPath>, timeout: Duration) -> Self {
        Self {
            sandbox,
            timeout: Some(timeout),
        }
    }
}

// In config:
ToolExecutionConfig {
    default_timeout: Duration::from_secs(60),
    per_tool_timeout: {
        let mut map = HashMap::new();
        map.insert("execute_command".to_string(), Duration::from_secs(30));
        map.insert("grep".to_string(), Duration::from_secs(120));  // Longer for large repos
        map.insert("glob".to_string(), Duration::from_secs(30));
        map
    },
    cancellation_token: CancellationToken::new(),
}
```

**Benefits**:
- Prevent runaway processes
- Graceful degradation
- Better resource management
- Per-tool flexibility

**Implementation Effort**: Low

---

### 7. Tool Dependency Validation

**Current State**: Tools fail at runtime if dependencies are missing.

**Problem**:
- No early detection of missing binaries/libraries
- Poor error messages ("command not found")
- Fail-late debugging

**Solution**: Pre-flight dependency checks

```rust
pub trait Tool: Send + Sync {
    // Existing methods...

    fn validate_dependencies(&self) -> Result<()> {
        Ok(())  // Override to check binaries, libraries, etc.
    }
}

// Example: CommandTool checks for shell
impl Tool for CommandTool {
    fn validate_dependencies(&self) -> Result<()> {
        let shell = if cfg!(target_os = "windows") {
            "powershell.exe"
        } else {
            "sh"
        };

        let status = std::process::Command::new(shell)
            .arg("--version")
            .output()?;

        if !status.status.success() {
            return Err(anyhow!(
                "Shell command not available: {}. Please install it.",
                shell
            ));
        }

        Ok(())
    }
}

// Example: GrepTool checks for grep binary
impl Tool for GrepTool {
    fn validate_dependencies(&self) -> Result<()> {
        let status = std::process::Command::new("grep")
            .arg("--version")
            .output()?;

        if !status.status.success() {
            return Err(anyhow!(
                "grep command not available. Please install ripgrep (rg) or GNU grep."
            ));
        }

        Ok(())
    }
}

// In Agent:
pub fn validate_all_tools(&self) -> Vec<ToolValidationResult> {
    self.tools.values()
        .map(|t| ToolValidationResult {
            name: t.name().to_string(),
            status: t.validate_dependencies(),
        })
        .collect()
}

pub struct ToolValidationResult {
    pub name: String,
    pub status: Result<()>,
}

// At startup:
fn main() {
    // ... create agent ...

    // Validate tools
    let results = agent.validate_all_tools();
    let failed: Vec<_> = results.iter()
        .filter(|r| r.status.is_err())
        .collect();

    if !failed.is_empty() {
        eprintln!("Tool validation failed:");
        for result in &failed {
            eprintln!("  - {}: {}", result.name, result.status.as_ref().unwrap_err());
        }
        std::process::exit(1);
    }

    // ... continue ...
}
```

**Benefits**:
- Fail-fast on missing dependencies
- Clear error messages during setup
- Better debugging experience
- Production readiness

**Implementation Effort**: Low

---

### 8. Structured Error Responses

**Current State**: Generic error strings (e.g., "Tool execution error: ...").

**Problem**:
- No error type classification
- No suggestions for recovery
- Poor LLM error recovery
- Difficult debugging

**Solution**: Rich error types with context

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ToolError {
    pub tool_name: String,
    pub error_type: ToolErrorType,
    pub message: String,
    pub suggestion: Option<String>,
    pub context: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ToolErrorType {
    MissingParameter(String),
    InvalidParameter {
        param: String,
        value: serde_json::Value,
        reason: String,
    },
    PermissionDenied,
    Timeout,
    DependencyMissing(String),
    ExecutionFailed(String),
    NotFound(String),
    ValidationFailed(String),
}

// Example errors:
// Missing parameter:
ToolError {
    tool_name: "read_file".to_string(),
    error_type: ToolErrorType::MissingParameter("path".to_string()),
    message: "Required parameter 'path' is missing".to_string(),
    suggestion: Some("Provide a 'path' parameter with the file path to read".to_string()),
    context: json!({ "args": {"content": "hello world"} }),
}

// Permission denied:
ToolError {
    tool_name: "execute_command".to_string(),
    error_type: ToolErrorType::PermissionDenied,
    message: "Command execution requires approval".to_string(),
    suggestion: Some("Add execute_command to permissions.allow in config or approve the request".to_string()),
    context: json!({
        "command": "rm -rf /",
        "attempted_level": "Ask",
        "current_level": "Ask"
    }),
}

// Invalid parameter:
ToolError {
    tool_name: "grep".to_string(),
    error_type: ToolErrorType::InvalidParameter {
        param: "output_mode".to_string(),
        value: json!("invalid"),
        reason: "Must be one of: files_with_matches, content, count".to_string(),
    },
    message: "Invalid value for parameter 'output_mode'".to_string(),
    suggestion: Some("Use one of: files_with_matches, content, count".to_string()),
    context: json!({ "args": {"pattern": "test", "output_mode": "invalid"} }),
}

// In Agent execute_tool:
async fn execute_tool(&self, tool_name: &str, args_str: &str, call_id: &str) -> ToolExecResult {
    // ... permission checks ...

    if let Some(tool) = self.tools.get(tool_name) {
        match serde_json::from_str::<serde_json::Value>(args_str) {
            Ok(args) => {
                // Validate args against schema
                if let Err(e) = self.validate_args(tool, &args) {
                    return ToolExecResult::Ok(serde_json::to_string(&ToolError {
                        tool_name: tool_name.to_string(),
                        error_type: e.error_type,
                        message: e.message,
                        suggestion: e.suggestion,
                        context: json!({ "args": args }),
                    }).unwrap());
                }

                match tool.execute(args).await {
                    Ok(output) => ToolExecResult::Ok(output),
                    Err(e) => {
                        // Parse error to create structured error
                        let error = self.parse_tool_error(tool_name, e);
                        ToolExecResult::Ok(serde_json::to_string(&error).unwrap())
                    }
                }
            }
            Err(e) => {
                let error = ToolError {
                    tool_name: tool_name.to_string(),
                    error_type: ToolErrorType::InvalidParameter {
                        param: "args".to_string(),
                        value: json!(args_str),
                        reason: e.to_string(),
                    },
                    message: "Failed to parse tool arguments".to_string(),
                    suggestion: Some("Ensure arguments are valid JSON and match the tool's schema".to_string()),
                    context: json!({ "args_str": args_str, "parse_error": e.to_string() }),
                };
                ToolExecResult::Ok(serde_json::to_string(&error).unwrap())
            }
        }
    } else {
        ToolExecResult::Ok(serde_json::to_string(&ToolError {
            tool_name: tool_name.to_string(),
            error_type: ToolErrorType::NotFound(tool_name.to_string()),
            message: format!("Tool '{}' not found", tool_name),
            suggestion: Some("Check that the tool is registered with the agent".to_string()),
            context: json!({}),
        }).unwrap())
    }
}
```

**Benefits**:
- Actionable error messages with suggestions
- Better LLM error recovery (can parse error type)
- Improved debugging with structured context
- Consistent error format across all tools

**Implementation Effort**: Low

---

### 9. Tool Telemetry & Observability

**Current State**: No metrics collection for tool usage.

**Problem**:
- No visibility into tool usage patterns
- Cannot identify slow or problematic tools
- No data for permission tuning

**Solution**: Metrics for monitoring and analytics

```rust
pub struct ToolMetrics {
    pub execution_count: Arc<AtomicU64>,
    pub execution_time_ms: Arc<AtomicU64>,
    pub error_count: Arc<AtomicU64>,
    pub permission_denials: Arc<AtomicU64>,
    pub cache_hits: Arc<AtomicU64>,
    pub cache_misses: Arc<AtomicU64>,
}

impl ToolMetrics {
    pub fn record_execution(&self, duration: Duration, success: bool, cached: bool) {
        self.execution_count.fetch_add(1, Ordering::Relaxed);
        self.execution_time_ms.fetch_add(
            duration.as_millis() as u64,
            Ordering::Relaxed
        );
        if !success {
            self.error_count.fetch_add(1, Ordering::Relaxed);
        }
        if cached {
            self.cache_hits.fetch_add(1, Ordering::Relaxed);
        } else {
            self.cache_misses.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn record_permission_denial(&self) {
        self.permission_denials.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get_stats(&self) -> ToolStats {
        let total = self.execution_count.load(Ordering::Relaxed);
        let errors = self.error_count.load(Ordering::Relaxed);
        let time_ms = self.execution_time_ms.load(Ordering::Relaxed);

        ToolStats {
            total_executions: total,
            avg_time_ms: if total > 0 { time_ms / total } else { 0 },
            error_rate: if total > 0 {
                (errors as f64 / total as f64) * 100.0
            } else {
                0.0
            },
            permission_denials: self.permission_denials.load(Ordering::Relaxed),
            cache_hit_rate: {
                let hits = self.cache_hits.load(Ordering::Relaxed);
                let misses = self.cache_misses.load(Ordering::Relaxed);
                let total = hits + misses;
                if total > 0 {
                    (hits as f64 / total as f64) * 100.0
                } else {
                    0.0
                }
            },
        }
    }
}

pub struct ToolStats {
    pub total_executions: u64,
    pub avg_time_ms: u64,
    pub error_rate: f64,
    pub permission_denials: u64,
    pub cache_hit_rate: f64,
}

// Per-tool metrics:
pub struct ToolMetricsRegistry {
    metrics: Arc<RwLock<HashMap<String, ToolMetrics>>>,
}

impl ToolMetricsRegistry {
    pub fn record(&self, tool_name: &str, duration: Duration, success: bool, cached: bool) {
        let mut metrics = self.metrics.write().unwrap();
        let tool_metrics = metrics.entry(tool_name.to_string())
            .or_insert_with(ToolMetrics::default);
        tool_metrics.record_execution(duration, success, cached);
    }

    pub fn get_tool_stats(&self, tool_name: &str) -> Option<ToolStats> {
        let metrics = self.metrics.read().ok()?;
        metrics.get(tool_name).map(|m| m.get_stats())
    }

    pub fn get_all_stats(&self) -> HashMap<String, ToolStats> {
        let metrics = self.metrics.read().unwrap();
        metrics.iter()
            .map(|(name, m)| (name.clone(), m.get_stats()))
            .collect()
    }
}

// Example output:
// Tool: read_file
//   Total executions: 1,234
//   Avg time: 45ms
//   Error rate: 0.2%
//   Permission denials: 0
//   Cache hit rate: 67.3%
//
// Tool: execute_command
//   Total executions: 89
//   Avg time: 1,234ms
//   Error rate: 12.4%
//   Permission denials: 23
//   Cache hit rate: 0.0%
```

**Benefits**:
- Identify slow/dangerous tools
- Usage analytics for permission tuning
- Performance optimization data
- Cache effectiveness monitoring

**Implementation Effort**: Medium

---

### 10. Dry-Run Mode

**Current State**: No way to preview tool operations without execution.

**Problem**:
- Risk of destructive operations
- Poor trust in tool behavior
- No safe experimentation

**Solution**: Preview tool operations before execution

```rust
pub trait Tool: Send + Sync {
    // Existing methods...

    fn dry_run(&self, args: Value) -> Result<DryRunResult> {
        // Default implementation: describe what would happen
        Ok(DryRunResult {
            description: format!(
                "Execute {} with args: {}",
                self.name(),
                serde_json::to_string(&args)?
            ),
            affected_resources: vec![],
            reversible: false,
            estimated_duration: None,
        })
    }
}

pub struct DryRunResult {
    pub description: String,
    pub affected_resources: Vec<String>,  // Files, processes, etc.
    pub reversible: bool,
    pub estimated_duration: Option<Duration>,
}

// Example: WriteFileTool dry-run
impl Tool for WriteFileTool {
    fn dry_run(&self, args: Value) -> Result<DryRunResult> {
        let path = args["path"].as_str().ok_or_else(|| anyhow!("Missing path"))?;
        let content = args["content"].as_str().ok_or_else(|| anyhow!("Missing content"))?;

        let full_path = self.sandbox.join(path)?;

        Ok(DryRunResult {
            description: format!(
                "Write {} bytes to file '{}'",
                content.len(),
                full_path.display()
            ),
            affected_resources: vec![full_path.to_string_lossy().to_string()],
            reversible: true,  // Could restore backup
            estimated_duration: Some(Duration::from_millis(10)),
        })
    }
}

// Example: CommandTool dry-run
impl Tool for CommandTool {
    fn dry_run(&self, args: Value) -> Result<DryRunResult> {
        let cmd = args["command"].as_str().ok_or_else(|| anyhow!("Missing command"))?;

        // Analyze command for safety
        let dangerous = cmd.contains("rm -rf") || cmd.contains("del /");

        Ok(DryRunResult {
            description: format!(
                "Execute shell command: {}",
                cmd
            ),
            affected_resources: if dangerous {
                vec!["Filesystem (potentially destructive)".to_string()]
            } else {
                vec![]
            },
            reversible: false,
            estimated_duration: Some(Duration::from_millis(100)),
        })
    }
}

// In Agent:
async fn execute_tool(
    &self,
    tool_name: &str,
    args_str: &str,
    call_id: &str,
    dry_run: bool,
) -> ToolExecResult {
    if dry_run {
        if let Some(tool) = self.tools.get(tool_name) {
            match serde_json::from_str::<serde_json::Value>(args_str) {
                Ok(args) => {
                    match tool.dry_run(args) {
                        Ok(result) => ToolExecResult::Ok(format!(
                            "DRY RUN:\n{}\nAffected resources: {}\nReversible: {}",
                            result.description,
                            result.affected_resources.join(", "),
                            result.reversible
                        )),
                        Err(e) => ToolExecResult::Ok(format!("Dry run failed: {}", e)),
                    }
                }
                Err(e) => ToolExecResult::Ok(format!("Failed to parse args: {}", e)),
            }
        } else {
            ToolExecResult::Ok("Tool not found".to_string())
        }
    } else {
        // Normal execution...
    }
}
```

**Benefits**:
- Safe experimentation without side effects
- Better user trust and transparency
- Educational tool behavior
- Reduced risk of destructive operations

**Implementation Effort**: Medium

---

## Implementation Priority

| Priority | Improvement | Effort | Impact | Dependencies |
|----------|-------------|--------|--------|--------------|
| **P0** | #8 Structured Errors | Low | High | None |
| **P0** | #6 Tool Timeout | Low | High | None |
| **P1** | #1 Tool Metadata | Medium | High | None |
| **P1** | #2 Parallel Execution | Medium | High | None |
| **P2** | #4 Approval History | Medium | Medium | #1 (metadata) |
| **P2** | #5 Streaming Output | High | Medium | SSE infrastructure |
| **P2** | #3 Tool Caching | Medium | Medium | None |
| **P3** | #7 Dependency Validation | Low | Medium | None |
| **P3** | #9 Telemetry | Medium | Low | None |
| **P4** | #10 Dry-Run Mode | Medium | Low | None |

### Recommended Implementation Order

**Phase 1: Foundation (Week 1)**
1. #8 Structured Errors - Immediate safety and debugging benefits
2. #6 Tool Timeout - Prevent runaway processes
3. #1 Tool Metadata - Enables subsequent features

**Phase 2: Performance (Week 2-3)**
4. #2 Parallel Execution - Major speedup
5. #3 Tool Caching - Reduce redundant work

**Phase 3: UX Enhancements (Week 4-5)**
6. #4 Approval History - Reduce friction
7. #7 Dependency Validation - Fail-fast setup
8. #9 Telemetry - Observability

**Phase 4: Advanced Features (Week 6+)**
9. #5 Streaming Output - Real-time feedback
10. #10 Dry-Run Mode - Safe experimentation

---

## Backward Compatibility

All proposed improvements maintain backward compatibility:

1. **Tool Trait Extensions**: New methods have default implementations
2. **Optional Features**: Caching, streaming, dry-run are opt-in
3. **Configuration Defaults**: New config fields have sensible defaults
4. **API Compatibility**: Existing tool implementations continue to work

Migration path:
```rust
// Existing tools work without changes
struct MyTool;

impl Tool for MyTool {
    fn name(&self) -> &str { "my_tool" }
    fn description(&self) -> &str { "Does something" }
    fn schema(&self) -> Value { json!({}) }
    async fn execute(&self, args: Value) -> Result<String> {
        Ok("result".to_string())
    }
    // All new methods have default implementations
}
```

---

## Testing Strategy

### Unit Tests

Each improvement requires comprehensive unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool_timeout() {
        let tool = SlowTool::new();
        let agent = Agent::new(/* ... */);

        // Should timeout
        let result = agent.execute_with_timeout(&tool, args, Duration::from_millis(100)).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("timed out"));
    }

    #[tokio::test]
    async fn test_parallel_execution_speedup() {
        // Sequential: 3s total
        // Parallel: 1s total (all run concurrently)
        // Verify parallel is faster
    }

    #[tokio::test]
    async fn test_cache_hit() {
        // Execute tool twice, second should hit cache
        let agent = Agent::new(/* ... */);

        let result1 = agent.execute_tool("read_file", args1).await;
        let result2 = agent.execute_tool("read_file", args1).await;

        assert_eq!(result1, result2);
        assert!(agent.cache_hits() > 0);
    }

    #[tokio::test]
    async fn test_approval_history() {
        // First execution: requires approval
        // Second execution: auto-approved from history
        let agent = Agent::new(/* ... */);

        let result1 = agent.execute_tool("read_file", args).await;
        assert!(matches!(result1, ToolExecResult::PermissionRequired(_)));

        agent.record_approval("read_file", args, true);

        let result2 = agent.execute_tool("read_file", args).await;
        assert!(matches!(result2, ToolExecResult::Ok(_)));
    }

    #[tokio::test]
    async fn test_structured_errors() {
        let error = execute_tool_with_invalid_args().await;

        let parsed: ToolError = serde_json::from_str(&error).unwrap();
        assert!(matches!(parsed.error_type, ToolErrorType::InvalidParameter { .. }));
        assert!(parsed.suggestion.is_some());
    }
}
```

### Integration Tests

End-to-end workflows with permission gating, caching, and parallel execution:

```rust
#[tokio::test]
async fn test_multi_tool_workflow() {
    // 1. Read multiple files in parallel
    // 2. Process with grep
    // 3. Write result
    // Verify cache hits, parallel speedup
}

#[tokio::test]
async fn test_approval_workflow_with_history() {
    // 1. Execute tool requiring approval
    // 2. Approve
    // 3. Execute same tool again (should auto-approve)
}
```

### Performance Tests

Benchmark tool execution with and without improvements:

```rust
#[bench]
fn bench_parallel_vs_sequential(b: &mut Bencher) {
    // Compare sequential vs parallel execution
    // Expect 3-10x speedup
}

#[bench]
fn bench_cache_effectiveness(b: &mut Bencher) {
    // Measure cache hit rate and latency reduction
}
```

---

## Open Questions

1. **Streaming Infrastructure**: Does the current SSE infrastructure support streaming tool results, or does it need enhancement?

2. **Cache Invalidation**: Should cache invalidation be time-based (TTL), event-based (file modified), or both?

3. **Approval Memory**: What's the right default TTL for approval history? Session duration, 1 hour, 24 hours?

4. **Metrics Storage**: Should metrics be in-memory only, or persisted to disk/database for long-term analysis?

5. **Dry-Run Scope**: Should dry-run be available for all tools, or limited to "dangerous" tools (write, delete, shell)?

---

## Conclusion

The proposed improvements significantly enhance the tooling system's safety, performance, and user experience while maintaining backward compatibility. The phased implementation approach allows incremental delivery and testing.

**Key Benefits**:
- **Safety**: Structured errors, timeouts, dependency validation, dry-run mode
- **Performance**: Parallel execution (3-10x), caching (avoid redundant work)
- **UX**: Approval history, streaming output, better error messages
- **Observability**: Metrics, telemetry, tool metadata

**Estimated Effort**: 6-8 weeks for full implementation with testing.

**Next Steps**: Review proposal, approve priority order, begin Phase 1 implementation.
