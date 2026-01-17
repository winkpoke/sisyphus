use crate::commands::connection::setup_connection;
use anyhow::Result;

pub async fn run(
    attach_url: Option<String>,
    message: String,
    config_path: Option<String>,
    log_file: Option<String>,
) -> Result<()> {
    let message = message.trim();

    if message.is_empty() {
        eprintln!("Error: Message cannot be empty");
        std::process::exit(1);
    }

    let mut ctx = setup_connection(attach_url, config_path, log_file).await?;

    let chat_resp = ctx
        .client
        .chat(&ctx.session_id, message.to_string())
        .await?;

    if !chat_resp.response.is_empty() {
        println!("{}", chat_resp.response);
    }

    if ctx.owns_server {
        ctx.server_manager.stop().await?;
    }

    Ok(())
}
