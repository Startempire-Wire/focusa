import { truncateToWidth, visibleWidth } from "@earendil-works/pi-tui";

export interface WorkRailWidgetSnapshot {
  providerItemId: string;
  workpointId: string;
  proofCount: number;
  nextAction: string;
  status: string;
  badges?: string[];
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
    providerItemId: String(workpoint?.work_item_id || packet?.work_item_id || "no-bead"),
    workpointId: String(workpoint?.workpoint_id || packet?.workpoint_id || "no-workpoint"),
    proofCount: evidence.length,
    nextAction: String(workpoint?.next_slice || packet?.next_slice || "checkpoint next action"),
    status: packet ? String(workpoint?.status || packet?.status || "active") : "unbound",
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
  if (width < 48) {
    return fitToWidth(
      [
        `${palette.accent(active)} ${item} · ${proof} ${snapshot.proofCount} · ${next} ${bounded(nextAction, 18)}`,
      ],
      width
    );
  }
  const lines = [
    `${palette.accent(active)} ${palette.good(item)}  ${palette.dim(`[${snapshot.status}]`)}  WP ${workpoint}`,
    `${palette.dim(`${proof} proof ${snapshot.proofCount}`)}  ${next} ${nextAction}`,
  ];
  if (width >= 76 && snapshot.badges?.length) lines.push(palette.dim(snapshot.badges.join(" · ")));
  return fitToWidth(lines, width);
}
