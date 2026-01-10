use crate::bootstrap;
use common::config::Config;
use rust_i18n::t;
use sisyphus_core::session::manager::SessionManager;
use std::sync::Arc;

pub async fn run(config: Config, port: u16) -> anyhow::Result<()> {
    println!("{}", t!("starting_server", port = port));

    let agents = bootstrap::build_builtins(&config).await?;

    let session_manager = Arc::new(SessionManager::new());
    let bus = agents.bus.clone();

    let registry = Arc::new(bootstrap::build_agent_registry(agents));

    server::Server::new(port, registry, session_manager, bus)
        .run()
        .await?;

    Ok(())
}
