import type { ExtensionAPI, ExtensionContext } from "@earendil-works/pi-coding-agent";
import { focusaFetch, getActiveWorkpointPacket, getContinuityId, getSessionCwd } from "./state.js";

type SplitOrientation = "none" | "horizontal" | "vertical";
type GroupMode = "project" | "workstream" | "session";
type SurfaceAction = "create" | "arrange" | "suspend" | "resume" | "close_view";

interface LayoutScope {
  projectRoot: string;
  continuityId: string;
  attachmentId: string;
}

interface LocalLayout {
  scope: LayoutScope;
  stateVersion: number;
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
let mutationOrdinal = 0;

function scope(): LayoutScope {
  const packet = getActiveWorkpointPacket();
  const workpoint = packet?.workpoint && typeof packet.workpoint === "object" ? packet.workpoint : packet;
  const projectRoot = getSessionCwd();
  const continuityId = getContinuityId();
  const attachmentId = String(
    workpoint?.attachment_id || packet?.attachment_id || `attachment:canvas:${continuityId}`
  );
  return { projectRoot, continuityId, attachmentId };
}

function key(value: LayoutScope): string {
  return `${value.projectRoot}\u0000${value.continuityId}\u0000${value.attachmentId}`;
}

function sortedSurfaces(surfaces: any[]): any[] {
  return [...surfaces].sort((left, right) => {
    const tabOrder = Number(left?.tab_index ?? 0) - Number(right?.tab_index ?? 0);
    return tabOrder || String(left?.work_surface_id ?? "").localeCompare(String(right?.work_surface_id ?? ""));
  });
}

function syncPresentationSets(layout: LocalLayout): void {
  layout.pinned = new Set(
    layout.surfaces.filter((surface) => surface.pinned).map((surface) => String(surface.work_surface_id))
  );
  layout.unread = new Set(
    layout.surfaces.filter((surface) => surface.unread).map((surface) => String(surface.work_surface_id))
  );
}

function replaceSurfaces(layout: LocalLayout, surfaces: any[]): void {
  const activeId = layout.surfaces[layout.activeIndex]?.work_surface_id;
  const secondaryId =
    layout.secondaryIndex == null ? undefined : layout.surfaces[layout.secondaryIndex]?.work_surface_id;
  layout.surfaces = sortedSurfaces(surfaces);
  layout.activeIndex = Math.max(
    0,
    activeId ? layout.surfaces.findIndex((surface) => surface.work_surface_id === activeId) : 0
  );
  if (layout.activeIndex < 0) layout.activeIndex = 0;
  const secondaryIndex = secondaryId
    ? layout.surfaces.findIndex((surface) => surface.work_surface_id === secondaryId)
    : -1;
  layout.secondaryIndex = secondaryIndex >= 0 ? secondaryIndex : null;
  syncPresentationSets(layout);
}

function layoutFor(layoutScope: LayoutScope, stateVersion: number, surfaces: any[]): LocalLayout {
  const layoutKey = key(layoutScope);
  const existing = layouts.get(layoutKey);
  if (existing) {
    existing.scope = layoutScope;
    existing.stateVersion = stateVersion;
    replaceSurfaces(existing, surfaces);
    return existing;
  }
  const layout: LocalLayout = {
    scope: layoutScope,
    stateVersion,
    surfaces: [],
    activeIndex: 0,
    secondaryIndex: null,
    split: "none",
    group: "workstream",
    pinned: new Set(),
    unread: new Set(),
  };
  replaceSurfaces(layout, surfaces);
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
  if (!surface) return "No active surface";
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
    "Closing this view never terminates its session or canonical work.",
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
    `Group: ${layout.group} · Split: ${layout.split} · Durable state: v${layout.stateVersion}`,
    "",
    "## Tab strip",
    ...(tabs.length ? tabs : ["No open Work Surfaces"]),
    "",
    `## Primary\n${inspector(active)}`,
    ...(secondary ? ["", `## Comparison (${layout.split})`, inspector(secondary)] : []),
  ].join("\n");
}

