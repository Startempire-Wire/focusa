//! HTTP server setup.
//!
//! The API server is a thin read/write facade:
//!   - Reads: snapshot current state via Arc<RwLock<FocusaState>>
//!   - Writes: dispatch Actions via mpsc::Sender<Action> to the daemon event loop
//!
//! The daemon owns the state; the API borrows a read handle and a command channel.

use crate::middleware;
use crate::routes;
use crate::routes::sse::EventBroadcaster;
use axum::Router;
use axum::middleware as axum_mw;
use focusa_core::runtime::persistence_sqlite::SqlitePersistence;
use focusa_core::types::{Action, FocusaConfig, FocusaState};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, broadcast, mpsc};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommandExecutionStatus {
    Accepted,
    Dispatched,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandLogEntry {
    pub ts: chrono::DateTime<chrono::Utc>,
    pub level: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandRecord {
    pub command_id: String,
    pub command: String,
    pub status: CommandExecutionStatus,
    pub submitted_at: chrono::DateTime<chrono::Utc>,
    pub dispatched_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
    pub logs: Vec<CommandLogEntry>,
}

pub type CommandStore = Arc<RwLock<HashMap<String, CommandRecord>>>;

/// Shared state between API server and daemon.
pub struct AppState {
    /// Read-only snapshot of cognitive state (daemon writes, API reads).
    pub focusa: Arc<RwLock<FocusaState>>,
    /// Command channel to the daemon event loop.
    pub command_tx: mpsc::Sender<Action>,
    /// Event broadcast channel (SSE clients subscribe).
    pub events_tx: broadcast::Sender<String>,
    /// SSE event broadcaster for real-time TUI updates.
    pub event_broadcaster: EventBroadcaster,
    pub config: FocusaConfig,
    /// Direct persistence access for sync routes.
    pub persistence: SqlitePersistence,
    /// In-memory command write-model state for /v1/commands/* endpoints.
    pub command_store: CommandStore,
    /// Token store for capability permissions (docs/25-26).
    pub token_store: Arc<RwLock<focusa_core::permissions::TokenStore>>,
    /// Process start time for uptime reporting.
    pub started_at: Instant,
}

/// Build the axum Router with all routes.
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(routes::health::router())
        .merge(routes::info::router())
        .merge(routes::env::router())
        .merge(routes::commands::router())
        .merge(routes::capabilities::router())
        .merge(routes::capabilities_extra::router())
        .merge(routes::instances::router())
        .merge(routes::attachments::router())
        .merge(routes::sync::router())
        .merge(routes::focus::router())
        .merge(routes::gate::router())
        .merge(routes::ecs::router())
        .merge(routes::memory::router())
        .merge(routes::events_sqlite::router())
        .merge(routes::session::router())
        .merge(routes::proxy::router())
        .merge(routes::clt::router())
        .merge(routes::uxp::router())
        .merge(routes::autonomy::router())
        .merge(routes::constitution::router())
        .merge(routes::telemetry::router())
        .merge(routes::trust::router())
        .merge(routes::threads::router())
        .merge(routes::proposals::router())
        .merge(routes::rfm::router())
        .merge(routes::reflection::router())
        .merge(routes::skills::router())
        .merge(routes::training::router())
        .merge(routes::turn::router())
        .merge(routes::ascc::router())
        .merge(routes::tokens::router())
        .merge(routes::sse::router())
        .layer(axum_mw::from_fn(middleware::auth::auth_layer))
        .layer(axum_mw::from_fn(
            middleware::error_envelope::error_envelope_layer,
        ))
        .with_state(state)
}

fn scheduler_base_url(bind_addr: &str) -> String {
    let port = bind_addr.rsplit(':').next().unwrap_or("8787");
    format!("http://127.0.0.1:{}", port)
}

async fn reflection_scheduler_loop(base_url: String) {
    let client = reqwest::Client::new();

    loop {
        let delay_secs = {
            let scheduler_url = format!("{}/v1/reflect/scheduler", base_url);
            match client.get(&scheduler_url).send().await {
                Ok(resp) => match resp.json::<serde_json::Value>().await {
                    Ok(body) => {
                        let enabled = body
                            .get("enabled")
                            .and_then(|v| v.as_bool())
                            .unwrap_or(false);
                        let interval = body
                            .get("interval_seconds")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(3600)
                            .max(1);

                        if enabled {
                            let tick_url = format!("{}/v1/reflect/scheduler/tick", base_url);
                            let _ = client
                                .post(&tick_url)
                                .json(&serde_json::json!({}))
                                .send()
                                .await
                                .map(|r| {
                                    tracing::debug!(status = %r.status(), "reflection scheduler tick executed");
                                });
                            interval
                        } else {
                            30
                        }
                    }
                    Err(_) => 30,
                },
                Err(_) => 30,
            }
        };

        tokio::time::sleep(Duration::from_secs(delay_secs)).await;
    }
}

/// Start the API server on the configured bind address.
pub async fn run(
    focusa: Arc<RwLock<FocusaState>>,
    command_tx: mpsc::Sender<Action>,
    events_tx: broadcast::Sender<String>,
    config: FocusaConfig,
    persistence: SqlitePersistence,
) -> anyhow::Result<()> {
    let bind_addr = config.api_bind.clone();

    let broadcaster = EventBroadcaster::new();
    
    let state = Arc::new(AppState {
        focusa,
        command_tx,
        events_tx,
        event_broadcaster: broadcaster,
        config,
        persistence,
        command_store: Arc::new(RwLock::new(HashMap::new())),
        token_store: Arc::new(RwLock::new(focusa_core::permissions::TokenStore::new())),
        started_at: Instant::now(),
    });

    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    let scheduler_url = scheduler_base_url(&bind_addr);
    tokio::spawn(async move {
        // Delay one cycle to allow server readiness, then run continuously.
        reflection_scheduler_loop(scheduler_url).await;
    });

    tracing::info!("Listening on {}", bind_addr);
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::scheduler_base_url;

    #[test]
    fn scheduler_base_url_uses_localhost_port() {
        assert_eq!(
            scheduler_base_url("127.0.0.1:8787"),
            "http://127.0.0.1:8787"
        );
        assert_eq!(scheduler_base_url("0.0.0.0:9999"), "http://127.0.0.1:9999");
    }
}
