import { SPEC138_OPERATIONS } from "./generated/spec138-operations.js";

export const MAX_MISSION_CANVAS_ROWS = 200;

export type WorkSurfaceKind =
  | "project_overview"
  | "pi_session"
  | "uiai_browser"
  | "silent_session"
  | "document"
  | "research"
  | "provider_item"
  | "evidence"
  | "custom";

export interface WorkSurfaceProjection {
  workSurfaceId: string;
  displayName: string;
  kind: WorkSurfaceKind;
  projectRoot: string;
  continuityId: string;
  workpointId: string;
  workItemRef: string;
  instanceId: string;
  sessionId: string;
  attachmentId: string;
  role: string;
  rendererId: string;
  pinned: boolean;
  groupId: string;
  splitGroupId: string;
  lifecycleState: string;
  semanticActivity: string;
  health: string;
  unreadEventCount: number;
  pendingApprovalCount: number;
  conflictCount: number;
  blockerCount: number;
  writerLeaseRef: string;
  worktreeRef: string;
  browserIsolationClass: string;
}

const KINDS = new Set<WorkSurfaceKind>([
  "project_overview",
  "pi_session",
  "uiai_browser",
  "silent_session",
  "document",
  "research",
  "provider_item",
  "evidence",
  "custom",
]);

function value(...items: unknown[]): string {
  for (const item of items) {
    const clean = String(item ?? "").trim();
    if (clean) return clean;
  }
  return "";
}

function count(item: unknown): number {
  const parsed = Number(item);
  return Number.isFinite(parsed) && parsed >= 0 ? Math.floor(parsed) : 0;
}

function asRecord(item: unknown): Record<string, any> {
  return item !== null && typeof item === "object" && !Array.isArray(item)
    ? item as Record<string, any>
    : {};
}

function text(item: unknown): string {
  return value(item);
}

export function projectWorkSurfaces(payload: any): WorkSurfaceProjection[] {
  const rows = Array.isArray(payload?.surfaces)
    ? payload.surfaces
    : Array.isArray(payload?.work_surfaces)
      ? payload.work_surfaces
      : [];
  return rows.slice(0, MAX_MISSION_CANVAS_ROWS).flatMap((row: any) => {
    const id = value(row?.work_surface_id, row?.surface_id);
    if (!id) return [];
    const rawKind = value(row?.kind, "custom") as WorkSurfaceKind;
    const kind = KINDS.has(rawKind) ? rawKind : "custom";
    return [
      {
        workSurfaceId: id,
        displayName: value(row?.display_name, row?.presentation?.title, id),
        kind,
        projectRoot: value(row?.scope?.project_root, row?.project_root),
        continuityId: value(row?.scope?.continuity_id, row?.continuity_id),
        workpointId: value(row?.scope?.workpoint_id, row?.workpoint_id),
        workItemRef: value(row?.scope?.work_item_ref, row?.work_item_ref),
        instanceId: value(row?.primary_attachment?.instance_id, row?.instance_id),
        sessionId: value(row?.primary_attachment?.session_id, row?.session_id),
        attachmentId: value(row?.primary_attachment?.attachment_id, row?.attachment_id),
        role: value(row?.primary_attachment?.role, row?.role),
        rendererId: value(row?.presentation?.renderer_id, row?.renderer_id),
        pinned: Boolean(row?.presentation?.pinned ?? row?.pinned),
        groupId: value(row?.presentation?.group_id, row?.group_id),
        splitGroupId: value(row?.presentation?.split_group_id, row?.split_group_id),
        lifecycleState: value(row?.activity?.lifecycle_state, row?.lifecycle_state, "unknown"),
        semanticActivity: value(row?.activity?.semantic_activity, row?.semantic_activity),
        health: value(row?.activity?.health, row?.health, "unknown"),
        unreadEventCount: count(row?.activity?.unread_event_count ?? row?.unread_event_count),
        pendingApprovalCount: count(row?.activity?.pending_approval_count ?? row?.pending_approval_count),
        conflictCount: count(row?.activity?.conflict_count ?? row?.conflict_count),
        blockerCount: count(row?.activity?.blocker_count ?? row?.blocker_count),
        writerLeaseRef: value(row?.isolation?.writer_lease_ref, row?.writer_lease_ref),
        worktreeRef: value(row?.isolation?.worktree_ref, row?.worktree_ref),
        browserIsolationClass: value(
          row?.isolation?.browser_isolation_class,
          row?.browser_isolation_class,
          "not-applicable"
        ),
      },
    ];
  });
}

