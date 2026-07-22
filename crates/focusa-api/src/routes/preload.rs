//! Spec 111 — Agent Context Bootstrap and Delivery.
//!
//! Slice 1: Spec + static contracts for AgentBootstrapPacket, AgentBootstrapProfile,
//! AgentBootstrapReceipt, FOCUSA_PRELOAD_FAIL, and bootstrap_delivery Focusa Receipt.
//! Slice 2 stub: read-mostly routes (/v1/preload/profiles|build|render|verify|doctor)
//! that emit static envelopes. Slice 3 will dispatch to renderers; Slice 4 will add
//! the safe-write route; Slice 5 will integrate with Spec 119 receipts.

use crate::routes::context_cognition::{CurateCandidate, curate_preload_candidates};
use crate::routes::project::project_identity_payload_for_scope;
use crate::routes::workpoint::active_workpoint_for_context;
use crate::server::AppState;
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    routing::{get, post},
};
use focusa_core::awareness::{
    self, AwarenessInput, PreloadAwarenessInput, SURFACE_AGENT_PRELOAD, SURFACE_PRELOAD_FAIL,
};
use serde_json::{Value, json};
use std::sync::Arc;

pub const PRELOAD_SCHEMA: &str = "focusa.preload.v1";
pub const BOOTSTRAP_RECEIPT_KIND: &str = "bootstrap_delivery";
pub const FAIL_CODE_PRELOAD: &str = "FOCUSA_PRELOAD_FAIL";

pub const PROFILE_RULES_ONLY: &str = "rules_only";
pub const PROFILE_RULES_AND_CONTEXT: &str = "rules_and_context";
pub const PROFILE_BUDGET_LIGHT: &str = "budget_light";
pub const PROFILE_BUDGET_DEEP: &str = "budget_deep";

pub const PROFILE_IDS: &[&str] = &[
    PROFILE_RULES_ONLY,
    PROFILE_RULES_AND_CONTEXT,
    PROFILE_BUDGET_LIGHT,
    PROFILE_BUDGET_DEEP,
];

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/preload/profiles", get(list_profiles))
        .route("/v1/preload/build", get(build).post(build_post))
        .route("/v1/preload/render", get(render).post(render_post))
        .route("/v1/preload/verify", get(verify).post(verify_post))
        .route("/v1/preload/doctor", get(doctor).post(doctor_post))
        .route(
            "/v1/preload/receipt-preview",
            get(receipt_preview).post(receipt_preview_post),
        )
        .route("/v1/preload/receipt-commit", post(receipt_commit))
        .route("/v1/preload/write", post(write_packet))
}

async fn list_profiles() -> Json<Value> {
    let profiles: Vec<Value> = AGENT_BOOTSTRAP_PROFILES
        .iter()
        .map(|p| {
            json!({
                "id": p.id,
                "label": p.label,
                "description": p.description,
                "includes_dynamic_context": p.includes_dynamic_context,
                "includes_acceptance_prompt": p.includes_acceptance_prompt,
                "max_dynamic_items": p.max_dynamic_items,
            })
        })
        .collect();
    Json(json!({
        "schema": PRELOAD_SCHEMA,
        "profiles": profiles,
        "default_profile": PROFILE_RULES_AND_CONTEXT,
        "read_only": true,
        "human_readable": format!(
            "{} preload profiles available: {}. Default: {}.",
            PROFILE_IDS.len(),
            PROFILE_IDS.join(", "),
            PROFILE_RULES_AND_CONTEXT
        ),
    }))
}

#[derive(serde::Deserialize, Default)]
struct ProfileQuery {
    #[serde(default)]
    profile: Option<String>,
}

#[derive(serde::Deserialize, Default)]
struct PreloadBuildRequest {
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    project_root: Option<String>,
    #[serde(default)]
    working_subpath_id: Option<String>,
    #[serde(default)]
    continuity_id: Option<String>,
    #[serde(default)]
    current_ask: Option<String>,
    #[serde(default)]
    include_context_cognition: Option<bool>,
    #[serde(default)]
    include_awareness: Option<bool>,
}

