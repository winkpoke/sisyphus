## ADDED Requirements

### Requirement: Startup Banner Display
The CLI MUST display a branded startup banner upon initialization.

#### Scenario: Launching the CLI
Given the user runs the `sisyphus` command
When the application starts
Then an orange "SISYPHUS" ASCII logo is displayed
And a summary box showing the version, current model, and working directory is shown
And a usage tip is displayed below the box.

### Requirement: Startup Banner Data Accuracy
The startup banner MUST display accurate runtime information derived from the application state.

#### Scenario: Displaying Active Configuration
Given the application is configured with model "gpt-4-turbo"
And the application version is "0.1.0"
When the banner is rendered
Then the displayed model field MUST be "gpt-4-turbo"
And the displayed version MUST match "0.1.0"
And the displayed directory MUST match the current working directory.

### Requirement: Banner Styling
The startup banner MUST use specific colors and formatting.

#### Scenario: Visual Elements
Given the banner is being rendered
Then the logo must be colored orange (approx. #E35728)
And the information box must use Unicode box-drawing characters
And the model and directory paths must be clearly legible.
