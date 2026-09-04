//! Shared bounded read-response helpers for large/read-heavy API surfaces.
//!
//! These helpers make truncation explicit without changing canonical state.
//! Route handlers own domain-specific selection/rehydration; this module owns
//! consistent limit resolution and metadata envelopes.
//!
//! # Runtime classification (Spec104 BND-01)
//!
//! Resource mode / pressure runtime state is explicitly keyed by a typed
//! Host `ScopeRef`. It never supplies project/workstream authority and has no
//! unkeyed mutable fallback.

use chrono::Utc;
use focusa_core::scoped_state::ScopeRef;
use serde::Serialize;
use serde_json::{Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::{LazyLock, Mutex};
use uuid::Uuid;

static TEST_PRESSURE_THRESHOLD_KB: LazyLock<Mutex<Option<u64>>> =
    LazyLock::new(|| Mutex::new(None));
static RUNTIME_RESOURCE_MODE_OVERRIDE: LazyLock<Mutex<BTreeMap<String, Option<String>>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));
static RESOURCE_MODE_LAST_OBSERVED: LazyLock<Mutex<BTreeMap<String, Option<String>>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));
static RESOURCE_MODE_TRANSITIONS: LazyLock<
    Mutex<BTreeMap<String, Vec<ResourceModeTransitionRecord>>>,
> = LazyLock::new(|| Mutex::new(BTreeMap::new()));
static RESOURCE_MODE_TRANSITION_OMITTED: LazyLock<Mutex<BTreeMap<String, usize>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));
static RESOURCE_MODE_HYSTERESIS_STATE: LazyLock<
    Mutex<BTreeMap<String, ResourceModeHysteresisRuntime>>,
> = LazyLock::new(|| Mutex::new(BTreeMap::new()));
const RESOURCE_MODE_TRANSITION_RING_LIMIT: usize = 32;

#[derive(Debug, Clone)]
struct ResourceModeHysteresisRuntime {
    current_mode: &'static str,
    recovery_candidate: Option<&'static str>,
    recovery_count: usize,
}

impl Default for ResourceModeHysteresisRuntime {
    fn default() -> Self {
        Self {
            current_mode: "normal",
            recovery_candidate: None,
            recovery_count: 0,
        }
    }
}

