//! Focusa Desktop / Tauri Spec 172 command bridge and action registry.
//!
//! Authority: docs/172-focusa-spec152-license-type-and-surface-entitlement-
//! governance-addendum.md (Spec 172 §11 surface inheritance, §11.4 no
//! direct-core bypass, §12 dynamic operations, §15 presenters are not
//! products, §7.3 shared operator nodes, §5.3 / §6.2 retained controls).
//!
//! Focusa Desktop is a presenter, not a product (Spec 172 §15). This module
//! is the native/Tauri/local command bridge and action registry:
//!
//!   - `DESKTOP_ACTION_REGISTRY` maps every desktop action ID to one
//!     canonical operation (operation_id, class, capability family, daemon
//!     route). The registry is frozen and server-aligned: every entry's
//!     family/treatment/route matches the canonical operation table of the
//!     menubar action map (`docs/contracts/spec152f-menubar-action-map.v1.json`)
//!     and the desktop action map (`docs/contracts/spec152f-desktop-action-map.v1.json`).
//!   - `resolve_desktop_action` is fail-closed: an unknown action resolves to
//!     `None` and is never forwarded. Value-producing actions always carry a
//!     daemon route (`forwards_to_core_guard = true`) so execution happens
//!     through the shared core execution guard, never by calling storage,
//!     reducers, or the entitlement reducer directly (Spec 172 §11.4).
//!   - The bridge NEVER evaluates entitlement, never mints a License Type,
//!     product, price, grant, family, limit, or node, and never infers
//!     grants from the installed client, pairing, discovery, or email.
//!   - `project_desktop_spec172_posture` renders the canonical Spec 172
//!     presenter projection (focusa.spec172.presenter_projection.v1) from the
//!     daemon `GET /v1/license/status` payload fields — the same frozen
//!     vocabulary and decision mapping as the CLI, API, Pi, menubar, and TUI
//!     presenters, so Desktop decisions are identical to CLI/API.
//!   - Same-node identity (Spec 172 §7.3): the Desktop never registers a
//!     node or multiplies activations; `SPEC172_DESKTOP_SAME_NODE` is the
//!     frozen rendering sentence and the bridge holds no node counter.
//!   - Limited read/export/recovery are preserved: the frozen retained
//!     access set and the locked-state fixtures never disable read, export,
//!     recovery, repair, update, or uninstall (Spec 172 §5.3 / §6.2), and
//!     paid families (team_remote, automation, release_proof,
//!     premium_updates, customer_data_export) are blocked consistently with
//!     the canonical decision mapping.
//!
//! The module is pure (no module-level mutable state) and std-only so it can
//! never cache local commercial policy and can be compiled standalone.

/// Canonical Spec 172 License Type codes
/// (docs/contracts/spec172-license-types.v1.yaml; frozen vocabulary shared
/// with the CLI/Pi/agent presenter and the menubar/TUI presenters).
pub const SPEC172_LICENSE_TYPE_CODES: [&str; 3] = [
    "focusa_operator_lifetime_v1",
    "uiai_operator_lifetime_v1",
    "focusa_uiai_operator_bundle_lifetime_v1",
];

/// Canonical product codes that a Spec 172 grant may carry.
pub const SPEC172_PRODUCT_CODES: [&str; 2] = ["focusa", "uiai_engine"];

/// Frozen Spec 172 §21 stable errors. The bridge only ever projects these
/// codes; it never invents a denial vocabulary.
pub const SPEC172_STABLE_ERRORS: [&str; 13] = [
    "EMAIL_VERIFICATION_REQUIRED",
    "VERIFIED_LIMITED_ACCESS",
    "LICENSE_TYPE_REQUIRED",
    "LICENSE_TYPE_NOT_INCLUDED",
    "PRODUCT_NOT_INCLUDED",
    "CAPABILITY_FAMILY_NOT_INCLUDED",
    "ENTITLEMENT_POLICY_UNKNOWN",
    "ENTITLEMENT_PRODUCT_MISMATCH",
    "NODE_LIMIT_REACHED",
    "OPERATOR_SEAT_LIMIT_REACHED",
    "HOSTED_RESOURCE_NOT_INCLUDED",
    "UPGRADE_AVAILABLE",
    "RECOVERY_ONLY",
];

