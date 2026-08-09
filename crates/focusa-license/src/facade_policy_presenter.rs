//! Facade policy presenter contract (Spec 152F §7 branded-facade row; P5, P9).
//!
//! Branded facades and Focusa website presenters display the canonical
//! authority decision and nothing else: the capability family (the base
//! product or one of the four premium families), the
//! Evaluation/purchase/recovery action, and a safe masked status derived from
//! authority output. They cannot select grants, prices, feature activation, or
//! runtime policy, and they cannot turn dormant future-granularity or premium
//! dimensions on or off.
//!
//! This module is the typed contract those surfaces bind to. The projection
//! accepts only the canonical decision (`EntitlementStateDecision` +
//! `CapabilityFamily` from the authority reducer) plus an authority status
//! label; it exposes no product, price, grant, feature, limit, or
//! runtime-policy input, has no setters, and fails closed on unknown families,
//! postures, and statuses. Exact-origin, session, and redirect rules remain
//! Spec 152E-owned (`activation_facade`, facade registry, facade security).

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::entitlement_policy::{
    CapabilityFamily, DecisionReason, EntitlementPolicyPosture, EntitlementStateDecision,
};

/// The frozen set of exactly what a branded facade may present. Anything else
/// is a presenter-owned commercial decision and is forbidden by construction.
pub const FACADE_PRESENTER_FIELDS: [&str; 8] = [
    "family",
    "posture",
    "action",
    "action_label",
    "explanation",
    "recovery_action",
    "masked_status",
    "always_reachable",
];

/// Caller/authority-envelope fields a facade presenter must never accept or
/// emit: selecting grants, prices, feature activation, runtime policy, or
/// dormant granularity dimensions is prohibited (Spec 152F P9, §9, §10).
pub const FACADE_PRESENTER_FORBIDDEN_FIELDS: [&str; 16] = [
    "grants",
    "prices",
    "price",
    "feature_activation",
    "runtime_policy",
    "dormant",
    "product_selection",
    "product_code",
    "limit_bucket",
    "limits",
    "lease",
    "tokens",
    "keys",
    "customer_email",
    "raw_status",
    "redirect_url",
];

/// Frozen always-reachable surface families shared with the menubar and TUI
/// presenters (Spec 152F P6). Facades present the same set; a denied
/// entitlement decision never disables navigation, status, account, read,
/// export, recovery, repair, update, or uninstall.
pub const FACADE_ALWAYS_REACHABLE: [&str; 9] = [
    "navigation",
    "status",
    "account",
    "read",
    "export",
    "recovery",
    "repair",
    "update",
    "uninstall",
];

/// Authority status labels a facade may present as its masked status. Unknown
/// or spoofed status strings fail closed (never rendered verbatim).
pub const FACADE_STATUS_ALLOWLIST: [&str; 16] = [
    // Spec 172 policy entitlement states
    "pending_unverified",
    "verified_no_license",
    "active_paid",
    "offline_grace",
    "expired",
    "refunded_or_revoked",
    "missing_or_corrupt",
    // Spec 152E presenter states
    "email_verification_pending",
    "email_verified",
    "selection_required",
    "checkout_required",
    "payment_pending",
    "license_delivery_ready",
    "activated",
    "denied",
    "recovery_only",
];

/// The capability family a branded facade may explain. Facades present the
/// base product or one of the four premium families; account recovery, read
/// projection, and customer-data export are always-reachable allowances and
/// are never sold or pitched as a premium. Internal maintenance has no
/// facade-presentable family — it inherits the initiating operation's family,
/// resolved by the authority, never by a page.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FacadeFamily {
    BaseFocusa,
    PremiumAutomation,
    PremiumTeamRemote,
    PremiumReleaseProof,
    PremiumUpdates,
    AlwaysReachable,
}

impl FacadeFamily {
    pub const fn label(self) -> &'static str {
        match self {
            Self::BaseFocusa => "base_focusa",
            Self::PremiumAutomation => "automation",
            Self::PremiumTeamRemote => "team_remote",
            Self::PremiumReleaseProof => "release_proof",
            Self::PremiumUpdates => "premium_updates",
            Self::AlwaysReachable => "always_reachable",
        }
    }

    /// Branded-page display name (presentation vocabulary only).
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::BaseFocusa => "Base Focusa",
            Self::PremiumAutomation => "Automation",
            Self::PremiumTeamRemote => "Team and remote",
            Self::PremiumReleaseProof => "Release proof",
            Self::PremiumUpdates => "Premium updates",
            Self::AlwaysReachable => "Always reachable",
        }
    }

    pub const fn is_premium(self) -> bool {
        matches!(
            self,
            Self::PremiumAutomation
                | Self::PremiumTeamRemote
                | Self::PremiumReleaseProof
                | Self::PremiumUpdates
        )
    }
}

