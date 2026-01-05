# Event Bus Optimization

## Motivation
The current `EventBus` implementation uses a single `broadcast` channel for all events. This causes a "thundering herd" problem where every subscriber is woken up for every event, even if they filter for a specific `SystemEventKind`. In a high-traffic system (e.g., streaming tokens vs. tool execution), this wastes CPU cycles on context switching and cloning/deserialization.

## Proposed Solution
Refactor `EventBus` to use a `HashMap<SystemEventKind, broadcast::Sender>` for topic-based distribution, while retaining a global channel for "subscribe all" use cases (like logging).

## Architecture
- **Global Channel**: Retained for `subscribe_all` (e.g., logging, auditing).
- **Topic Channels**: Created for each `SystemEventKind`.
- **Publishing**: Events are sent to *both* the specific topic channel (if active subscribers exist) and the global channel.
- **Subscription**:
    - `subscribe(kind)` connects to the specific topic channel.
    - `subscribe_all()` connects to the global channel.

## Trade-offs
- **Memory**: Slightly higher memory usage due to multiple channels, but `tokio::sync::broadcast` is lightweight.
- **Complexity**: Publishing logic is slightly more complex (two sends instead of one), but isolated in `publish`.
- **Performance**: Significant reduction in unnecessary wakeups for filtered subscriptions.
