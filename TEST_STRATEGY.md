# Sisyphus Testing Strategy

This document outlines the comprehensive testing strategy for the Sisyphus project (a Rust port of `opencode`). It ensures the reliability of agentic behaviors, LLM interactions, and system tool execution without relying on live LLM APIs during standard test runs.

## 1. Testing Philosophy: The Pyramid

We adopt a standard Testing Pyramid approach, adapted for AI Agent systems:

*   **Unit Tests (70%)**: Focus heavily on `core` logic (Agent loop, Session management) using **mocks** for LLM and Tools. These are fast, deterministic, and run on every commit.
*   **Integration Tests (20%)**: Verify that `providers` correctly format requests for APIs and that `tools` correctly interact with the OS (in sandboxed environments).
*   **End-to-End (E2E) Tests (10%)**: "Black box" testing of the CLI and Server using a "replay" mechanism for LLM responses.

## 2. Recommended Tooling & Stack

We leverage Rust ecosystem's best-in-class testing libraries. These are standard `dev-dependencies` in our crates.

| Crate | Purpose |
|-------|---------|
| **`mockall`** | **Critical**. Used to mock `LLMProvider` and `Tool` traits. Allows us to test Agent's decision loop without real AI. |
| **`tokio-test`** | For testing async functions and streams deterministically. |
| **`tempfile`** | For safely testing filesystem tools (`fs.rs`) without polluting the developer's machine. |
| **`wiremock`** | For testing HTTP providers (OpenAI, Anthropic) by mocking external API endpoints. |
| **`insta`** | **Snapshot Testing**. Extremely useful for verifying generated Prompts and JSON schemas. |
| **`cargo-llvm-cov`** | **Coverage Tracking**. Generates HTML coverage reports. |

**Note**: `tarpaulin` is configured but coverage is measured using `cargo-llvm-cov` for better workspace-wide support.

## 3. Component-Specific Strategy

### A. `crates/common`: Foundation
*   **Focus**: Pure logic, serialization, and event bus.
*   **Strategy**: Standard `#[test]` unit tests.
*   **Key Scenarios**:
    *   Verify `Message` and `ToolCall` serialization matches LLM provider expectations (using `insta` snapshots).
    *   Test `EventBus` subscription and publication ordering.

### B. `crates/core`: The Brain (High Priority)
*   **Focus**: The `Agent` loop, state management, and context window logic.
*   **Strategy**: **Heavy Mocking**.
*   **Key Scenarios**:
    *   **The "Happy Path" Loop**: Mock an LLM that returns a tool call -> Mock the Tool to return "Success" -> Mock LLM to return final answer. Verify the loop runs exactly X times.
    *   **Context Compaction**: Feed the Session 100 messages and verify the compaction algorithm correctly summarizes/truncates based on token limits.
    *   **Error Handling**: Mock an LLM failure (network error) and verify the Agent retries or degrades gracefully.
    *   **Tool Output Injection**: Verify that tool outputs are correctly appended to the `Session` history.

### C. `crates/provider`: LLM Adapters
*   **Focus**: Correctly translating our generic `Message` format to OpenAI/Anthropic specific JSON.
*   **Strategy**: **Wiremock** (HTTP Mocking).
*   **Key Scenarios**:
    *   Start a local `wiremock` server.
    *   Configure `OpenAIProvider` to point to `localhost:mock_port`.
    *   Send a `CompletionRequest` and assert the provider sends the correct JSON body (headers, auth, schema).
    *   **Do not** call real APIs in tests unless explicitly enabled via a `live-test` feature flag.

### D. `crates/tools`: System Interaction
*   **Focus**: File system and Shell command execution.
*   **Strategy**: **Sandboxing**.
*   **Key Scenarios**:
    *   **FS Tools**: Use `tempfile::tempdir()` for *every* test. Verify `write_file` creates the file and `read_file` reads it back. Ensure no file operations escape the temp dir.
    *   **Cmd Tools**: Only test safe commands (`echo`, `ls`) or mock the `Command` executor if possible. Avoid running side-effect heavy commands.

### E. `crates/server`: API Surface
*   **Focus**: HTTP/WebSocket endpoints.
*   **Strategy**: Integration tests using `axum::test` or `reqwest` against a running test server.
*   **Key Scenarios**:
    *   Spin up the server in a background task.
    *   Send a `POST /chat`.
    *   Verify auth middleware rejects invalid tokens.

## 4. Implementation Guidelines

### Writing a Core Agent Test

```rust
// Conceptual example of an Agent Unit Test using Mockall
#[tokio::test]
async fn test_agent_calls_tool_and_responds() {
    // 1. Setup Mocks
    let mut mock_provider = MockLLMProvider::new();
    let mut mock_tool = MockTool::new();

    // 2. Expect LLM to request a tool call
    mock_provider.expect_complete()
        .times(1)
        .returning(|_| Ok(Message::tool_call("weather_tool", "NYC")));

    // 3. Expect Tool to be executed
    mock_tool.expect_execute()
        .with(eq(json!({"location": "NYC"})))
        .times(1)
        .returning(|_| Ok("Sunny".to_string()));

    // 4. Expect LLM to see the result and finish
    mock_provider.expect_complete()
        .times(1) // Second call
        .returning(|_| Ok(Message::text("It is sunny in NYC")));

    // 5. Run Agent
    let mut agent = Agent::new(Box::new(mock_provider), ...);
    agent.register_tool(Box::new(mock_tool));
    
    let response = agent.chat("What's the weather?").await.unwrap();
    assert_eq!(response, "It is sunny in NYC");
}
```

## 5. Continuous Integration

*   Tests must pass on every PR.
*   `cargo test --workspace` is the standard command.
*   Tests should not require API keys (unless running a specific `live` suite).
