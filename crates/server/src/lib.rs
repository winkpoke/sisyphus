use std::sync::Arc;
use tokio::sync::Mutex;
use axum::{
    Router, 
    routing::{get, post},
    extract::{State, Path},
    response::sse::{Sse, Event},
    Json,
};
use tower_http::trace::TraceLayer;
use tower_http::cors::CorsLayer;
use sisyphus_core::agent::Agent;
use sisyphus_core::session::manager::SessionManager;
use sisyphus_core::session::Session;
use common::bus::EventBus;
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct AppState {
    pub agent: Arc<Agent>,
    pub session_manager: Arc<Mutex<SessionManager>>,
    pub bus: Arc<EventBus>,
}

pub struct Server {
    router: Router,
    port: u16,
}

impl Server {
    pub fn new(port: u16, agent: Arc<Agent>, session_manager: Arc<Mutex<SessionManager>>, bus: Arc<EventBus>) -> Self {
        let state = AppState {
            agent,
            session_manager,
            bus,
        };

        let router = Router::new()
            .route("/health", get(health_check))
            .route("/api/v1/sessions", get(list_sessions).post(create_session))
            .route("/api/v1/sessions/:id", get(get_session))
            .route("/api/v1/sessions/:id/chat", post(chat))
            .route("/api/v1/events", get(events))
            .layer(TraceLayer::new_for_http())
            .layer(CorsLayer::permissive())
            .with_state(state);

        Self {
            router,
            port,
        }
    }

    pub async fn run(self) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", self.port)).await?;
        tracing::info!("Server listening on {}", listener.local_addr()?);
        axum::serve(listener, self.router).await?;
        Ok(())
    }
}

async fn health_check() -> &'static str {
    "OK"
}

async fn list_sessions(State(state): State<AppState>) -> Json<Vec<Session>> {
    let manager = state.session_manager.lock().await;
    let sessions: Vec<Session> = manager.list_sessions().into_iter().cloned().collect();
    Json(sessions)
}

async fn create_session(State(state): State<AppState>) -> Json<Session> {
    let mut manager = state.session_manager.lock().await;
    let session = manager.create_session();
    Json(session.clone())
}

async fn get_session(State(state): State<AppState>, Path(id): Path<String>) -> Result<Json<Session>, (axum::http::StatusCode, String)> {
    let manager = state.session_manager.lock().await;
    if let Some(session) = manager.get_session(&id) {
        Ok(Json(session.clone()))
    } else {
        Err((axum::http::StatusCode::NOT_FOUND, "Session not found".to_string()))
    }
}

#[derive(Deserialize)]
struct ChatRequest {
    message: String,
}

#[derive(Serialize)]
struct ChatResponse {
    response: String,
}

async fn chat(
    State(state): State<AppState>, 
    Path(id): Path<String>, 
    Json(req): Json<ChatRequest>
) -> Result<Json<ChatResponse>, (axum::http::StatusCode, String)> {
    let mut manager = state.session_manager.lock().await;
    
    let session = manager.get_session_mut(&id)
        .ok_or((axum::http::StatusCode::NOT_FOUND, "Session not found".to_string()))?;

    let response = state.agent.chat(session, req.message).await
        .map_err(|e| (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ChatResponse { response }))
}

async fn events(State(state): State<AppState>) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let rx = state.bus.subscribe();
    
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
