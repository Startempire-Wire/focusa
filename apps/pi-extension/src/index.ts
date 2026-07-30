// Focusa Pi Bridge — Entry point
// Spec: docs/44-pi-focusa-integration-spec.md
// Wires all modules: config, tools, commands, wbm, compaction, session, turns
// Plus: provider registration (§33.6), shortcuts (§37.4), flags (§37.5), renderer (§37.6)

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { createRequire } from "module";
import {
  attachmentRuntimeRegistry,
  getAttachmentRuntime,
  getEffectiveFocusSnapshot,
  getFocusaAvailable,
  getActiveFrameId,
  getActiveWorkpointPacket,
  isProjectRootAuthoritySafe,
  makeAttachmentKey,
  runWithAttachmentRuntime,
} from "./state.js";
import { PiExtensionSessionBinding, attachmentRoutingHints } from "./scoped-state.js";

// ESM compat: require() for synchronous imports in message renderer callback
const require = createRequire(import.meta.url);
import { loadConfig } from "./config.js";
import { registerTools } from "./tools.js";
import { registerCommands } from "./commands.js";
import { registerAutomaticOtaActivation } from "./ota-activation.js";
import { registerWbm } from "./wbm.js";
import { registerCompaction } from "./compaction.js";
import { proactiveCompactionPolicy, registerAutoCompaction } from "./auto-compaction.js";
import { registerSession } from "./session.js";
import { registerTurns } from "./turns.js";
import { registerPolishHooks } from "./polish.js";
import { registerMissionCanvasWidget } from "./mission-canvas-widget.js";

