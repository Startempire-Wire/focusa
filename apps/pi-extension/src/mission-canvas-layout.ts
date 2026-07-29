import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { focusaFetch, getActiveWorkpointPacket, getContinuityId, getSessionCwd } from "./state.js";

type SplitOrientation = "none" | "horizontal" | "vertical";
type GroupMode = "project" | "workstream" | "session";

interface LocalLayout {
  surfaces: any[];
  activeIndex: number;
  secondaryIndex: number | null;
  split: SplitOrientation;
  group: GroupMode;
  pinned: Set<string>;
  unread: Set<string>;
}

const layouts = new Map<string, LocalLayout>();
let activeLayoutKey = "";

function scope() {
  const packet = getActiveWorkpointPacket();
  const workpoint = packet?.workpoint && typeof packet.workpoint === "object" ? packet.workpoint : packet;
  const projectRoot = getSessionCwd();
  const continuityId = getContinuityId();
  const attachmentId = String(
    workpoint?.attachment_id || packet?.attachment_id || `attachment:canvas:${continuityId}`
  );
  return { projectRoot, continuityId, attachmentId };
}

function key(projectRoot: string, continuityId: string, attachmentId: string) {
  return `${projectRoot}\u0000${continuityId}\u0000${attachmentId}`;
}

function layoutFor(layoutKey: string, surfaces: any[]): LocalLayout {
  const existing = layouts.get(layoutKey);
  if (existing) {
    existing.surfaces = surfaces;
    existing.activeIndex = Math.min(existing.activeIndex, Math.max(0, surfaces.length - 1));
    return existing;
  }
  const layout: LocalLayout = {
    surfaces,
    activeIndex: 0,
    secondaryIndex: null,
    split: "none",
    group: "workstream",
    pinned: new Set(surfaces.filter((surface) => surface.pinned).map((surface) => surface.work_surface_id)),
    unread: new Set(surfaces.filter((surface) => surface.unread).map((surface) => surface.work_surface_id)),
  };
  layouts.set(layoutKey, layout);
  return layout;
}

function groupLabel(surface: any, mode: GroupMode): string {
  if (mode === "project") return surface.project_root || "project";
  if (mode === "session") return surface.session_id || "no-session";
  return surface.continuity_id || "workstream";
}

function surfaceSummary(surface: any, layout: LocalLayout): string {
  const flags = [
    layout.pinned.has(surface.work_surface_id) ? "pinned" : "",
    layout.unread.has(surface.work_surface_id) ? "unread" : "read",
    surface.status || "unknown",
  ].filter(Boolean);
  return `${surface.title} · ${groupLabel(surface, layout.group)} · ${flags.join(" · ")}`;
}

function inspector(surface: any): string {
  return [
    `# Work Surface Inspector — ${surface.title}`,
    "",
    `Surface: ${surface.work_surface_id}`,
    `Kind: ${surface.surface_kind}`,
    `Status: ${surface.status}`,
    `Project: ${surface.project_root}`,
    `Workstream: ${surface.continuity_id}`,
    `Attachment: ${surface.attachment_id}`,
    `Instance: ${surface.instance_id}`,
    `Session: ${surface.session_id || "none"}`,
    `Workpoint: ${surface.workpoint_id || "none"}`,
    `Pane: ${surface.pane_id} · Tab: ${surface.tab_index}`,
    "",
    "## Canonical references",
    ...(surface.canonical_state_refs || []).map((reference: string) => `- ${reference}`),
    "",
    "Presentation controls never mutate Workpoint, provider, evidence, or closure authority.",
  ].join("\n");
}

function renderLayout(layout: LocalLayout): string {
  const active = layout.surfaces[layout.activeIndex];
  const secondary =
    layout.secondaryIndex == null ? null : layout.surfaces[layout.secondaryIndex];
  const tabs = layout.surfaces.map((surface, index) => {
    const marker = index === layout.activeIndex ? "▶" : "○";
    return `${marker} ${surfaceSummary(surface, layout)}`;
  });
  return [
    "# Mission Canvas Work Surface Layout",
    "",
    `Group: ${layout.group} · Split: ${layout.split}`,
    "",
    "## Tab strip",
    ...tabs,
    "",
    `## Primary\n${active ? inspector(active) : "No active surface"}`,
    ...(secondary
      ? ["", `## Comparison (${layout.split})`, inspector(secondary)]
      : []),
  ].join("\n");
}

