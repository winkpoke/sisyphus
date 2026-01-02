use super::transcript::{Transcript, TranscriptItemKind};

#[derive(Debug, Clone)]
pub struct TuiState {
    pub transcript: Transcript,
    pub input_buffer: String,
    pub session_id: String,
}

impl TuiState {
    pub fn new(session_id: String) -> Self {
        Self {
            transcript: Transcript::new(),
            input_buffer: String::new(),
            session_id,
        }
    }

    pub fn handle_char(&mut self, c: char) {
        self.input_buffer.push(c);
    }

    pub fn handle_backspace(&mut self) {
        self.input_buffer.pop();
    }

    pub fn get_input_and_clear(&mut self) -> Option<String> {
        if self.input_buffer.trim().is_empty() {
            return None;
        }
        Some(self.input_buffer.drain(..).collect())
    }

    pub fn add_message(&mut self, kind: TranscriptItemKind, msg: String) {
        self.transcript.add_message(kind, msg);
    }

    pub fn update_session_id(&mut self, session_id: String) {
        self.session_id = session_id;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_handling() {
        let mut state = TuiState::new("sess-1".to_string());
        state.handle_char('a');
        state.handle_char('b');
        assert_eq!(state.input_buffer, "ab");

        state.handle_backspace();
        assert_eq!(state.input_buffer, "a");

        let input = state.get_input_and_clear();
        assert_eq!(input, Some("a".to_string()));
        assert!(state.input_buffer.is_empty());
    }

    #[test]
    fn test_session_update() {
        let mut state = TuiState::new("sess-1".to_string());
        state.update_session_id("sess-2".to_string());
        assert_eq!(state.session_id, "sess-2");
    }
}
