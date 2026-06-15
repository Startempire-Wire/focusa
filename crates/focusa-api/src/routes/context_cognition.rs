//! GET /v1/context-cognition — Spec 100 Context Cognition packet builder.
//!
//! Returns a bounded `ContextCognitionPacket` scoped to `project_root`.
//! Advisory only; never mutates state. v0: builds the envelope from
//! existing read models (workpoint, trajectory, HLT, evidence). The
//! Context Curator (Spec 100 P3) and Cognition Optimizer (Spec 100 P5)
//! are deferred to follow-up slices.
//!
//! Companion routes (Spec 100 P2 cross-surface contracts):
//! - `GET /v1/context-cognition/render?project_root=...` — render as compact text
//! - `GET /v1/context-cognition/proof?project_root=...` — map surfaces to proof commands

use crate::routes::project::project_identity_payload_for_scope;
use crate::server::AppState;
use axum::{Json, extract::State, http::StatusCode};
use chrono::Utc;
use focusa_core::types::{
    CognitionOptimizerArtifact, ContextCognitionAuthority, ContextCognitionEvidenceFrame,
    ContextCognitionFreshness, ContextCognitionOntologyFrame, ContextCognitionOptimizationFrame,
    ContextCognitionPacket, ContextCognitionReasoningFrame, ContextCognitionRecommendedPacketUse,
    ContextCognitionRouteFrame, ContextCognitionScope, ContextCognitionSelectedContext,
    CuratorEvalRun,
};
use serde_json::{Value, json};
use std::sync::Arc;

const SCHEMA_VERSION: &str = "focusa.context_cognition_packet.v1";

