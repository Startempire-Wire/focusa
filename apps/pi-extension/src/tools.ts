// FOCUSA_SCRATCHPAD: two-file model
// Spec: G1-07 §AsccSections + doc 44 §10.5 + §Forbidden
//
// The two-file model:
//   /tmp/pi-scratch/<turn>/notes.txt  → agent's FULL working notebook (unlimited, no Focus State)
//   Focus State (Focusa)               → operator-curated cognitive state only
//
// Extension = thin bridge. Focus State = operator manages.
// Agent uses scratchpad for working notes. Operator manages Focus State.

import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";
import { Type } from "@sinclair/typebox";
import { S, focusaFetch, focusaPost } from "./state.js";

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

// ─────────────────────────────────────────────────────────────────────────────
// Validation helpers — per §AsccSections and G1-07 Delta Summarization Rule
// The agent IS the summarizer (LLM-assisted path). Validation enforces quality.
// ─────────────────────────────────────────────────────────────────────────────

const TASK_PATTERNS = /\b(Fix all|Implement|Add|Create|Update|Remove|Check|Verify|Test|Build|Deploy|NEXT:|Signal:)\b/i;
const DEBUG_PATTERNS = /(\bDEBUG\b|\bTODO\b|\bstack trace\b|\berror\b|\bfailed\b|\bcrash\b|\bbroken\b|\bbug\b|\bat line\b|\bTraceback\b)/i;
const SELF_REF_PATTERNS = /\b(I think|I tried|I'm working|I'm doing|working on|trying to|in this session|while I was|I was just)\b/i;
const MULTI_SENTENCE = /\.\s+\w/;

function validateDecision(decision: string): { valid: boolean; reason?: string } {
  // §AsccSections: decisions = crystallized choices that guide future action
  // Each <= 160 chars. Single sentence preferred.
  if (decision.length > 280) {
    return { valid: false, reason: "Too verbose — distill to ONE crystallized sentence (max 280 chars). Use scratchpad for elaboration." };
  }
  if (TASK_PATTERNS.test(decision)) {
    return { valid: false, reason: "Sounds like a task list — decisions capture ARCHITECTURAL CHOICES, not implementation plans. Write task in scratchpad. Distill the decision." };
  }
  if (DEBUG_PATTERNS.test(decision)) {
    return { valid: false, reason: "Sounds like debugging metadata — decisions are stable choices, not investigation notes. Move to scratchpad." };
  }
  if (SELF_REF_PATTERNS.test(decision)) {
    return { valid: false, reason: "Sounds like stream-of-consciousness — decisions should be objective architectural statements. Distill from scratchpad notes." };
  }
  if (MULTI_SENTENCE.test(decision) && decision.length > 160) {
    return { valid: false, reason: "Multiple sentences + long — decisions should be ONE crystallized sentence. Per §AsccSections (<=160 chars)." };
  }
  return { valid: true };
}

function validateConstraint(constraint: string): { valid: boolean; reason?: string } {
  // §AsccSections: constraints = DISCOVERED REQUIREMENTS (not self-imposed tasks)
  // Constraint is a hard boundary from environment/architecture, not "I should do X"
  if (constraint.length > 200) {
    return { valid: false, reason: "Too verbose — distill to one sentence (max 200 chars)." };
  }
  if (TASK_PATTERNS.test(constraint)) {
    return { valid: false, reason: "Sounds like a self-imposed task — constraints are DISCOVERED REQUIREMENTS from environment/architecture. Not 'I will do X'." };
  }
  if (/\b(will|should|must|need to|going to)\b/i.test(constraint)) {
    return { valid: false, reason: "Sounds like self-imposed obligation — constraints are discovered requirements from environment, not agent commitments. Use scratchpad." };
  }
  return { valid: true };
}

function validateFailure(failure: string): { valid: boolean; reason?: string } {
  // §AsccSections: failures = what failed and why
  // Specific, diagnostic, not just "it didn't work"
  if (failure.length > 300) {
    return { valid: false, reason: "Too verbose — distill to one diagnostic sentence (max 300 chars)." };
  }
  if (!/(\.|:)/.test(failure)) {
    return { valid: false, reason: "Vague — failures should be SPECIFIC: what failed AND why (or what you suspect). 'It didn't work' = scratchpad." };
  }
  if (SELF_REF_PATTERNS.test(failure) && !/^(Build|Test|Deploy|API|Request|Query|Compil|Cargo)/i.test(failure)) {
    return { valid: false, reason: "Sounds like investigation process — failures should be: SPECIFIC COMPONENT failed, with DIAGNOSIS. Move investigation notes to scratchpad." };
  }
  return { valid: true };
}

