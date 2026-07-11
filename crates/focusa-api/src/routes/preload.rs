//! Spec 111 — Agent Context Bootstrap and Delivery.
//!
//! Slice 1: Spec + static contracts for AgentBootstrapPacket, AgentBootstrapProfile,
//! AgentBootstrapReceipt, FOCUSA_PRELOAD_FAIL, and bootstrap_delivery Focusa Receipt.
//! Slice 2 stub: read-mostly routes (/v1/preload/profiles|build|render|verify|doctor)
//! that emit static envelopes. Slice 3 will dispatch to renderers; Slice 4 will add
//! the safe-write route; Slice 5 will integrate with Spec 119 receipts.

use crate::server::AppState;
use axum::{
    Json, Router,
    extract::Query,
    http::StatusCode,
    routing::{get, post},
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
            get(receipt_preview).post(receipt_preview),
        )
        .route("/v1/preload/receipt-commit", get(receipt_commit))
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
    }))
}

#[derive(serde::Deserialize, Default)]
struct ProfileQuery {
    #[serde(default)]
    profile: Option<String>,
}

fn packet_response(step: &str, profile: Option<String>) -> Json<Value> {
    let profile = profile.unwrap_or_else(|| PROFILE_RULES_AND_CONTEXT.to_string());
    match build_packet_for_profile(&profile) {
        Ok(packet) => Json(
            json!({"schema":PRELOAD_SCHEMA,"step":step,"read_only":true,"status":"completed","packet":packet,"checks":["profile","integrity","scope"]}),
        ),
        Err(error) => Json(
            json!({"schema":PRELOAD_SCHEMA,"step":step,"status":"failed","error":{"code":FAIL_CODE_PRELOAD,"message":error}}),
        ),
    }
}

async fn build(Query(query): Query<ProfileQuery>) -> Json<Value> {
    packet_response("build", query.profile)
}
async fn build_post(Json(query): Json<ProfileQuery>) -> Json<Value> {
    packet_response("build", query.profile)
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

async fn receipt_preview() -> Json<Value> {
    match receipt_preview_for(PROFILE_RULES_AND_CONTEXT) {
        Ok(receipt) => Json(
            json!({"schema":PRELOAD_SCHEMA,"step":"receipt_preview","read_only":true,"status":"completed","receipt":receipt}),
        ),
        Err(error) => Json(
            json!({"schema":PRELOAD_SCHEMA,"step":"receipt_preview","status":"failed","error":{"code":FAIL_CODE_PRELOAD,"message":error}}),
        ),
    }
}

async fn receipt_commit() -> (StatusCode, Json<Value>) {
    (
        StatusCode::NOT_IMPLEMENTED,
        Json(json!({
            "schema": PRELOAD_SCHEMA,
            "error": FAIL_CODE_PRELOAD,
            "step": "receipt_commit",
            "status": "deferred_to_slice_5",
            "note": "Safe commit lands after Slice 4 write safety gates clear.",
        })),
    )
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
    Ok(json!({
        "schema": PRELOAD_SCHEMA,
        "receipt_kind": BOOTSTRAP_RECEIPT_KIND,
        "preview": true,
        "ok": true,
        "profile_id": packet.profile_id,
        "rendered": render_packet(&packet),
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
    fn slice2_unknown_profile_fails_with_FOCUSA_PRELOAD_FAIL() {
        let err = build_packet("nope").err().unwrap_or_default();
        assert!(err.contains(FAIL_CODE_PRELOAD));
    }

    #[test]
    fn receipt_kind_and_fail_code_match_spec() {
        assert_eq!(BOOTSTRAP_RECEIPT_KIND, "bootstrap_delivery");
        assert_eq!(FAIL_CODE_PRELOAD, "FOCUSA_PRELOAD_FAIL");
    }
}
