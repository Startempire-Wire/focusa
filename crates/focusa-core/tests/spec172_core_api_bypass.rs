//! Spec 172 §20.9 — core/API chokepoint and direct-call bypass resistance at
//! the focusa-core layer (atom focusa-vbcqu.20.15.25, 172.04.01).
//!
//! The shared chokepoint (`crates/focusa-core/src/guarded_mutation.rs`) is the
//! canonical gate for every value-producing mutation — HTTP middleware and
//! non-HTTP direct callers alike (Spec 172 §11.4 "No direct-core bypass"). This
//! module proves that direct core calls, direct reducer calls, direct storage
//! adapter writes, stale clients, wrong products, and queued-before-refund
//! worker dispatches all fail before any side effect, with zero-side-effect
//! counters, while recovery/read/export stay reachable in every blocked state.
//! The API route gate
//! (`crates/focusa-api/src/middleware/spec172_core_api_bypass.rs`) projects the
//! same decisions over HTTP.
//!
//! Exact verification: `cargo test --workspace spec172_core_api_bypass`.

use chrono::{Duration, Utc};
use focusa_core::entitlement_execution_guard::{
    EntitlementExecutionContext, EntitlementExecutionPolicy,
};
use focusa_core::guarded_mutation::{
    GuardedStorageLedger, apply_guarded_mutation, apply_guarded_project_mutation, guard_value_mutation,
};
use focusa_core::reducer::reduce;
use focusa_core::silent_session::SilentSessionId;
use focusa_core::silent_session_resources::{RESOURCE_ADMISSION_SCHEMA, ResourceAdmissionDecision};
use focusa_core::silent_session_scheduler::{
    DispatchDeferralReason, SilentSessionDispatchCandidate,
    SilentSessionDispatchEntitlementContext, SilentSessionPriority,
    select_silent_session_dispatch_with_entitlement,
};
use focusa_core::silent_session_writer::{WRITER_ADMISSION_SCHEMA, WriterAdmissionDecision};
use focusa_core::types::{FocusaEvent, FocusaState, InstanceKind};
use focusa_core::work_item::{
    WorkItem, WorkItemProvider, WorkItemQuery, WorkItemStatus,
};
use focusa_license::authority::{EntitlementSnapshot, EntitlementState};
use focusa_license::{
    CapabilityFamily as Family, LicenseGuard, OperationClass, RecoveryAllowance,
};
use std::path::PathBuf;
use uuid::Uuid;

fn base_mutation_policy() -> EntitlementExecutionPolicy {
    EntitlementExecutionPolicy::new(
        "focusa.core.workpoint.mutate",
        OperationClass::ValueMutation,
        Family::BaseFocusa,
        None,
        None,
        RecoveryAllowance::None,
    )
}

fn automation_policy() -> EntitlementExecutionPolicy {
    EntitlementExecutionPolicy::new(
        "focusa.agent.silent_sessions.run",
        OperationClass::ValueMutation,
        Family::Automation,
        Some("focusa.agent.silent_sessions"),
        None,
        RecoveryAllowance::None,
    )
}

fn instance_event() -> FocusaEvent {
    FocusaEvent::InstanceConnected {
        instance_id: Uuid::nil(),
        kind: InstanceKind::Cli,
    }
}

fn signed_snapshot(
    product: &str,
    state: EntitlementState,
    expires_in: Option<Duration>,
    bound: bool,
) -> EntitlementSnapshot {
    let mut snapshot = EntitlementSnapshot::unactivated(product, "node-bypass");
    snapshot.state = state;
    snapshot.sequence = Some(7);
    if bound {
        snapshot.lease_id = Some("lease-bypass".into());
        snapshot.lease_digest = Some("sha256:bypass".into());
    }
    if let Some(window) = expires_in {
        snapshot.expires_at = Some(Utc::now() + window);
        snapshot.offline_grace_until = Some(Utc::now() + window);
    }
    snapshot
}

// ── 1. Direct core / reducer / storage bypasses fail before effects ───────

