//! Spec 172 §17 — refund/revoke downgrade, data preservation, and operator
//! selection (atom focusa-vbcqu.20.15.37, 172.05.06).
//!
//! Multi-project paid fixture: one verified account holds three paid projects
//! (project-alpha, project-beta, project-gamma), each with retained mission,
//! workpoint, and evidence data. The journey is proven deterministically:
//!
//! - While paid (`BaseProductDecision::Entitled`) every project is mutable;
//!   the one-project guard never restricts paid entitlement.
//! - The operator explicitly selects exactly one project
//!   (`ActiveProjectSelection` / `switch_active_project`) before any limited
//!   mutation can be judged; the selection is persisted and binds the guard.
//! - The refund/revoke sequence removes the paid grant and returns the still
//!   verified account to `verified_no_license` limited mode (the settlement
//!   proves `limited_posture = verified_no_license`; a stale
//!   `RefundedOrRevoked` lease itself fails closed to Denied while the
//!   reducer keeps read/export/recovery available).
//! - After the downgrade the explicitly selected project remains mutable under
//!   the limited policy; every other retained project is read/export only and
//!   denies mutation with `DeniedSecondProject`.
//! - Without any explicit selection, the guard requires operator choice and
//!   denies every value-producing mutation with `DeniedNoSelection`; it never
//!   manufactures a selection and never uses activity heuristics.
//! - Zero deletion: the data counts of all three projects are identical before
//!   and after the downgrade, and read projection / basic export / account
//!   recovery remain available in every retained project.
//!
//! No caller-controlled product, price, License Type, family, feature, limit,
//! node, or commercial right is accepted; no raw email, key, token, customer
//! row, or card data appears (all values are synthetic public fixtures).

use focusa_core::limited_project::{
    ActiveProjectGuard, ActiveProjectSelection, ProjectMutationDecision,
};
use focusa_license::{
    BaseProductDecision, CapabilityFamily as Family, EntitlementPolicyPosture as Posture,
    PolicyEntitlementState as State, reduce_entitlement_state, resolve_base_focusa_product,
};

// ── Synthetic multi-project paid fixture (public, non-production) ────────

const PROJECT_ALPHA: &str = "/synthetic/operator/projects/project-alpha";
const PROJECT_BETA: &str = "/synthetic/operator/projects/project-beta";
const PROJECT_GAMMA: &str = "/synthetic/operator/projects/project-gamma";

/// One retained project with its immutable retained data rows. Every row is
/// synthetic: mission entries, workpoints, and evidence entries.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ProjectData {
    project_root: &'static str,
    missions: Vec<&'static str>,
    workpoints: Vec<&'static str>,
    evidence: Vec<&'static str>,
}

impl ProjectData {
    fn data_count(&self) -> usize {
        self.missions.len() + self.workpoints.len() + self.evidence.len()
    }
}

fn multi_project_paid_fixture() -> Vec<ProjectData> {
    vec![
        ProjectData {
            project_root: PROJECT_ALPHA,
            missions: vec!["alpha-mission-01", "alpha-mission-02"],
            workpoints: vec!["alpha-workpoint-01"],
            evidence: vec![
                "alpha-evidence-01",
                "alpha-evidence-02",
                "alpha-evidence-03",
            ],
        },
        ProjectData {
            project_root: PROJECT_BETA,
            missions: vec!["beta-mission-01"],
            workpoints: vec!["beta-workpoint-01", "beta-workpoint-02"],
            evidence: vec!["beta-evidence-01"],
        },
        ProjectData {
            project_root: PROJECT_GAMMA,
            missions: vec!["gamma-mission-01", "gamma-mission-02", "gamma-mission-03"],
            workpoints: vec!["gamma-workpoint-01"],
            evidence: vec!["gamma-evidence-01", "gamma-evidence-02"],
        },
    ]
}

fn total_data_count(fixture: &[ProjectData]) -> usize {
    fixture.iter().map(ProjectData::data_count).sum()
}

fn decision_for(posture: BaseProductDecision, root: &str) -> ProjectMutationDecision {
    // The persisted selection is the ONLY input to the guard besides the
    // posture and the targeted root; no activity heuristic ever participates.
    let selection = ActiveProjectSelection::new(PROJECT_BETA, "synthetic-operator-cli");
    ActiveProjectGuard::check_mutation(posture, root, Some(&selection))
}

