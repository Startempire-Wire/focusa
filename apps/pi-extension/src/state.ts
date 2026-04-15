// Shared state, helpers, types for focusa-pi-bridge
// Spec: docs/44-pi-focusa-integration-spec.md

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import type { FocusaConfig } from "./config.js";

export type PiCurrentAskKind = "question" | "instruction" | "correction" | "meta" | "unknown";

export interface PiCurrentAsk {
  text: string;
  kind: PiCurrentAskKind;
  sourceTurnId: string;
  updatedAt: number;
}

export interface PiQueryScope {
  scopeKind: "fresh_question" | "mission_carryover" | "correction" | "meta";
  carryoverPolicy: "suppress_by_default" | "allow_if_relevant" | "prefer_reset";
  sourceTurnId: string;
  updatedAt: number;
}

export interface PiExcludedContext {
  labels: string[];
  reason: "budget_truncation" | "fresh_scope" | "correction_reset" | "irrelevance" | "none";
  sourceTurnId: string;
  updatedAt: number;
}

export interface PiFocusSelection {
  items: string[];
  excluded: string[];
  scores: Array<{ value: string; score: number }>;
}

export interface PiRankedItem {
  value: string;
  updatedAt?: string | null;
  pinned?: boolean;
}

export interface PiSliceSection {
  key: string;
  text: string;
  include: boolean;
  selectedCount?: number;
  excludedCount?: number;
  priority?: number;
  relevanceScore?: number;
}

