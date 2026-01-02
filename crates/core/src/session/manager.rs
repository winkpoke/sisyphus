use super::Session;
use std::collections::HashMap;

#[derive(Default)]
pub struct SessionManager {
    sessions: HashMap<String, Session>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
        }
    }

    pub fn create_session(&mut self) -> &mut Session {
        let session = Session::new();
        let id = session.id.clone();
        self.sessions.insert(id.clone(), session);
        self.sessions.get_mut(&id).unwrap()
    }

    pub fn get_session(&self, id: &str) -> Option<&Session> {
        self.sessions.get(id)
    }

    pub fn get_session_mut(&mut self, id: &str) -> Option<&mut Session> {
        self.sessions.get_mut(id)
    }

    pub fn list_sessions(&self) -> Vec<&Session> {
        self.sessions.values().collect()
    }
}
