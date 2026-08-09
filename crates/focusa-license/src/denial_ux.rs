//! Denial, purchase, and recovery UX registry (Spec 152F §10.8, §11, P6).
//!
//! One stable, cross-presenter message catalog: for each authority state and
//! capability family, a plain-language blocked action, a reason, the retained
//! always-reachable access, and exactly ONE safe next action with a stable
//! account/evaluation/checkout/recovery link. Messages never contain internal
//! route/lease details, false urgency, account enumeration, raw
//! email/key/token material, or dead-end paywalls.
//!
//! This module is the typed contract: `DenialUxErrorCode` is a closed enum of
//! stable error codes; `denial_ux_message_for` derives the canonical message
//! from the authority reducer; `embedded_denial_ux_catalog` loads the
//! committed cross-presenter catalog JSON (the same artifact the website and
//! Pi fixtures bind) so the typed derivation can be proven identical to the
//! artifact. Unknown codes, states, families, and links fail closed (never
//! rendered). Presenters improve explanation only; they never re-decide
//! policy, grants, prices, or recovery routes.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::OnceLock;
use thiserror::Error;

use crate::entitlement_policy::{
    CapabilityFamily, DecisionReason, EntitlementPolicyPosture, PolicyEntitlementState,
    reduce_entitlement_state,
};

/// Schema of the committed cross-presenter catalog artifact.
pub const DENIAL_UX_SCHEMA: &str = "focusa.spec152f.denial_ux_catalog.v1";

/// The committed catalog JSON is embedded so the typed contract and the
/// cross-presenter artifact are the same bytes by construction.
pub const DENIAL_UX_CATALOG_JSON: &str =
    include_str!("../../../docs/contracts/spec152f-denial-ux-catalog.v1.json");

/// Frozen retained-access set: the always-reachable allowances that survive a
/// commercial denial (Spec 152F P6). Identical to the menubar/TUI/facade
/// always-reachable fixtures and the catalog artifact.
pub const RETAINED_ACCESS: [&str; 9] = [
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

/// Frozen plain-language rules every catalog message must satisfy. Operator
/// metadata; customer-visible strings are enforced separately.
pub const PUBLIC_MESSAGE_RULES: [&str; 7] = [
    "plain language blocked action and reason",
    "retained access is always listed",
    "exactly one safe next action per message",
    "no internal route or lease details",
    "no false urgency",
    "no account enumeration or raw email/key/token",
    "no dead-end paywalls; every denial preserves a route to purchase or recovery",
];

/// Stable link ids for account/evaluation/checkout/recovery routes.
pub const DENIAL_UX_LINK_IDS: [&str; 4] = ["account", "evaluation", "checkout", "recovery"];

/// Stable relative link paths (no tokens, no emails, no absolute redirects).
pub const LINK_ACCOUNT: &str = "/account";
pub const LINK_EVALUATION: &str = "/activate/evaluate";
pub const LINK_CHECKOUT: &str = "/activate/checkout";
pub const LINK_RECOVERY: &str = "/activate/recovery";

/// Frozen action ids used by catalog messages.
pub const DENIAL_UX_ACTIONS: [&str; 7] = [
    "continue",
    "evaluate",
    "purchase",
    "recovery",
    "manage",
    "verify_identity",
    "diagnostics",
];

/// Stable public messages, one per stable error code. These strings are
/// byte-identical to `error_registry[].public_message` in the catalog JSON.
pub const MSG_BASE_REQUIRED: &str = "A verified Evaluation or paid Focusa entitlement is required for value-producing Focusa work. Registration, reading, export, recovery, repair, updates, and uninstall remain available.";
pub const MSG_FEATURE_REQUIRED: &str = "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.";
pub const MSG_REQUIRED: &str = "A usable authority-issued Focusa entitlement is required for this operation. Registration, reading, export, recovery, repair, updates, and uninstall remain available.";
pub const MSG_LIMIT_EXHAUSTED: &str = "The authority-granted capacity for this operation is unavailable or exhausted. Manage capacity or retry after settlement.";
pub const MSG_RECOVERY_ONLY: &str = "Your account is in recovery mode. Account, reading, export, recovery, repair, updates, and uninstall remain available.";
pub const MSG_SNAPSHOT_MISSING: &str = "Entitlement status is unavailable right now. Refresh status or run diagnostics; recovery remains available.";
pub const MSG_ROUTE_UNCLASSIFIED: &str = "This operation has no registered entitlement classification and is blocked before execution. Run diagnostics or update policy.";
pub const MSG_POLICY_UNKNOWN: &str = "The entitlement policy is unavailable or unrecognized and this operation is blocked. Run diagnostics or update policy.";
pub const MSG_RESERVATION_FAILED: &str = "Licensed capacity could not be reserved safely before execution. Retry with the same request or run diagnostics.";
pub const MSG_IDEMPOTENCY_REQUIRED: &str = "A stable request identifier is required before reserving licensed capacity. Retry with the same request identifier.";

/// Frozen action labels (presentation vocabulary only).
pub const ACTION_LABELS: [(&str, &str); 7] = [
    ("continue", "Continue"),
    ("evaluate", "Start a free Evaluation or purchase Focusa"),
    ("purchase", "Purchase or renew this optional family"),
    ("recovery", "Continue recovery"),
    ("manage", "Manage entitlement"),
    ("verify_identity", "Verify your account"),
    ("diagnostics", "Run diagnostics or update policy"),
];

/// Additional action labels used by specific registry entries.
pub const ACTION_LABEL_MANAGE_CAPACITY: &str = "Manage capacity or retry after settlement";
pub const ACTION_LABEL_REFRESH_DIAGNOSTICS: &str = "Refresh status or run diagnostics";
pub const ACTION_LABEL_RETRY_DIAGNOSTICS: &str =
    "Retry with the same request or run diagnostics";
pub const ACTION_LABEL_RETRY_IDENTIFIER: &str = "Retry with the same request identifier";

/// Stable relative link lookup. Unknown ids fail closed with `None` so a
/// presenter can never render a caller-chosen or absolute redirect.
pub fn denial_ux_link(id: &str) -> Option<&'static str> {
    match id {
        "account" => Some(LINK_ACCOUNT),
        "evaluation" => Some(LINK_EVALUATION),
        "checkout" => Some(LINK_CHECKOUT),
        "recovery" => Some(LINK_RECOVERY),
        _ => None,
    }
}

