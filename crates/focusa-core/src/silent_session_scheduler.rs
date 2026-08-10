//! Silent Session dispatch overlay for the one canonical Work Loop scheduler.

// Dispatch denials intentionally retain their complete entitlement evidence.
#![allow(clippy::result_large_err)]

use crate::silent_session::SilentSessionId;
use crate::silent_session_resources::ResourceAdmissionDecision;
use crate::silent_session_writer::WriterAdmissionDecision;
use crate::work_item::{
    WorkItem, WorkItemQuery, WorkItemReadiness, WorkItemRef, evaluate_readiness,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::guarded_mutation::guard_value_mutation;
use crate::license::{
    EntitlementExecutionContext,
    EntitlementExecutionPolicy,
};


pub const SILENT_SESSION_DISPATCH_SCHEMA: &str = "focusa.silent_session_dispatch.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SilentSessionPriority {
    Interactive,
    High,
    Normal,
    Background,
    Low,
    Maintenance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentSessionDispatchEntitlementContext {
    pub dispatch_policy: EntitlementExecutionPolicy,
    pub initiating_policy: Option<EntitlementExecutionPolicy>,
    pub reservation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentSessionDispatchCandidate {
    pub session_id: SilentSessionId,
    pub work_item: WorkItemRef,
    pub priority: SilentSessionPriority,
    pub queued_at: DateTime<Utc>,
    pub resource_admission: ResourceAdmissionDecision,
    pub writer_admission: WriterAdmissionDecision,
    pub entitlement_context: Option<SilentSessionDispatchEntitlementContext>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchDeferralReason {
    WorkItemNotReady,
    ResourceAdmissionDenied,
    WriterAdmissionDenied,
    EntitlementDenied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeferredDispatchCandidate {
    pub session_id: SilentSessionId,
    pub work_item: WorkItemRef,
    pub reason: DispatchDeferralReason,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SilentSessionDispatchDecision {
    pub schema: String,
    pub selected_session_id: Option<SilentSessionId>,
    pub selected_work_item: Option<WorkItemRef>,
    pub deferred: Vec<DeferredDispatchCandidate>,
    pub canonical_ready_count: usize,
}

/// Spec 152F.06.06 — bounded, label-only scheduler revalidation metrics.
///
/// Counts are derived from one `SilentSessionDispatchDecision` and keyed only
/// by the fixed deferral-reason enum plus a selection counter. The snapshot
/// escape returns canonical labels with counts and never retains session ids,
/// work-item refs, lease ids, digests, or customer data.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SilentSessionDispatchMetrics {
    selected: u64,
    deferred_work_item_not_ready: u64,
    deferred_resource: u64,
    deferred_writer: u64,
    deferred_entitlement: u64,
}

impl SilentSessionDispatchMetrics {
    /// Record one dispatch decision. Only the fixed deferral-reason set and the
    /// selection bit are counted; no caller-controlled string can grow the set.
    pub fn record(&mut self, decision: &SilentSessionDispatchDecision) {
        if decision.selected_work_item.is_some() {
            self.selected += 1;
        }
        for deferred in &decision.deferred {
            match deferred.reason {
                DispatchDeferralReason::WorkItemNotReady => self.deferred_work_item_not_ready += 1,
                DispatchDeferralReason::ResourceAdmissionDenied => self.deferred_resource += 1,
                DispatchDeferralReason::WriterAdmissionDenied => self.deferred_writer += 1,
                DispatchDeferralReason::EntitlementDenied => self.deferred_entitlement += 1,
            }
        }
    }

    /// Selections recorded across all recorded decisions.
    pub fn selected(&self) -> u64 {
        self.selected
    }

    /// Deferrals recorded for one canonical deferral reason.
    pub fn deferred(&self, reason: DispatchDeferralReason) -> u64 {
        match reason {
            DispatchDeferralReason::WorkItemNotReady => self.deferred_work_item_not_ready,
            DispatchDeferralReason::ResourceAdmissionDenied => self.deferred_resource,
            DispatchDeferralReason::WriterAdmissionDenied => self.deferred_writer,
            DispatchDeferralReason::EntitlementDenied => self.deferred_entitlement,
        }
    }

    /// Entitlement revalidation denials (the Spec 152F §7 dispatch check).
    pub fn entitlement_denied(&self) -> u64 {
        self.deferred_entitlement
    }

    /// Total recorded outcomes (selection + all deferral reasons).
    pub fn total(&self) -> u64 {
        self.selected
            + self.deferred_work_item_not_ready
            + self.deferred_resource
            + self.deferred_writer
            + self.deferred_entitlement
    }

    /// Fixed capacity in counter slots; independent of any recorded workload.
    pub const fn capacity(&self) -> usize {
        5
    }

    /// Label-only snapshot for logs and metrics. Values are counts; keys are
    /// canonical labels — never session, work-item, or lease identifiers.
    pub fn snapshot(&self) -> BTreeMap<String, u64> {
        let mut out = BTreeMap::new();
        out.insert("dispatch.selected.count".to_string(), self.selected);
        out.insert(
            "dispatch.deferred.work_item_not_ready.count".to_string(),
            self.deferred_work_item_not_ready,
        );
        out.insert(
            "dispatch.deferred.resource_denied.count".to_string(),
            self.deferred_resource,
        );
        out.insert(
            "dispatch.deferred.writer_denied.count".to_string(),
            self.deferred_writer,
        );
        out.insert(
            "dispatch.deferred.entitlement_denied.count".to_string(),
            self.deferred_entitlement,
        );
        out
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SilentSessionDispatchEntitlementError {
    pub code: String,
    pub message: String,
    pub required_feature: Option<String>,
    pub limit_bucket: Option<String>,
    pub initiating_posture: Option<String>,
    pub initiating_operation_id: Option<String>,
    pub reservation_id: Option<String>,
}

fn silent_session_dispatch_entitlement_policy() -> EntitlementExecutionPolicy {
    EntitlementExecutionPolicy::new(
        "focusa.silent_session.dispatch",
        focusa_license::OperationClass::InternalMaintenance,
        focusa_license::CapabilityFamily::InternalMaintenance,
        None,
        None,
        focusa_license::RecoveryAllowance::None,
    )
}

fn resolve_internal_maintenance_posture_from_guard(
    entitlement_guard: &focusa_license::LicenseGuard,
) -> Option<focusa_license::EntitlementPolicyPosture> {
    focusa_license::base_product_projection(entitlement_guard.entitlement.as_ref())
        .ok()
        .filter(|projection| projection.permits_base_mutations)
        .map(|_| focusa_license::EntitlementPolicyPosture::Base)
}

fn entitlement_status_to_posture(
    status: &str,
) -> Option<focusa_license::EntitlementPolicyPosture> {
    match status {
        s if s == focusa_license::EntitlementPolicyPosture::Allow.status() => {
            Some(focusa_license::EntitlementPolicyPosture::Allow)
        }
        s if s == focusa_license::EntitlementPolicyPosture::Read.status() => {
            Some(focusa_license::EntitlementPolicyPosture::Read)
        }
        s if s == focusa_license::EntitlementPolicyPosture::Base.status() => {
            Some(focusa_license::EntitlementPolicyPosture::Base)
        }
        s if s == focusa_license::EntitlementPolicyPosture::Feature.status() => {
            Some(focusa_license::EntitlementPolicyPosture::Feature)
        }
        _ => None,
    }
}

fn evaluate_silent_session_dispatch_entitlement(
    entitlement_guard: &focusa_license::LicenseGuard,
    policy: &EntitlementExecutionPolicy,
    context: EntitlementExecutionContext,
    initiating_operation_id: Option<&str>,
    reservation_id: Option<&str>,
) -> Result<(), SilentSessionDispatchEntitlementError> {
    // Spec 172 §11.5/§20.9: workers, schedulers, queues, and resumable jobs
    // inherit initiating authority and MUST revalidate at dispatch through the
    // shared core chokepoint. A previously queued operation cannot continue
    // after refund, revoke, higher sequence, or family denial, and a stale or
    // fabricated lease cannot produce value even when HTTP middleware is absent.
    guard_value_mutation(entitlement_guard, policy, context)
        .map(|_| ())
        .map_err(|error| SilentSessionDispatchEntitlementError {
            code: error.code,
            message: error.message,
            required_feature: error.required_feature,
            limit_bucket: error.limit_bucket,
            initiating_posture: context
                .initiating_posture
                .map(|posture| posture.status().to_string()),
            initiating_operation_id: initiating_operation_id
                .map(ToString::to_string)
                .or_else(|| Some(policy.operation_id.clone())),
            reservation_id: reservation_id.map(ToString::to_string),
        })
}

fn evaluate_silent_session_dispatch_candidate_entitlement(
    entitlement_guard: &focusa_license::LicenseGuard,
    candidate: &SilentSessionDispatchCandidate,
    dispatch_policy: &EntitlementExecutionPolicy,
    fallback_initiating_posture: Option<focusa_license::EntitlementPolicyPosture>,
    fallback_initiating_operation_id: Option<&str>,
) -> Result<(), SilentSessionDispatchEntitlementError> {
    let reference = candidate.entitlement_context.as_ref();
    let resolved_dispatch_policy = reference
        .map(|reference| &reference.dispatch_policy)
        .unwrap_or(dispatch_policy);
    let resolved_initiating_operation_id = reference
        .and_then(|reference| reference.initiating_policy.as_ref())
        .map(|policy| policy.operation_id.as_str())
        .or(fallback_initiating_operation_id);

    let resolved_initiating_posture = if resolved_dispatch_policy
        .capability_family
        == focusa_license::CapabilityFamily::InternalMaintenance
    {
        match reference.and_then(|reference| reference.initiating_policy.as_ref()) {
            Some(policy) => {
                let decision = guard_value_mutation(
                    entitlement_guard,
                    policy,
                    EntitlementExecutionContext::default(),
                )
                .map_err(|error| SilentSessionDispatchEntitlementError {
                    code: error.code,
                    message: error.message,
                    required_feature: error.required_feature,
                    limit_bucket: error.limit_bucket,
                    initiating_posture: Some(
                        focusa_license::EntitlementPolicyPosture::Deny.status().to_string(),
                    ),
                    initiating_operation_id: Some(policy.operation_id.clone()),
                    reservation_id: reference
                        .and_then(|context| context.reservation_id.as_deref())
                        .map(ToString::to_string),
                })?;
                let posture = entitlement_status_to_posture(&decision.status).ok_or_else(||
                    SilentSessionDispatchEntitlementError {
                        code: "ENTITLEMENT_ROUTE_UNCLASSIFIED".to_string(),
                        message: format!(
                            "unsupported entitlement posture {} for initiating operation {}",
                            decision.status, policy.operation_id
                        ),
                        required_feature: None,
                        limit_bucket: None,
                        initiating_posture: Some(
                            focusa_license::EntitlementPolicyPosture::Deny.status().to_string(),
                        ),
                        initiating_operation_id: Some(policy.operation_id.clone()),
                        reservation_id: reference
                            .and_then(|context| context.reservation_id.as_deref())
                            .map(ToString::to_string),
                    }
                )?;
                Some(posture)
            }
            None => fallback_initiating_posture,
        }
    } else {
        None
    };

    evaluate_silent_session_dispatch_entitlement(
        entitlement_guard,
        resolved_dispatch_policy,
        EntitlementExecutionContext {
            now: Utc::now(),
            initiating_posture: resolved_initiating_posture,
        },
        resolved_initiating_operation_id,
        reference
            .and_then(|context| context.reservation_id.as_deref()),
    )
}

pub fn select_silent_session_dispatch_with_entitlement(
    work_items: &[WorkItem],
    query: &WorkItemQuery,
    candidates: &[SilentSessionDispatchCandidate],
    entitlement_guard: &focusa_license::LicenseGuard,
    entitlement_policy: &EntitlementExecutionPolicy,
) -> Result<(WorkItemReadiness, SilentSessionDispatchDecision), SilentSessionDispatchEntitlementError> {
    select_silent_session_dispatch_with_entitlement_with_initiating_context(
        work_items,
        query,
        candidates,
        entitlement_guard,
        entitlement_policy,
        None,
        None,
    )
}

pub fn select_silent_session_dispatch_with_entitlement_with_initiating_context(
    work_items: &[WorkItem],
    query: &WorkItemQuery,
    candidates: &[SilentSessionDispatchCandidate],
    entitlement_guard: &focusa_license::LicenseGuard,
    entitlement_policy: &EntitlementExecutionPolicy,
    initiating_posture: Option<focusa_license::EntitlementPolicyPosture>,
    initiating_operation_id: Option<&str>,
) -> Result<(WorkItemReadiness, SilentSessionDispatchDecision), SilentSessionDispatchEntitlementError> {
    let has_entitlement_context = candidates
        .iter()
        .any(|candidate| candidate.entitlement_context.is_some());

    if !has_entitlement_context {
        evaluate_silent_session_dispatch_entitlement(
            entitlement_guard,
            entitlement_policy,
            EntitlementExecutionContext {
                now: Utc::now(),
                initiating_posture,
            },
            initiating_operation_id,
            None,
        )?;
        return Ok(select_silent_session_dispatch(
            work_items,
            query,
            candidates,
        ));
    }

    Ok(select_silent_session_dispatch_with_entitlement_contexts(
        work_items,
        query,
        candidates,
        entitlement_guard,
        entitlement_policy,
        initiating_posture,
        initiating_operation_id,
    ))
}

pub fn select_silent_session_dispatch_with_default_entitlement(
    work_items: &[WorkItem],
    query: &WorkItemQuery,
    candidates: &[SilentSessionDispatchCandidate],
    entitlement_guard: &focusa_license::LicenseGuard,
) -> Result<(WorkItemReadiness, SilentSessionDispatchDecision), SilentSessionDispatchEntitlementError> {
    select_silent_session_dispatch_with_default_entitlement_with_initiating_context(
        work_items,
        query,
        candidates,
        entitlement_guard,
        resolve_internal_maintenance_posture_from_guard(entitlement_guard),
        Some("focusa.silent_session.dispatch"),
    )
}

pub fn select_silent_session_dispatch_with_default_entitlement_with_initiating_context(
    work_items: &[WorkItem],
    query: &WorkItemQuery,
    candidates: &[SilentSessionDispatchCandidate],
    entitlement_guard: &focusa_license::LicenseGuard,
    initiating_posture: Option<focusa_license::EntitlementPolicyPosture>,
    initiating_operation_id: Option<&str>,
) -> Result<(WorkItemReadiness, SilentSessionDispatchDecision), SilentSessionDispatchEntitlementError> {
    let policy = silent_session_dispatch_entitlement_policy();
    select_silent_session_dispatch_with_entitlement_with_initiating_context(
        work_items,
        query,
        candidates,
        entitlement_guard,
        &policy,
        initiating_posture,
        initiating_operation_id,
    )
}

fn select_silent_session_dispatch_with_entitlement_contexts(
    work_items: &[WorkItem],
    query: &WorkItemQuery,
    candidates: &[SilentSessionDispatchCandidate],
    entitlement_guard: &focusa_license::LicenseGuard,
    entitlement_policy: &EntitlementExecutionPolicy,
    initiating_posture: Option<focusa_license::EntitlementPolicyPosture>,
    initiating_operation_id: Option<&str>,
) -> (WorkItemReadiness, SilentSessionDispatchDecision) {
    let readiness = evaluate_readiness(work_items, query);
    let mut eligible = Vec::new();
    let mut deferred = Vec::new();
    for candidate in candidates {
        let ready_item = readiness
            .ready
            .iter()
            .find(|item| same_work_item(&item.reference(), &candidate.work_item));
        let Some(item) = ready_item else {
            let detail = readiness
                .blocked
                .iter()
                .find(|blocked| same_work_item(&blocked.item.reference(), &candidate.work_item))
                .map(|blocked| blocked.reason.clone())
                .unwrap_or_else(|| "work item is absent from canonical ready set".into());
            deferred.push(DeferredDispatchCandidate {
                session_id: candidate.session_id,
                work_item: candidate.work_item.clone(),
                reason: DispatchDeferralReason::WorkItemNotReady,
                detail,
            });
            continue;
        };

        if let Err(error) = evaluate_silent_session_dispatch_candidate_entitlement(
            entitlement_guard,
            candidate,
            entitlement_policy,
            initiating_posture,
            initiating_operation_id,
        ) {
            deferred.push(DeferredDispatchCandidate {
                session_id: candidate.session_id,
                work_item: candidate.work_item.clone(),
                reason: DispatchDeferralReason::EntitlementDenied,
                detail: format!(
                    "entitlement denied: {} {} (reservation={})",
                    error.code,
                    error.message,
                    error
                        .reservation_id
                        .unwrap_or_else(|| "missing".to_string())
                ),
            });
            continue;
        }
        if !candidate.resource_admission.admitted {
            deferred.push(DeferredDispatchCandidate {
                session_id: candidate.session_id,
                work_item: candidate.work_item.clone(),
                reason: DispatchDeferralReason::ResourceAdmissionDenied,
                detail: format!(
                    "resource admission denied: {:?}",
                    candidate.resource_admission.denials
                ),
            });
            continue;
        }
        if !candidate.writer_admission.admitted {
            deferred.push(DeferredDispatchCandidate {
                session_id: candidate.session_id,
                work_item: candidate.work_item.clone(),
                reason: DispatchDeferralReason::WriterAdmissionDenied,
                detail: format!(
                    "writer admission denied: {:?}",
                    candidate.writer_admission.denials
                ),
            });
            continue;
        }
        eligible.push((candidate, item.priority));
    }
    eligible.sort_by(|(left, left_item_priority), (right, right_item_priority)| {
        left.priority
            .cmp(&right.priority)
            .then_with(|| left_item_priority.cmp(right_item_priority))
            .then_with(|| left.queued_at.cmp(&right.queued_at))
            .then_with(|| left.session_id.cmp(&right.session_id))
    });
    deferred.sort_by_key(|item| item.session_id);
    let selected = eligible.first().map(|(candidate, _)| *candidate);
    let decision = SilentSessionDispatchDecision {
        schema: SILENT_SESSION_DISPATCH_SCHEMA.into(),
        selected_session_id: selected.map(|candidate| candidate.session_id),
        selected_work_item: selected.map(|candidate| candidate.work_item.clone()),
        deferred,
        canonical_ready_count: readiness.ready.len(),
    };
    (readiness, decision)
}

/// Dispatch only from canonical Work Loop readiness. This function never
/// evaluates dependencies itself and therefore cannot become a second scheduler.
pub fn select_silent_session_dispatch(
    work_items: &[WorkItem],
    query: &WorkItemQuery,
    candidates: &[SilentSessionDispatchCandidate],
) -> (WorkItemReadiness, SilentSessionDispatchDecision) {
    let readiness = evaluate_readiness(work_items, query);
    let mut eligible = Vec::new();
    let mut deferred = Vec::new();
    for candidate in candidates {
        let ready_item = readiness
            .ready
            .iter()
            .find(|item| same_work_item(&item.reference(), &candidate.work_item));
        let Some(item) = ready_item else {
            let detail = readiness
                .blocked
                .iter()
                .find(|blocked| same_work_item(&blocked.item.reference(), &candidate.work_item))
                .map(|blocked| blocked.reason.clone())
                .unwrap_or_else(|| "work item is absent from canonical ready set".into());
            deferred.push(DeferredDispatchCandidate {
                session_id: candidate.session_id,
                work_item: candidate.work_item.clone(),
                reason: DispatchDeferralReason::WorkItemNotReady,
                detail,
            });
            continue;
        };
        if !candidate.resource_admission.admitted {
            deferred.push(DeferredDispatchCandidate {
                session_id: candidate.session_id,
                work_item: candidate.work_item.clone(),
                reason: DispatchDeferralReason::ResourceAdmissionDenied,
                detail: format!(
                    "resource admission denied: {:?}",
                    candidate.resource_admission.denials
                ),
            });
            continue;
        }
        if !candidate.writer_admission.admitted {
            deferred.push(DeferredDispatchCandidate {
                session_id: candidate.session_id,
                work_item: candidate.work_item.clone(),
                reason: DispatchDeferralReason::WriterAdmissionDenied,
                detail: format!(
                    "writer admission denied: {:?}",
                    candidate.writer_admission.denials
                ),
            });
            continue;
        }
        eligible.push((candidate, item.priority));
    }
    eligible.sort_by(|(left, left_item_priority), (right, right_item_priority)| {
        left.priority
            .cmp(&right.priority)
            .then_with(|| left_item_priority.cmp(right_item_priority))
            .then_with(|| left.queued_at.cmp(&right.queued_at))
            .then_with(|| left.session_id.cmp(&right.session_id))
    });
    deferred.sort_by_key(|item| item.session_id);
    let selected = eligible.first().map(|(candidate, _)| *candidate);
    let decision = SilentSessionDispatchDecision {
        schema: SILENT_SESSION_DISPATCH_SCHEMA.into(),
        selected_session_id: selected.map(|candidate| candidate.session_id),
        selected_work_item: selected.map(|candidate| candidate.work_item.clone()),
        deferred,
        canonical_ready_count: readiness.ready.len(),
    };
    (readiness, decision)
}

fn same_work_item(left: &WorkItemRef, right: &WorkItemRef) -> bool {
    left.provider == right.provider
        && left.provider_item_id == right.provider_item_id
        && left.project_root == right.project_root
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::silent_session_resources::{AdmissionDenial, RESOURCE_ADMISSION_SCHEMA};
    use crate::silent_session_writer::{WRITER_ADMISSION_SCHEMA, WriterAdmissionDenial};
    use crate::work_item::{WorkItemProvider, WorkItemStatus};
    use chrono::Duration;
    use focusa_license::{authority::{EntitlementSnapshot, EntitlementState}, EntitlementPolicyPosture};
    use std::path::PathBuf;

    fn item(id: &str, status: WorkItemStatus, priority: i32) -> WorkItem {
        WorkItem {
            provider: WorkItemProvider::Bd,
            provider_item_id: id.into(),
            project_root: PathBuf::from("/projects/focusa"),
            provider_status: status,
            title: id.into(),
            priority,
            parent: None,
            dependencies: vec![],
            acceptance_criteria: vec!["proof passes".into()],
            spec_refs: vec!["docs/133".into()],
            blocked_reason: None,
            url: None,
            revision: None,
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

    fn candidate(
        item: &WorkItem,
        priority: SilentSessionPriority,
        queued_at: DateTime<Utc>,
    ) -> SilentSessionDispatchCandidate {
        SilentSessionDispatchCandidate {
            session_id: SilentSessionId::new(),
            work_item: item.reference(),
            priority,
            queued_at,
            resource_admission: resource(true),
            writer_admission: writer(true),
            entitlement_context: None,
        }
    }

    fn contextual_policy_candidate(
        item: &WorkItem,
        priority: SilentSessionPriority,
        queued_at: DateTime<Utc>,
        context: SilentSessionDispatchEntitlementContext,
    ) -> SilentSessionDispatchCandidate {
        SilentSessionDispatchCandidate {
            session_id: SilentSessionId::new(),
            work_item: item.reference(),
            priority,
            queued_at,
            resource_admission: resource(true),
            writer_admission: writer(true),
            entitlement_context: Some(context),
        }
    }

    fn signed_snapshot(
        state: EntitlementState,
        offline_grace_until: Option<DateTime<Utc>>,
    ) -> focusa_license::LicenseGuard {
        let now = Utc::now();
        let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-core-scheduler");
        snapshot.state = state;
        snapshot.sequence = Some(7);
        snapshot.lease_id = Some("lease-3".into());
        snapshot.lease_digest = Some("sha256:scheduler".into());
        snapshot
            .features
            .insert("focusa.agent.silent_sessions".into(), true);
        snapshot
            .features
            .insert("focusa.agent.parallelism".into(), true);
        snapshot.expires_at = Some(now + chrono::Duration::hours(1));
        snapshot.offline_grace_until = offline_grace_until;
        focusa_license::LicenseGuard::from_entitlement(snapshot)
    }

    fn signed_base_snapshot() -> focusa_license::LicenseGuard {
        signed_snapshot(EntitlementState::Active, Some(Utc::now() + chrono::Duration::hours(1)))
    }

    #[test]
    fn delayed_dispatch_internal_maintenance_entitlement_rejects_without_posture() {
        let first = item("first", WorkItemStatus::Open, 0);
        let query = WorkItemQuery {
            project_root: PathBuf::from("/projects/focusa"),
            parent: None,
            limit: 100,
        };
        let decision = select_silent_session_dispatch_with_default_entitlement(
            &[first],
            &query,
            &[candidate(
                &item("first", WorkItemStatus::Open, 1),
                SilentSessionPriority::Normal,
                Utc::now(),
            )],
            &focusa_license::LicenseGuard::eval(7),
        )
        .expect_err("dispatch without initiating posture should fail");
        assert_eq!(decision.code, "ENTITLEMENT_ROUTE_UNCLASSIFIED");
    }

    #[test]
    fn delayed_dispatch_internal_maintenance_entitlement_inherits_initiating_posture() {
        let first = item("first", WorkItemStatus::Open, 0);
        let query = WorkItemQuery {
            project_root: PathBuf::from("/projects/focusa"),
            parent: None,
            limit: 100,
        };
        let (_readiness, dispatch) = select_silent_session_dispatch_with_default_entitlement_with_initiating_context(
            &[first.clone()],
            &query,
            &[candidate(&first, SilentSessionPriority::Normal, Utc::now())],
            &signed_base_snapshot(),
            Some(EntitlementPolicyPosture::Base),
            Some("focusa.scheduler.dispatch"),
        )
        .expect("dispatch with inherited posture should pass");
        assert_eq!(dispatch.selected_work_item, Some(first.reference()));
    }

    #[test]
    fn delayed_dispatch_allows_when_base_entitlement_exists() {
        let first = item("first", WorkItemStatus::Open, 0);
        let query = WorkItemQuery {
            project_root: PathBuf::from("/projects/focusa"),
            parent: None,
            limit: 100,
        };
        let (_readiness, dispatch) = select_silent_session_dispatch_with_default_entitlement(
            &[first.clone()],
            &query,
            &[candidate(&first, SilentSessionPriority::Normal, Utc::now())],
            &signed_base_snapshot(),
        )
        .expect("default entitlement should allow silent session dispatch selection");
        assert_eq!(dispatch.selected_work_item, Some(first.reference()));
    }

    #[test]
    fn scheduler_entitlement_revalidation_stops_when_entitlement_is_recovered_or_revoked() {
        let first = item("first", WorkItemStatus::Open, 0);
        let query = WorkItemQuery {
            project_root: PathBuf::from("/projects/focusa"),
            parent: None,
            limit: 100,
        };
        let context = SilentSessionDispatchEntitlementContext {
            dispatch_policy: EntitlementExecutionPolicy::new(
                "focusa.silent_session.dispatch",
                focusa_license::OperationClass::InternalMaintenance,
                focusa_license::CapabilityFamily::InternalMaintenance,
                None,
                None,
                focusa_license::RecoveryAllowance::None,
            ),
            initiating_policy: Some(EntitlementExecutionPolicy::new(
                "focusa.workpoint.checkpoint",
                focusa_license::OperationClass::ValueMutation,
                focusa_license::CapabilityFamily::BaseFocusa,
                None,
                None,
                focusa_license::RecoveryAllowance::None,
            )),
            reservation_id: Some("reservation-workflow".into()),
        };

        let (_readiness, active_dispatch) = select_silent_session_dispatch_with_entitlement(
            &[first.clone()],
            &query,
            &[contextual_policy_candidate(
                &first,
                SilentSessionPriority::Normal,
                Utc::now(),
                context.clone(),
            )],
            &signed_snapshot(EntitlementState::Active, Some(Utc::now() + Duration::hours(1))),
            &silent_session_dispatch_entitlement_policy(),
        )
        .expect("active entitlement should permit dispatch");
        assert_eq!(active_dispatch.selected_work_item, Some(first.reference()));

        let (_readiness, revoked_dispatch) = select_silent_session_dispatch_with_entitlement(
            &[first.clone()],
            &query,
            &[contextual_policy_candidate(
                &first,
                SilentSessionPriority::Normal,
                Utc::now(),
                context,
            )],
            &signed_snapshot(EntitlementState::RecoveryOnly, None),
            &silent_session_dispatch_entitlement_policy(),
        )
        .expect("dispatch should stay ordered after entitlement loss");
        assert_eq!(revoked_dispatch.selected_work_item, None);
        assert_eq!(revoked_dispatch.deferred.len(), 1);
        assert_eq!(
            revoked_dispatch.deferred[0].reason,
            DispatchDeferralReason::EntitlementDenied
        );
        assert_eq!(
            revoked_dispatch.deferred[0].detail.contains("ENTITLEMENT_BASE_REQUIRED"),
            true
        );
    }

    #[test]
    fn scheduler_entitlement_revalidation_refuses_past_offline_grace_for_features() {
        let first = item("first", WorkItemStatus::Open, 0);
        let query = WorkItemQuery {
            project_root: PathBuf::from("/projects/focusa"),
            parent: None,
            limit: 100,
        };
        let mut context = SilentSessionDispatchEntitlementContext {
            dispatch_policy: EntitlementExecutionPolicy::new(
                "focusa.silent_session.dispatch",
                focusa_license::OperationClass::ValueMutation,
                focusa_license::CapabilityFamily::Automation,
                Some("focusa.agent.parallelism"),
                Some("parallel_workers"),
                focusa_license::RecoveryAllowance::None,
            ),
            initiating_policy: None,
            reservation_id: Some("reservation-offline".into()),
        };

        let now = Utc::now();
        let (_readiness, valid_dispatch) = select_silent_session_dispatch_with_entitlement(
            &[first.clone()],
            &query,
            &[contextual_policy_candidate(
                &first,
                SilentSessionPriority::Normal,
                now,
                context.clone(),
            )],
            &signed_snapshot(
                EntitlementState::OfflineGrace,
                Some(now + Duration::hours(1)),
            ),
            &silent_session_dispatch_entitlement_policy(),
        )
        .expect("valid offline grace should permit premium dispatch");
        assert_eq!(valid_dispatch.selected_work_item, Some(first.reference()));

        context.dispatch_policy = EntitlementExecutionPolicy::new(
            "focusa.silent_session.dispatch",
            focusa_license::OperationClass::ValueMutation,
            focusa_license::CapabilityFamily::Automation,
            Some("focusa.agent.parallelism"),
            Some("parallel_workers"),
            focusa_license::RecoveryAllowance::None,
        );
        let (_readiness, expired_dispatch) = select_silent_session_dispatch_with_entitlement(
            &[first.clone()],
            &query,
            &[contextual_policy_candidate(
                &first,
                SilentSessionPriority::Normal,
                now,
                context,
            )],
            &signed_snapshot(EntitlementState::OfflineGrace, Some(now - Duration::hours(1))),
            &silent_session_dispatch_entitlement_policy(),
        )
        .expect("expired offline grace should stop dispatch");
        assert_eq!(expired_dispatch.selected_work_item, None);
        assert_eq!(
            expired_dispatch.deferred[0].detail.contains("ENTITLEMENT_REQUIRED"),
            true
        );
    }

    #[test]
    fn dependency_blocked_candidate_defers_and_alternate_ready_work_runs() {
        let dependency = item("dependency", WorkItemStatus::Open, 0);
        let mut blocked = item("blocked", WorkItemStatus::Open, 0);
        blocked.dependencies = vec![dependency.reference()];
        let alternate = item("alternate", WorkItemStatus::Open, 5);
        let now = Utc::now();
        let blocked_candidate = candidate(&blocked, SilentSessionPriority::Interactive, now);
        let alternate_candidate = candidate(&alternate, SilentSessionPriority::Normal, now);
        let items = vec![dependency, blocked, alternate.clone()];
        let query = WorkItemQuery {
            project_root: PathBuf::from("/projects/focusa"),
            parent: None,
            limit: 100,
        };
        let (_, decision) = select_silent_session_dispatch(
            &items,
            &query,
            &[blocked_candidate.clone(), alternate_candidate.clone()],
        );
        assert_eq!(
            decision.selected_session_id,
            Some(alternate_candidate.session_id)
        );
        assert_eq!(decision.selected_work_item, Some(alternate.reference()));
        assert!(decision.deferred.iter().any(|entry| {
            entry.session_id == blocked_candidate.session_id
                && entry.reason == DispatchDeferralReason::WorkItemNotReady
        }));
    }

    #[test]
    fn quota_and_writer_denials_defer_without_blocking_an_alternate() {
        let first = item("first", WorkItemStatus::Open, 0);
        let second = item("second", WorkItemStatus::Open, 1);
        let third = item("third", WorkItemStatus::Open, 2);
        let now = Utc::now();
        let mut resource_denied = candidate(&first, SilentSessionPriority::High, now);
        resource_denied.resource_admission = resource(false);
        let mut writer_denied = candidate(&second, SilentSessionPriority::High, now);
        writer_denied.writer_admission = writer(false);
        let alternate = candidate(&third, SilentSessionPriority::Background, now);
        let query = WorkItemQuery {
            project_root: PathBuf::from("/projects/focusa"),
            parent: None,
            limit: 100,
        };
        let (_, decision) = select_silent_session_dispatch(
            &[first, second, third],
            &query,
            &[resource_denied, writer_denied, alternate.clone()],
        );
        assert_eq!(decision.selected_session_id, Some(alternate.session_id));
        assert_eq!(decision.deferred.len(), 2);
    }

    #[test]
    fn priority_then_work_item_priority_then_age_is_deterministic() {
        let high_item = item("high-item", WorkItemStatus::Open, 9);
        let low_item = item("low-item", WorkItemStatus::Open, 0);
        let now = Utc::now();
        let high = candidate(&high_item, SilentSessionPriority::High, now);
        let older_high = candidate(
            &low_item,
            SilentSessionPriority::High,
            now - Duration::seconds(1),
        );
        let query = WorkItemQuery {
            project_root: PathBuf::from("/projects/focusa"),
            parent: None,
            limit: 100,
        };
        let (_, decision) = select_silent_session_dispatch(
            &[high_item, low_item],
            &query,
            &[high, older_high.clone()],
        );
        assert_eq!(decision.selected_session_id, Some(older_high.session_id));
    }

    fn metrics_decision(
        selected: bool,
        deferrals: &[(DispatchDeferralReason, &str)],
    ) -> SilentSessionDispatchDecision {
        SilentSessionDispatchDecision {
            schema: SILENT_SESSION_DISPATCH_SCHEMA.into(),
            selected_session_id: None,
            selected_work_item: selected
                .then(|| item("selected", WorkItemStatus::Open, 0).reference()),
            deferred: deferrals
                .iter()
                .map(|(reason, detail)| DeferredDispatchCandidate {
                    session_id: SilentSessionId::new(),
                    work_item: item("deferred", WorkItemStatus::Open, 0).reference(),
                    reason: *reason,
                    detail: detail.to_string(),
                })
                .collect(),
            canonical_ready_count: 1,
        }
    }

    #[test]
    fn spec152f_observability_scheduler_revalidation_metrics_count_by_reason() {
        let mut metrics = SilentSessionDispatchMetrics::default();
        metrics.record(&metrics_decision(
            true,
            &[(DispatchDeferralReason::EntitlementDenied, "ENTITLEMENT_BASE_REQUIRED")],
        ));
        metrics.record(&metrics_decision(
            false,
            &[
                (DispatchDeferralReason::EntitlementDenied, "ENTITLEMENT_REQUIRED"),
                (DispatchDeferralReason::ResourceAdmissionDenied, "quota"),
            ],
        ));
        metrics.record(&metrics_decision(
            false,
            &[(DispatchDeferralReason::WorkItemNotReady, "blocked")],
        ));
        metrics.record(&metrics_decision(true, &[]));

        assert_eq!(metrics.selected(), 2);
        assert_eq!(metrics.entitlement_denied(), 2);
        assert_eq!(metrics.deferred(DispatchDeferralReason::ResourceAdmissionDenied), 1);
        assert_eq!(metrics.deferred(DispatchDeferralReason::WriterAdmissionDenied), 0);
        assert_eq!(metrics.deferred(DispatchDeferralReason::WorkItemNotReady), 1);
        assert_eq!(metrics.total(), 2 + 2 + 1 + 1);
    }

    #[test]
    fn spec152f_observability_scheduler_revalidation_metrics_are_label_only_and_bounded() {
        let mut metrics = SilentSessionDispatchMetrics::default();
        metrics.record(&metrics_decision(
            true,
            &[(DispatchDeferralReason::EntitlementDenied, "ENTITLEMENT_BASE_REQUIRED")],
        ));
        let snapshot = metrics.snapshot();
        assert_eq!(snapshot.len(), metrics.capacity(), "fixed bounded capacity");
        assert_eq!(snapshot["dispatch.deferred.entitlement_denied.count"], 1);
        assert_eq!(snapshot["dispatch.selected.count"], 1);
        assert!(
            snapshot.keys().all(|key| {
                !key.contains("session")
                    && !key.contains("lease")
                    && !key.contains("digest")
                    && !key.contains("sha256")
                    && !key.contains("provider_item")
            }),
            "snapshot must never expose session, work-item, or lease identifiers"
        );
        assert!(snapshot.values().all(|count| *count <= 1));
    }
}
