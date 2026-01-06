use axum::{
    extract::{Path, State},
    response::sse::{Event, Sse},
    routing::{get, post, put},
    Json, Router,
};
use common::bus::{EventBus, SystemEvent};
use futures::stream::{self, Stream};
use serde::{Deserialize, Serialize};
use sisyphus_core::agent::registry::AgentRegistry;
use sisyphus_core::command::CommandInfo;
use sisyphus_core::service::ChatService;
use sisyphus_core::session::manager::SessionManager;
use sisyphus_core::session::{Session, SessionSummary};
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;

#[derive(Clone)]
pub struct AppState {
    pub registry: Arc<AgentRegistry>,
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
        registry: Arc<AgentRegistry>,
        session_manager: Arc<SessionManager>,
        bus: Arc<EventBus>,
    ) -> Self {
        let chat_service = Arc::new(ChatService::new(registry.clone(), session_manager.clone()));
        let state = AppState {
            registry,
            session_manager,
            bus: bus.clone(),
            chat_service,
        };

        let router = Router::new()
            .route("/health", get(health_check))
            .route("/api/v1/sessions", get(list_sessions).post(create_session))
            .route("/api/v1/sessions/:id", get(get_session))
            .route("/api/v1/sessions/:id/agent", put(update_session_agent))
            .route("/api/v1/sessions/:id/chat", post(chat))
            .route("/api/v1/sessions/:id/clear", post(clear_session))
            .route(
                "/api/v1/sessions/:id/approvals/:call_id",
                post(submit_approval),
            )
            .route("/api/v1/agents", get(list_agents))
            .route("/api/v1/agents/:id", get(get_agent))
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

#[derive(Deserialize)]
struct CreateSessionRequest {
    agent_id: Option<String>,
}

async fn create_session(
    State(state): State<AppState>,
    json: Option<Json<CreateSessionRequest>>,
) -> Result<Json<Session>, (axum::http::StatusCode, String)> {
    let agent_id = json.and_then(|j| j.agent_id.clone());

    // Validate agent_id if provided
    if let Some(ref aid) = agent_id {
        if state.registry.get_agent(aid).is_none() {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                format!("Agent not found: {}", aid),
            ));
        }
    }

    let session_lock = state.session_manager.create_session(agent_id);
    let session = session_lock.read().await.clone();
    Ok(Json(session))
}

#[derive(Deserialize)]
struct UpdateSessionAgentRequest {
    agent_id: String,
}

async fn update_session_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<UpdateSessionAgentRequest>,
) -> Result<Json<Session>, (axum::http::StatusCode, String)> {
    if state.registry.get_agent(&req.agent_id).is_none() {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            format!("Agent not found: {}", req.agent_id),
        ));
    }

    let session_lock = state.session_manager.get_session(&id).ok_or_else(|| {
        (
            axum::http::StatusCode::NOT_FOUND,
            "Session not found".to_string(),
        )
    })?;

    let mut session = session_lock.write().await;
    if session.status == sisyphus_core::session::SessionStatus::Busy {
        return Err((
            axum::http::StatusCode::CONFLICT,
            "Session is busy".to_string(),
        ));
    }

    session.agent_id = Some(req.agent_id);
    Ok(Json(session.clone()))
}

#[derive(Serialize)]
struct AgentResponse {
    id: String,
    model: String,
    name: String,
    description: String,
}

async fn list_agents(State(state): State<AppState>) -> Json<Vec<AgentResponse>> {
    let agents = state.registry.list_agents();
    let mut responses: Vec<AgentResponse> = agents
        .into_iter()
        .map(|(id, agent)| {
            let config = agent.config();
            AgentResponse {
                id,
                model: agent.model_name(),
                name: config.name.clone(),
                description: config.description.clone(),
            }
        })
        .collect();
    // Sort by ID for stable output
    responses.sort_by(|a, b| a.id.cmp(&b.id));
    Json(responses)
}

async fn get_agent(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<AgentResponse>, (axum::http::StatusCode, String)> {
    if let Some(agent) = state.registry.get_agent(&id) {
        let config = agent.config();
        Ok(Json(AgentResponse {
            id,
            model: agent.model_name(),
            name: config.name.clone(),
            description: config.description.clone(),
        }))
    } else {
        Err((
            axum::http::StatusCode::NOT_FOUND,
            "Agent not found".to_string(),
        ))
    }
}

async fn clear_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<Session>, (axum::http::StatusCode, String)> {
    let session_lock = state.session_manager.get_session(&id).ok_or_else(|| {
        (
            axum::http::StatusCode::NOT_FOUND,
            "Session not found".to_string(),
        )
    })?;

    let mut session = session_lock.write().await;
    session.clear_context();
    Ok(Json(session.clone()))
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
    agent_id: Option<String>,
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
        agent_id: Some(outcome.agent_id),
    }))
}

#[derive(Serialize)]
struct ModelInfo {
    model: String,
}

async fn get_model(State(state): State<AppState>) -> Json<ModelInfo> {
    Json(ModelInfo {
        model: state.registry.get_default_agent().model_name(),
    })
}

async fn list_commands(State(state): State<AppState>) -> Json<Vec<CommandInfo>> {
    Json(state.registry.get_default_agent().list_commands())
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
        agent_id: Some(outcome.agent_id),
    }))
}

async fn events(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = Result<Event, axum::Error>>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

    // Subscribe to all events and forward them to the channel
    let sub = state.bus.subscribe_all(move |event| {
        let _ = tx.send(event);
    });

    // Convert receiver into a stream
    let stream = stream::unfold((rx, sub), |(mut rx, _sub)| async move {
        match rx.recv().await {
            Some(event) => {
                let data = serde_json::to_string(&event).unwrap_or_default();
                Some((Ok(Event::default().data(data)), (rx, _sub)))
            }
            None => None,
        }
    });

    Sse::new(stream).keep_alive(axum::response::sse::KeepAlive::default())
}
