# Tasks: Phase 1 Foundation

1.  [x] **Initialize Workspace**
    - Create `Cargo.toml` with workspace members.
    - Create crate directories (`common`, `provider`, `core`, `server`, `cli`, `tools`).
    - Set up shared dependencies (tokio, serde, tracing, anyhow).

2.  [x] **Implement Common Crate**
    - Define `Config` struct and loading logic (config crate).
    - Set up `tracing` subscriber.
    - Define `SystemEvent` enum.
    - Implement `EventBus` (broadcast + mpsc).
    - Define `LLMProvider` trait (using `async-trait`).
    - Define `Tool` trait and `SandboxedPath`.

3.  [x] **Implement Tools Crate**
    - Implement `FileSystemTool` (safe read/write).
    - Implement `CommandTool` (exec).

4.  [x] **Implement Provider Crate**
    - Create `OpenAIProvider` struct.
    - Implement `complete` and `stream` methods using `reqwest`.
    - Map internal `Message` types to OpenAI API.

5.  [x] **Implement Core Crate**
    - Create basic `Agent` struct.
    - Implement `Session` state management.
    - Connect Agent to `LLMProvider` (mock/stub for now).

6.  [x] **Implement CLI Entry Point**
    - Parse args (clap).
    - Initialize Config and Logging.
    - Instantiate `OpenAIProvider`.
    - Run "Hello World" loop.

7.  [x] **Verify End-to-End**
    - Run a test that sends "Hello World" to OpenAI (or mock).
    - Verify logs appear in JSON format.
    - Verify events are broadcasted.
