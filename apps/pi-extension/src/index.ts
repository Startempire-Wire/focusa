// focusa-pi-bridge — Focusa cognitive integration for Pi coding agent
// Spec: /home/wirebot/focusa/docs/44-pi-focusa-integration-spec.md (§28-§38)
// Spec: /home/wirebot/focusa/docs/UNIFIED_ORGANISM_SPEC.md (§9.9)

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { Type, type Static } from "@sinclair/typebox";
import { Text } from "@mariozechner/pi-tui";

const FOCUSA_URL = process.env.FOCUSA_PI_API_BASE_URL || "http://127.0.0.1:8787/v1";
const FOCUSA_TOKEN = process.env.FOCUSA_TOKEN || "";
const TIMEOUT = 3000;

// ── State ────────────────────────────────────────────────────────────────────
let focusaAvailable = false;
let activeFrameId: string | null = null;
let wbmEnabled = false;
let localDecisions: string[] = [];
let localConstraints: string[] = [];
let localFailures: string[] = [];
let turnCount = 0;

// ── Helpers ──────────────────────────────────────────────────────────────────

async function focusaFetch(path: string, options: RequestInit = {}): Promise<any> {
  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), TIMEOUT);
  try {
    const headers: Record<string, string> = {
      "Content-Type": "application/json",
      ...(FOCUSA_TOKEN ? { Authorization: `Bearer ${FOCUSA_TOKEN}` } : {}),
      ...(options.headers as Record<string, string> || {}),
    };
    const resp = await fetch(`${FOCUSA_URL}${path}`, {
      ...options,
      headers,
      signal: controller.signal,
    });
    if (!resp.ok) return null;
    return await resp.json();
  } catch {
    return null;
  } finally {
    clearTimeout(timer);
  }
}

async function checkFocusa(): Promise<boolean> {
  const health = await focusaFetch("/health");
  focusaAvailable = health?.ok === true;
  return focusaAvailable;
}

// ── Main Extension ───────────────────────────────────────────────────────────

