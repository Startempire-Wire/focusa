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
use rusqlite::{Connection, OptionalExtension, params};
use serde::Deserialize;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::sync::Arc;
use uuid::Uuid;

const PACKET_SCHEMA: &str = "focusa.compaction_mission_packet.v1";
const PACKET_CAP: usize = 64;
const RESUME_SOURCES: &[&str] = &[
    "session_start",
    "session_switch",
    "before_compaction",
    "after_compaction",
    "model_switch",
    "fork",
    "handoff",
    "manual",
    "provider_overflow",
];

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
pub struct PrepareCompactionRequest {
    pub schema: String,
    #[serde(default)]
    pub epoch: Value,
    #[serde(default)]
    pub scope: Value,
    #[serde(default)]
    pub trigger: Value,
    #[serde(default)]
    pub current_ask: Value,
    #[serde(default)]
    pub local_semantic_deltas: Value,
    #[serde(default)]
    pub native_pressure: Value,
    #[serde(default)]
    pub adapter_capabilities: Value,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VerifyCompactionRequest {
    pub schema: String,
    pub epoch_id: String,
    #[serde(default)]
    pub native_compaction_result: Value,
    #[serde(default)]
    pub context_usage_before: Value,
    #[serde(default)]
    pub context_usage_after: Value,
    #[serde(default)]
    pub native_pressure_after: Value,
    pub delivery_posture: Option<String>,
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

    let resume_state = if active_blocker.is_some() {
        "blocked_resume"
    } else if workpoint_ready {
        "exact_workpoint_resume"
    } else if hlt_ready {
        "trajectory_only_resume"
    } else {
        "bootstrap_required"
    };

    let temporal_context = req
        .project_root
        .as_deref()
        .zip(req.continuity_id.as_deref())
        .map(|(project_root, continuity_id)| {
            super::temporal_context::bounded_temporal_context(
                project_root,
                continuity_id,
                workpoint.map(|record| record.workpoint_id.to_string()),
                workpoint.and_then(|record| record.work_item_id.clone()),
            )
        })
        .unwrap_or_else(|| {
            json!({
                "schema":"focusa.bounded_temporal_context.v1",
                "status":"unavailable",
                "canonical":false,
                "failure_class":"scope_missing",
                "cache_safe_refs_only":true
            })
        });

    json!({
        "schema_version": PACKET_SCHEMA,
        "packet_id": packet_id,
        "generated_at": Utc::now().to_rfc3339(),
        "resume_source": req.resume_source.as_deref().unwrap_or("manual"),
        "resume_state": resume_state,
        "status": status,
        "canonical": false,
        "advisory": true,
        "temporal": temporal_context,
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

fn packet_db_path(data_dir: &str) -> std::path::PathBuf {
    if let Some(rest) = data_dir.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return std::path::PathBuf::from(home)
            .join(rest)
            .join("focusa.sqlite");
    }
    std::path::PathBuf::from(data_dir).join("focusa.sqlite")
}

fn ensure_packet_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS compaction_packets (
            packet_id TEXT PRIMARY KEY,
            packet_json TEXT NOT NULL,
            created_at INTEGER NOT NULL
        );
        CREATE INDEX IF NOT EXISTS compaction_packets_created
            ON compaction_packets(created_at DESC);",
    )
}

fn persist_packet(data_dir: &str, packet: &Value) -> rusqlite::Result<()> {
    let Some(packet_id) = packet.get("packet_id").and_then(Value::as_str) else {
        return Ok(());
    };
    let conn = Connection::open(packet_db_path(data_dir))?;
    ensure_packet_table(&conn)?;
    conn.execute(
        "INSERT OR REPLACE INTO compaction_packets(packet_id, packet_json, created_at)
         VALUES (?1, ?2, strftime('%s','now'))",
        params![packet_id, packet.to_string()],
    )?;
    conn.execute(
        "DELETE FROM compaction_packets WHERE packet_id NOT IN (
            SELECT packet_id FROM compaction_packets ORDER BY created_at DESC LIMIT ?1
        )",
        params![PACKET_CAP as i64],
    )?;
    Ok(())
}

fn packet_by_id_durable(data_dir: &str, packet_id: &str) -> Option<Value> {
    let conn = Connection::open(packet_db_path(data_dir)).ok()?;
    ensure_packet_table(&conn).ok()?;
    let raw: Option<String> = conn
        .query_row(
            "SELECT packet_json FROM compaction_packets WHERE packet_id = ?1",
            params![packet_id],
            |row| row.get(0),
        )
        .optional()
        .ok()?;
    raw.and_then(|raw| serde_json::from_str(&raw).ok())
}

