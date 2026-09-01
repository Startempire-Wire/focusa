// Shared state, helpers, types for focusa-pi-bridge
// Spec: docs/44-pi-focusa-integration-spec.md

import { AsyncLocalStorage } from "async_hooks";
import { SPEC138_OPERATIONS } from "./generated/spec138-operations.js";
import { appendFileSync, existsSync, mkdirSync, readFileSync, readdirSync } from "fs";
import { dirname, join, resolve } from "path";
import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { DEFAULT_DAEMON_RESTART_COMMAND, type FocusaConfig } from "./config.js";
import type { NativeSessionPressureV1 } from "./session-pressure.js";
import {
  buildProjectWorkstreamKey,
  registerVerifiedScopeRef,
  verifiedScopeRefForRoot,
  type AttachmentKey,
} from "./scoped-state.js";
import { projectBindingAllowsDurableWrites, type ProjectBindingDecisionV1 } from "./project-binding.js";
import {
  resolveCanonicalMarkerProjectRoot,
  resolveProjectIdentityLookupCwd,
} from "./project-identity-working-context.js";
import {
  COMPACTION_PERSISTENCE_ANCHOR_REF_SCHEMA,
  COMPACTION_PERSISTENCE_ANCHOR_SCHEMA,
  NATIVE_ANCHOR_MAX_BYTES,
  PROJECT_SWITCH_ANCHOR_MAX_BYTES,
  loadPersistedRecoveryState,
  semanticPersistenceDigest,
  stableSemanticValue,
  writeRecoverySidecar,
} from "./persistence.js";
export {
  COMPACTION_PERSISTENCE_ANCHOR_REF_SCHEMA,
  COMPACTION_PERSISTENCE_ANCHOR_SCHEMA,
  NATIVE_ANCHOR_MAX_BYTES,
  PROJECT_SWITCH_ANCHOR_MAX_BYTES,
  loadPersistedRecoveryState,
  semanticPersistenceDigest,
} from "./persistence.js";

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
  durable_project_write_authority: boolean;
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
  current_ask_scope: {
    project_alias: string;
    project_root: string;
    confidence: number;
    evidence_ref: string;
  };
  action_authority_for_current_ask: boolean;
  durable_project_write_authority: boolean;
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

// ── Typed attachment runtime state ───────────────────────────────────────────
function createAttachmentRuntime() {
  return {
    pi: null as ExtensionAPI | null,
    cfg: null as FocusaConfig | null,
    focusaAvailable: false,
    activeFrameId: null as string | null,
    activeFramePromise: null as Promise<string | null> | null,
    activeFrameTitle: "" as string,
    activeFrameGoal: "" as string,
    uiCtx: null as any, // §93: SSE handler needs ctx.ui for high-priority agent alerts
    sessionFrameKey: "" as string,
    sessionCwd: "" as string,
    continuityId: "" as string,
    startupReceptionistActive: false,
    startupReceptionistStartedAt: 0,
    startupReceptionistPreviousThinkingLevel: "" as string,
    modelProvider: "" as string,
    modelId: "" as string,
    providerUsagePercent: null as number | null,
    providerRenewalAt: "" as string,
    providerUsageObservedAt: 0 as number,
    wbmEnabled: false,
    wbmDeep: false,
    wbmNoCatalogue: false, // §29 --no-catalogue flag
    // turnCount migrated to scope store (PI-07, removed from singleton)
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
    activeContextWindow: 200_000, // claude-opus-4-6 has 200K window; updated on model_select events
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
    northStarSnapshot: null as import("./north-star.js").NorthStarSnapshot | null,
    // lastStreamLen migrated to scope store (PI-07, removed from singleton)
    // Compaction delivery arbiter. Pi owns the native queue and next turn;
    // Focusa only persists a bounded next-turn projection/outcome.
    compactResumePending: false,
    compactionVerifyPendingKey: "",
    compactResumeDeliveryKey: "",
    compactResumeDeliveryState: "none" as
      | "none"
      | "pending"
      | "deferred_to_next_turn"
      | "superseded_by_operator"
      | "failed"
      | "unknown_completion",
    // Persisted compaction resume idempotency guard.
    lastCompactResumeKey: "",
    lastCompactResumeAt: 0,
    // Post-compaction: save last decision for steer message (cleared after localDecisions trim)
    lastCompactDecision: "",
    // Spec88/104 Workpoint, Trajectory, and identity shadows live in TypedScopeStore only.
    // Do not add singleton fallbacks for activeWorkpointPacket, activeWorkpointSummary,
    // lastTrajectoryClarity, lastProjectIdentity, or lastProjectVerify.
    // latestReportSummary migrated to scope store (PI-06, removed from singleton)
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
    // Non-triggering lifecycle advisories are injected into the next user-turn
    // tail; they never start an agent run or race an operator prompt.
    pendingLifecycleAdvisories: {} as Record<
      string,
      { key: string; text: string; reason: string; createdAt: number }
    >,
    sessionProjectClassification: "unknown" as
      | "unknown"
      | "new_session_new_project"
      | "new_session_existing_project"
      | "resumed_session_resumed_project"
      | "resumed_session_recoverable_project"
      | "resumed_session_worktree_rebound"
      | "session_project_mismatch"
      | "forked_compacted_continuation",
    piSessionProjectRegistry: {} as Record<
      string,
      {
        project_root: string;
        continuity_id: string;
        latest_workpoint_id?: string;
        classification: string;
        last_seen_at: number;
        provenance: string;
      }
    >,
    projectBindingDecisions: {} as Record<string, ProjectBindingDecisionV1>,
    projectBindingTelemetry: {
      startup_count: 0,
      automatic_resolution_count: 0,
      operator_interruption_count: 0,
      false_bind_count: 0,
      recovery_duration_ms: 0,
      blocked_write_reasons: {} as Record<string, number>,
    },
    // First-turn guard: only inject behavioral directive once per session, not on every before_agent_start
    seenFirstBeforeAgentStart: false,
    // ECS handle registry: kind -> id -> { content, stored_at }
    ecsRegistry: {} as Record<string, Record<string, { content: string; storedAt: number }>>,
    // toolUsageBatch migrated to scope store (PI-07, removed from singleton)
    // Spec92 bounded hook/token telemetry (in-memory Pi extension ring buffers)
    spec92HookTelemetry: [] as Array<Record<string, unknown>>,
    spec92TokenTelemetry: [] as Array<Record<string, unknown>>,
    spec92ToolStartTimes: {} as Record<string, number>,
    // FOCUSA_FIX-a52s: shell-tool reminder frequency gate state.
    lastShellReminderAt: 0 as number,
    lastShellReminderTurn: 0 as number,
    // compilationErrors/fileEditCounts migrated to scope store (PI-07, removed from singleton)
    // Session/task timing + token accounting
    sessionStartTime: Date.now(),
    currentTaskStartTime: Date.now(),
    currentTaskLabel: "",
    // currentTaskTurnStart migrated to scope store (PI-07, removed from singleton)
    currentTaskInputTokenEstimate: 0,
    currentTaskOutputTokenEstimate: 0,
    currentTaskProviderInputTokens: 0,
    currentTaskProviderOutputTokens: 0,
    currentTaskToolCalls: 0,
    // longSessionSignaled migrated to scope store (PI-07, removed from singleton)
    // WBM cataloguing (§29)
    cataloguedDecisions: [] as string[],
    cataloguedFacts: [] as string[],
    // Health (§38.3)
    healthInterval: null as ReturnType<typeof setInterval> | null,
    // Footer/session-title sync cadence (keeps Pi footer task label fresh between commands)
    footerSyncInterval: null as ReturnType<typeof setInterval> | null,
    healthBackoffMs: 30_000, // §11 exponential backoff
    healthFailCount: 0,
    // docs/165 + #496 — visual background state is attachment-local. A
    // process-global map leaks one Pi session's jobs into another transcript.
    backgroundJobs: {
      running: new Map<string, { name: string; startedAt: string }>(),
      recent: [] as Array<{ name: string; status: string; exitCode: number | null }>,
    },
    bgSeeded: false,
    daemonRestartAttempts: [] as number[],
    daemonRestartInFlight: null as Promise<boolean> | null,
    daemonHoldoverMode: false,
    // Outage audit (§11)
    outageStart: null as number | null,
    // §30 metacognitive indicators
    lastMetacogEvent: "",
    // totalCompactions migrated to scope store (PI-07, removed from singleton)
    // Fork suggestion dedup (§18 autoSuggestForkPct)
    forkSuggested: false,
    // Persistence dedup/throttle for appendEntry pressure
    lastPersistAt: 0,
    lastPersistHash: "",
    persistRevision: 0,
    pendingPersistAnchor: false,
    lastPersistSidecarKey: "",
    lastPersistSidecarBytes: 0,
    lastProjectSwitchPersistHash: "",
    lastNativeSessionPressure: null as NativeSessionPressureV1 | null,
    lastNativeSessionPressureNoticeKey: "",
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
    // §5.12 recent-turns ring buffer (bounded, capped at RECENT_TURNS_HARD_CAP)
    recentTurns: [] as RecentTurnSlice[],
    // §5.12 idempotency guard — last turn_index we emitted the slice for
    lastRecentTurnsSliceTurn: -1,
  };
}

export type AttachmentRuntimeState = ReturnType<typeof createAttachmentRuntime>;

function attachmentRuntimeKey(key: AttachmentKey): string {
  return [
    key.workstream.root_scope.scope_kind,
    key.workstream.root_scope.scope_id,
    key.workstream.root_scope.fingerprint,
    key.workstream.root_scope.root_path,
    key.workstream.continuity_id,
    key.instance_id,
    key.session_id,
    key.attachment_id,
  ].join("::");
}

export class AttachmentRuntimeRegistry {
  private readonly runtimes = new Map<string, AttachmentRuntimeState>();
  private readonly boundAttachmentsBySession = new Map<string, AttachmentKey>();

  getOrCreate(key: AttachmentKey): AttachmentRuntimeState {
    const id = attachmentRuntimeKey(key);
    let runtime = this.runtimes.get(id);
    if (!runtime) {
      runtime = createAttachmentRuntime();
      runtime.sessionCwd = key.workstream.root_scope.root_path;
      runtime.continuityId = key.workstream.continuity_id;
      runtime.sessionFrameKey = key.session_id;
      this.runtimes.set(id, runtime);
    }
    return runtime;
  }

  bindSessionAttachment(key: AttachmentKey): void {
    const root = key.workstream.root_scope.root_path;
    const continuity = key.workstream.continuity_id;
    if (!isProjectRootAuthoritySafe(root) || !continuity || continuity === "extension-bootstrap") return;
    this.boundAttachmentsBySession.set(key.session_id, key);
  }

  promoteRuntime(source: AttachmentKey, target: AttachmentKey): AttachmentRuntimeState {
    if (source.session_id !== target.session_id) {
      throw new Error("attachment_runtime_session_mismatch");
    }
    const sourceId = attachmentRuntimeKey(source);
    const targetId = attachmentRuntimeKey(target);
    const sourceRuntime = this.getOrCreate(source);
    if (sourceId === targetId) return sourceRuntime;
    const existingTarget = this.runtimes.get(targetId);
    if (existingTarget && existingTarget !== sourceRuntime) {
      throw new Error("attachment_runtime_target_already_exists");
    }
    this.runtimes.delete(sourceId);
    this.runtimes.set(targetId, sourceRuntime);
    return sourceRuntime;
  }

  boundSessionAttachment(sessionId: string): AttachmentKey | undefined {
    return this.boundAttachmentsBySession.get(sessionId);
  }

  reset(): void {
    this.runtimes.clear();
    this.boundAttachmentsBySession.clear();
    delete process.env.FOCUSA_ATTACHMENT_KEY_V1;
  }
}

export const attachmentRuntimeRegistry = new AttachmentRuntimeRegistry();
const attachmentRuntimeContext = new AsyncLocalStorage<AttachmentKey>();

export function makeSessionBootstrapAttachmentKey(sessionId: string): AttachmentKey {
  const boundedSessionId = String(sessionId || "").trim();
  if (!boundedSessionId) throw new Error("attachment_runtime_session_required");
  return {
    workstream: {
      root_scope: {
        scope_kind: "host",
        scope_id: "host:pi-extension-bootstrap",
        root_path: "/",
        canonical_name: "Pi Extension Bootstrap",
        fingerprint: "bootstrap:pi-extension",
      },
      continuity_id: "extension-bootstrap",
    },
    instance_id: `pi-${process.pid}`,
    session_id: boundedSessionId,
    attachment_id: `extension-bootstrap:${boundedSessionId}`,
  };
}

export function makeAttachmentKey(input: {
  projectRoot: string;
  continuityId: string;
  sessionId: string;
  instanceId?: string;
  attachmentId?: string;
}): AttachmentKey {
  return {
    workstream: buildProjectWorkstreamKey(input.projectRoot, input.continuityId),
    instance_id: input.instanceId || `pi-${process.pid}`,
    session_id: input.sessionId,
    attachment_id: input.attachmentId || input.sessionId,
  };
}

export function currentAttachmentKey(): AttachmentKey | undefined {
  return attachmentRuntimeContext.getStore();
}

/**
 * Promote one verified Pi session from its private host bootstrap runtime into
 * the exact project/workstream attachment. Re-key the same runtime object so
 * recovered session state survives; never mutate the global bootstrap key.
 */
export function promoteCurrentSessionAttachment(input: {
  projectRoot: string;
  continuityId: string;
  sessionId: string;
}): AttachmentKey {
  const current = currentAttachmentKey();
  if (!current) throw new Error("attachment_runtime_key_required");
  if (current.session_id !== input.sessionId) throw new Error("attachment_runtime_session_mismatch");
  if (!isProjectRootAuthoritySafe(input.projectRoot)) {
    throw new Error("attachment_runtime_safe_project_required");
  }
  if (!input.continuityId || input.continuityId === "extension-bootstrap") {
    throw new Error("attachment_runtime_continuity_required");
  }
  if (!verifiedScopeRefForRoot(input.projectRoot)) {
    throw new Error("attachment_runtime_verified_scope_required");
  }
  const promoted = makeAttachmentKey(input);
  const runtime = attachmentRuntimeRegistry.promoteRuntime(current, promoted);
  runtime.sessionCwd = input.projectRoot;
  runtime.continuityId = input.continuityId;
  runtime.sessionFrameKey = input.sessionId;
  attachmentRuntimeRegistry.bindSessionAttachment(promoted);
  attachmentRuntimeContext.enterWith(promoted);
  process.env.FOCUSA_ATTACHMENT_KEY_V1 = JSON.stringify(promoted);
  return promoted;
}

export function clearPublishedAttachmentEnvironment(sessionId?: string): boolean {
  const raw = process.env.FOCUSA_ATTACHMENT_KEY_V1;
  if (!raw) return false;
  if (sessionId) {
    try {
      const published = JSON.parse(raw) as Partial<AttachmentKey>;
      if (published.session_id !== sessionId) return false;
    } catch {
      // Malformed process-local routing state carries no authority and is cleared.
    }
  }
  delete process.env.FOCUSA_ATTACHMENT_KEY_V1;
  return true;
}

export function getAttachmentRuntime(key?: AttachmentKey): any {
  const resolved = key || attachmentRuntimeContext.getStore();
  if (!resolved) throw new Error("attachment_runtime_key_required");
  return attachmentRuntimeRegistry.getOrCreate(resolved);
}

export function runWithAttachmentRuntime<T>(key: AttachmentKey, fn: () => T): T {
  return attachmentRuntimeContext.run(key, fn);
}

const FOCUS_STATE_CACHE_TTL_MS = 1_200;
const AUX_CONTEXT_CACHE_TTL_MS = 3_000;
const CONTEXT_SEMANTIC_LIMIT = 64;
const CONTEXT_ECS_HANDLES_LIMIT = 128;
const HEALTHCHECK_STATUS_FALLBACK_PATH = "/status?summary_only=true";

// ─── §5.12 Recent Turns Ring Buffer ──────────────────────────────────────
// Spec: docs/101-focusa-bloatgaurd-spec.md §5.12
// Bounded, deduplicated orientation slice emitted after compaction / model switch
// to prevent the agent from re-reading prior tool outputs.

export const RECENT_TURNS_HARD_CAP = 8;
export const RECENT_TURNS_N_DEFAULT = 4;

export interface RecentTurnSlice {
  turn_id: string;
  mission_at_turn: string;
  outcome: "committed" | "filed_bead" | "observed" | "blocked" | "ack" | "tooled";
  evidence_refs: string[];
  tool_call_count: number;
  emitted_at: number;
}

export function classifyTurnOutcome(
  toolCallCount: number,
  hasNonTaskText: boolean,
  hasFailureSignal: boolean
): RecentTurnSlice["outcome"] {
  if (hasFailureSignal) return "blocked";
  if (toolCallCount === 0 && hasNonTaskText) return "ack";
  if (toolCallCount === 0) return "observed";
  return "tooled";
}

export function pushRecentTurn(slice: RecentTurnSlice): void {
  getAttachmentRuntime().recentTurns.push(slice);
  while (getAttachmentRuntime().recentTurns.length > RECENT_TURNS_HARD_CAP) {
    getAttachmentRuntime().recentTurns.shift();
  }
}

export function clearRecentTurns(): void {
  getAttachmentRuntime().recentTurns = [];
  getAttachmentRuntime().lastRecentTurnsSliceTurn = -1;
}

export function getRecentTurns(): RecentTurnSlice[] {
  return getAttachmentRuntime().recentTurns;
}

export function shouldEmitRecentTurnsSlice(currentTurnCount: number): boolean {
  if (currentTurnCount < 1) return false;
  if (getAttachmentRuntime().lastRecentTurnsSliceTurn === currentTurnCount) return false;
  return true;
}

export function markRecentTurnsSliceEmitted(currentTurnCount: number): void {
  getAttachmentRuntime().lastRecentTurnsSliceTurn = currentTurnCount;
}

function truncate(s: string, n: number): string {
  if (s.length <= n) return s;
  return s.slice(0, Math.max(0, n - 1)) + "\u2026";
}

export function formatRecentTurnsSection(n: number = RECENT_TURNS_N_DEFAULT): string {
  const cap = Math.max(0, Math.min(RECENT_TURNS_HARD_CAP, Math.floor(n)));
  if (cap === 0) return "";
  const recent = getAttachmentRuntime().recentTurns.slice(-cap).reverse();
  if (recent.length === 0) return "";
  const lines = ["Recent turns (last " + recent.length + "):"];
  for (const t of recent) {
    const refs = t.evidence_refs.length > 0 ? ` ev=${t.evidence_refs.join(",")}` : "";
    lines.push(
      `- T[${t.turn_id}] mission="${truncate(t.mission_at_turn, 120)}" outcome=${t.outcome} tools=${t.tool_call_count}${refs}`
    );
  }
  return lines.join("\n");
}

export interface CacheableSplit {
  stable: string;
  variable: string;
  cache_hint_supported: boolean;
}

/**
 * Conservative split of the injected system prompt into a stable prefix
 * (cacheable upstream) and a variable tail. Stable blocks MUST NOT contain
 * per-turn timestamps, paths, or tool output \u2014 otherwise the provider
 * invalidates the cache.
 */
export function splitCacheableSystemPrompt(systemPrompt: string): CacheableSplit {
  const boundaryMarkers = ["## Recent turns", "## Tool Result Tail", "## Active Step"];
  for (const marker of boundaryMarkers) {
    const idx = systemPrompt.indexOf(marker);
    if (idx > 0) {
      return {
        stable: systemPrompt.slice(0, idx),
        variable: systemPrompt.slice(idx),
        cache_hint_supported: false,
      };
    }
  }
  return { stable: systemPrompt, variable: "", cache_hint_supported: false };
}

