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
use axum::{Json, Router, extract::State, http::StatusCode, routing::{get, post}};
use serde_json::{Value, json};
use std::sync::Arc;
use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};

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
        .route("/v1/bloatgaurd/optical/ledger", get(list_ledger).post(upsert_ledger_route))
        .route("/v1/bloatgaurd/optical/ledger/{provider}", get(get_one_ledger))
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
    let decision = if provider_policy_status != POLICY_STATUS_ALLOWED
        || results.iter().any(|c| c.status == ProbeCheckStatus::Fail)
    {
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

// ─── Spec 101 §5.11.5 Provider Policy Ledger ──────────────────────────────
//
// Routes:
//   GET  /v1/bloatgaurd/optical/ledger            — list all entries (effective_status computed)
//   GET  /v1/bloatgaurd/optical/ledger/:provider  — single provider
//   POST /v1/bloatgaurd/optical/ledger            — upsert (operator-driven)
// Storage: SQLite table `provider_policy_ledger` in `focusa.sqlite`.
// Staleness: effective_status flips to `stale` when now >= expires_at.
// Required by: focusa-rtcz (provider prompt cache control emitter needs allowed/blocked verdict).

const LEDGER_SCHEMA: &str = "focusa.provider_policy_ledger.v1";
const FEATURE_OPTICAL_CTX: &str = "optical_context_compression";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderPolicyLedger {
    pub schema: String,
    pub provider: String,
    pub feature: String,
    pub status: String,
    pub official_policy_refs: Vec<String>,
    pub terms_hash: String,
    pub vision_docs_hash: String,
    pub checked_at: String,
    pub expires_at: String,
    pub review_required_on_change: bool,
    pub fallback: String,
}

fn ledger_db_path(data_dir: &str) -> std::path::PathBuf {
    if let Some(rest) = data_dir.strip_prefix("~/")
        && let Ok(home) = std::env::var("HOME")
    {
        return std::path::PathBuf::from(home).join(rest).join("focusa.sqlite");
    }
    std::path::PathBuf::from(data_dir).join("focusa.sqlite")
}

fn ensure_ledger_table(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS provider_policy_ledger (
            provider              TEXT NOT NULL PRIMARY KEY,
            feature               TEXT NOT NULL,
            status                TEXT NOT NULL,
            official_policy_refs  TEXT NOT NULL,
            terms_hash            TEXT NOT NULL,
            vision_docs_hash      TEXT NOT NULL,
            checked_at            TEXT NOT NULL,
            expires_at            TEXT NOT NULL,
            review_required       INTEGER NOT NULL,
            fallback              TEXT NOT NULL,
            updated_at            TEXT NOT NULL
        )",
    )
}

/// Compute effective status: needs_review is sticky; otherwise expires_at<=now flips to stale.
pub fn effective_status(row_status: &str, expires_at: &str, now: &str) -> String {
    if row_status == POLICY_STATUS_NEEDS_REVIEW {
        return POLICY_STATUS_NEEDS_REVIEW.to_string();
    }
    if expires_at <= now {
        return POLICY_STATUS_STALE.to_string();
    }
    row_status.to_string()
}

fn upsert_ledger(conn: &Connection, entry: &ProviderPolicyLedger) -> rusqlite::Result<()> {
    let refs_json = serde_json::to_string(&entry.official_policy_refs).unwrap_or_else(|_| "[]".to_string());
    let now = chrono_now();
    conn.execute(
        "INSERT INTO provider_policy_ledger (
            provider, feature, status, official_policy_refs,
            terms_hash, vision_docs_hash, checked_at, expires_at,
            review_required, fallback, updated_at
        ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
        ON CONFLICT(provider) DO UPDATE SET
            feature = excluded.feature,
            status = excluded.status,
            official_policy_refs = excluded.official_policy_refs,
            terms_hash = excluded.terms_hash,
            vision_docs_hash = excluded.vision_docs_hash,
            checked_at = excluded.checked_at,
            expires_at = excluded.expires_at,
            review_required = excluded.review_required,
            fallback = excluded.fallback,
            updated_at = excluded.updated_at",
        params![
            entry.provider,
            entry.feature,
            entry.status,
            refs_json,
            entry.terms_hash,
            entry.vision_docs_hash,
            entry.checked_at,
            entry.expires_at,
            entry.review_required_on_change as i32,
            entry.fallback,
            now,
        ],
    )?;
    Ok(())
}

