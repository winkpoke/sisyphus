use sisyphus_core::command::{CommandConfig, CommandRegistry};

#[test]
fn test_reserved_names() {
    let mut registry = CommandRegistry::new();
    
    let config = CommandConfig {
        description: Some("test".to_string()),
        template: "test".to_string(),
    };

    // Try to register reserved names
    registry.register_custom("help", config.clone());
    registry.register_custom("quit", config.clone());
    registry.register_custom("exit", config.clone());
    registry.register_custom("clear", config.clone());
    registry.register_custom("debug", config.clone());
    registry.register_custom("new", config.clone());

    // Verify they were NOT registered
    assert!(registry.get("help").is_none());
    assert!(registry.get("quit").is_none());
    assert!(registry.get("exit").is_none());
    assert!(registry.get("clear").is_none());
    assert!(registry.get("debug").is_none());
    assert!(registry.get("new").is_none());

    // Verify valid name works
    registry.register_custom("valid", config);
    assert!(registry.get("valid").is_some());
}
