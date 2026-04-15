// All /focusa-* slash commands
// Spec: §10.3 — Commands registry, §34.2E (explain-decision), §34.2F (lineage)
// Plus: §33.5 isolation commands: /focusa-on, /focusa-off, /focusa-reset

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { getSettingsListTheme } from "@mariozechner/pi-coding-agent";
import { Container, Text, type SettingItem, SettingsList } from "@mariozechner/pi-tui";
import { S, focusaFetch, getFocusState, getEffectiveFocusSnapshot, persistState, persistAuthoritativeState, createPiFrame, ensurePiFrame } from "./state.js";
import { saveConfigOverrides } from "./config.js";

function nonEmptyLines(items: any[] | undefined): string[] {
  return (items || []).map((v) => String(v || "").trim()).filter(Boolean);
}

const WARN_OPTIONS = ["40", "50", "60", "70"];
const COMPACT_OPTIONS = ["60", "70", "80", "85", "90"];
const HARD_OPTIONS = ["75", "85", "92", "95", "97"];
const WORK_LOOP_PRESET_OPTIONS = ["conservative", "balanced", "push", "audit"];
const WORK_LOOP_TURN_OPTIONS = ["6", "10", "12", "24"];
const WORK_LOOP_WALL_CLOCK_OPTIONS = ["900000", "1200000", "1800000", "3600000"];
const WORK_LOOP_RETRY_OPTIONS = ["1", "2", "3", "4"];
const WORK_LOOP_COOLDOWN_OPTIONS = ["500", "1000", "1500", "2000"];
const WORK_LOOP_LOW_PRODUCTIVITY_OPTIONS = ["2", "3", "4"];
const WORK_LOOP_FAILURE_OPTIONS = ["2", "3", "4"];
const WORK_LOOP_SAME_SUBPROBLEM_OPTIONS = ["1", "2", "3"];
const WORK_LOOP_HEARTBEAT_OPTIONS = ["2000", "3000", "5000"];
const BOOLEAN_OPTIONS = ["true", "false"];

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

  const currentFocus = String(fs?.current_focus || fs?.current_state || "").trim();
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

  const nextSteps = nonEmptyLines(fs?.next_steps);
  if (nextSteps.length) {
    lines.push("## Next Steps");
    lines.push(...nextSteps.map((item) => `- ${item}`));
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
    handler: async (args, ctx) => {
      const simpleProfiles = ["starter", "builder", "hands_off", "audit_safe"] as const;
      type SimpleProfileId = typeof simpleProfiles[number];
      const advancedMode = /\badvanced\b/i.test(String(args || ""));

      const draft = {
        contextStatusMode: S.cfg?.contextStatusMode || "actionable",
        warnPct: S.cfg?.warnPct || 50,
        compactPct: S.cfg?.compactPct || 70,
        hardPct: S.cfg?.hardPct || 85,
        workLoopPreset: S.cfg?.workLoopPreset || "balanced",
        workLoopMaxTurns: S.cfg?.workLoopMaxTurns || 12,
        workLoopMaxWallClockMs: S.cfg?.workLoopMaxWallClockMs || 1_800_000,
        workLoopMaxRetries: S.cfg?.workLoopMaxRetries || 3,
        workLoopCooldownMs: S.cfg?.workLoopCooldownMs || 1_000,
        workLoopAllowDestructiveActions: S.cfg?.workLoopAllowDestructiveActions || false,
        workLoopRequireOperatorForGovernance: S.cfg?.workLoopRequireOperatorForGovernance ?? true,
        workLoopRequireOperatorForScopeChange: S.cfg?.workLoopRequireOperatorForScopeChange ?? true,
        workLoopRequireVerificationBeforePersist: S.cfg?.workLoopRequireVerificationBeforePersist ?? true,
        workLoopMaxConsecutiveLowProductivityTurns: S.cfg?.workLoopMaxConsecutiveLowProductivityTurns || 3,
        workLoopMaxConsecutiveFailures: S.cfg?.workLoopMaxConsecutiveFailures || 3,
        workLoopAutoPauseOnOperatorMessage: S.cfg?.workLoopAutoPauseOnOperatorMessage ?? true,
        workLoopRequireExplainableContinueReason: S.cfg?.workLoopRequireExplainableContinueReason ?? true,
        workLoopMaxSameSubproblemRetries: S.cfg?.workLoopMaxSameSubproblemRetries || 2,
        workLoopStatusHeartbeatMs: S.cfg?.workLoopStatusHeartbeatMs || 5_000,
      };

      const applySimpleProfile = (profile: SimpleProfileId) => {
        if (profile === "starter") {
          draft.workLoopPreset = "conservative";
          draft.workLoopMaxTurns = 10;
          draft.workLoopMaxWallClockMs = 1_200_000;
          draft.workLoopMaxRetries = 2;
          draft.workLoopCooldownMs = 1_500;
          draft.workLoopMaxConsecutiveLowProductivityTurns = 2;
          draft.workLoopMaxConsecutiveFailures = 2;
          draft.workLoopMaxSameSubproblemRetries = 1;
          draft.workLoopStatusHeartbeatMs = 3_000;
          draft.contextStatusMode = "actionable";
        }
        if (profile === "builder") {
          draft.workLoopPreset = "balanced";
          draft.workLoopMaxTurns = 24;
          draft.workLoopMaxWallClockMs = 3_600_000;
          draft.workLoopMaxRetries = 3;
          draft.workLoopCooldownMs = 1_000;
          draft.workLoopMaxConsecutiveLowProductivityTurns = 3;
          draft.workLoopMaxConsecutiveFailures = 3;
          draft.workLoopMaxSameSubproblemRetries = 2;
          draft.workLoopStatusHeartbeatMs = 2_000;
          draft.contextStatusMode = "actionable";
        }
        if (profile === "hands_off") {
          draft.workLoopPreset = "push";
          draft.workLoopMaxTurns = 120;
          draft.workLoopMaxWallClockMs = 14_400_000;
          draft.workLoopMaxRetries = 8;
          draft.workLoopCooldownMs = 800;
          draft.workLoopMaxConsecutiveLowProductivityTurns = 4;
          draft.workLoopMaxConsecutiveFailures = 4;
          draft.workLoopMaxSameSubproblemRetries = 4;
          draft.workLoopStatusHeartbeatMs = 1_500;
          draft.contextStatusMode = "actionable";
        }
        if (profile === "audit_safe") {
          draft.workLoopPreset = "audit";
          draft.workLoopMaxTurns = 16;
          draft.workLoopMaxWallClockMs = 3_600_000;
          draft.workLoopMaxRetries = 2;
          draft.workLoopCooldownMs = 1_500;
          draft.workLoopMaxConsecutiveLowProductivityTurns = 2;
          draft.workLoopMaxConsecutiveFailures = 2;
          draft.workLoopMaxSameSubproblemRetries = 1;
          draft.workLoopStatusHeartbeatMs = 3_000;
          draft.contextStatusMode = "all";
        }

        draft.workLoopAllowDestructiveActions = false;
        draft.workLoopRequireOperatorForGovernance = true;
        draft.workLoopRequireOperatorForScopeChange = true;
        draft.workLoopRequireVerificationBeforePersist = true;
        // Steering should redirect the loop, not freeze it; hard pauses remain policy-gated.
        draft.workLoopAutoPauseOnOperatorMessage = false;
        draft.workLoopRequireExplainableContinueReason = true;
      };

      const inferSimpleProfile = (): SimpleProfileId => {
        if (draft.workLoopPreset === "push" && draft.workLoopMaxTurns >= 60) return "hands_off";
        if (draft.workLoopPreset === "audit") return "audit_safe";
        if (draft.workLoopPreset === "conservative") return "starter";
        return "builder";
      };

      let simpleProfile: SimpleProfileId = inferSimpleProfile();

      const persistDraft = () => {
        normalizeTierConfig(draft);
        const saved = saveConfigOverrides(ctx.cwd, draft, "project");
        S.cfg = saved.config;
        if (saved.errors.length) ctx.ui.notify(saved.errors.join("\n"), "warning");
        else ctx.ui.notify(`Saved Focusa settings → ${saved.path}`, "info");
      };

      const buildSimpleItems = (): SettingItem[] => [
        {
          id: "simpleProfile",
          label: "Quick profile",
          currentValue: simpleProfile,
          values: [
            "starter",
            "builder",
            "hands_off",
            "audit_safe",
          ],
        },
        { id: "workLoopMaxTurns", label: "How many turns before pause", currentValue: String(draft.workLoopMaxTurns), values: ["10", "24", "60", "120", "200"] },
        { id: "workLoopMaxWallClockMs", label: "Max run time (ms)", currentValue: String(draft.workLoopMaxWallClockMs), values: ["1200000", "3600000", "7200000", "14400000"] },
        { id: "workLoopStatusHeartbeatMs", label: "Refresh heartbeat (ms)", currentValue: String(draft.workLoopStatusHeartbeatMs), values: ["1500", "2000", "3000", "5000"] },
        { id: "contextStatusMode", label: "Footer hints", currentValue: draft.contextStatusMode, values: ["off", "actionable", "all"] },
        { id: "workLoopRequireVerificationBeforePersist", label: "Require verification before done", currentValue: String(draft.workLoopRequireVerificationBeforePersist), values: BOOLEAN_OPTIONS },
      ];

      const buildAdvancedItems = (): SettingItem[] => [
        { id: "contextStatusMode", label: "Footer context badge", currentValue: draft.contextStatusMode, values: ["off", "actionable", "all"] },
        { id: "warnPct", label: "Warn threshold %", currentValue: String(draft.warnPct), values: WARN_OPTIONS },
        { id: "compactPct", label: "Auto-compact threshold %", currentValue: String(draft.compactPct), values: COMPACT_OPTIONS },
        { id: "hardPct", label: "Critical threshold %", currentValue: String(draft.hardPct), values: HARD_OPTIONS },
        { id: "workLoopPreset", label: "Work-loop preset", currentValue: draft.workLoopPreset, values: WORK_LOOP_PRESET_OPTIONS },
        { id: "workLoopMaxTurns", label: "Work-loop max turns", currentValue: String(draft.workLoopMaxTurns), values: WORK_LOOP_TURN_OPTIONS },
        { id: "workLoopMaxWallClockMs", label: "Work-loop max wall clock ms", currentValue: String(draft.workLoopMaxWallClockMs), values: WORK_LOOP_WALL_CLOCK_OPTIONS },
        { id: "workLoopMaxRetries", label: "Work-loop retries", currentValue: String(draft.workLoopMaxRetries), values: WORK_LOOP_RETRY_OPTIONS },
        { id: "workLoopCooldownMs", label: "Work-loop cooldown ms", currentValue: String(draft.workLoopCooldownMs), values: WORK_LOOP_COOLDOWN_OPTIONS },
        { id: "workLoopAllowDestructiveActions", label: "Work-loop allow destructive actions", currentValue: String(draft.workLoopAllowDestructiveActions), values: BOOLEAN_OPTIONS },
        { id: "workLoopRequireOperatorForGovernance", label: "Work-loop require operator for governance", currentValue: String(draft.workLoopRequireOperatorForGovernance), values: BOOLEAN_OPTIONS },
        { id: "workLoopRequireOperatorForScopeChange", label: "Work-loop require operator for scope change", currentValue: String(draft.workLoopRequireOperatorForScopeChange), values: BOOLEAN_OPTIONS },
        { id: "workLoopRequireVerificationBeforePersist", label: "Work-loop require verification before persist", currentValue: String(draft.workLoopRequireVerificationBeforePersist), values: BOOLEAN_OPTIONS },
        { id: "workLoopMaxConsecutiveLowProductivityTurns", label: "Work-loop max low-productivity turns", currentValue: String(draft.workLoopMaxConsecutiveLowProductivityTurns), values: WORK_LOOP_LOW_PRODUCTIVITY_OPTIONS },
        { id: "workLoopMaxConsecutiveFailures", label: "Work-loop max consecutive failures", currentValue: String(draft.workLoopMaxConsecutiveFailures), values: WORK_LOOP_FAILURE_OPTIONS },
        { id: "workLoopAutoPauseOnOperatorMessage", label: "Work-loop auto-pause on operator message", currentValue: String(draft.workLoopAutoPauseOnOperatorMessage), values: BOOLEAN_OPTIONS },
        { id: "workLoopRequireExplainableContinueReason", label: "Work-loop require explainable continue reason", currentValue: String(draft.workLoopRequireExplainableContinueReason), values: BOOLEAN_OPTIONS },
        { id: "workLoopMaxSameSubproblemRetries", label: "Work-loop max same-subproblem retries", currentValue: String(draft.workLoopMaxSameSubproblemRetries), values: WORK_LOOP_SAME_SUBPROBLEM_OPTIONS },
        { id: "workLoopStatusHeartbeatMs", label: "Work-loop status heartbeat ms", currentValue: String(draft.workLoopStatusHeartbeatMs), values: WORK_LOOP_HEARTBEAT_OPTIONS },
      ];

      await ctx.ui.custom((_tui, theme, _kb, done) => {
        const container = new Container();
        container.addChild(new Text(theme.fg("accent", theme.bold(advancedMode ? "Focusa Settings (Advanced)" : "🍎 Focusa Quick Setup")), 1, 1));
        if (!advancedMode) {
          container.addChild(new Text(theme.fg("dim", "Preset-first setup for beginners. Run /focusa-settings advanced for full controls."), 1, 3));
        }

        const settingsList = new SettingsList(
          advancedMode ? buildAdvancedItems() : buildSimpleItems(),
          advancedMode ? 8 : 10,
          getSettingsListTheme(),
          (id, newValue) => {
            if (id === "simpleProfile") {
              simpleProfile = String(newValue) as SimpleProfileId;
              applySimpleProfile(simpleProfile);
              persistDraft();
              return;
            }
            if (id === "contextStatusMode") draft.contextStatusMode = String(newValue) as any;
            if (id === "warnPct") draft.warnPct = Number(newValue);
            if (id === "compactPct") draft.compactPct = Number(newValue);
            if (id === "hardPct") draft.hardPct = Number(newValue);
            if (id === "workLoopPreset") draft.workLoopPreset = String(newValue) as any;
            if (id === "workLoopMaxTurns") draft.workLoopMaxTurns = Number(newValue);
            if (id === "workLoopMaxWallClockMs") draft.workLoopMaxWallClockMs = Number(newValue);
            if (id === "workLoopMaxRetries") draft.workLoopMaxRetries = Number(newValue);
            if (id === "workLoopCooldownMs") draft.workLoopCooldownMs = Number(newValue);
            if (id === "workLoopAllowDestructiveActions") draft.workLoopAllowDestructiveActions = String(newValue) === "true";
            if (id === "workLoopRequireOperatorForGovernance") draft.workLoopRequireOperatorForGovernance = String(newValue) === "true";
            if (id === "workLoopRequireOperatorForScopeChange") draft.workLoopRequireOperatorForScopeChange = String(newValue) === "true";
            if (id === "workLoopRequireVerificationBeforePersist") draft.workLoopRequireVerificationBeforePersist = String(newValue) === "true";
            if (id === "workLoopMaxConsecutiveLowProductivityTurns") draft.workLoopMaxConsecutiveLowProductivityTurns = Number(newValue);
            if (id === "workLoopMaxConsecutiveFailures") draft.workLoopMaxConsecutiveFailures = Number(newValue);
            if (id === "workLoopAutoPauseOnOperatorMessage") draft.workLoopAutoPauseOnOperatorMessage = String(newValue) === "true";
            if (id === "workLoopRequireExplainableContinueReason") draft.workLoopRequireExplainableContinueReason = String(newValue) === "true";
            if (id === "workLoopMaxSameSubproblemRetries") draft.workLoopMaxSameSubproblemRetries = Number(newValue);
            if (id === "workLoopStatusHeartbeatMs") draft.workLoopStatusHeartbeatMs = Number(newValue);
            persistDraft();
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
      const focusState = await getFocusState();
      const titleLine = S.activeFrameTitle ? `\nTitle: ${S.activeFrameTitle}` : "";
      const goalLine = S.activeFrameGoal ? `\nGoal: ${S.activeFrameGoal}` : "";
      const loop = await focusaFetch("/work-loop");
      const loopLine = loop ? `\nLoop: ${loop.enabled ? "on" : "off"} | Status: ${loop.status} | Project: ${loop.project_status} | Tranche: ${loop.tranche_status}` : "";
      const whyLine = loop?.last_continue_reason || loop?.last_blocker_reason ? `\nWhy: ${loop.last_continue_reason || loop.last_blocker_reason}` : "";
      const budgetLine = loop?.budget_remaining ? `\nBudget: retries=${loop.budget_remaining.max_retries} remaining_failure_budget=${loop.budget_remaining.remaining_failure_budget}` : "";
      const checkpointLine = loop?.last_checkpoint_id ? `\nCheckpoint: ${loop.last_checkpoint_id}` : "";
      const supervisionLine = loop?.transport?.daemon_supervised_session ? `\nSupervision: daemon-owned ${loop.transport.daemon_supervised_session.session_id}` : "\nSupervision: none";
      const snapshot = getEffectiveFocusSnapshot(focusState?.fs);
      const missionLine = snapshot.intent ? `\nMission: ${snapshot.intent}` : "";
      const focusLine = snapshot.currentFocus ? `\nFocus: ${snapshot.currentFocus}` : "";
      ctx.ui.notify(
        `Focusa: ${up}\nFrame: ${frame}${titleLine}${goalLine}\nWBM: ${wbm}\nTurns: ${S.turnCount}${tier}${compactions}` + loopLine + whyLine + budgetLine + checkpointLine + supervisionLine + missionLine + focusLine + `\n` +
        `Decisions: ${snapshot.decisions.length} | Constraints: ${snapshot.constraints.length} | Failures: ${snapshot.failures.length}` +
        (S.cfg ? `\nConfig: warn=${S.cfg.warnPct}% compact=${S.cfg.compactPct}% hard=${S.cfg.hardPct}% | work-loop=${S.cfg.workLoopPreset}` : ""),
        "info",
      );
    },
  });

  pi.registerCommand("focus-work", {
    description: "Continuous work loop controls: on|off|pause|resume|stop|status|checkpoint|checkpoints",
    handler: async (args, ctx) => {
      const parts = String(args || "").trim().split(/\s+/).filter(Boolean);
      const sub = String(parts[0] || "status").toLowerCase();
      const rest = parts.slice(1).join(" ").trim();
      if (sub === "on") {
        const payload = {
          preset: S.cfg?.workLoopPreset || "balanced",
          policy_overrides: {
            max_turns: S.cfg?.workLoopMaxTurns,
            max_wall_clock_ms: S.cfg?.workLoopMaxWallClockMs,
            max_retries: S.cfg?.workLoopMaxRetries,
            cooldown_ms: S.cfg?.workLoopCooldownMs,
            allow_destructive_actions: S.cfg?.workLoopAllowDestructiveActions,
            require_operator_for_governance: S.cfg?.workLoopRequireOperatorForGovernance,
            require_operator_for_scope_change: S.cfg?.workLoopRequireOperatorForScopeChange,
            require_verification_before_persist: S.cfg?.workLoopRequireVerificationBeforePersist,
            max_consecutive_low_productivity_turns: S.cfg?.workLoopMaxConsecutiveLowProductivityTurns,
            max_consecutive_failures: S.cfg?.workLoopMaxConsecutiveFailures,
            auto_pause_on_operator_message: S.cfg?.workLoopAutoPauseOnOperatorMessage,
            require_explainable_continue_reason: S.cfg?.workLoopRequireExplainableContinueReason,
            max_same_subproblem_retries: S.cfg?.workLoopMaxSameSubproblemRetries,
            status_heartbeat_ms: S.cfg?.workLoopStatusHeartbeatMs,
          },
        };
        const res = await focusaFetch("/work-loop/enable", { method: "POST", headers: { "x-focusa-writer-id": `pi-${process.pid}`, "x-focusa-approval": "approved" }, body: JSON.stringify(payload) });
        ctx.ui.notify(`focus-work on → ${res?.status || res?.ok || "unknown"}`, "info");
        return;
      }
      if (sub === "pause") {
        const res = await focusaFetch("/work-loop/pause", { method: "POST", headers: { "x-focusa-writer-id": `pi-${process.pid}` }, body: JSON.stringify({ reason: "operator pause via /focus-work" }) });
        ctx.ui.notify(`focus-work pause → ${res?.status || res?.ok || "unknown"}`, "info");
        return;
      }
      if (sub === "resume") {
        const res = await focusaFetch("/work-loop/resume", { method: "POST", headers: { "x-focusa-writer-id": `pi-${process.pid}` }, body: JSON.stringify({ reason: "operator resume via /focus-work" }) });
        ctx.ui.notify(`focus-work resume → ${res?.status || res?.ok || "unknown"}`, "info");
        return;
      }
      if (sub === "off" || sub === "stop") {
        const res = await focusaFetch("/work-loop/stop", { method: "POST", headers: { "x-focusa-writer-id": `pi-${process.pid}` }, body: JSON.stringify({ reason: `operator ${sub} via /focus-work` }) });
        ctx.ui.notify(`focus-work ${sub} → ${res?.status || res?.ok || "unknown"}`, "info");
        return;
      }
      if (sub === "checkpoint") {
        const payload = rest
          ? { summary: `operator checkpoint via /focus-work: ${rest}` }
          : { summary: "operator checkpoint via /focus-work" };
        const res = await focusaFetch("/work-loop/checkpoint", {
          method: "POST",
          headers: { "x-focusa-writer-id": `pi-${process.pid}` },
          body: JSON.stringify(payload),
        });
        ctx.ui.notify(`focus-work checkpoint → ${res?.checkpoint_id || res?.status || res?.ok || "unknown"}`, "info");
        return;
      }
      if (sub === "checkpoints") {
        const res = await focusaFetch("/work-loop/checkpoints");
        const checkpoints = Array.isArray(res?.checkpoints) ? res.checkpoints : [];
        const lines = checkpoints.slice(0, 5).map((c: any) => `- ${c?.id || "(id?)"}: ${c?.summary || "(no summary)"}`);
        ctx.ui.notify(lines.length ? `Recent checkpoints (${checkpoints.length})\n${lines.join("\n")}` : "No checkpoints available", "info");
        return;
      }
      const fs = await getFocusState();
      const loop = await focusaFetch("/work-loop");
      const snapshot = getEffectiveFocusSnapshot(fs?.fs);
      const mission = snapshot.intent || "(none)";
      const focus = snapshot.currentFocus || "(none)";
      ctx.ui.notify(loop ? `Loop: ${loop.enabled ? "on" : "off"}\nStatus: ${loop.status}\nProject: ${loop.project_status}\nTranche: ${loop.tranche_status}\nMission: ${mission}\nFocus: ${focus}\nReason: ${loop.last_continue_reason || loop.last_blocker_reason || "(none)"}\nCheckpoint: ${loop.last_checkpoint_id || "(none)"}\nSupervision: ${loop.transport?.daemon_supervised_session?.session_id || "(none)"}\nPreset: ${loop.policy?.preset || S.cfg?.workLoopPreset || "balanced"}` : "Loop status unavailable", "info");
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

      const status = await focusaFetch("/status").catch(() => null);
      if (status?.session?.status !== "active") {
        await focusaFetch("/session/start", {
          method: "POST",
          body: JSON.stringify({ adapter_id: "pi", workspace_id: ctx.cwd || S.sessionCwd || "pi-workspace" }),
        }).catch(() => null);
      }

      if (!S.activeFrameId) {
        await ensurePiFrame(ctx.cwd, undefined, "pi-auto");
      }

      ctx.ui.setStatus("focusa", S.wbmEnabled ? "🤖 Focusa WBM" : "🧭 Focusa");
      if (S.activeFrameId) await persistAuthoritativeState();

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
      const clearedSnapshot = getEffectiveFocusSnapshot();
      const cleared = {
        decisions: clearedSnapshot.decisions.length,
        constraints: clearedSnapshot.constraints.length,
        failures: clearedSnapshot.failures.length,
      };
      S.localDecisions = [];
      S.localConstraints = [];
      S.localFailures = [];
      S.lastFocusSnapshot = { decisions: [], constraints: [], failures: [], intent: "", currentFocus: "" };
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
          await persistAuthoritativeState();
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
