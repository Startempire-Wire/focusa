import { focusaFetch, getAttachmentRuntime, currentProjectBindingDecision, getActiveWorkpointPacket, getContinuityId, getEffectiveFocusSnapshot, getSessionCwd, normalizeProjectRoot, normalizeWorkpointResumePacketEnvelope, refreshTrajectoryClarityLifecycle, setActiveWorkpointPacket, stampWorkpointPacketForCurrentPiSession, } from "./state.js";
import { resolveInteractionMode } from "./config.js";
import { renderWorkRailWidget, workRailSnapshotFromPacket } from "./work-rail-widget.js";
import { semanticSurfaceTruth } from "./semantic-surface-truth.js";
import { buildTruthfulScopedSurfaceSnapshot, currentScopedProjectRoot, latestScopedStateChange, publishScopedStateChange, scopedReceiptMatchesCurrentScope, subscribeScopedStateChanges, } from "./scoped-surface-refresh.js";
let refreshUnsubscribe = null;
let pollingTimer = null;
let latestUiContext = null;
let startupCwd = "";
let pollInFlight = false;
let semanticTruth = "schema_only";
let semanticOperations = 0;
let semanticMutations = 0;
let semanticSupported = 0;
let semanticSchemaOnly = 0;
let semanticOperationLines = [];
function bounded(value, max = 56) {
    const text = String(value || "")
        .replace(/\s+/g, " ")
        .trim();
    return text.length > max ? `${text.slice(0, max - 1)}…` : text;
}
function truthfulStatusLines(ctx) {
    const truth = buildTruthfulScopedSurfaceSnapshot(startupCwd || ctx?.cwd || "");
    const proof = truth.proof === "missing" ? "proof missing" : `${truth.proof} proof ${truth.proof_count}`;
    const stale = truth.stale_age_ms < 0 ? "never refreshed" : `age ${Math.round(truth.stale_age_ms / 1000)}s`;
    return [
        `scope ${bounded(truth.selected_scope)} · startup ${bounded(truth.startup_cwd)} · project ${truth.project}`,
        `trajectory ${truth.trajectory} · workpoint ${truth.workpoint} · bead ${truth.bead} · ${proof}`,
        `refresh ${truth.last_refresh_status} · ${stale}`,
        `semantic pair ${semanticTruth} · ${semanticOperations} operations · ${semanticSupported} supported · ${semanticSchemaOnly} schema-only · ${semanticMutations} mutations`,
        ...semanticOperationLines,
    ];
}
/**
 * Pi-native persistent Mission Canvas entry surface.
 * The detailed canvas opens with /mission-canvas; this Work Rail keeps the
 * active mission visible at the point of work without inventing state.
 */
export function refreshMissionCanvasWidget(ctx) {
    if (!ctx?.hasUI)
        return;
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
    const lines = renderWorkRailWidget(snapshot, 120, {
        accent: (text) => text,
        dim: (text) => text,
        good: (text) => text,
    }, true);
    ctx.ui.setWidget("focusa-mission-canvas-work-rail", [...truthfulStatusLines(ctx), ...lines], {
        placement: "aboveEditor",
    });
}
async function pollScopedSurfaceState(ctx) {
    if (pollInFlight)
        return;
    const projectRoot = currentScopedProjectRoot();
    const continuityId = getContinuityId();
    if (!projectRoot || !continuityId)
        return;
    pollInFlight = true;
    try {
        const trajectoryQuery = new URLSearchParams({
            project_root: projectRoot,
            continuity_id: continuityId,
            mode: "summary",
        });
        const [trajectoryRefresh, workpointResult, semanticResult, semanticRegistry] = await Promise.all([
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
            focusaFetch(`/semantic-integrity/status?${trajectoryQuery.toString()}`, { method: "GET" }).catch(() => null),
            focusaFetch(`/semantic-integrity/operations?${trajectoryQuery.toString()}&limit=100`, {
                method: "GET",
            }).catch(() => null),
        ]);
        const semanticSummary = semanticSurfaceTruth(semanticResult, semanticRegistry);
        semanticTruth = semanticSummary.state;
        semanticOperations = semanticSummary.operationCount;
        semanticMutations = semanticSummary.mutationCount;
        semanticSupported = semanticSummary.supportedCount;
        semanticSchemaOnly = semanticSummary.schemaOnlyCount;
        semanticOperationLines = semanticSummary.operationLines;
        const packet = normalizeWorkpointResumePacketEnvelope(workpointResult);
        const packetRoot = normalizeProjectRoot(packet?.project_root || packet?.scope?.project_root || workpointResult?.scope?.project_root || "");
        const packetContinuity = String(packet?.continuity_id || packet?.scope?.continuity_id || workpointResult?.scope?.continuity_id || "").trim();
        if (workpointResult?.status === "completed" &&
            packet &&
            packetRoot === projectRoot &&
            packetContinuity === continuityId) {
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
    }
    finally {
        pollInFlight = false;
        refreshMissionCanvasWidget(ctx);
    }
}
function ensureRefreshLifecycle(ctx) {
    latestUiContext = ctx;
    startupCwd ||= String(ctx?.cwd || "");
    if (!refreshUnsubscribe) {
        refreshUnsubscribe = subscribeScopedStateChanges((receipt) => {
            if (!scopedReceiptMatchesCurrentScope(receipt) || !latestUiContext)
                return;
            refreshMissionCanvasWidget(latestUiContext);
        });
    }
    if (!pollingTimer) {
        pollingTimer = setInterval(() => {
            if (!latestUiContext)
                return;
            const receipt = latestScopedStateChange();
            const age = receipt ? Date.now() - Date.parse(receipt.effective_at) : Number.POSITIVE_INFINITY;
            if (age >= 60_000)
                void pollScopedSurfaceState(latestUiContext);
        }, 30_000);
        pollingTimer.unref?.();
    }
}
export function registerMissionCanvasWidget(pi) {
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
