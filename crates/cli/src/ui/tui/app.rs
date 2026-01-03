use super::state::TuiState;
use super::theme::Theme;
use arboard::Clipboard;
use common::bus::EventBus;
use sisyphus_core::command::{
    builtins::{
        ClearHistoryCommand, DebugCommand, ExitCommand, HelpCommand, NewSessionCommand, QuitCommand,
    },
    CommandRegistry,
};
use std::sync::Arc;

pub struct App {
    pub state: TuiState,
    pub should_quit: bool,
    pub clipboard: Option<Clipboard>,
    pub theme: Theme,
    pub registry: Arc<CommandRegistry>,
    pub event_bus: Arc<EventBus>,
}

impl App {
    pub fn new(session_id: String) -> Self {
        let mut registry = CommandRegistry::new();
        registry.register_builtin(Box::new(HelpCommand));
        registry.register_builtin(Box::new(ExitCommand));
        registry.register_builtin(Box::new(QuitCommand));
        registry.register_builtin(Box::new(ClearHistoryCommand));
        registry.register_builtin(Box::new(NewSessionCommand));
        registry.register_builtin(Box::new(DebugCommand));

        let commands = registry.list().iter().map(|c| c.name.clone()).collect();

        Self {
            state: TuiState::new(session_id, commands),
            should_quit: false,
            clipboard: Clipboard::new().ok(),
            theme: Theme::default(),
            registry: Arc::new(registry),
            event_bus: Arc::new(EventBus::new(1)),
        }
    }

    pub fn tick(&mut self) {
        self.state.spinner_frame = self.state.spinner_frame.wrapping_add(1);
    }

    pub fn quit(&mut self) {
        self.should_quit = true;
    }
}
