use anyhow::{anyhow, Result};
use bytes::{Bytes, BytesMut};
use futures::{ready, Stream};
use std::pin::Pin;
use std::task::{Context, Poll};

const MAX_LINE_LENGTH: usize = 1024 * 1024; // 1MB

#[derive(Debug, Clone, PartialEq)]
pub struct SSEEvent {
    pub event: String,
    pub data: String,
    pub id: Option<String>,
}

impl Default for SSEEvent {
    fn default() -> Self {
        Self {
            event: "message".to_string(),
            data: String::new(),
            id: None,
        }
    }
}

pub struct SSEParser<S> {
    stream: S,
    buffer: BytesMut,
    current_event: SSEEvent,
}

impl<S> SSEParser<S> {
    pub fn new(stream: S) -> Self {
        Self {
            stream,
            buffer: BytesMut::new(),
            current_event: SSEEvent::default(),
        }
    }

    fn parse_buffer(&mut self) -> Result<Option<SSEEvent>> {
        loop {
            // Find newline
            let newline_pos = self.buffer.as_ref().iter().position(|&b| b == b'\n');

            if let Some(pos) = newline_pos {
                // Check max length
                if pos > MAX_LINE_LENGTH {
                    return Err(anyhow!("Line too long"));
                }

                // Extract line
                let mut line_bytes = self.buffer.split_to(pos + 1); // includes \n

                // Remove trailing \n
                line_bytes.truncate(line_bytes.len() - 1);

                // Remove optional \r
                if line_bytes.ends_with(b"\r") {
                    line_bytes.truncate(line_bytes.len() - 1);
                }

                // Decode UTF-8
                let line_str = std::str::from_utf8(&line_bytes)?;

                if line_str.is_empty() {
                    // Dispatch event
                    if !self.current_event.data.is_empty() {
                        let event = std::mem::take(&mut self.current_event);
                        return Ok(Some(event));
                    } else {
                        // Reset event state but don't emit if data is empty
                        self.current_event = SSEEvent::default();
                        continue;
                    }
                }

                // Check for comment
                if line_str.starts_with(':') {
                    continue;
                }

                // Parse field
                if let Some((field, value)) = line_str.split_once(':') {
                    let value = value.strip_prefix(' ').unwrap_or(value);
                    match field {
                        "event" => self.current_event.event = value.to_string(),
                        "data" => {
                            if !self.current_event.data.is_empty() {
                                self.current_event.data.push('\n');
                            }
                            self.current_event.data.push_str(value);
                        }
                        "id" => self.current_event.id = Some(value.to_string()),
                        _ => {} // Ignore other fields
                    }
                } else {
                    // Field without value
                    let field = line_str;
                    match field {
                        "event" => self.current_event.event = "".to_string(),
                        "data" => {
                            if !self.current_event.data.is_empty() {
                                self.current_event.data.push('\n');
                            }
                        }
                        "id" => self.current_event.id = Some("".to_string()),
                        _ => {}
                    }
                }
            } else {
                // No newline found
                if self.buffer.len() > MAX_LINE_LENGTH {
                    return Err(anyhow!("Line too long"));
                }
                return Ok(None);
            }
        }
    }
}

impl<S, E> Stream for SSEParser<S>
where
    S: Stream<Item = Result<Bytes, E>> + Unpin,
    E: std::error::Error + Send + Sync + 'static,
{
    type Item = Result<SSEEvent>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            // 1. Process existing buffer
            match self.parse_buffer() {
                Ok(Some(event)) => return Poll::Ready(Some(Ok(event))),
                Ok(None) => {} // Need more data
                Err(e) => return Poll::Ready(Some(Err(e))),
            }

            // 2. Pull more data
            match ready!(Pin::new(&mut self.stream).poll_next(cx)) {
                Some(Ok(bytes)) => {
                    self.buffer.extend_from_slice(&bytes);
                }
                Some(Err(e)) => return Poll::Ready(Some(Err(anyhow!(e)))),
                None => {
                    // Handle EOF: if buffer has data, try to process it as a final line
                    if !self.buffer.is_empty() {
                        // Ensure we have a newline to trigger parsing
                        if !self.buffer.ends_with(b"\n") {
                            self.buffer.extend_from_slice(b"\n");
                        }
                        // Try parsing one last time
                        match self.parse_buffer() {
                            Ok(Some(event)) => return Poll::Ready(Some(Ok(event))),
                            Ok(None) => return Poll::Ready(None), // Should not happen if we added \n
                            Err(e) => return Poll::Ready(Some(Err(e))),
                        }
                    }
                    return Poll::Ready(None);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::{stream, StreamExt};

    #[tokio::test]
    async fn test_simple_sse() {
        let input: Vec<Result<Bytes, std::io::Error>> = vec![
            Ok(Bytes::from("data: hello\n\n")),
            Ok(Bytes::from("data: world\n\n")),
        ];
        let stream = stream::iter(input);
        let mut parser = SSEParser::new(stream);

        let event1 = parser.next().await.unwrap().unwrap();
        assert_eq!(event1.data, "hello");

        let event2 = parser.next().await.unwrap().unwrap();
        assert_eq!(event2.data, "world");

        assert!(parser.next().await.is_none());
    }

    #[tokio::test]
    async fn test_split_packets() {
        let input: Vec<Result<Bytes, std::io::Error>> =
            vec![Ok(Bytes::from("data: hel")), Ok(Bytes::from("lo\n\n"))];
        let stream = stream::iter(input);
        let mut parser = SSEParser::new(stream);

        let event = parser.next().await.unwrap().unwrap();
        assert_eq!(event.data, "hello");
    }

    #[tokio::test]
    async fn test_multiline_data() {
        let input: Vec<Result<Bytes, std::io::Error>> =
            vec![Ok(Bytes::from("data: line1\ndata: line2\n\n"))];
        let stream = stream::iter(input);
        let mut parser = SSEParser::new(stream);

        let event = parser.next().await.unwrap().unwrap();
        assert_eq!(event.data, "line1\nline2");
    }

    #[tokio::test]
    async fn test_split_utf8() {
        // Snowman ☃ is 0xE2 0x98 0x83
        let chunks: Vec<Result<Bytes, std::io::Error>> = vec![
            Ok(Bytes::from_static(b"data: ")),
            Ok(Bytes::from(vec![0xe2, 0x98])),
            Ok(Bytes::from(vec![0x83])),
            Ok(Bytes::from_static(b"\n\n")),
        ];

        let stream = stream::iter(chunks);
        let mut parser = SSEParser::new(stream);

        let event = parser.next().await.unwrap().unwrap();
        assert_eq!(event.data, "☃");
    }

    #[tokio::test]
    async fn test_dos_protection() {
        let input: Vec<Result<Bytes, std::io::Error>> =
            vec![Ok(Bytes::from("a".repeat(MAX_LINE_LENGTH + 1)))];
        let stream = stream::iter(input);
        let mut parser = SSEParser::new(stream);

        let result = parser.next().await.unwrap();
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Line too long");
    }

    #[tokio::test]
    async fn test_ignore_comments() {
        let input: Vec<Result<Bytes, std::io::Error>> =
            vec![Ok(Bytes::from(": comment\ndata: hello\n\n"))];
        let stream = stream::iter(input);
        let mut parser = SSEParser::new(stream);

        let event = parser.next().await.unwrap().unwrap();
        assert_eq!(event.data, "hello");
    }
}
