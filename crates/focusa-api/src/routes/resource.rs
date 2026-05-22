//! Resource mode and LowMem status/control routes.

use crate::routes::bounded::{
    observe_resource_mode_transition, resource_mode_status, resource_mode_transition_records,
    set_runtime_resource_mode_override,
};
use crate::server::AppState;
use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Debug, Deserialize, Default)]
struct ResourceModeBody {
    action: Option<String>,
    mode: Option<String>,
    reason: Option<String>,
    #[serde(default)]
    preflight: bool,
}

fn mode_from_body(body: &ResourceModeBody) -> Option<String> {
    let action = body
        .action
        .as_deref()
        .unwrap_or("status")
        .to_ascii_lowercase();
    match action.as_str() {
        "activate_lowmem" | "lowmem_on" | "on" => Some("lowmem".to_string()),
        "deactivate_lowmem" | "lowmem_off" | "auto" | "clear" => Some("auto".to_string()),
        "set_mode" | "set" => body.mode.clone(),
        "set_normal" | "normal" => Some("normal".to_string()),
        "set_constrained" | "constrained" => Some("constrained".to_string()),
        "set_emergency" | "emergency" => Some("emergency".to_string()),
        "status" => None,
        _ => body.mode.clone(),
    }
}

async fn active_session_id(state: &Arc<AppState>) -> Option<String> {
    state
        .focusa
        .read()
        .await
        .session
        .as_ref()
        .map(|session| session.session_id.to_string())
}


fn tool_result_v1(ok: bool, status: &str, failure_class: Option<&str>, side_effects: Vec<&str>) -> Value {
    json!({
        "ok": ok,
        "status": status,
        "failure_class": failure_class,
        "canonical": ok,
        "degraded": !ok,
        "retry": {
            "safe": ok,
            "posture": if ok { "safe_retry" } else { "do_not_retry_unchanged" },
            "reason": failure_class,
        },
        "side_effects": side_effects.clone(),
        "next_tools": ["focusa_resource_mode", "focusa_tool_doctor", "focusa_trajectory_view"],
    })
}

async fn mode() -> Json<Value> {
    Json(json!({
        "status": "completed",
        "canonical": true,
        "degraded": false,
        "failure_class": Value::Null,
        "resource_mode": resource_mode_status(),
        "transition_history": resource_mode_transition_records(5),
        "side_effects": [],
        "next_tools": ["focusa_resource_mode", "focusa_tool_doctor"],
        "details": {"tool_result_v1": tool_result_v1(true, "completed", None, vec![])},
    }))
}

async fn set_mode(
    State(state): State<Arc<AppState>>,
    Json(body): Json<ResourceModeBody>,
) -> Json<Value> {
    let requested = mode_from_body(&body);
    let mut changed = false;
    let mut rejected = None;
    if let Some(mode) = requested.as_deref() {
        if body.preflight {
            changed = false;
        } else {
            match set_runtime_resource_mode_override(Some(mode)) {
                Ok(_) => changed = true,
                Err(err) => rejected = Some(err),
            }
        }
    }
    let status = if changed && rejected.is_none() {
        observe_resource_mode_transition("api_override", active_session_id(&state).await)
    } else {
        resource_mode_status()
    };
    let action = body.action.unwrap_or_else(|| "status".to_string());
    let summary = if let Some(err) = rejected.as_deref() {
        format!("resource mode control blocked: {err}")
    } else if body.preflight {
        format!(
            "resource mode preflight: requested={:?} current={} reason={}",
            requested, status.mode, status.reason
        )
    } else if changed {
        format!(
            "resource mode set: mode={} reason={}",
            status.mode, status.reason
        )
    } else {
        format!(
            "resource mode status: mode={} reason={}",
            status.mode, status.reason
        )
    };
    let status_label = if rejected.is_some() { "blocked" } else { "completed" };
    let failure_class = rejected.as_ref().map(|_| "validation_rejected");
    let side_effects = if changed { vec!["runtime_resource_mode_override"] } else { Vec::<&str>::new() };
    Json(json!({
        "status": status_label,
        "canonical": rejected.is_none(),
        "degraded": rejected.is_some(),
        "summary": summary,
        "action": action,
        "preflight": body.preflight,
        "requested_mode": requested,
        "resource_mode": status,
        "reason": body.reason,
        "side_effects": side_effects,
        "next_tools": ["focusa_tool_doctor", "focusa_trajectory_view", "focusa_workpoint_resume", "focusa_traverse"],
        "failure_class": failure_class,
        "details": {"tool_result_v1": tool_result_v1(rejected.is_none(), status_label, failure_class, side_effects)},
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/resource/mode", get(mode))
        .route("/v1/resource/mode", post(set_mode))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_mode_body_preflight_maps_to_requested_mode() {
        let body = ResourceModeBody {
            action: Some("activate_lowmem".to_string()),
            preflight: true,
            ..ResourceModeBody::default()
        };
        assert_eq!(mode_from_body(&body).as_deref(), Some("lowmem"));
    }

    #[test]
    fn resource_tool_result_exposes_required_envelope_fields() {
        let result = tool_result_v1(true, "completed", None, vec![]);
        assert_eq!(result.get("ok").and_then(Value::as_bool), Some(true));
        assert_eq!(result.get("canonical").and_then(Value::as_bool), Some(true));
        assert_eq!(result.get("degraded").and_then(Value::as_bool), Some(false));
        assert!(result.get("retry").is_some());
        assert!(result.get("side_effects").and_then(Value::as_array).is_some());
        assert!(result.get("next_tools").and_then(Value::as_array).is_some());
    }
}