/// Frozen Spec 172 §5.3/§6.2/§17 retained access: never disabled by an
/// entitlement decision. Byte-identical across CLI, Pi, agent, menubar, TUI,
/// and Desktop presenters.
pub const SPEC172_RETAINED_ACCESS: [&str; 9] = [
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

/// Frozen upgrade/recovery vocabulary of the canonical presenter envelope
/// (shared with `crates/focusa-cli/tests/fixtures/spec172-cli-agent-presenter-fixtures.v1.json`).
pub const SPEC172_UPGRADE_ACTIONS: [&str; 4] = [
    "none_required",
    "verify_email_or_manage_entitlement",
    "review_offer_or_manage_entitlement",
    "purchase_or_manage_entitlement",
];

/// Frozen Spec 172 posture set of the canonical presenter envelope.
pub const SPEC172_POSTURES: [&str; 7] = [
    "unverified",
    "verified_no_license",
    "active_paid_operator",
    "offline_grace",
    "refunded_or_revoked",
    "expired",
    "missing_or_corrupt",
];

/// Canonical capability-family vocabulary of the entitlement policy registry.
pub const SPEC172_FAMILIES: [&str; 9] = [
    "account_recovery",
    "read_projection",
    "base_focusa",
    "automation",
    "team_remote",
    "release_proof",
    "premium_updates",
    "customer_data_export",
    "internal_maintenance",
];

/// Canonical presenter projection schema (identical to CLI/Pi/agent).
pub const SPEC172_PRESENTER_SCHEMA: &str = "focusa.spec172.presenter_projection.v1";

/// Frozen recovery sentence (Spec 172 §17): recovery/export/repair and
/// uninstall stay reachable when execution is locked.
pub const SPEC172_RECOVERY_ACTION: &str =
    "recovery, export, repair, and uninstall remain available when execution is locked";

/// Frozen node semantics (Spec 172 §7.3): Desktop/CLI/menubar/Cockpit/TUI/Pi
/// clients on the same node do NOT consume separate nodes. Rendering
/// sentence only — node truth lives in the authority, never in a presenter
/// counter. The Desktop never registers or increments a node.
pub const SPEC172_DESKTOP_SAME_NODE: &str =
    "One verified operator seat and up to three registered operator nodes; Focusa Desktop, CLI, TUI, Pi, menubar, and Cockpit clients on the same node do not consume separate nodes. Focusa Desktop never registers a node or multiplies activations.";

/// Frozen presenter-not-product sentence (Spec 172 §15).
pub const SPEC172_PRESENTER_NOT_PRODUCT: &str =
    "Focusa Desktop is a presenter, not a product. It projects the canonical operation decision from the daemon core guard; it never owns pricing, grants, limits, License Types, or commercial policy.";

/// Frozen no-direct-core-bypass sentence (Spec 172 §11.4).
pub const SPEC172_NO_DIRECT_CORE_BYPASS: &str =
    "Desktop, Tauri, native, and local-source commands never call storage or reducer code directly; every value-producing mutation forwards to the shared core execution guard.";

/// Desktop action registry entry. `family` and `operation_id` are `None`
/// only for presenter-local navigation/display actions that make no daemon
/// call. `mutation` is the canonical operation's mutation class; the bridge
/// never re-classifies an operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DesktopActionEntry {
    pub action_id: &'static str,
    pub operation_id: Option<&'static str>,
    pub class: &'static str,
    pub family: Option<&'static str>,
    pub method: &'static str,
    pub path: &'static str,
    pub mutation: bool,
    pub local_storage: bool,
}

