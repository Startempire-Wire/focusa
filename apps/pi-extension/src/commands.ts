// All /focusa-* slash commands
// Spec: §10.3 — Commands registry, §34.2E (explain-decision), §34.2F (lineage)
// Plus: §33.5 isolation commands: /focusa-on, /focusa-off, /focusa-reset

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { getSettingsListTheme } from "@mariozechner/pi-coding-agent";
import { Container, Text, type SettingItem, SettingsList } from "@mariozechner/pi-tui";
import { S, focusaFetch, getFocusState, persistState, createPiFrame, ensurePiFrame } from "./state.js";
import { saveConfigOverrides } from "./config.js";

function nonEmptyLines(items: any[] | undefined): string[] {
  return (items || []).map((v) => String(v || "").trim()).filter(Boolean);
}

const WARN_OPTIONS = ["40", "50", "60", "70"];
const COMPACT_OPTIONS = ["60", "70", "80", "85", "90"];
const HARD_OPTIONS = ["75", "85", "92", "95", "97"];

function nextHigher(options: string[], value: number): string {
  return options.find((v) => Number(v) > value) || options[options.length - 1];
}

function nextLower(options: string[], value: number): string {
  const lower = options.filter((v) => Number(v) < value);
  return lower[lower.length - 1] || options[0];
}

function normalizeTierConfig(draft: { warnPct: number; compactPct: number; hardPct: number }) {
  if (draft.warnPct >= draft.compactPct) draft.compactPct = Number(nextHigher(COMPACT_OPTIONS, draft.warnPct));
  if (draft.compactPct >= draft.hardPct) draft.hardPct = Number(nextHigher(HARD_OPTIONS, draft.compactPct));
  if (draft.compactPct >= draft.hardPct) draft.compactPct = Number(nextLower(COMPACT_OPTIONS, draft.hardPct));
  if (draft.warnPct >= draft.compactPct) draft.warnPct = Number(nextLower(WARN_OPTIONS, draft.compactPct));
}

function renderFocusaContext(data: { frame: any; fs: any }): string {
  const { frame, fs } = data;
  const lines: string[] = [
    "# Focusa Context",
    "",
    "Rendered live from focusa-pi-bridge current state.",
    "",
  ];

  if (frame?.title) {
    lines.push(`## Current Focus Frame: ${frame.title}`);
    if (frame?.goal) lines.push(`**Goal:** ${frame.goal}`);
    lines.push("");
  }

  const decisions = nonEmptyLines(fs?.decisions);
  if (decisions.length) {
    lines.push("## Active Decisions");
    lines.push(...decisions.map((item) => `- ${item}`));
    lines.push("");
  }

  const constraints = nonEmptyLines(fs?.constraints);
  if (constraints.length) {
    lines.push("## Constraints");
    lines.push(...constraints.map((item) => `- ${item}`));
    lines.push("");
  }

  const currentFocus = String(fs?.current_focus || "").trim();
  if (currentFocus) {
    lines.push("## Current Focus");
    lines.push(currentFocus);
    lines.push("");
  }

  const openQuestions = nonEmptyLines(fs?.open_questions);
  if (openQuestions.length) {
    lines.push("## Open Questions");
    lines.push(...openQuestions.map((item) => `- ${item}`));
    lines.push("");
  }

  const failures = nonEmptyLines(fs?.failures);
  if (failures.length) {
    lines.push("## Known Failures");
    lines.push(...failures.map((item) => `- ${item}`));
    lines.push("");
  }

  lines.push("---");
  lines.push("Focusa structured context — rendered from live state; follow operator intent first.");
  return lines.join("\n").replace(/\n{3,}/g, "\n\n").trim();
}