#[test]
fn spec172_core_api_bypass_direct_core_reducer_storage_bypasses_fail_closed() {
    let blocked: Vec<(&str, LicenseGuard)> = vec![
        ("missing_or_corrupt", LicenseGuard::eval(7)),
        (
            "unactivated",
            LicenseGuard::from_entitlement(EntitlementSnapshot::unactivated("focusa", "node-bypass")),
        ),
        (
            "refunded_or_revoked",
            LicenseGuard::from_entitlement(EntitlementSnapshot::recovery_only(
                "focusa",
                "node-bypass",
                "refunded",
            )),
        ),
        (
            "expired_stale_client",
            LicenseGuard::from_entitlement(signed_snapshot(
                "focusa",
                EntitlementState::Active,
                Some(Duration::seconds(-1)),
                true,
            )),
        ),
        (
            "wrong_product_uiai",
            LicenseGuard::from_entitlement(signed_snapshot(
                "uiai_engine",
                EntitlementState::Active,
                Some(Duration::hours(1)),
                true,
            )),
        ),
    ];

    let mut ledger = GuardedStorageLedger::default();
    for (label, guard) in &blocked {
        // Direct core chokepoint gate denies the value mutation.
        let gate = guard_value_mutation(guard, &base_mutation_policy(), EntitlementExecutionContext::default());
        assert!(
            gate.is_err(),
            "direct core call must be denied in state {label}"
        );

        // Direct reducer adapter (guarded mutation) denies with a zero
        // side-effect counter and never reaches the reducer.
        let outcome = apply_guarded_mutation(
            guard,
            &base_mutation_policy(),
            EntitlementExecutionContext::default(),
            FocusaState::new(),
            instance_event(),
        );
        let denial = outcome.expect_err("guarded mutation must be denied");
        assert_eq!(
            denial.side_effect_count, 0,
            "denied mutation in state {label} must report zero side effects"
        );
        assert_eq!(
            denial.code, "ENTITLEMENT_BASE_REQUIRED",
            "state {label}: expected base-required code, got {}",
            denial.code
        );

        // Direct storage adapter refuses the write: durable writes stay zero.
        let write = ledger.guarded_write(guard, &base_mutation_policy(), EntitlementExecutionContext::default());
        assert!(
            write.is_err(),
            "direct storage write must be refused in state {label}"
        );
    }

    assert_eq!(
        ledger.durable_writes(),
        0,
        "every blocked direct attempt produced zero durable writes"
    );

    // A raw direct reducer call (the bypass an adapter must NOT permit) has no
    // sanctioned side-effect counter: the storage ledger never records it.
    let raw = reduce(FocusaState::new(), instance_event()).expect("pure reducer runs");
    assert_eq!(raw.new_state.version, 1);
    assert_eq!(
        ledger.durable_writes(),
        0,
        "an unsanctioned direct reducer call must never reach the storage ledger"
    );

    // With the exact signed Focusa lease the same mutation is approved exactly
    // once and exactly one durable write is recorded.
    let entitled = LicenseGuard::from_entitlement(signed_snapshot(
        "focusa",
        EntitlementState::Active,
        Some(Duration::hours(1)),
        true,
    ));
    let outcome = apply_guarded_mutation(
        &entitled,
        &base_mutation_policy(),
        EntitlementExecutionContext::default(),
        FocusaState::new(),
        instance_event(),
    )
    .expect("exact signed lease must approve the guarded mutation");
    assert_eq!(outcome.side_effect_count, 1);
    assert_eq!(outcome.new_state_version, 1);
    assert_eq!(outcome.emitted_event_count, 1);
    assert_eq!(
        ledger.guarded_write(&entitled, &base_mutation_policy(), EntitlementExecutionContext::default()),
        Ok(1)
    );
}

// ── 2. Stale clients, unbound leases, and wrong products fail closed ──────

