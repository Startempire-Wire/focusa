//! Compaction policy controller status surface (#112 slice 4-lite).
//!
//! Exposes the controller state (active lease, shadow history, quarantine
//! set) for observability. The full adaptive loop lands with the epoch
//! scheduler; this route is the read surface.

use axum::extract::State;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use std::sync::Arc;

use crate::server::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/compaction/controller-status", get(status))
}

fn state_path(state: &Arc<AppState>) -> std::path::PathBuf {
    std::path::PathBuf::from(&state.config.data_dir)
        .join(".focusa")
        .join("compaction-controller.json")
}

async fn status(State(state): State<Arc<AppState>>) -> Json<Value> {
    let path = state_path(&state);
    let controller = focusa_core::compaction_policy::load_controller_state(&path);
    Json(json!({
        "schema": "focusa.compaction_controller_status.v1",
        "epochs_seen": controller.epochs_seen,
        "active_policy": controller
            .active_lease
            .as_ref()
            .and_then(|lease| serde_json::to_value(lease.policy).ok()),
        "shadow_history_entries": controller.shadow_history.len(),
        "quarantine": controller.quarantine,
    }))
}
