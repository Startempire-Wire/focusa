//! HTTP server setup.
//!
//! The API server is a thin read/write facade:
//!   - Reads: snapshot current state via Arc<RwLock<FocusaState>>
//!   - Writes: dispatch Actions via mpsc::Sender<Action> to the daemon event loop
//!
//! The daemon owns the state; the API borrows a read handle and a command channel.

use crate::middleware;
use crate::routes;
use axum::Router;
use axum::middleware as axum_mw;
use focusa_core::types::{Action, FocusaConfig, FocusaState};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

/// Shared state between API server and daemon.
pub struct AppState {
    /// Read-only snapshot of cognitive state (daemon writes, API reads).
    pub focusa: Arc<RwLock<FocusaState>>,
    /// Command channel to the daemon event loop.
    pub command_tx: mpsc::Sender<Action>,
    /// Event broadcast channel (SSE clients subscribe).
    pub events_tx: broadcast::Sender<String>,
    pub config: FocusaConfig,
}

/// Build the axum Router with all routes.
pub fn build_router(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(routes::health::router())
        .merge(routes::info::router())
        .merge(routes::instances::router())
        .merge(routes::focus::router())
        .merge(routes::gate::router())
        .merge(routes::ecs::router())
        .merge(routes::memory::router())
        .merge(routes::events_sqlite::router())
        .merge(routes::events_stream::router())
        .merge(routes::session::router())
        .merge(routes::proxy::router())
        .merge(routes::clt::router())
        .merge(routes::uxp::router())
        .merge(routes::autonomy::router())
        .merge(routes::constitution::router())
        .merge(routes::telemetry::router())
        .merge(routes::threads::router())
        .merge(routes::proposals::router())
        .merge(routes::rfm::router())
        .merge(routes::skills::router())
        .merge(routes::training::router())
        .merge(routes::turn::router())
        .merge(routes::ascc::router())
        .layer(axum_mw::from_fn(middleware::auth::auth_layer))
        .with_state(state)
}

/// Start the API server on the configured bind address.
pub async fn run(
    focusa: Arc<RwLock<FocusaState>>,
    command_tx: mpsc::Sender<Action>,
    events_tx: broadcast::Sender<String>,
    config: FocusaConfig,
) -> anyhow::Result<()> {
    let bind_addr = config.api_bind.clone();

    let state = Arc::new(AppState {
        focusa,
        command_tx,
        events_tx,
        config,
    });

    let app = build_router(state);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Listening on {}", bind_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