// ── Mutable shared state ─────────────────────────────────────────────────────
export const S = {
  pi: null as ExtensionAPI | null,
  cfg: null as FocusaConfig | null,
  focusaAvailable: false,
  activeFrameId: null as string | null,
  activeFramePromise: null as Promise<string | null> | null,
  activeFrameTitle: "" as string,
  activeFrameGoal: "" as string,
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
  // Transient routing metadata — truthful bridge toward CurrentAsk/QueryScope work.
  currentAsk: null as PiCurrentAsk | null,
  queryScope: null as PiQueryScope | null,
  excludedContext: null as PiExcludedContext | null,
  lastFocusSnapshot: {
    decisions: [] as string[],
    constraints: [] as string[],
    failures: [] as string[],
    intent: "" as string,
    currentFocus: "" as string,
  },
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
  // Footer/session-title sync cadence (keeps Pi footer task label fresh between commands)
  footerSyncInterval: null as ReturnType<typeof setInterval> | null,
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

function hasQuotedFocusaPayload(text: string): boolean {
  return /\[focusa-context\]|#\s*focusa context|rendered live from focusa-pi-bridge current state\.?|current focus frame:|\bgoal:\b/i.test(String(text || ""));
}

function isContaminatedFrameIdentity(frame: any): boolean {
  const title = String(frame?.title || "");
  const goal = String(frame?.goal || "");
  return hasQuotedFocusaPayload(title) || hasQuotedFocusaPayload(goal);
}

function isFocusaPayloadWrapperText(text: string): boolean {
  const normalized = String(text || "").replace(/\s+/g, " ").trim().toLowerCase();
  if (!normalized) return false;
  if (/^(restarted again,?\s*)?(still wrong|wrong|not true|this is|this output|this context|look|see|after restart|same issue|again)[:\s]*$/.test(normalized)) return true;
  if (/^(why|how|what)[:\s]*$/.test(normalized)) return true;
  return false;
}

export function stripQuotedFocusaContext(text: string): string {
  const raw = String(text || "");
  if (!raw) return "";

  let stripped = raw;
  stripped = stripped.replace(/\[focusa-context\][\s\S]*$/i, "");
  stripped = stripped.replace(/#\s*focusa context[\s\S]*$/i, "");
  stripped = stripped.replace(/rendered live from focusa-pi-bridge current state\.?[\s\S]*$/i, "");
  stripped = stripped.replace(/focusa:\s.*?(?:frame:|title:|goal:|wbm:|turns:|config:)[\s\S]*$/i, "");
  stripped = stripped.replace(/[\s:;-]+$/g, "");
  const normalized = stripped.replace(/\s+/g, " ").trim();
  if (hasQuotedFocusaPayload(raw) && isFocusaPayloadWrapperText(normalized)) return "";
  return normalized;
}

export function isNonTaskStatusLikeText(text: string): boolean {
  const normalized = String(text || "").replace(/\s+/g, " ").trim();
  if (!normalized) return false;
  if (/^\//.test(normalized)) return true;
  if (/^#\s*focusa context\b/i.test(normalized)) return true;
  if (/^rendered live from focusa-pi-bridge current state\.?/i.test(normalized)) return true;
  if (/^focusa:\s/i.test(normalized) && /(frame:|title:|goal:|wbm:|turns:|config:)/i.test(normalized)) return true;
  if (hasQuotedFocusaPayload(normalized)) return !stripQuotedFocusaContext(normalized);
  return false;
}

export function classifyCurrentAsk(text: string): PiCurrentAskKind {
  const cleaned = stripQuotedFocusaContext(text);
  const lower = cleaned.trim().toLowerCase();
  if (isNonTaskStatusLikeText(text)) return "meta";
  if (!lower) return hasQuotedFocusaPayload(text) ? "meta" : "unknown";
  if (/^(no\b|undo\b|revert\b|wrong\b|that's incorrect\b|not what i asked\b|stop\b|instead\b|ignore previous\b|new task\b|different task\b|go back\b|don't\b)/i.test(lower)) return "correction";
  if (lower.endsWith("?") || /^(what|why|how|when|where|who|which|can|could|should|is|are|do|does|did)\b/.test(lower)) return "question";
  if (/^(note|remember|fyi|for context|meta|discussion:)\b/.test(lower)) return "meta";
  return "instruction";
}

export function isExplicitContinuationAsk(text: string): boolean {
  return /^(continue\b|go ahead\b|proceed\b|keep going\b|finish\b|resume\b|carry on\b|pick up where you left off\b|same task\b)/i.test(text.trim());
}

export function isOperatorSteeringInput(text: string, askKind: PiCurrentAskKind): boolean {
  const trimmed = stripQuotedFocusaContext(text).trim();
  if (!trimmed) return false;
  if (askKind === "question" || askKind === "correction") return true;
  if (askKind === "meta") return false;
  return /\b(continue|resume|instead|stop|don't|answer|focus on|work on|switch to|use|fix|implement|explain|summarize|show|verify|check)\b/i.test(trimmed);
}

export function deriveQueryScope(askKind: PiCurrentAskKind): Pick<PiQueryScope, "scopeKind" | "carryoverPolicy"> {
  return {
    scopeKind: askKind === "question"
      ? "fresh_question"
      : askKind === "correction"
        ? "correction"
        : askKind === "meta"
          ? "meta"
          : "mission_carryover",
    carryoverPolicy: askKind === "question"
      ? "suppress_by_default"
      : askKind === "correction"
        ? "prefer_reset"
        : "allow_if_relevant",
  };
}

function tokenizeForRelevance(text: string): string[] {
  return Array.from(new Set(
    text
      .toLowerCase()
      .match(/[a-z0-9_./:-]{3,}/g) || [],
  ));
}

function scoreRelevance(candidate: string, askText: string): number {
  const askTokens = tokenizeForRelevance(askText);
  if (!askTokens.length) return 0;

  const candidateText = candidate.toLowerCase();
  const candidateTokens = new Set(tokenizeForRelevance(candidate));
  let score = 0;

  for (const token of askTokens) {
    if (candidateTokens.has(token)) {
      score += token.length >= 8 ? 5 : 3;
      continue;
    }
    if (candidateText.includes(token)) {
      score += token.length >= 8 ? 3 : 2;
      continue;
    }
    if (token.includes("/") && candidateText.includes(token.split("/").pop() || token)) {
      score += 2;
    }
  }

  const normalizedAsk = askText.trim().toLowerCase();
  if (normalizedAsk && candidateText.includes(normalizedAsk)) score += 8;
  if (/\b(test|failing|error|bug|trace|constraint|decision|scope|question|correction)\b/.test(normalizedAsk) && /\b(test|failing|error|bug|trace|constraint|decision|scope|question|correction)\b/.test(candidateText)) {
    score += 2;
  }

  return score;
}

function freshnessBoost(updatedAt?: string | null, pinned?: boolean): number {
  if (pinned) return 4;
  if (!updatedAt) return 0;
  const ts = Date.parse(updatedAt);
  if (Number.isNaN(ts)) return 0;
  const ageHours = (Date.now() - ts) / 3_600_000;
  if (ageHours <= 6) return 4;
  if (ageHours <= 24) return 3;
  if (ageHours <= 72) return 2;
  if (ageHours <= 168) return 1;
  if (ageHours >= 24 * 30) return -3;
  if (ageHours >= 24 * 14) return -2;
  if (ageHours >= 24 * 7) return -1;
  return 0;
}

export function selectRelevantRankedItems(
  items: PiRankedItem[] | undefined,
  askText: string,
  options?: { maxItems?: number; fallbackItems?: number; minScore?: number; allowStaleFallback?: boolean },
): PiFocusSelection {
  const values = (items || []).filter((item): item is PiRankedItem => Boolean(item?.value && item.value.trim()));
  if (!values.length) return { items: [], excluded: [], scores: [] };

  const maxItems = options?.maxItems ?? 3;
  const fallbackItems = options?.fallbackItems ?? Math.min(2, maxItems);
  const minScore = options?.minScore ?? 2;
  const allowStaleFallback = options?.allowStaleFallback ?? true;
  const ranked = values
    .map((item, index) => ({
      value: item.value,
      index,
      score: scoreRelevance(item.value, askText) + freshnessBoost(item.updatedAt, item.pinned),
    }))
    .sort((a, b) => b.score - a.score || b.index - a.index);

  const relevant = ranked.filter((entry) => entry.score >= minScore).slice(0, maxItems);
  const fallbackPool = allowStaleFallback ? ranked : ranked.filter((entry) => entry.score >= 0);
  const chosen = relevant.length
    ? relevant
    : fallbackItems > 0
      ? fallbackPool.slice(Math.max(fallbackPool.length - fallbackItems, 0))
      : [];
  const chosenValues = chosen.map((entry) => entry.value);
  const chosenSet = new Set(chosenValues);

  return {
    items: chosenValues,
    excluded: values.map(({ value }) => value).filter((value) => !chosenSet.has(value)),
    scores: ranked.map(({ value, score }) => ({ value, score })),
  };
}

export function selectRelevantItems(
  items: string[] | undefined,
  askText: string,
  options?: { maxItems?: number; fallbackItems?: number; minScore?: number },
): PiFocusSelection {
  return selectRelevantRankedItems(
    (items || []).filter((item): item is string => Boolean(item && item.trim())).map((value) => ({ value })),
    askText,
    options,
  );
}

export function selectionRelevanceScore(selection: PiFocusSelection): number {
  if (!selection.items.length || !selection.scores.length) return 0;
  const selected = new Set(selection.items);
  const scores = selection.scores
    .filter(({ value }) => selected.has(value))
    .map(({ score }) => score);
  return scores.length ? Math.max(...scores) : 0;
}

export function formatWorkingSetItems(records: Array<{ key?: string; value?: string; updated_at?: string; pinned?: boolean }> | undefined): PiRankedItem[] {
  const out: PiRankedItem[] = [];
  for (const record of records || []) {
    const key = String(record?.key || "").trim();
    const value = String(record?.value || "").trim();
    if (!key || !value) continue;
    out.push({
      value: `${key} = ${value}`,
      updatedAt: record?.updated_at || null,
      pinned: Boolean(record?.pinned),
    });
  }
  return out;
}

export function formatVerifiedDeltaItems(handles: Array<{ kind?: string; id?: string; label?: string; created_at?: string; pinned?: boolean }> | undefined): PiRankedItem[] {
  const out: PiRankedItem[] = [];
  for (const handle of handles || []) {
    const kind = String(handle?.kind || "other").trim() || "other";
    const id = String(handle?.id || "").trim();
    const label = String(handle?.label || "unnamed").trim() || "unnamed";
    if (!id) continue;
    out.push({
      value: `[HANDLE:${kind}:${id} "${label}"]`,
      updatedAt: handle?.created_at || null,
      pinned: Boolean(handle?.pinned),
    });
  }
  return out;
}

export function buildCanonicalReferenceAliases(items: string[] | undefined): string[] {
  const out: string[] = [];
  const seen = new Set<string>();
  const re = /^\[HANDLE:([^:]+):([^\s]+)\s+"([^"]+)"\]$/;
  for (const item of items || []) {
    const match = item.match(re);
    if (!match) continue;
    const [, kind, id, label] = match;
    const alias = `${label} -> ${kind}:${id}`;
    if (seen.has(alias)) continue;
    seen.add(alias);
    out.push(alias);
  }
  return out;
}

export function orderSliceSections(sections: PiSliceSection[]): PiSliceSection[] {
  return [...sections].sort((a, b) => {
    const priorityDelta = (a.priority ?? 100) - (b.priority ?? 100);
    if (priorityDelta !== 0) return priorityDelta;
    const relevanceDelta = (b.relevanceScore ?? 0) - (a.relevanceScore ?? 0);
    if (relevanceDelta !== 0) return relevanceDelta;
    return (b.selectedCount ?? 0) - (a.selectedCount ?? 0);
  });
}

export function shouldIncludeMissionContext(
  askText: string,
  scopeKind: PiQueryScope["scopeKind"],
  missionLike: string[],
): boolean {
  if (scopeKind === "meta") return true;
  if (!missionLike.some(Boolean)) return false;
  if (isExplicitContinuationAsk(askText)) return true;

  const joinedMission = missionLike.filter(Boolean).join(" \n ").toLowerCase();
  const askTokens = tokenizeForRelevance(askText);
  if (!askTokens.length) return scopeKind === "mission_carryover";

  const overlapsMission = askTokens.some((token) => joinedMission.includes(token));
  if (scopeKind === "fresh_question" || scopeKind === "correction") return overlapsMission;
  return overlapsMission;
}

export function buildSliceSection(
  key: string,
  label: string,
  items: string[] | undefined,
  include: boolean,
  formatter?: (values: string[]) => string,
  excludedCount?: number,
  priority?: number,
  relevanceScore?: number,
): PiSliceSection {
  const values = (items || []).filter(Boolean);
  return {
    key,
    text: formatter ? formatter(values) : `${label}: ${values[0] || "(none)"}`,
    include: include && values.length > 0,
    selectedCount: values.length,
    excludedCount,
    priority,
    relevanceScore,
  };
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
      focusaPost("/telemetry/ops", {
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
      focusaFetch("/telemetry/ops", {
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
  // §33.5 + §34.2H: Pi context reads its own scoped frame from /focus/stack and hydrates live ASCC state from /ascc/state.
  if (!S.activeFrameId && !S.sessionFrameKey) return null;

  const [stack, asccState] = await Promise.all([
    focusaFetch("/focus/stack"),
    focusaFetch("/ascc/state").catch(() => null),
  ]);
  if (!stack?.stack?.frames?.length) return null;

  const frames = stack.stack.frames;
  let frame = S.activeFrameId ? frames.find((f: any) => f.id === S.activeFrameId) || null : null;

  if ((!frame || frame.status !== "active" || isContaminatedFrameIdentity(frame)) && S.sessionFrameKey) {
    const scopedActive = [...frames].reverse().find((f: any) =>
      f.status === "active" && Array.isArray(f.tags) && f.tags.includes(S.sessionFrameKey || "") && !isContaminatedFrameIdentity(f)
    ) || null;
    if (scopedActive) {
      frame = scopedActive;
      S.activeFrameId = scopedActive.id;
    } else if (frame && isContaminatedFrameIdentity(frame)) {
      S.activeFrameId = null;
      S.activeFrameTitle = "";
      S.activeFrameGoal = "";
      return null;
    }
  }

  if (!frame || isContaminatedFrameIdentity(frame)) {
    S.activeFrameId = null;
    S.activeFrameTitle = "";
    S.activeFrameGoal = "";
    return null;
  }

  const liveAscc = asccState?.frame_id === frame.id ? (asccState?.ascc || asccState?.focus_state || null) : null;
  const frameState = frame?.focus_state || {};
  const fs = {
    ...frameState,
    ...(liveAscc || {}),
    current_focus: liveAscc?.current_focus || frameState.current_focus || frameState.current_state || "",
    current_state: liveAscc?.current_state || frameState.current_state || frameState.current_focus || "",
  };

  S.activeFrameTitle = frame.title || S.activeFrameTitle || "";
  S.activeFrameGoal = frame.goal || S.activeFrameGoal || "";
  if (S.activeFrameTitle) S.pi?.setSessionName(S.activeFrameTitle);
  S.lastFocusSnapshot = {
    decisions: Array.isArray(fs?.decisions) ? fs.decisions : [],
    constraints: Array.isArray(fs?.constraints) ? fs.constraints : [],
    failures: Array.isArray(fs?.failures) ? fs.failures : [],
    intent: fs?.intent || "",
    currentFocus: fs?.current_focus || fs?.current_state || "",
  };

  // Never sync Pi's scoped frame from Focusa global active_frame_id.
  return { frame, fs, stack };
}

export function trimFrameText(text: string, max = 80): string {
  const normalized = String(text || "").replace(/\s+/g, " ").trim();
  if (!normalized) return "";
  return normalized.length <= max ? normalized : `${normalized.slice(0, max - 1)}…`;
}

function derivePiFrameIntent(cwd: string): { projectName: string; title: string; goal: string } {
  const projectName = cwd.split("/").filter(Boolean).pop() || "root";
  const ask = trimFrameText(S.currentAsk?.text || "", 100);
  const askKind = S.currentAsk?.kind || "unknown";

  if (ask && askKind !== "meta") {
    const titlePrefix = askKind === "question"
      ? "Pi Question"
      : askKind === "correction"
        ? "Pi Correction"
        : "Pi Task";
    return {
      projectName,
      title: `${titlePrefix}: ${trimFrameText(ask, 70)}`,
      goal: ask,
    };
  }

  return {
    projectName,
    title: `Pi: ${projectName}`,
    goal: `Work on ${projectName}`,
  };
}

export async function createPiFrame(cwd: string, source = "pi-auto"): Promise<string | null> {
  S.sessionCwd = cwd;
  const { projectName, title, goal } = derivePiFrameIntent(cwd);
  S.activeFrameTitle = title;
  S.activeFrameGoal = goal;
  const sessionKey = S.sessionFrameKey || `pi-${process.pid}-${Date.now()}`;
  S.sessionFrameKey = sessionKey;
  const beadsIssueId = `pi-session-${projectName}-${sessionKey}`;
  const tags = ["pi", projectName, source, sessionKey, "task-first-frame"]; 

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
      if (S.activeFrameTitle) S.pi?.setSessionName(S.activeFrameTitle);
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
        S.activeFrameTitle = match.title || title;
        S.activeFrameGoal = match.goal || goal;
        if (S.activeFrameTitle) S.pi?.setSessionName(S.activeFrameTitle);
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

export function isGenericPiFrameForCwd(cwd: string, title?: string | null, goal?: string | null): boolean {
  const projectName = cwd.split("/").filter(Boolean).pop() || "root";
  return (title || "") === `Pi: ${projectName}` && (goal || "") === `Work on ${projectName}`;
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

export async function rescopePiFrameFromCurrentAsk(cwd?: string, source = "pi-ask-rescope"): Promise<string | null> {
  if (!S.focusaAvailable || !S.activeFrameId) return S.activeFrameId;
  const resolvedCwd = cwd || S.sessionCwd || process.cwd();
  const ask = trimFrameText(stripQuotedFocusaContext(S.currentAsk?.text || ""), 100);
  const askKind = S.currentAsk?.kind || "unknown";
  if (!ask || askKind === "meta" || isNonTaskStatusLikeText(ask)) return S.activeFrameId;

  const activeGoal = trimFrameText(stripQuotedFocusaContext(S.activeFrameGoal || ""), 100).toLowerCase();
  const askNorm = ask.toLowerCase();
  const sameMission = Boolean(activeGoal) && (
    askNorm === activeGoal ||
    askNorm.includes(activeGoal) ||
    activeGoal.includes(askNorm)
  );

  const genericFrame = isGenericPiFrameForCwd(resolvedCwd, S.activeFrameTitle, S.activeFrameGoal);
  const explicitContinuation = isExplicitContinuationAsk(ask);
  const shouldRescope = genericFrame || (!explicitContinuation && !sameMission && askNorm.length >= 6);
  if (!shouldRescope) return S.activeFrameId;

  try {
    await focusaFetch("/focus/pop", {
      method: "POST",
      body: JSON.stringify({
        completion_reason: genericFrame
          ? "startup frame rescoped after first real ask"
          : "frame rescoped after mission shift",
      }),
    });
  } catch {
    return S.activeFrameId;
  }

  S.activeFrameId = null;
  return await createPiFrame(resolvedCwd, source);
}

export function getEffectiveFocusSnapshot(fs?: any): {
  decisions: string[];
  constraints: string[];
  failures: string[];
  intent: string;
  currentFocus: string;
} {
  return {
    decisions: fs?.decisions || S.lastFocusSnapshot.decisions || S.localDecisions,
    constraints: fs?.constraints || S.lastFocusSnapshot.constraints || S.localConstraints,
    failures: fs?.failures || S.lastFocusSnapshot.failures || S.localFailures,
    intent: fs?.intent || S.lastFocusSnapshot.intent || "",
    currentFocus: fs?.current_focus || fs?.current_state || S.lastFocusSnapshot.currentFocus || "",
  };
}

export async function persistAuthoritativeState(): Promise<void> {
  if (S.focusaAvailable && S.activeFrameId) {
    await getFocusState().catch(() => null);
  }
  persistState();
}

export function persistState(): void {
  const payload = {
    sessionId: S.sessionFrameKey,
    frameId: S.activeFrameId,
    frameTitle: S.activeFrameTitle,
    frameGoal: S.activeFrameGoal,
    currentAsk: S.currentAsk,
    queryScope: S.queryScope,
    decisions: S.localDecisions,
    constraints: S.localConstraints,
    failures: S.localFailures,
    authoritativeDecisions: S.lastFocusSnapshot.decisions,
    authoritativeConstraints: S.lastFocusSnapshot.constraints,
    authoritativeFailures: S.lastFocusSnapshot.failures,
    intent: S.lastFocusSnapshot.intent,
    currentFocus: S.lastFocusSnapshot.currentFocus,
    turnCount: S.turnCount,
    wbmEnabled: S.wbmEnabled,
    wbmNoCatalogue: S.wbmNoCatalogue,
    cataloguedDecisions: S.cataloguedDecisions,
    cataloguedFacts: S.cataloguedFacts,
    totalCompactions: S.totalCompactions,
    timestamp: Date.now(),
  };
  S.pi?.appendEntry("focusa-state", payload);
  S.pi?.appendEntry("focusa-wbm-state", payload);
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
