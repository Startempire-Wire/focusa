//! Silent Session dispatch overlay for the one canonical Work Loop scheduler.

use crate::silent_session::SilentSessionId;
use crate::silent_session_resources::ResourceAdmissionDecision;
use crate::silent_session_writer::WriterAdmissionDecision;
use crate::work_item::{
    WorkItem, WorkItemQuery, WorkItemReadiness, WorkItemRef, evaluate_readiness,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::license::{
    evaluate_entitlement_execution,
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
pub struct SilentSessionDispatchCandidate {
    pub session_id: SilentSessionId,
    pub work_item: WorkItemRef,
    pub priority: SilentSessionPriority,
    pub queued_at: DateTime<Utc>,
    pub resource_admission: ResourceAdmissionDecision,
    pub writer_admission: WriterAdmissionDecision,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DispatchDeferralReason {
    WorkItemNotReady,
    ResourceAdmissionDenied,
    WriterAdmissionDenied,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SilentSessionDispatchEntitlementError {
    pub code: String,
    pub message: String,
    pub required_feature: Option<String>,
    pub limit_bucket: Option<String>,
}

fn silent_session_dispatch_entitlement_policy() -> EntitlementExecutionPolicy {
    EntitlementExecutionPolicy::new(
        "focusa.silent_session.dispatch",
        focusa_license::OperationClass::ValueMutation,
        focusa_license::CapabilityFamily::Automation,
        Some("focusa.agent.silent_sessions"),
        Some("silent_session_runs"),
        focusa_license::RecoveryAllowance::None,
    )
}

fn evaluate_silent_session_dispatch_entitlement(
    entitlement_guard: &focusa_license::LicenseGuard,
    policy: &EntitlementExecutionPolicy,
) -> Result<(), SilentSessionDispatchEntitlementError> {
    evaluate_entitlement_execution(
        entitlement_guard,
        policy,
        EntitlementExecutionContext::default(),
    )
    .map(|_| ())
    .map_err(|error| SilentSessionDispatchEntitlementError {
        code: error.code,
        message: error.message,
        required_feature: error.required_feature,
        limit_bucket: error.limit_bucket,
    })
}

pub fn select_silent_session_dispatch_with_entitlement(
    work_items: &[WorkItem],
    query: &WorkItemQuery,
    candidates: &[SilentSessionDispatchCandidate],
    entitlement_guard: &focusa_license::LicenseGuard,
    entitlement_policy: &EntitlementExecutionPolicy,
) -> Result<(WorkItemReadiness, SilentSessionDispatchDecision), SilentSessionDispatchEntitlementError> {
    evaluate_silent_session_dispatch_entitlement(entitlement_guard, entitlement_policy)?;
    Ok(select_silent_session_dispatch(work_items, query, candidates))
}

pub fn select_silent_session_dispatch_with_default_entitlement(
    work_items: &[WorkItem],
    query: &WorkItemQuery,
    candidates: &[SilentSessionDispatchCandidate],
    entitlement_guard: &focusa_license::LicenseGuard,
) -> Result<(WorkItemReadiness, SilentSessionDispatchDecision), SilentSessionDispatchEntitlementError> {
    evaluate_silent_session_dispatch_entitlement(entitlement_guard, &silent_session_dispatch_entitlement_policy())?;
    Ok(select_silent_session_dispatch(work_items, query, candidates))
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
    use focusa_license::authority::{EntitlementSnapshot, EntitlementState};
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
        }
    }

    fn signed_base_snapshot() -> focusa_license::LicenseGuard {
        let now = Utc::now();
        let mut snapshot = EntitlementSnapshot::unactivated("focusa", "node-core-scheduler");
        snapshot.state = EntitlementState::Active;
        snapshot.sequence = Some(7);
        snapshot.lease_id = Some("lease-3".into());
        snapshot.lease_digest = Some("sha256:scheduler".into());
        snapshot.expires_at = Some(now + chrono::Duration::hours(1));
        snapshot.offline_grace_until = Some(now + chrono::Duration::hours(1));
        focusa_license::LicenseGuard::from_entitlement(snapshot)
    }

    #[test]
    fn delayed_dispatch_rejects_when_entitlement_gate_denies() {
        let first = item("first", WorkItemStatus::Open, 0);
        let query = WorkItemQuery {
            project_root: PathBuf::from("/projects/focusa"),
            parent: None,
            limit: 100,
        };
        let decision = select_silent_session_dispatch_with_default_entitlement(
            &[first],
            &query,
            &[candidate(&item("first", WorkItemStatus::Open, 1), SilentSessionPriority::Normal, Utc::now())],
            &focusa_license::LicenseGuard::eval(7),
        )
        .expect_err("dispatch without base entitlement should fail");
        assert_eq!(decision.code, "ENTITLEMENT_BASE_REQUIRED");
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
}
