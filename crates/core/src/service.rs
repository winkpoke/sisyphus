use anyhow::Result;
use std::sync::Arc;
use crate::agent::Agent;
use crate::session::manager::SessionManager;
use crate::command::CommandEffect;

pub struct ChatService {
    agent: Arc<Agent>,
    session_manager: Arc<SessionManager>,
}

pub struct ChatServiceResponse {
    pub response: String,
    pub session_id: String,
    pub effect: CommandEffect,
    pub usage: String,
    pub model: String,
}

impl ChatService {
    pub fn new(
        agent: Arc<Agent>,
        session_manager: Arc<SessionManager>,
    ) -> Self {
        Self {
            agent,
            session_manager,
        }
    }

    pub async fn chat(&self, session_id: &str, message: String) -> Result<ChatServiceResponse> {
        let session_lock = self.session_manager.get_session(session_id).ok_or_else(|| {
            anyhow::anyhow!("Session not found: {}", session_id)
        })?;

        let mut session = session_lock.write().await;
        let outcome = self.agent.chat(&mut *session, message).await?;

        let (effective_session_id, tokens) = match outcome.effect {
            CommandEffect::NewSession => {
                let new_session_lock = self.session_manager.create_session();
                let new_session = new_session_lock.read().await;
                (new_session.id.clone(), new_session.estimate_tokens())
            }
            CommandEffect::ClearHistory => {
                session.clear_context();
                (session_id.to_string(), session.estimate_tokens())
            }
            CommandEffect::Exit => {
                // Client-scoped effect, don't shut down process here.
                // The effect is returned to the client to handle.
                (session_id.to_string(), session.estimate_tokens())
            }
            _ => {
                (session_id.to_string(), session.estimate_tokens())
            }
        };

        let usage = format!("{} tokens", tokens);
        let model = self.agent.model_name();

        Ok(ChatServiceResponse {
            response: outcome.output.unwrap_or_default(),
            session_id: effective_session_id,
            effect: outcome.effect,
            usage,
            model,
        })
    }

    pub async fn resolve_approval(
        &self,
        session_id: &str,
        call_id: &str,
        approved: bool,
    ) -> Result<ChatServiceResponse> {
        let session_lock = self.session_manager.get_session(session_id).ok_or_else(|| {
            anyhow::anyhow!("Session not found: {}", session_id)
        })?;

        let mut session = session_lock.write().await;
        let response = self.agent.resolve_approval(&mut *session, call_id, approved).await?;

        let tokens = session.estimate_tokens();
        let usage = format!("{} tokens", tokens);
        let model = self.agent.model_name();

        Ok(ChatServiceResponse {
            response,
            session_id: session_id.to_string(),
            effect: CommandEffect::None, // Approvals don't trigger effects currently
            usage,
            model,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::config::AgentConfig;
    use crate::session::manager::SessionManager;
    use common::llm::{LLMProvider, CompletionRequest, Message, Role};
    use anyhow::Result;
    use std::path::PathBuf;
    use std::pin::Pin;
    use futures::Stream;

    struct MockProvider;
    #[async_trait::async_trait]
    impl LLMProvider for MockProvider {
        fn model(&self) -> String { "mock".to_string() }
        async fn complete(&self, _req: CompletionRequest) -> Result<Message> {
            Ok(Message {
                role: Role::Assistant,
                content: Some("response".to_string()),
                tool_calls: None,
                tool_call_id: None,
            })
        }
        async fn stream(&self, _req: CompletionRequest) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
            unimplemented!()
        }
    }

    #[tokio::test]
    async fn test_new_session_token_usage() {
        let provider = Box::new(MockProvider);
        let bus = Arc::new(common::bus::EventBus::new(100));
        let config = AgentConfig::default();
        let agent = Arc::new(Agent::new(provider, bus, config, PathBuf::from(".")));
        let session_manager = Arc::new(SessionManager::new());
        let service = ChatService::new(agent, session_manager.clone());

        // Create a session
        let session_lock = session_manager.create_session();
        let session_id = session_lock.read().await.id.clone();
        drop(session_lock);

        // Add some messages manually
        {
            let lock = session_manager.get_session(&session_id).unwrap();
            let mut session = lock.write().await;
            session.add_message(Message {
                role: Role::User,
                content: Some("hello ".repeat(100)),
                tool_calls: None,
                tool_call_id: None,
            }).unwrap();
            assert!(session.estimate_tokens() > 0);
        }

        // Call /new
        let res = service.chat(&session_id, "/new".to_string()).await.unwrap();
        
        assert_eq!(res.effect, CommandEffect::NewSession);
        assert_ne!(res.session_id, session_id);
        
        let tokens: u32 = res.usage.split_whitespace().next().unwrap().parse().unwrap();
        assert_eq!(tokens, 0, "Usage should be 0 for new session, but was {}", tokens);
    }

    #[tokio::test]
    async fn test_clear_history_token_usage() {
        let provider = Box::new(MockProvider);
        let bus = Arc::new(common::bus::EventBus::new(100));
        let config = AgentConfig::default();
        let agent = Arc::new(Agent::new(provider, bus, config, PathBuf::from(".")));
        let session_manager = Arc::new(SessionManager::new());
        let service = ChatService::new(agent, session_manager.clone());

        // Create a session
        let session_lock = session_manager.create_session();
        let session_id = session_lock.read().await.id.clone();
        drop(session_lock);

        // Add some messages manually
        {
            let lock = session_manager.get_session(&session_id).unwrap();
            let mut session = lock.write().await;
            session.add_message(Message {
                role: Role::User,
                content: Some("hello ".repeat(100)),
                tool_calls: None,
                tool_call_id: None,
            }).unwrap();
        }

        // Call /clear
        let res = service.chat(&session_id, "/clear".to_string()).await.unwrap();
        
        assert_eq!(res.effect, CommandEffect::ClearHistory);
        assert_eq!(res.session_id, session_id);
        
        let tokens: u32 = res.usage.split_whitespace().next().unwrap().parse().unwrap();
        assert_eq!(tokens, 0, "Usage should be 0 after clear, but was {}", tokens);
    }
}

