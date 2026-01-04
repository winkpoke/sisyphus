use super::action::Action;
use super::app::App;
use super::state::{AppStatus, InputMode, Toast, ToastKind, TranscriptItemKind};
use common::bus::SystemEvent;
use crossterm::event::{KeyCode, KeyModifiers};
use sisyphus_core::command::CommandEffect;
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub enum TuiInstruction {
    None,
    Chat { session_id: String, input: String },
    SubmitApproval { session_id: String, call_id: String, decision: String },
    Quit,
    DispatchCommand(String),
}

pub fn update(app: &mut App, action: Action) -> TuiInstruction {
    match action {
        Action::Tick => {
            app.tick();
            if let Some(toast) = &app.state.toast {
                if std::time::Instant::now() > toast.expires_at {
                    app.state.toast = None;
                }
            }
        }
        Action::Quit => {
            app.quit();
            return TuiInstruction::Quit;
        }
        Action::Resize(_, _) | Action::Render | Action::Init => {}
        Action::Error(err) => {
            app.state.status = AppStatus::Connected;
            app.state.add_message(TranscriptItemKind::Error, err);
        }
        Action::CommandResult(outcome) => {
            if let Some(output) = &outcome.output {
                app.state.add_message(TranscriptItemKind::System, output.clone());
            }
            return handle_command_effect(app, outcome.effect);
        }
        Action::MessageSent(msg) => {
            app.state.status = AppStatus::Processing;
            app.state.add_message(TranscriptItemKind::User, msg);
        }
        Action::ResponseReceived(resp) => {
            app.state.status = AppStatus::Connected;
            if let Some(sid) = resp.session_id {
                app.state.update_session_id(sid);
            }
            if let Some(usage) = resp.usage {
                app.state.token_usage = usage;
            }
            if let Some(model) = resp.model {
                app.state.active_model = model;
            }
            app.state.add_message(TranscriptItemKind::Assistant, resp.response);

            if let Some(effect) = resp.effect {
                return handle_command_effect(app, effect);
            }
        }
        Action::SystemEvent(event) => {
            handle_system_event(app, event);
        }
        Action::Key(key) => {
            return handle_key_event(app, key);
        }
    }
    TuiInstruction::None
}

fn handle_command_effect(app: &mut App, effect: CommandEffect) -> TuiInstruction {
    match effect {
        CommandEffect::ToggleDebug => {
            app.state.debug_mode = !app.state.debug_mode;
            let status = if app.state.debug_mode { "enabled" } else { "disabled" };
            app.state.add_message(TranscriptItemKind::System, format!("Debug mode {}", status));
        }
        CommandEffect::Exit => {
            app.quit();
            return TuiInstruction::Quit;
        }
        CommandEffect::ClearHistory => {
            app.state.transcript.clear();
        }
        CommandEffect::NewSession => {
            // New session ID is handled in Action::ResponseReceived via sid update
        }
        CommandEffect::None => {}
    }
    TuiInstruction::None
}

fn handle_system_event(app: &mut App, event: SystemEvent) {
    match event {
        SystemEvent::PermissionRequest { operation, tool_name, call_id } => {
            app.state.mode = InputMode::Overlay;
            app.state.overlay.enqueue_approval(
                "Permission Required".to_string(),
                format!("Operation: {}\nTool: {}\nCall ID: {}\n\nPress 'a' to Approve or 'd' to Deny.", operation, tool_name, call_id),
                call_id
            );
        }
        SystemEvent::ToolExecuted { tool, result } => {
            app.state.add_message(TranscriptItemKind::System, format!("Tool executed: {}", tool));
            let output = if app.state.debug_mode {
                result
            } else {
                truncate_payload(&result, 100)
            };
            app.state.add_message(TranscriptItemKind::System, format!("Result: {}", output));
        }
        SystemEvent::Error { message } => {
            app.state.add_message(TranscriptItemKind::Error, format!("Error: {}", message));
        }
        SystemEvent::AgentStateChanged { state, .. } => {
            app.state.add_message(TranscriptItemKind::System, format!("Agent state: {}", state));
            if state.eq_ignore_ascii_case("busy") {
                app.state.status = AppStatus::Processing;
            } else {
                app.state.status = AppStatus::Connected;
            }
        }
        SystemEvent::MessageReceived { .. } => {
            // Suppress as it's shown in chat UI
        }
        SystemEvent::Shutdown => {
            app.state.add_message(TranscriptItemKind::System, "Server shutting down".to_string());
        }
    }
}