fn packet_response(step: &str, profile: Option<String>) -> Json<Value> {
    let profile = profile.unwrap_or_else(|| PROFILE_RULES_AND_CONTEXT.to_string());
    match build_packet_for_profile(&profile) {
        Ok(packet) => Json(json!({
            "schema": PRELOAD_SCHEMA,
            "step": step,
            "read_only": true,
            "status": "completed",
            "packet": packet,
            "checks": ["profile", "integrity", "scope"],
            "human_readable": format!(
                "Preload {step} completed with profile {profile}. Next: verify before delivery."
            )
        })),
        Err(error) => Json(json!({
            "schema": PRELOAD_SCHEMA,
            "step": step,
            "status": "failed",
            "error": {"code": FAIL_CODE_PRELOAD, "message": error},
            "human_readable": format!(
                "Preload {step} failed because profile {profile:?} is invalid. Next: call focusa_preload_profiles and retry with a listed profile."
            )
        })),
    }
}

async fn build(Query(query): Query<ProfileQuery>) -> Json<Value> {
    packet_response("build", query.profile)
}

fn target_dynamic_max_lines(target: &str) -> usize {
    match target {
        "cursor" => 160,
        "claude" => 200,
        "codex" => 180,
        "pi" | "generic" | "opencode" => 120,
        _ => 120,
    }
}

fn preload_awareness(
    surface: &str,
    status: &str,
    scope_missing: bool,
    workpoint_missing: bool,
    evidence_gap: bool,
    recovery_tool: Option<&str>,
) -> Value {
    let input = AwarenessInput {
        surface: surface.to_string(),
        preload: Some(PreloadAwarenessInput {
            status: status.to_string(),
            verification_status: "pending".to_string(),
            scope_missing,
            workpoint_missing,
            evidence_gap,
            receipt_status: None,
            recovery_tool: recovery_tool.map(str::to_string),
        }),
        ..Default::default()
    };
    json!(awareness::render_packet(&input))
}