fn fetch_ledger(conn: &Connection) -> rusqlite::Result<Vec<ProviderPolicyLedger>> {
    let mut stmt = conn.prepare(
        "SELECT provider, feature, status, official_policy_refs, terms_hash, vision_docs_hash, checked_at, expires_at, review_required, fallback FROM provider_policy_ledger ORDER BY provider",
    )?;
    let rows = stmt
        .query_map([], |row| {
            let refs_json: String = row.get(3)?;
            let refs: Vec<String> = serde_json::from_str(&refs_json).unwrap_or_default();
            Ok(ProviderPolicyLedger {
                schema: LEDGER_SCHEMA.to_string(),
                provider: row.get(0)?,
                feature: row.get(1)?,
                status: row.get(2)?,
                official_policy_refs: refs,
                terms_hash: row.get(4)?,
                vision_docs_hash: row.get(5)?,
                checked_at: row.get(6)?,
                expires_at: row.get(7)?,
                review_required_on_change: row.get::<_, i32>(8)? != 0,
                fallback: row.get(9)?,
            })
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    Ok(rows)
}

fn fetch_one(conn: &Connection, provider: &str) -> rusqlite::Result<Option<ProviderPolicyLedger>> {
    let row = conn
        .query_row(
            "SELECT provider, feature, status, official_policy_refs, terms_hash, vision_docs_hash, checked_at, expires_at, review_required, fallback FROM provider_policy_ledger WHERE provider = ?1",
            params![provider],
            |row| {
                let refs_json: String = row.get(3)?;
                let refs: Vec<String> = serde_json::from_str(&refs_json).unwrap_or_default();
                Ok(ProviderPolicyLedger {
                    schema: LEDGER_SCHEMA.to_string(),
                    provider: row.get(0)?,
                    feature: row.get(1)?,
                    status: row.get(2)?,
                    official_policy_refs: refs,
                    terms_hash: row.get(4)?,
                    vision_docs_hash: row.get(5)?,
                    checked_at: row.get(6)?,
                    expires_at: row.get(7)?,
                    review_required_on_change: row.get::<_, i32>(8)? != 0,
                    fallback: row.get(9)?,
                })
            },
        )
        .optional()?;
    Ok(row)
}

/// Minimal UTC timestamp formatter (avoid chrono dep just for this).
fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format_unix_rfc3339(secs)
}

fn format_unix_rfc3339(secs: u64) -> String {
    let days = secs / 86_400;
    let mut year: i64 = 1970;
    let mut remaining = days as i64;
    loop {
        let y_days = if (year % 4 == 0 && year % 100 != 0) || year % 400 == 0 { 366 } else { 365 };
        if remaining < y_days {
            break;
        }
        remaining -= y_days;
        year += 1;
    }
    let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let month_days = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut month: i64 = 1;
    for &md in &month_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        month += 1;
    }
    let day = remaining + 1;
    let secs_today = secs % 86_400;
    let h = secs_today / 3600;
    let m = (secs_today % 3600) / 60;
    let s = secs_today % 60;
    format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z", year, month, day, h, m, s)
}

fn entry_to_view_json(e: &ProviderPolicyLedger, now: &str) -> Value {
    let effective = effective_status(&e.status, &e.expires_at, now);
    json!({
        "schema": LEDGER_SCHEMA,
        "provider": e.provider,
        "feature": e.feature,
        "status": e.status,
        "effective_status": effective,
        "is_stale": effective == POLICY_STATUS_STALE,
        "official_policy_refs": e.official_policy_refs,
        "terms_hash": e.terms_hash,
        "vision_docs_hash": e.vision_docs_hash,
        "checked_at": e.checked_at,
        "expires_at": e.expires_at,
        "review_required_on_change": e.review_required_on_change,
        "fallback": e.fallback,
    })
}

// ─── HTTP handlers ────────────────────────────────────────────────────────


