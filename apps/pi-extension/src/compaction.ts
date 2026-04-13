// Compaction handlers + tier logic + micro-compact
// Spec: §20 (tier), §21 (micro-compact), §25.7 (non-canonical), §33.1 (ASCC),
//        §33.10 (customInstructions), §35.6 (files), §38.1 (trim)

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { S, focusaFetch, getFocusState, buildCompactInstructions, persistState } from "./state.js";
import { pushDelta } from "./tools.js";

function setContextStatus(ctx: any, tier: "" | "warn" | "auto" | "hard", pct?: number) {
  S.currentContextPct = typeof pct === "number" ? pct : null;
  const mode = S.cfg?.contextStatusMode || "actionable";
  if (mode === "off") {
    ctx.ui.setStatus("focusa-ctx", "");
    return;
  }
  if (tier === "warn") {
    if (mode === "all" && typeof pct === "number") ctx.ui.setStatus("focusa-ctx", `📦 Context ${pct.toFixed(0)}% monitor`);
    else ctx.ui.setStatus("focusa-ctx", "");
    return;
  }
  if (tier === "auto" && typeof pct === "number") {
    ctx.ui.setStatus("focusa-ctx", `🗜️ Context ${pct.toFixed(0)}% compacting`);
    return;
  }
  if (tier === "hard" && typeof pct === "number") {
    ctx.ui.setStatus("focusa-ctx", `🚧 Context ${pct.toFixed(0)}% critical fork/new`);
    return;
  }
  ctx.ui.setStatus("focusa-ctx", "");
}

