use common::bus::EventBus;
use provider::mock::MockProvider;
use server::Server;
use sisyphus_core::agent::{config::AgentConfig, Agent};
use sisyphus_core::session::manager::SessionManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

fn init_tracing() {
    let _ = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .try_init();
}

#[tokio::test]
async fn test_server_health() {
    init_tracing();
    let bus = Arc::new(EventBus::new(100));
    let provider = Box::new(MockProvider::new());
    let agent = Arc::new(Agent::new(provider, bus.clone(), AgentConfig::default()));
    let session_manager = Arc::new(SessionManager::new());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = Server::new(port, agent, session_manager, bus);

    tokio::spawn(async move {
        if let Err(e) = server
            .run_on_listener(listener, std::future::pending::<()>())
            .await
        {
            eprintln!("Server error: {:?}", e);
        }
    });

    // Give it a moment to start
    tokio::time::sleep(Duration::from_millis(500)).await;

    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(0)
        .build()
        .unwrap();
    let resp = client
        .get(format!("http://localhost:{}/health", port))
        .send()
        .await
        .expect("Failed to send request");

    assert!(resp.status().is_success());
    let text = resp.text().await.unwrap();
    assert_eq!(text, "OK");
}

#[tokio::test]
async fn test_session_flow() {
    init_tracing();
    let bus = Arc::new(EventBus::new(100));
    let provider = Box::new(MockProvider::new());
    let agent = Arc::new(Agent::new(provider, bus.clone(), AgentConfig::default()));
    let session_manager = Arc::new(SessionManager::new());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = Server::new(port, agent, session_manager, bus);

    tokio::spawn(async move {
        if let Err(e) = server
            .run_on_listener(listener, std::future::pending::<()>())
            .await
        {
            eprintln!("Server error: {:?}", e);
        }
    });

    tokio::time::sleep(Duration::from_millis(500)).await;
    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(0)
        .build()
        .unwrap();
    let base_url = format!("http://localhost:{}", port);

    // 1. List sessions (should be empty)
    let resp = client
        .get(format!("{}/api/v1/sessions", base_url))
        .send()
        .await
        .expect("Failed to list sessions");
    assert!(resp.status().is_success());
    let sessions: Vec<serde_json::Value> = resp.json().await.unwrap();
    assert_eq!(sessions.len(), 0);

    // 2. Create session
    let resp = client
        .post(format!("{}/api/v1/sessions", base_url))
        .send()
        .await
        .expect("Failed to create session");
    assert!(resp.status().is_success());
    let session: serde_json::Value = resp.json().await.unwrap();
    let session_id = session["id"].as_str().unwrap();

    // 3. Get session
    let resp = client
        .get(format!("{}/api/v1/sessions/{}", base_url, session_id))
        .send()
        .await
        .expect("Failed to get session");
    assert!(resp.status().is_success());

    // 4. Chat
    let chat_body = serde_json::json!({
        "message": "Hello"
    });
    let resp = client
        .post(format!("{}/api/v1/sessions/{}/chat", base_url, session_id))
        .json(&chat_body)
        .send()
        .await
        .expect("Failed to chat");

    assert!(resp.status().is_success());
    let chat_resp: serde_json::Value = resp.json().await.unwrap();
    assert!(chat_resp["response"].is_string());

    // Check if session history updated
    let resp = client
        .get(format!("{}/api/v1/sessions/{}", base_url, session_id))
        .send()
        .await
        .expect("Failed to get session after chat");
    let session: serde_json::Value = resp.json().await.unwrap();
    let entries = session["context"]["entries"].as_array().unwrap();
    assert!(entries.len() >= 1); // Should have at least one turn
}

#[tokio::test]
async fn test_exit_command() {
    init_tracing();
    let bus = Arc::new(EventBus::new(100));
    let provider = Box::new(MockProvider::new());
    let agent = Arc::new(Agent::new(provider, bus.clone(), AgentConfig::default()));
    let session_manager = Arc::new(SessionManager::new());

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();

    let server = Server::new(port, agent, session_manager, bus);

    tokio::spawn(async move {
        if let Err(e) = server
            .run_on_listener(listener, std::future::pending::<()>())
            .await
        {
            eprintln!("Server error: {:?}", e);
        }
    });

    tokio::time::sleep(Duration::from_millis(500)).await;
    let client = reqwest::Client::builder()
        .pool_max_idle_per_host(0)
        .build()
        .unwrap();
    let base_url = format!("http://localhost:{}", port);

    // Create session
    let resp = client
        .post(format!("{}/api/v1/sessions", base_url))
        .send()
        .await
        .expect("Failed to create session");
    let session: serde_json::Value = resp.json().await.unwrap();
    let session_id = session["id"].as_str().unwrap();

    // Send /exit command
    let chat_body = serde_json::json!({
        "message": "/exit"
    });
    let resp = client
        .post(format!("{}/api/v1/sessions/{}/chat", base_url, session_id))
        .json(&chat_body)
        .send()
        .await
        .expect("Failed to send /exit");

    assert!(resp.status().is_success());
    let chat_resp: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(chat_resp["response"], "");

    // Verify server is still running by making another request
    let resp = client
        .get(format!("{}/health", base_url))
        .send()
        .await
        .expect("Failed to check health after /exit");
    assert!(resp.status().is_success());
}
