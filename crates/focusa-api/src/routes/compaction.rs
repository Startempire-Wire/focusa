//! Spec 130 — bounded CompactionMissionPacket construction and retrieval.
//!
//! The packet is advisory prompt material. Canonical authority remains in
//! Trajectory, Workpoint, Focus State, and evidence stores.

use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use chrono::Utc;
use focusa_core::{
    scope_safety::classify_project_root,
    types::{FocusaState, HltStatus, WorkpointStatus},
};
use serde::Deserialize;
use serde_json::{Value, json};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex, OnceLock},
};
use uuid::Uuid;

const PACKET_SCHEMA: &str = "focusa.compaction_mission_packet.v1";
const PACKET_CAP: usize = 64;

#[derive(Debug, Clone, Deserialize)]
pub struct BuildCompactionPacketRequest {
    pub resume_source: Option<String>,
    pub project_root: Option<String>,
    pub continuity_id: Option<String>,
    pub session_id: Option<String>,
    pub current_ask: Option<String>,
    pub ask_kind: Option<String>,
    pub source_turn_id: Option<String>,
    #[serde(default)]
    pub omitted_sections: Vec<String>,
    #[serde(default)]
    pub omitted_bytes: u64,
    #[serde(default)]
    pub omitted_tokens: u64,
    #[serde(default)]
    pub rehydrate_refs: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PacketIdRequest {
    pub packet_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DiffCompactionPacketRequest {
    pub before: String,
    pub after: String,
}

fn packet_store() -> &'static Mutex<VecDeque<(String, Value)>> {
    static STORE: OnceLock<Mutex<VecDeque<(String, Value)>>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(VecDeque::new()))
}

fn bounded_text(value: Option<&str>, max: usize) -> Option<String> {
    let value = value?.trim();
    if value.is_empty() {
        return None;
    }
    Some(value.chars().take(max).collect())
}

