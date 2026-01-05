use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
    routing::{get, post},
    Json, Router,
};
use common::bus::{EventBus, SystemEvent};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use sisyphus_core::agent::Agent;
use sisyphus_core::command::{CommandEffect, CommandInfo};
use sisyphus_core::service::ChatService;
use sisyphus_core::session::manager::SessionManager;
use sisyphus_core::session::{Session, SessionSummary};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub agent: Arc<Agent>,
    pub session_manager: Arc<SessionManager>,
    pub bus: Arc<EventBus>,
    pub chat_service: Arc<ChatService>,
}

pub struct Server {
    router: Router,
    port: u16,
    bus: Arc<EventBus>,
}

impl Server {
    pub fn new(
        port: u16,
        agent: Arc<Agent>,
        session_manager: Arc<SessionManager>,
        bus: Arc<EventBus>,
    ) -> Self {
        let chat_service = Arc::new(ChatService::new(
            agent.clone(),
            session_manager.clone(),
        ));
        let state = AppState {
            agent,
            session_manager,
            bus: bus.clone(),
            chat_service,
        };

        let router = Router::new()
            .route("/health", get(health_check))
            .route("/api/v1/sessions", get(list_sessions).post(create_session))
            .route("/api/v1/sessions/:id", get(get_session))
            .route("/api/v1/sessions/:id/chat", post(chat))
            .route(
                "/api/v1/sessions/:id/approvals/:call_id",
                post(submit_approval),
            )
            .route("/api/v1/commands", get(list_commands))
            .route("/api/v1/model", get(get_model))
            .route("/api/v1/events", get(events))
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::permissive())
            .with_state(state);

        Self { router, port, bus }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        // Use 127.0.0.1 explicitly to avoid issues with some environments preferring IPv6
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", self.port)).await?;

        let mut rx = self.bus.subscribe_raw();

        self.run_on_listener(listener, async move {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    tracing::info!("Received Ctrl+C, shutting down");
                }
                _ = async {
                    loop {
                        match rx.recv().await {
                            Ok(SystemEvent::Shutdown) => break,
                            Ok(_) => continue,
                            Err(tokio::sync::broadcast::error::RecvError::Lagged(skipped)) => {
                                tracing::warn!("Server event loop lagged, skipped {} events", skipped);
                                continue;
                            }
                            Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                                tracing::error!("Event bus closed unexpectedly");
                                break;
                            }
                        }
                    }
                    tracing::info!("Received Shutdown event, shutting down");
                } => {}
            }
        })
        .await
    }

    pub async fn run_with_signal<S>(self, signal: S) -> anyhow::Result<()>
    where
        S: std::future::Future<Output = ()> + Send + 'static,
    {
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", self.port)).await?;
        self.run_on_listener(listener, signal).await
    }

    pub async fn run_on_listener<S>(
        self,
        listener: tokio::net::TcpListener,
        signal: S,
    ) -> anyhow::Result<()>
    where
        S: std::future::Future<Output = ()> + Send + 'static,
    {
        tracing::info!("Server listening on {}", listener.local_addr()?);

        axum::serve(listener, self.router)
            .with_graceful_shutdown(signal)
            .await?;
        Ok(())
    }
}

async fn health_check() -> &'static str {
    "OK"
}

async fn list_sessions(State(state): State<AppState>) -> Json<Vec<SessionSummary>> {
    let session_locks = state.session_manager.list_sessions();
    let mut sessions = Vec::with_capacity(session_locks.len());
    for lock in session_locks {
        sessions.push(lock.read().await.summary());
    }
    Json(sessions)
}

async fn create_session(State(state): State<AppState>) -> Json<Session> {
    let session_lock = state.session_manager.create_session();
    let session = session_lock.read().await.clone();
    Json(session)
}

async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Session>, (axum::http::StatusCode, String)> {
    if let Some(session_lock) = state.session_manager.get_session(&id) {
        let session = session_lock.read().await.clone();
        Ok(Json(session))
    } else {
        Err((
            axum::http::StatusCode::NOT_FOUND,
            "Session not found".to_string(),
        ))
    }
}

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
}

#[derive(Serialize)]
struct ChatResponse {
    response: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    session_id: Option<String>,
    usage: Option<String>,
    model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    effect: Option<CommandEffect>,
}

async fn chat(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ChatRequest>,
) -> Result<Json<ChatResponse>, (axum::http::StatusCode, String)> {
    tracing::info!("Handling chat request for session {}", id);

    let outcome = state
        .chat_service
        .chat(&id, req.message)
        .await
        .map_err(|e| {
            tracing::error!("ChatService error: {:?}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    Ok(Json(ChatResponse {
        response: outcome.response,
        session_id: Some(outcome.session_id),
        usage: Some(outcome.usage),
        model: Some(outcome.model),
        effect: Some(outcome.effect),
    }))
}

#[derive(Serialize)]
struct ModelInfo {
    model: String,
}

async fn get_model(State(state): State<AppState>) -> Json<ModelInfo> {
    Json(ModelInfo {
        model: state.agent.model_name(),
    })
}

async fn list_commands(State(state): State<AppState>) -> Json<Vec<CommandInfo>> {
    Json(state.agent.list_commands())
}

#[derive(Deserialize)]
struct ApprovalRequest {
    decision: String,
}

async fn submit_approval(
    State(state): State<AppState>,
    Path((id, call_id)): Path<(String, String)>,
    Json(req): Json<ApprovalRequest>,
) -> Result<Json<ChatResponse>, (axum::http::StatusCode, String)> {
    tracing::info!("Handling approval for session {}, call {}", id, call_id);
    let approved = match req.decision.as_str() {
        "approve" => true,
        "deny" => false,
        _ => {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                "Invalid decision".to_string(),
            ))
        }
    };

    let outcome = state
        .chat_service
        .resolve_approval(&id, &call_id, approved)
        .await
        .map_err(|e| {
            tracing::error!("ChatService resolve_approval error: {:?}", e);
            (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        })?;

    Ok(Json(ChatResponse {
        response: outcome.response,
        session_id: Some(outcome.session_id),
        usage: Some(outcome.usage),
        model: Some(outcome.model),
        effect: Some(outcome.effect),
    }))
}

async fn events(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let rx = state.bus.subscribe_raw();

    let stream = stream::unfold(rx, |mut rx| async move {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    let data = serde_json::to_string(&event).unwrap_or_default();
                    return Some((Ok(Event::default().data(data)), rx));
                }
                Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => {
                    continue;
                }
                Err(tokio::sync::broadcast::error::RecvError::Closed) => {
                    return None;
                }
            }
        }
    });

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
