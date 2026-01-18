use crate::agent::registry::AgentRegistry;
use crate::session::manager::SessionManager;
use anyhow::Result;
use std::sync::Arc;

pub struct ChatService {
    registry: Arc<AgentRegistry>,
    session_manager: Arc<SessionManager>,
}

pub struct ChatServiceResponse {
    pub response: String,
    pub session_id: String,
    pub usage: String,
    pub model: String,
    pub agent_id: String,
}

impl ChatService {
    pub fn new(registry: Arc<AgentRegistry>, session_manager: Arc<SessionManager>) -> Self {
        Self {
            registry,
            session_manager,
        }
    }

    pub async fn chat(&self, session_id: &str, message: String) -> Result<ChatServiceResponse> {
        let session_lock = self
            .session_manager
            .get_session(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        let mut session = session_lock.write().await;
        let agent_id = session
            .agent_id
            .clone()
            .unwrap_or_else(|| self.registry.default_agent_id().to_string());
        let agent = self
            .registry
            .get_agent(&agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

        let outcome = agent.chat(&mut session, message).await?;

        let (effective_session_id, tokens) = (session_id.to_string(), session.estimate_tokens());

        let usage = format!("{} tokens", tokens);
        let model = agent.model_name();

        Ok(ChatServiceResponse {
            response: outcome.output.unwrap_or_default(),
            session_id: effective_session_id,
            usage,
            model,
            agent_id,
        })
    }

    pub async fn resolve_approval(
        &self,
        session_id: &str,
        call_id: &str,
        approved: bool,
    ) -> Result<ChatServiceResponse> {
        let session_lock = self
            .session_manager
            .get_session(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        let mut session = session_lock.write().await;
        let agent_id = session
            .agent_id
            .clone()
            .unwrap_or_else(|| self.registry.default_agent_id().to_string());
        let agent = self
            .registry
            .get_agent(&agent_id)
            .ok_or_else(|| anyhow::anyhow!("Agent not found: {}", agent_id))?;

        let response = agent
            .resolve_approval(&mut session, call_id, approved)
            .await?;

        let tokens = session.estimate_tokens();
        let usage = format!("{} tokens", tokens);
        let model = agent.model_name();

        Ok(ChatServiceResponse {
            response,
            session_id: session_id.to_string(),
            usage,
            model,
            agent_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::config::AgentConfig;
    use crate::agent::Agent;
    use crate::session::manager::SessionManager;
    use anyhow::Result;
    use common::llm::{CompletionRequest, LLMProvider, Message, Role};
    use futures::Stream;
    use std::path::PathBuf;
    use std::pin::Pin;

    struct MockProvider;
    #[async_trait::async_trait]
    impl LLMProvider for MockProvider {
        fn model(&self) -> String {
            "mock".to_string()
        }
        async fn complete(&self, _req: CompletionRequest) -> Result<Message> {
            Ok(Message {
                role: Role::Assistant,
                content: Some("response".to_string()),
                tool_calls: None,
                tool_call_id: None,
                reasoning_summary: None,
                reasoning_raw: None,
            })
        }
        async fn stream(
            &self,
            _req: CompletionRequest,
        ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_chat_response_structure() {
        let provider = Box::new(MockProvider);
        let bus = Arc::new(common::bus::EventBus::new(100));
        let config = AgentConfig::default();
        let agent = Arc::new(Agent::new(provider, bus, config, PathBuf::from(".")));
        let registry = Arc::new(AgentRegistry::new(agent));
        let session_manager = Arc::new(SessionManager::new());
        let service = ChatService::new(registry, session_manager.clone());

        // Create a session
        let session_lock = session_manager.create_session(None);
        let session_id = session_lock.read().await.id.clone();
        drop(session_lock);

        // Call chat
        let res = service
            .chat(&session_id, "hello".to_string())
            .await
            .unwrap();

        assert_eq!(res.response, "response");
        assert_eq!(res.session_id, session_id);
    }
}