// §AsccSections: validate_slot — rejects verbose output, task patterns, self-reference.
// MUST run on ALL tool writes before any Focus State update.
function validateSlot(value: string, maxChars: number): boolean {
  if (!value || value.length === 0) return false;
  if (value.length > maxChars) return false;
  const lower = value.toLowerCase();
  if (/\b(implement | add | create | update | remove | fix all | check | verify | next:|signal:)/.test(lower)) return false;
  if (/\b(i think|i tried|i'm working|i was|in this session|while i was|my fs\.|my fix|let me|i need to|i will|i'll need)/.test(lower)) return false;
  if (/(\*\*|\u2705|\u274C|- \[ \]|---|```)/.test(value)) return false;
  if (lower.includes("now") && lower.includes("need to")) return false;
  if (lower.includes("continue") && value.length > 80) return false;
  return true;
}

// Push delta to Focusa — validates ALL slot values before write.
async function pushDelta(delta: { decisions?: string[]; constraints?: string[]; failures?: string[]; intent?: string; current_focus?: string; next_steps?: string[]; open_questions?: string[]; recent_results?: string[]; notes?: string[]; artifacts?: Array<{ kind: string; label: string; path_or_id?: string }> }): Promise<boolean> {
  if (!S.focusaAvailable || !S.activeFrameId) return false;

  // Validate every string slot before sending.
  if (delta.decisions?.some(v => !validateSlot(v, 160))) return false;
  if (delta.constraints?.some(v => !validateSlot(v, 200))) return false;
  if (delta.failures?.some(v => !validateSlot(v, 300))) return false;
  if (delta.intent && !validateSlot(delta.intent, 500)) return false;
  if (delta.current_focus && !validateSlot(delta.current_focus, 300)) return false;
  if (delta.next_steps?.some(v => !validateSlot(v, 160))) return false;
  if (delta.open_questions?.some(v => !validateSlot(v, 200))) return false;
  if (delta.recent_results?.some(v => !validateSlot(v, 300))) return false;
  if (delta.notes?.some(v => !validateSlot(v, 200))) return false;

  try {
    await focusaFetch("/focus/update", {
      method: "POST",
      body: JSON.stringify({
        frame_id: S.activeFrameId,
        turn_id: `pi-turn-${S.turnCount}`,
        delta,
      }),
    });
    return true;
  } catch { return false; }
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
    description: "Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done.",
    promptSnippet: "Working notes → scratchpad. Crystallized decision → focusa_decide.",
    parameters: Type.Object({
      note: Type.String({ description: "Working note — reasoning, task list, hypothesis, dead end. Unlimited length." }),
      tag: Type.Optional(Type.String({ description: "Tag: reasoning|task|hypothesis|dead-end|self-correction|next-step" })),
    }),
    promptGuidelines: [
      "ALL working notes go HERE. scratchpad ≠ Focus State.",
      "NEXT:/Signal: directives, task lists, design notes, self-corrections → here.",
      "When done: distill ONE crystallized sentence → focusa_decide.",
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
  // Per G1-07 §Delta Summarization Rule: LLM-assisted delta summarization.
  // Agent IS the summarizer — distill crystallized decisions from scratchpad notes.
  //
  // Validation rules (per §AsccSections: decisions = crystallized choices <= 160 chars):
  //   - Must be ONE crystallized sentence (architectural choice)
  //   - NOT a task list ("Fix all", "Implement", "NEXT:")
  //   - NOT debugging metadata ("error", "failed", "DEBUG")
  //   - NOT stream-of-consciousness ("I think", "I tried")
  //   - Max 280 chars (leniency over §AsccSections 160 char limit)
  //
  // Use focusa_scratch for all working notes first. Then distill ONE decision.
  pi.registerTool({
    name: "focusa_decide",
    label: "Record Decision",
    description: "Record a crystallized architectural decision in Focus State. Use focusa_scratch for working notes first. Decisions are ONE sentence (<=280 chars) — architectural choices only, not task lists.",
    promptSnippet: "Crystallized decision → Focus State. Working notes → focusa_scratch first.",
    parameters: Type.Object({
      decision: Type.String({ description: "ONE crystallized architectural choice — what was decided and why (max 280 chars). NOT a task list or debugging note." }),
      rationale: Type.Optional(Type.String({ description: "Context: why this decision was made (max 200 chars). Summarize from scratchpad notes." })),
    }),
    promptGuidelines: [
      "Step 1: Write detailed reasoning in focusa_scratch",
      "Step 2: Distill ONE crystallized sentence → decision field",
      "decision = what was decided (architectural choice, not implementation plan)",
      "rationale = why (1-2 sentences max)",
      "VALIDATION FAILS if: task patterns (Fix/Add/Check), debug patterns (error/failed), self-reference (I think/I tried), or > 280 chars",
      "Example VALID: 'Use two-file model: /tmp/pi-scratch/ for working notes, Focus State for operator-managed decisions only.'",
      "Example INVALID: 'Fix all pi-extension spec gaps in priority order...' (task list, not decision)",
    ],
    async execute(_id, params) {
      const { decision, rationale } = params as { decision: string; rationale?: string };
      const v = validateDecision(decision);
      if (!v.valid) {
        return {
          content: [{ type: "text" as const, text: `❌ Decision rejected: ${v.reason}\n\nWrite detailed reasoning to focusa_scratch first, then distill ONE crystallized decision.` }],
          details: { valid: false, reason: v.reason, decision, rationale: rationale?.slice(0, 200) },
        };
      }
      const turn = S.turnCount;
      const ok = await pushDelta({ decisions: [decision] });
      if (!ok) {
        return {
          content: [{ type: "text" as const, text: `⚠️ Focus State unavailable — decision NOT recorded. Save to scratchpad: ${decision.slice(0, 80)}` }],
          details: { valid: false, reason: "Focusa unavailable", decision, rationale: rationale?.slice(0, 200) },
        };
      }
      return {
        content: [{ type: "text" as const, text: `✅ Decision recorded (turn ${turn}): ${decision.slice(0, 120)}${decision.length > 120 ? "…" : ""}` }],
        details: { valid: true, reason: undefined, decision, rationale: rationale?.slice(0, 200) },
      };
    },
  });

  // ── focusa_constraint ────────────────────────────────────────────────────
  // §AsccSections: constraints = DISCOVERED REQUIREMENTS from environment/architecture.
  // NOT self-imposed tasks or agent commitments.
  //
  // Valid constraints:
  //   - "MariaDB 10.6 only — no upgrade path to 11.x yet"
  //   - "cPanel API requires root — cannot run as user"
  //   - "Focus State cannot be cleared via /focus/update — only accumulation"
  //   - "Wirebot thoughts only in /wbm mode"
  //
  // Invalid (reject with validation):
  //   - "I must check git status first" (self-imposed task)
  //   - "Need to update the README" (implementation plan)
  //   - "I should use the scratchpad" (agent commitment)
  pi.registerTool({
    name: "focusa_constraint",
    label: "Record Constraint",
    description: "Record a DISCOVERED REQUIREMENT in Focus State. Constraints are hard boundaries from environment/architecture — NOT self-imposed tasks. Max 200 chars.",
    promptSnippet: "Constraints = discovered requirements. Self-imposed tasks → focusa_scratch.",
    parameters: Type.Object({
      constraint: Type.String({ description: "Discovered requirement — hard boundary from environment or architecture (max 200 chars). NOT a task or agent commitment." }),
      source: Type.Optional(Type.String({ description: "Where discovered: spec file, error message, API docs, operator directive." })),
    }),
    promptGuidelines: [
      "Constraints are DISCOVERED REQUIREMENTS, not self-imposed tasks.",
      "VALID: environment boundary, API limit, spec rule, architectural pattern, operator directive",
      "INVALID: 'I should X', 'Need to Y', implementation plans, agent commitments",
      "Example VALID: 'Focus State cannot be cleared — /focus/update only accumulates. Stale entries require fresh frame push.'",
      "Example INVALID: 'Need to fix the scratchpad path' (self-imposed task)",
    ],
    async execute(_id, params) {
      const { constraint, source } = params as { constraint: string; source?: string };
      const v = validateConstraint(constraint);
      if (!v.valid) {
        return {
          content: [{ type: "text" as const, text: `❌ Constraint rejected: ${v.reason}\n\nDiscovered requirements from environment → focusa_constraint. Self-imposed tasks → focusa_scratch.` }],
          details: { valid: false, reason: v.reason, constraint, source },
        };
      }
      const turn = S.turnCount;
      const ok = await pushDelta({ constraints: [constraint] });
      if (!ok) {
        return {
          content: [{ type: "text" as const, text: `⚠️ Focus State unavailable — constraint NOT recorded. Save to scratchpad: ${constraint.slice(0, 80)}` }],
          details: { valid: false, reason: "Focusa unavailable", constraint, source },
        };
      }
      return {
        content: [{ type: "text" as const, text: `✅ Constraint recorded (turn ${turn}): ${constraint.slice(0, 120)}${constraint.length > 120 ? "…" : ""}` }],
        details: { valid: true, reason: undefined, constraint, source },
      };
    },
  });

  // ── focusa_failure ───────────────────────────────────────────────────────
  // §AsccSections: failures = what failed and why (diagnostic, specific)
  // NOT investigation process or debugging metadata.
  pi.registerTool({
    name: "focusa_failure",
    label: "Record Failure",
    description: "Record a specific failure with diagnosis in Focus State. Must identify WHAT failed and WHY (or suspected why). Max 300 chars.",
    promptSnippet: "Failures = specific component + diagnosis. Investigation notes → focusa_scratch.",
    parameters: Type.Object({
      failure: Type.String({ description: "Specific failure: what failed + diagnosis (max 300 chars). Must contain period or colon." }),
      recovery: Type.Optional(Type.String({ description: "What was done to recover or workaround." })),
    }),
    promptGuidelines: [
      "Be SPECIFIC: what component failed + why (or suspected why).",
      "VALID: 'Focus State injection failed: stack.stack.stack.frames returned undefined (triple-nesting bug).'",
      "INVALID: 'Something went wrong', 'It didn't work', investigation process",
      "Move detailed investigation notes to focusa_scratch.",
      "recovery = what was done to fix or work around (optional).",
    ],
    async execute(_id, params) {
      const { failure, recovery } = params as { failure: string; recovery?: string };
      const v = validateFailure(failure);
      if (!v.valid) {
        return {
          content: [{ type: "text" as const, text: `❌ Failure rejected: ${v.reason}\n\nBe specific: WHAT failed + WHY. Move investigation to focusa_scratch.` }],
          details: { valid: false, reason: v.reason, failure, recovery },
        };
      }
      const turn = S.turnCount;
      const ok = await pushDelta({ failures: [failure] });
      if (!ok) {
        return {
          content: [{ type: "text" as const, text: `⚠️ Focus State unavailable — failure NOT recorded. Save to scratchpad: ${failure.slice(0, 80)}` }],
          details: { valid: false, reason: "Focusa unavailable", failure, recovery },
        };
      }
      return {
        content: [{ type: "text" as const, text: `✅ Failure recorded (turn ${turn}): ${failure.slice(0, 120)}${failure.length > 120 ? "…" : ""}` }],
        details: { valid: true, reason: undefined, failure, recovery },
      };
    },
  });

  // ── focusa_intent (§AsccSections) ──────────────────────────────────────────
  // Set the frame intent: what this session is trying to achieve. 1-3 sentences.
  pi.registerTool({
    name: "focusa_intent",
    label: "Set Intent",
    description: "Set the frame intent — what this session is trying to achieve (1-3 sentences, max 500 chars).",
    parameters: Type.Object({
      intent: Type.String({ description: "Intent: what this frame/session is trying to achieve (1-3 sentences, max 500 chars)." }),
    }),
    async execute(_id, params) {
      const { intent } = params as { intent: string };
      if (!S.focusaAvailable || !S.activeFrameId) return { content: [{ type: "text", text: "Focusa unavailable." }], details: { valid: false, intent } };
      if (intent.length > 500) return { content: [{ type: "text", text: "Intent exceeds 500 chars. Distill to 1-3 sentences." }], details: { valid: false, intent } };
      const ok = await pushDelta({ intent });
      return ok
        ? { content: [{ type: "text", text: `Intent set: ${intent.slice(0, 100)}` }], details: { valid: true, intent } }
        : { content: [{ type: "text", text: "Focusa unavailable." }], details: { valid: false, intent } };
    },
  });


  // ── focusa_current_focus (§AsccSections) ─────────────────────────────────
  // Update current focus: what the agent is actively working on. Replaces on each update.
  pi.registerTool({
    name: "focusa_current_focus",
    label: "Set Current Focus",
    description: "Update current focus — what you are actively working on right now (1-3 sentences, max 300 chars).",
    parameters: Type.Object({
      focus: Type.String({ description: "Current focus: what you are actively working on (1-3 sentences, max 300 chars)." }),
    }),
    async execute(_id, params) {
      const { focus } = params as { focus: string };
      if (!S.focusaAvailable || !S.activeFrameId) return { content: [{ type: "text", text: "Focusa unavailable." }], details: { valid: false, focus } };
      if (focus.length > 300) return { content: [{ type: "text", text: "Current focus exceeds 300 chars." }], details: { valid: false, focus } };
      const ok = await pushDelta({ current_focus: focus });
      return ok
        ? { content: [{ type: "text", text: `Current focus set: ${focus.slice(0, 100)}` }], details: { valid: true, focus } }
        : { content: [{ type: "text", text: "Focusa unavailable." }], details: { valid: false, focus } };
    },
  });

  // ── focusa_next_step (§AsccSections) ─────────────────────────────────────
  // Record next step. Replaces previous. Cap 15.
  pi.registerTool({
    name: "focusa_next_step",
    label: "Record Next Step",
    description: "Record what you plan to do next (max 160 chars).",
    parameters: Type.Object({
      step: Type.String({ description: "Next step (max 160 chars)." }),
    }),
    async execute(_id, params) {
      const { step } = params as { step: string };
      if (!S.focusaAvailable || !S.activeFrameId) return { content: [{ type: "text", text: "Focusa unavailable." }], details: { valid: false, step } };
      if (step.length > 160) return { content: [{ type: "text", text: "Step exceeds 160 chars." }], details: { valid: false, step } };
      const ok = await pushDelta({ next_steps: [step] });
      return ok
        ? { content: [{ type: "text", text: `Next step recorded: ${step.slice(0, 80)}` }], details: { valid: true, step } }
        : { content: [{ type: "text", text: "Focusa unavailable." }], details: { valid: false, step } };
    },
  });

  // ── focusa_open_question (§AsccSections) ─────────────────────────────────
  pi.registerTool({
    name: "focusa_open_question",
    label: "Record Open Question",
    description: "Record an open question that needs to be answered (max 200 chars).",
    parameters: Type.Object({
      question: Type.String({ description: "Open question (max 200 chars)." }),
    }),
    async execute(_id, params) {
      const { question } = params as { question: string };
      if (!S.focusaAvailable || !S.activeFrameId) return { content: [{ type: "text", text: "Focusa unavailable." }], details: { valid: false, question } };
      if (question.length > 200) return { content: [{ type: "text", text: "Question exceeds 200 chars." }], details: { valid: false, question } };
      const ok = await pushDelta({ open_questions: [question] });
      return ok
        ? { content: [{ type: "text", text: `Open question recorded: ${question.slice(0, 80)}` }], details: { valid: true, question } }
        : { content: [{ type: "text", text: "Focusa unavailable." }], details: { valid: false, question } };
    },
  });

  // ── focusa_recent_result (§AsccSections) ─────────────────────────────────
  // Record a recent result. Keeps last 10, newest first.
  pi.registerTool({
    name: "focusa_recent_result",
    label: "Record Recent Result",
    description: "Record a completed result, output, or reference (max 300 chars).",
    parameters: Type.Object({
      result: Type.String({ description: "Recent result (max 300 chars)." }),
    }),
    async execute(_id, params) {
      const { result } = params as { result: string };
      if (!S.focusaAvailable || !S.activeFrameId) return { content: [{ type: "text", text: "Focusa unavailable." }], details: { valid: false, result } };
      if (result.length > 300) return { content: [{ type: "text", text: "Result exceeds 300 chars." }], details: { valid: false, result } };
      const ok = await pushDelta({ recent_results: [result] });
      return ok
        ? { content: [{ type: "text", text: `Result recorded: ${result.slice(0, 80)}` }], details: { valid: true, result } }
        : { content: [{ type: "text", text: "Focusa unavailable." }], details: { valid: false, result } };
    },
  });

  // ── focusa_note (§AsccSections) ───────────────────────────────────────────
  // Misc notes, bounded at 20, oldest decay first.
  pi.registerTool({
    name: "focusa_note",
    label: "Record Note",
    description: "Miscellaneous note (max 200 chars). Bounded at 20, oldest decay first.",
    parameters: Type.Object({
      note: Type.String({ description: "Note (max 200 chars)." }),
    }),
    async execute(_id, params) {
      const { note } = params as { note: string };
      if (!S.focusaAvailable || !S.activeFrameId) return { content: [{ type: "text", text: "Focusa unavailable." }], details: { valid: false, note } };
      if (note.length > 200) return { content: [{ type: "text", text: "Note exceeds 200 chars." }], details: { valid: false, note } };
      const ok = await pushDelta({ notes: [note] });
      return ok
        ? { content: [{ type: "text", text: `Note recorded: ${note.slice(0, 80)}` }], details: { valid: true, note } }
        : { content: [{ type: "text", text: "Focusa unavailable." }], details: { valid: false, note } };
    },
  });
}