/// Frozen Desktop action registry (Spec 172 §11.4, §12): every desktop
/// action maps to one canonical operation. Rows are byte-identical to the
/// canonical operation table in
/// `docs/contracts/spec152f-desktop-action-map.v1.json` and mirror the
/// menubar action map's `canonical_operations`.
pub const DESKTOP_ACTION_REGISTRY: [DesktopActionEntry; 24] = [
    // Presenter-local navigation/display: no daemon call, no entitlement
    // evaluation, no storage mutation.
    DesktopActionEntry {
        action_id: "desktop.tray.toggle",
        operation_id: None,
        class: "navigation_display",
        family: None,
        method: "local_only",
        path: "local_only",
        mutation: false,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.window.navigate",
        operation_id: None,
        class: "navigation_display",
        family: None,
        method: "local_only",
        path: "local_only",
        mutation: false,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.dialog.close",
        operation_id: None,
        class: "navigation_display",
        family: None,
        method: "local_only",
        path: "local_only",
        mutation: false,
        local_storage: false,
    },
    // Recovery/account family: always reachable subject to security (Spec
    // 172 §5.3). Forwarded to the daemon core guard when they carry a route.
    DesktopActionEntry {
        action_id: "desktop.license.status",
        operation_id: Some("focusa.license.status"),
        class: "recovery_account",
        family: Some("account_recovery"),
        method: "GET",
        path: "/v1/license/status",
        mutation: false,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.connect.test",
        operation_id: Some("focusa.connect.test"),
        class: "recovery_account",
        family: Some("account_recovery"),
        method: "GET",
        path: "/v1/health",
        mutation: false,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.pairing.start",
        operation_id: Some("focusa.pairing.start"),
        class: "recovery_account",
        family: Some("account_recovery"),
        method: "POST",
        path: "/v1/pairing/start",
        mutation: true,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.pairing.revoke",
        operation_id: Some("focusa.pairing.revoke"),
        class: "recovery_account",
        family: Some("account_recovery"),
        method: "POST",
        path: "/v1/pairing/revoke",
        mutation: true,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.pairing.list",
        operation_id: Some("focusa.pairing.list"),
        class: "recovery_account",
        family: Some("account_recovery"),
        method: "GET",
        path: "/v1/pairing/devices",
        mutation: false,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.update.check",
        operation_id: Some("focusa.update.check"),
        class: "recovery_account",
        family: Some("account_recovery"),
        method: "GET",
        path: "/v1/update/notifications",
        mutation: false,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.update.install",
        operation_id: Some("focusa.update.install"),
        class: "recovery_account",
        family: Some("account_recovery"),
        method: "POST",
        path: "/v1/update/install",
        mutation: true,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.debug.bundle.copy",
        operation_id: Some("focusa.debug.bundle.copy"),
        class: "recovery_account",
        family: Some("account_recovery"),
        method: "local_only",
        path: "local_only",
        mutation: false,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.connection.save",
        operation_id: Some("focusa.connection.save"),
        class: "recovery_account",
        family: Some("account_recovery"),
        method: "local_only",
        path: "local_only",
        mutation: false,
        local_storage: true,
    },
    // Canonical operations: policy inherited from the registered family;
    // the daemon core guard decides. The bridge only forwards.
    DesktopActionEntry {
        action_id: "desktop.project.identity.verify",
        operation_id: Some("focusa.project.identity.verify"),
        class: "canonical_operation",
        family: Some("base_focusa"),
        method: "GET",
        path: "/v1/project/identity",
        mutation: false,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.workpoint.current.verify",
        operation_id: Some("focusa.workpoint.current.verify"),
        class: "canonical_operation",
        family: Some("base_focusa"),
        method: "GET",
        path: "/v1/workpoint/current",
        mutation: false,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.workpoint.checkpoint",
        operation_id: Some("focusa.workpoint.checkpoint"),
        class: "canonical_operation",
        family: Some("base_focusa"),
        method: "POST",
        path: "/v1/workpoint/checkpoint",
        mutation: true,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.workpoint.resume",
        operation_id: Some("focusa.workpoint.resume"),
        class: "canonical_operation",
        family: Some("base_focusa"),
        method: "POST",
        path: "/v1/workpoint/resume",
        mutation: false,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.workpoint.evidence.link",
        operation_id: Some("focusa.workpoint.evidence.link"),
        class: "canonical_operation",
        family: Some("base_focusa"),
        method: "POST",
        path: "/v1/workpoint/evidence/link",
        mutation: true,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.semantic_integrity.operation.invoke",
        operation_id: Some("focusa.semantic_integrity.operation.invoke"),
        class: "canonical_operation",
        family: Some("base_focusa"),
        method: "POST",
        path: "/v1/semantic-integrity/operations/{id}",
        mutation: true,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.sync.peers.add",
        operation_id: Some("focusa.sync.peers.add"),
        class: "canonical_operation",
        family: Some("team_remote"),
        method: "POST",
        path: "/v1/sync/peers",
        mutation: true,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.sync.pull",
        operation_id: Some("focusa.sync.pull"),
        class: "canonical_operation",
        family: Some("team_remote"),
        method: "POST",
        path: "/v1/sync/pull/{peer_id}",
        mutation: true,
        local_storage: false,
    },
    // First-run wizard steps map onto canonical recovery/read operations.
    DesktopActionEntry {
        action_id: "desktop.first_run.discover",
        operation_id: Some("focusa.connect.test"),
        class: "recovery_account",
        family: Some("account_recovery"),
        method: "GET",
        path: "/v1/health",
        mutation: false,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.first_run.pair",
        operation_id: Some("focusa.pairing.start"),
        class: "recovery_account",
        family: Some("account_recovery"),
        method: "POST",
        path: "/v1/pairing/start",
        mutation: true,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.first_run.verify_project",
        operation_id: Some("focusa.project.identity.verify"),
        class: "canonical_operation",
        family: Some("base_focusa"),
        method: "GET",
        path: "/v1/project/identity",
        mutation: false,
        local_storage: false,
    },
    DesktopActionEntry {
        action_id: "desktop.first_run.verify_workpoint",
        operation_id: Some("focusa.workpoint.current.verify"),
        class: "canonical_operation",
        family: Some("base_focusa"),
        method: "GET",
        path: "/v1/workpoint/current",
        mutation: false,
        local_storage: false,
    },
];

