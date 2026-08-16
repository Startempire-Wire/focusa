import {
  MAX_MISSION_CANVAS_ROWS,
  type WorkSurfaceProjection,
  type WorksetSurfaceSummary,
} from "./mission-canvas-model.js";

export interface MissionCanvasSessionInventoryRow {
  kind: string;
  projectRoot: string;
  continuityId: string;
  instanceId: string;
  sessionId: string;
  attachmentId: string;
  health: string;
  lifecycle: string;
  approvals: number;
  conflicts: number;
  writerLease: string;
  browserIsolation: string;
  origin: string;
}

function value(...items: unknown[]): string {
  for (const item of items) {
    const clean = String(item ?? "").trim();
    if (clean) return clean;
  }
  return "unknown";
}

function count(item: unknown): number {
  const parsed = Number(item);
  return Number.isFinite(parsed) && parsed >= 0 ? Math.floor(parsed) : 0;
}

export function projectSessionInventory(
  discoveredPayload: any,
  workSurfaces: WorkSurfaceProjection[],
  worksets: WorksetSurfaceSummary[],
  silentPayload: any
): MissionCanvasSessionInventoryRow[] {
  const discovered = Array.isArray(discoveredPayload?.sessions) ? discoveredPayload.sessions : [];
  const silent = Array.isArray(silentPayload?.sessions)
    ? silentPayload.sessions
    : Array.isArray(silentPayload?.items)
      ? silentPayload.items
      : [];
  const rows: MissionCanvasSessionInventoryRow[] = [];
  for (const session of discovered) {
    rows.push({
      kind: value(session?.agent, "pi"),
      projectRoot: value(session?.project_root),
      continuityId: value(session?.continuity_id),
      instanceId: "filesystem-discovery",
      sessionId: value(session?.session_id),
      attachmentId: "unbound",
      health: "discovered",
      lifecycle: session?.last_activity ? "observed" : "unknown",
      approvals: 0,
      conflicts: 0,
      writerLease: "unknown",
      browserIsolation: "not-applicable",
      origin: value(session?.session_path),
    });
  }
  for (const session of silent) {
    rows.push({
      kind: "silent_session",
      projectRoot: value(session?.project_root),
      continuityId: value(session?.continuity_id),
      instanceId: value(session?.instance_id, "silent-daemon"),
      sessionId: value(session?.session_id, session?.id),
      attachmentId: value(session?.attachment_id, "unbound"),
      health: value(session?.health, session?.status),
      lifecycle: value(session?.lifecycle_state, session?.status),
      approvals: count(session?.pending_approval_count),
      conflicts: count(session?.conflict_count),
      writerLease: value(session?.writer_lease_ref),
      browserIsolation: "not-applicable",
      origin: value(session?.run_id, session?.generation),
    });
  }
  for (const workset of worksets) {
    rows.push({
      kind: "workset",
      projectRoot: "workset-ledger",
      continuityId: value(workset?.worksetId),
      instanceId: "workset-ledger",
      sessionId: value(workset?.worksetId),
      attachmentId: value(workset?.worksetId),
      health: workset?.settled ? "settled" : "in_progress",
      lifecycle: workset?.settled ? "settled" : "active",
      approvals: 0,
      conflicts: 0,
      writerLease: "ledger-owned",
      browserIsolation: "not-applicable",
      origin: `workset:${workset?.worksetId} rev ${workset?.revision ?? "?"} (${workset?.requirementCount ?? 0} req)`,
    });
  }
  for (const surface of workSurfaces) {
    if (
      !surface.sessionId ||
      rows.some((row) => row.sessionId === surface.sessionId && row.attachmentId === surface.attachmentId)
    ) {
      continue;
    }
    rows.push({
      kind: surface.kind,
      projectRoot: value(surface.projectRoot),
      continuityId: value(surface.continuityId),
      instanceId: value(surface.instanceId),
      sessionId: value(surface.sessionId),
      attachmentId: value(surface.attachmentId),
      health: value(surface.health),
      lifecycle: value(surface.lifecycleState),
      approvals: surface.pendingApprovalCount,
      conflicts: surface.conflictCount,
      writerLease: value(surface.writerLeaseRef),
      browserIsolation: value(surface.browserIsolationClass),
      origin: surface.workSurfaceId,
    });
  }
  const kindPriority = (kind: string) =>
    kind === "silent_session"
      ? 0
      : kind === "uiai_browser"
        ? 1
        : kind === "pi_session" || kind === "pi"
          ? 2
          : 1;
  return rows
    .sort(
      (left, right) =>
        kindPriority(left.kind) - kindPriority(right.kind) ||
        [left.projectRoot, left.continuityId, left.kind, left.sessionId, left.attachmentId]
          .join("\u0000")
          .localeCompare(
            [right.projectRoot, right.continuityId, right.kind, right.sessionId, right.attachmentId].join(
              "\u0000"
            )
          )
    )
    .slice(0, MAX_MISSION_CANVAS_ROWS);
}

export function sessionInventoryLabel(row: MissionCanvasSessionInventoryRow): string {
  return `${row.kind} · ${row.sessionId} · ${row.lifecycle}/${row.health} · ${row.projectRoot} · ${row.continuityId} · attachment ${row.attachmentId} · ${row.approvals} approvals · ${row.conflicts} conflicts · ${row.writerLease} · ${row.browserIsolation}`;
}