/// Stable action label lookup; unknown actions fail closed.
pub fn denial_ux_action_label(action: &str) -> Option<&'static str> {
    match action {
        "continue" => Some("Continue"),
        "evaluate" => Some("Start a free Evaluation or purchase Focusa"),
        "purchase" => Some("Purchase or renew this optional family"),
        "recovery" => Some("Continue recovery"),
        "manage" => Some("Manage entitlement"),
        "verify_identity" => Some("Verify your account"),
        "diagnostics" => Some("Run diagnostics or update policy"),
        _ => None,
    }
}

/// Stable error codes of the cross-presenter registry. Unknown codes have no
/// typed variant and therefore cannot be produced or consumed; they fail
/// closed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DenialUxErrorCode {
    BaseRequired,
    FeatureRequired,
    Required,
    LimitExhausted,
    RecoveryOnly,
    SnapshotMissing,
    RouteUnclassified,
    PolicyUnknown,
    ReservationFailed,
    IdempotencyRequired,
}

impl DenialUxErrorCode {
    pub const ALL: [DenialUxErrorCode; 10] = [
        Self::BaseRequired,
        Self::FeatureRequired,
        Self::Required,
        Self::LimitExhausted,
        Self::RecoveryOnly,
        Self::SnapshotMissing,
        Self::RouteUnclassified,
        Self::PolicyUnknown,
        Self::ReservationFailed,
        Self::IdempotencyRequired,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::BaseRequired => "ENTITLEMENT_BASE_REQUIRED",
            Self::FeatureRequired => "ENTITLEMENT_FEATURE_REQUIRED",
            Self::Required => "ENTITLEMENT_REQUIRED",
            Self::LimitExhausted => "ENTITLEMENT_LIMIT_EXHAUSTED",
            Self::RecoveryOnly => "ENTITLEMENT_RECOVERY_ONLY",
            Self::SnapshotMissing => "ENTITLEMENT_SNAPSHOT_MISSING",
            Self::RouteUnclassified => "ENTITLEMENT_ROUTE_UNCLASSIFIED",
            Self::PolicyUnknown => "ENTITLEMENT_POLICY_UNKNOWN",
            Self::ReservationFailed => "ENTITLEMENT_RESERVATION_FAILED",
            Self::IdempotencyRequired => "ENTITLEMENT_IDEMPOTENCY_REQUIRED",
        }
    }

    /// Fail-closed label lookup: unknown strings never map to a typed code.
    pub fn from_label(label: &str) -> Option<Self> {
        Self::ALL.iter().copied().find(|code| code.label() == label)
    }

    pub const fn spec(self) -> DenialUxErrorSpec {
        match self {
            Self::BaseRequired => DenialUxErrorSpec {
                code: "ENTITLEMENT_BASE_REQUIRED",
                category: "authority",
                http_status: 403,
                retryable: false,
                public_message: MSG_BASE_REQUIRED,
                safe_next_action: "evaluate",
                action_label: "Start a free Evaluation or purchase Focusa",
                link: "evaluation",
            },
            Self::FeatureRequired => DenialUxErrorSpec {
                code: "ENTITLEMENT_FEATURE_REQUIRED",
                category: "feature",
                http_status: 403,
                retryable: false,
                public_message: MSG_FEATURE_REQUIRED,
                safe_next_action: "purchase",
                action_label: "Purchase or renew this optional family",
                link: "checkout",
            },
            Self::Required => DenialUxErrorSpec {
                code: "ENTITLEMENT_REQUIRED",
                category: "authority",
                http_status: 403,
                retryable: false,
                public_message: MSG_REQUIRED,
                safe_next_action: "evaluate",
                action_label: "Start a free Evaluation or purchase Focusa",
                link: "evaluation",
            },
            Self::LimitExhausted => DenialUxErrorSpec {
                code: "ENTITLEMENT_LIMIT_EXHAUSTED",
                category: "limit",
                http_status: 429,
                retryable: false,
                public_message: MSG_LIMIT_EXHAUSTED,
                safe_next_action: "manage",
                action_label: ACTION_LABEL_MANAGE_CAPACITY,
                link: "account",
            },
            Self::RecoveryOnly => DenialUxErrorSpec {
                code: "ENTITLEMENT_RECOVERY_ONLY",
                category: "recovery",
                http_status: 403,
                retryable: false,
                public_message: MSG_RECOVERY_ONLY,
                safe_next_action: "recovery",
                action_label: "Continue recovery",
                link: "recovery",
            },
            Self::SnapshotMissing => DenialUxErrorSpec {
                code: "ENTITLEMENT_SNAPSHOT_MISSING",
                category: "authority",
                http_status: 503,
                retryable: true,
                public_message: MSG_SNAPSHOT_MISSING,
                safe_next_action: "diagnostics",
                action_label: ACTION_LABEL_REFRESH_DIAGNOSTICS,
                link: "recovery",
            },
            Self::RouteUnclassified => DenialUxErrorSpec {
                code: "ENTITLEMENT_ROUTE_UNCLASSIFIED",
                category: "classification",
                http_status: 403,
                retryable: false,
                public_message: MSG_ROUTE_UNCLASSIFIED,
                safe_next_action: "diagnostics",
                action_label: "Run diagnostics or update policy",
                link: "recovery",
            },
            Self::PolicyUnknown => DenialUxErrorSpec {
                code: "ENTITLEMENT_POLICY_UNKNOWN",
                category: "policy",
                http_status: 403,
                retryable: false,
                public_message: MSG_POLICY_UNKNOWN,
                safe_next_action: "diagnostics",
                action_label: "Run diagnostics or update policy",
                link: "recovery",
            },
            Self::ReservationFailed => DenialUxErrorSpec {
                code: "ENTITLEMENT_RESERVATION_FAILED",
                category: "reservation",
                http_status: 503,
                retryable: true,
                public_message: MSG_RESERVATION_FAILED,
                safe_next_action: "diagnostics",
                action_label: ACTION_LABEL_RETRY_DIAGNOSTICS,
                link: "recovery",
            },
            Self::IdempotencyRequired => DenialUxErrorSpec {
                code: "ENTITLEMENT_IDEMPOTENCY_REQUIRED",
                category: "idempotency",
                http_status: 428,
                retryable: true,
                public_message: MSG_IDEMPOTENCY_REQUIRED,
                safe_next_action: "manage",
                action_label: ACTION_LABEL_RETRY_IDENTIFIER,
                link: "account",
            },
        }
    }
}

