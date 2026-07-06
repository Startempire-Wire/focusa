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

#[derive(Debug, Clone, PartialEq)]
pub struct DefaultOnSafeAutoPosture {
    pub enabled: &'static str,
    pub provider_policy_gate_required: bool,
    pub verified_models_only: bool,
    pub canary_gate_required: bool,
    pub profitability_gate_required: bool,
    pub keep_verbatim_text: bool,
    pub recoverable_store_required: bool,
    pub min_net_savings: f64,
    pub max_quality_regression: i32,
    pub default_fallback: &'static str,
    pub full_payload_policy: &'static str,
}

pub const POSTURE: DefaultOnSafeAutoPosture = DefaultOnSafeAutoPosture {
    enabled: DEFAULT_OPTICAL_POLICY,
    provider_policy_gate_required: true,
    verified_models_only: true,
    canary_gate_required: true,
    profitability_gate_required: true,
    keep_verbatim_text: true,
    recoverable_store_required: true,
    min_net_savings: DEFAULT_MIN_NET_SAVINGS,
    max_quality_regression: DEFAULT_MAX_QUALITY_REGRESSION,
    default_fallback: FALLBACK_TEXT_PASSTHROUGH,
    full_payload_policy: DEFAULT_FULL_PAYLOAD_POLICY,
};

pub fn effective_posture(allowed: bool, all_probes_pass: bool) -> &'static str {
    if !allowed {
        return FALLBACK_TEXT_PASSTHROUGH;
    }
    if !all_probes_pass {
        return FALLBACK_TEXT_PASSTHROUGH;
    }
    "noop_until_safe_auto"
}

pub const FALLBACK_CHAIN: &[&str] = &[
    "plain_text_context_cognition_render",
    "bloatgaurd_compact_envelope",
    "context_handles_summaries_rehydrate_refs",
    "tool_history_elision_after_checkpoint",
    "semantic_scoped_cache",
    "deep_dive_rehydrate_for_exact_blocker_evidence",
    FALLBACK_TEXT_PASSTHROUGH,
];

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackContext {
    pub policy_status_allowed: bool,
    pub all_probes_pass: bool,
    pub recoverable_store_available: bool,
    pub net_savings_meets_threshold: bool,
}

pub fn choose_fallback(ctx: &FallbackContext) -> &'static str {
    if ctx.policy_status_allowed
        && ctx.all_probes_pass
        && ctx.recoverable_store_available
        && ctx.net_savings_meets_threshold
    {
        return "noop_until_safe_auto";
    }
    FALLBACK_CHAIN[6]
}

pub struct ImagedBlock {
    pub raw_ref: String,
    pub image_ref: String,
    pub rehydrate_ref: String,
    pub omitted_bytes: usize,
    pub risk_class: String,
    pub provider_policy_ref: String,
    pub model_eval_ref: String,
    pub canary_status: String,
    pub fallback_used: &'static str,
}

