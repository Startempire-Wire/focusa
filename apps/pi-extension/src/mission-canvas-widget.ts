import type { ExtensionAPI, ExtensionContext } from "@earendil-works/pi-coding-agent";
import { truncateToWidth } from "@earendil-works/pi-tui";
import {
  currentAttachmentKey,
  getAttachmentRuntime,
  currentProjectBindingDecision,
  getSessionCwd,
  getScopedWorkpointPacket,
} from "./state.js";
import { resolveInteractionMode } from "./config.js";
import { LifecycleGenerationGuard } from "./lifecycle-guard.js";
import {
  scopedReceiptMatchesCurrentScope,
  subscribeScopedStateChanges,
} from "./scoped-surface-refresh.js";

import { renderWorkRailWidget, workRailSnapshotFromPacket } from "./work-rail-widget.js";

let refreshUnsubscribe: (() => void) | null = null;
let latestUiContext: ExtensionContext | null = null;
const refreshLifecycle = new LifecycleGenerationGuard();


/**
 * Pi-native persistent Mission Canvas entry surface.
 * The detailed canvas opens with /mission-canvas; this Work Rail keeps the
 * active mission visible at the point of work without inventing state.
 */
export function refreshMissionCanvasWidget(ctx: ExtensionContext): void {
  try {
    if (ctx.mode !== "tui") return;
    const runtime = getAttachmentRuntime();
    if (runtime.startupReceptionistActive) {
      ctx.ui.setWidget("focusa-mission-canvas-work-rail", undefined);
      return;
    }
    const interactionMode = resolveInteractionMode(getSessionCwd(), currentAttachmentKey());
    if (interactionMode.mode !== "canvas-guided") {
      ctx.ui.setWidget("focusa-mission-canvas-work-rail", undefined);
      return;
    }
    const bound = currentProjectBindingDecision()?.state === "BOUND";
    const packet = runtime.focusaAvailable && bound ? getScopedWorkpointPacket() : null;
    const rail = packet?.canonical === true ? workRailSnapshotFromPacket(packet) : null;
    const ascii = process.env.FOCUSA_ASCII_UI === "1";
    const status = !runtime.focusaAvailable
      ? "Connection unavailable"
      : bound
        ? "Workstream not verified"
        : "No project connected";
    ctx.ui.setWidget("focusa-mission-canvas-work-rail", (_tui, theme) => ({
      render: (width: number) => rail
        ? renderWorkRailWidget(rail, width, {
          accent: text => theme.fg("accent", text),
          dim: text => theme.fg("dim", text),
          good: text => theme.fg("accent", text),
        }, ascii)
        : [
        truncateToWidth(theme.fg("accent", `Focusa ${ascii ? "-" : "·"} ${status}`), width),
        truncateToWidth(theme.fg("dim", "Details: /focusa-status"), width),
      ],
      invalidate() {},
    }), { placement: "aboveEditor" });
  } catch (error) {
    // Teardown and generation fencing prevent every known stale-context path.
    // Keep this narrow guard as the final process-safety boundary for future callers.
    if (error instanceof Error && error.message.includes("extension ctx is stale")) {
      if (latestUiContext === ctx) latestUiContext = null;
      return;
    }
    throw error;
  }
}

function stopRefreshLifecycle(): void {
  refreshLifecycle.end();
  latestUiContext = null;
  refreshUnsubscribe?.();
  refreshUnsubscribe = null;
}

function startRefreshLifecycle(ctx: ExtensionContext): void {
  stopRefreshLifecycle();
  if (ctx.mode !== "tui") return;
  const generation = refreshLifecycle.begin();
  latestUiContext = ctx;
  refreshUnsubscribe = subscribeScopedStateChanges((receipt) => {
    if (!refreshLifecycle.isCurrent(generation)) return;
    if (!scopedReceiptMatchesCurrentScope(receipt) || !latestUiContext) return;
    refreshMissionCanvasWidget(latestUiContext);
  });
}

export function registerMissionCanvasWidget(pi: ExtensionAPI): void {
  pi.on("session_start", (_event, ctx) => {
    startRefreshLifecycle(ctx);
    refreshMissionCanvasWidget(ctx);
  });
  pi.on("turn_end", (_event, ctx) => {
    if (ctx.mode !== "tui") return;
    latestUiContext = ctx;
    refreshMissionCanvasWidget(ctx);
  });
  pi.on("session_shutdown", () => {
    stopRefreshLifecycle();
  });
}
