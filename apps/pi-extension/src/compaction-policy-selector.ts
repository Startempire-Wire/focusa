import type { ContextPressureTelemetry } from "./context-pressure-telemetry.js";
import type { ProviderCompactionCapabilities } from "./provider-compaction-capabilities.js";

export type CompactionPolicyRoute =
  "no_op" | "curate_context" | "checkpoint" | "summarize" | "native_compact" | "rollover";

export interface CompactionPolicySelection {
  schema: "focusa.compaction_policy_selection.v1";
  policyVersion: "1";
  route: CompactionPolicyRoute;
  executionOwner: "none" | "focusa" | "pi" | "operator";
  reason:
    | "usage_unknown"
    | "below_threshold"
    | "tool_history_dominant"
    | "checkpoint_boundary"
    | "summary_boundary"
    | "native_pressure"
    | "native_compaction_unavailable"
    | "policy_quarantined";
  percent: number | null;
  deterministicKey: string;
}

function stableKey(parts: Array<string | number | null>): string {
  return parts.map((part) => String(part ?? "unknown")).join(":");
}

/** Select a route from observable state only; names never imply capabilities. */
export function selectCompactionPolicy(
  telemetry: ContextPressureTelemetry,
  capabilities: ProviderCompactionCapabilities
): CompactionPolicySelection {
  const percent = telemetry.percent;
  const toolRatio =
    telemetry.messageEntryCount > 0 ? telemetry.toolResultCount / telemetry.messageEntryCount : 0;
  let route: CompactionPolicyRoute = "no_op";
  let executionOwner: CompactionPolicySelection["executionOwner"] = "none";
  let reason: CompactionPolicySelection["reason"] = "usage_unknown";
  if (percent === null) {
    // Unknown usage never authorizes destructive context reduction.
  } else if (percent < 70) {
    reason = "below_threshold";
  } else if (toolRatio >= 0.35 && percent < 85) {
    route = "curate_context";
    executionOwner = "focusa";
    reason = "tool_history_dominant";
  } else if (percent < 82) {
    route = "checkpoint";
    executionOwner = "focusa";
    reason = "checkpoint_boundary";
  } else if (percent < 92) {
    route = "summarize";
    executionOwner = "pi";
    reason = "summary_boundary";
  } else if (capabilities.nativeCompaction === "supported") {
    route = "native_compact";
    executionOwner = "pi";
    reason = "native_pressure";
  } else {
    route = "rollover";
    executionOwner = "operator";
    reason = "native_compaction_unavailable";
  }
  return {
    schema: "focusa.compaction_policy_selection.v1",
    policyVersion: "1",
    route,
    executionOwner,
    reason,
    percent,
    deterministicKey: stableKey([
      "v1",
      percent === null ? null : percent.toFixed(3),
      telemetry.branchEntryCount,
      telemetry.toolResultCount,
      capabilities.nativeCompaction,
    ]),
  };
}

/** Replace a quarantined policy with its deterministic safe rollback route. */
export function applyCompactionPolicyQuarantine(
  selection: CompactionPolicySelection,
  quarantinedPolicyKeys: readonly string[],
  rollbackRoute: string | null
): CompactionPolicySelection {
  if (!quarantinedPolicyKeys.includes(selection.deterministicKey)) return selection;
  const route = isCompactionPolicyRoute(rollbackRoute)
    ? rollbackRoute
    : rollbackRouteForSelection(selection.route);
  return {
    ...selection,
    route,
    executionOwner:
      route === "no_op"
        ? "none"
        : route === "rollover"
          ? "operator"
          : route === "native_compact" || route === "summarize"
            ? "pi"
            : "focusa",
    reason: "policy_quarantined",
    deterministicKey: stableKey(["rollback", selection.deterministicKey, route]),
  };
}

function isCompactionPolicyRoute(value: string | null): value is CompactionPolicyRoute {
  return ["no_op", "curate_context", "checkpoint", "summarize", "native_compact", "rollover"].includes(
    value ?? ""
  );
}

function rollbackRouteForSelection(route: CompactionPolicyRoute): CompactionPolicyRoute {
  if (route === "native_compact" || route === "summarize") return "checkpoint";
  if (route === "curate_context" || route === "rollover") return "no_op";
  return route;
}