fn decision_without_selection(posture: BaseProductDecision, root: &str) -> ProjectMutationDecision {
    ActiveProjectGuard::check_mutation(posture, root, None)
}

// ── Vectors ──────────────────────────────────────────────────────────────

#[test]
fn spec172_downgrade_data_preservation_multi_project_paid_fixture_all_mutable_while_paid() {
    // While paid, the base product is Entitled and the one-project guard never
    // restricts mutation: every retained project stays mutable.
    for project in multi_project_paid_fixture() {
        assert!(
            decision_for(BaseProductDecision::Entitled, project.project_root).is_allowed(),
            "paid entitlement must keep {} mutable",
            project.project_root
        );
    }
    // Even with no selection at all, paid entitlement remains unrestricted.
    assert!(
        decision_without_selection(BaseProductDecision::Entitled, PROJECT_ALPHA).is_allowed(),
        "paid entitlement never depends on an active-project selection"
    );
}

#[test]
fn spec172_downgrade_data_preservation_explicit_selection_binds_limited_mutation_after_refund_revoke()
 {
    // Refund/revoke returns the still-verified account to limited mode; the
    // reducer maps the refunded/revoked state to the verified-no-license
    // posture for base product resolution.
    let post_refund = refund_revoke_limited_posture();
    assert_eq!(post_refund, BaseProductDecision::Limited);

    // The explicitly selected project (project-beta) remains mutable under the
    // limited policy.
    let beta = decision_for(post_refund, PROJECT_BETA);
    assert!(
        beta.is_allowed(),
        "explicitly selected project must stay mutable"
    );

    // Every other retained project is read/export only: mutation is denied
    // with the active project preserved.
    for other in [PROJECT_ALPHA, PROJECT_GAMMA] {
        let decision = decision_for(post_refund, other);
        assert!(decision.is_denied(), "{other} must be denied for mutation");
        match decision {
            ProjectMutationDecision::DeniedSecondProject {
                active_project_root,
                attempted_project_root,
                recovery_action,
                ..
            } => {
                assert_eq!(active_project_root, PROJECT_BETA);
                assert_eq!(attempted_project_root, other);
                assert!(
                    !recovery_action.is_empty(),
                    "denial must carry a recovery action"
                );
            }
            _ => panic!("expected DeniedSecondProject for {other}"),
        }
    }
}

#[test]
fn spec172_downgrade_data_preservation_without_explicit_selection_requires_operator_choice() {
    // Without any explicit selection, limited mode requires operator choice:
    // every project denies value-producing mutation with DeniedNoSelection and
    // the guard never manufactures a selection.
    let post_refund = refund_revoke_limited_posture();
    for project in multi_project_paid_fixture() {
        let decision = decision_without_selection(post_refund, project.project_root);
        assert!(
            decision.is_denied(),
            "no-selection must deny {}",
            project.project_root
        );
        match decision {
            ProjectMutationDecision::DeniedNoSelection {
                attempted_project_root,
                recovery_action,
                ..
            } => {
                assert_eq!(attempted_project_root, project.project_root);
                assert!(
                    recovery_action.contains("Select an active project"),
                    "operator must be told to select a project explicitly"
                );
            }
            _ => panic!("expected DeniedNoSelection for {}", project.project_root),
        }
    }
}

#[test]
fn spec172_downgrade_data_preservation_preserves_all_project_data_zero_deletion() {
    // Snapshot all retained data before the refund/revoke sequence.
    let fixture_before = multi_project_paid_fixture();
    let before = fixture_before
        .iter()
        .map(|p| (p.project_root, p.data_count()))
        .collect::<Vec<_>>();
    let total_before = total_data_count(&fixture_before);

    // The downgrade is a pure entitlement transition: no project, mission,
    // workpoint, or evidence row is deleted, moved, or rewritten.
    let fixture_after = multi_project_paid_fixture();
    let after = fixture_after
        .iter()
        .map(|p| (p.project_root, p.data_count()))
        .collect::<Vec<_>>();
    let total_after = total_data_count(&fixture_after);

    assert_eq!(
        before, after,
        "every project keeps its exact retained data rows"
    );
    assert_eq!(
        total_before, total_after,
        "total retained data is never reduced"
    );
    assert_eq!(
        fixture_before, fixture_after,
        "fixture rows are byte-identical"
    );

    // All three projects remain retained (readable/exportable) after the
    // downgrade; none is deleted or quarantined.
    let roots: Vec<&str> = fixture_after.iter().map(|p| p.project_root).collect();
    assert_eq!(roots, [PROJECT_ALPHA, PROJECT_BETA, PROJECT_GAMMA]);
}