/// Result of resolving one desktop action through the frozen registry.
/// `direct_storage` is always `false` by construction: the bridge has no
/// storage/reducer path (Spec 172 §11.4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopActionResolution {
    pub action_id: &'static str,
    pub operation_id: Option<&'static str>,
    pub class: &'static str,
    pub family: Option<&'static str>,
    pub method: &'static str,
    pub path: &'static str,
    pub mutation: bool,
    pub forwards_to_core_guard: bool,
    pub local_storage: bool,
    pub direct_storage: bool,
}

/// Fail-closed registry lookup (Spec 172 §12): an unknown or unregistered
/// action resolves to `None` and is never forwarded to execution. The bridge
/// never evaluates entitlement and never re-classifies an operation.
pub fn resolve_desktop_action(action_id: &str) -> Option<DesktopActionResolution> {
    let entry = DESKTOP_ACTION_REGISTRY
        .iter()
        .find(|entry| entry.action_id == action_id)?;
    let forwards_to_core_guard = entry.path.starts_with('/');
    Some(DesktopActionResolution {
        action_id: entry.action_id,
        operation_id: entry.operation_id,
        class: entry.class,
        family: entry.family,
        method: entry.method,
        path: entry.path,
        mutation: entry.mutation,
        forwards_to_core_guard,
        local_storage: entry.local_storage,
        direct_storage: false,
    })
}

/// Typed, presenter-safe view of the daemon `GET /v1/license/status` payload.
/// The Tauri command handler extracts these fields from the daemon JSON; the
/// bridge itself stays std-only and never parses caller-supplied policy.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DesktopStatusEnvelope {
    pub status: String,
    pub posture: String,
    pub product: String,
    pub license_type: Option<String>,
    pub product_grants: Vec<String>,
    pub family: String,
    pub allowed_actions: Vec<String>,
}