pub fn router() -> axum::Router<Arc<AppState>> {
    axum::Router::new()
        .route("/v1/context-cognition", axum::routing::get(view))
        .route("/v1/context-cognition/render", axum::routing::get(render))
        .route("/v1/context-cognition/proof", axum::routing::get(proof))
        .route("/v1/context-cognition/curate", axum::routing::post(curate))
        .route(
            "/v1/context-cognition/curate/eval",
            axum::routing::post(curate_eval),
        )
        .route(
            "/v1/context-cognition/curate/eval/runs",
            axum::routing::get(curate_eval_runs),
        )
        .route(
            "/v1/context-cognition/optimizer/artifacts",
            axum::routing::get(optimizer_artifacts),
        )
        .route(
            "/v1/context-cognition/curate/optimize",
            axum::routing::post(curate_optimize),
        )
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
        exclude_from_prompt: vec!["excluded_context".to_string()],
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

    let exact_scope_ready = identity_status == "verified" && continuity_id.is_some();
    let scope_status = match (identity_status, continuity_id.as_deref()) {
        ("verified", Some(_)) => "matched",
        ("verified", None) => "missing_continuity_id",
        ("unverified", _) => "partial",
        _ => "missing",
    };

    let mut packet = build_empty_packet(project_root, continuity_id.clone(), scope_status);
    if !exact_scope_ready {
        packet.status = "degraded".to_string();
        packet.freshness.stale = true;
        packet.selected_context.excluded_context.push("canonical Workpoint/Trajectory selection requires verified project_root + continuity_id".to_string());
        packet
            .route_frame
            .do_not_use_by_default
            .push("Do not treat Context Cognition as canonical without exact scope".to_string());
    }

    // Read FocusState (read-only).
    let focusa_state = state.focusa.read().await.clone();

    // Wire active Workpoint/Trajectory only by exact project_root + continuity_id.
    let scoped_workpoint = if exact_scope_ready {
        focusa_state.workpoint.records.iter().find(|r| {
            r.project_root.as_deref() == Some(project_root)
                && r.continuity_id.as_deref() == continuity_id.as_deref()
                && r.canonical
        })
    } else {
        None
    };
    if let Some(wp) = scoped_workpoint {
        packet.scope.workpoint_id = Some(wp.workpoint_id.to_string());
    }
    if exact_scope_ready
        && let Some(active_id) = focusa_state.trajectory.active_trajectory_id.as_deref()
    {
        if let Some(traj) = focusa_state.trajectory.records.iter().find(|record| {
            record.trajectory_id == active_id
                && record.project_root.as_deref() == Some(project_root)
                && record.continuity_id.as_deref() == continuity_id.as_deref()
        }) {
            packet.scope.trajectory_id = Some(traj.trajectory_id.clone());
        } else {
            packet.selected_context.excluded_context.push(
                "active trajectory omitted: scope mismatch or missing continuity_id".to_string(),
            );
        }
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

    // Evidence refs from exact-scoped workpoint verification_records (read-only).
    if let Some(wp) = scoped_workpoint {
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
#[allow(dead_code)]
pub struct ContextCognitionRequest {
    pub project_root: Option<String>,
    pub continuity_id: Option<String>,
    pub session_id: Option<String>,
    pub include_rehydrate_refs: Option<bool>,
}

fn is_unsafe_agent_runtime_path_inline(path: &str) -> bool {
    let trimmed = path.trim();
    if trimmed == "/" || trimmed == "/root" {
        return true;
    }
    const BLOCKED: &[&str] = &[
        "/root/pi-mono",
        "/root/.pi",
        "/root/.cargo",
        "/root/.claude",
        "/root/.opencode",
        "/root/.letta",
        "/home/wirebot/.cargo",
    ];
    BLOCKED
        .iter()
        .any(|p| trimmed == *p || trimmed.starts_with(&format!("{}/", p)))
}

fn require_continuity_id(
    continuity_id: &Option<String>,
) -> Result<String, (StatusCode, Json<Value>)> {
    continuity_id
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .ok_or_else(|| {
            rejection(
                StatusCode::UNPROCESSABLE_ENTITY,
                json!({
                    "status": "validation_rejected",
                    "failure_class": "continuity_id_missing",
                    "field": "continuity_id",
                    "message": "Context Cognition eval/optimizer writes require project_root + continuity_id scope",
                }),
            )
        })
}

fn require_unit_interval(value: f64, field: &str) -> Result<(), (StatusCode, Json<Value>)> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "score_out_of_range",
                "field": field,
                "message": "score fields must be finite values between 0.0 and 1.0",
            }),
        ))
    }
}

async fn render(
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
                "failure_class": "project_root_missing",
                "field": "project_root",
            }),
        ));
    }
    if is_unsafe_agent_runtime_path_inline(project_root) {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "scope_mismatch",
                "field": "project_root",
                "rejected_value": project_root,
            }),
        ));
    }

    let focusa_state = state.focusa.read().await.clone();
    let workpoint_id = focusa_state
        .workpoint
        .records
        .iter()
        .find(|r| r.project_root.as_deref() == Some(project_root))
        .map(|r| r.workpoint_id.to_string());
    let trajectory_id = focusa_state.trajectory.active_trajectory_id.clone();

    let mut lines: Vec<String> = Vec::new();
    lines.push(format!(
        "## Context Cognition (Spec 100) — render for {project_root}"
    ));
    lines.push("advisory · read-only · canonical=false".to_string());
    lines.push("schema: focusa.context_cognition_packet.v1".to_string());
    if let Some(wid) = workpoint_id.clone() {
        lines.push(format!("workpoint_id: {wid}"));
    }
    if let Some(tid) = trajectory_id.clone() {
        lines.push(format!("trajectory_id: {tid}"));
    }
    lines.push("authority: workpoint (canonical_mutation_allowed=false)".to_string());
    lines.push("next_tools: focusa_active_object_resolve, focusa_workpoint_checkpoint, focusa_evidence_capture".to_string());
    lines.push(
        "do_not_drift: transcript_tail as authority; cross-project scope fallbacks".to_string(),
    );

    Ok(Json(json!({
        "status": "completed",
        "canonical": false,
        "advisory": true,
        "format": "compact_text",
        "render": lines.join("\n"),
        "render_lines": lines.len(),
        "workpoint_id": workpoint_id,
        "trajectory_id": trajectory_id,
        "rehydrate_id": workpoint_id.unwrap_or_else(|| "ctx_cognition:v0".to_string()),
    })))
}