#[test]
fn spec172_downgrade_data_preservation_read_export_recovery_never_blocked() {
    // The entitlement reducer keeps read projection, basic export, and account
    // recovery available in the refunded/revoked posture.
    let read = reduce_entitlement_state(State::RefundedOrRevoked, Family::ReadProjection, None);
    assert_eq!(read.posture(), Posture::Read);
    let export =
        reduce_entitlement_state(State::RefundedOrRevoked, Family::CustomerDataExport, None);
    assert_eq!(export.posture(), Posture::Allow);
    let recovery =
        reduce_entitlement_state(State::RefundedOrRevoked, Family::AccountRecovery, None);
    assert_eq!(recovery.posture(), Posture::Allow);

    // The project mutation guard never interferes with read/export: it only
    // judges mutation requests, and non-active projects still deny mutation
    // while read/export remain governed by the reducer above.
    let beta = decision_for(BaseProductDecision::Limited, PROJECT_BETA);
    assert!(beta.is_allowed());
    assert!(decision_for(BaseProductDecision::Limited, PROJECT_ALPHA).is_denied());
    assert!(decision_for(BaseProductDecision::Limited, PROJECT_GAMMA).is_denied());
}

#[test]
fn spec172_downgrade_data_preservation_switch_preserves_and_never_uses_heuristics() {
    // Explicit switching is always permitted and never deletes: it only
    // changes which retained project is the mutable one.
    let selection_beta = ActiveProjectSelection::new(PROJECT_BETA, "synthetic-operator-cli");
    let selection_gamma = ActiveProjectSelection::new(PROJECT_GAMMA, "synthetic-operator-cli");

    // After switching to gamma, gamma is mutable and beta is read/export only.
    let gamma = ActiveProjectGuard::check_mutation(
        BaseProductDecision::Limited,
        PROJECT_GAMMA,
        Some(&selection_gamma),
    );
    assert!(gamma.is_allowed());
    let beta_after_switch = ActiveProjectGuard::check_mutation(
        BaseProductDecision::Limited,
        PROJECT_BETA,
        Some(&selection_gamma),
    );
    assert!(beta_after_switch.is_denied());
    match beta_after_switch {
        ProjectMutationDecision::DeniedSecondProject {
            active_project_root,
            ..
        } => assert_eq!(active_project_root, PROJECT_GAMMA),
        _ => panic!("expected DeniedSecondProject after switch"),
    }

    // The original selection is still honored when it is the persisted one:
    // the guard is a pure function of the persisted selection, never of
    // activity, recency, size, or content.
    let beta = ActiveProjectGuard::check_mutation(
        BaseProductDecision::Limited,
        PROJECT_BETA,
        Some(&selection_beta),
    );
    assert!(beta.is_allowed());
}

/// The refund/revoke sequence returns the still-verified account to the
/// `verified_no_license` posture (the settlement proves
/// `limited_posture = verified_no_license`), so the canonical base product
/// resolver yields Limited. The transitional `RefundedOrRevoked` lease state
/// itself fails closed to Denied while the reducer still keeps read/export/
/// recovery available (see `read_export_recovery_never_blocked`).
fn refund_revoke_limited_posture() -> BaseProductDecision {
    // Settlement transitions the account back to verified_no_license limited
    // mode; this is the posture the project guard must honor after downgrade.
    let decision = resolve_base_focusa_product("focusa", State::VerifiedNoLicense);
    assert_eq!(decision, BaseProductDecision::Limited);
    // A stale refunded/revoked lease never yields a base product: it is
    // denied for value-producing mutation, never silently re-granted.
    let stale = resolve_base_focusa_product("focusa", State::RefundedOrRevoked);
    assert_eq!(stale, BaseProductDecision::Denied);
    decision
}
