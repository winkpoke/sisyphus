use anyhow::{anyhow, Result};
use async_trait::async_trait;
use common::llm::{
    CompletionRequest, LLMProvider, Message, ReasoningEffort, ReasoningMode, Role,
    RESERVED_REQUEST_KEYS,
};
use futures::{Stream, StreamExt};
use reqwest::{Client, Response};
use serde_json::{json, Value};
use std::pin::Pin;

#[cfg(feature = "dev_debug")]
use std::fs::OpenOptions;

#[cfg(feature = "dev_debug")]
use std::io::Write;

/// File the `dev_debug` feature appends request/response traces to.
#[cfg(feature = "dev_debug")]
const DEBUG_LOG_PATH: &str = "openai_debug.log";

/// Deep merge JSON values, excluding reserved keys from overrides
fn merge_with_reserved_protection(base: &mut Value, overrides: &Value, reserved_keys: &[&str]) {
    if let (Some(base_obj), Some(overrides_obj)) = (base.as_object_mut(), overrides.as_object()) {
        let reserved_set: std::collections::HashSet<&str> = reserved_keys.iter().cloned().collect();

        for (key, value) in overrides_obj {
            if reserved_set.contains(key.as_str()) {
                continue;
            }

            if let Some(base_value) = base_obj.get_mut(key) {
                if base_value.is_object() && value.is_object() {
                    merge_with_reserved_protection(base_value, value, reserved_keys);
                } else {
                    base_obj.insert(key.clone(), value.clone());
                }
            } else {
                base_obj.insert(key.clone(), value.clone());
            }
        }
    }
}

/// Render an `Authorization` header value safe for logging by redacting the
/// secret material. Returns a description of the redacted value rather than the
/// value itself, so logs can never leak credentials.
#[allow(dead_code)] // referenced only under the `dev_debug` feature
fn redact_auth(header_value: &str) -> String {
    if let Some(secret) = header_value.strip_prefix("Bearer ") {
        if secret.is_empty() {
            "Bearer <empty>".to_string()
        } else {
            format!("Bearer ***REDACTED*** ({} chars)", secret.chars().count())
        }
    } else {
        "***REDACTED***".to_string()
    }
}

pub struct OpenAIProvider {
    client: Client,
    api_key: String,
    base_url: String,
    model: String,
}