async function loadLayout() {
  const { projectRoot, continuityId, attachmentId } = scope();
  const query = new URLSearchParams({
    project_root: projectRoot,
    continuity_id: continuityId,
    attachment_id: attachmentId,
  });
  const response = await focusaFetch(`/mission-canvas/surfaces?${query}`);
  const surfaces = Array.isArray(response?.surfaces)
    ? [...response.surfaces].sort((left, right) => left.tab_index - right.tab_index)
    : [];
  const layoutKey = key(projectRoot, continuityId, attachmentId);
  activeLayoutKey = layoutKey;
  return layoutFor(layoutKey, surfaces);
}

export function registerMissionCanvasLayout(pi: ExtensionAPI): void {
  pi.registerCommand("focusa-surfaces", {
    description: "Switch, split, compare, group, pin, and inspect Mission Canvas Work Surfaces",
    handler: async (_args, ctx) => {
      if (!ctx.hasUI) return;
      const layout = await loadLayout();
      if (!layout.surfaces.length) {
        ctx.ui.notify("No Work Surfaces exist in the active attachment.", "info");
        return;
      }
      const action = await ctx.ui.select("Mission Canvas Work Surfaces", [
        "Switch tab",
        "Horizontal split",
        "Vertical split",
        "Side-by-side comparison",
        "Pin or unpin surface",
        "Mark read or unread",
        "Group by project",
        "Group by workstream",
        "Group by session",
        "Inspect active surface",
        "Clear split",
      ]);
      if (!action) return;
      const choose = async (title: string) => {
        const labels = layout.surfaces.map((surface) => surfaceSummary(surface, layout));
        const selected = await ctx.ui.select(title, labels);
        return selected ? labels.indexOf(selected) : -1;
      };
      if (action === "Switch tab") {
        const index = await choose("Active tab");
        if (index >= 0) layout.activeIndex = index;
      } else if (["Horizontal split", "Vertical split", "Side-by-side comparison"].includes(action)) {
        const index = await choose("Comparison surface");
        if (index >= 0) {
          layout.secondaryIndex = index;
          layout.split = action === "Vertical split" ? "vertical" : "horizontal";
        }
      } else if (action === "Pin or unpin surface" || action === "Mark read or unread") {
        const index = await choose(action);
        if (index >= 0) {
          const id = layout.surfaces[index].work_surface_id;
          const set = action.startsWith("Pin") ? layout.pinned : layout.unread;
          set.has(id) ? set.delete(id) : set.add(id);
        }
      } else if (action.startsWith("Group by ")) {
        layout.group = action.slice("Group by ".length) as GroupMode;
      } else if (action === "Clear split") {
        layout.split = "none";
        layout.secondaryIndex = null;
      }
      const active = layout.surfaces[layout.activeIndex];
      pi.sendMessage({
        customType: "focusa-canvas-layout",
        content: action === "Inspect active surface" ? inspector(active) : renderLayout(layout),
        display: true,
      });
    },
  });

  const switchBy = (delta: number) => async (ctx: any) => {
    const layout = layouts.get(activeLayoutKey);
    if (!layout?.surfaces.length) {
      ctx.ui.notify("Run /focusa-surfaces to initialize the Work Surface switcher.", "info");
      return;
    }
    layout.activeIndex =
      (layout.activeIndex + delta + layout.surfaces.length) % layout.surfaces.length;
    ctx.ui.notify(`Work Surface: ${layout.surfaces[layout.activeIndex].title}`, "info");
  };
  pi.registerShortcut("ctrl+shift+]", {
    description: "Next Mission Canvas Work Surface tab",
    handler: switchBy(1),
  });
  pi.registerShortcut("ctrl+shift+[", {
    description: "Previous Mission Canvas Work Surface tab",
    handler: switchBy(-1),
  });
}
