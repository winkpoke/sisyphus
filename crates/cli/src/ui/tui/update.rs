use super::action::Action;
use super::app::App;
use super::state::{
    AgentSelectionState, AppStatus, InputMode, Toast, ToastKind, TranscriptItemKind,
};
use common::bus::SystemEvent;
use crossterm::event::{KeyCode, KeyModifiers};
use sisyphus_core::command::parser::parse_command;
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub enum TuiInstruction {
    None,
    Chat {
        session_id: String,
        input: String,
    },
    SubmitApproval {
        session_id: String,
        call_id: String,
        decision: String,
    },
    Quit,
    DispatchCommand(String),
    NewSession,
    ClearSession,
    ToggleDebug,
    ShowAgentList,
    SwitchAgent(String, String), // agent_id, agent_name
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
                app.state
                    .add_message(TranscriptItemKind::System, output.clone());
            }
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
            app.state
                .add_message(TranscriptItemKind::Assistant, resp.response);
        }
        Action::SystemEvent(event) => {
            handle_system_event(app, event);
        }
        Action::Key(key) => {
            return handle_key_event(app, key);
        }
        Action::SessionCreated(session) => {
            app.state.update_session_id(session.id);
            app.state.transcript.clear();
            app.state.token_usage = String::new();
            app.state.add_message(
                TranscriptItemKind::System,
                "Started new session".to_string(),
            );
        }
        Action::ToggleDebug => {
            app.state.debug_mode = !app.state.debug_mode;
            let status = if app.state.debug_mode {
                "enabled"
            } else {
                "disabled"
            };
            app.state
                .add_message(TranscriptItemKind::System, format!("Debug mode {}", status));
        }
        Action::ClearHistory => {
            app.state.transcript.clear();
            app.state.token_usage = String::new();
        }
        Action::AgentListReceived(agents) => {
            app.state.agent_selection = Some(AgentSelectionState::new(agents));
            app.state.mode = InputMode::AgentSelection;
        }
        Action::AgentSwitched(_id, name) => {
            app.state.toast = Some(Toast::new(
                format!("Switched to agent: {}", name),
                ToastKind::Success,
                Duration::from_secs(2),
            ));
            app.state.mode = InputMode::Normal;
            app.state.agent_selection = None;
        }
    }
    TuiInstruction::None
}

