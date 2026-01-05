## ADDED Requirements

### Requirement: Unified Slash Command Dispatch Interface
The CLI SHALL route user input through a single slash-command dispatch interface shared by both REPL and TUI.

#### Scenario: REPL and TUI share the same dispatch policy
- **GIVEN** the user is interacting via the REPL or the TUI
- **WHEN** the user submits the same slash command input
- **THEN** the CLI SHALL make the same local-vs-remote dispatch decision

### Requirement: Client-Local Command Registration
The CLI SHALL allow frontends to register client-local slash commands.

A client-local command handler MUST be able to:
- consume the command locally
- forward the command to the server (optionally rewriting the forwarded message)
- decline handling so the command is treated as remote

#### Scenario: Client-local command consumes locally
- **GIVEN** a client-local command `/debug` is registered
- **WHEN** the user executes `/debug`
- **THEN** the CLI SHALL handle the command locally
- **AND** it SHALL NOT send a chat request to the server for that command

#### Scenario: Client-local command forwards to server
- **GIVEN** a client-local command `/local.fix` is registered to rewrite and forward
- **WHEN** the user executes `/local.fix main.rs`
- **THEN** the CLI SHALL forward a rewritten message to the server

### Requirement: Deterministic Local vs Remote Name Collision Policy
The CLI MUST define a deterministic policy for resolving name collisions between client-local and server-registered commands.

The default policy SHOULD prevent accidental shadowing of server-registered commands.

#### Scenario: Local command does not shadow remote by default
- **GIVEN** the server reports a registered command named `/new`
- **AND** the client registers a local command also named `/new` using the default policy
- **WHEN** the user executes `/new`
- **THEN** the CLI SHALL route the command to the server

