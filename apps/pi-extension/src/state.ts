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

export type ScopeFailureKind =
  | "scope_contamination"
  | "adjacent_thread_leakage"
  | "answer_broadening"
  | "wrong_question_answered"
  | "context_overcarry";

export interface ScopeFailureSignal {
  kind: ScopeFailureKind;
  severity: "low" | "medium" | "high";
  reason: string;
}

export type PiGoverningPriorKind =
  | "hard_safety_prior"
  | "identity_prior"
  | "current_ask_prior"
  | "mission_commitment_prior"
  | "affordance_reality_prior";

export interface PiFocusSelection {
  items: string[];
  excluded: string[];
  scores: Array<{
    value: string;
    score: number;
    relevanceScore?: number;
    freshnessBoost?: number;
    priorBoost?: number;
    appliedPriors?: PiGoverningPriorKind[];
  }>;
}

export interface PiRetentionBuckets {
  active: string[];
  decayed: string[];
  historical: string[];
  scores: PiFocusSelection["scores"];
}

export interface PiRankedItem {
  value: string;
  updatedAt?: string | null;
  pinned?: boolean;
  priorKinds?: PiGoverningPriorKind[];
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
  // Spec88 Workpoint resume packet projected from Focusa.
  activeWorkpointPacket: null as any | null,
  activeWorkpointSummary: "" as string,
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
  // Persistence dedup/throttle for appendEntry pressure
  lastPersistAt: 0,
  lastPersistHash: "",
  // Hot-path caches for context injection latency control
  focusStateCache: {
    key: "",
    at: 0,
    data: null as { frame: any; fs: any; stack: any } | null,
    inflight: null as Promise<{ frame: any; fs: any; stack: any } | null> | null,
  },
  semanticMemoryCache: {
    at: 0,
    data: null as any,
    inflight: null as Promise<any> | null,
  },
  ecsHandlesCache: {
    at: 0,
    data: null as any,
    inflight: null as Promise<any> | null,
  },
};

const FOCUS_STATE_CACHE_TTL_MS = 1_200;
const AUX_CONTEXT_CACHE_TTL_MS = 3_000;
const CONTEXT_SEMANTIC_LIMIT = 64;
const CONTEXT_ECS_HANDLES_LIMIT = 128;

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

