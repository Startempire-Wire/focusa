import type { ExtensionAPI, ExtensionContext } from "@earendil-works/pi-coding-agent";
import { truncateToWidth } from "@earendil-works/pi-tui";
import {
  currentAttachmentKey,
  getAttachmentRuntime,
  currentProjectBindingDecision,
  getSessionCwd,
} from "./state.js";
import { resolveInteractionMode } from "./config.js";
import { LifecycleGenerationGuard } from "./lifecycle-guard.js";
import {
  scopedReceiptMatchesCurrentScope,
  subscribeScopedStateChanges,
} from "./scoped-surface-refresh.js";

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
    // A Pi-local project selection is not a verified daemon workstream attachment.
    // Until that proof is available to this view, do not expose cached work or say Connected.
    const status = !runtime.focusaAvailable
      ? "Connection unavailable"
      : currentProjectBindingDecision()?.state === "BOUND"
        ? "Workstream not verified"
        : "No project connected";
    ctx.ui.setWidget("focusa-mission-canvas-work-rail", (_tui, theme) => ({
      render: (width: number) => [
        truncateToWidth(theme.fg("accent", `Focusa · ${status}`), width),
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
