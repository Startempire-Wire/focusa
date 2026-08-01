import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
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
  setActiveWorkpointPacket,
  setLastTrajectoryClarity,
  stampWorkpointPacketForCurrentPiSession,
} from "./state.js";
import { resolveInteractionMode } from "./config.js";
import { renderWorkRailWidget, workRailSnapshotFromPacket } from "./work-rail-widget.js";
import {
  buildTruthfulScopedSurfaceSnapshot,
  latestScopedStateChange,
  publishScopedStateChange,
  scopedReceiptMatchesCurrentScope,
  subscribeScopedStateChanges,
} from "./scoped-surface-refresh.js";

let refreshUnsubscribe: (() => void) | null = null;
let pollingTimer: ReturnType<typeof setInterval> | null = null;
let latestUiContext: any = null;
let startupCwd = "";
let pollInFlight = false;
let semanticTruth = "schema_only";
let semanticOperations = 0;
let semanticMutations = 0;
let semanticOperationLines: string[] = [];

function bounded(value: unknown, max = 56): string {
  const text = String(value || "")
    .replace(/\s+/g, " ")
    .trim();
  return text.length > max ? `${text.slice(0, max - 1)}…` : text;
}

function truthfulStatusLines(ctx: any): string[] {
  const truth = buildTruthfulScopedSurfaceSnapshot(startupCwd || ctx?.cwd || "");
  const proof = truth.proof === "missing" ? "proof missing" : `${truth.proof} proof ${truth.proof_count}`;
  const stale = truth.stale_age_ms < 0 ? "never refreshed" : `age ${Math.round(truth.stale_age_ms / 1000)}s`;
  return [
    `scope ${bounded(truth.selected_scope)} · startup ${bounded(truth.startup_cwd)} · project ${truth.project}`,
    `trajectory ${truth.trajectory} · workpoint ${truth.workpoint} · bead ${truth.bead} · ${proof}`,
    `refresh ${truth.last_refresh_status} · ${stale}`,
    `semantic pair ${semanticTruth} · ${semanticOperations} operations · ${semanticMutations} mutations visible`,
    ...semanticOperationLines,
  ];
}

/**
 * Pi-native persistent Mission Canvas entry surface.
 * The detailed canvas opens with /mission-canvas; this Work Rail keeps the
 * active mission visible at the point of work without inventing state.
 */
export function refreshMissionCanvasWidget(ctx: any): void {
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
}

