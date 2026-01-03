use anyhow::Result;
use async_trait::async_trait;
use common::bus::EventBus;
use common::llm::{CompletionRequest, LLMProvider, Message, Role};
use futures::Stream;
use sisyphus_core::agent::{config::AgentConfig, Agent};
use sisyphus_core::command::CommandEffect;
use sisyphus_core::session::Session;
use std::pin::Pin;
use std::sync::Arc;

// Mock Provider
struct MockProvider;
#[async_trait]
impl LLMProvider for MockProvider {
    async fn complete(&self, request: CompletionRequest) -> Result<Message> {
        let content = request.messages.last().unwrap().content.as_ref().unwrap();

        Ok(Message {
            role: Role::Assistant,
            content: Some(format!("Echo: {}", content)),
            tool_calls: None,
            tool_call_id: None,
        })
    }

    async fn stream(
        &self,
        _request: CompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        Ok(Box::pin(futures::stream::empty()))
    }
}

impl MockProvider {
    fn new() -> Self {
        Self
    }
}

#[tokio::test]
async fn test_custom_command() -> Result<()> {
    // Setup temp dir for commands
    let temp_dir = tempfile::tempdir()?;
    let cmd_dir = temp_dir.path(); // tempdir IS a directory, we can use it directly or subdir
                                   // Let's use it directly as the command dir

    let cmd_file = cmd_dir.join("greet.md");
    let content = r#"---
description: Greet someone
---
Hello {{args}}, how are you?"#;
    std::fs::write(&cmd_file, content)?;

    // Setup Agent
    let bus = Arc::new(EventBus::new(10));
    let mut config = AgentConfig::default();
    config.command_path = Some(cmd_dir.to_str().unwrap().to_string());

    // We assume default language is en, which matches our messages
    rust_i18n::set_locale("en");

    let agent = Agent::new(Box::new(MockProvider::new()), bus, config, std::path::PathBuf::from("."));
    let mut session = Session::new();

    // Execute command
    // Input: "/greet world"
    // Expect expansion to "Hello world, how are you?"
    // And LLM receives it.

    let outcome = agent.chat(&mut session, "/greet world".to_string()).await?;

    assert_eq!(outcome.output.unwrap(), "Echo: Hello world, how are you?");

    Ok(())
}

#[tokio::test]
async fn test_builtin_command_new() -> Result<()> {
    let bus = Arc::new(EventBus::new(10));
    let config = AgentConfig::default();
    rust_i18n::set_locale("en");

    let agent = Agent::new(Box::new(MockProvider::new()), bus, config, std::path::PathBuf::from("."));
    let mut session = Session::new();

    session.add_message(Message {
        role: Role::User,
        content: Some("test".to_string()),
        tool_calls: None,
        tool_call_id: None,
    })?;

    assert!(!session.history().is_empty());

    let outcome = agent.chat(&mut session, "/new".to_string()).await?;

    assert_eq!(outcome.output.unwrap(), "New session started.");
    assert_eq!(outcome.effect, CommandEffect::NewSession);

    Ok(())
}

#[tokio::test]
async fn test_resilient_loading() -> Result<()> {
    // Setup temp dir
    let temp_dir = tempfile::tempdir()?;
    let cmd_dir = temp_dir.path();

    // Create invalid command file
    let invalid_file = cmd_dir.join("invalid.txt"); // Not .md
    std::fs::write(&invalid_file, "invalid")?;

    let invalid_name = cmd_dir.join("invalid name.md"); // Whitespace
    std::fs::write(&invalid_name, "---")?;

    let bus = Arc::new(EventBus::new(10));
    let mut config = AgentConfig::default();
    config.command_path = Some(cmd_dir.to_str().unwrap().to_string());

    let agent = Agent::new(Box::new(MockProvider::new()), bus, config, std::path::PathBuf::from("."));

    // Should not crash
    assert!(agent
        .chat(&mut Session::new(), "/help".to_string())
        .await
        .is_ok());

    Ok(())
}

#[tokio::test]
async fn test_builtin_command_quit() -> Result<()> {
    let bus = Arc::new(EventBus::new(10));
    let config = AgentConfig::default();
    rust_i18n::set_locale("en");

    let agent = Agent::new(Box::new(MockProvider::new()), bus.clone(), config, std::path::PathBuf::from("."));
    let mut session = Session::new();

    let outcome = agent.chat(&mut session, "/quit".to_string()).await?;

    assert_eq!(outcome.effect, CommandEffect::Exit);

    Ok(())
}

#[tokio::test]
async fn test_builtin_command_exit() -> Result<()> {
    let bus = Arc::new(EventBus::new(10));
    let config = AgentConfig::default();
    rust_i18n::set_locale("en");

    let agent = Agent::new(Box::new(MockProvider::new()), bus.clone(), config, std::path::PathBuf::from("."));
    let mut session = Session::new();

    let outcome = agent.chat(&mut session, "/exit".to_string()).await?;

    assert_eq!(outcome.effect, CommandEffect::Exit);

    Ok(())
}
