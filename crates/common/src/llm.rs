use anyhow::Result;
use async_trait::async_trait;
use futures::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;

/// Reserved keys that cannot be overridden by request_overrides
/// These are core fields that control Sisyphus behavior
pub const RESERVED_REQUEST_KEYS: &[&str] = &[
    "model",
    "messages",
    "tools",
    "tool_calls",
    "tool_choice",
    "stream",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
    Tool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub function: FunctionCall,
    #[serde(rename = "type")]
    pub kind: String, // usually "function"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub role: Role,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    #[serde(skip_serializing)]
    pub reasoning_summary: Option<String>,
    #[serde(skip_serializing)]
    pub reasoning_raw: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    #[serde(rename = "type")]
    pub kind: String, // "function"
    pub function: ToolFunctionDefinition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunctionDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReasoningMode {
    Off,
    On,
    Auto,
}

impl Default for ReasoningMode {
    fn default() -> Self {
        Self::Auto
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReasoningEffort {
    Low,
    Medium,
    High,
}

impl Default for ReasoningEffort {
    fn default() -> Self {
        Self::Medium
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReasoningExposure {
    None,
    Summary,
    Debug,
}

impl Default for ReasoningExposure {
    fn default() -> Self {
        Self::Summary
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReasoningStorage {
    None,
    Summary,
}

impl Default for ReasoningStorage {
    fn default() -> Self {
        Self::None
    }
}

#[derive(Debug, Clone, Default)]
pub struct ReasoningConfig {
    pub mode: ReasoningMode,
    pub effort: ReasoningEffort,
    pub expose: ReasoningExposure,
    pub store: ReasoningStorage,
}

impl ReasoningConfig {
    pub fn with_defaults() -> Self {
        Self {
            mode: ReasoningMode::Auto,
            effort: ReasoningEffort::Medium,
            expose: ReasoningExposure::Summary,
            store: ReasoningStorage::None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompletionRequest {
    pub messages: Vec<Message>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u32>,
    pub tools: Option<Vec<ToolDefinition>>,
    pub reasoning: ReasoningConfig,
    pub request_overrides: Option<serde_json::Value>,
}

#[async_trait]
pub trait LLMProvider: Send + Sync {
    async fn complete(&self, request: CompletionRequest) -> Result<Message>;

    // For object safety with streaming, we return a pinned box stream.
    // The stream yields chunks of content (Strings).
    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>>;

    fn model(&self) -> String;
}
