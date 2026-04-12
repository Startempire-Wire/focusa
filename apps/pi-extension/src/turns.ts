// Turn lifecycle + per-call event handlers — ONE handler per event type
// Spec: §7.1 (10 ASCC slots), §7.4 (ECS thresholds), §33.2 (context), §33.3 (ECS replace),
//        §33.4 (tool usage), §34.2B (turns), §35.2 (behavioral), §35.5 (tokens),
//        §35.7 (correction), §36.1 (streaming), §36.2 (error signals), §36.3 (input),
//        §36.6 (injection layering), §36.7 (budget), §37.3 (widget), §37.8 (model),
//        §30 (metacognitive indicators)

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { S, focusaFetch, focusaPost, extractText, getFocusState, estimateTokens, wbExec, storeEcsArtifact } from "./state.js";

function latestUserText(messages: any[] | undefined): string {
  const latest = [...(messages || [])].reverse().find((m: any) => m?.role === "user");
  return extractText(latest?.content || latest?.message || "");
}

function classifyOperatorIntent(text: string): {
  directQuestion: boolean;
  steeringChange: boolean;
  focusRelevant: boolean;
} {
  const t = text.trim().toLowerCase();
  const directQuestion = /\?$/.test(t) || /^(what|why|how|when|where|who|can|could|should|is|are|do|does|did)\b/.test(t);
  const steeringChange = [
    "no ", "stop", "instead", "actually", "wait", "wrong", "not what i asked", "new task", "switch to", "different"
  ].some((needle) => t.includes(needle));
  const focusRelevant = /(focusa|focus state|stack|constraint|decision|thread|ontology|mission|frame|checkpoint|context)/.test(t);
  return { directQuestion, steeringChange, focusRelevant };
}

function uniqueItems(items: any[] | undefined): string[] {
  return [...new Set((items || []).filter(Boolean).map((x: any) => String(x).trim()).filter(Boolean))];
}
import { checkCompactionTier, checkMicroCompact } from "./compaction.js";
import { fetchWbmContext, catalogueFromMessages } from "./wbm.js";

