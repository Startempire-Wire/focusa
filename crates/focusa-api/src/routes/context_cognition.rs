//! GET /v1/context-cognition — Spec 100 Context Cognition packet builder.
//!
//! Returns a bounded `ContextCognitionPacket` scoped to `project_root`.
//! Advisory only; never mutates state. v0: builds the envelope from
//! existing read models (workpoint, trajectory, HLT, evidence). The
//! Context Curator (Spec 100 P3) and Cognition Optimizer (Spec 100 P5)
//! are deferred to follow-up slices.

use crate::routes::project::project_identity_payload_for_scope;
use crate::server::AppState;
use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;
use focusa_core::types::{
    ContextCognitionAuthority, ContextCognitionEvidenceFrame, ContextCognitionFreshness,
    ContextCognitionOntologyFrame, ContextCognitionOptimizationFrame,
    ContextCognitionReasoningFrame, ContextCognitionRecommendedPacketUse,
    ContextCognitionRouteFrame, ContextCognitionScope, ContextCognitionSelectedContext,
    ContextCognitionPacket,
};
use serde_json::{Value, json};
use std::sync::Arc;

const SCHEMA_VERSION: &str = "focusa.context_cognition_packet.v1";

pub fn router() -> axum::Router<Arc<AppState>> {
    axum::Router::new().route("/v1/context-cognition", axum::routing::get(view))
}

fn rejection(status: StatusCode, body: Value) -> (StatusCode, Json<Value>) {
    (status, Json(body))
}

fn default_authority() -> ContextCognitionAuthority {
    ContextCognitionAuthority {
        action_authority: "workpoint".to_string(),
        goal_context: "trajectory".to_string(),
        semantic_context: "ontology".to_string(),
        proof_context: "evidence".to_string(),
        canonical_mutation_allowed: false,
    }
}

fn default_route_frame() -> ContextCognitionRouteFrame {
    ContextCognitionRouteFrame {
        next_tools: vec![
            "focusa_active_object_resolve".to_string(),
            "focusa_workpoint_checkpoint".to_string(),
            "focusa_evidence_capture".to_string(),
        ],
        recovery_tools: vec![
            "focusa_workpoint_resume".to_string(),
            "focusa_trajectory_view".to_string(),
            "focusa_tool_doctor".to_string(),
        ],
        do_not_use_by_default: vec![
            "full lineage tree".to_string(),
            "full ontology graph".to_string(),
            "full telemetry logs".to_string(),
        ],
    }
}

fn default_recommended_use() -> ContextCognitionRecommendedPacketUse {
    ContextCognitionRecommendedPacketUse {
        include_in_prompt: vec![
            "scope.project_root".to_string(),
            "scope.continuity_id".to_string(),
            "authority.action_authority".to_string(),
            "reasoning_frame.likely_goal".to_string(),
        ],
        exclude_from_prompt: vec![
            "excluded_context".to_string(),
        ],
        next_tools: vec![
            "focusa_active_object_resolve".to_string(),
            "focusa_workpoint_checkpoint".to_string(),
        ],
        do_not_drift: vec![
            "transcript_tail as authority".to_string(),
            "cross-project scope fallbacks".to_string(),
        ],
    }
}

fn build_empty_packet(
    project_root: &str,
    continuity_id: Option<String>,
    scope_status: &str,
) -> ContextCognitionPacket {
    ContextCognitionPacket {
        schema_version: SCHEMA_VERSION.to_string(),
        status: "completed".to_string(),
        advisory: true,
        canonical: false,
        scope_status: scope_status.to_string(),
        freshness: ContextCognitionFreshness {
            generated_at: Some(Utc::now()),
            stale: false,
            source_snapshot: None,
        },
        scope: ContextCognitionScope {
            project_root: project_root.to_string(),
            continuity_id,
            session_id: None,
            workpoint_id: None,
            trajectory_id: None,
        },
        authority: default_authority(),
        selected_context: ContextCognitionSelectedContext::default(),
        ontology_frame: ContextCognitionOntologyFrame::default(),
        evidence_frame: ContextCognitionEvidenceFrame::default(),
        reasoning_frame: ContextCognitionReasoningFrame::default(),
        optimization_frame: ContextCognitionOptimizationFrame::default(),
        route_frame: default_route_frame(),
        side_effects: Vec::new(),
        evidence_refs: Vec::new(),
        recommended_packet_use: default_recommended_use(),
    }
}

