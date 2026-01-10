## 1. Data Structure Updates
- [ ] 1.1 Add PermissionMode enum to config.rs with 5 variants (Default, AcceptEdits, DontAsk, BypassPermissions, Plan)
- [ ] 1.2 Add allow/ask/deny fields to AgentPermissions struct (Vec<String>)
- [ ] 1.3 Update Default implementation for AgentPermissions to initialize new fields
- [ ] 1.4 Ensure backward compatibility: old edit/bash/skill fields remain functional

## 2. Permission Rule Parser
- [ ] 2.1 Implement rule parser to extract tool name and pattern from rule strings (e.g., "Bash(git:*)")
- [ ] 2.2 Implement glob pattern matching for file operations (Read/Write/Edit with patterns like "./src/**")
- [ ] 2.3 Implement prefix matching for Bash commands (e.g., "Bash(git:*)" matches "git push")
- [ ] 2.4 Add unit tests for rule parsing and pattern matching

## 3. Permission Resolution Engine
- [ ] 3.1 Implement resolution order: deny → allow → ask → mode fallback
- [ ] 3.2 Implement mode-based fallback behavior for all 5 PermissionMode variants
- [ ] 3.3 Add backward compatibility layer to check legacy edit/bash/skill fields when rules are empty
- [ ] 3.4 Add unit tests for resolution engine with all modes and rule combinations

## 4. Agent Integration
- [ ] 4.1 Update Agent::execute_tool to use new permission resolution
- [ ] 4.2 Update PermissionRequest event to include matched rule and current mode
- [ ] 4.3 Ensure subagent permission mode override support
- [ ] 4.4 Add integration tests for agent with new permission system

## 5. Migration and Documentation
- [ ] 5.1 Add deprecation warnings for legacy edit/bash/skill fields when rules are used
- [ ] 5.2 Create migration guide showing conversion examples
- [ ] 5.3 Update existing tests to use new permission system
- [ ] 5.4 Add comprehensive test coverage for edge cases

## 6. Validation
- [ ] 6.1 Run existing test suite to ensure backward compatibility
- [ ] 6.2 Verify all new unit and integration tests pass
- [ ] 6.3 Test migration scenarios from old to new config format
- [ ] 6.4 Validate permission modes work correctly with all tool types
