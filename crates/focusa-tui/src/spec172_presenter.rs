//! TUI Spec 172 presenter projection (docs/172-focusa-spec152-license-type-
//! and-surface-entitlement-governance-addendum.md §11, §15).
//!
//! The TUI, like the menubar, is a presenter, not a product (Spec 172 §15).
//! It renders the canonical License Type display, the Operator/Bundle upgrade
//! posture, the frozen node semantics (Spec 172 §7.3: approved clients on the
//! same node do not consume separate nodes), and the retained controls that
//! are never disabled by an entitlement decision (Spec 172 §5.3, §6.2).
//!
//! The projection is read-only and derived ONLY from the daemon
//! `GET /v1/license/status` payload:
//!   - a `license_type` value is projected only when it matches one of the
//!     frozen canonical codes — an unknown or caller-supplied value fails
//!     closed (a presenter never mints a License Type);
//!   - product grants are filtered to the canonical product codes;
//!   - the upgrade display is triggered only by the daemon presenter's own
//!     allowed-actions vocabulary; an actively granted License Type is
//!     managed, never re-sold as an Operator upgrade (Spec 172 §10.3);
//!   - no raw email, key, token, customer row, credential, or card data field
//!     exists by construction;
//!   - the module is pure with no module-level mutable state, so it can never
//!     cache local commercial policy.
//!
//! Frozen fixtures here mirror the menubar presenter
//! (apps/menubar/src/lib/spec172Posture.ts) and the menubar action map
//! (`spec172.locked_state_fixtures`); parity is bound by
//! tests/spec172_menubar_tui_presenter_test.mjs.

use serde_json::Value;

/// Canonical Spec 172 License Type codes
/// (docs/contracts/spec172-license-types.v1.yaml).
pub const SPEC172_LICENSE_TYPE_CODES: [&str; 3] = [
    "focusa_operator_lifetime_v1",
    "uiai_operator_lifetime_v1",
    "focusa_uiai_operator_bundle_lifetime_v1",
];

/// Canonical product codes that a Spec 172 grant may carry.
pub const SPEC172_PRODUCT_CODES: [&str; 2] = ["focusa", "uiai_engine"];

/// Frozen display labels (Spec 172 §4.1 canonical names). Labels only — no
/// prices, grants, limits, or sale status are ever rendered by a presenter.
pub fn license_type_label(code: &str) -> Option<&'static str> {
    match code {
        "focusa_operator_lifetime_v1" => Some("Focusa Operator Lifetime v1"),
        "uiai_operator_lifetime_v1" => Some("UIAI Engine Operator Lifetime v1"),
        "focusa_uiai_operator_bundle_lifetime_v1" => Some("Focusa + UIAI Operator Lifetime Bundle"),
        _ => None,
    }
}

/// Frozen node semantics (Spec 172 §7.3) — rendering sentence only; node
/// truth lives in the authority, never in a presenter counter.
pub const SPEC172_NODE_SEMANTICS: &str = "One verified operator seat and up to three registered operator nodes; CLI, TUI, Pi, menubar, Focusa Desktop, and Cockpit clients on the same node do not consume separate nodes.";

/// Frozen presenter-not-product sentence (Spec 172 §15).
pub const SPEC172_PRESENTER_NOT_PRODUCT: &str = "Menubar and TUI are presenters, not products. They project the canonical operation decision; they never own pricing, grants, limits, or commercial policy.";

/// Retained controls that are NEVER disabled by an entitlement decision
/// (Spec 172 §5.3 / §6.2; frozen fixture shared with the menubar presenter
/// and the menubar action map).
pub const SPEC172_RETAINED_CONTROLS: [&str; 9] = [
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

/// Frozen upgrade-display triggers — the ONLY presenter-accepted signals that
/// an upgrade/recovery action exists. They come from the daemon presenter
/// vocabulary (`presenter.allowed_actions`); a presenter never decides an
/// upgrade itself.
pub const SPEC172_UPGRADE_TRIGGERS: [&str; 3] = [
    "select_purchase",
    "open_checkout",
    "activate_or_manage_entitlement",
];

/// Presenter-safe Spec 172 posture projected from the daemon
/// `GET /v1/license/status` payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Spec172Posture {
    pub license_type: Option<String>,
    pub product_grants: Vec<String>,
    pub verified_no_license: bool,
    pub upgrade_available: bool,
    pub upgrade_action: String,
    pub upgrade_label: String,
    pub node_semantics: &'static str,
    pub presenter_not_product: &'static str,
    pub retained_controls: &'static [&'static str; 9],
}

