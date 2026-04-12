//! Ontology inspection routes.
//!
//! Read-only projection of the typed software/work/mission/execution world.
//! This keeps ontology additive in implementation while making the bounded
//! working world inspectable at runtime.

use crate::server::AppState;
use axum::extract::{Query, State};
use axum::{Json, Router, routing::get};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

const OBJECT_TYPES: &[&str] = &[
    "repo", "package", "module", "file", "symbol", "route", "endpoint", "schema", "migration", "dependency", "test", "environment",
    "task", "bug", "feature", "decision", "convention", "constraint", "risk", "milestone",
    "goal", "subgoal", "active_focus", "open_loop", "acceptance_criterion",
    "patch", "diff", "failure", "verification", "artifact",
];
const STATUS_VOCABULARY: &[&str] = &[
    "active", "speculative", "blocked", "verified", "stale", "deprecated", "canonical", "experimental",
];
const MEMBERSHIP_CLASSES: &[&str] = &[
    "pinned", "deterministic", "verified", "inferred", "provisional",
];
const PROVENANCE_CLASSES: &[&str] = &[
    "parser_derived", "tool_derived", "user_asserted", "model_inferred", "reducer_promoted", "verification_confirmed",
];
const LINK_TYPES: &[&str] = &[
    "imports", "calls", "renders", "persists_to", "depends_on", "configured_by", "tested_by", "implements", "violates", "blocks", "supersedes", "belongs_to_goal", "verifies", "derived_from",
];
const ACTION_TYPES: &[&str] = &[
    "refactor_module", "modify_schema", "add_route", "add_test", "verify_invariant", "promote_decision", "mark_blocked", "resolve_risk", "complete_task", "rollback_change",
];

#[derive(Deserialize)]
struct OntologyWorldQuery {
    frame_id: Option<String>,
}

async fn primitives() -> Json<Value> {
    let object_types: Vec<Value> = OBJECT_TYPES
        .iter()
        .map(|name| {
            json!({
                "type_name": name,
                "id_strategy": "stable_string_or_uuid",
                "required_properties": ["id", "status"],
                "allowed_links": LINK_TYPES,
                "allowed_actions": ACTION_TYPES,
                "status_vocabulary": STATUS_VOCABULARY,
            })
        })
        .collect();

    let link_types: Vec<Value> = LINK_TYPES
        .iter()
        .map(|name| {
            json!({
                "name": name,
                "source_types": OBJECT_TYPES,
                "target_types": OBJECT_TYPES,
                "multiplicity": "many",
                "directionality": "directed",
                "evidence_policy": "required",
                "promotion_policy": "reducer_only",
            })
        })
        .collect();

    let action_types: Vec<Value> = ACTION_TYPES
        .iter()
        .map(|name| {
            json!({
                "name": name,
                "target_types": OBJECT_TYPES,
                "input_schema": "json_object",
                "preconditions": ["applicable_constraints_checked"],
                "side_effects": ["reducer_visible_events"],
                "verification_hooks": ["runtime_or_test_gate"],
                "revertability": "varies",
                "emitted_events": ["ontology_delta_applied"],
            })
        })
        .collect();

    Json(json!({
        "object_types": object_types,
        "link_types": link_types,
        "action_types": action_types,
        "status_vocabulary": STATUS_VOCABULARY,
        "membership_classes": MEMBERSHIP_CLASSES,
        "provenance_classes": PROVENANCE_CLASSES,
    }))
}

