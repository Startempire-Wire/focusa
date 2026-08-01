import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { otaActivationPaths } from "./ota-activation.js";
import {
  getAttachmentRuntime,
  focusaFetch,
  focusaPost,
  getFocusaAvailable,
  getTurnCount,
  getActiveWorkpointPacket,
} from "./state.js";

const MAX_RECORDS = 80;
const MAX_TEXT = 500;
let semanticSequence = 0;
const MAX_OFFLINE_SPOOL = 64;
const SPOOL_PATH = join(
  String(process.env.FOCUSA_DATA_DIR || "").trim() || join(homedir(), ".focusa"),
  "pi-semantic-spool.json"
);
const offlineSemanticSpool: Array<Record<string, unknown>> = loadSemanticSpool();

function loadSemanticSpool(): Array<Record<string, unknown>> {
  if (!existsSync(SPOOL_PATH)) return [];
  try {
    const parsed = JSON.parse(readFileSync(SPOOL_PATH, "utf8"));
    return Array.isArray(parsed) ? parsed.slice(-MAX_OFFLINE_SPOOL) : [];
  } catch {
    return [];
  }
}

function persistSemanticSpool(): void {
  try {
    mkdirSync(dirname(SPOOL_PATH), { recursive: true, mode: 0o700 });
    const temporary = `${SPOOL_PATH}.tmp`;
    writeFileSync(temporary, `${JSON.stringify(offlineSemanticSpool)}\n`, { mode: 0o600 });
    renameSync(temporary, SPOOL_PATH);
  } catch {
    // Best effort only: semantic telemetry must never block Pi.
  }
}

type PiSemanticEventEnvelope = {
  schema: "focusa.pi_semantic_event.v1";
  event_id: string;
  sequence: number;
  project_root: string;
  continuity_id: string;
  session_id: string;
  turn_id?: string;
  event_type: string;
  message_id?: string;
  tool_call_id?: string;
  occurred_at: string;
  content: Record<string, unknown>;
  artifact_handle?: string;
};

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
  getAttachmentRuntime().spec92HookTelemetry.push(entry);
  if (getAttachmentRuntime().spec92HookTelemetry.length > MAX_RECORDS)
    getAttachmentRuntime().spec92HookTelemetry.splice(
      0,
      getAttachmentRuntime().spec92HookTelemetry.length - MAX_RECORDS
    );
}

function recordTokenTelemetry(record: Record<string, unknown>): void {
  const entry = { ts: nowIso(), ...record };
  getAttachmentRuntime().spec92TokenTelemetry.push(entry);
  if (getAttachmentRuntime().spec92TokenTelemetry.length > MAX_RECORDS)
    getAttachmentRuntime().spec92TokenTelemetry.splice(
      0,
      getAttachmentRuntime().spec92TokenTelemetry.length - MAX_RECORDS
    );
}

async function postSemanticTelemetry(body: Record<string, unknown>): Promise<void> {
  const pending = offlineSemanticSpool.splice(0, offlineSemanticSpool.length);
  persistSemanticSpool();
  const batch = [...pending, body];
  for (const item of batch) {
    let outbound = item;
    const encoded = JSON.stringify(item);
    if (encoded.length > 12_000) {
      const runtime = getAttachmentRuntime();
      const artifact = await focusaFetch("/ecs/store", {
        method: "POST",
        body: JSON.stringify({
          kind: "Json",
          label: `pi-semantic-event-${String(item.event_type || "unknown")}`,
          content: encoded,
          project_root: runtime.sessionCwd || undefined,
          continuity_id: runtime.continuityId || undefined,
        }),
      });
      const handle = artifact?.id || artifact?.handle?.id;
      if (typeof handle === "string") {
        const semantic = item.semantic_event as Record<string, unknown> | undefined;
        outbound = {
          ...item,
          payload: { artifact_handle: handle, original_size: encoded.length },
          semantic_event: semantic ? {
            ...semantic,
            artifact_handle: handle,
            content: { artifact_handle: handle, original_size: encoded.length },
          } : undefined,
        };
      }
    }
    const result = await focusaFetch("/telemetry/event", {
      method: "POST",
      body: JSON.stringify(outbound),
    });
    if (result === null) {
      offlineSemanticSpool.push(item);
      if (offlineSemanticSpool.length > MAX_OFFLINE_SPOOL) {
        offlineSemanticSpool.splice(0, offlineSemanticSpool.length - MAX_OFFLINE_SPOOL);
      }
      persistSemanticSpool();
      break;
    }
  }
}