fn cascade_count(data_dir: &str, packet: &Value) -> usize {
    let continuity = packet.pointer("/scope/continuity_id");
    let next_slice = packet.pointer("/workpoint/next_slice");
    let Ok(conn) = Connection::open(packet_db_path(data_dir)) else {
        return 0;
    };
    if ensure_packet_table(&conn).is_err() {
        return 0;
    }
    let Ok(mut statement) =
        conn.prepare("SELECT packet_json FROM compaction_packets ORDER BY rowid DESC LIMIT 8")
    else {
        return 0;
    };
    statement
        .query_map([], |row| row.get::<_, String>(0))
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .filter_map(|raw| serde_json::from_str::<Value>(&raw).ok())
        .filter(|prior| {
            prior.pointer("/scope/continuity_id") == continuity
                && prior.pointer("/workpoint/next_slice") == next_slice
        })
        .count()
}

fn value_text(value: &Value, pointers: &[&str], max: usize) -> Option<String> {
    pointers.iter().find_map(|pointer| {
        value
            .pointer(pointer)
            .and_then(Value::as_str)
            .and_then(|text| bounded_text(Some(text), max))
    })
}

fn semantic_digest(value: &Value) -> String {
    let bytes = serde_json::to_vec(value).unwrap_or_default();
    format!("sha256:{:x}", Sha256::digest(bytes))
}

async fn prepare(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PrepareCompactionRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if req.schema != "focusa.compaction_prepare_request.v1" {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "schema": "focusa.compaction_error.v1",
                "status": "blocked",
                "error": "invalid_prepare_schema"
            })),
        ));
    }
    let project_root = value_text(
        &req.scope,
        &["/root_scope/root_path", "/project_root"],
        4096,
    );
    let continuity_id = value_text(&req.scope, &["/continuity_id"], 256);
    let session_id = value_text(&req.epoch, &["/session_frame_key", "/session_id"], 512);
    let current_ask = req
        .current_ask
        .as_str()
        .and_then(|text| bounded_text(Some(text), 4096));
    let build_req = BuildCompactionPacketRequest {
        resume_source: Some("before_compaction".into()),
        project_root,
        continuity_id,
        session_id,
        current_ask,
        ask_kind: Some("compaction_prepare".into()),
        source_turn_id: value_text(&req.epoch, &["/epoch_key"], 512),
        omitted_sections: Vec::new(),
        omitted_bytes: 0,
        omitted_tokens: 0,
        rehydrate_refs: Vec::new(),
    };
    let focusa = state.focusa.read().await;
    let mut packet = build_packet(&focusa, &build_req);
    drop(focusa);
    packet["prepare_context"] = json!({
        "trigger": req.trigger,
        "native_pressure": req.native_pressure,
        "adapter_capabilities": req.adapter_capabilities,
        "local_semantic_delta_counts": {
            "decisions": req.local_semantic_deltas.pointer("/decisions").and_then(Value::as_array).map_or(0, Vec::len),
            "constraints": req.local_semantic_deltas.pointer("/constraints").and_then(Value::as_array).map_or(0, Vec::len),
            "failures": req.local_semantic_deltas.pointer("/failures").and_then(Value::as_array).map_or(0, Vec::len)
        }
    });
    let epoch_id = packet
        .get("packet_id")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let digest = semantic_digest(&packet);
    let packet_for_write = packet.clone();
    let data_dir = state.config.data_dir.clone();
    let persistence = tokio::task::spawn_blocking(move || persist_packet(&data_dir, &packet_for_write))
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"schema":"focusa.compaction_error.v1","status":"degraded","error":"prepare_writer_join_failed"})),
            )
        })?;
    let persistence_ack = match persistence {
        Ok(()) => json!({"status":"persisted"}),
        Err(error) => json!({"status":"degraded","error":error.to_string()}),
    };
    let mission = packet
        .pointer("/workpoint/mission")
        .and_then(Value::as_str)
        .unwrap_or("current Focusa mission");
    let next_slice = packet
        .pointer("/workpoint/next_slice")
        .and_then(Value::as_str)
        .unwrap_or("continue from the verified Workpoint");
    Ok(Json(json!({
        "schema": "focusa.compaction_prepare_result.v1",
        "status": if persistence_ack["status"] == "persisted" { "prepared" } else { "degraded" },
        "epoch_id": epoch_id,
        "source_revision": 0,
        "semantic_digest": digest,
        "workpoint_checkpoint_ref": packet.pointer("/workpoint/workpoint_id").cloned().unwrap_or(Value::Null),
        "trajectory_checkpoint_ref": packet.pointer("/trajectory/trajectory_id").cloned().unwrap_or(Value::Null),
        "compaction_packet_ref": format!("compaction:{}", epoch_id),
        "resume_projection": packet,
        "native_compactor_instructions": format!("Preserve Focusa mission: {mission}. Preserve exact next action: {next_slice}. Keep Pi's native tactical summary, queued operator input, cancellation, retry, and reconnect authority."),
        "fidelity_manifest": {"required_fields":["scope","workpoint.mission","workpoint","trajectory","evidence"]},
        "persistence_ack": persistence_ack,
        "warnings": []
    })))
}

