## ADDED Slash Command Support

The system shall support executing commands triggered by a forward slash `/` at the start of a message.

#### Scenario: Built-in Command Execution
Given the agent is running
And a built-in command `/help` is registered
When the user sends "/help"
Then the system should execute the help handler
And return the help text directly without invoking the LLM

#### Scenario: Custom Command Loading
Given a file `.opencode/command/fix.md` exists
And it contains YAML frontmatter and a prompt template
When the agent starts
Then it should load `/fix` into the command registry

#### Scenario: Custom Command Expansion
Given a custom command `/fix` is loaded with template "Fix the code in {{args}}"
When the user sends "/fix main.rs"
Then the system should expand the template to "Fix the code in main.rs"
And process this expanded message as a normal user prompt to the LLM
