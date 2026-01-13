use client::AgentResponse;
use std::time::{Duration, Instant};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum InputMode {
    Normal,
    CommandPalette,
    Selection,
    Overlay,
    AgentSelection,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AppStatus {
    Connected,
    Processing,
    Disconnected,
}

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
    pub timestamp: Instant,
    pub is_streaming: bool,
}

#[derive(Debug, Default)]
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

    pub fn clear(&mut self) {
        self.items.clear();
        self.scroll_offset = 0;
        self.stick_to_bottom = true;
    }
}

#[derive(Debug)]
pub struct CommandPaletteState {
    pub input: String,
    pub selected_index: usize,
    pub commands: Vec<String>,
    pub commands_lower: Vec<String>,
    pub filtered_commands: Vec<String>,
}

impl CommandPaletteState {
    pub fn new(commands: Vec<String>) -> Self {
        let commands_lower = commands.iter().map(|c| c.to_lowercase()).collect();
        Self {
            input: String::new(),
            selected_index: 0,
            filtered_commands: commands.clone(),
            commands,
            commands_lower,
        }
    }

    pub fn reset(&mut self) {
        self.input.clear();
        self.selected_index = 0;
        self.update_filter();
    }

    pub fn update_filter(&mut self) {
        if self.input.is_empty() || self.input == "/" {
            self.filtered_commands = self.commands.clone();
        } else {
            let query = self.input.to_lowercase();
            self.filtered_commands = self
                .commands
                .iter()
                .zip(self.commands_lower.iter())
                .filter(|(_, lower)| lower.contains(&query))
                .map(|(original, _)| original.clone())
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

#[derive(Debug, Default)]
pub struct OverlayState {
    pub call_id: Option<String>,
    pub title: String,
    pub content: String,
    pub scroll: u16,
    pub is_error: bool,
}

impl OverlayState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enqueue_approval(&mut self, title: String, content: String, call_id: String) {
        self.title = title;
        self.content = content;
        self.call_id = Some(call_id);
        self.scroll = 0;
        self.is_error = false;
    }

    pub fn show_next_approval(&mut self) -> bool {
        false
    }

    pub fn scroll_down(&mut self) {
        self.scroll = self.scroll.saturating_add(1);
    }

    pub fn scroll_up(&mut self) {
        self.scroll = self.scroll.saturating_sub(1);
    }
}

#[derive(Debug, Default)]
pub struct SelectionState {
    pub selected_message_index: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct AgentSelectionState {
    pub agents: Vec<AgentResponse>,
    pub selected_index: usize,
}

impl AgentSelectionState {
    pub fn new(agents: Vec<AgentResponse>) -> Self {
        Self {
            agents,
            selected_index: 0,
        }
    }

    pub fn select_next(&mut self) {
        if !self.agents.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.agents.len();
        }
    }

    pub fn select_prev(&mut self) {
        if !self.agents.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.agents.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn get_selected(&self) -> Option<&AgentResponse> {
        self.agents.get(self.selected_index)
    }
}

#[derive(Debug)]
pub struct Toast {
    pub message: String,
    pub kind: ToastKind,
    pub expires_at: Instant,
}

impl Toast {
    pub fn new(message: String, kind: ToastKind, duration: Duration) -> Self {
        Self {
            message,
            kind,
            expires_at: Instant::now() + duration,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ToastKind {
    Info,
    Success,
    Error,
}

pub struct TuiState {
    pub session_id: String,
    pub active_model: String,
    pub token_usage: String,
    pub transcript: Transcript,
    pub input_buffer: String,
    pub mode: InputMode,
    pub command_palette: CommandPaletteState,
    pub overlay: OverlayState,
    pub selection: SelectionState,
    pub agent_selection: Option<AgentSelectionState>,
    pub debug_mode: bool,
    pub status: AppStatus,
    pub spinner_frame: usize,
    pub toast: Option<Toast>,
    pub context_title: String,
}

impl TuiState {
    pub fn new(session_id: String, commands: Vec<String>) -> Self {
        Self {
            session_id,
            active_model: "Loading...".to_string(),
            token_usage: String::new(),
            transcript: Transcript::new(),
            input_buffer: String::new(),
            mode: InputMode::Normal,
            command_palette: CommandPaletteState::new(commands),
            overlay: OverlayState::new(),
            selection: SelectionState::default(),
            agent_selection: None,
            debug_mode: false,
            status: AppStatus::Disconnected,
            spinner_frame: 0,
            toast: None,
            context_title: "Context".to_string(),
        }
    }

    pub fn add_message(&mut self, kind: TranscriptItemKind, content: String) {
        self.transcript.items.push(TranscriptItem {
            kind,
            content,
            timestamp: Instant::now(),
            is_streaming: false,
        });
    }

    pub fn update_session_id(&mut self, session_id: String) {
        self.session_id = session_id;
    }

    pub fn update_commands(&mut self, commands: Vec<String>) {
        self.command_palette = CommandPaletteState::new(commands);
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
        let input = self.input_buffer.clone();
        self.input_buffer.clear();
        Some(input)
    }
}