impl Spec172Posture {
    /// Compact status line for the Deck Home surface.
    pub fn status_line(&self) -> String {
        let license = self
            .license_type
            .as_deref()
            .and_then(license_type_label)
            .unwrap_or(if self.verified_no_license {
                "Verified no-license limited access (no automatic expiry)"
            } else {
                "no granted License Type"
            });
        format!(
            "license_type={} grants=[{}] upgrade={} | {}",
            license,
            self.product_grants.join(","),
            self.upgrade_label,
            SPEC172_PRESENTER_NOT_PRODUCT
        )
    }

    /// Frozen locked-state accessibility fixture (Spec 172 §11.1, §5.3,
    /// §6.2): the upgrade action and the always-reachable retained controls.
    /// Identical rendering is enforced in the menubar presenter
    /// (apps/menubar/src/lib/spec172Posture.ts `lockedStateFixture`) and the
    /// menubar action map (`spec172.locked_state_fixtures`).
    pub fn locked_state_fixture(&self) -> String {
        format!(
            "locked_state_fixtures: upgrade_action={} retained_controls=[{}] never_disabled=read,export,recovery,repair,update,uninstall",
            self.upgrade_action,
            SPEC172_RETAINED_CONTROLS.join(",")
        )
    }
}

/// Project the daemon `GET /v1/license/status` payload onto the frozen
/// Spec 172 presenter posture. `None` only for non-object payloads; every
/// unknown or caller-controlled value fails closed (never mints a License
/// Type, product grant, or upgrade).
pub fn project_spec172_posture(payload: &Value) -> Option<Spec172Posture> {
    let record = payload.as_object()?;
    let status = record
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_lowercase();
    let license_type = record
        .get("license_type")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|code| SPEC172_LICENSE_TYPE_CODES.contains(code))
        .map(str::to_string);
    let product_grants = record
        .get("product_grants")
        .and_then(Value::as_array)
        .map(|values| {
            let mut seen = std::collections::BTreeSet::new();
            let mut grants = Vec::new();
            for value in values {
                if let Some(code) = value.as_str() {
                    let code = code.trim();
                    if SPEC172_PRODUCT_CODES.contains(&code) && seen.insert(code.to_string()) {
                        grants.push(code.to_string());
                    }
                }
            }
            grants
        })
        .unwrap_or_default();
    let posture_label = record
        .get("posture")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_lowercase();
    let verified_no_license = posture_label == "verified_no_license"
        || status == "unactivated"
        || status == "verified_no_license";
    let (upgrade_available, upgrade_action, upgrade_label) =
        project_upgrade(record, license_type.as_deref(), &status);
    Some(Spec172Posture {
        license_type,
        product_grants,
        verified_no_license,
        upgrade_available,
        upgrade_action,
        upgrade_label,
        node_semantics: SPEC172_NODE_SEMANTICS,
        presenter_not_product: SPEC172_PRESENTER_NOT_PRODUCT,
        retained_controls: &SPEC172_RETAINED_CONTROLS,
    })
}