export function registerCompaction(pi: ExtensionAPI) {
  // ── session_before_compact (§33.1 ASCC replacement, §33.10 fallback) ───────
  pi.on("session_before_compact", async (event, _ctx) => {
    // Sync local shadow → Focusa before compaction
    // §33.1 + N5: Use pushDelta() for ALL writes — enforces validateSlot() on every delta.
    // session_compact bypassed validation before this fix — every compaction refilled
    // recent_results with verbose entries that validateSlot would have rejected.
    if (S.focusaAvailable && S.activeFrameId) {
      await pushDelta({
        decisions: S.localDecisions.slice(-10),
        constraints: S.localConstraints.slice(-10),
        failures: S.localFailures.slice(-5),
      });
    }
    // Always persist to Pi session entries as backup
    persistState();

    // §33.1: Try Focusa ASCC replacement FIRST
    if (S.focusaAvailable) {
      try {
        const ascc = await focusaFetch("/ascc/state");
        if (ascc?.focus_state) {
          const fs = ascc.focus_state;
          const summary = [
            "# Focusa Cognitive Summary",
            `## Intent\n${fs.intent || "none"}`,
            `## Current Focus\n${fs.current_focus || fs.current_state || "none"}`,
            `## Decisions Made\n${(fs.decisions || []).map((d: string) => `- ${d}`).join("\n") || "none"}`,
            `## Active Constraints\n${(fs.constraints || []).map((c: string) => `- ${c}`).join("\n") || "none"}`,
            `## Failures Encountered\n${(fs.failures || []).map((f: string) => `- ${f}`).join("\n") || "none"}`,
            `## Next Steps\n${(fs.next_steps || []).map((n: string) => `- ${n}`).join("\n") || "none"}`,
            `## Open Questions\n${(fs.open_questions || []).map((q: string) => `- ${q}`).join("\n") || "none"}`,
            `## Recent Results\n${(fs.recent_results || []).map((r: string) => `- ${r}`).join("\n") || "none"}`,
            `## Artifacts\n${(fs.artifacts || []).map((a: any) => `- ${a.kind}:${a.label}${a.path_or_id ? "@" + a.path_or_id : ""}`).join("\n") || "none"}`,
            `## Notes\n${(fs.notes || []).map((n: string) => `- ${n}`).join("\n") || "none"}`,
          ].join("\n\n");
          const ev = event as any;
          return {
            compaction: {
              summary,
              firstKeptEntryId: ev.preparation?.firstKeptEntryId,
              tokensBefore: ev.preparation?.tokensBefore,
            },
          };
        }
      } catch { /* ASCC unavailable — fall through to §33.10 */ }

      // §33.10: Softer fallback — customInstructions to guide Pi's compaction
      return { customInstructions: buildCompactInstructions(
        "Preserve Focusa Focus State (decisions, constraints, intent). Summarize older turns.",
      ) };
    }

    // Focusa offline — fall through to Pi's default compaction
    return undefined;
  });

  // ── session_compact (§38.1 trim, §35.6 files + auto-resume) ───────────────
  pi.on("session_compact", async (event, ctx) => {
    // §38.1: Trim local shadow only after Focusa confirms state.
    // NOTE: S.lastCompactDecision is saved BEFORE trimming (used in steer below).
    const lastDecision = S.localDecisions[S.localDecisions.length - 1] ?? "pre-compaction work";
    S.lastCompactDecision = lastDecision;

    if (S.focusaAvailable && S.activeFrameId) {
      const data = await getFocusState();
      if (data?.fs?.decisions?.length || data?.fs?.constraints?.length) {
        S.localDecisions = [];
        S.localConstraints = [];
        S.localFailures = [];
      }
    }

    // §35.6: Feed modified files to Focusa
    const compaction = (event as any).compactionEntry;
    const files = compaction?.details?.modifiedFiles || compaction?.details?.fileOps;
    if (S.focusaAvailable && S.activeFrameId && files?.length) {
      focusaFetch("/focus/update", {
        method: "POST",
        body: JSON.stringify({
          frame_id: S.activeFrameId,
          turn_id: `pi-turn-${S.turnCount}`,
          delta: { artifacts: (Array.isArray(files) ? files : []).slice(0, 20) },
        }),
      }).catch(() => {});
    }

    // §38.3 CRITICAL FIX: queueMicrotask defers to next event-loop tick,
    // AFTER compaction_end fires (which calls flushCompactionQueue first,
    // then hasQueuedMessages() -> agent.continue()). Without deferral,
    // sendMessage is still async when hasQueuedMessages() fires -> miss.
    // Also dedup: only resume once per compaction cycle.
    if (!S.compactResumePending) {
      S.compactResumePending = true;
      const pi2 = S.pi;
      if (pi2) {
        queueMicrotask(() => {
          // lastDecision saved above, before localDecisions was cleared
          const directive = S.localDecisions.length > 0 || S.lastCompactDecision
            ? `Review the above decisions and constraints. Continue with the next logical step.`
            : `Continue executing. Context was compacted — preserve all progress.`;
          const note = S.totalCompactions > 0 ? ` [compaction #${S.totalCompactions}]` : "";
          try {
            pi2.sendMessage({
              customType: "focusa-compact-resume",
              content: `# Compaction Complete${note}
## Last Active Focus
${S.lastCompactDecision || "pre-compaction work"}
## Directive
${directive}

---

## Focusa Tool Guidance
When using focusa_scratch / focusa_decide / focusa_constraint / focusa_failure:
- **Working notes** → focusa_scratch (all internal monologue welcome)
- **Crystallized decision** → focusa_decide (ONE sentence, max 280 chars, architectural choice)
- **Discovered requirement** → focusa_constraint (hard boundary from environment/architecture)
- **Failure diagnosis** → focusa_failure (specific component + why it failed)
- **Validation** fails if: task patterns (Fix/Add/Check), debug patterns (error/failed), self-reference (I think/I tried), or exceeding char limits

See: ls /tmp/pi-scratch/ | cat /tmp/pi-scratch/turn-NNNN/notes.txt`,
              display: false,
            }, { deliverAs: "steer" });
            // Belt-and-suspenders: call agent.continue() directly via sessionManager.
            // Pi's compaction_end → hasQueuedMessages() check may miss sendMessage timing,
            // so we bypass the queue entirely and force continuation.
            const agent = (ctx as any).sessionManager?.getAgent?.();
            if (agent) {
              agent.continue().catch(() => {});
            }
            ctx.ui.notify(`✅ Compaction done — resuming work`, "info");
          } catch (e) {
            console.warn("[focusa] auto-resume failed:", e);
            S.compactResumePending = false;
          }
        });
      }
    }
  });
}

