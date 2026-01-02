# Implementation Tasks

## 1. Context Model
- [x] 1.1 Add `Context` module under `crates/core/src/session/`
- [x] 1.2 Implement turn/step storage and invariant-enforcing append APIs
- [x] 1.3 Implement request-time rendering with injected system messages

## 2. Compaction
- [x] 2.1 Add `ContextLimits` and `TokenEstimator` abstraction
- [x] 2.2 Implement deterministic compaction by dropping oldest turns
- [x] 2.3 Define and test failure modes when pinned content exceeds budget

## 3. Session Integration
- [x] 3.1 Replace `Session.history` with `Session.context`
- [x] 3.2 Update Agent prompt building and any direct history mutations
- [x] 3.3 Add temporary compatibility helpers if required during migration

## 4. Tests and Validation
- [x] 4.1 Test tool exchange atomicity and multi-step turns
- [x] 4.2 Test compaction behavior and pinned preservation
- [x] 4.3 Run formatting, linting, and tests for affected crates
