use client::ChatResponse;
use crossterm::event::KeyEvent;
use common::bus::SystemEvent;
use sisyphus_core::command::CommandOutcome;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum Action {
    Tick,
    Render,
    Resize(u16, u16),
    Key(KeyEvent),
    MessageSent(String),
    ResponseReceived(ChatResponse),
    SystemEvent(SystemEvent),
    CommandResult(Box<CommandOutcome>),
    Error(String),
    Init,
    Quit,
}