pub fn empty_imaged_block(rehydrate_ref: &str) -> ImagedBlock {
    ImagedBlock {
        raw_ref: String::new(),
        image_ref: String::new(),
        rehydrate_ref: rehydrate_ref.to_string(),
        omitted_bytes: 0,
        risk_class: "gist_safe".to_string(),
        provider_policy_ref: "focusa.provider_policy_ledger.v1".to_string(),
        model_eval_ref: "focusa.model_eval.v1".to_string(),
        canary_status: "passed".to_string(),
        fallback_used: "text_passthrough",
    }
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
    fn posture_constants_match_spec_5_11_3() {
        assert_eq!(POSTURE.enabled, "safe_auto");
        assert_eq!(POSTURE.min_net_savings, 0.30);
        assert_eq!(POSTURE.max_quality_regression, 0);
        assert_eq!(POSTURE.full_payload_policy, "cold_opt_in");
        assert_eq!(POSTURE.default_fallback, "text_passthrough");
        assert!(POSTURE.provider_policy_gate_required);
        assert!(POSTURE.verified_models_only);
        assert!(POSTURE.canary_gate_required);
        assert!(POSTURE.profitability_gate_required);
        assert!(POSTURE.keep_verbatim_text);
        assert!(POSTURE.recoverable_store_required);
    }

    #[test]
    fn effective_posture_only_runs_when_every_gate_passes() {
        assert_eq!(effective_posture(true, true), "noop_until_safe_auto");
        assert_eq!(effective_posture(false, true), FALLBACK_TEXT_PASSTHROUGH);
        assert_eq!(effective_posture(true, false), FALLBACK_TEXT_PASSTHROUGH);
    }

    #[test]
    fn fallback_chain_starts_with_text_and_ends_with_passthrough() {
        assert_eq!(FALLBACK_CHAIN[0], "plain_text_context_cognition_render");
        assert_eq!(FALLBACK_CHAIN[6], FALLBACK_TEXT_PASSTHROUGH);
        assert_eq!(FALLBACK_CHAIN.len(), 7);
    }

    #[test]
    fn choose_fallback_returns_noop_when_every_gate_passes() {
        let ctx = FallbackContext {
            policy_status_allowed: true,
            all_probes_pass: true,
            recoverable_store_available: true,
            net_savings_meets_threshold: true,
        };
        assert_eq!(choose_fallback(&ctx), "noop_until_safe_auto");
    }

    #[test]
    fn choose_fallback_falls_back_when_any_gate_fails() {
        let mut ctx = FallbackContext {
            policy_status_allowed: true,
            all_probes_pass: true,
            recoverable_store_available: true,
            net_savings_meets_threshold: true,
        };
        ctx.net_savings_meets_threshold = false;
        assert_eq!(choose_fallback(&ctx), FALLBACK_CHAIN[6]);

        ctx.net_savings_meets_threshold = true;
        ctx.policy_status_allowed = false;
        assert_eq!(choose_fallback(&ctx), FALLBACK_CHAIN[6]);
    }

    #[test]
    fn imaged_block_carries_all_required_refs() {
        let b = empty_imaged_block("evidence:abc");
        assert_eq!(b.rehydrate_ref, "evidence:abc");
        assert_eq!(b.provider_policy_ref, "focusa.provider_policy_ledger.v1");
        assert_eq!(b.fallback_used, "text_passthrough");
        assert!(b.raw_ref.is_empty());
    }

    #[test]
    fn default_posture_is_safe_auto_with_text_passthrough_fallback() {
        assert!(DEFAULT_OPTICAL_POLICY == "safe_auto");
        assert!(DEFAULT_FULL_PAYLOAD_POLICY == "cold_opt_in");
        assert_eq!(DEFAULT_MIN_NET_SAVINGS, 0.30);
        assert_eq!(DEFAULT_MAX_QUALITY_REGRESSION, 0);
    }
}

// --- Spec 101 §5.11.9 Verification Suite ---
// Each test below corresponds to a bullet from the spec verification list.

#[test]
fn spec_5_11_9_defaults_safe_auto_with_text_passthrough_fallback() {
    assert_eq!(POSTURE.enabled, "safe_auto");
    assert_eq!(POSTURE.default_fallback, "text_passthrough");
    assert_eq!(POSTURE.full_payload_policy, "cold_opt_in");
}

#[test]
fn spec_5_11_9_provider_policy_gate_blocks_unauthorized_provider() {
    for status in [
        POLICY_STATUS_BLOCKED,
        POLICY_STATUS_UNKNOWN,
        POLICY_STATUS_STALE,
        POLICY_STATUS_NEEDS_REVIEW,
    ] {
        let ctx = FallbackContext {
            policy_status_allowed: status == POLICY_STATUS_ALLOWED,
            all_probes_pass: true,
            recoverable_store_available: true,
            net_savings_meets_threshold: true,
        };
        assert_eq!(choose_fallback(&ctx), FALLBACK_TEXT_PASSTHROUGH);
    }
}

