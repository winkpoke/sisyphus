use crate::agent::Agent;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct AgentRegistry {
    agents: RwLock<HashMap<String, Arc<Agent>>>,
    default_agent_id: String,
}

impl AgentRegistry {
    pub fn new(default_agent: Arc<Agent>) -> Self {
        let default_id = default_agent.config().name.clone();
        let mut agents = HashMap::new();
        agents.insert(default_id.clone(), default_agent);

        Self {
            agents: RwLock::new(agents),
            default_agent_id: default_id,
        }
    }

    pub fn register(&self, id: String, agent: Arc<Agent>) {
        let mut agents = self.agents.write().expect("Failed to acquire write lock");
        agents.insert(id, agent);
    }

    pub fn get_agent(&self, id: &str) -> Option<Arc<Agent>> {
        let agents = self.agents.read().expect("Failed to acquire read lock");
        agents.get(id).cloned()
    }

    pub fn get_default_agent(&self) -> Arc<Agent> {
        let agents = self.agents.read().expect("Failed to acquire read lock");
        agents
            .get(&self.default_agent_id)
            .expect("Default agent must exist")
            .clone()
    }

    pub fn list_agents(&self) -> Vec<(String, Arc<Agent>)> {
        let agents = self.agents.read().expect("Failed to acquire read lock");
        agents
            .iter()
            .map(|(id, agent)| (id.clone(), agent.clone()))
            .collect()
    }

    pub fn default_agent_id(&self) -> &str {
        &self.default_agent_id
    }
}
