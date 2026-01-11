use anyhow::{anyhow, Result};
use async_trait::async_trait;
use common::llm::{CompletionRequest, LLMProvider, Message, Role};
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde_json::{json, Value};
use std::pin::Pin;

#[cfg(feature = "dev_debug")]
use std::fs::OpenOptions;

#[cfg(feature = "dev_debug")]
use std::io::Write;

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
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<Message> {
        let url = format!("{}/chat/completions", self.base_url);

        let mut payload = json!({
            "model": self.model,
            "messages": request.messages,
            "temperature": request.temperature.unwrap_or(0.7),
            "stream": false
        });

        if let Some(obj) = payload.as_object_mut() {
            if let Some(max_tokens) = request.max_tokens {
                obj.insert("max_tokens".to_string(), json!(max_tokens));
            }
            if let Some(tools) = request.tools {
                obj.insert("tools".to_string(), json!(tools));
            }
        }

        let res = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .send()
            .await?;

        #[cfg(feature = "dev_debug")]
        let mut debug_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("openai_debug.log");

        #[cfg(feature = "dev_debug")]
        if let Ok(ref mut file) = debug_file {
            let _ = writeln!(file, "\n=== OpenAI Request Debug ===");
            let _ = writeln!(file, "Timestamp: {:?}", std::time::SystemTime::now());
            let _ = writeln!(file, "URL: {}", url);
            let _ = writeln!(file, "Method: POST");
            let _ = writeln!(file, "Headers:");
            let _ = writeln!(file, "  Authorization: Bearer {}", self.api_key);
            let _ = writeln!(file, "  Content-Type: application/json");
            let _ = writeln!(
                file,
                "Payload: {}",
                serde_json::to_string_pretty(&payload)
                    .unwrap_or_else(|_| "Failed to serialize".to_string())
            );
            let _ = writeln!(file, "Response Status: {}", res.status());
            let _ = writeln!(file, "Response Headers:");
            for (key, value) in res.headers().iter() {
                let _ = writeln!(file, "  {}: {:?}", key, value);
            }
        }

        if !res.status().is_success() {
            let error = res.text().await?;
            #[cfg(feature = "dev_debug")]
            if let Ok(ref mut file) = debug_file {
                let _ = writeln!(file, "Error Response: {}", error);
                let _ = writeln!(file, "=== End Debug ===\n");
                let _ = file.flush();
            }
            return Err(anyhow!("OpenAI API error: {}", error));
        }

        let json: Value = res.json().await?;
        #[cfg(feature = "dev_debug")]
        if let Ok(ref mut file) = debug_file {
            let _ = writeln!(
                file,
                "Response Body: {}",
                serde_json::to_string_pretty(&json)
                    .unwrap_or_else(|_| "Failed to serialize".to_string())
            );
            let _ = writeln!(file, "=== End Debug ===\n");
            let _ = file.flush();
        }
        let choice = &json["choices"][0]["message"];

        let content = choice["content"].as_str().map(|s| s.to_string());

        let tool_calls = if let Some(calls) = choice["tool_calls"].as_array() {
            Some(serde_json::from_value(json!(calls))?)
        } else {
            None
        };

        Ok(Message {
            role: Role::Assistant,
            content,
            tool_calls,
            tool_call_id: None,
        })
    }

    async fn stream(
        &self,
        request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        let url = format!("{}/chat/completions", self.base_url);

        let mut payload = json!({
            "model": self.model,
            "messages": request.messages,
            "temperature": request.temperature.unwrap_or(0.7),
            "stream": true
        });

        if let Some(obj) = payload.as_object_mut() {
            if let Some(max_tokens) = request.max_tokens {
                obj.insert("max_tokens".to_string(), json!(max_tokens));
            }
            if let Some(tools) = request.tools {
                obj.insert("tools".to_string(), json!(tools));
            }
        }

        let res = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&payload)
            .send()
            .await?;

        #[cfg(feature = "dev_debug")]
        let mut debug_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("openai_debug.log");

        #[cfg(feature = "dev_debug")]
        if let Ok(ref mut file) = debug_file {
            let _ = writeln!(file, "\n=== OpenAI Stream Request Debug ===");
            let _ = writeln!(file, "Timestamp: {:?}", std::time::SystemTime::now());
            let _ = writeln!(file, "URL: {}", url);
            let _ = writeln!(file, "Method: POST");
            let _ = writeln!(file, "Headers:");
            let _ = writeln!(file, "  Authorization: Bearer {}", self.api_key);
            let _ = writeln!(file, "  Content-Type: application/json");
            let _ = writeln!(
                file,
                "Payload: {}",
                serde_json::to_string_pretty(&payload)
                    .unwrap_or_else(|_| "Failed to serialize".to_string())
            );
            let _ = writeln!(file, "Response Status: {}", res.status());
            let _ = writeln!(file, "Response Headers:");
            for (key, value) in res.headers().iter() {
                let _ = writeln!(file, "  {}: {:?}", key, value);
            }
            let _ = file.flush();
        }

        if !res.status().is_success() {
            let error = res.text().await?;
            #[cfg(feature = "dev_debug")]
            if let Ok(ref mut file) = debug_file {
                let _ = writeln!(file, "Error Response: {}", error);
                let _ = writeln!(file, "=== End Debug ===\n");
                let _ = file.flush();
            }
            return Err(anyhow!("OpenAI API error: {}", error));
        }

        let stream = res.bytes_stream();
        let parser = crate::sse::SSEParser::new(stream);

        // Use unfold to allow terminating the stream early when [DONE] is received
        let stream =
            futures::stream::unfold((parser, false), |(mut parser, finished)| async move {
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
