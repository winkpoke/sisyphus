# Change: Update OpenAI streaming parser for SSE chunk correctness

## Why
The current OpenAI streaming implementation assumes each network chunk is a complete SSE `data:` frame. This is a critical defect that causes data corruption, dropped messages, or crashes when:
-   Network packets split SSE lines.
-   Multi-byte characters (emojis, CJK) are split across chunk boundaries.
-   Malicious/malformed streams send excessive data without newlines (DoS risk).

## What Changes
-   **Introduce `SSEParser`:** A shared, robust SSE parsing utility in `crates/provider/src/sse.rs`.
    -   Handles arbitrary chunk boundaries.
    -   Buffers raw bytes to ensure correct UTF-8 decoding.
    -   Enforces safety limits (max line length).
-   **Update `OpenAIProvider`:** Refactor `stream()` to use `SSEParser` instead of ad-hoc string splitting.
-   **Compliance:** Properly handle SSE comments (ignore) and standard fields (`event`, `id`).

## Impact
-   **Affected specs:** `llm-provider`
-   **Affected code:**
    -   `crates/provider/src/openai.rs` (Refactor)
    -   `crates/provider/src/sse.rs` (New)
-   **Security:** Mitigates OOM DoS risks via buffer limits.
-   **Reliability:** Guarantees zero data loss from network fragmentation.
