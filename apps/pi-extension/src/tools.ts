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
import { S, focusaFetch, focusaPost, ensurePiFrame } from "./state.js";

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

function appendScratchpadLine(note: string, tag?: string): { saved: boolean; turn: number } {
  const turn = S.turnCount;
  const dir = scratchDir(turn);
  ensureScratchDir();
  const ts = new Date().toISOString().slice(11, 23);
  const line = `[${ts}]${tag ? ` [${tag}]` : ""} ${note}`;
  try {
    const { execSync } = require("child_process");
    execSync(`mkdir -p "${dir}" && echo ${JSON.stringify(line)} >> "${dir}/notes.txt"`, { stdio: "pipe" });
    return { saved: true, turn };
  } catch {
    return { saved: false, turn };
  }
}

function emitWriteTelemetry(event: string, body: Record<string, any>): void {
  if (!S.cfg?.emitMetrics) return;
  focusaPost("/telemetry/ops", {
    event,
    surface: "pi",
    turn_id: `pi-turn-${S.turnCount}`,
    frame_id: S.activeFrameId,
    ...body,
  });
}

function deltaTargets(delta: { decisions?: string[]; constraints?: string[]; failures?: string[]; intent?: string; current_focus?: string; next_steps?: string[]; open_questions?: string[]; recent_results?: string[]; notes?: string[]; artifacts?: Array<{ kind: string; label: string; path_or_id?: string }> }): string[] {
  return Object.entries(delta)
    .filter(([, value]) => value !== undefined)
    .map(([key]) => key);
}