export function registerTurns(pi: ExtensionAPI) {
  // ── before_agent_start (§29 WBM injection only; operator-first per docs 51/54a/54b) ────────────
  pi.on("before_agent_start", async (event, ctx) => {
    if (!S.focusaAvailable) {
      const h = await focusaFetch("/health");
      if (h?.ok) {
        S.focusaAvailable = true;
        ctx.ui.setStatus("focusa", S.wbmEnabled ? "🧠 Focusa [WBM]" : "🧠 Focusa");
      }
    }

    // WBM remains additive, but Focusa no longer appends coercive behavioral prose here.
    if (S.wbmEnabled) {
      const wbmCtx = await fetchWbmContext();
      (event as any).systemPrompt = `${(event as any).systemPrompt || ""}\n\n${wbmCtx}`.trim();
    }
  });

  // ── context — operator-first minimal applicable slice (docs 51/54a/54b) ─────
  // No always-on full focus-state block. Slice assembly happens after reading newest
  // operator input and suppresses irrelevant Focusa state on steering changes.
  pi.on("context", async (event: any, ctx: any) => {
    if (!S.focusaAvailable || !S.activeFrameId) return;

    const operatorText = latestUserText(event.messages);
    const intent = classifyOperatorIntent(operatorText);
    const data = await getFocusState();
    if (!data?.fs) return;
    const { fs, frame } = data;

    // Steering change / direct-answer cases suppress broad injected state.
    if (!intent.focusRelevant && (intent.directQuestion || intent.steeringChange)) {
      focusaPost("/telemetry/trace", {
        event_type: "subject_hijack_prevented",
        turn_id: `pi-turn-${S.turnCount}`,
        subject_hijack_prevented: true,
      });
      return;
    }

    const mission = frame?.thread_thesis || frame?.goal || frame?.title || fs.intent || "";
    const constraints = uniqueItems(fs.constraints).slice(0, 4);
    const decisions = uniqueItems(fs.decisions).slice(0, 4);
    const nextSteps = uniqueItems(fs.next_steps).slice(0, 3);
    const blockers = uniqueItems([...(fs.failures || []), ...(fs.open_questions || [])]).slice(0, 3);
    const recent = uniqueItems(fs.recent_results).slice(0, 2);
    const artifacts = uniqueItems((fs.artifacts || []).map((a: any) => `${a.kind}:${a.label}${a.path_or_id ? "@" + a.path_or_id : ""}`)).slice(0, 3);

    const sections: string[] = [];
    if (mission) sections.push(`MISSION:\n  - ${mission}`);
    if (constraints.length) sections.push(`APPLICABLE_CONSTRAINTS:\n${constraints.map((x) => `  - ${x}`).join("\n")}`);
    if (decisions.length && (intent.focusRelevant || /why|decision|previous|already|earlier|reuse/.test(operatorText.toLowerCase()))) {
      sections.push(`RELEVANT_DECISIONS:\n${decisions.map((x) => `  - ${x}`).join("\n")}`);
    }
    if (nextSteps.length && !intent.directQuestion) {
      sections.push(`OPEN_LOOPS:\n${nextSteps.map((x) => `  - ${x}`).join("\n")}`);
    }
    if (blockers.length && /blocked|failing|error|issue|problem|why/.test(operatorText.toLowerCase())) {
      sections.push(`BLOCKERS:\n${blockers.map((x) => `  - ${x}`).join("\n")}`);
    }
    if (recent.length && /latest|recent|changed|what happened|status/.test(operatorText.toLowerCase())) {
      sections.push(`RECENT_VERIFIED_DELTAS:\n${recent.map((x) => `  - ${x}`).join("\n")}`);
    }
    if (artifacts.length && /file|artifact|handle|output|evidence/.test(operatorText.toLowerCase())) {
      sections.push(`ARTIFACT_HANDLES:\n${artifacts.map((x) => `  - ${x}`).join("\n")}`);
    }

    if (!sections.length) return;

    const usage = ctx.getContextUsage?.();
    const window = S.activeContextWindow || 128000;
    const headroom = usage?.tokens ? window - usage.tokens - 16384 : window;
    const maxTokens = Math.min(Math.max(Math.floor(headroom * 0.08), 120), 600);

    let text = `[Focusa Minimal Applicable Slice]\n${sections.join("\n")}`;
    const tokens = estimateTokens(text);
    if (tokens > maxTokens) {
      const trimmed: string[] = [];
      for (const section of sections) {
        const candidate = `[Focusa Minimal Applicable Slice]\n${[...trimmed, section].join("\n")}`;
        if (estimateTokens(candidate) > maxTokens) break;
        trimmed.push(section);
      }
      if (!trimmed.length) return;
      text = `[Focusa Minimal Applicable Slice]\n${trimmed.join("\n")}`;
    }

    focusaPost("/telemetry/trace", {
      event_type: "focus_slice_size",
      turn_id: `pi-turn-${S.turnCount}`,
      focus_slice_size: estimateTokens(text),
    });

    return { messages: [{ role: "system" as const, content: [{ type: "text" as const, text }] }, ...(event.messages || [])] };
  });

  // ── input (§36.3 signal + §35.7 correction — single handler) ──────────────
  pi.on("input", async (event, _ctx) => {
    const text = (event as any).text || (event as any).message || "";

    if (S.focusaAvailable) {
      focusaPost("/focus-gate/ingest-signal", {
        signal_type: "user_input", surface: "pi",
        payload: { length: text.length, preview: String(text).slice(0, 200) },
      });
    }

    const lower = String(text).toLowerCase();
    const corrections = ["no that is wrong", "revert", "undo", "that's incorrect", "wrong approach", "go back", "not what i asked"];
    if (corrections.some(c => lower.includes(c))) {
      if (S.focusaAvailable && S.activeFrameId) {
        focusaPost("/focus/update", {
          frame_id: S.activeFrameId, turn_id: `pi-turn-${S.turnCount}`,
          delta: { failures: [`Operator correction: ${String(text).slice(0, 200)}`] },
        });
      }
      S.localFailures.push(`Operator correction: ${String(text).slice(0, 100)}`);
      // §35.7/§29: WBM trust metric update on correction
      if (S.wbmEnabled) {
        wbExec(["trust", "set", "--corrections", "+1"]).catch(() => {});
      }
    }
  });

  // ── turn_start (§34.2B) ───────────────────────────────────────────────────
  pi.on("turn_start", async (_event, _ctx) => {
    S.turnCount++;
    S.lastStreamLen = 0;
    S.toolUsageBatch = [];
    // Reset dedup flag so next compaction can re-trigger auto-resume
    S.compactResumePending = false;
    if (S.focusaAvailable) {
      focusaPost("/turn/start", { turn_id: `pi-turn-${S.turnCount}`, frame_id: S.activeFrameId });
    }
  });

  // ── turn_end (§35.5 tokens + §37.3 widget + §10.4 badges + §20 tier + §21 micro) ─
  pi.on("turn_end", async (event, ctx) => {
    const ev = event as any;
    const cfg = S.cfg;

    // §35.5: Token counts
    if (S.focusaAvailable) {
      focusaPost("/turn/complete", {
        turn_id: `pi-turn-${S.turnCount}`, frame_id: S.activeFrameId,
        tokens: {
          input: ev.usage?.inputTokens || ev.usage?.input || 0,
          output: ev.usage?.outputTokens || ev.usage?.output || 0,
          cache_read: ev.usage?.cacheReadInputTokens || 0,
          cache_write: ev.usage?.cacheCreationInputTokens || 0,
        },
      });
    }

    // §33.4: Flush batched tool usage
    if (S.focusaAvailable && S.toolUsageBatch.length) {
      focusaPost("/telemetry/tool-usage", { turn_id: `pi-turn-${S.turnCount}`, tools: S.toolUsageBatch });
      S.toolUsageBatch = [];
    }

    // §37.3 + §10.4: Widget with all badges
    const w: string[] = [];
    if (S.localDecisions.length) w.push(`📌 ${S.localDecisions.length} decisions`);
    if (S.localConstraints.length) w.push(`🔒 ${S.localConstraints.length} constraints`);
    if (S.localFailures.length) w.push(`⚠️ ${S.localFailures.length} failures`);
    if (S.wbmEnabled) w.push(`⚡ WBM ${S.wbmDeep ? "deep" : "on"}`);
    // §10.4: Compaction tier badge
    if (S.currentTier) w.push(`🔴 ${S.currentTier.toUpperCase()}`);
    // §10.4: Degraded-context badge
    if (!S.focusaAvailable) w.push("⚪ degraded");
    // §10.4: Thesis snippet
    if (S.focusaAvailable) {
      const data = await getFocusState();
      if (data?.frame?.thread_thesis) w.push(`🎯 ${data.frame.thread_thesis.slice(0, 50)}`);
    }
    // §30: Metacognitive indicator
    if (S.lastMetacogEvent) w.push(`🧠 ${S.lastMetacogEvent}`);
    ctx.ui.setWidget("focusa", w.length ? w : undefined);

    // §34.2C: Update Focus State on significant progress
    if (S.focusaAvailable && S.activeFrameId) {
      const hasSignificant = S.localDecisions.length > 0 || S.localConstraints.length > 0 || S.localFailures.length > 0;
      if (hasSignificant) {
        focusaFetch("/focus/update", {
          method: "POST",
          body: JSON.stringify({
            frame_id: S.activeFrameId,
            turn_id: `pi-turn-${S.turnCount}`,
            delta: {
              decisions: S.localDecisions.slice(-5),
              constraints: S.localConstraints.slice(-5),
              failures: S.localFailures.slice(-3),
            },
          }),
        }).catch(() => {});
      }
    }

    // §20: Compaction tier check
    await checkCompactionTier(ctx);
    // §21: Micro-compact check
    await checkMicroCompact();
  });

  // ── message_update (§36.1 streaming delta) ────────────────────────────────
  pi.on("message_update", async (event, _ctx) => {
    if (!S.focusaAvailable) return;
    const fullText = extractText((event as any).message?.content);
    if (S.turnCount % 10 !== 0 && fullText.length - S.lastStreamLen < 500) return;
    const delta = fullText.slice(S.lastStreamLen);
    if (!delta) return;
    S.lastStreamLen = fullText.length;
    focusaPost("/turn/append", { turn_id: `pi-turn-${S.turnCount}`, delta: delta.slice(-500) });
  });

  // ── model_select (§37.8) ──────────────────────────────────────────────────
  pi.on("model_select", async (event, _ctx) => {
    if (!S.focusaAvailable) return;
    const model = (event as any).model;
    S.activeContextWindow = model?.contextWindow || S.activeContextWindow;
    // §37.8: Wire model change to Focusa with frame context
    focusaPost("/focus-gate/ingest-signal", {
      signal_type: "model_change",
      surface: "pi",
      frame_id: S.activeFrameId,
      payload: { model_id: model?.id || "unknown", context_window: model?.contextWindow || S.activeContextWindow },
    });
  });

  // ── agent_end (§29 WBM catalogue + signals — single handler) ──────────────
  pi.on("agent_end", async (event, ctx) => {
    // §29: WBM outbound cataloguing
    if (S.wbmEnabled && !S.wbmNoCatalogue) {
      const messages = (event as any).messages || [];
      catalogueFromMessages(messages).catch(() => {});
    }

    // Long session detection
    const elapsed = (Date.now() - S.sessionStartTime) / 60_000;
    if (elapsed > 45 && !S.longSessionSignaled) {
      S.longSessionSignaled = true;
      if (S.focusaAvailable) {
        focusaPost("/focus-gate/ingest-signal", {
          signal_type: "long_session", surface: "pi",
          payload: { minutes: Math.round(elapsed), turns: S.turnCount },
        });
      }
    }

    // Tool error rate detection
    const recentErrors = S.compilationErrors.filter(t => Date.now() - t < 300_000);
    if (recentErrors.length >= 3) {
      ctx.ui.notify(`⚠️ ${recentErrors.length} compilation errors in 5 min — consider a different approach`, "warning");
      if (S.focusaAvailable) {
        focusaPost("/focus-gate/ingest-signal", {
          signal_type: "error_rate_high", surface: "pi",
          payload: { count: recentErrors.length, window_ms: 300_000 },
        });
      }
    }
  });

  // ── tool_result (§36.2 errors + §33.3 ECS REPLACE + §7.4 thresholds + §34.2D churn) ─
  pi.on("tool_result", async (event, _ctx) => {
    const ev = event as any;
    const toolName = ev.toolName || ev.name || "";
    const content = extractText(ev.content);
    const isError = ev.isError || /error|failed|ENOENT|EPERM/i.test(content.slice(0, 200));
    const cfg = S.cfg;

    // §36.2: Error signals
    if (isError && S.focusaAvailable) {
      focusaPost("/focus-gate/ingest-signal", {
        signal_type: "tool_error", surface: "pi",
        payload: { tool: toolName, error: content.slice(0, 500) },
      });
    }

    if (isError && /compil|tsc|typecheck|build|lint/i.test(toolName + " " + content.slice(0, 200))) {
      S.compilationErrors.push(Date.now());
    }

    // §7.4 + §33.3: ECS externalization — check BOTH thresholds, REPLACE content
    const byteThreshold = cfg?.externalizeThresholdBytes || 8192;
    const tokenThreshold = cfg?.externalizeThresholdTokens || 800;
    const tokens = estimateTokens(content);
    if ((content.length > byteThreshold || tokens > tokenThreshold) && S.focusaAvailable) {
      const handle = await focusaFetch("/ecs/store", {
        method: "POST",
        body: JSON.stringify({
          kind: "text", label: `${toolName}-output-${Date.now()}`,
          content: content.slice(0, 32_000), surface: "pi", turn_id: `pi-turn-${S.turnCount}`,
        }),
      });
      if (handle?.id) {
        // §33.3: REPLACE content with handle reference
        // §7.4: Also cache locally so handles resolve even if Focusa is temporarily down
        storeEcsArtifact("text", content);
        return {
          content: [{
            type: "text",
            text: `[HANDLE:text:${handle.id} "${toolName} output" (${content.length} bytes, ~${tokens} tokens)]\nUse /focusa-rehydrate ${handle.id} to retrieve full content.\n\n` +
                  content.slice(0, 1000) + (content.length > 1000 ? "\n...[truncated, full content in ECS]" : ""),
          }],
        };
      }
    }

    // §7.4 + §33.3: If Focusa unavailable but content still exceeds threshold,
    // store locally so the handle resolves without hitting Focusa.
    if (!S.focusaAvailable && (content.length > byteThreshold || tokens > tokenThreshold)) {
      const localId = storeEcsArtifact("text", content);
      return {
        content: [{
          type: "text",
          text: `[HANDLE:text:${localId} "${toolName} output" (${content.length} bytes, ~${tokens} tokens)]\nFocusa offline — content cached locally. Use /focusa-rehydrate ${localId} when available.\n\n` +
                content.slice(0, 500) + (content.length > 500 ? "\n...[truncated]" : ""),
        }],
      };
    }

    // §34.2D: File churn tracking
    if (toolName === "edit" || toolName === "write") {
      const path = ev.params?.path || ev.input?.path || "";
      if (path) {
        S.fileEditCounts[path] = (S.fileEditCounts[path] || 0) + 1;
        if (S.fileEditCounts[path] >= 5 && S.focusaAvailable) {
          focusaPost("/focus-gate/ingest-signal", {
            signal_type: "file_churn", surface: "pi",
            payload: { path, count: S.fileEditCounts[path] },
          });
        }
      }
    }
  });

  // ── tool_call (§33.4 batched usage) ───────────────────────────────────────
  pi.on("tool_call", async (event, _ctx) => {
    S.toolUsageBatch.push((event as any).toolName || (event as any).name || "");
  });
}
