# Implementation Tasks

## 1. Context Model
- [ ] 1.1 Add `Context` module under `crates/core/src/session/`
- [ ] 1.2 Implement turn/step storage and invariant-enforcing append APIs
- [ ] 1.3 Implement request-time rendering with injected system messages

## 2. Compaction
- [ ] 2.1 Add `ContextLimits` and `TokenEstimator` abstraction
- [ ] 2.2 Implement deterministic compaction by dropping oldest turns
- [ ] 2.3 Define and test failure modes when pinned content exceeds budget

## 3. Session Integration
- [ ] 3.1 Replace `Session.history` with `Session.context`
- [ ] 3.2 Update Agent prompt building and any direct history mutations
- [ ] 3.3 Add temporary compatibility helpers if required during migration

## 4. Tests and Validation
- [ ] 4.1 Test tool exchange atomicity and multi-step turns
- [ ] 4.2 Test compaction behavior and pinned preservation
- [ ] 4.3 Run formatting, linting, and tests for affected crates
