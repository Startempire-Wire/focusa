// Compaction handlers + tier logic + micro-compact
// Spec: §20 (tier), §21 (micro-compact), §25.7 (non-canonical), §33.1 (ASCC),
//        §33.10 (customInstructions), §35.6 (files), §38.1 (trim)

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { S, focusaFetch, getFocusState, buildCompactInstructions, persistState, persistAuthoritativeState, sanitizeFocusFailures } from "./state.js";
import { pushDelta } from "./tools.js";

function basename(value: string): string {
  const parts = String(value || "").split("/").filter(Boolean);
  return parts[parts.length - 1] || String(value || "file");
}

function normalizeCompactionArtifacts(files: any[]): Array<{ kind: "file"; label: string; path_or_id: string }> {
  return (Array.isArray(files) ? files : [])
    .map((file) => String(file || "").trim())
    .filter(Boolean)
    .slice(0, 20)
    .map((file) => ({ kind: "file" as const, label: basename(file), path_or_id: file }));
}

let compactResumeRetryTimer: ReturnType<typeof setTimeout> | null = null;

async function refreshWorkpointResumePacket(mode = "compact_prompt"): Promise<any | null> {
  if (!S.focusaAvailable) return null;
  try {
    const packet = await focusaFetch("/workpoint/resume", {
      method: "POST",
      body: JSON.stringify({ mode }),
    });
    if (packet && packet.status === "completed") {
      S.activeWorkpointPacket = packet.resume_packet || packet;
      S.activeWorkpointSummary = packet.rendered_summary || packet.next_step_hint || "";
      return packet;
    }
  } catch { /* best effort */ }
  return null;
}


function recordLocalWorkpointFallback(reason: string): void {
  const fallback = {
    status: "partial",
    canonical: false,
    reason,
    mission: S.currentAsk?.text || S.activeFrameGoal || S.lastFocusSnapshot.intent || "unknown mission",
    next_slice: S.lastFocusSnapshot.currentFocus || S.lastCompactDecision || "resume from local degraded fallback",
    source_turn_id: `pi-turn-${S.turnCount}`,
    recorded_at: new Date().toISOString(),
  };
  S.activeWorkpointPacket = fallback;
  S.activeWorkpointSummary = `NON-CANONICAL WORKPOINT FALLBACK: ${fallback.next_slice}`;
  try { S.pi?.appendEntry("focusa-workpoint-fallback", fallback); } catch { /* best effort */ }
  persistState();
}

async function checkpointBeforeCompaction(): Promise<any | null> {
  if (!S.focusaAvailable) return null;
  const mission = S.currentAsk?.text || S.activeFrameGoal || S.lastFocusSnapshot.intent || S.lastFocusSnapshot.currentFocus || "Pi work before compaction";
  const nextSlice = S.lastFocusSnapshot.currentFocus || S.lastCompactDecision || "Resume current task from typed Workpoint packet after compaction.";
  try {
    return await focusaFetch("/workpoint/checkpoint", {
      method: "POST",
      body: JSON.stringify({
        mission,
        next_slice: nextSlice,
        work_item_id: S.currentAsk?.sourceTurnId,
        checkpoint_reason: "before_compact",
        canonical: true,
        promote: true,
        source_turn_id: `pi-turn-${S.turnCount}`,
        action_intent: {
          action_type: "resume_workpoint",
          target_ref: S.currentAsk?.sourceTurnId || S.activeFrameId || "pi-session",
          verification_hooks: ["resume packet appears in compaction instructions", "post-compact steer uses WorkpointResumePacket"],
          status: "ready",
        },
      }),
    });
  } catch { return null; }
}

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

function submitCompactionResumeTurn(ctx: any, steerMessage: string): boolean {
  const pi2 = S.pi;
  if (!pi2) return false;
  pi2.sendMessage({
    customType: "focusa-compact-resume",
    content: steerMessage,
    display: false,
  }, { triggerTurn: true });
  ctx.ui.notify(`✅ Compaction done — auto-resume turn submitted`, "info");
  return true;
}

function scheduleCompactionResumeRetry(ctx: any, steerMessage: string, retryAttempt = 1) {
  if (!S.compactResumePending) return;
  const nextAttempt = retryAttempt + 1;
  compactResumeRetryTimer = setTimeout(() => {
    compactResumeRetryTimer = null;
    if (!S.compactResumePending) return;
    try {
      submitCompactionResumeTurn(ctx, steerMessage);
      scheduleCompactionResumeRetry(ctx, steerMessage, retryAttempt + 1);
    } catch (e) {
      console.warn(`[focusa] compaction auto-resume retry ${retryAttempt} failed:`, e);
      if (!S.compactResumePending) return;
      scheduleCompactionResumeRetry(ctx, steerMessage, nextAttempt);
    }
  }, Math.min(30_000, 2_000 * retryAttempt));
}

