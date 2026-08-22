//! Compaction policy controller status surface (#112 slice 4-lite).
//!
//! Exposes the controller state (active lease, shadow history, quarantine
//! set) for observability. The full adaptive loop lands with the epoch
//! scheduler; this route is the read surface.

use axum::extract::State;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::server::AppState;
use focusa_core::compaction_policy::{
    ControllerState, EpochLease, Policy, RuntimeFacts, Transition, append_epoch_history,
    compute_mask, evaluate_shadow, load_controller_state_sqlite, next_transition,
    save_controller_state_sqlite,
};

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/compaction/controller-status", get(status))
        .route("/v1/compaction/controller-epoch", post(apply_epoch))
}

#[derive(Deserialize)]
pub struct EpochRequest {
    pub epoch_id: String,
    pub facts: RuntimeFacts,
    pub outcome: focusa_core::compaction_policy::OutcomeMetrics,
    /// Target for shadow evaluation; defaults to the next lattice step.
    pub shadow_target: Option<Policy>,
    pub promotion_window: Option<usize>,
    pub quarantine_epochs: Option<u64>,
}

/// The controller loop's decision point: compute the mask, shadow-evaluate
/// the candidate, run the deterministic transition, seal the lease, and
/// persist state + history. Pure functions from the core module own every
/// decision; this route only wires persistence.
async fn apply_epoch(
    State(state): State<Arc<AppState>>,
    Json(request): Json<EpochRequest>,
) -> Json<Value> {
    let db = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let result = tokio::task::spawn_blocking(move || -> anyhow::Result<Value> {
        let conn = rusqlite::Connection::open(db)?;
        let mut controller: ControllerState = load_controller_state_sqlite(&conn)?;
        let mask = compute_mask(&request.facts);
        let active_policy = controller.active_lease.as_ref().map(|lease| lease.policy);
        let candidate = request.shadow_target.unwrap_or_else(|| match active_policy {
            Some(policy) => Policy::ALL
                .get(policy.rank().saturating_add(1) as usize)
                .copied()
                .unwrap_or(policy),
            None => Policy::WarnOnly,
        });
        let shadow = evaluate_shadow(candidate, &request.outcome, &request.epoch_id);
        let (transition, target) = next_transition(
            &controller,
            &request.outcome,
            Some(&shadow),
            request.promotion_window.unwrap_or(5),
            request.quarantine_epochs.unwrap_or(3),
        );
        if transition == Transition::Quarantine {
            if let Some(policy) = target {
                controller.quarantine.push(
                    focusa_core::compaction_policy::QuarantineEntry {
                        policy,
                        reason: "epoch regression".into(),
                        until_epoch: controller
                            .epochs_seen
                            .saturating_add(request.quarantine_epochs.unwrap_or(3)),
                    },
                );
            }
        }
        if transition == Transition::Promote {
            if let Some(policy) = target {
                controller.active_lease = Some(EpochLease::seal(
                    request.epoch_id.clone(),
                    policy,
                    &request.facts,
                    &mask,
                    &request.epoch_id,
                    &request.epoch_id,
                ));
            }
        }
        if transition == Transition::Rollback {
            if let Some(policy) = target {
                controller.active_lease = Some(EpochLease::seal(
                    request.epoch_id.clone(),
                    policy,
                    &request.facts,
                    &mask,
                    &request.epoch_id,
                    &request.epoch_id,
                ));
            } else {
                controller.active_lease = None;
            }
        }
        controller.epochs_seen = controller.epochs_seen.saturating_add(1);
        controller
            .shadow_history
            .push(shadow.clone());
        if controller.shadow_history.len() > 64 {
            controller.shadow_history.remove(0);
        }
        save_controller_state_sqlite(&conn, &controller)?;
        append_epoch_history(
            &conn,
            &request.epoch_id,
            controller
                .active_lease
                .as_ref()
                .map(|lease| lease.policy)
                .unwrap_or(Policy::None),
            transition,
            &request.outcome,
        )?;
        Ok(json!({
            "schema": "focusa.compaction_epoch_result.v1",
            "epoch_id": request.epoch_id,
            "transition": serde_json::to_value(transition)?,
            "active_policy": controller.active_lease.as_ref().and_then(|lease| serde_json::to_value(lease.policy).ok()),
            "mask": mask,
            "shadow_target": serde_json::to_value(candidate)?,
        }))
    })
    .await;
    match result {
        Ok(Ok(payload)) => Json(payload),
        Ok(Err(error)) => Json(focusa_core::error_envelope::internal_error(
            "route",
            &error.to_string(),
        )),
        Err(error) => Json(focusa_core::error_envelope::internal_error(
            "join",
            &format!("join error: {error}"),
        )),
    }
}

fn state_path(state: &Arc<AppState>) -> std::path::PathBuf {
    std::path::PathBuf::from(&state.config.data_dir)
        .join(".focusa")
        .join("compaction-controller.json")
}

async fn status(State(state): State<Arc<AppState>>) -> Json<Value> {
    let db = crate::routes::events_sqlite::focusa_db_path(&state.config.data_dir);
    let controller = match rusqlite::Connection::open(&db) {
        Ok(conn) => {
            focusa_core::compaction_policy::load_controller_state_sqlite(&conn).unwrap_or_default()
        }
        Err(_) => focusa_core::compaction_policy::load_controller_state(&state_path(&state)),
    };
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
