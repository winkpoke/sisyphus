use anyhow::{anyhow, Result};
use async_trait::async_trait;
use common::llm::{CompletionRequest, LLMProvider, Message, Role};
use futures::{Stream, StreamExt};
use reqwest::Client;
use serde_json::{json, Value};
use std::pin::Pin;

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

        if !res.status().is_success() {
            let error = res.text().await?;
            return Err(anyhow!("OpenAI API error: {}", error));
        }

        let json: Value = res.json().await?;
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

        if !res.status().is_success() {
            let error = res.text().await?;
            return Err(anyhow!("OpenAI API error: {}", error));
        }

        let stream = res.bytes_stream();

        let stream =
            futures::stream::unfold((stream, Vec::new()), |(mut stream, mut buf)| async move {
                loop {
                    if let Some(i) = buf.iter().position(|&b| b == b'\n') {
                        let line_bytes = buf.drain(..=i).collect::<Vec<_>>();
                        let line = String::from_utf8_lossy(&line_bytes);
                        let line = line.trim();

                        if line.starts_with("data: ") {
                            let data = &line["data: ".len()..];
                            if data != "[DONE]" {
                                if let Ok(json) = serde_json::from_str::<Value>(data) {
                                    if let Some(content) =
                                        json["choices"][0]["delta"]["content"].as_str()
                                    {
                                        if !content.is_empty() {
                                            return Some((Ok(content.to_string()), (stream, buf)));
                                        }
                                    }
                                }
                            }
                        }
                        continue;
                    }

                    match stream.next().await {
                        Some(Ok(bytes)) => {
                            buf.extend_from_slice(&bytes);
                        }
                        Some(Err(e)) => {
                            return Some((Err(anyhow::anyhow!(e)), (stream, buf)));
                        }
                        None => {
                            return None;
                        }
                    }
                }
            });

        Ok(Box::pin(stream))
    }
}
