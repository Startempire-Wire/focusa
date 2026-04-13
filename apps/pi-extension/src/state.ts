// Shared state, helpers, types for focusa-pi-bridge
// Spec: docs/44-pi-focusa-integration-spec.md

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import type { FocusaConfig } from "./config.js";

// ── Mutable shared state ─────────────────────────────────────────────────────
export const S = {
  pi: null as ExtensionAPI | null,
  cfg: null as FocusaConfig | null,
  focusaAvailable: false,
  activeFrameId: null as string | null,
  activeFramePromise: null as Promise<string | null> | null,
  sessionFrameKey: "" as string,
  sessionCwd: "" as string,
  wbmEnabled: false,
  wbmDeep: false,
  wbmNoCatalogue: false,       // §29 --no-catalogue flag
  turnCount: 0,
  // Local shadow (§35.4)
  localDecisions: [] as string[],
  localConstraints: [] as string[],
  localFailures: [] as string[],
  // Compaction tier (§20)
  lastCompactTime: 0,
  compactsThisHour: 0,
  turnsSinceCompact: 0,
  compactHourStart: Date.now(),
  activeContextWindow: 200_000,  // claude-opus-4-6 has 200K window; updated on model_select events
  currentTier: "" as "" | "warn" | "auto" | "hard", // §10.4 tier badge
  currentContextPct: null as number | null,
  // Streaming delta (§36.1)
  lastStreamLen: 0,
  // Auto-resume dedup: set when compaction fires, cleared after continuation sent
  compactResumePending: false,
  // Post-compaction: save last decision for steer message (cleared after localDecisions trim)
  lastCompactDecision: "",
  // First-turn guard: only inject behavioral directive once per session, not on every before_agent_start
  seenFirstBeforeAgentStart: false,
  // ECS handle registry: kind -> id -> { content, stored_at }
  ecsRegistry: {} as Record<string, Record<string, { content: string; storedAt: number }>>,
  // Tool usage batching (§33.4)
  toolUsageBatch: [] as string[],
  // Intuition signals (§36.2, §34.2D)
  compilationErrors: [] as number[],
  fileEditCounts: {} as Record<string, number>,
  // Session timing
  sessionStartTime: Date.now(),
  longSessionSignaled: false,
  // WBM cataloguing (§29)
  cataloguedDecisions: [] as string[],
  cataloguedFacts: [] as string[],
  // Health (§38.3)
  healthInterval: null as ReturnType<typeof setInterval> | null,
  healthBackoffMs: 30_000,     // §11 exponential backoff
  healthFailCount: 0,
  // Outage audit (§11)
  outageStart: null as number | null,
  // §30 metacognitive indicators
  lastMetacogEvent: "",
  // Total compactions for handoff suggestion (§18 autoSuggestHandoffAfterNCompactions)
  totalCompactions: 0,
  // Fork suggestion dedup (§18 autoSuggestForkPct)
  forkSuggested: false,
};

// ── HTTP helper ──────────────────────────────────────────────────────────────
export async function focusaFetch(path: string, opts: RequestInit = {}): Promise<any> {
  const timeout = S.cfg?.focusaApiTimeoutMs || 5000;
  const base = S.cfg?.focusaApiBaseUrl || "http://127.0.0.1:8787/v1";
  const token = S.cfg?.focusaToken || "";
  const ac = new AbortController();
  const t = setTimeout(() => ac.abort(), timeout);
  try {
    const r = await fetch(`${base}${path}`, {
      ...opts,
      headers: {
        "Content-Type": "application/json",
        ...(token ? { Authorization: `Bearer ${token}` } : {}),
        ...(opts.headers as Record<string, string> || {}),
      },
      signal: ac.signal,
    });
    if (!r.ok) return null;
    return await r.json();
  } catch { return null; }
  finally { clearTimeout(t); }
}

// Fire-and-forget variant
export function focusaPost(path: string, body: any): void {
  focusaFetch(path, { method: "POST", body: JSON.stringify(body) }).catch(() => {});
}

