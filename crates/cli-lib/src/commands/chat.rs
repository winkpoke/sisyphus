use crate::server_manager::ServerManager;
use crate::ui::repl::Repl;
use anyhow::Result;
use client::Client;
use common::bus::SystemEvent;
use futures::StreamExt;
use reqwest_eventsource::Event;
use rust_i18n::t;
use tokio::sync::mpsc;

pub struct ChatContext {
    pub client: Client,
    pub session_id: String,
    pub shutdown_rx: mpsc::Receiver<()>,
    pub server_manager: ServerManager,
}

pub async fn setup_chat(config_path: Option<String>) -> Result<ChatContext> {
    println!("{}", t!("starting_agent"));

    let port = std::net::TcpListener::bind("127.0.0.1:0")?
        .local_addr()?
        .port();
    let server_manager = ServerManager::start(port, config_path).await?;
    let client = server_manager.client();

    let session = client.create_session().await?;
    println!("Session ID: {}", session.id);

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

    Ok(ChatContext {
        client,
        session_id: session.id,
        shutdown_rx,
        server_manager,
    })
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