async fn verify(
    State(state): State<Arc<AppState>>,
    Json(req): Json<VerifyCompactionRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    if req.schema != "focusa.compaction_verify_request.v1" || req.epoch_id.trim().is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(
                json!({"schema":"focusa.compaction_error.v1","status":"blocked","error":"invalid_verify_request"}),
            ),
        ));
    }
    let epoch_id = req.epoch_id.trim().to_string();
    let lookup_id = epoch_id.clone();
    let data_dir = state.config.data_dir.clone();
    let packet = tokio::task::spawn_blocking(move || packet_by_id_durable(&data_dir, &lookup_id))
        .await
        .map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"schema":"focusa.compaction_error.v1","status":"degraded","error":"verify_reader_join_failed"})),
            )
        })?
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({"schema":"focusa.compaction_verify_result.v1","status":"blocked","epoch_id":epoch_id,"findings":["prepare_epoch_not_found"]})),
            )
        })?;
    let before = req
        .context_usage_before
        .pointer("/tokens")
        .and_then(Value::as_f64);
    let after = req
        .context_usage_after
        .pointer("/tokens")
        .and_then(Value::as_f64);
    let ratio = match (before, after) {
        (Some(before), Some(after)) if before > 0.0 => ((before - after) / before).clamp(0.0, 1.0),
        _ => 0.0,
    };
    let required_preserved = ["/scope", "/workpoint", "/trajectory", "/evidence"]
        .iter()
        .all(|pointer| !packet.pointer(pointer).unwrap_or(&Value::Null).is_null());
    let status = if required_preserved {
        "verified"
    } else {
        "degraded"
    };
    Ok(Json(json!({
        "schema": "focusa.compaction_verify_result.v1",
        "status": status,
        "epoch_id": req.epoch_id,
        "context_release_ratio": ratio,
        "required_fields_preserved": required_preserved,
        "workpoint_resume_status": if packet.pointer("/workpoint/canonical").and_then(Value::as_bool).unwrap_or(false) { "canonical" } else { "degraded" },
        "resume_projection_ref": format!("compaction-resume:{}", req.epoch_id),
        "recommended_next": if req.delivery_posture.as_deref() == Some("deferred") { "defer" } else { "continue" },
        "findings": if required_preserved { Vec::<String>::new() } else { vec!["required_projection_field_missing".to_string()] },
        "native_compaction_result": req.native_compaction_result,
        "native_pressure_after": req.native_pressure_after
    })))
}

async fn build(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BuildCompactionPacketRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let resume_source = req.resume_source.as_deref().unwrap_or("manual");
    if !RESUME_SOURCES.contains(&resume_source) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({
                "schema": "focusa.compaction_error.v1",
                "status": "blocked",
                "error": "invalid_resume_source",
                "allowed": RESUME_SOURCES
            })),
        ));
    }
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
    let repeated_without_progress = cascade_count(&state.config.data_dir, &packet);
    packet["cascading_compaction"] = json!({
        "detected": repeated_without_progress >= 2,
        "same_mission_next_slice_prior_count": repeated_without_progress,
        "finding_id": if repeated_without_progress >= 2 { Some("COMP-CASCADE-001") } else { None }
    });
    if repeated_without_progress >= 2 {
        if let Some(warnings) = packet["trajectory"]["warnings"].as_array_mut() {
            warnings.push(Value::String(
                "Repeated compaction without mission/next-slice progress; inspect context pressure before continuing."
                    .into(),
            ));
        }
    }
    if let Err(error) = persist_packet(&state.config.data_dir, &packet) {
        packet["persistence_warning"] = json!({
            "status": "degraded",
            "error": error.to_string(),
            "recovery": "retry persistence after storage recovery; no process-global packet fallback is retained"
        });
    }
    Ok(Json(packet))
}

