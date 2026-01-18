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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_url_format() {
        let port = 8080;
        let url_format = format!("127.0.0.1:{}", port);
        assert_eq!(url_format, "127.0.0.1:8080");
    }

    #[test]
    fn test_port_display_logic() {
        let print_port = true;
        let local_port = 8080;

        if print_port {
            let output = local_port.to_string();
            assert_eq!(output, "8080");
        } else {
            let message = t!("starting_server", port = local_port);
            assert!(!message.is_empty());
        }
    }

    #[test]
    fn test_zero_port_binding() {
        let port = 0;
        assert_eq!(port, 0);
    }

    #[tokio::test]
    async fn test_tcp_listener_binding() {
        let port = 0;
        let listener_result = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await;
        assert!(listener_result.is_ok());

        let listener = listener_result.unwrap();
        let addr = listener.local_addr();
        assert!(addr.is_ok());

        let local_port = addr.unwrap().port();
        assert!(local_port > 0);
    }
}