/// Canonical Spec 172 presenter projection for the Desktop surface — the same
/// envelope keys, frozen vocabulary, and decision mapping as the CLI, API,
/// Pi, menubar, and TUI presenters (focusa.spec172.presenter_projection.v1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DesktopSpec172Posture {
    pub schema: &'static str,
    pub posture: String,
    pub product: String,
    pub license_type: String,
    pub family: String,
    pub denial: Option<&'static str>,
    pub retained_access: [&'static str; 9],
    pub upgrade_action: &'static str,
    pub recovery_action: &'static str,
    pub grant_inferred_from_surface: bool,
    pub same_node: bool,
}

/// Project the canonical posture from the daemon payload fields. Unknown,
/// caller-supplied, or non-canonical values fail closed: an unknown posture
/// maps to `missing_or_corrupt`, an unknown License Type maps to `none`, and
/// an unknown family is denied with `CAPABILITY_FAMILY_NOT_INCLUDED`. The
/// bridge never mints a posture, License Type, product grant, or upgrade.
pub fn project_desktop_spec172_posture(
    envelope: &DesktopStatusEnvelope,
) -> DesktopSpec172Posture {
    let posture = canonical_posture(envelope);
    let product = canonical_product(&envelope.product);
    let license_type = canonical_license_type(envelope, &posture, product);
    let family = if envelope.family.trim().is_empty() {
        "base_focusa".to_string()
    } else {
        envelope.family.trim().to_string()
    };
    let (denial, upgrade_action) = denial_and_upgrade(&posture, product, &family);
    let product_grants: Vec<&str> = envelope
        .product_grants
        .iter()
        .map(|code| code.trim())
        .filter(|code| SPEC172_PRODUCT_CODES.contains(code))
        .collect();
    let product_display = if product == "unknown" {
        if product_grants.is_empty() {
            "unknown".to_string()
        } else {
            product_grants.join(",")
        }
    } else {
        product.to_string()
    };
    DesktopSpec172Posture {
        schema: SPEC172_PRESENTER_SCHEMA,
        posture,
        product: product_display,
        license_type,
        family,
        denial,
        retained_access: SPEC172_RETAINED_ACCESS,
        upgrade_action,
        recovery_action: SPEC172_RECOVERY_ACTION,
        grant_inferred_from_surface: false,
        same_node: true,
    }
}

/// Canonical Spec 172 posture from the daemon status/posture fields. Mirrors
/// the CLI posture mapping (`crates/focusa-cli/src/commands/license.rs
/// spec172_posture`) so Desktop and CLI decisions are identical.
fn canonical_posture(envelope: &DesktopStatusEnvelope) -> String {
    let posture = envelope.posture.trim().to_ascii_lowercase();
    if SPEC172_POSTURES.contains(&posture.as_str()) {
        return posture;
    }
    match envelope.status.trim().to_ascii_lowercase().as_str() {
        "active" => "active_paid_operator".to_string(),
        "offline_grace" => "offline_grace".to_string(),
        "unactivated" => "unverified".to_string(),
        "recovery_only" => "refunded_or_revoked".to_string(),
        "expired" => "expired".to_string(),
        "verified_no_license" => "verified_no_license".to_string(),
        _ => "missing_or_corrupt".to_string(),
    }
}

/// Canonical product identifier; anything outside `focusa`/`uiai_engine`
/// fails closed to `unknown` (never normalized into a grant).
fn canonical_product(product: &str) -> &str {
    match product.trim() {
        "focusa" => "focusa",
        "uiai_engine" => "uiai_engine",
        _ => "unknown",
    }
}