// ── Health check (§38.3, §11 backoff) ────────────────────────────────────────
export async function checkFocusa(): Promise<boolean> {
  const h = await focusaFetch("/health");
  const wasAvailable = S.focusaAvailable;
  S.focusaAvailable = h?.ok === true;

  if (S.focusaAvailable) {
    S.healthFailCount = 0;
    S.healthBackoffMs = 30_000;
    // §11: Outage recovery — record audit event
    if (!wasAvailable && S.outageStart) {
      const durationMs = Date.now() - S.outageStart;
      focusaPost("/telemetry/event", {
        event: "outage_recovered",
        surface: "pi",
        duration_ms: durationMs,
        missed_turns: S.turnCount,
      });
      S.outageStart = null;
    }
  } else {
    S.healthFailCount++;
    // §11: Exponential backoff (30s → 60s → 120s → max 300s)
    S.healthBackoffMs = Math.min(30_000 * Math.pow(2, S.healthFailCount - 1), 300_000);
    // §11: Record outage start
    if (wasAvailable && !S.outageStart) {
      S.outageStart = Date.now();
      // Fire-and-forget — may fail since Focusa is down
      focusaFetch("/telemetry/event", {
        method: "POST",
        body: JSON.stringify({ event: "outage_started", surface: "pi", turn_count: S.turnCount }),
      }).catch(() => {});
    }
  }
  return S.focusaAvailable;
}

// ── Extract text from TextContent[] | string ─────────────────────────────────
export function extractText(content: any): string {
  if (typeof content === "string") return content;
  if (Array.isArray(content)) return content.map((c: any) => c.text || "").join("");
  return String(content || "");
}

// ── Get Focus State from Focusa scoped to Pi's own frame (§33.5 isolation) ──
// CRITICAL: Never use Focusa's global active_frame_id — that belongs to Wirebot.
// Pi sessions must only read their own frame. If Pi has no frame, return empty.
export async function getFocusState(): Promise<{ frame: any; fs: any; stack: any } | null> {
  // §33.5: Strict scoping — if Pi has no frame, never leak global active frame
  if (!S.activeFrameId) return null;

  const stack = await focusaFetch("/focus/stack");
  if (!stack?.stack?.frames?.length) return null;

  // §33.5: Find Pi's frame by S.activeFrameId — never fall back to global active
  const frame = stack.stack.frames.find((f: any) => f.id === S.activeFrameId) || null;
  if (!frame) return null;

  // Never sync Pi's scoped frame from Focusa global active_frame_id.
  return { frame, fs: frame?.focus_state || {}, stack };
}

export async function createPiFrame(cwd: string, source = "pi-auto"): Promise<string | null> {
  S.sessionCwd = cwd;
  const projectName = cwd.split("/").filter(Boolean).pop() || "root";
  const title = `Pi: ${projectName}`;
  const goal = `Work on ${projectName}`;
  const sessionKey = S.sessionFrameKey || `pi-${process.pid}-${Date.now()}`;
  S.sessionFrameKey = sessionKey;
  const beadsIssueId = `pi-session-${projectName}-${sessionKey}`;
  const tags = ["pi", projectName, source, sessionKey];

  try {
    const r = await focusaFetch("/focus/push", {
      method: "POST",
      body: JSON.stringify({
        title,
        goal,
        beads_issue_id: beadsIssueId,
        constraints: [],
        tags,
      }),
    });
    if (r?.frame_id) {
      S.activeFrameId = r.frame_id;
      return r.frame_id;
    }

    for (let i = 0; i < 10; i++) {
      await new Promise((resolve) => setTimeout(resolve, 300));
      const stack = await focusaFetch("/focus/stack");
      const frames = stack?.stack?.frames || [];
      const match = [...frames].reverse().find((f: any) =>
        f.title === title &&
        f.beads_issue_id === beadsIssueId &&
        Array.isArray(f.tags) &&
        f.tags.includes(sessionKey));
      if (match?.id) {
        S.activeFrameId = match.id;
        return match.id;
      }
    }
  } catch {}
  return null;
}

