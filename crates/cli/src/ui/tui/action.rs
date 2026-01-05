use client::ChatResponse;
use crossterm::event::KeyEvent;
use common::bus::SystemEvent;
use sisyphus_core::command::CommandOutcome;
use sisyphus_core::session::Session;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Key(KeyEvent),
    MessageSent(String),
    ResponseReceived(ChatResponse),
    SessionCreated(Session),
    SystemEvent(SystemEvent),
    CommandResult(Box<CommandOutcome>),
    ToggleDebug,
    ClearHistory,
    Error(String),
    Init,
    Quit,
}