async fn world(Query(query): Query<OntologyWorldQuery>, State(state): State<Arc<AppState>>) -> Json<Value> {
    let focusa = state.focusa.read().await;
    let selected_frame = query
        .frame_id
        .as_deref()
        .and_then(|frame_id| focusa.focus_stack.frames.iter().find(|f| f.id.to_string() == frame_id));
    let active_frame = selected_frame.or_else(|| {
        focusa
            .focus_stack
            .active_id
            .and_then(|aid| focusa.focus_stack.frames.iter().find(|f| f.id == aid))
    });

    let mut objects: Vec<Value> = Vec::new();
    let mut links: Vec<Value> = Vec::new();

    if let Some(frame) = active_frame {
        let goal_id = format!("goal:{}", frame.id);
        let focus_id = format!("active_focus:{}", frame.id);
        objects.push(json!({
            "id": goal_id,
            "object_type": "goal",
            "title": if frame.goal.is_empty() { frame.title.clone() } else { frame.goal.clone() },
            "objective": frame.goal,
            "status": format!("{:?}", frame.status).to_lowercase(),
            "membership_class": "deterministic",
            "provenance_class": "reducer_promoted",
            "fresh": true,
        }));
        objects.push(json!({
            "id": focus_id,
            "object_type": "active_focus",
            "title": frame.title,
            "frame_id": frame.id,
            "status": "active",
            "membership_class": "deterministic",
            "provenance_class": "reducer_promoted",
            "fresh": true,
        }));
        links.push(json!({
            "type": "belongs_to_goal",
            "source_id": format!("active_focus:{}", frame.id),
            "target_id": format!("goal:{}", frame.id),
            "evidence": "focus_stack.active_frame",
            "status": "verified",
        }));

        for (idx, decision) in frame.focus_state.decisions.iter().take(4).enumerate() {
            let id = format!("decision:{}:{}", frame.id, idx);
            objects.push(json!({
                "id": id,
                "object_type": "decision",
                "statement": decision,
                "decision_kind": "runtime",
                "status": "canonical",
                "membership_class": "verified",
                "provenance_class": "reducer_promoted",
                "fresh": true,
            }));
            links.push(json!({
                "type": "belongs_to_goal",
                "source_id": id,
                "target_id": format!("goal:{}", frame.id),
                "evidence": "focus_state.decisions",
                "status": "verified",
            }));
        }

        for (idx, constraint) in frame.focus_state.constraints.iter().take(4).enumerate() {
            let id = format!("constraint:{}:{}", frame.id, idx);
            objects.push(json!({
                "id": id,
                "object_type": "constraint",
                "rule_text": constraint,
                "scope": "active_frame",
                "enforcement_level": "hard",
                "status": "active",
                "membership_class": "verified",
                "provenance_class": "reducer_promoted",
                "fresh": true,
            }));
            links.push(json!({
                "type": "configured_by",
                "source_id": format!("active_focus:{}", frame.id),
                "target_id": id,
                "evidence": "focus_state.constraints",
                "status": "verified",
            }));
        }

        for (idx, open_loop) in frame.focus_state.next_steps.iter().take(3).enumerate() {
            let id = format!("open_loop:{}:{}", frame.id, idx);
            objects.push(json!({
                "id": id,
                "object_type": "open_loop",
                "statement": open_loop,
                "urgency": "normal",
                "status": "active",
                "membership_class": "provisional",
                "provenance_class": "reducer_promoted",
                "fresh": true,
            }));
            links.push(json!({
                "type": "belongs_to_goal",
                "source_id": id,
                "target_id": format!("goal:{}", frame.id),
                "evidence": "focus_state.next_steps",
                "status": "verified",
            }));
        }

        for (idx, failure) in frame.focus_state.failures.iter().take(3).enumerate() {
            let id = format!("failure:{}:{}", frame.id, idx);
            objects.push(json!({
                "id": id,
                "object_type": "failure",
                "failure_kind": "runtime",
                "timestamp": frame.updated_at,
                "status": "blocked",
                "summary": failure,
                "membership_class": "verified",
                "provenance_class": "verification_confirmed",
                "fresh": true,
            }));
            links.push(json!({
                "type": "blocks",
                "source_id": id,
                "target_id": format!("goal:{}", frame.id),
                "evidence": "focus_state.failures",
                "status": "verified",
            }));
        }

        for (idx, verification) in frame.focus_state.recent_results.iter().take(3).enumerate() {
            let id = format!("verification:{}:{}", frame.id, idx);
            objects.push(json!({
                "id": id,
                "object_type": "verification",
                "method": "recent_result",
                "result": verification,
                "timestamp": frame.updated_at,
                "status": "verified",
                "membership_class": "verified",
                "provenance_class": "verification_confirmed",
                "fresh": true,
            }));
            links.push(json!({
                "type": "verifies",
                "source_id": id,
                "target_id": format!("goal:{}", frame.id),
                "evidence": "focus_state.recent_results",
                "status": "verified",
            }));
        }
    }

    let session_id = focusa.session.as_ref().map(|s| s.session_id);
    for handle in focusa
        .reference_index
        .handles
        .iter()
        .filter(|h| h.session_id == session_id || h.pinned)
        .take(5)
    {
        let id = format!("artifact:{}", handle.id);
        objects.push(json!({
            "id": id,
            "object_type": "artifact",
            "handle": handle.id,
            "artifact_kind": format!("{:?}", handle.kind).to_lowercase(),
            "status": if handle.pinned { "canonical" } else { "verified" },
            "membership_class": if handle.pinned { "pinned" } else { "verified" },
            "provenance_class": "tool_derived",
            "fresh": true,
        }));
        if let Some(frame) = active_frame {
            links.push(json!({
                "type": "derived_from",
                "source_id": id,
                "target_id": format!("active_focus:{}", frame.id),
                "evidence": "reference_index.handle",
                "status": "verified",
            }));
        }
    }

    let action_catalog: Vec<Value> = ACTION_TYPES
        .iter()
        .map(|name| {
            json!({
                "name": name,
                "constraint_checked": true,
                "reducer_visible": true,
                "verification_hooks": ["runtime_gate"],
            })
        })
        .collect();

    let active_mission_count = objects.len().min(12);
    Json(json!({
        "object_count": objects.len(),
        "link_count": links.len(),
        "objects": objects,
        "links": links,
        "action_catalog": action_catalog,
        "working_sets": {
            "active_mission_set": {
                "max_object_count": 12,
                "max_decision_constraint_count": 8,
                "max_artifact_handle_count": 5,
                "max_historical_delta_count": 3,
                "count": active_mission_count,
                "refresh_triggers": [
                    "active_frame_change",
                    "goal_change",
                    "accepted_ontology_delta",
                    "failure_signal",
                    "verification_result",
                    "action_intent_completion",
                    "session_resume"
                ]
            }
        }
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/ontology/primitives", get(primitives))
        .route("/v1/ontology/world", get(world))
}