export function sanitizeFocusFailures(items: string[]): string[] {
  return (Array.isArray(items) ? items : [])
    .map((item) => String(item || "").trim())
    .filter(Boolean)
    .filter((item) => !/^operator correction:/i.test(item));
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

export const FORBIDDEN_VISIBLE_OUTPUT_LEAK_CLASSES = [
  {
    class_id: "raw_focus_state_serialization",
    description: "Raw Focusa slice/state payload leaked into visible assistant text",
    pattern: /\[focusa focus slice|\bprojection_kind:\b|\bview_profile:\b|\bquery_scope:\b|\bcanonical_sources:\b|\bworking_set:\b|\bverified_deltas:\b/i,
  },
  {
    class_id: "internal_routing_reasons",
    description: "Internal routing/selection reason labels leaked into visible assistant text",
    pattern: /\brelevant_context_selected\b|\birrelevant_context_excluded\b|\bprior_mission_reused\b|\bquery_scope_built\b|\bsubject_hijack_prevented\b/i,
  },
  {
    class_id: "metacognitive_prose",
    description: "Internal metacognitive/planner phrasing leaked into visible assistant text",
    pattern: /\bminimal_focus_slice_builder\b|\bconsultation trace\b|\bfocusa cognitive guidance\b|\boperator-first routing\b/i,
  },
  {
    class_id: "hidden_trace_dimensions",
    description: "Hidden trace/event dimensions leaked into visible assistant text",
    pattern: /\bfocus_slice_relevance_score\b|\bresolved_reference_count\b|\bselected_counts\b|\bprojection_boundary\b|\bcanonical_sources\b/i,
  },
  {
    class_id: "reducer_internal_state",
    description: "Reducer/daemon internal state identifiers leaked into visible assistant text",
    pattern: /\bactive_writer\b|\bpause_flags\b|\blast_recorded_bd_transition_id\b|\btransport_session_state\b|\bwork_loop\.run\b|\bstate\.version\b/i,
  },
] as const;

export function detectForbiddenVisibleOutputLeakClasses(text: string): string[] {
  const normalized = stripQuotedFocusaContext(String(text || "")).trim();
  if (!normalized) return [];
  return FORBIDDEN_VISIBLE_OUTPUT_LEAK_CLASSES
    .filter((entry) => entry.pattern.test(normalized))
    .map((entry) => entry.class_id);
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

export function detectScopeFailureSignals(params: {
  askText: string;
  askKind: PiCurrentAskKind;
  scopeKind: PiQueryScope["scopeKind"];
  assistantOutput: string;
  leakClasses?: string[];
}): ScopeFailureSignal[] {
  const askText = stripQuotedFocusaContext(params.askText || "").trim().toLowerCase();
  const output = String(params.assistantOutput || "").trim();
  if (!output) return [];

  const outputLower = output.toLowerCase();
  const askTokens = tokenizeForRelevance(askText).filter((token) => token.length >= 4).slice(0, 12);
  const overlapCount = askTokens.filter((token) => outputLower.includes(token)).length;
  const failures: ScopeFailureSignal[] = [];

  const addFailure = (signal: ScopeFailureSignal) => {
    if (!failures.some((existing) => existing.kind === signal.kind)) failures.push(signal);
  };

  if ((params.leakClasses || []).some((cls) => cls === "raw_focus_state_serialization" || cls === "internal_routing_reasons")) {
    addFailure({
      kind: "scope_contamination",
      severity: "high",
      reason: "assistant output leaked internal Focusa routing/state payload",
    });
  }

  if (askTokens.length >= 2 && overlapCount === 0 && output.length >= 120) {
    addFailure({
      kind: "wrong_question_answered",
      severity: "medium",
      reason: "assistant output has no lexical overlap with current ask",
    });
  }

  if ((params.scopeKind === "fresh_question" || params.scopeKind === "correction") && /\b(as we discussed|as noted earlier|continuing from|from the previous task|carry(ing)? over)\b/i.test(outputLower)) {
    addFailure({
      kind: "context_overcarry",
      severity: "medium",
      reason: "fresh/correction scope output referenced prior-thread carryover",
    });
  }

  if ((params.scopeKind === "fresh_question" || params.scopeKind === "correction") && /\b(other thread|adjacent thread|another task|previous thread|neighbor(ing)? task)\b/i.test(outputLower)) {
    addFailure({
      kind: "adjacent_thread_leakage",
      severity: "medium",
      reason: "fresh/correction scope output referenced adjacent thread/task",
    });
  }

  if ((params.askKind === "question" || params.askKind === "instruction")
      && (params.scopeKind === "fresh_question" || params.scopeKind === "correction")
      && /\b(more broadly|in general|also consider|additionally|in broader terms)\b/i.test(outputLower)
      && overlapCount <= Math.max(1, Math.floor(askTokens.length / 4))) {
    addFailure({
      kind: "answer_broadening",
      severity: "low",
      reason: "fresh/correction scope output broadened beyond ask-specific terms",
    });
  }

  return failures;
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

const GOVERNING_PRIOR_BAND_BOOST: Record<PiGoverningPriorKind, number> = {
  hard_safety_prior: 10,
  identity_prior: 8,
  current_ask_prior: 7,
  mission_commitment_prior: 5,
  affordance_reality_prior: 4,
};

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

function normalizeActiveGoverningPriors(priors: PiGoverningPriorKind[] | undefined): PiGoverningPriorKind[] {
  const seen = new Set<PiGoverningPriorKind>();
  const out: PiGoverningPriorKind[] = [];
  for (const prior of priors || []) {
    if (seen.has(prior)) continue;
    seen.add(prior);
    out.push(prior);
  }
  return out;
}

function governingPriorContribution(
  itemPriorKinds: PiGoverningPriorKind[] | undefined,
  activePriors: PiGoverningPriorKind[],
): { priorBoost: number; appliedPriors: PiGoverningPriorKind[] } {
  const itemPriorSet = new Set(itemPriorKinds || []);
  const appliedPriors = activePriors.filter((prior) => itemPriorSet.has(prior));
  const priorBoost = appliedPriors.reduce((max, prior) => {
    const boost = GOVERNING_PRIOR_BAND_BOOST[prior] || 0;
    return boost > max ? boost : max;
  }, 0);
  return { priorBoost, appliedPriors };
}

export function selectRelevantRankedItems(
  items: PiRankedItem[] | undefined,
  askText: string,
  options?: {
    maxItems?: number;
    fallbackItems?: number;
    minScore?: number;
    allowStaleFallback?: boolean;
    governingPriors?: PiGoverningPriorKind[];
  },
): PiFocusSelection {
  const values = (items || []).filter((item): item is PiRankedItem => Boolean(item?.value && item.value.trim()));
  if (!values.length) return { items: [], excluded: [], scores: [] };

  const maxItems = options?.maxItems ?? 3;
  const fallbackItems = options?.fallbackItems ?? Math.min(2, maxItems);
  const minScore = options?.minScore ?? 2;
  const allowStaleFallback = options?.allowStaleFallback ?? true;
  const activePriors = normalizeActiveGoverningPriors(options?.governingPriors);
  const ranked = values
    .map((item, index) => {
      const relevanceScore = scoreRelevance(item.value, askText);
      const freshness = freshnessBoost(item.updatedAt, item.pinned);
      const { priorBoost, appliedPriors } = governingPriorContribution(item.priorKinds, activePriors);
      return {
        value: item.value,
        index,
        score: relevanceScore + freshness + priorBoost,
        relevanceScore,
        freshnessBoost: freshness,
        priorBoost,
        appliedPriors,
      };
    })
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
    scores: ranked.map(({ value, score, relevanceScore, freshnessBoost, priorBoost, appliedPriors }) => ({
      value,
      score,
      relevanceScore,
      freshnessBoost,
      priorBoost,
      appliedPriors,
    })),
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

export function retentionBucketsFromSelection(
  selection: PiFocusSelection,
  options?: { maxDecayed?: number; maxHistorical?: number },
): PiRetentionBuckets {
  const maxDecayed = Math.max(options?.maxDecayed ?? 2, 0);
  const maxHistorical = Math.max(options?.maxHistorical ?? 2, 0);
  const active = [...selection.items];
  const activeSet = new Set(active);
  const nonActive = selection.scores.filter((entry) => !activeSet.has(entry.value));

  const decayed = nonActive
    .filter((entry) => (entry.score ?? 0) >= 0)
    .slice(0, maxDecayed)
    .map((entry) => entry.value);

  const decayedSet = new Set(decayed);
  let historicalPool = nonActive.filter(
    (entry) => (entry.score ?? 0) < 0 && !decayedSet.has(entry.value),
  );
  if (!historicalPool.length) {
    historicalPool = nonActive.filter((entry) => !decayedSet.has(entry.value));
  }

  const historical = historicalPool
    .slice(Math.max(historicalPool.length - maxHistorical, 0))
    .map((entry) => entry.value);

  return {
    active,
    decayed,
    historical,
    scores: selection.scores,
  };
}

function inferGoverningPriorKinds(text: string): PiGoverningPriorKind[] {
  const lower = text.toLowerCase();
  const out: PiGoverningPriorKind[] = [];
  const add = (kind: PiGoverningPriorKind) => {
    if (!out.includes(kind)) out.push(kind);
  };

  if (/\b(safety|forbid|forbidden|never|must_not|must not|policy|destructive|high[-_ ]risk|constraint)\b/.test(lower)) {
    add("hard_safety_prior");
  }
  if (/\b(identity|role|operator|owner|author|user|persona)\b/.test(lower)) {
    add("identity_prior");
  }
  if (/\b(current[_ ]ask|query|scope|question|correction|steering|subject)\b/.test(lower)) {
    add("current_ask_prior");
  }
  if (/\b(mission|intent|goal|focus|commitment|work[_ ]item|task|tranche)\b/.test(lower)) {
    add("mission_commitment_prior");
  }
  if (/\b(affordance|permission|tool|execution|environment|transport|worktree|dependency|resource)\b/.test(lower)) {
    add("affordance_reality_prior");
  }

  return out;
}

export function formatWorkingSetItems(records: Array<{ key?: string; value?: string; updated_at?: string; pinned?: boolean }> | undefined): PiRankedItem[] {
  const out: PiRankedItem[] = [];
  for (const record of records || []) {
    const key = String(record?.key || "").trim();
    const value = String(record?.value || "").trim();
    if (!key || !value) continue;
    const priorKinds = inferGoverningPriorKinds(`${key} ${value}`);
    out.push({
      value: `${key} = ${value}`,
      updatedAt: record?.updated_at || null,
      pinned: Boolean(record?.pinned),
      priorKinds,
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
    const priorKinds = inferGoverningPriorKinds(`${kind} ${label}`);
    out.push({
      value: `[HANDLE:${kind}:${id} "${label}"]`,
      updatedAt: handle?.created_at || null,
      pinned: Boolean(handle?.pinned),
      priorKinds,
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

async function loadFocusState(): Promise<{ frame: any; fs: any; stack: any } | null> {
  const scopedQs = new URLSearchParams();
  if (S.activeFrameId) scopedQs.set("frame_id", S.activeFrameId);
  if (S.sessionFrameKey) scopedQs.set("session_key", S.sessionFrameKey);
  const scopedPath = scopedQs.size > 0 ? `/focus/frame/current?${scopedQs.toString()}` : null;

  const [scoped, asccState] = await Promise.all([
    scopedPath ? focusaFetch(scopedPath).catch(() => null) : Promise.resolve(null),
    focusaFetch("/ascc/state").catch(() => null),
  ]);

  let frame = scoped?.frame || null;
  let stack = frame
    ? { stack: { active_id: scoped?.active_frame_id || null, frames: [frame] }, active_frame_id: scoped?.active_frame_id || null }
    : null;

  if (!frame) {
    stack = await focusaFetch("/focus/stack");
    if (!stack?.stack?.frames?.length) return null;
    const frames = stack.stack.frames;
    frame = S.activeFrameId ? frames.find((f: any) => f.id === S.activeFrameId) || null : null;

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

  S.activeFrameId = frame.id || S.activeFrameId;
  S.activeFrameTitle = frame.title || S.activeFrameTitle || "";
  S.activeFrameGoal = frame.goal || S.activeFrameGoal || "";
  if (S.activeFrameTitle) S.pi?.setSessionName(S.activeFrameTitle);
  S.lastFocusSnapshot = {
    decisions: Array.isArray(fs?.decisions) ? fs.decisions : [],
    constraints: Array.isArray(fs?.constraints) ? fs.constraints : [],
    failures: sanitizeFocusFailures(Array.isArray(fs?.failures) ? fs.failures : []),
    intent: fs?.intent || "",
    currentFocus: fs?.current_focus || fs?.current_state || "",
  };

  return { frame, fs, stack };
}

// ── Get Focus State from Focusa scoped to Pi's own frame (§33.5 isolation) ──
// CRITICAL: Never use Focusa's global active_frame_id — that belongs to Wirebot.
// Pi sessions must only read their own frame. If Pi has no frame, return empty.
export async function getFocusState(): Promise<{ frame: any; fs: any; stack: any } | null> {
  if (!S.activeFrameId && !S.sessionFrameKey) return null;

  const cacheKey = `${S.activeFrameId || ""}|${S.sessionFrameKey || ""}`;
  const now = Date.now();
  if (S.focusStateCache.data && S.focusStateCache.key === cacheKey && now - S.focusStateCache.at < FOCUS_STATE_CACHE_TTL_MS) {
    return S.focusStateCache.data;
  }
  if (S.focusStateCache.inflight && S.focusStateCache.key === cacheKey) {
    return await S.focusStateCache.inflight;
  }

  const inflight = loadFocusState();
  S.focusStateCache.key = cacheKey;
  S.focusStateCache.inflight = inflight;
  try {
    const data = await inflight;
    if (data) {
      S.focusStateCache.data = data;
      S.focusStateCache.at = Date.now();
    }
    return data;
  } finally {
    if (S.focusStateCache.inflight === inflight) S.focusStateCache.inflight = null;
  }
}

export async function getSemanticMemorySummary(): Promise<any> {
  const now = Date.now();
  if (S.semanticMemoryCache.data && now - S.semanticMemoryCache.at < AUX_CONTEXT_CACHE_TTL_MS) {
    return S.semanticMemoryCache.data;
  }
  if (S.semanticMemoryCache.inflight) return await S.semanticMemoryCache.inflight;

  const inflight = focusaFetch(`/memory/semantic?limit=${CONTEXT_SEMANTIC_LIMIT}&summary_only=true`);
  S.semanticMemoryCache.inflight = inflight;
  try {
    const data = await inflight;
    if (data) {
      S.semanticMemoryCache.data = data;
      S.semanticMemoryCache.at = Date.now();
    }
    return data;
  } finally {
    if (S.semanticMemoryCache.inflight === inflight) S.semanticMemoryCache.inflight = null;
  }
}

export async function getEcsHandlesSummary(): Promise<any> {
  const now = Date.now();
  if (S.ecsHandlesCache.data && now - S.ecsHandlesCache.at < AUX_CONTEXT_CACHE_TTL_MS) {
    return S.ecsHandlesCache.data;
  }
  if (S.ecsHandlesCache.inflight) return await S.ecsHandlesCache.inflight;

  const inflight = focusaFetch(`/ecs/handles?limit=${CONTEXT_ECS_HANDLES_LIMIT}&summary_only=true`);
  S.ecsHandlesCache.inflight = inflight;
  try {
    const data = await inflight;
    if (data) {
      S.ecsHandlesCache.data = data;
      S.ecsHandlesCache.at = Date.now();
    }
    return data;
  } finally {
    if (S.ecsHandlesCache.inflight === inflight) S.ecsHandlesCache.inflight = null;
  }
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
    failures: sanitizeFocusFailures(fs?.failures || S.lastFocusSnapshot.failures || S.localFailures),
    intent: fs?.intent || S.lastFocusSnapshot.intent || "",
    currentFocus: fs?.current_focus || fs?.current_state || S.lastFocusSnapshot.currentFocus || "",
  };
}

const MAX_PERSIST_LIST_ITEMS = 40;
const MAX_PERSIST_TEXT_CHARS = 320;
const PERSIST_MIN_INTERVAL_MS = 3_000;
const MAX_ECS_ITEMS = 180;
const MAX_ECS_TOTAL_BYTES = 64 * 1024 * 1024;
const MAX_ECS_ITEM_BYTES = 1024 * 1024;
const ECS_TTL_MS = 6 * 60 * 60 * 1000;

function trimPersistText(input: string): string {
  const normalized = String(input || "").replace(/\s+/g, " ").trim();
  if (normalized.length <= MAX_PERSIST_TEXT_CHARS) return normalized;
  return `${normalized.slice(0, MAX_PERSIST_TEXT_CHARS - 1)}…`;
}

function tailBounded(items: string[], maxItems = MAX_PERSIST_LIST_ITEMS): string[] {
  return (items || [])
    .map((item) => trimPersistText(String(item || "")))
    .filter(Boolean)
    .slice(-maxItems);
}

function pruneEcsRegistry(now = Date.now()): void {
  type Flat = { kind: string; id: string; storedAt: number; bytes: number };
  const flat: Flat[] = [];

  for (const [kind, bucket] of Object.entries(S.ecsRegistry || {})) {
    for (const [id, record] of Object.entries(bucket || {})) {
      const age = now - (record?.storedAt || 0);
      if (!record || typeof record.content !== "string" || age > ECS_TTL_MS) {
        delete bucket[id];
        continue;
      }
      const bytes = Buffer.byteLength(record.content, "utf8");
      flat.push({ kind, id, storedAt: record.storedAt || 0, bytes });
    }
    if (!Object.keys(bucket || {}).length) delete S.ecsRegistry[kind];
  }

  flat.sort((a, b) => a.storedAt - b.storedAt);
  let totalBytes = flat.reduce((sum, item) => sum + item.bytes, 0);
  let totalItems = flat.length;

  while (flat.length && (totalItems > MAX_ECS_ITEMS || totalBytes > MAX_ECS_TOTAL_BYTES)) {
    const victim = flat.shift();
    if (!victim) break;
    if (S.ecsRegistry[victim.kind]?.[victim.id]) {
      delete S.ecsRegistry[victim.kind][victim.id];
      if (!Object.keys(S.ecsRegistry[victim.kind]).length) delete S.ecsRegistry[victim.kind];
      totalItems -= 1;
      totalBytes = Math.max(0, totalBytes - victim.bytes);
    }
  }
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
    frameTitle: trimPersistText(S.activeFrameTitle),
    frameGoal: trimPersistText(S.activeFrameGoal),
    currentAsk: S.currentAsk
      ? { ...S.currentAsk, text: trimPersistText(S.currentAsk.text) }
      : null,
    queryScope: S.queryScope,
    decisions: tailBounded(S.localDecisions),
    constraints: tailBounded(S.localConstraints),
    failures: tailBounded(sanitizeFocusFailures(S.localFailures), 20),
    authoritativeDecisions: tailBounded(S.lastFocusSnapshot.decisions),
    authoritativeConstraints: tailBounded(S.lastFocusSnapshot.constraints),
    authoritativeFailures: tailBounded(sanitizeFocusFailures(S.lastFocusSnapshot.failures), 20),
    intent: trimPersistText(S.lastFocusSnapshot.intent),
    currentFocus: trimPersistText(S.lastFocusSnapshot.currentFocus),
    activeWorkpointPacket: S.activeWorkpointPacket,
    activeWorkpointSummary: trimPersistText(S.activeWorkpointSummary),
    turnCount: S.turnCount,
    wbmEnabled: S.wbmEnabled,
    wbmNoCatalogue: S.wbmNoCatalogue,
    cataloguedDecisions: tailBounded(S.cataloguedDecisions),
    cataloguedFacts: tailBounded(S.cataloguedFacts),
    totalCompactions: S.totalCompactions,
    timestamp: Date.now(),
  };

  const now = Date.now();
  const payloadHash = JSON.stringify(payload);
  if (S.lastPersistHash === payloadHash && now - S.lastPersistAt < PERSIST_MIN_INTERVAL_MS) {
    return;
  }

  S.lastPersistHash = payloadHash;
  S.lastPersistAt = now;

  S.pi?.appendEntry("focusa-state", payload);
  if (S.wbmEnabled) S.pi?.appendEntry("focusa-wbm-state", payload);
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
  const raw = String(content || "");
  const clipped = Buffer.byteLength(raw, "utf8") > MAX_ECS_ITEM_BYTES
    ? `${raw.slice(0, MAX_ECS_ITEM_BYTES)}\n...[local ECS clipped due to memory cap]`
    : raw;
  S.ecsRegistry[kind][id] = { content: clipped, storedAt: Date.now() };
  pruneEcsRegistry();
  return id;
}

export function getEcsArtifact(kind: string, id: string): string | null {
  pruneEcsRegistry();
  return S.ecsRegistry[kind]?.[id]?.content ?? null;
}

export function extractHandles(text: string): Array<{ kind: string; id: string }> {
  const handles: Array<{ kind: string; id: string }> = [];
  const re = /\[HANDLE:([^:]+):([^\]]+)\]/g;
  let m;
  while ((m = re.exec(text)) !== null) handles.push({ kind: m[1], id: m[2] });
  return handles;
}
