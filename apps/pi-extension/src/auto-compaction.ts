import { createHash, randomUUID } from "node:crypto";

import {
  DEFAULT_COMPACTION_SETTINGS,
  estimateTokens,
  findCutPoint,
  type ContextUsage,
  type ExtensionAPI,
  type ExtensionContext,
} from "@earendil-works/pi-coding-agent";

import type { FocusaConfig } from "./config.js";
import {
  providerCompactionCapabilities,
  type ProviderCompactionCapabilities,
} from "./provider-compaction-capabilities.js";
import { contextPressureTelemetry } from "./context-pressure-telemetry.js";
import {
  applyCompactionPolicyQuarantine,
  type CompactionPolicySelection,
} from "./compaction-policy-selector.js";
import {
  observeFrozenCompactionOutcome,
  prewarmCompactionPolicy,
  selectFrozenCompactionPolicy,
} from "./compaction-policy-adapter.js";
import {
  evaluateCompactionOutcome,
  type CompactionContinuationSnapshot,
  type CompactionOutcomeBaseline,
} from "./compaction-outcome-evaluator.js";
import { currentAttachmentKey, focusaFetch, focusaPost, runWithAttachmentRuntime } from "./state.js";
import {
  emptyCompactionAuthorityProjection,
  reduceCompactionAuthorityEvents,
  type CompactionAuthorityProjection,
} from "./compaction-authority-projection.js";

declare module "@earendil-works/pi-coding-agent" {
  interface ExtensionAPI {
    on(
      event: "agent_settled",
      handler: (event: { type: "agent_settled" }, ctx: ExtensionContext) => Promise<void> | void
    ): void;
  }
}

export const PROACTIVE_COMPACTION_MIN_RESERVE_TOKENS = 16_384;
export const PROACTIVE_COMPACTION_RESERVE_FRACTION = 0.1;
// Provider-advertised windows can be hundreds of thousands of tokens. Waiting
// until the final reserve window caused real sessions to reach 371K before a
// manual /compact. Focusa's balanced policy compacts at 70% instead.
export const PROACTIVE_COMPACTION_TRIGGER_FRACTION = 0.7;
export const PROACTIVE_COMPACTION_ABSOLUTE_TOKEN_CAP = 256_000;
export const PROACTIVE_COMPACTION_COOLDOWN_MS = 60_000;
export const PROACTIVE_COMPACTION_SUCCESS_COOLDOWN_MS = 180_000;

export interface ProactiveCompactionPolicy {
  enabled: boolean;
  triggerPercent: number;
  tokenCap: number;
  reserveTokens: number;
  reservePercent: number;
  cooldownMs: number;
}

export const DEFAULT_PROACTIVE_COMPACTION_POLICY: ProactiveCompactionPolicy = {
  enabled: true,
  triggerPercent: PROACTIVE_COMPACTION_TRIGGER_FRACTION * 100,
  tokenCap: PROACTIVE_COMPACTION_ABSOLUTE_TOKEN_CAP,
  reserveTokens: PROACTIVE_COMPACTION_MIN_RESERVE_TOKENS,
  reservePercent: PROACTIVE_COMPACTION_RESERVE_FRACTION * 100,
  cooldownMs: PROACTIVE_COMPACTION_COOLDOWN_MS,
};

export interface ProactiveCompactionDecision {
  trigger: boolean;
  tokens: number | null;
  contextWindow: number;
  reserveTokens: number;
  triggerAtTokens: number;
  percent: number | null;
  reason: "disabled" | "unknown_usage" | "below_threshold" | "context_pressure";
}

export type CompactionTriggerClass =
  "predicted_pressure" | "hard_pressure" | "provider_overflow" | "manual" | "rollover";

export interface CoordinatedCompactionRequest {
  triggerClass: CompactionTriggerClass;
  customInstructions: string;
  onComplete?: () => void;
  onError?: (error: Error) => void;
}

export type CoordinatedCompactionRequestResult =
  "requested" | "deferred_to_native" | "suppressed" | "ineligible" | "coordinator_unavailable";

export interface ProactiveCompactionEligibility {
  eligible: boolean;
  terminal: boolean;
  reason:
    | "eligible"
    | "empty_session"
    | "already_compacted"
    | "session_needs_migration"
    | "insufficient_history"
    | "insufficient_reclaim"
    | "negative_roi";
  compactableTokens: number;
  estimatedOverheadTokens: number;
  estimatedNetSavingsTokens: number;
}

type BranchEntry = ReturnType<ExtensionContext["sessionManager"]["getBranch"]>[number];

type CompactionCoordinatorState =
  | "idle"
  | "observing"
  | "prepare_requested"
  | "preparing"
  | "prepared"
  | "native_compaction_requested"
  | "native_compaction_active"
  | "native_compaction_failed"
  | "native_compaction_complete"
  | "verifying"
  | "verified"
  | "resume_pending"
  | "resume_delivered"
  | "deferred_to_next_turn"
  | "blocked"
  | "cooldown";

type ActiveEpoch = {
  epochId: string;
  retryOfEpochId?: string;
  attempt: number;
  triggerClass: CompactionTriggerClass;
  contextKey: string;
  contextWindow: number;
  startedAt: number;
  nativeCompactionCallCount: number;
  state: CompactionCoordinatorState;
  settlement?: "complete" | "failed";
  primaryError?: string;
  exactEligibility?: ProactiveCompactionEligibility;
  providerCapabilities: ProviderCompactionCapabilities;
  policySelection?: CompactionPolicySelection;
  outcomeBaseline?: CompactionOutcomeBaseline;
};

type CompactionLeaseOwner = {
  registrationId: string;
  adapterInstanceId: string;
  extensionBuild: string;
  registrationSource: string;
  attachmentId: string;
  nativeSession?: string;
  registeredHandlers: string[];
  moduleLoadId: string;
};

type CompactionOperatorOverride = {
  receipt_id: string;
  route: CompactionPolicySelection["route"];
  reason: string;
  actor_ref: string;
  created_at: string;
};

type ProcessCompactionLease = {
  schema: "focusa.compaction.coordinator.v1";
  generation: number;
  owner?: CompactionLeaseOwner;
  activeEpoch?: ActiveEpoch;
  duplicateDiagnosticEmitted: boolean;
  inFlightEpochId?: string;
  attemptOwnerId?: string;
  retryOwnerId?: string;
  projection: CompactionAuthorityProjection;
  operatorOverride?: CompactionOperatorOverride;
  lastSuccessfulCompactionAt?: number;
  request?: (
    ctx: ExtensionContext,
    request: CoordinatedCompactionRequest
  ) => CoordinatedCompactionRequestResult;
};