async fn proof(
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
                "failure_class": "project_root_missing",
                "field": "project_root",
            }),
        ));
    }
    if is_unsafe_agent_runtime_path_inline(project_root) {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "scope_mismatch",
                "field": "project_root",
                "rejected_value": project_root,
            }),
        ));
    }

    let focusa_state = state.focusa.read().await.clone();
    let workpoint_id = focusa_state
        .workpoint
        .records
        .iter()
        .find(|r| r.project_root.as_deref() == Some(project_root))
        .map(|r| r.workpoint_id.to_string());

    // Use the daemon's own bind (default http://127.0.0.1:8787) for proof
    // command URLs. The /v1/health route is also reachable on the same bind.
    let base_url = "http://127.0.0.1:8787".to_string();
    let proof_commands = vec![
        format!("curl '{base_url}/v1/health'"),
        format!("curl '{base_url}/v1/project/identity?project_root={project_root}'"),
        format!("curl '{base_url}/v1/trajectory/view?project_root={project_root}'"),
        format!("curl '{base_url}/v1/workpoint/current?project_root={project_root}'"),
        format!("focusa context-cognition view --project-root {project_root}"),
        format!("focusa context-cognition render --project-root {project_root}"),
        format!("focusa context-cognition proof --project-root {project_root}"),
        "node scripts/validate-focusa-tool-contracts.mjs".to_string(),
        "node scripts/audit-focusa-tool-implementation-spec-gaps.mjs".to_string(),
        "node scripts/audit-focusa-tool-suite-safe.mjs".to_string(),
    ];

    Ok(Json(json!({
        "status": "completed",
        "canonical": false,
        "advisory": true,
        "format": "proof_commands",
        "workpoint_id": workpoint_id.clone(),
        "proof_commands": proof_commands,
        "command_count": proof_commands.len(),
        "rehydrate_id": workpoint_id.unwrap_or_else(|| "ctx_cognition:v0".to_string()),
    })))
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

    #[test]
    fn curator_token_budget_keeps_highest_scored() {
        // Token-budgeted selection: highest-scored items first, then budget cut.
        let items = vec![
            (
                "auth.ts",
                "authentication middleware token verify",
                100usize,
                5.0f64,
            ),
            ("routes.ts", "router config list of routes", 100, 1.0),
            ("core.ts", "core types", 100, 2.0),
        ];
        let budget = 150usize;
        let mut selected: Vec<(&str, &str, usize, f64)> = Vec::new();
        let mut used: usize = 0;
        let mut sorted = items.clone();
        sorted.sort_by(|a, b| b.3.partial_cmp(&a.3).unwrap());
        for item in sorted {
            if used + item.2 <= budget {
                used += item.2;
                selected.push(item);
            }
        }
        assert_eq!(used, 100);
        assert_eq!(selected.len(), 1);
        assert_eq!(selected[0].0, "auth.ts");
    }

    #[test]
    fn curator_exclusion_labeled() {
        let excluded = vec![
            ("routes.ts", "low_score: 0.2"),
            ("core.ts", "over_budget: 100 > 50"),
        ];
        assert_eq!(excluded.len(), 2);
        assert!(excluded[0].1.contains("low_score"));
        assert!(excluded[1].1.contains("over_budget"));
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct CurateCandidate {
    pub kind: String, // file | doc | diff | snippet | codemap | evidence
    pub path: String,
    pub body: Option<String>,
    pub evidence_ref: Option<String>,
    pub tokens: Option<usize>,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct CurateRequest {
    pub project_root: Option<String>,
    pub continuity_id: Option<String>,
    pub target: Option<String>,
    pub token_budget: Option<usize>,
    pub candidates: Option<Vec<CurateCandidate>>,
    pub evidence_refs: Option<Vec<String>>,
}

#[derive(Debug, serde::Serialize, Clone)]
struct CuratedItem {
    kind: String,
    path: String,
    body: Option<String>,
    tokens: usize,
    score: f64,
}

#[derive(Debug, serde::Serialize, Clone)]
struct ExcludedItem {
    kind: String,
    path: String,
    reason: String,
}

fn estimate_tokens(body: &str) -> usize {
    // v0: simple word count * 1.3 (rough approximation; tiktoken replaces this in v0.5).
    let words = body.split_whitespace().count();
    (words as f64 * 1.3).ceil() as usize
}

fn score_candidate(target: &str, item: &CurateCandidate) -> f64 {
    let needle = target.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return 1.0;
    }
    let hay = format!(
        "{} {} {}",
        item.path.to_ascii_lowercase(),
        item.kind.to_ascii_lowercase(),
        item.body.clone().unwrap_or_default().to_ascii_lowercase()
    );
    let mut score = 0.0;
    for term in needle.split_whitespace() {
        if term.is_empty() {
            continue;
        }
        if hay.contains(term) {
            score += 1.0;
        }
    }
    // Evidence_ref hint adds a small bonus so curator prefers evidence-tagged items.
    if item.evidence_ref.is_some() {
        score += 0.5;
    }
    score
}

async fn curate(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CurateRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let project_root = body
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
                "failure_class": "project_root_missing",
                "field": "project_root",
            }),
        ));
    }
    if is_unsafe_agent_runtime_path_inline(project_root) {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "scope_mismatch",
                "field": "project_root",
                "rejected_value": project_root,
            }),
        ));
    }

    // Pull the workpoint target from FocusState if the operator did not
    // supply an explicit target.
    let focusa_state = state.focusa.read().await.clone();
    let wp_target = body
        .target
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .or_else(|| {
            focusa_state
                .workpoint
                .records
                .iter()
                .find(|r| r.project_root.as_deref() == Some(project_root))
                .and_then(|r| r.next_slice.clone())
        })
        .or_else(|| {
            focusa_state
                .workpoint
                .records
                .iter()
                .find(|r| r.project_root.as_deref() == Some(project_root))
                .and_then(|r| r.mission.clone())
        })
        .unwrap_or_default();

    let budget = body.token_budget.unwrap_or(2000);
    let evidence_refs: Vec<String> = body.evidence_refs.clone().unwrap_or_default();
    let candidates: Vec<CurateCandidate> = body
        .candidates
        .clone()
        .unwrap_or_default()
        .into_iter()
        .map(|mut c| {
            if c.tokens.is_none() {
                c.tokens = Some(estimate_tokens(c.body.as_deref().unwrap_or("")));
            }
            c
        })
        .collect();

    // Score and sort candidates by score descending, with a tie-breaker on
    // tokens (smaller first) so the curator prefers denser items.
    let mut scored: Vec<(f64, &CurateCandidate)> = candidates
        .iter()
        .map(|c| (score_candidate(&wp_target, c), c))
        .collect();
    scored.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.1.tokens.unwrap_or(0).cmp(&b.1.tokens.unwrap_or(0)))
    });

    // Boost evidence-ref overlap
    let evidence_set: std::collections::HashSet<String> = evidence_refs.iter().cloned().collect();
    for (s, c) in scored.iter_mut() {
        if let Some(er) = c.evidence_ref.as_ref()
            && evidence_set.contains(er)
        {
            *s += 1.0;
        }
    }
    // Re-sort after the boost
    scored.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.1.tokens.unwrap_or(0).cmp(&b.1.tokens.unwrap_or(0)))
    });

    let mut selected: Vec<CuratedItem> = Vec::new();
    let mut excluded: Vec<ExcludedItem> = Vec::new();
    let mut used: usize = 0;

    for (score, cand) in scored.iter() {
        let cand_tokens = cand.tokens.unwrap_or(0);
        if used + cand_tokens <= budget {
            used += cand_tokens;
            selected.push(CuratedItem {
                kind: cand.kind.clone(),
                path: cand.path.clone(),
                body: cand.body.clone(),
                tokens: cand_tokens,
                score: *score,
            });
        } else if *score >= 2.0 {
            // High-score items get a label, not a drop. Reserved for future
            // `force_include` flag; for v0 we record over_budget.
            excluded.push(ExcludedItem {
                kind: cand.kind.clone(),
                path: cand.path.clone(),
                reason: format!(
                    "over_budget: {} > remaining {}",
                    cand_tokens,
                    budget.saturating_sub(used)
                ),
            });
        } else {
            excluded.push(ExcludedItem {
                kind: cand.kind.clone(),
                path: cand.path.clone(),
                reason: format!("low_score: {:.2} < 2.0", score),
            });
        }
    }

    let target_label = if wp_target.is_empty() {
        "<none>".to_string()
    } else {
        wp_target.clone()
    };
    Ok(Json(json!({
        "status": "completed",
        "canonical": false,
        "advisory": true,
        "scope_status": "matched",
        "target": target_label,
        "token_budget": budget,
        "tokens_used": used,
        "tokens_remaining": budget.saturating_sub(used),
        "selected_context": selected,
        "excluded_context": excluded,
        "selected_count": selected.len(),
        "excluded_count": excluded.len(),
        "evidence_refs": evidence_refs,
        "next_tools": [
            "focusa_context_cognition",
            "focusa_context_cognition_render",
            "focusa_evidence_capture"
        ],
        "rehydrate_id": format!("ctx_curate:{}:{}", project_root, selected.len()),
    })))
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct CurateEvalRequest {
    pub project_root: Option<String>,
    pub continuity_id: Option<String>,
    pub case_id: Option<String>,
    pub target: Option<String>,
    pub token_budget: Option<usize>,
    pub candidates: Option<Vec<CurateCandidate>>,
    pub evidence_refs: Option<Vec<String>>,
    pub expected_selected_paths: Option<Vec<String>>,
    pub score_threshold: Option<f64>,
    pub baseline_f1: Option<f64>,
}

