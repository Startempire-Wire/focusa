// Spec 152F §7 Pi/agent tool surface inheritance.
//
// The Pi/agent policy adapter projects the canonical entitlement decision into
// every Pi tool surface. It resolves the operation policy from the canonical
// tool registry (FOCUSA_TOOL_CONTRACTS), preflights value-producing mutations
// before side effects, returns stable machine JSON, and exposes recovery
// actions.
//
// Authority boundary (Spec 152F §5): licensing grants capability only. It
// never grants operator authority, cognitive authority, Workstream authority,
// Focus State authority, Trajectory authority, Workpoint authority, role
// permission, or mutation confirmation. Operator permission and confirmation
// are preserved independently and are NEVER inferred from entitlement state.
// Tool discovery/visibility is advisory routing metadata and never grants
// entitlement (Spec 152F §7 Pi/agent tool row: "Treating tool availability as
// entitlement" is forbidden).
//
// Deterministic and dependency-free so tests can import it directly.
import { FOCUSA_TOOL_CONTRACTS, } from "./tool-contracts.js";
export const ENTITLEMENT_DECISION_SCHEMA = "focusa.entitlement_decision.v1";
export const LICENSE_STATUS_PATH = "/v1/license/status";
const VALUE_MUTATION_FAMILIES = new Set([
    "base_focusa",
    "automation",
    "team_remote",
    "release_proof",
    "premium_updates",
]);
/** Resolve the canonical operation policy for a named tool; fail-closed null. */
export function resolveOperationPolicyForTool(toolName) {
    if (!toolName || typeof toolName !== "string")
        return null;
    const contract = FOCUSA_TOOL_CONTRACTS.find((entry) => entry.name === toolName);
    return contract?.operation_policy ?? null;
}
/** Canonical recovery actions derived from the policy recovery allowance. */
export function recoveryActionsFor(policy) {
    switch (policy?.recovery_allowance) {
        case "account_recovery":
            return {
                status_path: LICENSE_STATUS_PATH,
                action: "account_recovery",
                allowed: [
                    "health",
                    "version",
                    "license_status",
                    "activation",
                    "account_recovery",
                    "repair",
                    "update_for_recovery",
                    "uninstall",
                    "safe_read",
                ],
            };
        case "customer_data_export":
            return {
                status_path: LICENSE_STATUS_PATH,
                action: "customer_data_export",
                allowed: ["export", "license_status", "account_recovery", "uninstall", "safe_read"],
            };
        case "stable_security_update":
            return {
                status_path: LICENSE_STATUS_PATH,
                action: "stable_security_update",
                allowed: ["update_for_recovery", "repair", "rollback", "uninstall", "safe_read"],
            };
        case "repair_rollback":
            return {
                status_path: LICENSE_STATUS_PATH,
                action: "repair_rollback",
                allowed: ["repair", "rollback", "uninstall", "safe_read"],
            };
        case "uninstall":
            return {
                status_path: LICENSE_STATUS_PATH,
                action: "uninstall",
                allowed: ["uninstall", "safe_read"],
            };
        case "read_projection":
            return {
                status_path: LICENSE_STATUS_PATH,
                action: "read_projection",
                allowed: ["safe_read", "export", "license_status", "diagnostics"],
            };
        default:
            return {
                status_path: LICENSE_STATUS_PATH,
                action: "recovery_only",
                allowed: [
                    "health",
                    "version",
                    "license_status",
                    "export",
                    "diagnostics",
                    "repair",
                    "update_for_recovery",
                    "uninstall",
                    "safe_read",
                ],
            };
    }
}
/**
 * Project a daemon entitlement denial into the canonical stable machine JSON.
 * Commercial fields (operation class, family, treatment, feature, bucket,
 * recovery allowance) always come from the canonical contract — never from a
 * caller-controlled payload. Only the error code/state and recovery list may
 * be refined from the daemon's authoritative denial.
 */