fn handle_key_event(app: &mut App, key: crossterm::event::KeyEvent) -> TuiInstruction {
    if key.kind != crossterm::event::KeyEventKind::Press {
        return TuiInstruction::None;
    }

    match app.state.mode {
        InputMode::Overlay => match key.code {
            KeyCode::Esc | KeyCode::Enter => {
                if let Some(call_id) = app.state.overlay.call_id.take() {
                    app.state.mode = InputMode::Normal;
                    return TuiInstruction::SubmitApproval {
                        session_id: app.state.session_id.clone(),
                        call_id,
                        decision: "deny".to_string(),
                    };
                }
                app.state.mode = InputMode::Normal;
            }
            KeyCode::Char('a') if app.state.overlay.call_id.is_some() => {
                if let Some(call_id) = app.state.overlay.call_id.clone() {
                    let cmd = TuiInstruction::SubmitApproval {
                        session_id: app.state.session_id.clone(),
                        call_id,
                        decision: "approve".to_string(),
                    };
                    
                    if !app.state.overlay.show_next_approval() {
                        app.state.mode = InputMode::Normal;
                        app.state.overlay.call_id = None;
                    }
                    return cmd;
                }
            }
            KeyCode::Char('d') if app.state.overlay.call_id.is_some() => {
                if let Some(call_id) = app.state.overlay.call_id.clone() {
                     let cmd = TuiInstruction::SubmitApproval {
                        session_id: app.state.session_id.clone(),
                        call_id,
                        decision: "deny".to_string(),
                    };
                    
                    if !app.state.overlay.show_next_approval() {
                        app.state.mode = InputMode::Normal;
                        app.state.overlay.call_id = None;
                    }
                    return cmd;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                app.state.overlay.scroll_down();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                app.state.overlay.scroll_up();
            }
            _ => {}
        },
        InputMode::Selection => match key.code {
            KeyCode::Esc => {
                app.state.mode = InputMode::Normal;
                app.state.selection.selected_message_index = None;
            }
            KeyCode::Up => {
                if let Some(idx) = app.state.selection.selected_message_index {
                    if idx > 0 {
                        app.state.selection.selected_message_index = Some(idx - 1);
                    }
                } else if !app.state.transcript.items.is_empty() {
                    app.state.selection.selected_message_index = Some(app.state.transcript.items.len() - 1);
                }
            }
            KeyCode::Down => {
                if let Some(idx) = app.state.selection.selected_message_index {
                    if idx < app.state.transcript.items.len() - 1 {
                        app.state.selection.selected_message_index = Some(idx + 1);
                    }
                }
            }
            KeyCode::Char('c') | KeyCode::Enter => {
                if let Some(idx) = app.state.selection.selected_message_index {
                    if let Some(item) = app.state.transcript.items.get(idx) {
                        if let Some(cb) = &mut app.clipboard {
                            if cb.set_text(&item.content).is_err() {
                                app.state.toast = Some(Toast::new("Copy Failed".to_string(), ToastKind::Error, Duration::from_secs(2)));
                            } else {
                                app.state.toast = Some(Toast::new("✓ Copied to clipboard".to_string(), ToastKind::Success, Duration::from_secs(2)));
                            }
                        } else {
                            app.state.toast = Some(Toast::new("Clipboard Unavailable".to_string(), ToastKind::Error, Duration::from_secs(2)));
                        }
                        app.state.mode = InputMode::Normal;
                        app.state.selection.selected_message_index = None;
                    }
                }
            }
            _ => {}
        },
        InputMode::CommandPalette => match key.code {
            KeyCode::Esc => {
                app.state.mode = InputMode::Normal;
                app.state.command_palette.reset();
            }
            KeyCode::Char(c) => {
                app.state.command_palette.input.push(c);
                app.state.command_palette.update_filter();
            }
            KeyCode::Backspace => {
                app.state.command_palette.input.pop();
                app.state.command_palette.update_filter();
            }
            KeyCode::Down => app.state.command_palette.select_next(),
            KeyCode::Up => app.state.command_palette.select_prev(),
            KeyCode::Enter => {
                if let Some(cmd) = app.state.command_palette.filtered_commands.get(app.state.command_palette.selected_index) {
                    let cmd = cmd.clone();
                    app.state.mode = InputMode::Normal;
                    app.state.command_palette.reset();
                    
                    if cmd == "/clear" || cmd == "/new" {
                        return TuiInstruction::Chat {
                            session_id: app.state.session_id.clone(),
                            input: cmd,
                        };
                    }
                    
                    return TuiInstruction::DispatchCommand(cmd);
                } else {
                    app.state.mode = InputMode::Normal;
                    app.state.command_palette.reset();
                }
            }
            _ => {}
        },
        InputMode::Normal => match key.code {
            KeyCode::Char('p') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.state.mode = InputMode::CommandPalette;
                app.state.command_palette.reset();
                app.state.command_palette.input.push('/');
                app.state.command_palette.update_filter();
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                app.state.mode = InputMode::Selection;
                if !app.state.transcript.items.is_empty() {
                    app.state.selection.selected_message_index = Some(app.state.transcript.items.len() - 1);
                }
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                return TuiInstruction::Quit;
            }
            KeyCode::Char(c) => {
                if c == '/' && app.state.input_buffer.is_empty() {
                    app.state.mode = InputMode::CommandPalette;
                    app.state.command_palette.reset();
                    app.state.command_palette.input.push('/');
                    app.state.command_palette.update_filter();
                } else {
                    app.state.handle_char(c);
                }
            }
            KeyCode::Backspace => {
                app.state.handle_backspace();
            }
            KeyCode::PageUp => {
                app.state.transcript.stick_to_bottom = false;
                app.state.transcript.scroll_offset = app.state.transcript.scroll_offset.saturating_sub(5);
            }
            KeyCode::PageDown => {
                app.state.transcript.scroll_offset = app.state.transcript.scroll_offset.saturating_add(5);
            }
            KeyCode::End => {
                app.state.transcript.stick_to_bottom = true;
            }
            KeyCode::Enter => {
                if let Some(input) = app.state.get_input_and_clear() {
                    if input.starts_with('/') {
                        let cmd_name = input.split_whitespace().next().unwrap_or("");
                        
                        // Force these commands to be sent to server even if they are in registry
                        if cmd_name != "/clear" && cmd_name != "/new" {
                            if app.registry.get(cmd_name).is_some() {
                                return TuiInstruction::DispatchCommand(input);
                            }
                        }
                    }

                    if input == "/quit" || input == "/exit" {
                        // We send the command to the server and handle the Exit effect in ResponseReceived
                    }

                    app.state.transcript.stick_to_bottom = true;
                    return TuiInstruction::Chat {
                        session_id: app.state.session_id.clone(),
                        input,
                    };
                }
            }
            _ => {}
        }
    }
    TuiInstruction::None
}

fn truncate_payload(s: &str, max_len: usize) -> String {
    if s.chars().count() > max_len {
        let truncated: String = s.chars().take(max_len).collect();
        format!("{}... (truncated)", truncated)
    } else {
        s.to_string()
    }
}