const PROCESS_LEASE_SYMBOL = Symbol.for("focusa.compaction.coordinator.v1");
const PI_TOOL_BOUNDARY_COMPACTION_SYMBOL = Symbol.for("focusa.pi.tool-boundary-compaction.v1");
const EXTENSION_BUILD = "focusa-pi-bridge@0.9.144";
const REGISTRATION_SOURCE = import.meta.url;
const REGISTERED_HANDLERS = [
  "session_before_compact",
  "agent_end",
  "agent_settled",
  "session_compact",
  "session_start",
  "session_shutdown",
];
const EVENT_TYPE = "focusa_auto_compaction_event";
const INSTRUCTIONS =
  "Preserve the current user ask, project_root + continuity_id authority, Workpoint and Trajectory authority, verified evidence handles, blockers, exact next action, and do-not-drift boundaries. Keep stable instructions verbatim where practical; summarize only older conversation detail.";

function selectionOwner(
  route: CompactionPolicySelection["route"]
): CompactionPolicySelection["executionOwner"] {
  if (route === "no_op") return "none";
  if (route === "rollover") return "operator";
  if (route === "native_compact" || route === "summarize") return "pi";
  return "focusa";
}

function applyOperatorOverride(
  selection: CompactionPolicySelection,
  override: CompactionOperatorOverride | undefined
): CompactionPolicySelection {
  if (!override) return selection;
  return {
    ...selection,
    route: override.route,
    executionOwner: selectionOwner(override.route),
    reason: "operator_override",
    deterministicKey: `override:${override.receipt_id}:${override.route}`,
  };
}

function policyOverride(value: any): CompactionOperatorOverride | undefined {
  const candidate = value?.policy?.operator_override ?? value?.operator_override;
  return candidate &&
    typeof candidate.receipt_id === "string" &&
    ["no_op", "curate_context", "checkpoint", "summarize", "native_compact", "rollover"].includes(
      candidate.route
    )
    ? candidate
    : undefined;
}

function piSupportsToolBoundaryCompaction(): boolean {
  return Boolean((globalThis as any)[PI_TOOL_BOUNDARY_COMPACTION_SYMBOL]);
}

function processCompactionLease(): ProcessCompactionLease {
  const scope = globalThis as typeof globalThis & {
    [PROCESS_LEASE_SYMBOL]?: ProcessCompactionLease;
  };
  scope[PROCESS_LEASE_SYMBOL] ??= {
    schema: "focusa.compaction.coordinator.v1",
    generation: 0,
    duplicateDiagnosticEmitted: false,
    projection: emptyCompactionAuthorityProjection(),
  };
  scope[PROCESS_LEASE_SYMBOL].projection ??= emptyCompactionAuthorityProjection();
  return scope[PROCESS_LEASE_SYMBOL];
}

export function requestCoordinatedCompaction(
  ctx: ExtensionContext,
  request: CoordinatedCompactionRequest
): CoordinatedCompactionRequestResult {
  return processCompactionLease().request?.(ctx, request) ?? "coordinator_unavailable";
}

export function proactiveCompactionPolicy(
  config?: Pick<
    FocusaConfig,
    | "autoCompactionEnabled"
    | "compactPct"
    | "autoCompactionTokenCap"
    | "autoCompactionReserveTokens"
    | "autoCompactionReservePct"
    | "autoCompactionCooldownMs"
  >
): ProactiveCompactionPolicy {
  if (!config) return { ...DEFAULT_PROACTIVE_COMPACTION_POLICY };
  return {
    enabled: config.autoCompactionEnabled,
    triggerPercent: config.compactPct,
    tokenCap: config.autoCompactionTokenCap,
    reserveTokens: config.autoCompactionReserveTokens,
    reservePercent: config.autoCompactionReservePct,
    cooldownMs: config.autoCompactionCooldownMs,
  };
}

export function proactiveCompactionDecision(
  usage: ContextUsage | undefined,
  policy: ProactiveCompactionPolicy = DEFAULT_PROACTIVE_COMPACTION_POLICY
): ProactiveCompactionDecision {
  const tokens = usage?.tokens ?? null;
  const contextWindow = Math.max(0, usage?.contextWindow ?? 0);
  const reserveTokens =
    contextWindow > 0
      ? Math.min(
          Math.floor(contextWindow / 2),
          Math.max(policy.reserveTokens, Math.ceil(contextWindow * (policy.reservePercent / 100)))
        )
      : 0;
  const absoluteCap = policy.tokenCap > 0 ? policy.tokenCap : Number.MAX_SAFE_INTEGER;
  const triggerAtTokens =
    contextWindow > 0
      ? Math.max(
          1,
          Math.min(
            contextWindow - reserveTokens,
            Math.ceil(contextWindow * (policy.triggerPercent / 100)),
            absoluteCap
          )
        )
      : 0;
  const percent =
    tokens !== null && contextWindow > 0 ? Math.round((tokens / contextWindow) * 10_000) / 100 : null;
  if (!policy.enabled) {
    return {
      trigger: false,
      tokens,
      contextWindow,
      reserveTokens,
      triggerAtTokens,
      percent,
      reason: "disabled",
    };
  }
  if (tokens === null || contextWindow <= 0) {
    return {
      trigger: false,
      tokens,
      contextWindow,
      reserveTokens,
      triggerAtTokens,
      percent,
      reason: "unknown_usage",
    };
  }
  const trigger = tokens >= triggerAtTokens;
  return {
    trigger,
    tokens,
    contextWindow,
    reserveTokens,
    triggerAtTokens,
    percent,
    reason: trigger ? "context_pressure" : "below_threshold",
  };
}

/**
 * Conservative preflight using Pi's own cut-point algorithm. The exact gate runs
 * again in session_before_compact with Pi's live CompactionPreparation/settings.
 */
