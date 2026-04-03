// FOCUSA_SCRATCHPAD: two-file model
// Spec: G1-07 §AsccSections + doc 44 §10.5 + §Forbidden
//
// LESSON (live evidence 2026-04-03, read every word):
// Decision rationale: 50-char limit → agent wrote 200+ char task lists in rationale field
// Decision fields: reformatted task lists with "Fix all", "Tool-level guardrails"
// Constraints: ALL 30+ are agent's own self-referential stream-of-consciousness
//   — including agent's own root-cause analysis and meta-observation about this pollution
// Both guardrails FAILED. Agent worked around both.
//
// ROOT CAUSE: Agent needs a scratchpad. The tool fields became it.
//
// TWO-FILE MODEL:
//   /tmp/pi-scratch/<turn>/notes.txt  → agent's FULL working notebook (unlimited, no Focus State)
//   Focus State (Focusa)               → operator-curated decisions only
//
// Extension = thin bridge. Focus State = operator manages. Agent uses scratchpad.

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { Type } from "@sinclair/typebox";
import { S } from "./state.js";

const SCRATCHPAD_DIR = "/tmp/pi-scratch";

function scratchDir(turn: number): string {
  return `${SCRATCHPAD_DIR}/turn-${String(turn).padStart(4, "0")}`;
}

function ensureScratchDir(): void {
  try {
    const { execSync } = require("child_process");
    execSync(`mkdir -p "${SCRATCHPAD_DIR}"`, { stdio: "pipe" });
  } catch { /* best effort */ }
}

export function registerTools(pi: ExtensionAPI) {
  // ── focusa_scratch ──────────────────────────────────────────────────────
  // Agent's working notebook. Lives at /tmp/pi-scratch/. No Focus State write.
  // ALL working notes welcome: reasoning, task lists, hypotheses, dead ends,
  // self-corrections, design notes, NEXT:/Signal: directives.
  // Operator can read: ls /tmp/pi-scratch/ | cat /tmp/pi-scratch/turn-NNNN/notes.txt
  pi.registerTool({
    name: "focusa_scratch",
    label: "Scratchpad",
    description: "Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to /focusa-decide when done.",
    promptSnippet: "Working notes → scratchpad. Crystallized decision → /focusa-decide command.",
    parameters: Type.Object({
      note: Type.String({ description: "Working note — reasoning, task list, hypothesis, dead end. Unlimited length." }),
      tag: Type.Optional(Type.String({ description: "Tag: reasoning|task|hypothesis|dead-end|self-correction|next-step" })),
    }),
    promptGuidelines: [
      "ALL working notes go HERE. scratchpad ≠ Focus State.",
      "NEXT:/Signal: directives, task lists, design notes, self-corrections → here.",
      "When done: distill ONE crystallized sentence → /focusa-decide (manual command, operator approves).",
      "Scratchpad is your working notebook. Focus State is operator's decision journal.",
      "Run: ls /tmp/pi-scratch/ | cat /tmp/pi-scratch/turn-NNNN/notes.txt",
    ],
    async execute(_id, params) {
      const { note, tag } = params as { note: string; tag?: string };
      const turn = S.turnCount;
      const dir = scratchDir(turn);
      ensureScratchDir();
      const ts = new Date().toISOString().slice(11, 23);
      const line = `[${ts}]${tag ? ` [${tag}]` : ""} ${note}`;
      try {
        const { execSync } = require("child_process");
        execSync(`mkdir -p "${dir}" && echo ${JSON.stringify(line)} >> "${dir}/notes.txt"`, { stdio: "pipe" });
      } catch { /* best effort */ }
      return {
        content: [{ type: "text" as const, text: `📝 Scratchpad saved (turn ${turn}): ${note.slice(0, 80)}${note.length > 80 ? "…" : ""}` }],
        details: { note, tag, turn },
      };
    },
  });

  // ── focusa_decide ────────────────────────────────────────────────────────
  // DISABLED. Agent writes task lists in rationale field. Strip ALL Focus State writes.
  // Operator: use /focusa-decide command (manual, no auto-write).
  // Agent: use focusa_scratch for all notes. Operator manages Focus State.
  void 0;

  // ── focusa_constraint ────────────────────────────────────────────────────
  // DISABLED. ALL 30+ entries are agent's own self-referential monologue.
  // Agent even recorded its own meta-analysis of this problem as a constraint.
  // §AsccSections: "constraints = DISCOVERED REQUIREMENTS." Not agent monologue.
  void 0;

  // ── focusa_failure ───────────────────────────────────────────────────────
  // DISABLED. Strip ALL Focus State writes from extension.
  // Actual failures reported by operator → /focusa-failure command.
  void 0;
}
