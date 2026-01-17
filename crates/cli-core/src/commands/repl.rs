use crate::commands::connection::setup_connection;
use crate::ui::repl::Repl;
use anyhow::Result;

pub async fn run(
    attach_url: Option<String>,
    config_path: Option<String>,
    log_file: Option<String>,
) -> Result<()> {
    let mut ctx = setup_connection(attach_url, config_path, log_file).await?;

    let mut repl = Repl::new(
        ctx.client.clone(),
        ctx.session_id.clone(),
        ctx.shutdown_rx,
    );
    repl.run().await?;

    if ctx.owns_server {
        ctx.server_manager.stop().await?;
    }

    Ok(())
}
