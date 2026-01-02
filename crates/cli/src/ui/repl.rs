use anyhow::Result;
use client::Client;
use reedline::{
    Color, ColumnarMenu, DefaultPrompt, DefaultPromptSegment, EditCommand, Emacs, KeyCode,
    KeyModifiers, Reedline, ReedlineEvent, ReedlineMenu, Signal, default_emacs_keybindings,
};
use rust_i18n::t;
use tokio::sync::mpsc::Receiver;

use super::completer::CommandCompleter;

pub struct Repl {
    client: Client,
    session_id: String,
    shutdown_rx: Receiver<()>,
}

impl Repl {
    pub fn new(client: Client, session_id: String, shutdown_rx: Receiver<()>) -> Self {
        Self {
            client,
            session_id,
            shutdown_rx,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // Fetch commands for completion
        let commands = self.client.get_commands().await.unwrap_or_default();
        let completer = Box::new(CommandCompleter::new(commands));

        // Configure Menu
        let completion_menu = Box::new(ColumnarMenu::default().with_name("completion_menu"));

        // Configure Keybindings
        let mut keybindings = default_emacs_keybindings();
        keybindings.add_binding(
            KeyModifiers::NONE,
            KeyCode::Char('/'),
            ReedlineEvent::Multiple(vec![
                ReedlineEvent::Edit(vec![EditCommand::InsertChar('/')]),
                ReedlineEvent::Menu("completion_menu".to_string()),
            ]),
        );

        let mut line_editor = Reedline::create()
            .with_completer(completer)
            .with_menu(ReedlineMenu::EngineCompleter(completion_menu))
            .with_edit_mode(Box::new(Emacs::new(keybindings)));

        let prompt = DefaultPrompt::new(
            DefaultPromptSegment::Basic("›".to_string()),
            DefaultPromptSegment::Empty,
        );

        println!("{}", t!("type_exit"));
        loop {
            if let Ok(_) = self.shutdown_rx.try_recv() {
                break;
            }

            let sig = line_editor.read_line(&prompt);

            match sig {
                Ok(Signal::Success(buffer)) => {
                    let input = buffer.trim();
                    if input.is_empty() {
                        continue;
                    }

                    if input.eq_ignore_ascii_case("/quit") || input.eq_ignore_ascii_case("/exit") {
                        break;
                    }

                    match self.client.chat(&self.session_id, input.to_string()).await {
                        Ok(response) => {
                            if !response.is_empty() {
                                println!("{}", t!("assistant_prefix", msg = response));
                            }
                        }
                        Err(e) => {
                            eprintln!("{}", t!("error_prefix", err = e));
                        }
                    }
                }
                Ok(Signal::CtrlC) => {
                    println!("\nShutting down...");
                    break;
                }
                Ok(Signal::CtrlD) => {
                    break;
                }
                Err(e) => {
                    eprintln!("Error: {:?}", e);
                    break;
                }
            }
        }

        Ok(())
    }
}
