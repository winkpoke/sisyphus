pub mod event;
pub mod state;
pub mod terminal;
pub mod transcript;

use transcript::TranscriptItemKind;

use anyhow::Result;
use client::{ChatResponse, Client};
use common::bus::SystemEvent;
use event::{Event, EventHandler};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
};
use serde_json::Value;
use state::{AppStatus, InputMode, TuiState};
use std::time::Duration;
use tokio::sync::mpsc;

pub struct Tui {
    client: Client,
    shutdown_rx: mpsc::Receiver<()>,
    state: TuiState,
    clipboard: Option<arboard::Clipboard>,
}

enum Action {
    MessageSent(String),
    ResponseReceived(ChatResponse),
    Error(String),
}

impl Tui {
    pub fn new(client: Client, session_id: String, shutdown_rx: mpsc::Receiver<()>) -> Self {
        Self {
            client,
            shutdown_rx,
            state: TuiState::new(session_id),
            clipboard: arboard::Clipboard::new().ok(),
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        // Setup panic hook
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = terminal::restore();
            original_hook(panic_info);
        }));

        // Init terminal
        let mut terminal = terminal::init()?;
        let mut events = EventHandler::new(Duration::from_millis(250));
        let mut backend_events = self.client.subscribe_events()?;
        let (action_tx, mut action_rx) = mpsc::channel::<Action>(10);

        // Fetch initial model info
        if let Ok(model) = self.client.get_model().await {
            self.state.active_model = model;
        }

        // Loop
        loop {
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints(
                        [
                            Constraint::Min(1),
                            Constraint::Length(3),
                            Constraint::Length(1),
                        ]
                        .as_ref(),
                    )
                    .split(f.size());

                let transcript_block = Block::default()
                    .title(self.state.context_title.clone())
                    .borders(Borders::ALL)
                    .padding(ratatui::widgets::Padding::new(2, 2, 1, 1));
                let inner_area = transcript_block.inner(chunks[0]);
                let width = inner_area.width as usize;

                let mut lines = Vec::new();
                for (i, item) in self.state.transcript.items.iter().enumerate() {
                    let prefix = match item.kind {
                        TranscriptItemKind::User => "You: ",
                        TranscriptItemKind::Assistant => "Assistant: ",
                        TranscriptItemKind::System => "System: ",
                        TranscriptItemKind::Error => "Error: ",
                    };

                    let mut style = match item.kind {
                        TranscriptItemKind::User => Style::default().fg(Color::Cyan),
                        TranscriptItemKind::Assistant => Style::default().fg(Color::Green),
                        TranscriptItemKind::System => Style::default().fg(Color::Yellow),
                        TranscriptItemKind::Error => Style::default().fg(Color::Red),
                    };

                    if self.state.mode == InputMode::Selection
                        && Some(i) == self.state.selection.selected_message_index
                    {
                        style = style.add_modifier(Modifier::REVERSED);
                    }

                    let content = format!("{}{}", prefix, item.content);
                    lines.push(Line::from(Span::styled(content, style)));
                }

                // Calculate total lines for scrolling
                let mut total_lines = 0;
                for line in &lines {
                    let content_len = line.width();
                    if width > 0 {
                        total_lines += content_len.div_ceil(width);
                    } else {
                        total_lines += 1;
                    }
                }

                let height = inner_area.height as usize;
                let scroll_y = if self.state.transcript.stick_to_bottom {
                    if total_lines > height {
                        (total_lines - height) as u16
                    } else {
                        0
                    }
                } else {
                    self.state.transcript.scroll_offset
                };

                let transcript = Paragraph::new(lines)
                    .block(transcript_block)
                    .wrap(Wrap { trim: true })
                    .scroll((scroll_y, 0));

                f.render_widget(transcript, chunks[0]);

                let input_block = Block::default().title("Input").borders(Borders::ALL);
                let input_text = Paragraph::new(self.state.input_buffer.clone()).block(input_block);
                f.render_widget(input_text, chunks[1]);

                // Status Bar Layout
                let status_chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Percentage(25),
                        Constraint::Percentage(50),
                        Constraint::Percentage(25),
                    ])
                    .split(chunks[2]);

                // Left: Session ID
                let session_info = Paragraph::new(format!("Session: {}", self.state.session_id))
                    .style(Style::default().fg(Color::Blue));
                f.render_widget(session_info, status_chunks[0]);

                // Center: Model & Token Usage
                let model_info = Paragraph::new(format!("{} | {}", self.state.active_model, self.state.token_usage))
                    .alignment(ratatui::layout::Alignment::Center)
                    .style(Style::default().fg(Color::White));
                f.render_widget(model_info, status_chunks[1]);

                // Right: Status
                let spinner_chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                let status_text = match self.state.status {
                    AppStatus::Connected => Span::styled("● Connected", Style::default().fg(Color::Green)),
                    AppStatus::Disconnected => Span::styled("○ Disconnected", Style::default().fg(Color::Red)),
                    AppStatus::Processing => {
                        let frame = self.state.spinner_frame % spinner_chars.len();
                        Span::styled(format!("{} Processing", spinner_chars[frame]), Style::default().fg(Color::Yellow))
                    }
                };
                let status_widget = Paragraph::new(Line::from(status_text))
                    .alignment(ratatui::layout::Alignment::Right);
                f.render_widget(status_widget, status_chunks[2]);

                if self.state.mode == InputMode::CommandPalette {
                    let area = centered_rect(60, 40, f.size());
                    f.render_widget(Clear, area);

                    let items: Vec<ListItem> = self
                        .state
                        .command_palette
                        .filtered_commands
                        .iter()
                        .map(|c| ListItem::new(Line::from(c.as_str())))
                        .collect();

                    let title = format!(
                        "Command Palette (Filter: {})",
                        self.state.command_palette.input
                    );
                    let list = List::new(items)
                        .block(Block::default().title(title).borders(Borders::ALL))
                        .highlight_style(Style::default().add_modifier(Modifier::REVERSED))
                        .highlight_symbol("> ");

                    let mut state = ListState::default();
                    state.select(Some(self.state.command_palette.selected_index));

                    f.render_stateful_widget(list, area, &mut state);
                }

                if self.state.mode == InputMode::Overlay {
                    let area = centered_rect(60, 40, f.size());
                    f.render_widget(Clear, area);

                    let style = if self.state.overlay.is_error {
                        Style::default().fg(Color::Red)
                    } else {
                        Style::default()
                    };

                    let p = Paragraph::new(self.state.overlay.content.clone())
                        .block(
                            Block::default()
                                .title(self.state.overlay.title.clone())
                                .borders(Borders::ALL)
                                .border_style(style),
                        )
                        .scroll((self.state.overlay.scroll, 0))
                        .wrap(Wrap { trim: true });

                    f.render_widget(p, area);
                }
            })?;

            tokio::select! {
                Some(event) = events.next() => {
                    match event {
                        Event::Key(key) => {
                             if key.kind == crossterm::event::KeyEventKind::Press {
                                if self.state.mode == InputMode::Overlay {
                                    match key.code {
                                        crossterm::event::KeyCode::Esc | crossterm::event::KeyCode::Enter => {
                                            self.state.mode = InputMode::Normal;
                                            self.state.overlay.call_id = None;
                                        }
                                        crossterm::event::KeyCode::Char('a') if self.state.overlay.call_id.is_some() => {
                                             if let Some(call_id) = self.state.overlay.call_id.clone() {
                                                 let client = self.client.clone();
                                                 let session_id = self.state.session_id.clone();
                                                 let tx = action_tx.clone();

                                                 tokio::spawn(async move {
                                                     match client.submit_approval(&session_id, &call_id, "approve").await {
                                                         Ok(resp) => {
                                                             let _ = tx.send(Action::ResponseReceived(resp)).await;
                                                         }
                                                         Err(e) => {
                                                             let _ = tx.send(Action::Error(e.to_string())).await;
                                                         }
                                                     }
                                                 });

                                                 if !self.state.overlay.show_next_approval() {
                                                     self.state.mode = InputMode::Normal;
                                                     self.state.overlay.call_id = None;
                                                 }
                                             }
                                        }
                                        crossterm::event::KeyCode::Char('d') if self.state.overlay.call_id.is_some() => {
                                             if let Some(call_id) = self.state.overlay.call_id.clone() {
                                                 let client = self.client.clone();
                                                 let session_id = self.state.session_id.clone();
                                                 let tx = action_tx.clone();

                                                 tokio::spawn(async move {
                                                     match client.submit_approval(&session_id, &call_id, "deny").await {
                                                         Ok(resp) => {
                                                             let _ = tx.send(Action::ResponseReceived(resp)).await;
                                                         }
                                                         Err(e) => {
                                                             let _ = tx.send(Action::Error(e.to_string())).await;
                                                         }
                                                     }
                                                 });

                                                 if !self.state.overlay.show_next_approval() {
                                                     self.state.mode = InputMode::Normal;
                                                     self.state.overlay.call_id = None;
                                                 }
                                             }
                                        }
                                        crossterm::event::KeyCode::Down | crossterm::event::KeyCode::Char('j') => {
                                            self.state.overlay.scroll_down();
                                        }
                                        crossterm::event::KeyCode::Up | crossterm::event::KeyCode::Char('k') => {
                                            self.state.overlay.scroll_up();
                                        }
                                        _ => {}
                                    }
                                } else if self.state.mode == InputMode::Selection {
                                    match key.code {
                                        crossterm::event::KeyCode::Esc => {
                                            self.state.mode = InputMode::Normal;
                                            self.state.selection.selected_message_index = None;
                                        }
                                        crossterm::event::KeyCode::Up => {
                                            if let Some(idx) = self.state.selection.selected_message_index {
                                                if idx > 0 {
                                                    self.state.selection.selected_message_index = Some(idx - 1);
                                                }
                                            } else if !self.state.transcript.items.is_empty() {
                                                self.state.selection.selected_message_index = Some(self.state.transcript.items.len() - 1);
                                            }
                                        }
                                        crossterm::event::KeyCode::Down => {
                                             if let Some(idx) = self.state.selection.selected_message_index {
                                                if idx < self.state.transcript.items.len() - 1 {
                                                    self.state.selection.selected_message_index = Some(idx + 1);
                                                }
                                            }
                                        }
                                        crossterm::event::KeyCode::Char('c') | crossterm::event::KeyCode::Enter => {
                                             if let Some(idx) = self.state.selection.selected_message_index {
                                                 if let Some(item) = self.state.transcript.items.get(idx) {
                                                     if let Some(cb) = &mut self.clipboard {
                                                         if cb.set_text(&item.content).is_err() {
                                                             self.state.overlay.show("Copy Failed".to_string(), item.content.clone(), true);
                                                             self.state.mode = InputMode::Overlay;
                                                         } else {
                                                             self.state.mode = InputMode::Normal;
                                                         }
                                                     } else {
                                                         self.state.overlay.show("Copy (Clipboard Unavailable)".to_string(), item.content.clone(), false);
                                                         self.state.mode = InputMode::Overlay;
                                                     }
                                                     self.state.selection.selected_message_index = None;
                                                 }
                                             }
                                        }
                                        _ => {}
                                    }
                                } else if self.state.mode == InputMode::CommandPalette {
                                     match key.code {
                                         crossterm::event::KeyCode::Esc => {
                                             self.state.mode = InputMode::Normal;
                                             self.state.command_palette.reset();
                                         }
                                         crossterm::event::KeyCode::Char(c) => {
                                             self.state.command_palette.input.push(c);
                                             self.state.command_palette.update_filter();
                                         }
                                         crossterm::event::KeyCode::Backspace => {
                                             self.state.command_palette.input.pop();
                                             self.state.command_palette.update_filter();
                                         }
                                         crossterm::event::KeyCode::Down => self.state.command_palette.select_next(),
                                         crossterm::event::KeyCode::Up => self.state.command_palette.select_prev(),
                                         crossterm::event::KeyCode::Enter => {
                                             if let Some(cmd) = self.state.command_palette.filtered_commands.get(self.state.command_palette.selected_index) {
                                                 let cmd = cmd.clone();
                                                 self.state.mode = InputMode::Normal;
                                                 self.state.command_palette.reset();

                                                 if cmd == "/help" {
                                                      self.state.mode = InputMode::Overlay;
                                                      self.state.overlay.show(
                                                          "Help".to_string(),
                                                          "Available commands:\n/quit - Quit\n/exit - Quit\n/help - Show this help\n/clear - Clear transcript".to_string(),
                                                          false
                                                      );
                                                  } else if cmd == "/clear" {
                                                      self.state.transcript.clear();
                                                  } else if cmd == "/debug" {
                                                      self.state.debug_mode = !self.state.debug_mode;
                                                      let status = if self.state.debug_mode { "enabled" } else { "disabled" };
                                                      self.state.add_message(TranscriptItemKind::System, format!("Debug mode {}", status));
                                                  } else {
                                                      self.state.input_buffer = cmd;
                                                  }
                                             } else {
                                                 self.state.mode = InputMode::Normal;
                                                 self.state.command_palette.reset();
                                             }
                                         }
                                         _ => {}
                                     }
                                } else {
                                    match key.code {
                                        crossterm::event::KeyCode::Char('p') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                                             self.state.mode = InputMode::CommandPalette;
                                             self.state.command_palette.reset();
                                         }
                                         crossterm::event::KeyCode::Char('s') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                                             self.state.mode = InputMode::Selection;
                                             if !self.state.transcript.items.is_empty() {
                                                 self.state.selection.selected_message_index = Some(self.state.transcript.items.len() - 1);
                                             }
                                         }
                                         crossterm::event::KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                                            break;
                                        }
                                        crossterm::event::KeyCode::Char(c) => {
                                            if c == '/' && self.state.input_buffer.is_empty() {
                                                self.state.mode = InputMode::CommandPalette;
                                                self.state.command_palette.reset();
                                                self.state.command_palette.input.push('/');
                                                self.state.command_palette.update_filter();
                                            } else {
                                                self.state.handle_char(c);
                                            }
                                        }
                                        crossterm::event::KeyCode::Backspace => {
                                            self.state.handle_backspace();
                                        }
                                        crossterm::event::KeyCode::PageUp => {
                                            self.state.transcript.stick_to_bottom = false;
                                            self.state.transcript.scroll_offset = self.state.transcript.scroll_offset.saturating_sub(5);
                                        }
                                        crossterm::event::KeyCode::PageDown => {
                                            self.state.transcript.scroll_offset = self.state.transcript.scroll_offset.saturating_add(5);
                                        }
                                        crossterm::event::KeyCode::End => {
                                            self.state.transcript.stick_to_bottom = true;
                                        }
                                        crossterm::event::KeyCode::Enter => {
                                            if let Some(input) = self.state.get_input_and_clear() {
                                                if input == "/quit" || input == "/exit" {
                                                    break;
                                                }

                                                // User sent a message, so we should stick to bottom to see it
                                                self.state.transcript.stick_to_bottom = true;

                                                let client = self.client.clone();
                                                let session_id = self.state.session_id.clone();
                                                let tx = action_tx.clone();
                                                let input_clone = input.clone();

                                                tx.send(Action::MessageSent(input_clone.clone())).await.ok();

                                                tokio::spawn(async move {
                                                    match client.chat(&session_id, input_clone).await {
                                                        Ok(resp) => {
                                                            let _ = tx.send(Action::ResponseReceived(resp)).await;
                                                        }
                                                        Err(e) => {
                                                            let _ = tx.send(Action::Error(e.to_string())).await;
                                                        }
                                                    }
                                                });
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                             }
                        }
                        Event::Tick => {
                            self.state.spinner_frame = self.state.spinner_frame.wrapping_add(1);
                        }
                        _ => {}
                    }
                }
                Some(result) = backend_events.next() => {
                    match result {
                        Ok(event) => {
                             if let reqwest_eventsource::Event::Message(msg) = event {
                                 // Try to parse as SystemEvent
                                 match serde_json::from_str::<SystemEvent>(&msg.data) {
                                     Ok(sys_event) => {
                                         match sys_event {
                                             SystemEvent::PermissionRequest { operation, tool_name, call_id } => {
                                                self.state.mode = InputMode::Overlay;
                                                self.state.overlay.enqueue_approval(
                                                    "Permission Required".to_string(),
                                                    format!("Operation: {}\nTool: {}\nCall ID: {}\n\nPress 'a' to Approve or 'd' to Deny.", operation, tool_name, call_id),
                                                    call_id
                                                );
                                             }
                                             SystemEvent::ToolExecuted { tool, result } => {
                                                 self.state.add_message(TranscriptItemKind::System, format!("Tool executed: {}", tool));
                                                 let preview = truncate_payload(&result, 100);
                                                 self.state.add_message(TranscriptItemKind::System, format!("Result: {}", preview));
                                             }
                                             SystemEvent::Error { message } => {
                                                 self.state.add_message(TranscriptItemKind::Error, format!("Error: {}", message));
                                             }
                                             SystemEvent::AgentStateChanged { state, .. } => {
                                                self.state.add_message(TranscriptItemKind::System, format!("Agent state: {}", state));
                                                if state.eq_ignore_ascii_case("busy") {
                                                    self.state.status = AppStatus::Processing;
                                                } else {
                                                    self.state.status = AppStatus::Connected;
                                                }
                                            }
                                             SystemEvent::MessageReceived { .. } => {
                                                 // Suppress as it's shown in chat UI
                                             }
                                             SystemEvent::Shutdown => {
                                                 self.state.add_message(TranscriptItemKind::System, "Server shutting down".to_string());
                                             }
                                         }
                                     }
                                     Err(_) => {
                                         if !self.state.debug_mode {
                                             self.state.add_message(TranscriptItemKind::System, "Received system event".to_string());
                                         }
                                     }
                                 }

                                 // Debug output
                                 if self.state.debug_mode {
                                     let redacted = redact_json(&msg.data);
                                     let truncated = truncate_payload(&redacted, 500);
                                     self.state.add_message(TranscriptItemKind::System, format!("DEBUG: {}", truncated));
                                 }
                             }
                        }
                        Err(e) => {
                            self.state.add_message(TranscriptItemKind::Error, format!("Error: {}", e));
                        }
                    }
                }
                Some(action) = action_rx.recv() => {
                    match action {
                        Action::MessageSent(msg) => {
                            self.state.status = AppStatus::Processing;
                            self.state.add_message(TranscriptItemKind::User, msg);
                        }
                        Action::ResponseReceived(resp) => {
                            self.state.status = AppStatus::Connected;
                            if let Some(sid) = resp.session_id {
                                self.state.update_session_id(sid);
                            }
                            if let Some(usage) = resp.usage {
                                self.state.token_usage = usage;
                            }
                            if let Some(model) = resp.model {
                                self.state.active_model = model;
                            }
                            self.state.add_message(TranscriptItemKind::Assistant, resp.response);
                        }
                        Action::Error(err) => {
                            self.state.status = AppStatus::Connected;
                            self.state.add_message(TranscriptItemKind::Error, err);
                        }
                    }
                }
                _ = self.shutdown_rx.recv() => {
                    break;
                }
            }
        }

        // Restore terminal
        terminal::restore()?;
        Ok(())
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