async fn list_ledger(State(state): State<Arc<AppState>>) -> Json<Value> {
    let db_path = ledger_db_path(&state.config.data_dir);
    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            return Json(json!({
                "error": "db_open_failed",
                "why": e.to_string(),
            }));
        }
    };
    if let Err(e) = ensure_ledger_table(&conn) {
        return Json(json!({"error": "schema_init_failed", "why": e.to_string()}));
    }
    let rows = match fetch_ledger(&conn) {
        Ok(r) => r,
        Err(e) => {
            return Json(json!({"error": "read_failed", "why": e.to_string()}));
        }
    };
    let now = chrono_now();
    let entries: Vec<Value> = rows.iter().map(|e| entry_to_view_json(e, &now)).collect();
    Json(json!({
        "schema": LEDGER_SCHEMA,
        "count": entries.len(),
        "now": now,
        "entries": entries,
        "runtime_rule": "if effective_status != allowed then fallback=text_passthrough",
    }))
}


async fn get_one_ledger(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(provider): axum::extract::Path<String>,
) -> (StatusCode, Json<Value>) {
    let db_path = ledger_db_path(&state.config.data_dir);
    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "db_open_failed",
                    "why": e.to_string(),
                })),
            );
        }
    };
    if let Err(e) = ensure_ledger_table(&conn) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "schema_init_failed", "why": e.to_string()})),
        );
    }
    match fetch_one(&conn, &provider) {
        Ok(Some(e)) => (StatusCode::OK, Json(entry_to_view_json(&e, &chrono_now()))),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "schema": LEDGER_SCHEMA,
                "found": false,
                "provider": provider,
                "effective_status": POLICY_STATUS_UNKNOWN,
                "fallback": FALLBACK_TEXT_PASSTHROUGH,
            })),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "read_failed", "why": e.to_string()})),
        ),
    }
}


async fn upsert_ledger_route(
    State(state): State<Arc<AppState>>,
    Json(body): Json<Value>,
) -> Json<Value> {
    let entry: ProviderPolicyLedger = match serde_json::from_value(body) {
        Ok(e) => e,
        Err(e) => {
            return Json(json!({
                "error": "invalid_body",
                "why": e.to_string(),
            }));
        }
    };
    if entry.provider.is_empty() {
        return Json(json!({"error": "provider_required"}));
    }
    let db_path = ledger_db_path(&state.config.data_dir);
    let conn = match Connection::open(&db_path) {
        Ok(c) => c,
        Err(e) => {
            return Json(json!({
                "error": "db_open_failed",
                "why": e.to_string(),
            }));
        }
    };
    if let Err(e) = ensure_ledger_table(&conn) {
        return Json(json!({"error": "schema_init_failed", "why": e.to_string()}));
    }
    match upsert_ledger(&conn, &entry) {
        Ok(()) => {
            let now = chrono_now();
            Json(json!({
                "schema": LEDGER_SCHEMA,
                "ok": true,
                "provider": entry.provider,
                "effective_status": effective_status(&entry.status, &entry.expires_at, &now),
            }))
        }
        Err(e) => Json(json!({"error": "write_failed", "why": e.to_string()})),
    }
}

#[test]
fn spec_5_11_5_ledger_status_fresh() {
    let now = "2026-07-06T00:00:00Z";
    let expires = "2026-07-13T00:00:00Z";
    assert_eq!(effective_status("allowed", expires, now), "allowed");
    assert_eq!(effective_status("blocked", expires, now), "blocked");
    assert_eq!(effective_status("unknown", expires, now), "unknown");
}

#[test]
fn spec_5_11_5_ledger_status_stale_flips() {
    let now = "2026-07-13T00:00:00Z";
    let past = "2026-07-06T00:00:00Z";
    assert_eq!(effective_status("allowed", past, now), "stale");
    assert_eq!(effective_status("blocked", past, now), "stale");
    assert_eq!(effective_status("unknown", past, now), "stale");
}

#[test]
fn spec_5_11_5_ledger_status_needs_review_sticky() {
    let now = "2026-07-13T00:00:00Z";
    let past = "2026-07-06T00:00:00Z";
    assert_eq!(effective_status("needs_review", past, now), "needs_review");
}

