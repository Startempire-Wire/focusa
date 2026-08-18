//! Spec 172 §11.3 — UIAI Cockpit mixed-product presenter binding
//! (atom focusa-vbcqu.20.15.29, 172.04.05).
//!
//! UIAI Cockpit is a UIAI-owned rich shell that may display or invoke both
//! products. Every Cockpit action resolves through this canonical action
//! registry and its product-isolation adapter:
//!
//! - Focusa read/display uses Focusa policy (`base_focusa` base gate);
//!   rendering Focusa state in the Cockpit does not grant Focusa mutation;
//! - UIAI observation/action uses UIAI policy: the canonical
//!   `SPEC172_UIAI_OPERATION_MAP` limited/paid boundary
//!   (`resolve_uiai_operation_capability`);
//! - a combined workflow requires BOTH grants or the Bundle (the exact union
//!   of the two underlying License Type grants, Spec 172 §9/§20.7);
//! - pairing a Cockpit or Desktop proves identity/device posture, not entitlement.
//!   Pairing/auth proves identity only; no pairing/auth/device input is
//!   accepted by the resolver.
//!
//! No anonymous product capability, no local/self-issued grant, and no
//! caller-controlled product, price, License Type, family, feature, limit,
//! node, or commercial right enters any decision. The registry is
//! server-owned static metadata; every row carries Spec 172 §12 trusted
//! operation metadata (operation_id / product_owner / operation_class /
//! capability_family / side_effect_class).
//! No anonymous product capability, no local/self-issued grant, and no
//! caller-controlled product, price, License Type, family, feature, limit,
//! node, or commercial right enters any decision. The registry is
//! server-owned static metadata; every row carries Spec 172 §12 trusted
//! operation metadata (operation_id / product_owner / operation_class /
//! capability_family / side_effect_class).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::authority::EntitlementSnapshot;
use crate::entitlement_policy::{
    BaseProductDecision, authority_policy_state, resolve_base_focusa_product,
};
use crate::uiai_child_token::{
    UiaiCapabilityDecision, UiaiCapabilityDenial, UiaiOperationClass, UiaiOperationError,
    classify_uiai_operation, resolve_uiai_capability, resolve_uiai_operation_capability,
};

/// Schema label for the canonical Cockpit action registry (Spec 172 §12
/// trusted metadata envelope).
pub const COCKPIT_ACTION_REGISTRY_SCHEMA: &str = "focusa.cockpit_action_registry.v1";

/// One canonical UIAI Cockpit action-registry row (Spec 172 §11.3, §12).
///
/// Every field is server-owned constant metadata:
/// - `action_id` is the stable canonical Cockpit action identifier;
/// - `product_owner` is a registered owner (`focusa` | `uiai_engine`); the
///   Cockpit is a UIAI-owned shell, so combined workflows are `uiai_engine`
///   rows flagged `combined_workflow: true`;
/// - `operation_class` is a registered §12 class (`read` | `value_mutation`);
/// - `capability_family` is the registered family the action binds;
/// - `side_effect_class` is a registered §12 class (`none` | `local` |
///   `remote` | `external`);
/// - `combined_workflow` marks a workflow that invokes both products and
///   therefore requires both grants or the Bundle (§11.3);
/// - `uiai_operation` links UIAI/combined rows to their canonical
///   `SPEC172_UIAI_OPERATION_MAP` vector (`None` for pure Focusa rows).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CockpitActionMapEntry {
    pub action_id: &'static str,
    pub product_owner: &'static str,
    pub operation_class: &'static str,
    pub capability_family: &'static str,
    pub side_effect_class: &'static str,
    pub combined_workflow: bool,
    pub uiai_operation: Option<&'static str>,
}

