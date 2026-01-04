use anyhow::Result;
use reqwest::{Client as ReqwestClient, Url};
use reqwest_eventsource::EventSource;
use serde::{Deserialize, Serialize};
use sisyphus_core::command::{CommandEffect, CommandInfo};
use sisyphus_core::session::Session;

#[derive(Debug, Serialize)]
struct ChatRequest {
    message: String,
}

#[derive(Debug, Serialize)]
struct ApprovalRequest {
    decision: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct ChatResponse {
    pub response: String,
    pub session_id: Option<String>,
    pub usage: Option<String>,
    pub model: Option<String>,
    pub effect: Option<CommandEffect>,
}

#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub model: String,
}

#[derive(Clone)]
pub struct Client {
    base_url: Url,
    http: ReqwestClient,
}

impl Client {
    pub fn new(base_url: Url) -> Self {
        Self {
            base_url,
            http: ReqwestClient::new(),
        }
    }

    pub async fn get_model(&self) -> Result<String> {
        let url = self.base_url.join("/api/v1/model")?;
        let resp = self.http.get(url).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to get model: {}", error_text));
        }

        let info = resp.json::<ModelInfo>().await?;
        Ok(info.model)
    }

    pub async fn health_check(&self) -> Result<()> {
        let url = self.base_url.join("/health")?;
        let resp = self.http.get(url).send().await?;

        if resp.status().is_success() {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Health check failed: {}", resp.status()))
        }
    }

    pub async fn create_session(&self) -> Result<Session> {
        let url = self.base_url.join("/api/v1/sessions")?;
        let resp = self.http.post(url).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to create session: {}", error_text));
        }

        let session = resp.json::<Session>().await?;
        Ok(session)
    }

    pub async fn chat(&self, session_id: &str, message: String) -> Result<ChatResponse> {
        let url = self
            .base_url
            .join(&format!("/api/v1/sessions/{}/chat", session_id))?;
        let req = ChatRequest { message };

        let resp = self.http.post(url).json(&req).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to send message: {}", error_text));
        }

        let chat_resp = resp.json::<ChatResponse>().await?;
        Ok(chat_resp)
    }

    pub async fn submit_approval(
        &self,
        session_id: &str,
        call_id: &str,
        decision: &str,
    ) -> Result<ChatResponse> {
        let url = self.base_url.join(&format!(
            "/api/v1/sessions/{}/approvals/{}",
            session_id, call_id
        ))?;

        let req = ApprovalRequest {
            decision: decision.to_string(),
        };

        let resp = self.http.post(url).json(&req).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to submit approval: {}", error_text));
        }

        let chat_resp = resp.json::<ChatResponse>().await?;
        Ok(chat_resp)
    }

    pub fn subscribe_events(&self) -> Result<EventSource> {
        let url = self.base_url.join("/api/v1/events")?;
        let es = EventSource::get(url);
        Ok(es)
    }

    pub async fn list_sessions(&self) -> Result<Vec<Session>> {
        let url = self.base_url.join("/api/v1/sessions")?;
        let resp = self.http.get(url).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to list sessions: {}", error_text));
        }

        let sessions = resp.json::<Vec<Session>>().await?;
        Ok(sessions)
    }

    pub async fn get_session(&self, session_id: &str) -> Result<Session> {
        let url = self
            .base_url
            .join(&format!("/api/v1/sessions/{}", session_id))?;
        let resp = self.http.get(url).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to get session: {}", error_text));
        }

        let session = resp.json::<Session>().await?;
        Ok(session)
    }

    pub async fn get_commands(&self) -> Result<Vec<CommandInfo>> {
        let url = self.base_url.join("/api/v1/commands")?;
        let resp = self.http.get(url).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to get commands: {}", error_text));
        }

        let commands = resp.json::<Vec<CommandInfo>>().await?;
        Ok(commands)
    }
}
