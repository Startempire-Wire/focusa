import {
  currentProjectBindingDecision,
  getActiveWorkpointPacket,
  getAttachmentRuntime,
  getContinuityId,
  getLastTrajectoryClarity,
  getSessionCwd,
  normalizeProjectRoot,
} from "./state.js";

export type ScopedRefreshSource = "tool" | "sse" | "poll" | "session" | "rebind";

export interface ScopedStateChangeReceiptV1 {
  schema: "focusa.scoped_state_change_receipt.v1";
  receipt_id: string;
  source: ScopedRefreshSource;
  mutation_kind: string;
  project_root: string;
  continuity_id: string;
  status: "accepted" | "observed" | "degraded";
  evidence_revision?: string;
  effective_at: string;
}

export interface TruthfulScopedSurfaceSnapshotV1 {
  schema: "focusa.truthful_scoped_surface_snapshot.v1";
  project: "bound" | "recovering" | "quarantined" | "unbound";
  selected_scope: string;
  startup_cwd: string;
  trajectory: "absent" | "provisional" | "persisted";
  bead: "absent" | "present";
  workpoint: "absent" | "present" | "blocked";
  proof: "missing" | "linked" | "verified";
  proof_count: number;
  stale_age_ms: number;
  last_refresh_status: string;
  last_refresh_at?: string;
}

type ScopedRefreshListener = (receipt: ScopedStateChangeReceiptV1) => void;

const listeners = new Set<ScopedRefreshListener>();
const latestByScope = new Map<string, ScopedStateChangeReceiptV1>();

function bounded(value: unknown, max = 256): string {
  return String(value || "")
    .trim()
    .slice(0, max);
}

function scopeKey(projectRoot: string, continuityId: string): string {
  return `${normalizeProjectRoot(projectRoot)}|${bounded(continuityId)}`;
}

function receiptId(input: Omit<ScopedStateChangeReceiptV1, "schema" | "receipt_id">): string {
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

export function publishScopedStateChange(
  input: Omit<ScopedStateChangeReceiptV1, "schema" | "receipt_id">
): ScopedStateChangeReceiptV1 | null {
  const projectRoot = normalizeProjectRoot(input.project_root);
  const continuityId = bounded(input.continuity_id);
  if (!projectRoot || !continuityId) return null;
  const receipt: ScopedStateChangeReceiptV1 = {
    ...input,
    schema: "focusa.scoped_state_change_receipt.v1",
    receipt_id: receiptId({ ...input, project_root: projectRoot, continuity_id: continuityId }),
    project_root: projectRoot,
    continuity_id: continuityId,
  };
  latestByScope.set(scopeKey(projectRoot, continuityId), receipt);
  try {
    getAttachmentRuntime().pi?.appendEntry("focusa-scoped-state-change", receipt);
  } catch {
    // The durable mutation already succeeded. Surface refresh remains useful,
    // but a session-ledger write must never fabricate a failed mutation.
  }
  queueMicrotask(() => {
    for (const listener of listeners) listener(receipt);
  });
  return receipt;
}

export function rehydrateScopedStateChanges(entries: unknown[]): number {
  let accepted = 0;
  for (const candidate of entries) {
    if (!candidate || typeof candidate !== "object") continue;
    const entry = candidate as Record<string, any>;
    if (entry.type !== "custom" || entry.customType !== "focusa-scoped-state-change") continue;
    const receipt = entry.data as ScopedStateChangeReceiptV1 | undefined;
    if (receipt?.schema !== "focusa.scoped_state_change_receipt.v1") continue;
    const projectRoot = normalizeProjectRoot(receipt.project_root);
    const continuityId = bounded(receipt.continuity_id);
    if (!projectRoot || !continuityId || !bounded(receipt.receipt_id)) continue;
    latestByScope.set(scopeKey(projectRoot, continuityId), {
      ...receipt,
      project_root: projectRoot,
      continuity_id: continuityId,
    });
    accepted += 1;
  }
  return accepted;
}

export function subscribeScopedStateChanges(listener: ScopedRefreshListener): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

export function currentScopedProjectRoot(): string {
  const binding = currentProjectBindingDecision();
  const selected = normalizeProjectRoot(binding?.selected_project_root || "");
  return binding?.state === "BOUND" && selected ? selected : normalizeProjectRoot(getSessionCwd());
}

export function latestScopedStateChange(
  projectRoot = currentScopedProjectRoot(),
  continuityId = getContinuityId()
): ScopedStateChangeReceiptV1 | null {
  return latestByScope.get(scopeKey(projectRoot, continuityId)) || null;
}

export function scopedReceiptMatchesCurrentScope(receipt: ScopedStateChangeReceiptV1): boolean {
  return (
    normalizeProjectRoot(receipt.project_root) === currentScopedProjectRoot() &&
    receipt.continuity_id === getContinuityId()
  );
}

export function buildTruthfulScopedSurfaceSnapshot(
  startupCwd: string,
  now = Date.now()
): TruthfulScopedSurfaceSnapshotV1 {
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
  const verifiedProof = evidence.filter(
    (item: any) => item?.status === "verified" || item?.verified === true || item?.result
  ).length;
  const bindingState = binding?.state || (projectRoot ? "BOUND" : "RECOVERING");
  const trajectoryMatches =
    !trajectory.project_root || normalizeProjectRoot(trajectory.project_root) === projectRoot;
  const hasTrajectory =
    trajectoryMatches &&
    Boolean(
      trajectory.trajectory_id ||
      trajectory.long_term_goal ||
      trajectory.short_term_goal ||
      trajectory.desired_end_state
    );
  const trajectoryPersisted = hasTrajectory && trajectory.canonical === true && trajectory.degraded !== true;
  const workpointPresent = Boolean(workpoint?.workpoint_id || workpoint?.id);
  const workpointBlocked = Boolean(
    workpoint?.status === "blocked" || (Array.isArray(workpoint?.blockers) && workpoint.blockers.length > 0)
  );
  const lastRefreshMs = receipt ? Date.parse(receipt.effective_at) : 0;

  return {
    schema: "focusa.truthful_scoped_surface_snapshot.v1",
    project:
      bindingState === "BOUND"
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