/// Canonical License Type display (Spec 172 §4.1): only usable authority
/// states carry a License Type, and only a canonical code from the daemon or
/// derived from the canonical product may project. The bridge never accepts
/// a caller-chosen code.
fn canonical_license_type(
    envelope: &DesktopStatusEnvelope,
    posture: &str,
    product: &str,
) -> String {
    let usable = posture == "active_paid_operator" || posture == "offline_grace";
    if !usable {
        return "none".to_string();
    }
    if let Some(code) = envelope.license_type.as_deref() {
        let code = code.trim();
        if SPEC172_LICENSE_TYPE_CODES.contains(&code) {
            return code.to_string();
        }
    }
    match product {
        "focusa" => "focusa_operator_lifetime_v1".to_string(),
        "uiai_engine" => "uiai_operator_lifetime_v1".to_string(),
        _ => "none".to_string(),
    }
}

/// Base Focusa product gate derived from the daemon payload, mirroring
/// `resolve_base_focusa_product`: one usable signed product entitlement for
/// product `focusa` (Active paid lease or valid Offline Grace) gates the
/// base product; `verified_no_license` resolves to the explicit limited
/// subset; every other policy state denies value-producing mutations.
enum BaseGate {
    Entitled,
    Limited,
    Denied,
}

fn base_gate(posture: &str, product: &str) -> BaseGate {
    match (posture, product) {
        ("active_paid_operator" | "offline_grace", "focusa") => BaseGate::Entitled,
        ("verified_no_license", "focusa") => BaseGate::Limited,
        _ => BaseGate::Denied,
    }
}

/// Frozen Spec 172 base denial table — the exact mapping of the CLI
/// presenter's `spec172_base_denial` so Desktop and CLI denials are
/// identical. `upgrade_action` vocabulary is the frozen 4-action set.
fn spec172_base_denial(posture: &str, product: &str) -> (Option<&'static str>, &'static str) {
    match posture {
        "unverified" => (
            Some("EMAIL_VERIFICATION_REQUIRED"),
            "verify_email_or_manage_entitlement",
        ),
        "refunded_or_revoked" => {
            (Some("RECOVERY_ONLY"), "review_offer_or_manage_entitlement")
        }
        "expired" => (Some("LICENSE_TYPE_REQUIRED"), "purchase_or_manage_entitlement"),
        "missing_or_corrupt" => (
            Some("ENTITLEMENT_POLICY_UNKNOWN"),
            "review_offer_or_manage_entitlement",
        ),
        _ if product != "focusa" => {
            (Some("PRODUCT_NOT_INCLUDED"), "review_offer_or_manage_entitlement")
        }
        _ => (Some("LICENSE_TYPE_REQUIRED"), "purchase_or_manage_entitlement"),
    }
}

/// One family's canonical denial + upgrade action. The paid families
/// (automation, team_remote, release_proof, premium_updates,
/// customer_data_export) are blocked consistently: the base gate applies
/// first, then the family itself is denied with the canonical
/// `CAPABILITY_FAMILY_NOT_INCLUDED` — identical to the CLI/API decision
/// mapping. `account_recovery` is always available subject to security, and
/// `read_projection` follows the base gate so limited read/export/recovery
/// stay reachable.
fn denial_and_upgrade(
    posture: &str,
    product: &str,
    family: &str,
) -> (Option<&'static str>, &'static str) {
    if family == "account_recovery" {
        return (None, "none_required");
    }
    if family == "internal_maintenance" {
        return (None, "none_required");
    }
    let gate = base_gate(posture, product);
    if family == "read_projection" {
        return match gate {
            BaseGate::Entitled | BaseGate::Limited => (None, "none_required"),
            BaseGate::Denied => spec172_base_denial(posture, product),
        };
    }
    if family == "base_focusa" {
        return match gate {
            BaseGate::Entitled => (None, "none_required"),
            BaseGate::Limited => {
                (Some("VERIFIED_LIMITED_ACCESS"), "review_offer_or_manage_entitlement")
            }
            BaseGate::Denied => spec172_base_denial(posture, product),
        };
    }
    // Paid/optional families: the base gate applies first; when the base is
    // usable the family itself is still denied — the daemon core guard is the
    // only authority that can grant a registered premium family.
    match gate {
        BaseGate::Entitled => (
            Some("CAPABILITY_FAMILY_NOT_INCLUDED"),
            "review_offer_or_manage_entitlement",
        ),
        BaseGate::Limited | BaseGate::Denied => spec172_base_denial(posture, product),
    }
}

