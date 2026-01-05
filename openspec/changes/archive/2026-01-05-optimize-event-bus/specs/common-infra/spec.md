# Event Bus Specifications

## MODIFIED Requirements

### Requirement: Type-Safe Event Bus
The system MUST provide a typed, topic-based event bus for broadcasting system events efficiently. It MUST distribute events only to subscribers that have requested them (by topic), while supporting global auditing.

#### Scenario: Event Broadcasting
Given a subscriber to the event bus
When an `AgentStateChanged` event is published
Then the subscriber receives the event with the correct payload.

#### Scenario: Specific Subscription
Given an `EventBus` with a subscriber for `MessageReceived`
When a `ToolExecuted` event is published
Then the `MessageReceived` subscriber MUST NOT be woken up

#### Scenario: Global Subscription
Given an `EventBus` with a `subscribe_all` listener
When any event is published
Then the listener MUST receive the event
