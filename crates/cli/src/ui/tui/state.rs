use super::transcript::{Transcript, TranscriptItemKind};
use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub enum InputMode {
    Normal,
    CommandPalette,
    Overlay,
    Selection,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AppStatus {
    Connected,
    Disconnected,
    Processing,
}

impl Default for AppStatus {
    fn default() -> Self {
        Self::Connected
    }
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
    pub permission_queue: VecDeque<(String, String, String)>, // (title, content, call_id)
}

impl OverlayState {
    pub fn new() -> Self {
        Self {
            title: String::new(),
            content: String::new(),
            scroll: 0,
            is_error: false,
            call_id: None,
            permission_queue: VecDeque::new(),
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

    pub fn enqueue_approval(&mut self, title: String, content: String, call_id: String) {
        self.permission_queue.push_back((title, content, call_id));
        if self.call_id.is_none() {
            self.show_next_approval();
        }
    }

    pub fn show_next_approval(&mut self) -> bool {
        if let Some((title, content, call_id)) = self.permission_queue.pop_front() {
            self.show_approval(title, content, call_id);
            true
        } else {
            false
        }
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
            "/debug".to_string(),
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
    pub debug_mode: bool,
    pub status: AppStatus,
    pub spinner_frame: usize,
    pub active_model: String,
    pub token_usage: String,
    pub context_title: String,
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
            debug_mode: false,
            status: AppStatus::Connected,
            spinner_frame: 0,
            active_model: "claude-3-5-sonnet".to_string(), // Default or load from config
            token_usage: "0 tokens".to_string(),
            context_title: "Transcript".to_string(),
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
        self.session_id = session_id.clone();
        self.context_title = format!("Transcript - {}", session_id);
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
        assert_eq!(palette.filtered_commands.len(), 5);
    }

    #[test]
    fn test_overlay_queue() {
        let mut overlay = OverlayState::new();

        // Enqueue first approval
        overlay.enqueue_approval(
            "Title1".to_string(),
            "Content1".to_string(),
            "id1".to_string(),
        );

        // Should be showing immediately
        assert_eq!(overlay.call_id, Some("id1".to_string()));
        assert_eq!(overlay.title, "Title1");
        assert!(overlay.permission_queue.is_empty()); // Pop happened

        // Enqueue second approval while showing first
        overlay.enqueue_approval(
            "Title2".to_string(),
            "Content2".to_string(),
            "id2".to_string(),
        );

        // Still showing first
        assert_eq!(overlay.call_id, Some("id1".to_string()));
        assert_eq!(overlay.permission_queue.len(), 1);

        // Simulate approval of first (clearing call_id is done by caller usually, but here we just call show_next)
        // Actually show_next_approval pops the next one.

        let has_next = overlay.show_next_approval();
        assert!(has_next);
        assert_eq!(overlay.call_id, Some("id2".to_string()));
        assert_eq!(overlay.title, "Title2");
        assert!(overlay.permission_queue.is_empty());

        // Try show next again
        let has_next = overlay.show_next_approval();
        assert!(!has_next);
        // call_id remains as is unless cleared by caller, but show_next only updates if queue has item
    }
}
