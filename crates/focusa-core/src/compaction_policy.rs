//! Safe self-adaptive compaction policy controller — slice 1 (#112).
//!
//! Typed runtime facts, the capability/safety mask, the finite validated
//! policy lattice, and the immutable epoch lease. Design:
//! docs/163-safe-self-adaptive-compaction-policy-controller-design.md.
//!
//! Slice 1 delivers the deterministic core: facts collection, the pure
//! mask function, compiled lattice transitions, and sealed leases. Shadow
//! evaluation, promotion/quarantine state, and daemon wiring land in
//! slices 2+.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Runtime facts collected per epoch (turn-boundary snapshots only — no
/// continuous probing).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeFacts {
    pub provider: String,
    pub adapter: String,
    pub model: String,
    pub transport_posture: TransportPosture,
    pub task_phase: TaskPhase,
    pub growth_tokens_per_turn: u64,
    pub cache_hit_rate_bps: u32,
    pub cache_prefix_stable: bool,
    pub dynamic_slice_volatile: bool,
    pub bloatgaurd_intent: BloatgaurdIntent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransportPosture {
    WebsocketCached,
    Streaming,
    RetryBudget,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskPhase {
    Preload,
    ToolLoop,
    Review,
    Rollover,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BloatgaurdIntent {
    None,
    Diet,
    Firewall,
    CompactionPressure,
}

/// The policy lattice, conservative → aggressive. Transitions are compiled:
/// `validate_edge` proves each edge against the capability mask before any
/// selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Policy {
    None,
    WarnOnly,
    NativeLifecycle,
    ToolBoundaryCompaction,
    PromptRewrite,
    AsccPressureRoute,
}

impl Policy {
    pub fn rank(self) -> u8 {
        match self {
            Policy::None => 0,
            Policy::WarnOnly => 1,
            Policy::NativeLifecycle => 2,
            Policy::ToolBoundaryCompaction => 3,
            Policy::PromptRewrite => 4,
            Policy::AsccPressureRoute => 5,
        }
    }

    /// All policies (lattice order).
    pub const ALL: [Policy; 6] = [
        Policy::None,
        Policy::WarnOnly,
        Policy::NativeLifecycle,
        Policy::ToolBoundaryCompaction,
        Policy::PromptRewrite,
        Policy::AsccPressureRoute,
    ];
}

/// Capability and safety mask: which policies the runtime is legally
/// permitted to select, computed as a pure function of the facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CapabilityMask {
    pub permits_tool_boundary: bool,
    pub permits_native_lifecycle: bool,
    pub permits_prompt_rewrite: bool,
    pub permits_ascc: bool,
    /// Safety invariants hold regardless of policy; this digest identifies
    /// the mask so leases can detect drift.
    pub digest: String,
}

impl CapabilityMask {
    pub fn permits(&self, policy: Policy) -> bool {
        match policy {
            Policy::None | Policy::WarnOnly => true,
            Policy::NativeLifecycle => self.permits_native_lifecycle,
            Policy::ToolBoundaryCompaction => self.permits_tool_boundary,
            Policy::PromptRewrite => self.permits_prompt_rewrite,
            Policy::AsccPressureRoute => self.permits_ascc,
        }
    }
}

/// Pure mask function (docs/163 §2): facts never select a mutation policy
/// directly — they only establish what the runtime is CAPABLE of.
pub fn compute_mask(facts: &RuntimeFacts) -> CapabilityMask {
    let cache_healthy = facts.cache_hit_rate_bps >= 600 && facts.cache_prefix_stable;
    let mask = CapabilityMask {
        permits_tool_boundary: facts.transport_posture == TransportPosture::WebsocketCached
            && facts.task_phase != TaskPhase::Preload,
        permits_native_lifecycle: facts.transport_posture != TransportPosture::RetryBudget,
        permits_prompt_rewrite: cache_healthy && !facts.dynamic_slice_volatile,
        permits_ascc: facts.bloatgaurd_intent == BloatgaurdIntent::CompactionPressure
            && cache_healthy
            && facts.task_phase == TaskPhase::ToolLoop,
        digest: String::new(),
    };
    let mut mask = mask;
    mask.digest = digest_of(&serde_json::to_vec(&(
        mask.permits_tool_boundary,
        mask.permits_native_lifecycle,
        mask.permits_prompt_rewrite,
        mask.permits_ascc,
    ))
    .unwrap_or_default());
    mask
}

/// Compiled lattice edge validation (docs/163 §3): single-step transitions
/// only, and the target must be permitted by the mask.
pub fn validate_edge(from: Policy, to: Policy, mask: &CapabilityMask) -> bool {
    if !mask.permits(to) {
        return false;
    }
    to.rank() == from.rank() + 1
}

