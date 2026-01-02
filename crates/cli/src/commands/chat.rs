use crate::server_manager::ServerManager;
use crate::ui::repl::Repl;
use crate::ui::tui::Tui;
use anyhow::Result;
use common::{bus::SystemEvent, config::Config};
use futures::StreamExt;
use reqwest_eventsource::Event;
use rust_i18n::t;

pub async fn run(_config: Config, config_path: Option<String>, use_tui: bool) -> Result<()> {
    println!("{}", t!("starting_agent"));

    // 1. Start Server
    // Use dynamic port
    let port = std::net::TcpListener::bind("127.0.0.1:0")?
        .local_addr()?
        .port();
    let mut server_manager = ServerManager::start(port, config_path).await?;
    let client = server_manager.client();

    // 2. Create Session
    let session = client.create_session().await?;
    println!("Session ID: {}", session.id);

    // 3. Subscribe to Events
    let mut events = client.subscribe_events()?;
    let (shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel(1);

    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            if let Ok(Event::Message(msg)) = event {
                tracing::debug!("Event: {:?}", msg);
                if let Ok(SystemEvent::Shutdown) = serde_json::from_str::<SystemEvent>(&msg.data) {
                    let _ = shutdown_tx.send(()).await;
                    break;
                }
            }
        }
    });

    // 4. Start Repl or TUI
    if use_tui {
        let mut tui = Tui::new(client, session.id, shutdown_rx);
        tui.run().await?;
    } else {
        let mut repl = Repl::new(client, session.id, shutdown_rx);
        repl.run().await?;
    }

    server_manager.stop().await?;
    Ok(())
}

pub async fn attach(url: String) -> Result<()> {
    let url = url::Url::parse(&url)?;
    let server_manager = ServerManager::connect(url).await?;
    let client = server_manager.client();

    let session = client.create_session().await?;
    println!("Session ID: {}", session.id);

    // Events
    let mut events = client.subscribe_events()?;
    let (_shutdown_tx, shutdown_rx) = tokio::sync::mpsc::channel(1);

    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            if let Ok(Event::Message(msg)) = event {
                tracing::debug!("Event: {:?}", msg);
            }
        }
    });

    let mut repl = Repl::new(client, session.id, shutdown_rx);
    repl.run().await?;

    Ok(())
}