/// Canonical UIAI Cockpit action registry (Spec 172 §11.3).
///
/// Focusa display/read rows are `base_focusa` reads; Focusa value-mutation
/// rows are `base_focusa` mutations. UIAI rows map one-to-one onto the
/// canonical `SPEC172_UIAI_OPERATION_MAP` vectors so every Cockpit UIAI
/// action inherits the exact limited/paid boundary. Combined workflows bind
/// one Focusa grant side and one paid UIAI family. Unknown action ids fail
/// closed in the adapter; no caller can extend, rename, or reclassify a row.
pub const SPEC172_COCKPIT_ACTION_REGISTRY: &[CockpitActionMapEntry] = &[
    // ── Focusa-owned: display/read — rendering Focusa state in the Cockpit
    // does not grant Focusa mutation (§11.3). ───────────────────────────────
    CockpitActionMapEntry {
        action_id: "cockpit.focusa.display_mission",
        product_owner: "focusa",
        operation_class: "read",
        capability_family: "base_focusa",
        side_effect_class: "none",
        combined_workflow: false,
        uiai_operation: None,
    },
    CockpitActionMapEntry {
        action_id: "cockpit.focusa.display_workpoint",
        product_owner: "focusa",
        operation_class: "read",
        capability_family: "base_focusa",
        side_effect_class: "none",
        combined_workflow: false,
        uiai_operation: None,
    },
    CockpitActionMapEntry {
        action_id: "cockpit.focusa.display_trajectory",
        product_owner: "focusa",
        operation_class: "read",
        capability_family: "base_focusa",
        side_effect_class: "none",
        combined_workflow: false,
        uiai_operation: None,
    },
    CockpitActionMapEntry {
        action_id: "cockpit.focusa.display_evidence",
        product_owner: "focusa",
        operation_class: "read",
        capability_family: "base_focusa",
        side_effect_class: "none",
        combined_workflow: false,
        uiai_operation: None,
    },
    CockpitActionMapEntry {
        action_id: "cockpit.focusa.read_projection",
        product_owner: "focusa",
        operation_class: "read",
        capability_family: "base_focusa",
        side_effect_class: "none",
        combined_workflow: false,
        uiai_operation: None,
    },
    // ── Focusa-owned: value mutation — the base product gate must be
    // Entitled (one usable signed Focusa lease or valid Offline Grace). ─────
    CockpitActionMapEntry {
        action_id: "cockpit.focusa.mutate_project",
        product_owner: "focusa",
        operation_class: "value_mutation",
        capability_family: "base_focusa",
        side_effect_class: "local",
        combined_workflow: false,
        uiai_operation: None,
    },
    CockpitActionMapEntry {
        action_id: "cockpit.focusa.mutate_mission",
        product_owner: "focusa",
        operation_class: "value_mutation",
        capability_family: "base_focusa",
        side_effect_class: "local",
        combined_workflow: false,
        uiai_operation: None,
    },
    CockpitActionMapEntry {
        action_id: "cockpit.focusa.mutate_workpoint",
        product_owner: "focusa",
        operation_class: "value_mutation",
        capability_family: "base_focusa",
        side_effect_class: "local",
        combined_workflow: false,
        uiai_operation: None,
    },
    CockpitActionMapEntry {
        action_id: "cockpit.focusa.mutate_evidence",
        product_owner: "focusa",
        operation_class: "value_mutation",
        capability_family: "base_focusa",
        side_effect_class: "local",
        combined_workflow: false,
        uiai_operation: None,
    },
    CockpitActionMapEntry {
        action_id: "cockpit.focusa.run_work_loop",
        product_owner: "focusa",
        operation_class: "value_mutation",
        capability_family: "base_focusa",
        side_effect_class: "local",
        combined_workflow: false,
        uiai_operation: None,
    },
    // ── UIAI-owned: observation/action — one-to-one with the canonical
    // SPEC172_UIAI_OPERATION_MAP vectors; limited/paid boundary applies. ────
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.public_search",
        product_owner: "uiai_engine",
        operation_class: "read",
        capability_family: "public_search",
        side_effect_class: "remote",
        combined_workflow: false,
        uiai_operation: Some("public_search"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.source_to_markdown",
        product_owner: "uiai_engine",
        operation_class: "read",
        capability_family: "source_to_markdown",
        side_effect_class: "remote",
        combined_workflow: false,
        uiai_operation: Some("source_to_markdown"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.public_page_read",
        product_owner: "uiai_engine",
        operation_class: "read",
        capability_family: "public_page_read",
        side_effect_class: "remote",
        combined_workflow: false,
        uiai_operation: Some("public_page_read"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.accessibility_snapshot",
        product_owner: "uiai_engine",
        operation_class: "read",
        capability_family: "accessibility_snapshot",
        side_effect_class: "remote",
        combined_workflow: false,
        uiai_operation: Some("accessibility_snapshot"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.screenshot",
        product_owner: "uiai_engine",
        operation_class: "read",
        capability_family: "screenshot",
        side_effect_class: "remote",
        combined_workflow: false,
        uiai_operation: Some("screenshot"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.basic_diagnostics",
        product_owner: "uiai_engine",
        operation_class: "read",
        capability_family: "basic_diagnostics",
        side_effect_class: "local",
        combined_workflow: false,
        uiai_operation: Some("basic_diagnostics"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.browser_click",
        product_owner: "uiai_engine",
        operation_class: "value_mutation",
        capability_family: "browser_action",
        side_effect_class: "remote",
        combined_workflow: false,
        uiai_operation: Some("browser_click"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.browser_fill",
        product_owner: "uiai_engine",
        operation_class: "value_mutation",
        capability_family: "browser_action",
        side_effect_class: "remote",
        combined_workflow: false,
        uiai_operation: Some("browser_fill"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.browser_type",
        product_owner: "uiai_engine",
        operation_class: "value_mutation",
        capability_family: "browser_action",
        side_effect_class: "remote",
        combined_workflow: false,
        uiai_operation: Some("browser_type"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.browser_select",
        product_owner: "uiai_engine",
        operation_class: "value_mutation",
        capability_family: "browser_action",
        side_effect_class: "remote",
        combined_workflow: false,
        uiai_operation: Some("browser_select"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.browser_press",
        product_owner: "uiai_engine",
        operation_class: "value_mutation",
        capability_family: "browser_action",
        side_effect_class: "remote",
        combined_workflow: false,
        uiai_operation: Some("browser_press"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.browser_submit",
        product_owner: "uiai_engine",
        operation_class: "value_mutation",
        capability_family: "browser_action",
        side_effect_class: "remote",
        combined_workflow: false,
        uiai_operation: Some("browser_submit"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.cookie_persistence",
        product_owner: "uiai_engine",
        operation_class: "value_mutation",
        capability_family: "browser_persistence",
        side_effect_class: "local",
        combined_workflow: false,
        uiai_operation: Some("cookie_persistence"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.auth_state_persistence",
        product_owner: "uiai_engine",
        operation_class: "value_mutation",
        capability_family: "browser_persistence",
        side_effect_class: "local",
        combined_workflow: false,
        uiai_operation: Some("auth_state_persistence"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.session_persistence",
        product_owner: "uiai_engine",
        operation_class: "value_mutation",
        capability_family: "browser_persistence",
        side_effect_class: "local",
        combined_workflow: false,
        uiai_operation: Some("session_persistence"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.authenticated_private_dashboard",
        product_owner: "uiai_engine",
        operation_class: "value_mutation",
        capability_family: "authenticated_private_targets",
        side_effect_class: "remote",
        combined_workflow: false,
        uiai_operation: Some("authenticated_private_dashboard"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.unattended_automation",
        product_owner: "uiai_engine",
        operation_class: "value_mutation",
        capability_family: "unattended_browser_automation",
        side_effect_class: "remote",
        combined_workflow: false,
        uiai_operation: Some("unattended_browser_automation"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.scheduled_batch_qa",
        product_owner: "uiai_engine",
        operation_class: "value_mutation",
        capability_family: "scheduled_batch_qa",
        side_effect_class: "remote",
        combined_workflow: false,
        uiai_operation: Some("scheduled_batch_qa"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.premium_proxy",
        product_owner: "uiai_engine",
        operation_class: "value_mutation",
        capability_family: "premium_hosted_resources",
        side_effect_class: "remote",
        combined_workflow: false,
        uiai_operation: Some("premium_proxy"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.hosted_capacity",
        product_owner: "uiai_engine",
        operation_class: "value_mutation",
        capability_family: "premium_hosted_resources",
        side_effect_class: "remote",
        combined_workflow: false,
        uiai_operation: Some("hosted_capacity"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.uiai.paid_model_calls",
        product_owner: "uiai_engine",
        operation_class: "value_mutation",
        capability_family: "premium_hosted_resources",
        side_effect_class: "remote",
        combined_workflow: false,
        uiai_operation: Some("paid_model_calls"),
    },
    // ── Combined workflows (§11.3): UIAI-owned composites that invoke both
    // products; they require BOTH grants or the Bundle. ─────────────────────
    CockpitActionMapEntry {
        action_id: "cockpit.combined.research_apply",
        product_owner: "uiai_engine",
        operation_class: "value_mutation",
        capability_family: "combined_focusa_uiai",
        side_effect_class: "remote",
        combined_workflow: true,
        uiai_operation: Some("browser_submit"),
    },
    CockpitActionMapEntry {
        action_id: "cockpit.combined.observe_and_capture",
        product_owner: "uiai_engine",
        operation_class: "value_mutation",
        capability_family: "combined_focusa_uiai",
        side_effect_class: "remote",
        combined_workflow: true,
        uiai_operation: Some("public_search"),
    },
];

/// Typed failure for Cockpit action-registry resolution. Unknown action ids
/// fail closed before any entitlement decision or UI side effect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum CockpitActionError {
    UnknownAction,
}

/// Look up the canonical registry row for a Cockpit action id.
///
/// `None` for any unknown, prefixed, aliased, or caller-invented id: the
/// registry is the single authority for Cockpit action vectors.
pub fn classify_cockpit_action(action_id: &str) -> Option<&'static CockpitActionMapEntry> {
    SPEC172_COCKPIT_ACTION_REGISTRY
        .iter()
        .find(|entry| entry.action_id == action_id)
}

/// Typed denial reasons for the Cockpit mixed-product decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "reason", rename_all = "snake_case")]
pub enum CockpitActionDenial {
    /// No Focusa posture at all: Focusa display requires a verified posture
    /// (limited or entitled), and a UIAI-only account cannot display Focusa.
    FocusaDisplayDenied,
    /// The static registry row is internally inconsistent (missing vector or
    /// unregistered owner). Unreachable for shipped rows; fails closed.
    RegistryIntegrity,
    /// Focusa value mutation requires the base product gate Entitled; the
    /// verified no-license limited posture never satisfies it.
    FocusaMutationDenied,
    /// UIAI-side denial from the canonical UIAI limited/paid boundary
    /// (observe-only quota, paid family missing, hosted rights, etc.).
    UiaiDenied(UiaiCapabilityDenial),
    /// Combined workflow without the Focusa grant.
    CombinedMissingFocusaGrant,
    /// Combined workflow on a verified no-license limited Focusa posture:
    /// combined workflows require grants, never limited mode (§11.3).
    CombinedLimitedModeDenied,
    /// Combined workflow without the paid UIAI grant/family (or with a
    /// cross-account/cross-node grant).
    CombinedMissingUiaiGrant,
}

/// Canonical mixed-product decision for one UIAI Cockpit action (Spec 172
/// §11.3; §20.8 presenter-equivalence).
///
/// Every Cockpit action resolves through this decision BEFORE any UI side
/// effect: Focusa display never grants mutation; UIAI observation/action
/// follows the canonical limited/paid boundary; combined workflows require
/// both grants or the Bundle. Only the canonical action id and authority
/// snapshots are consumed — pairing/auth/device state and caller-selected
/// products, prices, License Types, families, features, limits, nodes, and
/// commercial rights never enter the decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum CockpitActionDecision {
    /// Focusa read/display. Rendering Focusa state in the Cockpit does not
    /// grant Focusa mutation (§11.3). Reachable for Entitled and Limited
    /// (verified no-license) postures; the base decision is carried for
    /// telemetry.
    FocusaDisplay {
        base: BaseProductDecision,
    },
    /// Focusa value mutation — permitted only when the base product gate is
    /// Entitled (one usable signed Focusa lease or valid Offline Grace).
    FocusaMutation {
        base: BaseProductDecision,
    },
    /// UIAI observation/action — delegated to the canonical UIAI boundary:
    /// one foreground ephemeral public-observe session, paid Operator v1
    /// family, or denial.
    Uiai(UiaiCapabilityDecision),
    /// Combined workflow — both grants (Focusa base Entitled + paid UIAI
    /// family) or the Bundle (exact union of the two underlying grants).
    CombinedAllowed {
        uiai_family: String,
        parent_lease_id: String,
        parent_sequence: u64,
        uiai_grant_sequence: u64,
    },
    Denied(CockpitActionDenial),
}

impl CockpitActionDecision {
    pub fn is_allowed(&self) -> bool {
        matches!(
            self,
            Self::FocusaDisplay { .. }
                | Self::FocusaMutation { .. }
                | Self::Uiai(UiaiCapabilityDecision::VerifiedNoLicensePublicObservation { .. })
                | Self::Uiai(UiaiCapabilityDecision::PaidFamily { .. })
                | Self::CombinedAllowed { .. }
        )
    }

    pub fn denial(&self) -> Option<&CockpitActionDenial> {
        match self {
            Self::Denied(denial) => Some(denial),
            _ => None,
        }
    }

    /// True only when this decision permits a Focusa value mutation.
    /// Rendering Focusa state in the Cockpit (`FocusaDisplay`) never grants
    /// mutation, for any base posture.
    pub fn permits_focusa_mutation(&self) -> bool {
        matches!(
            self,
            Self::FocusaMutation { base } if base.permits_base_mutations()
        )
    }
}

/// True when the snapshot is the authority-signed verified no-license limited
/// posture: identity verified, no paid lease identity, sequence, or digest.
fn is_verified_no_license_posture(snapshot: &EntitlementSnapshot) -> bool {
    snapshot.lease_id.as_deref().is_none_or(str::is_empty)
        && snapshot.sequence.is_none_or(|sequence| sequence == 0)
        && snapshot.lease_digest.as_deref().is_none_or(str::is_empty)
}

/// Product-isolation adapter: resolve one UIAI Cockpit action by owning
/// product (Spec 172 §11.3, §20.8).
///
/// - Focusa rows: the base product gate decides — display is reachable for
///   Entitled and Limited postures and never grants mutation; value mutation
///   requires Entitled.
/// - UIAI rows: delegated to the canonical UIAI operation boundary, so the
///   observe-only limited mode and the paid Operator v1 action boundary apply
///   unchanged.
/// - Combined rows: require BOTH the Focusa base grant AND the paid UIAI
///   family (or the Bundle) — a verified no-license limited posture never
///   satisfies a combined workflow.
///
/// Pairing, device proof, authentication state, and caller-selected
/// products/prices/License Types/families/features/limits/nodes/commercial
/// rights are never accepted here: pairing proves identity only.
pub fn resolve_cockpit_action(
    action_id: &str,
    focusa_snapshot: Option<&EntitlementSnapshot>,
    uiai_snapshot: Option<&EntitlementSnapshot>,
    active_uiai_sessions: u32,
    now: DateTime<Utc>,
) -> Result<CockpitActionDecision, CockpitActionError> {
    let entry = classify_cockpit_action(action_id).ok_or(CockpitActionError::UnknownAction)?;
    if entry.combined_workflow {
        return Ok(resolve_combined_workflow(
            entry,
            focusa_snapshot,
            uiai_snapshot,
            active_uiai_sessions,
            now,
        ));
    }
    match entry.product_owner {
        "focusa" => Ok(resolve_focusa_action(entry, focusa_snapshot)),
        "uiai_engine" => Ok(resolve_uiai_action(
            entry,
            focusa_snapshot,
            uiai_snapshot,
            active_uiai_sessions,
            now,
        )),
        _ => Ok(CockpitActionDecision::Denied(
            CockpitActionDenial::RegistryIntegrity,
        )),
    }
}

/// Focusa side of the Cockpit decision: the base product gate.
fn resolve_focusa_action(
    entry: &CockpitActionMapEntry,
    focusa_snapshot: Option<&EntitlementSnapshot>,
) -> CockpitActionDecision {
    let base = match focusa_snapshot {
        Some(snapshot) if is_verified_no_license_posture(snapshot) => {
            // Identity verified, no signed lease: the explicit limited posture.
            BaseProductDecision::Limited
        }
        Some(snapshot) => {
            resolve_base_focusa_product(&snapshot.product, authority_policy_state(snapshot))
        }
        None => {
            // No Focusa posture at all: display and mutation both fail
            // closed with their per-class denial.
            return match entry.operation_class {
                "read" => CockpitActionDenial::FocusaDisplayDenied.into_decision(),
                _ => CockpitActionDenial::FocusaMutationDenied.into_decision(),
            };
        }
    };
    match entry.operation_class {
        "read" => match base {
            BaseProductDecision::Entitled | BaseProductDecision::Limited => {
                CockpitActionDecision::FocusaDisplay { base }
            }
            BaseProductDecision::Denied => {
                CockpitActionDecision::Denied(CockpitActionDenial::FocusaDisplayDenied)
            }
        },
        // value_mutation (and any unregistered class) requires the base gate.
        _ => match base {
            BaseProductDecision::Entitled => CockpitActionDecision::FocusaMutation { base },
            _ => CockpitActionDecision::Denied(CockpitActionDenial::FocusaMutationDenied),
        },
    }
}

/// UIAI side of the Cockpit decision: delegate to the canonical UIAI
/// limited/paid boundary via the row's `SPEC172_UIAI_OPERATION_MAP` vector.
fn resolve_uiai_action(
    entry: &CockpitActionMapEntry,
    focusa_snapshot: Option<&EntitlementSnapshot>,
    uiai_snapshot: Option<&EntitlementSnapshot>,
    active_uiai_sessions: u32,
    now: DateTime<Utc>,
) -> CockpitActionDecision {
    let Some(operation_id) = entry.uiai_operation else {
        return CockpitActionDecision::Denied(CockpitActionDenial::RegistryIntegrity);
    };
    match resolve_uiai_operation_capability(
        operation_id,
        focusa_snapshot,
        uiai_snapshot,
        active_uiai_sessions,
        now,
    ) {
        Ok(decision) => CockpitActionDecision::Uiai(decision),
        Err(UiaiOperationError::UnknownOperation) => {
            CockpitActionDecision::Denied(CockpitActionDenial::RegistryIntegrity)
        }
    }
}

/// Combined workflow: both grants or the Bundle (§11.3). The Focusa base gate
/// must be Entitled and the row's canonical UIAI vector must resolve to a
/// PAID Operator v1 family (limited mode and no-grant both deny).
fn resolve_combined_workflow(
    entry: &CockpitActionMapEntry,
    focusa_snapshot: Option<&EntitlementSnapshot>,
    uiai_snapshot: Option<&EntitlementSnapshot>,
    active_uiai_sessions: u32,
    now: DateTime<Utc>,
) -> CockpitActionDecision {
    let focusa_base = match focusa_snapshot {
        Some(snapshot) if is_verified_no_license_posture(snapshot) => {
            return CockpitActionDecision::Denied(CockpitActionDenial::CombinedLimitedModeDenied);
        }
        Some(snapshot) => {
            resolve_base_focusa_product(&snapshot.product, authority_policy_state(snapshot))
        }
        None => {
            return CockpitActionDecision::Denied(CockpitActionDenial::CombinedMissingFocusaGrant);
        }
    };
    if focusa_base != BaseProductDecision::Entitled {
        return CockpitActionDecision::Denied(CockpitActionDenial::CombinedMissingFocusaGrant);
    }
    let Some(operation_id) = entry.uiai_operation else {
        return CockpitActionDecision::Denied(CockpitActionDenial::RegistryIntegrity);
    };
    let Some(vector) = classify_uiai_operation(operation_id) else {
        return CockpitActionDecision::Denied(CockpitActionDenial::RegistryIntegrity);
    };
    match resolve_uiai_capability(
        focusa_snapshot,
        uiai_snapshot,
        UiaiOperationClass::RemotePremium,
        vector.limited_family,
        vector.paid_feature,
        active_uiai_sessions,
        now,
    ) {
        UiaiCapabilityDecision::PaidFamily {
            family,
            parent_lease_id,
            parent_sequence,
            uiai_grant_sequence,
        } => CockpitActionDecision::CombinedAllowed {
            uiai_family: family,
            parent_lease_id,
            parent_sequence,
            uiai_grant_sequence,
        },
        _ => CockpitActionDecision::Denied(CockpitActionDenial::CombinedMissingUiaiGrant),
    }
}

impl CockpitActionDenial {
    fn into_decision(self) -> CockpitActionDecision {
        CockpitActionDecision::Denied(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::authority::EntitlementState;
    use crate::uiai_activation::{PRODUCT_FOCUSA, PRODUCT_UIAI_ENGINE};
    use chrono::Duration;
    use std::collections::BTreeSet;

    /// Focusa mixed-grant fixture: a bound signed lease for the exact product
    /// `focusa` on node-001 for one EDD account.
    fn focusa_snapshot(state: EntitlementState, subject: Option<&str>) -> EntitlementSnapshot {
        let mut snapshot = EntitlementSnapshot::unactivated(PRODUCT_FOCUSA, "node-001");
        snapshot.state = state;
        snapshot.subject_id = subject.map(str::to_string);
        match state {
            EntitlementState::Active => {
                snapshot.lease_id = Some("lease-focusa".to_string());
                snapshot.sequence = Some(7);
                snapshot.lease_digest = Some("sha256:bound-grant-digest".to_string());
                snapshot.expires_at = Some(Utc::now() + Duration::hours(1));
            }
            EntitlementState::OfflineGrace => {
                snapshot.lease_id = Some("lease-focusa".to_string());
                snapshot.sequence = Some(7);
                snapshot.lease_digest = Some("sha256:bound-grant-digest".to_string());
                snapshot.offline_grace_until = Some(Utc::now() + Duration::hours(1));
            }
            _ => {}
        }
        snapshot
    }

    /// UIAI mixed-grant fixture: a bound signed grant for the exact product
    /// `uiai-engine` carrying the given paid Operator v1 family features.
    fn uiai_snapshot(features: &[&str], subject: Option<&str>) -> EntitlementSnapshot {
        let mut snapshot = EntitlementSnapshot::unactivated(PRODUCT_UIAI_ENGINE, "node-001");
        snapshot.state = EntitlementState::Active;
        snapshot.subject_id = subject.map(str::to_string);
        snapshot.lease_id = Some("lease-uiai-engine".to_string());
        snapshot.sequence = Some(7);
        snapshot.lease_digest = Some("sha256:bound-grant-digest".to_string());
        snapshot.expires_at = Some(Utc::now() + Duration::hours(1));
        for feature in features {
            snapshot.features.insert(feature.to_string(), true);
        }
        snapshot
    }

    /// Verified no-license limited fixture: identity verified, no signed
    /// lease identity, sequence, or digest.
    fn limited_focusa_posture() -> EntitlementSnapshot {
        EntitlementSnapshot::unactivated(PRODUCT_FOCUSA, "node-001")
    }

    fn now() -> DateTime<Utc> {
        Utc::now()
    }

    #[test]
    fn spec172_cockpit_registry_metadata_conforms_to_section12() {
        // Product owners, operation classes, and side-effect classes must be
        // registered §12 values; action ids must be unique.
        let mut seen = BTreeSet::new();
        for entry in SPEC172_COCKPIT_ACTION_REGISTRY {
            assert!(
                ["focusa", "uiai_engine"].contains(&entry.product_owner),
                "unregistered product owner: {}",
                entry.action_id
            );
            assert!(
                ["read", "value_mutation"].contains(&entry.operation_class),
                "unregistered operation class: {}",
                entry.action_id
            );
            assert!(
                ["none", "local", "remote", "external"].contains(&entry.side_effect_class),
                "unregistered side-effect class: {}",
                entry.action_id
            );
            assert!(seen.insert(entry.action_id), "duplicate action id");
            if entry.combined_workflow {
                // Combined workflows are UIAI-owned composites (§11.3).
                assert_eq!(entry.product_owner, "uiai_engine");
                assert_eq!(entry.operation_class, "value_mutation");
                assert_eq!(entry.capability_family, "combined_focusa_uiai");
            }
            // Every UIAI/combined row binds a canonical UIAI operation vector;
            // Focusa rows bind none.
            match entry.product_owner {
                "uiai_engine" => {
                    let operation_id = entry.uiai_operation.expect("uiai row vector");
                    assert!(
                        classify_uiai_operation(operation_id).is_some(),
                        "uiai row must bind a canonical UIAI vector: {}",
                        entry.action_id
                    );
                }
                _ => assert!(entry.uiai_operation.is_none()),
            }
        }
        // Every canonical UIAI operation map vector has a Cockpit binding.
        // 23 total vector references (21 UIAI rows + 2 combined rows) cover
        // all 21 unique canonical vectors; combined rows reuse canonical
        // vectors (browser_submit, public_search).
        let cockpit_vectors: BTreeSet<&str> = SPEC172_COCKPIT_ACTION_REGISTRY
            .iter()
            .filter_map(|entry| entry.uiai_operation)
            .collect();
        assert_eq!(
            cockpit_vectors.len(),
            21,
            "all 21 UIAI vectors must be Cockpit-bound"
        );
        let vector_references: usize = SPEC172_COCKPIT_ACTION_REGISTRY
            .iter()
            .filter(|entry| entry.uiai_operation.is_some())
            .count();
        assert_eq!(vector_references, 23, "21 UIAI rows + 2 combined rows");
    }

    #[test]
    fn spec172_cockpit_focusa_only_account_gets_focusa_but_never_uiai() {
        let focusa = focusa_snapshot(EntitlementState::Active, Some("account-001"));
        let t = now();
        // Focusa display and mutation are granted by the base product gate.
        assert_eq!(
            resolve_cockpit_action("cockpit.focusa.display_mission", Some(&focusa), None, 0, t),
            Ok(CockpitActionDecision::FocusaDisplay {
                base: BaseProductDecision::Entitled
            })
        );
        assert_eq!(
            resolve_cockpit_action("cockpit.focusa.mutate_project", Some(&focusa), None, 0, t),
            Ok(CockpitActionDecision::FocusaMutation {
                base: BaseProductDecision::Entitled
            })
        );
        // Display never grants mutation.
        let display =
            resolve_cockpit_action("cockpit.focusa.display_mission", Some(&focusa), None, 0, t)
                .expect("decision");
        assert!(!display.permits_focusa_mutation());
        // UIAI observation/action is never granted by a Focusa-only account.
        assert_eq!(
            resolve_cockpit_action("cockpit.uiai.public_search", Some(&focusa), None, 0, t),
            Ok(CockpitActionDecision::Uiai(UiaiCapabilityDecision::Denied(
                UiaiCapabilityDenial::FocusaOnlyCannotGrantUiai
            )))
        );
        assert_eq!(
            resolve_cockpit_action("cockpit.uiai.browser_click", Some(&focusa), None, 0, t),
            Ok(CockpitActionDecision::Uiai(UiaiCapabilityDecision::Denied(
                UiaiCapabilityDenial::FocusaOnlyCannotGrantUiai
            )))
        );
        // Combined workflows need the UIAI grant too.
        assert_eq!(
            resolve_cockpit_action("cockpit.combined.research_apply", Some(&focusa), None, 0, t),
            Ok(CockpitActionDecision::Denied(
                CockpitActionDenial::CombinedMissingUiaiGrant
            ))
        );
    }

    #[test]
    fn spec172_cockpit_uiai_only_account_gets_uiai_boundary_but_no_focusa() {
        let uiai = uiai_snapshot(
            &["uiai_public_observation", "uiai_browser_action"],
            Some("account-001"),
        );
        let t = now();
        // UIAI-only: no Focusa posture, so Focusa display and mutation deny.
        assert_eq!(
            resolve_cockpit_action("cockpit.focusa.display_mission", None, Some(&uiai), 0, t),
            Ok(CockpitActionDecision::Denied(
                CockpitActionDenial::FocusaDisplayDenied
            ))
        );
        assert_eq!(
            resolve_cockpit_action("cockpit.focusa.mutate_project", None, Some(&uiai), 0, t),
            Ok(CockpitActionDecision::Denied(
                CockpitActionDenial::FocusaMutationDenied
            ))
        );
        // UIAI observation/action follows the paid boundary.
        assert_eq!(
            resolve_cockpit_action("cockpit.uiai.public_search", None, Some(&uiai), 0, t),
            Ok(CockpitActionDecision::Uiai(
                UiaiCapabilityDecision::PaidFamily {
                    family: "uiai_public_observation".to_string(),
                    parent_lease_id: String::new(),
                    parent_sequence: 0,
                    uiai_grant_sequence: 7,
                }
            ))
        );
        assert_eq!(
            resolve_cockpit_action("cockpit.uiai.browser_click", None, Some(&uiai), 0, t),
            Ok(CockpitActionDecision::Uiai(
                UiaiCapabilityDecision::PaidFamily {
                    family: "uiai_browser_action".to_string(),
                    parent_lease_id: String::new(),
                    parent_sequence: 0,
                    uiai_grant_sequence: 7,
                }
            ))
        );
        // Hosted rights carry no canonical paid feature: still denied.
        assert_eq!(
            resolve_cockpit_action("cockpit.uiai.premium_proxy", None, Some(&uiai), 0, t),
            Ok(CockpitActionDecision::Uiai(UiaiCapabilityDecision::Denied(
                UiaiCapabilityDenial::FamilyNotGranted
            )))
        );
        // Combined workflows need the Focusa grant too.
        assert_eq!(
            resolve_cockpit_action("cockpit.combined.research_apply", None, Some(&uiai), 0, t),
            Ok(CockpitActionDecision::Denied(
                CockpitActionDenial::CombinedMissingFocusaGrant
            ))
        );
    }

    #[test]
    fn spec172_cockpit_bundle_account_resolves_exact_union() {
        // The Bundle is the exact union of the two underlying grants on one
        // node/account: Focusa base + paid UIAI Operator v1 families.
        let focusa = focusa_snapshot(EntitlementState::Active, Some("account-001"));
        let uiai = uiai_snapshot(
            &[
                "uiai_public_observation",
                "uiai_browser_action",
                "uiai_persistence",
                "uiai_batch_responsive",
            ],
            Some("account-001"),
        );
        let t = now();
        // Both standalone grants work.
        assert_eq!(
            resolve_cockpit_action(
                "cockpit.focusa.mutate_mission",
                Some(&focusa),
                Some(&uiai),
                0,
                t
            ),
            Ok(CockpitActionDecision::FocusaMutation {
                base: BaseProductDecision::Entitled
            })
        );
        assert_eq!(
            resolve_cockpit_action(
                "cockpit.uiai.browser_fill",
                Some(&focusa),
                Some(&uiai),
                0,
                t
            ),
            Ok(CockpitActionDecision::Uiai(
                UiaiCapabilityDecision::PaidFamily {
                    family: "uiai_browser_action".to_string(),
                    parent_lease_id: "lease-focusa".to_string(),
                    parent_sequence: 7,
                    uiai_grant_sequence: 7,
                }
            ))
        );
        // Combined workflows resolve only with BOTH grants (or the Bundle).
        assert_eq!(
            resolve_cockpit_action(
                "cockpit.combined.research_apply",
                Some(&focusa),
                Some(&uiai),
                0,
                t,
            ),
            Ok(CockpitActionDecision::CombinedAllowed {
                uiai_family: "uiai_browser_action".to_string(),
                parent_lease_id: "lease-focusa".to_string(),
                parent_sequence: 7,
                uiai_grant_sequence: 7,
            })
        );
        assert_eq!(
            resolve_cockpit_action(
                "cockpit.combined.observe_and_capture",
                Some(&focusa),
                Some(&uiai),
                0,
                t,
            ),
            Ok(CockpitActionDecision::CombinedAllowed {
                uiai_family: "uiai_public_observation".to_string(),
                parent_lease_id: "lease-focusa".to_string(),
                parent_sequence: 7,
                uiai_grant_sequence: 7,
            })
        );
        // Even the Bundle never grants hosted/private rights.
        assert_eq!(
            resolve_cockpit_action(
                "cockpit.uiai.paid_model_calls",
                Some(&focusa),
                Some(&uiai),
                0,
                t
            ),
            Ok(CockpitActionDecision::Uiai(UiaiCapabilityDecision::Denied(
                UiaiCapabilityDenial::FamilyNotGranted
            )))
        );
        // A cross-account UIAI grant never combines with the Focusa grant.
        let other_customer = uiai_snapshot(&["uiai_browser_action"], Some("account-002"));
        assert_eq!(
            resolve_cockpit_action(
                "cockpit.combined.research_apply",
                Some(&focusa),
                Some(&other_customer),
                0,
                t,
            ),
            Ok(CockpitActionDecision::Denied(
                CockpitActionDenial::CombinedMissingUiaiGrant
            ))
        );
    }

    #[test]
    fn spec172_cockpit_limited_account_gets_display_and_observe_only() {
        let limited = limited_focusa_posture();
        let t = now();
        // Display is reachable for the limited posture (read/export stay
        // usable), but value mutation is denied.
        assert_eq!(
            resolve_cockpit_action(
                "cockpit.focusa.display_evidence",
                Some(&limited),
                None,
                0,
                t
            ),
            Ok(CockpitActionDecision::FocusaDisplay {
                base: BaseProductDecision::Limited
            })
        );
        let display = resolve_cockpit_action(
            "cockpit.focusa.display_evidence",
            Some(&limited),
            None,
            0,
            t,
        )
        .expect("decision");
        assert!(!display.permits_focusa_mutation());
        assert_eq!(
            resolve_cockpit_action("cockpit.focusa.mutate_project", Some(&limited), None, 0, t),
            Ok(CockpitActionDecision::Denied(
                CockpitActionDenial::FocusaMutationDenied
            ))
        );
        // UIAI limited mode: exactly one foreground ephemeral public-observe
        // session; every action/persistence/hosted vector fails closed.
        assert_eq!(
            resolve_cockpit_action("cockpit.uiai.public_search", Some(&limited), None, 0, t),
            Ok(CockpitActionDecision::Uiai(
                UiaiCapabilityDecision::VerifiedNoLicensePublicObservation { session_quota: 1 }
            ))
        );
        assert_eq!(
            resolve_cockpit_action("cockpit.uiai.browser_click", Some(&limited), None, 0, t),
            Ok(CockpitActionDecision::Uiai(UiaiCapabilityDecision::Denied(
                UiaiCapabilityDenial::UiaiGrantRequired
            )))
        );
        // A second concurrent limited session is denied.
        assert_eq!(
            resolve_cockpit_action("cockpit.uiai.public_search", Some(&limited), None, 1, t),
            Ok(CockpitActionDecision::Uiai(UiaiCapabilityDecision::Denied(
                UiaiCapabilityDenial::LimitedModeRestricted
            )))
        );
        // Combined workflows never run on a limited posture.
        assert_eq!(
            resolve_cockpit_action(
                "cockpit.combined.research_apply",
                Some(&limited),
                None,
                0,
                t,
            ),
            Ok(CockpitActionDecision::Denied(
                CockpitActionDenial::CombinedLimitedModeDenied
            ))
        );
    }

    #[test]
    fn spec172_cockpit_unknown_action_and_no_posture_fail_closed() {
        let t = now();
        assert_eq!(
            resolve_cockpit_action("cockpit.anonymous.fabricated", None, None, 0, t),
            Err(CockpitActionError::UnknownAction)
        );
        // No posture at all: Focusa display denied, UIAI missing posture,
        // combined missing Focusa grant.
        assert_eq!(
            resolve_cockpit_action("cockpit.focusa.display_mission", None, None, 0, t),
            Ok(CockpitActionDecision::Denied(
                CockpitActionDenial::FocusaDisplayDenied
            ))
        );
        assert_eq!(
            resolve_cockpit_action("cockpit.uiai.public_search", None, None, 0, t),
            Ok(CockpitActionDecision::Uiai(UiaiCapabilityDecision::Denied(
                UiaiCapabilityDenial::MissingPosture
            )))
        );
        assert_eq!(
            resolve_cockpit_action("cockpit.combined.research_apply", None, None, 0, t),
            Ok(CockpitActionDecision::Denied(
                CockpitActionDenial::CombinedMissingFocusaGrant
            ))
        );
    }

    #[test]
    fn spec172_cockpit_offline_grace_preserves_display_and_mutation() {
        let offline = focusa_snapshot(EntitlementState::OfflineGrace, Some("account-001"));
        let t = now();
        assert_eq!(
            resolve_cockpit_action(
                "cockpit.focusa.mutate_workpoint",
                Some(&offline),
                None,
                0,
                t
            ),
            Ok(CockpitActionDecision::FocusaMutation {
                base: BaseProductDecision::Entitled
            })
        );
        assert_eq!(
            resolve_cockpit_action("cockpit.focusa.read_projection", Some(&offline), None, 0, t),
            Ok(CockpitActionDecision::FocusaDisplay {
                base: BaseProductDecision::Entitled
            })
        );
    }
}