async fn build_post(
    State(state): State<Arc<AppState>>,
    Json(query): Json<PreloadBuildRequest>,
) -> (StatusCode, Json<Value>) {
    let profile_id = query
        .profile
        .as_deref()
        .unwrap_or(PROFILE_RULES_AND_CONTEXT);
    let Some(profile) = profile_by_id(profile_id) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(
                json!({"schema":PRELOAD_SCHEMA,"step":"build","status":"failed","error":{"code":FAIL_CODE_PRELOAD,"message":format!("unknown profile {profile_id:?}")}}),
            ),
        );
    };
    if query.include_context_cognition == Some(false) {
        return (StatusCode::OK, packet_response("build", query.profile));
    }
    let project_root = query.project_root.as_deref().map(str::trim).unwrap_or("");
    let continuity_id = query.continuity_id.as_deref().map(str::trim).unwrap_or("");
    if project_root.is_empty() || continuity_id.is_empty() {
        let failure = if project_root.is_empty() {
            "project_root_missing"
        } else {
            "continuity_id_missing"
        };
        let awareness = if query.include_awareness == Some(false) {
            Value::Null
        } else {
            preload_awareness(
                SURFACE_PRELOAD_FAIL,
                "blocked",
                true,
                false,
                true,
                Some("focusa_project_identity"),
            )
        };
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(
                json!({"schema":PRELOAD_SCHEMA,"step":"build","status":"blocked","canonical":false,"advisory":true,"failure_class":failure,"awareness":awareness,"error":{"code":FAIL_CODE_PRELOAD,"message":failure}}),
            ),
        );
    }
    let identity = project_identity_payload_for_scope(Some(project_root), Some(project_root), None);
    let identity_status = identity
        .pointer("/project_identity/status")
        .and_then(Value::as_str)
        .unwrap_or("unknown");
    if identity_status != "verified" {
        let awareness = if query.include_awareness == Some(false) {
            Value::Null
        } else {
            preload_awareness(
                SURFACE_PRELOAD_FAIL,
                "blocked",
                true,
                false,
                true,
                Some("focusa_project_identity"),
            )
        };
        return (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(
                json!({"schema":PRELOAD_SCHEMA,"step":"build","status":"blocked","canonical":false,"advisory":true,"failure_class":"project_identity_unverified","project_identity":identity,"awareness":awareness,"error":{"code":FAIL_CODE_PRELOAD,"message":"project identity is not verified"}}),
            ),
        );
    }

    let focus = state.focusa.read().await;
    let workpoint = active_workpoint_for_context(
        &focus,
        Some(project_root),
        Some(continuity_id),
        query.working_subpath_id.as_deref(),
    );
    let mut candidates = Vec::new();
    let mut evidence_refs = Vec::new();
    let mut selection_target = query.current_ask.clone().unwrap_or_default();
    if let Some(workpoint) = workpoint {
        if let Some(next) = workpoint
            .next_slice
            .as_deref()
            .filter(|value| !value.trim().is_empty())
        {
            selection_target = next.to_string();
            candidates.push(CurateCandidate {
                kind: "snippet".into(),
                path: "workpoint:next_action".into(),
                body: Some(next.into()),
                evidence_ref: None,
                tokens: None,
            });
        }
        if let Some(mission) = workpoint.mission.as_deref() {
            candidates.push(CurateCandidate {
                kind: "snippet".into(),
                path: "workpoint:mission".into(),
                body: Some(mission.into()),
                evidence_ref: None,
                tokens: None,
            });
        }
        for object in &workpoint.active_object_refs {
            candidates.push(CurateCandidate {
                kind: "codemap".into(),
                path: object.clone(),
                body: Some(format!("active object: {object}")),
                evidence_ref: None,
                tokens: None,
            });
        }
        for blocker in &workpoint.blockers {
            candidates.push(CurateCandidate {
                kind: "snippet".into(),
                path: format!(
                    "workpoint:blocker:{}",
                    blocker.target_ref.as_deref().unwrap_or("unknown")
                ),
                body: Some(blocker.reason.clone()),
                evidence_ref: None,
                tokens: None,
            });
        }
        for verification in &workpoint.verification_records {
            if let Some(evidence_ref) = verification.evidence_ref.clone() {
                evidence_refs.push(evidence_ref.clone());
                candidates.push(CurateCandidate {
                    kind: "evidence".into(),
                    path: verification.target_ref.clone(),
                    body: Some(verification.result.clone()),
                    evidence_ref: Some(evidence_ref),
                    tokens: None,
                });
            }
        }
    }
    let workpoint_found = workpoint.is_some();
    drop(focus);

    let target = query.target.as_deref().unwrap_or("generic");
    let dynamic_max_lines = target_dynamic_max_lines(target);
    let token_budget = dynamic_max_lines.saturating_mul(8);
    let mut selection =
        curate_preload_candidates(&selection_target, token_budget, candidates, &evidence_refs);
    let mut packet = build_packet_for_profile(profile_id).expect("profile checked above");
    let selected = selection["selected_context"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let item_limit = profile.max_dynamic_items.min(dynamic_max_lines);
    let included: Vec<Value> = selected.iter().take(item_limit).cloned().collect();
    let dynamic_lines: Vec<String> = included
        .iter()
        .filter_map(|item| item["body"].as_str())
        .map(|body| body.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect();
    let mut excluded = selection["excluded_context"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    for item in selected.iter().skip(item_limit) {
        excluded
            .push(json!({"kind":item["kind"],"path":item["path"],"reason":"profile_item_budget"}));
    }
    selection["selected_context"] = json!(included);
    selection["excluded_context"] = json!(excluded);
    packet["dynamic_context_lines"] = json!(dynamic_lines);
    packet["selected_context"] = json!({"include":selection["selected_context"],"exclude":selection["excluded_context"],"over_budget":selection["over_budget"]});
    packet["context_selection"] = json!("context_cognition");
    packet["canonical"] = json!(false);
    packet["advisory"] = json!(true);

    let status = if workpoint_found {
        "completed"
    } else {
        "degraded"
    };
    let awareness = if query.include_awareness == Some(false) {
        Value::Null
    } else {
        preload_awareness(
            SURFACE_AGENT_PRELOAD,
            status,
            false,
            !workpoint_found,
            evidence_refs.is_empty(),
            (!workpoint_found).then_some("focusa_workpoint_resume"),
        )
    };
    (
        StatusCode::OK,
        Json(
            json!({"schema":PRELOAD_SCHEMA,"step":"build","status":status,"canonical":false,"advisory":true,"project_identity":identity,"packet":packet,"context_cognition":selection,"awareness":awareness,"proof_gaps":if workpoint_found{Vec::<&str>::new()}else{vec!["workpoint_missing"]},"next_tools":["focusa_preload_render","focusa_preload_write","focusa_preload_verify","focusa_preload_receipt_preview"]}),
        ),
    )
}
async fn render(Query(query): Query<ProfileQuery>) -> Json<Value> {
    packet_response("render", query.profile)
}
async fn render_post(Json(query): Json<ProfileQuery>) -> Json<Value> {
    packet_response("render", query.profile)
}
async fn verify(Query(query): Query<ProfileQuery>) -> Json<Value> {
    packet_response("verify", query.profile)
}
async fn verify_post(Json(query): Json<ProfileQuery>) -> Json<Value> {
    packet_response("verify", query.profile)
}
async fn doctor(Query(query): Query<ProfileQuery>) -> Json<Value> {
    packet_response("doctor", query.profile)
}
async fn doctor_post(Json(query): Json<ProfileQuery>) -> Json<Value> {
    packet_response("doctor", query.profile)
}

pub fn build_packet_for_profile(profile_id: &str) -> Result<Value, String> {
    let packet = build_packet(profile_id)?;
    Ok(json!({
        "schema": packet.schema,
        "profile_id": packet.profile_id,
        "render_mode": format!("{:?}", packet.render_mode),
        "static_rule_lines": packet.static_rule_lines,
        "dynamic_context_lines": packet.dynamic_context_lines,
        "acceptance_prompt": packet.acceptance_prompt,
        "bounded_dynamic_items": packet.bounded_dynamic_items,
        "rendered": render_packet(&packet),
    }))
}

fn receipt_preview_response(profile: Option<String>) -> Json<Value> {
    let profile = profile.unwrap_or_else(default_profile);
    match receipt_preview_for(&profile) {
        Ok(receipt) => Json(
            json!({"schema":PRELOAD_SCHEMA,"step":"receipt_preview","read_only":true,"status":"completed","receipt":receipt}),
        ),
        Err(error) => Json(
            json!({"schema":PRELOAD_SCHEMA,"step":"receipt_preview","status":"failed","error":{"code":FAIL_CODE_PRELOAD,"message":error}}),
        ),
    }
}

async fn receipt_preview(Query(query): Query<ProfileQuery>) -> Json<Value> {
    receipt_preview_response(query.profile)
}

async fn receipt_preview_post(Json(query): Json<ProfileQuery>) -> Json<Value> {
    receipt_preview_response(query.profile)
}

#[derive(serde::Deserialize)]
struct ReceiptCommitRequest {
    #[serde(default = "default_profile")]
    profile: String,
    idempotency_key: String,
}

fn default_profile() -> String {
    PROFILE_RULES_AND_CONTEXT.to_string()
}

fn receipt_ledger_path(key: &str) -> std::path::PathBuf {
    let safe = key
        .chars()
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect::<String>();
    std::env::var_os("HOME")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir)
        .join(".local/state/focusa/preload-receipts")
        .join(format!("{safe}.json"))
}

pub fn commit_receipt_for(profile: &str, idempotency_key: &str) -> Result<(Value, bool), String> {
    if idempotency_key.trim().is_empty() {
        return Err("idempotency_key is required".to_string());
    }
    let path = receipt_ledger_path(idempotency_key);
    if let Ok(body) = std::fs::read(&path)
        && let Ok(receipt) = serde_json::from_slice::<Value>(&body)
    {
        return Ok((receipt, true));
    }
    let receipt = receipt_preview_for(profile)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let committed = json!({"receipt_kind":BOOTSTRAP_RECEIPT_KIND,"idempotency_key":idempotency_key,"receipt":receipt});
    std::fs::write(
        &path,
        serde_json::to_vec_pretty(&committed).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;
    Ok((committed, false))
}

async fn receipt_commit(Json(req): Json<ReceiptCommitRequest>) -> (StatusCode, Json<Value>) {
    if req.idempotency_key.trim().is_empty() || profile_by_id(&req.profile).is_none() {
        let message = if req.idempotency_key.trim().is_empty() {
            "idempotency_key is required".to_string()
        } else {
            format!("unknown profile {:?}", req.profile)
        };
        return (
            StatusCode::BAD_REQUEST,
            Json(
                json!({"schema":PRELOAD_SCHEMA,"step":"receipt_commit","status":"failed","error":{"code":FAIL_CODE_PRELOAD,"message":message}}),
            ),
        );
    }
    match commit_receipt_for(&req.profile, &req.idempotency_key) {
        Ok((receipt, replay)) => (
            StatusCode::OK,
            Json(
                json!({"schema":PRELOAD_SCHEMA,"step":"receipt_commit","status":"completed","idempotent_replay":replay,"receipt":receipt}),
            ),
        ),
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(
                json!({"schema":PRELOAD_SCHEMA,"step":"receipt_commit","status":"failed","error":{"code":FAIL_CODE_PRELOAD,"message":error}}),
            ),
        ),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderMode {
    StaticRule,
    DynamicContext,
    AcceptancePrompt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentBootstrapProfile {
    pub id: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    pub includes_dynamic_context: bool,
    pub includes_acceptance_prompt: bool,
    pub max_dynamic_items: usize,
}

pub const AGENT_BOOTSTRAP_PROFILES: &[AgentBootstrapProfile] = &[
    AgentBootstrapProfile {
        id: PROFILE_RULES_ONLY,
        label: "Rules only",
        description: "Static rule render only, no dynamic context, no acceptance prompt.",
        includes_dynamic_context: false,
        includes_acceptance_prompt: false,
        max_dynamic_items: 0,
    },
    AgentBootstrapProfile {
        id: PROFILE_RULES_AND_CONTEXT,
        label: "Rules and context",
        description: "Static rule render + bounded dynamic context render.",
        includes_dynamic_context: true,
        includes_acceptance_prompt: false,
        max_dynamic_items: 8,
    },
    AgentBootstrapProfile {
        id: PROFILE_BUDGET_LIGHT,
        label: "Budget light",
        description: "Static rules + minimal dynamic context + acceptance prompt.",
        includes_dynamic_context: true,
        includes_acceptance_prompt: true,
        max_dynamic_items: 4,
    },
    AgentBootstrapProfile {
        id: PROFILE_BUDGET_DEEP,
        label: "Budget deep",
        description: "Static rules + larger dynamic context + acceptance prompt.",
        includes_dynamic_context: true,
        includes_acceptance_prompt: true,
        max_dynamic_items: 16,
    },
];

pub fn profile_by_id(id: &str) -> Option<&'static AgentBootstrapProfile> {
    AGENT_BOOTSTRAP_PROFILES.iter().find(|p| p.id == id)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentBootstrapPacket {
    pub schema: &'static str,
    pub profile_id: &'static str,
    pub render_mode: RenderMode,
    pub static_rule_lines: Vec<String>,
    pub dynamic_context_lines: Vec<String>,
    pub acceptance_prompt: String,
    pub bounded_dynamic_items: usize,
}

pub fn build_packet(profile_id: &str) -> Result<AgentBootstrapPacket, String> {
    let profile = profile_by_id(profile_id)
        .ok_or_else(|| format!("{FAIL_CODE_PRELOAD}: unknown profile {profile_id:?}"))?;
    let static_rule_lines = vec![
        "Focusa does not bypass install/checksum/license/update rules.".to_string(),
        "Canonical Workpoint authority requires operator approval.".to_string(),
        "Proof is required before declaring completion.".to_string(),
        "Scope is verified before changing files.".to_string(),
    ];
    let dynamic_context_lines: Vec<String> = if profile.includes_dynamic_context {
        (0..profile.max_dynamic_items)
            .map(|i| format!("bounded_context_item_{i}"))
            .collect()
    } else {
        Vec::new()
    };
    let acceptance_prompt = if profile.includes_acceptance_prompt {
        "Acknowledge Focusa rules, then proceed with the next safe action.".to_string()
    } else {
        String::new()
    };
    Ok(AgentBootstrapPacket {
        schema: PRELOAD_SCHEMA,
        profile_id: profile.id,
        render_mode: RenderMode::StaticRule,
        static_rule_lines,
        dynamic_context_lines,
        acceptance_prompt,
        bounded_dynamic_items: profile.max_dynamic_items,
    })
}

pub fn render_packet(packet: &AgentBootstrapPacket) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "# Focusa Agent Bootstrap ({})\n",
        packet.profile_id
    ));
    out.push_str("\n## Rules\n");
    for line in &packet.static_rule_lines {
        out.push_str(&format!("- {line}\n"));
    }
    if !packet.dynamic_context_lines.is_empty() {
        out.push_str("\n## Context (bounded)\n");
        for line in &packet.dynamic_context_lines {
            out.push_str(&format!("- {line}\n"));
        }
    }
    if !packet.acceptance_prompt.is_empty() {
        out.push_str("\n## Acceptance\n");
        out.push_str(&packet.acceptance_prompt);
        out.push('\n');
    }
    out
}

#[derive(serde::Deserialize)]
struct WriteRequest {
    profile_id: String,
    target_path: String,
    idempotency_key: String,
    #[serde(default)]
    overwrite: bool,
}

#[derive(serde::Serialize)]
struct WriteFailure {
    error: &'static str,
    code: String,
    reason: String,
}

fn write_failure(err: &'static str, reason: impl Into<String>) -> Json<WriteFailure> {
    Json(WriteFailure {
        error: err,
        code: FAIL_CODE_PRELOAD.to_string(),
        reason: reason.into(),
    })
}

fn is_safe_target(target: &str) -> bool {
    let p = std::path::Path::new(target);
    if target.contains('\0') {
        return false;
    }
    if p.is_absolute() {
        target.starts_with("/tmp/focusa-preload/")
            || target.starts_with("/var/cache/focusa/preload/")
    } else {
        false
    }
}

async fn write_packet(Json(req): Json<WriteRequest>) -> (StatusCode, Json<Value>) {
    if req.idempotency_key.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "schema": PRELOAD_SCHEMA,
                "error": FAIL_CODE_PRELOAD,
                "reason": "missing_idempotency_key",
            })),
        );
    }
    if !is_safe_target(&req.target_path) {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({
                "schema": PRELOAD_SCHEMA,
                "error": FAIL_CODE_PRELOAD,
                "reason": "unsafe_target_path",
                "allowed_prefixes": ["/tmp/focusa-preload/", "/var/cache/focusa/preload/"],
            })),
        );
    }
    let packet = match build_packet(&req.profile_id) {
        Ok(p) => p,
        Err(reason) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "schema": PRELOAD_SCHEMA,
                    "error": FAIL_CODE_PRELOAD,
                    "reason": reason,
                })),
            );
        }
    };
    let path = std::path::Path::new(&req.target_path);
    if path.exists() && !req.overwrite {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "schema": PRELOAD_SCHEMA,
                "error": FAIL_CODE_PRELOAD,
                "reason": "target_exists_set_overwrite_true_to_replace",
            })),
        );
    }
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let body = render_packet(&packet);
    let write_result = std::fs::write(path, body.as_bytes());
    match write_result {
        Ok(()) => (
            StatusCode::OK,
            Json(json!({
                "schema": PRELOAD_SCHEMA,
                "step": "write",
                "profile_id": req.profile_id,
                "target_path": req.target_path,
                "idempotency_key": req.idempotency_key,
                "ok": true,
            })),
        ),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "schema": PRELOAD_SCHEMA,
                "error": FAIL_CODE_PRELOAD,
                "reason": "io_error",
                "detail": err.to_string(),
            })),
        ),
    }
}

