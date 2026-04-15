// Focusa Pi Bridge — Entry point
// Spec: docs/44-pi-focusa-integration-spec.md
// Wires all modules: config, tools, commands, wbm, compaction, session, turns
// Plus: provider registration (§33.6), shortcuts (§37.4), flags (§37.5), renderer (§37.6)

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { createRequire } from "module";
import { S, getEffectiveFocusSnapshot } from "./state.js";

// ESM compat: require() for synchronous imports in message renderer callback
const require = createRequire(import.meta.url);
import { loadConfig } from "./config.js";
import { registerTools } from "./tools.js";
import { registerCommands } from "./commands.js";
import { registerWbm } from "./wbm.js";
import { registerCompaction } from "./compaction.js";
import { registerSession } from "./session.js";
import { registerTurns } from "./turns.js";

export default function focusaPiBridge(pi: ExtensionAPI) {
  S.pi = pi;

  // ── Load config (§18 settings.json → §19 env vars → defaults) ──────────
  const { config, errors } = loadConfig(process.cwd());
  S.cfg = config;
  if (errors.length) {
    // §25.1: Validation errors — warn but continue with defaults
    for (const e of errors) console.warn(`[focusa] config: ${e}`);
  }
  if (!config.enabled) {
    console.info("[focusa] integration disabled via config");
    return;
  }

  // ── Wire all modules ────────────────────────────────────────────────────
  registerTools(pi);
  registerCommands(pi);
  registerWbm(pi);
  registerCompaction(pi);
  registerSession(pi);
  registerTurns(pi);

  // ── §33.6: Provider registration ───────────────────────────────────────
  pi.registerProvider("focusa", {
    baseUrl: config.focusaApiBaseUrl,
    apiKey: config.focusaToken || "FOCUSA_TOKEN",
    api: "openai-chat",
    models: [
      {
        id: "focusa-proxy",
        name: "Focusa Proxy",
        reasoning: false,
        input: ["text"],
        cost: { input: 0, output: 0, cacheRead: 0, cacheWrite: 0 },
        contextWindow: 128000,
        maxTokens: 16384,
      },
    ],
  });

  // ── §37.4: Keyboard shortcuts ──────────────────────────────────────────
  pi.registerShortcut("ctrl+shift+f", {
    description: "Show Focusa status",
    handler: async (ctx) => {
      const up = S.focusaAvailable ? "✅" : "❌";
      const snapshot = getEffectiveFocusSnapshot();
      const tier = S.currentTier ? ` | ${S.currentTier.toUpperCase()}` : "";
      const title = S.activeFrameTitle ? ` | ${S.activeFrameTitle}` : "";
      const goal = S.activeFrameGoal ? ` | ${S.activeFrameGoal}` : "";
      const mission = snapshot.intent ? ` | Mission: ${snapshot.intent}` : "";
      const focus = snapshot.currentFocus ? ` | Focus: ${snapshot.currentFocus}` : "";
      ctx.ui.notify(`Focusa: ${up}${title}${goal}${mission}${focus} | Frame: ${S.activeFrameId ?? "none"} | D:${snapshot.decisions.length} C:${snapshot.constraints.length} F:${snapshot.failures.length}${tier}`, "info");
    },
  });

  pi.registerShortcut("ctrl+shift+b", {
    description: "Toggle Wirebot Mode",
    handler: async (ctx) => {
      S.wbmEnabled = !S.wbmEnabled;
      ctx.ui.notify(`WBM: ${S.wbmEnabled ? "ON" : "OFF"}`, "info");
      ctx.ui.setStatus("focusa", S.wbmEnabled ? "🤖 Focusa WBM" : "🧭 Focusa");
    },
  });

  // ── §37.5: CLI flags ──────────────────────────────────────────────────
  pi.registerFlag("wbm", {
    description: "Enable Wirebot Mode on startup",
    type: "boolean",
  });

  pi.registerFlag("no-focusa", {
    description: "Disable Focusa integration",
    type: "boolean",
  });

  // ── §37.6: Custom message renderer for persisted Focusa state entries ───
  const renderFocusaState = (message: any, _options: any, theme: any) => {
    const { Text } = require("@mariozechner/pi-tui");
    const d = (message as any).details;
    if (!d) return undefined;
    const decisions = d.authoritativeDecisions || d.decisions || [];
    const constraints = d.authoritativeConstraints || d.constraints || [];
    const failures = d.authoritativeFailures || d.failures || [];
    const parts: string[] = ["📎 Focusa State"];
    if (d.frameTitle) parts.push(`Title: ${d.frameTitle}`);
    if (d.frameGoal) parts.push(`Goal: ${d.frameGoal}`);
    if (d.intent) parts.push(`Mission: ${d.intent}`);
    if (d.currentFocus) parts.push(`Focus: ${d.currentFocus}`);
    if (d.frameId) parts.push(`Frame: ${d.frameId}`);
    if (d.sessionId) parts.push(`Session: ${d.sessionId}`);
    if (decisions.length) parts.push(`D:${decisions.length}`);
    if (constraints.length) parts.push(`C:${constraints.length}`);
    if (failures.length) parts.push(`F:${failures.length}`);
    parts.push(`T:${d.turnCount || 0}`);
    if (d.totalCompactions) parts.push(`Compactions:${d.totalCompactions}`);
    return new Text(theme.fg("dim", parts.join(" | ")), 0, 0);
  };
  pi.registerMessageRenderer("focusa-state", renderFocusaState);
  pi.registerMessageRenderer("focusa-wbm-state", renderFocusaState);
}
