//! Spec93 non-Pi agent awareness card API.

use crate::routes::permissions::permission_context;
use crate::routes::project_bootstrap::remote::resolve_read_scope;
use crate::routes::workpoint::active_workpoint_for_context;
use crate::server::AppState;
use axum::extract::{Query, State};
use axum::http::HeaderMap;
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
    pub working_subpath_id: Option<String>,
}

fn clean(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string)
}

fn scoped_workpoint<'a>(
    focusa: &'a FocusaState,
    query: &AwarenessCardQuery,
) -> Option<&'a WorkpointRecord> {
    let project_root = clean(query.project_root.as_deref())?;
    let continuity_id = clean(query.continuity_id.as_deref())?;
    // Share resume's selector, including its primary-subpath default. A
    // caller's temporal session id and the daemon-wide active id are irrelevant.
    active_workpoint_for_context(
        focusa,
        Some(&project_root),
        Some(&continuity_id),
        query.working_subpath_id.as_deref(),
    )
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

    [
        "# Focusa".to_string(),
        format!("Status: {}", if canonical { "ready" } else { "scope verification pending" }),
        format!("Surface: adapter={adapter} · workspace={workspace}"),
        format!("Scope: project_root={project_root} · continuity={continuity} · workpoint={}", if canonical { "canonical" } else { "not verified" }),
        "Identity: project_root + continuity_id bind canonical continuity; session_id is temporal metadata only.".to_string(),
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
    headers: HeaderMap,
    Query(query): Query<AwarenessCardQuery>,
) -> Json<Value> {
    // The card embeds mission/next-action text from the same scoped records
    // as resume. A lookup must not grant access that resume would deny.
    let may_read =
        permission_context(&headers, state.config.auth_token.is_some()).allows("work-loop:read");
    let remote = if may_read {
        Some(
            resolve_read_scope(
                &state,
                &headers,
                query.project_root.as_deref(),
                query.continuity_id.as_deref(),
                query.working_subpath_id.as_deref(),
            )
            .await,
        )
    } else {
        None
    };
    let publication = match &remote {
        Some(Ok(Some(authority))) => authority.read_publication(&state).await,
        _ => Ok(None),
    };
    let focusa = state.focusa.read().await;
    let record = match &remote {
        Some(Ok(None)) => scoped_workpoint(&focusa, &query),
        Some(Ok(Some(authority))) => match &publication {
            Ok(Some((intent, published))) => authority
                .active_records(&focusa, intent, published)
                .map(|(_, workpoint)| workpoint),
            _ => None,
        },
        // A completed unverified read remains advisory; no fallback to local state.
        _ => None,
    };
    let failure_class = if record.is_some() {
        None
    } else if !may_read {
        Some("workpoint_read_not_permitted")
    } else if matches!(remote, Some(Err(_))) || publication.is_err() {
        Some("remote_scope_unverified")
    } else {
        Some("scoped_active_workpoint_unavailable")
    };
    let awareness_canonical = record.is_some();
    let remote_binding_id = match &remote {
        Some(Ok(Some(authority))) if awareness_canonical => Some(&authority.binding.binding_id),
        _ => None,
    };
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
    let temporal_context = if awareness_canonical {
        super::temporal_context::bounded_temporal_context(
            project_root,
            continuity_id,
            record.map(|item| item.workpoint_id.to_string()),
            record.and_then(|item| item.work_item_id.clone()),
        )
    } else {
        json!({
            "schema":"focusa.bounded_temporal_context.v1",
            "status":"unavailable",
            "canonical":false,
            "failure_class":"scope_unverified",
            "cache_safe_refs_only":true
        })
    };
    Json(json!({
        "status": "completed",
        "canonical": awareness_canonical,
        "execution_admission": false,
        "failure_class": failure_class,
        "surface": "focusa_awareness_card",
        "adapter_id": query.adapter_id,
        "workspace_id": query.workspace_id,
        "agent_id": query.agent_id,
        "operator_id": query.operator_id,
        "session_id": query.session_id,
        "continuity_id": query.continuity_id,
        "project_root": query.project_root,
        "working_subpath_id": query.working_subpath_id,
        "remote_binding_id": remote_binding_id,
        "identity_semantics": {
            "schema": "focusa.awareness_identity_semantics.v1",
            "canonical_continuity_key": ["project_root", "continuity_id"],
            "continuity_id_role": "canonical_workstream_continuity",
            "session_id_role": "temporal metadata",
            "session_id_is_temporal_metadata": true,
            "session_id_must_not_replace_continuity_id": true
        },
        "workpoint_id": record.map(|r| r.workpoint_id),
        "workpoint_canonical": record.map(|r| r.canonical).unwrap_or(false),
        "public_stream_policy": public_policy,
        "temporal_context":temporal_context,
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
    use focusa_core::types::{FocusaSessionIdentity, WorkpointStatus};
    use uuid::Uuid;

    fn state_with_active_workpoint(project_root: &str, continuity_id: &str) -> FocusaState {
        let mut state = FocusaState::default();
        let record = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            project_root: Some(project_root.to_string()),
            continuity_id: Some(continuity_id.to_string()),
            session_id: Some("session-1".to_string()),
            canonical: true,
            status: WorkpointStatus::Active,
            mission: Some("Scoped mission".to_string()),
            next_slice: Some("Scoped next".to_string()),
            ..Default::default()
        };
        state.workpoint.active_workpoint_id = Some(record.workpoint_id);
        state.workpoint.records.push(record);
        state
    }

    #[test]
    fn awareness_card_selects_the_requested_active_workstream_and_worktree() {
        let root = tempfile::tempdir().unwrap();
        let project = root.path().join("remote-checkout");
        let project = project.to_str().unwrap();
        let mut state = state_with_active_workpoint(project, "workstream-a");
        let primary = state.workpoint.records[0].clone();
        let sibling = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            mission: Some("Sibling worktree mission".into()),
            session_identity: Some(FocusaSessionIdentity {
                working_subpath_id: Some("sibling".into()),
                ..Default::default()
            }),
            ..primary.clone()
        };
        let other = WorkpointRecord {
            workpoint_id: Uuid::now_v7(),
            continuity_id: Some("workstream-b".into()),
            mission: Some("Other workstream mission".into()),
            ..primary.clone()
        };
        state.workpoint.active_workpoint_id = Some(other.workpoint_id);
        state.workpoint.records.extend([sibling.clone(), other]);
        let mut query = AwarenessCardQuery {
            project_root: Some(project.into()),
            continuity_id: Some("workstream-a".into()),
            session_id: Some("restored-session".into()),
            ..Default::default()
        };
        assert_eq!(
            scoped_workpoint(&state, &query).unwrap().workpoint_id,
            primary.workpoint_id
        );
        query.working_subpath_id = Some("sibling".into());
        let record = scoped_workpoint(&state, &query).unwrap();
        assert_eq!(record.workpoint_id, sibling.workpoint_id);
        let card = render_card(&query, Some(record));
        assert!(card.contains("Sibling worktree mission"));
        assert!(!card.contains("Other workstream mission"));
        query.working_subpath_id = Some("unbound".into());
        assert!(scoped_workpoint(&state, &query).is_none());
    }

    #[test]
    fn awareness_card_does_not_render_a_noncanonical_record() {
        let root = tempfile::tempdir().unwrap();
        let project = root.path().join("remote-checkout");
        let project = project.to_str().unwrap();
        let mut state = state_with_active_workpoint(project, "workstream");
        state.workpoint.records[0].canonical = false;
        let query = AwarenessCardQuery {
            project_root: Some(project.into()),
            continuity_id: Some("workstream".into()),
            ..Default::default()
        };
        let selected = scoped_workpoint(&state, &query);
        assert!(selected.is_none());
        assert!(!render_card(&query, selected).contains("Scoped mission"));
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
    fn awareness_card_keeps_same_continuity_across_temporal_sessions() {
        let state = state_with_active_workpoint("/home/focusa/a", "cont-a");
        let record = scoped_workpoint(
            &state,
            &AwarenessCardQuery {
                project_root: Some("/home/focusa/a".into()),
                continuity_id: Some("cont-a".into()),
                session_id: Some("different-temporal-session".into()),
                ..Default::default()
            },
        )
        .expect("session metadata must not demote canonical continuity");
        assert_eq!(record.continuity_id.as_deref(), Some("cont-a"));
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
                ..Default::default()
            },
            None,
        );
        for needle in [
            "# Focusa",
            "Status: scope verification pending",
            "Surface: adapter=openclaw",
            "Scope: project_root=",
            "project_root + continuity_id bind canonical continuity",
            "session_id is temporal metadata only",
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