async fn view(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<ContextCognitionRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let project_root = query
        .project_root
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("");
    if project_root.is_empty() {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "canonical": false,
                "advisory": true,
                "failure_class": "project_root_missing",
                "field": "project_root",
                "message": "project_root is required",
            }),
        ));
    }
    if is_unsafe_agent_runtime_path_inline(project_root) {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "canonical": false,
                "advisory": true,
                "failure_class": "scope_mismatch",
                "field": "project_root",
                "rejected_value": project_root,
                "message": "agent runtime paths are not allowed as project_root",
            }),
        ));
    }
    let identity = project_identity_payload_for_scope(Some(project_root), Some(project_root));
    let identity_status = identity
        .get("project_identity")
        .and_then(|pi: &Value| pi.get("status"))
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    if identity_status == "unsafe_project_root" {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "canonical": false,
                "advisory": true,
                "failure_class": "project_root_unverified",
                "field": "project_root",
                "message": "project_root is not verified",
            }),
        ));
    }

    let continuity_id = query
        .continuity_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let scope_status = match identity_status {
        "verified" => "matched",
        "unverified" => "partial",
        _ => "missing",
    };

    let mut packet = build_empty_packet(project_root, continuity_id.clone(), scope_status);

    // Read FocusState (read-only).
    let focusa_state = state.focusa.read().await.clone();

    // Wire active workpoint id from FocusState (read-only).
    if let Some(wp) = focusa_state
        .workpoint
        .records
        .iter()
        .find(|r| r.project_root.as_deref() == Some(project_root))
    {
        packet.scope.workpoint_id = Some(wp.workpoint_id.to_string());
    }
    if let Some(tid) = focusa_state.trajectory.active_trajectory_id.clone() {
        packet.scope.trajectory_id = Some(tid);
    }

    // HLT freshness from HLT ledger (append-only, no mutation).
    if let Ok(Some(latest)) = state
        .persistence
        .latest_hlt_for_project(project_root, continuity_id.as_deref())
    {
        packet.optimization_frame.baseline_score = Some(0.0);
        packet.optimization_frame.eval_score = Some(0.0);
        packet.optimization_frame.module_name = Some("context_cognition".to_string());
        packet
            .evidence_refs
            .push(format!("hlt:{}", latest.event_id));
    }

    // Evidence refs from existing workpoint verification_records (read-only).
    if let Some(wp) = focusa_state
        .workpoint
        .records
        .iter()
        .find(|r| r.project_root.as_deref() == Some(project_root))
    {
        let handles: Vec<String> = wp
            .verification_records
            .iter()
            .filter_map(|v| v.evidence_ref.clone())
            .take(8)
            .collect();
        for handle in handles {
            packet.evidence_refs.push(handle);
        }
    }

    // Ontology: surface "focusa" + "ProjectIdentity" affordances (advisory).
    packet.ontology_frame.affordances = vec![
        "focusa_project_identity".to_string(),
        "focusa_trajectory_view".to_string(),
        "focusa_workpoint_resume".to_string(),
    ];

    // Reasoning: placeholders; Spec 100 P3 (Curator) and P5 (Optimizer) fill these in.
    packet.reasoning_frame.likely_goal = Some("focusa.spec100.p1.packet_schema".to_string());
    packet.reasoning_frame.active_gap = Some("context_curator_unimplemented_v0".to_string());
    packet.reasoning_frame.confidence = Some(0.5);

    Ok(Json(json!({
        "status": "completed",
        "canonical": false,
        "advisory": true,
        "scope_status": packet.scope_status,
        "packet": packet,
        "next_tools": packet.route_frame.next_tools,
        "rehydrate_id": packet.scope.workpoint_id
            .clone()
            .unwrap_or_else(|| "ctx_cognition:v0".to_string()),
    })))
}

#[derive(Debug, serde::Deserialize)]
pub struct ContextCognitionRequest {
    pub project_root: Option<String>,
    pub continuity_id: Option<String>,
    pub session_id: Option<String>,
    pub include_rehydrate_refs: Option<bool>,
}

fn is_unsafe_agent_runtime_path_inline(path: &str) -> bool {
    const BLOCKED: &[&str] = &[
        "/root/pi-mono",
        "/root/.pi",
        "/root/.claude",
        "/root/.opencode",
        "/root/.letta",
    ];
    BLOCKED.iter().any(|p| path == *p || path.starts_with(&format!("{}/", p)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_authority_is_advisory() {
        let a = default_authority();
        assert_eq!(a.action_authority, "workpoint");
        assert!(!a.canonical_mutation_allowed);
    }

    #[test]
    fn default_route_frame_lists_next_and_recovery() {
        let r = default_route_frame();
        assert!(!r.next_tools.is_empty());
        assert!(!r.recovery_tools.is_empty());
        assert!(!r.do_not_use_by_default.is_empty());
    }

    #[test]
    fn empty_packet_uses_schema_v1() {
        let p = build_empty_packet("/tmp/x", None, "matched");
        assert_eq!(p.schema_version, SCHEMA_VERSION);
        assert!(p.advisory);
        assert!(!p.canonical);
        assert_eq!(p.scope_status, "matched");
    }
}
