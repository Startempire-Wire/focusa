import type { ExtensionAPI, ExtensionContext } from "@earendil-works/pi-coding-agent";
import {
  focusaFetch,
  getAttachmentRuntime,
  currentProjectBindingDecision,
  getActiveWorkpointPacket,
  getContinuityId,
  getEffectiveFocusSnapshot,
  getSessionCwd,
  normalizeProjectRoot,
  normalizeWorkpointResumePacketEnvelope,
  refreshTrajectoryClarityLifecycle,
  setActiveWorkpointPacket,
  stampWorkpointPacketForCurrentPiSession,
} from "./state.js";
import { resolveInteractionMode } from "./config.js";
import { LifecycleGenerationGuard } from "./lifecycle-guard.js";
import { renderWorkRailWidget, workRailSnapshotFromPacket } from "./work-rail-widget.js";
import {
  buildTruthfulScopedSurfaceSnapshot,
  currentScopedProjectRoot,
  latestScopedStateChange,
  publishScopedStateChange,
  scopedReceiptMatchesCurrentScope,
  subscribeScopedStateChanges,
} from "./scoped-surface-refresh.js";

let refreshUnsubscribe: (() => void) | null = null;
let initialPollTimer: ReturnType<typeof setTimeout> | null = null;
let pollingTimer: ReturnType<typeof setInterval> | null = null;
let latestUiContext: ExtensionContext | null = null;
let startupCwd = "";
const refreshLifecycle = new LifecycleGenerationGuard();
let pollInFlightGeneration: number | null = null;

function bounded(value: unknown, max = 56): string {
  const text = String(value || "")
    .replace(/\s+/g, " ")
    .trim();
  return text.length > max ? `${text.slice(0, max - 1)}…` : text;
}

function truthfulStatusLines(ctx: ExtensionContext): string[] {
  const truth = buildTruthfulScopedSurfaceSnapshot(startupCwd || ctx?.cwd || "");
  const proof = truth.proof === "missing" ? "proof missing" : `${truth.proof} proof ${truth.proof_count}`;
  const stale = truth.stale_age_ms < 0 ? "never refreshed" : `age ${Math.round(truth.stale_age_ms / 1000)}s`;
  return [
    `scope ${bounded(truth.selected_scope)} · startup ${bounded(truth.startup_cwd)} · project ${truth.project}`,
    `trajectory ${truth.trajectory} · workpoint ${truth.workpoint} · bead ${truth.bead} · ${proof}`,
    `refresh ${truth.last_refresh_status} · ${stale}`,
  ];
}

/**
 * Pi-native persistent Mission Canvas entry surface.
 * The detailed canvas opens with /mission-canvas; this Work Rail keeps the
 * active mission visible at the point of work without inventing state.
 */