export function resetPiSessionScopedState(reason = "session_boundary"): void {
  getAttachmentRuntime().seenFirstBeforeAgentStart = false;
  getAttachmentRuntime().seenFirstBeforeAgentStart = false;
  getAttachmentRuntime().activeFrameId = null;
  getAttachmentRuntime().activeFramePromise = null;
  getAttachmentRuntime().activeFrameTitle = "";
  getAttachmentRuntime().activeFrameGoal = "";
  getAttachmentRuntime().continuityId = "";
  setActiveWorkpointPacket(null);
  setActiveWorkpointSummary("");
  setLastTrajectoryClarity(null);
  setLastProjectIdentity(null);
  setLatestReportSummary(null);
  resetToolOutputPressureWindow(Date.now());
  getAttachmentRuntime().projectSwitchLedger = [];
  getAttachmentRuntime().currentAsk = null;
  getAttachmentRuntime().queryScope = null;
  getAttachmentRuntime().excludedContext = null;
  getAttachmentRuntime().northStarSnapshot = null;
  getAttachmentRuntime().lastFocusSnapshot = {
    decisions: [],
    constraints: [],
    failures: [],
    intent: "",
    currentFocus: "",
  };
  getAttachmentRuntime().localDecisions = [];
  getAttachmentRuntime().localConstraints = [];
  getAttachmentRuntime().localFailures = [];
  getAttachmentRuntime().lastCompactTime = 0;
  getAttachmentRuntime().compactsThisHour = 0;
  getAttachmentRuntime().turnsSinceCompact = 0;
  getAttachmentRuntime().compactHourStart = Date.now();
  getAttachmentRuntime().currentTier = "";
  getAttachmentRuntime().currentContextPct = null;
  getAttachmentRuntime().compactResumePending = false;
  getAttachmentRuntime().compactionVerifyPendingKey = "";
  getAttachmentRuntime().compactResumeDeliveryKey = "";
  getAttachmentRuntime().compactResumeDeliveryState = "none";
  getAttachmentRuntime().lastCompactResumeKey = "";
  getAttachmentRuntime().lastCompactResumeAt = 0;
  getAttachmentRuntime().lastCompactDecision = "";
  getAttachmentRuntime().spec92HookTelemetry = [];
  getAttachmentRuntime().spec92TokenTelemetry = [];
  getAttachmentRuntime().spec92ToolStartTimes = {};
  // compilationErrors/fileEditCounts/longSessionSignaled migrated to scope store (PI-07)
  getAttachmentRuntime().cataloguedDecisions = [];
  getAttachmentRuntime().cataloguedFacts = [];
  // totalCompactions migrated to scope store (PI-07, removed from singleton)
  getAttachmentRuntime().forkSuggested = false;
  getAttachmentRuntime().focusStateCache = { key: "", at: 0, data: null, inflight: null };
  getAttachmentRuntime().semanticMemoryCache = { at: 0, data: null, inflight: null };
  getAttachmentRuntime().ecsHandlesCache = { at: 0, data: null, inflight: null };
  getAttachmentRuntime().lastPersistAt = 0;
  getAttachmentRuntime().lastPersistHash = "";
  getAttachmentRuntime().persistRevision = 0;
  getAttachmentRuntime().pendingPersistAnchor = false;
  getAttachmentRuntime().lastPersistSidecarKey = "";
  getAttachmentRuntime().lastPersistSidecarBytes = 0;
  getAttachmentRuntime().lastProjectSwitchPersistHash = "";
  getAttachmentRuntime().lastNativeSessionPressure = null;
  getAttachmentRuntime().lastNativeSessionPressureNoticeKey = "";
  getAttachmentRuntime().wbmEnabled = false;
  getAttachmentRuntime().wbmDeep = false;
  getAttachmentRuntime().wbmNoCatalogue = false;
  focusaPost("/telemetry/trace", {
    event_type: "pi_session_scoped_state_reset",
    payload: {
      reason,
      session_id: getAttachmentRuntime().sessionFrameKey,
      cwd: getAttachmentRuntime().sessionCwd,
    },
  });
}

// ── Typed Work Loop compatibility ───────────────────────────────────────────
const WORK_LOOP_STATUS_SCHEMA = "focusa.work_loop_status.v3";
const WORK_LOOP_TYPED_STATES = new Set([
  "absent",
  "unavailable",
  "stale",
  "unsupported",
  "blocked",
  "exhausted",
  "zero",
  "healthy",
]);

export function compatibleWorkLoopStatusState(payload: any): string {
  const state = String(payload?.state || "").trim();
  return payload?.schema === WORK_LOOP_STATUS_SCHEMA && WORK_LOOP_TYPED_STATES.has(state)
    ? state
    : "unsupported";
}

// ── HTTP helper ──────────────────────────────────────────────────────────────
export async function focusaFetch(path: string, opts: RequestInit = {}): Promise<any> {
  // Settings callbacks can run outside Pi's attachment async context. Global routes
  // remain usable there; scoped routes receive no authority headers and must reject
  // safely at the daemon rather than crashing Pi with attachment_runtime_key_required.
  const attachment = currentAttachmentKey();
  const runtime = attachment ? getAttachmentRuntime(attachment) : null;
  const timeout = runtime?.cfg?.focusaApiTimeoutMs || 5000;
  const base = runtime?.cfg?.focusaApiBaseUrl || "http://127.0.0.1:8787/v1";
  const token = runtime?.cfg?.focusaToken || "";
  const root = attachment?.workstream.root_scope.root_path || "";
  const continuity = attachment?.workstream.continuity_id || "";
  const typedScopeHeaders: Record<string, string> =
    attachment && isProjectRootAuthoritySafe(root) && continuity && continuity !== "extension-bootstrap"
      ? {
          "X-Scope-Project-Root": root,
          "X-Scope-Continuity-Id": continuity,
          "X-Scope-Session-Id": attachment.session_id,
        }
      : {};
  const mutationMethod = String(opts.method || "GET").toUpperCase();
  let idempotencyHeader: Record<string, string> = {};
  if (["POST", "PUT", "PATCH", "DELETE"].includes(mutationMethod) && typeof opts.body === "string") {
    try {
      const body = JSON.parse(opts.body) as Record<string, unknown>;
      const key = body.idempotency_key ?? body.idempotencyKey ?? body.request_id ?? body.requestId;
      if (typeof key === "string" && key.trim()) {
        idempotencyHeader = { "Idempotency-Key": key.trim() };
      }
    } catch {
      // The daemon JSON guard rejects malformed mutation bodies before handlers.
    }
  }
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
          "X-Extension-Token": `focusa-pi-${runtime?.cfg?.focusaExtensionBuild || "v0"}`,
          ...(token ? { Authorization: `Bearer ${token}` } : {}),
          ...typedScopeHeaders,
          ...idempotencyHeader,
          ...((opts.headers as Record<string, string>) || {}),
        },
        signal: ac.signal,
      });
      if (r.ok) return await r.json();
      if (r.status === 403) {
        const blocked = await r.json().catch(() => null);
        const code = String(blocked?.error?.code || "");
        if (code.startsWith("ENTITLEMENT_")) {
          return {
            ok: false,
            status: "blocked",
            failure_class: "entitlement_blocked",
            error: {
              code,
              state: blocked?.error?.state || "recovery_only",
              required_feature: blocked?.error?.required_feature || null,
              limit_bucket: blocked?.error?.limit_bucket || null,
              recovery: blocked?.error?.recovery || {
                status_path: "/v1/license/status",
                action: "recovery_only",
                allowed: [
                  "health",
                  "version",
                  "license_status",
                  "export",
                  "diagnostics",
                  "repair",
                  "update_for_recovery",
                  "uninstall",
                  "safe_read",
                ],
              },
            },
          };
        }
        return null;
      }
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
  return /\[focusa-context\]|#\s*focusa context|rendered live from focusa-pi-bridge current state\.?|current focus frame:|\bgoal:\b/i.test(
    String(text || "")
  );
}

function isContaminatedFrameIdentity(frame: any): boolean {
  const title = String(frame?.title || "");
  const goal = String(frame?.goal || "");
  return hasQuotedFocusaPayload(title) || hasQuotedFocusaPayload(goal);
}

function isFocusaPayloadWrapperText(text: string): boolean {
  const normalized = String(text || "")
    .replace(/\s+/g, " ")
    .trim()
    .toLowerCase();
  if (!normalized) return false;
  if (
    /^(restarted again,?\s*)?(still wrong|wrong|not true|this is|this output|this context|look|see|after restart|same issue|again)[:\s]*$/.test(
      normalized
    )
  )
    return true;
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
  // Focusa advisories are injected control-plane context, not operator scope evidence.
  // Strip the whole advisory before project-alias inference to prevent recursive scope poisoning.
  stripped = stripped.replace(/\[\s*focusa advisory[^\]]*\][\s\S]*$/i, "");
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
    pattern:
      /\[focusa focus slice|\bprojection_kind:\b|\bview_profile:\b|\bquery_scope:\b|\bcanonical_sources:\b|\bworking_set:\b|\bverified_deltas:\b/i,
  },
  {
    class_id: "internal_routing_reasons",
    description: "Internal routing/selection reason labels leaked into visible assistant text",
    pattern:
      /\brelevant_context_selected\b|\birrelevant_context_excluded\b|\bprior_mission_reused\b|\bquery_scope_built\b|\bsubject_hijack_prevented\b/i,
  },
  {
    class_id: "metacognitive_prose",
    description: "Internal metacognitive/planner phrasing leaked into visible assistant text",
    pattern:
      /\bminimal_focus_slice_builder\b|\bconsultation trace\b|\bfocusa cognitive guidance\b|\boperator-first routing\b/i,
  },
  {
    class_id: "hidden_trace_dimensions",
    description: "Hidden trace/event dimensions leaked into visible assistant text",
    pattern:
      /\bfocus_slice_relevance_score\b|\bresolved_reference_count\b|\bselected_counts\b|\bprojection_boundary\b|\bcanonical_sources\b/i,
  },
  {
    class_id: "reducer_internal_state",
    description: "Reducer/daemon internal state identifiers leaked into visible assistant text",
    pattern:
      /\bactive_writer\b|\bpause_flags\b|\blast_recorded_bd_transition_id\b|\btransport_session_state\b|\bwork_loop\.run\b|\bstate\.version\b/i,
  },
] as const;

export function detectForbiddenVisibleOutputLeakClasses(text: string): string[] {
  const normalized = stripQuotedFocusaContext(String(text || "")).trim();
  if (!normalized) return [];
  return FORBIDDEN_VISIBLE_OUTPUT_LEAK_CLASSES.filter((entry) => entry.pattern.test(normalized)).map(
    (entry) => entry.class_id
  );
}

export function isNonTaskStatusLikeText(text: string): boolean {
  const normalized = String(text || "")
    .replace(/\s+/g, " ")
    .trim();
  if (!normalized) return false;
  if (/^\//.test(normalized)) return true;
  if (/^#\s*focusa context\b/i.test(normalized)) return true;
  if (/^rendered live from focusa-pi-bridge current state\.?/i.test(normalized)) return true;
  if (/^focusa:\s/i.test(normalized) && /(frame:|title:|goal:|wbm:|turns:|config:)/i.test(normalized))
    return true;
  if (hasQuotedFocusaPayload(normalized)) return !stripQuotedFocusaContext(normalized);
  return false;
}

export function classifyCurrentAsk(text: string): PiCurrentAskKind {
  const cleaned = stripQuotedFocusaContext(text);
  const lower = cleaned.trim().toLowerCase();
  if (isNonTaskStatusLikeText(text)) return "meta";
  if (!lower) return hasQuotedFocusaPayload(text) ? "meta" : "unknown";
  if (
    /^(no\b|undo\b|revert\b|wrong\b|that's incorrect\b|not what i asked\b|stop\b|instead\b|ignore previous\b|new task\b|different task\b|go back\b|don't\b)/i.test(
      lower
    )
  )
    return "correction";
  if (
    lower.endsWith("?") ||
    /^(what|why|how|when|where|who|which|can|could|should|is|are|do|does|did)\b/.test(lower)
  )
    return "question";
  if (/^(note|remember|fyi|for context|meta|discussion:)\b/.test(lower)) return "meta";
  return "instruction";
}

export function isExplicitContinuationAsk(text: string): boolean {
  return /^(continue\b|go ahead\b|proceed\b|keep going\b|finish\b|resume\b|carry on\b|pick up where you left off\b|same task\b)/i.test(
    text.trim()
  );
}

export function isOperatorSteeringInput(text: string, askKind: PiCurrentAskKind): boolean {
  const trimmed = stripQuotedFocusaContext(text).trim();
  if (!trimmed) return false;
  if (askKind === "question" || askKind === "correction") return true;
  if (askKind === "meta") return false;
  return /\b(continue|resume|instead|stop|don't|answer|focus on|work on|switch to|use|fix|implement|explain|summarize|show|verify|check)\b/i.test(
    trimmed
  );
}

export function deriveQueryScope(
  askKind: PiCurrentAskKind
): Pick<PiQueryScope, "scopeKind" | "carryoverPolicy"> {
  return {
    scopeKind:
      askKind === "question"
        ? "fresh_question"
        : askKind === "correction"
          ? "correction"
          : askKind === "meta"
            ? "meta"
            : "mission_carryover",
    carryoverPolicy:
      askKind === "question"
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
  const askText = stripQuotedFocusaContext(params.askText || "")
    .trim()
    .toLowerCase();
  const output = String(params.assistantOutput || "").trim();
  if (!output) return [];

  const outputLower = output.toLowerCase();
  const askTokens = tokenizeForRelevance(askText)
    .filter((token) => token.length >= 4)
    .slice(0, 12);
  const overlapCount = askTokens.filter((token) => outputLower.includes(token)).length;
  const failures: ScopeFailureSignal[] = [];

  const addFailure = (signal: ScopeFailureSignal) => {
    if (!failures.some((existing) => existing.kind === signal.kind)) failures.push(signal);
  };

  if (
    (params.leakClasses || []).some(
      (cls) => cls === "raw_focus_state_serialization" || cls === "internal_routing_reasons"
    )
  ) {
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

  if (
    (params.scopeKind === "fresh_question" || params.scopeKind === "correction") &&
    /\b(as we discussed|as noted earlier|continuing from|from the previous task|carry(ing)? over)\b/i.test(
      outputLower
    )
  ) {
    addFailure({
      kind: "context_overcarry",
      severity: "medium",
      reason: "fresh/correction scope output referenced prior-thread carryover",
    });
  }

  if (
    (params.scopeKind === "fresh_question" || params.scopeKind === "correction") &&
    /\b(other thread|adjacent thread|another task|previous thread|neighbor(ing)? task)\b/i.test(outputLower)
  ) {
    addFailure({
      kind: "adjacent_thread_leakage",
      severity: "medium",
      reason: "fresh/correction scope output referenced adjacent thread/task",
    });
  }

  if (
    (params.askKind === "question" || params.askKind === "instruction") &&
    (params.scopeKind === "fresh_question" || params.scopeKind === "correction") &&
    /\b(more broadly|in general|also consider|additionally|in broader terms)\b/i.test(outputLower) &&
    overlapCount <= Math.max(1, Math.floor(askTokens.length / 4))
  ) {
    addFailure({
      kind: "answer_broadening",
      severity: "low",
      reason: "fresh/correction scope output broadened beyond ask-specific terms",
    });
  }

  return failures;
}

function boundedAttentionText(value: unknown, max = 180): string {
  const text = String(value ?? "")
    .replace(/\s+/g, " ")
    .trim();
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
    ...arrayField(focusState?.artifacts).map(
      (artifact: any) =>
        `${artifact?.kind || "artifact"}:${artifact?.label || artifact?.path_or_id || "unknown"}${artifact?.path_or_id ? `@${artifact.path_or_id}` : ""}`
    ),
  ]
    .map((item) => String(item || "").trim())
    .filter(Boolean);
  const report = [...candidates]
    .reverse()
    .find((item) => /\b(report|summary|spec|audit|proof)\b/i.test(item));
  return report ? boundedAttentionText(report, 160) : "none";
}

function resetToolOutputPressureWindow(now = Date.now()): void {
  getAttachmentRuntime().toolOutputPressure = {
    windowStartedAt: now,
    resultCount: 0,
    totalBytes: 0,
    totalTokens: 0,
    largeResultCount: 0,
    recapRequired: false,
    recapReason: "",
    lastToolName: "",
    lastEventAt: now,
    lastRecapAt: getAttachmentRuntime().toolOutputPressure?.lastRecapAt || 0,
  };
}

export function recordToolOutputPressure(
  toolName: string,
  bytes: number,
  tokens: number
): PiToolOutputPressure {
  const now = Date.now();
  if (
    !getAttachmentRuntime().toolOutputPressure.windowStartedAt ||
    now - getAttachmentRuntime().toolOutputPressure.windowStartedAt > TOOL_OUTPUT_FLOOD_WINDOW_MS
  ) {
    resetToolOutputPressureWindow(now);
  }
  const pressure = getAttachmentRuntime().toolOutputPressure;
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
    pressure.largeResultCount >= TOOL_OUTPUT_FLOOD_LARGE_RESULT_THRESHOLD
      ? `large_outputs=${pressure.largeResultCount}`
      : "",
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
        payload: {
          reason: pressure.recapReason,
          result_count: pressure.resultCount,
          total_bytes: pressure.totalBytes,
        },
      });
      persistState();
    }
  }
  return { ...pressure };
}

export function toolOutputVisibleRecapReason(): string {
  if (!getAttachmentRuntime().toolOutputPressure?.recapRequired) return "";
  if (
    getAttachmentRuntime().toolOutputPressure.windowStartedAt &&
    Date.now() - getAttachmentRuntime().toolOutputPressure.windowStartedAt > TOOL_OUTPUT_FLOOD_WINDOW_MS
  ) {
    resetToolOutputPressureWindow(Date.now());
    persistState();
    return "";
  }
  return getAttachmentRuntime().toolOutputPressure.recapReason;
}

export function formatToolOutputVisibleRecapLines(reason = toolOutputVisibleRecapReason()): string[] {
  if (!reason) return [];
  return [
    `FOCUSA_MEMORY_REFRESH: reason=${boundedAttentionText(reason, 120)}; latest_report=${getLatestReportSummary()?.handle || "none"}; visibility=internal; operator_flow=continue`,
  ];
}

export function markVisibleRecapEmittedIfPresent(assistantOutput: string): boolean {
  const reason = toolOutputVisibleRecapReason();
  if (!reason) return false;
  const preview = String(assistantOutput || "").slice(0, 700);
  const consumed = preview.trim().length > 0;
  focusaPost("/telemetry/trace", {
    event_type: consumed ? "memory_refresh_consumed" : "memory_refresh_deferred",
    payload: {
      reason,
      assistant_preview: boundedAttentionText(preview, 220),
      latest_report_summary_ref: getLatestReportSummary()?.handle || "none",
      visible_recap_forced: false,
    },
  });
  if (!consumed) return false;
  getAttachmentRuntime().toolOutputPressure.lastRecapAt = Date.now();
  resetToolOutputPressureWindow(Date.now());
  persistState();
  return true;
}

function cleanResumeVisibleRecapReason(reason?: string): string {
  const value = String(reason || "").trim();
  return value ? boundedAttentionText(value, 220) : "";
}

function projectRootFromAbsolutePath(value: string): string {
  const match = String(value || "").match(
    /\/(?:home|Users)\/[A-Za-z0-9._-]+\/[A-Za-z0-9._-]+(?:\/[A-Za-z0-9._-]+)?/
  );
  if (!match) return "";
  const parts = match[0].split("/").filter(Boolean);
  if (parts[0] === "home")
    return normalizeProjectRoot(`/${parts.slice(0, Math.min(parts.length, 3)).join("/")}`);
  if (parts[0] === "Users") {
    const depth = parts[2] === "Projects" || parts[2] === "projects" ? 4 : 3;
    return normalizeProjectRoot(`/${parts.slice(0, Math.min(parts.length, depth)).join("/")}`);
  }
  return normalizeProjectRoot(match[0]);
}

function currentAskDeclaresProjectScope(value: string): boolean {
  const text = stripQuotedFocusaContext(String(value || ""));
  if (!text.trim()) return false;
  return (
    /\b(?:wrong place|not this repo|not this project|different project|remote project|switch project)\b/i.test(
      text
    ) ||
    /\b(?:switch|change|move|bind|rebind)\s+(?:to\s+)?(?:the\s+)?(?:project|repo|repository|scope|root)\b/i.test(
      text
    ) ||
    /\bwork(?:ing)?\s+(?:on|in|from)\b/i.test(text) ||
    /\b(?:project(?:\s+root)?|repo(?:sitory)?|scope)\s*(?:is|=|:|at)\b/i.test(text) ||
    /\b(?:use|open|target)\s+(?:the\s+)?(?:remote\s+)?(?:project|repo|repository|scope)\b/i.test(text)
  );
}

const NON_PROJECT_ARTIFACT_SUFFIXES = new Set([
  "log",
  "txt",
  "md",
  "json",
  "jsonl",
  "yaml",
  "yml",
  "toml",
  "rs",
  "ts",
  "js",
  "mjs",
  "sh",
  "py",
]);

function isPlausibleProjectAlias(value: string): boolean {
  const alias = String(value || "")
    .trim()
    .toLowerCase();
  if (!alias || !/[a-z]/.test(alias) || /^\d+(?:\.\d+)+$/.test(alias)) return false;
  const suffix = alias.includes(".") ? alias.split(".").at(-1) || "" : "";
  return !NON_PROJECT_ARTIFACT_SUFFIXES.has(suffix);
}

