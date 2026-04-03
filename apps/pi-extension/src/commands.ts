// All /focusa-* slash commands
// Spec: §10.3 — Commands registry, §34.2E (explain-decision), §34.2F (lineage)
// Plus: §33.5 isolation commands: /focusa-on, /focusa-off, /focusa-reset

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { S, focusaFetch, getFocusState } from "./state.js";

export function registerCommands(pi: ExtensionAPI) {
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

  // /focusa-stack (§10.3)
  pi.registerCommand("focusa-stack", {
    description: "Show Focus Stack frames",
    handler: async (_args, ctx) => {
      if (!S.focusaAvailable) { ctx.ui.notify("Focusa offline", "warning"); return; }
      const data = await getFocusState();
      if (!data) { ctx.ui.notify("Empty stack", "info"); return; }
      const lines = data.stack.stack.frames.map((f: any, i: number) =>
        `${f.id === data.stack.active_frame_id ? "→ " : "  "}${i}: ${f.title || "(unnamed)"} [${f.id}]`,
      );
      ctx.ui.notify(lines.join("\n"), "info");
    },
  });

  // /focusa-pin <candidate_id> (§10.3, §22.2)
  pi.registerCommand("focusa-pin", {
    description: "Pin a Focus Gate candidate",
    handler: async (args, ctx) => {
      if (!S.focusaAvailable) { ctx.ui.notify("Focusa offline", "warning"); return; }
      const id = args?.trim();
      if (!id) { ctx.ui.notify("Usage: /focusa-pin <candidate_id>", "info"); return; }
      const r = await focusaFetch("/commands/submit", {
        method: "POST",
        body: JSON.stringify({ command: "gate.pin", args: { candidate_id: id }, idempotency_key: `pin-${id}-${Date.now()}` }),
      });
      ctx.ui.notify(r?.accepted ? `Pinned: ${id}` : "Pin failed", r?.accepted ? "info" : "error");
    },
  });

  // /focusa-suppress <candidate_id> [duration] (§10.3, §22.2)
  pi.registerCommand("focusa-suppress", {
    description: "Suppress a Focus Gate candidate",
    handler: async (args, ctx) => {
      if (!S.focusaAvailable) { ctx.ui.notify("Focusa offline", "warning"); return; }
      const parts = (args || "").trim().split(/\s+/);
      if (!parts[0]) { ctx.ui.notify("Usage: /focusa-suppress <candidate_id> [duration]", "info"); return; }
      const r = await focusaFetch("/commands/submit", {
        method: "POST",
        body: JSON.stringify({ command: "gate.suppress", args: { candidate_id: parts[0], duration: parts[1] || "10m" }, idempotency_key: `suppress-${parts[0]}-${Date.now()}` }),
      });
      ctx.ui.notify(r?.accepted ? `Suppressed: ${parts[0]}` : "Suppress failed", r?.accepted ? "info" : "error");
    },
  });

  // /focusa-checkpoint (§10.3)
  pi.registerCommand("focusa-checkpoint", {
    description: "Create ASCC checkpoint",
    handler: async (_args, ctx) => {
      if (!S.focusaAvailable) { ctx.ui.notify("Focusa offline", "warning"); return; }
      const r = await focusaFetch("/commands/submit", {
        method: "POST",
        body: JSON.stringify({ command: "ascc.checkpoint", args: {}, idempotency_key: `ckpt-${Date.now()}` }),
      });
      ctx.ui.notify(r?.accepted ? "✓ Checkpoint created" : "Checkpoint failed", r?.accepted ? "info" : "error");
    },
  });

  // /focusa-rehydrate <handle_id> [max_tokens] (§10.3)
  pi.registerCommand("focusa-rehydrate", {
    description: "Rehydrate ECS handle content",
    handler: async (args, ctx) => {
      if (!S.focusaAvailable) { ctx.ui.notify("Focusa offline", "warning"); return; }
      const parts = (args || "").trim().split(/\s+/);
      if (!parts[0]) { ctx.ui.notify("Usage: /focusa-rehydrate <handle_id> [max_tokens]", "info"); return; }
      const r = await focusaFetch(`/ecs/rehydrate?handle=${encodeURIComponent(parts[0])}&max_tokens=${parts[1] || "300"}`);
      ctx.ui.notify(r?.content ? `[${parts[0]}]:\n${String(r.content).slice(0, 2000)}` : "Handle not found or rehydrate failed", r?.content ? "info" : "error");
    },
  });

  // /focusa-gate-explain <candidate_id> (§10.3, §22.1)
  pi.registerCommand("focusa-gate-explain", {
    description: "Explain Focus Gate candidate scoring",
    handler: async (args, ctx) => {
      if (!S.focusaAvailable) { ctx.ui.notify("Focusa offline", "warning"); return; }
      const id = args?.trim();
      if (!id) { ctx.ui.notify("Usage: /focusa-gate-explain <candidate_id>", "info"); return; }
      const r = await focusaFetch(`/focus-gate/explain?candidate_id=${encodeURIComponent(id)}`);
      ctx.ui.notify(r ? JSON.stringify(r, null, 2).slice(0, 2000) : "Explain failed", r ? "info" : "error");
    },
  });

  // /focusa-explain-decision [query] (§34.2E)
  pi.registerCommand("focusa-explain-decision", {
    description: "Show/search recorded decisions",
    handler: async (args, ctx) => {
      const data = S.focusaAvailable ? await getFocusState() : null;
      const remote = data?.fs?.decisions || [];
      const all = [...remote, ...S.localDecisions];
      if (!all.length) { ctx.ui.notify("No decisions recorded", "info"); return; }
      const q = (args || "").trim().toLowerCase();
      const matched = q ? all.filter((d: string) => d.toLowerCase().includes(q)) : all;
      ctx.ui.notify(matched.length ? matched.map((d: string, i: number) => `${i + 1}. ${d}`).join("\n") : "No matches", "info");
    },
  });

  // /focusa-lineage (§34.2F)
  pi.registerCommand("focusa-lineage", {
    description: "Show CLT lineage path",
    handler: async (_args, ctx) => {
      if (!S.focusaAvailable) { ctx.ui.notify("Focusa offline", "warning"); return; }
      const r = await focusaFetch("/clt");
      if (!r?.nodes?.length) { ctx.ui.notify("No lineage nodes", "info"); return; }
      const lines = r.nodes.slice(-10).map((n: any) => `[${n.node_id}] ${n.node_type} ${n.created_at?.slice(0, 19) || ""}`);
      ctx.ui.notify(lines.join("\n"), "info");
    },
  });

  // /focusa-on (§33.5) — re-enable Focusa writes after /focusa-off
  pi.registerCommand("focusa-on", {
    description: "Re-enable Focusa integration and writes",
    handler: async (_args, ctx) => {
      if (S.focusaAvailable) { ctx.ui.notify("Focusa already enabled", "info"); return; }
      const h = await focusaFetch("/health");
      if (h?.ok) {
        S.focusaAvailable = true;
        S.outageStart = null;
        S.healthBackoffMs = 30_000;
        // Push a fresh Pi frame so writes go to clean slate
        if (!S.activeFrameId) {
          const r = await focusaFetch("/focus/push", {
            method: "POST",
            body: JSON.stringify({ title: `Pi session in ${ctx.cwd}`, source: "pi-auto" }),
          });
          if (r?.frame_id) S.activeFrameId = r.frame_id;
        }
        ctx.ui.setStatus("focusa", S.wbmEnabled ? "🧠 Focusa [WBM]" : "🧠 Focusa");
        ctx.ui.notify("✅ Focusa re-enabled", "info");
      } else {
        ctx.ui.notify("❌ Focusa unavailable", "error");
      }
    },
  });

  // /focusa-off (§33.5) — stop ALL Focusa writes; keep reads for status only
  pi.registerCommand("focusa-off", {
    description: "Stop all Focusa writes — Focus State local only",
    handler: async (_args, ctx) => {
      if (!S.focusaAvailable) { ctx.ui.notify("Focusa already disabled", "info"); return; }
      S.focusaAvailable = false;
      // Leave local shadow intact — operator can still see decisions via status
      ctx.ui.setStatus("focusa", "🧠 Focusa [disabled]");
      ctx.ui.notify("⚠️ Focusa writes disabled — Focus State local only", "warning");
    },
  });

  // /focusa-reset (§33.5) — clear all Focus State entries in Focusa's DB + push fresh frame
  // Use when Focus State is polluted or stale. Wipes decisions/constraints/failures.
  pi.registerCommand("focusa-reset", {
    description: "Clear Focus State in Focusa + push fresh Pi frame",
    handler: async (_args, ctx) => {
      // Step 1: Clear local shadow
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

      // Step 2: Clear Focusa frame (if available and we have a frame)
      if (S.activeFrameId) {
        await focusaFetch("/focus/update", {
          method: "POST",
          body: JSON.stringify({
            frame_id: S.activeFrameId,
            turn_id: `pi-turn-${S.turnCount}`,
            delta: {
              decisions: [],
              constraints: [],
              failures: [],
              notes: [],
              open_questions: [],
              next_steps: [],
              recent_results: [],
            },
          }),
        }).catch(() => {});
      }

      // Step 3: Push fresh Pi frame (always, ensures clean slate)
      if (S.focusaAvailable) {
        const r = await focusaFetch("/focus/push", {
          method: "POST",
          body: JSON.stringify({ title: `Pi session in ${ctx.cwd}`, source: "pi-reset" }),
        });
        if (r?.frame_id) {
          S.activeFrameId = r.frame_id;
          ctx.ui.notify(
            `✅ Focus State reset (cleared D:${cleared.decisions} C:${cleared.constraints} F:${cleared.failures})\nFresh Pi frame: ${r.frame_id}`,
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
