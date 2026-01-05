# Event Bus Optimization Design

## Problem
The `EventBus` broadcasts every `SystemEvent` to every subscriber. Subscribers then filter events manually:
```rust
if event.kind() == kind { callback(event); }
```
This means if we have 100 listeners for `ToolExecuted` and 1 listener for `MessageReceived`, publishing a `MessageReceived` event wakes up 101 tasks, 100 of which immediately sleep again.

## Solution: Topic-Based Routing
We introduce a hybrid approach:
1. **Topic Channels**: One `broadcast::Sender` per `SystemEventKind`.
2. **Global Channel**: One `broadcast::Sender` for everything.

### Data Structures
```rust
pub struct EventBus {
    global_tx: broadcast::Sender<SystemEvent>,
    topic_txs: HashMap<SystemEventKind, broadcast::Sender<SystemEvent>>,
}
```

### Routing Logic
When `publish(event)` is called:
1. Identify `kind = event.kind()`.
2. Look up `topic_txs[kind]`.
3. If found, send `event` to that channel.
4. Always send `event` to `global_tx`.

### Subscription Logic
- `subscribe(kind)` -> Connects to `topic_txs[kind]`.
- `subscribe_all()` -> Connects to `global_tx`.

## Performance Implications
- **Clone Cost**: `SystemEvent` is cloned twice if both global and topic subscribers exist. This is acceptable for the flexibility gained.
- **Locking**: `HashMap` lookup is fast (hashed by enum discriminant).
- **Concurrency**: `tokio::sync::broadcast` handles concurrency safely.

## Alternative Considered
- **Multicast Trait**: Implementing a custom trait for event routing. Rejected for now to keep it simple within `EventBus`.
- **Arc<SystemEvent>**: Wrapping events in `Arc` to reduce clone costs. Deferred until profiling shows `clone()` is a bottleneck.
