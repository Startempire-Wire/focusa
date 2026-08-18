//! Spec 152F.06.05 — offline grace, authority outage, and bypass resistance
//! matrix (focusa-core execution layer).
//!
//! Proves the canonical chokepoints fail closed during an authority outage and
//! against direct bypass attempts:
//!
//! 1. worker/scheduler dispatch revalidates authority at dispatch — a worker
//!    cannot run a silent session when the base entitlement is missing or
//!    refunded/revoked, and a closed Offline Grace window stops premium
//!    dispatch while recovery surfaces stay available;
//! 2. the core execution guard denies direct value-producing mutations under
//!    outage/refund/revoke and only permits them for a usable signed
//!    entitlement (Active paid or valid Offline Grace);
//! 3. recovery, read, export, and maintenance allowances remain available in
//!    every blocked state.
//!
//! Exact verification: `cargo test --workspace spec152f_bypass_resistance`.

use std::path::PathBuf;

use chrono::{Duration, Utc};
use focusa_core::license::{
    EntitlementExecutionContext, EntitlementExecutionFailure, EntitlementExecutionPolicy,
    evaluate_entitlement_execution,
};
use focusa_core::silent_session::SilentSessionId;
use focusa_core::silent_session_resources::{
    AdmissionDenial, RESOURCE_ADMISSION_SCHEMA, ResourceAdmissionDecision,
};
use focusa_core::silent_session_scheduler::{
    DispatchDeferralReason, SilentSessionDispatchCandidate,
    SilentSessionDispatchEntitlementContext, SilentSessionPriority,
    select_silent_session_dispatch_with_default_entitlement,
    select_silent_session_dispatch_with_entitlement,
};
use focusa_core::silent_session_writer::{
    WRITER_ADMISSION_SCHEMA, WriterAdmissionDecision, WriterAdmissionDenial,
};
use focusa_core::work_item::{WorkItem, WorkItemProvider, WorkItemQuery, WorkItemStatus};
use focusa_license::authority::{EntitlementSnapshot, EntitlementState};
use focusa_license::{CapabilityFamily, LicenseGuard, OperationClass, RecoveryAllowance};

fn now() -> chrono::DateTime<Utc> {
    Utc::now()
}

fn item(id: &str) -> WorkItem {
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
        spec_refs: vec!["docs/133".into()],
        blocked_reason: None,
        url: None,
        revision: None,
    }
}

fn query() -> WorkItemQuery {
    WorkItemQuery {
        project_root: PathBuf::from("/projects/focusa"),
        parent: None,
        limit: 100,
    }
}

fn resource(admitted: bool) -> ResourceAdmissionDecision {
    ResourceAdmissionDecision {
        schema: RESOURCE_ADMISSION_SCHEMA.into(),
        admitted,
        degraded: false,
        denials: if admitted {
            vec![]
        } else {
            vec![AdmissionDenial::GlobalQuota]
        },
    }
}

fn writer(admitted: bool) -> WriterAdmissionDecision {
    WriterAdmissionDecision {
        schema: WRITER_ADMISSION_SCHEMA.into(),
        admitted,
        read_only: false,
        renewal: false,
        denials: if admitted {
            vec![]
        } else {
            vec![WriterAdmissionDenial::WorkspaceConflict]
        },
        conflicting_actor_refs: vec![],
        conflicting_lease_ids: vec![],
        isolated_worktree_required: !admitted,
    }
}

fn candidate(first: &WorkItem) -> SilentSessionDispatchCandidate {
    SilentSessionDispatchCandidate {
        session_id: SilentSessionId::new(),
        work_item: first.reference(),
        priority: SilentSessionPriority::Normal,
        queued_at: now(),
        resource_admission: resource(true),
        writer_admission: writer(true),
        entitlement_context: None,
    }
}

fn contextual_candidate(
    first: &WorkItem,
    dispatch_policy: EntitlementExecutionPolicy,
    reservation_id: Option<&str>,
) -> SilentSessionDispatchCandidate {
    SilentSessionDispatchCandidate {
        session_id: SilentSessionId::new(),
        work_item: first.reference(),
        priority: SilentSessionPriority::Normal,
        queued_at: now(),
        resource_admission: resource(true),
        writer_admission: writer(true),
        entitlement_context: Some(SilentSessionDispatchEntitlementContext {
            dispatch_policy,
            initiating_policy: None,
            reservation_id: reservation_id.map(str::to_string),
        }),
    }
}

fn signed_snapshot(
    state: EntitlementState,
    grace_until: Option<chrono::DateTime<Utc>>,
) -> EntitlementSnapshot {
    let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-bypass-core");
    snapshot.state = state;
    snapshot.sequence = Some(7);
    snapshot.lease_id = Some("lease-core-7".into());
    snapshot.lease_digest = Some("sha256:core-bypass".into());
    snapshot
        .features
        .insert("focusa.agent.parallelism".into(), true);
    snapshot.limits.insert("parallel_workers".into(), 2);
    snapshot.expires_at = Some(now() + Duration::hours(1));
    snapshot.offline_grace_until = grace_until;
    snapshot
}

