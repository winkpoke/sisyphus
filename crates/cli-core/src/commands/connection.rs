use crate::server_manager::ServerManager;
use anyhow::{Context, Result};
use client::Client;
use tokio::sync::broadcast;
use url::Url;

pub struct ConnectionContext {
    pub client: Client,
    pub session_id: String,
    pub server_manager: ServerManager,
    pub owns_server: bool,
    pub shutdown_tx: broadcast::Sender<()>,
    pub shutdown_rx: broadcast::Receiver<()>,
}

pub async fn setup_connection(
    attach_url: Option<String>,
    config_path: Option<String>,
    log_file: Option<String>,
) -> Result<ConnectionContext> {
    let (tx, rx) = broadcast::channel(1);

    // If attach_url is provided, connect to existing server
    if let Some(url_str) = attach_url {
        let url = Url::parse(&url_str).context("Failed to parse attach URL")?;
        let server_manager = ServerManager::connect(url.clone()).await?;
        let client = Client::new(url);

        // Create a session
        let session = client
            .create_session()
            .await
            .context("Failed to create session")?;

        return Ok(ConnectionContext {
            client,
            session_id: session.id,
            server_manager,
            owns_server: false,
            shutdown_tx: tx,
            shutdown_rx: rx,
        });
    }

    // Otherwise, start a new server
    // 0 means let OS choose port
    let port = 0;
    let server_manager = ServerManager::start(port, config_path, log_file).await?;
    let client = server_manager.client();

    // Create a session
    let session = client
        .create_session()
        .await
        .context("Failed to create session")?;

    Ok(ConnectionContext {
        client,
        session_id: session.id,
        server_manager,
        owns_server: true,
        shutdown_tx: tx,
        shutdown_rx: rx,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // We skip this test as it requires the server binary to be built and available
    // #[tokio::test]
    // async fn test_setup_connection_local() {
    //     let ctx = setup_connection(None, None, None).await;
    //     assert!(ctx.is_ok());
    //     let ctx = ctx.unwrap();
    //     let client = ctx.client.clone();
    //     drop(ctx);
    //     client
    //         .health_check()
    //         .await
    //         .expect("Server should still be running");
    // }

    #[test]
    fn test_url_parsing() {
        let valid_url = "http://localhost:3000";
        assert!(Url::parse(valid_url).is_ok());

        let invalid_url = "not-a-url";
        assert!(Url::parse(invalid_url).is_err());
    }
}
