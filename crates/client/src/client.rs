use anyhow::Result;
use reqwest::{Client as ReqwestClient, Url};
use reqwest_eventsource::EventSource;
use serde::{Deserialize, Serialize};
use sisyphus_core::command::CommandInfo;
use sisyphus_core::session::{Session, SessionSummary};

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
}

#[derive(Debug, Deserialize)]
pub struct ModelInfo {
    pub model: String,
}

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct AgentResponse {
    pub id: String,
    pub model: String,
    pub name: String,
    pub description: String,
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

    pub async fn list_sessions(&self) -> Result<Vec<SessionSummary>> {
        let url = self.base_url.join("/api/v1/sessions")?;
        let resp = self.http.get(url).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to list sessions: {}", error_text));
        }

        let sessions = resp.json::<Vec<SessionSummary>>().await?;
        Ok(sessions)
    }

    pub async fn clear_session(&self, session_id: &str) -> Result<Session> {
        let url = self
            .base_url
            .join(&format!("/api/v1/sessions/{}/clear", session_id))?;
        let resp = self.http.post(url).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to clear session: {}", error_text));
        }

        let session = resp.json::<Session>().await?;
        Ok(session)
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

    pub async fn list_agents(&self) -> Result<Vec<AgentResponse>> {
        let url = self.base_url.join("/api/v1/agents")?;
        let resp = self.http.get(url).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Failed to list agents: {}", error_text));
        }

        let agents = resp.json::<Vec<AgentResponse>>().await?;
        Ok(agents)
    }

    pub async fn update_session_agent(&self, session_id: &str, agent_id: &str) -> Result<Session> {
        let url = self
            .base_url
            .join(&format!("/api/v1/sessions/{}/agent", session_id))?;
        let req = serde_json::json!({ "agent_id": agent_id });

        let resp = self.http.put(url).json(&req).send().await?;

        if !resp.status().is_success() {
            let error_text = resp.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Failed to update session agent: {}",
                error_text
            ));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_initialization() {
        let url = url::Url::parse("http://localhost:3000").unwrap();
        let client = Client::new(url);

        let base_url = url::Url::parse("http://localhost:3000").unwrap();
        assert_eq!(client.base_url, base_url);
    }

    #[test]
    fn test_url_construction() {
        let base = url::Url::parse("http://localhost:3000").unwrap();

        assert!(base.join("/api/v1/sessions").is_ok());
        assert!(base.join("/api/v1/model").is_ok());
        assert!(base.join("/health").is_ok());
        assert!(base
            .join(&format!("/api/v1/sessions/{}/chat", "session-123"))
            .is_ok());
    }

    #[test]
    fn test_chat_request_serialization() {
        let req = ChatRequest {
            message: "Hello, world!".to_string(),
        };

        let json = serde_json::to_string(&req);
        assert!(json.is_ok());

        let parsed = json.unwrap();
        assert!(parsed.contains("Hello, world!"));
        assert!(parsed.contains("message"));
    }

    #[test]
    fn test_approval_request_serialization() {
        let req = ApprovalRequest {
            decision: "approve".to_string(),
        };

        let json = serde_json::to_string(&req);
        assert!(json.is_ok());

        let parsed = json.unwrap();
        assert!(parsed.contains("approve"));
        assert!(parsed.contains("decision"));
    }

    #[test]
    fn test_chat_response_deserialization() {
        let json = r#"{
            "response": "Test response",
            "session_id": "session-123",
            "usage": "100 tokens",
            "model": "gpt-4"
        }"#;

        let resp: Result<ChatResponse, _> = serde_json::from_str(json);
        assert!(resp.is_ok());

        let parsed = resp.unwrap();
        assert_eq!(parsed.response, "Test response");
        assert_eq!(parsed.session_id, Some("session-123".to_string()));
        assert_eq!(parsed.usage, Some("100 tokens".to_string()));
        assert_eq!(parsed.model, Some("gpt-4".to_string()));
    }

    #[test]
    fn test_chat_response_with_optional_fields() {
        let json = r#"{
            "response": "Test"
        }"#;

        let resp: Result<ChatResponse, _> = serde_json::from_str(json);
        assert!(resp.is_ok());

        let parsed = resp.unwrap();
        assert_eq!(parsed.response, "Test");
        assert!(parsed.session_id.is_none());
        assert!(parsed.usage.is_none());
        assert!(parsed.model.is_none());
    }

    #[test]
    fn test_agent_response_deserialization() {
        let json = r#"{
            "id": "plan",
            "name": "Plan Agent",
            "description": "Planning agent",
            "model": "gpt-4"
        }"#;

        let resp: Result<AgentResponse, _> = serde_json::from_str(json);
        assert!(resp.is_ok());

        let parsed = resp.unwrap();
        assert_eq!(parsed.id, "plan");
        assert_eq!(parsed.name, "Plan Agent");
        assert_eq!(parsed.description, "Planning agent");
        assert_eq!(parsed.model, "gpt-4");
    }

    #[test]
    fn test_model_info_deserialization() {
        let json = r#"{
            "model": "gpt-4"
        }"#;

        let resp: Result<ModelInfo, _> = serde_json::from_str(json);
        assert!(resp.is_ok());

        let parsed = resp.unwrap();
        assert_eq!(parsed.model, "gpt-4");
    }

    #[test]
    fn test_client_url_error_handling() {
        let url = url::Url::parse("http://localhost:3000").unwrap();
        let client = Client::new(url);

        let result = client.base_url.join("relative/path");
        assert!(result.is_ok());

        let absolute_path = "http://external.com/path";
        let result = client.base_url.join(absolute_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_session_id_formatting() {
        let session_id = "session-123";
        let base = url::Url::parse("http://localhost:3000").unwrap();

        let chat_url = base.join(&format!("/api/v1/sessions/{}/chat", session_id));
        assert!(chat_url.is_ok());

        let agent_url = base.join(&format!("/api/v1/sessions/{}/agent", session_id));
        assert!(agent_url.is_ok());

        let clear_url = base.join(&format!("/api/v1/sessions/{}/clear", session_id));
        assert!(clear_url.is_ok());
    }
}