function mirrorFailedFocusWrite(kind: "decision" | "constraint" | "failure", reason: PushDeltaFailureReason, payload: string, meta: Record<string, string | undefined>): { saved: boolean; turn: number } {
  const note = JSON.stringify({
    type: "focusa_write_fallback",
    kind,
    reason,
    payload,
    meta,
    turn: S.turnCount,
    at: new Date().toISOString(),
  });
  const scratch = appendScratchpadLine(note, "focusa-fallback");
  emitWriteTelemetry("focusa_write_fallback", {
    kind,
    reason,
    scratch_saved: scratch.saved,
    scratch_turn: scratch.turn,
  });
  return scratch;
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
  if (/\b(status:|next action:|blocker:)/.test(lower)) return false;
  if (/(\*\*|\u2705|\u274C|- \[ \]|---|```)/.test(value)) return false;
  if (lower.includes("now") && lower.includes("need to")) return false;
  if (lower.includes("continue") && value.length > 80) return false;
  return true;
}

function validateNamedSlot(value: string, maxChars: number, kind: "intent" | "current_focus" | "next_step" | "open_question" | "recent_result" | "note"): { valid: boolean; reason?: string } {
  const trimmed = String(value || "").trim();
  if (!trimmed) return { valid: false, reason: `${kind.replace("_", " ")} cannot be empty.` };
  if (trimmed.length > maxChars) return { valid: false, reason: `${kind.replace("_", " ")} exceeds ${maxChars} chars.` };
  if (kind === "open_question" && !trimmed.includes("?")) {
    return { valid: false, reason: "Open question should be phrased as a question (include '?')." };
  }
  if (!validateSlot(trimmed, maxChars)) {
    return { valid: false, reason: `Rejected by Focus State slot validator — distill this ${kind.replace("_", " ")} to concise objective text or move verbose/process notes to scratchpad.` };
  }
  return { valid: true };
}

export type PushDeltaFailureReason = "offline" | "no_active_frame" | "validation_rejected" | "write_failed";

export type PushDeltaResult =
  | { ok: true }
  | { ok: false; reason: PushDeltaFailureReason };

function formatPushDeltaFailure(reason: PushDeltaFailureReason): string {
  switch (reason) {
    case "offline":
      return "Focusa offline";
    case "no_active_frame":
      return "No active Pi frame";
    case "validation_rejected":
      return "Focus State validation rejected the write";
    case "write_failed":
    default:
      return "Focusa write failed";
  }
}

function formatNonCriticalWriteFailure(slotLabel: string, reason: PushDeltaFailureReason): string {
  const base = formatPushDeltaFailure(reason);
  if (reason === "no_active_frame") return `⚠️ ${base} — ${slotLabel} NOT recorded. Frame recovery was attempted; retry after /focusa-status.`;
  if (reason === "offline") return `⚠️ ${base} — ${slotLabel} NOT recorded. Retry when Focusa is reachable.`;
  if (reason === "validation_rejected") return `⚠️ ${base} — ${slotLabel} NOT recorded. Distill wording or use scratchpad.`;
  return `⚠️ ${base} — ${slotLabel} NOT recorded.`;
}

// Push delta to Focusa — validates ALL slot values before write.
export async function pushDelta(delta: { decisions?: string[]; constraints?: string[]; failures?: string[]; intent?: string; current_focus?: string; next_steps?: string[]; open_questions?: string[]; recent_results?: string[]; notes?: string[]; artifacts?: Array<{ kind: string; label: string; path_or_id?: string }> }): Promise<PushDeltaResult> {
  const targets = deltaTargets(delta);
  let recoveredFrame = false;
  emitWriteTelemetry("focusa_write_attempt", { targets, had_frame: !!S.activeFrameId });

  if (!S.focusaAvailable) {
    emitWriteTelemetry("focusa_write_failed", { targets, reason: "offline" });
    return { ok: false, reason: "offline" };
  }

  // Validate every string slot before sending.
  if (delta.decisions?.some(v => !validateSlot(v, 160))) { emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected" }); return { ok: false, reason: "validation_rejected" }; }
  if (delta.constraints?.some(v => !validateSlot(v, 200))) { emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected" }); return { ok: false, reason: "validation_rejected" }; }
  if (delta.failures?.some(v => !validateSlot(v, 300))) { emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected" }); return { ok: false, reason: "validation_rejected" }; }
  if (delta.intent && !validateSlot(delta.intent, 500)) { emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected" }); return { ok: false, reason: "validation_rejected" }; }
  if (delta.current_focus && !validateSlot(delta.current_focus, 300)) { emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected" }); return { ok: false, reason: "validation_rejected" }; }
  if (delta.next_steps?.some(v => !validateSlot(v, 160))) { emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected" }); return { ok: false, reason: "validation_rejected" }; }
  if (delta.open_questions?.some(v => !validateSlot(v, 200))) { emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected" }); return { ok: false, reason: "validation_rejected" }; }
  if (delta.recent_results?.some(v => !validateSlot(v, 300))) { emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected" }); return { ok: false, reason: "validation_rejected" }; }
  if (delta.notes?.some(v => !validateSlot(v, 200))) { emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected" }); return { ok: false, reason: "validation_rejected" }; }

  if (!S.activeFrameId) {
    emitWriteTelemetry("focusa_write_recovery_attempt", { targets, reason: "no_active_frame" });
    const frameId = await ensurePiFrame(undefined, undefined, "pi-auto-recover");
    recoveredFrame = !!frameId;
    emitWriteTelemetry("focusa_write_recovery_result", { targets, reason: "no_active_frame", recovered: recoveredFrame });
    if (!frameId) {
      emitWriteTelemetry("focusa_write_failed", { targets, reason: "no_active_frame" });
      return { ok: false, reason: "no_active_frame" };
    }
  }

  try {
    const response = await focusaFetch("/focus/update", {
      method: "POST",
      body: JSON.stringify({
        frame_id: S.activeFrameId,
        turn_id: `pi-turn-${S.turnCount}`,
        delta,
      }),
    });
    if (!response || response.status === "write_failed") {
      emitWriteTelemetry("focusa_write_failed", { targets, reason: "write_failed", recovered_frame: recoveredFrame });
      return { ok: false, reason: "write_failed" };
    }
    if (response.status === "no_active_frame") {
      emitWriteTelemetry("focusa_write_failed", { targets, reason: "no_active_frame", recovered_frame: recoveredFrame });
      return { ok: false, reason: "no_active_frame" };
    }
    if (response.status === "rejected") {
      emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected", recovered_frame: recoveredFrame });
      return { ok: false, reason: "validation_rejected" };
    }
    if (response.status !== "accepted") {
      emitWriteTelemetry("focusa_write_failed", { targets, reason: "write_failed", recovered_frame: recoveredFrame, status: response.status || "unknown" });
      return { ok: false, reason: "write_failed" };
    }
    emitWriteTelemetry("focusa_write_succeeded", { targets, recovered_frame: recoveredFrame, frame_id: response.frame_id || S.activeFrameId });
    return { ok: true };
  } catch {
    emitWriteTelemetry("focusa_write_failed", { targets, reason: "write_failed", recovered_frame: recoveredFrame });
    return { ok: false, reason: "write_failed" };
  }
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
      const scratch = appendScratchpadLine(note, tag);
      return {
        content: [{ type: "text" as const, text: `📝 Scratchpad saved (turn ${scratch.turn}): ${note.slice(0, 80)}${note.length > 80 ? "…" : ""}` }],
        details: { note, tag, turn: scratch.turn },
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
      const result = await pushDelta({ decisions: [decision] });
      if (!result.ok) {
        const fallback = mirrorFailedFocusWrite("decision", result.reason, decision, { rationale: rationale?.slice(0, 200) });
        const fallbackText = fallback.saved ? `Saved to scratchpad automatically (turn ${fallback.turn}).` : "Scratchpad fallback also failed.";
        return {
          content: [{ type: "text" as const, text: `⚠️ ${formatPushDeltaFailure(result.reason)} — decision NOT recorded in Focus State. ${fallbackText}` }],
          details: { valid: false, reason: result.reason, decision, rationale: rationale?.slice(0, 200) },
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
      const result = await pushDelta({ constraints: [constraint] });
      if (!result.ok) {
        const fallback = mirrorFailedFocusWrite("constraint", result.reason, constraint, { source });
        const fallbackText = fallback.saved ? `Saved to scratchpad automatically (turn ${fallback.turn}).` : "Scratchpad fallback also failed.";
        return {
          content: [{ type: "text" as const, text: `⚠️ ${formatPushDeltaFailure(result.reason)} — constraint NOT recorded in Focus State. ${fallbackText}` }],
          details: { valid: false, reason: result.reason, constraint, source },
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
      const result = await pushDelta({ failures: [failure] });
      if (!result.ok) {
        const fallback = mirrorFailedFocusWrite("failure", result.reason, failure, { recovery });
        const fallbackText = fallback.saved ? `Saved to scratchpad automatically (turn ${fallback.turn}).` : "Scratchpad fallback also failed.";
        return {
          content: [{ type: "text" as const, text: `⚠️ ${formatPushDeltaFailure(result.reason)} — failure NOT recorded in Focus State. ${fallbackText}` }],
          details: { valid: false, reason: result.reason, failure, recovery },
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
      const v = validateNamedSlot(intent, 500, "intent");
      if (!v.valid) return { content: [{ type: "text", text: v.reason || "Invalid intent." }], details: { valid: false, intent } };
      const result = await pushDelta({ intent: intent.trim() });
      return result.ok
        ? { content: [{ type: "text", text: `Intent set: ${intent.slice(0, 100)}` }], details: { valid: true, reason: undefined, intent } }
        : { content: [{ type: "text", text: formatNonCriticalWriteFailure("intent", result.reason) }], details: { valid: false, intent } };
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
      const v = validateNamedSlot(focus, 300, "current_focus");
      if (!v.valid) return { content: [{ type: "text", text: v.reason || "Invalid current focus." }], details: { valid: false, focus } };
      const result = await pushDelta({ current_focus: focus.trim() });
      return result.ok
        ? { content: [{ type: "text", text: `Current focus set: ${focus.slice(0, 100)}` }], details: { valid: true, reason: undefined, focus } }
        : { content: [{ type: "text", text: formatNonCriticalWriteFailure("current focus", result.reason) }], details: { valid: false, focus } };
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
      const v = validateNamedSlot(step, 160, "next_step");
      if (!v.valid) return { content: [{ type: "text", text: v.reason || "Invalid next step." }], details: { valid: false, step } };
      const result = await pushDelta({ next_steps: [step.trim()] });
      return result.ok
        ? { content: [{ type: "text", text: `Next step recorded: ${step.slice(0, 80)}` }], details: { valid: true, reason: undefined, step } }
        : { content: [{ type: "text", text: formatNonCriticalWriteFailure("next step", result.reason) }], details: { valid: false, step } };
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
      const v = validateNamedSlot(question, 200, "open_question");
      if (!v.valid) return { content: [{ type: "text", text: v.reason || "Invalid open question." }], details: { valid: false, question } };
      const result = await pushDelta({ open_questions: [question.trim()] });
      return result.ok
        ? { content: [{ type: "text", text: `Open question recorded: ${question.slice(0, 80)}` }], details: { valid: true, reason: undefined, question } }
        : { content: [{ type: "text", text: formatNonCriticalWriteFailure("open question", result.reason) }], details: { valid: false, question } };
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
      const v = validateNamedSlot(result, 300, "recent_result");
      if (!v.valid) return { content: [{ type: "text", text: v.reason || "Invalid recent result." }], details: { valid: false, result } };
      const writeResult = await pushDelta({ recent_results: [result.trim()] });
      return writeResult.ok
        ? { content: [{ type: "text", text: `Result recorded: ${result.slice(0, 80)}` }], details: { valid: true, reason: undefined, result } }
        : { content: [{ type: "text", text: formatNonCriticalWriteFailure("recent result", writeResult.reason) }], details: { valid: false, result } };
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
      const v = validateNamedSlot(note, 200, "note");
      if (!v.valid) return { content: [{ type: "text", text: v.reason || "Invalid note." }], details: { valid: false, note } };
      const result = await pushDelta({ notes: [note.trim()] });
      return result.ok
        ? { content: [{ type: "text", text: `Note recorded: ${note.slice(0, 80)}` }], details: { valid: true, reason: undefined, note } }
        : { content: [{ type: "text", text: formatNonCriticalWriteFailure("note", result.reason) }], details: { valid: false, note } };
    },
  });

  // ── Continuous Work Loop bridge tools (Spec79 §23 small bridge surface) ──

  async function focusaFetchDetailed(path: string, opts: RequestInit = {}): Promise<{ ok: boolean; status: number; body: any | null }> {
    const timeout = S.cfg?.focusaApiTimeoutMs || 5000;
    const base = S.cfg?.focusaApiBaseUrl || "http://127.0.0.1:8787/v1";
    const token = S.cfg?.focusaToken || "";
    const ac = new AbortController();
    const t = setTimeout(() => ac.abort(), timeout);
    try {
      const r = await fetch(`${base}${path}`, {
        ...opts,
        headers: {
          "Content-Type": "application/json",
          ...(token ? { Authorization: `Bearer ${token}` } : {}),
          ...(opts.headers as Record<string, string> || {}),
        },
        signal: ac.signal,
      });
      let body: any = null;
      try { body = await r.json(); } catch { body = null; }
      return { ok: r.ok, status: r.status, body };
    } catch {
      return { ok: false, status: 0, body: null };
    } finally {
      clearTimeout(t);
    }
  }

  function explainWorkLoopResult(result: { ok: boolean; status: number; body: any | null }, fallback: string): string {
    if (result.ok) return fallback;
    const msg = String(result.body?.error || "").toLowerCase();
    const activeWriter = result.body?.active_writer ? ` (${result.body.active_writer})` : "";
    if (msg.includes("claimed by another writer")) return `blocked: loop controlled by another session${activeWriter}`;
    if (msg.includes("worktree is not clean")) return "blocked: worktree has uncommitted changes";
    if (msg.includes("missing required header")) return "blocked: controller identity header missing";
    if (result.status === 0) return "blocked: daemon unavailable";
    return `blocked: ${result.body?.error || `request failed (${result.status})`}`;
  }

  async function preferredWriterId(): Promise<string> {
    const status = await focusaFetchDetailed("/work-loop/status");
    const claimed = String(status.body?.active_writer || "").trim();
    return claimed || `pi-${process.pid}`;
  }

  function firstBdReadyIdFromText(text: string): string | null {
    const t = String(text || "");
    const m = t.match(/\b([a-z0-9]+-[a-z0-9]+(?:\.[0-9]+)?)\b/i);
    return m ? m[1] : null;
  }

  async function inferRootWorkItemId(explicit?: string): Promise<string | null> {
    const direct = String(explicit || "").trim();
    if (direct) return direct;

    const status = await focusaFetchDetailed("/work-loop/status");
    const currentTask = String(status.body?.current_task?.work_item_id || "").trim();
    if (currentTask) return currentTask;

    try {
      const { execSync } = require("child_process");
      const out = String(execSync("bd ready", { stdio: ["ignore", "pipe", "ignore"] }) || "");
      return firstBdReadyIdFromText(out);
    } catch {
      return null;
    }
  }

  pi.registerTool({
    name: "focusa_work_loop_status",
    label: "Work Loop Status",
    description: "Get current continuous work-loop state and budgets.",
    parameters: Type.Object({}),
    async execute() {
      const result = await focusaFetchDetailed("/work-loop/status");
      if (!result.ok || !result.body) {
        return {
          content: [{ type: "text", text: `Work-loop status ${explainWorkLoopResult(result, "ok")}` }],
          details: { ok: false, status: result.status, response: result.body ?? null },
        };
      }
      const loopStatus = result.body;
      const statusText = String(loopStatus?.status || loopStatus?.work_loop?.status || "unknown");
      const enabled = typeof loopStatus?.enabled === "boolean"
        ? loopStatus.enabled
        : !!loopStatus?.work_loop?.enabled;
      return {
        content: [{ type: "text", text: `Work-loop: ${statusText} (enabled=${enabled ? "yes" : "no"})` }],
        details: { ok: true, status: result.status, response: result.body },
      };
    },
  });

  pi.registerTool({
    name: "focusa_work_loop_control",
    label: "Work Loop Control",
    description: "Control continuous work loop: on, pause, resume, stop.",
    parameters: Type.Object({
      action: Type.Union([
        Type.Literal("on"),
        Type.Literal("pause"),
        Type.Literal("resume"),
        Type.Literal("stop"),
      ]),
      reason: Type.Optional(Type.String({ description: "Optional operator reason (max 200 chars)." })),
      preset: Type.Optional(Type.Union([
        Type.Literal("conservative"),
        Type.Literal("balanced"),
        Type.Literal("push"),
        Type.Literal("audit"),
      ])),
      root_work_item_id: Type.Optional(Type.String({ description: "Optional root BD/task/item id. If omitted, tool infers from active task or bd ready." })),
    }),
    async execute(_id, params) {
      const { action, reason, preset, root_work_item_id } = params as { action: "on" | "pause" | "resume" | "stop"; reason?: string; preset?: "conservative" | "balanced" | "push" | "audit"; root_work_item_id?: string };
      const writerId = await preferredWriterId();

      if (action === "on") {
        const rootWorkItemId = await inferRootWorkItemId(root_work_item_id);
        const payload = {
          preset: preset || S.cfg?.workLoopPreset || "balanced",
          root_work_item_id: rootWorkItemId || undefined,
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
        const res = await focusaFetchDetailed("/work-loop/enable", {
          method: "POST",
          headers: { "x-focusa-writer-id": writerId, "x-focusa-approval": "approved" },
          body: JSON.stringify(payload),
        });
        return {
          content: [{ type: "text", text: `work-loop on → ${explainWorkLoopResult(res, String(res.body?.status || "accepted"))}` }],
          details: { ok: res.ok, action: String(action), status: res.status, response: res.body },
        };
      }

      const route = action === "pause" ? "/work-loop/pause" : action === "resume" ? "/work-loop/resume" : "/work-loop/stop";
      const res = await focusaFetchDetailed(route, {
        method: "POST",
        headers: { "x-focusa-writer-id": writerId },
        body: JSON.stringify({ reason: reason?.slice(0, 200) || `operator ${action} via focusa_work_loop_control` }),
      });
      return {
        content: [{ type: "text", text: `work-loop ${action} → ${explainWorkLoopResult(res, String(res.body?.status || "accepted"))}` }],
        details: { ok: res.ok, action: String(action), status: res.status, response: res.body },
      };
    },
  });

  pi.registerTool({
    name: "focusa_work_loop_context",
    label: "Work Loop Context",
    description: "Update continuation decision context (current ask/scope/steering).",
    parameters: Type.Object({
      current_ask: Type.String({ description: "Current ask for continuation context (max 240 chars)." }),
      ask_kind: Type.Optional(Type.String({ description: "ask_kind hint (optional)." })),
      scope_kind: Type.Optional(Type.String({ description: "scope_kind hint (optional)." })),
      carryover_policy: Type.Optional(Type.String({ description: "carryover policy hint (optional)." })),
      excluded_context_reason: Type.Optional(Type.String({ description: "Reason for excluding carryover context (optional)." })),
      excluded_context_labels: Type.Optional(Type.Array(Type.String())),
      operator_steering_detected: Type.Optional(Type.Boolean()),
      source_turn_id: Type.Optional(Type.String()),
    }),
    async execute(_id, params) {
      const p = params as {
        current_ask: string;
        ask_kind?: string;
        scope_kind?: string;
        carryover_policy?: string;
        excluded_context_reason?: string;
        excluded_context_labels?: string[];
        operator_steering_detected?: boolean;
        source_turn_id?: string;
      };
      if (!p.current_ask?.trim()) {
        return { content: [{ type: "text", text: "current_ask required." }], details: { ok: false, status: 0, response: null } };
      }
      const writerId = await preferredWriterId();
      const res = await focusaFetchDetailed("/work-loop/context", {
        method: "POST",
        headers: { "x-focusa-writer-id": writerId },
        body: JSON.stringify({
          current_ask: p.current_ask.slice(0, 240),
          ask_kind: p.ask_kind,
          scope_kind: p.scope_kind,
          carryover_policy: p.carryover_policy,
          excluded_context_reason: p.excluded_context_reason,
          excluded_context_labels: p.excluded_context_labels,
          operator_steering_detected: p.operator_steering_detected,
          source_turn_id: p.source_turn_id || `pi-turn-${S.turnCount}`,
        }),
      });
      return {
        content: [{ type: "text", text: `work-loop context → ${explainWorkLoopResult(res, String(res.body?.status || "accepted"))}` }],
        details: { ok: res.ok, status: res.status, response: res.body },
      };
    },
  });

  pi.registerTool({
    name: "focusa_work_loop_checkpoint",
    label: "Work Loop Checkpoint",
    description: "Create a manual continuous-loop checkpoint.",
    parameters: Type.Object({
      summary: Type.Optional(Type.String({ description: "Checkpoint summary (max 240 chars)." })),
    }),
    async execute(_id, params) {
      const { summary } = params as { summary?: string };
      const writerId = await preferredWriterId();
      const res = await focusaFetchDetailed("/work-loop/checkpoint", {
        method: "POST",
        headers: { "x-focusa-writer-id": writerId },
        body: JSON.stringify({ summary: (summary || "manual checkpoint via focusa_work_loop_checkpoint").slice(0, 240) }),
      });
      return {
        content: [{ type: "text", text: `work-loop checkpoint → ${explainWorkLoopResult(res, String(res.body?.checkpoint_id || res.body?.status || "accepted"))}` }],
        details: { ok: res.ok, status: res.status, response: res.body },
      };
    },
  });

  pi.registerTool({
    name: "focusa_work_loop_select_next",
    label: "Work Loop Select Next",
    description: "Ask daemon to defer blocked work and select next ready work item.",
    parameters: Type.Object({
      parent_work_item_id: Type.Optional(Type.String({ description: "Parent work item id. If omitted, use active current_task work_item_id." })),
    }),
    async execute(_id, params) {
      const { parent_work_item_id } = params as { parent_work_item_id?: string };
      const writerId = await preferredWriterId();
      const parentWorkItemId = await inferRootWorkItemId(parent_work_item_id);
      if (!parentWorkItemId) {
        return {
          content: [{ type: "text", text: "work-loop select-next → blocked: no active parent work item (pass parent_work_item_id or create ready BD)" }],
          details: { ok: false, status: 422, response: { error: "parent_work_item_id required when no current_task is active and no bd ready item is available" } },
        };
      }
      const res = await focusaFetchDetailed("/work-loop/select-next", {
        method: "POST",
        headers: { "x-focusa-writer-id": writerId },
        body: JSON.stringify({ parent_work_item_id: parentWorkItemId }),
      });
      return {
        content: [{ type: "text", text: `work-loop select-next → ${explainWorkLoopResult(res, String(res.body?.status || "accepted"))}` }],
        details: { ok: res.ok, status: res.status, response: res.body },
      };
    },
  });
}
