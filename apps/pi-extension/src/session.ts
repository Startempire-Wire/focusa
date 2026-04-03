// Session lifecycle events — ONE handler per event type (merged)
// Spec: §11 (outage audit + backoff), §30 (SSE metacog), §34.2A (instance),
//        §35.1 (auto-frame), §36.4 (resume), §36.5 (fork/tree), §37.5 (flags),
//        §35.8 (session name), §37.9 (Context Core), §37.10 (cross-surface SSE),
//        §38.3 (health toggle)

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { S, focusaFetch, focusaPost, checkFocusa, persistState, getFocusState } from "./state.js";

// §30 + §37.10: SSE connection for metacognitive + cross-surface events
let sseAbort: AbortController | null = null;

function connectSSE() {
  if (sseAbort) sseAbort.abort();
  if (!S.focusaAvailable) return;

  const base = S.cfg?.focusaApiBaseUrl || "http://127.0.0.1:8787/v1";
  sseAbort = new AbortController();

  fetch(`${base}/events/stream`, { signal: sseAbort.signal })
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
      // §30: "If background work fails, the extension shows nothing (fail silent)"
      // Reconnect with backoff — use same exponential backoff as health checks (§11)
      setTimeout(() => { if (S.focusaAvailable) connectSSE(); }, S.healthBackoffMs);
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

    // §37.5: Check CLI flags FIRST
    if (pi.getFlag("--no-focusa")) {
      S.focusaAvailable = false;
      ctx.ui.setStatus("focusa", "🧠 Focusa [disabled]");
      return;
    }
    if (pi.getFlag("--wbm")) S.wbmEnabled = true;

    // Health check
    await checkFocusa();

    // §36.4: Restore state from Pi session entries (resume/restart)
    // Try event.entries first, fall back to ctx.sessionManager.getEntries()
    const entries = (event as any).entries || (ctx as any).sessionManager?.getEntries?.() || [];
    for (let i = entries.length - 1; i >= 0; i--) {
      const e = entries[i];
      if (e.customType === "focusa-state" && e.data) {
        S.activeFrameId = e.data.frameId ?? S.activeFrameId;
        S.localDecisions = e.data.decisions || [];
        S.localConstraints = e.data.constraints || [];
        S.localFailures = e.data.failures || [];
        S.turnCount = e.data.turnCount || 0;
        S.wbmEnabled = e.data.wbmEnabled || S.wbmEnabled;
        S.wbmNoCatalogue = e.data.wbmNoCatalogue || false;
        S.cataloguedDecisions = e.data.cataloguedDecisions || [];
        S.cataloguedFacts = e.data.cataloguedFacts || [];
        S.totalCompactions = e.data.totalCompactions || 0;
        break;
      }
    }

    if (!S.focusaAvailable) {
      ctx.ui.setStatus("focusa", "🧠 Focusa [offline]");
      return;
    }

    // §34.2A: Register instance
    focusaPost("/instance/connect", {
      instance_id: `pi-${process.pid}`,
      surface: "pi",
      session_id: (event as any).sessionId || `pi-session-${Date.now()}`,
      cwd: ctx.cwd,
    });

    // §35.1: Auto-frame push
    if (!S.activeFrameId) {
      const r = await focusaFetch("/focus/push", {
        method: "POST",
        body: JSON.stringify({ title: `Pi session in ${ctx.cwd}`, source: "pi-auto" }),
      });
      if (r?.frame_id) S.activeFrameId = r.frame_id;
    }

    // §35.8: Session name sync from focus frame
    const data = await focusaFetch("/focus/stack");
    if (data?.stack?.frames?.length) {
      const active = data.stack.frames.find((f: any) => f.id === data.active_frame_id);
      if (active?.title) {
        pi.setSessionName(active.title);
      }
    }

    // §37.9: Context Core activity signal + wb me --set pi_active
    focusaPost("/telemetry/activity", { surface: "pi", event: "session_start", cwd: ctx.cwd });
    pi.exec("wb", ["me", "--set", "pi_active=true"]).catch(() => {});

    // §30 + §37.10: Start SSE connection for metacognitive + cross-surface events
    connectSSE();

    // §38.3 + §11: Health check with exponential backoff via recursive setTimeout
    function scheduleHealthCheck() {
      if (S.healthInterval) clearTimeout(S.healthInterval);
      S.healthInterval = setTimeout(async () => {
        const wasAvailable = S.focusaAvailable;
        await checkFocusa();

      if (wasAvailable && !S.focusaAvailable) {
        // Went down — disable tools, disconnect SSE
        const active = pi.getActiveTools();
        const filtered = active.filter((t: any) =>
          !["focusa_decide", "focusa_constraint", "focusa_failure"].includes(typeof t === "string" ? t : t.name));
        pi.setActiveTools(filtered.map((t: any) => typeof t === "string" ? t : t.name));
        ctx.ui.setStatus("focusa", "🧠 Focusa [offline]");
        ctx.ui.notify("Focusa daemon went offline — tools disabled", "warning");
        if (sseAbort) { sseAbort.abort(); sseAbort = null; }
      } else if (!wasAvailable && S.focusaAvailable) {
        // Came back — re-enable tools, reconnect SSE
        const all = pi.getAllTools();
        const active = pi.getActiveTools();
        const focusaTools = all.filter((t: any) =>
          ["focusa_decide", "focusa_constraint", "focusa_failure"].includes(t.name));
        const names = [...new Set([...active.map((t: any) => typeof t === "string" ? t : t.name), ...focusaTools.map((t: any) => t.name)])];
        pi.setActiveTools(names);
        ctx.ui.setStatus("focusa", S.wbmEnabled ? "🧠 Focusa [WBM]" : "🧠 Focusa");
        ctx.ui.notify("Focusa daemon reconnected — tools re-enabled", "info");
        connectSSE();

        // §11/§25.7: Soft resync — reconcile local shadow with Focusa on reconnect
        if (S.activeFrameId) {
          // Push any local shadow accumulated during outage
          if (S.localDecisions.length || S.localConstraints.length || S.localFailures.length) {
            await focusaFetch("/focus/update", {
              method: "POST",
              body: JSON.stringify({
                frame_id: S.activeFrameId,
                turn_id: `pi-turn-${S.turnCount}`,
                delta: {
                  decisions: S.localDecisions.slice(-10),
                  constraints: S.localConstraints.slice(-10),
                  failures: S.localFailures.slice(-5),
                  notes: ["Reconciled after Focusa outage"],
                },
              }),
            });
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
      }
        // Schedule next check with (possibly updated) backoff interval
        scheduleHealthCheck();
      }, S.healthBackoffMs);
    }
    scheduleHealthCheck();

    ctx.ui.setStatus("focusa", S.wbmEnabled ? "🧠 Focusa [WBM]" : "🧠 Focusa");
  });

  // ── session_shutdown — single handler (§33.8, §34.2A, §37.9) ──────────────
  pi.on("session_shutdown", async (_event, _ctx) => {
    persistState();

    // §37.9: Tell Context Core Pi is no longer active
    S.pi?.exec("wb", ["me", "--set", "pi_active=false"]).catch(() => {});

    // Close SSE
    if (sseAbort) { sseAbort.abort(); sseAbort = null; }

    if (S.focusaAvailable && S.activeFrameId) {
      await focusaFetch("/session/close", {
        method: "POST",
        body: JSON.stringify({ frame_id: S.activeFrameId, turn_count: S.turnCount }),
      });
    }
    if (S.focusaAvailable) {
      focusaPost("/instance/disconnect", { instance_id: `pi-${process.pid}` });
      focusaPost("/telemetry/activity", { surface: "pi", event: "session_shutdown" });
    }
    if (S.healthInterval) { clearInterval(S.healthInterval); S.healthInterval = null; }
  });

  // ── session_before_switch (§37.7) ─────────────────────────────────────────
  pi.on("session_before_switch", async (_event, _ctx) => {
    persistState();
    if (S.focusaAvailable && S.activeFrameId) {
      await focusaFetch("/focus/update", {
        method: "POST",
        body: JSON.stringify({
          frame_id: S.activeFrameId, turn_id: `pi-turn-${S.turnCount}`,
          delta: { decisions: S.localDecisions.slice(-5), constraints: S.localConstraints.slice(-5) },
        }),
      });
    }
  });

  // ── session_switch (§37.7) ────────────────────────────────────────────────
  pi.on("session_switch", async (event, ctx) => {
    S.localDecisions = []; S.localConstraints = []; S.localFailures = [];
    S.turnCount = 0; S.cataloguedDecisions = []; S.cataloguedFacts = [];
    S.fileEditCounts = {}; S.compilationErrors = []; S.longSessionSignaled = false;
    S.totalCompactions = 0; S.wbmNoCatalogue = false;

    const switchEntries = (event as any).entries || (ctx as any).sessionManager?.getEntries?.() || [];
    S.forkSuggested = false;
    for (let i = switchEntries.length - 1; i >= 0; i--) {
      if (switchEntries[i].customType === "focusa-state" && switchEntries[i].data) {
        const d = switchEntries[i].data;
        S.activeFrameId = d.frameId ?? null;
        S.localDecisions = d.decisions || [];
        S.localConstraints = d.constraints || [];
        S.localFailures = d.failures || [];
        S.turnCount = d.turnCount || 0;
        S.wbmEnabled = d.wbmEnabled || false;
        S.wbmNoCatalogue = d.wbmNoCatalogue || false;
        S.totalCompactions = d.totalCompactions || 0;
        break;
      }
    }

    if (S.focusaAvailable) {
      focusaPost("/instance/connect", {
        instance_id: `pi-${process.pid}`, surface: "pi",
        session_id: (event as any).sessionId || "unknown",
      });
    }
  });

  // ── session_before_fork (§36.5) ───────────────────────────────────────────
  pi.on("session_before_fork", async (_event, _ctx) => {
    persistState();
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
    persistState();
  });
}