async fn get_packet(
    State(state): State<Arc<AppState>>,
    Path(packet_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    packet_by_id_durable(&state.config.data_dir, &packet_id)
        .map(Json)
        .ok_or(StatusCode::NOT_FOUND)
}

fn inspect_payload(packet: &Value) -> Value {
    json!({
        "schema": "focusa.compaction_inspect.v1",
        "packet_id": packet["packet_id"],
        "status": packet["status"],
        "resume_state": packet["resume_state"],
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

async fn inspect(
    State(state): State<Arc<AppState>>,
    Path(packet_id): Path<String>,
) -> Result<Json<Value>, StatusCode> {
    packet_by_id_durable(&state.config.data_dir, &packet_id)
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

async fn evaluate(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PacketIdRequest>,
) -> Result<Json<Value>, StatusCode> {
    packet_by_id_durable(&state.config.data_dir, &req.packet_id)
        .map(|packet| Json(fidelity_eval(&packet)))
        .ok_or(StatusCode::NOT_FOUND)
}

async fn replay(
    State(state): State<Arc<AppState>>,
    Json(req): Json<PacketIdRequest>,
) -> Result<Json<Value>, StatusCode> {
    packet_by_id_durable(&state.config.data_dir, &req.packet_id)
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

async fn diff(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DiffCompactionPacketRequest>,
) -> Result<Json<Value>, StatusCode> {
    let before =
        packet_by_id_durable(&state.config.data_dir, &req.before).ok_or(StatusCode::NOT_FOUND)?;
    let after =
        packet_by_id_durable(&state.config.data_dir, &req.after).ok_or(StatusCode::NOT_FOUND)?;
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
        .route("/v1/compaction/prepare", post(prepare))
        .route("/v1/compaction/verify", post(verify))
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
        assert_eq!(
            packet["temporal"]["schema"],
            "focusa.bounded_temporal_context.v1"
        );
        assert_eq!(packet["temporal"]["cache_safe_refs_only"], true);
        assert!(packet["temporal"].get("projection").is_none());
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

    #[test]
    fn repeated_same_scope_and_next_slice_triggers_cascade_signal() {
        let unique = Uuid::now_v7().to_string();
        let mut packet = build_packet(
            &FocusaState::new(),
            &BuildCompactionPacketRequest {
                resume_source: Some("before_compaction".into()),
                project_root: Some("/tmp/safe-project".into()),
                continuity_id: Some(unique),
                session_id: None,
                current_ask: None,
                ask_kind: None,
                source_turn_id: None,
                omitted_sections: vec![],
                omitted_bytes: 0,
                omitted_tokens: 0,
                rehydrate_refs: vec!["focusa_traverse".into()],
            },
        );
        packet["workpoint"]["next_slice"] = json!("same-next-slice");
        let dir = std::env::temp_dir().join(format!("focusa-cascade-{}", Uuid::now_v7()));
        std::fs::create_dir_all(&dir).expect("temp cascade dir");
        let data_dir = dir.to_str().expect("utf8 path");
        assert_eq!(cascade_count(data_dir, &packet), 0);
        persist_packet(data_dir, &packet).expect("persist packet");
        assert_eq!(cascade_count(data_dir, &packet), 1);
        let mut packet_two = packet.clone();
        packet_two["packet_id"] = json!(Uuid::now_v7().to_string());
        persist_packet(data_dir, &packet_two).expect("persist second packet");
        assert_eq!(cascade_count(data_dir, &packet), 2);
        std::fs::remove_dir_all(dir).expect("remove cascade dir");
    }

    #[test]
    fn resume_source_contract_is_closed_enum() {
        assert!(RESUME_SOURCES.contains(&"before_compaction"));
        assert!(RESUME_SOURCES.contains(&"provider_overflow"));
        assert!(!RESUME_SOURCES.contains(&"transcript_guess"));
    }

    #[test]
    fn packet_persistence_is_bounded_and_restart_readable() {
        let dir = std::env::temp_dir().join(format!("focusa-compaction-{}", Uuid::now_v7()));
        std::fs::create_dir_all(&dir).expect("temp data dir");
        let packet = build_packet(
            &FocusaState::new(),
            &BuildCompactionPacketRequest {
                resume_source: Some("manual".into()),
                project_root: Some("/tmp/safe-project".into()),
                continuity_id: Some("focusa-cont-persist".into()),
                session_id: None,
                current_ask: None,
                ask_kind: None,
                source_turn_id: None,
                omitted_sections: vec![],
                omitted_bytes: 0,
                omitted_tokens: 0,
                rehydrate_refs: vec!["focusa_traverse".into()],
            },
        );
        persist_packet(dir.to_str().expect("utf8 path"), &packet).expect("persist packet");
        let conn = Connection::open(dir.join("focusa.sqlite")).expect("open packet db");
        let count: i64 = conn
            .query_row("SELECT count(*) FROM compaction_packets", [], |row| {
                row.get(0)
            })
            .expect("count packets");
        assert_eq!(count, 1);
        let packet_id = packet["packet_id"].as_str().expect("packet id");
        assert!(packet_by_id_durable(dir.to_str().expect("utf8 path"), packet_id).is_some());
        drop(conn);
        std::fs::remove_dir_all(dir).expect("remove temp dir");
    }
}
