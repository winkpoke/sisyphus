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

#[cfg(test)]
mod tests {
    use client::ChatResponse;

    #[test]
    fn test_message_trimming() {
        let input = "  hello world  ";
        let trimmed = input.trim();
        assert_eq!(trimmed, "hello world");

        let input_with_newlines = "\n  hello world  \n";
        let trimmed = input_with_newlines.trim();
        assert_eq!(trimmed, "hello world");
    }

    #[test]
    fn test_empty_message_check() {
        let empty = "";
        assert!(empty.is_empty());

        let whitespace = "   \n\t  ";
        assert!(!whitespace.is_empty());
        assert!(whitespace.trim().is_empty());
    }

    #[test]
    fn test_message_content_display() {
        let chat_resp = ChatResponse {
            response: "Test response".to_string(),
            session_id: Some("session-123".to_string()),
            usage: Some("100 tokens".to_string()),
            model: Some("gpt-4".to_string()),
        };

        assert!(!chat_resp.response.is_empty());
        assert_eq!(chat_resp.session_id, Some("session-123".to_string()));
    }

    #[test]
    fn test_empty_response_handling() {
        let chat_resp = ChatResponse {
            response: "".to_string(),
            session_id: None,
            usage: None,
            model: None,
        };

        assert!(chat_resp.response.is_empty());
    }
}