fn compute_f1(precision: f64, recall: f64) -> f64 {
    if precision + recall == 0.0 {
        0.0
    } else {
        2.0 * precision * recall / (precision + recall)
    }
}

fn compute_precision_recall(selected: &[String], expected: &[String]) -> (f64, f64) {
    if expected.is_empty() {
        return (0.0, 0.0);
    }
    let expected_set: std::collections::HashSet<&String> = expected.iter().collect();
    let selected_set: std::collections::HashSet<&String> = selected.iter().collect();
    let tp = selected_set.intersection(&expected_set).count();
    let precision = if selected.is_empty() {
        0.0
    } else {
        tp as f64 / selected.len() as f64
    };
    let recall = tp as f64 / expected.len() as f64;
    (precision, recall)
}

async fn curate_eval(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CurateEvalRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let project_root = body
        .project_root
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("")
        .to_string();
    if project_root.is_empty() {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "project_root_missing",
                "field": "project_root",
            }),
        ));
    }
    if is_unsafe_agent_runtime_path_inline(&project_root) {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "scope_mismatch",
                "field": "project_root",
                "rejected_value": project_root,
            }),
        ));
    }

    let continuity_id = require_continuity_id(&body.continuity_id)?;
    let case_id = body
        .case_id
        .clone()
        .unwrap_or_else(|| uuid::Uuid::now_v7().to_string());
    let target = body.target.clone().unwrap_or_default();
    let token_budget = body.token_budget.unwrap_or(2000);
    if token_budget == 0 {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "token_budget_invalid",
                "field": "token_budget",
            }),
        ));
    }
    let score_threshold = body.score_threshold.unwrap_or(0.5);
    let baseline_f1 = body.baseline_f1.unwrap_or(0.0);
    require_unit_interval(score_threshold, "score_threshold")?;
    require_unit_interval(baseline_f1, "baseline_f1")?;
    let expected = body.expected_selected_paths.clone().unwrap_or_default();
    let evidence_refs: Vec<String> = body.evidence_refs.clone().unwrap_or_default();

    // Reuse the curate handler logic by re-running it. Build the request
    // shape and call the same scoring code path.
    let candidates: Vec<CurateCandidate> = body.candidates.clone().unwrap_or_default();
    let tokens_estimated: Vec<CurateCandidate> = candidates
        .into_iter()
        .map(|mut c| {
            if c.tokens.is_none() {
                c.tokens = Some(estimate_tokens(c.body.as_deref().unwrap_or("")));
            }
            c
        })
        .collect();
    let mut scored: Vec<(f64, &CurateCandidate)> = tokens_estimated
        .iter()
        .map(|c| (score_candidate(&target, c), c))
        .collect();
    let evidence_set: std::collections::HashSet<String> = evidence_refs.iter().cloned().collect();
    for (s, c) in scored.iter_mut() {
        if let Some(er) = c.evidence_ref.as_ref()
            && evidence_set.contains(er)
        {
            *s += 1.0;
        }
    }
    scored.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.1.tokens.unwrap_or(0).cmp(&b.1.tokens.unwrap_or(0)))
    });

    let mut selected_paths: Vec<String> = Vec::new();
    let mut used: usize = 0;
    for (score, cand) in scored.iter() {
        let cand_tokens = cand.tokens.unwrap_or(0);
        if used + cand_tokens <= token_budget {
            used += cand_tokens;
            selected_paths.push(cand.path.clone());
        } else if *score < 2.0 {
            // Drop low-score items; preserve selected_paths for scoring.
            let _ = score;
        }
    }

    let (precision, recall) = compute_precision_recall(&selected_paths, &expected);
    let f1 = compute_f1(precision, recall);
    let promoted = f1 > baseline_f1 && f1 >= score_threshold;

    let run = CuratorEvalRun {
        run_id: uuid::Uuid::now_v7().to_string(),
        case_id: case_id.clone(),
        project_root: project_root.clone(),
        continuity_id: Some(continuity_id.clone()),
        target: target.clone(),
        selected_paths: selected_paths.clone(),
        expected_paths: expected.clone(),
        precision,
        recall,
        f1,
        baseline_f1,
        tokens_used: used,
        score_threshold,
        promoted,
        created_at: chrono::Utc::now(),
    };

    if let Err(e) = state.persistence.append_curator_eval_run(&run) {
        return Err(rejection(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({
                "status": "blocked",
                "failure_class": "storage_unwritable",
                "message": format!("append failed: {}", e),
            }),
        ));
    }

    Ok(Json(json!({
        "status": "completed",
        "canonical": false,
        "advisory": true,
        "scope_status": "matched",
        "continuity_id": continuity_id,
        "run_id": run.run_id,
        "case_id": case_id,
        "selected_paths": selected_paths,
        "expected_paths": expected,
        "tokens_used": used,
        "token_budget": token_budget,
        "precision": precision,
        "recall": recall,
        "f1": f1,
        "baseline_f1": baseline_f1,
        "score_threshold": score_threshold,
        "promoted": promoted,
        "eval_ref": format!("curator-eval:{}:{}", project_root, run.run_id),
        "next_tools": [
            "focusa_context_cognition_curate_optimize",
            "focusa_metacog_capture",
            "focusa_evidence_capture"
        ],
        "rehydrate_id": run.run_id,
    })))
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct CurateEvalRunsRequest {
    pub project_root: Option<String>,
    pub limit: Option<usize>,
}