/// Map the canonical capability family to the facade-presentable family.
/// `InternalMaintenance` is not facade-presentable: the authority resolves its
/// initiating operation's family, and a page must never present it as its own
/// product decision (fail closed).
pub fn facade_family(family: CapabilityFamily) -> Result<FacadeFamily, FacadePresenterError> {
    match family {
        CapabilityFamily::BaseFocusa => Ok(FacadeFamily::BaseFocusa),
        CapabilityFamily::Automation => Ok(FacadeFamily::PremiumAutomation),
        CapabilityFamily::TeamRemote => Ok(FacadeFamily::PremiumTeamRemote),
        CapabilityFamily::ReleaseProof => Ok(FacadeFamily::PremiumReleaseProof),
        CapabilityFamily::PremiumUpdates => Ok(FacadeFamily::PremiumUpdates),
        CapabilityFamily::AccountRecovery
        | CapabilityFamily::ReadProjection
        | CapabilityFamily::CustomerDataExport => Ok(FacadeFamily::AlwaysReachable),
        CapabilityFamily::InternalMaintenance => Err(FacadePresenterError::FamilyNotPresentable),
    }
}

/// The Evaluation/purchase/recovery action a branded facade may show. Derived
/// from the canonical authority decision; the facade never chooses it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FacadeNextAction {
    Evaluate,
    Purchase,
    Recovery,
    Manage,
}

impl FacadeNextAction {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Evaluate => "evaluate",
            Self::Purchase => "purchase",
            Self::Recovery => "recovery",
            Self::Manage => "manage",
        }
    }

    pub const fn display_label(self) -> &'static str {
        match self {
            Self::Evaluate => "Evaluate Focusa",
            Self::Purchase => "Purchase or renew entitlement",
            Self::Recovery => "Continue recovery",
            Self::Manage => "Manage entitlement",
        }
    }
}

/// Canonical posture-to-action projection. Denied decisions always show the
/// Evaluation/purchase action; feature-gated decisions show the offer/manage
/// action; usable and read/allow decisions show manage. This mirrors the
/// frozen menubar/TUI action vocabulary and never re-decides policy per page.
pub fn facade_next_action_for_posture(posture: EntitlementPolicyPosture) -> FacadeNextAction {
    match posture {
        EntitlementPolicyPosture::Deny => FacadeNextAction::Evaluate,
        EntitlementPolicyPosture::Feature => FacadeNextAction::Purchase,
        _ => FacadeNextAction::Manage,
    }
}

/// Status-based override: a `recovery_only` authority status always shows the
/// recovery action; any other status leaves the posture-derived action intact.
pub fn facade_next_action_for_status(status: &str) -> FacadeNextAction {
    if status == "recovery_only" {
        FacadeNextAction::Recovery
    } else {
        FacadeNextAction::Manage
    }
}

/// One bounded, presenter-safe view of the canonical authority decision. Every
/// field is a static, authority-derived projection; there are no booleans, no
/// setters, and no grant/price/feature/runtime-policy selectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FacadePolicyDecision {
    family: FacadeFamily,
    posture: EntitlementPolicyPosture,
    action: FacadeNextAction,
    action_label: &'static str,
    explanation: &'static str,
    recovery_action: &'static str,
}

impl FacadePolicyDecision {
    /// Project the canonical decision into the facade-presentable view.
    /// `status` is the authority status label (e.g. `recovery_only`); it is
    /// only consulted for the recovery override, never rendered raw.
    pub fn project(
        decision: EntitlementStateDecision,
        family: CapabilityFamily,
        status: &str,
    ) -> Result<Self, FacadePresenterError> {
        let family = facade_family(family)?;
        let posture = decision.posture();
        let action = match facade_next_action_for_status(status) {
            FacadeNextAction::Recovery => FacadeNextAction::Recovery,
            _ => facade_next_action_for_posture(posture),
        };
        let recovery_action = decision.reason().recovery_action();
        Ok(Self {
            family,
            posture,
            action,
            action_label: action.display_label(),
            explanation: explanation(family, posture),
            recovery_action,
        })
    }

    pub const fn family(self) -> FacadeFamily {
        self.family
    }

    pub const fn posture(self) -> EntitlementPolicyPosture {
        self.posture
    }

    pub const fn action(self) -> FacadeNextAction {
        self.action
    }

    pub const fn action_label(self) -> &'static str {
        self.action_label
    }

    pub const fn explanation(self) -> &'static str {
        self.explanation
    }

    pub const fn recovery_action(self) -> &'static str {
        self.recovery_action
    }
}