export function evaluateProactiveCompactionEligibility(
  entries: readonly BranchEntry[],
  contextWindow: number
): ProactiveCompactionEligibility {
  if (entries.length === 0) return ineligible("empty_session", true);
  if (entries.at(-1)?.type === "compaction") return ineligible("already_compacted", true);

  let previousCompactionIndex = -1;
  for (let index = entries.length - 1; index >= 0; index -= 1) {
    if (entries[index]?.type === "compaction") {
      previousCompactionIndex = index;
      break;
    }
  }

  let boundaryStart = 0;
  if (previousCompactionIndex >= 0) {
    const previous = entries[previousCompactionIndex];
    if (previous?.type === "compaction") {
      const firstKeptIndex = entries.findIndex((entry) => entry.id === previous.firstKeptEntryId);
      boundaryStart = firstKeptIndex >= 0 ? firstKeptIndex : previousCompactionIndex + 1;
    }
  }

  const cutPoint = findCutPoint(
    [...entries],
    boundaryStart,
    entries.length,
    DEFAULT_COMPACTION_SETTINGS.keepRecentTokens
  );
  if (!entries[cutPoint.firstKeptEntryIndex]?.id) {
    return ineligible("session_needs_migration", true);
  }

  const historyEnd = cutPoint.isSplitTurn ? cutPoint.turnStartIndex : cutPoint.firstKeptEntryIndex;
  let compactableTokens = estimateEntryRange(entries, boundaryStart, historyEnd);
  if (cutPoint.isSplitTurn) {
    compactableTokens += estimateEntryRange(entries, cutPoint.turnStartIndex, cutPoint.firstKeptEntryIndex);
  }
  const maxSummaryTokens = Math.ceil(
    DEFAULT_COMPACTION_SETTINGS.reserveTokens * (cutPoint.isSplitTurn ? 1.3 : 0.8)
  );
  return evaluateEstimatedRoi(compactableTokens, contextWindow, maxSummaryTokens);
}

function evaluateExactPreparation(
  messagesToSummarize: readonly unknown[],
  turnPrefixMessages: readonly unknown[],
  contextWindow: number,
  reserveTokens: number
): ProactiveCompactionEligibility {
  const compactableTokens = [...messagesToSummarize, ...turnPrefixMessages].reduce<number>(
    (total, message) => total + estimateTokens(message as Parameters<typeof estimateTokens>[0]),
    0
  );
  const maxSummaryTokens = Math.ceil(reserveTokens * (turnPrefixMessages.length > 0 ? 1.3 : 0.8));
  return evaluateEstimatedRoi(compactableTokens, contextWindow, maxSummaryTokens);
}

function evaluateEstimatedRoi(
  compactableTokens: number,
  contextWindow: number,
  maxSummaryTokens: number
): ProactiveCompactionEligibility {
  const minimumReclaimTokens = Math.max(4_096, Math.ceil(contextWindow * 0.02));
  const estimatedOverheadTokens = Math.max(2_048, maxSummaryTokens + 1_024);
  const estimatedNetSavingsTokens = compactableTokens - estimatedOverheadTokens;
  if (compactableTokens <= 0) {
    return ineligible("insufficient_history", true, compactableTokens, estimatedOverheadTokens);
  }
  if (compactableTokens < minimumReclaimTokens) {
    return ineligible("insufficient_reclaim", false, compactableTokens, estimatedOverheadTokens);
  }
  if (estimatedNetSavingsTokens <= 0 || compactableTokens / estimatedOverheadTokens < 1.5) {
    return ineligible("negative_roi", false, compactableTokens, estimatedOverheadTokens);
  }
  return {
    eligible: true,
    terminal: false,
    reason: "eligible",
    compactableTokens,
    estimatedOverheadTokens,
    estimatedNetSavingsTokens,
  };
}

function ineligible(
  reason: Exclude<ProactiveCompactionEligibility["reason"], "eligible">,
  terminal: boolean,
  compactableTokens = 0,
  estimatedOverheadTokens = 0
): ProactiveCompactionEligibility {
  return {
    eligible: false,
    terminal,
    reason,
    compactableTokens,
    estimatedOverheadTokens,
    estimatedNetSavingsTokens: compactableTokens - estimatedOverheadTokens,
  };
}

function estimateEntryRange(entries: readonly BranchEntry[], start: number, end: number): number {
  let total = 0;
  for (let index = Math.max(0, start); index < Math.max(start, end); index += 1) {
    const entry = entries[index];
    if (entry?.type === "message") total += estimateTokens(entry.message);
    if (entry?.type === "custom_message") total += Math.ceil(JSON.stringify(entry.content).length / 4);
    if (entry?.type === "branch_summary") total += Math.ceil(entry.summary.length / 4);
  }
  return total;
}

const MODULE_LOAD_ID = randomUUID();