function projectAliasesForText(text: string, root = ""): string[] {
  const lower = String(text || "").toLowerCase();
  const aliases = new Set<string>();
  if (
    /\b(ptm|planmarr|plan-the-marriage|plan the marriage)\b/i.test(lower) ||
    /planmarr|plan-the-marriage/i.test(root)
  )
    aliases.add("PTM");
  if (/\bfocusa\b/i.test(lower) || /\/focusa$/i.test(root)) aliases.add("Focusa");
  for (const match of lower.matchAll(/(?:https?:\/\/)?([a-z0-9][a-z0-9-]*(?:\.[a-z0-9][a-z0-9-]*)+)/gi)) {
    const host = match[1].replace(/^www\./, "");
    if (!isPlausibleProjectAlias(host)) continue;
    aliases.add(host);
    const firstLabel = host.split(".")[0];
    if (firstLabel && !["www", "app", "api"].includes(firstLabel)) aliases.add(firstLabel);
  }
  const base = root.split("/").filter(Boolean).at(-1);
  if (base) aliases.add(base);
  return [...aliases].filter(Boolean).slice(0, 8);
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
  const alias = boundedAttentionText(
    input.project_alias ||
      projectAliasesForText(input.action || "", projectRoot)[0] ||
      projectRoot.split("/").filter(Boolean).at(-1) ||
      "unknown",
    80
  );
  if (!projectRoot && !alias) return null;
  const now = Date.now();
  const keyRoot = projectRoot || `alias:${alias.toLowerCase()}`;
  const existing = getAttachmentRuntime().projectSwitchLedger.find(
    (entry: PiProjectThreadObservation) =>
      (entry.project_root && entry.project_root === projectRoot) ||
      entry.project_alias.toLowerCase() === alias.toLowerCase()
  );
  const action = boundedProjectAction(input.source, input.action);
  const observation: PiProjectThreadObservation = existing
    ? {
        ...existing,
        project_alias: existing.project_alias || alias,
        project_root: existing.project_root || projectRoot || keyRoot,
        remote_host: input.remote_host || existing.remote_host,
        evidence_ref: boundedAttentionText(input.evidence_ref || existing.evidence_ref, 160),
        last_seen_turn: input.turn_id,
        recent_actions: [action, ...existing.recent_actions.filter((item: string) => item !== action)].slice(
          0,
          PROJECT_SWITCH_LEDGER_MAX_ACTIONS
        ),
        confidence: Math.max(existing.confidence || 0, input.confidence ?? 0.6),
        source: input.source,
        updatedAt: now,
      }
    : {
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
  getAttachmentRuntime().projectSwitchLedger = [
    observation,
    ...getAttachmentRuntime().projectSwitchLedger.filter(
      (entry: PiProjectThreadObservation) => entry !== existing
    ),
  ]
    .sort((a: PiProjectThreadObservation, b: PiProjectThreadObservation) => b.updatedAt - a.updatedAt)
    .slice(0, PROJECT_SWITCH_LEDGER_MAX_OBSERVATIONS);
  persistProjectSwitchLedgerAnchor();
  persistState();
  return observation;
}

// FOCUSA-GitHub#3: Mark a project observation as supporting work
// Use this when the agent filed a ticket/PR/evidence in another repo
// while pursuing the primary active scope. Does NOT transfer action authority.
export function markObservationAsSupportingWork(
  alias: string,
  primaryScopeRef: string,
  whyRelated: string
): boolean {
  const lower = alias.toLowerCase().trim();
  const entry = getAttachmentRuntime().projectSwitchLedger.find(
    (e: PiProjectThreadObservation) =>
      e.project_alias.toLowerCase() === lower || e.project_root.toLowerCase().includes(lower)
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
  persistProjectSwitchLedgerAnchor();
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
  for (const entry of getAttachmentRuntime().projectSwitchLedger as PiProjectThreadObservation[]) {
    switch (entry.relationship_kind) {
      case "active_scope":
        result.active_scope.push(entry);
        break;
      case "supporting_work":
        result.supporting_work.push(entry);
        break;
      case "artifact_link":
        result.artifact_link.push(entry);
        break;
      case "scope_switch":
        result.scope_switch.push(entry);
        break;
      default:
        result.uncategorized.push(entry);
    }
  }
  return result;
}

function rememberedProjectRootForAlias(alias: string): string {
  const lower = String(alias || "")
    .toLowerCase()
    .trim();
  if (!lower) return "";
  const scored: Array<{ root: string; score: number }> = [];
  const last = getLastProjectIdentity() || {};
  const lastAliases = Array.isArray(last.aliases) ? last.aliases : [];
  const lastText = [last.project_id, last.canonical_name, ...lastAliases]
    .filter(Boolean)
    .join(" ")
    .toLowerCase();
  if (last.project_root && lastText.includes(lower)) {
    scored.push({
      root: normalizeProjectRoot(last.project_root),
      score: last.confidence === "high" ? 1 : 0.8,
    });
  }
  for (const entry of getAttachmentRuntime().projectSwitchLedger || []) {
    const entryText = `${entry.project_alias || ""} ${entry.project_root || ""}`.toLowerCase();
    if (entry.project_root && entryText.includes(lower)) {
      scored.push({ root: normalizeProjectRoot(entry.project_root), score: entry.confidence || 0.5 });
    }
  }
  scored.sort((a: { score: number }, b: { score: number }) => b.score - a.score);
  if (scored[0]?.root) return scored[0].root;

  // Core directory detector: when no project_root is stored for this alias/domain,
  // search configured/project host roots for a matching .focusa-project.json marker.
  return searchProjectMarkerForAlias(lower);
}

// Search filesystem for a project's canonical root from alias/domain marker data.
function normalizeProjectHint(value: string): string {
  return String(value || "")
    .trim()
    .replace(/^https?:\/\//i, "")
    .replace(/^www\./i, "")
    .replace(/\/$/, "")
    .toLowerCase();
}

function markerHintValues(marker: any): string[] {
  const values = [
    marker?.project_id,
    marker?.canonical_name,
    marker?.live_url,
    marker?.root_url,
    marker?.local_url,
  ];
  if (Array.isArray(marker?.aliases)) values.push(...marker.aliases);
  if (marker?.project_urls && typeof marker.project_urls === "object")
    values.push(...Object.values(marker.project_urls));
  return values.map((v) => normalizeProjectHint(String(v || ""))).filter(Boolean);
}

function markerMatchesProjectHint(marker: any, alias: string): boolean {
  const hint = normalizeProjectHint(alias);
  if (!hint) return false;
  return markerHintValues(marker).some(
    (value) => value === hint || value.startsWith(`${hint}.`) || value.startsWith(`${hint}-`)
  );
}

function searchProjectMarkerForAlias(alias: string): string {
  // Core directory detection: recursive bounded marker search. This is not
  // Perpetua-specific; it resolves parent/child/subdomain folders from markers.
  const candidateDirs = [
    ...(process.env.FOCUSA_PROJECT_SEARCH_DIRS || "").split(":").filter(Boolean),
    process.env.HOME || "",
    "/home",
  ].filter(Boolean);
  const queue = candidateDirs.map((dir) => ({ dir, depth: 0 }));
  const seen = new Set<string>();
  let visited = 0;

  while (queue.length && visited < 300) {
    const item = queue.shift()!;
    const dir = normalizeProjectRoot(item.dir);
    if (!dir || seen.has(dir) || item.depth > 4) continue;
    seen.add(dir);
    visited++;
    const markerPath = `${dir}/.focusa-project.json`;
    try {
      const marker = JSON.parse(readFileSync(markerPath, "utf-8"));
      if (markerMatchesProjectHint(marker, alias)) {
        const root = normalizeProjectRoot(String(marker.project_root || dir));
        try {
          mkdirSync("/tmp/pi-scratch", { recursive: true });
          appendFileSync(
            "/tmp/pi-scratch/alias-resolution.log",
            `[alias-resolution] directory_detector: resolved ${alias} to ${root} via ${markerPath}\n`
          );
        } catch {
          /* best effort */
        }
        return root;
      }
      // Marker roots are project boundaries; do not let a parent project swallow children.
      continue;
    } catch {
      /* not a marker file or unreadable */
    }
    try {
      for (const entry of readdirSync(dir, { withFileTypes: true })) {
        if (entry.isDirectory() && ![".git", "node_modules", "target"].includes(entry.name)) {
          queue.push({ dir: `${dir}/${entry.name}`, depth: item.depth + 1 });
        }
      }
    } catch {
      /* directory unreadable */
    }
  }
  return "";
}

export function observeProjectThreadHintsFromText(
  text: string,
  turnId: string,
  source: PiProjectThreadObservation["source"],
  action?: string
): PiProjectThreadObservation[] {
  const raw = String(text || "").slice(0, 2000);
  const scopeIntent = source !== "current_ask" || currentAskDeclaresProjectScope(raw);
  const root = scopeIntent ? projectRootFromAbsolutePath(raw) : "";
  const aliases = scopeIntent ? projectAliasesForText(raw, root) : [];
  const observations: PiProjectThreadObservation[] = [];
  if (root) {
    observations.push(
      observeProjectThreadEvidence({
        project_root: root,
        project_alias: aliases[0],
        evidence_ref: `${source}:${turnId}:project_path`,
        turn_id: turnId,
        action: action || `path=${root}`,
        confidence: 0.9,
        source,
      })!
    );
  }
  for (const alias of aliases) {
    if (root && alias === aliases[0]) continue;
    const knownRoot = root || rememberedProjectRootForAlias(alias);
    observations.push(
      observeProjectThreadEvidence({
        project_root: knownRoot,
        project_alias: alias,
        evidence_ref: `${source}:${turnId}:project_alias:${alias}`,
        turn_id: turnId,
        action: action || `alias=${alias}`,
        confidence: knownRoot ? 0.75 : 0.58,
        source,
      })!
    );
  }
  return observations.filter(Boolean);
}

function aliasMatchesSavedProject(alias: string, savedProjectRoot: string): boolean {
  const savedRoot = normalizeProjectRoot(savedProjectRoot);
  if (!savedRoot) return false;
  const lowerAlias = String(alias || "")
    .toLowerCase()
    .trim();
  if (!lowerAlias) return false;
  return projectAliasesForText("", savedRoot).some((candidate) => candidate.toLowerCase() === lowerAlias);
}

function isAliasOnlyProjectRoot(root: string): boolean {
  return normalizeProjectRoot(root).startsWith("alias:");
}

function projectSwitchLedgerCandidateForAsk(
  currentAskText: string,
  savedProjectRoot: string
): PiProjectThreadObservation | null {
  const ask = stripQuotedFocusaContext(currentAskText || "");
  if (
    !ask.trim() ||
    !currentAskDeclaresProjectScope(ask) ||
    !getAttachmentRuntime().projectSwitchLedger.length
  )
    return null;
  const lower = ask.toLowerCase();
  const savedRoot = normalizeProjectRoot(savedProjectRoot);
  const scored = getAttachmentRuntime()
    .projectSwitchLedger.filter((entry: PiProjectThreadObservation) =>
      isPlausibleProjectAlias(entry.project_alias)
    )
    .map((entry: PiProjectThreadObservation) => {
      let score = entry.confidence || 0;
      const alias = entry.project_alias.toLowerCase();
      const entryRoot = normalizeProjectRoot(entry.project_root);
      if (savedRoot && isAliasOnlyProjectRoot(entryRoot) && aliasMatchesSavedProject(alias, savedRoot))
        score -= 2.0;
      if (alias && lower.includes(alias.toLowerCase())) score += 0.5;
      if (
        /\b(ptm|planmarr|plan-the-marriage|plan the marriage)\b/i.test(lower) &&
        /ptm|planmarr|plan-the-marriage/i.test(`${entry.project_alias} ${entry.project_root}`)
      )
        score += 0.7;
      if (entryRoot && !isAliasOnlyProjectRoot(entryRoot) && lower.includes(entryRoot.toLowerCase()))
        score += 0.8;
      if (
        /\b(wrong place|not this repo|not this project|different project|remote project|switch project)\b/i.test(
          lower
        )
      )
        score += 0.15;
      if (savedRoot && entryRoot === savedRoot) score -= 0.6;
      score += Math.max(0, 0.2 - ((Date.now() - entry.updatedAt) / 86_400_000) * 0.05);
      return { entry, score };
    })
    .sort((a: { score: number }, b: { score: number }) => b.score - a.score);
  const best = scored[0];
  if (!best || best.score < PROJECT_SWITCH_LEDGER_MIN_CONFLICT_CONFIDENCE) return null;
  const bestRoot = normalizeProjectRoot(best.entry.project_root);
  if (savedRoot && bestRoot === savedRoot) return null;
  if (
    savedRoot &&
    isAliasOnlyProjectRoot(bestRoot) &&
    aliasMatchesSavedProject(best.entry.project_alias, savedRoot)
  )
    return null;
  return best.entry;
}

export function formatProjectSwitchLedgerLines(
  currentAskText = getAttachmentRuntime().currentAsk?.text || ""
): string[] {
  const candidate = projectSwitchLedgerCandidateForAsk(
    currentAskText,
    getScopedWorkpointPacket()?.project_root || getAttachmentRuntime().sessionCwd || ""
  );
  const entries = (
    candidate
      ? [
          candidate,
          ...getAttachmentRuntime().projectSwitchLedger.filter(
            (entry: PiProjectThreadObservation) => entry !== candidate
          ),
        ]
      : getAttachmentRuntime().projectSwitchLedger
  ).slice(0, 4);
  return entries.map(
    (entry: PiProjectThreadObservation) =>
      `${entry.project_alias} root=${entry.project_root || "unknown"} confidence=${entry.confidence.toFixed(2)} evidence=${entry.evidence_ref} recent=${entry.recent_actions.slice(0, 2).join(" | ")}`
  );
}

export function emitCurrentAskScopeVerdictTelemetry(
  verdict: PiCurrentAskScopeVerdict,
  sourceTurnId = getAttachmentRuntime().currentAsk?.sourceTurnId ||
    getAttachmentRuntime().sessionFrameKey ||
    "pi-current-ask-scope"
): void {
  if (!getAttachmentRuntime().focusaAvailable || verdict.status !== "conflict") return;
  const key = `${sourceTurnId}:${verdict.status}:${verdict.saved_scope.project_root}:${verdict.current_ask_scope.project_root}:${verdict.reason}`;
  if (getAttachmentRuntime().lastCurrentAskScopeTelemetryKey === key) return;
  getAttachmentRuntime().lastCurrentAskScopeTelemetryKey = key;
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

export function buildCurrentAskScopeVerdict(
  options: {
    currentAskText?: string;
    workpointPacket?: any;
    projectRoot?: string;
    continuityId?: string;
  } = {}
): PiCurrentAskScopeVerdict {
  const ask = stripQuotedFocusaContext(
    options.currentAskText ?? getAttachmentRuntime().currentAsk?.text ?? ""
  );
  const packet = options.workpointPacket || getScopedWorkpointPacket() || {};
  const savedRoot = normalizeProjectRoot(
    workpointValue(packet, "project_root") || options.projectRoot || getAttachmentRuntime().sessionCwd || ""
  );
  const continuityId = String(
    options.continuityId ||
      getAttachmentRuntime().continuityId ||
      workpointValue(packet, "continuity_id") ||
      ""
  ).trim();
  const scopeIntent = currentAskDeclaresProjectScope(ask);
  const explicitRoot = scopeIntent ? projectRootFromAbsolutePath(ask) : "";
  const aliases = projectAliasesForText(ask, explicitRoot);
  const ledgerCandidate = scopeIntent ? projectSwitchLedgerCandidateForAsk(ask, savedRoot) : null;
  const alias = aliases[0] || ledgerCandidate?.project_alias || "unknown";
  const aliasKnownRoot = scopeIntent ? rememberedProjectRootForAlias(alias) : "";
  const rawCandidateRoot = normalizeProjectRoot(
    explicitRoot || ledgerCandidate?.project_root || aliasKnownRoot || ""
  );
  const candidateRoot =
    savedRoot && isAliasOnlyProjectRoot(rawCandidateRoot) && aliasMatchesSavedProject(alias, savedRoot)
      ? ""
      : rawCandidateRoot;
  const evidenceRef = explicitRoot
    ? "current_ask:explicit_project_path"
    : ledgerCandidate?.evidence_ref || (aliases.length ? `current_ask:project_alias:${alias}` : "none");
  const confidence = explicitRoot
    ? 0.95
    : (ledgerCandidate?.confidence ?? (candidateRoot ? 0.7 : aliases.length ? 0.58 : 0));
  const hasCorrectionPhrase =
    /\b(wrong place|not this repo|not this project|different project|remote project|switch project)\b/i.test(
      ask
    );
  const sameProjectAliasMention =
    savedRoot && aliases.some((candidate) => aliasMatchesSavedProject(candidate, savedRoot));
  let status: PiCurrentAskScopeVerdict["status"] = "unknown";
  let reason = "no current-ask project signal";
  if (candidateRoot && savedRoot && candidateRoot !== savedRoot) {
    status = "conflict";
    reason = `current ask indicates ${alias} at ${candidateRoot}, saved scope is ${savedRoot}`;
  } else if (candidateRoot && (!savedRoot || candidateRoot === savedRoot)) {
    status = "aligned";
    reason =
      candidateRoot === savedRoot
        ? "current ask project matches saved scope"
        : "current ask names project but saved scope is unbound";
  } else if (sameProjectAliasMention && savedRoot && !hasCorrectionPhrase) {
    status = "aligned";
    reason = "current ask names saved project alias without competing root";
  } else if (scopeIntent && (aliases.length || hasCorrectionPhrase)) {
    status = "override_candidate";
    reason = aliases.length
      ? `current ask names project alias ${alias} without verified root`
      : "operator correction implies saved project/root may be wrong";
  } else if (savedRoot) {
    status = "aligned";
    reason = "no competing project signal in current ask";
  }
  const actionAllowed = true;
  const sessionBindingDecision = currentProjectBindingDecision();
  const durableProjectWriteAllowed =
    status === "aligned" &&
    isProjectRootAuthoritySafe(savedRoot) &&
    (!candidateRoot || candidateRoot === savedRoot) &&
    projectBindingAllowsDurableWrites(sessionBindingDecision);
  const verdict = {
    status,
    saved_scope: { project_root: savedRoot || "unknown", continuity_id: continuityId || "unknown" },
    current_ask_scope: {
      project_alias: alias,
      project_root: candidateRoot || "unknown",
      confidence,
      evidence_ref: evidenceRef,
    },
    action_authority_for_current_ask: actionAllowed,
    durable_project_write_authority: durableProjectWriteAllowed,
    required_next: durableProjectWriteAllowed
      ? []
      : ["focusa_project_verify", "focusa_project_identity", "focusa_workpoint_checkpoint"],
    reason,
  };
  emitCurrentAskScopeVerdictTelemetry(verdict);
  return verdict;
}

export function formatCurrentAskScopeVerdictLines(verdict = buildCurrentAskScopeVerdict()): string[] {
  if (verdict.status === "aligned" && verdict.durable_project_write_authority) return [];
  return [
    `FOCUSA_SCOPE: status=${verdict.status}; conversation=continue; durable_writes=${verdict.durable_project_write_authority ? "allowed" : "verify_first"}; saved=${verdict.saved_scope.project_root}; candidate=${verdict.current_ask_scope.project_root}; reason=${boundedAttentionText(verdict.reason, 120)}; next=${verdict.required_next.join(" -> ") || "none"}`,
  ];
}

function currentAskProjectConflictReason(
  currentAskText: string,
  projectRoot: string,
  workpointProjectRoot: string
): string {
  const ask = stripQuotedFocusaContext(currentAskText || "");
  if (!ask.trim()) return "";
  const lower = ask.toLowerCase();
  const explicitPath = currentAskDeclaresProjectScope(ask)
    ? ask.match(/\/(?:home|Users)\/[A-Za-z0-9._-]+\/[A-Za-z0-9._/-]+/)
    : null;
  if (
    explicitPath &&
    normalizeProjectRoot(explicitPath[0]) !== normalizeProjectRoot(projectRoot || workpointProjectRoot)
  ) {
    return `operator named different project path ${boundedAttentionText(explicitPath[0], 120)}`;
  }
  const ledgerCandidate = projectSwitchLedgerCandidateForAsk(ask, projectRoot || workpointProjectRoot);
  if (ledgerCandidate) {
    return `project_switch_ledger indicates ${boundedAttentionText(ledgerCandidate.project_alias, 80)} at ${boundedAttentionText(ledgerCandidate.project_root, 120)} (${boundedAttentionText(ledgerCandidate.evidence_ref, 120)})`;
  }
  if (
    /\b(wrong place|not this repo|not this project|different project|remote project|switch project)\b/i.test(
      ask
    )
  ) {
    return "operator text indicates current project/root may be wrong";
  }
  if (
    /\b(ptm|planmarr|plan-the-marriage)\b/i.test(lower) &&
    !/planmarr|plan-the-marriage/i.test(projectRoot || workpointProjectRoot)
  ) {
    return "operator text names PTM/planmarr while saved scope is different";
  }
  return "";
}

export function buildAttentionRecallVerdict(
  options: {
    focusState?: any;
    workpointPacket?: any;
    currentAskText?: string;
    currentAskKind?: PiCurrentAskKind | string;
    queryScopeKind?: PiQueryScope["scopeKind"] | string;
    projectRoot?: string;
    continuityId?: string;
    visibleRecapReason?: string;
  } = {}
): PiAttentionRecallVerdict {
  const packet = options.workpointPacket || getScopedWorkpointPacket() || {};
  const askText = stripQuotedFocusaContext(
    options.currentAskText ?? getAttachmentRuntime().currentAsk?.text ?? ""
  );
  const projectRoot = normalizeProjectRoot(
    options.projectRoot || getAttachmentRuntime().sessionCwd || workpointValue(packet, "project_root")
  );
  const packetProjectRoot = normalizeProjectRoot(workpointValue(packet, "project_root"));
  const continuityId = String(
    options.continuityId ||
      getAttachmentRuntime().continuityId ||
      workpointValue(packet, "continuity_id") ||
      ""
  ).trim();
  const mission =
    workpointValue(packet, "mission") ||
    getAttachmentRuntime().activeFrameGoal ||
    getAttachmentRuntime().activeFrameTitle ||
    "current Focusa task";
  const nextAction =
    workpointValue(packet, "next_slice") ||
    getAttachmentRuntime().lastCompactDecision ||
    askText ||
    getAttachmentRuntime().lastFocusSnapshot.currentFocus ||
    "continue bounded current task";
  const ledgerCandidate = projectSwitchLedgerCandidateForAsk(askText, projectRoot || packetProjectRoot);
  const conflictReason = currentAskProjectConflictReason(askText, projectRoot, packetProjectRoot);
  const scopeStatus = conflictReason ? "conflict" : projectRoot || packetProjectRoot ? "aligned" : "unknown";
  const visibleRecapReason = cleanResumeVisibleRecapReason(options.visibleRecapReason);
  const memoryRefreshRecommended = Boolean(
    visibleRecapReason ||
    conflictReason ||
    options.queryScopeKind === "correction" ||
    options.currentAskKind === "correction"
  );
  const visibleRecapRequired = false;
  const attentionRisks = [
    visibleRecapReason ? "tool_output_flood" : "",
    conflictReason ? "scope_conflict" : "",
    options.queryScopeKind === "correction" || options.currentAskKind === "correction"
      ? "operator_correction"
      : "",
  ].filter(Boolean);
  const mustNotForget = [
    askText ? `current_ask=${boundedAttentionText(askText, 160)}` : "current_ask=(none)",
    `task=${boundedAttentionText(mission, 140)}`,
    projectRoot ? `project_root=${boundedAttentionText(projectRoot, 140)}` : "project_root=(unbound)",
    continuityId ? `continuity_id=${boundedAttentionText(continuityId, 100)}` : "continuity_id=(unbound)",
    conflictReason
      ? `scope_conflict=${boundedAttentionText(conflictReason, 140)}`
      : "scope_conflict=none_detected",
    visibleRecapReason
      ? `visible_recap_reason=${boundedAttentionText(visibleRecapReason, 140)}`
      : "visible_recap_reason=none",
    "transcript_tail_is_not_authority",
  ];
  const evidenceRefs = [
    packet?.workpoint_id ? `workpoint:${packet.workpoint_id}` : "",
    packetProjectRoot ? `saved_scope:${packetProjectRoot}` : "",
    ledgerCandidate ? `project_thread:${ledgerCandidate.project_alias}@${ledgerCandidate.project_root}` : "",
    ...arrayField(packet?.verification_records)
      .slice(0, 3)
      .map((record: any) => String(record?.evidence_ref || record?.result || "").trim()),
  ]
    .filter(Boolean)
    .map((item) => boundedAttentionText(item, 140));
  return {
    schema: "focusa.attention_recall_verdict.v1",
    status: conflictReason ? "conflict" : memoryRefreshRecommended ? "attention_risk" : "attentive",
    visible_recap_required: visibleRecapRequired,
    visible_recap_reason: visibleRecapReason || "none",
    attention_risks: attentionRisks,
    required_next: conflictReason ? ["verify_project_scope_before_durable_write"] : [],
    current_ask_scope_status: scopeStatus,
    scope_conflict_reason: conflictReason || "none",
    memory_anchor: {
      task: boundedAttentionText(mission, 160),
      must_not_forget: mustNotForget.slice(0, 8),
      latest_report_summary_ref: latestReportSummaryRefFromFocusState(options.focusState),
      evidence_refs: evidenceRefs.slice(0, 5),
      next_action: conflictReason
        ? "continue diagnosis; verify project scope before durable writes"
        : boundedAttentionText(nextAction, 180),
      action_authority_for_current_ask: true,
      durable_project_write_authority:
        !conflictReason && isProjectRootAuthoritySafe(projectRoot || packetProjectRoot),
    },
  };
}

export function formatAttentionRecallFocusSliceLines(verdict: PiAttentionRecallVerdict): string[] {
  const anchor = verdict.memory_anchor;
  if (verdict.status === "attentive" && !verdict.attention_risks.length) return [];
  return [
    `FOCUSA_ATTENTION: status=${verdict.status}; conversation=continue; durable_writes=${anchor.durable_project_write_authority ? "allowed" : "verify_first"}; risks=${verdict.attention_risks.join(",") || "none"}; next=${boundedAttentionText(anchor.next_action, 140)}; report=${anchor.latest_report_summary_ref}`,
  ];
}

function assistantOutputLooksLikeReport(text: string): boolean {
  const normalized = String(text || "").trim();
  if (normalized.length < 240) return false;
  const headingHits = (
    normalized.match(
      /^#{1,3}\s+(status|summary|task summary|evidence|proof|result|results|next|blocker|implementation|audit|spec)/gim
    ) || []
  ).length;
  const labelHits = (
    normalized.match(/\b(Status|Proof|Evidence|Result|Next action|Blocker|Commit|Tests?):/g) || []
  ).length;
  return (
    headingHits >= 1 ||
    labelHits >= 2 ||
    /\b(task summary|end-of-task|implementation report|audit report|spec update|proof:)\b/i.test(normalized)
  );
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

export function nativeSessionAllowsNonessentialPersistence(): boolean {
  const posture = getAttachmentRuntime().lastNativeSessionPressure?.posture;
  return posture !== "hard_pressure" && posture !== "emergency" && posture !== "oversized_at_start";
}

export function maybeCaptureReportSummaryFromAssistantOutput(
  text: string,
  turnId: string
): PiReportSummaryHandle | null {
  if (!assistantOutputLooksLikeReport(text)) return null;
  const summary = reportSummaryFromAssistantOutput(text);
  if (!summary) return null;
  const id = storeEcsArtifact("report-summary", summary);
  const handle = `[HANDLE:report-summary:${id}]`;
  const captured: PiReportSummaryHandle = {
    handle,
    summary: boundedAttentionText(summary, 240),
    capturedAt: Date.now(),
    turnId,
  };
  setLatestReportSummary(captured);
  if (nativeSessionAllowsNonessentialPersistence()) {
    try {
      getAttachmentRuntime().pi?.appendEntry("focusa-report-summary", captured);
    } catch {
      /* best effort */
    }
  }
  persistState();
  return captured;
}

function tokenizeForRelevance(text: string): string[] {
  return Array.from(new Set(text.toLowerCase().match(/[a-z0-9_./:-]{3,}/g) || []));
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
  if (
    /\b(test|failing|error|bug|trace|constraint|decision|scope|question|correction)\b/.test(normalizedAsk) &&
    /\b(test|failing|error|bug|trace|constraint|decision|scope|question|correction)\b/.test(candidateText)
  ) {
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
  activePriors: PiGoverningPriorKind[]
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
  }
): PiFocusSelection {
  const values = (items || []).filter((item): item is PiRankedItem =>
    Boolean(item?.value && item.value.trim())
  );
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
  options?: { maxItems?: number; fallbackItems?: number; minScore?: number }
): PiFocusSelection {
  return selectRelevantRankedItems(
    (items || []).filter((item): item is string => Boolean(item && item.trim())).map((value) => ({ value })),
    askText,
    options
  );
}

export function selectionRelevanceScore(selection: PiFocusSelection): number {
  if (!selection.items.length || !selection.scores.length) return 0;
  const selected = new Set(selection.items);
  const scores = selection.scores.filter(({ value }) => selected.has(value)).map(({ score }) => score);
  return scores.length ? Math.max(...scores) : 0;
}

export function retentionBucketsFromSelection(
  selection: PiFocusSelection,
  options?: { maxDecayed?: number; maxHistorical?: number }
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
  let historicalPool = nonActive.filter((entry) => (entry.score ?? 0) < 0 && !decayedSet.has(entry.value));
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

  if (
    /\b(safety|forbid|forbidden|never|must_not|must not|policy|destructive|high[-_ ]risk|constraint)\b/.test(
      lower
    )
  ) {
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
  if (
    /\b(affordance|permission|tool|execution|environment|transport|worktree|dependency|resource)\b/.test(
      lower
    )
  ) {
    add("affordance_reality_prior");
  }

  return out;
}

export function formatWorkingSetItems(
  records: Array<{ key?: string; value?: string; updated_at?: string; pinned?: boolean }> | undefined
): PiRankedItem[] {
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

export function formatVerifiedDeltaItems(
  handles:
    Array<{ kind?: string; id?: string; label?: string; created_at?: string; pinned?: boolean }> | undefined
): PiRankedItem[] {
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
  missionLike: string[]
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
  relevanceScore?: number
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
  if (!getAttachmentRuntime().cfg?.daemonAutoRestart || !getAttachmentRuntime().pi) return false;
  if (getAttachmentRuntime().daemonRestartInFlight) return getAttachmentRuntime().daemonRestartInFlight;
  const now = Date.now();
  const hourAgo = now - 3_600_000;
  getAttachmentRuntime().daemonRestartAttempts = getAttachmentRuntime().daemonRestartAttempts.filter(
    (t: number) => t >= hourAgo
  );
  if (
    getAttachmentRuntime().daemonRestartAttempts.length >=
    (getAttachmentRuntime().cfg.daemonRestartMaxPerHour || 20)
  )
    return false;
  const last =
    getAttachmentRuntime().daemonRestartAttempts[getAttachmentRuntime().daemonRestartAttempts.length - 1] ||
    0;
  if (now - last < (getAttachmentRuntime().cfg.daemonRestartCooldownMs || 5_000)) return false;

  getAttachmentRuntime().daemonRestartAttempts.push(now);
  const cmd = getAttachmentRuntime().cfg.daemonRestartCommand || DEFAULT_DAEMON_RESTART_COMMAND;
  getAttachmentRuntime().daemonRestartInFlight = (async () => {
    try {
      if (cmd !== DEFAULT_DAEMON_RESTART_COMMAND) return false;
      await getAttachmentRuntime().pi!.exec("systemctl", ["restart", "focusa-daemon"]);
      for (let i = 0; i < 12; i++) {
        await sleep(getAttachmentRuntime().cfg?.daemonRecoveryProbeMs || 750);
        const h = await focusaFetch("/health");
        if (h?.ok === true) return true;
      }
    } catch {
      return false;
    } finally {
      getAttachmentRuntime().daemonRestartInFlight = null;
    }
    return false;
  })();
  const ok = await getAttachmentRuntime().daemonRestartInFlight;
  if (ok) {
    focusaPost("/telemetry/ops", { event: "daemon_kickstart_recovered", surface: "pi", reason });
  }
  return ok;
}

export async function checkFocusa(): Promise<boolean> {
  const h = await focusaFetch("/health");
  const status = h?.ok === true ? null : await focusaFetch(HEALTHCHECK_STATUS_FALLBACK_PATH);
  const fallbackHotOk = status?.status === "ok" && status?.summary_only !== false;
  const wasAvailable = getAttachmentRuntime().focusaAvailable;
  getAttachmentRuntime().focusaAvailable = h?.ok === true || fallbackHotOk || status?.session != null;

  if (getAttachmentRuntime().focusaAvailable && h?.ok !== true && fallbackHotOk) {
    focusaPost("/telemetry/ops", {
      event: "healthcheck_hot_fallback_ok",
      surface: "pi",
      failed_route: "/v1/health",
      fallback_route: `/v1${HEALTHCHECK_STATUS_FALLBACK_PATH}`,
      route_tier: status?.route_tier || "hot",
    });
  }

  if (getAttachmentRuntime().focusaAvailable) {
    getAttachmentRuntime().healthFailCount = 0;
    getAttachmentRuntime().healthBackoffMs = 30_000;
    getAttachmentRuntime().daemonHoldoverMode = false;
    // §11: Outage recovery — record audit event
    if (!wasAvailable && getAttachmentRuntime().outageStart) {
      const durationMs = Date.now() - getAttachmentRuntime().outageStart;
      focusaPost("/telemetry/ops", {
        event: "outage_recovered",
        surface: "pi",
        duration_ms: durationMs,
        missed_turns: getTurnCount(),
      });
      getAttachmentRuntime().outageStart = null;
    }
  } else {
    getAttachmentRuntime().healthFailCount++;
    getAttachmentRuntime().daemonHoldoverMode = true;
    // During daemon outage, probe quickly enough to recover inside the same Pi session.
    getAttachmentRuntime().healthBackoffMs = Math.min(
      1_000 * Math.pow(2, Math.min(getAttachmentRuntime().healthFailCount - 1, 4)),
      15_000
    );
    // §11: Record outage start
    if (wasAvailable && !getAttachmentRuntime().outageStart) {
      getAttachmentRuntime().outageStart = Date.now();
      // Fire-and-forget — may fail since Focusa is down
      focusaFetch("/telemetry/ops", {
        method: "POST",
        body: JSON.stringify({ event: "outage_started", surface: "pi", turn_count: getTurnCount() }),
      }).catch(() => {});
    }
  }
  return getAttachmentRuntime().focusaAvailable;
}

// ── Extract text from TextContent[] | string ─────────────────────────────────
export function extractText(content: any): string {
  if (typeof content === "string") return content;
  if (Array.isArray(content)) return content.map((c: any) => c.text || "").join("");
  return String(content || "");
}

async function loadFocusState(): Promise<{ frame: any; fs: any; stack: any } | null> {
  const scopedQs = new URLSearchParams();
  if (getAttachmentRuntime().activeFrameId) scopedQs.set("frame_id", getAttachmentRuntime().activeFrameId);
  if (getAttachmentRuntime().continuityId) scopedQs.set("continuity_id", getAttachmentRuntime().continuityId);
  if (isProjectRootAuthoritySafe(getAttachmentRuntime().sessionCwd))
    scopedQs.set("project_root", normalizeProjectRoot(getAttachmentRuntime().sessionCwd));
  if (getAttachmentRuntime().sessionFrameKey)
    scopedQs.set("session_key", getAttachmentRuntime().sessionFrameKey);
  const scopedPath = scopedQs.size > 0 ? `/focus/frame/current?${scopedQs.toString()}` : null;

  const [scoped, asccState] = await Promise.all([
    scopedPath ? focusaFetch(scopedPath).catch(() => null) : Promise.resolve(null),
    focusaFetch("/ascc/state").catch(() => null),
  ]);

  let frame = scoped?.frame || null;
  let stack = frame
    ? {
        stack: { active_id: scoped?.active_frame_id || null, frames: [frame] },
        active_frame_id: scoped?.active_frame_id || null,
      }
    : null;

  // Explicit frame_id can become stale after frame rescope/compaction. If the
  // scoped frame is no longer active, fall back to stack lookup so the session
  // key can find the current active Pi frame before reads/writes.
  if (frame && frame.status !== "active" && getAttachmentRuntime().sessionFrameKey) {
    frame = null;
    stack = null;
  }

  if (!frame) {
    stack = await focusaFetch("/focus/stack");
    if (!stack?.stack?.frames?.length) return null;
    const frames = stack.stack.frames;
    frame = getAttachmentRuntime().activeFrameId
      ? frames.find((f: any) => f.id === getAttachmentRuntime().activeFrameId) || null
      : null;

    if (
      (!frame || frame.status !== "active" || isContaminatedFrameIdentity(frame)) &&
      getAttachmentRuntime().sessionFrameKey
    ) {
      const scopedActive =
        [...frames]
          .reverse()
          .find(
            (f: any) =>
              f.status === "active" &&
              Array.isArray(f.tags) &&
              f.tags.includes(getAttachmentRuntime().sessionFrameKey || "") &&
              !isContaminatedFrameIdentity(f)
          ) || null;
      if (scopedActive) {
        frame = scopedActive;
        getAttachmentRuntime().activeFrameId = scopedActive.id;
      } else if (frame && isContaminatedFrameIdentity(frame)) {
        getAttachmentRuntime().activeFrameId = null;
        getAttachmentRuntime().activeFrameTitle = "";
        getAttachmentRuntime().activeFrameGoal = "";
        return null;
      }
    }
  }

  if (!frame || isContaminatedFrameIdentity(frame)) {
    getAttachmentRuntime().activeFrameId = null;
    getAttachmentRuntime().activeFrameTitle = "";
    getAttachmentRuntime().activeFrameGoal = "";
    return null;
  }

  const liveAscc =
    asccState?.frame_id === frame.id ? asccState?.ascc || asccState?.focus_state || null : null;
  const frameState = frame?.focus_state || {};
  const trajectoryShortTermGoal = getLastTrajectoryClarity()?.short_term_goal || "";
  const fs = {
    ...frameState,
    ...(liveAscc || {}),
    current_focus:
      liveAscc?.current_focus ||
      frameState.current_focus ||
      frameState.current_state ||
      trajectoryShortTermGoal ||
      "",
    current_state:
      liveAscc?.current_state ||
      frameState.current_state ||
      frameState.current_focus ||
      trajectoryShortTermGoal ||
      "",
  };

  getAttachmentRuntime().activeFrameId = frame.id || getAttachmentRuntime().activeFrameId;
  getAttachmentRuntime().activeFrameTitle = frame.title || getAttachmentRuntime().activeFrameTitle || "";
  getAttachmentRuntime().activeFrameGoal = frame.goal || getAttachmentRuntime().activeFrameGoal || "";
  getAttachmentRuntime().lastFocusSnapshot = {
    decisions: Array.isArray(fs?.decisions) ? fs.decisions : [],
    constraints: Array.isArray(fs?.constraints) ? fs.constraints : [],
    failures: sanitizeFocusFailures(Array.isArray(fs?.failures) ? fs.failures : []),
    intent: fs?.intent || "",
    currentFocus: fs?.current_focus || fs?.current_state || getLastTrajectoryClarity()?.short_term_goal || "",
  };

  return { frame, fs, stack };
}

// ── Get Focus State from Focusa scoped to Pi's own frame (§33.5 isolation) ──
// CRITICAL: Never use Focusa's global active_frame_id — that belongs to Wirebot.
// Pi sessions must only read their own frame. If Pi has no frame, return empty.
export function getCachedFocusState(): { frame: any; fs: any; stack: any } | null {
  const cacheKey = `${getAttachmentRuntime().activeFrameId || ""}|${getAttachmentRuntime().sessionFrameKey || ""}`;
  return getAttachmentRuntime().focusStateCache.key === cacheKey
    ? getAttachmentRuntime().focusStateCache.data
    : null;
}

export function getCachedSemanticMemorySummary(): any {
  return getAttachmentRuntime().semanticMemoryCache.data || null;
}

export function getCachedEcsHandlesSummary(): any {
  return getAttachmentRuntime().ecsHandlesCache.data || null;
}

export async function getFocusState(): Promise<{ frame: any; fs: any; stack: any } | null> {
  if (!getAttachmentRuntime().activeFrameId && !getAttachmentRuntime().sessionFrameKey) return null;

  const cacheKey = `${getAttachmentRuntime().activeFrameId || ""}|${getAttachmentRuntime().sessionFrameKey || ""}`;
  const now = Date.now();
  if (
    getAttachmentRuntime().focusStateCache.data &&
    getAttachmentRuntime().focusStateCache.key === cacheKey &&
    now - getAttachmentRuntime().focusStateCache.at < FOCUS_STATE_CACHE_TTL_MS
  ) {
    return getAttachmentRuntime().focusStateCache.data;
  }
  if (
    getAttachmentRuntime().focusStateCache.inflight &&
    getAttachmentRuntime().focusStateCache.key === cacheKey
  ) {
    return await getAttachmentRuntime().focusStateCache.inflight;
  }

  const inflight = loadFocusState();
  getAttachmentRuntime().focusStateCache.key = cacheKey;
  getAttachmentRuntime().focusStateCache.inflight = inflight;
  try {
    const data = await inflight;
    if (data) {
      getAttachmentRuntime().focusStateCache.data = data;
      getAttachmentRuntime().focusStateCache.at = Date.now();
    }
    return data;
  } finally {
    if (getAttachmentRuntime().focusStateCache.inflight === inflight)
      getAttachmentRuntime().focusStateCache.inflight = null;
  }
}

export async function getSemanticMemorySummary(): Promise<any> {
  const now = Date.now();
  if (
    getAttachmentRuntime().semanticMemoryCache.data &&
    now - getAttachmentRuntime().semanticMemoryCache.at < AUX_CONTEXT_CACHE_TTL_MS
  ) {
    return getAttachmentRuntime().semanticMemoryCache.data;
  }
  if (getAttachmentRuntime().semanticMemoryCache.inflight)
    return await getAttachmentRuntime().semanticMemoryCache.inflight;

  const inflight = focusaFetch(`/memory/semantic?limit=${CONTEXT_SEMANTIC_LIMIT}&summary_only=true`);
  getAttachmentRuntime().semanticMemoryCache.inflight = inflight;
  try {
    const data = await inflight;
    if (data) {
      getAttachmentRuntime().semanticMemoryCache.data = data;
      getAttachmentRuntime().semanticMemoryCache.at = Date.now();
    }
    return data;
  } finally {
    if (getAttachmentRuntime().semanticMemoryCache.inflight === inflight)
      getAttachmentRuntime().semanticMemoryCache.inflight = null;
  }
}

export async function getEcsHandlesSummary(): Promise<any> {
  const now = Date.now();
  if (
    getAttachmentRuntime().ecsHandlesCache.data &&
    now - getAttachmentRuntime().ecsHandlesCache.at < AUX_CONTEXT_CACHE_TTL_MS
  ) {
    return getAttachmentRuntime().ecsHandlesCache.data;
  }
  if (getAttachmentRuntime().ecsHandlesCache.inflight)
    return await getAttachmentRuntime().ecsHandlesCache.inflight;

  const inflight = focusaFetch(`/ecs/handles?limit=${CONTEXT_ECS_HANDLES_LIMIT}&summary_only=true`);
  getAttachmentRuntime().ecsHandlesCache.inflight = inflight;
  try {
    const data = await inflight;
    if (data) {
      getAttachmentRuntime().ecsHandlesCache.data = data;
      getAttachmentRuntime().ecsHandlesCache.at = Date.now();
    }
    return data;
  } finally {
    if (getAttachmentRuntime().ecsHandlesCache.inflight === inflight)
      getAttachmentRuntime().ecsHandlesCache.inflight = null;
  }
}

export function trimFrameText(text: string, max = 80): string {
  const normalized = String(text || "")
    .replace(/\s+/g, " ")
    .trim();
  if (!normalized) return "";
  return normalized.length <= max ? normalized : `${normalized.slice(0, max - 1)}…`;
}

function derivePiFrameIntent(cwd: string): { projectName: string; title: string; goal: string } {
  const projectName = cwd.split("/").filter(Boolean).pop() || "root";
  const ask = trimFrameText(getAttachmentRuntime().currentAsk?.text || "", 100);
  const askKind = getAttachmentRuntime().currentAsk?.kind || "unknown";

  if (ask && askKind !== "meta") {
    const titlePrefix =
      askKind === "question" ? "Pi Question" : askKind === "correction" ? "Pi Correction" : "Pi Task";
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
  if (getAttachmentRuntime().continuityId) return getAttachmentRuntime().continuityId;
  const root =
    String(cwd || getAttachmentRuntime().sessionCwd || process.cwd())
      .split("/")
      .filter(Boolean)
      .pop() || "root";
  let randomPart = `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 10)}`;
  try {
    randomPart = require("crypto").randomUUID();
  } catch {
    /* fallback above */
  }
  getAttachmentRuntime().continuityId = `focusa-cont-${root}-${randomPart}`
    .replace(/[^a-zA-Z0-9._:-]/g, "-")
    .slice(0, 140);
  return getAttachmentRuntime().continuityId;
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
      const parsed = lines
        .map((line) => {
          try {
            return JSON.parse(line);
          } catch {
            return null;
          }
        })
        .filter(Boolean) as Array<{
        id?: string;
        status?: string;
        priority?: number;
        updated_at?: string;
        created_at?: string;
      }>;
      const preferred =
        parsed
          .filter((issue) => typeof issue.id === "string" && issue.id.trim() && issue.status !== "closed")
          .sort(
            (a, b) =>
              Number(a.priority ?? 4) - Number(b.priority ?? 4) ||
              String(b.updated_at || b.created_at || "").localeCompare(
                String(a.updated_at || a.created_at || "")
              )
          )[0] || parsed.find((issue) => typeof issue.id === "string" && issue.id.trim());
      if (preferred?.id) return preferred.id.trim();
    } catch {
      /* try next path */
    }
  }
  // Fallback: project has no Beads issues at all. Auto-create a single
  // open P0 issue to anchor the focus frame, so focusa_decide/constraint
  // can write to Focus State instead of being rejected with
  // "Attentive and awaiting operator direction" (bead focusa-oh7t).
  return ensureAutocreatedBeadsIssueForProject(projectRoot);
}

function ensureAutocreatedBeadsIssueForProject(projectRoot: string): string | null {
  try {
    const beadsDir = `${projectRoot.replace(/\/+$/, "")}/.beads`;
    const issuesFile = `${beadsDir}/issues.jsonl`;
    if (!existsSync(beadsDir)) {
      // No .beads directory at all — refuse to create one; the operator
      // must opt in. This avoids polluting unrelated project directories.
      return null;
    }
    if (!existsSync(issuesFile)) {
      // Empty .beads directory is acceptable; the JSONL will be created on first write.
    }
    const stamp = new Date()
      .toISOString()
      .replace(/[^0-9]/g, "")
      .slice(0, 14);
    const id = `pi-auto-${stamp}-${process.pid}`;
    const issue = {
      id,
      title: "Pi focus frame anchor (auto-created)",
      status: "open",
      priority: 0,
      type: "task",
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString(),
    };
    appendFileSync(issuesFile, JSON.stringify(issue) + "\n", "utf8");
    return id;
  } catch {
    return null;
  }
}

export async function ensureFocusaSessionForFrame(
  projectRoot: string,
  continuityId: string
): Promise<boolean> {
  if (!isProjectRootAuthoritySafe(projectRoot) || !continuityId) return false;
  const status = await focusaFetch("/status").catch(() => null);
  const session = status?.session;
  if (
    session?.status === "active" &&
    normalizeProjectRoot(session.project_root) === normalizeProjectRoot(projectRoot)
  ) {
    return true;
  }
  const started = await focusaFetch("/session/start", {
    method: "POST",
    body: JSON.stringify({
      adapter_id: "pi",
      workspace_id: projectRoot,
      project_root: projectRoot,
      continuity_id: continuityId,
    }),
  }).catch(() => null);
  return started?.status === "accepted";
}

export async function createPiFrame(cwd: string, source = "pi-auto"): Promise<string | null> {
  getAttachmentRuntime().sessionCwd = cwd;
  const { projectName, title, goal } = derivePiFrameIntent(cwd);
  getAttachmentRuntime().activeFrameTitle = title;
  getAttachmentRuntime().activeFrameGoal = goal;
  const sessionKey = getAttachmentRuntime().sessionFrameKey || `pi-${process.pid}-${Date.now()}`;
  getAttachmentRuntime().sessionFrameKey = sessionKey;
  const continuityId = ensureContinuityId(cwd);
  const beadsIssueId = selectExistingBeadsIssueIdForFocusFrame(cwd);
  if (!beadsIssueId) {
    focusaPost("/telemetry/trace", {
      event_type: "pi_frame_creation_blocked_missing_beads_issue",
      payload: { project_root: cwd, source },
    });
    return null;
  }
  const tags = [
    "pi",
    projectName,
    source,
    sessionKey,
    continuityId,
    `continuity_id:${continuityId}`,
    "task-first-frame",
  ];

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
      getAttachmentRuntime().activeFrameId = r.frame_id;
      return r.frame_id;
    }

    for (let i = 0; i < 10; i++) {
      await new Promise((resolve) => setTimeout(resolve, 300));
      const stack = await focusaFetch("/focus/stack");
      const frames = stack?.stack?.frames || [];
      const match = [...frames]
        .reverse()
        .find(
          (f: any) =>
            f.title === title &&
            f.beads_issue_id === beadsIssueId &&
            Array.isArray(f.tags) &&
            (f.continuity_id === continuityId ||
              f.tags.includes(continuityId) ||
              f.tags.includes(`continuity_id:${continuityId}`))
        );
      if (match?.id) {
        getAttachmentRuntime().activeFrameId = match.id;
        getAttachmentRuntime().activeFrameTitle = match.title || title;
        getAttachmentRuntime().activeFrameGoal = match.goal || goal;
        return match.id;
      }
    }
  } catch {}
  return null;
}

export function normalizeProjectRoot(value: unknown): string {
  const normalized = String(value || "")
    .trim()
    .replace(/\/+$/, "");
  return normalized === "" ? "" : normalized;
}

// QN Addendum (2026-06-08): Agent runtime paths must NEVER be treated as project scope
// Matches Rust unsafe_project_root_reason() in project.rs and workpoint.rs
const UNSAFE_PROJECT_AUTHORITY_ROOTS = new Set([
  "/",
  "/root",
  "/home",
  "/Users",
  "/tmp",
  "/var",
  "/usr",
  "/opt",
]);

// Agent runtime directory patterns - must check before project markers
const AGENT_RUNTIME_PATTERNS = [
  // Pi agent
  /\/root\/pi-mono$/,
  /^\/root\/pi-/, // /root/pi-*
  // Node/npm agent installs
  /^\/opt\/node-/, // /opt/node-*
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

function confidenceForScore(score: number): {
  confidence: "high" | "medium" | "low";
  confidenceScore: number;
} {
  if (score >= 10_000) return { confidence: "high", confidenceScore: 0.99 };
  if (score >= 8_000) return { confidence: "high", confidenceScore: 0.95 };
  if (score >= 6_000) return { confidence: "high", confidenceScore: 0.9 };
  if (score >= 2_000) return { confidence: "medium", confidenceScore: 0.75 };
  return { confidence: "low", confidenceScore: 0.25 };
}

function projectRootScoreRequiresConfirmation(confidenceScore: number): boolean {
  return confidenceScore < 0.9;
}
function findAncestorProjectRootCandidates(
  start: string
): Array<{ root: string; score: number; depth: number; markers: string[] }> {
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
  return candidates.sort((a, b) => b.score - a.score || a.depth - b.depth);
}

function rootCandidatesForOutput(
  candidates: Array<{ root: string; score: number; markers: string[] }>
): Array<{ projectRoot: string; confidenceScore: number; markers: string[]; source: string }> {
  return candidates.slice(0, 5).map((candidate) => ({
    projectRoot: candidate.root,
    confidenceScore: confidenceForScore(candidate.score).confidenceScore,
    markers: candidate.markers,
    source: "ancestor_markers",
  }));
}

type ProjectRootResolution = {
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
};

export function resolvePiProjectRootCandidate(
  cwdInput?: unknown,
  persistedPacket?: any
): ProjectRootResolution {
  const explicit = normalizeProjectRoot(cwdInput);
  const explicitCandidates = explicit ? findAncestorProjectRootCandidates(explicit) : [];
  const explicitCandidate = explicitCandidates[0] || null;
  if (explicitCandidate) {
    const confidence = confidenceForScore(explicitCandidate.score);
    return {
      projectRoot: explicitCandidate.root,
      ...confidence,
      source: "cwd_ancestor_markers",
      reason: `markers=${explicitCandidate.markers.join(",")}`,
      safe: true,
      requiresOperatorConfirmation: projectRootScoreRequiresConfirmation(confidence.confidenceScore),
      markerScore: explicitCandidate.score,
      markers: explicitCandidate.markers,
      candidates: rootCandidatesForOutput(explicitCandidates),
    };
  }

  const sessionRoot = normalizeProjectRoot(getAttachmentRuntime().sessionCwd);
  const sessionCandidates =
    sessionRoot && sessionRoot !== explicit ? findAncestorProjectRootCandidates(sessionRoot) : [];
  const sessionCandidate = sessionCandidates[0] || null;
  if (sessionCandidate) {
    const confidence = confidenceForScore(sessionCandidate.score);
    return {
      projectRoot: sessionCandidate.root,
      ...confidence,
      source: "session_cwd_ancestor_markers",
      reason: `markers=${sessionCandidate.markers.join(",")}`,
      safe: true,
      requiresOperatorConfirmation: projectRootScoreRequiresConfirmation(confidence.confidenceScore),
      markerScore: sessionCandidate.score,
      markers: sessionCandidate.markers,
      candidates: rootCandidatesForOutput(sessionCandidates),
    };
  }

  const packet = persistedPacket?.resume_packet?.workpoint || persistedPacket?.workpoint || persistedPacket;
  const packetRoot = normalizeProjectRoot(packet?.project_root);
  const packetSessionKey = String(packet?.pi_session_frame_key || packet?.session_id || "").trim();
  const currentSessionKey = String(getAttachmentRuntime().sessionFrameKey || "").trim();
  if (
    packetRoot &&
    isProjectRootAuthoritySafe(packetRoot) &&
    currentSessionKey &&
    packetSessionKey === currentSessionKey
  ) {
    return {
      projectRoot: packetRoot,
      confidence: "medium",
      confidenceScore: 0.75,
      source: "same_session_workpoint_packet",
      reason: "same-session Workpoint packet supplied project_root; operator confirmation recommended",
      safe: true,
      requiresOperatorConfirmation: true,
      candidates: [
        {
          projectRoot: packetRoot,
          confidenceScore: 0.75,
          markers: ["workpoint_packet"],
          source: "same_session_workpoint_packet",
        },
      ],
    };
  }

  const fallback = explicit || sessionRoot || normalizeProjectRoot(process.cwd());
  const safe = isProjectRootAuthoritySafe(fallback);
  return {
    projectRoot: fallback,
    confidence: "low",
    confidenceScore: 0.1,
    source: "unverified_cwd",
    reason: safe
      ? "no project markers found; ask operator or pass explicit project_root"
      : projectRootAuthorityFailure(fallback) || "unsafe_project_root",
    safe,
    requiresOperatorConfirmation: true,
    candidates: safe
      ? [{ projectRoot: fallback, confidenceScore: 0.1, markers: [], source: "unverified_cwd" }]
      : [],
  };
}

export function resolvePiProjectRoot(cwdInput?: unknown, persistedPacket?: any): string {
  return resolvePiProjectRootCandidate(cwdInput, persistedPacket).projectRoot;
}

export function resolveFocusWriteProjectRoot(liveCwdInput: unknown, cachedCwdInput: unknown): string {
  const live = resolvePiProjectRootCandidate(liveCwdInput);
  if (
    live.safe === true &&
    live.requiresOperatorConfirmation !== true &&
    isProjectRootAuthoritySafe(live.projectRoot)
  )
    return live.projectRoot;

  const cached = resolvePiProjectRootCandidate(cachedCwdInput);
  if (
    cached.safe === true &&
    cached.requiresOperatorConfirmation !== true &&
    isProjectRootAuthoritySafe(cached.projectRoot)
  )
    return cached.projectRoot;

  return live.projectRoot;
}

export function projectRootConfirmationRequired(projectRoot?: string): boolean {
  const resolution = getLastProjectRootResolution();
  if (!resolution) return false;
  if (projectRoot && normalizeProjectRoot(projectRoot) !== normalizeProjectRoot(resolution.projectRoot))
    return false;
  return resolution.requiresOperatorConfirmation === true || resolution.safe !== true;
}

export function projectRootConfirmationSummary(projectRoot?: string): string {
  const resolution = getLastProjectRootResolution();
  if (
    !resolution ||
    (projectRoot && normalizeProjectRoot(projectRoot) !== normalizeProjectRoot(resolution.projectRoot))
  )
    return "project root is unverified";
  const candidates = (resolution.candidates || [])
    .slice(0, 3)
    .map((candidate) => `${candidate.projectRoot} (${Math.round(candidate.confidenceScore * 100)}%)`)
    .join(", ");
  return `project_root=${resolution.projectRoot} confidence=${Math.round(resolution.confidenceScore * 100)}% source=${resolution.source}; ${resolution.reason}${candidates ? `; candidates: ${candidates}` : ""}`;
}

export function adoptPiProjectRoot(cwdInput?: unknown, persistedPacket?: any): string {
  const resolution = resolvePiProjectRootCandidate(cwdInput, persistedPacket);
  setLastProjectRootResolution(resolution);
  getAttachmentRuntime().sessionCwd = resolution.projectRoot;
  return resolution.projectRoot;
}

export function confirmPiProjectRoot(
  projectRootInput: unknown,
  source = "operator_confirmed_project_root"
): string | null {
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
    candidates: base.candidates?.length
      ? base.candidates
      : [{ projectRoot, confidenceScore: 0.95, markers: base.markers || ["operator_confirmed"], source }],
  };
  setLastProjectRootResolution(confirmed);
  getAttachmentRuntime().sessionCwd = projectRoot;
  return projectRoot;
}

export function normalizeWorkpointResumePacketEnvelope(packet: any): any | null {
  if (!packet || typeof packet !== "object") return null;
  const base =
    packet.resume_packet && typeof packet.resume_packet === "object" ? packet.resume_packet : packet;
  if (!base || typeof base !== "object") return null;
  const normalized = { ...base };
  if (packet.resume_packet_v2 && typeof packet.resume_packet_v2 === "object")
    normalized.resume_packet_v2 = packet.resume_packet_v2;
  if (packet.rendered_summary && !normalized.rendered_summary)
    normalized.rendered_summary = packet.rendered_summary;
  if (packet.schema_version && !normalized.envelope_schema_version)
    normalized.envelope_schema_version = packet.schema_version;
  return normalized;
}

export async function buildFocusaSessionIdentity(
  projectRootInput?: string,
  resumeSource:
    | "session_start"
    | "session_switch"
    | "compaction"
    | "model_switch"
    | "fork"
    | "manual"
    | "unknown" = "manual",
  overrides: { continuityId?: string; sessionId?: string } = {}
): Promise<Record<string, unknown>> {
  const projectRoot = normalizeProjectRoot(
    projectRootInput || getAttachmentRuntime().sessionCwd || process.cwd()
  );
  const safe = isProjectRootAuthoritySafe(projectRoot);
  const ambientCwd = normalizeProjectRoot(getAttachmentRuntime().sessionCwd || process.cwd());
  const persisted = getLastProjectIdentity() || {};
  const persistedBody: any = (persisted as any).project_identity || persisted;
  const cwdForIdentity = safe
    ? resolveProjectIdentityLookupCwd({ projectRoot, ambientCwd, persistedIdentity: persisted })
    : ambientCwd;
  const sessionId = String(overrides.sessionId || getAttachmentRuntime().sessionFrameKey || "").trim();
  let projectIdentity: any = null;
  if (safe) {
    const query = new URLSearchParams();
    query.set("cwd", cwdForIdentity);
    // `projectRoot` is normally the ambient Pi cwd, not operator-confirmed
    // authority. Only a local project marker may promote a canonical root.
    const markerProjectRoot = resolveCanonicalMarkerProjectRoot(cwdForIdentity);
    const authorityProjectRoot = markerProjectRoot || projectRoot;
    if (markerProjectRoot) query.set("project_root", markerProjectRoot);
    if (sessionId) query.set("pi_session_id", sessionId);
    const remoteContext: any = persistedBody.remote_context || {};
    if (remoteContext.remote_host) query.set("remote_host", String(remoteContext.remote_host));
    if (remoteContext.remote_user) query.set("remote_user", String(remoteContext.remote_user));
    if (remoteContext.remote_port) query.set("remote_port", String(remoteContext.remote_port));
    if (remoteContext.remote_repo_remote)
      query.set("remote_repo_remote", String(remoteContext.remote_repo_remote));
    if (remoteContext.remote_workspace_kind)
      query.set("remote_workspace_kind", String(remoteContext.remote_workspace_kind));
    if (remoteContext.remote_deploy_root)
      query.set("remote_deploy_root", String(remoteContext.remote_deploy_root));
    if (normalizeProjectRoot(persistedBody.project_root) === authorityProjectRoot) {
      if (persistedBody.project_root)
        query.set("persisted_project_root", normalizeProjectRoot(persistedBody.project_root));
      if (persistedBody.fingerprint)
        query.set("persisted_project_fingerprint", String(persistedBody.fingerprint));
      if (persistedBody.project_id)
        query.set("persisted_project_id", String(persistedBody.project_id));
      if (persistedBody.canonical_name)
        query.set("persisted_canonical_name", String(persistedBody.canonical_name));
    }
    const response = await focusaFetch(`/project/identity?${query.toString()}`).catch(() => null);
    projectIdentity = response?.project_identity || null;
    if (projectIdentity) setLastProjectIdentity(projectIdentity);
  }
  const canonicalProjectRoot = normalizeProjectRoot(
    projectIdentity?.canonical_parent_root || projectIdentity?.project_root || projectRoot
  );
  const activeWorktreeRoot = normalizeProjectRoot(
    projectIdentity?.active_worktree_root ||
      projectIdentity?.working_context?.active_worktree_root ||
      cwdForIdentity
  );
  const workingSubpath = projectIdentity?.working_context?.working_subpath || null;
  const workingSubpathId = String(workingSubpath?.working_subpath_id || "primary").trim();
  const sharedBeadsRoot = String(workingSubpath?.beads_root || "").trim();
  const workingBeadsPrefix = String(workingSubpath?.beads_prefix || "").trim();
  if (sharedBeadsRoot) process.env.BEADS_DIR = sharedBeadsRoot;
  else delete process.env.BEADS_DIR;
  if (workingSubpathId) process.env.FOCUSA_WORKING_SUBPATH_ID = workingSubpathId;
  else delete process.env.FOCUSA_WORKING_SUBPATH_ID;
  if (workingBeadsPrefix) process.env.FOCUSA_BEADS_PREFIX = workingBeadsPrefix;
  else delete process.env.FOCUSA_BEADS_PREFIX;
  const continuityId = String(
    overrides.continuityId || ensureContinuityId(canonicalProjectRoot || process.cwd()) || ""
  ).trim();
  const rootParts = activeWorktreeRoot.split("/").filter(Boolean);
  const resolution = (
    getLastProjectRootResolution() &&
    normalizeProjectRoot(getLastProjectRootResolution()!.projectRoot) === projectRoot
      ? getLastProjectRootResolution()
      : resolvePiProjectRootCandidate(projectRootInput || getSessionCwd() || process.cwd())
  )!;
  return {
    schema: "focusa.session_identity.v1",
    project_identity: projectIdentity,
    pi_session_id: sessionId || undefined,
    session_frame_key: sessionId || "unknown-session",
    session_incarnation_id: `${sessionId || "unknown"}:${process.pid}:${getAttachmentRuntime().sessionStartTime}`,
    continuity_id: continuityId || undefined,
    project_root: canonicalProjectRoot,
    canonical_parent_root: canonicalProjectRoot,
    cwd: activeWorktreeRoot,
    active_worktree_root: activeWorktreeRoot,
    working_subpath_id: workingSubpathId,
    working_subpath: workingSubpath,
    workspace_id: workingSubpathId || rootParts[rootParts.length - 1] || "workspace",
    process_id: process.pid,
    started_at: new Date(getAttachmentRuntime().sessionStartTime).toISOString(),
    resume_source: resumeSource,
    session_project_classification: getAttachmentRuntime().sessionProjectClassification,
    session_project_registry_record: sessionId
      ? getAttachmentRuntime().piSessionProjectRegistry[sessionId]
      : undefined,
    canonical_scope: safe && !resolution.requiresOperatorConfirmation,
    scope_failure: safe
      ? resolution.requiresOperatorConfirmation
        ? "project_root_confirmation_required"
        : null
      : projectRootAuthorityFailure(projectRoot),
    project_root_confidence: resolution.confidence,
    project_root_confidence_score: resolution.confidenceScore,
    project_root_resolution_source: resolution.source,
    requires_operator_confirmation: resolution.requiresOperatorConfirmation,
    project_root_candidates: resolution.candidates || [],
  };
}

export async function refreshTrajectoryClarityLifecycle(
  reason: string,
  projectRootInput?: string
): Promise<Record<string, unknown> | null> {
  if (!getAttachmentRuntime().focusaAvailable) return null;
  const projectRoot = normalizeProjectRoot(
    projectRootInput || getAttachmentRuntime().sessionCwd || process.cwd()
  );
  const continuityId = String(getAttachmentRuntime().continuityId || "").trim();
  let expectedScope;
  try {
    expectedScope = buildProjectWorkstreamKey(projectRoot, continuityId);
  } catch {
    return null;
  }
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
  if (getAttachmentRuntime().sessionFrameKey) query.set("session_id", getAttachmentRuntime().sessionFrameKey);
  query.set("continuity_id", continuityId);
  try {
    const view = await focusaFetch(`/trajectory/view?${query.toString()}`);
    const projectIdentity = view?.project_identity || {};
    const projectIdentityApi = projectIdentity?.project_identity_api || {};
    const scopeRef = projectIdentity?.scope_ref || projectIdentityApi?.scope_ref || {};
    const candidateRoot = normalizeProjectRoot(
      projectIdentity?.project_root || projectIdentityApi?.project_root || ""
    );
    const scopeRoot = normalizeProjectRoot(scopeRef?.root_path || "");
    const candidateContinuity = String(projectIdentity?.continuity_id || "").trim();
    const trajectoryId = String(view?.trajectory?.trajectory_id || "").trim();
    const receipt = view?.trajectory?.scope_verification || {};
    const receiptScope = receipt?.scope_ref || {};
    if (
      projectIdentity?.status !== "verified" ||
      scopeRef?.scope_kind !== expectedScope.root_scope.scope_kind ||
      String(scopeRef?.scope_id || "") !== expectedScope.root_scope.scope_id ||
      String(scopeRef?.fingerprint || "") !== expectedScope.root_scope.fingerprint ||
      candidateRoot !== expectedScope.root_scope.root_path ||
      scopeRoot !== expectedScope.root_scope.root_path ||
      candidateContinuity !== expectedScope.continuity_id ||
      !trajectoryId ||
      String(receipt?.rendered_trajectory_id || "").trim() !== trajectoryId ||
      !String(receipt?.source_trajectory_id || "").trim() ||
      normalizeProjectRoot(receipt?.project_root || "") !== expectedScope.root_scope.root_path ||
      receiptScope?.scope_kind !== expectedScope.root_scope.scope_kind ||
      String(receiptScope?.scope_id || "") !== expectedScope.root_scope.scope_id ||
      String(receiptScope?.fingerprint || "") !== expectedScope.root_scope.fingerprint ||
      normalizeProjectRoot(receiptScope?.root_path || "") !== expectedScope.root_scope.root_path
    ) {
      throw new Error("trajectory_scope_verification_failed");
    }
    if (view?.trajectory?.fallback_prior_project_trajectory === true) {
      const sourceContinuity = String(view?.trajectory?.fallback_source_continuity_id || "").trim();
      if (
        receipt?.status !== "verified_same_project_fallback" ||
        !sourceContinuity ||
        sourceContinuity === expectedScope.continuity_id ||
        String(receipt?.continuity_id || "").trim() !== sourceContinuity
      ) {
        throw new Error("trajectory_fallback_source_invalid");
      }
    } else if (
      receipt?.status !== "verified_exact" ||
      String(receipt?.continuity_id || "").trim() !== expectedScope.continuity_id
    ) {
      throw new Error("trajectory_exact_scope_receipt_invalid");
    }
    const clarity = view?.intelligence_view?.clarity_gate || {};
    const snapshot = {
      reason,
      refreshed_at: Date.now(),
      project_root: projectRoot,
      continuity_id: getAttachmentRuntime().continuityId || null,
      session_id: getAttachmentRuntime().sessionFrameKey || null,
      status: String(clarity.status || view?.trajectory?.definition_status || "unknown"),
      recommended_action: String(
        clarity.recommended_action ||
          view?.intelligence_view?.context_sufficiency?.recommended_action ||
          "unknown"
      ),
      canonical: view?.canonical === true,
      degraded: view?.degraded === true,
      project_identity_status: String(view?.project_identity?.status || "unknown"),
      trajectory_id: view?.trajectory?.trajectory_id || null,
      scope_verification: view?.trajectory?.scope_verification || null,
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
      project_urls:
        view?.project_identity?.project_urls ||
        view?.project_identity?.project_summary?.urls ||
        view?.project?.project_urls ||
        null,
      deployment:
        view?.project_identity?.deployment ||
        view?.project_identity?.project_summary?.deployment ||
        view?.project?.deployment ||
        null,
      // Spec 125 §9.5: provenance fields.
      hlt_status: view?.hlt_status || null,
      trajectory_required: view?.trajectory_required ?? true,
      hlt_required: view?.hlt_required ?? true,
      action_authority_from_trajectory: view?.action_authority_from_trajectory ?? false,
      generic_bootstrap: view?.generic_bootstrap ?? false,
      loud_warning: view?.loud_warning || null,
      // Spec 125 §9.5: fallback metadata.
      allow_previous_valid_trajectory: view?.allow_previous_valid_trajectory ?? false,
      previous_valid_trajectory_fallback: view?.previous_valid_trajectory_fallback ?? false,
      fallback_level: view?.fallback_level || "none",
      fallback_source_scope: view?.fallback_source_scope || null,
      // Spec 125 §9.5: provenance for no canonical local backfill.
      provenance: {
        source: "trajectory_view_api",
        project_root: projectRoot,
        session_id: getAttachmentRuntime().sessionFrameKey || null,
        continuity_id: getAttachmentRuntime().continuityId || null,
        refreshed_at: Date.now(),
      },
      next_tools: view?.next_tools || [
        "focusa_trajectory_view",
        "focusa_project_verify",
        "focusa_workpoint_resume",
      ],
    };
    setLastTrajectoryClarity(snapshot);
    if (snapshot.project_identity) setLastProjectIdentity(snapshot.project_identity);
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
  getAttachmentRuntime().continuityId = "";
  getAttachmentRuntime().activeFrameId = null;
  getAttachmentRuntime().activeFrameTitle = "";
  getAttachmentRuntime().activeFrameGoal = "";
  focusaPost("/telemetry/trace", {
    event_type: "pi_scope_rejected_unsafe_cwd",
    payload: { reason, cwd: getAttachmentRuntime().sessionCwd || process.cwd() },
  });
}

export function stampWorkpointPacketForCurrentPiSession(packet: any): any {
  if (!packet || typeof packet !== "object") return packet;
  return {
    ...packet,
    pi_session_frame_key: getAttachmentRuntime().sessionFrameKey || null,
    pi_session_scope_checked_at: new Date().toISOString(),
  };
}

export function isWorkpointPacketScopedToCurrentSession(packet: any): boolean {
  if (!packet || typeof packet !== "object") return false;
  const currentProjectRoot = resolvePiProjectRoot(getAttachmentRuntime().sessionCwd || process.cwd());
  const currentContinuityId = String(getAttachmentRuntime().continuityId || "").trim();
  const packetProjectRoot = normalizeProjectRoot(packet.project_root);
  const packetContinuityId = String(packet.continuity_id || "").trim();
  if (!currentProjectRoot || !currentContinuityId || !packetProjectRoot || !packetContinuityId) return false;
  if (!isProjectRootAuthoritySafe(currentProjectRoot) || !isProjectRootAuthoritySafe(packetProjectRoot))
    return false;
  if (currentProjectRoot !== packetProjectRoot) return false;
  if (currentContinuityId !== packetContinuityId) return false;
  // Pi session ids are temporal metadata, never Workpoint identity. Exact
  // project_root + continuity_id plus current-ask authority is the boundary;
  // accepted packets are re-stamped for this Pi session.
  if (
    packet.canonical === false ||
    packet.status === "partial" ||
    packet.status === "rejected_scope_mismatch"
  )
    return false;
  return true;
}

export function getScopedWorkpointPacket(): any | null {
  return isWorkpointPacketScopedToCurrentSession(getActiveWorkpointPacket())
    ? getActiveWorkpointPacket()
    : null;
}

const verifiedContinuityBySessionRoot = new Map<string, string>();

function verifiedContinuityKey(sessionId: string, projectRoot: string): string {
  return `${String(sessionId || "").trim()}|${normalizeProjectRoot(projectRoot)}`;
}

export function adoptVerifiedContinuityForCurrentSession(
  projectRoot: string,
  continuityId: string
): boolean {
  const root = normalizeProjectRoot(projectRoot);
  const continuity = String(continuityId || "").trim();
  const identityEnvelope: any = getLastProjectIdentity() || {};
  const identity: any = identityEnvelope.project_identity || identityEnvelope;
  const decision: any = currentProjectBindingDecision() || {};
  const verifiedRoot = normalizeProjectRoot(
    resolveCanonicalMarkerProjectRoot(process.cwd()) ||
      identity.canonical_parent_root ||
      identity.project_root ||
      decision.selected_project_root
  );
  if (!root || !continuity || !isProjectRootAuthoritySafe(root) || verifiedRoot !== root) return false;
  getAttachmentRuntime().continuityId = continuity;
  verifiedContinuityBySessionRoot.set(
    verifiedContinuityKey(getSessionFrameKey(), root),
    continuity
  );
  syncRuntimeFieldsToScopeStore();
  return true;
}

export function adoptPersistedContinuityForSession(data: any, eventSessionId: string, cwd: string): void {
  const persistedSessionId = String(data?.sessionId || "").trim();
  const persistedContinuityId = String(data?.continuityId || "").trim();
  if (!persistedSessionId || persistedSessionId !== eventSessionId || !persistedContinuityId) {
    setActiveWorkpointPacket(null);
    setActiveWorkpointSummary("");
    return;
  }
  getAttachmentRuntime().continuityId = persistedContinuityId;
  const packet = data?.activeWorkpointPacket || null;
  const packetProjectRoot = normalizeProjectRoot(packet?.project_root);
  const packetContinuityId = String(packet?.continuity_id || "").trim();
  if (
    packet &&
    isProjectRootAuthoritySafe(cwd) &&
    isProjectRootAuthoritySafe(packetProjectRoot) &&
    packetProjectRoot === normalizeProjectRoot(cwd) &&
    packetContinuityId === persistedContinuityId &&
    packet.canonical !== false &&
    packet.status !== "partial" &&
    packet.status !== "rejected_scope_mismatch"
  ) {
    setActiveWorkpointPacket(stampWorkpointPacketForCurrentPiSession(packet));
    setActiveWorkpointSummary(String(data?.activeWorkpointSummary || ""));
    getAttachmentRuntime().lastWorkpointUpdate = Date.now();
  } else {
    setActiveWorkpointPacket(null);
    setActiveWorkpointSummary("");
  }
}

// ── Build compact instructions with local shadow (§33.10) ────────────────────
export function buildCompactInstructions(prefix: string): string {
  const base =
    getAttachmentRuntime().cfg?.compactInstructions ||
    "Preserve intent, decisions, constraints, next_steps, failures.";
  const workpoint = getScopedWorkpointPacket() || {};
  const mission = String(
    workpoint?.mission ||
      getAttachmentRuntime().currentAsk?.text ||
      getAttachmentRuntime().activeFrameGoal ||
      getAttachmentRuntime().activeFrameTitle ||
      ""
  ).trim();
  const nextSlice = String(workpoint?.next_slice || getAttachmentRuntime().lastCompactDecision || "").trim();
  const projectRoot = String(
    workpoint?.project_root ||
      (isProjectRootAuthoritySafe(getAttachmentRuntime().sessionCwd)
        ? getAttachmentRuntime().sessionCwd
        : "") ||
      ""
  ).trim();
  const attentionLines = formatAttentionRecallFocusSliceLines(
    buildAttentionRecallVerdict({
      workpointPacket: workpoint,
      currentAskText: getAttachmentRuntime().currentAsk?.text,
      projectRoot,
    })
  );
  const parts = [
    prefix,
    "\n" + attentionLines.join("\n"),
    "\n" + base,
    "\nFallback policy: never emit bare 'none' for Focusa Cognitive Summary fields. If a slot is empty, fill it with the nearest related canonical source: Workpoint mission/next_slice/project_root/session_id, current operator ask, active frame goal/title, local shadow decisions/constraints/failures, git/beads/evidence mentioned in the conversation. If no related source exists, say 'No recorded <field>; no safe related fallback available.'",
  ];
  if (mission) parts.push(`Fallback Mission:\n- ${mission}`);
  if (nextSlice) parts.push(`Fallback Next Step:\n- ${nextSlice}`);
  if (projectRoot) parts.push(`Fallback Scope:\n- project_root:${projectRoot}`);
  if (getAttachmentRuntime().localDecisions.length)
    parts.push(
      `Decisions:\n${getAttachmentRuntime()
        .localDecisions.map((d: string) => `- ${d}`)
        .join("\n")}`
    );
  if (getAttachmentRuntime().localConstraints.length)
    parts.push(
      `Constraints:\n${getAttachmentRuntime()
        .localConstraints.map((c: string) => `- ${c}`)
        .join("\n")}`
    );
  if (getAttachmentRuntime().localFailures.length)
    parts.push(
      `Failures:\n${getAttachmentRuntime()
        .localFailures.map((f: string) => `- ${f}`)
        .join("\n")}`
    );
  return parts.join("\n");
}

// ── wb CLI with HTTP fallback (§38.2) ────────────────────────────────────────
export async function wbExec(args: string[], fallbackUrl?: string, fallbackBody?: any): Promise<any> {
  if (getAttachmentRuntime().pi) {
    try {
      const r = await getAttachmentRuntime().pi.exec("wb", args, { timeout: 5000 });
      if (r.code === 0) {
        try {
          return JSON.parse(r.stdout);
        } catch {
          return true;
        }
      }
    } catch {
      /* fall through */
    }
  }
  if (fallbackUrl) {
    const token = getAttachmentRuntime().cfg?.scoreboardToken || "";
    try {
      const r = await fetch(fallbackUrl, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          ...(token ? { Authorization: `Bearer ${token}` } : {}),
        },
        body: JSON.stringify(fallbackBody),
        signal: AbortSignal.timeout(5000),
      });
      return r.ok ? await r.json().catch(() => true) : null;
    } catch {
      return null;
    }
  }
  return null;
}

export function isGenericPiFrameForCwd(cwd: string, title?: string | null, goal?: string | null): boolean {
  const projectName = cwd.split("/").filter(Boolean).pop() || "root";
  return (title || "") === `Pi: ${projectName}` && (goal || "") === `Work on ${projectName}`;
}

export function adoptWorkpointScopeForFrameRecovery(
  packet: any,
  source: string,
  expectedScope?: {
    projectRoot: string;
    continuityId: string;
    allowSessionTransfer: boolean;
  }
): string | null {
  if (!packet || typeof packet !== "object") return null;
  const workpoint = packet.resume_packet?.workpoint || packet.workpoint || packet;
  const packetProjectRoot = normalizeProjectRoot(workpoint.project_root || packet.project_root);
  const packetContinuityId = String(workpoint.continuity_id || packet.continuity_id || "").trim();
  const packetPiSessionKey = String(
    workpoint.pi_session_frame_key || packet.pi_session_frame_key || ""
  ).trim();
  const packetSessionId = String(workpoint.session_id || packet.session_id || "").trim();
  const currentSessionKey = String(getAttachmentRuntime().sessionFrameKey || "").trim();
  const currentContinuityId = String(getAttachmentRuntime().continuityId || "").trim();
  if (!packetProjectRoot || !packetContinuityId || !isProjectRootAuthoritySafe(packetProjectRoot))
    return null;
  if (
    packet.canonical === false ||
    workpoint.canonical === false ||
    packet.status === "partial" ||
    packet.status === "rejected_scope_mismatch"
  )
    return null;
  const explicitScopeMatch =
    expectedScope?.allowSessionTransfer === true &&
    normalizeProjectRoot(expectedScope.projectRoot) === packetProjectRoot &&
    String(expectedScope.continuityId || "").trim() === packetContinuityId;
  if (currentContinuityId && currentContinuityId !== packetContinuityId && !explicitScopeMatch) return null;
  if (
    currentSessionKey &&
    packetPiSessionKey &&
    packetPiSessionKey !== currentSessionKey &&
    !explicitScopeMatch
  )
    return null;
  if (
    currentSessionKey &&
    !packetPiSessionKey &&
    packetSessionId &&
    packetSessionId !== currentSessionKey &&
    !explicitScopeMatch
  )
    return null;
  if (currentSessionKey && !packetPiSessionKey && !packetSessionId && !explicitScopeMatch) return null;
  getAttachmentRuntime().continuityId = packetContinuityId;
  getAttachmentRuntime().sessionCwd = packetProjectRoot;
  setActiveWorkpointPacket(stampWorkpointPacketForCurrentPiSession(workpoint));
  getAttachmentRuntime().lastWorkpointUpdate = Date.now();
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
  getAttachmentRuntime().activeFrameId = frame.id;
  getAttachmentRuntime().sessionCwd = frameProjectRoot;
  if (frame.continuity_id) getAttachmentRuntime().continuityId = String(frame.continuity_id);
  return frame.id;
}

// ── Persist Focusa state to Pi session (§33.7) ──────────────────────────────
export async function ensurePiFrame(
  cwd?: string,
  sessionId?: string,
  source = "pi-auto"
): Promise<string | null> {
  // Cached health is advisory and can lag a healthy daemon after reload/restart.
  // The scoped /focus/push request below is the authoritative frame-recovery
  // check; stale focusaAvailable=false must not veto it before an API attempt.
  const requestedResolution = resolvePiProjectRootCandidate(cwd || getSessionCwd() || process.cwd());
  setLastProjectRootResolution(requestedResolution);
  const requestedCwd = requestedResolution.projectRoot;
  let resolvedCwd = requestedCwd;
  const requestedScopeUsable =
    requestedResolution.safe === true &&
    requestedResolution.requiresOperatorConfirmation !== true &&
    isProjectRootAuthoritySafe(requestedCwd);

  // Broad agent launch directories (especially /root) are not project authority,
  // but a canonical same-session Workpoint is. Recover from that packet before
  // rejecting the broad cwd; the previous order made this branch unreachable.
  if (!requestedScopeUsable) {
    const packetCwd = scopedWorkpointFrameRecoveryCwd();
    if (packetCwd) {
      resolvedCwd = packetCwd;
      setLastProjectRootResolution({
        projectRoot: packetCwd,
        confidence: "high",
        confidenceScore: 0.95,
        source: "canonical_session_workpoint_recovery",
        reason: "canonical same-session Workpoint supplied project authority",
        safe: true,
        requiresOperatorConfirmation: false,
        candidates: [
          {
            projectRoot: packetCwd,
            confidenceScore: 0.95,
            markers: ["canonical_workpoint"],
            source: "canonical_session_workpoint_recovery",
          },
        ],
      });
    } else {
      const adoptedFrameId = await adoptExistingSafeFrameForRecovery();
      if (adoptedFrameId) return adoptedFrameId;
      focusaPost("/telemetry/trace", {
        event_type: "pi_frame_creation_blocked_unconfirmed_project_root",
        payload: {
          project_root: requestedCwd,
          summary: projectRootConfirmationSummary(requestedCwd),
          source,
        },
      });
      clearScopedWorkpointForUnsafeCwd("ensure_pi_frame_unconfirmed_scope");
      return null;
    }
  }

  if (
    getAttachmentRuntime().activeFrameId &&
    isProjectRootAuthoritySafe(getAttachmentRuntime().sessionCwd || resolvedCwd)
  )
    return getAttachmentRuntime().activeFrameId;
  if (getAttachmentRuntime().activeFramePromise) return await getAttachmentRuntime().activeFramePromise;

  if (!isProjectRootAuthoritySafe(resolvedCwd)) return null;
  getAttachmentRuntime().sessionCwd = resolvedCwd;

  getAttachmentRuntime().activeFramePromise = (async () => {
    const continuityId = ensureContinuityId(resolvedCwd);
    if (!(await ensureFocusaSessionForFrame(resolvedCwd, continuityId))) return null;
    focusaPost("/instance/connect", {
      instance_id: `pi-${process.pid}`,
      surface: "pi",
      session_id: sessionId || getAttachmentRuntime().sessionFrameKey || `pi-session-${Date.now()}`,
      cwd: resolvedCwd,
    });

    const frameId = await createPiFrame(resolvedCwd, source);
    if (frameId) persistState();
    return frameId;
  })();

  try {
    return await getAttachmentRuntime().activeFramePromise;
  } finally {
    getAttachmentRuntime().activeFramePromise = null;
  }
}

export async function rescopePiFrameFromCurrentAsk(
  cwd?: string,
  source = "pi-ask-rescope"
): Promise<string | null> {
  if (!getAttachmentRuntime().focusaAvailable || !getAttachmentRuntime().activeFrameId)
    return getAttachmentRuntime().activeFrameId;
  const resolvedCwd = cwd || getAttachmentRuntime().sessionCwd || process.cwd();
  const ask = trimFrameText(stripQuotedFocusaContext(getAttachmentRuntime().currentAsk?.text || ""), 100);
  const askKind = getAttachmentRuntime().currentAsk?.kind || "unknown";
  if (!ask || askKind === "meta" || isNonTaskStatusLikeText(ask)) return getAttachmentRuntime().activeFrameId;

  const activeGoal = trimFrameText(
    stripQuotedFocusaContext(getAttachmentRuntime().activeFrameGoal || ""),
    100
  ).toLowerCase();
  const askNorm = ask.toLowerCase();
  const sameMission =
    Boolean(activeGoal) &&
    (askNorm === activeGoal || askNorm.includes(activeGoal) || activeGoal.includes(askNorm));

  const genericFrame = isGenericPiFrameForCwd(
    resolvedCwd,
    getAttachmentRuntime().activeFrameTitle,
    getAttachmentRuntime().activeFrameGoal
  );
  const explicitContinuation = isExplicitContinuationAsk(ask);
  const shouldRescope = genericFrame || (!explicitContinuation && !sameMission && askNorm.length >= 6);
  if (!shouldRescope) return getAttachmentRuntime().activeFrameId;

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
    return getAttachmentRuntime().activeFrameId;
  }

  getAttachmentRuntime().activeFrameId = null;
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
    decisions:
      fs?.decisions ||
      getAttachmentRuntime().lastFocusSnapshot.decisions ||
      getAttachmentRuntime().localDecisions,
    constraints:
      fs?.constraints ||
      getAttachmentRuntime().lastFocusSnapshot.constraints ||
      getAttachmentRuntime().localConstraints,
    failures: sanitizeFocusFailures(
      fs?.failures ||
        getAttachmentRuntime().lastFocusSnapshot.failures ||
        getAttachmentRuntime().localFailures
    ),
    intent: fs?.intent || getAttachmentRuntime().lastFocusSnapshot.intent || "",
    currentFocus:
      fs?.current_focus ||
      fs?.current_state ||
      getAttachmentRuntime().lastFocusSnapshot.currentFocus ||
      getLastTrajectoryClarity()?.short_term_goal ||
      "",
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
  const normalized = String(input || "")
    .replace(/\s+/g, " ")
    .trim();
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

  for (const [kind, bucket] of Object.entries(getAttachmentRuntime().ecsRegistry || {}) as Array<
    [string, Record<string, { content: string; storedAt: number }>]
  >) {
    for (const [id, record] of Object.entries(bucket || {})) {
      const age = now - (record?.storedAt || 0);
      if (!record || typeof record.content !== "string" || age > ECS_TTL_MS) {
        delete bucket[id];
        continue;
      }
      const bytes = Buffer.byteLength(record.content, "utf8");
      flat.push({ kind, id, storedAt: record.storedAt || 0, bytes });
    }
    if (!Object.keys(bucket || {}).length) delete getAttachmentRuntime().ecsRegistry[kind];
  }

  flat.sort((a, b) => a.storedAt - b.storedAt);
  let totalBytes = flat.reduce((sum, item) => sum + item.bytes, 0);
  let totalItems = flat.length;

  while (flat.length && (totalItems > MAX_ECS_ITEMS || totalBytes > MAX_ECS_TOTAL_BYTES)) {
    const victim = flat.shift();
    if (!victim) break;
    if (getAttachmentRuntime().ecsRegistry[victim.kind]?.[victim.id]) {
      delete getAttachmentRuntime().ecsRegistry[victim.kind][victim.id];
      if (!Object.keys(getAttachmentRuntime().ecsRegistry[victim.kind]).length)
        delete getAttachmentRuntime().ecsRegistry[victim.kind];
      totalItems -= 1;
      totalBytes = Math.max(0, totalBytes - victim.bytes);
    }
  }
}

export async function persistAuthoritativeState(): Promise<void> {
  if (getAttachmentRuntime().focusaAvailable && getAttachmentRuntime().activeFrameId) {
    await getFocusState().catch(() => null);
  }
  persistState();
}

function boundedObject(value: any, maxBytes: number, fallback: Record<string, any>): any {
  if (value == null) return value;
  try {
    if (Buffer.byteLength(JSON.stringify(value), "utf8") <= maxBytes) return value;
  } catch {
    return fallback;
  }
  return fallback;
}

function compactWorkpointPacket(packet: any): any {
  const workpoint = packet?.resume_packet?.workpoint || packet?.workpoint || packet || {};
  return {
    workpoint_id: workpoint.workpoint_id || packet?.workpoint_id || null,
    revision: workpoint.revision || packet?.revision || null,
    checkpoint_id: workpoint.checkpoint_id || packet?.checkpoint_id || null,
    project_root: workpoint.project_root || packet?.project_root || null,
    continuity_id: workpoint.continuity_id || packet?.continuity_id || null,
    mission: trimPersistText(workpoint.mission || packet?.mission || ""),
    current_action: workpoint.current_action || packet?.current_action || null,
    next_slice: trimPersistText(workpoint.next_slice || packet?.next_slice || ""),
    blockers: Array.isArray(workpoint.blockers) ? workpoint.blockers.slice(0, 8) : [],
    evidence_refs: Array.isArray(workpoint.evidence_refs) ? workpoint.evidence_refs.slice(0, 12) : [],
    canonical: packet?.canonical !== false,
  };
}

function compactTrajectoryClarity(trajectory: any): any {
  if (!trajectory) return null;
  return {
    trajectory_id: trajectory.trajectory_id || trajectory.id || null,
    project_root: trajectory.project_root || null,
    continuity_id: trajectory.continuity_id || null,
    session_id: trajectory.session_id || null,
    status: trajectory.status || null,
    hlt_status: trajectory.hlt_status || null,
    long_term_goal: trimPersistText(trajectory.long_term_goal || trajectory.hlt || ""),
    mid_level_goal: trimPersistText(trajectory.mid_level_goal || trajectory.mlg || ""),
    short_term_goal: trimPersistText(trajectory.short_term_goal || trajectory.stg || ""),
    current_state: trimPersistText(trajectory.current_state || ""),
    recommended_action: trimPersistText(trajectory.recommended_action || ""),
    fallback_prior_project_trajectory: trajectory.fallback_prior_project_trajectory === true,
  };
}

function boundedVitalInfoPrompted(value: Record<string, number>): Record<string, number> {
  return Object.fromEntries(
    Object.entries(value || {})
      .sort((a, b) => Number(b[1] || 0) - Number(a[1] || 0))
      .slice(0, 40)
  );
}

function buildPersistedRecoveryState(): Record<string, any> {
  const workpoint = getScopedWorkpointPacket();
  return {
    sessionId: getAttachmentRuntime().sessionFrameKey,
    continuityId: getAttachmentRuntime().continuityId,
    projectRoot: normalizeProjectRoot(
      getLastProjectRootResolution()?.projectRoot || getAttachmentRuntime().sessionCwd || process.cwd()
    ),
    frameId: getAttachmentRuntime().activeFrameId,
    frameTitle: trimPersistText(getAttachmentRuntime().activeFrameTitle),
    frameGoal: trimPersistText(getAttachmentRuntime().activeFrameGoal),
    currentAsk: getAttachmentRuntime().currentAsk
      ? {
          ...getAttachmentRuntime().currentAsk,
          text: trimPersistText(getAttachmentRuntime().currentAsk.text),
        }
      : null,
    queryScope: getAttachmentRuntime().queryScope,
    decisions: tailBounded(getAttachmentRuntime().localDecisions),
    constraints: tailBounded(getAttachmentRuntime().localConstraints),
    failures: tailBounded(sanitizeFocusFailures(getAttachmentRuntime().localFailures), 20),
    authoritativeDecisions: tailBounded(getAttachmentRuntime().lastFocusSnapshot.decisions),
    authoritativeConstraints: tailBounded(getAttachmentRuntime().lastFocusSnapshot.constraints),
    authoritativeFailures: tailBounded(
      sanitizeFocusFailures(getAttachmentRuntime().lastFocusSnapshot.failures),
      20
    ),
    intent: trimPersistText(getAttachmentRuntime().lastFocusSnapshot.intent),
    currentFocus: trimPersistText(getAttachmentRuntime().lastFocusSnapshot.currentFocus),
    projectRootResolution: getLastProjectRootResolution(),
    activeWorkpointPacket: boundedObject(workpoint, 64 * 1024, compactWorkpointPacket(workpoint)),
    activeWorkpointSummary: workpoint ? trimPersistText(getActiveWorkpointSummary()) : "",
    lastTrajectoryClarity: boundedObject(
      getLastTrajectoryClarity(),
      48 * 1024,
      compactTrajectoryClarity(getLastTrajectoryClarity())
    ),
    lastProjectIdentity: boundedObject(getLastProjectIdentity(), 16 * 1024, {
      project_root: getLastProjectIdentity()?.project_root || null,
      project_id: getLastProjectIdentity()?.project_id || null,
      canonical_name: getLastProjectIdentity()?.canonical_name || null,
      status: getLastProjectIdentity()?.status || null,
    }),
    lastProjectVerify: boundedObject(getLastProjectVerify(), 16 * 1024, {
      project_root: getLastProjectVerify()?.project_root || null,
      verified: getLastProjectVerify()?.verified === true,
      status: getLastProjectVerify()?.status || null,
    }),
    latestReportSummary: getLatestReportSummary(),
    northStarSnapshot: getAttachmentRuntime().northStarSnapshot,
    toolOutputPressure: getAttachmentRuntime().toolOutputPressure?.recapRequired
      ? {
          recapRequired: true,
          recapReason: trimPersistText(getAttachmentRuntime().toolOutputPressure.recapReason),
          lastToolName: trimPersistText(getAttachmentRuntime().toolOutputPressure.lastToolName),
        }
      : null,
    projectSwitchLedger: getAttachmentRuntime().projectSwitchLedger.slice(
      0,
      PROJECT_SWITCH_LEDGER_MAX_OBSERVATIONS
    ),
    vitalInfoPrompted: boundedVitalInfoPrompted(getAttachmentRuntime().vitalInfoPrompted),
    pendingLifecycleAdvisories: Object.fromEntries(
      Object.entries(getAttachmentRuntime().pendingLifecycleAdvisories).slice(-8)
    ),
    sessionProjectClassification: getAttachmentRuntime().sessionProjectClassification,
    piSessionProjectRegistry: Object.fromEntries(
      Object.entries(getAttachmentRuntime().piSessionProjectRegistry).slice(-64)
    ),
    projectBindingDecisions: Object.fromEntries(
      Object.entries(getAttachmentRuntime().projectBindingDecisions).slice(-16)
    ),
    projectBindingTelemetry: getAttachmentRuntime().projectBindingTelemetry,
    lastCompactResumeKey: getAttachmentRuntime().lastCompactResumeKey,
    lastCompactResumeAt: getAttachmentRuntime().lastCompactResumeAt,
    compactResumeDeliveryKey: getAttachmentRuntime().compactResumeDeliveryKey,
    compactResumeDeliveryState: getAttachmentRuntime().compactResumeDeliveryState,
    turnCount: getTurnCount(),
    wbmEnabled: getAttachmentRuntime().wbmEnabled,
    wbmNoCatalogue: getAttachmentRuntime().wbmNoCatalogue,
    cataloguedDecisions: tailBounded(getAttachmentRuntime().cataloguedDecisions),
    cataloguedFacts: tailBounded(getAttachmentRuntime().cataloguedFacts),
    totalCompactions: getTotalCompactions(),
  };
}

function appendBoundedNativeEntry(
  customType: string,
  payload: Record<string, any>,
  hardCap: number
): boolean {
  const bytes = Buffer.byteLength(JSON.stringify(payload), "utf8");
  if (bytes > hardCap) {
    focusaPost("/telemetry/trace", {
      event_type: "pi_persistence_anchor_rejected_oversized",
      payload: {
        custom_type: customType,
        bytes,
        hard_cap: hardCap,
        session_id: getAttachmentRuntime().sessionFrameKey,
      },
    });
    return false;
  }
  if (!getAttachmentRuntime().pi) return false;
  try {
    getAttachmentRuntime().pi.appendEntry(customType, payload);
    return true;
  } catch (error) {
    focusaPost("/telemetry/trace", {
      event_type: "pi_persistence_native_append_failed",
      payload: {
        custom_type: customType,
        bytes,
        session_id: getAttachmentRuntime().sessionFrameKey,
        error: String((error as Error)?.message || error),
      },
    });
    return false;
  }
}

function projectSwitchSemanticPayload(): Record<string, any> {
  return {
    observations: stableSemanticValue(
      getAttachmentRuntime().projectSwitchLedger.slice(0, 6),
      "projectSwitchLedger"
    ),
  };
}

function persistProjectSwitchLedgerAnchor(): void {
  if (!getAttachmentRuntime().pi) return;
  const semantic = projectSwitchSemanticPayload();
  const digest = semanticPersistenceDigest(semantic);
  if (digest === getAttachmentRuntime().lastProjectSwitchPersistHash) return;
  const payload: Record<string, any> = {
    schema: "focusa.project_switch_anchor.v1",
    semanticDigest: digest,
    ...semantic,
    createdAt: new Date().toISOString(),
  };
  while (
    Array.isArray(payload.observations) &&
    payload.observations.length > 1 &&
    Buffer.byteLength(JSON.stringify(payload), "utf8") > PROJECT_SWITCH_ANCHOR_MAX_BYTES
  ) {
    payload.observations.pop();
  }
  if (Buffer.byteLength(JSON.stringify(payload), "utf8") > PROJECT_SWITCH_ANCHOR_MAX_BYTES) return;
  if (appendBoundedNativeEntry("focusa-project-switch-ledger", payload, PROJECT_SWITCH_ANCHOR_MAX_BYTES)) {
    getAttachmentRuntime().lastProjectSwitchPersistHash = digest;
  }
}

export function persistState(): void {
  if (!getAttachmentRuntime().sessionFrameKey) return;
  const recoveryState = buildPersistedRecoveryState();
  const semanticDigest = semanticPersistenceDigest(recoveryState);
  const semanticChanged = semanticDigest !== getAttachmentRuntime().lastPersistHash;

  if (semanticChanged) {
    const revision = getAttachmentRuntime().persistRevision + 1;
    let sidecar: { key: string; bytes: number };
    try {
      sidecar = writeRecoverySidecar(recoveryState, semanticDigest, revision);
    } catch (error) {
      focusaPost("/telemetry/trace", {
        event_type: "pi_persistence_sidecar_write_failed",
        payload: {
          session_id: getAttachmentRuntime().sessionFrameKey,
          error: String((error as Error)?.message || error),
        },
      });
      return;
    }

    getAttachmentRuntime().lastPersistHash = semanticDigest;
    getAttachmentRuntime().persistRevision = revision;
    getAttachmentRuntime().lastPersistSidecarKey = sidecar.key;
    getAttachmentRuntime().lastPersistSidecarBytes = sidecar.bytes;
    getAttachmentRuntime().pendingPersistAnchor = true;
  }

  if (!getAttachmentRuntime().pendingPersistAnchor || !getAttachmentRuntime().lastPersistSidecarKey) return;
  const now = Date.now();
  if (
    getAttachmentRuntime().lastPersistAt > 0 &&
    now - getAttachmentRuntime().lastPersistAt < PERSIST_MIN_INTERVAL_MS
  )
    return;

  const workpoint = compactWorkpointPacket(recoveryState.activeWorkpointPacket);
  const trajectory = compactTrajectoryClarity(recoveryState.lastTrajectoryClarity);
  const anchor = {
    schema: COMPACTION_PERSISTENCE_ANCHOR_SCHEMA,
    anchorRevision: getAttachmentRuntime().persistRevision,
    semanticDigest: getAttachmentRuntime().lastPersistHash,
    sessionId: recoveryState.sessionId,
    continuityId: recoveryState.continuityId,
    projectRoot: recoveryState.projectRoot,
    frameId: recoveryState.frameId,
    currentAsk: recoveryState.currentAsk
      ? {
          text: trimPersistText(recoveryState.currentAsk.text || ""),
          kind: recoveryState.currentAsk.kind || "unknown",
          sourceTurnId: recoveryState.currentAsk.sourceTurnId || "",
        }
      : null,
    workpointId: workpoint?.workpoint_id || null,
    workpointRevision: workpoint?.revision || null,
    checkpointId: workpoint?.checkpoint_id || null,
    trajectoryId: trajectory?.trajectory_id || null,
    hltStatus: trajectory?.hlt_status || null,
    sidecarKey: getAttachmentRuntime().lastPersistSidecarKey,
    sidecarBytes: getAttachmentRuntime().lastPersistSidecarBytes,
    createdAt: new Date(now).toISOString(),
  };
  if (!appendBoundedNativeEntry("focusa-state", anchor, NATIVE_ANCHOR_MAX_BYTES)) return;

  if (getAttachmentRuntime().wbmEnabled) {
    appendBoundedNativeEntry(
      "focusa-wbm-state",
      {
        schema: COMPACTION_PERSISTENCE_ANCHOR_REF_SCHEMA,
        anchorRevision: getAttachmentRuntime().persistRevision,
        semanticDigest: getAttachmentRuntime().lastPersistHash,
        sessionId: recoveryState.sessionId,
        continuityId: recoveryState.continuityId,
        projectRoot: recoveryState.projectRoot,
        sidecarKey: getAttachmentRuntime().lastPersistSidecarKey,
        createdAt: new Date(now).toISOString(),
      },
      NATIVE_ANCHOR_MAX_BYTES
    );
  }
  getAttachmentRuntime().pendingPersistAnchor = false;
  getAttachmentRuntime().lastPersistAt = now;
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
  if (!getAttachmentRuntime().ecsRegistry[kind]) getAttachmentRuntime().ecsRegistry[kind] = {};
  const raw = String(content || "");
  const clipped =
    Buffer.byteLength(raw, "utf8") > MAX_ECS_ITEM_BYTES
      ? `${raw.slice(0, MAX_ECS_ITEM_BYTES)}\n...[local ECS clipped due to memory cap]`
      : raw;
  getAttachmentRuntime().ecsRegistry[kind][id] = { content: clipped, storedAt: Date.now() };
  pruneEcsRegistry();
  return id;
}

export function getEcsArtifact(kind: string, id: string): string | null {
  pruneEcsRegistry();
  return getAttachmentRuntime().ecsRegistry[kind]?.[id]?.content ?? null;
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
// Migration path from `the mutable runtime object to typed scope-keyed stores.
// Each scope (project, host, workstream) gets its own TypedScopeStore instance.
// Consumers eventually read from getScopeStore() through typed runtime accessors.
//
// The registry (ScopeStoreRegistry) is an infra-only singleton — it is NOT
// an authority-bearing global. It simply manages lifecycle of stores.

/**
 * Canonical scope identity that keys a TypedScopeStore.
 */
export interface TypedScopeIdentity {
  scopeKind: "project" | "host" | "workstream" | "unknown";
  scopeId: string;
  fingerprint: string;
  rootPath: string;
  continuityId: string;
  workingSubpathId?: string;
  sessionId?: string;
}

/**
 * A typed, scoped runtime store containing all state relevant to one scope.
 * This replaces ad-hoc mutable fields with explicit typed accessors.
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

  /** Rebuildable scoped ontology inner-world projection (Spec95/100/125/151). */
  lastOntologyContext: null | Record<string, any> = null;

  /** Report shadow: latest report summary (PI-06) */
  latestReportSummary: null | { handle?: string; mission?: string; nextAction?: string } = null;

  /** Report shadow: last project verify result (PI-05) */
  lastProjectVerify: null | Record<string, any> = null;

  /** Turn state: task start turn (PI-07) */
  currentTaskTurnStart: number = 0;

  /** Turn state: tool usage batch (PI-07) */
  toolUsageBatch: Array<string> = [];

  /** Turn state: last stream len (PI-07) */
  lastStreamLen: number = 0;

  /** Turn state: compilation errors (PI-07) */
  compilationErrors: Array<number> = [];

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
    return [
      id.scopeKind,
      id.scopeId,
      id.fingerprint,
      id.rootPath,
      id.continuityId,
      id.workingSubpathId || "primary",
    ].join("::");
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
  const requestedRoot = normalizeProjectRoot(getAttachmentRuntime().sessionCwd || "");
  const continuityId = String(getAttachmentRuntime().continuityId || "").trim();
  if (!requestedRoot || !continuityId) return null;
  const verifiedScope = verifiedScopeRefForRoot(requestedRoot);
  if (!verifiedScope) return null;
  const identity: TypedScopeIdentity = {
    scopeKind: verifiedScope.scope_kind,
    scopeId: verifiedScope.scope_id,
    fingerprint: verifiedScope.fingerprint,
    rootPath: normalizeProjectRoot(verifiedScope.root_path),
    continuityId,
    workingSubpathId: process.env.FOCUSA_WORKING_SUBPATH_ID || "primary",
    sessionId: getAttachmentRuntime().sessionFrameKey || undefined,
  };
  const store = scopeStoreRegistry.getOrCreate(identity);
  if (!store.sessionCwd) store.sessionCwd = getAttachmentRuntime().sessionCwd;
  if (!store.continuityId) store.continuityId = getAttachmentRuntime().continuityId;
  if (!store.sessionFrameKey) store.sessionFrameKey = getAttachmentRuntime().sessionFrameKey;
  return store;
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
 * Sync current runtime fields into the active TypedScopeStore.
 * Call this after getAttachmentRuntime().sessionCwd/getAttachmentRuntime().continuityId/getAttachmentRuntime().sessionFrameKey are set
 * (e.g., at session start/resume) so scope-keyed consumers are consistent.
 */
export function syncRuntimeFieldsToScopeStore(): void {
  const store = getCurrentScopeStore();
  if (!store) return;
  store.sessionCwd = getAttachmentRuntime().sessionCwd;
  store.continuityId = getAttachmentRuntime().continuityId;
  store.sessionFrameKey = getAttachmentRuntime().sessionFrameKey;
  // turnCount migrated to scope store (PI-07, no longer synced from runtime)
  store.activeFrameId = getAttachmentRuntime().activeFrameId;
  store.focusaAvailable = getAttachmentRuntime().focusaAvailable;
}

/**
 * Sync current TypedScopeStore fields back into getAttachmentRuntime().
 * Call this after scope-keyed operations update the store.
 */
export function syncScopeStoreFieldsToRuntime(): void {
  const store = getCurrentScopeStore();
  if (!store) return;
  getAttachmentRuntime().sessionCwd = store.sessionCwd || getAttachmentRuntime().sessionCwd;
  getAttachmentRuntime().continuityId = store.continuityId || getAttachmentRuntime().continuityId;
  getAttachmentRuntime().sessionFrameKey = store.sessionFrameKey || getAttachmentRuntime().sessionFrameKey;
  // turnCount synced via scope store only (PI-07, removed from singleton)
  if (store.activeFrameId !== null) getAttachmentRuntime().activeFrameId = store.activeFrameId;
  if (store.focusaAvailable !== getAttachmentRuntime().focusaAvailable)
    getAttachmentRuntime().focusaAvailable = store.focusaAvailable;
}

/**
 * PI-07: Get turn count from scope store only.
 */
export function getTurnCount(): number {
  const store = getCurrentScopeStore();
  return store ? store.turnCount : 0;
}

/**
 * PI-07: Set turn count on scope store only.
 */
export function setTurnCount(v: number): void {
  const store = getCurrentScopeStore();
  if (store) store.turnCount = v;
}

/**
 * PI-07: Increment turn count on scope store only.
 */
export function incrementTurnCount(): void {
  const store = getCurrentScopeStore();
  if (store) store.turnCount++;
}

/**
 * Convenience: get active frame id from scope store (preferred) or runtime value.
 */
export function getActiveFrameId(): string | null {
  const store = getCurrentScopeStore();
  return store?.activeFrameId ?? getAttachmentRuntime().activeFrameId;
}

/**
 * Convenience: get continuity id from scope store (preferred) or runtime value.
 */
export function getContinuityId(): string {
  const store = getCurrentScopeStore();
  const sessionId = store?.sessionFrameKey || getAttachmentRuntime().sessionFrameKey || "";
  const markerRoot = resolveCanonicalMarkerProjectRoot(process.cwd());
  const verified = markerRoot
    ? verifiedContinuityBySessionRoot.get(verifiedContinuityKey(sessionId, markerRoot))
    : "";
  return verified || store?.continuityId || getAttachmentRuntime().continuityId || "";
}

/**
 * Convenience: get session frame key from scope store (preferred) or runtime value.
 */
export function getSessionFrameKey(): string {
  const store = getCurrentScopeStore();
  return store?.sessionFrameKey || getAttachmentRuntime().sessionFrameKey || "";
}

/**
 * Convenience: get session cwd from scope store (preferred) or runtime value.
 */
export function getSessionCwd(): string {
  const store = getCurrentScopeStore();
  return store?.sessionCwd || getAttachmentRuntime().sessionCwd || "";
}

/**
 * Convenience: get focusa available flag from scope store (preferred) or runtime value.
 */
export function getFocusaAvailable(): boolean {
  const store = getCurrentScopeStore();
  return store?.focusaAvailable ?? getAttachmentRuntime().focusaAvailable;
}

/**
 * PI-02: Get the last project root resolution from the typed scope store only.
 */
export function getLastProjectRootResolution(): TypedScopeStore["lastProjectRootResolution"] {
  const store = getCurrentScopeStore();
  return store?.lastProjectRootResolution ?? null;
}

/**
 * PI-02: Set lastProjectRootResolution on the typed scope store only.
 */
export function setLastProjectRootResolution(resolution: TypedScopeStore["lastProjectRootResolution"]): void {
  if (!resolution?.projectRoot) return;
  const store = getCurrentScopeStore();
  if (!store) return;
  if (normalizeProjectRoot(resolution.projectRoot) !== store.identity.rootPath) return;
  store.lastProjectRootResolution = resolution;
}

/**
 * PI-02: Get lastProjectIdentity from the typed scope store only.
 */
export function getLastProjectIdentity(): Record<string, any> | null {
  const store = getCurrentScopeStore();
  return store ? store.lastProjectIdentity : null;
}

/**
 * PI-02: Set lastProjectIdentity on the typed scope store only.
 */
export function setLastProjectIdentity(identity: Record<string, any> | null): void {
  const nested = identity?.project_identity || identity;
  if (nested?.status === "verified" && nested?.scope_ref) {
    registerVerifiedScopeRef(nested.scope_ref);
  }
  const store = getCurrentScopeStore();
  if (store) store.lastProjectIdentity = identity;
}

/**
 * PI-03: Get active workpoint packet from the typed scope store only.
 */
export function getActiveWorkpointPacket(): Record<string, any> | null {
  const store = getCurrentScopeStore();
  return store ? store.activeWorkpointPacket : null;
}

/**
 * PI-03: Set activeWorkpointPacket on the typed scope store only.
 */
export function setActiveWorkpointPacket(packet: Record<string, any> | null): void {
  const store = getCurrentScopeStore();
  if (store) store.activeWorkpointPacket = packet;
}

/**
 * PI-03: Get active workpoint summary from the typed scope store only.
 */
export function getActiveWorkpointSummary(): string {
  const store = getCurrentScopeStore();
  return store ? store.activeWorkpointSummary || "" : "";
}

/**
 * PI-03: Set activeWorkpointSummary on the typed scope store only.
 */
export function setActiveWorkpointSummary(summary: string): void {
  const store = getCurrentScopeStore();
  if (store) store.activeWorkpointSummary = summary;
}

/** PI-04: Get lastTrajectoryClarity from the typed scope store only. */
export function getLastTrajectoryClarity(): Record<string, any> | null {
  const store = getCurrentScopeStore();
  return store ? store.lastTrajectoryClarity : null;
}

function trajectorySnapshotMatchesStore(snapshot: Record<string, any>, store: TypedScopeStore): boolean {
  const projectIdentity = snapshot?.project_identity || {};
  const projectIdentityApi = projectIdentity?.project_identity_api || {};
  const scopeRef = projectIdentity?.scope_ref || projectIdentityApi?.scope_ref || {};
  const projectRoot = normalizeProjectRoot(snapshot?.project_root || "");
  const continuityId = String(snapshot?.continuity_id || "").trim();
  const trajectoryId = String(snapshot?.trajectory_id || "").trim();
  const receipt = snapshot?.scope_verification || {};
  const receiptScope = receipt?.scope_ref || {};
  if (
    !trajectoryId ||
    projectIdentity?.status !== "verified" ||
    projectRoot !== store.identity.rootPath ||
    continuityId !== store.identity.continuityId ||
    scopeRef?.scope_kind !== store.identity.scopeKind ||
    String(scopeRef?.scope_id || "") !== store.identity.scopeId ||
    String(scopeRef?.fingerprint || "") !== store.identity.fingerprint ||
    normalizeProjectRoot(scopeRef?.root_path || "") !== store.identity.rootPath ||
    String(receipt?.rendered_trajectory_id || "").trim() !== trajectoryId ||
    !String(receipt?.source_trajectory_id || "").trim() ||
    normalizeProjectRoot(receipt?.project_root || "") !== store.identity.rootPath ||
    receiptScope?.scope_kind !== store.identity.scopeKind ||
    String(receiptScope?.scope_id || "") !== store.identity.scopeId ||
    String(receiptScope?.fingerprint || "") !== store.identity.fingerprint ||
    normalizeProjectRoot(receiptScope?.root_path || "") !== store.identity.rootPath
  ) {
    return false;
  }
  if (snapshot?.fallback_prior_project_trajectory === true) {
    const sourceContinuity = String(snapshot?.fallback_source_continuity_id || "").trim();
    return Boolean(
      receipt?.status === "verified_same_project_fallback" &&
        sourceContinuity &&
        sourceContinuity !== store.identity.continuityId &&
        String(receipt?.continuity_id || "").trim() === sourceContinuity
    );
  }
  return (
    receipt?.status === "verified_exact" &&
    String(receipt?.continuity_id || "").trim() === store.identity.continuityId
  );
}

/** PI-04: Set lastTrajectoryClarity only after stable ScopeRef + continuity verification. */
export function setLastTrajectoryClarity(snapshot: Record<string, any> | null): void {
  const store = getCurrentScopeStore();
  if (!store) return;
  store.lastTrajectoryClarity = snapshot && trajectorySnapshotMatchesStore(snapshot, store) ? snapshot : null;
}

function ontologyContextMatchesStore(packet: Record<string, any>, store: TypedScopeStore): boolean {
  const scope = packet?.scope || {};
  const rootScope = scope?.root_scope || {};
  const verification = packet?.scope_verification || {};
  const verifiedScope = verification?.scope_ref || {};
  return Boolean(
    packet?.status === "ok" &&
      packet?.stale !== true &&
      rootScope?.scope_kind === store.identity.scopeKind &&
      String(rootScope?.scope_id || "") === store.identity.scopeId &&
      String(rootScope?.fingerprint || "") === store.identity.fingerprint &&
      normalizeProjectRoot(rootScope?.root_path || "") === store.identity.rootPath &&
      String(scope?.continuity_id || "").trim() === store.identity.continuityId &&
      verification?.status === "verified_exact" &&
      verifiedScope?.scope_kind === store.identity.scopeKind &&
      String(verifiedScope?.scope_id || "") === store.identity.scopeId &&
      String(verifiedScope?.fingerprint || "") === store.identity.fingerprint &&
      normalizeProjectRoot(verifiedScope?.root_path || "") === store.identity.rootPath &&
      normalizeProjectRoot(verification?.project_root || "") === store.identity.rootPath &&
      String(verification?.continuity_id || "").trim() === store.identity.continuityId
  );
}

export function getCachedOntologyContext(): Record<string, any> | null {
  const store = getCurrentScopeStore();
  return store ? store.lastOntologyContext : null;
}

export function setCachedOntologyContext(packet: Record<string, any> | null): void {
  const store = getCurrentScopeStore();
  if (!store) return;
  store.lastOntologyContext = packet && ontologyContextMatchesStore(packet, store) ? packet : null;
}

export async function refreshOntologyContextLifecycle(
  reason: string,
  currentAsk?: string,
  operatorSteeringDetected = false
): Promise<Record<string, any> | null> {
  const store = getCurrentScopeStore();
  if (!store || !getAttachmentRuntime().focusaAvailable) return null;
  const workpoint = getActiveWorkpointPacket();
  const targetRefs = Array.isArray(workpoint?.active_object_refs)
    ? workpoint.active_object_refs.slice(0, 8)
    : [];
  try {
    const packet = await focusaFetch("/ontology/context", {
      method: "POST",
      body: JSON.stringify({
        current_ask: String(currentAsk || getAttachmentRuntime().currentAsk?.text || "").trim() || null,
        frame_id: getAttachmentRuntime().activeFrameId || null,
        workpoint_id: workpoint?.workpoint_id || null,
        target_refs: targetRefs,
        active_object_refs: targetRefs,
        budget_tokens: 600,
        view_profile: "pi_operator_view",
        slice_type: "active_context",
        operator_steering_detected: operatorSteeringDetected,
      }),
    });
    setCachedOntologyContext(packet);
    const cached = getCachedOntologyContext();
    if (cached) {
      focusaPost("/telemetry/activity", {
        surface: "pi",
        event: "ontology_context_refreshed",
        reason,
        project_root: store.identity.rootPath,
        continuity_id: store.identity.continuityId,
        cross_plane_agreement: cached?.cross_plane_agreement?.status || "unknown",
      });
    }
    return cached;
  } catch {
    setCachedOntologyContext(null);
    return null;
  }
}

/** PI-05: Get lastProjectVerify from the typed scope store only. */
export function getLastProjectVerify(): Record<string, any> | null {
  const store = getCurrentScopeStore();
  return store ? store.lastProjectVerify : null;
}

/** PI-05: Set lastProjectVerify on the typed scope store only. */
export function setLastProjectVerify(result: Record<string, any> | null): void {
  if (result?.canonical === true || result?.verification?.verified === true) {
    registerVerifiedScopeRef(result?.project_identity?.scope_ref);
  }
  const store = getCurrentScopeStore();
  if (store) store.lastProjectVerify = result;
}

/** Session-scoped startup binding receipt; project authority remains root+continuity scoped. */
export function currentProjectBindingDecision(
  sessionId = getAttachmentRuntime().sessionFrameKey
): ProjectBindingDecisionV1 | null {
  return sessionId ? getAttachmentRuntime().projectBindingDecisions[sessionId] || null : null;
}

export function setCurrentProjectBindingDecision(
  decision: ProjectBindingDecisionV1,
  sessionId = getAttachmentRuntime().sessionFrameKey
): void {
  if (!sessionId) return;
  getAttachmentRuntime().projectBindingDecisions[sessionId] = decision;
  const bounded = Object.entries(getAttachmentRuntime().projectBindingDecisions).slice(-16);
  getAttachmentRuntime().projectBindingDecisions = Object.fromEntries(bounded);
}

/** PI-06: Get latestReportSummary from scope store only. */
export function getLatestReportSummary(): Record<string, any> | null {
  const store = getCurrentScopeStore();
  return store ? store.latestReportSummary : null;
}

/** PI-06: Set latestReportSummary on scope store only. */
export function setLatestReportSummary(summary: Record<string, any> | null): void {
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

/** PI-07: Increment totalCompactions counter on scope store only. */
export function incrementTotalCompactions(): void {
  const store = getCurrentScopeStore();
  if (store) store.totalCompactions++;
}

/** PI-07: Get current task turn start from scope store only. */
export function getCurrentTaskTurnStart(): number {
  const store = getCurrentScopeStore();
  return store ? store.currentTaskTurnStart : 0;
}

/** PI-07: Set current task turn start on scope store only. */
export function setCurrentTaskTurnStart(v: number): void {
  const store = getCurrentScopeStore();
  if (store) store.currentTaskTurnStart = v;
}

/** PI-07: Get last stream length from scope store only. */
export function getLastStreamLen(): number {
  const store = getCurrentScopeStore();
  return store ? store.lastStreamLen : 0;
}

/** PI-07: Set last stream length on scope store only. */
export function setLastStreamLen(v: number): void {
  const store = getCurrentScopeStore();
  if (store) store.lastStreamLen = v;
}
/** PI-07: Get tool usage batch from scope store only. */
export function getToolUsageBatch(): Array<string> {
  const store = getCurrentScopeStore();
  return store ? store.toolUsageBatch : [];
}

/** PI-07: Push a tool name to tool usage batch on scope store only. */
export function pushToToolUsageBatch(name: string): void {
  const store = getCurrentScopeStore();
  if (store) store.toolUsageBatch.push(name);
}

/** PI-07: Reset tool usage batch on scope store only. */
export function resetToolUsageBatch(): void {
  const store = getCurrentScopeStore();
  if (store) store.toolUsageBatch = [];
}

/** PI-07: Get total compactions from scope store only. */
export function getTotalCompactions(): number {
  const store = getCurrentScopeStore();
  return store ? store.totalCompactions : 0;
}

/** PI-07: Get long session signaled flag from scope store only. */
export function getLongSessionSignaled(): boolean {
  const store = getCurrentScopeStore();
  return store ? store.longSessionSignaled : false;
}

/** PI-07: Set long session signaled flag on scope store only. */
export function setLongSessionSignaled(v: boolean): void {
  const store = getCurrentScopeStore();
  if (store) store.longSessionSignaled = v;
}

/** PI-07: Get compilation errors array from scope store only. */
export function getCompilationErrors(): Array<number> {
  const store = getCurrentScopeStore();
  return store ? store.compilationErrors : [];
}

/** PI-07: Push a compilation error on scope store only. */
export function pushCompilationError(err: number): void {
  const store = getCurrentScopeStore();
  if (store) store.compilationErrors.push(err);
}

/** PI-07: Get file edit counts record from scope store only. */
export function getFileEditCounts(): Record<string, number> {
  const store = getCurrentScopeStore();
  return store ? store.fileEditCounts : {};
}

/** PI-07: Increment file edit count for a path on scope store only. */
export function incrementFileEditCount(fpath: string): void {
  const store = getCurrentScopeStore();
  if (store) {
    const orig = store.fileEditCounts[fpath] ?? 0;
    store.fileEditCounts[fpath] = orig + 1;
  }
}
/** PI-07: Set total compactions on scope store only. */
export function setTotalCompactions(v: number): void {
  const store = getCurrentScopeStore();
  if (store) store.totalCompactions = v;
}

/** PI-07: Set compilation errors array on scope store only. */
export function setCompilationErrors(arr: Array<number>): void {
  const store = getCurrentScopeStore();
  if (store) store.compilationErrors = arr;
}

/** PI-07: Reset file edit counts on both scope store and getAttachmentRuntime(). */
export function resetFileEditCounts(): void {
  const store = getCurrentScopeStore();
  if (store) store.fileEditCounts = {};
}

/** PI-07: Set file edit counts record on scope store only. */
export function setFileEditCounts(rec: Record<string, number>): void {
  const store = getCurrentScopeStore();
  if (store) store.fileEditCounts = rec;
}

/** PI-08: Get in-tool-context flag from scope store (no global fallback — new field). */
export function getInToolContext(): boolean {
  const store = getCurrentScopeStore();
  return store ? store.inToolContext : false;
}

/** PI-08: Set in-tool-context flag on scope store only (no global fallback — new field). */
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

/** Focus Slice affordances are descriptor projections only; daemon responses remain authoritative. */
export function spec138FocusSliceAffordances(canMutate: boolean) {
  return SPEC138_OPERATIONS.map((operation) => ({
    operation_id: operation.operation_id,
    method: operation.method,
    path: operation.path,
    label: operation.label,
    available: operation.mode === "read" || canMutate,
    disabled_reason: operation.mode === "canonical_mutation" && !canMutate
      ? "canonical_daemon_authority_required"
      : undefined,
    client_authority: false as const,
  }));
}