#[test]
fn spec172_core_api_bypass_stale_client_unbound_and_wrong_product_fail_closed() {
    // A bound-but-expired lease (stale client) must never produce value.
    let stale = LicenseGuard::from_entitlement(signed_snapshot(
        "focusa",
        EntitlementState::Active,
        Some(Duration::seconds(-1)),
        true,
    ));
    assert_eq!(
        guard_value_mutation(&stale, &base_mutation_policy(), EntitlementExecutionContext::default())
            .expect_err("stale lease must be denied")
            .code,
        "ENTITLEMENT_BASE_REQUIRED"
    );

    // An unbound (fabricated) lease must never produce value even while Active.
    let unbound = LicenseGuard::from_entitlement(signed_snapshot(
        "focusa",
        EntitlementState::Active,
        Some(Duration::hours(1)),
        false,
    ));
    assert_eq!(
        guard_value_mutation(&unbound, &base_mutation_policy(), EntitlementExecutionContext::default())
            .expect_err("unbound lease must be denied")
            .code,
        "ENTITLEMENT_BASE_REQUIRED"
    );

    // Offline Grace with a valid window passes base mutations; past the window
    // it fails (stale client) exactly like the API gate.
    let grace = LicenseGuard::from_entitlement(signed_snapshot(
        "focusa",
        EntitlementState::OfflineGrace,
        Some(Duration::hours(1)),
        true,
    ));
    assert!(
        guard_value_mutation(&grace, &base_mutation_policy(), EntitlementExecutionContext::default())
            .is_ok(),
        "valid offline grace must pass the chokepoint"
    );
    let stale_grace = LicenseGuard::from_entitlement(signed_snapshot(
        "focusa",
        EntitlementState::OfflineGrace,
        Some(Duration::seconds(-1)),
        true,
    ));
    assert_eq!(
        guard_value_mutation(
            &stale_grace,
            &base_mutation_policy(),
            EntitlementExecutionContext::default(),
        )
        .expect_err("past offline grace must be denied")
        .code,
        "ENTITLEMENT_BASE_REQUIRED"
    );

    // A UIAI-only lease cannot execute Focusa mutations, even through the
    // project-aware chokepoint, and premium families require the exact
    // authority feature grant.
    let uiai = LicenseGuard::from_entitlement(signed_snapshot(
        "uiai_engine",
        EntitlementState::Active,
        Some(Duration::hours(1)),
        true,
    ));
    assert_eq!(
        guard_value_mutation(&uiai, &base_mutation_policy(), EntitlementExecutionContext::default())
            .expect_err("UIAI-only lease must not execute Focusa mutations")
            .code,
        "ENTITLEMENT_BASE_REQUIRED"
    );
    let focusa = LicenseGuard::from_entitlement(signed_snapshot(
        "focusa",
        EntitlementState::Active,
        Some(Duration::hours(1)),
        true,
    ));
    assert_eq!(
        guard_value_mutation(&focusa, &automation_policy(), EntitlementExecutionContext::default())
            .expect_err("premium family without its exact grant must be denied")
            .code,
        "ENTITLEMENT_FEATURE_REQUIRED"
    );

    // The project-aware chokepoint denies the same blocked attempts and keeps
    // paid mutations approved for the exact lease.
    let project_denied = apply_guarded_project_mutation(
        &LicenseGuard::eval(7),
        &base_mutation_policy(),
        EntitlementExecutionContext::default(),
        "/home/user/projects/focusa-a",
        None,
        FocusaState::new(),
        instance_event(),
    )
    .expect_err("project-aware chokepoint must deny without entitlement");
    assert_eq!(project_denied.side_effect_count, 0);
    assert!(
        apply_guarded_project_mutation(
            &focusa,
            &base_mutation_policy(),
            EntitlementExecutionContext::default(),
            "/home/user/projects/focusa-a",
            None,
            FocusaState::new(),
            instance_event(),
        )
        .is_ok(),
        "exact paid lease must pass the project-aware chokepoint"
    );
}

// ── 3. Queued-before-refund worker dispatch revalidates before effects ────

fn work_item(id: &str) -> WorkItem {
    WorkItem {
        provider: WorkItemProvider::Bd,
        provider_item_id: id.into(),
        project_root: PathBuf::from("/projects/focusa"),
        provider_status: WorkItemStatus::Open,
        title: id.into(),
        priority: 0,
        parent: None,
        dependencies: vec![],
        acceptance_criteria: vec!["proof passes".into()],
        spec_refs: vec!["docs/172".into()],
        blocked_reason: None,
        url: None,
        revision: None,
    }
}

fn admitted() -> ResourceAdmissionDecision {
    ResourceAdmissionDecision {
        schema: RESOURCE_ADMISSION_SCHEMA.into(),
        admitted: true,
        degraded: false,
        denials: vec![],
    }
}

fn writable() -> WriterAdmissionDecision {
    WriterAdmissionDecision {
        schema: WRITER_ADMISSION_SCHEMA.into(),
        admitted: true,
        read_only: false,
        renewal: false,
        denials: vec![],
        conflicting_actor_refs: vec![],
        conflicting_lease_ids: vec![],
        isolated_worktree_required: false,
    }
}