export function registerAutoCompaction(
  pi: ExtensionAPI,
  getPolicy: () => ProactiveCompactionPolicy = () => DEFAULT_PROACTIVE_COMPACTION_POLICY,
  getConfig: () => FocusaConfig | undefined = () => undefined
): boolean {
  const processLease = processCompactionLease();
  if (processLease.owner) {
    if (processLease.owner.moduleLoadId === MODULE_LOAD_ID) {
      if (!processLease.duplicateDiagnosticEmitted) {
        processLease.duplicateDiagnosticEmitted = true;
        console.warn(
          `[focusa] duplicate extension suppressed; active compaction owner=${processLease.owner.registrationId} build=${processLease.owner.extensionBuild} source=${processLease.owner.registrationSource}. Remove the duplicate Focusa installation and reload Pi.`
        );
      }
      return false;
    }
    processLease.owner = undefined;
  }

  // Spec130A §16 permits one linked retry per pressure crossing. Provider
  // WebSocket/network failures are retryable, but never through an unbounded
  // request loop or an additional summarizer call.
  const maxTransientRetries = 1;
  const registrationId = randomUUID();
  processLease.generation += 1;
  processLease.owner = {
    registrationId,
    adapterInstanceId: `pi-process-${process.pid}-${registrationId}`,
    extensionBuild: EXTENSION_BUILD,
    registrationSource: REGISTRATION_SOURCE,
    attachmentId: `pending:${registrationId}`,
    registeredHandlers: [...REGISTERED_HANDLERS],
    moduleLoadId: MODULE_LOAD_ID,
  };
  processLease.duplicateDiagnosticEmitted = false;
  const ownsRegistrationLease = (): boolean => processLease.owner?.registrationId === registrationId;
  let inFlight = false;
  let lastAttemptAt: number | undefined;
  let consecutiveTransientFailures = 0;
  let retryTimer: NodeJS.Timeout | undefined;
  let heartbeatTimer: NodeJS.Timeout | undefined;
  let activeEpoch: ActiveEpoch | undefined;
  let activeRequest: CoordinatedCompactionRequest | undefined;
  let terminalNoopContextKey: string | undefined;
  let lastNoticeKey: string | undefined;

  const setActiveEpoch = (epoch: ActiveEpoch | undefined): void => {
    activeEpoch = epoch;
    if (ownsRegistrationLease()) processLease.activeEpoch = epoch;
  };

  const createEpoch = (
    ctx: ExtensionContext,
    triggerClass: ActiveEpoch["triggerClass"],
    contextWindow: number,
    attempt = 1,
    retryOfEpochId?: string
  ): ActiveEpoch => {
    const contextKey = contextEpochKey(ctx);
    const epochId = `sha256:${createHash("sha256")
      .update(
        JSON.stringify({
          adapter: "pi",
          instance_id: processLease.owner?.adapterInstanceId,
          attachment_id: processLease.owner?.attachmentId,
          project_root: ctx.cwd,
          native_session: ctx.sessionManager.getSessionId(),
          source_turn: contextKey,
          trigger_class: triggerClass,
          attempt,
          retry_of_epoch_id: retryOfEpochId,
        })
      )
      .digest("hex")}`;
    return {
      epochId,
      retryOfEpochId,
      attempt,
      triggerClass,
      contextKey,
      contextWindow,
      startedAt: 0,
      nativeCompactionCallCount: 0,
      state: "prepare_requested",
      providerCapabilities: providerCompactionCapabilities(ctx),
    };
  };

  const persist = (
    kind: string,
    details: Record<string, unknown> = {},
    epoch: ActiveEpoch | undefined = activeEpoch
  ): void => {
    try {
      const event = {
        schema: "focusa.auto_compaction_event.v1" as const,
        kind,
        recorded_at: new Date().toISOString(),
        epoch_id: epoch?.epochId,
        retry_of_epoch_id: epoch?.retryOfEpochId,
        attempt: epoch?.attempt,
        coordinator_state: epoch?.state,
        native_compaction_call_count: epoch?.nativeCompactionCallCount,
        provider_capabilities: epoch?.providerCapabilities,
        registration_id: registrationId,
        registration_generation: processLease.generation,
        ...details,
      };
      pi.appendEntry(EVENT_TYPE, event);
      processLease.projection = reduceCompactionAuthorityEvents([event], processLease.projection);
    } catch (error) {
      console.warn("[focusa] could not persist auto-compaction telemetry", error);
    }
  };

  const recordOutcome = (
    ctx: ExtensionContext,
    epoch: ActiveEpoch,
    outcome: CompactionContinuationSnapshot
  ): void => {
    if (!epoch.outcomeBaseline) return;
    const evaluation = evaluateCompactionOutcome(epoch.outcomeBaseline, outcome);
    persist("outcome_evaluated", { outcome_evaluation: evaluation }, epoch);
    observeFrozenCompactionOutcome(ctx, {
      epochId: epoch.epochId,
      triggerClass: epoch.triggerClass,
      tokensBefore: epoch.outcomeBaseline.snapshot.contextTokens,
      tokensAfter: outcome.contextTokens,
      projectionTokens: 900,
      hardFindings: evaluation.reasons,
      rollbackTriggered: evaluation.rollbackRequired,
    });
    if (evaluation.rollbackRequired) {
      persist(
        "policy_rollback_required",
        {
          outcome_evaluation: evaluation,
          quarantined_policy_key: evaluation.policyKey,
          rollback_route: evaluation.rollbackRoute,
        },
        epoch
      );
    } else if (evaluation.disposition === "promote") {
      persist("policy_promoted", { outcome_evaluation: evaluation }, epoch);
    }
  };

  pi.registerCommand("focusa-compaction-policy", {
    description: "Show or override the scoped adaptive compaction policy",
    handler: async (args, ctx) => {
      const [action = "status", route, ...reasonParts] = String(args || "")
        .trim()
        .split(/\s+/)
        .filter(Boolean);
      let response: any;
      if (action === "set") {
        response = await focusaFetch("/compaction/policy/override", {
          method: "POST",
          body: JSON.stringify({
            action: "set",
            route,
            reason: reasonParts.join(" ") || "explicit Pi operator override",
            actor_ref: "pi-operator",
          }),
        });
      } else if (action === "clear") {
        response = await focusaFetch("/compaction/policy/override", {
          method: "POST",
          body: JSON.stringify({
            action: "clear",
            reason: [route, ...reasonParts].filter(Boolean).join(" ") || "Pi operator cleared override",
            actor_ref: "pi-operator",
          }),
        });
      } else {
        response = await focusaFetch("/compaction/policy");
      }
      processLease.operatorOverride = policyOverride(response);
      const policy = response?.policy ?? response;
      const text = response
        ? `Compaction policy: ${policy?.pressure_percent ?? "?"}% · route=${policy?.selected_route ?? "none"} · reason=${policy?.reason ?? "none"} · rollback=${policy?.rollback_route ?? "none"} · override=${policy?.operator_override?.route ?? "none"}${response?.receipt?.receipt_id ? ` · receipt=${response.receipt.receipt_id}` : ""}`
        : "Compaction policy unavailable; no local authority changed.";
      if (ctx.hasUI) ctx.ui.notify(text, response ? "info" : "warning");
    },
  });

  const notifyOnce = (
    ctx: ExtensionContext,
    key: string,
    message: string,
    level: "info" | "warning" | "error"
  ): void => {
    if (lastNoticeKey === key) return;
    lastNoticeKey = key;
    if (ctx.hasUI) ctx.ui.notify(message, level);
  };

  const stopCompactionHeartbeat = (ctx?: ExtensionContext): void => {
    if (heartbeatTimer) clearInterval(heartbeatTimer);
    heartbeatTimer = undefined;
    if (ctx?.hasUI) ctx.ui.setStatus("focusa-auto-compaction", undefined);
  };

  const startCompactionHeartbeat = (
    ctx: ExtensionContext,
    epoch: ActiveEpoch,
    contextPercent: number | undefined
  ): void => {
    stopCompactionHeartbeat();
    const startedAt = Date.now();
    let nextVisibleNoticeSeconds = 15;
    const render = (): void => {
      const elapsedSeconds = Math.max(0, Math.floor((Date.now() - startedAt) / 1000));
      if (ctx.hasUI) {
        ctx.ui.setStatus(
          "focusa-auto-compaction",
          `⏳ compacting · ${elapsedSeconds}s · ${contextPercent ?? "?"}% context · attempt ${epoch.attempt}`
        );
        if (elapsedSeconds >= nextVisibleNoticeSeconds) {
          ctx.ui.notify(
            `⏳ Focusa compaction still running · ${elapsedSeconds}s · attempt ${epoch.attempt}`,
            "info"
          );
          nextVisibleNoticeSeconds += 30;
        }
      }
    };
    render();
    heartbeatTimer = setInterval(render, 5_000);
    heartbeatTimer.unref?.();
  };

  const releaseProcessAttempt = (epochId: string | undefined): void => {
    if (!epochId || processLease.inFlightEpochId !== epochId) return;
    processLease.inFlightEpochId = undefined;
    processLease.attemptOwnerId = undefined;
  };

  const clearProcessRetry = (): void => {
    if (processLease.retryOwnerId === registrationId) processLease.retryOwnerId = undefined;
  };

  // Focusa owns the epoch and decision. Patched Pi owns safe execution: idle
  // requests run natively without replacing the extension context; active-loop
  // requests queue until turn_end and compact before the next model call.
  const attemptCompaction = (ctx: ExtensionContext, usageBefore: ContextUsage): void => {
    if (!activeEpoch) return;
    if (!ownsRegistrationLease()) {
      persist("attempt_suppressed", { reason: "registration_lease_lost" });
      setActiveEpoch(undefined);
      return;
    }
    if (processLease.inFlightEpochId && processLease.inFlightEpochId !== activeEpoch.epochId) {
      persist("attempt_suppressed", {
        reason: "process_compaction_already_in_flight",
        owner_id: processLease.attemptOwnerId,
        in_flight_epoch_id: processLease.inFlightEpochId,
      });
      setActiveEpoch(undefined);
      return;
    }
    if (activeEpoch.nativeCompactionCallCount >= 1) {
      activeEpoch.state = "blocked";
      persist("attempt_suppressed", { reason: "native_call_budget_exhausted" });
      setActiveEpoch(undefined);
      return;
    }
    const invokedEpoch = activeEpoch;
    clearProcessRetry();
    processLease.inFlightEpochId = invokedEpoch.epochId;
    processLease.attemptOwnerId = registrationId;
    inFlight = true;
    lastAttemptAt = Date.now();
    invokedEpoch.startedAt = lastAttemptAt;
    invokedEpoch.nativeCompactionCallCount += 1;
    invokedEpoch.state = "native_compaction_active";
    persist(
      "attempt_started",
      {
        cwd: ctx.cwd,
        context_key: invokedEpoch.contextKey,
        tokens_before: usageBefore.tokens,
        context_window: usageBefore.contextWindow,
      },
      invokedEpoch
    );
    startCompactionHeartbeat(ctx, invokedEpoch, usageBefore.percent ?? undefined);
    const attachmentKey = currentAttachmentKey();
    const withinAttachment = <T>(operation: () => T): T =>
      attachmentKey ? runWithAttachmentRuntime(attachmentKey, operation) : operation();
    const bindAttachmentCallback = <Args extends unknown[]>(
      callback: (...args: Args) => void
    ) =>
      (...args: Args): void =>
        withinAttachment(() => callback(...args));

    ctx.compact({
      customInstructions: activeRequest?.customInstructions ?? INSTRUCTIONS,
      onComplete: bindAttachmentCallback((result) => {
        if (invokedEpoch.settlement) {
          persist(
            "secondary_duplicate_settlement",
            { first_settlement: invokedEpoch.settlement, duplicate_settlement: "complete" },
            invokedEpoch
          );
          return;
        }
        invokedEpoch.settlement = "complete";
        invokedEpoch.state = "native_compaction_complete";
        const completedEpoch = invokedEpoch;
        const usageAfter = ctx.getContextUsage();
        const tokensAfter = usageAfter?.tokens ?? undefined;
        if (completedEpoch.outcomeBaseline) {
          recordOutcome(ctx, completedEpoch, {
            ...completedEpoch.outcomeBaseline.snapshot,
            providerOutcome: "succeeded",
            qualityScore: null,
            contextTokens: tokensAfter ?? null,
          });
        }
        const savedTokens = tokensAfter === undefined ? undefined : result.tokensBefore - tokensAfter;
        persist(
          "attempt_completed",
          {
            tokens_before: result.tokensBefore,
            tokens_after: tokensAfter,
            saved_tokens: savedTokens,
            net_positive:
              savedTokens === undefined
                ? undefined
                : savedTokens > (completedEpoch.exactEligibility?.estimatedOverheadTokens ?? 0),
            duration_ms: Date.now() - completedEpoch.startedAt,
          },
          completedEpoch
        );
        releaseProcessAttempt(completedEpoch.epochId);
        inFlight = false;
        stopCompactionHeartbeat(ctx);
        setActiveEpoch(undefined);
        consecutiveTransientFailures = 0;
        terminalNoopContextKey = undefined;
        lastNoticeKey = undefined;
        notifyOnce(
          ctx,
          `complete:${result.firstKeptEntryId}`,
          "Focusa compacted context proactively.",
          "info"
        );
        const completedRequest = activeRequest;
        activeRequest = undefined;
        completedRequest?.onComplete?.();
      }),
      onError: bindAttachmentCallback((error) => {
        const message = error.message || String(error);
        if (invokedEpoch.settlement) {
          persist(
            "secondary_duplicate_settlement",
            {
              first_settlement: invokedEpoch.settlement,
              duplicate_settlement: "failed",
              secondary_error: message,
              failure_class: compactionFailureClass(message, undefined),
            },
            invokedEpoch
          );
          return;
        }
        invokedEpoch.settlement = "failed";
        invokedEpoch.state = "native_compaction_failed";
        invokedEpoch.primaryError = message;
        const failedEpoch = invokedEpoch;
        const exactRejection =
          failedEpoch.exactEligibility?.eligible === false ? failedEpoch.exactEligibility : undefined;
        const failureClass = compactionFailureClass(message, exactRejection);
        const retryableFailure = isRetryableCompactionError(message);
        if (failedEpoch.outcomeBaseline) {
          recordOutcome(ctx, failedEpoch, {
            ...failedEpoch.outcomeBaseline.snapshot,
            providerOutcome: "failed",
            qualityScore: null,
            contextTokens: ctx.getContextUsage()?.tokens ?? null,
          });
        }
        stopCompactionHeartbeat(ctx);
        persist(
          exactRejection ? "eligibility_rejected" : "attempt_failed",
          {
            primary_error: message,
            failure_class: failureClass,
            terminal: exactRejection?.terminal ?? isTerminalNoopError(message),
            eligibility: exactRejection,
            duration_ms: Date.now() - failedEpoch.startedAt,
          },
          failedEpoch
        );
        releaseProcessAttempt(failedEpoch.epochId);
        inFlight = false;

        if (exactRejection || isTerminalNoopError(message)) {
          terminalNoopContextKey = failedEpoch.contextKey;
          consecutiveTransientFailures = 0;
          setActiveEpoch(undefined);
          notifyOnce(
            ctx,
            `noop:${terminalNoopContextKey}:${exactRejection?.reason ?? "nothing_to_compact"}`,
            `Focusa deferred proactive compaction: ${exactRejection?.reason ?? "nothing_to_compact"}.`,
            "warning"
          );
          const rejectedRequest = activeRequest;
          activeRequest = undefined;
          rejectedRequest?.onError?.(error);
          return;
        }

        if (retryableFailure && consecutiveTransientFailures < maxTransientRetries) {
          consecutiveTransientFailures += 1;
          // Spec130A's retry budget is 60s; a smaller explicit test/operator
          // cooldown remains valid, but ordinary policy cannot postpone recovery longer.
          const retryDelay = Math.min(getPolicy().cooldownMs, 60_000);
          const priorEpochId = failedEpoch.epochId;
          persist(
            "retry_scheduled",
            {
              primary_error: message,
              retry_delay_ms: retryDelay,
              next_attempt: consecutiveTransientFailures + 1,
            },
            failedEpoch
          );
          notifyOnce(
            ctx,
            `retry:${failedEpoch.epochId}:${consecutiveTransientFailures + 1}`,
            `Focusa compaction attempt ${failedEpoch.attempt} failed: ${message.slice(0, 240)}. Retrying in ${Math.ceil(retryDelay / 1000)}s.`,
            "warning"
          );
          setActiveEpoch(
            createEpoch(
              ctx,
              failedEpoch.triggerClass,
              failedEpoch.contextWindow,
              consecutiveTransientFailures + 1,
              priorEpochId
            )
          );
          processLease.retryOwnerId = registrationId;
          retryTimer = setTimeout(bindAttachmentCallback(() => {
            retryTimer = undefined;
            if (!ownsRegistrationLease()) {
              persist("retry_suppressed", { reason: "registration_lease_lost" });
              clearProcessRetry();
              setActiveEpoch(undefined);
              return;
            }
            if (!ctx.isIdle() || ctx.hasPendingMessages()) {
              persist("retry_suppressed", { reason: "session_not_idle" });
              clearProcessRetry();
              setActiveEpoch(undefined);
              return;
            }
            const liveUsage = ctx.getContextUsage();
            const liveDecision = proactiveCompactionDecision(liveUsage, getPolicy());
            if (!liveUsage || !liveDecision.trigger) {
              persist("retry_suppressed", { reason: "live_context_no_longer_requires_action" });
              clearProcessRetry();
              setActiveEpoch(undefined);
              return;
            }
            const liveKey = contextEpochKey(ctx);
            if (liveKey !== activeEpoch?.contextKey) {
              persist("retry_suppressed", { reason: "context_epoch_changed" });
              clearProcessRetry();
              setActiveEpoch(undefined);
              return;
            }
            attemptCompaction(ctx, liveUsage);
          }), retryDelay);
          retryTimer.unref?.();
          return;
        }

        const failedAttempts = consecutiveTransientFailures + 1;
        consecutiveTransientFailures = 0;
        if (retryableFailure) {
          failedEpoch.state = "native_compaction_failed";
          persist(
            "native_recovery_deferred_to_pi",
            {
              reason: "provider_transport_retry_exhausted",
              attempts: failedAttempts,
              primary_error: message,
              recovery_owner: "pi_native_threshold_or_overflow_compaction",
              operator_input_preserved: true,
              canonical_checkpoint_preserved: true,
            },
            failedEpoch
          );
        }
        const recoveryInstruction = retryableFailure
          ? ". Pi retains operator input and owns native threshold/overflow recovery; Focusa will not force session rollover."
          : ".";
        setActiveEpoch(undefined);
        notifyOnce(
          ctx,
          `failed:${failedEpoch.contextKey}:${message}`,
          `Focusa proactive compaction failed after ${failedAttempts} attempt(s): ${message}${recoveryInstruction}`,
          retryableFailure ? "warning" : "error"
        );
        const failedRequest = activeRequest;
        activeRequest = undefined;
        failedRequest?.onError?.(error);
      }),
    });
  };

  const maybeCompact = (
    ctx: ExtensionContext,
    request: CoordinatedCompactionRequest = {
      triggerClass: "predicted_pressure",
      customInstructions: INSTRUCTIONS,
    }
  ): CoordinatedCompactionRequestResult => {
    if (!ownsRegistrationLease()) return "coordinator_unavailable";
    const toolBoundaryRequest =
      !ctx.isIdle() &&
      ["predicted_pressure", "hard_pressure"].includes(request.triggerClass) &&
      piSupportsToolBoundaryCompaction();
    if (
      inFlight ||
      retryTimer ||
      processLease.inFlightEpochId ||
      processLease.retryOwnerId ||
      ctx.hasPendingMessages() ||
      (!ctx.isIdle() && !toolBoundaryRequest)
    ) {
      return "suppressed";
    }

    const usage = ctx.getContextUsage();
    const policy = getPolicy();
    const decision = proactiveCompactionDecision(usage, policy);
    const capabilities = providerCompactionCapabilities(ctx);
    const pressureTelemetry = contextPressureTelemetry(ctx, capabilities);
    const candidatePolicy = selectFrozenCompactionPolicy(ctx, pressureTelemetry, capabilities);
    const selectedPolicy = applyOperatorOverride(
      applyCompactionPolicyQuarantine(
        candidatePolicy,
        processLease.projection.quarantinedPolicyKeys,
        processLease.projection.rollbackRoute
      ),
      processLease.operatorOverride
    );
    const policySelection =
      decision.trigger &&
      capabilities.nativeCompaction === "supported" &&
      ["no_op", "curate_context", "checkpoint"].includes(selectedPolicy.route)
        ? {
            ...selectedPolicy,
            route: "native_compact" as const,
            executionOwner: "pi" as const,
            reason: "native_pressure" as const,
            deterministicKey: `${selectedPolicy.deterministicKey}:focusa-threshold-upgrade`,
          }
        : selectedPolicy;
    const successCooldownRemaining = processLease.lastSuccessfulCompactionAt
      ? PROACTIVE_COMPACTION_SUCCESS_COOLDOWN_MS - (Date.now() - processLease.lastSuccessfulCompactionAt)
      : 0;
    if (
      request.triggerClass !== "hard_pressure" &&
      successCooldownRemaining > 0 &&
      policySelection.route !== "no_op" &&
      !processLease.operatorOverride
    ) {
      persist("successful_compaction_hysteresis", {
        remaining_ms: successCooldownRemaining,
        selected_policy: policySelection,
      });
      return "suppressed";
    }
    focusaPost("/compaction/policy/report", {
      pressure_percent: pressureTelemetry.percent,
      selected_route: policySelection.route,
      reason: policySelection.reason,
      evidence_refs: capabilities.evidenceRefs,
      rollback_route: processLease.projection.rollbackRoute,
    });
    persist("pressure_observed", {
      pressure_telemetry: pressureTelemetry,
      provider_capabilities: capabilities,
      policy_selection: policySelection,
    });
    if (["no_op", "curate_context", "checkpoint"].includes(policySelection.route)) {
      return "suppressed";
    }
    if (policySelection.route === "rollover") {
      notifyOnce(
        ctx,
        `rollover:${policySelection.deterministicKey}`,
        "Focusa cannot safely compact with the current provider capability; checkpoint and rollover are required.",
        "warning"
      );
      return "ineligible";
    }
    if (!usage) return "suppressed";

    if (!decision.trigger) {
      if (decision.reason === "below_threshold") {
        terminalNoopContextKey = undefined;
        lastNoticeKey = undefined;
      }
      return "suppressed";
    }

    const contextKey = contextEpochKey(ctx);
    if (terminalNoopContextKey === contextKey) return "ineligible";

    if (lastAttemptAt !== undefined && Date.now() - lastAttemptAt < policy.cooldownMs) {
      notifyOnce(
        ctx,
        `cooldown:${contextKey}`,
        `Focusa context pressure is ${(decision.percent ?? 0).toFixed(1)}%; proactive compaction is cooling down.`,
        "warning"
      );
      return "suppressed";
    }

    const eligibility = evaluateProactiveCompactionEligibility(
      ctx.sessionManager.getBranch(),
      usage.contextWindow
    );
    if (!eligibility.eligible) {
      terminalNoopContextKey = contextKey;
      lastAttemptAt = Date.now();
      const rejectedEpoch = createEpoch(ctx, request.triggerClass, usage.contextWindow);
      rejectedEpoch.startedAt = lastAttemptAt;
      rejectedEpoch.state = "blocked";
      rejectedEpoch.exactEligibility = eligibility;
      setActiveEpoch(rejectedEpoch);
      persist("preflight_rejected", { eligibility, tokens_before: usage.tokens });
      setActiveEpoch(undefined);
      notifyOnce(
        ctx,
        `preflight:${contextKey}:${eligibility.reason}`,
        `Focusa deferred proactive compaction: ${eligibility.reason}.`,
        "warning"
      );
      return "ineligible";
    }

    // The same coordinator serves settled sessions and active tool boundaries.
    // Focusa acquires the process epoch; Pi executes the one native compaction at
    // its safe lifecycle boundary and naturally continues the active loop.
    const requestedEpoch = createEpoch(ctx, request.triggerClass, usage.contextWindow);
    requestedEpoch.exactEligibility = eligibility;
    requestedEpoch.policySelection = policySelection;
    requestedEpoch.state = "native_compaction_requested";
    activeRequest = request;
    setActiveEpoch(requestedEpoch);
    persist(
      "native_compaction_requested",
      {
        reason: toolBoundaryRequest
          ? "focusa_active_tool_boundary_pressure"
          : "focusa_settled_pressure_threshold",
        tokens_before: usage.tokens,
        context_window: usage.contextWindow,
        eligibility,
      },
      requestedEpoch
    );
    attemptCompaction(ctx, usage);
    return "requested";
  };

  processLease.request = maybeCompact;

  pi.on("session_before_compact", async (event, ctx) => {
    const externalNativeInvocation = !inFlight || !activeEpoch;
    if (externalNativeInvocation) {
      const usage = ctx.getContextUsage();
      const nativeReason = (event as { reason?: string }).reason;
      const reason = String(nativeReason || "").toLowerCase();
      const triggerClass: CompactionTriggerClass =
        reason === "manual"
          ? "manual"
          : reason.includes("overflow")
            ? "provider_overflow"
            : "predicted_pressure";
      const observedEpoch = createEpoch(
        ctx,
        triggerClass,
        usage?.contextWindow ?? event.preparation.tokensBefore
      );
      observedEpoch.nativeCompactionCallCount = 1;
      observedEpoch.state = "preparing";
      setActiveEpoch(observedEpoch);
      persist("native_invocation_observed", { native_reason: nativeReason }, observedEpoch);
    }
    if (!activeEpoch) return;
    const selectedPolicy = activeEpoch.policySelection ?? {
      schema: "focusa.compaction_policy_selection.v1" as const,
      policyVersion: "1" as const,
      route: "native_compact" as const,
      executionOwner: "pi" as const,
      reason: "native_pressure" as const,
      percent: null,
      deterministicKey: `native:${activeEpoch.triggerClass}:${activeEpoch.contextKey}`,
    };
    activeEpoch.outcomeBaseline = {
      schema: "focusa.compaction_outcome_baseline.v1",
      policyVersion: selectedPolicy.policyVersion,
      policyKey: selectedPolicy.deterministicKey,
      route: selectedPolicy.route,
      snapshot: {
        projectRoot: ctx.cwd,
        sessionId: ctx.sessionManager.getSessionId(),
        continuityRef: null,
        workpointRef: null,
        evidenceRefs: [],
        providerOutcome: "unknown",
        qualityScore: null,
        contextTokens: event.preparation.tokensBefore,
      },
    };
    persist("outcome_baseline_recorded", { outcome_baseline: activeEpoch.outcomeBaseline }, activeEpoch);
    // Manual and provider-overflow recovery outrank optional ROI optimization,
    // but their outcomes are still measured for authority/evidence regression.
    if (["manual", "provider_overflow"].includes(activeEpoch.triggerClass)) return;
    const exactEligibility = evaluateExactPreparation(
      event.preparation.messagesToSummarize,
      event.preparation.turnPrefixMessages,
      activeEpoch.contextWindow,
      event.preparation.settings.reserveTokens
    );
    activeEpoch.exactEligibility = exactEligibility;
    activeEpoch.state = exactEligibility.eligible ? "prepared" : "blocked";
    if (!exactEligibility.eligible) {
      if (externalNativeInvocation) {
        // Focusa improves Pi-owned threshold/overflow compaction but never vetoes
        // the baseline recovery path because optional ROI preparation is degraded.
        persist("native_eligibility_observed", { eligibility: exactEligibility }, activeEpoch);
        setActiveEpoch(undefined);
        return;
      }
      return { cancel: true };
    }
  });

  pi.on("agent_end", async (_event, _ctx) => {
    // Pi may still schedule native post-run compaction after agent_end. Running our
    // own compaction here races that path and can create duplicate summaries.
    if (!ownsRegistrationLease()) return;
    if (retryTimer) {
      clearTimeout(retryTimer);
      retryTimer = undefined;
      persist("retry_suppressed", { reason: "new_agent_run_or_native_compaction_pending" });
      clearProcessRetry();
      setActiveEpoch(undefined);
    }
  });

  pi.on("agent_settled", async (_event, ctx) => {
    maybeCompact(ctx);
  });

  pi.on("input", async (event, ctx) => {
    if (!ownsRegistrationLease()) return { action: "continue" as const };
    const usage = ctx.getContextUsage();
    const percent = proactiveCompactionDecision(usage, getPolicy()).percent ?? 0;
    if (percent < 95) return { action: "continue" as const };

    // Pi owns threshold/overflow compaction and retries the accepted prompt with
    // its text, images, and steering semantics intact. Focusa must never return
    // `handled` here: doing so drops Pi's native retry path and forces the operator
    // to run a command and resend steering manually.
    persist("input_passthrough_native_overflow_recovery", {
      context_percent: percent,
      input_preserved_by: "pi_native_prompt_queue",
      image_count: event.images?.length ?? 0,
      recovery_owner: "pi_native_threshold_or_overflow_compaction",
    });
    if (ctx.hasUI) {
      ctx.ui.setStatus("focusa-auto-compaction", "🧭 Pi native compaction");
    }
    return { action: "continue" as const };
  });

  pi.on("session_compact", async (_event, ctx) => {
    // The public ctx.compact callback owns an active process epoch. Native/manual
    // completion may reset observation state only when no Focusa call is active.
    if (processLease.inFlightEpochId) return;
    if (!ownsRegistrationLease()) return;
    processLease.lastSuccessfulCompactionAt = Date.now();
    if (retryTimer) clearTimeout(retryTimer);
    retryTimer = undefined;
    stopCompactionHeartbeat();
    clearProcessRetry();
    setActiveEpoch(undefined);
    consecutiveTransientFailures = 0;
    terminalNoopContextKey = undefined;
    lastNoticeKey = undefined;
  });

  pi.on("session_start", async (_event, ctx) => {
    if (!ownsRegistrationLease()) return;
    // Rebind on every session activation. This is idempotent and repairs any
    // prior lifecycle that cleared the callable while retaining ownership.
    processLease.request = maybeCompact;
    if (processLease.owner) {
      processLease.owner.nativeSession = ctx.sessionManager.getSessionId();
      processLease.owner.attachmentId =
        ctx.sessionManager.getSessionFile() ?? ctx.sessionManager.getSessionId();
    }
    const persistedEvents = ctx.sessionManager
      .getBranch()
      .filter(
        (entry): entry is Extract<BranchEntry, { type: "custom" }> =>
          entry.type === "custom" && entry.customType === EVENT_TYPE
      )
      .map((entry) => entry.data);
    processLease.projection = reduceCompactionAuthorityEvents(persistedEvents);
    persist("runtime_registration_verified", {
      extension_build: EXTENSION_BUILD,
      registration_source: REGISTRATION_SOURCE,
      native_session: ctx.sessionManager.getSessionId(),
    });
    await prewarmCompactionPolicy(ctx, getConfig()).catch(() => undefined);
    const policyStatus = await focusaFetch("/compaction/policy").catch(() => null);
    processLease.operatorOverride = policyOverride(policyStatus);
    inFlight = false;
    lastAttemptAt = undefined;
    stopCompactionHeartbeat(ctx);
    setActiveEpoch(undefined);
    consecutiveTransientFailures = 0;
    terminalNoopContextKey = undefined;
    lastNoticeKey = undefined;
    if (retryTimer) clearTimeout(retryTimer);
    retryTimer = undefined;
    clearProcessRetry();
    if (!processLease.inFlightEpochId) processLease.attemptOwnerId = undefined;
  });

  pi.on("session_shutdown", async () => {
    if (!ownsRegistrationLease()) return;
    if (retryTimer) clearTimeout(retryTimer);
    retryTimer = undefined;
    stopCompactionHeartbeat();
    clearProcessRetry();
    setActiveEpoch(undefined);
    inFlight = false;
    if (!processLease.inFlightEpochId) {
      processLease.attemptOwnerId = undefined;
      // session_shutdown is a session lifecycle boundary, not an extension
      // unload. Preserve coordinator ownership and its request function so the
      // next session_start can resume compaction without re-registering code.
      if (processLease.owner) processLease.owner.nativeSession = undefined;
      processLease.duplicateDiagnosticEmitted = false;
    }
  });
  return true;
}

