import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { getActiveWorkpointPacket, getEffectiveFocusSnapshot, getSessionCwd } from "./state.js";
import { resolveInteractionMode } from "./config.js";
import { renderWorkRailWidget, workRailSnapshotFromPacket } from "./work-rail-widget.js";

/**
 * Pi-native persistent Mission Canvas entry surface.
 * The detailed canvas opens with /mission-canvas; this Work Rail keeps the
 * active mission visible at the point of work without inventing state.
 */
export function refreshMissionCanvasWidget(ctx: any): void {
  if (!ctx.hasUI) return;
  const interactionMode = resolveInteractionMode(getSessionCwd());
  if (interactionMode.mode !== "canvas-guided") {
    ctx.ui.setWidget("focusa-mission-canvas-work-rail", undefined);
    return;
  }
  const workpoint = getActiveWorkpointPacket();
  const focus = getEffectiveFocusSnapshot();
  const snapshot = workRailSnapshotFromPacket(workpoint ?? focus ?? null);
  const lines = renderWorkRailWidget(
    snapshot,
    120,
    {
      accent: (text) => text,
      dim: (text) => text,
      good: (text) => text,
    },
    true
  );
  ctx.ui.setWidget("focusa-mission-canvas-work-rail", lines, { placement: "aboveEditor" });
}

export function registerMissionCanvasWidget(pi: ExtensionAPI): void {
  pi.on("session_start", (_event, ctx) => refreshMissionCanvasWidget(ctx));
  pi.on("turn_end", (_event, ctx) => refreshMissionCanvasWidget(ctx));
}