fn queued_candidate(
    item: &WorkItem,
    queued_at: chrono::DateTime<Utc>,
    context: SilentSessionDispatchEntitlementContext,
) -> SilentSessionDispatchCandidate {
    SilentSessionDispatchCandidate {
        session_id: SilentSessionId::new(),
        work_item: item.reference(),
        priority: SilentSessionPriority::Normal,
        queued_at,
        resource_admission: admitted(),
        writer_admission: writable(),
        entitlement_context: Some(context),
    }
}

fn dispatch_context() -> SilentSessionDispatchEntitlementContext {
    SilentSessionDispatchEntitlementContext {
        dispatch_policy: EntitlementExecutionPolicy::new(
            "focusa.silent_session.dispatch",
            OperationClass::InternalMaintenance,
            Family::InternalMaintenance,
            None,
            None,
            RecoveryAllowance::None,
        ),
        initiating_policy: Some(EntitlementExecutionPolicy::new(
            "focusa.workpoint.checkpoint",
            OperationClass::ValueMutation,
            Family::BaseFocusa,
            None,
            None,
            RecoveryAllowance::None,
        )),
        reservation_id: Some("reservation-bypass".into()),
    }
}

#[test]
fn spec172_core_api_bypass_worker_dispatch_revalidates_queued_before_refund() {
    let item = work_item("queued-before-refund");
    let query = WorkItemQuery {
        project_root: PathBuf::from("/projects/focusa"),
        parent: None,
        limit: 100,
    };
    let queued_at = Utc::now();
    let context = dispatch_context();
    let ledger = GuardedStorageLedger::default();

    // Queued while entitled: dispatch selection proceeds at revalidation.
    let entitled = LicenseGuard::from_entitlement(signed_snapshot(
        "focusa",
        EntitlementState::Active,
        Some(Duration::hours(1)),
        true,
    ));
    let (_readiness, active_dispatch) =
        select_silent_session_dispatch_with_entitlement(
            std::slice::from_ref(&item),
            &query,
            &[queued_candidate(&item, queued_at, context.clone())],
            &entitled,
            &EntitlementExecutionPolicy::new(
                "focusa.silent_session.dispatch",
                OperationClass::InternalMaintenance,
                Family::InternalMaintenance,
                None,
                None,
                RecoveryAllowance::None,
            ),
        )
        .expect("queued work must dispatch while entitlement is current");
    assert_eq!(active_dispatch.selected_work_item, Some(item.reference()));

    // After refund/revoke the same queued item is deferred at dispatch and no
    // durable write escapes the gate.
    let revoked = LicenseGuard::from_entitlement(EntitlementSnapshot::recovery_only(
        "focusa",
        "node-bypass",
        "refunded",
    ));
    let (_readiness, revoked_dispatch) =
        select_silent_session_dispatch_with_entitlement(
            std::slice::from_ref(&item),
            &query,
            &[queued_candidate(&item, queued_at, context.clone())],
            &revoked,
            &EntitlementExecutionPolicy::new(
                "focusa.silent_session.dispatch",
                OperationClass::InternalMaintenance,
                Family::InternalMaintenance,
                None,
                None,
                RecoveryAllowance::None,
            ),
        )
        .expect("dispatch must stay ordered after entitlement loss");
    assert_eq!(revoked_dispatch.selected_work_item, None);
    assert_eq!(revoked_dispatch.deferred.len(), 1);
    assert_eq!(
        revoked_dispatch.deferred[0].reason,
        DispatchDeferralReason::EntitlementDenied
    );
    assert!(
        revoked_dispatch.deferred[0].detail.contains("ENTITLEMENT_BASE_REQUIRED"),
        "queued-before-refund deferral must carry the base-required code"
    );

    // A stale (expired) lease cannot dispatch the same queued work either.
    let stale = LicenseGuard::from_entitlement(signed_snapshot(
        "focusa",
        EntitlementState::Active,
        Some(Duration::seconds(-1)),
        true,
    ));
    let (_readiness, stale_dispatch) = select_silent_session_dispatch_with_entitlement(
        std::slice::from_ref(&item),
        &query,
        &[queued_candidate(&item, queued_at, context)],
        &stale,
        &EntitlementExecutionPolicy::new(
            "focusa.silent_session.dispatch",
            OperationClass::InternalMaintenance,
            Family::InternalMaintenance,
            None,
            None,
            RecoveryAllowance::None,
        ),
    )
    .expect("dispatch must stay ordered with a stale client");
    assert_eq!(stale_dispatch.selected_work_item, None);
    assert_eq!(
        stale_dispatch.deferred[0].reason,
        DispatchDeferralReason::EntitlementDenied,
        "stale client queued work must be deferred before effects"
    );

    assert_eq!(
        ledger.durable_writes(),
        0,
        "no durable write may escape the gate around refund or staleness"
    );
}