fn build_packet(state: &FocusaState, req: &BuildCompactionPacketRequest) -> Value {
    let packet_id = Uuid::now_v7().to_string();
    let project_root = bounded_text(req.project_root.as_deref(), 4096);
    let continuity_id = bounded_text(req.continuity_id.as_deref(), 256);
    let scope_safe = project_root
        .as_deref()
        .map(|root| classify_project_root(root).is_safe())
        .unwrap_or(false);

    let trajectory = state
        .trajectory
        .active_trajectory_id
        .as_ref()
        .and_then(|id| {
            state
                .trajectory
                .records
                .iter()
                .find(|record| &record.trajectory_id == id)
        })
        .or_else(|| state.trajectory.records.last());
    let workpoint = state
        .workpoint
        .active_workpoint_id
        .as_ref()
        .and_then(|id| {
            state
                .workpoint
                .records
                .iter()
                .find(|record| &record.workpoint_id == id)
        })
        .or_else(|| state.workpoint.records.last());
    let frame = state.focus_stack.active_id.as_ref().and_then(|id| {
        state
            .focus_stack
            .frames
            .iter()
            .find(|frame| &frame.id == id)
    });

    let hlt_status = trajectory
        .map(|record| record.hlt_status)
        .unwrap_or_default();
    let hlt_ready = matches!(
        hlt_status,
        HltStatus::CanonicalExplicit | HltStatus::PreviousValidFallback
    );
    let workpoint_ready = workpoint
        .map(|record| record.canonical && record.status == WorkpointStatus::Active)
        .unwrap_or(false);
    let scope_status = if project_root.is_none() {
        "missing"
    } else if !scope_safe {
        "unsafe"
    } else if trajectory
        .and_then(|record| record.project_root.as_deref())
        .zip(project_root.as_deref())
        .is_some_and(|(saved, requested)| saved != requested)
    {
        "mismatch"
    } else {
        "verified"
    };
    let status = if scope_status != "verified" {
        "blocked"
    } else if hlt_ready && workpoint_ready {
        "verified"
    } else {
        "degraded"
    };

    let evidence_refs: Vec<String> = workpoint
        .map(|record| {
            record
                .verification_records
                .iter()
                .filter_map(|verification| verification.evidence_ref.clone())
                .take(32)
                .collect()
        })
        .unwrap_or_default();
    let artifact_refs: Vec<String> = frame
        .map(|frame| {
            frame
                .focus_state
                .artifacts
                .iter()
                .filter_map(|artifact| artifact.path_or_id.clone())
                .take(32)
                .collect()
        })
        .unwrap_or_default();
    let blockers = workpoint
        .map(|record| &record.blockers)
        .cloned()
        .unwrap_or_default();
    let active_blocker = blockers.first();
    let allowed_next_tools = if workpoint_ready {
        vec![
            "focusa_workpoint_resume",
            "focusa_trajectory_view",
            "focusa_traverse",
        ]
    } else {
        vec![
            "focusa_project_identity",
            "focusa_trajectory_view",
            "focusa_workpoint_resume",
        ]
    };

    json!({
        "schema_version": PACKET_SCHEMA,
        "packet_id": packet_id,
        "generated_at": Utc::now().to_rfc3339(),
        "resume_source": req.resume_source.as_deref().unwrap_or("manual"),
        "status": status,
        "canonical": false,
        "advisory": true,
        "scope": {
            "scope_kind": "project",
            "project_root": project_root,
            "host_scope_id": Value::Null,
            "continuity_id": continuity_id,
            "session_id": bounded_text(req.session_id.as_deref(), 256),
            "scope_status": scope_status
        },
        "current_ask": {
            "text": bounded_text(req.current_ask.as_deref(), 1000),
            "ask_kind": req.ask_kind.as_deref().unwrap_or("unknown"),
            "source_turn_id": bounded_text(req.source_turn_id.as_deref(), 160)
        },
        "trajectory": {
            "packet_ref": trajectory.map(|record| format!("trajectory_resume_packet_v3:{}", record.trajectory_id)),
            "hlt": trajectory.map(|record| record.long_term_goal.clone()),
            "hlt_status": hlt_status,
            "hlt_required": true,
            "hlt_source": trajectory.and_then(|record| record.goal_provenance.iter().find(|item| item.field == "long_term_goal").map(|item| item.source.clone())).unwrap_or_else(|| "none".into()),
            "generic_bootstrap": matches!(hlt_status, HltStatus::GenericDegraded),
            "fallback": if matches!(hlt_status, HltStatus::PreviousValidFallback) { "previous_valid_trajectory" } else { "none" },
            "fallback_level": if matches!(hlt_status, HltStatus::PreviousValidFallback) { "same_project_any_continuity" } else { "none" },
            "action_authority_from_trajectory": hlt_ready,
            "desired_end_state": trajectory.map(|record| record.desired_end_state.clone()),
            "current_verified_state": trajectory.and_then(|record| record.current_state.clone()),
            "active_gap": trajectory.and_then(|record| record.gap_summary.clone()),
            "warnings": trajectory.map(|record| record.blockers.iter().take(8).cloned().collect::<Vec<_>>()).unwrap_or_default()
        },
        "workpoint": {
            "packet_ref": workpoint.map(|record| format!("workpoint_resume_packet_v2:{}", record.workpoint_id)),
            "workpoint_id": workpoint.map(|record| record.workpoint_id.to_string()),
            "mission": workpoint.and_then(|record| record.mission.clone()),
            "next_slice": workpoint.and_then(|record| record.next_slice.clone()),
            "action_authority": workpoint_ready,
            "status": if workpoint_ready { "ready" } else if workpoint.is_some() { "stale" } else { "missing" }
        },
        "focus_state": {
            "intent": frame.map(|frame| frame.focus_state.intent.clone()),
            "current_focus": frame.map(|frame| frame.focus_state.current_state.clone()),
            "decisions": frame.map(|frame| frame.focus_state.decisions.iter().rev().take(8).cloned().collect::<Vec<_>>()).unwrap_or_default(),
            "constraints": frame.map(|frame| frame.focus_state.constraints.iter().rev().take(8).cloned().collect::<Vec<_>>()).unwrap_or_default(),
            "failures": frame.map(|frame| frame.focus_state.failures.iter().rev().take(4).cloned().collect::<Vec<_>>()).unwrap_or_default(),
            "recent_results": frame.map(|frame| frame.focus_state.recent_results.iter().take(6).cloned().collect::<Vec<_>>()).unwrap_or_default()
        },
        "active_blocker": {
            "present": active_blocker.is_some(),
            "error_class": active_blocker.and_then(|blocker| blocker.status.clone()),
            "test_name": Value::Null,
            "file_path": active_blocker.and_then(|blocker| blocker.target_ref.clone()),
            "line_range": Value::Null,
            "exact_blocker_excerpt": active_blocker.map(|blocker| blocker.reason.clone()),
            "rehydrate_ref": active_blocker.and_then(|blocker| blocker.target_ref.clone())
        },
        "evidence": {
            "evidence_refs": evidence_refs,
            "ecs_handles": Vec::<String>::new(),
            "receipt_refs": Vec::<String>::new(),
            "proof_refs": artifact_refs,
            "missing_evidence_warning": if workpoint_ready && evidence_refs.is_empty() { Some("active Workpoint has no linked evidence") } else { None }
        },
        "bloatgaurd": {
            "omitted_sections": req.omitted_sections.iter().take(32).collect::<Vec<_>>(),
            "omitted_bytes": req.omitted_bytes,
            "omitted_tokens": req.omitted_tokens,
            "rehydrate_refs": req.rehydrate_refs.iter().take(32).collect::<Vec<_>>(),
            "full_payload_policy": "cold_opt_in",
            "tool_history_elided": true
        },
        "recent_turns": { "count": 0, "refs": Vec::<String>::new() },
        "next": {
            "exact_next_tool": "focusa_workpoint_resume",
            "allowed_next_tools": allowed_next_tools,
            "do_not_use": ["transcript_tail_as_authority", "full_lineage_tree_by_default", "raw_tool_history_by_default"]
        },
        "receipt_expectation": {
            "required_before_completion": true,
            "trajectory_hlt_status_required": true,
            "evidence_required": true,
            "closure_authority_required": true
        }
    })
}

