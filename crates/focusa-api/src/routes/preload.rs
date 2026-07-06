//! Spec 111 — Agent Context Bootstrap and Delivery.
//!
//! Slice 1: Spec + static contracts for AgentBootstrapPacket, AgentBootstrapProfile,
//! AgentBootstrapReceipt, FOCUSA_PRELOAD_FAIL, and bootstrap_delivery Focusa Receipt.
//! Slice 2 stub: read-mostly routes (/v1/preload/profiles|build|render|verify|doctor)
//! that emit static envelopes. Slice 3 will dispatch to renderers; Slice 4 will add
//! the safe-write route; Slice 5 will integrate with Spec 119 receipts.

use crate::server::AppState;
use axum::{Json, Router, http::StatusCode, routing::get};
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
        .route("/v1/preload/build", get(build))
        .route("/v1/preload/render", get(render))
        .route("/v1/preload/verify", get(verify))
        .route("/v1/preload/doctor", get(doctor))
        .route("/v1/preload/receipt-preview", get(receipt_preview))
        .route("/v1/preload/receipt-commit", get(receipt_commit))
}

async fn list_profiles() -> Json<Value> {
    Json(json!({
        "schema": PRELOAD_SCHEMA,
        "profiles": PROFILE_IDS,
        "default_profile": PROFILE_RULES_AND_CONTEXT,
        "read_only": true,
    }))
}

async fn build() -> Json<Value> {
    Json(json!({
        "schema": PRELOAD_SCHEMA,
        "step": "build",
        "read_only": true,
        "status": "noop",
        "note": "Slice 1 stub; full render + safe write land in slices 3-4",
    }))
}

async fn render() -> Json<Value> {
    Json(json!({
        "schema": PRELOAD_SCHEMA,
        "step": "render",
        "read_only": true,
        "status": "noop",
        "render_modes": ["static_rule", "dynamic_context", "acceptance_prompt"],
    }))
}

async fn verify() -> Json<Value> {
    Json(json!({
        "schema": PRELOAD_SCHEMA,
        "step": "verify",
        "read_only": true,
        "status": "noop",
        "checks": ["scope", "auth", "integrity", "idempotency"],
    }))
}

async fn doctor() -> Json<Value> {
    Json(json!({
        "schema": PRELOAD_SCHEMA,
        "step": "doctor",
        "read_only": true,
        "status": "noop",
        "report": {"ok": false, "checks": []},
    }))
}

async fn receipt_preview() -> Json<Value> {
    Json(json!({
        "schema": PRELOAD_SCHEMA,
        "step": "receipt_preview",
        "read_only": true,
        "receipt_kind": BOOTSTRAP_RECEIPT_KIND,
        "status": "noop",
    }))
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