async function pollScopedSurfaceState(ctx: any): Promise<void> {
  if (pollInFlight) return;
  const projectRoot = normalizeProjectRoot(getSessionCwd());
  const continuityId = getContinuityId();
  if (!projectRoot || !continuityId) return;
  pollInFlight = true;
  try {
    const trajectoryQuery = new URLSearchParams({
      project_root: projectRoot,
      continuity_id: continuityId,
      mode: "summary",
    });
    const [trajectoryResult, workpointResult, semanticResult, semanticRegistry] = await Promise.all([
      focusaFetch(`/trajectory/view?${trajectoryQuery.toString()}`, { method: "GET" }).catch(() => null),
      focusaFetch("/workpoint/resume", {
        method: "POST",
        body: JSON.stringify({
          mode: "compact_prompt",
          project_root: projectRoot,
          continuity_id: continuityId,
        }),
      }).catch(() => null),
      focusaFetch(`/semantic-integrity/status?${trajectoryQuery.toString()}`, { method: "GET" }).catch(() => null),
      focusaFetch(`/semantic-integrity/operations?${trajectoryQuery.toString()}&limit=100`, { method: "GET" }).catch(() => null),
    ]);
    const semantic = semanticResult && typeof semanticResult === "object"
      ? semanticResult as Record<string, unknown> : {};
    semanticTruth = bounded(semantic.state || "degraded", 40);
    const registry = semanticRegistry && typeof semanticRegistry === "object"
      ? semanticRegistry as Record<string, unknown> : {};
    const operations = Array.isArray(registry.items) ? registry.items : [];
    semanticOperations = operations.length;
    semanticMutations = operations.filter((item) =>
      item && typeof item === "object" && (item as Record<string, unknown>).kind === "mutation"
    ).length;
    semanticOperationLines = operations.map((item) => {
      const op = item && typeof item === "object" ? item as Record<string, unknown> : {};
      const kind = bounded(op.kind || "read", 12);
      const support = kind === "mutation"
        ? "unsupported on this Pi surface" : bounded(op.availability || "available", 24);
      return `  ${bounded(op.operation_id || "unknown", 48)} · ${kind} · ${support}`;
    });
    const trajectoryProjectRoot = normalizeProjectRoot(
      trajectoryResult?.project_identity?.project_root || trajectoryResult?.trajectory?.project_root
    );
    if (trajectoryProjectRoot === projectRoot) {
      const trajectory = trajectoryResult?.trajectory || {};
      setLastTrajectoryClarity({
        status: trajectoryResult?.status || "projected",
        canonical: trajectoryResult?.canonical === true,
        degraded: trajectoryResult?.degraded === true,
        project_root: projectRoot,
        continuity_id: continuityId,
        trajectory_id: trajectory.trajectory_id || null,
        long_term_goal: trajectory.long_term_goal || trajectory.trajectory_ladder?.hlt || null,
        desired_end_state:
          trajectory.desired_end_state || trajectory.trajectory_ladder?.desired_end_state || null,
        mid_level_goal: trajectory.mid_level_goal || trajectory.trajectory_ladder?.mlg || null,
        short_term_goal: trajectory.short_term_goal || trajectory.trajectory_ladder?.stg || null,
        waypoints: trajectory.waypoints || trajectory.trajectory_ladder?.waypoints || [],
        current_state: trajectory.current_state || null,
        active_gap: trajectory.active_gap || null,
      });
    }
    if (
      workpointResult?.status === "completed" &&
      workpointResult?.matches_current_ask_scope !== false &&
      normalizeProjectRoot(workpointResult?.scope?.project_root || projectRoot) === projectRoot
    ) {
      const packet = normalizeWorkpointResumePacketEnvelope(workpointResult);
      setActiveWorkpointPacket(stampWorkpointPacketForCurrentPiSession(packet));
    }
    publishScopedStateChange({
      source: "poll",
      mutation_kind: "scoped_surface_refresh",
      project_root: projectRoot,
      continuity_id: continuityId,
      status: trajectoryResult || workpointResult ? "observed" : "degraded",
      effective_at: new Date().toISOString(),
    });
  } finally {
    pollInFlight = false;
    refreshMissionCanvasWidget(ctx);
  }
}

function ensureRefreshLifecycle(ctx: any): void {
  latestUiContext = ctx;
  startupCwd ||= String(ctx?.cwd || "");
  if (!refreshUnsubscribe) {
    refreshUnsubscribe = subscribeScopedStateChanges((receipt) => {
      if (!scopedReceiptMatchesCurrentScope(receipt) || !latestUiContext) return;
      refreshMissionCanvasWidget(latestUiContext);
    });
  }
  if (!pollingTimer) {
    pollingTimer = setInterval(() => {
      if (!latestUiContext) return;
      const receipt = latestScopedStateChange();
      const age = receipt ? Date.now() - Date.parse(receipt.effective_at) : Number.POSITIVE_INFINITY;
      if (age >= 60_000) void pollScopedSurfaceState(latestUiContext);
    }, 30_000);
    pollingTimer.unref?.();
  }
}

export function registerMissionCanvasWidget(pi: ExtensionAPI): void {
  pi.on("session_start", (_event, ctx) => {
    ensureRefreshLifecycle(ctx);
    publishScopedStateChange({
      source: "session",
      mutation_kind: "session_start",
      project_root: getSessionCwd(),
      continuity_id: getContinuityId(),
      status: "observed",
      effective_at: new Date().toISOString(),
    });
    refreshMissionCanvasWidget(ctx);
    setTimeout(() => void pollScopedSurfaceState(ctx), 1_000).unref?.();
  });
  pi.on("turn_end", (_event, ctx) => {
    ensureRefreshLifecycle(ctx);
    refreshMissionCanvasWidget(ctx);
  });
}
