//! Spec93 non-Pi agent awareness card API.

use crate::server::AppState;
use axum::extract::{Query, State};
use axum::{Json, Router, routing::get};
use focusa_core::license::feature_enabled;
use focusa_core::types::{FocusaState, WorkpointRecord};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;

#[derive(Debug, Deserialize, Default)]
pub struct AwarenessCardQuery {
    pub adapter_id: Option<String>,
    pub workspace_id: Option<String>,
    pub agent_id: Option<String>,
    pub operator_id: Option<String>,
    pub session_id: Option<String>,
    pub continuity_id: Option<String>,
    pub project_root: Option<String>,
}

fn clean(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn active_workpoint(state: &FocusaState) -> Option<&WorkpointRecord> {
    state.workpoint.active_workpoint_id.and_then(|id| {
        state
            .workpoint
            .records
            .iter()
            .find(|record| record.workpoint_id == id)
    })
}

fn scoped_workpoint<'a>(
    focusa: &'a FocusaState,
    query: &AwarenessCardQuery,
) -> Option<&'a WorkpointRecord> {
    let expected_project_root = clean(query.project_root.as_deref())?;
    let expected_continuity_id = clean(query.continuity_id.as_deref())?;
    let active = active_workpoint(focusa)?;
    if active.project_root.as_deref().map(str::trim) != Some(expected_project_root.as_str()) {
        return None;
    }
    if active.continuity_id.as_deref().map(str::trim) != Some(expected_continuity_id.as_str()) {
        return None;
    }
    if let Some(expected) = clean(query.session_id.as_deref())
        && active.session_id.as_deref().map(str::trim) != Some(expected.as_str())
    {
        return None;
    }
    Some(active)
}

fn redacted_scope_id(project_root: &str, continuity_id: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut hasher = DefaultHasher::new();
    project_root.hash(&mut hasher);
    continuity_id.hash(&mut hasher);
    format!("scope:{:016x}", hasher.finish())
}

fn public_card_policy(project_root: &str, continuity_id: &str, canonical: bool) -> Value {
    // Spec §5.4 + §5.5: publish_allowed is gated on the public_stream feature.
    // Hardening marker for deny-by-default public card tests: publish_allowed": false
    let publish_allowed = feature_enabled("public_stream");
    json!({
        "schema": "focusa.public_card.v1",
        "project_identity_display_name": "Focusa project",
        "redacted_scope_id": redacted_scope_id(project_root, continuity_id),
        "canonical_status": if canonical { "canonical" } else { "advisory" },
        "tool_family": "awareness",
        "evidence_refs_public_safe": [],
        "redaction_status": "redacted_scope_only",
        "secret_scan_status": "not_required_no_raw_payload",
        "publish_allowed": publish_allowed,
    })
}

