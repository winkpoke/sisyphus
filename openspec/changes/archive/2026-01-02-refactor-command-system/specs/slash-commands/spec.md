# slash-commands Specification

## MODIFIED Requirements

### Requirement: Slash Command Support
The system SHALL support executing commands triggered by a forward slash `/` at the start of a message.

The system SHALL support both:
- Built-in commands implemented in code.
- Custom commands loaded from markdown files and expanded into a normal prompt.

Built-in command execution SHALL support asynchronous operations.
Lifecycle commands SHALL signal intent via structured return values so the host applies lifecycle changes.

#### Scenario: Async Command Execution
Given the agent is running
And a command requires asynchronous operations (e.g., network calls)
When the user triggers the command
Then the system SHALL await the execution without blocking the main thread

#### Scenario: Lifecycle Management via Return Types
Given the user sends a lifecycle command like "/new" or "/exit"
When the command executes
Then it SHALL return a structured lifecycle result (e.g., `NewSession`, `Exit`)
And the host SHALL handle this result to perform the actual state change

#### Scenario: New Session Creation
Given the agent is running with an active session
When the user sends "/new"
Then the system SHALL drop the current session
And create a completely new session with a new Session ID
And the session history SHALL be empty

#### Scenario: Clear History
Given the agent is running with populated history
When the user sends "/clear"
Then the system SHALL keep the current session ID
And remove all messages from the history

#### Scenario: Built-in Command Exit
Given the agent is running
When the user sends "/exit" or "/quit"
Then the system SHALL request shutdown

#### Scenario: Custom Command Loading
Given a file `.sisyphus/command/fix.md` exists
And it contains YAML frontmatter and a prompt template
When the agent starts
Then it SHALL load `/fix` into the command registry

#### Scenario: Resilient Loading
Given a directory with multiple command files
And one file is unreadable or has invalid syntax
When the agent loads commands
Then it SHALL log a warning for the invalid file
And it SHALL successfully load the remaining valid commands

#### Scenario: Invalid Command Names
Given a command file named `my command.md` (with spaces)
When the agent loads commands
Then it SHALL skip this file or log a warning
Because command names cannot contain whitespace

#### Scenario: Custom Command Expansion
Given a custom command `/fix` is loaded with template "Fix the code in {{args}}"
When the user sends "/fix main.rs"
Then the system SHALL expand the template to "Fix the code in main.rs"
And process this expanded message as a normal user prompt to the LLM