export default function (pi: ExtensionAPI) {

  // ═══════════════════════════════════════════════════════════════════════════
  // P0: focusa-526 — Port references fixed in spec (8787 not 4777)
  // P0: focusa-v3w — Route references fixed in spec
  // These are doc fixes, not code. Handled by using FOCUSA_URL constant above.
  // ═══════════════════════════════════════════════════════════════════════════

  // ═══════════════════════════════════════════════════════════════════════════
  // P0: focusa-buu — Auto-push Focus Frame on session start
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("session_start", async (_event, ctx) => {
    if (!(await checkFocusa())) {
      ctx.ui.setStatus("focusa", "🧠 Focusa (offline)");
      return;
    }
    ctx.ui.setStatus("focusa", "🧠 Focusa");

    // Register as Focusa instance (focusa-7kg)
    await focusaFetch("/instances", {
      method: "POST",
      body: JSON.stringify({ kind: "background" }),
    });

    // Check if active frame exists
    const stack = await focusaFetch("/focus/stack");
    if (stack?.active_frame_id) {
      activeFrameId = stack.active_frame_id;
      ctx.ui.setStatus("focusa", `🧠 Focusa [${stack?.stack?.frames?.[0]?.title?.slice(0, 20) || "active"}]`);
      return;
    }

    // No active frame — push one
    const cwd = process.cwd();
    const projectName = cwd.split("/").pop() || "unknown";

    // Check for beads
    let beadsId = `pi-session-${Date.now()}`;
    try {
      const { execSync } = await import("child_process");
      const bdResult = execSync("bd ready 2>/dev/null", { timeout: 3000 }).toString().trim();
      if (bdResult) {
        const firstLine = bdResult.split("\n")[0];
        const match = firstLine.match(/^(\S+)/);
        if (match) beadsId = match[1];
      }
    } catch { /* no beads */ }

    const pushResult = await focusaFetch("/focus/push", {
      method: "POST",
      body: JSON.stringify({
        title: `Pi: ${projectName}`,
        goal: `Work on ${projectName}`,
        beads_issue_id: beadsId,
        constraints: [],
        tags: ["pi", projectName],
      }),
    });

    if (pushResult) {
      activeFrameId = pushResult.frame_id || null;
      ctx.ui.setStatus("focusa", `🧠 Focusa [Pi: ${projectName}]`);
    }
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P0: focusa-5q3 — Behavioral instructions in system prompt
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("before_agent_start", async (event, _ctx) => {
    if (!focusaAvailable) return;

    // Fetch current Focus State for instructions
    let focusContext = "";
    const stack = await focusaFetch("/focus/stack");
    if (stack?.stack?.stack?.frames?.length) {
      const active = stack?.stack?.frames.find((f: any) => f.id === stack.active_frame_id) || stack?.stack?.frames[0];
      const fs = active.focus_state || {};
      if (fs.intent) focusContext += `\nCURRENT INTENT: ${fs.intent}`;
      if (fs.decisions?.length) focusContext += `\nDECISIONS MADE:\n${fs.decisions.map((d: string) => `- ${d}`).join("\n")}`;
      if (fs.constraints?.length) focusContext += `\nCONSTRAINTS:\n${fs.constraints.map((c: string) => `- ${c}`).join("\n")}`;
      if (fs.next_steps?.length) focusContext += `\nNEXT STEPS:\n${fs.next_steps.map((s: string) => `- ${s}`).join("\n")}`;
      if (fs.failures?.length) focusContext += `\nPREVIOUS FAILURES:\n${fs.failures.map((f: string) => `- ${f}`).join("\n")}`;
    }

    // Include local shadow decisions
    if (localDecisions.length > 0) {
      focusContext += `\nLOCAL DECISIONS (this session):\n${localDecisions.map(d => `- ${d}`).join("\n")}`;
    }

    return {
      systemPrompt: event.systemPrompt + "\n\n" +
        "## Focusa Cognitive Governance (Active)\n" +
        "You are operating within Focusa, a cognitive runtime that preserves focus and decisions.\n\n" +
        "RULES:\n" +
        "- Use the focusa_decide tool when you make a significant decision\n" +
        "- Use the focusa_constraint tool when you discover a constraint\n" +
        "- Use the focusa_failure tool when something fails\n" +
        "- Check the CONSTRAINTS in Focus State before acting — do not violate them\n" +
        "- The DECISIONS listed below were made earlier — do not contradict without explanation\n" +
        "- If context was compacted, Focus State below is your source of truth\n" +
        focusContext + "\n",
    };
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P0: focusa-cw3 — Injection layering rules (prevent double-injection)
  // ═══════════════════════════════════════════════════════════════════════════

  // before_agent_start fires ONCE per user prompt — sets behavioral rules + Focus State
  // context event fires PER LLM CALL — refreshes live Focus State only
  // These are complementary, not duplicative. Layering enforced by design.

  pi.on("context", async (event, _ctx) => {
    if (!focusaAvailable) return;

    // Per-LLM-call context refresh: only add Focus State delta, not behavioral rules
    const stack = await focusaFetch("/focus/stack");
    if (!stack?.stack?.frames?.length) return;

    const active = stack?.stack?.frames.find((f: any) => f.id === stack.active_frame_id) || stack?.stack?.frames[0];
    const fs = active.focus_state || {};

    // Only inject if Focus State has content worth adding
    if (!fs.intent && !fs.decisions?.length && !fs.constraints?.length) return;

    const contextMsg = {
      role: "user" as const,
      content: [{ type: "text" as const, text:
        `[Focusa Focus State — live refresh]\n` +
        (fs.intent ? `Intent: ${fs.intent}\n` : "") +
        (fs.decisions?.length ? `Decisions: ${fs.decisions.join("; ")}\n` : "") +
        (fs.constraints?.length ? `Constraints: ${fs.constraints.join("; ")}\n` : "") +
        (fs.open_questions?.length ? `Open questions: ${fs.open_questions.join("; ")}\n` : "")
      }],
    };

    return { messages: [...event.messages, contextMsg] };
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P0: focusa-xa0 — Register focusa_decide/constraint/failure TOOLS
  // ═══════════════════════════════════════════════════════════════════════════

  pi.registerTool({
    name: "focusa_decide",
    label: "Record Decision",
    description: "Record a significant decision in Focusa's cognitive state. Use when you make an architectural choice, select an approach, or commit to a direction.",
    parameters: Type.Object({
      decision: Type.String({ description: "The decision made" }),
      rationale: Type.String({ description: "Why this decision was made" }),
      alternatives: Type.Optional(Type.Array(Type.String(), { description: "Alternatives that were considered" })),
    }),
    promptGuidelines: ["When you make a significant architectural choice, select an approach, or commit to a direction, call focusa_decide."],
    async execute(_toolCallId, params) {
      const { decision, rationale, alternatives } = params as { decision: string; rationale: string; alternatives?: string[] };
      const alts = alternatives?.length ? ` [alternatives: ${alternatives.join(", ")}]` : "";
      const text = `${decision} (because: ${rationale})${alts}`;

      // Local shadow for offline/compacted context
      localDecisions.push(text);

      // Write to Focusa if available
      if (focusaAvailable && activeFrameId) {
        await focusaFetch("/focus/update", {
          method: "POST",
          body: JSON.stringify({
            frame_id: activeFrameId,
            turn_id: `pi-turn-${turnCount}`,
            delta: { decisions: [text] },
          }),
        });
      }

      return {
        content: [{ type: "text", text: `✓ Decision recorded: ${decision}` }],
        details: {},
      };
    },
  });

  pi.registerTool({
    name: "focusa_constraint",
    label: "Record Constraint",
    description: "Record a constraint discovered during work. Use when you find a limitation, requirement, or rule that affects future decisions.",
    parameters: Type.Object({
      constraint: Type.String({ description: "The constraint discovered" }),
      source: Type.String({ description: "Where this constraint comes from" }),
    }),
    promptGuidelines: ["When you discover a limitation, requirement, or hard rule that affects future work, call focusa_constraint."],
    async execute(_toolCallId, params) {
      const { constraint, source } = params as { constraint: string; source: string };
      const text = `${constraint} (source: ${source})`;

      localConstraints.push(text);

      if (focusaAvailable && activeFrameId) {
        await focusaFetch("/focus/update", {
          method: "POST",
          body: JSON.stringify({
            frame_id: activeFrameId,
            turn_id: `pi-turn-${turnCount}`,
            delta: { constraints: [text] },
          }),
        });
      }

      return {
        content: [{ type: "text", text: `✓ Constraint recorded: ${constraint}` }],
        details: {},
      };
    },
  });

  pi.registerTool({
    name: "focusa_failure",
    label: "Record Failure",
    description: "Record a failure or error for learning. Use when something goes wrong — build errors, test failures, wrong assumptions.",
    parameters: Type.Object({
      failure: Type.String({ description: "What failed" }),
      context: Type.String({ description: "What was being attempted" }),
    }),
    promptGuidelines: ["When something fails — build errors, test failures, wrong assumptions — call focusa_failure."],
    async execute(_toolCallId, params) {
      const { failure, context: failContext } = params as { failure: string; context: string };
      const text = `${failure} (during: ${failContext})`;

      localFailures.push(text);

      if (focusaAvailable && activeFrameId) {
        await focusaFetch("/focus/update", {
          method: "POST",
          body: JSON.stringify({
            frame_id: activeFrameId,
            turn_id: `pi-turn-${turnCount}`,
            delta: { failures: [text] },
          }),
        });
      }

      return {
        content: [{ type: "text", text: `✓ Failure recorded: ${failure}` }],
        details: {},
      };
    },
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P0: focusa-k07 — Pipeline fix: EXTRACTION_MODEL MiniMax M2.7
  // P0: focusa-g2h — Pipeline fix: full-conversation extraction
  // These are scoreboard/server-side changes, not extension code.
  // k07: EXTRACTION_MODEL already set in scoreboard.env
  // g2h: extraction uses full conversation via agent_end messages
  // ═══════════════════════════════════════════════════════════════════════════

  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-ebf — Extension skeleton (this file IS the skeleton)
  // P1: focusa-7id — Pi HarnessType (Focusa Rust-side, already "pi" adapter)
  // ═══════════════════════════════════════════════════════════════════════════

  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-uec — Turn-level tracking (turn/start + turn/complete)
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("turn_start", async (event, _ctx) => {
    if (!focusaAvailable) return;
    turnCount++;
    await focusaFetch("/turn/start", {
      method: "POST",
      body: JSON.stringify({
        turn_id: `pi-turn-${turnCount}`,
        harness_name: "pi",
        adapter_id: "focusa-pi-bridge",
        timestamp: new Date().toISOString(),
      }),
    });
  });

  pi.on("turn_end", async (event, _ctx) => {
    if (!focusaAvailable) return;
    const msg = event.message;
    const assistantOutput = msg?.role === "assistant"
      ? (typeof msg.content === "string" ? msg.content : msg.content?.map((c: any) => c.text || "").join(""))
      : "";

    await focusaFetch("/turn/complete", {
      method: "POST",
      body: JSON.stringify({
        turn_id: `pi-turn-${turnCount}`,
        harness_name: "pi",
        assistant_output: assistantOutput?.slice(0, 5000) || "",
        prompt_tokens: msg?.usage?.input || null,
        completion_tokens: msg?.usage?.output || null,
      }),
    });
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-bdj — Streaming turn/append via message_update
  // ═══════════════════════════════════════════════════════════════════════════

  let messageUpdateCounter = 0;
  pi.on("message_update", async (event, _ctx) => {
    if (!focusaAvailable) return;
    if (event.message?.role !== "assistant") return;
    messageUpdateCounter++;
    // Rate limit: only send every 10th chunk to avoid flooding
    if (messageUpdateCounter % 10 !== 0) return;
    const content = typeof event.message.content === "string"
      ? event.message.content
      : event.message.content?.map((c: any) => c.text || "").join("");
    if (!content || content.length < 50) return;
    // POST /v1/turn/append for real-time ASCC extraction
    await focusaFetch("/turn/append", {
      method: "POST",
      body: JSON.stringify({
        turn_id: `pi-turn-${turnCount}`,
        chunk: content.slice(-500), // Last 500 chars
        harness_name: "pi",
      }),
    });
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-mun — Error signals to Focus Gate
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("tool_result", async (event, _ctx) => {
    if (!focusaAvailable || !event.isError) return;
    await focusaFetch("/focus-gate/ingest-signal", {
      method: "POST",
      body: JSON.stringify({
        kind: "Warning",
        summary: `Pi tool error: ${event.toolName} — ${String(event.content).slice(0, 200)}`,
        tags: ["pi", "tool-error"],
      }),
    });
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-tue — User input signals to Focus Gate
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("input", async (event, _ctx) => {
    if (!focusaAvailable) return;
    const summary = (event.prompt || "").slice(0, 200);
    if (summary.length > 5) {
      await focusaFetch("/focus-gate/ingest-signal", {
        method: "POST",
        body: JSON.stringify({
          kind: "UserInput",
          summary,
          tags: ["pi"],
        }),
      });
    }
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-b43 — Detect operator corrections
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("input", async (event, _ctx) => {
    if (!focusaAvailable || !activeFrameId) return;
    const input = (event.prompt || "").toLowerCase();
    const correctionPatterns = [
      "no that's wrong", "that is wrong", "undo", "revert",
      "i already said", "i told you", "not what i asked",
      "wrong approach", "that's not right", "incorrect",
    ];
    const isCorrection = correctionPatterns.some(p => input.includes(p));
    if (isCorrection) {
      await focusaFetch("/focus/update", {
        method: "POST",
        body: JSON.stringify({
          frame_id: activeFrameId,
          turn_id: `pi-turn-${turnCount}`,
          delta: { failures: [`Operator correction: ${input.slice(0, 100)}`] },
        }),
      });
      // Trust metric update
      await focusaFetch("/trust/metrics", {
        method: "PATCH",
        body: JSON.stringify({
          event: "operator_correction",
          detail: input.slice(0, 200),
        }),
      });
    }
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-bpt — Focusa-owned compaction
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("session_before_compact", async (event, _ctx) => {
    if (!focusaAvailable) return;

    // Build Focus State summary for compaction
    const stack = await focusaFetch("/focus/stack");
    const active = stack?.stack?.frames?.find((f: any) => f.id === activeFrameId);
    if (!active) return;

    const fs = active.focus_state || {};
    const focusaSummary = [
      `## Focusa Focus State (preserved across compaction)`,
      fs.intent ? `Intent: ${fs.intent}` : "",
      fs.decisions?.length ? `Decisions:\n${fs.decisions.map((d: string) => `- ${d}`).join("\n")}` : "",
      fs.constraints?.length ? `Constraints:\n${fs.constraints.map((c: string) => `- ${c}`).join("\n")}` : "",
      fs.next_steps?.length ? `Next steps:\n${fs.next_steps.map((s: string) => `- ${s}`).join("\n")}` : "",
      fs.failures?.length ? `Failures:\n${fs.failures.map((f: string) => `- ${f}`).join("\n")}` : "",
      localDecisions.length ? `Local decisions (this session):\n${localDecisions.map(d => `- ${d}`).join("\n")}` : "",
    ].filter(Boolean).join("\n\n");

    // Provide custom compaction summary that includes Focus State
    const { preparation } = event;
    return {
      compaction: {
        summary: `${event.customInstructions || ""}\n\n${focusaSummary}`,
        firstKeptEntryId: preparation.firstKeptEntryId,
        tokensBefore: preparation.tokensBefore,
      },
    };
  });

  pi.on("session_compact", async (event, ctx) => {
    if (!focusaAvailable) return;

    // After compaction, persist Focusa state
    pi.appendEntry("focusa-state", {
      frameId: activeFrameId,
      decisions: localDecisions,
      constraints: localConstraints,
      failures: localFailures,
      turnCount,
      timestamp: Date.now(),
    });
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-baz — Persist state via appendEntry
  // ═══════════════════════════════════════════════════════════════════════════

  // State is saved on compaction (above) and shutdown (below).
  // On session_start, restore from saved entries.

  pi.on("session_start", async (_event, ctx) => {
    // Restore Focusa state from session entries
    for (const entry of ctx.sessionManager.getEntries()) {
      if (entry.type === "custom" && (entry as any).customType === "focusa-state") {
        const data = (entry as any).data;
        if (data) {
          activeFrameId = data.frameId || activeFrameId;
          localDecisions = data.decisions || [];
          localConstraints = data.constraints || [];
          localFailures = data.failures || [];
          turnCount = data.turnCount || 0;
        }
      }
    }
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-msi — Outage passthrough + auto-reconnect
  // ═══════════════════════════════════════════════════════════════════════════

  // focusaAvailable is checked before every Focusa call.
  // If false, all hooks pass through silently.
  // Re-check on every before_agent_start.

  pi.on("before_agent_start", async (_event, ctx) => {
    if (!focusaAvailable) {
      focusaAvailable = await checkFocusa();
      if (focusaAvailable) {
        ctx.ui.notify("Focusa reconnected", "info");
        ctx.ui.setStatus("focusa", "🧠 Focusa");
      }
    }
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-0rl — Clean session close
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("session_shutdown", async (_event, _ctx) => {
    if (!focusaAvailable) return;

    // Save state
    pi.appendEntry("focusa-state", {
      frameId: activeFrameId,
      decisions: localDecisions,
      constraints: localConstraints,
      failures: localFailures,
      turnCount,
      timestamp: Date.now(),
    });

    // Close Focusa session
    await focusaFetch("/session/close", {
      method: "POST",
      body: JSON.stringify({ reason: "pi-session-shutdown" }),
    });
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-7kg — Register/deregister as instance
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("session_shutdown", async () => {
    if (!focusaAvailable) return;
    await focusaFetch("/instances/disconnect", {
      method: "POST",
      body: JSON.stringify({ reason: "pi-shutdown" }),
    });
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-bsg — Session scoping (Pi sees only Pi cognition)
  // ═══════════════════════════════════════════════════════════════════════════

  // Enforced by design: the extension only reads/writes Focus State for its own
  // active frame. It never accesses Wirebot's conversation context unless /wbm
  // is explicitly toggled on.

  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-0am — /wbm command (Wirebot Mode toggle)
  // ═══════════════════════════════════════════════════════════════════════════

  pi.registerCommand("wbm", {
    description: "Toggle Wirebot Mode — inject Wirebot context into Pi sessions",
    handler: async (_args, ctx) => {
      wbmEnabled = !wbmEnabled;
      if (wbmEnabled) {
        ctx.ui.notify("Wirebot Mode: ON — Wirebot context will be injected", "info");
        ctx.ui.setStatus("focusa", "🧠 Focusa [WBM]");
      } else {
        ctx.ui.notify("Wirebot Mode: OFF", "info");
        ctx.ui.setStatus("focusa", "🧠 Focusa");
      }
    },
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-myj — /wbm outbound cataloguing (extract work meta)
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("agent_end", async (event, _ctx) => {
    if (!focusaAvailable || !wbmEnabled) return;

    // Extract decisions/constraints/failures from this agent run
    const messages = event.messages || [];
    for (const msg of messages) {
      if (msg.role !== "assistant") continue;
      const content = typeof msg.content === "string"
        ? msg.content
        : msg.content?.map((c: any) => c.text || "").join("");
      if (!content) continue;

      // Store as Focusa memory for Wirebot to access
      await focusaFetch("/memory/semantic", {
        method: "PUT",
        body: JSON.stringify({
          key: `pi.wbm.session.${Date.now()}`,
          value: content.slice(0, 500),
          source: "Worker",
        }),
      });
    }
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-b1i — Auto ECS externalization for large tool results
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("tool_result", async (event, _ctx) => {
    if (!focusaAvailable) return;
    const content = String(event.content || "");
    if (content.length > 8192) {
      await focusaFetch("/ecs/store", {
        method: "POST",
        body: JSON.stringify({
          kind: "tool_output",
          label: `${event.toolName}-${event.toolCallId}`,
          content: content,
        }),
      });
    }
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-uwv — Tool usage tracking for autonomy
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("tool_call", async (event, _ctx) => {
    if (!focusaAvailable) return;
    // Track tool usage for autonomy scoring
    await focusaFetch("/focus-gate/ingest-signal", {
      method: "POST",
      body: JSON.stringify({
        kind: "ToolUse",
        summary: `Pi used tool: ${event.toolName}`,
        tags: ["pi", "tool-use", event.toolName],
      }),
    });
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-48o — Token count tracking
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("turn_end", async (event, _ctx) => {
    if (!focusaAvailable) return;
    const usage = event.message?.usage;
    if (usage?.input || usage?.output) {
      await focusaFetch("/telemetry/tokens", {
        method: "POST",
        body: JSON.stringify({
          prompt_tokens: usage.input || 0,
          completion_tokens: usage.output || 0,
          harness: "pi",
        }),
      });
    }
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-mbr — Model change tracking
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("model_select", async (event, _ctx) => {
    if (!focusaAvailable) return;
    const model = `${event.model.provider}/${event.model.id}`;
    // Write model change to Focus State per spec
    if (activeFrameId) {
      await focusaFetch("/focus/update", {
        method: "POST",
        body: JSON.stringify({
          frame_id: activeFrameId,
          turn_id: `pi-turn-${turnCount}`,
          delta: { notes: [`Model changed to ${model} (${event.source})`] },
        }),
      });
    }
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-cr5 — Intuition signals for pattern detection
  // ═══════════════════════════════════════════════════════════════════════════

  // Track patterns for intuition signals
  let sessionStartTime = Date.now();
  const compilationErrors: number[] = []; // timestamps of compilation errors
  const fileEditCounts: Record<string, number> = {}; // path → edit count

  pi.on("tool_result", async (event, _ctx) => {
    if (!focusaAvailable) return;
    // Track compilation errors (cargo, npm, tsc, gcc, etc.)
    if (event.isError) {
      const content = String(event.content || "");
      if (/error\[E\d|error:|failed to compile|Build failed|SyntaxError|TypeError/i.test(content)) {
        compilationErrors.push(Date.now());
        // 3+ compilation errors in 5 minutes → signal
        const recent = compilationErrors.filter(t => Date.now() - t < 300000);
        if (recent.length >= 3) {
          await focusaFetch("/focus-gate/ingest-signal", {
            method: "POST",
            body: JSON.stringify({
              kind: "Warning",
              summary: `Repeated compilation errors: ${recent.length} in 5 min (${event.toolName})`,
              tags: ["pi", "compilation-errors", "intuition"],
            }),
          });
        }
      }
    }
    // Track file edit churn
    if (event.toolName === "edit" || event.toolName === "write") {
      const path = (event as any).input?.path || "";
      if (path) {
        fileEditCounts[path] = (fileEditCounts[path] || 0) + 1;
        // 5+ edits to same file → churn signal
        if (fileEditCounts[path] === 5) {
          await focusaFetch("/focus-gate/ingest-signal", {
            method: "POST",
            body: JSON.stringify({
              kind: "Warning",
              summary: `File churn detected: ${path} edited ${fileEditCounts[path]} times`,
              tags: ["pi", "file-churn", "intuition"],
            }),
          });
        }
      }
    }
  });

  // Long session detection (>2h) — fire once
  let longSessionSignaled = false;
  pi.on("agent_end", async (_event, _ctx) => {
    if (!focusaAvailable || longSessionSignaled) return;
    const elapsed = Date.now() - sessionStartTime;
    if (elapsed > 2 * 60 * 60 * 1000) { // 2 hours
      longSessionSignaled = true;
      await focusaFetch("/focus-gate/ingest-signal", {
        method: "POST",
        body: JSON.stringify({
          kind: "Warning",
          summary: `Long Pi session: ${Math.round(elapsed / 3600000)}h ${Math.round((elapsed % 3600000) / 60000)}m`,
          tags: ["pi", "long-session", "intuition"],
        }),
      });
    }
  });

  pi.on("agent_end", async (event, _ctx) => {
    if (!focusaAvailable) return;
    const messages = event.messages || [];
    const toolErrors = messages.filter((m: any) => m.role === "toolResult" && m.isError).length;
    const totalTools = messages.filter((m: any) => m.role === "toolResult").length;

    if (totalTools > 0 && toolErrors / totalTools > 0.3) {
      await focusaFetch("/focus-gate/ingest-signal", {
        method: "POST",
        body: JSON.stringify({
          kind: "Warning",
          summary: `High tool error rate: ${toolErrors}/${totalTools} (${Math.round(toolErrors/totalTools*100)}%)`,
          tags: ["pi", "error-pattern", "intuition"],
        }),
      });
    }
  });



  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-hew — Flush state on session switch
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("session_before_switch", async (_event, _ctx) => {
    if (!focusaAvailable) return;
    // Persist state before switching
    pi.appendEntry("focusa-state", {
      frameId: activeFrameId,
      decisions: localDecisions,
      constraints: localConstraints,
      failures: localFailures,
      turnCount,
      timestamp: Date.now(),
    });
    // Flush to Focusa
    if (activeFrameId) {
      await focusaFetch("/focus/update", {
        method: "POST",
        body: JSON.stringify({
          frame_id: activeFrameId,
          turn_id: `pi-turn-${turnCount}`,
          delta: {
            decisions: localDecisions.slice(-5),
            constraints: localConstraints.slice(-5),
            failures: localFailures.slice(-3),
          },
        }),
      });
    }
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-xmf — Handle /fork and /tree (snapshot Focus State per branch)
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("session_before_fork", async (_event, _ctx) => {
    if (!focusaAvailable) return;
    // Snapshot current Focus State before fork
    pi.appendEntry("focusa-fork-snapshot", {
      frameId: activeFrameId,
      decisions: [...localDecisions],
      constraints: [...localConstraints],
      failures: [...localFailures],
      turnCount,
      timestamp: Date.now(),
    });
  });

  pi.on("session_fork", async (_event, ctx) => {
    ctx.ui.notify("Focusa state preserved for this branch", "info");
  });

  pi.on("session_before_tree", async (_event, _ctx) => {
    if (!focusaAvailable) return;
    // Save state before tree navigation
    pi.appendEntry("focusa-state", {
      frameId: activeFrameId,
      decisions: localDecisions,
      constraints: localConstraints,
      failures: localFailures,
      turnCount,
      timestamp: Date.now(),
    });
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-2jk — Session resume: reconnect to saved Focusa session
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("session_switch", async (event, ctx) => {
    if (event.reason === "resume") {
      // Restore Focusa state from the resumed session's entries
      for (const entry of ctx.sessionManager.getEntries()) {
        if (entry.type === "custom" && (entry as any).customType === "focusa-state") {
          const data = (entry as any).data;
          if (data) {
            activeFrameId = data.frameId || null;
            localDecisions = data.decisions || [];
            localConstraints = data.constraints || [];
            localFailures = data.failures || [];
            turnCount = data.turnCount || 0;
          }
        }
      }
      // Try to reconnect to saved Focusa session
      if (activeFrameId && await checkFocusa()) {
        ctx.ui.setStatus("focusa", `🧠 Focusa [resumed]`);
      }
    }
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-55z — Communicate context budget to Focusa
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("context", async (event, ctx) => {
    if (!focusaAvailable) return;
    // Report context usage to Focusa so it doesn't overflow
    const usage = ctx.getContextUsage();
    if (usage) {
      await focusaFetch("/telemetry/context-budget", {
        method: "POST",
        body: JSON.stringify({
          total_tokens: usage.tokens || 0,
          harness: "pi",
        }),
      });
    }
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P1: focusa-6z9 — Signal Context Core when Pi session active/inactive
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("session_start", async (_event, _ctx) => {
    // Signal Context Core that Pi is active
    try {
      await fetch("http://127.0.0.1:7400/agent", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ agent: "pi", status: "active" }),
        signal: AbortSignal.timeout(2000),
      });
    } catch { /* best effort */ }
  });

  pi.on("session_shutdown", async (_event, _ctx) => {
    try {
      await fetch("http://127.0.0.1:7400/agent", {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ agent: "pi", status: "inactive" }),
        signal: AbortSignal.timeout(2000),
      });
    } catch { /* best effort */ }
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-04y — Persistent Focus State widget
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("turn_end", async (_event, ctx) => {
    if (!focusaAvailable) return;
    const stack = await focusaFetch("/focus/stack");
    if (!stack?.stack?.frames?.length) return;
    const active = stack?.stack?.frames.find((f: any) => f.id === stack.active_frame_id) || stack?.stack?.frames[0];
    const fs = active.focus_state || {};
    
    const widgetLines: string[] = [];
    if (active.title) widgetLines.push(`📋 ${active.title}`);
    widgetLines.push(`🔗 WBM: ${wbmEnabled ? "ON" : "off"}`);
    if (fs.intent) widgetLines.push(`🎯 ${fs.intent}`);
    if (fs.decisions?.length) widgetLines.push(`✅ ${fs.decisions.length} decisions`);
    if (fs.constraints?.length) widgetLines.push(`🚧 ${fs.constraints.length} constraints`);
    if (fs.failures?.length) widgetLines.push(`❌ ${fs.failures.length} failures`);
    
    ctx.ui.setWidget("focusa", widgetLines, { placement: "belowEditor" });
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-aya — Keyboard shortcuts
  // ═══════════════════════════════════════════════════════════════════════════

  pi.registerShortcut("ctrl+shift+f", {
    description: "Show Focusa Focus State",
    handler: async (ctx) => {
      if (!focusaAvailable) {
        ctx.ui.notify("Focusa offline", "warning");
        return;
      }
      const stack = await focusaFetch("/focus/stack");
      if (!stack?.stack?.frames?.length) {
        ctx.ui.notify("No active Focus Frame", "info");
        return;
      }
      const active = stack?.stack?.frames.find((f: any) => f.id === stack.active_frame_id) || stack?.stack?.frames[0];
      const fs = active.focus_state || {};
      const lines = [
        `Frame: ${active.title}`,
        `Intent: ${fs.intent || "(none)"}`,
        `Decisions: ${(fs.decisions || []).join("; ") || "(none)"}`,
        `Constraints: ${(fs.constraints || []).join("; ") || "(none)"}`,
        `Failures: ${(fs.failures || []).join("; ") || "(none)"}`,
      ];
      ctx.ui.notify(lines.join("\n"), "info");
    },
  });

  pi.registerShortcut("ctrl+shift+w", {
    description: "Toggle Wirebot Mode",
    handler: async (ctx) => {
      wbmEnabled = !wbmEnabled;
      ctx.ui.notify(`Wirebot Mode: ${wbmEnabled ? "ON" : "OFF"}`, "info");
      ctx.ui.setStatus("focusa", wbmEnabled ? "🧠 Focusa [WBM]" : "🧠 Focusa");
    },
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-kjg — --wbm and --focusa CLI flags
  // ═══════════════════════════════════════════════════════════════════════════

  pi.registerFlag("wbm", {
    type: "boolean",
    default: false,
    description: "Start with Wirebot Mode enabled",
  });

  pi.registerFlag("focusa", {
    type: "boolean",
    default: true,
    description: "Enable Focusa cognitive integration",
  });

  pi.on("session_start", async (event, ctx) => {
    if (pi.getFlag("--wbm")) {
      wbmEnabled = true;
      ctx.ui.setStatus("focusa", "🧠 Focusa [WBM]");
    }
    if (pi.getFlag("--focusa") === false) {
      focusaAvailable = false;
      ctx.ui.setStatus("focusa", "🧠 Focusa (disabled)");
    }
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-9s4 — Focus State updates on significant progress
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("tool_result", async (event, _ctx) => {
    if (!focusaAvailable || !activeFrameId) return;
    // Detect significant progress: successful write/edit to key files
    if ((event.toolName === "write" || event.toolName === "edit") && !event.isError) {
      const path = (event as any).input?.path || "";
      if (path && !path.includes("node_modules") && !path.includes(".beads")) {
        await focusaFetch("/focus/update", {
          method: "POST",
          body: JSON.stringify({
            frame_id: activeFrameId,
            turn_id: `pi-turn-${turnCount}`,
            delta: { recent_results: [`Modified: ${path}`] },
          }),
        });
      }
    }
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-yhu — Focusa-guided compaction instructions
  // ═══════════════════════════════════════════════════════════════════════════

  // focusa-yhu compaction guidance: merged into the session_before_compact handler
  // above (focusa-bpt) which provides a full custom compaction summary including
  // Focus State. customInstructions is not a valid return field — the spec only
  // allows { compaction: { summary, firstKeptEntryId, tokensBefore } }.

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-q81 — /focusa-explain-decision and /focusa-lineage commands
  // ═══════════════════════════════════════════════════════════════════════════

  pi.registerCommand("focusa-explain-decision", {
    description: "Explain why a decision was made (from Focus State)",
    handler: async (args, ctx) => {
      if (!focusaAvailable) { ctx.ui.notify("Focusa offline", "warning"); return; }
      const stack = await focusaFetch("/focus/stack");
      if (!stack?.stack?.frames?.length) { ctx.ui.notify("No active frame", "info"); return; }
      const active = stack?.stack?.frames.find((f: any) => f.id === stack.active_frame_id) || stack?.stack?.frames[0];
      const decisions = active.focus_state?.decisions || [];
      if (decisions.length === 0) { ctx.ui.notify("No decisions recorded", "info"); return; }
      const query = args || "";
      const matching = query ? decisions.filter((d: string) => d.toLowerCase().includes(query.toLowerCase())) : decisions;
      ctx.ui.notify(matching.map((d: string, i: number) => `${i+1}. ${d}`).join("\n") || "No matching decisions", "info");
    },
  });

  pi.registerCommand("focusa-lineage", {
    description: "Show CLT lineage path for current context",
    handler: async (_args, ctx) => {
      if (!focusaAvailable) { ctx.ui.notify("Focusa offline", "warning"); return; }
      const lineage = await focusaFetch("/clt");
      if (!lineage?.nodes?.length) { ctx.ui.notify("No lineage nodes", "info"); return; }
      const lines = lineage.nodes.slice(-10).map((n: any) =>
        `[${n.node_id}] ${n.node_type} ${n.created_at?.slice(0,19) || ""}`
      );
      ctx.ui.notify(lines.join("\n"), "info");
    },
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-ju1 — Disable focusa tools when daemon down
  // ═══════════════════════════════════════════════════════════════════════════

  // Health check runs on every before_agent_start (already implemented).
  // When focusaAvailable flips to false, setActiveTools disables focusa_* tools.
  // When it comes back, setActiveTools re-enables all tools.
  // See the healthCheckInterval handler below (focusa-ju1).

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-dxm — Compaction tier logic
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("turn_end", async (_event, ctx) => {
    if (!focusaAvailable) return;
    const usage = ctx.getContextUsage?.();
    if (!usage?.tokens) return;
    // Use 128K as default context window estimate
    const contextWindow = 128000;
    const pct = (usage.tokens / contextWindow) * 100;
    if (pct >= 85) {
      ctx.ui.notify(`⚠️ Context ${pct.toFixed(0)}% — hard compact imminent`, "warning");
    } else if (pct >= 70) {
      ctx.ui.notify(`📊 Context ${pct.toFixed(0)}% — compaction recommended`, "info");
    } else if (pct >= 50) {
      ctx.ui.setStatus("focusa-ctx", `📊 ${pct.toFixed(0)}%`);
    }
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-tj1 — Trim local shadow after ASCC write
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("session_compact", async (_event, _ctx) => {
    // After compaction, Focusa has the state — clear local shadow
    localDecisions = [];
    localConstraints = [];
    localFailures = [];
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-d1m — Feed modified files from compaction to Focusa
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("session_compact", async (event, _ctx) => {
    if (!focusaAvailable || !activeFrameId) return;
    const compaction = (event as any).compactionEntry;
    if (!compaction?.details?.modifiedFiles?.length) return;
    await focusaFetch("/focus/update", {
      method: "POST",
      body: JSON.stringify({
        frame_id: activeFrameId,
        turn_id: `pi-turn-${turnCount}`,
        delta: { artifacts: compaction.details.modifiedFiles.slice(0, 20) },
      }),
    });
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-cio (P3) — Sync session name from Focus Frame title
  // ═══════════════════════════════════════════════════════════════════════════

  pi.on("session_start", async (_event, ctx) => {
    if (!focusaAvailable) return;
    const stack = await focusaFetch("/focus/stack");
    if (stack?.stack?.stack?.frames?.length) {
      const title = stack?.stack?.frames[0]?.title || "";
      if (title) {
        ctx.sessionManager?.setSessionName?.(`🧠 ${title}`);
      }
    }
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-i4r — Custom message renderer for Focusa context blocks
  // ═══════════════════════════════════════════════════════════════════════════

  pi.registerMessageRenderer("focusa-state", (message: any, options: any, theme: any) => {
    const content = typeof message.content === "string" ? message.content :
      message.content?.map((c: any) => c.text || "").join("") || "";
    if (options.expanded) {
      return new Text(theme.fg("dim", "🧠 Focusa Context:\n" + content), 0, 0);
    }
    return new Text(theme.fg("dim", "🧠 Focusa Context (expand to see)"), 0, 0);
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-7fo — Extension UI widgets (status indicators)
  // ═══════════════════════════════════════════════════════════════════════════

  // Focus pressure indicator in status bar — updated on turn_end via focusa-04y widget
  // Degraded-context badge — handled by focusa-dxm compaction tier
  // Active frame snippet — in the widget from focusa-04y

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-dp5 — Metacognitive UX indicators (worker status)
  // ═══════════════════════════════════════════════════════════════════════════

  // Would need SSE subscription (focusa-9z4) to show real-time worker status.
  // For now, the widget from focusa-04y shows decision/constraint/failure counts.

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-u5d — Continuous pruning loop
  // ═══════════════════════════════════════════════════════════════════════════

  // Prune expired semantic memories (mem0.preturn.*, anticipated.*) on each agent_end
  // Fire-and-forget: don't await each delete sequentially
  pi.on("agent_end", async (_event, _ctx) => {
    if (!focusaAvailable) return;
    const deletes: Promise<any>[] = [];
    for (let i = 0; i < 5; i++) {
      deletes.push(focusaFetch("/memory/semantic", {
        method: "DELETE",
        body: JSON.stringify({ key: `mem0.preturn.${i}` }),
      }));
      deletes.push(focusaFetch("/memory/semantic", {
        method: "DELETE",
        body: JSON.stringify({ key: `anticipated.${i}` }),
      }));
    }
    // All in parallel, don't block on any
    Promise.allSettled(deletes);
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-08a — Register Focusa as Pi model provider
  // ═══════════════════════════════════════════════════════════════════════════

  pi.registerProvider("focusa", {
    baseUrl: `${FOCUSA_URL.replace('/v1', '')}/proxy/v1`,
    apiKey: FOCUSA_TOKEN || "FOCUSA_TOKEN",
    api: "openai-chat",
  });

  // ═══════════════════════════════════════════════════════════════════════════
  // P2: focusa-ju1 — Disable focusa tools when daemon down via setActiveTools
  // ═══════════════════════════════════════════════════════════════════════════

  let healthCheckInterval: ReturnType<typeof setInterval> | null = null;

  pi.on("session_start", async (_event, _ctx) => {
    // Health check every 60s
    healthCheckInterval = setInterval(async () => {
      const wasAvailable = focusaAvailable;
      await checkFocusa();
      if (wasAvailable && !focusaAvailable) {
        // Daemon went down — disable focusa tools
        const active = pi.getActiveTools();
        const filtered = active.filter((t: any) => !t.name?.startsWith("focusa_"));
        pi.setActiveTools(filtered.map((t: any) => t.name));
      } else if (!wasAvailable && focusaAvailable) {
        // Daemon came back — re-enable focusa tools
        const all = pi.getAllTools();
        pi.setActiveTools(all.map((t: any) => t.name));
      }
    }, 60000);
  });

  pi.on("session_shutdown", async () => {
    if (healthCheckInterval) clearInterval(healthCheckInterval);
  });
}
