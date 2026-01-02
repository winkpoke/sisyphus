use crate::session::context::ContextLimits;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentMode {
    Primary,
    SubAgent,
    All,
}

impl Default for AgentMode {
    fn default() -> Self {
        Self::Primary
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PermissionLevel {
    Full,
    ReadOnly,
    Ask,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentPermissions {
    pub edit: PermissionLevel,
    pub bash: PermissionLevel,
    pub skill: PermissionLevel,
}

impl Default for AgentPermissions {
    fn default() -> Self {
        Self {
            edit: PermissionLevel::Ask,
            bash: PermissionLevel::Ask,
            skill: PermissionLevel::Ask,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentConfig {
    pub name: String,
    pub description: String,
    pub instructions: String,
    pub mode: AgentMode,
    pub permissions: AgentPermissions,
    pub command_path: Option<String>,
    pub context_limits: Option<ContextLimits>,
}

impl AgentConfig {
    pub fn get_command_path(&self) -> String {
        self.command_path
            .clone()
            .unwrap_or_else(|| ".sisyphus/command".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = AgentConfig {
            name: "Test Agent".to_string(),
            description: "A test agent".to_string(),
            instructions: "Be helpful".to_string(),
            mode: AgentMode::Primary,
            permissions: AgentPermissions::default(),
            command_path: None,
            context_limits: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: AgentConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.name, deserialized.name);
        assert_eq!(config.mode, deserialized.mode);
        assert_eq!(config.permissions.edit, deserialized.permissions.edit);
    }
}