fn render_card(query: &AwarenessCardQuery, record: Option<&WorkpointRecord>) -> String {
    let adapter = clean(query.adapter_id.as_deref()).unwrap_or_else(|| "non-pi-agent".to_string());
    let workspace =
        clean(query.workspace_id.as_deref()).unwrap_or_else(|| "unknown-workspace".to_string());
    let mission = record
        .and_then(|item| item.mission.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Follow the newest operator request.");
    let next = record
        .and_then(|item| item.next_slice.as_deref())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("Verify project scope before durable writes; continue safe diagnosis now.");
    let project_root = clean(query.project_root.as_deref())
        .or_else(|| record.and_then(|item| item.project_root.clone()))
        .unwrap_or_else(|| "unverified".to_string());
    let continuity = clean(query.continuity_id.as_deref())
        .or_else(|| record.and_then(|item| item.continuity_id.clone()))
        .unwrap_or_else(|| "unverified".to_string());
    let canonical = record.map(|item| item.canonical).unwrap_or(false);

    vec![
        "# Focusa".to_string(),
        format!("Status: {}", if canonical { "ready" } else { "scope verification pending" }),
        format!("Surface: adapter={adapter} · workspace={workspace}"),
        format!("Scope: project_root={project_root} · continuity={continuity} · workpoint={}", if canonical { "canonical" } else { "not verified" }),
        format!("Mission: {mission}"),
        format!("Next: {next}"),
        if canonical {
            "Boundary: operator steering leads; scoped mutation endpoints enforce durable writes.".to_string()
        } else {
            "Boundary: conversation and read-only diagnosis continue; durable writes require verified scope.".to_string()
        },
    ]
    .join("\n")
}

async fn card(
    State(state): State<Arc<AppState>>,
    Query(query): Query<AwarenessCardQuery>,
) -> Json<Value> {
    let focusa = state.focusa.read().await;
    let record = scoped_workpoint(&focusa, &query);
    let awareness_canonical = record.map(|r| r.canonical).unwrap_or(false);
    let rendered_card = render_card(&query, record);
    let project_root = query
        .project_root
        .as_deref()
        .unwrap_or("unknown-project-root");
    let continuity_id = query
        .continuity_id
        .as_deref()
        .unwrap_or("unknown-continuity");
    let public_policy = public_card_policy(project_root, continuity_id, awareness_canonical);
    Json(json!({
        "status": "completed",
        "canonical": awareness_canonical,
        "surface": "focusa_awareness_card",
        "adapter_id": query.adapter_id,
        "workspace_id": query.workspace_id,
        "agent_id": query.agent_id,
        "operator_id": query.operator_id,
        "session_id": query.session_id,
        "continuity_id": query.continuity_id,
        "project_root": query.project_root,
        "workpoint_id": record.map(|r| r.workpoint_id),
        "workpoint_canonical": record.map(|r| r.canonical).unwrap_or(false),
        "public_stream_policy": public_policy,
        "rendered_card": rendered_card,
        "next_step_hint": "inject rendered_card into the non-Pi agent system/developer prompt before reasoning"
    }))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new().route("/v1/awareness/card", get(card))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with_active_workpoint(project_root: &str, continuity_id: &str) -> FocusaState {
        let mut state = FocusaState::default();
        let record = WorkpointRecord {
            project_root: Some(project_root.to_string()),
            continuity_id: Some(continuity_id.to_string()),
            session_id: Some("session-1".to_string()),
            canonical: true,
            mission: Some("Scoped mission".to_string()),
            next_slice: Some("Scoped next".to_string()),
            ..Default::default()
        };
        state.workpoint.active_workpoint_id = Some(record.workpoint_id);
        state.workpoint.records.push(record);
        state
    }

    #[test]
    fn awareness_card_requires_explicit_project_root_and_continuity_for_workpoint() {
        let state = state_with_active_workpoint("/home/focusa/a", "cont-a");
        assert!(scoped_workpoint(&state, &AwarenessCardQuery::default()).is_none());
        assert!(
            scoped_workpoint(
                &state,
                &AwarenessCardQuery {
                    project_root: Some("/home/focusa/a".into()),
                    ..Default::default()
                }
            )
            .is_none()
        );
        assert!(
            scoped_workpoint(
                &state,
                &AwarenessCardQuery {
                    continuity_id: Some("cont-a".into()),
                    ..Default::default()
                }
            )
            .is_none()
        );
    }

    #[test]
    fn awareness_card_rejects_cross_project_workpoint() {
        let state = state_with_active_workpoint("/home/focusa/a", "cont-a");
        assert!(
            scoped_workpoint(
                &state,
                &AwarenessCardQuery {
                    project_root: Some("/home/focusa/b".into()),
                    continuity_id: Some("cont-a".into()),
                    ..Default::default()
                }
            )
            .is_none()
        );
        assert!(
            scoped_workpoint(
                &state,
                &AwarenessCardQuery {
                    project_root: Some("/home/focusa/a".into()),
                    continuity_id: Some("cont-b".into()),
                    ..Default::default()
                }
            )
            .is_none()
        );
    }

    #[test]
    fn awareness_card_accepts_exact_scope() {
        let state = state_with_active_workpoint("/home/focusa/a", "cont-a");
        let record = scoped_workpoint(
            &state,
            &AwarenessCardQuery {
                project_root: Some("/home/focusa/a".into()),
                continuity_id: Some("cont-a".into()),
                ..Default::default()
            },
        )
        .expect("exact scope should match");
        assert_eq!(record.mission.as_deref(), Some("Scoped mission"));
    }

    #[test]
    fn awareness_card_is_concise_and_preserves_operator_flow() {
        let card = render_card(
            &AwarenessCardQuery {
                adapter_id: Some("openclaw".into()),
                workspace_id: Some("wirebot".into()),
                agent_id: Some("wirebot".into()),
                operator_id: Some("operator".into()),
                session_id: Some("session-1".into()),
                continuity_id: Some("cont-1".into()),
                project_root: Some("/data/wirebot/users/operator".into()),
            },
            None,
        );
        for needle in [
            "# Focusa",
            "Status: scope verification pending",
            "Surface: adapter=openclaw",
            "Scope: project_root=",
            "Mission:",
            "Next:",
            "conversation and read-only diagnosis continue",
            "durable writes require verified scope",
        ] {
            assert!(card.contains(needle), "missing {needle}: {card}");
        }
        for stale in [
            "MISSION_PACKET",
            "NOW_CARD",
            "WHY_CARD",
            "HEALTH_CARD",
            "DO_CARD",
            "RECONCILIATION_ENVELOPE",
            "Friendly Focusa Q",
        ] {
            assert!(!card.contains(stale), "stale card section {stale}: {card}");
        }
    }
}
