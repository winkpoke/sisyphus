#[cfg(feature = "tui")]
use anyhow::Result;
#[cfg(feature = "tui")]
use sisyphus_cli_core::commands::connection::setup_connection;
#[cfg(feature = "tui")]
use tui::Tui;

#[cfg(feature = "tui")]
pub async fn run(
    attach_url: Option<String>,
    config_path: Option<String>,
    log_file: Option<String>,
) -> Result<()> {
    let mut ctx = setup_connection(attach_url, config_path, log_file).await?;
    let mut tui_app = Tui::new(
        ctx.client.clone(),
        ctx.session_id.clone(),
        ctx.shutdown_rx.resubscribe(),
    );
    tui_app.run().await?;

    if ctx.owns_server {
        ctx.server_manager.stop().await?;
    }

    Ok(())
}

#[cfg(not(feature = "tui"))]
pub async fn run(
    _attach_url: Option<String>,
    _config_path: Option<String>,
    _log_file: Option<String>,
) -> anyhow::Result<()> {
    eprintln!("Error: TUI feature not enabled. Build with --features tui");
    std::process::exit(1);
}