async function loadLayout(): Promise<LocalLayout> {
  const layoutScope = scope();
  const query = new URLSearchParams({
    project_root: layoutScope.projectRoot,
    continuity_id: layoutScope.continuityId,
    attachment_id: layoutScope.attachmentId,
  });
  const response = await focusaFetch(`/mission-canvas/surfaces?${query}`);
  const surfaces = Array.isArray(response?.surfaces) ? response.surfaces : [];
  const layoutKey = key(layoutScope);
  activeLayoutKey = layoutKey;
  return layoutFor(layoutScope, Number(response?.state_version ?? 0), surfaces);
}

function activeWorkpoint(): any {
  const packet = getActiveWorkpointPacket();
  return packet?.workpoint && typeof packet.workpoint === "object" ? packet.workpoint : packet ?? {};
}

async function mutateSurface(
  layout: LocalLayout,
  action: SurfaceAction,
  surface?: any,
  patch: Record<string, unknown> = {}
): Promise<any> {
  const body = {
    project_root: layout.scope.projectRoot,
    continuity_id: layout.scope.continuityId,
    attachment_id: layout.scope.attachmentId,
    idempotency_key: `pi-canvas:${action}:${Date.now().toString(36)}:${++mutationOrdinal}`,
    expected_state_version: layout.stateVersion,
    expected_surface_revision: Number(surface?.state_revision ?? 0),
    action,
    ...(surface?.work_surface_id ? { work_surface_id: surface.work_surface_id } : {}),
    ...patch,
  };
  const response = await focusaFetch("/mission-canvas/surfaces/mutate", {
    method: "POST",
    body: JSON.stringify(body),
  });
  if (!response?.surface) {
    throw new Error("Mission Canvas rejected the Work Surface change; refresh and retry");
  }
  layout.stateVersion = Number(response.state_version ?? layout.stateVersion + 1);
  const next = layout.surfaces.filter(
    (candidate) => candidate.work_surface_id !== response.surface.work_surface_id
  );
  next.push(response.surface);
  replaceSurfaces(layout, next);
  return response.surface;
}

async function chooseSurface(ctx: ExtensionContext, layout: LocalLayout, title: string): Promise<any | null> {
  if (!layout.surfaces.length) {
    ctx.ui.notify("No Work Surfaces exist in the active attachment.", "info");
    return null;
  }
  const labels = layout.surfaces.map((surface) => surfaceSummary(surface, layout));
  const selected = await ctx.ui.select(title, labels);
  const index = selected ? labels.indexOf(selected) : -1;
  return index >= 0 ? layout.surfaces[index] : null;
}

async function createSurface(ctx: ExtensionContext, layout: LocalLayout): Promise<void> {
  const title = (await ctx.ui.input("Work Surface title"))?.trim();
  if (!title) return;
  const kindLabel = await ctx.ui.select("Work Surface kind", [
    "Project overview",
    "Pi session",
    "Document",
    "Research",
    "Evidence",
    "Provider item",
  ]);
  if (!kindLabel) return;
  const workpoint = activeWorkpoint();
  const workpointId = String(workpoint?.workpoint_id || workpoint?.id || "").trim();
  const missionRef = String(
    workpoint?.mission_ref || workpointId || `continuity:${layout.scope.continuityId}`
  );
  const canonicalStateRefs = [
    workpointId,
    missionRef,
    layout.scope.attachmentId,
  ].filter((value, index, values) => value && values.indexOf(value) === index);
  const surface = await mutateSurface(layout, "create", undefined, {
    instance_id: `pi:${Date.now().toString(36)}:${mutationOrdinal + 1}`,
    workpoint_id: workpointId || undefined,
    mission_ref: missionRef,
    title,
    surface_kind: kindLabel.toLowerCase().replaceAll(" ", "_"),
    pane_id: "primary",
    tab_index: layout.surfaces.length,
    pinned: false,
    unread: false,
    canonical_state_refs: canonicalStateRefs,
  });
  layout.activeIndex = layout.surfaces.findIndex(
    (candidate) => candidate.work_surface_id === surface.work_surface_id
  );
}

