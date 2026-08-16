use anyhow::Result;
use client::Client;
use sisyphus_core::command::parser::parse_command;
use std::borrow::Cow;
use std::cell::RefCell;

use reedline::{
    default_emacs_keybindings, Color, ColumnarMenu, EditCommand, Emacs, KeyCode, KeyModifiers,
    Prompt, PromptEditMode, PromptHistorySearch, Reedline, ReedlineEvent, ReedlineMenu, Signal,
};
use rust_i18n::t;
use tokio::sync::broadcast::Receiver;

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
    show_reasoning_summary: RefCell<bool>,
}

impl Repl {
    pub fn new(client: Client, session_id: String, shutdown_rx: Receiver<()>) -> Self {
        Self {
            client,
            session_id,
            shutdown_rx,
            show_reasoning_summary: RefCell::new(true),
        }
    }

    /// Whether reasoning summary transcript entries are currently visible.
    /// Summaries are shown by default.
    pub fn reasoning_summary_visible(&self) -> bool {
        *self.show_reasoning_summary.borrow()
    }

    /// Toggle reasoning summary visibility (`/think`). Returns the new status
    /// string ("enabled"/"disabled") for display.
    fn toggle_reasoning_summary(&self) -> &'static str {
        let current = *self.show_reasoning_summary.borrow();
        *self.show_reasoning_summary.borrow_mut() = !current;
        if current {
            "disabled"
        } else {
            "enabled"
        }
    }

    async fn handle_agents_list(&self) -> Result<()> {
        let session = self.client.get_session(&self.session_id).await?;
        let agents = self.client.list_agents().await?;

        println!("Available Agents:");
        if agents.is_empty() {
            println!("  No agents available");
            return Ok(());
        }

        let current_agent_id = session.agent_id.as_deref();

        for agent in &agents {
            let prefix = if current_agent_id == Some(agent.id.as_str()) {
                "* "
            } else {
                "  "
            };
            println!("{}{} - {} ({})", prefix, agent.id, agent.name, agent.model);
            println!("    {}", agent.description);
        }
        Ok(())
    }

    async fn handle_agents_switch(&mut self, agent_id: &str) -> Result<()> {
        let updated_session = self
            .client
            .update_session_agent(&self.session_id, agent_id)
            .await?;
        println!(
            "Agent changed to '{}' for session {}",
            agent_id, updated_session.id
        );
        Ok(())
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
                                "/think" => {
                                    let status = self.toggle_reasoning_summary();
                                    println!("Reasoning summary visibility: {}", status);
                                    continue;
                                }
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
                                "/agents" => {
                                    if _args.is_empty() {
                                        match self.handle_agents_list().await {
                                            Ok(_) => continue,
                                            Err(e) => {
                                                eprintln!("{}", t!("error_prefix", err = e));
                                                continue;
                                            }
                                        }
                                    } else {
                                        let agent_id = _args[0].as_str();
                                        match self.handle_agents_switch(agent_id).await {
                                            Ok(_) => continue,
                                            Err(e) => {
                                                eprintln!("{}", t!("error_prefix", err = e));
                                                continue;
                                            }
                                        }
                                    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::broadcast;

    #[test]
    fn test_prompt_rendering() {
        let prompt = SisyphusPrompt;

        assert_eq!(prompt.render_prompt_left(), "› ");
        assert_eq!(prompt.render_prompt_right(), "");
        assert_eq!(prompt.render_prompt_indicator(PromptEditMode::Emacs), "");
        assert_eq!(prompt.render_prompt_multiline_indicator(), "::: ");
        assert_eq!(prompt.get_prompt_color(), Color::Cyan);
    }

    #[test]
    fn test_repl_creation() {
        let (tx, _rx) = broadcast::channel(1);
        let url = url::Url::parse("http://localhost:3000").unwrap();
        let client = Client::new(url);

        let repl = Repl::new(client, "test-session-id".to_string(), _rx);
        assert_eq!(repl.session_id, "test-session-id");

        drop(tx);
    }

    fn make_repl() -> (Repl, tokio::sync::broadcast::Sender<()>) {
        let (tx, rx) = broadcast::channel(1);
        let url = url::Url::parse("http://localhost:3000").unwrap();
        let client = Client::new(url);
        (Repl::new(client, "test-session-id".to_string(), rx), tx)
    }

    // ---- Task 7.2: /think toggle behavior (REPL) ----

    #[test]
    fn test_reasoning_summary_visible_by_default() {
        let (repl, _tx) = make_repl();
        assert!(
            repl.reasoning_summary_visible(),
            "REPL must show reasoning summaries by default (spec: cli-tui)"
        );
    }

    #[test]
    fn test_think_toggle_flips_visibility_and_reports_status() {
        let (repl, _tx) = make_repl();

        // First toggle: shown -> hidden
        assert_eq!(repl.toggle_reasoning_summary(), "disabled");
        assert!(!repl.reasoning_summary_visible());

        // Second toggle: hidden -> shown
        assert_eq!(repl.toggle_reasoning_summary(), "enabled");
        assert!(repl.reasoning_summary_visible());
    }
}
