// All /focusa-* slash commands
// Spec: §10.3 — Commands registry, §34.2E (explain-decision), §34.2F (lineage)

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
}