fn handle_system_event(app: &mut App, event: SystemEvent) {
    match event {
        SystemEvent::PermissionRequest {
            operation,
            tool_name,
            call_id,
        } => {
            app.state.mode = InputMode::Overlay;
            app.state.overlay.enqueue_approval(
                "Permission Required".to_string(),
                format!(
                    "Operation: {}\nTool: {}\nCall ID: {}\n\nPress 'a' to Approve or 'd' to Deny.",
                    operation, tool_name, call_id
                ),
                call_id,
            );
        }
        SystemEvent::ToolExecuted { tool, result } => {
            app.state.add_message(
                TranscriptItemKind::System,
                format!("Tool executed: {}", tool),
            );
            let output = if app.state.debug_mode {
                result
            } else {
                truncate_payload(&result, 100)
            };
            app.state
                .add_message(TranscriptItemKind::System, format!("Result: {}", output));
        }
        SystemEvent::Error { message } => {
            app.state
                .add_message(TranscriptItemKind::Error, format!("Error: {}", message));
        }
        SystemEvent::AgentStateChanged { state, .. } => {
            app.state.add_message(
                TranscriptItemKind::System,
                format!("Agent state: {}", state),
            );
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
            app.state.add_message(
                TranscriptItemKind::System,
                "Server shutting down".to_string(),
            );
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
        InputMode::AgentSelection => match key.code {
            KeyCode::Esc => {
                app.state.mode = InputMode::Normal;
                app.state.agent_selection = None;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if let Some(selection) = &mut app.state.agent_selection {
                    selection.select_prev();
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if let Some(selection) = &mut app.state.agent_selection {
                    selection.select_next();
                }
            }
            KeyCode::Enter => {
                if let Some(selection) = &app.state.agent_selection {
                    if let Some(agent) = selection.get_selected() {
                        return TuiInstruction::SwitchAgent(agent.id.clone(), agent.name.clone());
                    }
                }
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
                    app.state.selection.selected_message_index =
                        Some(app.state.transcript.items.len() - 1);
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
                                app.state.toast = Some(Toast::new(
                                    "Copy Failed".to_string(),
                                    ToastKind::Error,
                                    Duration::from_secs(2),
                                ));
                            } else {
                                app.state.toast = Some(Toast::new(
                                    "✓ Copied to clipboard".to_string(),
                                    ToastKind::Success,
                                    Duration::from_secs(2),
                                ));
                            }
                        } else {
                            app.state.toast = Some(Toast::new(
                                "Clipboard Unavailable".to_string(),
                                ToastKind::Error,
                                Duration::from_secs(2),
                            ));
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
                if let Some(cmd) = app
                    .state
                    .command_palette
                    .filtered_commands
                    .get(app.state.command_palette.selected_index)
                {
                    let cmd = cmd.clone();
                    app.state.mode = InputMode::Normal;
                    app.state.command_palette.reset();

                    // Check if it is a Builtin command
                    let cmd_type = app
                        .registry
                        .get(&cmd)
                        .or_else(|| cmd.strip_prefix('/').and_then(|s| app.registry.get(s)));

                    if let Some(sisyphus_core::command::CommandType::Builtin(c)) = cmd_type {
                        match c.name() {
                            "exit" | "quit" => {
                                app.quit();
                                return TuiInstruction::Quit;
                            }
                            "debug" => return TuiInstruction::ToggleDebug,
                            "clear" => return TuiInstruction::ClearSession,
                            "new" => return TuiInstruction::NewSession,
                            "agents" => return TuiInstruction::ShowAgentList,
                            _ => return TuiInstruction::DispatchCommand(cmd),
                        }
                    }

                    // For Remote commands or anything else, send as Chat
                    return TuiInstruction::Chat {
                        session_id: app.state.session_id.clone(),
                        input: cmd,
                    };
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
                    app.state.selection.selected_message_index =
                        Some(app.state.transcript.items.len() - 1);
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
                app.state.transcript.scroll_offset =
                    app.state.transcript.scroll_offset.saturating_sub(5);
            }
            KeyCode::PageDown => {
                app.state.transcript.scroll_offset =
                    app.state.transcript.scroll_offset.saturating_add(5);
            }
            KeyCode::End => {
                app.state.transcript.stick_to_bottom = true;
            }
            KeyCode::Enter => {
                if let Some(input) = app.state.get_input_and_clear() {
                    // Handle escape sequence // for literal /
                    if input.starts_with("//") {
                        let content = input[1..].to_string();
                        app.state.transcript.stick_to_bottom = true;
                        return TuiInstruction::Chat {
                            session_id: app.state.session_id.clone(),
                            input: content,
                        };
                    }

                    if input.starts_with('/') {
                        match parse_command(&input) {
                            Ok((cmd_name, _, _)) => {
                                // Check if it is a UiCommand (registered locally)
                                // Try exact match or match without leading slash
                                let cmd_type = app.registry.get(&cmd_name).or_else(|| {
                                    cmd_name.strip_prefix('/').and_then(|s| app.registry.get(s))
                                });

                                if let Some(sisyphus_core::command::CommandType::Builtin(cmd)) =
                                    cmd_type
                                {
                                    match cmd.name() {
                                        "exit" | "quit" => {
                                            app.quit();
                                            return TuiInstruction::Quit;
                                        }
                                        "debug" => return TuiInstruction::ToggleDebug,
                                        "clear" => return TuiInstruction::ClearSession,
                                        "new" => return TuiInstruction::NewSession,
                                        "agents" => return TuiInstruction::ShowAgentList,
                                        _ => return TuiInstruction::DispatchCommand(input),
                                    }
                                }
                                // For Remote commands or unrecognized commands, send as Chat
                                app.state.transcript.stick_to_bottom = true;
                                return TuiInstruction::Chat {
                                    session_id: app.state.session_id.clone(),
                                    input,
                                };
                            }
                            Err(e) => {
                                app.state.add_message(
                                    TranscriptItemKind::Error,
                                    format!("Command parse error: {}", e),
                                );
                                return TuiInstruction::None;
                            }
                        }
                    }

                    // Not a command (no leading slash), treat as Chat
                    app.state.transcript.stick_to_bottom = true;
                    return TuiInstruction::Chat {
                        session_id: app.state.session_id.clone(),
                        input,
                    };
                }
            }
            _ => {}
        },
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_routing_ui_commands() {
        let mut app = App::new("test-session".to_string());

        // Test /new
        app.state.input_buffer = "/new".to_string();
        let action = Action::Key(crossterm::event::KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::empty(),
        ));
        let instruction = update(&mut app, action);
        assert_eq!(instruction, TuiInstruction::NewSession);

        // Test /clear
        app.state.input_buffer = "/clear".to_string();
        let action = Action::Key(crossterm::event::KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::empty(),
        ));
        let instruction = update(&mut app, action);
        assert_eq!(instruction, TuiInstruction::ClearSession);

        // Test /debug
        app.state.input_buffer = "/debug".to_string();
        let action = Action::Key(crossterm::event::KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::empty(),
        ));
        let instruction = update(&mut app, action);
        assert_eq!(instruction, TuiInstruction::ToggleDebug);

        // Test /quit
        app.state.input_buffer = "/quit".to_string();
        let action = Action::Key(crossterm::event::KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::empty(),
        ));
        let instruction = update(&mut app, action);
        assert_eq!(instruction, TuiInstruction::Quit);
        assert!(app.should_quit);
    }

    #[test]
    fn test_routing_unknown_command() {
        let mut app = App::new("test-session".to_string());
        app.state.input_buffer = "/unknown".to_string();
        let action = Action::Key(crossterm::event::KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::empty(),
        ));
        let instruction = update(&mut app, action);

        // Should be sent as chat/slash command to server
        match instruction {
            TuiInstruction::Chat { session_id, input } => {
                assert_eq!(session_id, "test-session");
                assert_eq!(input, "/unknown");
            }
            _ => panic!("Expected Chat instruction for unknown command"),
        }
    }

    #[test]
    fn test_routing_chat() {
        let mut app = App::new("test-session".to_string());
        app.state.input_buffer = "hello world".to_string();
        let action = Action::Key(crossterm::event::KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::empty(),
        ));
        let instruction = update(&mut app, action);

        match instruction {
            TuiInstruction::Chat { session_id, input } => {
                assert_eq!(session_id, "test-session");
                assert_eq!(input, "hello world");
            }
            _ => panic!("Expected Chat instruction"),
        }
    }

    #[test]
    fn test_routing_help() {
        let mut app = App::new("test-session".to_string());
        app.state.input_buffer = "/help".to_string();
        let action = Action::Key(crossterm::event::KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::empty(),
        ));
        let instruction = update(&mut app, action);

        match instruction {
            TuiInstruction::DispatchCommand(cmd) => {
                assert_eq!(cmd, "/help");
            }
            _ => panic!("Expected DispatchCommand for help"),
        }
    }

    #[test]
    fn test_clear_token_usage() {
        let mut app = App::new("test-session".to_string());
        app.state.token_usage = "100 tokens".to_string();

        let _ = update(&mut app, Action::ClearHistory);

        assert_eq!(app.state.token_usage, "");
        assert!(app.state.transcript.items.is_empty());
    }

    #[test]
    fn test_new_session_clears_token_usage() {
        let mut app = App::new("test-session".to_string());
        app.state.token_usage = "100 tokens".to_string();

        let mut session = sisyphus_core::session::Session::new(None);
        session.id = "new-session".to_string();

        let _ = update(&mut app, Action::SessionCreated(session));

        assert_eq!(app.state.token_usage, "");
        assert_eq!(app.state.session_id, "new-session");
    }

    #[test]
    fn test_agent_flow() {
        use client::AgentResponse;
        let mut app = App::new("test-session".to_string());

        // 1. Test /agents command
        app.state.input_buffer = "/agents".to_string();
        let action = Action::Key(crossterm::event::KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::empty(),
        ));
        let instruction = update(&mut app, action);
        assert_eq!(instruction, TuiInstruction::ShowAgentList);

        // 2. Test receiving agent list
        let agents = vec![
            AgentResponse {
                id: "agent1".to_string(),
                name: "Agent 1".to_string(),
                model: "model1".to_string(),
                description: "desc1".to_string(),
            },
            AgentResponse {
                id: "agent2".to_string(),
                name: "Agent 2".to_string(),
                model: "model2".to_string(),
                description: "desc2".to_string(),
            },
        ];
        let action = Action::AgentListReceived(agents);
        let instruction = update(&mut app, action);
        assert_eq!(instruction, TuiInstruction::None);
        assert_eq!(app.state.mode, InputMode::AgentSelection);
        assert!(app.state.agent_selection.is_some());
        assert_eq!(
            app.state.agent_selection.as_ref().unwrap().selected_index,
            0
        );

        // 3. Test navigation
        let action = Action::Key(crossterm::event::KeyEvent::new(
            KeyCode::Down,
            KeyModifiers::empty(),
        ));
        update(&mut app, action);
        assert_eq!(
            app.state.agent_selection.as_ref().unwrap().selected_index,
            1
        );

        // 4. Test selection
        let action = Action::Key(crossterm::event::KeyEvent::new(
            KeyCode::Enter,
            KeyModifiers::empty(),
        ));
        let instruction = update(&mut app, action);
        assert_eq!(
            instruction,
            TuiInstruction::SwitchAgent("agent2".to_string(), "Agent 2".to_string())
        );

        // 5. Test switch confirmation
        let action = Action::AgentSwitched("agent2".to_string(), "Agent 2".to_string());
        update(&mut app, action);
        assert_eq!(app.state.mode, InputMode::Normal);
        assert!(app.state.agent_selection.is_none());
        assert!(app.state.toast.is_some());
    }
}