fn redact_json(json_str: &str) -> String {
    if let Ok(mut value) = serde_json::from_str::<Value>(json_str) {
        redact_value(&mut value);
        serde_json::to_string(&value).unwrap_or_else(|_| json_str.to_string())
    } else {
        json_str.to_string()
    }
}

fn redact_value(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (k, v) in map.iter_mut() {
                let key_lower = k.to_lowercase();
                if key_lower.contains("token")
                    || key_lower.contains("secret")
                    || key_lower.contains("api_key")
                    || key_lower.contains("authorization")
                {
                    *v = Value::String("[REDACTED]".to_string());
                } else {
                    redact_value(v);
                }
            }
        }
        Value::Array(arr) => {
            for v in arr {
                redact_value(v);
            }
        }
        _ => {}
    }
}

fn truncate_payload(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        let truncated: String = s.chars().take(max_len).collect();
        format!("{}... (truncated)", truncated)
    } else {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redaction() {
        let json = r#"{"token": "secret123", "other": "value", "nested": {"api_key": "abc"}}"#;
        let redacted = redact_json(json);
        assert!(redacted.contains("[REDACTED]"));
        assert!(!redacted.contains("secret123"));
        assert!(!redacted.contains("abc"));
        assert!(redacted.contains("value"));
    }

    #[test]
    fn test_truncation() {
        let s = "1234567890";
        let truncated = truncate_payload(s, 5);
        assert_eq!(truncated, "12345... (truncated)");

        let not_truncated = truncate_payload(s, 10);
        assert_eq!(not_truncated, "1234567890");

        // Test with multi-byte characters (emoji)
        let s_emoji = "👋 world";
        let truncated_emoji = truncate_payload(s_emoji, 1);
        assert_eq!(truncated_emoji, "👋... (truncated)");
    }
}
