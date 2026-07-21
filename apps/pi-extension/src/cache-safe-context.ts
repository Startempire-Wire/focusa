import { createHash } from "node:crypto";

export type CacheMissReason =
  | "cache_hit"
  | "stable_system_prefix_changed"
  | "historical_prefix_shifted"
  | "model_changed"
  | "ttl_expired"
  | "compaction_or_branch_reset"
  | "provider_cache_unavailable"
  | "unknown_provider_miss";

export type CachePrefixSnapshot = {
  sessionCacheKeyHash: string;
  stableSystemPrefixHash: string;
  historyPrefixHash: string;
  historyMessageHashes: string[];
  dynamicSliceHash: string;
  dynamicSliceEstimatedTokens: number;
  capturedAt: number;
};

export type CacheUsageObservation = {
  sessionKey: string;
  provider: string;
  model: string;
  inputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens?: number;
  layoutMode: "cache_safe_tail" | "legacy_prepend";
  observedAt?: number;
};

export type NormalizedCacheUsage = {
  inputTokens: number;
  uncachedInputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
};

export function normalizeCacheUsage(rawUsage: any): NormalizedCacheUsage {
  const usage = rawUsage || {};
  const uncachedInputTokens = Number(usage.inputTokens || usage.input || usage.input_tokens || 0);
  const cacheReadTokens = Number(usage.cacheReadInputTokens || usage.cacheRead || usage.cache_read || 0);
  const cacheWriteTokens = Number(
    usage.cacheCreationInputTokens || usage.cacheWrite || usage.cache_write || 0
  );
  return {
    inputTokens: uncachedInputTokens + cacheReadTokens + cacheWriteTokens,
    uncachedInputTokens,
    cacheReadTokens,
    cacheWriteTokens,
  };
}

export type CacheSafetyObservation = {
  miss: boolean;
  reason: CacheMissReason;
  provider: string;
  model: string;
  inputTokens: number;
  cacheReadTokens: number;
  cacheWriteTokens: number;
  estimatedRebilledTokens: number;
  cacheReadRatio: number;
  idleDurationMs: number | null;
  layoutMode: "cache_safe_tail" | "legacy_prepend";
  consecutivePrefixMisses: number;
  cacheSafeDegraded: boolean;
  transitionedToDegraded: boolean;
  sessionCacheKeyHash: string;
  stableSystemPrefixHash: string;
  historyPrefixHash: string;
  dynamicSliceHash: string;
  dynamicSliceEstimatedTokens: number;
};

type SessionCacheState = {
  systemPrompt: string;
  current?: CachePrefixSnapshot;
  previous?: CachePrefixSnapshot;
  lastProvider?: string;
  lastModel?: string;
  lastObservedAt?: number;
  lastCacheReadTokens?: number;
  consecutivePrefixMisses: number;
  degraded: boolean;
};

const FIVE_MINUTES_MS = 5 * 60 * 1000;
const LARGE_PROMPT_TOKENS = 8_000;
const MISS_READ_RATIO = 0.5;
const DEGRADED_AFTER_MISSES = 2;

function hash(value: string): string {
  return createHash("sha256").update(value).digest("hex");
}

function textContentContains(content: unknown, marker: string): boolean {
  if (typeof content === "string") return content.includes(marker);
  return (
    Array.isArray(content) &&
    content.some((part: any) => part?.type === "text" && String(part.text || "").includes(marker))
  );
}

/** Attach volatile Focusa context to the newest user turn, after cacheable history. */
export function attachFocusSliceToNewestUser(messages: any[], slice: string): any[] {
  const marker = "[Focusa Focus Slice — minimal applicable context]";
  const next = [...messages];
  let userIndex = -1;
  for (let index = next.length - 1; index >= 0; index -= 1) {
    if (next[index]?.role === "user") {
      userIndex = index;
      break;
    }
  }
  if (userIndex < 0) {
    return [...next, { role: "user", content: [{ type: "text", text: slice }] }];
  }
  const current = next[userIndex];
  if (textContentContains(current?.content, marker)) return next;
  const content = current?.content;
  const updatedContent =
    typeof content === "string"
      ? `${content}\n\n${slice}`
      : Array.isArray(content)
        ? [...content, { type: "text", text: slice }]
        : [{ type: "text", text: slice }];
  next[userIndex] = { ...current, content: updatedContent };
  return next;
}

export function buildCachePrefixSnapshot(
  systemPrompt: string,
  messages: any[],
  dynamicSlice: string,
  capturedAt: number = Date.now(),
  sessionKey: string = "no-session"
): CachePrefixSnapshot {
  let newestUserIndex = -1;
  for (let index = messages.length - 1; index >= 0; index -= 1) {
    if (messages[index]?.role === "user") {
      newestUserIndex = index;
      break;
    }
  }
  const history = newestUserIndex >= 0 ? messages.slice(0, newestUserIndex) : messages;
  const historyMessageHashes = history.map((message) => hash(JSON.stringify(message)));
  return {
    sessionCacheKeyHash: hash(sessionKey),
    stableSystemPrefixHash: hash(systemPrompt),
    historyPrefixHash: hash(JSON.stringify(historyMessageHashes)),
    historyMessageHashes,
    dynamicSliceHash: hash(dynamicSlice),
    dynamicSliceEstimatedTokens: Math.ceil(dynamicSlice.length / 4),
    capturedAt,
  };
}

function preservesHistoricalPrefix(previous: string[], current: string[]): boolean {
  if (current.length < previous.length) return false;
  return previous.every((value, index) => current[index] === value);
}