fn premium_dispatch_policy() -> EntitlementExecutionPolicy {
    EntitlementExecutionPolicy::new(
        "focusa.silent_session.dispatch",
        OperationClass::ValueMutation,
        CapabilityFamily::Automation,
        Some("focusa.agent.parallelism"),
        Some("parallel_workers"),
        RecoveryAllowance::None,
    )
}

// ── 1. Worker/scheduler dispatch revalidates authority at dispatch ────────

#[test]
fn spec152f_bypass_resistance_worker_dispatch_revalidates_authority_at_dispatch() {
    let first = item("first");
    let work_items = [first.clone()];

    // Outage / missing authority: `LicenseGuard::eval(7)` carries no usable
    // base entitlement, so dispatch is refused before any side effect.
    let outage = select_silent_session_dispatch_with_default_entitlement(
        &work_items,
        &query(),
        &[candidate(&first)],
        &LicenseGuard::eval(7),
    );
    let error = outage.expect_err("worker dispatch without base entitlement must fail");
    assert_eq!(error.code, "ENTITLEMENT_ROUTE_UNCLASSIFIED");

    // Refunded/revoked authority: dispatch is refused too.
    let revoked = select_silent_session_dispatch_with_default_entitlement(
        &work_items,
        &query(),
        &[candidate(&first)],
        &LicenseGuard::from_entitlement(signed_snapshot(EntitlementState::RecoveryOnly, None)),
    );
    let error = revoked.expect_err("worker dispatch after revoke must fail");
    assert_eq!(error.code, "ENTITLEMENT_ROUTE_UNCLASSIFIED");

    // Valid signed entitlement: dispatch selects the work item.
    let (readiness, dispatch) = select_silent_session_dispatch_with_default_entitlement(
        &work_items,
        &query(),
        &[candidate(&first)],
        &LicenseGuard::from_entitlement(signed_snapshot(EntitlementState::Active, None)),
    )
    .expect("worker dispatch with a usable signed entitlement passes");
    assert_eq!(readiness.ready.len(), 1);
    assert_eq!(dispatch.selected_work_item, Some(first.reference()));
    assert!(dispatch.deferred.is_empty());
}

#[test]
fn spec152f_bypass_resistance_worker_premium_dispatch_stops_when_offline_grace_closes() {
    let first = item("first");
    let work_items = [first.clone()];
    let dispatch_policy = premium_dispatch_policy();

    // Premium dispatch inside a valid Offline Grace window: the cached grant
    // is still within its signed bounds and the worker runs.
    let (_, valid) = select_silent_session_dispatch_with_entitlement(
        &work_items,
        &query(),
        &[contextual_candidate(
            &first,
            dispatch_policy.clone(),
            Some("reservation-offline-bypass"),
        )],
        &LicenseGuard::from_entitlement(signed_snapshot(
            EntitlementState::OfflineGrace,
            Some(now() + Duration::minutes(5)),
        )),
        &dispatch_policy,
    )
    .expect("valid offline grace permits premium worker dispatch");
    assert_eq!(valid.selected_work_item, Some(first.reference()));

    // Offline Grace closed: the cached grant is outside its signed window and
    // the worker is deferred — no stale cached flag extends the window.
    let (_, expired) = select_silent_session_dispatch_with_entitlement(
        &work_items,
        &query(),
        &[contextual_candidate(
            &first,
            dispatch_policy.clone(),
            Some("reservation-offline-bypass"),
        )],
        &LicenseGuard::from_entitlement(signed_snapshot(
            EntitlementState::OfflineGrace,
            Some(now() - Duration::minutes(1)),
        )),
        &dispatch_policy,
    )
    .expect("dispatch stays ordered after grace closure");
    assert_eq!(expired.selected_work_item, None);
    assert_eq!(expired.deferred.len(), 1);
    assert_eq!(
        expired.deferred[0].reason,
        DispatchDeferralReason::EntitlementDenied
    );
    assert!(
        expired.deferred[0].detail.contains("ENTITLEMENT_REQUIRED"),
        "closed offline grace defers premium dispatch with a typed entitlement denial"
    );
}

// ── 2. Core guard: direct value-mutation bypass attempts fail closed ──────

