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

        let mut effective_session_id = session_id.to_string();

        match outcome.effect {
            CommandEffect::NewSession => {
                let new_session_lock = self.session_manager.create_session();
                let new_session = new_session_lock.read().await;
                effective_session_id = new_session.id.clone();
            }
            CommandEffect::ClearHistory => {
                session.clear_context();
            }
            CommandEffect::Exit => {
                // Client-scoped effect, don't shut down process here.
                // The effect is returned to the client to handle.
            }
            CommandEffect::ToggleDebug => {}
            CommandEffect::None => {}
        }

        let tokens = session.estimate_tokens();
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
