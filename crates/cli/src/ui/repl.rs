use anyhow::Result;
use client::Client;
use sisyphus_core::command::parser::parse_command;
use std::borrow::Cow;

use reedline::{
    default_emacs_keybindings, Color, ColumnarMenu, EditCommand, Emacs, KeyCode, KeyModifiers,
    Prompt, PromptEditMode, PromptHistorySearch, Reedline, ReedlineEvent, ReedlineMenu, Signal,
};
use rust_i18n::t;
use tokio::sync::mpsc::Receiver;

use super::completer::CommandCompleter;

pub struct SisyphusPrompt;

impl Prompt for SisyphusPrompt {
    fn render_prompt_left(&self) -> Cow<'_, str> {
        Cow::Borrowed("› ")
    }

    fn render_prompt_right(&self) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_indicator(&self, _prompt_mode: PromptEditMode) -> Cow<'_, str> {
        Cow::Borrowed("")
    }

    fn render_prompt_multiline_indicator(&self) -> Cow<'_, str> {
        Cow::Borrowed("::: ")
    }

    fn render_prompt_history_search_indicator(
        &self,
        _history_search: PromptHistorySearch,
    ) -> Cow<'_, str> {
        Cow::Borrowed("? ")
    }

    fn get_prompt_color(&self) -> Color {
        Color::Cyan
    }
}

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
        let completion_menu = Box::new(
            ColumnarMenu::default()
                .with_name("completion_menu")
                .with_marker("".to_string()),
        );

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

        let prompt = SisyphusPrompt;

        println!("{}", t!("type_exit"));
        loop {
            if self.shutdown_rx.try_recv().is_ok() {
                break;
            }

            let sig = line_editor.read_line(&prompt);

            match sig {
                Ok(Signal::Success(buffer)) => {
                    let input = buffer.trim();
                    if input.is_empty() {
                        continue;
                    }

                    if input.starts_with('/') {
                        if let Ok((cmd, _args, _raw)) = parse_command(input) {
                            match cmd.as_str() {
                                "/exit" | "/quit" => break,
                                "/new" => {
                                    match self.client.create_session().await {
                                        Ok(session) => {
                                            self.session_id = session.id;
                                            println!("New session created: {}", self.session_id);
                                        }
                                        Err(e) => eprintln!("{}", t!("error_prefix", err = e)),
                                    }
                                    continue;
                                }
                                "/clear" => {
                                    match self.client.clear_session(&self.session_id).await {
                                        Ok(_) => println!("Session cleared."),
                                        Err(e) => eprintln!("{}", t!("error_prefix", err = e)),
                                    }
                                    continue;
                                }
                                _ => {}
                            }
                        }
                    }

                    match self.client.chat(&self.session_id, input.to_string()).await {
                        Ok(chat_resp) => {
                            if let Some(new_id) = chat_resp.session_id {
                                self.session_id = new_id;
                            }
                            if !chat_resp.response.is_empty() {
                                println!("{}", t!("assistant_prefix", msg = chat_resp.response));
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
