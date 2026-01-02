use common::{bus::EventBus, config::Config, llm::LLMProvider, logging, path::SandboxedPath};
use provider::{mock::MockProvider, openai::OpenAIProvider};
use sisyphus_core::agent::{config::AgentConfig, Agent};
use std::sync::Arc;
use tools::{
    cmd::CommandTool,
    fs::{ReadFileTool, WriteFileTool},
};

pub struct AgentComponents {
    pub agent: Arc<Agent>,
    pub bus: Arc<EventBus>,
}

pub async fn build_agent(config: &Config) -> anyhow::Result<AgentComponents> {
    let bus = Arc::new(EventBus::new(100));

    // Subscribe to bus for logging
    logging::start_event_logger(&bus).await;

    let provider: Box<dyn LLMProvider> = match config.llm.provider.as_str() {
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
    };

    let mut agent = Agent::new(provider, bus.clone(), AgentConfig::default());

    // Register tools
    agent.register_tool(Box::new(CommandTool));

    let cwd = std::env::current_dir()?;
    let sandbox = Arc::new(SandboxedPath::new(cwd)?);

    agent.register_tool(Box::new(ReadFileTool::new(sandbox.clone())));
    agent.register_tool(Box::new(WriteFileTool::new(sandbox)));

    Ok(AgentComponents {
        agent: Arc::new(agent),
        bus,
    })
}