function scheduleCompactionResumeWatchdog(ctx: any, steerMessage: string) {
  if (!S.compactResumePending) return;
  scheduleCompactionResumeRetry(ctx, steerMessage, 1);
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
        failures: sanitizeFocusFailures(S.localFailures).slice(-5),
      });
    }
    await checkpointBeforeCompaction();
    const workpointPacket = await refreshWorkpointResumePacket("compact_prompt");

    // Always persist to Pi session entries as backup
    await persistAuthoritativeState();

    // §33.1: Try Focusa ASCC replacement FIRST
    if (S.focusaAvailable) {
      try {
        const ascc = await focusaFetch("/ascc/state");
        if (ascc?.focus_state) {
          const fs = ascc.focus_state;
          const workpointSection = workpointPacket ? [
            "# Workpoint Resume Packet",
            workpointPacket.rendered_summary || S.activeWorkpointSummary || "active workpoint packet available",
            JSON.stringify(workpointPacket.resume_packet || S.activeWorkpointPacket || {}, null, 2).slice(0, 4000),
          ].join("\n\n") : "";
          const summary = [
            workpointSection,
            "# Focusa Cognitive Summary",
            `## Intent\n${fs.intent || "none"}`,
            `## Current Focus\n${fs.current_focus || fs.current_state || "none"}`,
            `## Decisions Made\n${(fs.decisions || []).map((d: string) => `- ${d}`).join("\n") || "none"}`,
            `## Active Constraints\n${(fs.constraints || []).map((c: string) => `- ${c}`).join("\n") || "none"}`,
            `## Failures Encountered\n${sanitizeFocusFailures(fs.failures || []).map((f: string) => `- ${f}`).join("\n") || "none"}`,
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

    // §35.6: Feed modified/read files to Focusa as canonical artifact lines
    const compaction = (event as any).compactionEntry;
    const modifiedFiles = compaction?.details?.modifiedFiles || compaction?.details?.fileOps || [];
    const readFiles = compaction?.details?.readFiles || [];
    const artifacts = normalizeCompactionArtifacts(modifiedFiles);
    const compactNotes: string[] = [];
    if (artifacts.length) compactNotes.push(`Session compacted. Modified: ${artifacts.map((a) => a.path_or_id).join(", ")}`);
    if (Array.isArray(readFiles) && readFiles.length) compactNotes.push(`Session compacted. Read: ${readFiles.slice(0, 20).join(", ")}`);
    if (S.focusaAvailable && S.activeFrameId && (artifacts.length || compactNotes.length)) {
      await focusaFetch("/focus/update", {
        method: "POST",
        body: JSON.stringify({
          frame_id: S.activeFrameId,
          turn_id: `pi-turn-${S.turnCount}`,
          delta: {
            ...(artifacts.length ? { artifacts } : {}),
            ...(compactNotes.length ? { notes: compactNotes } : {}),
          },
        }),
      }).catch(() => {});
      await persistAuthoritativeState();
    }

    // §38.3 CRITICAL FIX: queueMicrotask defers to next event-loop tick,
    // AFTER compaction_end fires (which calls flushCompactionQueue first,
    // then hasQueuedMessages() -> agent.continue()). Without deferral,
    // sendMessage is still async when hasQueuedMessages() fires -> miss.
    // Also dedup: only resume once per compaction cycle.
    const compactionEntry = (event as any).compactionEntry || {};
    const compactOrdinal = S.totalCompactions || compactionEntry.details?.totalCompactions || "unknown";
    const compactResumeKey = String(compactionEntry.id || compactionEntry.uuid || compactionEntry.timestamp || `${S.sessionFrameKey || "session"}:compact:${compactOrdinal}`);
    const recentlySubmitted = S.lastCompactResumeKey === compactResumeKey || (Date.now() - S.lastCompactResumeAt < 30_000 && compactOrdinal !== "unknown");
    if (!S.compactResumePending && !recentlySubmitted) {
      S.lastCompactResumeKey = compactResumeKey;
      S.lastCompactResumeAt = Date.now();
      persistState();
      if (compactResumeRetryTimer) {
        clearTimeout(compactResumeRetryTimer);
        compactResumeRetryTimer = null;
      }
      S.compactResumePending = true;
      const pi2 = S.pi;
      if (pi2) {
        queueMicrotask(() => {
          // lastDecision saved above, before localDecisions was cleared
          const workpoint = S.activeWorkpointSummary || S.activeWorkpointPacket?.next_slice || "";
          const directive = workpoint
            ? `Resume from the WorkpointResumePacket below. Treat it as the canonical continuation contract unless the operator steers otherwise.`
            : (S.localDecisions.length > 0 || S.lastCompactDecision
              ? `Review the above decisions and constraints. Continue with the next logical step.`
              : `Continue executing. Context was compacted — preserve all progress.`);
          const note = S.totalCompactions > 0 ? ` [compaction #${S.totalCompactions}]` : "";
          const steerMessage = `# Compaction Complete${note}
## Last Active Focus
${S.lastCompactDecision || "pre-compaction work"}
## WorkpointResumePacket
${S.activeWorkpointSummary || "none"}
${S.activeWorkpointPacket ? JSON.stringify(S.activeWorkpointPacket, null, 2).slice(0, 4000) : ""}
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

See: ls /tmp/pi-scratch/ | cat /tmp/pi-scratch/turn-NNNN/notes.txt`;
          try {
            submitCompactionResumeTurn(ctx, steerMessage);
            scheduleCompactionResumeWatchdog(ctx, steerMessage);
          } catch (e) {
            console.warn("[focusa] auto-resume failed:", e);
            S.compactResumePending = false;
          }
        });
      }
    } else if (recentlySubmitted) {
      ctx.ui.notify("↩️ Compaction auto-resume already submitted for this compact cycle; suppressing duplicate.", "info");
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
  if (typeof usage.contextWindow === "number" && usage.contextWindow > 0) {
    S.activeContextWindow = usage.contextWindow;
  }
  const pct = typeof usage.percent === "number"
    ? usage.percent
    : (usage.tokens / (usage.contextWindow || S.activeContextWindow)) * 100;

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
    recordLocalWorkpointFallback("hard context fallback before local compact");
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
    recordLocalWorkpointFallback("auto context fallback before local compact");
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