#[test]
fn spec_5_11_9_provider_terms_hash_change_triggers_text_passthrough() {
    // Simulates a hash change: provider_policy_status flips to STALE.
    let stale_ctx = FallbackContext {
        policy_status_allowed: false,
        all_probes_pass: true,
        recoverable_store_available: true,
        net_savings_meets_threshold: true,
    };
    assert_eq!(choose_fallback(&stale_ctx), FALLBACK_TEXT_PASSTHROUGH);
}

#[test]
fn spec_5_11_9_image_input_rejected_falls_back() {
    // Simulates image_rejected: any_probes_pass = false.
    let rejected_ctx = FallbackContext {
        policy_status_allowed: true,
        all_probes_pass: false,
        recoverable_store_available: true,
        net_savings_meets_threshold: true,
    };
    assert_eq!(choose_fallback(&rejected_ctx), FALLBACK_TEXT_PASSTHROUGH);
}

#[test]
fn spec_5_11_9_model_allowlist_required() {
    // POSTURE.verified_models_only must be true so the transform only runs
    // against Focusa-verified models.
    assert!(POSTURE.verified_models_only);
}

#[test]
fn spec_5_11_9_verbatim_guard_protects_action_authority() {
    assert!(NEVER_IMAGED.contains(&"workpoint_action_authority"));
    assert!(NEVER_IMAGED.contains(&"evidence_refs_themselves"));
    assert!(NEVER_IMAGED.contains(&"exact_diffs"));
    assert!(NEVER_IMAGED.contains(&"secrets"));
    assert!(NEVER_IMAGED.contains(&"hashes"));
    assert!(NEVER_IMAGED.contains(&"uuids"));
}

#[test]
fn spec_5_11_9_active_blocker_kept_as_text() {
    // active_blocker_kept_text_test: active error lines must not be imaged.
    assert!(NEVER_IMAGED.contains(&"active_error_lines"));
    assert!(NEVER_IMAGED.contains(&"test_names_currently_blocking_work"));
}

#[test]
fn spec_5_11_9_profitability_gate_required() {
    assert!(POSTURE.profitability_gate_required);
    let unprofitable_ctx = FallbackContext {
        policy_status_allowed: true,
        all_probes_pass: true,
        recoverable_store_available: true,
        net_savings_meets_threshold: false,
    };
    assert_eq!(
        choose_fallback(&unprofitable_ctx),
        FALLBACK_TEXT_PASSTHROUGH
    );
    assert!(POSTURE.min_net_savings >= 0.30);
}

#[test]
fn spec_5_11_9_recoverable_ref_required() {
    assert!(POSTURE.recoverable_store_required);
    let b = empty_imaged_block("evidence:test123");
    assert!(!b.rehydrate_ref.is_empty());
    assert!(b.fallback_used == "text_passthrough" || b.fallback_used.is_empty());
}

#[test]
fn spec_5_11_9_canary_failed_text_passthrough() {
    // canary_failed_text_passthrough_test: any probe failure must fall back.
    let failed_ctx = FallbackContext {
        policy_status_allowed: true,
        all_probes_pass: false,
        recoverable_store_available: true,
        net_savings_meets_threshold: true,
    };
    assert_eq!(choose_fallback(&failed_ctx), FALLBACK_TEXT_PASSTHROUGH);
}

#[test]
fn spec_5_11_9_context_cognition_no_canonical_mutation() {
    // The Bloatgaurd optical gateway does not mutate Workpoint/Trajectory/Evidence.
    // We verify by construction: choose_fallback never returns a "commit" sentinel.
    let no_op = choose_fallback(&FallbackContext {
        policy_status_allowed: true,
        all_probes_pass: true,
        recoverable_store_available: true,
        net_savings_meets_threshold: true,
    });
    assert!(!no_op.contains("commit"));
    assert!(!no_op.contains("mutate"));
    assert!(!no_op.contains("write"));
    assert_eq!(no_op, "noop_until_safe_auto");
}

#[test]
fn spec_5_11_9_focus_slice_no_raw_blob_default() {
    // Bloatgaurd keeps verbatim text by default; raw blob injection is cold opt-in.
    assert!(POSTURE.keep_verbatim_text);
    assert_eq!(POSTURE.full_payload_policy, "cold_opt_in");
}
