pub mod action;
pub mod app;
pub mod commands;
pub mod event;
pub mod state;
pub mod terminal;
pub mod theme;
pub mod transcript;
pub mod ui;
pub mod update;

use anyhow::Result;
use client::Client;
use common::bus::SystemEvent;
use event::EventHandler;
use futures::StreamExt;
use sisyphus_core::command::{CommandContext, CommandType};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::broadcast;

use action::Action;
use app::App;
use update::{update, TuiInstruction};

pub struct Tui {
    client: Client,
    shutdown_rx: broadcast::Receiver<()>,
    session_id: String,
}

impl Tui {
    pub fn new(client: Client, session_id: String, shutdown_rx: broadcast::Receiver<()>) -> Self {
        Self {
            client,
            shutdown_rx,
            session_id,
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

        // Init App and Event System
        let mut app = App::new(self.session_id.clone());
        let mut events = EventHandler::new(Duration::from_millis(250));
        let action_tx = events.sender();

        // Fetch initial model info
        if let Ok(model) = self.client.get_model().await {
            app.state.active_model = model;
        }

        // Fetch commands from server and merge with local builtins
        let mut commands = app
            .registry
            .list()
            .iter()
            .map(|c| {
                if c.name.starts_with('/') {
                    c.name.clone()
                } else {
                    format!("/{}", c.name)
                }
            })
            .collect::<Vec<_>>();

        if let Ok(server_commands) = self.client.get_commands().await {
            // We can safely get mutable access here as we haven't spawned tasks yet
            if let Some(registry) = Arc::get_mut(&mut app.registry) {
                for cmd in server_commands {
                    // Prepend / to name if not present (SlashCommand convention)
                    let name = if cmd.name.starts_with('/') {
                        cmd.name.clone()
                    } else {
                        format!("/{}", cmd.name)
                    };

                    // Register as Remote so HelpCommand can see it
                    let mut info = cmd.clone();
                    info.name = name.clone();
                    registry.register_remote(info);

                    if !commands.contains(&name) {
                        commands.push(name);
                    }
                }
            }
        }
        commands.sort();
        app.state.update_commands(commands);

        // Spawn System Event Listener
        let mut backend_events = self.client.subscribe_events()?;
        let tx_clone = action_tx.clone();
        tokio::spawn(async move {
            while let Some(event) = backend_events.next().await {
                match event {
                    Ok(reqwest_eventsource::Event::Open) => continue,
                    Ok(reqwest_eventsource::Event::Message(msg)) => {
                        if let Ok(sys_event) = serde_json::from_str::<SystemEvent>(&msg.data) {
                            let _ = tx_clone.send(Action::SystemEvent(sys_event));
                        }
                    }
                    Err(_) => {
                        // Ignore transient errors
                    }
                }
            }
        });

        // Main Loop
        loop {
            terminal.draw(|f| ui::draw(f, &app))?;

            tokio::select! {
                Some(action) = events.next() => {
                    let instruction = update(&mut app, action);
                    match instruction {
                        TuiInstruction::Quit => break,
                        TuiInstruction::Chat { session_id, input } => {
                            let _ = action_tx.send(Action::MessageSent(input.clone()));
                            let client = self.client.clone();
                            let tx = action_tx.clone();
                            tokio::spawn(async move {
                                match client.chat(&session_id, input).await {
                                    Ok(resp) => { let _ = tx.send(Action::ResponseReceived(resp)); }
                                    Err(e) => { let _ = tx.send(Action::Error(e.to_string())); }
                                }
                            });
                        }
                        TuiInstruction::SubmitApproval { session_id, call_id, decision } => {
                             let client = self.client.clone();
                             let tx = action_tx.clone();
                             tokio::spawn(async move {
                                 match client.submit_approval(&session_id, &call_id, &decision).await {
                                     Ok(resp) => { let _ = tx.send(Action::ResponseReceived(resp)); }
                                     Err(e) => { let _ = tx.send(Action::Error(e.to_string())); }
                                 }
                             });
                        }
                        TuiInstruction::DispatchCommand(input) => {
                             let registry = app.registry.clone();
                             let event_bus = app.event_bus.clone();
                             let session_id = app.state.session_id.clone();
                             let tx = action_tx.clone();

                             tokio::spawn(async move {
                                 let (cmd_name, args) = match sisyphus_core::command::parser::parse_command(&input) {
                                     Ok((cmd, args, _)) => (cmd, args),
                                     Err(e) => {
                                         let _ = tx.send(Action::Error(format!("Command parse error: {}", e)));
                                         return;
                                     }
                                 };

                                let cmd_type = registry.get(&cmd_name).or_else(|| {
                                    cmd_name.strip_prefix('/').and_then(|s| registry.get(s))
                                });

                                if let Some(cmd_type) = cmd_type {
                                    match cmd_type {
                                        CommandType::Builtin(cmd) => {
                                             let ctx = CommandContext {
                                                 session_id,
                                                 event_bus,
                                                 registry: &registry,
                                             };

                                             match cmd.execute(&ctx, args).await {
                                                 Ok(outcome) => {
                                                      let _ = tx.send(Action::CommandResult(Box::new(outcome)));
                                                 }
                                                 Err(e) => {
                                                      let _ = tx.send(Action::Error(e.to_string()));
                                                 }
                                             }
                                         }
                                         CommandType::Custom(_) => {}
                                         CommandType::Remote(_) => {}
                                     }
                                 }
                             });
                        }
                        TuiInstruction::NewSession => {
                             let client = self.client.clone();
                             let tx = action_tx.clone();
                             tokio::spawn(async move {
                                 match client.create_session().await {
                                     Ok(session) => {
                                         let _ = tx.send(Action::SessionCreated(session));
                                     }
                                     Err(e) => {
                                         let _ = tx.send(Action::Error(e.to_string()));
                                     }
                                 }
                             });
                        }
                        TuiInstruction::ClearSession => {
                             let client = self.client.clone();
                             let tx = action_tx.clone();
                             let session_id = app.state.session_id.clone();
                             tokio::spawn(async move {
                                 match client.clear_session(&session_id).await {
                                     Ok(_) => {
                                         let _ = tx.send(Action::ClearHistory);
                                     }
                                     Err(e) => {
                                         let _ = tx.send(Action::Error(e.to_string()));
                                     }
                                 }
                             });
                        }
                        TuiInstruction::ShowAgentList => {
                            let client = self.client.clone();
                            let tx = action_tx.clone();
                            tokio::spawn(async move {
                                match client.list_agents().await {
                                    Ok(agents) => {
                                        let _ = tx.send(Action::AgentListReceived(agents));
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Action::Error(e.to_string()));
                                    }
                                }
                            });
                        }
                        TuiInstruction::SwitchAgent(agent_id, agent_name) => {
                            let client = self.client.clone();
                            let tx = action_tx.clone();
                            let session_id = app.state.session_id.clone();
                            tokio::spawn(async move {
                                match client.update_session_agent(&session_id, &agent_id).await {
                                    Ok(_) => {
                                        let _ = tx.send(Action::AgentSwitched(agent_id, agent_name));
                                    }
                                    Err(e) => {
                                        let _ = tx.send(Action::Error(e.to_string()));
                                    }
                                }
                            });
                        }
                        TuiInstruction::ToggleDebug => {
                            let _ = action_tx.send(Action::ToggleDebug);
                        }
                        TuiInstruction::None => {}
                    }
                    if app.should_quit {
                        break;
                    }
                }
                _ = self.shutdown_rx.recv() => {
                    break;
                }
            }
        }

        terminal::restore()?;
        Ok(())
    }
}