export function projectEntitlementDecision(toolName, daemonBlocked) {
    const policy = resolveOperationPolicyForTool(toolName);
    const daemon = daemonBlocked && typeof daemonBlocked === "object" ? daemonBlocked : {};
    const daemonError = daemon.error && typeof daemon.error === "object" ? daemon.error : {};
    const daemonRecovery = daemonError.recovery && typeof daemonError.recovery === "object"
        ? daemonError.recovery
        : null;
    const recovery = daemonRecovery && typeof daemonRecovery.status_path === "string"
        ? {
            status_path: String(daemonRecovery.status_path),
            action: String(daemonRecovery.action || "recovery_only"),
            allowed: Array.isArray(daemonRecovery.allowed) ? daemonRecovery.allowed.map(String) : ["safe_read"],
        }
        : recoveryActionsFor(policy);
    const decision = {
        schema: ENTITLEMENT_DECISION_SCHEMA,
        tool: toolName,
        decision: "blocked",
        ok: false,
        status: "blocked",
        failure_class: "entitlement_blocked",
        operation_class: policy?.operation_class ?? "unknown",
        capability_family: policy?.capability_family ?? "unknown",
        commercial_treatment: policy?.commercial_treatment ?? "unknown",
        required_feature: policy?.required_feature ?? null,
        limit_bucket: policy?.limit_bucket ?? null,
        recovery_allowance: policy?.recovery_allowance ?? "none",
        recovery,
        licensing_grants_capability_only: true,
        operator_authority_granted: false,
        cognitive_authority_granted: false,
        approval_inferred: false,
        discovery_visibility_granted: false,
    };
    if (typeof daemonError.state === "string")
        decision.authority_state = daemonError.state;
    return decision;
}
/**
 * Fail-closed preflight resolved BEFORE any side effect. Recovery and read
 * families remain usable at the tool layer (the daemon still enforces its own
 * security); value-producing mutations require usable authority. Unknown
 * tools resolve to blocked: discovery and visibility never grant entitlement.
 */
export function preflightAuthority(toolName, posture) {
    const policy = resolveOperationPolicyForTool(toolName);
    if (!policy) {
        return {
            decision: "blocked",
            reason: "unknown_tool_has_no_operation_policy",
            entitlement: projectEntitlementDecision(toolName),
        };
    }
    if (policy.capability_family === "account_recovery") {
        return { decision: "allow", reason: "account_recovery_is_always_available" };
    }
    if (policy.operation_class === "read" ||
        policy.operation_class === "internal_maintenance" ||
        policy.capability_family === "customer_data_export") {
        return { decision: "allow", reason: "read_recovery_allowance" };
    }
    if (VALUE_MUTATION_FAMILIES.has(policy.capability_family)) {
        if (posture === "usable")
            return { decision: "allow", reason: "usable_authority" };
        return {
            decision: "blocked",
            reason: `authority_posture_${posture}`,
            entitlement: projectEntitlementDecision(toolName),
        };
    }
    return { decision: "allow", reason: `family_${policy.capability_family}_advisory` };
}
/** Canonical boundary statement (Spec 152F §5): capability only, never
 * operator/cognitive authority, role permission, or mutation confirmation. */
export function authorityBoundaryStatement() {
    return {
        schema: "focusa.license_authority_boundary.v1",
        licensing_grants_capability_only: true,
        operator_authority_granted: false,
        cognitive_authority_granted: false,
        workstream_authority_granted: false,
        trajectory_authority_granted: false,
        workpoint_authority_granted: false,
        role_permission_granted: false,
        mutation_confirmation_granted: false,
        operator_confirmation_inferred: false,
    };
}
/** Discovery/visibility is advisory routing metadata and never grants
 * entitlement (Spec 152F §7 Pi/agent tool row). */
export function discoveryGrantsNothing() {
    return {
        schema: "focusa.tool_discovery_policy.v1",
        discovery_grants_entitlement: false,
        visibility: "advisory",
    };
}
// ── Spec 172 canonical presenter projection (Spec 172 §2.6, §4.1, §11, §21) ──
//
// The Pi/agent surface renders the same canonical posture, product, License
// Type, capability family, denial, retained access, and upgrade/recovery
// action the CLI and agent descriptors inherit. The adapter never accepts a
// caller-selected product, price, License Type, family, feature, limit, node,
// or commercial right, never infers a grant from the installed client,
// pairing, tool discovery, or email, and only ever mirrors what the daemon
// authority already decided.
export const SPEC172_PRESENTER_SCHEMA = "focusa.spec172.presenter_projection.v1";
/** Canonical Spec 172 postures (Spec 172 §4.1). `verified_no_license` is the
 * explicit authority-issued limited-access posture; presenters never
 * synthesize it from a paid-lease snapshot. */
export const SPEC172_POSTURES = [
    "unverified",
    "verified_no_license",
    "active_paid_operator",
    "offline_grace",
    "refunded_or_revoked",
    "expired",
    "missing_or_corrupt",
];
/** Canonical License Type codes and the composite Bundle SKU (Spec 172 §4.1).
 * Presenters render only frozen codes for the surface's own product; they
 * never select, price, or invent a License Type. */
