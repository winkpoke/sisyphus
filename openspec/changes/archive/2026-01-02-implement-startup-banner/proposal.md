# Proposal: Implement Codex-like Startup Banner

## Background
The user wants Sisyphus to have a polished startup experience similar to OpenAI Codex. This includes a distinct ASCII art logo, version information, configuration summary (model, directory), and helpful tips, all presented in a visually appealing format.

## Goal
Implement a high-fidelity startup screen in the CLI that displays:
- "SISYPHUS" ASCII logo in orange.
- Version number and current working directory.
- Current LLM model configuration.
- A "box" layout containing this information.
- A random or static tip below the box.

## Scope
- **CLI**: Add `colored` dependency, implement `banner` module, and integrate into `main.rs`.
- **UI**: Ensure the design matches the provided visual reference (orange logo, grey box, clean typography).

## Risks
- **Terminal Compatibility**: Colors and box-drawing characters might not render correctly in all terminals (e.g., cmd.exe vs PowerShell vs Windows Terminal). We will target modern terminals (ANSI support).