fn project_upgrade(
    record: &serde_json::Map<String, Value>,
    license_type: Option<&str>,
    status: &str,
) -> (bool, String, String) {
    let allowed = record
        .get("presenter")
        .and_then(|presenter| presenter.get("allowed_actions"))
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect::<Vec<String>>()
        })
        .unwrap_or_default();
    let triggered = allowed
        .iter()
        .any(|action| SPEC172_UPGRADE_TRIGGERS.contains(&action.as_str()));
    // Accurate display: an actively granted License Type is managed, never
    // re-sold as an Operator upgrade by a presenter (Spec 172 §10.3).
    let active_grant = license_type.is_some() && (status == "active" || status == "offline_grace");
    let available = triggered && !active_grant;
    if available {
        (
            true,
            "activate_or_manage_entitlement".to_string(),
            if license_type.is_some() {
                "Manage entitlement".to_string()
            } else {
                "Operator upgrade available".to_string()
            },
        )
    } else {
        (
            false,
            "manage".to_string(),
            "Manage entitlement".to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn verified_no_license_projects_limited_posture_with_operator_upgrade() {
        let payload = json!({
            "status": "verified_no_license",
            "posture": "verified_no_license",
            "license_type": null,
            "product_grants": [],
            "presenter": {
                "presenter_state": "selection_required",
                "allowed_actions": ["select_purchase", "select_limited_access", "select_existing_key"]
            }
        });
        let posture = project_spec172_posture(&payload).expect("projects");
        assert_eq!(posture.license_type, None);
        assert!(posture.product_grants.is_empty());
        assert!(posture.verified_no_license);
        assert!(posture.upgrade_available);
        assert_eq!(posture.upgrade_action, "activate_or_manage_entitlement");
        assert_eq!(posture.upgrade_label, "Operator upgrade available");
        assert!(
            posture
                .status_line()
                .contains("Verified no-license limited access (no automatic expiry)")
        );
    }

    #[test]
    fn operator_and_bundle_active_grants_are_managed_not_resold() {
        let focusa = json!({
            "status": "active",
            "license_type": "focusa_operator_lifetime_v1",
            "product_grants": ["focusa"],
            "presenter": {
                "presenter_state": "activated",
                "allowed_actions": ["resume"]
            }
        });
        let posture = project_spec172_posture(&focusa).expect("projects");
        assert_eq!(
            posture.license_type.as_deref(),
            Some("focusa_operator_lifetime_v1")
        );
        assert_eq!(posture.product_grants, vec!["focusa".to_string()]);
        assert!(!posture.upgrade_available);
        assert_eq!(posture.upgrade_label, "Manage entitlement");

        let bundle = json!({
            "status": "active",
            "license_type": "focusa_uiai_operator_bundle_lifetime_v1",
            "product_grants": ["focusa", "uiai_engine"],
            "presenter": {
                "presenter_state": "activated",
                "allowed_actions": ["resume"]
            }
        });
        let posture = project_spec172_posture(&bundle).expect("projects");
        assert_eq!(
            posture.license_type.as_deref(),
            Some("focusa_uiai_operator_bundle_lifetime_v1")
        );
        assert_eq!(
            posture.product_grants,
            vec!["focusa".to_string(), "uiai_engine".to_string()]
        );
        assert!(!posture.upgrade_available);
        assert_eq!(posture.upgrade_label, "Manage entitlement");
        assert!(
            posture
                .status_line()
                .contains("Focusa + UIAI Operator Lifetime Bundle")
        );
    }

    #[test]
    fn unknown_and_caller_supplied_values_fail_closed() {
        // A caller-supplied License Type never projects.
        let unknown = json!({
            "status": "active",
            "license_type": "mega_gold_platinum_v9",
            "product_grants": ["focusa", "everything_else"],
            "presenter": {"allowed_actions": ["select_purchase"]}
        });
        let posture = project_spec172_posture(&unknown).expect("projects");
        assert_eq!(posture.license_type, None);
        assert_eq!(posture.product_grants, vec!["focusa".to_string()]);
        // Non-object payload fails closed.
        assert_eq!(project_spec172_posture(&json!([1, 2, 3])), None);
    }

    #[test]
    fn denied_posture_keeps_recovery_upgrade_and_retained_controls() {
        let denied = json!({
            "status": "recovery_only",
            "authority": {"recovery_reason": "lease_revoked"},
            "presenter": {
                "presenter_state": "denied",
                "allowed_actions": ["activate_or_manage_entitlement", "recovery"]
            }
        });
        let posture = project_spec172_posture(&denied).expect("projects");
        assert!(posture.upgrade_available);
        assert_eq!(posture.upgrade_action, "activate_or_manage_entitlement");
        assert!(posture
            .locked_state_fixture()
            .contains("retained_controls=[navigation,status,account,read,export,recovery,repair,update,uninstall]"));
        assert_eq!(SPEC172_RETAINED_CONTROLS.len(), 9);
        for control in SPEC172_RETAINED_CONTROLS {
            assert!(SPEC172_RETAINED_CONTROLS.contains(&control));
        }
    }

    #[test]
    fn frozen_fixtures_match_menubar_contract() {
        assert_eq!(SPEC172_LICENSE_TYPE_CODES.len(), 3);
        assert_eq!(SPEC172_PRODUCT_CODES.len(), 2);
        assert_eq!(SPEC172_UPGRADE_TRIGGERS.len(), 3);
        assert!(SPEC172_NODE_SEMANTICS.contains("do not consume separate nodes"));
        assert!(SPEC172_PRESENTER_NOT_PRODUCT.contains("presenters, not products"));
        assert_eq!(
            license_type_label("focusa_uiai_operator_bundle_lifetime_v1"),
            Some("Focusa + UIAI Operator Lifetime Bundle")
        );
        assert_eq!(license_type_label("navigator_future_v1"), None);
    }
}