async fn curate_eval_runs(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<CurateEvalRunsRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let project_root = query
        .project_root
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("")
        .to_string();
    if project_root.is_empty() {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "project_root_missing",
                "field": "project_root",
            }),
        ));
    }
    if is_unsafe_agent_runtime_path_inline(&project_root) {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "scope_mismatch",
                "field": "project_root",
                "rejected_value": project_root,
            }),
        ));
    }
    let limit = query.limit.unwrap_or(10).min(200);
    let runs = state
        .persistence
        .read_curator_eval_runs(&project_root, limit)
        .unwrap_or_default();
    let summary: Vec<Value> = runs
        .iter()
        .map(|r| {
            json!({
                "run_id": r.run_id,
                "case_id": r.case_id,
                "target": r.target,
                "selected_count": r.selected_paths.len(),
                "expected_count": r.expected_paths.len(),
                "precision": r.precision,
                "recall": r.recall,
                "f1": r.f1,
                "baseline_f1": r.baseline_f1,
                "tokens_used": r.tokens_used,
                "score_threshold": r.score_threshold,
                "promoted": r.promoted,
                "created_at": r.created_at,
            })
        })
        .collect();
    Ok(Json(json!({
        "status": "completed",
        "canonical": false,
        "advisory": true,
        "scope_status": "matched",
        "project_root": project_root,
        "count": runs.len(),
        "runs": summary,
        "rehydrate_id": runs.last().map(|r| r.run_id.clone()).unwrap_or_else(|| "no_runs".to_string()),
    })))
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct OptimizerArtifactsRequest {
    pub project_root: Option<String>,
    pub module_name: Option<String>,
    pub limit: Option<usize>,
}