export function registerCommands(pi: ExtensionAPI) {
  // /focusa-context (§34.2H runtime render)
  pi.registerCommand("focusa-context", {
    description: "Render current Focusa context inline in the conversation",
    handler: async (_args, ctx) => {
      if (!S.focusaAvailable) {
        const text = "Focusa offline — no live context available.";
        ctx.ui.notify(text, "warning");
        pi.sendMessage({ customType: "focusa-context", content: text, display: true });
        return;
      }
      let data = await getFocusState();
      if (!data) {
        await ensurePiFrame(ctx.cwd, undefined, "pi-auto-recover");
        data = await getFocusState();
      }
      if (!data) {
        const text = "No active Focusa frame for this Pi session.";
        ctx.ui.notify(text, "info");
        pi.sendMessage({ customType: "focusa-context", content: text, display: true });
        return;
      }
      const rendered = renderFocusaContext(data);
      ctx.ui.notify("Rendered live Focusa context", "info");
      pi.sendMessage({ customType: "focusa-context", content: rendered, display: true });
    },
  });

  // /focusa-settings — native settings UI
  pi.registerCommand("focusa-settings", {
    description: "Open Focusa settings panel",
    handler: async (_args, ctx) => {
      const draft = {
        contextStatusMode: S.cfg?.contextStatusMode || "actionable",
        warnPct: S.cfg?.warnPct || 50,
        compactPct: S.cfg?.compactPct || 70,
        hardPct: S.cfg?.hardPct || 85,
      };

      const buildItems = (): SettingItem[] => [
        { id: "contextStatusMode", label: "Footer context badge", currentValue: draft.contextStatusMode, values: ["off", "actionable", "all"] },
        { id: "warnPct", label: "Warn threshold %", currentValue: String(draft.warnPct), values: WARN_OPTIONS },
        { id: "compactPct", label: "Auto-compact threshold %", currentValue: String(draft.compactPct), values: COMPACT_OPTIONS },
        { id: "hardPct", label: "Critical threshold %", currentValue: String(draft.hardPct), values: HARD_OPTIONS },
      ];

      await ctx.ui.custom((_tui, theme, _kb, done) => {
        const container = new Container();
        container.addChild(new Text(theme.fg("accent", theme.bold("Focusa Settings")), 1, 1));

        const settingsList = new SettingsList(
          buildItems(),
          8,
          getSettingsListTheme(),
          (id, newValue) => {
            if (id === "contextStatusMode") draft.contextStatusMode = String(newValue) as any;
            if (id === "warnPct") draft.warnPct = Number(newValue);
            if (id === "compactPct") draft.compactPct = Number(newValue);
            if (id === "hardPct") draft.hardPct = Number(newValue);
            normalizeTierConfig(draft);
            const saved = saveConfigOverrides(ctx.cwd, draft, "project");
            S.cfg = saved.config;
            if (saved.errors.length) ctx.ui.notify(saved.errors.join("\n"), "warning");
            else ctx.ui.notify(`Saved Focusa settings → ${saved.path}`, "info");
          },
          () => done(undefined),
          { enableSearch: true },
        );
        container.addChild(settingsList);

        return {
          render: (w: number) => container.render(w),
          invalidate: () => container.invalidate(),
          handleInput: (data: string) => settingsList.handleInput?.(data),
        };
      });
    },
  });

  // /focusa-status (§10.3)
  pi.registerCommand("focusa-status", {
    description: "Show Focusa integration status",
    handler: async (_args, ctx) => {
      const up = S.focusaAvailable ? "✅ Connected" : "❌ Offline";
      const frame = S.activeFrameId ?? "none";
      const wbm = S.wbmEnabled ? (S.wbmDeep ? "deep" : S.wbmNoCatalogue ? "on (no-catalogue)" : "on") : "off";
      const tier = S.currentTier ? ` | Tier: ${S.currentTier.toUpperCase()}` : "";
      const compactions = S.totalCompactions ? ` | Compactions: ${S.totalCompactions}` : "";
      ctx.ui.notify(
        `Focusa: ${up}\nFrame: ${frame}\nWBM: ${wbm}\nTurns: ${S.turnCount}${tier}${compactions}\n` +
        `Decisions: ${S.localDecisions.length} | Constraints: ${S.localConstraints.length} | Failures: ${S.localFailures.length}` +
        (S.cfg ? `\nConfig: warn=${S.cfg.warnPct}% compact=${S.cfg.compactPct}% hard=${S.cfg.hardPct}%` : ""),
        "info",
      );
    },
  });

  // /focusa-on (§33.5) — re-enable Focusa writes after /focusa-off
  pi.registerCommand("focusa-on", {
    description: "Re-enable Focusa integration and writes",
    handler: async (_args, ctx) => {
      const h = await focusaFetch("/health");
      if (!h?.ok) {
        ctx.ui.notify("❌ Focusa unavailable", "error");
        return;
      }

      const alreadyEnabled = S.focusaAvailable;
      S.focusaAvailable = true;
      S.outageStart = null;
      S.healthBackoffMs = 30_000;

      if (!S.activeFrameId) {
        await ensurePiFrame(ctx.cwd, undefined, "pi-auto");
      }

      ctx.ui.setStatus("focusa", S.wbmEnabled ? "🤖 Focusa WBM" : "🧭 Focusa");
      if (S.activeFrameId) persistState();

      if (alreadyEnabled && S.activeFrameId) {
        ctx.ui.notify(`✅ Focusa already enabled — frame ready: ${S.activeFrameId}`, "info");
      } else if (S.activeFrameId) {
        ctx.ui.notify(`✅ Focusa enabled — frame ready: ${S.activeFrameId}`, "info");
      } else {
        ctx.ui.notify("⚠️ Focusa enabled but no Pi frame could be created", "warning");
      }
    },
  });

  // /focusa-off (§33.5) — stop ALL Focusa writes; keep reads for status only
  pi.registerCommand("focusa-off", {
    description: "Stop all Focusa writes — Focus State local only",
    handler: async (_args, ctx) => {
      if (!S.focusaAvailable) { ctx.ui.notify("Focusa already disabled", "info"); return; }
      S.focusaAvailable = false;
      ctx.ui.setStatus("focusa", "⏸️ Focusa disabled");
      ctx.ui.notify("⚠️ Focusa writes disabled — Focus State local only", "warning");
    },
  });

  // /focusa-reset (§33.5) — clear all Focus State entries in Focusa's DB + push fresh frame
  pi.registerCommand("focusa-reset", {
    description: "Clear Focus State in Focusa + push fresh Pi frame",
    handler: async (_args, ctx) => {
      const cleared = {
        decisions: S.localDecisions.length,
        constraints: S.localConstraints.length,
        failures: S.localFailures.length,
      };
      S.localDecisions = [];
      S.localConstraints = [];
      S.localFailures = [];
      S.compilationErrors = [];
      S.fileEditCounts = {};
      S.cataloguedDecisions = [];
      S.cataloguedFacts = [];
      S.compactResumePending = false;
      S.forkSuggested = false;
      S.currentTier = "";
      const previousFrameId = S.activeFrameId;
      S.activeFrameId = null;
      persistState();

      if (S.focusaAvailable && previousFrameId) {
        await focusaFetch("/focus/update", {
          method: "POST",
          body: JSON.stringify({
            frame_id: previousFrameId,
            turn_id: `pi-turn-${S.turnCount || 0}`,
            delta: { decisions: [], constraints: [], failures: [], recent_results: [] },
          }),
        }).catch(() => {});
      }

      if (S.focusaAvailable) {
        const frameId = await ensurePiFrame(ctx.cwd, undefined, "pi-reset");
        if (frameId) {
          S.activeFrameId = frameId;
          ctx.ui.notify(
            `✅ Focus State reset (cleared D:${cleared.decisions} C:${cleared.constraints} F:${cleared.failures})\nFresh Pi frame: ${frameId}`,
            "info",
          );
        } else {
          ctx.ui.notify(
            `✅ Local shadow cleared (D:${cleared.decisions} C:${cleared.constraints} F:${cleared.failures})\n⚠️ Focusa frame clear failed — writes may resume on old frame`,
            "warning",
          );
        }
      } else {
        ctx.ui.notify(
          `✅ Local shadow cleared (D:${cleared.decisions} C:${cleared.constraints} F:${cleared.failures})\n⚠️ Focusa offline — run /focusa-on to push fresh frame`,
          "warning",
        );
      }
    },
  });
}
