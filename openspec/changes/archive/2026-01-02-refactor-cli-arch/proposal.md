# CLI Architecture Refactor

## Summary
Refactor the `crates/cli` codebase to follow enterprise architectural standards, improving modularity, testability, and maintainability.

## Motivation
The current CLI implementation suffers from:
- **God Class**: `main.rs` handles too many responsibilities.
- **Coupled Composition Root**: Agent setup is hardcoded in `run_serve`.
- **Code Duplication**: `run_chat` and `run_attach` share logic.
- **Deep Nesting**: Hard to follow control flow.

## Goal
Transform the monolithic `main.rs` into a modular structure with clear separation of concerns, enabling easier testing and future extensions.
