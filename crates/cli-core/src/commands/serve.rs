use crate::bootstrap;
use common::config::Config;
use rust_i18n::t;
use sisyphus_core::session::manager::SessionManager;
use std::sync::Arc;
use tokio::net::TcpListener;

pub async fn run(
    config: Config,
    port: u16,
    print_port: bool,
    _log_file: Option<String>,
) -> anyhow::Result<()> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    let local_port = listener.local_addr()?.port();

    if print_port {
        println!("{}", local_port);
    } else {
        println!("{}", t!("starting_server", port = local_port));
    }

    let agents = bootstrap::build_builtins(&config).await?;

    let session_manager = Arc::new(SessionManager::new());
    let bus = agents.bus.clone();

    let registry = Arc::new(bootstrap::build_agent_registry(agents));

    server::Server::new(local_port, registry, session_manager, bus)
        .run_on_listener(listener, async {
            let _ = tokio::signal::ctrl_c().await;
        })
        .await?;

    Ok(())
}