export function workSurfaceDetail(surface: WorkSurfaceProjection): string[] {
  return [
    `Surface: ${surface.workSurfaceId} · ${surface.kind} · ${surface.lifecycleState} · ${surface.health}`,
    `Scope: ${surface.projectRoot || "unknown project"} · ${surface.continuityId || "unknown continuity"}`,
    `Attachment: ${surface.instanceId || "unknown instance"} · ${surface.sessionId || "unknown session"} · ${surface.attachmentId || "unknown attachment"}`,
    `Work: ${surface.workpointId || "no Workpoint"} · ${surface.workItemRef || "no provider item"}`,
    `Activity: ${surface.semanticActivity || "not reported"} · ${surface.unreadEventCount} unread · ${surface.pendingApprovalCount} approvals`,
    `Isolation: ${surface.writerLeaseRef || "no writer lease"} · ${surface.worktreeRef || "no worktree"} · ${surface.browserIsolationClass}`,
  ];
}

export function workSurfaceLabel(surface: WorkSurfaceProjection): string {
  const markers = [
    surface.pinned ? "pinned" : "",
    surface.unreadEventCount ? `${surface.unreadEventCount} unread` : "",
    surface.pendingApprovalCount ? `${surface.pendingApprovalCount} approvals` : "",
    surface.conflictCount ? `${surface.conflictCount} conflicts` : "",
    surface.splitGroupId ? `split:${surface.splitGroupId}` : "",
    surface.writerLeaseRef ? "writer lease" : "",
    surface.worktreeRef ? `worktree:${surface.worktreeRef}` : "",
    surface.browserIsolationClass !== "not-applicable" ? surface.browserIsolationClass : "",
  ].filter(Boolean);
  return `${surface.displayName} · ${surface.kind} · ${surface.lifecycleState}${markers.length ? ` · ${markers.join(" · ")}` : ""}`;
}

export type SemanticPairTruthState =
  | "schema_only" | "pack_missing" | "migration_required" | "verification_required"
  | "verification_blocked" | "operator_required" | "unsupported_future_definition"
  | "writer_blocked" | "degraded" | "stale" | "conflicted" | "quarantined";

export interface SemanticPairPortfolioItem {
  pair_id: string; title: string; state: SemanticPairTruthState;
  obligations: Array<{ obligation_id: string; statement: string; source_refs: string[]; status: string }>;
  findings: Array<{ finding_id: string; severity: string; verdict: string; summary: string; evidence_refs: string[] }>;
  settlement: { status: string; verdict?: string; receipt_refs: string[] };
  replay: { status: string; generation: number; receipt_refs: string[] };
  recovery: { required: boolean; state?: SemanticPairTruthState; next_operation?: string; reason?: string };
  evidence_refs: string[]; receipt_refs: string[];
}
export interface SemanticPairPortfolio {
  schema: "focusa.semantic_pair.portfolio.v1";
  scope: { project_root: string; continuity_id: string };
  items: SemanticPairPortfolioItem[]; state: SemanticPairTruthState;
  stale: boolean; conflicted: boolean; quarantined: boolean;
}
export interface SemanticPairAction {
  operation_id: string; kind: "read" | "mutation"; available: boolean;
  disabled_reason?: SemanticPairTruthState | "read_only_surface";
}

const TRUTH_STATES = new Set<SemanticPairTruthState>([
  "schema_only", "pack_missing", "migration_required", "verification_required",
  "verification_blocked", "operator_required", "unsupported_future_definition",
  "writer_blocked", "degraded", "stale", "conflicted", "quarantined",
]);

export function normalizeSemanticPairState(value: unknown): SemanticPairTruthState {
  return typeof value === "string" && TRUTH_STATES.has(value as SemanticPairTruthState)
    ? value as SemanticPairTruthState : "schema_only";
}

/** Generated epistemic affordances remain visible; Mission Canvas never grants mutation authority. */
export function spec138MissionCanvasAffordances(canMutate: boolean) {
  return SPEC138_OPERATIONS.map((operation) => ({
    operation_id: operation.operation_id,
    label: operation.label,
    kind: operation.mode === "read" ? "read" as const : "mutation" as const,
    available: operation.mode === "read" || canMutate,
    disabled_reason: operation.mode === "canonical_mutation" && !canMutate
      ? "canonical_daemon_authority_required"
      : undefined,
    client_authority: false as const,
  }));
}

/** Preserve every daemon operation; unsupported Pi mutations remain visible and disabled. */
export function normalizeSemanticPairActions(payload: unknown, canMutate = true): SemanticPairAction[] {
  const root = asRecord(payload);
  const rows = Array.isArray(root.operations) ? root.operations : Array.isArray(root.items) ? root.items : [];
  return rows.map((raw) => {
    const row = asRecord(raw);
    const kind: SemanticPairAction["kind"] = row.kind === "mutation" ? "mutation" : "read";
    const state = normalizeSemanticPairState(row.state ?? root.state);
    const writerBlocked = row.availability === "writer_blocked" || state === "writer_blocked";
    const surfaceBlocked = kind === "mutation" && !canMutate;
    return {
      operation_id: text(row.operation_id), kind,
      available: !writerBlocked && !surfaceBlocked,
      disabled_reason: surfaceBlocked ? "read_only_surface" as const : writerBlocked ? "writer_blocked" as const : undefined,
    };
  }).filter((row) => row.operation_id.length > 0);
}
