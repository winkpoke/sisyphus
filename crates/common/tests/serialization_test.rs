use common::llm::{Message, Role};
use serde_json;

#[test]
fn test_reasoning_fields_not_serialized() {
    // Create a message with reasoning fields populated (as they would be from a provider response)
    let message = Message {
        role: Role::Assistant,
        content: Some("Final answer".to_string()),
        tool_calls: None,
        tool_call_id: None,
        reasoning_summary: Some("Thought for 1s".to_string()),
        reasoning_raw: Some("This is detailed reasoning content".to_string()),
    };

    // Serialize to JSON
    let serialized = serde_json::to_string(&message).unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&serialized).unwrap();

    // Verify reasoning fields are NOT in the serialized output
    assert!(
        !parsed.get("reasoning_summary").is_some(),
        "reasoning_summary should not be serialized in requests"
    );
    assert!(
        !parsed.get("reasoning_raw").is_some(),
        "reasoning_raw should not be serialized in requests"
    );

    // Verify standard fields ARE present
    assert_eq!(parsed["role"], "assistant");
    assert_eq!(parsed["content"], "Final answer");
}

#[test]
fn test_reasoning_fields_can_be_deserialized() {
    // Provider response with reasoning fields (as they would appear in a response)
    let json_response = r#"{
        "role": "assistant",
        "content": "Final answer",
        "reasoning_summary": "Thought for 1s",
        "reasoning_raw": "Detailed reasoning"
    }"#;

    // Should deserialize successfully
    let message: Message = serde_json::from_str(json_response).unwrap();

    assert_eq!(message.role, Role::Assistant);
    assert_eq!(message.content, Some("Final answer".to_string()));
    assert_eq!(
        message.reasoning_summary,
        Some("Thought for 1s".to_string())
    );
    assert_eq!(
        message.reasoning_raw,
        Some("Detailed reasoning".to_string())
    );
}
