use anyhow::anyhow;
use common::{bus::EventBus, config::Config, llm::LLMProvider, logging, path::SandboxedPath};
use provider::{mock::MockProvider, openai::OpenAIProvider};
use sisyphus_core::agent::{
    config::{AgentConfig, AgentPermissions, PermissionLevel},
    Agent,
};
use std::collections::HashMap;
use std::sync::Arc;
use tools::{
    cmd::CommandTool,
    fs::{ReadFileTool, ReplaceInFileTool, WriteFileTool},
    glob::GlobTool,
    grep::GrepTool,
};

pub fn plan_agent_config() -> AgentConfig {
    let mut overrides = HashMap::new();
    overrides.insert("execute_command".to_string(), PermissionLevel::Deny);

    AgentConfig {
        id: "plan".to_string(),
        name: "Plan Agent".to_string(),
        description: "Read-only planning and analysis agent".to_string(),
        instructions: String::from(
            "You are a planning and analysis agent. Your role is to:\n\
             - Analyze code and architecture\n\
             - Research and investigate issues\n\
             - Create detailed plans and strategies\n\
             - Identify potential risks and trade-offs\n\
             \n\
             You operate in read-only mode. You can read files, search code, and understand the codebase, \
             but you cannot write files or execute commands. Focus on thorough analysis and clear communication.",
        ),
        mode: sisyphus_core::agent::config::AgentMode::Primary,
        permissions: AgentPermissions {
            edit: PermissionLevel::Deny,
            bash: PermissionLevel::Deny,
            skill: PermissionLevel::Allow,
            mode: sisyphus_core::agent::config::PermissionMode::Plan,
            overrides,
            allow: Vec::new(),
            ask: Vec::new(),
            deny: Vec::new(),
        },
        command_path: None,
        context_limits: None,
        system_prompt_template: None,
    }
}

pub fn build_agent_config() -> AgentConfig {
    AgentConfig {
        id: "build".to_string(),
        name: "Build Agent".to_string(),
        description: "Execution and implementation agent".to_string(),
        instructions: String::from(
            "You are a build and execution agent. Your role is to:\n\
             - Implement code changes and features\n\
             - Execute commands and scripts\n\
             - Write and modify files\n\
             - Test and verify implementations\n\
             \n\
             You have access to file operations and command execution. Always seek approval before \
             making potentially destructive changes. Verify your work thoroughly after completion.",
        ),
        mode: sisyphus_core::agent::config::AgentMode::Primary,
        permissions: AgentPermissions {
            edit: PermissionLevel::Ask,
            bash: PermissionLevel::Ask,
            skill: PermissionLevel::Allow,
            mode: sisyphus_core::agent::config::PermissionMode::Default,
            overrides: HashMap::new(),
            allow: Vec::new(),
            ask: Vec::new(),
            deny: Vec::new(),
        },
        command_path: None,
        context_limits: None,
        system_prompt_template: None,
    }
}

pub struct BuiltInAgents {
    pub plan: Arc<Agent>,
    pub build: Arc<Agent>,
    pub bus: Arc<EventBus>,
}

pub async fn build_builtins(config: &Config) -> anyhow::Result<BuiltInAgents> {
    let bus = Arc::new(EventBus::new(100));

    logging::start_event_logger(&bus).await;

    let provider = create_provider(config)?;

    let workspace_root = std::path::PathBuf::from(&config.workspace.root);
    let cwd = std::env::current_dir()?;
    let sandbox = Arc::new(SandboxedPath::new(cwd)?);

    let provider_for_plan = create_provider(config)?;
    let mut plan_agent = Agent::new(
        provider_for_plan,
        bus.clone(),
        plan_agent_config(),
        workspace_root.clone(),
    );

    let mut build_agent = Agent::new(provider, bus.clone(), build_agent_config(), workspace_root);

    plan_agent
        .register_read_only_tools(vec![
            Box::new(ReadFileTool::new(sandbox.clone())),
            Box::new(GlobTool::new(sandbox.clone())),
            Box::new(GrepTool::new(sandbox.clone())),
        ])
        .map_err(|e| anyhow!("Failed to register read-only tools for Plan agent: {}", e))?;

    build_agent.register_tool(Box::new(CommandTool));
    build_agent.register_tool(Box::new(ReadFileTool::new(sandbox.clone())));
    build_agent.register_tool(Box::new(WriteFileTool::new(sandbox.clone())));
    build_agent.register_tool(Box::new(ReplaceInFileTool::new(sandbox.clone())));
    build_agent.register_tool(Box::new(GlobTool::new(sandbox.clone())));
    build_agent.register_tool(Box::new(GrepTool::new(sandbox)));

    Ok(BuiltInAgents {
        plan: Arc::new(plan_agent),
        build: Arc::new(build_agent),
        bus,
    })
}

fn create_provider(config: &Config) -> anyhow::Result<Box<dyn LLMProvider>> {
    Ok(match config.llm.provider.as_str() {
        "mock" => Box::new(MockProvider::new()),
        _ => {
            println!(
                "Initializing provider: {} (model: {})",
                config.llm.provider, config.llm.model
            );
            if let Some(ref url) = config.llm.base_url {
                println!("Base URL: {}", url);
            } else if config.llm.provider != "openai" {
                println!(
                    "Warning: No base_url specified for custom provider. Defaulting to OpenAI."
                );
            }

            let api_key = config
                .llm
                .api_key
                .clone()
                .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                .expect("API Key must be set");
            Box::new(OpenAIProvider::new(
                api_key,
                config.llm.base_url.clone(),
                config.llm.model.clone(),
            ))
        }
    })
}

pub fn build_agent_registry(
    agents: BuiltInAgents,
) -> sisyphus_core::agent::registry::AgentRegistry {
    let registry = sisyphus_core::agent::registry::AgentRegistry::new(agents.plan.clone());
    let build_id = agents.build.config().id.clone();
    registry.register(build_id, agents.build);
    registry
}
