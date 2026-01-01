use common::llm::{LLMProvider, CompletionRequest, Message, Role};
use async_trait::async_trait;
use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_json::{json, Value};
use futures::Stream;
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

    fn map_messages(messages: &[Message]) -> Vec<Value> {
        messages.iter().map(|m| {
            json!({
                "role": match m.role {
                    Role::System => "system",
                    Role::User => "user",
                    Role::Assistant => "assistant",
                    Role::Tool => "tool", 
                },
                "content": m.content
            })
        }).collect()
    }
}

#[async_trait]
impl LLMProvider for OpenAIProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<String> {
        let url = format!("{}/chat/completions", self.base_url);
        let body = json!({
            "model": self.model,
            "messages": Self::map_messages(&request.messages),
            "temperature": request.temperature.unwrap_or(0.7),
            "max_tokens": request.max_tokens,
            "stream": false
        });

        let res = self.client.post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&body)
            .send()
            .await?;
            
        if !res.status().is_success() {
            let error = res.text().await?;
            return Err(anyhow!("OpenAI API error: {}", error));
        }

        let json: Value = res.json().await?;
        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| anyhow!("No content in response"))?
            .to_string();

        Ok(content)
    }

    async fn stream(&self, _request: CompletionRequest) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        Err(anyhow!("Streaming not implemented yet"))
    }
}
