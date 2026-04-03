// Metacognition tools: focusa_decide, focusa_constraint, focusa_failure
// Spec: §37.2 — Tools > text markers for decision capture

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { Type } from "@sinclair/typebox";
import { S, focusaFetch } from "./state.js";

export function registerTools(pi: ExtensionAPI) {
  pi.registerTool({
    name: "focusa_decide",
    label: "Record Decision",
    description: "Record a significant decision in Focusa's cognitive state. Use when you make an architectural choice, select an approach, or commit to a direction.",
    promptSnippet: "Record decisions, constraints, and failures for cognitive tracking",
    parameters: Type.Object({
      decision: Type.String({ description: "The decision made" }),
      rationale: Type.String({ description: "Why this decision was made" }),
      alternatives: Type.Optional(Type.Array(Type.String(), { description: "Alternatives that were considered" })),
    }),
    promptGuidelines: [
      "When you make a significant architectural choice, select an approach, or commit to a direction, call focusa_decide.",
    ],
    async execute(_id, params) {
      const { decision, rationale, alternatives } = params as { decision: string; rationale: string; alternatives?: string[] };
      const alts = alternatives?.length ? ` [alternatives: ${alternatives.join(", ")}]` : "";
      const text = `${decision} (because: ${rationale})${alts}`;
      S.localDecisions.push(text);
      if (S.wbmEnabled) S.cataloguedDecisions.push(text);
      if (S.focusaAvailable && S.activeFrameId) {
        await focusaFetch("/focus/update", {
          method: "POST",
          body: JSON.stringify({ frame_id: S.activeFrameId, turn_id: `pi-turn-${S.turnCount}`, delta: { decisions: [text] } }),
        });
      }
      return { content: [{ type: "text", text: `✓ Decision recorded: ${decision}` }], details: { decision, rationale } };
    },
  });

  pi.registerTool({
    name: "focusa_constraint",
    label: "Record Constraint",
    description: "Record a constraint discovered during work. Use when you find a limitation, requirement, or rule that affects future decisions.",
    promptSnippet: "Record constraints that affect future decisions",
    parameters: Type.Object({
      constraint: Type.String({ description: "The constraint discovered" }),
      source: Type.String({ description: "Where this constraint comes from" }),
    }),
    promptGuidelines: [
      "When you discover a limitation, requirement, or hard rule that affects future work, call focusa_constraint.",
    ],
    async execute(_id, params) {
      const { constraint, source } = params as { constraint: string; source: string };
      const text = `${constraint} (source: ${source})`;
      S.localConstraints.push(text);
      if (S.focusaAvailable && S.activeFrameId) {
        await focusaFetch("/focus/update", {
          method: "POST",
          body: JSON.stringify({ frame_id: S.activeFrameId, turn_id: `pi-turn-${S.turnCount}`, delta: { constraints: [text] } }),
        });
      }
      return { content: [{ type: "text", text: `✓ Constraint recorded: ${constraint}` }], details: { constraint, source } };
    },
  });

  pi.registerTool({
    name: "focusa_failure",
    label: "Record Failure",
    description: "Record a failure or error for learning. Use when something goes wrong — build errors, test failures, wrong assumptions.",
    promptSnippet: "Record failures for learning and pattern detection",
    parameters: Type.Object({
      failure: Type.String({ description: "What failed" }),
      context: Type.String({ description: "What was being attempted" }),
    }),
    promptGuidelines: [
      "When something fails — build errors, test failures, wrong assumptions — call focusa_failure.",
    ],
    async execute(_id, params) {
      const { failure, context: ctx } = params as { failure: string; context: string };
      const text = `${failure} (during: ${ctx})`;
      S.localFailures.push(text);
      if (S.focusaAvailable && S.activeFrameId) {
        await focusaFetch("/focus/update", {
          method: "POST",
          body: JSON.stringify({ frame_id: S.activeFrameId, turn_id: `pi-turn-${S.turnCount}`, delta: { failures: [text] } }),
        });
      }
      return { content: [{ type: "text", text: `✓ Failure recorded: ${failure}` }], details: { failure, context: ctx } };
    },
  });
}
