# Design: Phase 1 Foundation

## Workspace Structure
We adopt a Cargo Workspace to enforce separation of concerns.

```
sisyphus/
├── Cargo.toml              # Workspace root
├── crates/
│   ├── common/             # Shared utilities, Traits (LLMProvider), Config, Errors
│   ├── provider/           # LLM Implementations (OpenAI, Anthropic)
│   ├── tools/              # Tool definitions and implementations (FS, Shell)
│   ├── core/               # Main Domain Logic (Agent, Session, Memory)
│   ├── server/             # HTTP/WebSocket API (Axum)
│   └── cli/                # Command Line Interface
```

## Component Details

### 1. Common Crate (`crates/common`)
- **Purpose**: Prevent circular dependencies; act as the glue.
- **Components**:
    - `Config`: Loads from `sisyphus.toml` and ENV vars.
    - `Logging`: `tracing` setup with JSON (UI) and Pretty (CLI) subscribers.
    - `EventBus`:
        - `broadcast`: For "fire-and-forget" updates to UI/logs.
        - `mpsc`: For critical actor-model control flow within the Agent.
    - `LLMProvider` Trait:
        - Defined here so `core` and `provider` can both reference it.
        - Uses `#[async_trait]` for object safety (`Box<dyn LLMProvider>`).
    - `SandboxedPath`: Safe file path handling wrapper.

### 2. Tools Crate (`crates/tools`)
- **Purpose**: Decouple tool logic from the Agent loop.
- **Design**:
    - Defines `Tool` trait.
    - Implements standard tools: `fs`, `shell`, `search`.
    - Tools are injected into the Agent at runtime.

### 3. Provider Crate (`crates/provider`)
- **Purpose**: LLM API adapters.
- **Design**:
    - Implements `LLMProvider` for OpenAI.
    - Handles HTTP requests (reqwest) and SSE streaming.
    - Maps internal `Message` types to provider-specific payloads.

### 4. Core Crate (`crates/core`)
- **Purpose**: Business logic.
- **Design**:
    - `Agent`: The main actor.
    - `Session`: Manages state and history.
    - `Memory`: Short-term and long-term storage.
    - Depends on `common` (for traits) and `tools` (for definitions).
    - Does *not* depend on `provider` (receives `Box<dyn LLMProvider>` via injection).

## Dependencies & Flow
- `server` and `cli` are the "Application Roots".
- They instantiate `Config`, specific `Provider`, and `Tools`.
- They inject these into `Core` to start an `Agent`.
