// Session lifecycle events — ONE handler per event type (merged)
// Spec: §11 (outage audit + backoff), §30 (SSE metacog), §34.2A (instance),
//        §35.1 (auto-frame), §36.4 (resume), §36.5 (fork/tree), §37.5 (flags),
//        §35.8 (session name), §37.9 (Context Core), §37.10 (cross-surface SSE),
//        §38.3 (health toggle)

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { S, focusaFetch, focusaPost, checkFocusa, kickstartFocusaDaemon, persistState, persistAuthoritativeState, getFocusState, createPiFrame, ensurePiFrame, classifyCurrentAsk, isNonTaskStatusLikeText, isGenericPiFrameForCwd, trimFrameText, stripQuotedFocusaContext } from "./state.js";
import { pushDelta } from "./tools.js";

// §30 + §37.10: SSE connection for metacognitive + cross-surface events
let sseAbort: AbortController | null = null;
let sseReconnectTimer: ReturnType<typeof setTimeout> | null = null;


async function ensureLowConfidenceWorkpoint(reason: string): Promise<void> {
  if (!S.focusaAvailable) return;
  const mission = S.currentAsk?.text || S.activeFrameGoal || S.lastFocusSnapshot.intent || S.lastFocusSnapshot.currentFocus;
  const nextSlice = S.lastFocusSnapshot.currentFocus || S.activeFrameGoal || S.currentAsk?.text;
  if (!mission && !nextSlice) return;
  await focusaFetch("/workpoint/checkpoint", {
    method: "POST",
    body: JSON.stringify({
      mission: mission || "Pi session resume",
      next_slice: nextSlice || "Resume from low-confidence session state and immediately refine Workpoint.",
      checkpoint_reason: reason === "session_start" ? "session_start" : "session_resume",
      confidence: "low",
      canonical: true,
      promote: true,
      session_id: S.sessionFrameKey,
      source_turn_id: `pi-turn-${S.turnCount}`,
      action_intent: { action_type: "resume_workpoint", target_ref: S.activeFrameId || S.sessionFrameKey || "pi-session", verification_hooks: ["low-confidence checkpoint created because no active workpoint existed"], status: "needs_refinement" },
    }),
  }).catch(() => null);
}

async function refreshSessionWorkpointPacket(reason: string): Promise<void> {
  if (!S.focusaAvailable) return;
  try {
    const packet = await focusaFetch("/workpoint/resume", {
      method: "POST",
      body: JSON.stringify({ mode: "compact_prompt" }),
    });
    if (packet?.status === "completed") {
      S.activeWorkpointPacket = packet.resume_packet || packet;
      S.activeWorkpointSummary = packet.rendered_summary || packet.next_step_hint || "";
      focusaPost("/telemetry/trace", {
        event_type: "workpoint_resume_packet_loaded",
        payload: { reason, workpoint_id: packet.workpoint_id, canonical: packet.canonical },
      });
    }
  } catch { /* best effort */ }
}

function seedCurrentAskFromPersistedState(ctx: any, data: any) {
  const restoredAsk = data?.currentAsk;
  const cleanedRestoredAsk = stripQuotedFocusaContext(restoredAsk?.text || "");
  if (cleanedRestoredAsk && !isNonTaskStatusLikeText(cleanedRestoredAsk)) {
    S.currentAsk = {
      text: trimFrameText(cleanedRestoredAsk, 500),
      kind: restoredAsk.kind || classifyCurrentAsk(cleanedRestoredAsk),
      sourceTurnId: restoredAsk.sourceTurnId || "restored",
      updatedAt: restoredAsk.updatedAt || Date.now(),
    };
    if (data?.queryScope) S.queryScope = data.queryScope;
    return;
  }

  const goal = stripQuotedFocusaContext(String(data?.frameGoal || "").trim());
  const title = String(data?.frameTitle || "").trim();
  const cwd = ctx?.cwd || S.sessionCwd || process.cwd();
  if (!goal || isNonTaskStatusLikeText(goal) || isGenericPiFrameForCwd(cwd, title, goal)) return;
  if (!/^Pi (Task|Question|Correction): /.test(title)) return;

  S.currentAsk = {
    text: trimFrameText(goal, 500),
    kind: classifyCurrentAsk(goal),
    sourceTurnId: "restored-frame-goal",
    updatedAt: Date.now(),
  };
}

