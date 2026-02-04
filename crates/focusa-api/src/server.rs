//! HTTP server setup.

use crate::routes;
use axum::Router;
use focusa_core::types::{FocusaConfig, FocusaState};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Shared daemon state.
pub struct AppState {
    pub focusa: RwLock<FocusaState>,
    pub config: FocusaConfig,
}

pub async fn run() -> anyhow::Result<()> {
    let config = FocusaConfig::default();
    let bind_addr = config.api_bind.clone();

    let state = Arc::new(AppState {
        focusa: RwLock::new(FocusaState::new()),
        config,
    });

    let app = Router::new()
        .merge(routes::health::router())
        .merge(routes::focus::router())
        .merge(routes::gate::router())
        .merge(routes::ecs::router())
        .merge(routes::memory::router())
        .merge(routes::events::router())
        .merge(routes::session::router())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;
    tracing::info!("Listening on {}", bind_addr);
    axum::serve(listener, app).await?;

    Ok(())
}