/// Safe masked status from authority output: a bounded status label plus an
/// already-masked identity. Raw statuses, raw identities, secrets, and
/// customer PII cannot be represented by construction.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct FacadeMaskedStatus {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub masked_email: Option<String>,
}

/// Build a safe masked status. The status must be an authority-recognized
/// label (`FACADE_STATUS_ALLOWLIST`) and, when present, the identity must
/// already match the frozen masked pattern `^[^@]*\*[^@]*@[^@]+$`. Any other
/// input fails closed with `None` so a facade never renders a raw value.
pub fn safe_masked_status(status: &str, masked_email: Option<&str>) -> Option<FacadeMaskedStatus> {
    if !FACADE_STATUS_ALLOWLIST.contains(&status) {
        return None;
    }
    let masked_email = match masked_email {
        Some(value) if looks_masked(value) => Some(value.to_string()),
        Some(_) => return None,
        None => None,
    };
    Some(FacadeMaskedStatus {
        status: status.to_string(),
        masked_email,
    })
}

/// Frozen masked-identity check: `^[^@]*\*[^@]*@[^@]+$` (same semantics as
/// the activation envelope).
fn looks_masked(value: &str) -> bool {
    let Some((local, domain)) = value.split_once('@') else {
        return false;
    };
    if local.is_empty() || domain.is_empty() || domain.contains('@') || domain.contains('*') {
        return false;
    }
    local.contains('*')
}

/// Bounded static explanation for one (family, posture) pair. Presentation
/// vocabulary only; it never re-decides the authority decision.
fn explanation(family: FacadeFamily, posture: EntitlementPolicyPosture) -> &'static str {
    use EntitlementPolicyPosture as Posture;
    use FacadeFamily as Family;
    match (family, posture) {
        (Family::BaseFocusa, Posture::Allow) => {
            "A verified Evaluation or paid Focusa entitlement enables the complete base Focusa value loop."
        }
        (Family::BaseFocusa, Posture::Read) => {
            "Read-only projection is available for existing local data."
        }
        (Family::BaseFocusa, Posture::Base) => {
            "A valid Evaluation, Active paid lease, or valid Offline Grace enables the complete base Focusa value loop."
        }
        (Family::BaseFocusa, _) => {
            "A verified Evaluation or paid Focusa entitlement is required for value-producing Focusa work."
        }
        (
            Family::PremiumAutomation
            | Family::PremiumTeamRemote
            | Family::PremiumReleaseProof
            | Family::PremiumUpdates,
            Posture::Feature,
        ) => {
            "This optional premium family requires an authority-issued entitlement; this branded page cannot grant it."
        }
        (
            Family::PremiumAutomation
            | Family::PremiumTeamRemote
            | Family::PremiumReleaseProof
            | Family::PremiumUpdates,
            _,
        ) => "This optional premium family is not available in the current entitlement state.",
        (Family::AlwaysReachable, _) => {
            "Account recovery, read, export, repair, and uninstall remain available when execution is locked."
        }
    }
}

/// Typed failure for facade-presenter contract violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Error)]
pub enum FacadePresenterError {
    #[error("internal maintenance has no facade-presentable family; present the initiating operation's family")]
    FamilyNotPresentable,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entitlement_policy::{reduce_entitlement_state, PolicyEntitlementState};

    fn decision(state: PolicyEntitlementState, family: CapabilityFamily) -> EntitlementStateDecision {
        reduce_entitlement_state(state, family, None)
    }

    #[test]
    fn base_family_projects_the_canonical_base_decision() {
        let view = FacadePolicyDecision::project(
            decision(PolicyEntitlementState::ActivePaid, CapabilityFamily::BaseFocusa),
            CapabilityFamily::BaseFocusa,
            "active_paid",
        )
        .expect("project");
        assert_eq!(view.family(), FacadeFamily::BaseFocusa);
        assert_eq!(view.posture(), EntitlementPolicyPosture::Base);
        assert_eq!(view.action(), FacadeNextAction::Manage);
        assert_eq!(view.action_label(), "Manage entitlement");
        assert_eq!(view.recovery_action(), "activate_evaluation_purchase_or_manage_entitlement");
    }

    #[test]
    fn denied_base_decision_projects_evaluate_action() {
        let view = FacadePolicyDecision::project(
            decision(PolicyEntitlementState::Expired, CapabilityFamily::BaseFocusa),
            CapabilityFamily::BaseFocusa,
            "expired",
        )
        .expect("project");
        assert_eq!(view.posture(), EntitlementPolicyPosture::Deny);
        assert_eq!(view.action(), FacadeNextAction::Evaluate);
        assert_eq!(view.action_label(), "Evaluate Focusa");
    }

