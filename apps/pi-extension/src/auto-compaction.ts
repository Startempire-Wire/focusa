// Proactive context compaction fallback for Spec 130.
// Pi's native threshold compaction remains primary. This guard prevents silent
// regressions when host auto-compaction is disabled or misconfigured.

import type { ContextUsage, ExtensionAPI, ExtensionContext } from "@mariozechner/pi-coding-agent";

// The deployed Pi runtime exposes agent_settled (documented public event), while
// the extension's pinned 0.64 development declarations predate that overload.
declare module "@mariozechner/pi-coding-agent" {
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

export interface ProactiveCompactionDecision {
  trigger: boolean;
  tokens: number | null;
  contextWindow: number;
  reserveTokens: number;
  triggerAtTokens: number;
  percent: number | null;
  reason: "unknown_usage" | "below_threshold" | "context_pressure";
}

export function proactiveCompactionDecision(usage: ContextUsage | undefined): ProactiveCompactionDecision {
  const tokens = usage?.tokens ?? null;
  const contextWindow = Math.max(0, usage?.contextWindow ?? 0);
  const reserveTokens =
    contextWindow > 0
      ? Math.min(
          Math.floor(contextWindow / 2),
          Math.max(
            PROACTIVE_COMPACTION_MIN_RESERVE_TOKENS,
            Math.ceil(contextWindow * PROACTIVE_COMPACTION_RESERVE_FRACTION)
          )
        )
      : 0;
  const triggerAtTokens =
    contextWindow > 0
      ? Math.max(
          1,
          Math.min(
            contextWindow - reserveTokens,
            Math.ceil(contextWindow * PROACTIVE_COMPACTION_TRIGGER_FRACTION),
            PROACTIVE_COMPACTION_ABSOLUTE_TOKEN_CAP
          )
        )
      : 0;
  const percent =
    tokens !== null && contextWindow > 0 ? Math.round((tokens / contextWindow) * 10_000) / 100 : null;
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

export function registerAutoCompaction(pi: ExtensionAPI): void {
  let pending = false;
  let lastTriggeredAt = 0;
  let evaluationTimer: ReturnType<typeof setTimeout> | null = null;

  const clearTimer = () => {
    if (evaluationTimer) clearTimeout(evaluationTimer);
    evaluationTimer = null;
  };

  const maybeCompact = (ctx: ExtensionContext) => {
    if (pending || !ctx.isIdle()) return;
    const decision = proactiveCompactionDecision(ctx.getContextUsage());
    if (!decision.trigger) return;
    const now = Date.now();
    if (now - lastTriggeredAt < PROACTIVE_COMPACTION_COOLDOWN_MS) return;

    pending = true;
    lastTriggeredAt = now;
    ctx.ui.setStatus("focusa-auto-compaction", `auto-compacting · ${decision.percent ?? "?"}% context`);
    ctx.ui.notify(
      `Focusa auto-compaction triggered at ${decision.percent ?? "unknown"}% context usage.`,
      "info"
    );

    const clearPending = () => {
      pending = false;
      ctx.ui.setStatus("focusa-auto-compaction", undefined);
    };
    try {
      ctx.compact({
        customInstructions:
          "Focusa proactive pressure compaction: preserve the operator's current ask, canonical Workpoint/Trajectory authority, verified evidence handles, blockers, exact next action, and do-not-drift boundaries.",
        onComplete: clearPending,
        onError: (error: Error) => {
          clearPending();
          ctx.ui.notify(`Focusa auto-compaction failed: ${error.message}`, "error");
        },
      });
    } catch (error) {
      clearPending();
      const message = error instanceof Error ? error.message : String(error);
      ctx.ui.notify(`Focusa auto-compaction failed: ${message}`, "error");
    }
  };

  pi.on("session_start", async () => {
    clearTimer();
    pending = false;
    lastTriggeredAt = 0;
  });

  pi.on("session_compact", async () => {
    clearTimer();
    pending = false;
  });

  pi.on("agent_end", async (_event, ctx) => {
    clearTimer();
    // Yield to Pi's native post-run compaction check. The host can still be busy
    // with retry/continuation work after agent_end, so this is only the fast path.
    evaluationTimer = setTimeout(() => {
      evaluationTimer = null;
      maybeCompact(ctx);
    }, 0);
  });

  pi.on("agent_settled", async (_event, ctx) => {
    // agent_settled is the authoritative idle boundary. Re-evaluate here so an
    // agent_end check skipped while Pi was busy cannot silently lose compaction.
    clearTimer();
    maybeCompact(ctx);
  });
}
