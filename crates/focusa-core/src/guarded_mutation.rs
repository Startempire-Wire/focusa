//! Shared core chokepoint for every value-producing mutation (Spec 172
//! §11.4 "No direct-core bypass", §20.9 "Direct local/core, delayed worker,
//! stale-client, and dynamic-plugin bypasses fail closed").
//!
//! HTTP middleware, CLI, TUI, Pi, menubar, Focusa Desktop, Cockpit, workers,
//! schedulers, installers, and direct local clients all route protected
//! mutations through this single gate. The chokepoint applies the
//! product/type/family decisions BEFORE any HTTP or non-HTTP side effect and
//! reports a zero-side-effect counter on every denial, so an adapter can prove
//! that no partial reducer application, storage write, or worker enqueue
//! escaped the gate.
//!
//! No caller-controlled product, price, License Type, family, feature, limit,
//! node, or commercial right is accepted here: the gate resolves the exact
//! authority product id (`focusa`) from the signed entitlement only, and the
//! execution guard resolves operation class / capability family / required
//! feature from canonical operation metadata. Read, recovery, export, repair,
//! rollback, stable-security-update, and uninstall surfaces pass through the
//! execution guard alone (Spec 172 §5.3, §17) so customer data is never
//! trapped or deleted by a denial.

use crate::entitlement_execution_guard::{
    EntitlementExecutionContext, EntitlementExecutionDecision, EntitlementExecutionFailure,
    EntitlementExecutionPolicy, evaluate_entitlement_execution,
    evaluate_entitlement_execution_for_project,
};
use crate::limited_project::ActiveProjectSelection;
use crate::reducer::reduce;
use crate::types::{FocusaEvent, FocusaState};
use focusa_license::{
    BaseProductDecision, LicenseGuard, PolicyEntitlementState, authority_policy_state,
    resolve_base_focusa_product,
};
use serde::{Deserialize, Serialize};

/// Stable schema label for chokepoint outcomes and denials.
pub const GUARDED_MUTATION_SCHEMA: &str = "focusa.guarded_mutation.v1";
/// Approved outcome status.
pub const GUARDED_MUTATION_ALLOWED: &str = "GUARDED_MUTATION_ALLOWED";
/// Denied outcome status.
pub const GUARDED_MUTATION_DENIED: &str = "GUARDED_MUTATION_DENIED";
/// Stable code emitted when the reducer rejects the guarded mutation before
/// any persistence side effect.
pub const ENTITLEMENT_REDUCER_REJECTED: &str = "ENTITLEMENT_REDUCER_REJECTED";

/// Canonical denial emitted by the shared chokepoint. `side_effect_count` is
/// always zero for a denial: the protected mutation never reached the reducer,
/// the storage adapter, or a worker enqueue.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardedMutationDenial {
    pub schema: String,
    pub status: String,
    pub code: String,
    pub message: String,
    pub operation_id: String,
    pub base_product_decision: String,
    pub side_effect_count: u64,
    pub required_feature: Option<String>,
    pub limit_bucket: Option<String>,
}

impl GuardedMutationDenial {
    /// Exactly one bounded JSON line for logs and evidence; never carries raw
    /// email, key, token, customer, credential, or card data.
    pub fn to_bounded_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| {
            r#"{"schema":"focusa.guarded_mutation.v1","status":"GUARDED_MUTATION_DENIED","code":"ENTITLEMENT_POLICY_UNKNOWN","message":"unserializable denial","side_effect_count":0}"#
                .to_string()
        })
    }
}

/// Canonical outcome after the shared chokepoint approved and exactly one
/// protected mutation was applied. `side_effect_count` is exactly one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GuardedMutationOutcome {
    pub schema: String,
    pub status: String,
    pub operation_id: String,
    pub decision_code: String,
    pub base_product_decision: String,
    pub side_effect_count: u64,
    pub new_state_version: u64,
    pub emitted_event_count: usize,
}

fn denial_from_failure(
    failure: EntitlementExecutionFailure,
    operation_id: &str,
    base_product_decision: &str,
) -> GuardedMutationDenial {
    GuardedMutationDenial {
        schema: GUARDED_MUTATION_SCHEMA.to_string(),
        status: GUARDED_MUTATION_DENIED.to_string(),
        code: failure.code,
        message: failure.message,
        operation_id: operation_id.to_string(),
        base_product_decision: base_product_decision.to_string(),
        side_effect_count: 0,
        required_feature: failure.required_feature,
        limit_bucket: failure.limit_bucket,
    }
}

