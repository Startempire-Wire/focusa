import { truncateToWidth, visibleWidth } from "@earendil-works/pi-tui";

export interface WorkRailWidgetSnapshot {
  provider: string;
  providerItemId: string;
  title: string;
  providerStatus: string;
  focusaStatus: string;
  workpointId: string;
  projectRoot: string;
  continuityId: string;
  instanceId: string;
  sessionId: string;
  attachmentId: string;
  workSurfaceIds: string[];
  priority: string;
  blockers: string[];
  proofCount: number;
  receiptRef: string;
  nextAction: string;
  status: string;
  mode: "surface-local" | "project-aggregate" | "cross-project-advisory";
  providerCapability:
    | "operational"
    | "read-only"
    | "credentials-missing"
    | "unhealthy"
    | "adapter-unavailable"
    | "schema-only"
    | "approval-required";
  badges?: string[];
}

const WORK_RAIL_STATES: Record<string, string> = {
  ready: "READY",
  active: "ACTIVE",
  verifying: "VERIFYING",
  proof_missing: "PROOF MISSING",
  reconciling: "RECONCILING",
  verified_complete: "VERIFIED COMPLETE",
  provider_closed_focusa_unverified: "PROVIDER CLOSED / FOCUSA UNVERIFIED",
  cancelled: "CANCELLED",
};

function workRailState(value: unknown): string {
  const normalized = String(value || "unbound")
    .trim()
    .toLowerCase()
    .replace(/[-\s]+/g, "_");
  return WORK_RAIL_STATES[normalized] ?? normalized.replace(/_/g, " ").toUpperCase();
}

export interface WorkRailWidgetPalette {
  accent(text: string): string;
  dim(text: string): string;
  good(text: string): string;
}

function bounded(value: string, length: number): string {
  const clean = String(value || "")
    .replace(/\s+/g, " ")
    .trim();
  return clean.length <= length ? clean : `${clean.slice(0, Math.max(1, length - 1))}…`;
}

function fitToWidth(lines: string[], width: number): string[] {
  const maxWidth = Math.max(0, Math.floor(width));
  if (maxWidth === 0) return lines.map(() => "");
  return lines.map((line) => (visibleWidth(line) <= maxWidth ? line : truncateToWidth(line, maxWidth, "…")));
}

export function workRailSnapshotFromPacket(packet: Record<string, any> | null): WorkRailWidgetSnapshot {
  const workpoint = packet?.workpoint && typeof packet.workpoint === "object" ? packet.workpoint : packet;
  const evidence = Array.isArray(workpoint?.verification_records)
    ? workpoint.verification_records
    : Array.isArray(packet?.evidence_refs)
      ? packet.evidence_refs
      : [];
  return {
    provider: String(workpoint?.provider || packet?.provider || "unbound"),
    providerItemId: String(workpoint?.work_item_id || packet?.work_item_id || "no-bead"),
    title: String(workpoint?.title || packet?.title || workpoint?.mission || "No active work item"),
    providerStatus: String(workpoint?.provider_status || packet?.provider_status || "unknown"),
    focusaStatus: String(workpoint?.focusa_status || workpoint?.status || packet?.status || "unbound"),
    workpointId: String(workpoint?.workpoint_id || packet?.workpoint_id || "no-workpoint"),
    projectRoot: String(workpoint?.project_root || packet?.project_root || "unknown"),
    continuityId: String(workpoint?.continuity_id || packet?.continuity_id || "unknown"),
    instanceId: String(workpoint?.instance_id || packet?.instance_id || "unknown"),
    sessionId: String(workpoint?.session_id || packet?.session_id || "unknown"),
    attachmentId: String(workpoint?.attachment_id || packet?.attachment_id || "unknown"),
    workSurfaceIds: Array.isArray(workpoint?.work_surface_ids) ? workpoint.work_surface_ids.map(String) : [],
    priority: String(workpoint?.priority || packet?.priority || "normal"),
    blockers: Array.isArray(workpoint?.blockers) ? workpoint.blockers.map(String) : [],
    proofCount: evidence.length,
    receiptRef: String(workpoint?.receipt_ref || packet?.receipt_ref || "none"),
    nextAction: String(workpoint?.next_slice || packet?.next_slice || "checkpoint next action"),
    status: packet ? String(workpoint?.status || packet?.status || "unbound") : "unbound",
    mode: packet?.cross_project
      ? "cross-project-advisory"
      : packet?.project_aggregate
        ? "project-aggregate"
        : "surface-local",
    providerCapability: String(
      packet?.provider_capability || workpoint?.provider_capability || "adapter-unavailable"
    ) as WorkRailWidgetSnapshot["providerCapability"],
  };
}

export function renderWorkRailWidget(
  snapshot: WorkRailWidgetSnapshot,
  width: number,
  palette: WorkRailWidgetPalette,
  ascii = false
): string[] {
  const active = ascii ? ">" : "▶";
  const proof = ascii ? "proof" : "✓";
  const next = ascii ? "next" : "→";
  const item = bounded(snapshot.providerItemId, width < 48 ? 16 : 32);
  const workpoint = bounded(snapshot.workpointId, 22);
  const nextAction = bounded(snapshot.nextAction, Math.max(18, width - 11));
  const state = workRailState(snapshot.focusaStatus || snapshot.status);
  const mode = snapshot.mode ?? "surface-local";
  const capability = snapshot.providerCapability ?? "adapter-unavailable";
  if (width < 48) {
    return fitToWidth(
      [
        `${palette.accent(active)} ${item} · ${proof} ${snapshot.proofCount} · ${next} ${bounded(nextAction, 18)}`,
      ],
      width
    );
  }
  const lines = [
    `${palette.accent(active)} ${palette.good(item)}  ${palette.dim(`[${state}]`)}  WP ${workpoint}`,
    `${palette.dim(`${proof} proof ${snapshot.proofCount} · ${snapshot.blockers?.length ?? 0} blockers · ${mode} · ${capability}`)}  ${next} ${nextAction}`,
  ];
  if (width >= 76 && snapshot.badges?.length) lines.push(palette.dim(snapshot.badges.join(" · ")));
  return fitToWidth(lines, width);
}