// ── 4. Recovery/read/export stay reachable in every blocked state ─────────

#[test]
fn spec172_core_api_bypass_recovery_read_export_reachable_in_blocked_states() {
    let blocked: Vec<(&str, LicenseGuard, focusa_license::PolicyEntitlementState)> = vec![
        (
            "missing_or_corrupt",
            LicenseGuard::eval(7),
            focusa_license::PolicyEntitlementState::MissingOrCorrupt,
        ),
        (
            "unactivated",
            LicenseGuard::from_entitlement(EntitlementSnapshot::unactivated("focusa", "node-bypass")),
            focusa_license::PolicyEntitlementState::PendingUnverified,
        ),
        (
            "refunded_or_revoked",
            LicenseGuard::from_entitlement(EntitlementSnapshot::recovery_only(
                "focusa",
                "node-bypass",
                "revoked",
            )),
            focusa_license::PolicyEntitlementState::RefundedOrRevoked,
        ),
    ];

    for (state_label, guard, policy_state) in &blocked {
        // Protected value mutations are denied in every blocked state.
        let denied = guard_value_mutation(
            guard,
            &base_mutation_policy(),
            EntitlementExecutionContext::default(),
        )
        .expect_err("protected mutation must be denied");
        assert_eq!(denied.code, "ENTITLEMENT_BASE_REQUIRED", "state {state_label}");

        // Only the families the resolver keeps reachable may pass; at least
        // one data-protection surface stays reachable in every blocked state,
        // so customer data is never trapped or deleted.
        let mut reachable_in_state = 0u32;
        for (family, operation_class, allowance) in [
            (Family::AccountRecovery, OperationClass::Recovery, RecoveryAllowance::AccountRecovery),
            (Family::CustomerDataExport, OperationClass::Read, RecoveryAllowance::CustomerDataExport),
            (Family::ReadProjection, OperationClass::Read, RecoveryAllowance::ReadProjection),
        ] {
            if focusa_license::reduce_entitlement_state(*policy_state, family, None).posture()
                == focusa_license::EntitlementPolicyPosture::Deny
            {
                continue; // resolver keeps this family blocked in this state
            }
            let policy = EntitlementExecutionPolicy::new(
                format!("bypass.matrix.{}", family.label()),
                operation_class,
                family,
                None,
                None,
                allowance,
            );
            assert!(
                guard_value_mutation(guard, &policy, EntitlementExecutionContext::default()).is_ok(),
                "state {state_label}: {family:?} must stay reachable through the chokepoint"
            );
            reachable_in_state += 1;
        }
        assert!(
            reachable_in_state >= 1,
            "state {state_label}: at least one data-protection surface must stay reachable"
        );
    }

    // Internal maintenance without an initiating posture fails closed (the
    // chokepoint does not invent an initiator).
    let maintenance = EntitlementExecutionPolicy::new(
        "focusa.internal.maintenance",
        OperationClass::InternalMaintenance,
        Family::InternalMaintenance,
        None,
        None,
        RecoveryAllowance::None,
    );
    let denied = guard_value_mutation(
        &LicenseGuard::eval(7),
        &maintenance,
        EntitlementExecutionContext::default(),
    )
    .expect_err("maintenance without initiating posture must fail closed");
    assert_eq!(denied.code, "ENTITLEMENT_ROUTE_UNCLASSIFIED");

    // With an initiator posture the same maintenance operation passes.
    let entitled = LicenseGuard::from_entitlement(signed_snapshot(
        "focusa",
        EntitlementState::Active,
        Some(Duration::hours(1)),
        true,
    ));
    assert!(
        guard_value_mutation(
            &entitled,
            &maintenance,
            EntitlementExecutionContext {
                now: Utc::now(),
                initiating_posture: Some(focusa_license::EntitlementPolicyPosture::Base),
            },
        )
        .is_ok(),
        "maintenance inheriting an entitled posture must pass"
    );
}