#[test]
fn spec152f_bypass_resistance_core_guard_direct_bypass_fails_closed() {
    let base_mutation = EntitlementExecutionPolicy::new(
        "focusa.core.workpoint.mutate",
        OperationClass::ValueMutation,
        CapabilityFamily::BaseFocusa,
        None,
        Some("workpoints"),
        RecoveryAllowance::None,
    );

    // Direct mutation during an outage (no snapshot) is refused.
    let outage = evaluate_entitlement_execution(
        &LicenseGuard::eval(7),
        &base_mutation,
        EntitlementExecutionContext::default(),
    )
    .expect_err("outage must deny direct base mutation");
    assert_eq!(outage.code, "ENTITLEMENT_BASE_REQUIRED");

    // Direct mutation after refund/revoke is refused even when stale cached
    // feature claims are present.
    let revoked = evaluate_entitlement_execution(
        &LicenseGuard::from_entitlement(signed_snapshot(EntitlementState::RecoveryOnly, None)),
        &base_mutation,
        EntitlementExecutionContext::default(),
    )
    .expect_err("revoked authority must deny direct base mutation");
    assert_eq!(revoked.code, "ENTITLEMENT_BASE_REQUIRED");

    // A usable signed entitlement permits the base mutation.
    let active = evaluate_entitlement_execution(
        &LicenseGuard::from_entitlement(signed_snapshot(EntitlementState::Active, None)),
        &base_mutation,
        EntitlementExecutionContext::default(),
    )
    .expect("active signed entitlement permits base mutation");
    assert_eq!(active.code, "ENTITLEMENT_ALLOWED");

    // A valid Offline Grace window permits the base mutation (cached base
    // grant within its signed bounds).
    let grace = evaluate_entitlement_execution(
        &LicenseGuard::from_entitlement(signed_snapshot(
            EntitlementState::OfflineGrace,
            Some(now() + Duration::minutes(5)),
        )),
        &base_mutation,
        EntitlementExecutionContext::default(),
    )
    .expect("valid offline grace permits base mutation");
    assert_eq!(grace.code, "ENTITLEMENT_ALLOWED");
    assert_eq!(grace.entitlement_state, "offline_grace");

    // Premium automation without a signed feature is denied at the guard.
    let premium = EntitlementExecutionPolicy::new(
        "focusa.agent.parallelism.run",
        OperationClass::ValueMutation,
        CapabilityFamily::Automation,
        Some("focusa.agent.parallelism"),
        Some("parallel_workers"),
        RecoveryAllowance::None,
    );
    let mut snapshot = signed_snapshot(EntitlementState::Active, None);
    snapshot.features.clear();
    let denied = evaluate_entitlement_execution(
        &LicenseGuard::from_entitlement(snapshot),
        &premium,
        EntitlementExecutionContext::default(),
    )
    .expect_err("unsigned premium feature must be denied at the guard");
    assert_eq!(denied.code, "ENTITLEMENT_FEATURE_REQUIRED");
}

// ── 3. Outage preserves recovery, read, export, and maintenance ───────────

#[test]
fn spec152f_bypass_resistance_recovery_surfaces_survive_outage_and_bypass_attempts() {
    // During the outage (no snapshot), recovery/read/export/update/repair/
    // uninstall allowances stay available while value mutations are denied.
    let guard = LicenseGuard::eval(7);
    for (operation_id, family, allowance, expected_status) in [
        (
            "focusa.account.recovery",
            CapabilityFamily::AccountRecovery,
            RecoveryAllowance::AccountRecovery,
            "allow",
        ),
        (
            "focusa.export.basic",
            CapabilityFamily::CustomerDataExport,
            RecoveryAllowance::CustomerDataExport,
            "allow",
        ),
        (
            "focusa.project.read",
            CapabilityFamily::ReadProjection,
            RecoveryAllowance::ReadProjection,
            "read",
        ),
        (
            "focusa.update.apply",
            CapabilityFamily::AccountRecovery,
            RecoveryAllowance::StableSecurityUpdate,
            "allow",
        ),
        (
            "focusa.update.rollback",
            CapabilityFamily::AccountRecovery,
            RecoveryAllowance::RepairRollback,
            "allow",
        ),
        (
            "focusa.install.uninstall",
            CapabilityFamily::AccountRecovery,
            RecoveryAllowance::Uninstall,
            "allow",
        ),
    ] {
        let policy = EntitlementExecutionPolicy::new(
            operation_id,
            if family == CapabilityFamily::ReadProjection {
                OperationClass::Read
            } else {
                OperationClass::Recovery
            },
            family,
            None,
            None,
            allowance,
        );
        let decision =
            evaluate_entitlement_execution(&guard, &policy, EntitlementExecutionContext::default())
                .expect("recovery/read/export/maintenance stays available during outage");
        assert_eq!(decision.status, expected_status);
        assert_eq!(decision.code, "ENTITLEMENT_ALLOWED");
    }

    // A direct bypass that tries to smuggle a value mutation through the
    // internal-maintenance classification is refused when the initiating
    // posture is missing, exactly like any other value-producing mutation.
    let policy = EntitlementExecutionPolicy::new(
        "focusa.internal.maintenance",
        OperationClass::InternalMaintenance,
        CapabilityFamily::InternalMaintenance,
        None,
        None,
        RecoveryAllowance::None,
    );
    let denied: EntitlementExecutionFailure =
        evaluate_entitlement_execution(&guard, &policy, EntitlementExecutionContext::default())
            .expect_err(
                "unclassified maintenance mutation without initiating posture fails closed",
            );
    assert_eq!(denied.code, "ENTITLEMENT_ROUTE_UNCLASSIFIED");
}