export class CacheSafetyMonitor {
  private readonly sessions = new Map<string, SessionCacheState>();

  private state(sessionKey: string): SessionCacheState {
    let state = this.sessions.get(sessionKey);
    if (!state) {
      state = { systemPrompt: "", consecutivePrefixMisses: 0, degraded: false };
      this.sessions.set(sessionKey, state);
    }
    return state;
  }

  captureSystemPrompt(sessionKey: string, systemPrompt: string): string {
    const state = this.state(sessionKey);
    state.systemPrompt = systemPrompt;
    return hash(systemPrompt);
  }

  captureRequest(sessionKey: string, messages: any[], dynamicSlice: string): CachePrefixSnapshot {
    const state = this.state(sessionKey);
    const snapshot = buildCachePrefixSnapshot(
      state.systemPrompt,
      messages,
      dynamicSlice,
      Date.now(),
      sessionKey
    );
    state.previous = state.current;
    state.current = snapshot;
    return snapshot;
  }

  isDegraded(sessionKey: string): boolean {
    return this.state(sessionKey).degraded;
  }

  resetForDiscontinuity(sessionKey: string): void {
    const state = this.state(sessionKey);
    state.previous = undefined;
    state.current = undefined;
    state.lastProvider = undefined;
    state.lastModel = undefined;
    state.lastObservedAt = undefined;
    state.lastCacheReadTokens = undefined;
    state.consecutivePrefixMisses = 0;
    state.degraded = false;
  }

  observeUsage(observation: CacheUsageObservation): CacheSafetyObservation | null {
    const state = this.state(observation.sessionKey);
    const current = state.current;
    if (!current) return null;
    const now = observation.observedAt ?? Date.now();
    const inputTokens = Math.max(0, observation.inputTokens);
    const cacheReadTokens = Math.max(0, observation.cacheReadTokens);
    const cacheWriteTokens = Math.max(0, observation.cacheWriteTokens ?? 0);
    const totalPromptTokens = inputTokens + cacheReadTokens;
    const cacheReadRatio = totalPromptTokens > 0 ? cacheReadTokens / totalPromptTokens : 0;
    const idleDurationMs =
      state.lastObservedAt === undefined ? null : Math.max(0, now - state.lastObservedAt);
    const miss = totalPromptTokens >= LARGE_PROMPT_TOKENS && cacheReadRatio < MISS_READ_RATIO;
    const modelChanged =
      Boolean(state.lastModel) &&
      (state.lastModel !== observation.model || state.lastProvider !== observation.provider);
    const ttlExpired = idleDurationMs !== null && idleDurationMs > FIVE_MINUTES_MS;
    const systemChanged =
      Boolean(state.previous) && state.previous?.stableSystemPrefixHash !== current.stableSystemPrefixHash;
    const historyShortened =
      Boolean(state.previous) &&
      current.historyMessageHashes.length < (state.previous?.historyMessageHashes.length || 0);
    const historyShifted =
      Boolean(state.previous) &&
      !preservesHistoricalPrefix(state.previous?.historyMessageHashes || [], current.historyMessageHashes);
    const cacheReadPlateau =
      state.lastCacheReadTokens !== undefined &&
      cacheReadTokens > 0 &&
      Math.abs(cacheReadTokens - state.lastCacheReadTokens) <=
        Math.max(256, Math.floor(state.lastCacheReadTokens * 0.05));

    let reason: CacheMissReason = "cache_hit";
    if (miss) {
      if (modelChanged) reason = "model_changed";
      else if (ttlExpired) reason = "ttl_expired";
      else if (systemChanged) reason = "stable_system_prefix_changed";
      else if (historyShortened) reason = "compaction_or_branch_reset";
      else if (historyShifted) reason = "historical_prefix_shifted";
      else if (cacheReadTokens === 0) reason = "provider_cache_unavailable";
      else reason = "unknown_provider_miss";
    }

    const qualifyingMiss =
      miss &&
      !modelChanged &&
      !ttlExpired &&
      (reason === "stable_system_prefix_changed" ||
        reason === "historical_prefix_shifted" ||
        reason === "unknown_provider_miss");
    state.consecutivePrefixMisses = qualifyingMiss ? state.consecutivePrefixMisses + 1 : 0;
    const structuralPrefixChange =
      reason === "stable_system_prefix_changed" || reason === "historical_prefix_shifted";
    const transitionedToDegraded =
      !state.degraded &&
      state.consecutivePrefixMisses >= DEGRADED_AFTER_MISSES &&
      (structuralPrefixChange || cacheReadPlateau);
    if (transitionedToDegraded) state.degraded = true;
    state.lastProvider = observation.provider;
    state.lastModel = observation.model;
    state.lastObservedAt = now;
    state.lastCacheReadTokens = cacheReadTokens;

    return {
      miss,
      reason,
      provider: observation.provider,
      model: observation.model,
      inputTokens,
      cacheReadTokens,
      cacheWriteTokens,
      estimatedRebilledTokens: inputTokens,
      cacheReadRatio,
      idleDurationMs,
      layoutMode: observation.layoutMode,
      consecutivePrefixMisses: state.consecutivePrefixMisses,
      cacheSafeDegraded: state.degraded,
      transitionedToDegraded,
      sessionCacheKeyHash: current.sessionCacheKeyHash,
      stableSystemPrefixHash: current.stableSystemPrefixHash,
      historyPrefixHash: current.historyPrefixHash,
      dynamicSliceHash: current.dynamicSliceHash,
      dynamicSliceEstimatedTokens: current.dynamicSliceEstimatedTokens,
    };
  }
}