async function ensureActiveFrame(ctx: any, sessionId?: string) {
  return ensurePiFrame(ctx.cwd, sessionId, "pi-auto");
}

async function ensureFocusaSession(ctx: any) {
  const status = await focusaFetch("/status").catch(() => null);
  if (status?.session?.status === "active") return status.session;
  return focusaFetch("/session/start", {
    method: "POST",
    body: JSON.stringify({
      adapter_id: "pi",
      workspace_id: ctx.cwd || S.sessionCwd || "pi-workspace",
    }),
  });
}

function connectSSE() {
  if (sseReconnectTimer) {
    clearTimeout(sseReconnectTimer);
    sseReconnectTimer = null;
  }
  if (sseAbort) sseAbort.abort();
  if (!S.focusaAvailable) return;

  const base = S.cfg?.focusaApiBaseUrl || "http://127.0.0.1:8787/v1";
  const controller = new AbortController();
  sseAbort = controller;

  fetch(`${base}/events/stream`, { signal: controller.signal })
    .then(async (res) => {
      if (!res.body) return;
      const reader = res.body.getReader();
      const decoder = new TextDecoder();
      let buffer = "";
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split("\n");
        buffer = lines.pop() || "";
        for (const line of lines) {
          if (!line.startsWith("data: ")) continue;
          try {
            const evt = JSON.parse(line.slice(6));
            handleSSEEvent(evt);
          } catch { /* malformed SSE */ }
        }
      }
    })
    .catch(() => {
      if (controller.signal.aborted || !S.focusaAvailable) return;
      // §30: "If background work fails, the extension shows nothing (fail silent)"
      // Reconnect with backoff — use same exponential backoff as health checks (§11)
      sseReconnectTimer = setTimeout(() => {
        sseReconnectTimer = null;
        if (S.focusaAvailable) connectSSE();
      }, S.healthBackoffMs);
    });
}

// §30: Metacognitive awareness indicators + §37.10: Cross-surface events
function handleSSEEvent(evt: any) {
  switch (evt.type) {
    case "worker_started":
      S.lastMetacogEvent = "thinking...";
      break;
    case "extraction_complete":
      S.lastMetacogEvent = `extracted ${evt.count || "N"} items`;
      setTimeout(() => { S.lastMetacogEvent = ""; }, 5000);
      break;
    case "thesis_updated":
      S.lastMetacogEvent = "thesis updated";
      setTimeout(() => { S.lastMetacogEvent = ""; }, 5000);
      break;
    case "quality_flag":
      S.lastMetacogEvent = `⚠️ ${evt.message || "quality issue"}`;
      break;
    case "focus_state_updated":
      // §37.10: Cross-surface decision notification
      if (evt.source && evt.source !== "pi" && evt.decisions?.length) {
        S.pi?.exec("echo", [], { timeout: 1 }).catch(() => {}); // no-op to access ctx
      }
      break;
    default:
      break;
  }
}

