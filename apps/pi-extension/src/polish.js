import { existsSync, mkdirSync, readFileSync, renameSync, writeFileSync } from "node:fs";
import { homedir } from "node:os";
import { dirname, join } from "node:path";
import { otaActivationPaths } from "./ota-activation.js";
import { saveConfigOverrides } from "./config.js";
import { classifyShellReminderInteraction, } from "./shell-reminder-classification.js";
import { createOperatorWidgetRegistry, migrateOperatorStatusSettings, operatorStatusRollbackPatch, renderOperatorStatusBar, } from "./operator-status-widgets.js";
import { getAttachmentRuntime, focusaFetch, focusaPost, getFocusaAvailable, getTurnCount, getActiveWorkpointPacket, } from "./state.js";
const MAX_RECORDS = 80;
const MAX_TEXT = 500;
let semanticSequence = 0;
const MAX_OFFLINE_SPOOL = 64;
const SPOOL_PATH = join(String(process.env.FOCUSA_DATA_DIR || "").trim() || join(homedir(), ".focusa"), "pi-semantic-spool.json");
const offlineSemanticSpool = loadSemanticSpool();
const shellReminderClassifications = new Map();
function loadSemanticSpool() {
    if (!existsSync(SPOOL_PATH))
        return [];
    try {
        const parsed = JSON.parse(readFileSync(SPOOL_PATH, "utf8"));
        return Array.isArray(parsed) ? parsed.slice(-MAX_OFFLINE_SPOOL) : [];
    }
    catch {
        return [];
    }
}
function persistSemanticSpool() {
    try {
        mkdirSync(dirname(SPOOL_PATH), { recursive: true, mode: 0o700 });
        const temporary = `${SPOOL_PATH}.tmp`;
        writeFileSync(temporary, `${JSON.stringify(offlineSemanticSpool)}\n`, { mode: 0o600 });
        renameSync(temporary, SPOOL_PATH);
    }
    catch {
        // Best effort only: semantic telemetry must never block Pi.
    }
}
function nowIso() {
    return new Date().toISOString();
}
function boundText(value, max = MAX_TEXT) {
    const text = String(value ?? "");
    return text.length > max ? `${text.slice(0, max)}…` : text;
}
function safeJsonSize(value) {
    try {
        return JSON.stringify(value ?? null).length;
    }
    catch {
        return 0;
    }
}
function simpleHash(value) {
    let h = 2166136261;
    for (let i = 0; i < value.length; i++) {
        h ^= value.charCodeAt(i);
        h = Math.imul(h, 16777619);
    }
    return `fnv1a:${(h >>> 0).toString(16).padStart(8, "0")}`;
}
function estimateTokensFromChars(chars) {
    return Math.ceil(chars / 4);
}
function recordHookTelemetry(record) {
    const entry = { ts: nowIso(), ...record };
    getAttachmentRuntime().spec92HookTelemetry.push(entry);
    if (getAttachmentRuntime().spec92HookTelemetry.length > MAX_RECORDS)
        getAttachmentRuntime().spec92HookTelemetry.splice(0, getAttachmentRuntime().spec92HookTelemetry.length - MAX_RECORDS);
}
function recordTokenTelemetry(record) {
    const entry = { ts: nowIso(), ...record };
    getAttachmentRuntime().spec92TokenTelemetry.push(entry);
    if (getAttachmentRuntime().spec92TokenTelemetry.length > MAX_RECORDS)
        getAttachmentRuntime().spec92TokenTelemetry.splice(0, getAttachmentRuntime().spec92TokenTelemetry.length - MAX_RECORDS);
}
async function postSemanticTelemetry(body) {
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
                const semantic = item.semantic_event;
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
function bestEffortTelemetry(kind, payload) {
    const runtime = getAttachmentRuntime();
    if (!getFocusaAvailable())
        return;
    const sequence = ++semanticSequence;
    const sessionId = String(payload.session_id || "pi-session");
    const messageId = typeof payload.message_id === "string" ? payload.message_id : undefined;
    const toolCallId = typeof payload.tool_call_id === "string" ? payload.tool_call_id : undefined;
    const eventId = simpleHash(`${sessionId}:${kind}:${messageId || toolCallId || "none"}:${sequence}`);
    const semantic_event = {
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
function messageId(message) {
    return String(message?.id || message?.messageId || message?.uuid || "unknown");
}
function messageSummary(message) {
    const size = safeJsonSize(message);
    return {
        message_id: messageId(message),
        role: message?.role || message?.type || "unknown",
        size_bytes: size,
        token_estimate: estimateTokensFromChars(size),
        has_tool_calls: JSON.stringify(message ?? {}).includes("toolCall"),
    };
}
function payloadSummary(payload) {
    const text = JSON.stringify(payload ?? {});
    const size = text.length;
    const tokenEstimate = estimateTokensFromChars(size);
    const messageCount = Array.isArray(payload?.messages) ? payload.messages.length : 0;
    const toolSchemaBytes = safeJsonSize(payload?.tools || payload?.tool_choice || payload?.toolConfig);
    const budgetClass = tokenEstimate > 120_000
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
function otaStatus() {
    try {
        const paths = otaActivationPaths();
        if (existsSync(paths.activating))
            return "activating";
        if (existsSync(paths.restart) || existsSync(paths.legacy))
            return "ready";
        if (existsSync(paths.receipt))
            return "current";
    }
    catch {
        return "unknown";
    }
    return "idle";
}
function headerValue(event, name) {
    const headers = event?.headers || event?.response?.headers;
    if (typeof headers?.get === "function")
        return String(headers.get(name) || "");
    const key = Object.keys(headers || {}).find((candidate) => candidate.toLowerCase() === name);
    return key ? String(headers[key] || "") : "";
}
function updateProviderUsageFromHeaders(event) {
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
    if (Number.isFinite(used)) {
        getAttachmentRuntime().providerUsagePercent = Math.max(0, Math.min(100, used));
        getAttachmentRuntime().providerUsageObservedAt = Date.now();
    }
    const renewal = headerValue(event, "x-codex-primary-reset-at") ||
        headerValue(event, "x-ratelimit-reset-tokens") ||
        headerValue(event, "x-ratelimit-reset");
    if (renewal) {
        getAttachmentRuntime().providerRenewalAt = renewal;
        getAttachmentRuntime().providerUsageObservedAt = Date.now();
    }
}
const operatorWidgetRegistry = createOperatorWidgetRegistry();
function operatorWidgetSettings(cfg) {
    return migrateOperatorStatusSettings(cfg?.operatorStatusWidgets, {
        time: Boolean(cfg?.operatorStatusTimeEnabled || cfg?.operatorStatusDeadlineEnabled),
        prediction: cfg?.operatorStatusPredictionEnabled,
        version: cfg?.operatorStatusVersionEnabled,
        ota: cfg?.operatorStatusOtaEnabled,
        "provider-usage": cfg?.operatorStatusModelUsageEnabled,
    }, operatorWidgetRegistry);
}
function parseObservedAt(value) {
    if (typeof value === "number" && Number.isFinite(value))
        return value;
    const parsed = Date.parse(String(value ?? ""));
    return Number.isFinite(parsed) ? parsed : undefined;
}
function renderOperatorStatus(ctx) {
    const runtime = getAttachmentRuntime();
    const cfg = runtime.cfg;
    if (!cfg?.operatorStatusBarEnabled) {
        ctx?.ui?.setStatus?.("focusa-operator-status", undefined);
        ctx?.ui?.setWidget?.("focusa-next-prediction", undefined);
        return;
    }
    const packet = getActiveWorkpointPacket();
    const prediction = packet?.next_action || packet?.next_slice;
    const ota = otaStatus();
    const result = renderOperatorStatusBar({
        now: Date.now(),
        timezone: String(process.env.TZ || "").trim() || undefined,
        deadline: process.env.FOCUSA_CONFIRMED_DEADLINE,
        prediction,
        predictionLoading: runtime.startupReceptionistActive,
        predictionObservedAt: parseObservedAt(packet?.updated_at || packet?.observed_at),
        version: cfg.focusaExtensionBuild,
        ota,
        otaState: ota === "unknown" ? "degraded" : "ready",
        otaObservedAt: Date.now(),
        provider: runtime.modelProvider,
        model: runtime.modelId,
        usagePercent: runtime.providerUsagePercent,
        renewalAt: runtime.providerRenewalAt,
        providerObservedAt: runtime.providerUsageObservedAt || undefined,
    }, operatorWidgetSettings(cfg), Math.max(24, Number(process.stdout.columns || 120)), operatorWidgetRegistry);
    ctx?.ui?.setStatus?.("focusa-operator-status", result.text || undefined);
    ctx?.ui?.setWidget?.("focusa-next-prediction", undefined);
}
function receptionistProgressGreeting() {
    const hourText = new Intl.DateTimeFormat("en-US", {
        hour: "numeric",
        hourCycle: "h23",
        timeZone: String(process.env.TZ || "").trim() || undefined,
    }).format(new Date());
    const hour = Number.parseInt(hourText, 10);
    const greeting = hour < 12 ? "Good morning" : hour < 17 ? "Good afternoon" : "Good evening";
    const preferred = String(process.env.FOCUSA_PREFERRED_ADDRESS || process.env.OPERATOR_PREFERRED_ADDRESS || "").trim();
    return preferred ? `${greeting}, ${preferred}` : greeting;
}
function updateReceptionistProgress(ctx, message) {
    if (!getAttachmentRuntime().startupReceptionistActive)
        return;
    ctx?.ui?.setWidget?.("focusa-vital", [`${receptionistProgressGreeting()} — ${message}`], { placement: "belowEditor" });
}
function receptionistToolProgress(toolName) {
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
export function registerPolishHooks(pi) {
    const hookApi = pi;
    pi.registerCommand("focusa-bar", {
        description: "Show or change Focusa operator status widgets",
        handler: async (args, ctx) => {
            const runtime = getAttachmentRuntime();
            const settings = operatorWidgetSettings(runtime.cfg);
            const tokens = String(args || "").trim().split(/\s+/).filter(Boolean);
            if (tokens[0] === "list") {
                const summary = operatorWidgetRegistry.map((widget) => `${settings.enabled[widget.id] ? "on" : "off"} ${widget.id}`).join(" · ");
                ctx.ui.notify(`Focusa bar: ${summary}`, "info");
                return;
            }
            let widgetId = tokens[0];
            if (!widgetId) {
                const choice = await ctx.ui.select("Focusa bar widgets", operatorWidgetRegistry.map((widget) => `${settings.enabled[widget.id] ? "✓" : "○"} ${widget.label} (${widget.id})`));
                widgetId = operatorWidgetRegistry.find((widget) => choice?.endsWith(`(${widget.id})`))?.id;
            }
            const widget = operatorWidgetRegistry.find((candidate) => candidate.id === widgetId);
            if (!widget) {
                ctx.ui.notify("Usage: /focusa-bar <time|prediction|version|ota|provider-usage> <on|off|toggle>, or /focusa-bar list", "warning");
                return;
            }
            const requested = tokens[1];
            if (requested && !["on", "off", "toggle"].includes(requested)) {
                ctx.ui.notify("Widget state must be on, off, or toggle; no setting was changed.", "warning");
                return;
            }
            settings.enabled[widget.id] = requested === "on" ? true : requested === "off" ? false : !settings.enabled[widget.id];
            try {
                const saved = saveConfigOverrides(ctx.cwd, {
                    operatorStatusWidgets: settings,
                    ...operatorStatusRollbackPatch(settings),
                });
                runtime.cfg = saved.config;
                renderOperatorStatus(ctx);
                ctx.ui.notify(`${widget.label} ${settings.enabled[widget.id] ? "enabled" : "disabled"}; saved in ${saved.path}.`, "info");
            }
            catch (error) {
                ctx.ui.notify(`Focusa bar setting not saved: ${String(error).slice(0, 180)}`, "error");
            }
        },
    });
    hookApi.on("resources_discover", async (_event, _ctx) => {
        // Pi settings/package installation is the single skill-path authority.
        // Dynamically injecting cwd, package, and legacy home paths caused noisy
        // name collisions and nondeterministic first-wins behavior.
        recordHookTelemetry({ hook: "resources_discover", skill_authority: "pi_configuration" });
        return {};
    });
    hookApi.on("session_start", async (_event, ctx) => {
        getAttachmentRuntime().modelProvider = String(ctx?.model?.provider || "");
        getAttachmentRuntime().modelId = String(ctx?.model?.id || "");
        renderOperatorStatus(ctx);
    });
    hookApi.on("model_select", async (event, ctx) => {
        getAttachmentRuntime().modelProvider = String(event?.model?.provider || ctx?.model?.provider || "");
        getAttachmentRuntime().modelId = String(event?.model?.id || ctx?.model?.id || "");
        renderOperatorStatus(ctx);
    });
    hookApi.on("agent_start", async (event, ctx) => {
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
    hookApi.on("message_start", async (event, ctx) => {
        recordHookTelemetry({ hook: "message_start", ...messageSummary(event?.message || event) });
        updateReceptionistProgress(ctx, "I’m checking recent projects and preparing a few clear options…");
    });
    hookApi.on("message_end", async (event, ctx) => {
        const record = { hook: "message_end", ...messageSummary(event?.message || event) };
        recordHookTelemetry(record);
        bestEffortTelemetry("spec92.message_end", record);
        updateReceptionistProgress(ctx, "I’ve finished checking and I’m putting the best options into plain language…");
    });
    hookApi.on("before_provider_request", async (event, ctx) => {
        getAttachmentRuntime().modelProvider = String(event?.provider || event?.model?.provider || ctx?.model?.provider || "");
        getAttachmentRuntime().modelId = String(event?.model?.id || event?.model || ctx?.model?.id || "");
        const summary = payloadSummary(event?.payload || event?.request || event);
        const record = {
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
    hookApi.on("after_provider_response", async (event, ctx) => {
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
    hookApi.on("tool_execution_start", async (event, ctx) => {
        const record = {
            hook: "tool_execution_start",
            tool_call_id: event?.toolCallId || event?.id || "unknown",
            tool_name: event?.toolName || event?.name || "unknown",
            args_size_bytes: safeJsonSize(event?.args),
        };
        const toolCallId = String(record.tool_call_id);
        getAttachmentRuntime().spec92ToolStartTimes[toolCallId] = Date.now();
        shellReminderClassifications.set(toolCallId, classifyShellReminderInteraction(record.tool_name, event?.args));
        recordHookTelemetry(record);
        updateReceptionistProgress(ctx, receptionistToolProgress(String(record.tool_name)));
    });
    hookApi.on("tool_execution_update", async (event, _ctx) => {
        recordHookTelemetry({
            hook: "tool_execution_update",
            tool_call_id: event?.toolCallId || event?.id || "unknown",
            tool_name: event?.toolName || event?.name || "unknown",
            partial_size_bytes: safeJsonSize(event?.partialResult || event?.update || event),
        });
    });
    hookApi.on("tool_execution_end", async (event, ctx) => {
        const id = String(event?.toolCallId || event?.id || "unknown");
        const started = getAttachmentRuntime().spec92ToolStartTimes[id];
        if (started)
            delete getAttachmentRuntime().spec92ToolStartTimes[id];
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
        const shellClassification = shellReminderClassifications.get(id) || classifyShellReminderInteraction(toolName, event?.args);
        shellReminderClassifications.delete(id);
        bestEffortTelemetry("agent_shell_classification", {
            tool_name: toolName,
            classification: shellClassification.classification,
            confidence: shellClassification.confidence,
            equivalent_tool: shellClassification.equivalent_tool,
            reason: shellClassification.reason,
        });
        const reminderCfg = getAttachmentRuntime().cfg;
        if (reminderCfg?.agentReminderMode === "shell" &&
            shellClassification.classification === "actual_focusa_bypass" &&
            shellClassification.confidence === "high" &&
            shellClassification.equivalent_tool &&
            getFocusaAvailable()) {
            const now = Date.now();
            const lastReminder = getAttachmentRuntime().lastShellReminderAt || 0;
            const turnCount = getTurnCount();
            const lastReminderTurn = getAttachmentRuntime().lastShellReminderTurn || 0;
            const frequency = Math.max(2, reminderCfg.agentReminderShellFrequency || 3);
            const cooldownMs = Math.max(0, reminderCfg.agentReminderCooldownMs || 30_000);
            if (turnCount !== lastReminderTurn && turnCount % frequency === 0 && now - lastReminder > cooldownMs) {
                getAttachmentRuntime().lastShellReminderAt = now;
                getAttachmentRuntime().lastShellReminderTurn = turnCount;
                const prefix = reminderCfg.agentReminderUseEmoji ? "🧭 " : "";
                const reminder = {
                    customType: "focusa_agent_prompt",
                    content: `${prefix}Raw Focusa daemon/state access detected. Prefer ${shellClassification.equivalent_tool}; it preserves scope, authority, recovery, and evidence.`,
                    display: true,
                };
                bestEffortTelemetry("agent_tool_layer_reminder", {
                    tool_name: toolName,
                    classification: "actual_focusa_bypass",
                    equivalent_tool: shellClassification.equivalent_tool,
                    turn: turnCount,
                    frequency,
                    cooldown_ms: cooldownMs,
                });
                try {
                    getAttachmentRuntime().pi?.sendMessage(reminder);
                }
                catch {
                    /* best-effort */
                }
            }
        }
    });
    hookApi.on("agent_end", async (_event, ctx) => {
        if (!getAttachmentRuntime().startupReceptionistActive)
            return;
        getAttachmentRuntime().startupReceptionistActive = false;
        const previousThinking = getAttachmentRuntime().startupReceptionistPreviousThinkingLevel;
        getAttachmentRuntime().startupReceptionistPreviousThinkingLevel = "";
        if (previousThinking) {
            try {
                getAttachmentRuntime().pi?.setThinkingLevel(previousThinking);
            }
            catch {
                // Keep reception completion nonblocking even if model state changed.
            }
        }
        renderOperatorStatus(ctx);
        ctx?.ui?.setWidget?.("focusa-vital", ["Ready — I’ve shared the clearest next options above. Nothing was changed while I checked."], { placement: "belowEditor" });
    });
    hookApi.on("session_tree", async (event, _ctx) => {
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