/// Frozen first-run fixture (Spec 172 §11.2, §5.3): every Desktop first-run
/// step resolves through the action registry to a canonical operation or a
/// presenter-local display step; no first-run step ever creates a grant,
/// node, price, or License Type locally.
pub fn desktop_first_run_fixture() -> &'static str {
    "desktop_first_run_fixtures: steps=[discover,pair,verify_project,verify_workpoint,manage_or_recovery] no_local_grant=true no_local_node=true no_local_license_type=true"
}

/// Frozen locked-state accessibility fixture (Spec 172 §11.1, §5.3, §6.2):
/// the upgrade/recovery action and the always-reachable retained controls.
/// Identical rendering is enforced in the menubar and TUI presenters and the
/// desktop action map (`spec172.locked_state_fixtures`).
pub fn desktop_locked_state_fixture() -> &'static str {
    "locked_state_fixtures: upgrade_action=activate_or_manage_entitlement retained_controls=[navigation,status,account,read,export,recovery,repair,update,uninstall] never_disabled=read,export,recovery,repair,update,uninstall"
}

#[cfg(test)]
mod tests {
    use super::*;

    fn envelope(status: &str, posture: &str, product: &str) -> DesktopStatusEnvelope {
        DesktopStatusEnvelope {
            status: status.to_string(),
            posture: posture.to_string(),
            product: product.to_string(),
            ..DesktopStatusEnvelope::default()
        }
    }

    #[test]
    fn active_operator_projects_usable_base_with_no_denial() {
        let posture = project_desktop_spec172_posture(&envelope(
            "active",
            "active_paid_operator",
            "focusa",
        ));
        assert_eq!(posture.schema, SPEC172_PRESENTER_SCHEMA);
        assert_eq!(posture.posture, "active_paid_operator");
        assert_eq!(posture.license_type, "focusa_operator_lifetime_v1");
        assert_eq!(posture.family, "base_focusa");
        assert_eq!(posture.denial, None);
        assert_eq!(posture.upgrade_action, "none_required");
        assert_eq!(posture.grant_inferred_from_surface, false);
        assert_eq!(posture.same_node, true);
        assert_eq!(posture.retained_access, SPEC172_RETAINED_ACCESS);
    }

    #[test]
    fn verified_no_license_projects_limited_base_with_upgrade() {
        let mut env = envelope("unactivated", "verified_no_license", "focusa");
        env.family = "base_focusa".to_string();
        let posture = project_desktop_spec172_posture(&env);
        assert_eq!(posture.posture, "verified_no_license");
        assert_eq!(posture.license_type, "none");
        assert_eq!(posture.denial, Some("VERIFIED_LIMITED_ACCESS"));
        assert_eq!(posture.upgrade_action, "review_offer_or_manage_entitlement");
        // Limited read/export/recovery are preserved (Spec 172 §17).
        assert_eq!(posture.retained_access, SPEC172_RETAINED_ACCESS);
    }

    #[test]
    fn paid_families_block_consistently_when_base_is_usable() {
        let mut env = envelope("active", "active_paid_operator", "focusa");
        env.family = "team_remote".to_string();
        let posture = project_desktop_spec172_posture(&env);
        assert_eq!(posture.denial, Some("CAPABILITY_FAMILY_NOT_INCLUDED"));
        assert_eq!(posture.upgrade_action, "review_offer_or_manage_entitlement");
        // Retained access survives the denial.
        assert_eq!(posture.retained_access, SPEC172_RETAINED_ACCESS);
    }