export function registerSession(pi: ExtensionAPI) {
  // ── session_start — single merged handler ──────────────────────────────────
  pi.on("session_start", async (event, ctx) => {
    S.pi = pi;
    S.sessionStartTime = Date.now();
    S.turnCount = 0;
    S.seenFirstBeforeAgentStart = false; // Reset: inject directive on first before_agent_start only
    S.lastCompactDecision = "";
    S.compactResumePending = false;
    S.sessionFrameKey = (event as any).sessionId || `pi-${process.pid}-${Date.now()}`;
    S.activeWorkpointPacket = null;
    S.activeWorkpointSummary = "";
    S.sessionCwd = ctx.cwd;

    // §37.5: Check CLI flags FIRST
    if (pi.getFlag("--no-focusa")) {
      S.focusaAvailable = false;
      ctx.ui.setStatus("focusa", "⏸️ Focusa disabled");
      return;
    }
    if (pi.getFlag("--wbm")) S.wbmEnabled = true;

    // Health check
    await checkFocusa();

    // §36.4 + §33.5: Restore decisions from Pi session entries.
    // CRITICAL §33.5: Never restore activeFrameId from previous sessions — that
    // points to Wirebot/TEP frames and pollutes Pi sessions with stale Wirebot
    // state. Pi ALWAYS gets its own FRESH frame. Only WBM mode may reuse frames.
    const entries = (event as any).entries || (ctx as any).sessionManager?.getEntries?.() || [];
    for (let i = entries.length - 1; i >= 0; i--) {
      const e = entries[i];
      if ((e.customType === "focusa-wbm-state" || e.customType === "focusa-state") && e.data) {
        // §33.5 + §33.7: restore resumable session metadata and safe local shadow,
        // but do not blindly reuse stale frame identity outside WBM mode.
        S.localDecisions = e.data.decisions || [];
        S.turnCount = e.data.turnCount || 0;
        S.wbmEnabled = e.data.wbmEnabled || S.wbmEnabled;
        S.wbmNoCatalogue = e.data.wbmNoCatalogue || false;
        S.cataloguedDecisions = e.data.cataloguedDecisions || [];
        S.cataloguedFacts = e.data.cataloguedFacts || [];
        S.totalCompactions = e.data.totalCompactions || 0;
        S.lastCompactResumeKey = e.data.lastCompactResumeKey || "";
        S.lastCompactResumeAt = e.data.lastCompactResumeAt || 0;
        S.activeFrameTitle = e.data.frameTitle || "";
        S.activeFrameGoal = e.data.frameGoal || "";
        seedCurrentAskFromPersistedState(ctx, e.data);
        S.lastFocusSnapshot = {
          decisions: e.data.authoritativeDecisions || [],
          constraints: e.data.authoritativeConstraints || [],
          failures: e.data.authoritativeFailures || [],
          intent: e.data.intent || "",
          currentFocus: e.data.currentFocus || "",
        };
        S.activeWorkpointPacket = e.data.activeWorkpointPacket || null;
        S.activeWorkpointSummary = e.data.activeWorkpointSummary || "";
        if (e.data.sessionId) S.sessionFrameKey = e.data.sessionId;
        // Explicitly clear stale pollution — do NOT carry across sessions
        S.localConstraints = [];
        S.localFailures = [];
        break;
      }
    }
    // §33.5: Always NULL out activeFrameId — force-push fresh Pi frame.
    // This prevents Wirebot/TEP frame state from leaking into Pi sessions.
    // WBM mode may override this via --wbm flag above.
    if (!S.wbmEnabled) S.activeFrameId = null;

    if (!S.focusaAvailable) {
      ctx.ui.setStatus("focusa", "📡 Focusa offline");
      return;
    }

    await ensureFocusaSession(ctx);
    await ensureActiveFrame(ctx, (event as any).sessionId || `pi-session-${Date.now()}`);
    await refreshSessionWorkpointPacket("session_start");
    if (!S.activeWorkpointPacket) {
      await ensureLowConfidenceWorkpoint("session_start");
      await refreshSessionWorkpointPacket("session_start_low_confidence");
    }

    // §35.8: Session name sync from Pi's scoped focus frame, never global active frame
    const data = await getFocusState().catch(() => null);
    if (data?.frame?.title) {
      S.activeFrameTitle = data.frame.title;
      S.activeFrameGoal = data.frame.goal || S.activeFrameGoal;
      pi.setSessionName(data.frame.title);
    } else if (S.activeFrameTitle) {
      pi.setSessionName(S.activeFrameTitle);
    }

    // §37.9: Context Core activity signal + wb me --set pi_active
    focusaPost("/telemetry/activity", { surface: "pi", event: "session_start", cwd: ctx.cwd });
    pi.exec("wb", ["me", "--set", "pi_active=true"]).catch(() => {});

    // §30 + §37.10: Start SSE connection for metacognitive + cross-surface events
    connectSSE();

    // Keep Pi footer task label fresh between explicit commands.
    // Default is event-driven (no periodic polling); polling can be enabled explicitly.
    if (S.footerSyncInterval) clearInterval(S.footerSyncInterval);
    S.footerSyncInterval = null;
    const bridgeSyncMode = S.cfg?.bridgeSyncMode || "event-driven";
    if (bridgeSyncMode === "polling") {
      const footerRefreshMs = Math.max(5_000, S.cfg?.bridgePollMs || 15_000);
      let footerSyncInFlight = false;
      S.footerSyncInterval = setInterval(async () => {
        if (!S.focusaAvailable || footerSyncInFlight) return;
        footerSyncInFlight = true;
        try {
          await getFocusState().catch(() => null);
          if (S.activeFrameTitle) pi.setSessionName(S.activeFrameTitle);
        } finally {
          footerSyncInFlight = false;
        }
      }, footerRefreshMs);
    }

    // Debounce transient health blips to reduce false "offline" warnings.
    // Require consecutive failures before disabling tools.
    const offlineWarnThreshold = 2;
    let outageMode = false;

    // §38.3 + §11: Health check with exponential backoff via recursive setTimeout
    function scheduleHealthCheck() {
      if (S.healthInterval) clearTimeout(S.healthInterval);
      S.healthInterval = setTimeout(async () => {
        await checkFocusa();

        if (!S.focusaAvailable && !outageMode && S.healthFailCount >= offlineWarnThreshold) {
          // Confirmed outage (not single blip) — preserve tool availability, enter holdover, and kickstart daemon.
          ctx.ui.setStatus("focusa", "🛟 Focusa holdover · restarting");
          ctx.ui.notify(`Focusa daemon unavailable (${S.healthFailCount} checks) — holdover active; kickstarting daemon without restarting session`, "warning");
          if (sseAbort) { sseAbort.abort(); sseAbort = null; }
          outageMode = true;
          const recovered = await kickstartFocusaDaemon("session_health_check");
          if (recovered) {
            await checkFocusa();
            ctx.ui.notify("Focusa daemon kickstarted — session preserved", "info");
          }
        } else if (!S.focusaAvailable && outageMode) {
          ctx.ui.setStatus("focusa", "🛟 Focusa holdover · retrying");
          await kickstartFocusaDaemon("session_health_retry");
        } else if (S.focusaAvailable && outageMode) {
          // Came back — reconnect SSE and reconcile holdover state; tools were never disabled.
          ctx.ui.setStatus("focusa", S.wbmEnabled ? "🤖 Focusa WBM" : "🧭 Focusa");
          ctx.ui.notify("Focusa daemon reconnected — holdover reconciled; session preserved", "info");
          await ensureActiveFrame(ctx);
          connectSSE();

          // §11/§25.7: Soft resync — reconcile local shadow with Focusa on reconnect
          if (S.activeFrameId) {
            // Push any local shadow accumulated during outage
            if (S.localDecisions.length || S.localConstraints.length || S.localFailures.length) {
              await pushDelta({
                decisions: S.localDecisions.slice(-10),
                constraints: S.localConstraints.slice(-10),
                failures: S.localFailures.slice(-5),
                notes: ["Reconciled after Focusa outage"],
              }).catch(() => null);
            }
            // Fetch fresh state + recent candidates
            const data = await getFocusState();
            if (data?.fs) {
              ctx.ui.notify(`Resync complete — ${data.fs.decisions?.length || 0} decisions, ${data.fs.constraints?.length || 0} constraints`, "info");
            }
            // Fetch recent Focus Gate candidates
            focusaFetch("/focus-gate/candidates?limit=5").then((r: any) => {
              if (r?.candidates?.length) {
                ctx.ui.notify(`Focus Gate: ${r.candidates.length} pending candidates`, "info");
              }
            }).catch(() => {});
          }
          outageMode = false;
        }

        // Schedule next check with (possibly updated) backoff interval
        scheduleHealthCheck();
      }, S.healthBackoffMs);
    }
    scheduleHealthCheck();

    ctx.ui.setStatus("focusa", S.wbmEnabled ? "🤖 Focusa WBM" : "🧭 Focusa");
  });

  // ── session_shutdown — single handler (§33.8, §34.2A, §37.9) ──────────────
  pi.on("session_shutdown", async (_event, _ctx) => {
    await persistAuthoritativeState();

    // §37.9: Tell Context Core Pi is no longer active
    S.pi?.exec("wb", ["me", "--set", "pi_active=false"]).catch(() => {});

    // Close SSE
    if (sseReconnectTimer) { clearTimeout(sseReconnectTimer); sseReconnectTimer = null; }
    if (sseAbort) { sseAbort.abort(); sseAbort = null; }

    if (S.focusaAvailable) {
      await focusaFetch("/session/close", {
        method: "POST",
        body: JSON.stringify({ reason: "pi_session_shutdown" }),
      });
    }
    if (S.focusaAvailable) {
      focusaPost("/instance/disconnect", { instance_id: `pi-${process.pid}` });
      focusaPost("/telemetry/activity", { surface: "pi", event: "session_shutdown" });
    }
    if (S.healthInterval) { clearInterval(S.healthInterval); S.healthInterval = null; }
    if (S.footerSyncInterval) { clearInterval(S.footerSyncInterval); S.footerSyncInterval = null; }
  });

  // ── session_before_switch (§37.7) ─────────────────────────────────────────
  pi.on("session_before_switch", async (_event, _ctx) => {
    await persistAuthoritativeState();
    if (S.focusaAvailable && S.activeFrameId) {
      await pushDelta({
        decisions: S.localDecisions.slice(-5),
        constraints: S.localConstraints.slice(-5),
      }).catch(() => null);
    }
    if (S.focusaAvailable) {
      await focusaFetch("/session/close", {
        method: "POST",
        body: JSON.stringify({ reason: "pi_session_switch" }),
      });
    }
  });

  // ── session_switch (§37.7) ────────────────────────────────────────────────
  pi.on("session_switch", async (event, ctx) => {
    S.localDecisions = []; S.localConstraints = []; S.localFailures = [];
    S.lastFocusSnapshot = { decisions: [], constraints: [], failures: [], intent: "", currentFocus: "" };
    S.turnCount = 0; S.cataloguedDecisions = []; S.cataloguedFacts = [];
    S.sessionFrameKey = (event as any).sessionId || `pi-${process.pid}-${Date.now()}`;
    S.activeWorkpointPacket = null;
    S.activeWorkpointSummary = "";
    S.sessionCwd = ctx.cwd;
    S.activeFrameTitle = ""; S.activeFrameGoal = "";
    S.fileEditCounts = {}; S.compilationErrors = []; S.longSessionSignaled = false;
    S.totalCompactions = 0; S.lastCompactResumeKey = ""; S.lastCompactResumeAt = 0; S.wbmNoCatalogue = false;

    const switchEntries = (event as any).entries || (ctx as any).sessionManager?.getEntries?.() || [];
    S.forkSuggested = false;
    for (let i = switchEntries.length - 1; i >= 0; i--) {
      if ((switchEntries[i].customType === "focusa-wbm-state" || switchEntries[i].customType === "focusa-state") && switchEntries[i].data) {
        const d = switchEntries[i].data;
        S.localDecisions = d.decisions || [];
        S.localConstraints = d.constraints || [];
        S.localFailures = d.failures || [];
        S.turnCount = d.turnCount || 0;
        S.wbmEnabled = d.wbmEnabled || false;
        S.wbmNoCatalogue = d.wbmNoCatalogue || false;
        S.totalCompactions = d.totalCompactions || 0;
        S.lastCompactResumeKey = d.lastCompactResumeKey || "";
        S.lastCompactResumeAt = d.lastCompactResumeAt || 0;
        S.activeFrameTitle = d.frameTitle || "";
        S.activeFrameGoal = d.frameGoal || "";
        seedCurrentAskFromPersistedState(ctx, d);
        S.lastFocusSnapshot = {
          decisions: d.authoritativeDecisions || [],
          constraints: d.authoritativeConstraints || [],
          failures: d.authoritativeFailures || [],
          intent: d.intent || "",
          currentFocus: d.currentFocus || "",
        };
        S.activeWorkpointPacket = d.activeWorkpointPacket || null;
        S.activeWorkpointSummary = d.activeWorkpointSummary || "";
        if (d.sessionId) S.sessionFrameKey = d.sessionId;
        break;
      }
    }

    if (!S.wbmEnabled) S.activeFrameId = null;
    if (S.focusaAvailable) {
      await ensureFocusaSession(ctx);
      await ensureActiveFrame(ctx, (event as any).sessionId || "unknown");
      await refreshSessionWorkpointPacket("session_switch");
      if (!S.activeWorkpointPacket) {
        await ensureLowConfidenceWorkpoint("session_resume");
        await refreshSessionWorkpointPacket("session_switch_low_confidence");
      }
    }
  });

  // ── session_before_fork (§36.5) ───────────────────────────────────────────
  pi.on("session_before_fork", async (_event, _ctx) => {
    if (S.focusaAvailable) {
      await focusaFetch("/workpoint/checkpoint", {
        method: "POST",
        body: JSON.stringify({
          mission: S.currentAsk?.text || S.activeFrameGoal || S.lastFocusSnapshot.intent || "Pi fork boundary",
          next_slice: S.lastFocusSnapshot.currentFocus || "Resume from fork WorkpointResumePacket.",
          checkpoint_reason: "fork",
          canonical: true,
          promote: true,
          source_turn_id: `pi-turn-${S.turnCount}`,
          action_intent: { action_type: "resume_workpoint", target_ref: S.activeFrameId || "pi-fork", verification_hooks: ["fork refreshes workpoint"], status: "ready" },
        }),
      }).catch(() => null);
      await refreshSessionWorkpointPacket("fork");
    }
    await persistAuthoritativeState();
    if (S.focusaAvailable && S.activeFrameId) {
      focusaPost("/focus/update", {
        frame_id: S.activeFrameId, turn_id: `pi-turn-${S.turnCount}`,
        delta: { meta: { event: "fork", timestamp: Date.now() } },
      });
    }
  });

  // ── session_fork (§36.5) ──────────────────────────────────────────────────
  pi.on("session_fork", async (_event, _ctx) => {
    // §36.5: Take Focusa snapshot of branch point before fork diverges
    if (S.focusaAvailable && S.activeFrameId) {
      focusaPost("/focus/update", {
        frame_id: S.activeFrameId,
        turn_id: `pi-turn-${S.turnCount}`,
        delta: { meta: { event: "fork", turn_count: S.turnCount, decisions_count: S.localDecisions.length } },
      });
    }
  });

  // ── session_before_tree (§36.5) ───────────────────────────────────────────
  pi.on("session_before_tree", async (_event, _ctx) => {
    await persistAuthoritativeState();
  });
}
