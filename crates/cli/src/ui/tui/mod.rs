pub mod event;
pub mod state;
pub mod terminal;
pub mod transcript;

use transcript::TranscriptItemKind;

use anyhow::Result;
use client::Client;
use event::{Event, EventHandler};
use futures::StreamExt;
use ratatui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use state::TuiState;
use std::time::Duration;
use tokio::sync::mpsc;

pub struct Tui {
    client: Client,
    shutdown_rx: mpsc::Receiver<()>,
    state: TuiState,
}

enum Action {
    MessageSent(String),
    ResponseReceived(String, Option<String>), // response, new_session_id
    StreamStart(TranscriptItemKind),
    StreamDelta(String),
    StreamEnd,
    Error(String),
}

impl Tui {
    pub fn new(client: Client, session_id: String, shutdown_rx: mpsc::Receiver<()>) -> Self {
        Self {
            client,
            shutdown_rx,
            state: TuiState::new(session_id),
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

        // Loop
        loop {
            terminal.draw(|f| {
                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
                    .split(f.size());

                let transcript_block = Block::default().title("Transcript").borders(Borders::ALL);
                let inner_area = transcript_block.inner(chunks[0]);
                let width = inner_area.width as usize;

                let mut lines = Vec::new();
                for item in &self.state.transcript.items {
                    let prefix = match item.kind {
                        TranscriptItemKind::User => "You: ",
                        TranscriptItemKind::Assistant => "Assistant: ",
                        TranscriptItemKind::System => "System: ",
                        TranscriptItemKind::Error => "Error: ",
                    };

                    let style = match item.kind {
                        TranscriptItemKind::User => Style::default().fg(Color::Cyan),
                        TranscriptItemKind::Assistant => Style::default().fg(Color::Green),
                        TranscriptItemKind::System => Style::default().fg(Color::Yellow),
                        TranscriptItemKind::Error => Style::default().fg(Color::Red),
                    };

                    let content = format!("{}{}", prefix, item.content);
                    lines.push(Line::from(Span::styled(content, style)));
                }

                // Calculate total lines for scrolling
                let mut total_lines = 0;
                for line in &lines {
                    let content_len = line.width();
                    if width > 0 {
                        total_lines += (content_len + width - 1) / width;
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
            })?;

            tokio::select! {
                Some(event) = events.next() => {
                    match event {
                        Event::Key(key) => {
                             if key.kind == crossterm::event::KeyEventKind::Press {
                                match key.code {
                                    crossterm::event::KeyCode::Char('c') if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) => {
                                        break;
                                    }
                                    crossterm::event::KeyCode::Char(c) => {
                                        self.state.handle_char(c);
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
                                        // If we are at the bottom, we could re-enable stick_to_bottom
                                        // For simplicity, user must explicitly go to end or type something to re-enable?
                                        // Or we can check here, but we don't know total_lines easily without width.
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
                                                        let _ = tx.send(Action::ResponseReceived(resp.response, resp.session_id)).await;
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
                        Event::Tick => {}
                        Event::Resize(w, h) => {
                             tracing::debug!("Resize: {}x{}", w, h);
                        }
                    }
                }
                Some(result) = backend_events.next() => {
                    match result {
                        Ok(event) => {
                             if let reqwest_eventsource::Event::Message(msg) = event {
                                 // Handle streaming events here if possible
                                 // For now, mapping to System messages
                                 self.state.add_message(TranscriptItemKind::System, format!("Event: {}", msg.data));
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
                            self.state.add_message(TranscriptItemKind::User, msg);
                        }
                        Action::ResponseReceived(response, new_sid) => {
                            if let Some(sid) = new_sid {
                                self.state.update_session_id(sid);
                            }
                            self.state.add_message(TranscriptItemKind::Assistant, response);
                        }
                        Action::StreamStart(kind) => {
                            self.state.transcript.start_streaming(kind);
                        }
                        Action::StreamDelta(delta) => {
                            self.state.transcript.append_streaming(&delta);
                        }
                        Action::StreamEnd => {
                            self.state.transcript.finish_streaming();
                        }
                        Action::Error(err) => {
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
