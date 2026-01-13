# workspace Specification

## Purpose
Defines the Cargo workspace structure that enforces separation of concerns across crates (common, core, provider, tools, server, cli). Ensures proper dependency flow with shared types in the common crate preventing circular dependencies.
## Requirements
### Requirement: Cargo Workspace Structure
The project MUST use a Cargo Workspace with isolated crates to enforce separation of concerns.

#### Scenario: Directory Layout
Given a fresh clone
When I list the root directory
Then I see a `Cargo.toml` defining the workspace
And I see `crates/` directory containing `common`, `core`, `provider`, `tools`, `server`, `cli`.

### Requirement: Shared Common Crate
A `common` crate MUST exist to hold shared types and prevent circular dependencies.

#### Scenario: Dependency Graph
Given the `common` crate
When I check `Cargo.toml` of `core`, `provider`, and `server`
Then they all depend on `common`.

