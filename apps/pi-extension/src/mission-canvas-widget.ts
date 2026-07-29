import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { getActiveWorkpointPacket, getEffectiveFocusSnapshot, getSessionCwd } from "./state.js";
import { resolveInteractionMode } from "./config.js";
import { renderWorkRailWidget, workRailSnapshotFromPacket } from "./work-rail-widget.js";

const WIDGET_ID = "focusa-mission-canvas-work-rail";

/**
 * Pi-native persistent Mission Canvas entry surface.
 * Spec 135A requires one Work Rail above the editor in compatibility mode.
 */
export function refreshMissionCanvasWidget(ctx: any, badges: string[] = []): void {
  if (!ctx.hasUI) return;
  const interactionMode = resolveInteractionMode(getSessionCwd());
  if (interactionMode.mode !== "canvas-guided") {
    ctx.ui.setWidget(WIDGET_ID, undefined);
    return;
  }
  const workpoint = getActiveWorkpointPacket();
  const focus = getEffectiveFocusSnapshot();
  const snapshot = workRailSnapshotFromPacket(workpoint ?? focus ?? null);
  snapshot.badges = badges;
  const ascii = process.env.FOCUSA_ASCII_UI === "1" || process.env.TERM === "dumb";
  ctx.ui.setWidget(
    WIDGET_ID,
    (_tui: any, theme: any) => ({
      render(width: number) {
        return renderWorkRailWidget(
          snapshot,
          width,
          {
            accent: (text) => theme.fg("accent", text),
            dim: (text) => theme.fg("dim", text),
            good: (text) => theme.fg("accent", text),
          },
          ascii
        );
      },
      invalidate() {},
    }),
    { placement: "aboveEditor" }
  );
}

export function registerMissionCanvasWidget(pi: ExtensionAPI): void {
  pi.on("session_start", (_event, ctx) => refreshMissionCanvasWidget(ctx));
}