export default function focusaPiBridge(pi: ExtensionAPI) {
  const extensionKey = makeAttachmentKey({
    projectRoot: process.cwd(),
    continuityId: "extension-bootstrap",
    sessionId: `pi-extension-${process.pid}`,
    attachmentId: "extension-bootstrap",
  });
  const withRuntime = <T>(fn: () => T): T => runWithAttachmentRuntime(extensionKey, fn);
  const sessionBinding = new PiExtensionSessionBinding();
  return withRuntime(() => {
    const runtimeFor = (ctx?: any, eventOrParams?: any) => {
      const hints = attachmentRoutingHints(eventOrParams);
      const sessionId = String(
        hints.sessionId ||
          ctx?.sessionManager?.getSessionFile?.() ||
          getAttachmentRuntime(extensionKey).sessionFrameKey ||
          `pi-extension-${process.pid}`
      );
      const explicitProjectRoot = hints.projectRoot;
      const explicitContinuity = hints.continuityId;
      if (!explicitProjectRoot && !explicitContinuity) {
        const bound = attachmentRuntimeRegistry.boundSessionAttachment(sessionId) || sessionBinding.resolve();
        if (bound) return bound;
      }
      const projectRoot = String(explicitProjectRoot || ctx?.cwd || process.cwd());
      const continuityId = String(
        explicitContinuity || getAttachmentRuntime(extensionKey).continuityId || "extension-bootstrap"
      );
      return makeAttachmentKey({ projectRoot, continuityId, sessionId, attachmentId: sessionId });
    };
    const prepareRuntime = (key: ReturnType<typeof makeAttachmentKey>) => {
      const bootstrap = getAttachmentRuntime(extensionKey);
      const target = getAttachmentRuntime(key);
      // Process configuration and the Pi adapter are attachment dependencies,
      // not project authority. Seed every typed attachment explicitly so event
      // handlers do not silently lose compaction policy after registry lookup.
      if (!target.cfg && bootstrap.cfg) target.cfg = bootstrap.cfg;
      if (!target.pi) target.pi = pi;
      attachmentRuntimeRegistry.bindSessionAttachment(key);
      if (
        isProjectRootAuthoritySafe(key.workstream.root_scope.root_path) &&
        key.workstream.continuity_id &&
        key.workstream.continuity_id !== "extension-bootstrap"
      ) {
        sessionBinding.bind(key);
      }
      return key;
    };
    const originalOn = (pi as any).on?.bind(pi);
    if (originalOn) {
      (pi as any).on = (eventName: string, handler: (event: any, ctx: any) => any) =>
        originalOn(eventName, (event: any, ctx: any) =>
          runWithAttachmentRuntime(prepareRuntime(runtimeFor(ctx, event)), () => handler(event, ctx))
        );
    }
    const originalRegisterCommand = (pi as any).registerCommand?.bind(pi);
    if (originalRegisterCommand) {
      (pi as any).registerCommand = (name: string, command: any) =>
        originalRegisterCommand(name, {
          ...command,
          handler: command?.handler
            ? (args: string, ctx: any) =>
                runWithAttachmentRuntime(prepareRuntime(runtimeFor(ctx, { command: name })), () =>
                  command.handler(args, ctx)
                )
            : command?.handler,
        });
    }
    const originalRegisterShortcut = (pi as any).registerShortcut?.bind(pi);
    if (originalRegisterShortcut) {
      (pi as any).registerShortcut = (keys: string, shortcut: any) =>
        originalRegisterShortcut(keys, {
          ...shortcut,
          handler: shortcut?.handler
            ? (ctx: any) =>
                runWithAttachmentRuntime(prepareRuntime(runtimeFor(ctx, { shortcut: keys })), () =>
                  shortcut.handler(ctx)
                )
            : shortcut?.handler,
        });
    }
    const originalRegisterTool = (pi as any).registerTool?.bind(pi);
    if (originalRegisterTool) {
      (pi as any).registerTool = (tool: any) =>
        originalRegisterTool({
          ...tool,
          execute: tool?.execute
            ? (id: string, params: any, signal: AbortSignal, onUpdate: any, ctx: any) =>
                runWithAttachmentRuntime(prepareRuntime(runtimeFor(ctx, params || {})), () =>
                  tool.execute(id, params, signal, onUpdate, ctx)
                )
            : tool?.execute,
        });
    }

    getAttachmentRuntime().pi = pi;

    // ── Load config (§18 settings.json → §19 env vars → defaults) ──────────
    const { config, errors } = loadConfig(process.cwd());
    getAttachmentRuntime().cfg = config;
    if (errors.length) {
      // §25.1: Validation errors — warn but continue with defaults
      for (const e of errors) console.warn(`[focusa] config: ${e}`);
    }
    if (!config.enabled) {
      console.info("[focusa] integration disabled via config");
      return;
    }

    // ── Wire all modules ────────────────────────────────────────────────────
    // Acquire the process-wide lease before any Focusa handlers are registered.
    // A duplicate installation emits one diagnostic and registers nothing.
    const ownsCompactionCoordinator = registerAutoCompaction(pi, () =>
      proactiveCompactionPolicy(getAttachmentRuntime().cfg)
    );
    if (!ownsCompactionCoordinator) return;
    registerTools(pi);
    registerCommands(pi);
    registerMissionCanvasWidget(pi);
    registerAutomaticOtaActivation(pi);
    registerWbm(pi);
    registerCompaction(pi);
    registerSession(pi);
    registerTurns(pi);
    registerPolishHooks(pi);

    // ── §33.6: Optional proxy provider registration ───────────────────────
    // Default off: normal Focusa/Pi bridge sessions use direct providers plus
    // Focusa tools/hooks. Registering an extra provider without a token creates
    // noisy startup/auth warnings and is unnecessary for loopback local use.
    if (config.registerProxyProvider && config.focusaToken) {
      pi.registerProvider("focusa", {
        baseUrl: config.focusaApiBaseUrl,
        apiKey: config.focusaToken,
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
    }

    // ── §37.4: Keyboard shortcuts ──────────────────────────────────────────
    pi.registerShortcut("ctrl+shift+f", {
      description: "Show Focusa status",
      handler: async (ctx) => {
        const up = getFocusaAvailable() ? "✅" : "❌";
        const snapshot = getEffectiveFocusSnapshot();
        const tier = getAttachmentRuntime().currentTier
          ? ` | ${getAttachmentRuntime().currentTier.toUpperCase()}`
          : "";
        const title = getAttachmentRuntime().activeFrameTitle
          ? ` | ${getAttachmentRuntime().activeFrameTitle}`
          : "";
        const goal = getAttachmentRuntime().activeFrameGoal
          ? ` | ${getAttachmentRuntime().activeFrameGoal}`
          : "";
        const mission = snapshot.intent ? ` | Mission: ${snapshot.intent}` : "";
        const focus = snapshot.currentFocus ? ` | Focus: ${snapshot.currentFocus}` : "";
        ctx.ui.notify(
          `Focusa: ${up}${title}${goal}${mission}${focus} | Frame: ${getActiveFrameId() ?? "none"} | D:${snapshot.decisions.length} C:${snapshot.constraints.length} F:${snapshot.failures.length}${tier}`,
          "info"
        );
      },
    });

    pi.registerShortcut("ctrl+shift+r", {
      description: "Inspect active Work Rail row",
      handler: async (ctx) => {
        const packet = getActiveWorkpointPacket();
        const workpoint =
          packet?.workpoint && typeof packet.workpoint === "object" ? packet.workpoint : packet;
        const bead = workpoint?.work_item_id || packet?.work_item_id || "no bead";
        const workpointId = workpoint?.workpoint_id || packet?.workpoint_id || "no workpoint";
        const proof = Array.isArray(workpoint?.verification_records)
          ? workpoint.verification_records.length
          : Array.isArray(packet?.evidence_refs)
            ? packet.evidence_refs.length
            : 0;
        const next = workpoint?.next_slice || packet?.next_slice || "checkpoint next action";
        ctx.ui.notify(
          `Work Rail | ${bead} | ${workpointId} | proof:${proof} | next:${next}`,
          packet ? "info" : "warning"
        );
      },
    });

    pi.registerShortcut("ctrl+shift+b", {
      description: "Toggle Wirebot Mode",
      handler: async (ctx) => {
        getAttachmentRuntime().wbmEnabled = !getAttachmentRuntime().wbmEnabled;
        ctx.ui.notify(`WBM: ${getAttachmentRuntime().wbmEnabled ? "ON" : "OFF"}`, "info");
        ctx.ui.setStatus("focusa", getAttachmentRuntime().wbmEnabled ? "🤖 Focusa WBM" : "🧭 Focusa");
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
      const { Text } = require("@earendil-works/pi-tui");
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
  });
}