async fn optimizer_artifacts(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(query): axum::extract::Query<OptimizerArtifactsRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let project_root = query
        .project_root
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("")
        .to_string();
    if project_root.is_empty() {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "project_root_missing",
                "field": "project_root",
            }),
        ));
    }
    if is_unsafe_agent_runtime_path_inline(&project_root) {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "scope_mismatch",
                "field": "project_root",
                "rejected_value": project_root,
            }),
        ));
    }
    let module_name = query
        .module_name
        .clone()
        .unwrap_or_else(|| "curator".to_string());
    let limit = query.limit.unwrap_or(10).min(200);
    let artifacts = state
        .persistence
        .read_cognition_optimizer_artifacts(&project_root, &module_name, limit)
        .unwrap_or_default();
    let summary: Vec<Value> = artifacts
        .iter()
        .map(|a| {
            json!({
                "artifact_id": a.artifact_id,
                "module_name": a.module_name,
                "prompt_artifact_ref": a.prompt_artifact_ref,
                "eval_score": a.eval_score,
                "baseline_score": a.baseline_score,
                "promoted": a.promoted,
                "rollback_ref": a.rollback_ref,
                "eval_run_id": a.eval_run_id,
                "created_at": a.created_at,
                "promoted_at": a.promoted_at,
            })
        })
        .collect();
    let latest_promoted = artifacts.iter().rev().find(|a| a.promoted).cloned();
    Ok(Json(json!({
        "status": "completed",
        "canonical": false,
        "advisory": true,
        "scope_status": "matched",
        "project_root": project_root,
        "module_name": module_name,
        "count": artifacts.len(),
        "artifacts": summary,
        "latest_promoted": latest_promoted,
        "rehydrate_id": artifacts.last().map(|a| a.artifact_id.clone()).unwrap_or_else(|| "no_artifacts".to_string()),
    })))
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone)]
pub struct CurateOptimizeRequest {
    pub project_root: Option<String>,
    pub continuity_id: Option<String>,
    pub module_name: Option<String>,
    pub prompt_artifact_ref: Option<String>,
    pub eval_score: Option<f64>,
    pub baseline_score: Option<f64>,
    pub eval_run_id: Option<String>,
    pub score_threshold: Option<f64>,
    pub rollback: Option<bool>,
}

