function stableKey(parts) {
    return parts.map((part) => String(part ?? "unknown")).join(":");
}
/** Select a route from observable state only; names never imply capabilities. */
export function selectCompactionPolicy(telemetry, capabilities) {
    const percent = telemetry.percent;
    let route = "no_op";
    let executionOwner = "none";
    let reason = "usage_unknown";
    if (percent === null) {
        // Unknown usage never authorizes destructive context reduction.
    }
    else if (percent < 66) {
        reason = "below_threshold";
    }
    else if (percent < 70) {
        route = "checkpoint";
        executionOwner = "focusa";
        reason = "checkpoint_boundary";
    }
    else if (capabilities.nativeCompaction === "supported") {
        route = "native_compact";
        executionOwner = "pi";
        reason = "native_pressure";
    }
    else if (percent < 85) {
        route = "checkpoint";
        executionOwner = "focusa";
        reason = "native_compaction_unavailable";
    }
    else {
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
            "legacy_current_v1",
            capabilities.nativeCompaction,
        ]),
    };
}
/** Replace a quarantined policy with its deterministic safe rollback route. */
export function applyCompactionPolicyQuarantine(selection, quarantinedPolicyKeys, rollbackRoute) {
    if (!quarantinedPolicyKeys.includes(selection.deterministicKey))
        return selection;
    const route = isCompactionPolicyRoute(rollbackRoute)
        ? rollbackRoute
        : rollbackRouteForSelection(selection.route);
    return {
        ...selection,
        route,
        executionOwner: route === "no_op"
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
function isCompactionPolicyRoute(value) {
    return ["no_op", "curate_context", "checkpoint", "summarize", "native_compact", "rollover"].includes(value ?? "");
}
function rollbackRouteForSelection(route) {
    if (route === "native_compact" || route === "summarize")
        return "checkpoint";
    if (route === "curate_context" || route === "rollover")
        return "no_op";
    return route;
}
