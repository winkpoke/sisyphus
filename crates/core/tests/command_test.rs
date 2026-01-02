use std::sync::Arc;
use common::bus::EventBus;
use sisyphus_core::agent::{Agent, config::AgentConfig};
use sisyphus_core::session::Session;
use common::llm::{LLMProvider, CompletionRequest, Message, Role};
use async_trait::async_trait;
use anyhow::Result;
use futures::Stream;
use std::pin::Pin;

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

    async fn stream(&self, _request: CompletionRequest) -> Result<Pin<Box<dyn Stream<Item = Result<String>> + Send>>> {
        Ok(Box::pin(futures::stream::empty()))
    }
}

impl MockProvider {
    fn new() -> Self { Self }
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

    let agent = Agent::new(Box::new(MockProvider::new()), bus, config);
    let mut session = Session::new();
    
    // Execute command
    // Input: "/greet world"
    // Expect expansion to "Hello world, how are you?"
    // And LLM receives it.
    
    let response = agent.chat(&mut session, "/greet world".to_string()).await?;
    
    assert_eq!(response, "Echo: Hello world, how are you?");
    
    Ok(())
}

#[tokio::test]
async fn test_builtin_command_new() -> Result<()> {
    let bus = Arc::new(EventBus::new(10));
    let config = AgentConfig::default();
    rust_i18n::set_locale("en");
    
    let agent = Agent::new(Box::new(MockProvider::new()), bus, config);
    let mut session = Session::new();
    
    session.add_message(Message {
        role: Role::User,
        content: Some("test".to_string()),
        tool_calls: None,
        tool_call_id: None,
    });
    
    assert!(!session.history.is_empty());
    
    let response = agent.chat(&mut session, "/new".to_string()).await?;
    
    assert_eq!(response, "New session started.");
    assert!(session.history.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_resilient_loading() -> Result<()> {
    // Setup temp dir
    let temp_dir = tempfile::tempdir()?;
    let cmd_dir = temp_dir.path();

    // 1. Valid command
    let valid_file = cmd_dir.join("valid.md");
    std::fs::write(&valid_file, "---\ndescription: Valid\n---\nValid command")?;

    // 2. Invalid frontmatter (should be skipped)
    let invalid_file = cmd_dir.join("invalid.md");
    std::fs::write(&invalid_file, "No frontmatter here")?;

    // 3. Invalid filename (should be skipped)
    let bad_name_file = cmd_dir.join("bad name.md");
    std::fs::write(&bad_name_file, "---\ndescription: Bad Name\n---\nBad Name")?;

    // Setup Agent
    let bus = Arc::new(EventBus::new(10));
    let mut config = AgentConfig::default();
    config.command_path = Some(cmd_dir.to_str().unwrap().to_string());
    rust_i18n::set_locale("en");

    let agent = Agent::new(Box::new(MockProvider::new()), bus, config);
    let mut session = Session::new();

    // Check if /valid works
    let response = agent.chat(&mut session, "/valid".to_string()).await?;
    assert_eq!(response, "Echo: Valid command");

    // Check that /invalid fails (command not found)
    let err = agent.chat(&mut session, "/invalid".to_string()).await;
    assert!(err.is_err());

    // Check that /bad fails (since "bad name.md" -> name "bad name" -> contains space -> skipped)
    // Note: If it wasn't skipped, it would be registered as "/bad name".
    // But our parser splits by whitespace, so input "/bad name" is command "/bad" with arg "name".
    // Since "bad name.md" is skipped, "/bad" should not exist (unless there was a "bad.md").
    let err = agent.chat(&mut session, "/bad".to_string()).await;
    assert!(err.is_err());

    Ok(())
}