fn packet_by_id(packet_id: &str) -> Option<Value> {
    packet_store()
        .lock()
        .ok()?
        .iter()
        .rev()
        .find(|(id, _)| id == packet_id)
        .map(|(_, packet)| packet.clone())
}

fn store_packet(packet: &Value) {
    let Some(packet_id) = packet.get("packet_id").and_then(Value::as_str) else {
        return;
    };
    let mut store = packet_store()
        .lock()
        .expect("compaction packet store poisoned");
    store.push_back((packet_id.to_string(), packet.clone()));
    while store.len() > PACKET_CAP {
        store.pop_front();
    }
}

async fn build(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BuildCompactionPacketRequest>,
) -> Json<Value> {
    let focusa = state.focusa.read().await;
    let mut packet = build_packet(&focusa, &req);
    drop(focusa);
    let recent_turns = req
        .continuity_id
        .as_deref()
        .map(|continuity_id| {
            crate::routes::turn_recent::read_recent_turns_bounded(
                &state.config.data_dir,
                continuity_id,
                4,
            )
        })
        .unwrap_or_default();
    let recent_refs: Vec<String> = recent_turns
        .iter()
        .map(|turn| format!("recent_turn:{}:{}", turn.continuity_id, turn.turn_id))
        .collect();
    let recent_evidence: Vec<String> = recent_turns
        .iter()
        .flat_map(|turn| turn.evidence_refs.iter().cloned())
        .take(32)
        .collect();
    packet["recent_turns"] = json!({
        "count": recent_turns.len(),
        "refs": recent_refs
    });
    if let Some(evidence) = packet["evidence"]["evidence_refs"].as_array_mut() {
        for evidence_ref in recent_evidence {
            if !evidence.iter().any(|existing| existing == &evidence_ref) {
                evidence.push(Value::String(evidence_ref));
            }
        }
    }
    store_packet(&packet);
    Json(packet)
}