/// One frozen, presenter-safe error spec. Fields are all static, authority
/// derived; there are no setters and no commercial selectors.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DenialUxErrorSpec {
    pub code: &'static str,
    pub category: &'static str,
    pub http_status: u16,
    pub retryable: bool,
    pub public_message: &'static str,
    pub safe_next_action: &'static str,
    pub action_label: &'static str,
    pub link: &'static str,
}

/// Catalog message kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DenialUxKind {
    Available,
    Limited,
    Feature,
    Denied,
}

/// One canonical denial/purchase/recovery message. Retained access is always
/// the frozen always-reachable set; exactly one safe next action and one
/// stable link are always present.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct DenialUxMessage {
    kind: DenialUxKind,
    code: Option<DenialUxErrorCode>,
    blocked_action: &'static str,
    reason: &'static str,
    retained_access: [&'static str; 9],
    safe_next_action: &'static str,
    action_label: &'static str,
    link: &'static str,
}

impl DenialUxMessage {
    const fn new(
        kind: DenialUxKind,
        code: Option<DenialUxErrorCode>,
        blocked_action: &'static str,
        reason: &'static str,
        safe_next_action: &'static str,
        action_label: &'static str,
        link: &'static str,
    ) -> Self {
        Self {
            kind,
            code,
            blocked_action,
            reason,
            retained_access: RETAINED_ACCESS,
            safe_next_action,
            action_label,
            link,
        }
    }

