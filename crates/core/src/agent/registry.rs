use crate::agent::Agent;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct AgentRegistry {
    agents: RwLock<HashMap<String, Arc<Agent>>>,
    default_agent_id: String,
}

impl AgentRegistry {
    pub fn new(default_agent: Arc<Agent>) -> Self {
        let default_id = default_agent.config().id.clone();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::config::AgentConfig;
    use common::bus::EventBus;
    use common::llm::{LLMProvider, Message, Role};
    use std::path::PathBuf;

    struct TestProvider;

    #[async_trait::async_trait]
    impl LLMProvider for TestProvider {
        fn model(&self) -> String {
            "test-model".to_string()
        }

        async fn complete(
            &self,
            _request: common::llm::CompletionRequest,
        ) -> anyhow::Result<Message> {
            Ok(Message {
                role: Role::Assistant,
                content: Some("Test response".to_string()),
                tool_calls: None,
                tool_call_id: None,
            })
        }

        async fn stream(
            &self,
            _request: common::llm::CompletionRequest,
        ) -> anyhow::Result<
            std::pin::Pin<Box<dyn futures::Stream<Item = anyhow::Result<String>> + Send>>,
        > {
            let (tx, rx) = futures::channel::mpsc::unbounded();
            tx.unbounded_send(Ok("Test response".to_string()))?;
            Ok(Box::pin(rx))
        }
    }

    fn create_test_agent(id: &str, name: &str) -> Arc<Agent> {
        let config = AgentConfig {
            id: id.to_string(),
            name: name.to_string(),
            description: "Test agent".to_string(),
            instructions: "Be helpful".to_string(),
            mode: crate::agent::config::AgentMode::Primary,
            permissions: crate::agent::config::AgentPermissions::default(),
            command_path: None,
            context_limits: None,
            system_prompt_template: None,
        };

        let provider = Box::new(TestProvider) as Box<dyn LLMProvider>;
        let bus = Arc::new(EventBus::new(100));
        let workspace_root = PathBuf::from("/tmp/test");

        Arc::new(Agent::new(provider, bus, config, workspace_root))
    }

    #[test]
    fn test_registry_uses_stable_id() {
        let agent1 = create_test_agent("stable-id", "Display Name 1");
        let agent2 = create_test_agent("another-id", "Display Name 2");

        let registry = AgentRegistry::new(agent1.clone());

        assert_eq!(registry.default_agent_id(), "stable-id");

        let retrieved = registry.get_agent("stable-id");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().config().id, "stable-id");

        registry.register("another-id".to_string(), agent2);
        let retrieved2 = registry.get_agent("another-id");
        assert!(retrieved2.is_some());
        assert_eq!(retrieved2.unwrap().config().id, "another-id");
    }

    #[test]
    fn test_registry_multiple_agents() {
        let agent1 = create_test_agent("agent-1", "First Agent");
        let agent2 = create_test_agent("agent-2", "Second Agent");

        let registry = AgentRegistry::new(agent1);
        registry.register("agent-2".to_string(), agent2);

        let agents = registry.list_agents();
        assert_eq!(agents.len(), 2);

        let ids: Vec<String> = agents.iter().map(|(id, _)| id.clone()).collect();
        assert!(ids.contains(&"agent-1".to_string()));
        assert!(ids.contains(&"agent-2".to_string()));
    }
}