// ── Compaction tier check — called from turn_end in turns.ts (§20) ───────────
export async function checkCompactionTier(ctx: any): Promise<void> {
  const cfg = S.cfg;
  if (!cfg) return;
  S.turnsSinceCompact++;

  const usage = ctx.getContextUsage?.();
  if (!usage?.tokens) return;
  const pct = (usage.tokens / S.activeContextWindow) * 100;

  // Reset hourly counter
  if (Date.now() - S.compactHourStart > 3_600_000) {
    S.compactsThisHour = 0;
    S.compactHourStart = Date.now();
  }

  const cooldownOk = Date.now() - S.lastCompactTime > cfg.cooldownMs;
  const hourlyOk = S.compactsThisHour < cfg.maxCompactionsPerHour;
  const turnsOk = S.turnsSinceCompact >= cfg.minTurnsBetweenCompactions;
  const canCompact = cooldownOk && hourlyOk && turnsOk;

  const onDone = () => {
    S.lastCompactTime = Date.now();
    S.compactsThisHour++;
    S.totalCompactions++;
    S.turnsSinceCompact = 0;
    S.currentTier = "";
    S.forkSuggested = false; // Reset after compaction frees space
  };

  // §18: autoSuggestForkPct — check BEFORE tier branches so it fires at any tier
  if (pct >= cfg.autoSuggestForkPct && !S.forkSuggested) {
    S.forkSuggested = true;
    ctx.ui.notify(`💡 Context at ${pct.toFixed(0)}% — consider /fork to preserve context quality`, "warning");
  }

  if (pct >= cfg.hardPct) {
    S.currentTier = "hard";
    setContextStatus(ctx, "hard", pct);
    ctx.ui.notify(`⚠️ Context ${pct.toFixed(0)}% — hard compacting. Consider /fork or /new.`, "warning");
    // §18: Suggest handoff after N compactions
    if (S.totalCompactions >= cfg.autoSuggestHandoffAfterNCompactions) {
      ctx.ui.notify(`💡 ${S.totalCompactions} compactions — consider /fork or session handoff`, "warning");
    }
    const r = S.focusaAvailable
      ? await focusaFetch("/commands/submit", {
          method: "POST",
          body: JSON.stringify({ command: "compact", args: { force: true, tier: "hard" }, idempotency_key: `hard-${Date.now()}` }),
        })
      : null;
    if (r?.accepted) { onDone(); return; }
    // §25.7: Fallback marked non-canonical — guard ctx.compact existence
    if ((cfg.fallbackMode === "passthrough" || cfg.fallbackMode === "local-compact") && typeof ctx.compact === "function") {
      ctx.compact({
        customInstructions: buildCompactInstructions(
          "[NON-CANONICAL FALLBACK — Focusa unavailable] HARD COMPACT: Context critically full.",
        ),
        onComplete: onDone,
        onError: (e: Error) => ctx.ui.notify(`Compaction failed: ${e.message}`, "error"),
      });
    }
  } else if (pct >= cfg.compactPct && canCompact) {
    S.currentTier = "auto";
    setContextStatus(ctx, "auto", pct);
    ctx.ui.notify(`📊 Context ${pct.toFixed(0)}% — compacting`, "info");
    const r = S.focusaAvailable
      ? await focusaFetch("/commands/submit", {
          method: "POST",
          body: JSON.stringify({ command: "compact", args: { force: false, tier: "auto" }, idempotency_key: `auto-${Date.now()}` }),
        })
      : null;
    if (r?.accepted) { onDone(); return; }
    if ((cfg.fallbackMode === "passthrough" || cfg.fallbackMode === "local-compact") && typeof ctx.compact === "function") {
      ctx.compact({
        customInstructions: buildCompactInstructions(
          "[NON-CANONICAL FALLBACK] Context approaching limit. Preserve Focus State.",
        ),
        onComplete: onDone,
        onError: (e: Error) => ctx.ui.notify(`Compaction failed: ${e.message}`, "error"),
      });
    }
  } else if (pct >= cfg.warnPct) {
    S.currentTier = "warn";
    setContextStatus(ctx, "warn", pct);
  } else {
    S.currentTier = "";
    setContextStatus(ctx, "");
  }
}

// ── Periodic micro-compact (§21) — called from turn_end ─────────────────────
export async function checkMicroCompact(): Promise<void> {
  const n = S.cfg?.microCompactEveryNTurns || 5;
  if (S.turnCount > 0 && S.turnCount % n === 0 && S.focusaAvailable) {
    // §21: Request micro-compact via Focusa API (not extension-owned summarization)
    focusaFetch("/commands/submit", {
      method: "POST",
      body: JSON.stringify({
        command: "micro-compact",
        args: { turn_count: S.turnCount, surface: "pi" },
        idempotency_key: `micro-${S.turnCount}-${Date.now()}`,
      }),
    }).catch(() => {});
  }
}
