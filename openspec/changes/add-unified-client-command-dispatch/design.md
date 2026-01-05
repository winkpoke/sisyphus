# Design: Unified Client Slash Command Dispatch (Local + Remote)

## Context
Sisyphus supports slash commands executed server-side via the core command system, including custom command expansion and lifecycle effects.

The REPL currently forwards user input (including slash commands) to the server.
The TUI currently has a separate local registry and performs local execution for some commands.

## Problem
The client frontends do not share a single, deterministic slash-command routing model. This can produce:
- inconsistent parsing behavior (quoted args vs whitespace splitting)
- inconsistent command discovery and availability
- duplicated logic that must be maintained in multiple UI frontends

## Goals
- Provide a single client-side interface for slash command handling used by both REPL and TUI.
- Support client-local slash commands that can consume locally or forward to the server.
- Keep server-side slash command semantics authoritative for session state and custom command expansion.
- Enable progressive adoption with minimal behavior changes, avoiding a large refactor.

## Non-Goals
- Redesigning the core command system, templates, or parser.
- Changing server endpoints beyond what is required for discovery.

## Proposed Architecture

### Components

1. **SlashCommandRouter (shared, pure routing)**
- Input: raw user input string
- Output: a dispatch decision

2. **ClientLocalCommandRegistry (pluggable interceptors)**
- Stores client-registered slash commands.
- Each command handler returns one of:
  - consume locally (produce a local outcome)
  - forward (optionally rewritten message)
  - pass (not handled)

3. **RemoteExecutor (existing client.chat call)**
- Sends message to the server.
- Server applies command effects to session state and returns effect metadata to client.

### Dispatch Decision Model
The router produces exactly one of:
- **NotSlash**: not a slash command; forward as normal chat.
- **LocalConsume**: run client handler and apply local effects/output.
- **RemoteForward**: forward a message to server (original or rewritten).

### Parsing Consistency
Both REPL and TUI MUST identify slash command name using the shared core parser. This ensures quoted-argument semantics match server behavior.

### Precedence and Collision
Default precedence is **local-first**:
- If a client-local command name matches, its handler executes.
- Otherwise the message is forwarded to the server.

To prevent accidental shadowing of remote commands, local registration includes a collision policy:
- `AllowShadowRemote`: local handler may override a remote command name.
- `DenyShadowRemote` (default): the client MUST treat the command as remote if the name also exists in the remote command list.

Clients MAY expose origin metadata in discovery UI to make execution location clear.

### Command Discovery
The TUI command palette should render a merged list:
- Remote commands fetched from the server commands endpoint.
- Local commands from the client-local registry.

The palette MAY annotate commands with origin (local vs server) and may indicate forwarding behavior.

## Progressive Migration
Phase 1:
- Introduce router + local registry.
- Register only UI-scoped local commands (e.g., `/debug`).
- Forward all other slash commands to the server.

Phase 2:
- Remove TUI-local execution of server commands.
- Keep local-only commands as interceptors.

Phase 3 (optional):
- Expand local commands for UI-only functionality where appropriate.

## Trade-offs
- Local-first increases flexibility for UI features but risks shadowing remote commands; explicit collision policy mitigates this.
- Fetching remote commands for discovery adds a dependency on the server endpoint; clients can cache and refresh lazily.

