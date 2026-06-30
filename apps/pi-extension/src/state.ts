// Shared state, helpers, types for focusa-pi-bridge
// Spec: docs/44-pi-focusa-integration-spec.md

import { existsSync, mkdirSync, readFileSync, writeFileSync } from "fs";
import { homedir } from "os";
import { dirname, join, resolve } from "path";
import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { DEFAULT_DAEMON_RESTART_COMMAND, type FocusaConfig } from "./config.js";

export type PiCurrentAskKind = "question" | "instruction" | "correction" | "meta" | "unknown";

export interface PiCurrentAsk {
  text: string;
  kind: PiCurrentAskKind;
  sourceTurnId: string;
  updatedAt: number;
  sessionId?: string;
  projectRoot?: string;
  continuityId?: string;
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

export type AttentionRecallVerdictStatus = "attentive" | "attention_risk" | "conflict" | "unknown";

export interface PiMemoryAnchor {
  task: string;
  must_not_forget: string[];
  latest_report_summary_ref: string;
  evidence_refs: string[];
  next_action: string;
  action_authority_for_current_ask: boolean;
}

export interface PiAttentionRecallVerdict {
  schema: "focusa.attention_recall_verdict.v1";
  status: AttentionRecallVerdictStatus;
  visible_recap_required: boolean;
  visible_recap_reason: string;
  attention_risks: string[];
  required_next: string[];
  current_ask_scope_status: "aligned" | "conflict" | "unknown";
  scope_conflict_reason: string;
  memory_anchor: PiMemoryAnchor;
}

export interface PiReportSummaryHandle {
  handle: string;
  summary: string;
  capturedAt: number;
  turnId: string;
}

export interface PiToolOutputPressure {
  windowStartedAt: number;
  resultCount: number;
  totalBytes: number;
  totalTokens: number;
  largeResultCount: number;
  recapRequired: boolean;
  recapReason: string;
  lastToolName: string;
  lastEventAt: number;
  lastRecapAt: number;
}

export interface PiProjectThreadObservation {
  project_alias: string;
  project_root: string;
  remote_host?: string;
  evidence_ref: string;
  first_seen_turn: string;
  last_seen_turn: string;
  recent_actions: string[];
  confidence: number;
  source: "current_ask" | "tool_evidence" | "project_identity" | "session_entry";
  updatedAt: number;
  // FOCUSA-GitHub#3: Relationship kind for multi-repo agent work
  // - active_scope: current operator-approved primary work
  // - supporting_work: related action in another repo that supports active scope
  // - artifact_link: issue/PR/evidence/ticket created while pursuing active scope
  // - scope_switch: only when operator-confirmed
  relationship_kind?: "active_scope" | "supporting_work" | "artifact_link" | "scope_switch";
  primary_scope_ref?: string;
  supporting_scope_ref?: string;
  operator_confirmed_scope_switch?: boolean;
  action_authority_transfers?: boolean;
}

export interface PiCurrentAskScopeVerdict {
  status: "aligned" | "override_candidate" | "conflict" | "unknown";
  saved_scope: { project_root: string; continuity_id: string };
  current_ask_scope: { project_alias: string; project_root: string; confidence: number; evidence_ref: string };
  action_authority_for_current_ask: boolean;
  required_next: string[];
  reason: string;
}

export const PROJECT_SWITCH_LEDGER_MAX_OBSERVATIONS = 12;
export const PROJECT_SWITCH_LEDGER_MAX_ACTIONS = 5;
export const PROJECT_SWITCH_LEDGER_MIN_CONFLICT_CONFIDENCE = 0.55;

export const TOOL_OUTPUT_FLOOD_WINDOW_MS = 120_000;
export const TOOL_OUTPUT_FLOOD_RESULT_THRESHOLD = 4;
export const TOOL_OUTPUT_FLOOD_BYTES_THRESHOLD = 24_000;
export const TOOL_OUTPUT_FLOOD_TOKENS_THRESHOLD = 4_000;
export const TOOL_OUTPUT_FLOOD_LARGE_RESULT_BYTES = 8_192;
export const TOOL_OUTPUT_FLOOD_LARGE_RESULT_THRESHOLD = 2;

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
  lastProjectRootResolution: null as null | { projectRoot: string; confidence: "high" | "medium" | "low"; confidenceScore: number; source: string; reason: string; safe: boolean; requiresOperatorConfirmation: boolean; markerScore?: number; markers?: string[]; candidates?: Array<{ projectRoot: string; confidenceScore: number; markers: string[]; source: string }> },
  activeFrameId: null as string | null,
  activeFramePromise: null as Promise<string | null> | null,
  activeFrameTitle: "" as string,
  activeFrameGoal: "" as string,
  uiCtx: null as any,  // §93: SSE handler needs ctx.ui for high-priority agent alerts
  sessionFrameKey: "" as string,
  sessionCwd: "" as string,
  continuityId: "" as string,
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
  // Spec108 awareness substrate cadence state
  awarenessCadenceState: null as null | {
    lastShownAt: number;
    lastPct: number;
    lastTier: "low" | "medium" | "high" | "critical";
    lastAnchorState: string;
    compactionCountAtLastShown: number;
    transitionCount: number;
    suppressionCount: number;
  },
  lastWorkpointUpdate: 0, // timestamp ms of last Workpoint update
  // Streaming delta (§36.1)
  lastStreamLen: 0,
  // Auto-resume dedup: set when compaction fires, cleared after continuation sent
  compactResumePending: false,
  // Persisted compaction auto-resume idempotency guard; prevents repeated post-compact resume spam across extension reloads.
  lastCompactResumeKey: "",
  lastCompactResumeAt: 0,
  // Post-compaction: save last decision for steer message (cleared after localDecisions trim)
  lastCompactDecision: "",
  // Spec88 Workpoint resume packet projected from Focusa.
  activeWorkpointPacket: null as any | null,
  activeWorkpointSummary: "" as string,
  lastTrajectoryClarity: null as any | null,
  lastProjectIdentity: null as any | null,
  lastProjectVerify: null as any | null,
  latestReportSummary: null as PiReportSummaryHandle | null,
  toolOutputPressure: {
    windowStartedAt: 0,
    resultCount: 0,
    totalBytes: 0,
    totalTokens: 0,
    largeResultCount: 0,
    recapRequired: false,
    recapReason: "",
    lastToolName: "",
    lastEventAt: 0,
    lastRecapAt: 0,
  } as PiToolOutputPressure,
  projectSwitchLedger: [] as PiProjectThreadObservation[],
  lastCurrentAskScopeTelemetryKey: "",
  vitalInfoPrompted: {} as Record<string, number>,
  // First-turn guard: only inject behavioral directive once per session, not on every before_agent_start
  seenFirstBeforeAgentStart: false,
  // ECS handle registry: kind -> id -> { content, stored_at }
  ecsRegistry: {} as Record<string, Record<string, { content: string; storedAt: number }>>,
  // Tool usage batching (§33.4)
  toolUsageBatch: [] as string[],
  // Spec92 bounded hook/token telemetry (in-memory Pi extension ring buffers)
  spec92HookTelemetry: [] as Array<Record<string, unknown>>,
  spec92TokenTelemetry: [] as Array<Record<string, unknown>>,
  spec92ToolStartTimes: {} as Record<string, number>,
  // Intuition signals (§36.2, §34.2D)
  compilationErrors: [] as number[],
  fileEditCounts: {} as Record<string, number>,
  // Session/task timing + token accounting
  sessionStartTime: Date.now(),
  currentTaskStartTime: Date.now(),
  currentTaskLabel: "",
  currentTaskTurnStart: 0,
  currentTaskInputTokenEstimate: 0,
  currentTaskOutputTokenEstimate: 0,
  currentTaskProviderInputTokens: 0,
  currentTaskProviderOutputTokens: 0,
  currentTaskToolCalls: 0,
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
  daemonRestartAttempts: [] as number[],
  daemonRestartInFlight: null as Promise<boolean> | null,
  daemonHoldoverMode: false,
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
const HEALTHCHECK_STATUS_FALLBACK_PATH = "/status?summary_only=true";

export function resetPiSessionScopedState(reason = "session_boundary"): void {
  S.turnCount = 0;
  S.seenFirstBeforeAgentStart = false;
  S.activeFrameId = null;
  S.activeFramePromise = null;
  S.activeFrameTitle = "";
  S.activeFrameGoal = "";
  S.continuityId = "";
  setActiveWorkpointPacket(null);
  setActiveWorkpointSummary("");
  setLastTrajectoryClarity(null);
  S.lastProjectIdentity = null;
  setLatestReportSummary(null);
  resetToolOutputPressureWindow(Date.now());
  S.projectSwitchLedger = [];
  S.currentAsk = null;
  S.queryScope = null;
  S.excludedContext = null;
  S.lastFocusSnapshot = { decisions: [], constraints: [], failures: [], intent: "", currentFocus: "" };
  S.localDecisions = [];
  S.localConstraints = [];
  S.localFailures = [];
  S.lastCompactTime = 0;
  S.compactsThisHour = 0;
  S.turnsSinceCompact = 0;
  S.compactHourStart = Date.now();
  S.currentTier = "";
  S.currentContextPct = null;
  S.lastStreamLen = 0;
  S.compactResumePending = false;
  S.lastCompactResumeKey = "";
  S.lastCompactResumeAt = 0;
  S.lastCompactDecision = "";
  S.toolUsageBatch = [];
  S.spec92HookTelemetry = [];
  S.spec92TokenTelemetry = [];
  S.spec92ToolStartTimes = {};
  S.compilationErrors = [];
  S.fileEditCounts = {};
  S.longSessionSignaled = false;
  S.cataloguedDecisions = [];
  S.cataloguedFacts = [];
  S.totalCompactions = 0;
  S.forkSuggested = false;
  S.focusStateCache = { key: "", at: 0, data: null, inflight: null };
  S.semanticMemoryCache = { at: 0, data: null, inflight: null };
  S.ecsHandlesCache = { at: 0, data: null, inflight: null };
  S.lastPersistAt = 0;
  S.lastPersistHash = "";
  S.wbmEnabled = false;
  S.wbmDeep = false;
  S.wbmNoCatalogue = false;
  focusaPost("/telemetry/trace", { event_type: "pi_session_scoped_state_reset", payload: { reason, session_id: S.sessionFrameKey, cwd: S.sessionCwd } });
}

// ── HTTP helper ──────────────────────────────────────────────────────────────
export async function focusaFetch(path: string, opts: RequestInit = {}): Promise<any> {
  const timeout = S.cfg?.focusaApiTimeoutMs || 5000;
  const base = S.cfg?.focusaApiBaseUrl || "http://127.0.0.1:8787/v1";
  const token = S.cfg?.focusaToken || "";
  const attempts = 2;
  for (let attempt = 0; attempt < attempts; attempt++) {
    const ac = new AbortController();
    const t = setTimeout(() => ac.abort(), timeout);
    try {
      const r = await fetch(`${base}${path}`, {
        ...opts,
        headers: {
          "Content-Type": "application/json",
          // Mark this client as the focusa-pi-extension so the daemon can
          // surface the agent-layer prompt (X-Focusa-Agent-Prompt response
          // header + structured /v1/agent/prompt body). See
          // crates/focusa-api/src/routes/agent_reminder.rs.
          "X-Focusa-Client": "pi",
          "X-Extension-Token": `focusa-pi-${S.cfg?.focusaExtensionBuild || "v0"}`,
          ...(token ? { Authorization: `Bearer ${token}` } : {}),
          ...(opts.headers as Record<string, string> || {}),
        },
        signal: ac.signal,
      });
      if (r.ok) return await r.json();
      if (![429, 502, 503, 504].includes(r.status) || attempt === attempts - 1) return null;
    } catch {
      if (attempt === attempts - 1) return null;
    } finally {
      clearTimeout(t);
    }
    await new Promise((resolve) => setTimeout(resolve, 150));
  }
  return null;
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

function boundedAttentionText(value: unknown, max = 180): string {
  const text = String(value ?? "").replace(/\s+/g, " ").trim();
  if (!text) return "none";
  return text.length > max ? `${text.slice(0, Math.max(0, max - 1))}…` : text;
}

function arrayField(value: unknown): unknown[] {
  return Array.isArray(value) ? value : [];
}

function workpointValue(packet: any, key: string): string {
  return String(packet?.[key] || packet?.workpoint?.[key] || "").trim();
}

function latestReportSummaryRefFromFocusState(focusState?: any): string {
  if (getLatestReportSummary()?.handle) return getLatestReportSummary()!.handle;
  const candidates = [
    ...arrayField(focusState?.recent_results),
    ...arrayField(focusState?.notes),
    ...arrayField(focusState?.artifacts).map((artifact: any) => `${artifact?.kind || "artifact"}:${artifact?.label || artifact?.path_or_id || "unknown"}${artifact?.path_or_id ? `@${artifact.path_or_id}` : ""}`),
  ].map((item) => String(item || "").trim()).filter(Boolean);
  const report = [...candidates].reverse().find((item) => /\b(report|summary|spec|audit|proof)\b/i.test(item));
  return report ? boundedAttentionText(report, 160) : "none";
}

function resetToolOutputPressureWindow(now = Date.now()): void {
  S.toolOutputPressure = {
    windowStartedAt: now,
    resultCount: 0,
    totalBytes: 0,
    totalTokens: 0,
    largeResultCount: 0,
    recapRequired: false,
    recapReason: "",
    lastToolName: "",
    lastEventAt: now,
    lastRecapAt: S.toolOutputPressure?.lastRecapAt || 0,
  };
}

export function recordToolOutputPressure(toolName: string, bytes: number, tokens: number): PiToolOutputPressure {
  const now = Date.now();
  if (!S.toolOutputPressure.windowStartedAt || now - S.toolOutputPressure.windowStartedAt > TOOL_OUTPUT_FLOOD_WINDOW_MS) {
    resetToolOutputPressureWindow(now);
  }
  const pressure = S.toolOutputPressure;
  pressure.resultCount += 1;
  pressure.totalBytes += Math.max(0, bytes || 0);
  pressure.totalTokens += Math.max(0, tokens || 0);
  pressure.largeResultCount += bytes >= TOOL_OUTPUT_FLOOD_LARGE_RESULT_BYTES ? 1 : 0;
  pressure.lastToolName = toolName || "unknown_tool";
  pressure.lastEventAt = now;
  const reasons = [
    pressure.resultCount >= TOOL_OUTPUT_FLOOD_RESULT_THRESHOLD ? `tool_results=${pressure.resultCount}` : "",
    pressure.totalBytes >= TOOL_OUTPUT_FLOOD_BYTES_THRESHOLD ? `bytes=${pressure.totalBytes}` : "",
    pressure.totalTokens >= TOOL_OUTPUT_FLOOD_TOKENS_THRESHOLD ? `tokens=${pressure.totalTokens}` : "",
    pressure.largeResultCount >= TOOL_OUTPUT_FLOOD_LARGE_RESULT_THRESHOLD ? `large_outputs=${pressure.largeResultCount}` : "",
  ].filter(Boolean);
  if (reasons.length) {
    const newlyRequired = !pressure.recapRequired;
    pressure.recapRequired = true;
    pressure.recapReason = `tool_output_flood:${reasons.join(",")}; last_tool=${boundedAttentionText(pressure.lastToolName, 80)}`;
    if (newlyRequired) {
      focusaPost("/telemetry/trace", {
        event_type: "tool_output_flood_detected",
        payload: {
          reason: pressure.recapReason,
          window_ms: TOOL_OUTPUT_FLOOD_WINDOW_MS,
          result_count: pressure.resultCount,
          total_bytes: pressure.totalBytes,
          total_tokens: pressure.totalTokens,
          large_result_count: pressure.largeResultCount,
          latest_report_summary_ref: getLatestReportSummary()?.handle || "none",
        },
      });
      focusaPost("/telemetry/trace", {
        event_type: "visible_recap_required",
        payload: {
          reason: pressure.recapReason,
          required_next: "recap_memory_anchor_before_project_scoped_action",
        },
      });
      focusaPost("/focus-gate/ingest-signal", {
        signal_type: "tool_output_flood",
        surface: "pi",
        payload: { reason: pressure.recapReason, result_count: pressure.resultCount, total_bytes: pressure.totalBytes },
      });
      persistState();
    }
  }
  return { ...pressure };
}

export function toolOutputVisibleRecapReason(): string {
  if (!S.toolOutputPressure?.recapRequired) return "";
  if (S.toolOutputPressure.windowStartedAt && Date.now() - S.toolOutputPressure.windowStartedAt > TOOL_OUTPUT_FLOOD_WINDOW_MS) {
    resetToolOutputPressureWindow(Date.now());
    persistState();
    return "";
  }
  return S.toolOutputPressure.recapReason;
}

export function formatToolOutputVisibleRecapLines(reason = toolOutputVisibleRecapReason()): string[] {
  if (!reason) return [];
  return [
    "VISIBLE_RECAP_REQUIRED:",
    `  reason=${boundedAttentionText(reason, 180)}`,
    `  latest_report_summary_ref=${getLatestReportSummary()?.handle || "none"}`,
    "  required_before_next_action=Recap MEMORY_ANCHOR/latest report in 1-2 lines before any tool/file/API action.",
    "END_VISIBLE_RECAP_REQUIRED",
  ];
}

export function markVisibleRecapEmittedIfPresent(assistantOutput: string): boolean {
  const reason = toolOutputVisibleRecapReason();
  if (!reason) return false;
  const preview = String(assistantOutput || "").slice(0, 700);
  const recapped = /\b(recap|memory anchor|latest report|report-summary|current task|continuing)\b/i.test(preview);
  focusaPost("/telemetry/trace", {
    event_type: recapped ? "visible_recap_emitted" : "visible_recap_missing",
    payload: {
      reason,
      assistant_preview: boundedAttentionText(preview, 220),
      latest_report_summary_ref: getLatestReportSummary()?.handle || "none",
    },
  });
  if (!recapped) return false;
  S.toolOutputPressure.lastRecapAt = Date.now();
  resetToolOutputPressureWindow(Date.now());
  persistState();
  return true;
}

function cleanResumeVisibleRecapReason(reason?: string): string {
  const value = String(reason || "").trim();
  return value ? boundedAttentionText(value, 220) : "";
}

function projectRootFromAbsolutePath(value: string): string {
  const match = String(value || "").match(/\/(?:home|Users)\/[A-Za-z0-9._-]+\/[A-Za-z0-9._-]+(?:\/[A-Za-z0-9._-]+)?/);
  if (!match) return "";
  const parts = match[0].split("/").filter(Boolean);
  if (parts[0] === "home") return normalizeProjectRoot(`/${parts.slice(0, Math.min(parts.length, 3)).join("/")}`);
  if (parts[0] === "Users") {
    const depth = parts[2] === "Projects" || parts[2] === "projects" ? 4 : 3;
    return normalizeProjectRoot(`/${parts.slice(0, Math.min(parts.length, depth)).join("/")}`);
  }
  return normalizeProjectRoot(match[0]);
}

function projectAliasesForText(text: string, root = ""): string[] {
  const lower = String(text || "").toLowerCase();
  const aliases = new Set<string>();
  if (/\b(ptm|planmarr|plan-the-marriage|plan the marriage)\b/i.test(lower) || /planmarr|plan-the-marriage/i.test(root)) aliases.add("PTM");
  if (/\bfocusa\b/i.test(lower) || /\/focusa$/i.test(root)) aliases.add("Focusa");
  const base = root.split("/").filter(Boolean).at(-1);
  if (base) aliases.add(base);
  return [...aliases].filter(Boolean).slice(0, 4);
}

function boundedProjectAction(source: string, action?: string): string {
  return boundedAttentionText(`${source}:${action || "observed project evidence"}`, 180);
}

export function observeProjectThreadEvidence(input: {
  project_root?: string;
  project_alias?: string;
  remote_host?: string;
  evidence_ref: string;
  turn_id: string;
  action?: string;
  confidence?: number;
  source: PiProjectThreadObservation["source"];
}): PiProjectThreadObservation | null {
  const projectRoot = normalizeProjectRoot(input.project_root || "");
  const alias = boundedAttentionText(input.project_alias || projectAliasesForText(input.action || "", projectRoot)[0] || projectRoot.split("/").filter(Boolean).at(-1) || "unknown", 80);
  if (!projectRoot && !alias) return null;
  const now = Date.now();
  const keyRoot = projectRoot || `alias:${alias.toLowerCase()}`;
  const existing = S.projectSwitchLedger.find((entry) => (entry.project_root && entry.project_root === projectRoot) || entry.project_alias.toLowerCase() === alias.toLowerCase());
  const action = boundedProjectAction(input.source, input.action);
  const observation: PiProjectThreadObservation = existing ? {
    ...existing,
    project_alias: existing.project_alias || alias,
    project_root: existing.project_root || projectRoot || keyRoot,
    remote_host: input.remote_host || existing.remote_host,
    evidence_ref: boundedAttentionText(input.evidence_ref || existing.evidence_ref, 160),
    last_seen_turn: input.turn_id,
    recent_actions: [action, ...existing.recent_actions.filter((item) => item !== action)].slice(0, PROJECT_SWITCH_LEDGER_MAX_ACTIONS),
    confidence: Math.max(existing.confidence || 0, input.confidence ?? 0.6),
    source: input.source,
    updatedAt: now,
  } : {
    project_alias: alias,
    project_root: projectRoot || keyRoot,
    remote_host: input.remote_host,
    evidence_ref: boundedAttentionText(input.evidence_ref, 160),
    first_seen_turn: input.turn_id,
    last_seen_turn: input.turn_id,
    recent_actions: [action].slice(0, PROJECT_SWITCH_LEDGER_MAX_ACTIONS),
    confidence: input.confidence ?? (projectRoot ? 0.8 : 0.55),
    source: input.source,
    updatedAt: now,
  };
  S.projectSwitchLedger = [observation, ...S.projectSwitchLedger.filter((entry) => entry !== existing)]
    .sort((a, b) => b.updatedAt - a.updatedAt)
    .slice(0, PROJECT_SWITCH_LEDGER_MAX_OBSERVATIONS);
  try { S.pi?.appendEntry("focusa-project-switch-ledger", { observations: S.projectSwitchLedger.slice(0, 6) }); } catch { /* best effort */ }
  persistState();
  return observation;
}

// FOCUSA-GitHub#3: Mark a project observation as supporting work
// Use this when the agent filed a ticket/PR/evidence in another repo
// while pursuing the primary active scope. Does NOT transfer action authority.
export function markObservationAsSupportingWork(
  alias: string,
  primaryScopeRef: string,
  whyRelated: string,
): boolean {
  const lower = alias.toLowerCase().trim();
  const entry = S.projectSwitchLedger.find(
    (e) => e.project_alias.toLowerCase() === lower || e.project_root.toLowerCase().includes(lower)
  );
  if (!entry) return false;
  entry.relationship_kind = "supporting_work";
  entry.primary_scope_ref = primaryScopeRef;
  entry.supporting_scope_ref = entry.project_root;
  entry.action_authority_transfers = false;
  entry.recent_actions = [
    `supporting_for=${primaryScopeRef}`,
    `why=${whyRelated.slice(0, 80)}`,
    ...entry.recent_actions,
  ].slice(0, PROJECT_SWITCH_LEDGER_MAX_ACTIONS);
  try { S.pi?.appendEntry("focusa-project-switch-ledger", { observations: S.projectSwitchLedger.slice(0, 6) }); } catch { /* best effort */ }
  persistState();
  return true;
}

// FOCUSA-GitHub#3: Get all observations grouped by relationship kind
export function groupObservationsByRelationship(): {
  active_scope: PiProjectThreadObservation[];
  supporting_work: PiProjectThreadObservation[];
  artifact_link: PiProjectThreadObservation[];
  scope_switch: PiProjectThreadObservation[];
  uncategorized: PiProjectThreadObservation[];
} {
  const result = {
    active_scope: [] as PiProjectThreadObservation[],
    supporting_work: [] as PiProjectThreadObservation[],
    artifact_link: [] as PiProjectThreadObservation[],
    scope_switch: [] as PiProjectThreadObservation[],
    uncategorized: [] as PiProjectThreadObservation[],
  };
  for (const entry of S.projectSwitchLedger) {
    switch (entry.relationship_kind) {
      case "active_scope": result.active_scope.push(entry); break;
      case "supporting_work": result.supporting_work.push(entry); break;
      case "artifact_link": result.artifact_link.push(entry); break;
      case "scope_switch": result.scope_switch.push(entry); break;
      default: result.uncategorized.push(entry);
    }
  }
  return result;
}

function rememberedProjectRootForAlias(alias: string): string {
  const lower = String(alias || "").toLowerCase().trim();
  if (!lower) return "";
  const scored: Array<{ root: string; score: number }> = [];
  const last = getLastProjectIdentity() || {};
  const lastAliases = Array.isArray(last.aliases) ? last.aliases : [];
  const lastText = [last.project_id, last.canonical_name, ...lastAliases].filter(Boolean).join(" ").toLowerCase();
  if (last.project_root && lastText.includes(lower)) {
    scored.push({ root: normalizeProjectRoot(last.project_root), score: last.confidence === "high" ? 1 : 0.8 });
  }
  for (const entry of S.projectSwitchLedger || []) {
    const entryText = `${entry.project_alias || ""} ${entry.project_root || ""}`.toLowerCase();
    if (entry.project_root && entryText.includes(lower)) {
      scored.push({ root: normalizeProjectRoot(entry.project_root), score: entry.confidence || 0.5 });
    }
  }
  scored.sort((a, b) => b.score - a.score);
  if (scored[0]?.root) return scored[0].root;

  // GitHub #4 fix: When no project_root is stored for this alias,
  // search known project directories for a .focusa-project.json marker
  // matching the alias name (e.g., 'perpetua' should resolve to /home/focusadev/perpetua)
  return searchProjectMarkerForAlias(lower);
}

// FOCUSA_FIX-GitHub#4: Search filesystem for a project's canonical root
// When a project alias is mentioned but no project_root is in the ledger,
// look in known project directories for a matching .focusa-project.json
function searchProjectMarkerForAlias(alias: string): string {
  // Known candidate directories where sub-projects live
  const candidateDirs = [
    "/home/focusadev",
    "/home/wirebot",
    process.env.HOME || "",
  ].filter(Boolean);

  for (const dir of candidateDirs) {
    try {
      // Read directory entries - synchronous fs would be better but use execSync
      const entries = require("child_process").execSync(
        `ls -1 ${dir} 2>/dev/null`,
        { stdio: ["pipe", "pipe", "pipe"] }
      ).toString().trim().split("\n");

      for (const entry of entries) {
        if (!entry) continue;
        const markerPath = `${dir}/${entry}/.focusa-project.json`;
        try {
          const markerContent = require("fs").readFileSync(markerPath, "utf-8");
          const marker = JSON.parse(markerContent);
          const markerId = String(marker.project_id || "").toLowerCase();
          const markerName = String(marker.canonical_name || "").toLowerCase();
          const markerAliases = Array.isArray(marker.aliases) ? marker.aliases.map((a: string) => a.toLowerCase()) : [];
          if (markerId === alias || markerName === alias || markerAliases.includes(alias)) {
            const root = String(marker.project_root || `${dir}/${entry}`);
            try {
              require("child_process").execSync(
                `mkdir -p /tmp/pi-scratch && echo '[alias-resolution] searchProjectMarkerForAlias: resolved alias ${alias} to ${root} via marker at ${markerPath}' >> /tmp/pi-scratch/alias-resolution.log`,
                { stdio: "pipe" }
              );
            } catch { /* best effort */ }
            return normalizeProjectRoot(root);
          }
        } catch { /* not a marker file or unreadable */ }
      }
    } catch { /* directory unreadable */ }
  }
  return "";
}

export function observeProjectThreadHintsFromText(text: string, turnId: string, source: PiProjectThreadObservation["source"], action?: string): PiProjectThreadObservation[] {
  const raw = String(text || "").slice(0, 2000);
  const root = projectRootFromAbsolutePath(raw);
  const aliases = projectAliasesForText(raw, root);
  const observations: PiProjectThreadObservation[] = [];
  if (root) {
    observations.push(observeProjectThreadEvidence({ project_root: root, project_alias: aliases[0], evidence_ref: `${source}:${turnId}:project_path`, turn_id: turnId, action: action || `path=${root}`, confidence: 0.9, source })!);
  }
  for (const alias of aliases) {
    if (root && alias === aliases[0]) continue;
    const knownRoot = root || rememberedProjectRootForAlias(alias);
    observations.push(observeProjectThreadEvidence({ project_root: knownRoot, project_alias: alias, evidence_ref: `${source}:${turnId}:project_alias:${alias}`, turn_id: turnId, action: action || `alias=${alias}`, confidence: knownRoot ? 0.75 : 0.58, source })!);
  }
  return observations.filter(Boolean);
}

function aliasMatchesSavedProject(alias: string, savedProjectRoot: string): boolean {
  const savedRoot = normalizeProjectRoot(savedProjectRoot);
  if (!savedRoot) return false;
  const lowerAlias = String(alias || "").toLowerCase().trim();
  if (!lowerAlias) return false;
  return projectAliasesForText("", savedRoot).some((candidate) => candidate.toLowerCase() === lowerAlias);
}

function isAliasOnlyProjectRoot(root: string): boolean {
  return normalizeProjectRoot(root).startsWith("alias:");
}

function projectSwitchLedgerCandidateForAsk(currentAskText: string, savedProjectRoot: string): PiProjectThreadObservation | null {
  const ask = stripQuotedFocusaContext(currentAskText || "");
  if (!ask.trim() || !S.projectSwitchLedger.length) return null;
  const lower = ask.toLowerCase();
  const savedRoot = normalizeProjectRoot(savedProjectRoot);
  const scored = S.projectSwitchLedger.map((entry) => {
    let score = entry.confidence || 0;
    const alias = entry.project_alias.toLowerCase();
    const entryRoot = normalizeProjectRoot(entry.project_root);
    if (savedRoot && isAliasOnlyProjectRoot(entryRoot) && aliasMatchesSavedProject(alias, savedRoot)) score -= 2.0;
    if (alias && lower.includes(alias.toLowerCase())) score += 0.5;
    if (/\b(ptm|planmarr|plan-the-marriage|plan the marriage)\b/i.test(lower) && /ptm|planmarr|plan-the-marriage/i.test(`${entry.project_alias} ${entry.project_root}`)) score += 0.7;
    if (entryRoot && !isAliasOnlyProjectRoot(entryRoot) && lower.includes(entryRoot.toLowerCase())) score += 0.8;
    if (/\b(wrong place|not this repo|not this project|different project|remote project|switch project)\b/i.test(lower)) score += 0.15;
    if (savedRoot && entryRoot === savedRoot) score -= 0.6;
    score += Math.max(0, 0.2 - (Date.now() - entry.updatedAt) / 86_400_000 * 0.05);
    return { entry, score };
  }).sort((a, b) => b.score - a.score);
  const best = scored[0];
  if (!best || best.score < PROJECT_SWITCH_LEDGER_MIN_CONFLICT_CONFIDENCE) return null;
  const bestRoot = normalizeProjectRoot(best.entry.project_root);
  if (savedRoot && bestRoot === savedRoot) return null;
  if (savedRoot && isAliasOnlyProjectRoot(bestRoot) && aliasMatchesSavedProject(best.entry.project_alias, savedRoot)) return null;
  return best.entry;
}

export function formatProjectSwitchLedgerLines(currentAskText = S.currentAsk?.text || ""): string[] {
  const candidate = projectSwitchLedgerCandidateForAsk(currentAskText, getScopedWorkpointPacket()?.project_root || S.sessionCwd || "");
  const entries = (candidate ? [candidate, ...S.projectSwitchLedger.filter((entry) => entry !== candidate)] : S.projectSwitchLedger).slice(0, 4);
  return entries.map((entry) => `${entry.project_alias} root=${entry.project_root || "unknown"} confidence=${entry.confidence.toFixed(2)} evidence=${entry.evidence_ref} recent=${entry.recent_actions.slice(0, 2).join(" | ")}`);
}

export function emitCurrentAskScopeVerdictTelemetry(verdict: PiCurrentAskScopeVerdict, sourceTurnId = S.currentAsk?.sourceTurnId || S.sessionFrameKey || "pi-current-ask-scope"): void {
  if (!S.focusaAvailable || verdict.status !== "conflict") return;
  const key = `${sourceTurnId}:${verdict.status}:${verdict.saved_scope.project_root}:${verdict.current_ask_scope.project_root}:${verdict.reason}`;
  if (S.lastCurrentAskScopeTelemetryKey === key) return;
  S.lastCurrentAskScopeTelemetryKey = key;
  focusaPost("/telemetry/trace", {
    event_type: "scope_conflict_detected",
    turn_id: sourceTurnId,
    payload: {
      schema: "focusa.current_scope_verdict.v1",
      failure_class: "scope_conflict",
      status: verdict.status,
      saved_scope: verdict.saved_scope,
      current_ask_scope: verdict.current_ask_scope,
      action_authority_for_current_ask: verdict.action_authority_for_current_ask,
      required_next: verdict.required_next,
      reason: verdict.reason,
    },
  });
}

export function buildCurrentAskScopeVerdict(options: { currentAskText?: string; workpointPacket?: any; projectRoot?: string; continuityId?: string } = {}): PiCurrentAskScopeVerdict {
  const ask = stripQuotedFocusaContext(options.currentAskText ?? S.currentAsk?.text ?? "");
  const packet = options.workpointPacket || getScopedWorkpointPacket() || {};
  const savedRoot = normalizeProjectRoot(workpointValue(packet, "project_root") || options.projectRoot || S.sessionCwd || "");
  const continuityId = String(options.continuityId || S.continuityId || workpointValue(packet, "continuity_id") || "").trim();
  const explicitRoot = projectRootFromAbsolutePath(ask);
  const aliases = projectAliasesForText(ask, explicitRoot);
  const ledgerCandidate = projectSwitchLedgerCandidateForAsk(ask, savedRoot);
  const alias = aliases[0] || ledgerCandidate?.project_alias || "unknown";
  const aliasKnownRoot = rememberedProjectRootForAlias(alias);
  const rawCandidateRoot = normalizeProjectRoot(explicitRoot || ledgerCandidate?.project_root || aliasKnownRoot || "");
  const candidateRoot = savedRoot && isAliasOnlyProjectRoot(rawCandidateRoot) && aliasMatchesSavedProject(alias, savedRoot) ? "" : rawCandidateRoot;
  const evidenceRef = explicitRoot ? "current_ask:explicit_project_path" : ledgerCandidate?.evidence_ref || (aliases.length ? `current_ask:project_alias:${alias}` : "none");
  const confidence = explicitRoot ? 0.95 : ledgerCandidate?.confidence ?? (candidateRoot ? 0.7 : aliases.length ? 0.58 : 0);
  const hasCorrectionPhrase = /\b(wrong place|not this repo|not this project|different project|remote project|switch project)\b/i.test(ask);
  const sameProjectAliasMention = savedRoot && aliases.some((candidate) => aliasMatchesSavedProject(candidate, savedRoot));
  let status: PiCurrentAskScopeVerdict["status"] = "unknown";
  let reason = "no current-ask project signal";
  if (candidateRoot && savedRoot && candidateRoot !== savedRoot) {
    status = "conflict";
    reason = `current ask indicates ${alias} at ${candidateRoot}, saved scope is ${savedRoot}`;
  } else if (candidateRoot && (!savedRoot || candidateRoot === savedRoot)) {
    status = "aligned";
    reason = candidateRoot === savedRoot ? "current ask project matches saved scope" : "current ask names project but saved scope is unbound";
  } else if (sameProjectAliasMention && savedRoot && !hasCorrectionPhrase) {
    status = "aligned";
    reason = "current ask names saved project alias without competing root";
  } else if (aliases.length || hasCorrectionPhrase) {
    status = "override_candidate";
    reason = aliases.length ? `current ask names project alias ${alias} without verified root` : "operator correction implies saved project/root may be wrong";
  } else if (savedRoot) {
    status = "aligned";
    reason = "no competing project signal in current ask";
  }
  const actionAllowed = status === "aligned";
  const verdict = {
    status,
    saved_scope: { project_root: savedRoot || "unknown", continuity_id: continuityId || "unknown" },
    current_ask_scope: { project_alias: alias, project_root: candidateRoot || "unknown", confidence, evidence_ref: evidenceRef },
    action_authority_for_current_ask: actionAllowed,
    required_next: actionAllowed ? [] : ["recap_scope_conflict", "focusa_project_verify", "focusa_project_identity", "focusa_workpoint_checkpoint"],
    reason,
  };
  emitCurrentAskScopeVerdictTelemetry(verdict);
  return verdict;
}

export function formatCurrentAskScopeVerdictLines(verdict = buildCurrentAskScopeVerdict()): string[] {
  return [
    "CURRENT_ASK_SCOPE_VERDICT:",
    `  status=${verdict.status}`,
    `  saved_project_root=${verdict.saved_scope.project_root}`,
    `  current_ask_project=${verdict.current_ask_scope.project_alias}`,
    `  current_ask_project_root=${verdict.current_ask_scope.project_root}`,
    `  action_authority_for_current_ask=${verdict.action_authority_for_current_ask}`,
    `  reason=${boundedAttentionText(verdict.reason, 180)}`,
    `  required_next=${verdict.required_next.join(" -> ") || "none"}`,
    "END_CURRENT_ASK_SCOPE_VERDICT",
  ];
}

function currentAskProjectConflictReason(currentAskText: string, projectRoot: string, workpointProjectRoot: string): string {
  const ask = stripQuotedFocusaContext(currentAskText || "");
  if (!ask.trim()) return "";
  const lower = ask.toLowerCase();
  const explicitPath = ask.match(/\/(?:home|Users)\/[A-Za-z0-9._-]+\/[A-Za-z0-9._/-]+/);
  if (explicitPath && normalizeProjectRoot(explicitPath[0]) !== normalizeProjectRoot(projectRoot || workpointProjectRoot)) {
    return `operator named different project path ${boundedAttentionText(explicitPath[0], 120)}`;
  }
  const ledgerCandidate = projectSwitchLedgerCandidateForAsk(ask, projectRoot || workpointProjectRoot);
  if (ledgerCandidate) {
    return `project_switch_ledger indicates ${boundedAttentionText(ledgerCandidate.project_alias, 80)} at ${boundedAttentionText(ledgerCandidate.project_root, 120)} (${boundedAttentionText(ledgerCandidate.evidence_ref, 120)})`;
  }
  if (/\b(wrong place|not this repo|not this project|different project|remote project|switch project)\b/i.test(ask)) {
    return "operator text indicates current project/root may be wrong";
  }
  if (/\b(ptm|planmarr|plan-the-marriage)\b/i.test(lower) && !/planmarr|plan-the-marriage/i.test(projectRoot || workpointProjectRoot)) {
    return "operator text names PTM/planmarr while saved scope is different";
  }
  return "";
}

export function buildAttentionRecallVerdict(options: {
  focusState?: any;
  workpointPacket?: any;
  currentAskText?: string;
  currentAskKind?: PiCurrentAskKind | string;
  queryScopeKind?: PiQueryScope["scopeKind"] | string;
  projectRoot?: string;
  continuityId?: string;
  visibleRecapReason?: string;
} = {}): PiAttentionRecallVerdict {
  const packet = options.workpointPacket || getScopedWorkpointPacket() || {};
  const askText = stripQuotedFocusaContext(options.currentAskText ?? S.currentAsk?.text ?? "");
  const projectRoot = normalizeProjectRoot(options.projectRoot || S.sessionCwd || workpointValue(packet, "project_root"));
  const packetProjectRoot = normalizeProjectRoot(workpointValue(packet, "project_root"));
  const continuityId = String(options.continuityId || S.continuityId || workpointValue(packet, "continuity_id") || "").trim();
  const mission = workpointValue(packet, "mission") || S.activeFrameGoal || S.activeFrameTitle || "current Focusa task";
  const nextAction = workpointValue(packet, "next_slice") || S.lastCompactDecision || askText || S.lastFocusSnapshot.currentFocus || "continue bounded current task";
  const ledgerCandidate = projectSwitchLedgerCandidateForAsk(askText, projectRoot || packetProjectRoot);
  const conflictReason = currentAskProjectConflictReason(askText, projectRoot, packetProjectRoot);
  const scopeStatus = conflictReason ? "conflict" : (projectRoot || packetProjectRoot ? "aligned" : "unknown");
  const visibleRecapReason = cleanResumeVisibleRecapReason(options.visibleRecapReason);
  const visibleRecapRequired = Boolean(visibleRecapReason || conflictReason || options.queryScopeKind === "correction" || options.currentAskKind === "correction");
  const attentionRisks = [
    visibleRecapReason ? "tool_output_flood" : "",
    conflictReason ? "scope_conflict" : "",
    options.queryScopeKind === "correction" || options.currentAskKind === "correction" ? "operator_correction" : "",
  ].filter(Boolean);
  const mustNotForget = [
    askText ? `current_ask=${boundedAttentionText(askText, 160)}` : "current_ask=(none)",
    `task=${boundedAttentionText(mission, 140)}`,
    projectRoot ? `project_root=${boundedAttentionText(projectRoot, 140)}` : "project_root=(unbound)",
    continuityId ? `continuity_id=${boundedAttentionText(continuityId, 100)}` : "continuity_id=(unbound)",
    conflictReason ? `scope_conflict=${boundedAttentionText(conflictReason, 140)}` : "scope_conflict=none_detected",
    visibleRecapReason ? `visible_recap_reason=${boundedAttentionText(visibleRecapReason, 140)}` : "visible_recap_reason=none",
    "transcript_tail_is_not_authority",
  ];
  const evidenceRefs = [
    packet?.workpoint_id ? `workpoint:${packet.workpoint_id}` : "",
    packetProjectRoot ? `saved_scope:${packetProjectRoot}` : "",
    ledgerCandidate ? `project_thread:${ledgerCandidate.project_alias}@${ledgerCandidate.project_root}` : "",
    ...arrayField(packet?.verification_records).slice(0, 3).map((record: any) => String(record?.evidence_ref || record?.result || "").trim()),
  ].filter(Boolean).map((item) => boundedAttentionText(item, 140));
  return {
    schema: "focusa.attention_recall_verdict.v1",
    status: conflictReason ? "conflict" : visibleRecapRequired ? "attention_risk" : "attentive",
    visible_recap_required: visibleRecapRequired,
    visible_recap_reason: visibleRecapReason || "none",
    attention_risks: attentionRisks,
    required_next: visibleRecapRequired ? ["recap_memory_anchor"] : [],
    current_ask_scope_status: scopeStatus,
    scope_conflict_reason: conflictReason || "none",
    memory_anchor: {
      task: boundedAttentionText(mission, 160),
      must_not_forget: mustNotForget.slice(0, 8),
      latest_report_summary_ref: latestReportSummaryRefFromFocusState(options.focusState),
      evidence_refs: evidenceRefs.slice(0, 5),
      // FOCUSA_FIX-r4n9: Cut next_action when authority suppressed (scope conflict)
      // When conflictReason is truthy, action_authority_for_current_ask=false
      // In this case, do NOT provide a next_action — force verification before continuing
      next_action: conflictReason
        ? "BLOCKED: scope conflict — verify project scope with focusa_project_identity before continuing"
        : boundedAttentionText(nextAction, 180),
      action_authority_for_current_ask: !conflictReason,
    },
  };
}

export function formatAttentionRecallFocusSliceLines(verdict: PiAttentionRecallVerdict): string[] {
  const anchor = verdict.memory_anchor;
  const authorityBlocked = anchor.action_authority_for_current_ask === false;
  const lines: string[] = [
    "MEMORY_ANCHOR:",
    `  task=${anchor.task}`,
    `  must_not_forget=${anchor.must_not_forget.join(" | ") || "none"}`,
    `  latest_report_summary_ref=${anchor.latest_report_summary_ref}`,
    // FOCUSA_FIX-r4n9: Show blocked indicator when authority is suppressed
    authorityBlocked
      ? `  ⛔ next_action=${anchor.next_action}  ← EXECUTION BLOCKED`
      : `  next_action=${anchor.next_action}`,
    `  action_authority_for_current_ask=${anchor.action_authority_for_current_ask}`,
    `ATTENTION_RECALL_VERDICT: schema=${verdict.schema} status=${verdict.status} visible_recap_required=${verdict.visible_recap_required} visible_recap_reason=${boundedAttentionText(verdict.visible_recap_reason || "none", 140)} attention_risks=${(verdict.attention_risks || []).join(",") || "none"} required_next=${(verdict.required_next || []).join(",") || "none"} current_ask_scope=${verdict.current_ask_scope_status} scope_conflict_reason=${boundedAttentionText(verdict.scope_conflict_reason, 140)}`,
    "END_ATTENTION_RECALL",
  ];
  return lines;
}

function assistantOutputLooksLikeReport(text: string): boolean {
  const normalized = String(text || "").trim();
  if (normalized.length < 240) return false;
  const headingHits = (normalized.match(/^#{1,3}\s+(status|summary|task summary|evidence|proof|result|results|next|blocker|implementation|audit|spec)/gim) || []).length;
  const labelHits = (normalized.match(/\b(Status|Proof|Evidence|Result|Next action|Blocker|Commit|Tests?):/g) || []).length;
  return headingHits >= 1 || labelHits >= 2 || /\b(task summary|end-of-task|implementation report|audit report|spec update|proof:)\b/i.test(normalized);
}

function reportSummaryFromAssistantOutput(text: string): string {
  const lines = String(text || "")
    .split(/\r?\n/)
    .map((line) => line.trim())
    .filter(Boolean)
    .filter((line) => !/^```/.test(line))
    .slice(0, 18);
  const summary = lines.join("\n");
  return summary.length > 1400 ? `${summary.slice(0, 1399)}…` : summary;
}

export function maybeCaptureReportSummaryFromAssistantOutput(text: string, turnId: string): PiReportSummaryHandle | null {
  if (!assistantOutputLooksLikeReport(text)) return null;
  const summary = reportSummaryFromAssistantOutput(text);
  if (!summary) return null;
  const id = storeEcsArtifact("report-summary", summary);
  const handle = `[HANDLE:report-summary:${id}]`;
  const captured: PiReportSummaryHandle = { handle, summary: boundedAttentionText(summary, 240), capturedAt: Date.now(), turnId };
  setLatestReportSummary(captured);
  try { S.pi?.appendEntry("focusa-report-summary", captured); } catch { /* best effort */ }
  persistState();
  return captured;
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
async function sleep(ms: number): Promise<void> {
  await new Promise((resolve) => setTimeout(resolve, ms));
}

export async function kickstartFocusaDaemon(reason = "health_check"): Promise<boolean> {
  if (!S.cfg?.daemonAutoRestart || !S.pi) return false;
  if (S.daemonRestartInFlight) return S.daemonRestartInFlight;
  const now = Date.now();
  const hourAgo = now - 3_600_000;
  S.daemonRestartAttempts = S.daemonRestartAttempts.filter((t) => t >= hourAgo);
  if (S.daemonRestartAttempts.length >= (S.cfg.daemonRestartMaxPerHour || 20)) return false;
  const last = S.daemonRestartAttempts[S.daemonRestartAttempts.length - 1] || 0;
  if (now - last < (S.cfg.daemonRestartCooldownMs || 5_000)) return false;

  S.daemonRestartAttempts.push(now);
  const cmd = S.cfg.daemonRestartCommand || DEFAULT_DAEMON_RESTART_COMMAND;
  S.daemonRestartInFlight = (async () => {
    try {
      if (cmd !== DEFAULT_DAEMON_RESTART_COMMAND) return false;
      await S.pi!.exec("systemctl", ["restart", "focusa-daemon"]);
      for (let i = 0; i < 12; i++) {
        await sleep(S.cfg?.daemonRecoveryProbeMs || 750);
        const h = await focusaFetch("/health");
        if (h?.ok === true) return true;
      }
    } catch {
      return false;
    } finally {
      S.daemonRestartInFlight = null;
    }
    return false;
  })();
  const ok = await S.daemonRestartInFlight;
  if (ok) {
    focusaPost("/telemetry/ops", { event: "daemon_kickstart_recovered", surface: "pi", reason });
  }
  return ok;
}

export async function checkFocusa(): Promise<boolean> {
  const h = await focusaFetch("/health");
  const status = h?.ok === true ? null : await focusaFetch(HEALTHCHECK_STATUS_FALLBACK_PATH);
  const fallbackHotOk = status?.status === "ok" && status?.summary_only !== false;
  const wasAvailable = S.focusaAvailable;
  S.focusaAvailable = h?.ok === true || fallbackHotOk || status?.session != null;

  if (S.focusaAvailable && h?.ok !== true && fallbackHotOk) {
    focusaPost("/telemetry/ops", {
      event: "healthcheck_hot_fallback_ok",
      surface: "pi",
      failed_route: "/v1/health",
      fallback_route: `/v1${HEALTHCHECK_STATUS_FALLBACK_PATH}`,
      route_tier: status?.route_tier || "hot",
    });
  }

  if (S.focusaAvailable) {
    S.healthFailCount = 0;
    S.healthBackoffMs = 30_000;
    S.daemonHoldoverMode = false;
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
    S.daemonHoldoverMode = true;
    // During daemon outage, probe quickly enough to recover inside the same Pi session.
    S.healthBackoffMs = Math.min(1_000 * Math.pow(2, Math.min(S.healthFailCount - 1, 4)), 15_000);
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
  if (S.continuityId) scopedQs.set("continuity_id", S.continuityId);
  if (isProjectRootAuthoritySafe(S.sessionCwd)) scopedQs.set("project_root", normalizeProjectRoot(S.sessionCwd));
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

  // Explicit frame_id can become stale after frame rescope/compaction. If the
  // scoped frame is no longer active, fall back to stack lookup so the session
  // key can find the current active Pi frame before reads/writes.
  if (frame && frame.status !== "active" && S.sessionFrameKey) {
    frame = null;
    stack = null;
  }

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

export function ensureContinuityId(cwd?: string): string {
  if (S.continuityId) return S.continuityId;
  const root = String(cwd || S.sessionCwd || process.cwd()).split("/").filter(Boolean).pop() || "root";
  let randomPart = `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;
  try { randomPart = require("crypto").randomUUID(); } catch { /* fallback above */ }
  S.continuityId = `focusa-cont-${root}-${randomPart}`.replace(/[^a-zA-Z0-9._:-]/g, "-").slice(0, 140);
  return S.continuityId;
}

function projectBeadsIssueJsonlPaths(projectRoot: string): string[] {
  return [
    join(projectRoot, ".beads", "issues.jsonl"),
    join(projectRoot, ".git", "beads-worktrees", "beads-sync", ".beads", "issues.jsonl"),
  ];
}

export function selectExistingBeadsIssueIdForFocusFrame(projectRoot: string): string | null {
  if (!isProjectRootAuthoritySafe(projectRoot)) return null;
  for (const file of projectBeadsIssueJsonlPaths(projectRoot)) {
    if (!existsSync(file)) continue;
    try {
      const lines = readFileSync(file, "utf8").split(/\r?\n/).filter(Boolean);
      const parsed = lines.map((line) => {
        try { return JSON.parse(line); } catch { return null; }
      }).filter(Boolean) as Array<{ id?: string; status?: string; priority?: number; updated_at?: string; created_at?: string }>;
      const preferred = parsed
        .filter((issue) => typeof issue.id === "string" && issue.id.trim() && issue.status !== "closed")
        .sort((a, b) => Number(a.priority ?? 4) - Number(b.priority ?? 4) || String(b.updated_at || b.created_at || "").localeCompare(String(a.updated_at || a.created_at || "")))[0]
        || parsed.find((issue) => typeof issue.id === "string" && issue.id.trim());
      if (preferred?.id) return preferred.id.trim();
    } catch { /* try next path */ }
  }
  return null;
}

export async function createPiFrame(cwd: string, source = "pi-auto"): Promise<string | null> {
  S.sessionCwd = cwd;
  const { projectName, title, goal } = derivePiFrameIntent(cwd);
  S.activeFrameTitle = title;
  S.activeFrameGoal = goal;
  const sessionKey = S.sessionFrameKey || `pi-${process.pid}-${Date.now()}`;
  S.sessionFrameKey = sessionKey;
  const continuityId = ensureContinuityId(cwd);
  const beadsIssueId = selectExistingBeadsIssueIdForFocusFrame(cwd);
  if (!beadsIssueId) {
    focusaPost("/telemetry/trace", { event_type: "pi_frame_creation_blocked_missing_beads_issue", payload: { project_root: cwd, source } });
    return null;
  }
  const tags = ["pi", projectName, source, sessionKey, continuityId, `continuity_id:${continuityId}`, "task-first-frame"];

  try {
    const r = await focusaFetch("/focus/push", {
      method: "POST",
      body: JSON.stringify({
        title,
        goal,
        beads_issue_id: beadsIssueId,
        ...(isProjectRootAuthoritySafe(cwd) ? { project_root: cwd } : {}),
        continuity_id: continuityId,
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
        (f.continuity_id === continuityId || f.tags.includes(continuityId) || f.tags.includes(`continuity_id:${continuityId}`)));
      if (match?.id) {
        S.activeFrameId = match.id;
        S.activeFrameTitle = match.title || title;
        S.activeFrameGoal = match.goal || goal;
        return match.id;
      }
    }
  } catch {}
  return null;
}


export function normalizeProjectRoot(value: unknown): string {
  const normalized = String(value || "").trim().replace(/\/+$/, "");
  return normalized === "" ? "" : normalized;
}

// QN Addendum (2026-06-08): Agent runtime paths must NEVER be treated as project scope
// Matches Rust unsafe_project_root_reason() in project.rs and workpoint.rs
const UNSAFE_PROJECT_AUTHORITY_ROOTS = new Set(["/", "/root", "/home", "/Users", "/tmp", "/var", "/usr", "/opt"]);

// Agent runtime directory patterns - must check before project markers
const AGENT_RUNTIME_PATTERNS = [
  // Pi agent
  /\/root\/pi-mono$/,
  /^\/root\/pi-/,  // /root/pi-*
  // Node/npm agent installs
  /^\/opt\/node-/,  // /opt/node-*
  /^\/usr\/local\/bin$/,
  /^\/usr\/local\/lib\/node_modules\//,
  // Claude Code
  /\/.claude\//,
  /\/.claude$/,
  // OpenCode
  /\/.opencode\//,
  /\/.opencode$/,
  // Letta
  /\/.letta\//,
  /\/.letta$/,
  // Pi config/state
  /\/.pi\//,
  /\.pi$/,
  // Python site-packages agent installs
  /\/site-packages\/letta\//,
  /\/site-packages\/open-code\//,
  /\/site-packages\/pi-coding-agent\//,
  /\/site-packages\/claude-code\//,
];

export function projectRootAuthorityFailure(value: unknown): string | null {
  const root = normalizeProjectRoot(value);
  if (!root) return "missing_project_root";
  if (UNSAFE_PROJECT_AUTHORITY_ROOTS.has(root)) return "unsafe_broad_project_root";
  if (/^\/home\/[^/]+$/.test(root) || /^\/Users\/[^/]+$/.test(root)) return "unsafe_user_home_project_root";
  // QN Addendum: Check agent runtime patterns
  for (const pattern of AGENT_RUNTIME_PATTERNS) {
    if (pattern.test(root)) return "agent_runtime_directory";
  }
  return null;
}

export function isProjectRootAuthoritySafe(value: unknown): boolean {
  return projectRootAuthorityFailure(value) === null;
}

const PROJECT_MARKERS = [
  [".focusa-project.json", 10_000],
  [".git", 9_000],
  [".beads", 6_000],
  ["Cargo.toml", 2_000],
  ["package.json", 2_000],
  ["pnpm-workspace.yaml", 2_000],
  ["bun.lockb", 1_000],
  ["yarn.lock", 1_000],
  ["package-lock.json", 1_000],
  ["pyproject.toml", 2_000],
  ["go.mod", 2_000],
  ["composer.json", 2_000],
] as const;

function projectMarkers(dir: string): { score: number; markers: string[] } {
  let score = 0;
  const markers: string[] = [];
  for (const [marker, weight] of PROJECT_MARKERS) {
    if (existsSync(join(dir, marker))) {
      score += weight;
      markers.push(marker);
    }
  }
  return { score, markers };
}

function confidenceForScore(score: number): { confidence: "high" | "medium" | "low"; confidenceScore: number } {
  if (score >= 10_000) return { confidence: "high", confidenceScore: 0.99 };
  if (score >= 8_000) return { confidence: "high", confidenceScore: 0.95 };
  if (score >= 6_000) return { confidence: "high", confidenceScore: 0.90 };
  if (score >= 2_000) return { confidence: "medium", confidenceScore: 0.75 };
  return { confidence: "low", confidenceScore: 0.25 };
}

function projectRootScoreRequiresConfirmation(confidenceScore: number): boolean {
  return confidenceScore < 0.90;
}
function findAncestorProjectRootCandidates(start: string): Array<{ root: string; score: number; depth: number; markers: string[] }> {
  let dir = normalizeProjectRoot(resolve(start || "."));
  const candidates: Array<{ root: string; score: number; depth: number; markers: string[] }> = [];
  let depth = 0;
  while (dir && dir !== "/") {
    if (isProjectRootAuthoritySafe(dir)) {
      const { score, markers } = projectMarkers(dir);
      if (score > 0) candidates.push({ root: dir, score, depth, markers });
    }
    const parent = normalizeProjectRoot(dirname(dir));
    if (!parent || parent === dir) break;
    dir = parent;
    depth += 1;
  }
  return candidates.sort((a, b) => (b.score - a.score) || (a.depth - b.depth));
}

function rootCandidatesForOutput(candidates: Array<{ root: string; score: number; markers: string[] }>): Array<{ projectRoot: string; confidenceScore: number; markers: string[]; source: string }> {
  return candidates.slice(0, 5).map(candidate => ({
    projectRoot: candidate.root,
    confidenceScore: confidenceForScore(candidate.score).confidenceScore,
    markers: candidate.markers,
    source: "ancestor_markers",
  }));
}

type ProjectRootResolution = { projectRoot: string; confidence: "high" | "medium" | "low"; confidenceScore: number; source: string; reason: string; safe: boolean; requiresOperatorConfirmation: boolean; markerScore?: number; markers?: string[]; candidates?: Array<{ projectRoot: string; confidenceScore: number; markers: string[]; source: string }> };

function rememberedProjectRootPath(): string {
  const home = process.env.HOME || homedir() || ".";
  return process.env.FOCUSA_PI_PROJECT_ROOT_CACHE || join(home, ".pi", "agent", "focusa-project-root.json");
}

function readRememberedProjectRoot(): string {
  try {
    const raw = JSON.parse(readFileSync(rememberedProjectRootPath(), "utf8"));
    return normalizeProjectRoot(raw?.project_root || raw?.projectRoot || "");
  } catch {
    return "";
  }
}

function rememberedProjectRootResolution(cwdInput?: unknown): ProjectRootResolution | null {
  const remembered = readRememberedProjectRoot();
  if (!remembered || !isProjectRootAuthoritySafe(remembered)) return null;
  const cwd = normalizeProjectRoot(cwdInput || S.sessionCwd || process.cwd());
  // Hard isolation: durable project-root cache is only a same-tree hint.
  // It must never pull a broad/ambiguous or different-project Pi session into another project.
  if (!cwd || (cwd !== remembered && !cwd.startsWith(`${remembered}/`))) return null;
  const candidates = findAncestorProjectRootCandidates(remembered);
  const exact = candidates.find(candidate => candidate.root === remembered) || candidates[0] || null;
  if (!exact || exact.root !== remembered) return null;
  const confidence = confidenceForScore(exact.score);
  return {
    projectRoot: remembered,
    ...confidence,
    source: "remembered_project_root",
    reason: `durable Pi project_root cache; markers=${exact.markers.join(",")}`,
    safe: true,
    requiresOperatorConfirmation: projectRootScoreRequiresConfirmation(confidence.confidenceScore),
    markerScore: exact.score,
    markers: exact.markers,
    candidates: [{ projectRoot: remembered, confidenceScore: confidence.confidenceScore, markers: exact.markers, source: "remembered_project_root" }],
  };
}

function rememberProjectRoot(resolution: ProjectRootResolution): void {
  if (!resolution.safe || resolution.requiresOperatorConfirmation || !isProjectRootAuthoritySafe(resolution.projectRoot)) return;
  try {
    const path = rememberedProjectRootPath();
    mkdirSync(dirname(path), { recursive: true });
    writeFileSync(path, JSON.stringify({
      schema: "focusa.pi.project_root_cache.v1",
      project_root: resolution.projectRoot,
      confidence: resolution.confidence,
      confidence_score: resolution.confidenceScore,
      source: resolution.source,
      markers: resolution.markers || [],
      updated_at: new Date().toISOString(),
    }, null, 2));
  } catch {
    // Best-effort cache only; never block Focusa session startup.
  }
}

export function resolvePiProjectRootCandidate(cwdInput?: unknown, persistedPacket?: any): ProjectRootResolution {
  const explicit = normalizeProjectRoot(cwdInput);
  const explicitCandidates = explicit ? findAncestorProjectRootCandidates(explicit) : [];
  const explicitCandidate = explicitCandidates[0] || null;
  if (explicitCandidate) {
    const confidence = confidenceForScore(explicitCandidate.score);
    return { projectRoot: explicitCandidate.root, ...confidence, source: "cwd_ancestor_markers", reason: `markers=${explicitCandidate.markers.join(",")}`, safe: true, requiresOperatorConfirmation: projectRootScoreRequiresConfirmation(confidence.confidenceScore), markerScore: explicitCandidate.score, markers: explicitCandidate.markers, candidates: rootCandidatesForOutput(explicitCandidates) };
  }

  const sessionRoot = normalizeProjectRoot(S.sessionCwd);
  const sessionCandidates = sessionRoot && sessionRoot !== explicit ? findAncestorProjectRootCandidates(sessionRoot) : [];
  const sessionCandidate = sessionCandidates[0] || null;
  if (sessionCandidate) {
    const confidence = confidenceForScore(sessionCandidate.score);
    return { projectRoot: sessionCandidate.root, ...confidence, source: "session_cwd_ancestor_markers", reason: `markers=${sessionCandidate.markers.join(",")}`, safe: true, requiresOperatorConfirmation: projectRootScoreRequiresConfirmation(confidence.confidenceScore), markerScore: sessionCandidate.score, markers: sessionCandidate.markers, candidates: rootCandidatesForOutput(sessionCandidates) };
  }

  const packet = persistedPacket?.resume_packet?.workpoint || persistedPacket?.workpoint || persistedPacket;
  const packetRoot = normalizeProjectRoot(packet?.project_root);
  const packetSessionKey = String(packet?.pi_session_frame_key || packet?.session_id || "").trim();
  const currentSessionKey = String(S.sessionFrameKey || "").trim();
  if (packetRoot && isProjectRootAuthoritySafe(packetRoot) && currentSessionKey && packetSessionKey === currentSessionKey) {
    return { projectRoot: packetRoot, confidence: "medium", confidenceScore: 0.75, source: "same_session_workpoint_packet", reason: "same-session Workpoint packet supplied project_root; operator confirmation recommended", safe: true, requiresOperatorConfirmation: true, candidates: [{ projectRoot: packetRoot, confidenceScore: 0.75, markers: ["workpoint_packet"], source: "same_session_workpoint_packet" }] };
  }

  const remembered = rememberedProjectRootResolution(explicit || sessionRoot);
  if (remembered) return remembered;

  const fallback = explicit || sessionRoot || normalizeProjectRoot(process.cwd());
  const safe = isProjectRootAuthoritySafe(fallback);
  return { projectRoot: fallback, confidence: "low", confidenceScore: 0.10, source: "unverified_cwd", reason: safe ? "no project markers found; ask operator or pass explicit project_root" : projectRootAuthorityFailure(fallback) || "unsafe_project_root", safe, requiresOperatorConfirmation: true, candidates: safe ? [{ projectRoot: fallback, confidenceScore: 0.10, markers: [], source: "unverified_cwd" }] : [] };
}

export function resolvePiProjectRoot(cwdInput?: unknown, persistedPacket?: any): string {
  return resolvePiProjectRootCandidate(cwdInput, persistedPacket).projectRoot;
}

export function projectRootConfirmationRequired(projectRoot?: string): boolean {
  const resolution = getLastProjectRootResolution();
  if (!resolution) return false;
  if (projectRoot && normalizeProjectRoot(projectRoot) !== normalizeProjectRoot(resolution.projectRoot)) return false;
  return resolution.requiresOperatorConfirmation === true || resolution.safe !== true;
}

export function projectRootConfirmationSummary(projectRoot?: string): string {
  const resolution = getLastProjectRootResolution();
  if (!resolution || (projectRoot && normalizeProjectRoot(projectRoot) !== normalizeProjectRoot(resolution.projectRoot))) return "project root is unverified";
  const candidates = (resolution.candidates || [])
    .slice(0, 3)
    .map(candidate => `${candidate.projectRoot} (${Math.round(candidate.confidenceScore * 100)}%)`)
    .join(", ");
  return `project_root=${resolution.projectRoot} confidence=${Math.round(resolution.confidenceScore * 100)}% source=${resolution.source}; ${resolution.reason}${candidates ? `; candidates: ${candidates}` : ""}`;
}

export function adoptPiProjectRoot(cwdInput?: unknown, persistedPacket?: any): string {
  const resolution = resolvePiProjectRootCandidate(cwdInput, persistedPacket);
  setLastProjectRootResolution(resolution);
  S.sessionCwd = resolution.projectRoot;
  rememberProjectRoot(resolution);
  return resolution.projectRoot;
}

export function confirmPiProjectRoot(projectRootInput: unknown, source = "operator_confirmed_project_root"): string | null {
  const projectRoot = normalizeProjectRoot(projectRootInput);
  if (!projectRoot || !isProjectRootAuthoritySafe(projectRoot)) return null;
  const base = resolvePiProjectRootCandidate(projectRoot);
  const confirmed: ProjectRootResolution = {
    ...base,
    projectRoot,
    confidence: "high",
    confidenceScore: Math.max(base.confidenceScore || 0, 0.95),
    source,
    reason: `operator confirmed project_root; ${base.reason}`,
    safe: true,
    requiresOperatorConfirmation: false,
    candidates: base.candidates?.length ? base.candidates : [{ projectRoot, confidenceScore: 0.95, markers: base.markers || ["operator_confirmed"], source }],
  };
  setLastProjectRootResolution(confirmed);
  S.sessionCwd = projectRoot;
  rememberProjectRoot(confirmed);
  return projectRoot;
}

export function normalizeWorkpointResumePacketEnvelope(packet: any): any | null {
  if (!packet || typeof packet !== "object") return null;
  const base = packet.resume_packet && typeof packet.resume_packet === "object" ? packet.resume_packet : packet;
  if (!base || typeof base !== "object") return null;
  const normalized = { ...base };
  if (packet.resume_packet_v2 && typeof packet.resume_packet_v2 === "object") normalized.resume_packet_v2 = packet.resume_packet_v2;
  if (packet.rendered_summary && !normalized.rendered_summary) normalized.rendered_summary = packet.rendered_summary;
  if (packet.schema_version && !normalized.envelope_schema_version) normalized.envelope_schema_version = packet.schema_version;
  return normalized;
}

export async function buildFocusaSessionIdentity(
  projectRootInput?: string,
  resumeSource: "session_start" | "session_switch" | "compaction" | "model_switch" | "fork" | "manual" | "unknown" = "manual",
  overrides: { continuityId?: string; sessionId?: string } = {},
): Promise<Record<string, unknown>> {
  const projectRoot = normalizeProjectRoot(projectRootInput || S.sessionCwd || process.cwd());
  const safe = isProjectRootAuthoritySafe(projectRoot);
  const ambientCwd = normalizeProjectRoot(S.sessionCwd || process.cwd());
  const ambientInsideProject = ambientCwd === projectRoot || ambientCwd.startsWith(`${projectRoot}/`);
  const cwdForIdentity = safe && !ambientInsideProject ? projectRoot : ambientCwd;
  const sessionId = String(overrides.sessionId || S.sessionFrameKey || "").trim();
  const continuityId = String(overrides.continuityId || ensureContinuityId(projectRoot || process.cwd()) || "").trim();
  let projectIdentity: any = null;
  if (safe) {
    const query = new URLSearchParams();
    query.set("cwd", cwdForIdentity);
    query.set("project_root", projectRoot);
    const persisted = getLastProjectIdentity() || {};
    if (normalizeProjectRoot(persisted.project_root) === projectRoot) {
      if (persisted.project_root) query.set("persisted_project_root", normalizeProjectRoot(persisted.project_root));
      if (persisted.fingerprint) query.set("persisted_project_fingerprint", String(persisted.fingerprint));
      if (persisted.project_id) query.set("persisted_project_id", String(persisted.project_id));
      if (persisted.canonical_name) query.set("persisted_canonical_name", String(persisted.canonical_name));
    }
    const response = await focusaFetch(`/project/identity?${query.toString()}`).catch(() => null);
    projectIdentity = response?.project_identity || null;
    if (projectIdentity) setLastProjectIdentity(projectIdentity);
  }
  const rootParts = projectRoot.split("/").filter(Boolean);
  const resolution = (getLastProjectRootResolution() && normalizeProjectRoot(getLastProjectRootResolution()!.projectRoot) === projectRoot
    ? getLastProjectRootResolution()
    : resolvePiProjectRootCandidate(projectRootInput || getSessionCwd() || process.cwd()))!;
  return {
    schema: "focusa.session_identity.v1",
    project_identity: projectIdentity,
    pi_session_id: sessionId || undefined,
    session_frame_key: sessionId || "unknown-session",
    session_incarnation_id: `${sessionId || "unknown"}:${process.pid}:${S.sessionStartTime}`,
    continuity_id: continuityId || undefined,
    project_root: projectRoot,
    cwd: cwdForIdentity,
    workspace_id: rootParts[rootParts.length - 1] || "workspace",
    process_id: process.pid,
    started_at: new Date(S.sessionStartTime).toISOString(),
    resume_source: resumeSource,
    canonical_scope: safe && !resolution.requiresOperatorConfirmation,
    scope_failure: safe ? (resolution.requiresOperatorConfirmation ? "project_root_confirmation_required" : null) : projectRootAuthorityFailure(projectRoot),
    project_root_confidence: resolution.confidence,
    project_root_confidence_score: resolution.confidenceScore,
    project_root_resolution_source: resolution.source,
    requires_operator_confirmation: resolution.requiresOperatorConfirmation,
    project_root_candidates: resolution.candidates || [],
  };
}

export async function refreshTrajectoryClarityLifecycle(reason: string, projectRootInput?: string): Promise<Record<string, unknown> | null> {
  if (!S.focusaAvailable) return null;
  const projectRoot = normalizeProjectRoot(projectRootInput || S.sessionCwd || process.cwd());
  if (!isProjectRootAuthoritySafe(projectRoot)) {
    setLastTrajectoryClarity({
      reason,
      status: "skipped_unsafe_project_root",
      project_root: projectRoot,
      scope_failure: projectRootAuthorityFailure(projectRoot),
      refreshed_at: Date.now(),
    });
    return getLastTrajectoryClarity();
  }
  const query = new URLSearchParams();
  query.set("mode", "summary");
  query.set("project_root", projectRoot);
  query.set("allow_prior_project_trajectory", "true");
  if (S.sessionFrameKey) query.set("session_id", S.sessionFrameKey);
  if (S.continuityId) query.set("continuity_id", S.continuityId);
  try {
    const view = await focusaFetch(`/trajectory/view?${query.toString()}`);
    const clarity = view?.intelligence_view?.clarity_gate || {};
    const snapshot = {
      reason,
      refreshed_at: Date.now(),
      project_root: projectRoot,
      continuity_id: S.continuityId || null,
      session_id: S.sessionFrameKey || null,
      status: String(clarity.status || view?.trajectory?.definition_status || "unknown"),
      recommended_action: String(clarity.recommended_action || view?.intelligence_view?.context_sufficiency?.recommended_action || "unknown"),
      canonical: view?.canonical === true,
      degraded: view?.degraded === true,
      project_identity_status: String(view?.project_identity?.status || "unknown"),
      trajectory_id: view?.trajectory?.trajectory_id || null,
      fallback_prior_project_trajectory: view?.trajectory?.fallback_prior_project_trajectory === true,
      fallback_source_continuity_id: view?.trajectory?.fallback_source_continuity_id || null,
      long_term_goal: view?.trajectory?.long_term_goal || null,
      desired_end_state: view?.trajectory?.desired_end_state || null,
      mid_level_goal: view?.trajectory?.mid_level_goal || view?.trajectory?.trajectory_ladder?.mlg || null,
      short_term_goal: view?.trajectory?.short_term_goal || view?.trajectory?.trajectory_ladder?.stg || null,
      waypoints: view?.trajectory?.waypoints || view?.trajectory?.trajectory_ladder?.waypoints || [],
      current_state: view?.trajectory?.current_state || null,
      active_gap: view?.trajectory?.active_gap || null,
      project_identity: view?.project_identity || null,
      project_urls: view?.project_identity?.project_urls || view?.project_identity?.project_summary?.urls || view?.project?.project_urls || null,
      deployment: view?.project_identity?.deployment || view?.project_identity?.project_summary?.deployment || view?.project?.deployment || null,
      next_tools: view?.next_tools || ["focusa_trajectory_view", "focusa_project_verify", "focusa_workpoint_resume"],
    };
    setLastTrajectoryClarity(snapshot);
    if (snapshot.project_identity) S.lastProjectIdentity = snapshot.project_identity;
    focusaPost("/telemetry/activity", {
      surface: "pi",
      event: "trajectory_clarity_refreshed",
      reason,
      project_root: projectRoot,
      status: snapshot.status,
      recommended_action: snapshot.recommended_action,
      canonical: snapshot.canonical,
      degraded: snapshot.degraded,
    });
    return snapshot;
  } catch {
    setLastTrajectoryClarity({
      reason,
      status: "unavailable",
      project_root: projectRoot,
      refreshed_at: Date.now(),
      next_tools: ["focusa_tool_doctor", "focusa_trajectory_view"],
    });
    return getLastTrajectoryClarity();
  }
}

export function clearScopedWorkpointForUnsafeCwd(reason = "unsafe_cwd_scope_guard"): void {
  setActiveWorkpointPacket(null);
  setActiveWorkpointSummary("");
  S.continuityId = "";
  S.activeFrameId = null;
  S.activeFrameTitle = "";
  S.activeFrameGoal = "";
  focusaPost("/telemetry/trace", { event_type: "pi_scope_rejected_unsafe_cwd", payload: { reason, cwd: S.sessionCwd || process.cwd() } });
}

export function stampWorkpointPacketForCurrentPiSession(packet: any): any {
  if (!packet || typeof packet !== "object") return packet;
  return {
    ...packet,
    pi_session_frame_key: S.sessionFrameKey || null,
    pi_session_scope_checked_at: new Date().toISOString(),
  };
}

export function isWorkpointPacketScopedToCurrentSession(packet: any): boolean {
  if (!packet || typeof packet !== "object") return false;
  const currentProjectRoot = resolvePiProjectRoot(S.sessionCwd || process.cwd());
  const currentContinuityId = String(S.continuityId || "").trim();
  const currentSessionKey = String(S.sessionFrameKey || "").trim();
  const packetProjectRoot = normalizeProjectRoot(packet.project_root);
  const packetContinuityId = String(packet.continuity_id || "").trim();
  const packetPiSessionKey = String(packet.pi_session_frame_key || "").trim();
  const packetSessionId = String(packet.session_id || "").trim();
  if (!currentProjectRoot || !currentContinuityId || !packetProjectRoot || !packetContinuityId) return false;
  if (!isProjectRootAuthoritySafe(currentProjectRoot) || !isProjectRootAuthoritySafe(packetProjectRoot)) return false;
  if (currentProjectRoot !== packetProjectRoot) return false;
  if (currentContinuityId !== packetContinuityId) return false;
  if (currentSessionKey && packetPiSessionKey && packetPiSessionKey !== currentSessionKey) return false;
  if (currentSessionKey && !packetPiSessionKey && packetSessionId && packetSessionId !== currentSessionKey) return false;
  if (packet.canonical === false || packet.status === "partial" || packet.status === "rejected_scope_mismatch") return false;
  return true;
}

export function getScopedWorkpointPacket(): any | null {
  return isWorkpointPacketScopedToCurrentSession(getActiveWorkpointPacket()) ? getActiveWorkpointPacket() : null;
}

export function adoptPersistedContinuityForSession(data: any, eventSessionId: string, cwd: string): void {
  const persistedSessionId = String(data?.sessionId || "").trim();
  const persistedContinuityId = String(data?.continuityId || "").trim();
  if (!persistedSessionId || persistedSessionId !== eventSessionId || !persistedContinuityId) {
    setActiveWorkpointPacket(null);
    setActiveWorkpointSummary("");
    return;
  }
  S.continuityId = persistedContinuityId;
  const packet = data?.activeWorkpointPacket || null;
  const packetProjectRoot = normalizeProjectRoot(packet?.project_root);
  const packetContinuityId = String(packet?.continuity_id || "").trim();
  if (packet && isProjectRootAuthoritySafe(cwd) && isProjectRootAuthoritySafe(packetProjectRoot) && packetProjectRoot === normalizeProjectRoot(cwd) && packetContinuityId === persistedContinuityId && packet.canonical !== false && packet.status !== "partial" && packet.status !== "rejected_scope_mismatch") {
    setActiveWorkpointPacket(stampWorkpointPacketForCurrentPiSession(packet));
    setActiveWorkpointSummary(String(data?.activeWorkpointSummary || ""));
    S.lastWorkpointUpdate = Date.now();
  } else {
    setActiveWorkpointPacket(null);
    setActiveWorkpointSummary("");
  }
}

// ── Build compact instructions with local shadow (§33.10) ────────────────────
export function buildCompactInstructions(prefix: string): string {
  const base = S.cfg?.compactInstructions || "Preserve intent, decisions, constraints, next_steps, failures.";
  const workpoint = getScopedWorkpointPacket() || {};
  const mission = String(workpoint?.mission || S.currentAsk?.text || S.activeFrameGoal || S.activeFrameTitle || "").trim();
  const nextSlice = String(workpoint?.next_slice || S.lastCompactDecision || "").trim();
  const projectRoot = String(workpoint?.project_root || (isProjectRootAuthoritySafe(S.sessionCwd) ? S.sessionCwd : "") || "").trim();
  const attentionLines = formatAttentionRecallFocusSliceLines(buildAttentionRecallVerdict({ workpointPacket: workpoint, currentAskText: S.currentAsk?.text, projectRoot }));
  const parts = [
    prefix,
    "\n" + attentionLines.join("\n"),
    "\n" + base,
    "\nFallback policy: never emit bare 'none' for Focusa Cognitive Summary fields. If a slot is empty, fill it with the nearest related canonical source: Workpoint mission/next_slice/project_root/session_id, current operator ask, active frame goal/title, local shadow decisions/constraints/failures, git/beads/evidence mentioned in the conversation. If no related source exists, say 'No recorded <field>; no safe related fallback available.'",
  ];
  if (mission) parts.push(`Fallback Mission:\n- ${mission}`);
  if (nextSlice) parts.push(`Fallback Next Step:\n- ${nextSlice}`);
  if (projectRoot) parts.push(`Fallback Scope:\n- project_root:${projectRoot}`);
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

function adoptWorkpointScopeForFrameRecovery(packet: any, source: string): string | null {
  if (!packet || typeof packet !== "object") return null;
  const workpoint = packet.resume_packet?.workpoint || packet.workpoint || packet;
  const packetProjectRoot = normalizeProjectRoot(workpoint.project_root || packet.project_root);
  const packetContinuityId = String(workpoint.continuity_id || packet.continuity_id || "").trim();
  const packetPiSessionKey = String(workpoint.pi_session_frame_key || packet.pi_session_frame_key || "").trim();
  const packetSessionId = String(workpoint.session_id || packet.session_id || "").trim();
  const currentSessionKey = String(S.sessionFrameKey || "").trim();
  const currentContinuityId = String(S.continuityId || "").trim();
  if (!packetProjectRoot || !packetContinuityId || !isProjectRootAuthoritySafe(packetProjectRoot)) return null;
  if (packet.canonical === false || workpoint.canonical === false || packet.status === "partial" || packet.status === "rejected_scope_mismatch") return null;
  if (currentContinuityId && currentContinuityId !== packetContinuityId) return null;
  if (currentSessionKey && packetPiSessionKey && packetPiSessionKey !== currentSessionKey) return null;
  if (currentSessionKey && !packetPiSessionKey && packetSessionId && packetSessionId !== currentSessionKey) return null;
  if (currentSessionKey && !packetPiSessionKey && !packetSessionId) return null;
  S.continuityId = packetContinuityId;
  setActiveWorkpointPacket(stampWorkpointPacketForCurrentPiSession(workpoint));
  S.lastWorkpointUpdate = Date.now();
  return packetProjectRoot;
}

function scopedWorkpointFrameRecoveryCwd(): string | null {
  return adoptWorkpointScopeForFrameRecovery(getActiveWorkpointPacket(), "session_scoped_workpoint");
}

async function adoptExistingSafeFrameForRecovery(): Promise<string | null> {
  const data = await loadFocusState().catch(() => null);
  const frame = data?.frame;
  const frameProjectRoot = normalizeProjectRoot(frame?.project_root);
  if (!frame?.id || !frameProjectRoot || !isProjectRootAuthoritySafe(frameProjectRoot)) return null;
  S.activeFrameId = frame.id;
  S.sessionCwd = frameProjectRoot;
  if (frame.continuity_id) S.continuityId = String(frame.continuity_id);
  return frame.id;
}

// ── Persist Focusa state to Pi session (§33.7) ──────────────────────────────
export async function ensurePiFrame(cwd?: string, sessionId?: string, source = "pi-auto"): Promise<string | null> {
  if (!S.focusaAvailable) return S.activeFrameId;

  const requestedResolution = resolvePiProjectRootCandidate(cwd || getSessionCwd() || process.cwd());
  setLastProjectRootResolution(requestedResolution);
  const requestedCwd = requestedResolution.projectRoot;
  if (requestedResolution.requiresOperatorConfirmation || requestedResolution.safe !== true) {
    focusaPost("/telemetry/trace", { event_type: "pi_frame_creation_blocked_unconfirmed_project_root", payload: { project_root: requestedCwd, summary: projectRootConfirmationSummary(requestedCwd), source } });
    return null;
  }
  let resolvedCwd = requestedCwd;
  if (!isProjectRootAuthoritySafe(resolvedCwd)) {
    const adoptedFrameId = await adoptExistingSafeFrameForRecovery();
    if (adoptedFrameId) return adoptedFrameId;
    const packetCwd = scopedWorkpointFrameRecoveryCwd();
    if (packetCwd) {
      resolvedCwd = packetCwd;
    } else {
      clearScopedWorkpointForUnsafeCwd("ensure_pi_frame_unsafe_cwd");
      return null;
    }
  }

  if (S.activeFrameId && isProjectRootAuthoritySafe(S.sessionCwd || resolvedCwd)) return S.activeFrameId;
  if (S.activeFramePromise) return await S.activeFramePromise;

  if (!isProjectRootAuthoritySafe(resolvedCwd)) return null;
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
    currentFocus: fs?.current_focus || fs?.current_state || S.lastFocusSnapshot.currentFocus || getLastTrajectoryClarity()?.short_term_goal || "",
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
    continuityId: S.continuityId,
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
    projectRootResolution: getLastProjectRootResolution(),
    activeWorkpointPacket: getScopedWorkpointPacket(),
    activeWorkpointSummary: getScopedWorkpointPacket() ? trimPersistText(getActiveWorkpointSummary()) : "",
    lastTrajectoryClarity: getLastTrajectoryClarity(),
    lastProjectIdentity: S.lastProjectIdentity,
    lastProjectVerify: getLastProjectVerify(),
    latestReportSummary: getLatestReportSummary(),
    toolOutputPressure: S.toolOutputPressure?.recapRequired ? S.toolOutputPressure : null,
    projectSwitchLedger: S.projectSwitchLedger.slice(0, PROJECT_SWITCH_LEDGER_MAX_OBSERVATIONS),
    vitalInfoPrompted: S.vitalInfoPrompted,
    lastCompactResumeKey: S.lastCompactResumeKey,
    lastCompactResumeAt: S.lastCompactResumeAt,
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

// ═══════════════════════════════════════════════════════════════════════════════
// Spec 104 — Typed Scoped Runtime Stores (PI-01 foundation)
// ═══════════════════════════════════════════════════════════════════════════════
//
// Migration path from `const S` singleton to typed scope-keyed stores.
// Each scope (project, host, workstream) gets its own TypedScopeStore instance.
// Consumers eventually read from getScopeStore() instead of S directly.
//
// The registry (ScopeStoreRegistry) is an infra-only singleton — it is NOT
// an authority-bearing global. It simply manages lifecycle of stores.

/**
 * Canonical scope identity that keys a TypedScopeStore.
 */
export interface TypedScopeIdentity {
  scopeKind: "project" | "host" | "workstream" | "unknown";
  rootPath: string;
  continuityId?: string;
  sessionId?: string;
}

/**
 * A typed, scoped runtime store containing all state relevant to one scope.
 * This replaces ad-hoc S fields with explicit typed accessors.
 */
export class TypedScopeStore {
  readonly identity: TypedScopeIdentity;

  /** Request-local scope authority: last verified identity packet */
  verifiedIdentity: null | {
    projectRoot: string;
    projectId: string;
    canonicalName: string;
    confidence: "high" | "medium" | "low";
    verifiedAt: number;
  } = null;

  /** Current authoritative scope envelope for tool calls */
  currentScopeEnvelope: null | {
    scopeKind: "project" | "host" | "workstream" | "unknown";
    projectRoot?: string;
    continuityId?: string;
    sessionId?: string;
    authority: "canonical" | "advisory" | "blocked" | "degraded";
    authorityReason?: string;
    verifiedAt: number;
  } = null;

  /** Turn/continuation state bound to this scope */
  turnCount: number = 0;

  /** Frame state bound to this scope */
  activeFrameId: string | null = null;

  /** Session identity bound to this scope */
  sessionCwd: string = "";
  continuityId: string = "";
  sessionFrameKey: string = "";

  /** Whether Focusa daemon is available for this scope */
  focusaAvailable: boolean = false;

  /** Identity shadow: last project root resolution (PI-02) */
  lastProjectRootResolution: null | {
    projectRoot: string;
    confidence: "high" | "medium" | "low";
    confidenceScore: number;
    source: string;
    reason: string;
    safe: boolean;
    requiresOperatorConfirmation: boolean;
    markerScore?: number;
    markers?: string[];
    candidates?: Array<{ projectRoot: string; confidenceScore: number; markers: string[]; source: string }>;
  } = null;

  /** Identity shadow: project identity from trajectory (PI-02) */
  lastProjectIdentity: null | Record<string, any> = null;

  /** Workpoint shadow: active workpoint packet (PI-03) */
  activeWorkpointPacket: null | Record<string, any> = null;

  /** Workpoint shadow: active workpoint summary (PI-03) */
  activeWorkpointSummary: string = "";

  /** Trajectory shadow: last clarity snapshot (PI-04) */
  lastTrajectoryClarity: null | Record<string, any> = null;

  /** Report shadow: latest report summary (PI-06) */
  latestReportSummary: null | { handle?: string; mission?: string; nextAction?: string } = null;

  /** Report shadow: last project verify result (PI-05) */
  lastProjectVerify: null | Record<string, any> = null;

  /** Turn state: task start turn (PI-07) */
  currentTaskTurnStart: number = 0;

  /** Turn state: tool usage batch (PI-07) */
  toolUsageBatch: Array<{ name: string; result: string; tookMs: number; at: number }> = [];

  /** Turn state: last stream len (PI-07) */
  lastStreamLen: number = 0;

  /** Turn state: compilation errors (PI-07) */
  compilationErrors: Array<{ file: string; line: number; message: string }> = [];

  /** Turn state: file edit counts (PI-07) */
  fileEditCounts: Record<string, number> = {};

  /** Turn state: long session warning shown (PI-07) */
  longSessionSignaled: boolean = false;

  /** Turn state: total compactions (PI-07) */
  totalCompactions: number = 0;

  /** Tool context: in tool execution scope (PI-08) */
  inToolContext: boolean = false;

  constructor(identity: TypedScopeIdentity) {
    this.identity = identity;
  }

  /** Check if this store is authoritative for a given scope identity */
  matches(scope: { rootPath: string; continuityId?: string }): boolean {
    return this.identity.rootPath === scope.rootPath;
  }

  /** Reset ephemeral state (keep identity, clear runtime data) */
  resetRuntime(): void {
    this.verifiedIdentity = null;
    this.currentScopeEnvelope = null;
    this.turnCount = 0;
    this.activeFrameId = null;
    this.sessionCwd = "";
    this.continuityId = "";
    this.sessionFrameKey = "";
    this.focusaAvailable = false;
  }
}

/**
 * ScopeStoreRegistry manages lifecycle of TypedScopeStore instances.
 *
 * This is an infra-level singleton — it is NOT authority-bearing:
 * - It only stores/retrieves stores by identity string
 * - It never decides which scope is "active"
 * - It never falls back to a remembered scope
 * - Consumers must provide an explicit identity to get a store
 */
class ScopeStoreRegistry {
  private stores = new Map<string, TypedScopeStore>();

  /** Get or create a store for the given identity. */
  getOrCreate(identity: TypedScopeIdentity): TypedScopeStore {
    const key = this.makeKey(identity);
    let store = this.stores.get(key);
    if (!store) {
      store = new TypedScopeStore(identity);
      this.stores.set(key, store);
    }
    return store;
  }

  /** Get an existing store, returning null if none. Never falls back. */
  get(identity: TypedScopeIdentity): TypedScopeStore | null {
    return this.stores.get(this.makeKey(identity)) ?? null;
  }

  /** Remove a store (scope change/close). */
  remove(identity: TypedScopeIdentity): void {
    this.stores.delete(this.makeKey(identity));
  }

  /** Clear all stores (daemon restart). */
  clearAll(): void {
    this.stores.clear();
  }

  /** Number of active stores. */
  get size(): number {
    return this.stores.size;
  }

  private makeKey(id: TypedScopeIdentity): string {
    return `${id.scopeKind}::${id.rootPath}::${id.continuityId || ""}::${id.sessionId || ""}`;
  }
}

/** Infra-only singleton registry (NOT authority-bearing — stores only, no fallback) */
export const scopeStoreRegistry = new ScopeStoreRegistry();

/**
 * Get a typed scope store for the current session scope.
 * Convenience helper that reads from the last verified scope identity.
 * Returns null if no verified scope exists (caller must handle blocked state).
 */
export function getCurrentScopeStore(): TypedScopeStore | null {
  if (!getLastProjectRootResolution() || !getSessionCwd()) return null;
  const rootPath = getLastProjectRootResolution()?.projectRoot || getSessionCwd();
  const identity: TypedScopeIdentity = {
    scopeKind: "project",
    rootPath,
    continuityId: S.continuityId || undefined,
    sessionId: S.sessionFrameKey || undefined,
  };
  return scopeStoreRegistry.getOrCreate(identity);
}

/**
 * Set the current scope authority envelope.
 * Call this after verification to record canonical scope state in the typed store.
 */
export function setCurrentScopeEnvelope(envelope: {
  scopeKind: "project" | "host" | "workstream" | "unknown";
  projectRoot?: string;
  continuityId?: string;
  sessionId?: string;
  authority: "canonical" | "advisory" | "blocked" | "degraded";
  authorityReason?: string;
}): void {
  const store = getCurrentScopeStore();
  if (!store) return;
  store.currentScopeEnvelope = { ...envelope, verifiedAt: Date.now() };
}

/**
 * Sync current S fields into the active TypedScopeStore.
 * Call this after S.sessionCwd/S.continuityId/S.sessionFrameKey are set
 * (e.g., at session start/resume) so scope-keyed consumers are consistent.
 */
export function syncSFieldsToScopeStore(): void {
  const store = getCurrentScopeStore();
  if (!store) return;
  store.sessionCwd = S.sessionCwd;
  store.continuityId = S.continuityId;
  store.sessionFrameKey = S.sessionFrameKey;
  store.turnCount = S.turnCount;
  store.activeFrameId = S.activeFrameId;
  store.focusaAvailable = S.focusaAvailable;
}

/**
 * Sync current TypedScopeStore fields back into S.
 * Call this after scope-keyed operations update the store.
 */
export function syncScopeStoreFieldsToS(): void {
  const store = getCurrentScopeStore();
  if (!store) return;
  S.sessionCwd = store.sessionCwd || S.sessionCwd;
  S.continuityId = store.continuityId || S.continuityId;
  S.sessionFrameKey = store.sessionFrameKey || S.sessionFrameKey;
  if (store.turnCount > 0) S.turnCount = store.turnCount;
  if (store.activeFrameId !== null) S.activeFrameId = store.activeFrameId;
  if (store.focusaAvailable !== S.focusaAvailable) S.focusaAvailable = store.focusaAvailable;
}

/**
 * Convenience: get turn count from scope store (preferred) or S fallback.
 */
export function getTurnCount(): number {
  const store = getCurrentScopeStore();
  return store ? store.turnCount : S.turnCount;
}

/**
 * Convenience: get active frame id from scope store (preferred) or S fallback.
 */
export function getActiveFrameId(): string | null {
  const store = getCurrentScopeStore();
  return store ? store.activeFrameId : S.activeFrameId;
}

/**
 * Convenience: get continuity id from scope store (preferred) or S fallback.
 */
export function getContinuityId(): string {
  const store = getCurrentScopeStore();
  return store ? store.continuityId : S.continuityId;
}

/**
 * Convenience: get session frame key from scope store (preferred) or S fallback.
 */
export function getSessionFrameKey(): string {
  const store = getCurrentScopeStore();
  return store ? store.sessionFrameKey : S.sessionFrameKey;
}

/**
 * Convenience: get session cwd from scope store (preferred) or S fallback.
 */
export function getSessionCwd(): string {
  const store = getCurrentScopeStore();
  return store ? store.sessionCwd : S.sessionCwd;
}

/**
 * Convenience: get focusa available flag from scope store (preferred) or S fallback.
 */
export function getFocusaAvailable(): boolean {
  const store = getCurrentScopeStore();
  return store ? store.focusaAvailable : S.focusaAvailable;
}

/**
 * PI-02: Get the last project root resolution from scope store (preferred) or S fallback.
 */
export function getLastProjectRootResolution(): TypedScopeStore["lastProjectRootResolution"] {
  const store = getCurrentScopeStore();
  return store ? store.lastProjectRootResolution : S.lastProjectRootResolution;
}

/**
 * PI-02: Set lastProjectRootResolution on both scope store and S singleton.
 */
export function setLastProjectRootResolution(
  resolution: TypedScopeStore["lastProjectRootResolution"]
): void {
  S.lastProjectRootResolution = resolution;
  const store = getCurrentScopeStore();
  if (store) store.lastProjectRootResolution = resolution;
}

/**
 * PI-02: Get lastProjectIdentity from scope store or S fallback.
 */
export function getLastProjectIdentity(): Record<string, any> | null {
  const store = getCurrentScopeStore();
  return store ? store.lastProjectIdentity : S.lastProjectIdentity;
}

/**
 * PI-02: Set lastProjectIdentity on both scope store and S singleton.
 */
export function setLastProjectIdentity(identity: Record<string, any> | null): void {
  S.lastProjectIdentity = identity;
  const store = getCurrentScopeStore();
  if (store) store.lastProjectIdentity = identity;
}

/**
 * PI-03: Get active workpoint packet from scope store (preferred) or S fallback.
 */
export function getActiveWorkpointPacket(): Record<string, any> | null {
  const store = getCurrentScopeStore();
  return store ? store.activeWorkpointPacket : getActiveWorkpointPacket();
}

/**
 * PI-03: Set activeWorkpointPacket on both scope store and S singleton.
 */
export function setActiveWorkpointPacket(packet: Record<string, any> | null): void {
  setActiveWorkpointPacket(packet);
  const store = getCurrentScopeStore();
  if (store) store.activeWorkpointPacket = packet;
}

/**
 * PI-03: Get active workpoint summary from scope store (preferred) or S fallback.
 */
export function getActiveWorkpointSummary(): string {
  const store = getCurrentScopeStore();
  return store ? store.activeWorkpointSummary : getActiveWorkpointSummary();
}

/**
 * PI-03: Set activeWorkpointSummary on both scope store and S singleton.
 */
export function setActiveWorkpointSummary(summary: string): void {
  S.activeWorkpointSummary = summary;
  const store = getCurrentScopeStore();
  if (store) store.activeWorkpointSummary = summary;
}

/** PI-04: Get lastTrajectoryClarity from scope store or S fallback. */
export function getLastTrajectoryClarity(): Record<string, any> | null {
  const store = getCurrentScopeStore();
  return store ? store.lastTrajectoryClarity : getLastTrajectoryClarity();
}

/** PI-04: Set lastTrajectoryClarity on both scope store and S. */
export function setLastTrajectoryClarity(snapshot: Record<string, any> | null): void {
  S.lastTrajectoryClarity = snapshot;
  const store = getCurrentScopeStore();
  if (store) store.lastTrajectoryClarity = snapshot;
}

/** PI-05: Get lastProjectVerify from scope store or S fallback. */
export function getLastProjectVerify(): Record<string, any> | null {
  const store = getCurrentScopeStore();
  return store ? store.lastProjectVerify : getLastProjectVerify();
}

/** PI-05: Set lastProjectVerify on both scope store and S. */
export function setLastProjectVerify(result: Record<string, any> | null): void {
  S.lastProjectVerify = result;
  const store = getCurrentScopeStore();
  if (store) store.lastProjectVerify = result;
}

/** PI-06: Get latestReportSummary from scope store or S fallback. */
export function getLatestReportSummary(): Record<string, any> | null {
  const store = getCurrentScopeStore();
  return store ? store.latestReportSummary : (getLatestReportSummary() as any) || null;
}

/** PI-06: Set latestReportSummary on both scope store and S. */
export function setLatestReportSummary(summary: Record<string, any> | null): void {
  S.latestReportSummary = summary as any;
  const store = getCurrentScopeStore();
  if (store) store.latestReportSummary = summary;
}

/** PI-07: Reset turn-scoped runtime state on the active store. */
export function resetTurnRuntimeState(): void {
  const store = getCurrentScopeStore();
  if (!store) return;
  store.toolUsageBatch = [];
  store.lastStreamLen = 0;
  store.compilationErrors = [];
  store.fileEditCounts = {};
  store.longSessionSignaled = false;
}

/** PI-07: Increment totalCompactions counter on both scope store and S. */
export function incrementTotalCompactions(): void {
  S.totalCompactions++;
  const store = getCurrentScopeStore();
  if (store) store.totalCompactions = S.totalCompactions;
}

/** PI-07: Get current task turn start from scope store or S fallback. */
export function getCurrentTaskTurnStart(): number {
  const store = getCurrentScopeStore();
  return store ? store.currentTaskTurnStart : S.currentTaskTurnStart;
}

/** PI-07: Set current task turn start on scope store and S. */
export function setCurrentTaskTurnStart(v: number): void {
  S.currentTaskTurnStart = v;
  const store = getCurrentScopeStore();
  if (store) store.currentTaskTurnStart = v;
}

/** PI-07: Get last stream length from scope store or S fallback. */
export function getLastStreamLen(): number {
  const store = getCurrentScopeStore();
  return store ? store.lastStreamLen : S.lastStreamLen;
}

/** PI-07: Set last stream length on scope store and S. */
export function setLastStreamLen(v: number): void {
  S.lastStreamLen = v;
  const store = getCurrentScopeStore();
  if (store) store.lastStreamLen = v;
}

/** PI-08: Get in-tool-context flag from scope store (no S fallback — new field). */
export function getInToolContext(): boolean {
  const store = getCurrentScopeStore();
  return store ? store.inToolContext : false;
}

/** PI-08: Set in-tool-context flag on scope store only (no S fallback — new field). */
export function setInToolContext(v: boolean): void {
  const store = getCurrentScopeStore();
  if (store) store.inToolContext = v;
}

/**
 * Reset all scope stores (e.g., on daemon restart or scope purge).
 */
export function resetAllScopeStores(): void {
  scopeStoreRegistry.clearAll();
}
