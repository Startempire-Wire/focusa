//! Fast-forward fan-out route — #169 slice 2. Multiplies workloop-bound
//! silent sessions with deterministic division; every lane uses the
//! EXISTING silent-session create/start machinery.

use axum::Json;
use axum::Router;
use axum::extract::State;
use axum::routing::post;
use focusa_core::session_fanout::{FanoutInput, compile_fanout};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

use crate::server::AppState;

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/silent-sessions/fanout", post(fanout))
}

#[derive(Deserialize)]
pub struct FanoutBody {
    pub work_items: Vec<String>,
    pub multiplier: u32,
    #[serde(default = "default_turns")]
    pub policy_max_turns_per_session: u32,
    #[serde(default = "default_wall")]
    pub policy_max_wall_clock_ms: u64,
}

fn default_turns() -> u32 {
    12
}

fn default_wall() -> u64 {
    1_800_000
}

async fn fanout(State(state): State<Arc<AppState>>, Json(body): Json<FanoutBody>) -> Json<Value> {
    let plan = compile_fanout(&FanoutInput {
        work_items: body.work_items,
        multiplier: body.multiplier,
        policy_max_turns_per_session: body.policy_max_turns_per_session,
        policy_max_wall_clock_ms: body.policy_max_wall_clock_ms,
        orchestrator_capability_refs: vec![
            // Strong frontier tier — planning/division/adjudication.
            "orchestration".to_string(),
            "adjudication".to_string(),
        ],
        worker_capability_refs: vec![
            // Weaker implementation tier.
            "implementation".to_string(),
        ],
    });
    match plan {
        Ok(plan) => Json(json!({
            "status": "planned",
            "plan": plan,
            "next": "POST /v1/silent-sessions per lane with identity.work_item_ref bound (docs/168)",
        })),
        Err(error) => Json(focusa_core::error_envelope::standard_error(
            "rejected",
            "fanout_invalid",
            "do_not_retry_unchanged",
            "supply >= 1 work item and a multiplier >= 1",
            &error,
        )),
    }
}
