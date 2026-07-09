import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { S, focusaPost, getFocusaAvailable, getTurnCount, getActiveWorkpointPacket } from "./state.js";

const MAX_RECORDS = 80;
const MAX_TEXT = 500;

function nowIso(): string {
  return new Date().toISOString();
}

function boundText(value: unknown, max = MAX_TEXT): string {
  const text = String(value ?? "");
  return text.length > max ? `${text.slice(0, max)}…` : text;
}

function safeJsonSize(value: unknown): number {
  try {
    return JSON.stringify(value ?? null).length;
  } catch {
    return 0;
  }
}

function simpleHash(value: string): string {
  let h = 2166136261;
  for (let i = 0; i < value.length; i++) {
    h ^= value.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return `fnv1a:${(h >>> 0).toString(16).padStart(8, "0")}`;
}

function estimateTokensFromChars(chars: number): number {
  return Math.ceil(chars / 4);
}

function recordHookTelemetry(record: Record<string, unknown>): void {
  const entry = { ts: nowIso(), ...record };
  S.spec92HookTelemetry.push(entry);
  if (S.spec92HookTelemetry.length > MAX_RECORDS)
    S.spec92HookTelemetry.splice(0, S.spec92HookTelemetry.length - MAX_RECORDS);
}

function recordTokenTelemetry(record: Record<string, unknown>): void {
  const entry = { ts: nowIso(), ...record };
  S.spec92TokenTelemetry.push(entry);
  if (S.spec92TokenTelemetry.length > MAX_RECORDS)
    S.spec92TokenTelemetry.splice(0, S.spec92TokenTelemetry.length - MAX_RECORDS);
}

function bestEffortTelemetry(kind: string, payload: Record<string, unknown>): void {
  if (!getFocusaAvailable()) return;
  focusaPost("/telemetry/event", {
    event_type: kind,
    source: "pi-extension-spec92",
    payload,
  });
}

function messageId(message: any): string {
  return String(message?.id || message?.messageId || message?.uuid || "unknown");
}

function messageSummary(message: any): Record<string, unknown> {
  const size = safeJsonSize(message);
  return {
    message_id: messageId(message),
    role: message?.role || message?.type || "unknown",
    size_bytes: size,
    token_estimate: estimateTokensFromChars(size),
    has_tool_calls: JSON.stringify(message ?? {}).includes("toolCall"),
  };
}

function payloadSummary(payload: any): Record<string, unknown> {
  const text = JSON.stringify(payload ?? {});
  const size = text.length;
  const tokenEstimate = estimateTokensFromChars(size);
  const messageCount = Array.isArray(payload?.messages) ? payload.messages.length : 0;
  const toolSchemaBytes = safeJsonSize(payload?.tools || payload?.tool_choice || payload?.toolConfig);
  const budgetClass =
    tokenEstimate > 120_000
      ? "critical"
      : tokenEstimate > 80_000
        ? "high"
        : tokenEstimate > 40_000
          ? "watch"
          : "ok";
  return {
    payload_hash: simpleHash(text),
    prefix_hash: simpleHash(text.slice(0, 12_000)),
    size_bytes: size,
    input_token_estimate: tokenEstimate,
    message_count: messageCount,
    tool_schema_token_estimate: estimateTokensFromChars(toolSchemaBytes),
    budget_class: budgetClass,
    cache_eligible: size > 0,
  };
}

function skillPaths(): string[] {
  const homeSkills = process.env.PI_SKILLS_DIR || (process.env.HOME ? `${process.env.HOME}/.pi/skills` : "");
  return [`${process.cwd()}/.pi/skills`, `${process.cwd()}/apps/pi-extension/skills`, homeSkills].filter(
    Boolean
  );
}

export function registerPolishHooks(pi: ExtensionAPI) {
  const hookApi = pi as any;
  hookApi.on("resources_discover", async (_event: any, _ctx: any) => {
    const paths = Array.from(new Set(skillPaths()));
    recordHookTelemetry({ hook: "resources_discover", skill_paths: paths });
    return { skillPaths: paths };
  });

  hookApi.on("agent_start", async (event: any, _ctx: any) => {
    const record = {
      hook: "agent_start",
      event_keys: Object.keys(event || {}).slice(0, 20),
      workpoint_id: (() => {
        const wp = getActiveWorkpointPacket();
        return wp?.workpoint_id || wp?.id || null;
      })(),
      current_ask: boundText(S.currentAsk?.text || ""),
    };
    recordHookTelemetry(record);
    bestEffortTelemetry("spec92.agent_start", record);
  });

  hookApi.on("message_start", async (event: any, _ctx: any) => {
    recordHookTelemetry({ hook: "message_start", ...messageSummary(event?.message || event) });
  });

  hookApi.on("message_end", async (event: any, _ctx: any) => {
    const record = { hook: "message_end", ...messageSummary(event?.message || event) };
    recordHookTelemetry(record);
    bestEffortTelemetry("spec92.message_end", record);
  });

  hookApi.on("before_provider_request", async (event: any, _ctx: any) => {
    const summary = payloadSummary(event?.payload || event?.request || event);
    const record: any = {
      hook: "before_provider_request",
      turn_id: `pi-turn-${getTurnCount()}`,
      provider: event?.provider || event?.model?.provider || "unknown",
      model: event?.model?.id || event?.model || "unknown",
      ...summary,
    };
    recordTokenTelemetry(record);
    recordHookTelemetry(record);
    if (getFocusaAvailable()) {
      focusaPost("/telemetry/token-budget", record);
      focusaPost("/telemetry/cache-metadata", {
        hook: record.hook,
        provider: record.provider,
        model: record.model,
        cache_key: record.repeated_prefix_hash,
        payload_hash: record.payload_hash,
        size_bytes: record.size_bytes,
        input_token_estimate: record.input_token_estimate,
        cache_eligible: record.cache_eligible,
      });
    }
    bestEffortTelemetry("spec92.before_provider_request", record);
    return undefined;
  });

  hookApi.on("after_provider_response", async (event: any, _ctx: any) => {
    const record = {
      hook: "after_provider_response",
      status: event?.status || event?.response?.status || "unknown",
      header_keys: event?.headers ? Object.keys(event.headers).slice(0, 12) : [],
      size_bytes: safeJsonSize(event?.response || event),
    };
    recordHookTelemetry(record);
    bestEffortTelemetry("spec92.after_provider_response", record);
  });

  hookApi.on("tool_execution_start", async (event: any, _ctx: any) => {
    const record = {
      hook: "tool_execution_start",
      tool_call_id: event?.toolCallId || event?.id || "unknown",
      tool_name: event?.toolName || event?.name || "unknown",
      args_size_bytes: safeJsonSize(event?.args),
    };
    S.spec92ToolStartTimes[String(record.tool_call_id)] = Date.now();
    recordHookTelemetry(record);
  });

  hookApi.on("tool_execution_update", async (event: any, _ctx: any) => {
    recordHookTelemetry({
      hook: "tool_execution_update",
      tool_call_id: event?.toolCallId || event?.id || "unknown",
      tool_name: event?.toolName || event?.name || "unknown",
      partial_size_bytes: safeJsonSize(event?.partialResult || event?.update || event),
    });
  });

  hookApi.on("tool_execution_end", async (event: any, _ctx: any) => {
    const id = String(event?.toolCallId || event?.id || "unknown");
    const started = S.spec92ToolStartTimes[id];
    if (started) delete S.spec92ToolStartTimes[id];
    const toolName = (event?.toolName || event?.name || "unknown").toLowerCase();
    const record = {
      hook: "tool_execution_end",
      tool_call_id: id,
      tool_name: toolName,
      duration_ms: started ? Date.now() - started : null,
      result_size_bytes: safeJsonSize(event?.result || event),
      status: event?.status || "completed",
    };
    recordHookTelemetry(record);
    bestEffortTelemetry("spec92.tool_execution_end", record);
    // FOCUSA_FIX-tgij: shell-tool reminder — when the agent uses a shell-like
    // tool that could touch the Focusa daemon, emit a visible reminder to
    // prefer focusa_* tools for governed interactions.
    // FOCUSA_FIX-a52s: frequency gate — limit reminders to once per turn
    // (per-turn) and at most once per 30 seconds (per-minute equivalent).
    const SHELL_TOOLS = ["bash", "sh", "fish", "zsh", "csh", "dash"];
    if (SHELL_TOOLS.includes(toolName) && getFocusaAvailable()) {
      const now = Date.now();
      const lastReminder = S.lastShellReminderAt || 0;
      const turnCount = getTurnCount();
      const lastReminderTurn = S.lastShellReminderTurn || 0;
      // Gate: per-turn (same turn = skip) + per-minute (30s debounce)
      if (turnCount !== lastReminderTurn && (now - lastReminder) > 30000) {
        S.lastShellReminderAt = now;
        S.lastShellReminderTurn = turnCount;
        const reminder = {
          customType: "focusa_agent_prompt",
          content: "For Focusa daemon/state interactions, prefer focusa_* Pi tools over shell/bash — they handle scope, authority, recovery, and evidence automatically.",
          display: true,
        };
        try {
          S.pi?.sendMessage(reminder);
        } catch {
          /* best-effort */
        }
      }
    }
  });

  hookApi.on("session_tree", async (event: any, _ctx: any) => {
    const record = {
      hook: "session_tree",
      new_leaf_id: event?.newLeafId || null,
      old_leaf_id: event?.oldLeafId || null,
      recommendation: "Run focusa_workpoint_resume if target branch changes active mission or next action.",
    };
    recordHookTelemetry(record);
    bestEffortTelemetry("spec92.session_tree", record);
  });
}
