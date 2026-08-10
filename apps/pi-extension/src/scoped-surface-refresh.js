import { currentProjectBindingDecision, getActiveWorkpointPacket, getAttachmentRuntime, getContinuityId, getLastTrajectoryClarity, getSessionCwd, normalizeProjectRoot, } from "./state.js";
const listeners = new Set();
const latestByScope = new Map();
function bounded(value, max = 256) {
    return String(value || "")
        .trim()
        .slice(0, max);
}
function scopeKey(projectRoot, continuityId) {
    return `${normalizeProjectRoot(projectRoot)}|${bounded(continuityId)}`;
}
function receiptId(input) {
    const seed = [
        input.source,
        input.mutation_kind,
        normalizeProjectRoot(input.project_root),
        input.continuity_id,
        input.evidence_revision || "",
        input.effective_at,
    ].join("|");
    let hash = 2166136261;
    for (let index = 0; index < seed.length; index += 1) {
        hash ^= seed.charCodeAt(index);
        hash = Math.imul(hash, 16777619);
    }
    return `scoped-refresh:${(hash >>> 0).toString(16).padStart(8, "0")}`;
}
export function publishScopedStateChange(input) {
    const projectRoot = normalizeProjectRoot(input.project_root);
    const continuityId = bounded(input.continuity_id);
    if (!projectRoot || !continuityId)
        return null;
    const receipt = {
        ...input,
        schema: "focusa.scoped_state_change_receipt.v1",
        receipt_id: receiptId({ ...input, project_root: projectRoot, continuity_id: continuityId }),
        project_root: projectRoot,
        continuity_id: continuityId,
    };
    latestByScope.set(scopeKey(projectRoot, continuityId), receipt);
    try {
        getAttachmentRuntime().pi?.appendEntry("focusa-scoped-state-change", receipt);
    }
    catch {
        // The durable mutation already succeeded. Surface refresh remains useful,
        // but a session-ledger write must never fabricate a failed mutation.
    }
    queueMicrotask(() => {
        for (const listener of listeners)
            listener(receipt);
    });
    return receipt;
}
export function rehydrateScopedStateChanges(entries) {
    let accepted = 0;
    for (const candidate of entries) {
        if (!candidate || typeof candidate !== "object")
            continue;
        const entry = candidate;
        if (entry.type !== "custom" || entry.customType !== "focusa-scoped-state-change")
            continue;
        const receipt = entry.data;
        if (receipt?.schema !== "focusa.scoped_state_change_receipt.v1")
            continue;
        const projectRoot = normalizeProjectRoot(receipt.project_root);
        const continuityId = bounded(receipt.continuity_id);
        if (!projectRoot || !continuityId || !bounded(receipt.receipt_id))
            continue;
        latestByScope.set(scopeKey(projectRoot, continuityId), {
            ...receipt,
            project_root: projectRoot,
            continuity_id: continuityId,
        });
        accepted += 1;
    }
    return accepted;
}
export function subscribeScopedStateChanges(listener) {
    listeners.add(listener);
    return () => listeners.delete(listener);
}
export function currentScopedProjectRoot() {
    const binding = currentProjectBindingDecision();
    const selected = normalizeProjectRoot(binding?.selected_project_root || "");
    return binding?.state === "BOUND" && selected ? selected : normalizeProjectRoot(getSessionCwd());
}
export function latestScopedStateChange(projectRoot = currentScopedProjectRoot(), continuityId = getContinuityId()) {
    return latestByScope.get(scopeKey(projectRoot, continuityId)) || null;
}
export function scopedReceiptMatchesCurrentScope(receipt) {
    return (normalizeProjectRoot(receipt.project_root) === currentScopedProjectRoot() &&
        receipt.continuity_id === getContinuityId());
}
export function buildTruthfulScopedSurfaceSnapshot(startupCwd, now = Date.now()) {
    const binding = currentProjectBindingDecision();
    const trajectory = getLastTrajectoryClarity() || {};
    const workpoint = getActiveWorkpointPacket();
    const projectRoot = currentScopedProjectRoot();
    const continuityId = getContinuityId();
    const receipt = latestScopedStateChange(projectRoot, continuityId);
    const evidence = Array.isArray(workpoint?.verification_records)
        ? workpoint.verification_records
        : Array.isArray(workpoint?.evidence_refs)
            ? workpoint.evidence_refs
            : [];
    const verifiedProof = evidence.filter((item) => item?.status === "verified" || item?.verified === true || item?.result).length;
    const bindingState = binding?.state || (projectRoot ? "BOUND" : "RECOVERING");
    const trajectoryMatches = !trajectory.project_root || normalizeProjectRoot(trajectory.project_root) === projectRoot;
    const hasTrajectory = trajectoryMatches &&
        Boolean(trajectory.trajectory_id ||
            trajectory.long_term_goal ||
            trajectory.short_term_goal ||
            trajectory.desired_end_state);
    const trajectoryPersisted = hasTrajectory && trajectory.canonical === true && trajectory.degraded !== true;
    const workpointPresent = Boolean(workpoint?.workpoint_id || workpoint?.id);
    const workpointBlocked = Boolean(workpoint?.status === "blocked" || (Array.isArray(workpoint?.blockers) && workpoint.blockers.length > 0));
    const lastRefreshMs = receipt ? Date.parse(receipt.effective_at) : 0;
    return {
        schema: "focusa.truthful_scoped_surface_snapshot.v1",
        project: bindingState === "BOUND"
            ? "bound"
            : bindingState === "QUARANTINED"
                ? "quarantined"
                : bindingState === "RECOVERING" || bindingState === "VERIFY"
                    ? "recovering"
                    : "unbound",
        selected_scope: projectRoot || "unbound",
        startup_cwd: normalizeProjectRoot(startupCwd) || "unknown",
        trajectory: trajectoryPersisted ? "persisted" : hasTrajectory ? "provisional" : "absent",
        bead: workpoint?.work_item_id ? "present" : "absent",
        workpoint: workpointBlocked ? "blocked" : workpointPresent ? "present" : "absent",
        proof: verifiedProof > 0 ? "verified" : evidence.length > 0 ? "linked" : "missing",
        proof_count: evidence.length,
        stale_age_ms: lastRefreshMs > 0 ? Math.max(0, now - lastRefreshMs) : -1,
        last_refresh_status: receipt?.status || "not_observed",
        last_refresh_at: receipt?.effective_at,
    };
}