function bestEffortTelemetry(kind: string, payload: Record<string, unknown>): void {
  const runtime = getAttachmentRuntime();
  if (!getFocusaAvailable()) return;
  const sequence = ++semanticSequence;
  const sessionId = String((payload.session_id as string | undefined) || "pi-session");
  const messageId = typeof payload.message_id === "string" ? payload.message_id : undefined;
  const toolCallId = typeof payload.tool_call_id === "string" ? payload.tool_call_id : undefined;
  const eventId = simpleHash(`${sessionId}:${kind}:${messageId || toolCallId || "none"}:${sequence}`);
  const semantic_event: PiSemanticEventEnvelope = {
    schema: "focusa.pi_semantic_event.v1",
    event_id: eventId,
    sequence,
    project_root: runtime.sessionCwd || "",
    continuity_id: runtime.continuityId || "",
    session_id: sessionId,
    turn_id: typeof payload.turn_id === "string" ? payload.turn_id : undefined,
    event_type: kind,
    message_id: messageId,
    tool_call_id: toolCallId,
    occurred_at: nowIso(),
    content: payload,
  };
  void postSemanticTelemetry({
    event_type: kind,
    source: "pi-extension-spec92",
    payload,
    semantic_event,
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

function compactStatusText(value: unknown, fallback: string, max = 72): string {
  const text = String(value ?? "").replace(/\s+/g, " ").trim();
  if (!text) return fallback;
  return text.length > max ? `${text.slice(0, max - 1)}…` : text;
}

function otaStatus(): string {
  try {
    const paths = otaActivationPaths();
    if (existsSync(paths.activating)) return "activating";
    if (existsSync(paths.restart) || existsSync(paths.legacy)) return "ready";
    if (existsSync(paths.receipt)) return "current";
  } catch {
    return "unknown";
  }
  return "idle";
}

function humanTimeClaim(value: unknown, fallback: string): string {
  const text = String(value ?? "").trim();
  if (!text) return fallback;
  const numeric = Number(text);
  const date = Number.isFinite(numeric)
    ? new Date(numeric > 10_000_000_000 ? numeric : numeric * 1_000)
    : new Date(text);
  if (!Number.isNaN(date.getTime())) {
    try {
      return new Intl.DateTimeFormat("en-US", {
        month: "short",
        day: "numeric",
        hour: "numeric",
        minute: "2-digit",
        timeZoneName: "short",
        timeZone: String(process.env.TZ || "").trim() || undefined,
      }).format(date);
    } catch {
      return date.toISOString();
    }
  }
  return compactStatusText(text, fallback, 28);
}

function localClock(): string {
  try {
    return new Intl.DateTimeFormat("en-US", {
      hour: "numeric",
      minute: "2-digit",
      timeZoneName: "short",
      timeZone: String(process.env.TZ || "").trim() || undefined,
    }).format(new Date());
  } catch {
    return new Date().toLocaleTimeString([], { hour: "numeric", minute: "2-digit" });
  }
}

function headerValue(event: any, name: string): string {
  const headers = event?.headers || event?.response?.headers;
  if (typeof headers?.get === "function") return String(headers.get(name) || "");
  const key = Object.keys(headers || {}).find((candidate) => candidate.toLowerCase() === name);
  return key ? String(headers[key] || "") : "";
}

function updateProviderUsageFromHeaders(event: any): void {
  const usedRaw = headerValue(event, "x-codex-primary-used-percent");
  const remainingRaw = headerValue(event, "x-ratelimit-remaining-tokens");
  const limitRaw = headerValue(event, "x-ratelimit-limit-tokens");
  let used = Number.parseFloat(usedRaw);
  if (!Number.isFinite(used)) {
    const remaining = Number.parseFloat(remainingRaw);
    const limit = Number.parseFloat(limitRaw);
    if (Number.isFinite(remaining) && Number.isFinite(limit) && limit > 0)
      used = ((limit - remaining) / limit) * 100;
  }
  if (Number.isFinite(used))
    getAttachmentRuntime().providerUsagePercent = Math.max(0, Math.min(100, used));
  const renewal =
    headerValue(event, "x-codex-primary-reset-at") ||
    headerValue(event, "x-ratelimit-reset-tokens") ||
    headerValue(event, "x-ratelimit-reset");
  if (renewal) getAttachmentRuntime().providerRenewalAt = renewal;
}

function renderOperatorStatus(ctx: any): void {
  const runtime = getAttachmentRuntime();
  const contextUsage = ctx?.getContextUsage?.();
  if (
    Number.isFinite(contextUsage?.tokens) &&
    Number.isFinite(contextUsage?.contextWindow) &&
    contextUsage.contextWindow > 0
  )
    runtime.currentContextPct = (contextUsage.tokens / contextUsage.contextWindow) * 100;
  const cfg = runtime.cfg;
  if (!cfg?.operatorStatusBarEnabled) {
    ctx?.ui?.setStatus?.("focusa-operator-status", undefined);
    ctx?.ui?.setWidget?.("focusa-next-prediction", undefined);
    return;
  }
  const segments: string[] = [];
  if (cfg.operatorStatusVersionEnabled)
    segments.push(`Focusa ${compactStatusText(cfg.focusaExtensionBuild, "unknown").replace(/^focusa-pi-bridge@/, "")}`);
  if (cfg.operatorStatusOtaEnabled) segments.push(`OTA ${otaStatus()}`);
  if (cfg.operatorStatusModelUsageEnabled) {
    const provider = compactStatusText(runtime.modelProvider, "provider unknown", 24);
    const model = compactStatusText(runtime.modelId, "model unknown", 28);
    const usage = runtime.providerUsagePercent === null
      ? runtime.currentContextPct === null
        ? "usage unavailable"
        : `context ${Math.round(runtime.currentContextPct)}%`
      : `usage ${Math.round(runtime.providerUsagePercent)}%`;
    const renewal = humanTimeClaim(runtime.providerRenewalAt, "renewal unavailable");
    segments.push(`${provider}/${model} · ${usage} · renew ${renewal}`);
  }
  if (cfg.operatorStatusTimeEnabled) segments.push(localClock());
  ctx?.ui?.setStatus?.("focusa-operator-status", segments.join(" · "));

  const lines: string[] = [];
  if (cfg.operatorStatusDeadlineEnabled)
    lines.push(`Deadline: ${humanTimeClaim(process.env.FOCUSA_CONFIRMED_DEADLINE, "none confirmed")}`);
  if (cfg.operatorStatusPredictionEnabled) {
    const packet: any = getActiveWorkpointPacket();
    const prediction = runtime.startupReceptionistActive
      ? "identify the project or task you want, then continue without changing anything prematurely"
      : packet?.next_action || packet?.next_slice || "continue from the next verified project action";
    lines.push(`Next likely: ${compactStatusText(prediction, "waiting for your direction", 120)}`);
  }
  ctx?.ui?.setWidget?.("focusa-next-prediction", lines, { placement: "belowEditor" });
}

function receptionistProgressGreeting(): string {
  const hourText = new Intl.DateTimeFormat("en-US", {
    hour: "numeric",
    hourCycle: "h23",
    timeZone: String(process.env.TZ || "").trim() || undefined,
  }).format(new Date());
  const hour = Number.parseInt(hourText, 10);
  const greeting = hour < 12 ? "Good morning" : hour < 17 ? "Good afternoon" : "Good evening";
  const preferred = String(
    process.env.FOCUSA_PREFERRED_ADDRESS || process.env.OPERATOR_PREFERRED_ADDRESS || ""
  ).trim();
  return preferred ? `${greeting}, ${preferred}` : greeting;
}

function updateReceptionistProgress(ctx: any, message: string): void {
  if (!getAttachmentRuntime().startupReceptionistActive) return;
  ctx?.ui?.setWidget?.(
    "focusa-vital",
    [`${receptionistProgressGreeting()} — ${message}`],
    { placement: "belowEditor" }
  );
}

function receptionistToolProgress(toolName: string): string {
  const name = toolName.toLowerCase();
  if (["bash", "read", "find", "fd", "rg"].includes(name))
    return "I’m looking through nearby folders for likely projects (read-only)…";
  if (name === "focusa_project_identity")
    return "I’m checking whether the likely folders are existing Focusa projects…";
  if (name === "focusa_project_verify")
    return "I found a possible match and I’m checking it safely…";
  if (name.includes("trajectory") || name.includes("workpoint") || name.includes("hlt"))
    return "I’m reviewing existing project history so I don’t treat an established project as new…";
  return "I’m gathering enough context to give you useful, simple choices…";
}

export function registerPolishHooks(pi: ExtensionAPI) {
  const hookApi = pi as any;
  hookApi.on("resources_discover", async (_event: any, _ctx: any) => {
    // Pi settings/package installation is the single skill-path authority.
    // Dynamically injecting cwd, package, and legacy home paths caused noisy
    // name collisions and nondeterministic first-wins behavior.
    recordHookTelemetry({ hook: "resources_discover", skill_authority: "pi_configuration" });
    return {};
  });

  hookApi.on("session_start", async (_event: any, ctx: any) => {
    getAttachmentRuntime().modelProvider = String(ctx?.model?.provider || "");
    getAttachmentRuntime().modelId = String(ctx?.model?.id || "");
    renderOperatorStatus(ctx);
  });

  hookApi.on("model_select", async (event: any, ctx: any) => {
    getAttachmentRuntime().modelProvider = String(event?.model?.provider || ctx?.model?.provider || "");
    getAttachmentRuntime().modelId = String(event?.model?.id || ctx?.model?.id || "");
    renderOperatorStatus(ctx);
  });

  hookApi.on("agent_start", async (event: any, ctx: any) => {
    renderOperatorStatus(ctx);
    const record = {
      hook: "agent_start",
      event_keys: Object.keys(event || {}).slice(0, 20),
      workpoint_id: (() => {
        const wp = getActiveWorkpointPacket();
        return wp?.workpoint_id || wp?.id || null;
      })(),
      current_ask: boundText(getAttachmentRuntime().currentAsk?.text || ""),
    };
    recordHookTelemetry(record);
    bestEffortTelemetry("spec92.agent_start", record);
  });

  hookApi.on("message_start", async (event: any, ctx: any) => {
    recordHookTelemetry({ hook: "message_start", ...messageSummary(event?.message || event) });
    updateReceptionistProgress(ctx, "I’m checking recent projects and preparing a few clear options…");
  });

  hookApi.on("message_end", async (event: any, ctx: any) => {
    const record = { hook: "message_end", ...messageSummary(event?.message || event) };
    recordHookTelemetry(record);
    bestEffortTelemetry("spec92.message_end", record);
    updateReceptionistProgress(ctx, "I’ve finished checking and I’m putting the best options into plain language…");
  });

  hookApi.on("before_provider_request", async (event: any, ctx: any) => {
    getAttachmentRuntime().modelProvider = String(event?.provider || event?.model?.provider || ctx?.model?.provider || "");
    getAttachmentRuntime().modelId = String(event?.model?.id || event?.model || ctx?.model?.id || "");
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

  hookApi.on("after_provider_response", async (event: any, ctx: any) => {
    updateProviderUsageFromHeaders(event);
    renderOperatorStatus(ctx);
    const record = {
      hook: "after_provider_response",
      status: event?.status || event?.response?.status || "unknown",
      header_keys: event?.headers ? Object.keys(event.headers).slice(0, 12) : [],
      size_bytes: safeJsonSize(event?.response || event),
    };
    recordHookTelemetry(record);
    bestEffortTelemetry("spec92.after_provider_response", record);
  });

  hookApi.on("tool_execution_start", async (event: any, ctx: any) => {
    const record = {
      hook: "tool_execution_start",
      tool_call_id: event?.toolCallId || event?.id || "unknown",
      tool_name: event?.toolName || event?.name || "unknown",
      args_size_bytes: safeJsonSize(event?.args),
    };
    getAttachmentRuntime().spec92ToolStartTimes[String(record.tool_call_id)] = Date.now();
    recordHookTelemetry(record);
    updateReceptionistProgress(ctx, receptionistToolProgress(String(record.tool_name)));
  });

  hookApi.on("tool_execution_update", async (event: any, _ctx: any) => {
    recordHookTelemetry({
      hook: "tool_execution_update",
      tool_call_id: event?.toolCallId || event?.id || "unknown",
      tool_name: event?.toolName || event?.name || "unknown",
      partial_size_bytes: safeJsonSize(event?.partialResult || event?.update || event),
    });
  });

  hookApi.on("tool_execution_end", async (event: any, ctx: any) => {
    const id = String(event?.toolCallId || event?.id || "unknown");
    const started = getAttachmentRuntime().spec92ToolStartTimes[id];
    if (started) delete getAttachmentRuntime().spec92ToolStartTimes[id];
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
    updateReceptionistProgress(ctx, "I’m comparing what I found and narrowing it to useful choices…");
    // FOCUSA_FIX-tgij: shell-tool reminder — when the agent uses a shell-like
    // tool that could touch the Focusa daemon, emit a visible reminder to
    // prefer focusa_* tools for governed interactions.
    const SHELL_TOOLS = ["bash", "sh", "fish", "zsh", "csh", "dash"];
    const reminderCfg = getAttachmentRuntime().cfg;
    if (
      reminderCfg?.agentReminderMode === "shell" &&
      SHELL_TOOLS.includes(toolName) &&
      getFocusaAvailable()
    ) {
      const now = Date.now();
      const lastReminder = getAttachmentRuntime().lastShellReminderAt || 0;
      const turnCount = getTurnCount();
      const lastReminderTurn = getAttachmentRuntime().lastShellReminderTurn || 0;
      const frequency = Math.max(1, reminderCfg.agentReminderShellFrequency || 1);
      const cooldownMs = Math.max(0, reminderCfg.agentReminderCooldownMs || 30_000);
      if (turnCount !== lastReminderTurn && turnCount % frequency === 0 && now - lastReminder > cooldownMs) {
        getAttachmentRuntime().lastShellReminderAt = now;
        getAttachmentRuntime().lastShellReminderTurn = turnCount;
        const prefix = reminderCfg.agentReminderUseEmoji ? "🧭 " : "";
        const reminder = {
          customType: "focusa_agent_prompt",
          content: `${prefix}For Focusa daemon/state interactions, prefer focusa_* Pi tools over shell/bash — they handle scope, authority, recovery, and evidence automatically.`,
          display: true,
        };
        bestEffortTelemetry("agent_tool_layer_reminder", {
          tool_name: toolName,
          turn: turnCount,
          frequency,
          cooldown_ms: cooldownMs,
        });
        try {
          getAttachmentRuntime().pi?.sendMessage(reminder);
        } catch {
          /* best-effort */
        }
      }
    }
  });

  hookApi.on("agent_end", async (_event: any, ctx: any) => {
    if (!getAttachmentRuntime().startupReceptionistActive) return;
    getAttachmentRuntime().startupReceptionistActive = false;
    renderOperatorStatus(ctx);
    ctx?.ui?.setWidget?.(
      "focusa-vital",
      ["Ready — I’ve shared the clearest next options above. Nothing was changed while I checked."],
      { placement: "belowEditor" }
    );
    setTimeout(() => ctx?.ui?.setWidget?.("focusa-vital", undefined), 6_000);
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