/// Immutable epoch policy lease (docs/163 §6). A lease is sealed once
/// created; drift between its facts digest and current facts forces a new
/// epoch rather than a mid-epoch override.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EpochLease {
    pub epoch_id: String,
    pub policy: Policy,
    pub facts_digest: String,
    pub mask_digest: String,
    pub selected_at: String,
    pub expires_at: String,
}

impl EpochLease {
    pub fn seal(
        epoch_id: impl Into<String>,
        policy: Policy,
        facts: &RuntimeFacts,
        mask: &CapabilityMask,
        selected_at: impl Into<String>,
        expires_at: impl Into<String>,
    ) -> Self {
        Self {
            epoch_id: epoch_id.into(),
            policy,
            facts_digest: digest_of(&serde_json::to_vec(facts).unwrap_or_default()),
            mask_digest: mask.digest.clone(),
            selected_at: selected_at.into(),
            expires_at: expires_at.into(),
        }
    }

    /// Drift check: current facts no longer match the lease's facts digest
    /// → a new epoch is required; the lease itself never mutates.
    pub fn facts_match(&self, facts: &RuntimeFacts) -> bool {
        self.facts_digest == digest_of(&serde_json::to_vec(facts).unwrap_or_default())
    }
}

fn digest_of(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Outcome metrics recorded per epoch (docs/163 §7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutcomeMetrics {
    pub latency_ms: u64,
    pub cache_hit_rate_bps: u32,
    pub token_growth: u64,
    pub error_count: u32,
    pub operator_interruptions: u32,
}

/// Shadow/off-policy evaluation (docs/163 §4): simulate the next-more-
/// aggressive policy against the same facts with zero side effects.
/// The simulation is a deterministic conservative model — each policy has
/// an expected effect vector; shadow results can never promote by
/// themselves.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShadowEvaluation {
    pub target_policy: Policy,
    pub simulated_outcome: OutcomeMetrics,
    pub evaluated_at: String,
}

impl Policy {
    /// Conservative expected effect of this policy on the active outcome,
    /// used by the shadow simulator. Never optimistic.
    fn expected_effect(self) -> (i64, i64, i64) {
        // (latency_delta_ms, cache_delta_bps, growth_delta_tokens)
        match self {
            Policy::None | Policy::WarnOnly => (0, 0, 0),
            Policy::NativeLifecycle => (200, 0, -2_000),
            Policy::ToolBoundaryCompaction => (150, 0, -3_500),
            Policy::PromptRewrite => (-100, -150, -4_000),
            Policy::AsccPressureRoute => (300, -50, -5_000),
        }
    }
}

pub fn evaluate_shadow(
    target: Policy,
    active: &OutcomeMetrics,
    evaluated_at: impl Into<String>,
) -> ShadowEvaluation {
    let (latency_delta, cache_delta, growth_delta) = target.expected_effect();
    ShadowEvaluation {
        target_policy: target,
        simulated_outcome: OutcomeMetrics {
            latency_ms: active.latency_ms.saturating_add_signed(latency_delta),
            cache_hit_rate_bps: active
                .cache_hit_rate_bps
                .saturating_add_signed(cache_delta as i32)
                .min(1_000),
            token_growth: active.token_growth.saturating_add_signed(growth_delta),
            error_count: active.error_count,
            operator_interruptions: active.operator_interruptions,
        },
        evaluated_at: evaluated_at.into(),
    }
}

/// Conservative promotion comparison (docs/163 §5): the shadowed outcome
/// must beat the active outcome on latency, growth, and error metrics
/// across the window — cache regressions disqualify.
pub fn shadow_beats_active(shadow: &OutcomeMetrics, active: &OutcomeMetrics) -> bool {
    shadow.latency_ms <= active.latency_ms
        && shadow.token_growth < active.token_growth
        && shadow.error_count <= active.error_count
        && shadow.operator_interruptions <= active.operator_interruptions
        && shadow.cache_hit_rate_bps >= active.cache_hit_rate_bps
}

/// Controller decision state (docs/163 §7): the active lease, a bounded
/// shadow history ring, and the quarantine set with expiry windows.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ControllerState {
    pub active_lease: Option<EpochLease>,
    pub shadow_history: Vec<ShadowEvaluation>,
    pub quarantine: Vec<QuarantineEntry>,
    pub epochs_seen: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuarantineEntry {
    pub policy: Policy,
    pub reason: String,
    pub until_epoch: u64,
}

impl ControllerState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn is_quarantined(&self, policy: Policy) -> bool {
        self.quarantine
            .iter()
            .any(|entry| entry.policy == policy && entry.until_epoch >= self.epochs_seen)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Transition {
    Promote,
    Retain,
    Quarantine,
    Rollback,
}

/// Deterministic epoch transition (docs/163 §7). One lattice step at a time;
/// a hard regression always rolls back; a soft regression quarantines.
pub fn next_transition(
    state: &ControllerState,
    active_outcome: &OutcomeMetrics,
    shadow: Option<&ShadowEvaluation>,
    promotion_window: usize,
    quarantine_epochs: u64,
) -> (Transition, Option<Policy>) {
    let active_policy = state.active_lease.as_ref().map(|lease| lease.policy);
    let Some(shadow) = shadow else {
        return (Transition::Retain, None);
    };
    let hard_regression = shadow.simulated_outcome.error_count > 0
        || shadow.simulated_outcome.operator_interruptions > active_outcome.operator_interruptions;
    if hard_regression {
        return (
            Transition::Rollback,
            active_policy.and_then(|policy| Policy::ALL.get(policy.rank().saturating_sub(1) as usize).copied()),
        );
    }
    let history = state.shadow_history.iter().rev().take(promotion_window);
    let wins = std::iter::once(shadow)
        .chain(history)
        .filter(|evaluation| shadow_beats_active(&evaluation.simulated_outcome, active_outcome))
        .count();
    let target = shadow.target_policy;
    if state.is_quarantined(target) {
        return (Transition::Retain, None);
    }
    let soft_regression = shadow.simulated_outcome.cache_hit_rate_bps < active_outcome.cache_hit_rate_bps
        || shadow.simulated_outcome.latency_ms > active_outcome.latency_ms;
    if soft_regression {
        return (
            Transition::Quarantine,
            Some(target),
        );
    }
    if wins >= promotion_window {
        (Transition::Promote, Some(target))
    } else {
        (Transition::Retain, None)
    }
}

/// Persist the controller state as JSON (slice 4-lite). The daemon keeps one
/// canonical file under its data dir; the full SQLite-backed ledger lands
/// with the event-ledger retention work (doc 158).
pub fn save_controller_state(state: &ControllerState, path: &std::path::Path) -> std::io::Result<()> {
    let serialized = serde_json::to_vec_pretty(state).map_err(std::io::Error::other)?;
    std::fs::write(path, serialized)
}

pub fn load_controller_state(path: &std::path::Path) -> ControllerState {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str(&raw).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts() -> RuntimeFacts {
        RuntimeFacts {
            provider: "openai-codex".into(),
            adapter: "pi".into(),
            model: "gpt-5.6-sol".into(),
            transport_posture: TransportPosture::WebsocketCached,
            task_phase: TaskPhase::ToolLoop,
            growth_tokens_per_turn: 4_000,
            cache_hit_rate_bps: 800,
            cache_prefix_stable: true,
            dynamic_slice_volatile: false,
            bloatgaurd_intent: BloatgaurdIntent::None,
        }
    }

    #[test]
    fn mask_permits_by_capability_not_intent() {
        let mask = compute_mask(&facts());
        assert!(mask.permits_tool_boundary);
        assert!(mask.permits_native_lifecycle);
        assert!(mask.permits_prompt_rewrite);
        assert!(!mask.permits_ascc, "ascc needs compaction-pressure intent");
    }

    #[test]
    fn preload_never_permits_tool_boundary() {
        let mut preload = facts();
        preload.task_phase = TaskPhase::Preload;
        let mask = compute_mask(&preload);
        assert!(!mask.permits_tool_boundary);
    }

    #[test]
    fn retry_budget_blocks_native_lifecycle() {
        let mut retried = facts();
        retried.transport_posture = TransportPosture::RetryBudget;
        let mask = compute_mask(&retried);
        assert!(!mask.permits_native_lifecycle);
    }

    #[test]
    fn volatile_slice_blocks_prompt_rewrite() {
        let mut volatile = facts();
        volatile.dynamic_slice_volatile = true;
        let mask = compute_mask(&volatile);
        assert!(!mask.permits_prompt_rewrite);
    }

    #[test]
    fn lattice_edges_are_single_step_and_permitted() {
        let mask = compute_mask(&facts());
        assert!(validate_edge(Policy::WarnOnly, Policy::NativeLifecycle, &mask));
        assert!(!validate_edge(Policy::None, Policy::NativeLifecycle, &mask), "two-step jump");
        assert!(!validate_edge(Policy::WarnOnly, Policy::AsccPressureRoute, &mask), "unpermitted target");
    }

    #[test]
    fn shadow_evaluation_is_conservative_and_side_effect_free() {
        let active = OutcomeMetrics {
            latency_ms: 1_000,
            cache_hit_rate_bps: 800,
            token_growth: 4_000,
            error_count: 0,
            operator_interruptions: 0,
        };
        let shadow = evaluate_shadow(Policy::ToolBoundaryCompaction, &active, "2026-08-15T00:00:00Z");
        assert_eq!(shadow.simulated_outcome.token_growth, 500);
        assert_eq!(shadow.simulated_outcome.latency_ms, 1_150);
        // Shadow never mutates the active outcome.
        assert_eq!(active.token_growth, 4_000);
    }

    #[test]
    fn promotion_requires_improvement_without_cache_regression() {
        let active = OutcomeMetrics {
            latency_ms: 1_000,
            cache_hit_rate_bps: 800,
            token_growth: 4_000,
            error_count: 0,
            operator_interruptions: 0,
        };
        let better = OutcomeMetrics {
            latency_ms: 950,
            cache_hit_rate_bps: 800,
            token_growth: 500,
            error_count: 0,
            operator_interruptions: 0,
        };
        assert!(shadow_beats_active(&better, &active));
        let mut cache_regressed = better.clone();
        cache_regressed.cache_hit_rate_bps = 500;
        assert!(!shadow_beats_active(&cache_regressed, &active));
    }

    fn active_metrics() -> OutcomeMetrics {
        OutcomeMetrics {
            latency_ms: 1_000,
            cache_hit_rate_bps: 800,
            token_growth: 4_000,
            error_count: 0,
            operator_interruptions: 0,
        }
    }

    #[test]
    fn hard_regression_rolls_back_one_step() {
        let mut state = ControllerState::new();
        state.epochs_seen = 3;
        let lease = EpochLease::seal(
            "e3",
            Policy::ToolBoundaryCompaction,
            &facts(),
            &compute_mask(&facts()),
            "2026-08-15T00:00:00Z",
            "2026-08-15T00:05:00Z",
        );
        state.active_lease = Some(lease);
        let mut shadow = evaluate_shadow(Policy::PromptRewrite, &active_metrics(), "t");
        shadow.simulated_outcome.error_count = 3;
        let (transition, target) = next_transition(&state, &active_metrics(), Some(&shadow), 5, 3);
        assert_eq!(transition, Transition::Rollback);
        assert_eq!(target, Some(Policy::NativeLifecycle));
    }

    #[test]
    fn cache_regression_quarantines_without_promotion() {
        let state = ControllerState::new();
        let mut shadow = evaluate_shadow(Policy::PromptRewrite, &active_metrics(), "t");
        shadow.simulated_outcome.cache_hit_rate_bps = 500;
        let (transition, target) = next_transition(&state, &active_metrics(), Some(&shadow), 5, 3);
        assert_eq!(transition, Transition::Quarantine);
        assert_eq!(target, Some(Policy::PromptRewrite));
    }

    #[test]
    fn promotion_requires_full_window_of_wins() {
        let state = ControllerState::new();
        // A shadow whose simulated outcome BEATS the active metrics, but
        // only once — the promotion window is 5.
        let mut shadow = evaluate_shadow(Policy::ToolBoundaryCompaction, &active_metrics(), "t");
        shadow.simulated_outcome.latency_ms = 900;
        shadow.simulated_outcome.token_growth = 500;
        let (transition, target) = next_transition(&state, &active_metrics(), Some(&shadow), 5, 3);
        assert_eq!(transition, Transition::Retain);
        assert_eq!(target, None);
    }

    #[test]
    fn quarantined_policy_never_promotes() {
        let mut state = ControllerState::new();
        state.epochs_seen = 10;
        state.quarantine.push(QuarantineEntry {
            policy: Policy::ToolBoundaryCompaction,
            reason: "cache regression".into(),
            until_epoch: 13,
        });
        let shadow = evaluate_shadow(Policy::ToolBoundaryCompaction, &active_metrics(), "t");
        let (transition, _) = next_transition(&state, &active_metrics(), Some(&shadow), 1, 3);
        assert_eq!(transition, Transition::Retain);
    }

    #[test]
    fn controller_state_round_trips_through_json() {
        let mut state = ControllerState::new();
        state.epochs_seen = 7;
        state.quarantine.push(QuarantineEntry {
            policy: Policy::PromptRewrite,
            reason: "cache regression".into(),
            until_epoch: 10,
        });
        let path = std::env::temp_dir().join(format!(
            "focusa-controller-state-{}-{}.json",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or_default()
        ));
        save_controller_state(&state, &path).unwrap();
        let loaded = load_controller_state(&path);
        assert_eq!(loaded.epochs_seen, 7);
        assert_eq!(loaded.quarantine.len(), 1);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn lease_detects_facts_drift() {
        let lease = EpochLease::seal(
            "epoch-1",
            Policy::ToolBoundaryCompaction,
            &facts(),
            &compute_mask(&facts()),
            "2026-08-15T00:00:00Z",
            "2026-08-15T00:05:00Z",
        );
        assert!(lease.facts_match(&facts()));
        let mut changed = facts();
        changed.growth_tokens_per_turn = 99_999;
        assert!(!lease.facts_match(&changed));
    }
}
