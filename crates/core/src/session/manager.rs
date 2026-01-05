use super::Session;
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct SessionManager {
    sessions: DashMap<String, Arc<RwLock<Session>>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: DashMap::new(),
        }
    }

    pub fn create_session(&self, agent_id: Option<String>) -> Arc<RwLock<Session>> {
        let session = Session::new(agent_id);
        let id = session.id.clone();
        let session = Arc::new(RwLock::new(session));
        self.sessions.insert(id.clone(), session.clone());
        session
    }

    pub fn get_session(&self, id: &str) -> Option<Arc<RwLock<Session>>> {
        self.sessions.get(id).map(|entry| entry.value().clone())
    }

    pub fn list_sessions(&self) -> Vec<Arc<RwLock<Session>>> {
        self.sessions
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }
}