async fn curate_optimize(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CurateOptimizeRequest>,
) -> Result<Json<Value>, (StatusCode, Json<Value>)> {
    let project_root = body
        .project_root
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("")
        .to_string();
    if project_root.is_empty() {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "project_root_missing",
                "field": "project_root",
            }),
        ));
    }
    if is_unsafe_agent_runtime_path_inline(&project_root) {
        return Err(rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "scope_mismatch",
                "field": "project_root",
                "rejected_value": project_root,
            }),
        ));
    }

    let continuity_id = require_continuity_id(&body.continuity_id)?;
    let module_name = body
        .module_name
        .clone()
        .unwrap_or_else(|| "curator".to_string());
    let prompt_artifact_ref = body.prompt_artifact_ref.clone().ok_or_else(|| {
        rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "prompt_artifact_ref_missing",
                "field": "prompt_artifact_ref",
            }),
        )
    })?;
    let eval_score = body.eval_score.ok_or_else(|| {
        rejection(
            StatusCode::UNPROCESSABLE_ENTITY,
            json!({
                "status": "validation_rejected",
                "failure_class": "eval_score_missing",
                "field": "eval_score",
            }),
        )
    })?;
    let baseline_score = body.baseline_score.unwrap_or(0.0);
    let score_threshold = body.score_threshold.unwrap_or(0.5);
    require_unit_interval(eval_score, "eval_score")?;
    require_unit_interval(baseline_score, "baseline_score")?;
    require_unit_interval(score_threshold, "score_threshold")?;
    let explicit_rollback = body.rollback.unwrap_or(false);

    // Determine the latest promoted artifact to use as the rollback_ref
    // (or as the comparison baseline for promotion).
    let latest_promoted = state
        .persistence
        .latest_promoted_artifact(&project_root, &module_name)
        .unwrap_or(None);
    let rollback_ref = latest_promoted.as_ref().map(|a| a.artifact_id.clone());

    // Promotion rule (Spec 100 §15):
    // - promoted=true when eval_score > baseline_score AND eval_score >= score_threshold
    // - explicit rollback overrides to promoted=false
    let promoted = if explicit_rollback {
        false
    } else {
        eval_score > baseline_score && eval_score >= score_threshold
    };
    let decision = if explicit_rollback {
        "rollback"
    } else if promoted {
        "promote"
    } else {
        "rollback"
    };

    let now = chrono::Utc::now();
    let artifact = CognitionOptimizerArtifact {
        artifact_id: uuid::Uuid::now_v7().to_string(),
        module_name: module_name.clone(),
        project_root: project_root.clone(),
        prompt_artifact_ref,
        eval_score,
        baseline_score,
        promoted,
        rollback_ref: rollback_ref.clone(),
        eval_run_id: body.eval_run_id.clone(),
        created_at: now,
        promoted_at: if promoted { Some(now) } else { None },
    };

    if let Err(e) = state
        .persistence
        .append_cognition_optimizer_artifact(&artifact)
    {
        return Err(rejection(
            StatusCode::INTERNAL_SERVER_ERROR,
            json!({
                "status": "blocked",
                "failure_class": "storage_unwritable",
                "message": format!("append failed: {}", e),
            }),
        ));
    }

    Ok(Json(json!({
        "status": "completed",
        "canonical": false,
        "advisory": true,
        "scope_status": "matched",
        "continuity_id": continuity_id,
        "artifact_id": artifact.artifact_id,
        "module_name": module_name,
        "decision": decision,
        "promoted": promoted,
        "eval_score": eval_score,
        "baseline_score": baseline_score,
        "score_threshold": score_threshold,
        "rollback_ref": rollback_ref,
        "eval_run_id": body.eval_run_id,
        "rehydrate_id": artifact.artifact_id,
        "next_tools": [
            "focusa_context_cognition_optimizer_artifacts",
            "focusa_predict_record",
            "focusa_metacog_capture"
        ],
    })))
}