pub fn receipt_preview_for(profile_id: &str) -> Result<Value, String> {
    let packet = build_packet(profile_id)?;
    // Spec 125 §14: receipt includes trajectory HLT posture.
    let hlt_status = "unknown_from_receipt";
    let hlt_required = true;
    let generic_bootstrap = false;
    let fallback_level = "none";
    let completion_blocked = false;
    let completion_degraded = false;
    Ok(json!({
        "schema": PRELOAD_SCHEMA,
        "receipt_kind": BOOTSTRAP_RECEIPT_KIND,
        "preview": true,
        "ok": true,
        "profile_id": packet.profile_id,
        "rendered": render_packet(&packet),
        // Spec 125 §14: HLT posture in receipt.
        "trajectory_hlt_posture": {
            "hlt_status": hlt_status,
            "hlt_required": hlt_required,
            "generic_bootstrap": generic_bootstrap,
            "fallback_level": fallback_level,
            "completion_blocked": completion_blocked,
            "completion_degraded": completion_degraded,
            "warning": if hlt_status != "verified" { Some("HLT status not verified; use focusa_trajectory_view to confirm") } else { None },
        },
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_constant_matches_spec() {
        assert_eq!(PRELOAD_SCHEMA, "focusa.preload.v1");
    }

    #[test]
    fn profile_ids_are_stable() {
        assert!(PROFILE_IDS.contains(&PROFILE_RULES_AND_CONTEXT));
        assert!(PROFILE_IDS.contains(&PROFILE_BUDGET_LIGHT));
        assert!(PROFILE_IDS.contains(&PROFILE_BUDGET_DEEP));
    }

    #[tokio::test]
    async fn profile_discovery_and_failures_are_human_readable() {
        let Json(profiles) = list_profiles().await;
        let readable = profiles
            .get("human_readable")
            .and_then(Value::as_str)
            .expect("profile list human_readable");
        for profile in PROFILE_IDS {
            assert!(readable.contains(profile));
        }

        let Json(failed) = packet_response("doctor", Some("pi".to_string()));
        assert_eq!(failed.get("status").and_then(Value::as_str), Some("failed"));
        assert!(
            failed
                .get("human_readable")
                .and_then(Value::as_str)
                .is_some_and(|text| text.contains("focusa_preload_profiles"))
        );
    }

    #[test]
    fn slice2_packet_build_and_render() {
        let p = build_packet(PROFILE_RULES_ONLY).expect("profile");
        assert_eq!(p.profile_id, PROFILE_RULES_ONLY);
        assert!(p.static_rule_lines.len() >= 4);
        assert!(p.dynamic_context_lines.is_empty());
        assert!(p.acceptance_prompt.is_empty());
        let s = render_packet(&p);
        assert!(s.contains("Focusa Agent Bootstrap"));
        assert!(s.contains("## Rules"));
    }

    #[test]
    fn slice2_budget_deep_includes_context_and_acceptance() {
        let p = build_packet(PROFILE_BUDGET_DEEP).expect("profile");
        let profile = profile_by_id(PROFILE_BUDGET_DEEP).expect("profile");
        assert!(profile.includes_dynamic_context);
        assert_eq!(p.bounded_dynamic_items, 16);
        assert!(!p.acceptance_prompt.is_empty());
    }

    #[test]
    fn slice4_unsafe_target_path_is_rejected() {
        assert!(!is_safe_target("/etc/passwd"));
        assert!(!is_safe_target("./local.txt"));
        assert!(is_safe_target("/tmp/focusa-preload/packet.md"));
        assert!(is_safe_target("/var/cache/focusa/preload/x.md"));
    }

    #[test]
    fn slice5_receipt_preview_returns_rendered_packet() {
        let v = receipt_preview_for(PROFILE_RULES_AND_CONTEXT).expect("preview");
        assert_eq!(v["receipt_kind"], BOOTSTRAP_RECEIPT_KIND);
        assert!(
            v["rendered"]
                .as_str()
                .unwrap_or_default()
                .contains("Focusa Agent Bootstrap")
        );
    }

    #[test]
    fn slice3_profile_list_contains_required_ids() {
        for id in [
            PROFILE_RULES_ONLY,
            PROFILE_RULES_AND_CONTEXT,
            PROFILE_BUDGET_LIGHT,
            PROFILE_BUDGET_DEEP,
        ] {
            assert!(profile_by_id(id).is_some(), "missing {id}");
        }
    }

    #[test]
    fn slice3_build_packet_for_profile_json_includes_rendered() {
        let v = build_packet_for_profile(PROFILE_RULES_AND_CONTEXT).expect("packet");
        let rendered = v["rendered"].as_str().unwrap_or_default();
        assert!(rendered.contains("Focusa Agent Bootstrap"));
    }

    #[test]
    fn slice2_unknown_profile_fails_with_focusa_preload_fail() {
        let err = build_packet("nope").err().unwrap_or_default();
        assert!(err.contains(FAIL_CODE_PRELOAD));
    }

    #[test]
    fn receipt_kind_and_fail_code_match_spec() {
        assert_eq!(BOOTSTRAP_RECEIPT_KIND, "bootstrap_delivery");
        assert_eq!(FAIL_CODE_PRELOAD, "FOCUSA_PRELOAD_FAIL");
    }

    #[test]
    fn context_cognition_target_budgets_match_spec() {
        assert_eq!(target_dynamic_max_lines("cursor"), 160);
        assert_eq!(target_dynamic_max_lines("claude"), 200);
        assert_eq!(target_dynamic_max_lines("codex"), 180);
        assert_eq!(target_dynamic_max_lines("pi"), 120);
        assert_eq!(target_dynamic_max_lines("generic"), 120);
    }

    #[test]
    fn preload_awareness_is_compact_status_not_delivery_artifact() {
        let packet = preload_awareness(
            SURFACE_AGENT_PRELOAD,
            "degraded",
            false,
            true,
            true,
            Some("focusa_workpoint_resume"),
        );
        assert_eq!(packet["surface"], SURFACE_AGENT_PRELOAD);
        let serialized = packet.to_string();
        assert!(serialized.contains("preload degraded"));
        assert!(serialized.contains("Workpoint missing"));
        assert!(!serialized.contains("static_rule_lines"));
        assert!(!serialized.contains("dynamic_context_lines"));
    }
}
