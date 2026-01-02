# server-core Specification Delta

## ADDED Requirements

### Requirement: Server Lifecycle Strategy
The server SHALL employ a configurable `LifecyclePolicy` to determine its response to global system events, supporting distinct behaviors for "Standalone" and "Service" deployments.

#### Scenario: Standalone Mode Initialization
Given the server binary
When executed with the `--standalone` flag
Then the server should initialize with the `StandalonePolicy`.

#### Scenario: Service Mode Initialization
Given the server binary
When executed without the `--standalone` flag
Then the server should initialize with the `ServicePolicy`.

### Requirement: Policy-Driven Shutdown
The server SHALL consult its active `LifecyclePolicy` when receiving a `system::shutdown` event.

#### Scenario: Standalone Graceful Shutdown
Given a server running with `StandalonePolicy`
When a `system::shutdown` event is received
Then the policy should authorize the shutdown
And the server should stop accepting new requests
And flush pending writes
And terminate the process cleanly.

#### Scenario: Service Shutdown Rejection
Given a server running with `ServicePolicy`
When a `system::shutdown` event is received
Then the policy should reject/ignore the shutdown request
And log a warning audit event
And the server should continue running without interruption.

### Requirement: Session Ownership & Authorization
The system SHALL associate every session with an owner and enforce ownership during session termination.

#### Scenario: End Session - Authorized
Given a session S1 owned by user U1
When user U1 requests to end session S1
Then the system should validate the ownership
And terminate session S1
And release associated resources.

#### Scenario: End Session - Unauthorized
Given a session S1 owned by user U1
When user U2 requests to end session S1
Then the system should reject the request
And log a security audit event
And session S1 should remain active.

### Requirement: Session Termination Event
The system SHALL support an explicit, reasoned event to end a specific session.

#### Scenario: End Session with Reason
Given a running server (in any mode)
When a `SystemEvent::EndSession` is received with a valid `session_id` and optional `reason`
Then the server should terminate that specific session
And release associated resources
And log the termination reason for audit purposes.
