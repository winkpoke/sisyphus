# Design: Shared SSE Parser Utility

## Problem
The current OpenAI provider implements ad-hoc SSE parsing that assumes 1:1 mapping between network chunks and SSE frames. This is brittle and fails when:
1.  TCP packets split a single SSE line.
2.  Multi-byte UTF-8 characters are split across chunks.
3.  Multiple SSE frames arrive in a single chunk.

Additionally, embedding this logic in `OpenAIProvider` violates DRY, as other providers (Anthropic, Mistral) use SSE.

## Solution
Introduce a generic, robust `SSEParser` utility in `crates/provider/src/sse.rs`.

### Architecture

#### `SSEParser` Struct
A stateful parser that consumes a stream of `Bytes` and yields `Result<SSEEvent>`.

```rust
pub struct SSEParser {
    buffer: BytesMut,
    // Configuration
    max_line_length: usize, // e.g., 1MB
}

pub struct SSEEvent {
    pub event: String, // Defaults to "message"
    pub data: String,
    pub id: Option<String>,
}
```

### Buffering Strategy
-   **Input:** `Bytes` (raw `u8`).
-   **Internal Buffer:** `BytesMut` (efficient dynamic buffer).
-   **Decoding:**
    -   Scan buffer for `\n` or `\r\n`.
    -   If found, extract the slice *bytes*.
    -   **Validate UTF-8** only on the extracted line.
    -   Parse the line (`field: value`).
    -   If valid field, update current event state.
    -   If empty line, emit the accumulated event.
-   **Safety:**
    -   If buffer size > `max_line_length` without a newline, return `Err(LineTooLong)`.
    -   This prevents OOM attacks from malicious servers sending endless streams without newlines.

### Integration
`OpenAIProvider::stream` will:
1.  Get the raw `reqwest::Response`.
2.  Convert `response.bytes_stream()` into an `SSEParser` stream.
3.  Map `SSEEvent` to `Result<String>` (content delta).
    -   Ignore `event` types other than default/message (unless OpenAI uses others).
    -   Parse `data` as JSON.
    -   Handle `[DONE]`.

### Alternatives Considered
-   **Using an external crate (e.g., `eventsource-stream`):**
    -   *Pros:* Less code to maintain.
    -   *Cons:* Adds dependency; might not expose low-level control over buffering/limits needed for our specific constraints.
    -   *Decision:* Implement a lightweight internal parser (~100 lines) to ensure strict control over memory usage and dependencies, as this is a core reliability component.

## Security Implications
-   **DoS Protection:** Enforced via `max_line_length`.
-   **Data Integrity:** Byte-level buffering prevents UTF-8 replacement characters from corrupting output.