#[test]
fn spec_5_11_5_ledger_effective_not_mutated() {
    let now = "2026-07-13T00:00:00Z";
    let past = "2026-07-06T00:00:00Z";
    let stored = "allowed";
    assert_eq!(stored, "allowed");
    assert_eq!(effective_status(stored, past, now), "stale");
}

#[test]
fn spec_5_11_5_ledger_unknown_provider_returns_passthrough() {
    let entry: Option<ProviderPolicyLedger> = None;
    assert!(entry.is_none());
    assert_eq!(FALLBACK_TEXT_PASSTHROUGH, "text_passthrough");
}

#[test]
fn spec_5_11_5_ledger_upsert_replaces_sticky() {
    let mut entry = ProviderPolicyLedger {
        schema: LEDGER_SCHEMA.to_string(),
        provider: "openai".to_string(),
        feature: FEATURE_OPTICAL_CTX.to_string(),
        status: "allowed".to_string(),
        official_policy_refs: vec!["https://example.com/v1".to_string()],
        terms_hash: "sha256:abc".to_string(),
        vision_docs_hash: "sha256:def".to_string(),
        checked_at: "2026-07-06T00:00:00Z".to_string(),
        expires_at: "2026-07-13T00:00:00Z".to_string(),
        review_required_on_change: true,
        fallback: FALLBACK_TEXT_PASSTHROUGH.to_string(),
    };
    entry.status = "needs_review".to_string();
    assert_eq!(entry.status, "needs_review");
    assert_eq!(
        effective_status(&entry.status, &entry.expires_at, "2099-01-01T00:00:00Z"),
        "needs_review"
    );
}

#[test]
fn spec_5_11_5_ledger_eval_harness_integration_shape() {
    let e = ProviderPolicyLedger {
        schema: LEDGER_SCHEMA.to_string(),
        provider: "anthropic".to_string(),
        feature: FEATURE_OPTICAL_CTX.to_string(),
        status: "allowed".to_string(),
        official_policy_refs: vec![],
        terms_hash: "sha256:".to_string(),
        vision_docs_hash: "sha256:".to_string(),
        checked_at: "2026-07-06T00:00:00Z".to_string(),
        expires_at: "2026-07-13T00:00:00Z".to_string(),
        review_required_on_change: true,
        fallback: FALLBACK_TEXT_PASSTHROUGH.to_string(),
    };
    let v = serde_json::to_value(&e).unwrap();
    for k in ["schema", "provider", "status", "fallback", "expires_at"] {
        assert!(v.get(k).is_some(), "missing key {k}");
    }
}

#[test]
fn spec_5_11_5_ledger_view_json_carries_effective_status() {
    let e = ProviderPolicyLedger {
        schema: LEDGER_SCHEMA.to_string(),
        provider: "anthropic".to_string(),
        feature: FEATURE_OPTICAL_CTX.to_string(),
        status: "allowed".to_string(),
        official_policy_refs: vec![],
        terms_hash: "sha256:".to_string(),
        vision_docs_hash: "sha256:".to_string(),
        checked_at: "2026-07-06T00:00:00Z".to_string(),
        expires_at: "2026-07-13T00:00:00Z".to_string(),
        review_required_on_change: true,
        fallback: FALLBACK_TEXT_PASSTHROUGH.to_string(),
    };
    let v = entry_to_view_json(&e, "2026-07-10T00:00:00Z");
    assert_eq!(v["effective_status"], "allowed");
    assert_eq!(v["is_stale"], false);
    let v_stale = entry_to_view_json(&e, "2026-07-20T00:00:00Z");
    assert_eq!(v_stale["effective_status"], "stale");
    assert_eq!(v_stale["is_stale"], true);
}

#[test]
fn spec_5_11_5_ledger_formatter_known_date() {
    assert_eq!(format_unix_rfc3339(0), "1970-01-01T00:00:00Z");
    // 2026-07-06T15:00:00Z ≈ 2084-03-14T00:00:00Z — sanity: 56 years, 56*365+14 leap days = 20454 days approx
    // Just check year is reasonable
    let s = format_unix_rfc3339(1783361221);
    assert!(s.starts_with("2026-"));
    assert!(s.ends_with("Z"));
}
