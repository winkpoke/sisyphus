#[derive(Debug, Clone, PartialEq)]
pub enum TranscriptItemKind {
    User,
    Assistant,
    System,
    Error,
}

#[derive(Debug, Clone)]
pub struct TranscriptItem {
    pub kind: TranscriptItemKind,
    pub content: String,
    pub is_streaming: bool,
}

#[derive(Debug, Clone)]
pub struct Transcript {
    pub items: Vec<TranscriptItem>,
    pub scroll_offset: u16,
    pub stick_to_bottom: bool,
}

impl Transcript {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            scroll_offset: 0,
            stick_to_bottom: true,
        }
    }

    pub fn add_message(&mut self, kind: TranscriptItemKind, content: String) {
        self.items.push(TranscriptItem {
            kind,
            content,
            is_streaming: false,
        });
    }

    pub fn start_streaming(&mut self, kind: TranscriptItemKind) {
        self.items.push(TranscriptItem {
            kind,
            content: String::new(),
            is_streaming: true,
        });
    }

    pub fn append_streaming(&mut self, content: &str) {
        if let Some(last) = self.items.last_mut() {
            if last.is_streaming {
                last.content.push_str(content);
            }
        }
    }

    pub fn finish_streaming(&mut self) {
        if let Some(last) = self.items.last_mut() {
            last.is_streaming = false;
        }
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.scroll_offset = 0;
        self.stick_to_bottom = true;
    }
}

impl Default for Transcript {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_streaming() {
        let mut transcript = Transcript::new();
        transcript.start_streaming(TranscriptItemKind::Assistant);
        transcript.append_streaming("Hello");
        transcript.append_streaming(" World");

        assert_eq!(transcript.items.len(), 1);
        assert_eq!(transcript.items[0].content, "Hello World");
        assert!(transcript.items[0].is_streaming);

        transcript.finish_streaming();
        assert!(!transcript.items[0].is_streaming);
    }

    #[test]
    fn test_add_message() {
        let mut transcript = Transcript::new();
        transcript.add_message(TranscriptItemKind::User, "Hi".into());

        assert_eq!(transcript.items.len(), 1);
        assert_eq!(transcript.items[0].kind, TranscriptItemKind::User);
    }
}