/// Resolve the canonical base Focusa product decision for the current guard.
/// The product id is always the literal authority id `focusa`; caller-supplied
/// product codes are never accepted at this boundary.
fn resolve_base_product(guard: &LicenseGuard) -> BaseProductDecision {
    let state = guard
        .entitlement
        .as_ref()
        .map(authority_policy_state)
        .unwrap_or(PolicyEntitlementState::MissingOrCorrupt);
    resolve_base_focusa_product("focusa", state)
}

/// A signed lease is current only when it is bound (lease id + sha256 digest)
/// and not past its expiry / offline-grace window. A stale or fabricated lease
/// must never produce value even when the policy state grid would allow it.
fn lease_is_current(guard: &LicenseGuard) -> bool {
    let now = chrono::Utc::now();
    guard.entitlement.as_ref().is_some_and(|snapshot| {
        let bound = snapshot
            .lease_id
            .as_deref()
            .is_some_and(|value| !value.is_empty())
            && snapshot
                .lease_digest
                .as_deref()
                .is_some_and(|value| value.starts_with("sha256:"));
        bound
            && match snapshot.state {
                focusa_license::authority::EntitlementState::Active => {
                    snapshot.expires_at.is_some_and(|expiry| expiry > now)
                }
                focusa_license::authority::EntitlementState::OfflineGrace => snapshot
                    .offline_grace_until
                    .is_some_and(|grace_until| grace_until > now),
                focusa_license::authority::EntitlementState::Unactivated
                | focusa_license::authority::EntitlementState::RecoveryOnly => false,
            }
    })
}

/// Apply the product/type/family decisions before any side effect for one
/// canonical operation.
///
/// - The execution guard resolves operation class, capability family, required
///   feature, and recovery allowances (the "type/family" decisions).
/// - For value-producing mutations the chokepoint additionally requires a
///   current bound signed lease and a base Focusa product decision that
///   permits base mutations ("product" decision). A stale, expired, revoked,
///   unbound, or wrong-product lease is denied here — no HTTP middleware is
///   required for this guarantee (Spec 172 §11.4).
///
/// Read/recovery/customer-control classes pass through the execution guard
/// alone so export, repair, rollback, stable security update, diagnostics, and
/// recovery stay reachable in blocked states.
pub fn guard_value_mutation(
    guard: &LicenseGuard,
    policy: &EntitlementExecutionPolicy,
    context: EntitlementExecutionContext,
) -> Result<EntitlementExecutionDecision, EntitlementExecutionFailure> {
    let decision = evaluate_entitlement_execution(guard, policy, context)?;
    apply_mutation_lease_gate(guard, policy)?;
    Ok(decision)
}

/// Project-aware variant of [`guard_value_mutation`].
///
/// Composes the execution guard's project guard
/// ([`evaluate_entitlement_execution_for_project`]) with the shared
/// product/type/family gate, so verified-no-license one-project mutations and
/// paid mutations both cross the same chokepoint.
pub fn guard_project_mutation(
    guard: &LicenseGuard,
    policy: &EntitlementExecutionPolicy,
    context: EntitlementExecutionContext,
    project_root: &str,
    active_selection: Option<&ActiveProjectSelection>,
) -> Result<EntitlementExecutionDecision, EntitlementExecutionFailure> {
    let decision = evaluate_entitlement_execution_for_project(
        guard,
        policy,
        context,
        project_root,
        active_selection,
    )?;
    apply_mutation_lease_gate(guard, policy)?;
    Ok(decision)
}

fn apply_mutation_lease_gate(
    guard: &LicenseGuard,
    policy: &EntitlementExecutionPolicy,
) -> Result<(), EntitlementExecutionFailure> {
    if policy.operation_class != focusa_license::OperationClass::ValueMutation {
        return Ok(());
    }
    if !lease_is_current(guard) {
        return Err(EntitlementExecutionFailure {
            code: "ENTITLEMENT_BASE_REQUIRED".to_string(),
            message:
                "a current signed Focusa authority lease is required before this value-producing mutation runs"
                    .to_string(),
            required_feature: policy.required_feature.clone(),
            limit_bucket: policy.limit_bucket.clone(),
        });
    }
    let base = resolve_base_product(guard);
    if !base.permits_base_mutations() {
        return Err(EntitlementExecutionFailure {
            code: "ENTITLEMENT_BASE_REQUIRED".to_string(),
            message: "base Focusa product gate not satisfied; no caller-controlled product may widen it"
                .to_string(),
            required_feature: policy.required_feature.clone(),
            limit_bucket: policy.limit_bucket.clone(),
        });
    }
    Ok(())
}

