use crate::server::AppState;
use axum::extract::Query;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

const REFLEX_REGISTRY: &str =
    include_str!("../../../../docs/current/focusa-reflex-primitives.json");
const DEFAULT_LIMIT: usize = 20;
const MAX_LIMIT: usize = 50;

pub fn reflex_suggestions_for_failure(failure_class: &str) -> Vec<&'static str> {
    match failure_class {
        "scope_mismatch" => vec!["diagnose_scope_mismatch", "confirm_continuity_scope"],
        "hot_path_timeout" | "cold_path_timeout" | "resource_exhausted" | "daemon_unavailable" => {
            vec!["resource_mode_fallback", "degrade_with_recovery"]
        }
        "read_model_lag" => vec!["retry_safe_pending"],
        "frame_unavailable" | "noncanonical_fallback" | "unknown_ambiguous_completion" => {
            vec!["route_noncanonical_result"]
        }
        "writer_conflict" => vec!["preflight_writer_ownership"],
        "approval_required" | "permission_denied" => vec!["require_destructive_confirmation"],
        "validation_rejected" => vec!["guard_stale_focus_state"],
        _ => Vec::new(),
    }
}

#[derive(Debug, Deserialize, Default)]
struct ReflexPrimitiveQuery {
    family: Option<String>,
    query: Option<String>,
    q: Option<String>,
    limit: Option<usize>,
    include_payload: Option<bool>,
}

fn bounded_text(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        return value.to_string();
    }
    let mut out: String = value.chars().take(max.saturating_sub(1)).collect();
    out.push('…');
    out
}

fn registry() -> Value {
    serde_json::from_str(REFLEX_REGISTRY).unwrap_or_else(|_| json!({"primitives": []}))
}

fn primitive_summary(mut primitive: Value, include_payload: bool) -> Value {
    if include_payload {
        if let Some(obj) = primitive.as_object_mut() {
            obj.insert(
                "source".to_string(),
                json!("spec97_reflex_primitive_registry"),
            );
            obj.insert("advisory_only".to_string(), json!(true));
        }
        return primitive;
    }
    json!({
        "primitive_id": primitive.get("primitive_id"),
        "family": primitive.get("family"),
        "trigger": primitive.get("trigger").and_then(Value::as_str).map(|value| bounded_text(value, 160)),
        "recommended_tool": primitive.pointer("/reflex_action/recommended_tool"),
        "authority_boundary": primitive.get("authority_boundary").and_then(Value::as_str).map(|value| bounded_text(value, 160)),
        "escalation_boundary": primitive.get("escalation_boundary").and_then(Value::as_str).map(|value| bounded_text(value, 160)),
        "hot_path_budget": primitive.get("hot_path_budget"),
        "failure_envelope": primitive.get("failure_envelope"),
        "source": "spec97_reflex_primitive_registry",
        "advisory_only": true,
    })
}

fn reflex_primitives_payload(query: &ReflexPrimitiveQuery) -> Value {
    let registry = registry();
    let family = query
        .family
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let text_query = query
        .query
        .as_deref()
        .or(query.q.as_deref())
        .unwrap_or_default()
        .to_ascii_lowercase();
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let include_payload = query.include_payload.unwrap_or(false);
    let primitives = registry
        .get("primitives")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let total = primitives.len();
    let mut matched = Vec::new();
    for primitive in primitives {
        if !family.is_empty()
            && !primitive
                .get("family")
                .and_then(Value::as_str)
                .map(|value| value.eq_ignore_ascii_case(&family))
                .unwrap_or(false)
        {
            continue;
        }
        if !text_query.is_empty()
            && !serde_json::to_string(&primitive)
                .unwrap_or_default()
                .to_ascii_lowercase()
                .contains(&text_query)
        {
            continue;
        }
        matched.push(primitive);
    }
    let matched_total = matched.len();
    let items: Vec<Value> = matched
        .into_iter()
        .take(limit)
        .map(|primitive| primitive_summary(primitive, include_payload))
        .collect();
    json!({
        "status": "completed",
        "canonical": true,
        "degraded": false,
        "schema": registry.get("schema"),
        "version": registry.get("version"),
        "read_only": true,
        "advisory_only": true,
        "authority_boundary": "Reflex primitives are advisory routing metadata; existing Focusa tools/reducers retain mutation authority.",
        "items": items,
        "bounds": {
            "limit": limit,
            "returned": items.len(),
            "matched_total": matched_total,
            "registry_total": total,
            "truncated": matched_total > limit,
            "include_payload": include_payload,
        },
        "next_tools": ["focusa_traverse", "focusa_tool_doctor"],
        "details": {"tool_result_v1": {"ok": true, "status": "completed", "canonical": true, "degraded": false, "failure_class": null, "side_effects": [], "evidence_refs": [], "next_tools": ["focusa_traverse"]}}
    })
}

async fn list(Query(query): Query<ReflexPrimitiveQuery>) -> Json<Value> {
    Json(reflex_primitives_payload(&query))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/reflex/primitives", get(list))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reflex_primitives_route_is_bounded_and_read_only() {
        let payload = reflex_primitives_payload(&ReflexPrimitiveQuery {
            family: Some("recovery".to_string()),
            limit: Some(3),
            ..ReflexPrimitiveQuery::default()
        });
        assert_eq!(payload["status"].as_str(), Some("completed"));
        assert_eq!(payload["read_only"].as_bool(), Some(true));
        assert_eq!(payload["advisory_only"].as_bool(), Some(true));
        assert!(payload["items"].as_array().unwrap().len() <= 3);
        assert_eq!(
            payload["items"][0]["source"].as_str(),
            Some("spec97_reflex_primitive_registry")
        );
    }
}