    pub const fn kind(&self) -> DenialUxKind {
        self.kind
    }

    pub const fn code(&self) -> Option<DenialUxErrorCode> {
        self.code
    }

    pub const fn blocked_action(&self) -> &'static str {
        self.blocked_action
    }

    pub const fn reason(&self) -> &'static str {
        self.reason
    }

    pub const fn retained_access(&self) -> &[&'static str; 9] {
        &self.retained_access
    }

    pub const fn safe_next_action(&self) -> &'static str {
        self.safe_next_action
    }

    pub const fn action_label(&self) -> &'static str {
        self.action_label
    }

    pub const fn link(&self) -> &'static str {
        self.link
    }

    /// Every denied/limited/feature message preserves a route to purchase or
    /// recovery: a stable link plus one safe next action are always present.
    pub fn preserves_route(&self) -> bool {
        match denial_ux_link(self.link) {
            Some(_) => !self.safe_next_action.is_empty(),
            None => false,
        }
    }
}

const RETAINED_SENTENCE: &str =
    "Registration, reading, export, recovery, repair, updates, and uninstall remain available.";

/// Plain-language blocked-action description per capability family.
pub fn blocked_action_for_family(family: CapabilityFamily) -> &'static str {
    match family {
        CapabilityFamily::AccountRecovery => "Account, recovery, repair, and uninstall actions",
        CapabilityFamily::ReadProjection => "Reading your existing local data",
        CapabilityFamily::BaseFocusa => {
            "Creating or changing projects, missions, Focus State, Workpoints, Trajectories, and evidence"
        }
        CapabilityFamily::Automation => "Silent sessions, scheduled, parallel, and unattended work",
        CapabilityFamily::TeamRemote => "Adding devices, peers, and remote collaboration",
        CapabilityFamily::ReleaseProof => "Release orchestration and governed proof bundles",
        CapabilityFamily::PremiumUpdates => "Unattended and preview or nightly updates",
        CapabilityFamily::CustomerDataExport => "Exporting your own customer data",
        CapabilityFamily::InternalMaintenance => "Background maintenance work",
    }
}