// ── Build compact instructions with local shadow (§33.10) ────────────────────
export function buildCompactInstructions(prefix: string): string {
  const base = S.cfg?.compactInstructions || "Preserve intent, decisions, constraints, next_steps, failures.";
  const parts = [prefix, "\n" + base];
  if (S.localDecisions.length) parts.push(`Decisions:\n${S.localDecisions.map(d => `- ${d}`).join("\n")}`);
  if (S.localConstraints.length) parts.push(`Constraints:\n${S.localConstraints.map(c => `- ${c}`).join("\n")}`);
  if (S.localFailures.length) parts.push(`Failures:\n${S.localFailures.map(f => `- ${f}`).join("\n")}`);
  return parts.join("\n");
}

// ── wb CLI with HTTP fallback (§38.2) ────────────────────────────────────────
export async function wbExec(args: string[], fallbackUrl?: string, fallbackBody?: any): Promise<any> {
  if (S.pi) {
    try {
      const r = await S.pi.exec("wb", args, { timeout: 5000 });
      if (r.code === 0) {
        try { return JSON.parse(r.stdout); } catch { return true; }
      }
    } catch { /* fall through */ }
  }
  if (fallbackUrl) {
    const token = S.cfg?.scoreboardToken || "";
    try {
      const r = await fetch(fallbackUrl, {
        method: "POST",
        headers: { "Content-Type": "application/json", ...(token ? { Authorization: `Bearer ${token}` } : {}) },
        body: JSON.stringify(fallbackBody),
        signal: AbortSignal.timeout(5000),
      });
      return r.ok ? await r.json().catch(() => true) : null;
    } catch { return null; }
  }
  return null;
}

// ── Persist Focusa state to Pi session (§33.7) ──────────────────────────────
export async function ensurePiFrame(cwd?: string, sessionId?: string, source = "pi-auto"): Promise<string | null> {
  if (!S.focusaAvailable || S.activeFrameId) return S.activeFrameId;
  if (S.activeFramePromise) return await S.activeFramePromise;

  const resolvedCwd = cwd || S.sessionCwd || process.cwd();
  S.sessionCwd = resolvedCwd;

  S.activeFramePromise = (async () => {
    focusaPost("/instance/connect", {
      instance_id: `pi-${process.pid}`,
      surface: "pi",
      session_id: sessionId || S.sessionFrameKey || `pi-session-${Date.now()}`,
      cwd: resolvedCwd,
    });

    const frameId = await createPiFrame(resolvedCwd, source);
    if (frameId) persistState();
    return frameId;
  })();

  try {
    return await S.activeFramePromise;
  } finally {
    S.activeFramePromise = null;
  }
}

export function persistState(): void {
  S.pi?.appendEntry("focusa-state", {
    frameId: S.activeFrameId,
    decisions: S.localDecisions,
    constraints: S.localConstraints,
    failures: S.localFailures,
    turnCount: S.turnCount,
    wbmEnabled: S.wbmEnabled,
    wbmNoCatalogue: S.wbmNoCatalogue,
    cataloguedDecisions: S.cataloguedDecisions,
    cataloguedFacts: S.cataloguedFacts,
    totalCompactions: S.totalCompactions,
    timestamp: Date.now(),
  });
}

// ── Estimate tokens from bytes (§7.4) ────────────────────────────────────────
export function estimateTokens(text: string): number {
  return Math.ceil(text.length / 4);
}

// ── ECS artifact registry (§7.4, §33.3) ─────────────────────────────────────
// Handles are [HANDLE:<kind>:<id>] refs. After compaction Focusa may be slow.
// Store artifacts locally so LLM can resolve handles even if Focusa is temporarily
// unavailable. Re-hydrated from Focusa on reconnect.

let _handleCounter = 0;

export function storeEcsArtifact(kind: string, content: string): string {
  const id = `local-${Date.now()}-${++_handleCounter}`;
  if (!S.ecsRegistry[kind]) S.ecsRegistry[kind] = {};
  S.ecsRegistry[kind][id] = { content, storedAt: Date.now() };
  return id;
}

export function getEcsArtifact(kind: string, id: string): string | null {
  return S.ecsRegistry[kind]?.[id]?.content ?? null;
}

export function extractHandles(text: string): Array<{ kind: string; id: string }> {
  const handles: Array<{ kind: string; id: string }> = [];
  const re = /\[HANDLE:([^:]+):([^\]]+)\]/g;
  let m;
  while ((m = re.exec(text)) !== null) handles.push({ kind: m[1], id: m[2] });
  return handles;
}
