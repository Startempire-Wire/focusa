//! Spec 101 §5.11 — Bloatgaurd Optical Context Gateway
//!
//! Default-on safe_auto: converts dense, old, non-verbatim-critical context into
//! recoverable image artifacts only when EVERY gate passes. Until then the
//! transform is a no-op (text_passthrough).
//!
//! Gates (§5.11.3-5.11.6):
//!   - bloatgaurd.optical_context.enabled = "safe_auto"
//!   - provider_policy_gate required (focusa.provider_policy_ledger.v1)
//!   - verified_models_only + compatibility probe
//!   - profitability_gate + min_net_savings >= 0.30
//!   - canary_gate required
//!   - keep_verbatim_text = true
//!   - recoverable_store required (raw_ref / image_ref / rehydrate_ref)
//!   - default_fallback = "text_passthrough"
//!   - max_quality_regression = 0
//!   - full_payload_policy = "cold_opt_in"

use crate::server::AppState;
use axum::{Json, Router, routing::get};
use serde_json::{Value, json};
use std::sync::Arc;

pub const BLOATGAURD_OPTICAL_SCHEMA: &str = "focusa.bloatgaurd_optical.v1";
pub const PROVIDER_POLICY_LEDGER_SCHEMA: &str = "focusa.provider_policy_ledger.v1";

pub const POLICY_STATUS_ALLOWED: &str = "allowed";
pub const POLICY_STATUS_BLOCKED: &str = "blocked";
pub const POLICY_STATUS_UNKNOWN: &str = "unknown";
pub const POLICY_STATUS_STALE: &str = "stale";
pub const POLICY_STATUS_NEEDS_REVIEW: &str = "needs_review";

pub const FALLBACK_TEXT_PASSTHROUGH: &str = "text_passthrough";
pub const POSTURE_FORBIDDEN_REASONS: &[&str] = &[
    "provider_policy_unknown",
    "provider_banned",
    "model_not_verified",
    "image_rejected",
    "canary_failed",
    "not_profitable",
];

pub const IMAGED_ALLOWED_KINDS: &[&str] = &[
    "old_dense_tool_output",
    "old_command_logs",
    "old_collapsed_history_after_checkpoint",
    "large_non_current_tool_docs",
    "large_structured_json_behind_rehydrate_ref",
    "diagnostic_dumps_gist_only",
];

pub const NEVER_IMAGED: &[&str] = &[
    "operator_current_ask",
    "recent_live_turns",
    "workpoint_action_authority",
    "trajectory_current_goal_gap_authority",
    "evidence_refs_themselves",
    "secrets",
    "tokens",
    "hashes",
    "uuids",
    "twelve_char_identifiers",
    "file_paths_needed_for_edits",
    "exact_diffs",
    "active_error_lines",
    "test_names_currently_blocking_work",
    "package_versions_in_fix",
    "security_sensitive_content",
];

pub const DEFAULT_OPTICAL_POLICY: &str = "safe_auto";
pub const DEFAULT_MIN_NET_SAVINGS: f64 = 0.30;
pub const DEFAULT_MAX_QUALITY_REGRESSION: i32 = 0;
pub const DEFAULT_FULL_PAYLOAD_POLICY: &str = "cold_opt_in";

pub fn router() -> Router<Arc<AppState>> {
    Router::new()
        .route("/v1/bloatgaurd/optical/policy", get(optical_policy))
        .route("/v1/bloatgaurd/optical/ledger", get(provider_policy_ledger))
        .route("/v1/bloatgaurd/optical/probe", get(compatibility_probe))
        .route("/v1/bloatgaurd/optical/imaged-kinds", get(imaged_kinds))
        .route("/v1/bloatgaurd/optical/never-imaged", get(never_imaged))
}

async fn optical_policy() -> Json<Value> {
    Json(json!({
        "schema": BLOATGAURD_OPTICAL_SCHEMA,
        "default_posture": DEFAULT_OPTICAL_POLICY,
        "min_net_savings": DEFAULT_MIN_NET_SAVINGS,
        "max_quality_regression": DEFAULT_MAX_QUALITY_REGRESSION,
        "full_payload_policy": DEFAULT_FULL_PAYLOAD_POLICY,
        "default_fallback": FALLBACK_TEXT_PASSTHROUGH,
        "keep_verbatim_text": true,
        "provider_policy_gate_required": true,
        "verified_models_only": true,
        "canary_gate_required": true,
        "recoverable_store_required": true,
    }))
}

async fn provider_policy_ledger() -> Json<Value> {
    Json(json!({
        "schema": PROVIDER_POLICY_LEDGER_SCHEMA,
        "statuses": [
            POLICY_STATUS_ALLOWED,
            POLICY_STATUS_BLOCKED,
            POLICY_STATUS_UNKNOWN,
            POLICY_STATUS_STALE,
            POLICY_STATUS_NEEDS_REVIEW,
        ],
        "runtime_rule": "if status != allowed then fallback=text_passthrough",
        "review_required_on_change": true,
    }))
}

async fn compatibility_probe() -> Json<Value> {
    Json(json!({
        "schema": BLOATGAURD_OPTICAL_SCHEMA,
        "probe": [
            "provider_supports_image_input",
            "provider_counts_image_input_as_tokens",
            "model_accepts_image_input",
            "model_is_focusa_verified_for_dense_text_reading",
            "pricing_did_not_flip_profitability_math",
            "request_limits_still_allow_payload",
            "canary_read_passes",
        ],
        "any_failure_fallback": FALLBACK_TEXT_PASSTHROUGH,
    }))
}