async fn get_packet(Path(packet_id): Path<String>) -> Result<Json<Value>, StatusCode> {
    packet_by_id(&packet_id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

fn inspect_payload(packet: &Value) -> Value {
    json!({
        "schema": "focusa.compaction_inspect.v1",
        "packet_id": packet["packet_id"],
        "status": packet["status"],
        "kept": {
            "current_ask": packet["current_ask"],
            "scope": packet["scope"],
            "trajectory": packet["trajectory"],
            "workpoint": packet["workpoint"],
            "focus_state": packet["focus_state"],
            "active_blocker": packet["active_blocker"],
            "evidence": packet["evidence"],
            "receipt_expectation": packet["receipt_expectation"]
        },
        "omitted": packet["bloatgaurd"]["omitted_sections"],
        "omission_policy": packet["bloatgaurd"]["full_payload_policy"],
        "raw_evidence_refs": packet["bloatgaurd"]["rehydrate_refs"],
        "authority_surface": {
            "trajectory": packet["trajectory"]["packet_ref"],
            "workpoint": packet["workpoint"]["packet_ref"],
            "trajectory_action_authority": packet["trajectory"]["action_authority_from_trajectory"],
            "workpoint_action_authority": packet["workpoint"]["action_authority"]
        },
        "hlt_posture": packet["trajectory"]["hlt_status"],
        "exact_next_tool": packet["next"]["exact_next_tool"],
        "receipt_expectation": packet["receipt_expectation"]
    })
}

async fn inspect(Path(packet_id): Path<String>) -> Result<Json<Value>, StatusCode> {
    packet_by_id(&packet_id)
        .map(|packet| Json(inspect_payload(&packet)))
        .ok_or(StatusCode::NOT_FOUND)
}

fn fidelity_eval(packet: &Value) -> Value {
    const REQUIRED: &[(&str, &[&str])] = &[
        ("current_ask", &["current_ask", "text"]),
        ("scope_status", &["scope", "scope_status"]),
        ("project_root", &["scope", "project_root"]),
        ("continuity_id", &["scope", "continuity_id"]),
        ("session_id", &["scope", "session_id"]),
        ("hlt_status", &["trajectory", "hlt_status"]),
        ("fallback", &["trajectory", "fallback"]),
        ("fallback_level", &["trajectory", "fallback_level"]),
        ("workpoint_next_slice", &["workpoint", "next_slice"]),
        ("active_blocker", &["active_blocker", "present"]),
        ("constraints", &["focus_state", "constraints"]),
        ("decisions", &["focus_state", "decisions"]),
        ("evidence_refs", &["evidence", "evidence_refs"]),
        ("receipt_expectation", &["receipt_expectation"]),
        ("do_not_use", &["next", "do_not_use"]),
        ("rehydrate_refs", &["bloatgaurd", "rehydrate_refs"]),
    ];
    fn at_path<'a>(value: &'a Value, path: &[&str]) -> Option<&'a Value> {
        path.iter().try_fold(value, |cursor, key| cursor.get(*key))
    }
    let missing: Vec<&str> = REQUIRED
        .iter()
        .filter(|(_, path)| at_path(packet, path).is_none_or(Value::is_null))
        .map(|(name, _)| *name)
        .collect();
    let generic_authority = packet["trajectory"]["generic_bootstrap"] == true
        && packet["trajectory"]["action_authority_from_trajectory"] == true;
    let missing_rehydrate = packet["bloatgaurd"]["rehydrate_refs"]
        .as_array()
        .is_none_or(Vec::is_empty);
    let expected = REQUIRED.len();
    let preserved = expected.saturating_sub(missing.len());
    let score = if expected == 0 {
        1.0
    } else {
        preserved as f64 / expected as f64
    };
    let status = if generic_authority || missing.len() > 3 {
        "fail"
    } else if !missing.is_empty() || missing_rehydrate {
        "warn"
    } else {
        "pass"
    };
    json!({
        "schema": "focusa.compaction_fidelity_eval.v1",
        "packet_id": packet["packet_id"],
        "status": status,
        "required_fields": { "expected": expected, "preserved": preserved, "missing": missing },
        "authority_failures": if generic_authority { vec!["generic_hlt_authority"] } else { Vec::<&str>::new() },
        "bloat_failures": if missing_rehydrate { vec!["missing_rehydrate_ref"] } else { Vec::<&str>::new() },
        "trajectory_failures": Vec::<String>::new(),
        "receipt_failures": Vec::<String>::new(),
        "metrics": {
            "preserved_required_fields_count": preserved,
            "missing_required_fields_count": missing.len(),
            "hallucinated_authority_count": usize::from(generic_authority),
            "lost_blocker_count": 0,
            "lost_hlt_warning_count": 0,
            "stale_hlt_backfill_detected": false,
            "missing_rehydrate_ref_count": usize::from(missing_rehydrate),
            "raw_tool_output_leak_count": 0,
            "generic_hlt_authority_count": usize::from(generic_authority),
            "completion_claim_without_receipt_count": 0
        },
        "score": score,
        "evidence_refs": packet["evidence"]["evidence_refs"]
    })
}

