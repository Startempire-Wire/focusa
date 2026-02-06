//! Focusa Daemon — long-lived process hosting cognitive state.
//!
//! Source: docs/G1-12-api.md
//!
//! Runs two concurrent tasks:
//!   1. Daemon event loop (single-writer state machine)
//!   2. HTTP API server (read state + dispatch commands)
//!
//! Default bind: 127.0.0.1:8787
//! No auth in MVP (localhost only).

mod middleware;
mod routes;
mod server;

use focusa_core::runtime::daemon::Daemon;
use focusa_core::types::{FocusaConfig, FocusaState};
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "focusa=info".into()),
        )
        .init();

    let mut config = FocusaConfig::default();

    // Allow overriding bind address via env (e.g., for Tailscale access from Mac).
    // FOCUSA_BIND=0.0.0.0:8787 or FOCUSA_BIND=100.94.238.56:8787
    if let Ok(bind) = std::env::var("FOCUSA_BIND") {
        config.api_bind = bind;
    }

    // Shared state: daemon writes after every reduction, API reads.
    let shared_state = Arc::new(RwLock::new(FocusaState::default()));

    // Event bus for SSE.
    let (events_tx, _events_rx) = tokio::sync::broadcast::channel::<String>(1024);

    // Initialize daemon (loads saved state from disk, syncs to shared_state on run).
    let mut daemon = Daemon::new(config.clone(), shared_state.clone())?;
    daemon.attach_event_bus(focusa_core::runtime::event_bus::EventBus::new(events_tx.clone()));
    let command_tx = daemon.command_sender();
    let events_tx_for_api = events_tx.clone();

    // Clone persistence for API server (sync routes need direct DB access).
    let persistence = daemon.persistence();

    // Spawn daemon event loop.
    let daemon_handle = tokio::spawn(async move {
        if let Err(e) = daemon.run().await {
            tracing::error!("Daemon error: {}", e);
        }
    });

    // Start API server (blocks until shutdown).
    let api_handle = tokio::spawn(async move {
        if let Err(e) = server::run(shared_state, command_tx, events_tx_for_api, config, persistence).await {
            tracing::error!("API server error: {}", e);
        }
    });

    // Wait for either to finish (normally neither should).
    tokio::select! {
        _ = daemon_handle => tracing::warn!("Daemon exited"),
        _ = api_handle => tracing::warn!("API server exited"),
    }

    Ok(())
}
