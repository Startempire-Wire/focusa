// Focusa Pi Bridge — Entry point
// Spec: docs/44-pi-focusa-integration-spec.md
// Wires all modules: config, tools, commands, wbm, compaction, session, turns
// Plus: provider registration (§33.6), shortcuts (§37.4), flags (§37.5), renderer (§37.6)

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { createRequire } from "module";
import { S } from "./state.js";

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
      const d = S.localDecisions.length, c = S.localConstraints.length, f = S.localFailures.length;
      const tier = S.currentTier ? ` | ${S.currentTier.toUpperCase()}` : "";
      ctx.ui.notify(`Focusa: ${up} | Frame: ${S.activeFrameId ?? "none"} | D:${d} C:${c} F:${f}${tier}`, "info");
    },
  });

  pi.registerShortcut("ctrl+shift+w", {
    description: "Toggle Wirebot Mode",
    handler: async (ctx) => {
      S.wbmEnabled = !S.wbmEnabled;
      ctx.ui.notify(`WBM: ${S.wbmEnabled ? "ON" : "OFF"}`, "info");
      ctx.ui.setStatus("focusa", S.wbmEnabled ? "🧠 Focusa [WBM]" : "🧠 Focusa");
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

  // ── §37.6: Custom message renderer for focusa-state entries ───────────
  pi.registerMessageRenderer("focusa-state", (message, _options, theme) => {
    const { Text } = require("@mariozechner/pi-tui");
    const d = (message as any).details;
    if (!d) return undefined;
    const parts: string[] = ["📎 Focusa State"];
    if (d.frameId) parts.push(`Frame: ${d.frameId}`);
    if (d.decisions?.length) parts.push(`D:${d.decisions.length}`);
    if (d.constraints?.length) parts.push(`C:${d.constraints.length}`);
    if (d.failures?.length) parts.push(`F:${d.failures.length}`);
    parts.push(`T:${d.turnCount || 0}`);
    if (d.totalCompactions) parts.push(`Compactions:${d.totalCompactions}`);
    return new Text(theme.fg("dim", parts.join(" | ")), 0, 0);
  });
}
