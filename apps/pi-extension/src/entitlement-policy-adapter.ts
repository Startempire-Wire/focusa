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

import {
  FOCUSA_TOOL_CONTRACTS,
  type FocusaOperationPolicy,
  type FocusaOperationClass,
  type FocusaCapabilityFamily,
  type FocusaCommercialTreatment,
} from "./tool-contracts.js";

export const ENTITLEMENT_DECISION_SCHEMA = "focusa.entitlement_decision.v1";
export const LICENSE_STATUS_PATH = "/v1/license/status";

export type AuthorityPosture = "usable" | "recovery_only" | "unverified" | "expired" | "revoked" | "unknown";

export interface EntitlementRecoveryV1 {
  status_path: string;
  action: string;
  allowed: string[];
}

/** Stable machine JSON projected when a canonical decision blocks a tool. */
export interface EntitlementDecisionV1 {
  schema: string;
  tool: string;
  decision: "blocked";
  ok: false;
  status: "blocked";
  failure_class: "entitlement_blocked";
  operation_class: FocusaOperationClass | "unknown";
  capability_family: FocusaCapabilityFamily | "unknown";
  commercial_treatment: FocusaCommercialTreatment | "unknown";
  required_feature: string | null;
  limit_bucket: string | null;
  recovery_allowance: string;
  recovery: EntitlementRecoveryV1;
  authority_state?: string;
  /** Licensing grants capability only. Always true by construction. */
  licensing_grants_capability_only: true;
  /** An entitlement decision can never mint operator/cognitive authority. */
  operator_authority_granted: false;
  cognitive_authority_granted: false;
  /** Operator permission/confirmation is preserved independently. */
  approval_inferred: false;
  /** Tool visibility never grants entitlement. */
  discovery_visibility_granted: false;
}

const VALUE_MUTATION_FAMILIES = new Set<FocusaCapabilityFamily>([
  "base_focusa",
  "automation",
  "team_remote",
  "release_proof",
  "premium_updates",
]);

/** Resolve the canonical operation policy for a named tool; fail-closed null. */
export function resolveOperationPolicyForTool(toolName: string): FocusaOperationPolicy | null {
  if (!toolName || typeof toolName !== "string") return null;
  const contract = FOCUSA_TOOL_CONTRACTS.find((entry) => entry.name === toolName);
  return contract?.operation_policy ?? null;
}

/** Canonical recovery actions derived from the policy recovery allowance. */
export function recoveryActionsFor(policy: FocusaOperationPolicy | null): EntitlementRecoveryV1 {
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
export function projectEntitlementDecision(
  toolName: string,
  daemonBlocked?: Record<string, unknown> | null
): EntitlementDecisionV1 {
  const policy = resolveOperationPolicyForTool(toolName);
  const daemon = daemonBlocked && typeof daemonBlocked === "object" ? daemonBlocked : {};
  const daemonError =
    daemon.error && typeof daemon.error === "object" ? (daemon.error as Record<string, unknown>) : {};
  const daemonRecovery =
    daemonError.recovery && typeof daemonError.recovery === "object"
      ? (daemonError.recovery as Record<string, unknown>)
      : null;
  const recovery =
    daemonRecovery && typeof daemonRecovery.status_path === "string"
      ? {
          status_path: String(daemonRecovery.status_path),
          action: String(daemonRecovery.action || "recovery_only"),
          allowed: Array.isArray(daemonRecovery.allowed) ? daemonRecovery.allowed.map(String) : ["safe_read"],
        }
      : recoveryActionsFor(policy);
  const decision: EntitlementDecisionV1 = {
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
  if (typeof daemonError.state === "string") decision.authority_state = daemonError.state;
  return decision;
}

export interface PreflightResult {
  decision: "allow" | "blocked";
  reason: string;
  entitlement?: EntitlementDecisionV1;
}

/**
 * Fail-closed preflight resolved BEFORE any side effect. Recovery and read
 * families remain usable at the tool layer (the daemon still enforces its own
 * security); value-producing mutations require usable authority. Unknown
 * tools resolve to blocked: discovery and visibility never grant entitlement.
 */
export function preflightAuthority(toolName: string, posture: AuthorityPosture): PreflightResult {
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
  if (
    policy.operation_class === "read" ||
    policy.operation_class === "internal_maintenance" ||
    policy.capability_family === "customer_data_export"
  ) {
    return { decision: "allow", reason: "read_recovery_allowance" };
  }
  if (VALUE_MUTATION_FAMILIES.has(policy.capability_family)) {
    if (posture === "usable") return { decision: "allow", reason: "usable_authority" };
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
export function authorityBoundaryStatement(): Record<string, unknown> {
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
export function discoveryGrantsNothing(): {
  schema: string;
  discovery_grants_entitlement: false;
  visibility: "advisory";
} {
  return {
    schema: "focusa.tool_discovery_policy.v1",
    discovery_grants_entitlement: false,
    visibility: "advisory",
  };
}
