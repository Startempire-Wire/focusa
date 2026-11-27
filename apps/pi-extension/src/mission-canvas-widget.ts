import type { ExtensionAPI, ExtensionContext } from "@earendil-works/pi-coding-agent";
import { truncateToWidth } from "@earendil-works/pi-tui";
import {
  currentAttachmentKey,
  runWithAttachmentRuntime,
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
let latestDisplayOwner: ReturnType<typeof currentAttachmentKey>;
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
    const owner = currentAttachmentKey();
    const ascii = process.env.FOCUSA_ASCII_UI === "1";
    ctx.ui.setWidget("focusa-mission-canvas-work-rail", (_tui, theme) => ({
      render: (width: number) => {
        if (!owner || owner !== latestDisplayOwner) return [];
        return runWithAttachmentRuntime(owner, () => {
          const bound = currentProjectBindingDecision()?.state === "BOUND";
          const available = getAttachmentRuntime().focusaAvailable;
          const packet = available && bound ? getScopedWorkpointPacket() : null;
          const rail = packet?.canonical === true ? workRailSnapshotFromPacket(packet) : null;
          if (rail) return renderWorkRailWidget(rail, width, {
            accent: text => theme.fg("accent", text),
            dim: text => theme.fg("dim", text),
            good: text => theme.fg("accent", text),
          }, ascii);
          const status = !available ? "Connection unavailable" : bound ? "Workstream not verified" : "No project connected";
          return [
            truncateToWidth(theme.fg("accent", `Focusa ${ascii ? "-" : "·"} ${status}`), width),
            truncateToWidth(theme.fg("dim", "Details: /focusa-status"), width),
          ];
        });
      },
      invalidate() {},
    }), { placement: "aboveEditor" });
  } catch (error) {
    // Teardown and generation fencing prevent every known stale-context path.
    // Keep this narrow guard as the final process-safety boundary for future callers.
    if (error instanceof Error && error.message.includes("extension ctx is stale")) {
      if (latestUiContext === ctx) stopRefreshLifecycle();
      return;
    }
    throw error;
  }
}

function stopRefreshLifecycle(): void {
  refreshLifecycle.end();
  latestUiContext = null;
  latestDisplayOwner = undefined;
  refreshUnsubscribe?.();
  refreshUnsubscribe = null;
}

function startRefreshLifecycle(ctx: ExtensionContext): void {
  stopRefreshLifecycle();
  if (ctx.mode !== "tui") return;
  const generation = refreshLifecycle.begin();
  latestUiContext = ctx;
  latestDisplayOwner = currentAttachmentKey();
  refreshUnsubscribe = subscribeScopedStateChanges((receipt) => {
    if (!refreshLifecycle.isCurrent(generation) || !latestDisplayOwner || !latestUiContext) return;
    const display = latestUiContext;
    // The publisher may be another native session. Only the display's binding
    // can select what is rendered, including on a width-only redraw.
    runWithAttachmentRuntime(latestDisplayOwner, () => {
      if (scopedReceiptMatchesCurrentScope(receipt)) refreshMissionCanvasWidget(display);
    });
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
    latestDisplayOwner = currentAttachmentKey();
    refreshMissionCanvasWidget(ctx);
  });
  pi.on("session_shutdown", () => {
    stopRefreshLifecycle();
  });
}
