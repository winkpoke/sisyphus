# Implementation Tasks

- [ ] **Core Context Model**
  - [ ] Add `Context` module under `crates/core/src/session/`.
  - [ ] Implement context block storage and append APIs with invariant checks.
  - [ ] Implement `render` to produce a provider-ready linear message list.

- [ ] **Compaction (Minimal)**
  - [ ] Add `ContextLimits` and `TokenEstimator` abstraction.
  - [ ] Implement an initial heuristic token estimator.
  - [ ] Implement a compaction policy that drops oldest non-pinned blocks.

- [ ] **Session Refactor**
  - [ ] Replace `Session.history` with `Session.context`.
  - [ ] Update `SessionManager` and call sites to use the new API.

- [ ] **Tests and Validation**
  - [ ] Add unit tests for tool exchange atomicity during compaction.
  - [ ] Add unit tests for pinned message preservation.
  - [ ] Run formatting, linting, and tests for the affected crates.