function contextEpochKey(ctx: ExtensionContext): string {
  const branch = ctx.sessionManager.getBranch();
  const meaningful = [...branch]
    .reverse()
    .find((entry) => entry.type !== "custom" || entry.customType !== EVENT_TYPE);
  return `${ctx.sessionManager.getSessionId()}:${meaningful?.id ?? "empty"}`;
}

function isTerminalNoopError(message: string): boolean {
  return /nothing to compact|already compacted/i.test(message);
}

function compactionFailureClass(
  message: string,
  exactRejection: ProactiveCompactionEligibility | undefined
): "eligibility_rejection" | "terminal_noop" | "primary_transport" | "secondary_reentrancy" | "primary" {
  if (exactRejection) return "eligibility_rejection";
  if (isTerminalNoopError(message)) return "terminal_noop";
  if (/undefined.*signal|reading ['\"]signal['\"]|compaction.*already.*progress/i.test(message)) {
    return "secondary_reentrancy";
  }
  if (isTransientCompactionError(message)) return "primary_transport";
  return "primary";
}

function isTransientCompactionError(message: string): boolean {
  return /websocket|network|socket|timeout|timed out|connection|temporar|rate.?limit|429|502|503|504/i.test(
    message
  );
}

function isRetryableCompactionError(message: string): boolean {
  // Any error classified as provider transport is retried with the bounded
  // exponential-backoff budget above. Treating WebSocket/network failures as
  // terminal strands an unchanged, already-over-limit context forever because
  // the next run cannot reach compaction. Never narrow this to HTTP status codes.
  return isTransientCompactionError(message);
}
