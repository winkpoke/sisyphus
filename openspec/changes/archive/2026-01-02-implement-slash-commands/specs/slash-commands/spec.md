## ADDED Requirements

### Requirement: Slash Command Support
The system SHALL support executing commands triggered by a forward slash `/` at the start of a message.

#### Scenario: Built-in Command Execution
Given the agent is running
And a built-in command `/help` is registered
When the user sends "/help"
Then the system should execute the help handler
And return the help text directly without invoking the LLM

#### Scenario: Built-in Command Exit
Given the agent is running
When the user sends "/exit" or "/quit"
Then the system should publish a shutdown event
And the system should exit the program completely

#### Scenario: Built-in Command New Session
Given the agent is running
When the user sends "/new"
Then the system should create a new session

#### Scenario: Custom Command Loading
Given a file `.sisyphus/command/fix.md` exists
And it contains YAML frontmatter and a prompt template
When the agent starts
Then it should load `/fix` into the command registry

#### Scenario: Resilient Loading
Given a directory with multiple command files
And one file is unreadable or has invalid syntax
When the agent loads commands
Then it should log a warning for the invalid file
And it should successfully load the remaining valid commands

#### Scenario: Invalid Command Names
Given a command file named `my command.md` (with spaces)
When the agent loads commands
Then it should skip this file or log a warning
Because command names cannot contain whitespace

#### Scenario: Custom Command Expansion
Given a custom command `/fix` is loaded with template "Fix the code in {{args}}"
When the user sends "/fix main.rs"
Then the system should expand the template to "Fix the code in main.rs"
And process this expanded message as a normal user prompt to the LLM