/// Evaluate the shared chokepoint for one canonical operation and, only on
/// approval, apply exactly one protected mutation through the single-writer
/// reducer. The returned outcome reports `side_effect_count == 1`; every
/// denial reports `side_effect_count == 0` and never reaches the reducer.
pub fn apply_guarded_mutation(
    guard: &LicenseGuard,
    policy: &EntitlementExecutionPolicy,
    context: EntitlementExecutionContext,
    state: FocusaState,
    event: FocusaEvent,
) -> Result<GuardedMutationOutcome, GuardedMutationDenial> {
    let base = resolve_base_product(guard);
    let decision = guard_value_mutation(guard, policy, context)
        .map_err(|failure| denial_from_failure(failure, &policy.operation_id, base.label()))?;
    match reduce(state, event) {
        Ok(result) => Ok(GuardedMutationOutcome {
            schema: GUARDED_MUTATION_SCHEMA.to_string(),
            status: GUARDED_MUTATION_ALLOWED.to_string(),
            operation_id: policy.operation_id.clone(),
            decision_code: decision.code,
            base_product_decision: base.label().to_string(),
            side_effect_count: 1,
            new_state_version: result.new_state.version,
            emitted_event_count: result.emitted_events.len(),
        }),
        Err(error) => Err(GuardedMutationDenial {
            schema: GUARDED_MUTATION_SCHEMA.to_string(),
            status: GUARDED_MUTATION_DENIED.to_string(),
            code: ENTITLEMENT_REDUCER_REJECTED.to_string(),
            message: format!(
                "reducer rejected the guarded mutation before any persistence side effect: {error}"
            ),
            operation_id: policy.operation_id.clone(),
            base_product_decision: base.label().to_string(),
            side_effect_count: 0,
            required_feature: None,
            limit_bucket: None,
        }),
    }
}

/// Project-aware variant of [`apply_guarded_mutation`] that additionally
/// composes the execution guard's project guard so verified-no-license
/// one-project mutations cross the same chokepoint with the same
/// zero-side-effect counters.
pub fn apply_guarded_project_mutation(
    guard: &LicenseGuard,
    policy: &EntitlementExecutionPolicy,
    context: EntitlementExecutionContext,
    project_root: &str,
    active_selection: Option<&ActiveProjectSelection>,
    state: FocusaState,
    event: FocusaEvent,
) -> Result<GuardedMutationOutcome, GuardedMutationDenial> {
    let base = resolve_base_product(guard);
    let decision = guard_project_mutation(guard, policy, context, project_root, active_selection)
        .map_err(|failure| denial_from_failure(failure, &policy.operation_id, base.label()))?;
    match reduce(state, event) {
        Ok(result) => Ok(GuardedMutationOutcome {
            schema: GUARDED_MUTATION_SCHEMA.to_string(),
            status: GUARDED_MUTATION_ALLOWED.to_string(),
            operation_id: policy.operation_id.clone(),
            decision_code: decision.code,
            base_product_decision: base.label().to_string(),
            side_effect_count: 1,
            new_state_version: result.new_state.version,
            emitted_event_count: result.emitted_events.len(),
        }),
        Err(error) => Err(GuardedMutationDenial {
            schema: GUARDED_MUTATION_SCHEMA.to_string(),
            status: GUARDED_MUTATION_DENIED.to_string(),
            code: ENTITLEMENT_REDUCER_REJECTED.to_string(),
            message: format!(
                "reducer rejected the guarded mutation before any persistence side effect: {error}"
            ),
            operation_id: policy.operation_id.clone(),
            base_product_decision: base.label().to_string(),
            side_effect_count: 0,
            required_feature: None,
            limit_bucket: None,
        }),
    }
}

/// Bounded durable-write ledger modeling the direct storage adapter contract.
///
/// A storage adapter MUST NOT accept a write from a direct caller unless the
/// shared chokepoint approved the operation first. `durable_writes` counts
/// only approved writes; every denied attempt reports `side_effect_count == 0`
/// and leaves the counter unchanged, proving no partial durable side effect
/// escaped the gate (Spec 152F denial atomicity, Spec 172 §20.9).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GuardedStorageLedger {
    durable_writes: u64,
}

impl GuardedStorageLedger {
    /// Durable writes recorded only after chokepoint approval.
    pub const fn durable_writes(&self) -> u64 {
        self.durable_writes
    }

    /// Attempt one protected durable write through the shared chokepoint. On
    /// approval the counter increments by exactly one; on denial it stays
    /// unchanged and the error reports `side_effect_count == 0`.
    pub fn guarded_write(
        &mut self,
        guard: &LicenseGuard,
        policy: &EntitlementExecutionPolicy,
        context: EntitlementExecutionContext,
    ) -> Result<u64, GuardedMutationDenial> {
        let base = resolve_base_product(guard);
        guard_value_mutation(guard, policy, context)
            .map_err(|failure| denial_from_failure(failure, &policy.operation_id, base.label()))?;
        self.durable_writes += 1;
        Ok(self.durable_writes)
    }
}