export function refreshMissionCanvasWidget(ctx: ExtensionContext): void {
  try {
    if (!ctx?.hasUI) return;
    const binding = currentProjectBindingDecision();
    if (getAttachmentRuntime().startupReceptionistActive || !binding || binding.state !== "BOUND") {
      ctx.ui.setWidget("focusa-mission-canvas-work-rail", undefined);
      return;
    }
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
    ctx.ui.setWidget("focusa-mission-canvas-work-rail", [...truthfulStatusLines(ctx), ...lines], {
      placement: "aboveEditor",
    });
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

async function pollScopedSurfaceState(generation: number): Promise<void> {
  if (!refreshLifecycle.isCurrent(generation) || pollInFlightGeneration === generation) return;
  const projectRoot = currentScopedProjectRoot();
  const continuityId = getContinuityId();
  if (!projectRoot || !continuityId) return;
  pollInFlightGeneration = generation;
  try {
    const [trajectoryRefresh, workpointResult] = await Promise.all([
      refreshTrajectoryClarityLifecycle("mission_canvas_poll", projectRoot).catch(() => null),
      focusaFetch("/workpoint/resume", {
        method: "POST",
        body: JSON.stringify({
          mode: "compact_prompt",
          project_root: projectRoot,
          continuity_id: continuityId,
          current_ask: getAttachmentRuntime().currentAsk?.text || undefined,
        }),
      }).catch(() => null),
    ]);
    if (!refreshLifecycle.isCurrent(generation)) return;
    const packet = normalizeWorkpointResumePacketEnvelope(workpointResult);
    const packetRoot = normalizeProjectRoot(
      packet?.project_root || packet?.scope?.project_root || workpointResult?.scope?.project_root || ""
    );
    const packetContinuity = String(
      packet?.continuity_id || packet?.scope?.continuity_id || workpointResult?.scope?.continuity_id || ""
    ).trim();
    if (
      workpointResult?.status === "completed" &&
      packet &&
      packetRoot === projectRoot &&
      packetContinuity === continuityId
    ) {
      setActiveWorkpointPacket(stampWorkpointPacketForCurrentPiSession(packet));
    }
    publishScopedStateChange({
      source: "poll",
      mutation_kind: "scoped_surface_refresh",
      project_root: projectRoot,
      continuity_id: continuityId,
      status: trajectoryRefresh || workpointResult ? "observed" : "degraded",
      effective_at: new Date().toISOString(),
    });
  } finally {
    if (pollInFlightGeneration === generation) pollInFlightGeneration = null;
    if (refreshLifecycle.isCurrent(generation) && latestUiContext) {
      refreshMissionCanvasWidget(latestUiContext);
    }
  }
}

function stopRefreshLifecycle(): void {
  refreshLifecycle.end();
  latestUiContext = null;
  startupCwd = "";
  if (initialPollTimer) {
    clearTimeout(initialPollTimer);
    initialPollTimer = null;
  }
  if (refreshUnsubscribe) {
    refreshUnsubscribe();
    refreshUnsubscribe = null;
  }
  if (pollingTimer) {
    clearInterval(pollingTimer);
    pollingTimer = null;
  }
}

function startRefreshLifecycle(ctx: ExtensionContext): number {
  stopRefreshLifecycle();
  const generation = refreshLifecycle.begin();
  latestUiContext = ctx;
  startupCwd = String(ctx.cwd || "");
  refreshUnsubscribe = subscribeScopedStateChanges((receipt) => {
    if (!refreshLifecycle.isCurrent(generation)) return;
    if (!scopedReceiptMatchesCurrentScope(receipt) || !latestUiContext) return;
    refreshMissionCanvasWidget(latestUiContext);
  });
  pollingTimer = setInterval(() => {
    if (!refreshLifecycle.isCurrent(generation) || !latestUiContext) return;
    const receipt = latestScopedStateChange();
    const age = receipt ? Date.now() - Date.parse(receipt.effective_at) : Number.POSITIVE_INFINITY;
    if (age >= 60_000) void pollScopedSurfaceState(generation);
  }, 30_000);
  pollingTimer.unref?.();
  return generation;
}

export function registerMissionCanvasWidget(pi: ExtensionAPI): void {
  pi.on("session_start", (_event, ctx) => {
    const generation = startRefreshLifecycle(ctx);
    publishScopedStateChange({
      source: "session",
      mutation_kind: "session_start",
      project_root: getSessionCwd(),
      continuity_id: getContinuityId(),
      status: "observed",
      effective_at: new Date().toISOString(),
    });
    refreshMissionCanvasWidget(ctx);
    const timer = setTimeout(() => {
      if (initialPollTimer === timer) initialPollTimer = null;
      void pollScopedSurfaceState(generation);
    }, 1_000);
    initialPollTimer = timer;
    timer.unref?.();
  });
  pi.on("turn_end", (_event, ctx) => {
    latestUiContext = ctx;
    refreshMissionCanvasWidget(ctx);
  });
  pi.on("session_shutdown", () => {
    stopRefreshLifecycle();
  });
}