fn host_runtime_scope_key() -> String {
    ScopeRef::host(
        "host:focusa-daemon-runtime",
        "/",
        "focusa-daemon-runtime",
        "sha256:focusa-daemon-runtime",
    )
    .expect("static host runtime scope is valid")
    .storage_key()
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LowMemBudget {
    pub rss_soft_mb: u64,
    pub rss_hard_mb: u64,
    pub hot_route_timeout_ms: u64,
    pub warm_route_timeout_ms: u64,
    pub cold_route_timeout_ms: u64,
    pub hot_payload_bytes: usize,
    pub max_items_default: usize,
    pub max_items_hard: usize,
    pub max_rehydrate_refs: usize,
    pub background_concurrency: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ResourceModeStatus {
    pub mode: &'static str,
    pub forced: bool,
    pub reason: &'static str,
    pub rss_kb: Option<u64>,
    pub peak_rss_kb: Option<u64>,
    pub host_mem_available_kb: Option<u64>,
    pub pressure: PressureStatus,
    pub budget: LowMemBudget,
    pub tool_availability_policy: &'static str,
    pub pruning_order: Vec<&'static str>,
    pub cold_surfaces_deferred: Vec<&'static str>,
    pub hysteresis: serde_json::Value,
    pub latest_transition: Option<ResourceModeTransitionRecord>,
    pub transition_omitted_count: usize,
    pub retention_policy: LowMemRetentionPolicy,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct LowMemRetentionPolicy {
    pub retain_order: Vec<&'static str>,
    pub evict_first: Vec<&'static str>,
    pub trace_event_limit: usize,
    pub raw_log_limit: usize,
    pub full_payload_cache_limit: usize,
    pub evidence_handle_policy: &'static str,
    pub safety_policy: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ResourceModeTransitionRecord {
    pub transition_id: String,
    pub observed_at: String,
    pub from_mode: String,
    pub to_mode: String,
    pub reason: String,
    pub trigger: String,
    pub active_session_id: Option<String>,
    pub rss_kb: Option<u64>,
    pub peak_rss_kb: Option<u64>,
    pub host_mem_available_kb: Option<u64>,
    pub budget: LowMemBudget,
    pub hysteresis_state: serde_json::Value,
    pub durability: &'static str,
    pub recovery_hint: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RehydrateHint {
    pub mode: &'static str,
    pub parameter: &'static str,
    pub value: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PressureStatus {
    pub status: &'static str,
    pub active: bool,
    pub configured: bool,
    pub rss_kb: Option<u64>,
    pub threshold_kb: Option<u64>,
    pub mode: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct PressureTransition {
    pub transitioned_at: String,
    pub from_status: &'static str,
    pub to_status: &'static str,
    pub rss_kb: Option<u64>,
    pub threshold_kb: Option<u64>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ResponseSizeHistogram {
    pub route: String,
    pub samples: usize,
    pub min_bytes: usize,
    pub p50_bytes: usize,
    pub p95_bytes: usize,
    pub max_bytes: usize,
}

type ResponseSizeSamples = BTreeMap<String, BTreeMap<String, Vec<usize>>>;

static PRESSURE_TRANSITION: LazyLock<Mutex<BTreeMap<String, Option<PressureTransition>>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));
static PRESSURE_LAST_ACTIVE: LazyLock<Mutex<BTreeMap<String, Option<bool>>>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));
static RESPONSE_SIZE_SAMPLES: LazyLock<Mutex<ResponseSizeSamples>> =
    LazyLock::new(|| Mutex::new(BTreeMap::new()));

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CursorWindow {
    pub offset: usize,
    pub limit: usize,
    pub returned: usize,
    pub total: usize,
    pub next_cursor: Option<String>,
    pub previous_cursor: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct FieldProjection {
    pub requested: Vec<String>,
    pub applied: Vec<String>,
    pub omitted: Vec<String>,
    pub allowed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct TraversalBounds {
    pub requested_path: Option<String>,
    pub path_segments: Vec<String>,
    pub omitted_path_segments: usize,
    pub requested_depth: Option<usize>,
    pub depth: usize,
    pub max_depth: usize,
    pub max_path_segments: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct BoundedReadMetadata {
    pub total: usize,
    pub returned: usize,
    pub omitted: usize,
    pub truncated: bool,
    pub more_available: bool,
    pub pagination_hint: String,
    pub limit: usize,
    pub requested_limit: Option<usize>,
    pub default_limit: usize,
    pub full_limit: usize,
    pub include_full_payload: bool,
    pub summary_only: bool,
    pub cursor: Option<String>,
    pub next_cursor: Option<String>,
    pub rehydrate: Option<RehydrateHint>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundedReadOptions {
    pub requested_limit: Option<usize>,
    pub include_full_payload: bool,
    pub summary_only: bool,
    pub cursor: Option<String>,
    pub next_cursor: Option<String>,
    pub default_limit: usize,
    pub full_limit: usize,
}

impl BoundedReadOptions {
    pub fn resolved_limit(&self) -> usize {
        let default_limit = self.default_limit.max(1);
        let full_limit = self.full_limit.max(default_limit);
        let ceiling = if self.include_full_payload {
            full_limit
        } else {
            default_limit
        };
        self.requested_limit.unwrap_or(ceiling).clamp(1, ceiling)
    }
}

pub fn env_limit(name: &str, fallback: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(fallback)
        .max(1)
}

pub fn lowmem_caps_active() -> bool {
    matches!(
        resource_mode_status().mode,
        "constrained" | "lowmem" | "emergency"
    )
}

pub fn budgeted_default_limit(name: &str, normal_fallback: usize) -> usize {
    let configured = env_limit(name, normal_fallback);
    if lowmem_caps_active() {
        configured.min(lowmem_budget().max_items_default).max(1)
    } else {
        configured
    }
}

pub fn budgeted_hard_limit(name: &str, normal_fallback: usize, default_floor: usize) -> usize {
    let configured = env_limit(name, normal_fallback);
    let hard = if lowmem_caps_active() {
        configured.min(lowmem_budget().max_items_hard)
    } else {
        configured
    };
    hard.max(default_floor.max(1))
}

pub fn budgeted_requested_limit(
    requested_limit: Option<usize>,
    default_limit: usize,
    hard_limit: usize,
) -> usize {
    requested_limit
        .unwrap_or(default_limit)
        .clamp(1, hard_limit.max(default_limit.max(1)))
}

fn telemetry_trace_retention_limit_for_mode(mode: &str, budget: &LowMemBudget) -> usize {
    match mode {
        "emergency" => budget.max_items_hard.max(1),
        "lowmem" => budget
            .max_items_hard
            .saturating_mul(2)
            .max(budget.max_items_default),
        "constrained" => budget
            .max_items_hard
            .saturating_mul(4)
            .max(budget.max_items_default),
        _ => env_limit("FOCUSA_TELEMETRY_TRACE_RETENTION_LIMIT", 5000),
    }
}

pub fn telemetry_trace_retention_limit() -> usize {
    let status = resource_mode_status();
    telemetry_trace_retention_limit_for_mode(status.mode, &status.budget)
}

fn lowmem_retention_policy_for_mode(mode: &str, budget: &LowMemBudget) -> LowMemRetentionPolicy {
    let trace_event_limit = telemetry_trace_retention_limit_for_mode(mode, budget);
    LowMemRetentionPolicy {
        retain_order: vec![
            "liveness",
            "workpoint",
            "trajectory",
            "project_identity",
            "safety_scope",
            "evidence_handles",
            "active_object_refs",
            "surgical_context",
            "learning_risk_top_k",
            "diagnostics_history",
        ],
        evict_first: vec![
            "raw_telemetry_trace_events",
            "replay_bundles",
            "full_payload_caches",
            "snapshot_bodies",
            "full_ontology_graphs",
            "full_lineage_trees",
        ],
        trace_event_limit,
        raw_log_limit: trace_event_limit,
        full_payload_cache_limit: budget.max_rehydrate_refs.max(1),
        evidence_handle_policy: "retain_handles_before_raw_payloads",
        safety_policy: "never_prune_constraints_decisions_failures_or_approval_state_silently",
    }
}

pub fn lowmem_retention_policy() -> LowMemRetentionPolicy {
    let status = resource_mode_status();
    lowmem_retention_policy_for_mode(status.mode, &status.budget)
}

fn parse_status_value_kb(status_text: &str, key: &str) -> Option<u64> {
    status_text.lines().find_map(|line| {
        let (line_key, rest) = line.split_once(':')?;
        if line_key != key {
            return None;
        }
        rest.split_whitespace().next()?.parse::<u64>().ok()
    })
}

pub fn current_rss_kb() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|text| parse_status_value_kb(&text, "VmRSS"))
}

pub fn peak_rss_kb() -> Option<u64> {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|text| parse_status_value_kb(&text, "VmHWM"))
}

pub fn host_mem_available_kb() -> Option<u64> {
    std::fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|text| parse_status_value_kb(&text, "MemAvailable"))
}

fn env_u64_optional(name: &str) -> Option<u64> {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
}

fn env_u64(name: &str, fallback: u64) -> u64 {
    env_u64_optional(name).unwrap_or(fallback)
}

fn resolve_lowmem_rss_budget(
    soft_mb: Option<u64>,
    hard_mb: Option<u64>,
    legacy_hard_mb: Option<u64>,
) -> (u64, u64) {
    let hard_mb = hard_mb.or(legacy_hard_mb).unwrap_or(1000);
    let soft_mb = soft_mb.unwrap_or(700);
    let soft_mb = if hard_mb > 0 && soft_mb > hard_mb {
        hard_mb
    } else {
        soft_mb
    };
    (soft_mb, hard_mb)
}

pub fn env_usize(name: &str, fallback: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(fallback)
        .max(1)
}

pub fn lowmem_budget() -> LowMemBudget {
    let (rss_soft_mb, rss_hard_mb) = resolve_lowmem_rss_budget(
        env_u64_optional("FOCUSA_LOWMEM_RSS_SOFT_MB"),
        env_u64_optional("FOCUSA_LOWMEM_RSS_HARD_MB"),
        env_u64_optional("FOCUSA_MEMORY_BUDGET_MB"),
    );
    LowMemBudget {
        rss_soft_mb,
        rss_hard_mb,
        hot_route_timeout_ms: env_u64("FOCUSA_LOWMEM_HOT_TIMEOUT_MS", 1500),
        warm_route_timeout_ms: env_u64("FOCUSA_LOWMEM_WARM_TIMEOUT_MS", 1000),
        cold_route_timeout_ms: env_u64("FOCUSA_LOWMEM_COLD_TIMEOUT_MS", 3000),
        hot_payload_bytes: env_usize("FOCUSA_LOWMEM_HOT_PAYLOAD_BYTES", 32768),
        max_items_default: env_usize("FOCUSA_LOWMEM_DEFAULT_LIMIT", 10),
        max_items_hard: env_usize("FOCUSA_LOWMEM_HARD_LIMIT", 50),
        max_rehydrate_refs: env_usize("FOCUSA_LOWMEM_MAX_REHYDRATE_REFS", 8),
        background_concurrency: std::env::var("FOCUSA_LOWMEM_BACKGROUND_CONCURRENCY")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(0)
            .min(1),
    }
}

fn normalize_resource_mode_value(value: &str) -> Option<String> {
    let normalized = value.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "normal" | "constrained" | "lowmem" | "emergency" => Some(normalized),
        "auto" | "clear" | "deactivate_lowmem" => None,
        _ => Some("lowmem".to_string()),
    }
}

pub fn set_runtime_resource_mode_override(mode: Option<&str>) -> Result<Option<String>, String> {
    let normalized = mode.and_then(normalize_resource_mode_value);
    if let Ok(mut slots) = RUNTIME_RESOURCE_MODE_OVERRIDE.lock() {
        slots.insert(host_runtime_scope_key(), normalized.clone());
        Ok(normalized)
    } else {
        Err("resource mode override lock poisoned".to_string())
    }
}

pub fn runtime_resource_mode_override() -> Option<String> {
    RUNTIME_RESOURCE_MODE_OVERRIDE
        .lock()
        .ok()
        .and_then(|slots| slots.get(&host_runtime_scope_key()).cloned().flatten())
}

fn resource_mode_recovery_hint(mode: &str) -> &'static str {
    match mode {
        "emergency" => {
            "hot core only; defer cold routes and reduce background work until RSS drops"
        }
        "lowmem" => "use summary tools, small limits, and explicit rehydrate refs only when needed",
        "constrained" => {
            "prefer top-k summaries and avoid deep diagnostics unless explicitly requested"
        }
        _ => "continue normally; monitor ResourceMode transitions",
    }
}

fn transition_snapshot() -> (Option<ResourceModeTransitionRecord>, usize) {
    let scope_key = host_runtime_scope_key();
    let latest = RESOURCE_MODE_TRANSITIONS.lock().ok().and_then(|records| {
        records
            .get(&scope_key)
            .and_then(|records| records.last().cloned())
    });
    let omitted = RESOURCE_MODE_TRANSITION_OMITTED
        .lock()
        .map(|values| values.get(&scope_key).copied().unwrap_or_default())
        .unwrap_or_default();
    (latest, omitted)
}

fn push_resource_mode_transition(record: ResourceModeTransitionRecord) {
    let scope_key = host_runtime_scope_key();
    if let Ok(mut scoped_records) = RESOURCE_MODE_TRANSITIONS.lock() {
        let records = scoped_records.entry(scope_key.clone()).or_default();
        records.push(record);
        if records.len() > RESOURCE_MODE_TRANSITION_RING_LIMIT {
            let overflow = records.len() - RESOURCE_MODE_TRANSITION_RING_LIMIT;
            records.drain(0..overflow);
            if let Ok(mut omitted) = RESOURCE_MODE_TRANSITION_OMITTED.lock() {
                *omitted.entry(scope_key).or_default() += overflow;
            }
        }
    }
}

pub fn resource_mode_transition_records(limit: usize) -> Vec<ResourceModeTransitionRecord> {
    let limit = limit.clamp(1, RESOURCE_MODE_TRANSITION_RING_LIMIT);
    RESOURCE_MODE_TRANSITIONS
        .lock()
        .map(|records| {
            records
                .get(&host_runtime_scope_key())
                .into_iter()
                .flat_map(|records| records.iter().rev().take(limit).cloned())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn resource_mode_severity(mode: &str) -> u8 {
    match mode {
        "emergency" => 3,
        "lowmem" => 2,
        "constrained" => 1,
        _ => 0,
    }
}

fn resource_mode_hysteresis_recovery_samples() -> usize {
    env_usize("FOCUSA_RESOURCE_MODE_HYSTERESIS_RECOVERY_SAMPLES", 3)
}

fn apply_resource_mode_hysteresis(
    raw_mode: &'static str,
    raw_reason: &'static str,
    forced: bool,
) -> (&'static str, &'static str, serde_json::Value) {
    if forced {
        return (
            raw_mode,
            raw_reason,
            serde_json::json!({
                "status": "hysteresis_v1",
                "policy": "bypassed_forced_override",
                "raw_mode": raw_mode,
                "effective_mode": raw_mode,
                "recovery_samples": resource_mode_hysteresis_recovery_samples(),
                "recovery_count": 0,
            }),
        );
    }

    let recovery_samples = resource_mode_hysteresis_recovery_samples();
    let Ok(mut states) = RESOURCE_MODE_HYSTERESIS_STATE.lock() else {
        return (
            raw_mode,
            raw_reason,
            serde_json::json!({
                "status": "hysteresis_unavailable",
                "policy": "raw_mode_fallback",
                "raw_mode": raw_mode,
                "effective_mode": raw_mode,
            }),
        );
    };
    let state = states.entry(host_runtime_scope_key()).or_default();

    let previous_mode = state.current_mode;
    let previous_severity = resource_mode_severity(previous_mode);
    let raw_severity = resource_mode_severity(raw_mode);
    let mut action = "stable";
    let mut reason = raw_reason;

    if raw_severity > previous_severity {
        state.current_mode = raw_mode;
        state.recovery_candidate = None;
        state.recovery_count = 0;
        action = "immediate_escalation";
    } else if raw_severity < previous_severity {
        if state.recovery_candidate == Some(raw_mode) {
            state.recovery_count = state.recovery_count.saturating_add(1);
        } else {
            state.recovery_candidate = Some(raw_mode);
            state.recovery_count = 1;
        }
        if state.recovery_count >= recovery_samples {
            state.current_mode = raw_mode;
            state.recovery_candidate = None;
            state.recovery_count = 0;
            action = "delayed_recovery_applied";
        } else {
            action = "delayed_recovery_hold";
            reason = "hysteresis_recovery_hold";
        }
    } else {
        state.recovery_candidate = None;
        state.recovery_count = 0;
    }

    let effective_mode = state.current_mode;
    let recovery_candidate = state.recovery_candidate;
    let recovery_count = state.recovery_count;
    (
        effective_mode,
        reason,
        serde_json::json!({
            "status": "hysteresis_v1",
            "policy": "immediate_escalation_delayed_recovery",
            "raw_mode": raw_mode,
            "effective_mode": effective_mode,
            "previous_mode": previous_mode,
            "action": action,
            "recovery_candidate": recovery_candidate,
            "recovery_count": recovery_count,
            "recovery_samples": recovery_samples,
        }),
    )
}

pub fn observe_resource_mode_transition(
    trigger: &str,
    active_session_id: Option<String>,
) -> ResourceModeStatus {
    let mut status = resource_mode_status();
    let scope_key = host_runtime_scope_key();
    let previous_mode = RESOURCE_MODE_LAST_OBSERVED
        .lock()
        .ok()
        .and_then(|slots| slots.get(&scope_key).cloned().flatten());
    let should_record = match previous_mode.as_deref() {
        Some(previous) => previous != status.mode,
        None => status.mode != "normal" || trigger != "background_resource_monitor",
    };

    if should_record {
        push_resource_mode_transition(ResourceModeTransitionRecord {
            transition_id: Uuid::now_v7().to_string(),
            observed_at: Utc::now().to_rfc3339(),
            from_mode: previous_mode.unwrap_or_else(|| "unknown".to_string()),
            to_mode: status.mode.to_string(),
            reason: status.reason.to_string(),
            trigger: trigger.to_string(),
            active_session_id,
            rss_kb: status.rss_kb,
            peak_rss_kb: status.peak_rss_kb,
            host_mem_available_kb: status.host_mem_available_kb,
            budget: status.budget.clone(),
            hysteresis_state: status.hysteresis.clone(),
            durability: "pending",
            recovery_hint: resource_mode_recovery_hint(status.mode),
        });
        let (latest_transition, transition_omitted_count) = transition_snapshot();
        status.latest_transition = latest_transition;
        status.transition_omitted_count = transition_omitted_count;
    }

    if let Ok(mut slots) = RESOURCE_MODE_LAST_OBSERVED.lock() {
        slots.insert(scope_key, Some(status.mode.to_string()));
    }

    status
}

fn normalized_resource_mode_override() -> Option<(String, &'static str)> {
    if let Some(mode) = runtime_resource_mode_override() {
        return Some((mode, "runtime_override"));
    }
    std::env::var("FOCUSA_RESOURCE_MODE")
        .ok()
        .and_then(|value| normalize_resource_mode_value(&value))
        .map(|mode| (mode, "env_override"))
}

pub fn resource_mode_status() -> ResourceModeStatus {
    let budget = lowmem_budget();
    let rss_kb = current_rss_kb();
    let peak_rss_kb = peak_rss_kb();
    let host_mem_available_kb = host_mem_available_kb();
    let pressure = pressure_status();
    let forced_mode = normalized_resource_mode_override();
    let forced = forced_mode.is_some();

    let (raw_mode, raw_reason) = match forced_mode
        .as_ref()
        .map(|(mode, source)| (mode.as_str(), *source))
    {
        Some(("emergency", source)) => ("emergency", source),
        Some(("lowmem", source)) => ("lowmem", source),
        Some(("constrained", source)) => ("constrained", source),
        Some(("normal", source)) => ("normal", source),
        Some((_, _)) => ("lowmem", "invalid_override_safe_lowmem"),
        None => {
            let rss_mb = rss_kb.unwrap_or(0) / 1024;
            let lowmem_available_floor_mb = env_u64("FOCUSA_LOWMEM_MEM_AVAILABLE_MB", 0);
            if budget.rss_hard_mb > 0 && rss_mb >= budget.rss_hard_mb {
                ("emergency", "rss_hard_exceeded")
            } else if budget.rss_soft_mb > 0 && rss_mb >= budget.rss_soft_mb {
                ("lowmem", "rss_soft_exceeded")
            } else if lowmem_available_floor_mb > 0
                && host_mem_available_kb
                    .map(|kb| kb / 1024 <= lowmem_available_floor_mb)
                    .unwrap_or(false)
            {
                ("lowmem", "host_mem_available_floor")
            } else if pressure.active {
                ("constrained", "memory_pressure_threshold")
            } else {
                ("normal", "within_budget")
            }
        }
    };
    let (mode, reason, hysteresis) = apply_resource_mode_hysteresis(raw_mode, raw_reason, forced);

    let (latest_transition, transition_omitted_count) = transition_snapshot();
    let retention_policy = lowmem_retention_policy_for_mode(mode, &budget);

    ResourceModeStatus {
        mode,
        forced,
        reason,
        rss_kb,
        peak_rss_kb,
        host_mem_available_kb,
        pressure,
        budget,
        tool_availability_policy: "all_tools_callable_with_bounded_or_degraded_envelopes",
        pruning_order: vec![
            "liveness",
            "continuation",
            "safety_scope",
            "evidence_handles",
            "surgical_context",
            "learning_risk_top_k",
            "diagnostics_history",
        ],
        cold_surfaces_deferred: vec![
            "full_lineage_tree",
            "full_ontology_graph",
            "deep_work_loop_status",
            "replay_bundles",
            "full_telemetry_logs",
            "snapshot_bodies",
        ],
        hysteresis,
        latest_transition,
        transition_omitted_count,
        retention_policy,
    }
}

pub fn set_test_pressure_threshold(threshold_kb: Option<u64>) {
    if let Ok(mut threshold) = TEST_PRESSURE_THRESHOLD_KB.lock() {
        *threshold = threshold_kb;
    }
}

pub fn pressure_status() -> PressureStatus {
    let threshold_kb = TEST_PRESSURE_THRESHOLD_KB
        .lock()
        .ok()
        .and_then(|threshold| *threshold)
        .or_else(|| {
            std::env::var("FOCUSA_MEMORY_PRESSURE_RSS_KB")
                .ok()
                .and_then(|value| value.parse::<u64>().ok())
                .filter(|value| *value > 0)
        });
    let rss_kb = current_rss_kb();
    let active = threshold_kb
        .zip(rss_kb)
        .map(|(threshold, rss)| rss >= threshold)
        .unwrap_or(false);
    let status = PressureStatus {
        status: if active { "pressure" } else { "ok" },
        active,
        configured: threshold_kb.is_some(),
        rss_kb,
        threshold_kb,
        mode: if active {
            "summary_only_by_default"
        } else {
            "normal"
        },
    };
    let scope_key = host_runtime_scope_key();
    if let Ok(mut last_active_by_scope) = PRESSURE_LAST_ACTIVE.lock() {
        let last_active = last_active_by_scope
            .entry(scope_key.clone())
            .or_insert(None);
        let should_record = match *last_active {
            Some(previous) => previous != active,
            None => active && threshold_kb.is_some(),
        };
        if should_record {
            let from_status = last_active
                .map(|previous| if previous { "pressure" } else { "ok" })
                .unwrap_or("unknown");
            if let Ok(mut transitions) = PRESSURE_TRANSITION.lock() {
                transitions.insert(
                    scope_key,
                    Some(PressureTransition {
                        transitioned_at: Utc::now().to_rfc3339(),
                        from_status,
                        to_status: status.status,
                        rss_kb,
                        threshold_kb,
                    }),
                );
            }
        }
        *last_active = Some(active);
    }
    status
}

pub fn last_pressure_transition() -> Option<PressureTransition> {
    PRESSURE_TRANSITION
        .lock()
        .ok()
        .and_then(|values| values.get(&host_runtime_scope_key()).cloned().flatten())
}

pub fn record_response_size(route: &str, bytes: usize) {
    if let Ok(mut samples_by_scope) = RESPONSE_SIZE_SAMPLES.lock() {
        let samples = samples_by_scope
            .entry(host_runtime_scope_key())
            .or_default();
        let route_samples = samples.entry(route.to_string()).or_default();
        route_samples.push(bytes);
        if route_samples.len() > 512 {
            let overflow = route_samples.len() - 512;
            route_samples.drain(0..overflow);
        }
    }
}

pub fn response_size_histograms() -> Vec<ResponseSizeHistogram> {
    RESPONSE_SIZE_SAMPLES
        .lock()
        .map(|samples_by_scope| {
            samples_by_scope
                .get(&host_runtime_scope_key())
                .into_iter()
                .flat_map(|samples| samples.iter())
                .filter_map(|(route, values)| {
                    if values.is_empty() {
                        return None;
                    }
                    let mut sorted = values.clone();
                    sorted.sort_unstable();
                    let p50 = sorted[sorted.len() / 2];
                    let p95 = sorted[((sorted.len() * 95).div_ceil(100)).saturating_sub(1)];
                    Some(ResponseSizeHistogram {
                        route: route.clone(),
                        samples: sorted.len(),
                        min_bytes: *sorted.first().unwrap_or(&0),
                        p50_bytes: p50,
                        p95_bytes: p95,
                        max_bytes: *sorted.last().unwrap_or(&0),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

pub fn record_json_response_size(route: &str, value: &serde_json::Value) {
    if let Ok(bytes) = serde_json::to_vec(value) {
        record_response_size(route, bytes.len());
    }
}

pub fn full_payload_blocked_by_pressure(
    include_full_payload: bool,
    force_full_payload: bool,
) -> bool {
    if !include_full_payload || force_full_payload {
        return false;
    }
    let status = resource_mode_status();
    status.pressure.active || matches!(status.mode, "lowmem" | "emergency")
}

pub fn parse_cursor_offset(cursor: Option<&str>) -> usize {
    cursor
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(0)
}

pub fn cursor_window(total: usize, cursor: Option<&str>, limit: usize) -> CursorWindow {
    let limit = limit.max(1);
    let offset = parse_cursor_offset(cursor).min(total);
    let remaining = total.saturating_sub(offset);
    let returned = remaining.min(limit);
    let next_offset = offset.saturating_add(returned);
    CursorWindow {
        offset,
        limit,
        returned,
        total,
        next_cursor: (next_offset < total).then(|| next_offset.to_string()),
        previous_cursor: (offset > 0).then(|| offset.saturating_sub(limit).to_string()),
    }
}

pub fn bounded_window<T: Clone>(
    items: &[T],
    cursor: Option<&str>,
    limit: usize,
) -> (Vec<T>, CursorWindow) {
    let window = cursor_window(items.len(), cursor, limit);
    let values = items
        .iter()
        .skip(window.offset)
        .take(window.returned)
        .cloned()
        .collect::<Vec<_>>();
    (values, window)
}

fn split_fields(fields: Option<&str>) -> Vec<String> {
    fields
        .unwrap_or_default()
        .split(',')
        .map(|field| field.trim().to_ascii_lowercase())
        .filter(|field| !field.is_empty())
        .collect()
}

pub fn field_projection(
    fields: Option<&str>,
    default_fields: &[&str],
    allowed_fields: &[&str],
) -> FieldProjection {
    let allowed = allowed_fields
        .iter()
        .map(|field| field.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let allowed_set = allowed.iter().cloned().collect::<BTreeSet<_>>();
    let requested = split_fields(fields);
    let effective = if requested.is_empty() {
        default_fields
            .iter()
            .map(|field| field.to_ascii_lowercase())
            .collect::<Vec<_>>()
    } else if requested.iter().any(|field| field == "*") {
        allowed.clone()
    } else {
        requested.clone()
    };
    let mut seen = BTreeSet::new();
    let applied = effective
        .iter()
        .filter(|field| allowed_set.contains(*field))
        .filter(|field| seen.insert((*field).clone()))
        .cloned()
        .collect::<Vec<_>>();
    let omitted = effective
        .iter()
        .filter(|field| !allowed_set.contains(*field))
        .cloned()
        .collect::<Vec<_>>();
    FieldProjection {
        requested,
        applied,
        omitted,
        allowed,
    }
}

pub fn project_json_fields(value: &Value, projection: &FieldProjection) -> Value {
    let Some(object) = value.as_object() else {
        return value.clone();
    };
    let mut out = Map::new();
    for field in &projection.applied {
        if let Some(value) = object.get(field) {
            out.insert(field.clone(), value.clone());
        }
    }
    Value::Object(out)
}

pub fn traversal_bounds(
    path: Option<&str>,
    requested_depth: Option<usize>,
    max_depth: usize,
    max_path_segments: usize,
) -> TraversalBounds {
    let max_depth = max_depth.max(1);
    let max_path_segments = max_path_segments.max(1);
    let all_segments = path
        .unwrap_or_default()
        .split('/')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(ToString::to_string)
        .collect::<Vec<_>>();
    let path_segments = all_segments
        .iter()
        .take(max_path_segments)
        .cloned()
        .collect::<Vec<_>>();
    TraversalBounds {
        requested_path: path.map(ToString::to_string),
        path_segments,
        omitted_path_segments: all_segments.len().saturating_sub(max_path_segments),
        requested_depth,
        depth: requested_depth.unwrap_or(max_depth).clamp(1, max_depth),
        max_depth,
        max_path_segments,
    }
}

pub fn bounded_metadata(
    total: usize,
    returned: usize,
    options: BoundedReadOptions,
) -> BoundedReadMetadata {
    let limit = options.resolved_limit();
    let omitted = total.saturating_sub(returned);
    let truncated = omitted > 0;
    let more_available = options.next_cursor.is_some();
    let pagination_hint = if more_available {
        format!(
            "more items remain beyond this window (next_cursor={}); pass cursor=<value> to fetch the next page",
            options.next_cursor.as_deref().unwrap_or("?")
        )
    } else {
        "no more items; window covers the full result set".to_string()
    };
    BoundedReadMetadata {
        total,
        returned,
        omitted,
        truncated,
        more_available,
        pagination_hint,
        limit,
        requested_limit: options.requested_limit,
        default_limit: options.default_limit.max(1),
        full_limit: options.full_limit.max(options.default_limit.max(1)),
        include_full_payload: options.include_full_payload,
        summary_only: options.summary_only,
        cursor: options.cursor,
        next_cursor: options.next_cursor,
        rehydrate: truncated.then_some(RehydrateHint {
            mode: "full_payload_opt_in",
            parameter: "include_full_payload",
            value: "true",
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static TEST_RESOURCE_MODE_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

    #[cfg(test)]
    fn reset_resource_mode_transitions_for_test() {
        let scope_key = host_runtime_scope_key();
        if let Ok(mut slots) = RESOURCE_MODE_LAST_OBSERVED.lock() {
            slots.remove(&scope_key);
        }
        if let Ok(mut records) = RESOURCE_MODE_TRANSITIONS.lock() {
            records.remove(&scope_key);
        }
        if let Ok(mut omitted) = RESOURCE_MODE_TRANSITION_OMITTED.lock() {
            omitted.remove(&scope_key);
        }
        if let Ok(mut hysteresis) = RESOURCE_MODE_HYSTERESIS_STATE.lock() {
            hysteresis.remove(&scope_key);
        }
    }

    #[test]
    fn rss_budget_has_one_canonical_resolution_with_legacy_fallback() {
        assert_eq!(resolve_lowmem_rss_budget(None, None, None), (700, 1000));
        assert_eq!(
            resolve_lowmem_rss_budget(None, None, Some(4096)),
            (700, 4096)
        );
        assert_eq!(
            resolve_lowmem_rss_budget(Some(600), Some(900), Some(4096)),
            (600, 900),
            "canonical low-memory keys must outrank the legacy alias"
        );
        assert_eq!(
            resolve_lowmem_rss_budget(None, None, Some(1)),
            (1, 1),
            "the default soft limit must not exceed a legacy hard limit"
        );
    }

    #[test]
    fn resource_mode_hysteresis_delays_recovery_not_escalation() {
        let _guard = TEST_RESOURCE_MODE_LOCK
            .lock()
            .expect("resource mode test lock");
        reset_resource_mode_transitions_for_test();
        let (mode, reason, state) =
            apply_resource_mode_hysteresis("lowmem", "rss_soft_exceeded", false);
        assert_eq!(mode, "lowmem");
        assert_eq!(reason, "rss_soft_exceeded");
        assert_eq!(state["action"], "immediate_escalation");
        let (mode, reason, state) =
            apply_resource_mode_hysteresis("normal", "within_budget", false);
        if state["recovery_samples"].as_u64().unwrap_or(3) > 1 {
            assert_eq!(mode, "lowmem");
            assert_eq!(reason, "hysteresis_recovery_hold");
            assert_eq!(state["action"], "delayed_recovery_hold");
        }
        reset_resource_mode_transitions_for_test();
    }

    #[test]
    fn background_resource_monitor_records_transition_without_active_session() {
        let _guard = TEST_RESOURCE_MODE_LOCK
            .lock()
            .expect("resource mode test lock");
        reset_resource_mode_transitions_for_test();
        set_runtime_resource_mode_override(Some("lowmem")).expect("set override");
        let status = observe_resource_mode_transition("background_resource_monitor", None);
        set_runtime_resource_mode_override(None).expect("clear override");
        assert_eq!(status.mode, "lowmem");
        let transition = status.latest_transition.expect("transition record");
        assert_eq!(transition.from_mode, "unknown");
        assert_eq!(transition.to_mode, "lowmem");
        assert_eq!(transition.trigger, "background_resource_monitor");
        assert_eq!(transition.active_session_id, None);
        assert_eq!(transition.durability, "pending");
        assert_eq!(
            transition.budget.max_items_default,
            lowmem_budget().max_items_default
        );
    }

    #[test]
    fn budgeted_limits_derive_from_lowmem_budget() {
        let _guard = TEST_RESOURCE_MODE_LOCK
            .lock()
            .expect("resource mode test lock");
        set_runtime_resource_mode_override(Some("lowmem")).expect("set override");
        let default_limit = budgeted_default_limit("FOCUSA_TEST_UNSET_DEFAULT", 100);
        let hard_limit = budgeted_hard_limit("FOCUSA_TEST_UNSET_HARD", 1000, default_limit);
        set_runtime_resource_mode_override(None).expect("clear override");
        assert_eq!(default_limit, lowmem_budget().max_items_default);
        assert_eq!(
            hard_limit,
            lowmem_budget().max_items_hard.max(default_limit)
        );
        assert_eq!(
            budgeted_requested_limit(Some(500), default_limit, hard_limit),
            hard_limit
        );
    }

    #[test]
    fn lowmem_retention_policy_evicts_raw_history_before_core_context() {
        let _guard = TEST_RESOURCE_MODE_LOCK
            .lock()
            .expect("resource mode test lock");
        set_runtime_resource_mode_override(Some("emergency")).expect("set override");
        let policy = lowmem_retention_policy();
        set_runtime_resource_mode_override(None).expect("clear override");
        assert_eq!(policy.retain_order.first(), Some(&"liveness"));
        assert!(policy.retain_order.contains(&"workpoint"));
        assert!(policy.retain_order.contains(&"safety_scope"));
        assert_eq!(
            policy.evict_first.first(),
            Some(&"raw_telemetry_trace_events")
        );
        assert!(policy.trace_event_limit <= lowmem_budget().max_items_hard.max(1));
        assert_eq!(
            policy.evidence_handle_policy,
            "retain_handles_before_raw_payloads"
        );
    }

    #[test]
    fn resolves_default_limit_without_full_payload() {
        let options = BoundedReadOptions {
            requested_limit: Some(500),
            include_full_payload: false,
            summary_only: true,
            cursor: None,
            next_cursor: None,
            default_limit: 100,
            full_limit: 1000,
        };
        assert_eq!(options.resolved_limit(), 100);
    }

    #[test]
    fn resolves_full_limit_with_explicit_opt_in() {
        let options = BoundedReadOptions {
            requested_limit: Some(500),
            include_full_payload: true,
            summary_only: false,
            cursor: None,
            next_cursor: None,
            default_limit: 100,
            full_limit: 1000,
        };
        assert_eq!(options.resolved_limit(), 500);
    }

    #[test]
    fn pressure_status_is_safe_without_threshold() {
        let status = pressure_status();
        assert!(matches!(status.status, "ok" | "pressure"));
        assert!(matches!(status.mode, "normal" | "summary_only_by_default"));
    }

    #[test]
    fn resource_mode_forced_lowmem_reports_all_tools_callable() {
        let _guard = TEST_RESOURCE_MODE_LOCK
            .lock()
            .expect("resource mode test lock");
        set_runtime_resource_mode_override(Some("lowmem")).expect("set override");
        let status = resource_mode_status();
        set_runtime_resource_mode_override(None).expect("clear override");
        assert_eq!(status.mode, "lowmem");
        assert!(status.forced);
        assert_eq!(status.reason, "runtime_override");
        assert_eq!(
            status.tool_availability_policy,
            "all_tools_callable_with_bounded_or_degraded_envelopes"
        );
        assert!(status.cold_surfaces_deferred.contains(&"full_lineage_tree"));
    }

    #[test]
    fn full_payload_pressure_block_respects_force_flag() {
        // The active pressure bit is environment/runtime dependent; this still proves
        // that non-full requests are never blocked and forced requests are never blocked.
        assert!(!full_payload_blocked_by_pressure(false, false));
        assert!(!full_payload_blocked_by_pressure(true, true));
    }

    #[test]
    fn forced_lowmem_blocks_full_payload_without_force_override() {
        let _guard = TEST_RESOURCE_MODE_LOCK
            .lock()
            .expect("resource mode test lock");
        set_runtime_resource_mode_override(Some("lowmem")).expect("set override");
        assert!(full_payload_blocked_by_pressure(true, false));
        assert!(!full_payload_blocked_by_pressure(true, true));
        assert!(!full_payload_blocked_by_pressure(false, false));
        set_runtime_resource_mode_override(Some("emergency")).expect("set override");
        assert!(full_payload_blocked_by_pressure(true, false));
        set_runtime_resource_mode_override(None).expect("clear override");
    }

    #[test]
    fn cursor_window_returns_next_cursor_and_previous_cursor() {
        let window = cursor_window(10, Some("3"), 4);
        assert_eq!(window.offset, 3);
        assert_eq!(window.returned, 4);
        assert_eq!(window.next_cursor.as_deref(), Some("7"));
        assert_eq!(window.previous_cursor.as_deref(), Some("0"));
    }

    #[test]
    fn field_projection_filters_to_allowed_fields() {
        let projection =
            field_projection(Some("id,label,unknown"), &["id"], &["id", "label", "score"]);
        assert_eq!(projection.applied, vec!["id", "label"]);
        assert_eq!(projection.omitted, vec!["unknown"]);
        let value = serde_json::json!({"id":"x","label":"ok","score":10,"other":true});
        let projected = project_json_fields(&value, &projection);
        assert_eq!(projected, serde_json::json!({"id":"x","label":"ok"}));
    }

    #[test]
    fn traversal_bounds_caps_depth_and_path_segments() {
        let bounds = traversal_bounds(Some("/a/b/c/d"), Some(99), 3, 2);
        assert_eq!(bounds.path_segments, vec!["a", "b"]);
        assert_eq!(bounds.omitted_path_segments, 2);
        assert_eq!(bounds.depth, 3);
    }

    #[test]
    fn metadata_makes_truncation_and_rehydrate_path_explicit() {
        let metadata = bounded_metadata(
            10,
            3,
            BoundedReadOptions {
                requested_limit: Some(3),
                include_full_payload: false,
                summary_only: true,
                cursor: Some("0".to_string()),
                next_cursor: Some("3".to_string()),
                default_limit: 5,
                full_limit: 50,
            },
        );
        assert_eq!(metadata.total, 10);
        assert_eq!(metadata.returned, 3);
        assert_eq!(metadata.omitted, 7);
        assert!(metadata.truncated);
        assert_eq!(
            metadata.rehydrate.unwrap().parameter,
            "include_full_payload"
        );
    }
}