/// Availability reason per always-reachable/read/base family.
pub fn available_reason_for_family(family: CapabilityFamily) -> &'static str {
    match family {
        CapabilityFamily::AccountRecovery => {
            "Account, recovery, repair, update, and uninstall actions remain available in every entitlement state."
        }
        CapabilityFamily::ReadProjection => {
            "Read-only projection of your existing local data remains available."
        }
        CapabilityFamily::BaseFocusa => {
            "A verified Evaluation or paid Focusa entitlement enables the complete base Focusa value loop."
        }
        CapabilityFamily::CustomerDataExport => {
            "You always retain access to your own data, including export, even when execution is locked."
        }
        CapabilityFamily::InternalMaintenance => {
            "Background maintenance inherits the initiating operation's entitlement decision."
        }
        CapabilityFamily::Automation
        | CapabilityFamily::TeamRemote
        | CapabilityFamily::ReleaseProof
        | CapabilityFamily::PremiumUpdates => {
            "This optional family requires an additional authority-issued grant."
        }
    }
}

/// Derive the canonical catalog message for one (state, family) cell from the
/// authority reducer. The derived message is identical to the committed
/// catalog artifact (proven by unit test over all 63 cells).
pub fn denial_ux_message_for(
    state: PolicyEntitlementState,
    family: CapabilityFamily,
) -> DenialUxMessage {
    use DecisionReason as Reason;
    use EntitlementPolicyPosture as Posture;
    let decision = reduce_entitlement_state(state, family, None);
    let blocked = blocked_action_for_family(family);
    match decision.posture() {
        Posture::Allow | Posture::Read | Posture::Base => {
            if decision.reason() == Reason::AllowVerifiedLimited {
                DenialUxMessage::new(
                    DenialUxKind::Limited,
                    Some(DenialUxErrorCode::BaseRequired),
                    blocked,
                    "Your identity is verified, but there is no active entitlement yet; the one-project manual Focusa subset remains available.",
                    "evaluate",
                    "Start a free Evaluation or purchase Focusa",
                    "evaluation",
                )
            } else {
                DenialUxMessage::new(
                    DenialUxKind::Available,
                    None,
                    blocked,
                    available_reason_for_family(family),
                    "continue",
                    "Continue",
                    "account",
                )
            }
        }
        Posture::Feature => DenialUxMessage::new(
            DenialUxKind::Feature,
            Some(DenialUxErrorCode::FeatureRequired),
            blocked,
            "This optional family requires an additional authority-issued grant. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
            "manage",
            "Manage entitlement",
            "account",
        ),
        Posture::Deny => match family {
            CapabilityFamily::BaseFocusa => DenialUxMessage::new(
                DenialUxKind::Denied,
                Some(DenialUxErrorCode::BaseRequired),
                blocked,
                "A verified Evaluation or paid Focusa entitlement is required for value-producing Focusa work. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
                "evaluate",
                "Start a free Evaluation or purchase Focusa",
                "evaluation",
            ),
            CapabilityFamily::Automation
            | CapabilityFamily::TeamRemote
            | CapabilityFamily::ReleaseProof
            | CapabilityFamily::PremiumUpdates => DenialUxMessage::new(
                DenialUxKind::Denied,
                Some(DenialUxErrorCode::FeatureRequired),
                blocked,
                "This optional family requires an authority-issued entitlement. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
                "purchase",
                "Purchase or renew this optional family",
                "checkout",
            ),
            CapabilityFamily::ReadProjection => DenialUxMessage::new(
                DenialUxKind::Denied,
                Some(DenialUxErrorCode::Required),
                blocked,
                "Verifying your account unlocks read access to your existing local data. Registration, reading, export, recovery, repair, updates, and uninstall remain available.",
                "verify_identity",
                "Verify your account",
                "account",
            ),
            CapabilityFamily::AccountRecovery | CapabilityFamily::CustomerDataExport => {
                DenialUxMessage::new(
                    DenialUxKind::Available,
                    None,
                    blocked,
                    available_reason_for_family(family),
                    "continue",
                    "Continue",
                    "account",
                )
            }
            CapabilityFamily::InternalMaintenance => DenialUxMessage::new(
                DenialUxKind::Denied,
                Some(DenialUxErrorCode::RouteUnclassified),
                blocked,
                "This operation has no registered entitlement classification and is blocked before execution.",
                "diagnostics",
                "Run diagnostics or update policy",
                "recovery",
            ),
        },
    }
}

