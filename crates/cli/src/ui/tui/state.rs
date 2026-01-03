use super::transcript::{Transcript, TranscriptItemKind};

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    CommandPalette,
    Overlay,
    Selection,
}

#[derive(Debug, Clone)]
pub struct SelectionState {
    pub selected_message_index: Option<usize>,
}

impl SelectionState {
    pub fn new() -> Self {
        Self {
            selected_message_index: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct OverlayState {
    pub title: String,
    pub content: String,
    pub scroll: u16,
    pub is_error: bool,
    pub call_id: Option<String>,
}

impl OverlayState {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            content: String::new(),
            scroll: 0,
            is_error: false,
            call_id: None,
        }
    }

    pub fn show(&mut self, title: String, content: String, is_error: bool) {
        self.title = title;
        self.content = content;
        self.scroll = 0;
        self.is_error = is_error;
        self.call_id = None;
    }

    pub fn show_approval(&mut self, title: String, content: String, call_id: String) {
        self.title = title;
        self.content = content;
        self.scroll = 0;
        self.is_error = false;
        self.call_id = Some(call_id);
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }
}

#[derive(Debug, Clone)]
pub struct CommandPaletteState {
    pub selected_index: usize,
    pub input: String,
    pub commands: Vec<String>,
    pub filtered_commands: Vec<String>,
}

impl CommandPaletteState {
    pub fn new() -> Self {
        let commands = vec![
            "/quit".to_string(),
            "/exit".to_string(),
            "/help".to_string(),
            "/clear".to_string(),
        ];
        Self {
            selected_index: 0,
            input: String::new(),
            filtered_commands: commands.clone(),
            commands,
        }
    }

    pub fn reset(&mut self) {
        self.input.clear();
        self.update_filter();
    }

    pub fn update_filter(&mut self) {
        if self.input.is_empty() {
            self.filtered_commands = self.commands.clone();
        } else {
            self.filtered_commands = self
                .commands
                .iter()
                .filter(|c| c.starts_with(&self.input))
                .cloned()
                .collect();
        }
        self.selected_index = 0;
    }

    pub fn select_next(&mut self) {
        if !self.filtered_commands.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.filtered_commands.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.filtered_commands.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.filtered_commands.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct TuiState {
    pub transcript: Transcript,
    pub input_buffer: String,
    pub session_id: String,
    pub mode: InputMode,
    pub command_palette: CommandPaletteState,
    pub overlay: OverlayState,
    pub selection: SelectionState,
}

impl TuiState {
    pub fn new(session_id: String) -> Self {
        Self {
            transcript: Transcript::new(),
            input_buffer: String::new(),
            session_id,
            mode: InputMode::Normal,
            command_palette: CommandPaletteState::new(),
            overlay: OverlayState::new(),
            selection: SelectionState::new(),
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

    #[test]
    fn test_command_palette() {
        let mut palette = CommandPaletteState::new();
        assert_eq!(palette.selected_index, 0);

        palette.select_next();
        assert_eq!(palette.selected_index, 1);

        palette.input.push_str("/h");
        palette.update_filter();
        assert_eq!(palette.filtered_commands.len(), 1); // /help
        assert_eq!(palette.filtered_commands[0], "/help");

        palette.reset();
        assert!(palette.input.is_empty());
        assert_eq!(palette.filtered_commands.len(), 4);
    }
}