async fn evaluate(Json(req): Json<PacketIdRequest>) -> Result<Json<Value>, StatusCode> {
    packet_by_id(&req.packet_id)
        .map(|packet| Json(fidelity_eval(&packet)))
        .ok_or(StatusCode::NOT_FOUND)
}

async fn replay(Json(req): Json<PacketIdRequest>) -> Result<Json<Value>, StatusCode> {
    packet_by_id(&req.packet_id)
        .map(|packet| {
            Json(json!({
                "schema": "focusa.compaction_replay.v1",
                "status": "replayed_advisory",
                "canonical": false,
                "packet": packet,
                "next_tool": "focusa_workpoint_resume"
            }))
        })
        .ok_or(StatusCode::NOT_FOUND)
}

fn comparable_fields(packet: &Value) -> Value {
    json!({
        "status": packet["status"],
        "scope": packet["scope"],
        "current_ask": packet["current_ask"],
        "trajectory": packet["trajectory"],
        "workpoint": packet["workpoint"],
        "active_blocker": packet["active_blocker"],
        "next": packet["next"],
        "receipt_expectation": packet["receipt_expectation"]
    })
}

async fn diff(Json(req): Json<DiffCompactionPacketRequest>) -> Result<Json<Value>, StatusCode> {
    let before = packet_by_id(&req.before).ok_or(StatusCode::NOT_FOUND)?;
    let after = packet_by_id(&req.after).ok_or(StatusCode::NOT_FOUND)?;
    let before_fields = comparable_fields(&before);
    let after_fields = comparable_fields(&after);
    let changed: Vec<String> = before_fields
        .as_object()
        .into_iter()
        .flat_map(|map| map.keys())
        .filter(|key| before_fields.get(*key) != after_fields.get(*key))
        .cloned()
        .collect();
    Ok(Json(json!({
        "schema": "focusa.compaction_diff.v1",
        "before": req.before,
        "after": req.after,
        "changed_fields": changed,
        "before_fields": before_fields,
        "after_fields": after_fields
    })))
}

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/compaction/build", post(build))
        .route("/v1/compaction/packet/{packet_id}", get(get_packet))
        .route("/v1/compaction/inspect/{packet_id}", get(inspect))
        .route("/v1/compaction/evaluate", post(evaluate))
        .route("/v1/compaction/replay", post(replay))
        .route("/v1/compaction/diff", post(diff))
}

#[cfg(test)]
mod tests {
    use super::*;
    use focusa_core::types::{HltStatus, TrajectoryProjectionRecord};

