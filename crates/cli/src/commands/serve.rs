use crate::bootstrap;
use common::config::Config;
use rust_i18n::t;
use sisyphus_core::session::manager::SessionManager;
use std::sync::Arc;

pub async fn run(config: Config, port: u16) -> anyhow::Result<()> {
    println!("{}", t!("starting_server", port = port));

    let components = bootstrap::build_agent(&config).await?;

    let session_manager = Arc::new(tokio::sync::Mutex::new(SessionManager::new()));

    server::Server::new(port, components.agent, session_manager, components.bus)
        .run()
        .await?;

    Ok(())
}