async fn imaged_kinds() -> Json<Value> {
    Json(json!({
        "schema": BLOATGAURD_OPTICAL_SCHEMA,
        "imaged_by_default": IMAGED_ALLOWED_KINDS,
    }))
}

async fn never_imaged() -> Json<Value> {
    Json(json!({
        "schema": BLOATGAURD_OPTICAL_SCHEMA,
        "never_imaged": NEVER_IMAGED,
    }))
}

pub fn decide(action: &str, status: &str) -> &'static str {
    if status != POLICY_STATUS_ALLOWED {
        return FALLBACK_TEXT_PASSTHROUGH;
    }
    match action {
        "decide_imaged" => "noop_until_safe_auto",
        _ => FALLBACK_TEXT_PASSTHROUGH,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeCheckStatus {
    Pass,
    Fail,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeCheck {
    pub id: &'static str,
    pub status: ProbeCheckStatus,
}

pub const COMPATIBILITY_PROBE_IDS: &[&str] = &[
    "provider_supports_image_input",
    "provider_counts_image_input_as_tokens",
    "model_accepts_image_input",
    "model_is_focusa_verified_for_dense_text_reading",
    "pricing_did_not_flip_profitability_math",
    "request_limits_still_allow_payload",
    "canary_read_passes",
];

/// Run the compatibility probe given a provider-policy status and per-probe
/// pass/fail map. Returns the per-probe results + the overall fallback
/// decision per §5.11.6: any failure => text_passthrough with a reason.
pub fn run_compatibility_probe(
    provider_policy_status: &str,
    per_probe: &[(String, ProbeCheckStatus)],
) -> (Vec<ProbeCheck>, &'static str) {
    let mut results: Vec<ProbeCheck> = per_probe
        .iter()
        .map(|(id, status)| ProbeCheck {
            id: Box::leak(id.clone().into_boxed_str()),
            status: status.clone(),
        })
        .collect();
    for id in COMPATIBILITY_PROBE_IDS {
        if !results.iter().any(|c| c.id == *id) {
            results.push(ProbeCheck {
                id,
                status: ProbeCheckStatus::Pass,
            });
        }
    }
    let decision = if provider_policy_status != POLICY_STATUS_ALLOWED {
        FALLBACK_TEXT_PASSTHROUGH
    } else if results.iter().any(|c| c.status == ProbeCheckStatus::Fail) {
        FALLBACK_TEXT_PASSTHROUGH
    } else {
        "noop_until_safe_auto"
    };
    (results, decision)
}

#[cfg(test)]
mod probe_tests {
    use super::*;

    #[test]
    fn any_probe_failure_falls_back_to_text_passthrough() {
        let mut per = Vec::new();
        for id in COMPATIBILITY_PROBE_IDS {
            per.push((id.to_string(), ProbeCheckStatus::Pass));
        }
        per[3].1 = ProbeCheckStatus::Fail;
        let (_results, decision) = run_compatibility_probe(POLICY_STATUS_ALLOWED, &per);
        assert_eq!(decision, FALLBACK_TEXT_PASSTHROUGH);
    }

    #[test]
    fn blocked_provider_status_falls_back_regardless_of_probe() {
        let mut per = Vec::new();
        for id in COMPATIBILITY_PROBE_IDS {
            per.push((id.to_string(), ProbeCheckStatus::Pass));
        }
        let (_results, decision) = run_compatibility_probe(POLICY_STATUS_BLOCKED, &per);
        assert_eq!(decision, FALLBACK_TEXT_PASSTHROUGH);
    }

    #[test]
    fn all_pass_allowed_policy_returns_noop_until_safe_auto() {
        let mut per = Vec::new();
        for id in COMPATIBILITY_PROBE_IDS {
            per.push((id.to_string(), ProbeCheckStatus::Pass));
        }
        let (_results, decision) = run_compatibility_probe(POLICY_STATUS_ALLOWED, &per);
        assert_eq!(decision, "noop_until_safe_auto");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_constants_match_spec() {
        assert_eq!(BLOATGAURD_OPTICAL_SCHEMA, "focusa.bloatgaurd_optical.v1");
        assert_eq!(
            PROVIDER_POLICY_LEDGER_SCHEMA,
            "focusa.provider_policy_ledger.v1"
        );
        assert_eq!(DEFAULT_OPTICAL_POLICY, "safe_auto");
        assert_eq!(FALLBACK_TEXT_PASSTHROUGH, "text_passthrough");
    }

    #[test]
    fn non_allowed_provider_status_falls_back_to_text_passthrough() {
        for status in [
            POLICY_STATUS_BLOCKED,
            POLICY_STATUS_UNKNOWN,
            POLICY_STATUS_STALE,
            POLICY_STATUS_NEEDS_REVIEW,
        ] {
            assert_eq!(decide("decide_imaged", status), FALLBACK_TEXT_PASSTHROUGH);
        }
    }

    #[test]
    fn never_imaged_contains_exact_identifiers_and_action_authority() {
        assert!(NEVER_IMAGED.contains(&"workpoint_action_authority"));
        assert!(NEVER_IMAGED.contains(&"evidence_refs_themselves"));
        assert!(NEVER_IMAGED.contains(&"exact_diffs"));
        assert!(NEVER_IMAGED.contains(&"secrets"));
    }

    #[test]
    fn default_posture_is_safe_auto_with_text_passthrough_fallback() {
        assert!(DEFAULT_OPTICAL_POLICY == "safe_auto");
        assert!(DEFAULT_FULL_PAYLOAD_POLICY == "cold_opt_in");
        assert_eq!(DEFAULT_MIN_NET_SAVINGS, 0.30);
        assert_eq!(DEFAULT_MAX_QUALITY_REGRESSION, 0);
    }
}