impl OpenAIProvider {
    pub fn new(api_key: String, base_url: Option<String>, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            base_url: base_url.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
            model,
        }
    }

    /// Build the OpenAI chat-completions request payload from a
    /// [`CompletionRequest`]. Shared by both the streaming and non-streaming
    /// paths.
    fn build_payload(&self, request: CompletionRequest, stream: bool) -> Value {
        let CompletionRequest {
            messages,
            temperature,
            max_tokens,
            tools,
            reasoning,
            request_overrides,
        } = request;

        let mut payload = json!({
            "model": self.model,
            "messages": messages,
            "temperature": temperature.unwrap_or(0.7),
            "stream": stream,
        });

        if let Some(obj) = payload.as_object_mut() {
            if let Some(max_tokens) = max_tokens {
                obj.insert("max_tokens".to_string(), json!(max_tokens));
            }
            if let Some(tools) = tools {
                obj.insert("tools".to_string(), json!(tools));
            }

            // Add reasoning settings if mode is not Off
            if reasoning.mode != ReasoningMode::Off {
                let effort = match reasoning.effort {
                    ReasoningEffort::Low => "low",
                    ReasoningEffort::Medium => "medium",
                    ReasoningEffort::High => "high",
                };
                obj.insert("reasoning_effort".to_string(), json!(effort));
                obj.insert("thinking".to_string(), json!({ "type": "enabled" }));
            }

            // Apply request_overrides with reserved-key protection
            if let Some(overrides) = request_overrides {
                merge_with_reserved_protection(&mut payload, &overrides, RESERVED_REQUEST_KEYS);
            }
        }

        payload
    }

    /// Issue a POST to the completions endpoint with the bearer token set.
    async fn send(&self, url: &str, payload: &Value) -> Result<Response> {
        self.client
            .post(url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(payload)
            .send()
            .await
            .map_err(Into::into)
    }

    /// Send the payload and, on a retryable client error (HTTP 400/422 or a
    /// body containing "Param Incorrect"), progressively strip the
    /// `reasoning_effort` and `thinking` fields and resend. This keeps the
    /// provider working against endpoints that reject one or both of those
    /// non-standard fields.
    ///
    /// `label` ("complete" / "stream") is used only for `dev_debug` traces.
    ///
    /// Returns a successful [`Response`], or the first non-retryable / final
    /// error.
    async fn post_with_reasoning_fallback(
        &self,
        url: &str,
        mut payload: Value,
        label: &str,
    ) -> Result<Response> {
        let mut res = self.send(url, &payload).await?;

        if res.status().is_client_error() {
            let status = res.status();
            // Consume the body to read the error text; `res` is now exhausted
            // and must not be used again unless reassigned by a resend.
            let error_text = res.text().await?;
            self.debug_log_error(label, status, &error_text);

            let retryable = matches!(status.as_u16(), 400 | 422)
                || error_text.contains("Param Incorrect");
            if !retryable {
                return Err(anyhow!("OpenAI API error: {}", error_text));
            }

            // Two-stage fallback, preserving the original preference of keeping
            // `thinking` when dropping `reasoning_effort` alone is enough.
            // `next_res` stays `None` until a resend actually happens, which
            // lets us tell whether `res` is still a valid response.
            let mut next_res: Option<Response> = None;

            // Attempt 1: remove reasoning_effort first
            if let Some(obj) = payload.as_object_mut() {
                if obj.remove("reasoning_effort").is_some() {
                    self.debug_log_retry(label, "reasoning_effort", &error_text);
                    next_res = Some(self.send(url, &payload).await?);
                }
            }

            // Attempt 2: only needed if we're still erroring (or attempt 1
            // never ran because reasoning_effort was absent).
            let need_attempt_2 = match &next_res {
                Some(r) => r.status().is_client_error(),
                None => true,
            };
            if need_attempt_2 {
                if let Some(obj) = payload.as_object_mut() {
                    if obj.remove("thinking").is_some() {
                        self.debug_log_retry(label, "thinking", &error_text);
                        next_res = Some(self.send(url, &payload).await?);
                    }
                }
            }

            res = match next_res {
                Some(r) => r,
                // Retryable but nothing to strip -- surface the original error.
                None => return Err(anyhow!("OpenAI API error: {}", error_text)),
            };

            if res.status().is_client_error() {
                let final_status = res.status();
                let final_error = res.text().await?;
                self.debug_log_error(label, final_status, &final_error);
                return Err(anyhow!("OpenAI API error: {}", final_error));
            }
        }

        self.debug_log_request_response(label, url, &payload, &res);

        if !res.status().is_success() {
            let error_status = res.status();
            let error = res.text().await?;
            self.debug_log_error(label, error_status, &error);
            return Err(anyhow!("OpenAI API error: {}", error));
        }

        Ok(res)
    }

    #[cfg(feature = "dev_debug")]
    fn debug_log_error(&self, label: &str, status: reqwest::StatusCode, error: &str) {
        eprintln!(
            "OpenAI Provider [{label}]: request failed ({status}): {error}"
        );
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(DEBUG_LOG_PATH) {
            let _ = writeln!(file, "[{label}] Request failed ({status}): {error}");
        }
    }

    #[cfg(not(feature = "dev_debug"))]
    fn debug_log_error(&self, _label: &str, _status: reqwest::StatusCode, _error: &str) {}

    #[cfg(feature = "dev_debug")]
    fn debug_log_retry(&self, label: &str, removed: &str, because: &str) {
        eprintln!("OpenAI Provider [{label}]: retrying without {removed}...");
        if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(DEBUG_LOG_PATH) {
            let _ = writeln!(
                file,
                "[{label}] Retrying without {removed} due to error: {because}"
            );
        }
    }

    #[cfg(not(feature = "dev_debug"))]
    fn debug_log_retry(&self, _label: &str, _removed: &str, _because: &str) {}

    /// Trace request metadata and response status/headers. The bearer token is
    /// always redacted so credentials can never reach the log file.
    #[cfg(feature = "dev_debug")]
    fn debug_log_request_response(&self, label: &str, url: &str, payload: &Value, res: &Response) {
        let Ok(mut file) = OpenOptions::new().create(true).append(true).open(DEBUG_LOG_PATH)
        else {
            return;
        };
        let _ = writeln!(file, "\n=== OpenAI [{label}] Request Debug ===");
        let _ = writeln!(file, "Timestamp: {:?}", std::time::SystemTime::now());
        let _ = writeln!(file, "URL: {url}");
        let _ = writeln!(file, "Method: POST");
        let _ = writeln!(file, "Headers:");
        let _ = writeln!(
            file,
            "  Authorization: {}",
            redact_auth(&format!("Bearer {}", self.api_key))
        );
        let _ = writeln!(file, "  Content-Type: application/json");
        let _ = writeln!(
            file,
            "Payload: {}",
            serde_json::to_string_pretty(payload)
                .unwrap_or_else(|_| "Failed to serialize".to_string())
        );
        let _ = writeln!(file, "Response Status: {}", res.status());
        let _ = writeln!(file, "Response Headers:");
        for (key, value) in res.headers().iter() {
            let _ = writeln!(file, "  {key}: {value:?}");
        }
        let _ = file.flush();
    }

    #[cfg(not(feature = "dev_debug"))]
    fn debug_log_request_response(&self, _label: &str, _url: &str, _payload: &Value, _res: &Response) {
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<Message> {
        let url = format!("{}/chat/completions", self.base_url);
        let payload = self.build_payload(request, false);
        let res = self.post_with_reasoning_fallback(&url, payload, "complete").await?;

        let json: Value = res.json().await?;

        #[cfg(feature = "dev_debug")]
        {
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(DEBUG_LOG_PATH)
            {
                let _ = writeln!(
                    file,
                    "Response Body: {}",
                    serde_json::to_string_pretty(&json)
                        .unwrap_or_else(|_| "Failed to serialize".to_string())
                );
                let _ = writeln!(file, "=== End Debug ===\n");
                let _ = file.flush();
            }
        }

        let choice = &json["choices"][0]["message"];

        // `content` is null on assistant messages that only carry tool calls;
        // preserve that as `None` rather than coercing to an empty string.
        let content = choice["content"].as_str().map(|s| s.to_string());

        let tool_calls = if let Some(calls) = choice["tool_calls"].as_array() {
            Some(serde_json::from_value(json!(calls))?)
        } else {
            None
        };

        // Extract reasoning_summary from OpenAI-compatible responses
        let reasoning_summary = choice["reasoning_summary"].as_str().map(|s| s.to_string());

        // Extract reasoning_content (new standard) or reasoning_raw (older/custom)
        let reasoning_raw = choice["reasoning_content"]
            .as_str()
            .or_else(|| choice["reasoning"].as_str())
            .map(|s| s.to_string());

        Ok(Message {
            role: Role::Assistant,
            content,
            tool_calls,
            tool_call_id: None,
            reasoning_summary,
            reasoning_raw,
        })
    }

    /// Stream assistant text deltas from the chat-completions endpoint.
    ///
    /// Note: this yields assistant *content* deltas only. Tool-call deltas and
    /// reasoning deltas are intentionally not surfaced because the provider
    /// stream trait returns `Stream<Item = Result<String>>`. Surfacing tool
    /// calls through streaming requires a richer stream item type; until then,
    /// callers that need tool calls should use [`LLMProvider::complete`].
    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let url = format!("{}/chat/completions", self.base_url);
        let payload = self.build_payload(request, true);
        let res = self.post_with_reasoning_fallback(&url, payload, "stream").await?;

        let parser = crate::sse::SSEParser::new(res.bytes_stream());

        let stream = futures::stream::unfold((parser, false), |(mut parser, finished)| async move {
                if finished {
                    return None;
                }

                match parser.next().await {
                    Some(Ok(event)) => {
                        if event.data == "[DONE]" {
                            return Some((None, (parser, true)));
                        }

                        if let Ok(json) = serde_json::from_str::<Value>(&event.data) {
                            if let Some(content) = json["choices"][0]["delta"]["content"].as_str() {
                                return Some((Some(Ok(content.to_string())), (parser, false)));
                            }
                        }
                        Some((None, (parser, false)))
                    }
                    Some(Err(e)) => Some((Some(Err(e)), (parser, true))),
                    None => None,
                }
            })
            .filter_map(|opt| async { opt });

        Ok(Box::pin(stream))
    }

    fn model(&self) -> String {
        self.model.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use common::llm::{CompletionRequest, ReasoningConfig, ToolDefinition, ToolFunctionDefinition};
    use wiremock::matchers::{header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    fn redact_auth_never_leaks_the_secret() {
        // The full bearer token must never appear in log output.
        let redacted = super::redact_auth("Bearer sk-live-key-1234567890");
        assert!(redacted.contains("***REDACTED***"));
        assert!(!redacted.contains("sk-live-key"));
    }

    #[test]
    fn redact_auth_handles_non_bearer_and_empty() {
        assert_eq!(super::redact_auth("Basic xyz"), "***REDACTED***");
        assert_eq!(super::redact_auth("Bearer "), "Bearer <empty>");
    }

    #[tokio::test]
    async fn test_openai_model_method() {
        let provider =
            OpenAIProvider::new("test-key".to_string(), None, "gpt-3.5-turbo".to_string());

        assert_eq!(provider.model(), "gpt-3.5-turbo");
    }

    #[tokio::test]
    async fn test_openai_default_base_url() {
        let provider = OpenAIProvider::new("test-key".to_string(), None, "gpt-4".to_string());

        assert_eq!(provider.base_url, "https://api.openai.com/v1");
    }

    #[tokio::test]
    async fn test_openai_custom_base_url() {
        let custom_url = "https://custom.api.com/v1".to_string();
        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            Some(custom_url.clone()),
            "gpt-4".to_string(),
        );

        assert_eq!(provider.base_url, custom_url);
    }

    #[tokio::test]
    async fn test_completion_request_serialization() {
        let mock_server = MockServer::start().await;

        let expected_response = json!({
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "created": 1677652288,
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Hello, world!"
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 9,
                "completion_tokens": 12,
                "total_tokens": 21
            }
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .and(header("Authorization", "Bearer test-key"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&expected_response))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            Some(mock_server.uri()),
            "gpt-3.5-turbo".to_string(),
        );

        let request = CompletionRequest {
            messages: vec![Message {
                role: Role::User,
                content: Some("Say hello".to_string()),
                tool_calls: None,
                tool_call_id: None,
                reasoning_summary: None,
                reasoning_raw: None,
            }],
            temperature: Some(0.7),
            max_tokens: None,
            tools: None,
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.complete(request).await;
        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(message.content, Some("Hello, world!".to_string()));
        assert_eq!(message.role, Role::Assistant);
    }

    #[tokio::test]
    async fn test_text_response_parsing() {
        let mock_server = MockServer::start().await;

        let response = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "This is a text response"
                }
            }]
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            Some(mock_server.uri()),
            "gpt-3.5-turbo".to_string(),
        );

        let request = CompletionRequest {
            messages: vec![],
            temperature: None,
            max_tokens: None,
            tools: None,
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.complete(request).await;
        assert!(result.is_ok());
        let message = result.unwrap();
        assert_eq!(message.content, Some("This is a text response".to_string()));
    }

    #[tokio::test]
    async fn test_tool_call_response_parsing() {
        let mock_server = MockServer::start().await;

        let response = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [
                        {
                            "id": "call_abc123",
                            "type": "function",
                            "function": {
                                "name": "search",
                                "arguments": "{\"query\": \"test\"}"
                            }
                        }
                    ]
                }
            }]
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            Some(mock_server.uri()),
            "gpt-3.5-turbo".to_string(),
        );

        let request = CompletionRequest {
            messages: vec![],
            temperature: None,
            max_tokens: None,
            tools: None,
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.complete(request).await;
        assert!(result.is_ok());
        let message = result.unwrap();
        assert!(message.content.is_none());
        assert!(message.tool_calls.is_some());

        let tool_calls = message.tool_calls.unwrap();
        assert_eq!(tool_calls.len(), 1);
        assert_eq!(tool_calls[0].id, "call_abc123");
        assert_eq!(tool_calls[0].function.name, "search");
        assert_eq!(tool_calls[0].function.arguments, "{\"query\": \"test\"}");
    }

    #[tokio::test]
    async fn test_request_with_tools() {
        let mock_server = MockServer::start().await;

        let response = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": null
                }
            }]
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            Some(mock_server.uri()),
            "gpt-3.5-turbo".to_string(),
        );

        let tools = vec![ToolDefinition {
            kind: "function".to_string(),
            function: ToolFunctionDefinition {
                name: "search".to_string(),
                description: "Search the web".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"}
                    },
                    "required": ["query"]
                }),
            },
        }];

        let request = CompletionRequest {
            messages: vec![],
            temperature: None,
            max_tokens: None,
            tools: Some(tools),
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.complete(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_request_with_max_tokens() {
        let mock_server = MockServer::start().await;

        let response = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "Response"
                }
            }]
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            Some(mock_server.uri()),
            "gpt-3.5-turbo".to_string(),
        );

        let request = CompletionRequest {
            messages: vec![],
            temperature: None,
            max_tokens: None,
            tools: None,
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.complete(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_network_error_handling() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(502).set_body_string("Bad Gateway"))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            Some(mock_server.uri()),
            "gpt-3.5-turbo".to_string(),
        );

        let request = CompletionRequest {
            messages: vec![],
            temperature: None,
            max_tokens: None,
            tools: None,
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.complete(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("OpenAI API error"));
    }

    #[tokio::test]
    async fn test_rate_limit_error_handling() {
        let mock_server = MockServer::start().await;

        let error_response = json!({
            "error": {
                "message": "Rate limit exceeded",
                "type": "rate_limit_error",
                "code": "rate_limit_exceeded"
            }
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(429).set_body_json(&error_response))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            Some(mock_server.uri()),
            "gpt-3.5-turbo".to_string(),
        );

        let request = CompletionRequest {
            messages: vec![],
            temperature: None,
            max_tokens: None,
            tools: None,
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.complete(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_server_error_handling() {
        let mock_server = MockServer::start().await;

        let error_response = json!({
            "error": {
                "message": "Internal server error",
                "type": "server_error",
                "code": "internal_error"
            }
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(500).set_body_json(&error_response))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            Some(mock_server.uri()),
            "gpt-3.5-turbo".to_string(),
        );

        let request = CompletionRequest {
            messages: vec![],
            temperature: None,
            max_tokens: None,
            tools: None,
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.complete(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_authentication_error_handling() {
        let mock_server = MockServer::start().await;

        let error_response = json!({
            "error": {
                "message": "Invalid API key",
                "type": "invalid_request_error",
                "code": "invalid_api_key"
            }
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(401).set_body_json(&error_response))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            "invalid-key".to_string(),
            Some(mock_server.uri()),
            "gpt-3.5-turbo".to_string(),
        );

        let request = CompletionRequest {
            messages: vec![],
            temperature: None,
            max_tokens: None,
            tools: None,
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.complete(request).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("OpenAI API error"));
    }

    #[tokio::test]
    async fn test_invalid_json_response_handling() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            Some(mock_server.uri()),
            "gpt-3.5-turbo".to_string(),
        );

        let request = CompletionRequest {
            messages: vec![],
            temperature: None,
            max_tokens: None,
            tools: None,
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.complete(request).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stream_request_formatting() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "data: {\"choices\":[{\"delta\":{\"content\":\"Hello\"}]}\n\n\
                 data: {\"choices\":[{\"delta\":{\"content\":\" world\"}]}\n\n\
                 data: [DONE]\n\n",
            ))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            Some(mock_server.uri()),
            "gpt-3.5-turbo".to_string(),
        );

        let request = CompletionRequest {
            messages: vec![],
            temperature: Some(0.5),
            max_tokens: None,
            tools: None,
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.stream(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_stream_with_tools() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "data: {\"choices\":[{\"delta\":{\"content\":\"test\"}]}\n\n\
                 data: [DONE]\n\n",
            ))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            Some(mock_server.uri()),
            "gpt-3.5-turbo".to_string(),
        );

        let tools = vec![ToolDefinition {
            kind: "function".to_string(),
            function: ToolFunctionDefinition {
                name: "tool_name".to_string(),
                description: "Test tool".to_string(),
                parameters: json!({}),
            },
        }];

        let request = CompletionRequest {
            messages: vec![],
            temperature: None,
            max_tokens: None,
            tools: Some(tools),
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.stream(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_empty_tool_calls() {
        let mock_server = MockServer::start().await;

        let response = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": "No tools needed"
                }
            }]
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            Some(mock_server.uri()),
            "gpt-3.5-turbo".to_string(),
        );

        let request = CompletionRequest {
            messages: vec![],
            temperature: None,
            max_tokens: None,
            tools: None,
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.complete(request).await;
        assert!(result.is_ok());
        let message = result.unwrap();
        assert!(message.tool_calls.is_none());
        assert_eq!(message.content, Some("No tools needed".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_tool_calls() {
        let mock_server = MockServer::start().await;

        let response = json!({
            "choices": [{
                "message": {
                    "role": "assistant",
                    "content": null,
                    "tool_calls": [
                        {
                            "id": "call_1",
                            "type": "function",
                            "function": {
                                "name": "tool1",
                                "arguments": "{\"arg1\":\"value1\"}"
                            }
                        },
                        {
                            "id": "call_2",
                            "type": "function",
                            "function": {
                                "name": "tool2",
                                "arguments": "{\"arg2\":\"value2\"}"
                            }
                        }
                    ]
                }
            }]
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            Some(mock_server.uri()),
            "gpt-3.5-turbo".to_string(),
        );

        let request = CompletionRequest {
            messages: vec![],
            temperature: None,
            max_tokens: None,
            tools: None,
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.complete(request).await;
        assert!(result.is_ok());
        let message = result.unwrap();
        assert!(message.tool_calls.is_some());

        let tool_calls = message.tool_calls.unwrap();
        assert_eq!(tool_calls.len(), 2);
        assert_eq!(tool_calls[0].id, "call_1");
        assert_eq!(tool_calls[0].function.name, "tool1");
        assert_eq!(tool_calls[1].id, "call_2");
        assert_eq!(tool_calls[1].function.name, "tool2");
    }

    #[tokio::test]
    async fn test_default_temperature_value() {
        let mock_server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_string(
                "{\"choices\":[{\"message\":{\"role\":\"assistant\",\"content\":\"test\"}}]}",
            ))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            Some(mock_server.uri()),
            "gpt-3.5-turbo".to_string(),
        );

        let request = CompletionRequest {
            messages: vec![],
            temperature: None,
            max_tokens: None,
            tools: None,
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.complete(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_malformed_response_missing_choices() {
        let mock_server = MockServer::start().await;

        let response = json!({
            "id": "chatcmpl-123",
            "object": "chat.completion",
            "choices": []
        });

        Mock::given(method("POST"))
            .and(path("/chat/completions"))
            .respond_with(ResponseTemplate::new(200).set_body_json(&response))
            .mount(&mock_server)
            .await;

        let provider = OpenAIProvider::new(
            "test-key".to_string(),
            Some(mock_server.uri()),
            "gpt-3.5-turbo".to_string(),
        );

        let request = CompletionRequest {
            messages: vec![],
            temperature: None,
            max_tokens: None,
            tools: None,
            reasoning: Default::default(),
            request_overrides: None,
        };

        let result = provider.complete(request).await;

        let result_value = result.is_ok() && result.unwrap().content.is_none();
        assert!(
            result_value,
            "Expected empty result or error for missing choices"
        );
    }

    #[tokio::test]
    async fn test_tool_schema_snapshot() {
        let tool_definition = ToolDefinition {
            kind: "function".to_string(),
            function: ToolFunctionDefinition {
                name: "search".to_string(),
                description: "Search web".to_string(),
                parameters: json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "Search query"}
                    },
                    "required": ["query"]
                }),
            },
        };

        let json_str = serde_json::to_string(&tool_definition).unwrap();
        insta::assert_snapshot!(json_str);
    }

    // ---- Task 7.1: override merge order, reserved-key rejection ----

    fn merge_test_provider() -> OpenAIProvider {
        OpenAIProvider::new("test-key".to_string(), None, "gpt-4".to_string())
    }

    #[test]
    fn merge_ignores_reserved_keys() {
        let mut base = json!({
            "model": "gpt-4",
            "messages": [{"role": "user", "content": "hi"}],
            "stream": false
        });
        let overrides = json!({
            "model": "evil-model",
            "messages": [{"role": "user", "content": "injected"}],
            "stream": true,
            "temperature": 0.2
        });

        merge_with_reserved_protection(&mut base, &overrides, RESERVED_REQUEST_KEYS);

        // Reserved keys keep the base values.
        assert_eq!(base["model"], json!("gpt-4"));
        assert_eq!(base["messages"][0]["content"], json!("hi"));
        assert_eq!(base["stream"], json!(false));
        // Non-reserved keys still apply.
        assert_eq!(base["temperature"], json!(0.2));
    }

    #[test]
    fn merge_deep_merges_nested_objects_and_adds_new_keys() {
        let mut base = json!({
            "nested": {"a": 1, "b": 2},
            "keep": "base"
        });
        let overrides = json!({
            "nested": {"b": 99, "c": 3},
            "extra": true
        });

        merge_with_reserved_protection(&mut base, &overrides, RESERVED_REQUEST_KEYS);

        assert_eq!(
            base["nested"],
            json!({"a": 1, "b": 99, "c": 3}),
            "objects deep-merge; non-conflicting keys survive; overrides win"
        );
        assert_eq!(base["extra"], json!(true), "new keys are added");
    }

    #[test]
    fn merge_ignores_reserved_keys_in_nested_objects() {
        let mut base = json!({
            "top": {"messages": "base", "other": 1}
        });
        let overrides = json!({
            "top": {"messages": "hijacked", "other": 2}
        });

        merge_with_reserved_protection(&mut base, &overrides, RESERVED_REQUEST_KEYS);

        assert_eq!(base["top"]["messages"], json!("base"));
        assert_eq!(base["top"]["other"], json!(2));
    }

    fn plain_request(overrides: Option<Value>, reasoning: ReasoningConfig) -> CompletionRequest {
        CompletionRequest {
            messages: vec![Message {
                role: Role::User,
                content: Some("hi".to_string()),
                tool_calls: None,
                tool_call_id: None,
                reasoning_summary: None,
                reasoning_raw: None,
            }],
            temperature: Some(0.7),
            max_tokens: None,
            tools: None,
            reasoning,
            request_overrides: overrides,
        }
    }

    #[test]
    fn build_payload_applies_overrides_after_standard_fields() {
        let provider = merge_test_provider();
        let request = plain_request(
            Some(json!({"temperature": 0.1, "custom_flag": true})),
            Default::default(),
        );

        let payload = provider.build_payload(request, false);

        // Overrides take precedence on conflicts and are merged after the
        // standard Sisyphus fields.
        assert_eq!(payload["temperature"], json!(0.1));
        assert_eq!(payload["custom_flag"], json!(true));
        // Standard fields remain intact.
        assert_eq!(payload["model"], json!("gpt-4"));
        assert_eq!(payload["stream"], json!(false));
    }

    #[test]
    fn build_payload_preserves_reserved_keys_from_overrides() {
        let provider = merge_test_provider();
        let overrides = json!({
            "model": "hijacked",
            "messages": [],
            "tools": [],
            "stream": true,
            "tool_choice": "auto",
            "tool_calls": []
        });
        let request = plain_request(Some(overrides), Default::default());

        let payload = provider.build_payload(request, false);

        assert_eq!(payload["model"], json!("gpt-4"), "model is reserved");
        assert_eq!(
            payload["messages"].as_array().map(|a| a.len()),
            Some(1),
            "messages is reserved"
        );
        assert_eq!(payload["stream"], json!(false), "stream is reserved");
        assert!(payload.get("tools").is_none(), "tools is reserved");
        assert!(payload.get("tool_choice").is_none(), "tool_choice is reserved");
        assert!(payload.get("tool_calls").is_none(), "tool_calls is reserved");
    }

    #[test]
    fn build_payload_omits_reasoning_fields_when_mode_off() {
        let provider = merge_test_provider();
        let reasoning = ReasoningConfig {
            mode: ReasoningMode::Off,
            ..ReasoningConfig::default()
        };
        let request = plain_request(None, reasoning);

        let payload = provider.build_payload(request, false);

        assert!(payload.get("reasoning_effort").is_none());
        assert!(payload.get("thinking").is_none());
    }

    #[test]
    fn build_payload_includes_reasoning_fields_when_enabled() {
        let provider = merge_test_provider();
        let reasoning = ReasoningConfig {
            mode: ReasoningMode::On,
            effort: ReasoningEffort::High,
            ..ReasoningConfig::default()
        };
        let request = plain_request(None, reasoning);

        let payload = provider.build_payload(request, false);

        assert_eq!(payload["reasoning_effort"], json!("high"));
        assert_eq!(payload["thinking"], json!({"type": "enabled"}));
    }
}
