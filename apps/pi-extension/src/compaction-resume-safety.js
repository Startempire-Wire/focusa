function exact(value) {
    return String(value || "")
        .trim()
        .replace(/[\\/]+$/, "");
}
/**
 * A hidden continuation may carry action context only when the daemon packet
 * exactly matches the live project/continuity and both authority layers agree.
 * Blocked/degraded packets are diagnostics, never resumable missions.
 */
export function canInjectCompactionMission(packet, currentScope) {
    if (!packet || typeof packet !== "object")
        return false;
    const candidate = packet;
    const scope = candidate.scope && typeof candidate.scope === "object" ? candidate.scope : {};
    const trajectory = candidate.trajectory && typeof candidate.trajectory === "object" ? candidate.trajectory : {};
    const workpoint = candidate.workpoint && typeof candidate.workpoint === "object" ? candidate.workpoint : {};
    const expectedRoot = exact(currentScope?.root_scope?.root_path);
    const expectedContinuity = exact(currentScope?.continuity_id);
    return (candidate.schema_version === "focusa.compaction_mission_packet.v1" &&
        candidate.status === "verified" &&
        scope.scope_status === "verified" &&
        exact(scope.project_root) === expectedRoot &&
        exact(scope.continuity_id) === expectedContinuity &&
        Boolean(expectedRoot) &&
        Boolean(expectedContinuity) &&
        trajectory.action_authority_from_trajectory === true &&
        workpoint.status === "ready" &&
        workpoint.action_authority === true);
}
export function safeCompactionRecoveryContext() {
    return [
        "Focusa post-compaction continuation authority is not verified.",
        "Do not rely on a prior mission or next action.",
        "Re-verify the current project scope and resume a canonical Workpoint before continuing.",
    ].join(" ");
}
