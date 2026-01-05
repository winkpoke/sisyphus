# Tasks: Optimize Event Bus

- [x] Refactor `EventBus` struct to hold `topic_txs` map <!-- id: 0 -->
- [x] Update `EventBus::new` to initialize topic channels <!-- id: 1 -->
- [x] Update `EventBus::publish` to route events to both topic and global channels <!-- id: 2 -->
- [x] Update `EventBus::subscribe` and `subscribe_once` to use topic channels <!-- id: 3 -->
- [x] Verify `subscribe_all` still works via global channel <!-- id: 4 -->
- [x] Fix compilation errors in `logging.rs` (update usage) <!-- id: 5 -->
- [x] Add unit tests for filtering and global subscription <!-- id: 6 -->