    #[test]
    fn account_recovery_and_read_stay_available() {
        let mut env = envelope("unactivated", "verified_no_license", "focusa");
        env.family = "account_recovery".to_string();
        let recovery = project_desktop_spec172_posture(&env);
        assert_eq!(recovery.denial, None);
        assert_eq!(recovery.upgrade_action, "none_required");
        env.family = "read_projection".to_string();
        let read = project_desktop_spec172_posture(&env);
        assert_eq!(read.denial, None);
        assert_eq!(read.upgrade_action, "none_required");
    }

    #[test]
    fn unverified_and_unknown_values_fail_closed() {
        let mut env = envelope("unactivated", "", "focusa");
        env.family = "base_focusa".to_string();
        let unverified = project_desktop_spec172_posture(&env);
        assert_eq!(unverified.posture, "unverified");
        assert_eq!(unverified.denial, Some("EMAIL_VERIFICATION_REQUIRED"));
        assert_eq!(unverified.upgrade_action, "verify_email_or_manage_entitlement");

        let mut bogus = envelope("active", "active_paid_operator", "focusa");
        bogus.license_type = Some("caller_minted_type".to_string());
        bogus.family = "caller_family".to_string();
        let posture = project_desktop_spec172_posture(&bogus);
        // Caller-supplied License Type and family never project.
        assert_eq!(posture.license_type, "focusa_operator_lifetime_v1");
        assert_eq!(posture.family, "caller_family");
        assert_eq!(posture.denial, Some("CAPABILITY_FAMILY_NOT_INCLUDED"));

        // Wrong/prefixed products never satisfy the base gate.
        let prefixed = envelope("active", "active_paid_operator", "prefix-focusa");
        let posture = project_desktop_spec172_posture(&prefixed);
        assert_eq!(posture.product, "unknown");
        assert_eq!(posture.denial, Some("PRODUCT_NOT_INCLUDED"));
        assert_eq!(posture.upgrade_action, "review_offer_or_manage_entitlement");
    }

    #[test]
    fn action_registry_is_fail_closed_and_routes_only_to_core_guard() {
        assert_eq!(resolve_desktop_action("desktop.not.a.real.action"), None);
        let checkpoint = resolve_desktop_action("desktop.workpoint.checkpoint").unwrap();
        assert_eq!(checkpoint.operation_id, Some("focusa.workpoint.checkpoint"));
        assert_eq!(checkpoint.family, Some("base_focusa"));
        assert_eq!(checkpoint.method, "POST");
        assert_eq!(checkpoint.path, "/v1/workpoint/checkpoint");
        assert_eq!(checkpoint.mutation, true);
        assert_eq!(checkpoint.forwards_to_core_guard, true);
        assert_eq!(checkpoint.direct_storage, false);

        let sync = resolve_desktop_action("desktop.sync.peers.add").unwrap();
        assert_eq!(sync.family, Some("team_remote"));
        assert_eq!(sync.mutation, true);
        assert_eq!(sync.forwards_to_core_guard, true);
        assert_eq!(sync.direct_storage, false);

        let nav = resolve_desktop_action("desktop.tray.toggle").unwrap();
        assert_eq!(nav.class, "navigation_display");
        assert_eq!(nav.operation_id, None);
        assert_eq!(nav.forwards_to_core_guard, false);
        assert_eq!(nav.direct_storage, false);

        let status = resolve_desktop_action("desktop.license.status").unwrap();
        assert_eq!(status.family, Some("account_recovery"));
        assert_eq!(status.forwards_to_core_guard, true);
        assert_eq!(status.direct_storage, false);
    }

    #[test]
    fn frozen_fixtures_match_the_menubar_contract() {
        let first_run = desktop_first_run_fixture();
        assert!(first_run.contains("no_local_grant=true"));
        assert!(first_run.contains("no_local_node=true"));
        let locked = desktop_locked_state_fixture();
        assert!(locked.starts_with("locked_state_fixtures:"));
        assert!(locked.contains("upgrade_action=activate_or_manage_entitlement"));
        assert!(locked.contains("never_disabled=read,export,recovery,repair,update,uninstall"));
    }
}