export const SPEC172_LICENSE_TYPE_CODES = [
    "focusa_operator_lifetime_v1",
    "uiai_operator_lifetime_v1",
    "focusa_uiai_operator_bundle_lifetime_v1",
];
/** Stable error vocabulary (Spec 172 §21). Denials use only these codes. */
export const SPEC172_STABLE_ERRORS = [
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
/** Frozen retained-access set (Spec 172 §5.3/§17, Spec 152F P6): navigation,
 * status, account, read, export, recovery, repair, update, and uninstall stay
 * available regardless of commercial state. Byte-identical across CLI, Pi,
 * and agent presenters. */
export const SPEC172_RETAINED_ACCESS = [
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
/** Stable upgrade actions a denial may recommend (presentation vocabulary
 * only; the action never grants or prices anything). */
export const SPEC172_UPGRADE_ACTIONS = [
    "none_required",
    "verify_email_or_manage_entitlement",
    "review_offer_or_manage_entitlement",
    "purchase_or_manage_entitlement",
];
export const SPEC172_RECOVERY_ACTION = "recovery, export, repair, and uninstall remain available when execution is locked";
/** The Focusa Pi extension is the Focusa presenter surface. The product is
 * surface identity, never caller-selected and never a grant source. */
export const SPEC172_SURFACE_PRODUCT = "focusa";
/** Map the daemon authority posture to the canonical Spec 172 posture.
 * Unknown/missing authority fails closed as `missing_or_corrupt`. */
export function spec172PostureForAuthority(posture) {
    switch (posture) {
        case "usable":
            return "active_paid_operator";
        case "recovery_only":
        case "revoked":
            return "refunded_or_revoked";
        case "unverified":
            return "unverified";
        case "expired":
            return "expired";
        case "unknown":
            return "missing_or_corrupt";
    }
}
/** Stable Spec 172 denial + upgrade action for one preflight outcome. The
 * denial is always one of the frozen Spec 172 §21 stable errors. */
export function spec172DenialAndUpgrade(toolName, preflight, canonicalPosture) {
    if (preflight.decision === "allow") {
        return { denial: null, upgrade_action: "none_required" };
    }
    if (!resolveOperationPolicyForTool(toolName)) {
        return {
            denial: "ENTITLEMENT_POLICY_UNKNOWN",
            upgrade_action: "review_offer_or_manage_entitlement",
        };
    }
    switch (canonicalPosture) {
        case "unverified":
            return {
                denial: "EMAIL_VERIFICATION_REQUIRED",
                upgrade_action: "verify_email_or_manage_entitlement",
            };
        case "refunded_or_revoked":
            return {
                denial: "RECOVERY_ONLY",
                upgrade_action: "review_offer_or_manage_entitlement",
            };
        case "expired":
            return {
                denial: "LICENSE_TYPE_REQUIRED",
                upgrade_action: "purchase_or_manage_entitlement",
            };
        case "missing_or_corrupt":
            return {
                denial: "ENTITLEMENT_POLICY_UNKNOWN",
                upgrade_action: "review_offer_or_manage_entitlement",
            };
        default:
            return {
                denial: "CAPABILITY_FAMILY_NOT_INCLUDED",
                upgrade_action: "review_offer_or_manage_entitlement",
            };
    }
}
/** Project the Spec 172 canonical presenter envelope for one Pi tool. The
 * family comes from the canonical tool contract; the posture comes from the
 * daemon authority; product/license type are frozen surface vocabulary. No
 * caller-controlled commercial field is accepted. */
export function projectSpec172PresenterV1(toolName, posture) {
    const policy = resolveOperationPolicyForTool(toolName);
    const preflight = preflightAuthority(toolName, posture);
    const canonicalPosture = spec172PostureForAuthority(posture);
    const family = policy?.capability_family ?? "unknown";
    const licenseType = canonicalPosture === "active_paid_operator" || canonicalPosture === "offline_grace"
        ? SPEC172_LICENSE_TYPE_CODES[0]
        : "none";
    const { denial, upgrade_action } = spec172DenialAndUpgrade(toolName, preflight, canonicalPosture);
    return {
        schema: SPEC172_PRESENTER_SCHEMA,
        posture: canonicalPosture,
        product: SPEC172_SURFACE_PRODUCT,
        license_type: licenseType,
        family,
        denial,
        retained_access: SPEC172_RETAINED_ACCESS,
        upgrade_action,
        recovery_action: SPEC172_RECOVERY_ACTION,
        grant_inferred_from_surface: false,
    };
}