function publishLayout(pi: ExtensionAPI, layout: LocalLayout, content?: string): void {
  pi.sendMessage({
    customType: "focusa-canvas-layout",
    content: content ?? renderLayout(layout),
    display: true,
  });
}

export async function openMissionCanvasSurfaceManager(
  pi: ExtensionAPI,
  ctx: ExtensionContext
): Promise<void> {
  if (!ctx.hasUI) return;
  const layout = await loadLayout();
  const action = await ctx.ui.select("Mission Canvas Work Surfaces", [
    "Create surface",
    "Switch tab",
    "Move surface",
    "Horizontal split",
    "Vertical split",
    "Side-by-side comparison",
    "Pin or unpin surface",
    "Mark read or unread",
    "Suspend surface",
    "Resume surface",
    "Close view (work continues)",
    "Group by project",
    "Group by workstream",
    "Group by session",
    "Inspect active surface",
    "Clear split",
  ]);
  if (!action) return;

  try {
    if (action === "Create surface") {
      await createSurface(ctx, layout);
    } else if (action === "Switch tab") {
      const surface = await chooseSurface(ctx, layout, "Active tab");
      if (surface) layout.activeIndex = layout.surfaces.indexOf(surface);
    } else if (action === "Move surface") {
      const surface = await chooseSurface(ctx, layout, action);
      if (surface) {
        const raw = await ctx.ui.input("Tab position (1 is first)", String(Number(surface.tab_index ?? 0) + 1));
        const position = Number(raw);
        if (Number.isInteger(position) && position > 0) {
          await mutateSurface(layout, "arrange", surface, { tab_index: position - 1 });
        }
      }
    } else if (["Horizontal split", "Vertical split", "Side-by-side comparison"].includes(action)) {
      const surface = await chooseSurface(ctx, layout, "Comparison surface");
      if (surface) {
        layout.secondaryIndex = layout.surfaces.indexOf(surface);
        layout.split = action === "Vertical split" ? "vertical" : "horizontal";
        await mutateSurface(layout, "arrange", surface, { pane_id: `secondary:${layout.split}` });
      }
    } else if (action === "Pin or unpin surface" || action === "Mark read or unread") {
      const surface = await chooseSurface(ctx, layout, action);
      if (surface) {
        const patch = action.startsWith("Pin")
          ? { pinned: !Boolean(surface.pinned) }
          : { unread: !Boolean(surface.unread) };
        await mutateSurface(layout, "arrange", surface, patch);
      }
    } else if (action === "Suspend surface" || action === "Resume surface") {
      const surface = await chooseSurface(ctx, layout, action);
      if (surface) {
        await mutateSurface(layout, action.startsWith("Suspend") ? "suspend" : "resume", surface);
      }
    } else if (action === "Close view (work continues)") {
      const surface = await chooseSurface(ctx, layout, action);
      if (surface) {
        const confirmed = await ctx.ui.confirm(
          "Close Work Surface view?",
          "This removes only the view. Its session and canonical work continue."
        );
        if (confirmed) await mutateSurface(layout, "close_view", surface);
      }
    } else if (action.startsWith("Group by ")) {
      layout.group = action.slice("Group by ".length) as GroupMode;
    } else if (action === "Clear split") {
      const secondary =
        layout.secondaryIndex == null ? null : layout.surfaces[layout.secondaryIndex];
      if (secondary) await mutateSurface(layout, "arrange", secondary, { pane_id: "primary" });
      layout.split = "none";
      layout.secondaryIndex = null;
    }

    const active = layout.surfaces[layout.activeIndex];
    publishLayout(pi, layout, action === "Inspect active surface" ? inspector(active) : undefined);
  } catch (error) {
    ctx.ui.notify(`Work Surface change failed: ${String(error)}`, "error");
  }
}

export function registerMissionCanvasLayout(pi: ExtensionAPI): void {
  pi.registerCommand("focusa-surfaces", {
    description: "Create, switch, arrange, suspend, resume, and close Mission Canvas Work Surfaces",
    handler: async (_args, ctx) => openMissionCanvasSurfaceManager(pi, ctx),
  });

  const switchBy = (delta: number) => async (ctx: ExtensionContext) => {
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