    #[test]
    fn packet_is_advisory_and_blocks_unsafe_scope() {
        let packet = build_packet(
            &FocusaState::new(),
            &BuildCompactionPacketRequest {
                resume_source: Some("before_compaction".into()),
                project_root: Some("/root".into()),
                continuity_id: Some("focusa-cont-test".into()),
                session_id: None,
                current_ask: Some("continue".into()),
                ask_kind: None,
                source_turn_id: None,
                omitted_sections: vec![],
                omitted_bytes: 0,
                omitted_tokens: 0,
                rehydrate_refs: vec![],
            },
        );
        assert_eq!(packet["schema_version"], PACKET_SCHEMA);
        assert_eq!(packet["canonical"], false);
        assert_eq!(packet["scope"]["scope_status"], "unsafe");
        assert_eq!(packet["status"], "blocked");
    }

    #[test]
    fn generic_hlt_never_grants_trajectory_authority() {
        let mut state = FocusaState::new();
        state.trajectory.records.push(TrajectoryProjectionRecord {
            trajectory_id: "generic".into(),
            long_term_goal: "Maintain project".into(),
            desired_end_state: "Done".into(),
            hlt_status: HltStatus::GenericDegraded,
            ..TrajectoryProjectionRecord::default()
        });
        state.trajectory.active_trajectory_id = Some("generic".into());
        let packet = build_packet(
            &state,
            &BuildCompactionPacketRequest {
                resume_source: None,
                project_root: Some("/tmp/safe-project".into()),
                continuity_id: None,
                session_id: None,
                current_ask: None,
                ask_kind: None,
                source_turn_id: None,
                omitted_sections: vec![],
                omitted_bytes: 0,
                omitted_tokens: 0,
                rehydrate_refs: vec![],
            },
        );
        assert_eq!(packet["trajectory"]["generic_bootstrap"], true);
        assert_eq!(
            packet["trajectory"]["action_authority_from_trajectory"],
            false
        );
        assert_ne!(packet["status"], "verified");
    }

    #[test]
    fn inspect_answers_authority_omission_and_next_tool_questions() {
        let packet = build_packet(
            &FocusaState::new(),
            &BuildCompactionPacketRequest {
                resume_source: Some("manual".into()),
                project_root: Some("/tmp/safe-project".into()),
                continuity_id: Some("focusa-cont-test".into()),
                session_id: Some("pi-test".into()),
                current_ask: Some("continue implementation".into()),
                ask_kind: Some("instruction".into()),
                source_turn_id: Some("pi-turn-1".into()),
                omitted_sections: vec!["raw_tool_history".into()],
                omitted_bytes: 1024,
                omitted_tokens: 256,
                rehydrate_refs: vec!["focusa_traverse".into()],
            },
        );
        let inspect = inspect_payload(&packet);
        assert_eq!(inspect["schema"], "focusa.compaction_inspect.v1");
        assert_eq!(inspect["omitted"][0], "raw_tool_history");
        assert_eq!(inspect["exact_next_tool"], "focusa_workpoint_resume");
        assert!(inspect.get("authority_surface").is_some());
        assert!(inspect.get("receipt_expectation").is_some());
    }

    #[test]
    fn fidelity_eval_flags_missing_fields_without_granting_authority() {
        let packet = build_packet(
            &FocusaState::new(),
            &BuildCompactionPacketRequest {
                resume_source: None,
                project_root: Some("/tmp/safe-project".into()),
                continuity_id: None,
                session_id: None,
                current_ask: None,
                ask_kind: None,
                source_turn_id: None,
                omitted_sections: vec![],
                omitted_bytes: 0,
                omitted_tokens: 0,
                rehydrate_refs: vec![],
            },
        );
        let evaluation = fidelity_eval(&packet);
        assert_eq!(evaluation["schema"], "focusa.compaction_fidelity_eval.v1");
        assert_ne!(evaluation["status"], "pass");
        assert!(
            evaluation["required_fields"]["missing"]
                .as_array()
                .is_some()
        );
        assert_eq!(evaluation["metrics"]["generic_hlt_authority_count"], 0);
    }
}
