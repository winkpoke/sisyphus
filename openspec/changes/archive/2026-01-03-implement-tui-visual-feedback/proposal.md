# Proposal: Implement TUI Visual Polish and Feedback

## Summary
Enhance the CLI TUI with a "pro-tool" aesthetic through a semantic color palette, loading state indicators, and visual feedback for actions like copying to clipboard.

## Motivation
The current TUI uses basic, hardcoded colors (Cyan, Green, Yellow, Red) and lacks visual feedback for background actions or states. To improve user experience and perceived intelligence:
1.  **Color Theory**: Adopt a sophisticated palette (Soft Blue, Lavender/Mint, Muted Grey) to distinguish roles and reduce visual fatigue.
2.  **Visual Feedback**: Provide immediate confirmation for actions (e.g., "Copied!" toast) and indicate active processing (e.g., streaming cursor/spinner).

## Proposed Changes
1.  **Theming System**: Introduce a centralized `Theme` struct to manage semantic colors, replacing hardcoded values.
2.  **Toast Notifications**: Add a transient overlay system to display temporary success/info messages (e.g., clipboard copy confirmation).
3.  **Loading Indicators**: Implement a visual cursor or spinner at the end of streaming assistant messages to indicate active generation.

## Alternatives Considered
-   **Hardcoding new colors**: Rejected as it maintains the maintenance burden and makes future theming difficult.
-   **Status bar only feedback**: Rejected as it is too subtle for immediate user actions like "Copy".