/// Canonical message for a stable error code; unknown codes fail closed.
pub fn denial_ux_message_for_code(code: DenialUxErrorCode) -> DenialUxMessage {
    let spec = code.spec();
    DenialUxMessage::new(
        DenialUxKind::Denied,
        Some(code),
        spec.public_message,
        spec.public_message,
        spec.safe_next_action,
        spec.action_label,
        spec.link,
    )
}

/// Typed failure for catalog contract violations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Error)]
pub enum DenialUxError {
    #[error("unknown denial UX code cannot be rendered")]
    UnknownCode,
    #[error("unknown state/family cell cannot be rendered")]
    UnknownCell,
}

/// Load the validated, embedded cross-presenter catalog artifact. The same
/// bytes the website and Pi fixtures bind; unknown codes fail closed.
pub fn embedded_denial_ux_catalog() -> Result<&'static Value, DenialUxError> {
    static CATALOG: OnceLock<Result<Value, DenialUxError>> = OnceLock::new();
    CATALOG
        .get_or_init(|| {
            serde_json::from_str(DENIAL_UX_CATALOG_JSON).map_err(|_| DenialUxError::UnknownCode)
        })
        .as_ref()
        .map_err(|error| *error)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn all_states() -> [PolicyEntitlementState; 7] {
        [
            PolicyEntitlementState::PendingUnverified,
            PolicyEntitlementState::VerifiedNoLicense,
            PolicyEntitlementState::ActivePaid,
            PolicyEntitlementState::OfflineGrace,
            PolicyEntitlementState::Expired,
            PolicyEntitlementState::RefundedOrRevoked,
            PolicyEntitlementState::MissingOrCorrupt,
        ]
    }

    fn all_families() -> [CapabilityFamily; 9] {
        [
            CapabilityFamily::AccountRecovery,
            CapabilityFamily::ReadProjection,
            CapabilityFamily::BaseFocusa,
            CapabilityFamily::Automation,
            CapabilityFamily::TeamRemote,
            CapabilityFamily::ReleaseProof,
            CapabilityFamily::PremiumUpdates,
            CapabilityFamily::CustomerDataExport,
            CapabilityFamily::InternalMaintenance,
        ]
    }

    fn state_from_label(label: &str) -> Option<PolicyEntitlementState> {
        all_states()
            .iter()
            .copied()
            .find(|state| state.label() == label)
    }

    fn family_from_label(label: &str) -> Option<CapabilityFamily> {
        all_families()
            .iter()
            .copied()
            .find(|family| family.label() == label)
    }

    #[test]
    fn frozen_contract_surface_matches_catalog() {
        assert_eq!(DENIAL_UX_SCHEMA, "focusa.spec152f.denial_ux_catalog.v1");
        assert_eq!(RETAINED_ACCESS.len(), 9);
        assert_eq!(PUBLIC_MESSAGE_RULES.len(), 7);
        assert_eq!(DENIAL_UX_LINK_IDS.len(), 4);
        assert_eq!(DENIAL_UX_ACTIONS.len(), 7);
        assert_eq!(DenialUxErrorCode::ALL.len(), 10);
        for id in DENIAL_UX_LINK_IDS {
            assert!(denial_ux_link(id).is_some(), "link {id} resolves");
        }
        assert_eq!(denial_ux_link("bogus"), None, "unknown link fails closed");
        assert_eq!(denial_ux_action_label("bogus"), None, "unknown action fails closed");
        assert_eq!(
            DenialUxErrorCode::from_label("ENTITLEMENT_MAGIC"),
            None,
            "unknown code fails closed"
        );
        for code in DenialUxErrorCode::ALL {
            assert_eq!(
                DenialUxErrorCode::from_label(code.label()),
                Some(code),
                "label round-trip for {code:?}"
            );
        }
    }

    #[test]
    fn embedded_catalog_is_loadable_and_complete() {
        let catalog = embedded_denial_ux_catalog().expect("embedded catalog loads");
        let grid = catalog["message_grid"].as_array().expect("grid array");
        let registry = catalog["error_registry"].as_array().expect("registry array");
        assert_eq!(grid.len(), 63, "7 states x 9 families");
        assert_eq!(registry.len(), 10, "stable error registry size");
        assert_eq!(
            catalog["schema"].as_str(),
            Some(DENIAL_UX_SCHEMA),
            "artifact schema matches the typed constant"
        );
        let always = catalog["always_reachable"].as_array().expect("always");
        assert_eq!(always.len(), 9);
        for cell in grid {
            let family = cell["family"].as_str().expect("family");
            let code = cell["code"].as_str();
            let link = cell["link"].as_str().expect("link");
            assert!(denial_ux_link(link).is_some(), "grid link resolves for {family}");
            if code.is_some() {
                assert!(DenialUxErrorCode::from_label(code.unwrap()).is_some());
            }
        }
    }

    #[test]
    fn derived_messages_are_identical_to_embedded_artifact() {
        let catalog = embedded_denial_ux_catalog().expect("embedded catalog loads");
        let grid = catalog["message_grid"].as_array().expect("grid array");
        for cell in grid {
            let state = state_from_label(cell["state"].as_str().expect("state"));
            let family = family_from_label(
                cell["family"].as_str().expect("family"),
            );
            let (Some(state), Some(family)) = (state, family) else {
                panic!("unknown state/family in artifact");
            };
            let message = denial_ux_message_for(state, family);
            assert_eq!(
                message.code().map(DenialUxErrorCode::label),
                cell["code"].as_str(),
                "code parity for {}",
                cell["family"].as_str().unwrap()
            );
            assert_eq!(
                message.safe_next_action(),
                cell["safe_next_action"].as_str().expect("action"),
                "next-action parity"
            );
            assert_eq!(message.link(), cell["link"].as_str().expect("link"), "link parity");
            assert_eq!(
                message.action_label(),
                cell["action_label"].as_str().expect("label"),
                "label parity"
            );
            assert_eq!(
                message.reason(),
                cell["reason"].as_str().expect("reason"),
                "reason parity"
            );
            assert_eq!(
                message.blocked_action(),
                cell["blocked_action"].as_str().expect("blocked"),
                "blocked-action parity"
            );
        }
    }

    #[test]
    fn denied_messages_always_preserve_a_route() {
        for state in all_states() {
            for family in all_families() {
                let message = denial_ux_message_for(state, family);
                assert_eq!(message.retained_access(), &RETAINED_ACCESS);
                assert!(
                    message.preserves_route(),
                    "message for {state:?}/{family:?} preserves a route"
                );
                if message.kind() == DenialUxKind::Denied
                    || message.kind() == DenialUxKind::Limited
                    || message.kind() == DenialUxKind::Feature
                {
                    assert!(message.code().is_some(), "denied message carries a code");
                    assert!(!message.safe_next_action().is_empty());
                }
            }
        }
    }

    #[test]
    fn error_specs_are_stable_and_redacted() {
        for code in DenialUxErrorCode::ALL {
            let spec = code.spec();
            assert_eq!(spec.code, code.label());
            assert!(!spec.public_message.is_empty());
            assert!(spec.http_status >= 400 && spec.http_status < 600);
            assert!(denial_ux_link(spec.link).is_some());
            assert!(!spec.public_message.contains("lease "));
            assert!(!spec.public_message.contains(" key "));
            assert!(!spec.public_message.contains(" token "));
            assert!(!spec.public_message.contains('@'));
        }
        let recovery = DenialUxErrorCode::RecoveryOnly.spec();
        assert_eq!(recovery.link, "recovery");
        assert_eq!(recovery.safe_next_action, "recovery");
        assert_eq!(
            denial_ux_link(recovery.link),
            Some("/activate/recovery"),
            "recovery route is stable and relative"
        );
    }
}
