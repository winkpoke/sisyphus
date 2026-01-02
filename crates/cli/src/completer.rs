use reedline::{Completer, Suggestion, Span};
use sisyphus_core::command::CommandInfo;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct CommandCompleter {
    commands: Arc<Mutex<Vec<CommandInfo>>>,
}

impl CommandCompleter {
    pub fn new(commands: Vec<CommandInfo>) -> Self {
        Self {
            commands: Arc::new(Mutex::new(commands)),
        }
    }

    pub fn update_commands(&self, commands: Vec<CommandInfo>) {
        let mut guard = self.commands.lock().unwrap();
        *guard = commands;
    }
}

impl Completer for CommandCompleter {
    fn complete(&mut self, line: &str, pos: usize) -> Vec<Suggestion> {
        if !line.starts_with('/') {
            return vec![];
        }

        let guard = self.commands.lock().unwrap();
        guard.iter()
            .filter(|cmd| cmd.name.starts_with(line))
            .map(|cmd| Suggestion {
                value: cmd.name.clone(),
                description: Some(cmd.description.clone()),
                extra: None,
                span: Span::new(0, pos),
                append_whitespace: true,
            })
            .collect()
    }
}