    #[test]
    fn premium_family_projects_purchase_or_deny_and_is_premium() {
        let granted = FacadePolicyDecision::project(
            decision(PolicyEntitlementState::ActivePaid, CapabilityFamily::Automation),
            CapabilityFamily::Automation,
            "active_paid",
        )
        .expect("project");
        assert_eq!(granted.family(), FacadeFamily::PremiumAutomation);
        assert!(granted.family().is_premium());
        assert_eq!(granted.posture(), EntitlementPolicyPosture::Feature);
        assert_eq!(granted.action(), FacadeNextAction::Purchase);

        let denied = FacadePolicyDecision::project(
            decision(PolicyEntitlementState::Expired, CapabilityFamily::ReleaseProof),
            CapabilityFamily::ReleaseProof,
            "expired",
        )
        .expect("project");
        assert_eq!(denied.posture(), EntitlementPolicyPosture::Deny);
        assert_eq!(denied.action(), FacadeNextAction::Evaluate);
        for family in [
            CapabilityFamily::Automation,
            CapabilityFamily::TeamRemote,
            CapabilityFamily::ReleaseProof,
            CapabilityFamily::PremiumUpdates,
        ] {
            let mapped = facade_family(family).expect("premium family maps");
            assert!(mapped.is_premium());
        }
    }

    #[test]
    fn always_reachable_families_never_sell_or_pitch_premium() {
        for family in [
            CapabilityFamily::AccountRecovery,
            CapabilityFamily::ReadProjection,
            CapabilityFamily::CustomerDataExport,
        ] {
            let view = FacadePolicyDecision::project(
                decision(PolicyEntitlementState::RefundedOrRevoked, family),
                family,
                "refunded_or_revoked",
            )
            .expect("project");
            assert_eq!(view.family(), FacadeFamily::AlwaysReachable);
            assert!(!view.family().is_premium());
        }
    }

    #[test]
    fn recovery_only_status_always_shows_recovery_action() {
        let view = FacadePolicyDecision::project(
            decision(PolicyEntitlementState::Expired, CapabilityFamily::BaseFocusa),
            CapabilityFamily::BaseFocusa,
            "recovery_only",
        )
        .expect("project");
        assert_eq!(view.action(), FacadeNextAction::Recovery);
        assert_eq!(view.action_label(), "Continue recovery");
    }

    #[test]
    fn internal_maintenance_is_not_facade_presentable() {
        assert_eq!(
            facade_family(CapabilityFamily::InternalMaintenance),
            Err(FacadePresenterError::FamilyNotPresentable)
        );
        assert!(FacadePolicyDecision::project(
            decision(PolicyEntitlementState::ActivePaid, CapabilityFamily::InternalMaintenance),
            CapabilityFamily::InternalMaintenance,
            "active_paid",
        )
        .is_err());
    }

    #[test]
    fn masked_status_accepts_only_authority_labels_and_masked_identities() {
        let masked = safe_masked_status("recovery_only", Some("c***@example.com"));
        assert_eq!(
            masked,
            Some(FacadeMaskedStatus {
                status: "recovery_only".to_string(),
                masked_email: Some("c***@example.com".to_string()),
            })
        );
        assert!(safe_masked_status("recovery_only", None).is_some());
        // Unknown status fails closed.
        assert_eq!(
            safe_masked_status("spoofed_status", Some("c***@example.com")),
            None
        );
        // Raw identity fails closed.
        assert_eq!(safe_masked_status("active_paid", Some("customer@example.com")), None);
    }

    #[test]
    fn presenter_field_contract_is_frozen_and_forbids_commercial_selectors() {
        assert_eq!(FACADE_PRESENTER_FIELDS.len(), 8);
        assert_eq!(FACADE_PRESENTER_FORBIDDEN_FIELDS.len(), 16);
        assert_eq!(FACADE_ALWAYS_REACHABLE.len(), 9);
        assert!(FACADE_PRESENTER_FORBIDDEN_FIELDS.contains(&"grants"));
        assert!(FACADE_PRESENTER_FORBIDDEN_FIELDS.contains(&"prices"));
        assert!(FACADE_PRESENTER_FORBIDDEN_FIELDS.contains(&"feature_activation"));
        assert!(FACADE_PRESENTER_FORBIDDEN_FIELDS.contains(&"runtime_policy"));
        assert!(FACADE_PRESENTER_FORBIDDEN_FIELDS.contains(&"dormant"));
        // The presenter fields and the forbidden fields never overlap.
        for field in FACADE_PRESENTER_FIELDS {
            assert!(!FACADE_PRESENTER_FORBIDDEN_FIELDS.contains(&field));
        }
    }
}
