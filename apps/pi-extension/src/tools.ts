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
import { S, checkFocusa, focusaFetch, focusaPost, ensurePiFrame } from "./state.js";

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

function validateConstraint(constraint: string, source?: string): { valid: boolean; reason?: string } {
  // §AsccSections: constraints = DISCOVERED REQUIREMENTS (not self-imposed tasks)
  // Constraint is a hard boundary from environment/architecture, not "I should do X".
  // Operator directives are discovered requirements even when phrased with "must/must not".
  const operatorDirective = /operator directive/i.test(source || "") || /^operator directive\b/i.test(constraint);
  if (constraint.length > 200) {
    return { valid: false, reason: "Too verbose — distill to one sentence (max 200 chars)." };
  }
  if (TASK_PATTERNS.test(constraint)) {
    return { valid: false, reason: "Sounds like a self-imposed task — constraints are DISCOVERED REQUIREMENTS from environment/architecture. Not 'I will do X'." };
  }
  if (!operatorDirective && /\b(will|should|must|need to|going to)\b/i.test(constraint)) {
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

type FocusaToolStatus = "accepted" | "completed" | "no_op" | "blocked" | "validation_rejected" | "degraded" | "offline" | "error";
type FocusaRetryPosture = "safe_retry" | "retry_with_idempotency_key" | "check_side_effects_first" | "do_not_retry_unchanged" | "operator_required";

interface FocusaToolResultV1 {
  ok: boolean;
  status: FocusaToolStatus;
  canonical: boolean;
  degraded: boolean;
  summary: string;
  tool?: string;
  family?: string;
  endpoint?: string;
  workpoint_id?: string | null;
  retry: { safe: boolean; posture: FocusaRetryPosture; reason?: string };
  side_effects: string[];
  evidence_refs: string[];
  next_tools: string[];
  error?: { field?: string; code?: string; message?: string; allowed_values?: string[] } | null;
  raw?: unknown;
}

function focusaToolResult(params: {
  ok: boolean;
  status: FocusaToolStatus;
  summary: string;
  canonical?: boolean;
  degraded?: boolean;
  tool?: string;
  family?: string;
  endpoint?: string;
  workpoint_id?: string | null;
  retry?: Partial<FocusaToolResultV1["retry"]>;
  side_effects?: string[];
  evidence_refs?: string[];
  next_tools?: string[];
  error?: FocusaToolResultV1["error"];
  raw?: unknown;
}): FocusaToolResultV1 {
  const degraded = params.degraded ?? (params.status === "degraded" || params.status === "offline");
  return {
    ok: params.ok,
    status: params.status,
    canonical: params.canonical ?? (!degraded && params.ok),
    degraded,
    summary: params.summary.slice(0, 500),
    tool: params.tool,
    family: params.family,
    endpoint: params.endpoint,
    workpoint_id: params.workpoint_id ?? null,
    retry: {
      safe: params.retry?.safe ?? (params.status === "completed" || params.status === "no_op"),
      posture: params.retry?.posture ?? (params.ok ? "safe_retry" : "operator_required"),
      reason: params.retry?.reason,
    },
    side_effects: params.side_effects ?? [],
    evidence_refs: params.evidence_refs ?? [],
    next_tools: params.next_tools ?? [],
    error: params.error ?? null,
    raw: params.raw,
  };
}

function focusaToolDetails(details: Record<string, unknown>, result: FocusaToolResultV1): Record<string, unknown> {
  return { ...details, tool_result_v1: result };
}

function resolveActiveWorkpointContext(): { workpoint_id: string | null; evidence_refs: string[]; summary?: string } {
  const packet = S.activeWorkpointPacket || null;
  const workpoint = packet?.resume_packet?.workpoint || packet?.workpoint || packet;
  const workpointId = String(workpoint?.workpoint_id || packet?.workpoint_id || "") || null;
  const verificationRecords = Array.isArray(workpoint?.verification_records) ? workpoint.verification_records : [];
  const evidenceRefs = verificationRecords
    .map((record: any) => String(record?.evidence_ref || record?.result || ""))
    .filter(Boolean)
    .slice(0, 8);
  return { workpoint_id: workpointId, evidence_refs: evidenceRefs, summary: S.activeWorkpointSummary || undefined };
}

function inferToolResult(tool: string, result: any): FocusaToolResultV1 {
  const details = (result?.details || {}) as Record<string, any>;
  if (details.tool_result_v1) return details.tool_result_v1 as FocusaToolResultV1;
  const text = String(result?.content?.[0]?.text || details.summary || "");
  const family = tool.startsWith("focusa_workpoint_") ? "workpoint"
    : tool.startsWith("focusa_work_loop_") ? "work_loop"
      : tool.startsWith("focusa_tree_") ? "tree_snapshot_lineage"
        : tool.startsWith("focusa_metacog_") ? "metacognition"
          : tool.startsWith("focusa_lineage") || tool.startsWith("focusa_li_") ? "lineage_intelligence"
            : tool === "focusa_scratch" ? "scratchpad" : "focus_state";
  const ok = details.ok === true || details.valid === true || (!/^❌|blocked|.* unavailable/.test(text) && details.ok !== false && details.valid !== false);
  const validationRejected = details.valid === false || /validation_rejected|rejected/.test(text);
  const offline = /offline|unavailable/.test(text);
  const blocked = /blocked/.test(text);
  const degraded = details.canonical === false || /degraded|NON-CANONICAL/.test(text);
  const status: FocusaToolStatus = validationRejected ? "validation_rejected" : offline ? "offline" : blocked ? "blocked" : degraded ? "degraded" : ok ? "completed" : "error";
  const readOnly = family === "lineage_intelligence" || tool.endsWith("_status") || tool.endsWith("_resume") || tool.endsWith("_head") || tool.endsWith("_path") || tool.includes("_retrieve") || tool.includes("_recent") || tool.includes("_doctor") || tool.includes("_diff_");
  const activeWorkpoint = resolveActiveWorkpointContext();
  const resultWorkpointId = String(details.response?.workpoint_id || details.response?.active_workpoint_id || details.workpoint_id || activeWorkpoint.workpoint_id || "") || null;
  return focusaToolResult({
    ok,
    status,
    canonical: !degraded && !offline,
    degraded,
    summary: text || `${tool} ${status}`,
    tool,
    family,
    endpoint: typeof details.endpoint === "string" ? details.endpoint : undefined,
    workpoint_id: resultWorkpointId,
    retry: {
      safe: readOnly || status === "validation_rejected" || status === "offline",
      posture: status === "validation_rejected" ? "do_not_retry_unchanged" : readOnly ? "safe_retry" : "check_side_effects_first",
      reason: status,
    },
    side_effects: readOnly ? [] : [family],
    evidence_refs: activeWorkpoint.evidence_refs,
    next_tools: status === "offline" ? [] : family === "workpoint" ? ["focusa_workpoint_resume"] : [],
    error: validationRejected || blocked || offline ? { code: status, message: text.slice(0, 240) } : null,
    raw: details.response ?? details,
  });
}

function withToolResultEnvelope(tool: any): any {
  if (!tool?.name?.startsWith?.("focusa_") || typeof tool.execute !== "function") return tool;
  const execute = tool.execute;
  return {
    ...tool,
    async execute(id: string, params: unknown) {
      const result = await execute(id, params);
      const details = (result?.details || {}) as Record<string, unknown>;
      const toolResult = inferToolResult(tool.name, result);
      return { ...result, details: focusaToolDetails(details, toolResult) };
    },
  };
}

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
    const recoveredOnline = await checkFocusa().catch(() => false);
    // Health probes can race daemon restarts or stale bridge state. Do not let a
    // failed probe veto a real write; /focus/update is the authoritative check.
    emitWriteTelemetry("focusa_write_recovery_result", { targets, reason: "offline", recovered: recoveredOnline, probe_only: true });
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
    S.focusaAvailable = true;
    emitWriteTelemetry("focusa_write_succeeded", { targets, recovered_frame: recoveredFrame, frame_id: response.frame_id || S.activeFrameId });
    return { ok: true };
  } catch {
    const online = await checkFocusa().catch(() => false);
    const reason: PushDeltaFailureReason = online ? "write_failed" : "offline";
    emitWriteTelemetry("focusa_write_failed", { targets, reason, recovered_frame: recoveredFrame });
    return { ok: false, reason };
  }
}

export function registerTools(pi: ExtensionAPI) {
  const registerTool = pi.registerTool.bind(pi);
  pi.registerTool = ((tool: any) => registerTool(withToolResultEnvelope(tool))) as typeof pi.registerTool;
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
      const v = validateConstraint(constraint, source);
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

  function replayConsumerSurface(result: { ok: boolean; status: number; body: any | null }): {
    replayStatus: string;
    pairObserved: boolean;
    pairLabel: "observed" | "missing" | "unknown";
    continuityGate: "open" | "fail-closed";
    continuityFailClosed: boolean;
    nonClosureObjectiveEvents: number | null;
    nonClosureObjectiveRate: number | null;
  } {
    const payload = result.body || {};
    const replayPayload = payload?.secondary_loop_replay_consumer || payload;
    const continuityPayload = payload?.secondary_loop_continuity_gate || null;
    const objectiveProfile = payload?.secondary_loop_eval_bundle?.secondary_loop_objective_profile || null;

    const replayStatus = String(replayPayload?.status || (result.ok ? "ok" : "error"));
    const healthy = result.ok && replayStatus === "ok";
    const pairObserved = healthy && !!replayPayload?.secondary_loop_closure_replay_evidence?.evidence?.current_task_pair_observed;
    const pairLabel = healthy ? (pairObserved ? "observed" : "missing") : "unknown";
    const continuityGateRaw = String(continuityPayload?.state || (healthy ? "open" : "fail-closed"));
    const continuityGate: "open" | "fail-closed" = continuityGateRaw === "open" ? "open" : "fail-closed";
    const continuityFailClosed = continuityGate !== "open";

    const nonClosureObjectiveEvents = objectiveProfile?.non_closure_objective_events != null
      ? Number(objectiveProfile.non_closure_objective_events)
      : null;
    const nonClosureObjectiveRate = objectiveProfile?.non_closure_objective_rate != null
      ? Number(objectiveProfile.non_closure_objective_rate)
      : null;

    return {
      replayStatus,
      pairObserved,
      pairLabel,
      continuityGate,
      continuityFailClosed,
      nonClosureObjectiveEvents,
      nonClosureObjectiveRate,
    };
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
    name: "focusa_work_loop_writer_status",
    label: "Work Loop Writer Status",
    description: "Read current work-loop writer ownership and mutation preflight guidance without mutating state.",
    parameters: Type.Object({}),
    async execute() {
      const result = await focusaFetchDetailed("/work-loop/status");
      const body = result.body || {};
      const activeWriter = String(body.active_writer || "none");
      const status = String(body.status || body.current_task?.status || "unknown");
      const text = `work-loop writer-status → active_writer=${activeWriter} status=${status} preflight=read_only`;
      return { content: [{ type: "text", text }], details: { ok: result.ok, status: String(result.status), active_writer: activeWriter, authorship_mode: body.authorship_mode, preflight: { mutates: false, writer_required_for: ["control", "context", "checkpoint", "select_next"] }, response: body } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_work_loop_status",
    label: "Work Loop Status",
    description: "Get current continuous work-loop state and budgets.",
    parameters: Type.Object({}),
    async execute() {
      const result = await focusaFetchDetailed("/work-loop/status");
      let replayResult = await focusaFetchDetailed("/work-loop/replay/closure-bundle");
      if (!replayResult.ok || !replayResult.body) {
        replayResult = await focusaFetchDetailed("/work-loop/replay/closure-evidence");
      }
      const replay = replayConsumerSurface(replayResult);
      const objectiveSegment = replay.nonClosureObjectiveEvents == null
        ? ""
        : ` | non_closure_objectives=${replay.nonClosureObjectiveEvents}${replay.nonClosureObjectiveRate == null ? "" : ` (${(replay.nonClosureObjectiveRate * 100).toFixed(1)}%)`}`;

      if (!result.ok || !result.body) {
        return {
          content: [{ type: "text", text: `Work-loop status ${explainWorkLoopResult(result, "ok")} | Replay consumer: ${replay.replayStatus} | pair=${replay.pairLabel} | continuity_gate=${replay.continuityGate}${objectiveSegment}` }],
          details: {
            ok: false,
            status: result.status,
            response: result.body ?? null,
            closure_replay_consumer: {
              status: replay.replayStatus,
              pair_observed: replay.pairObserved,
              pair_label: replay.pairLabel,
              continuity_gate: replay.continuityGate,
              continuity_fail_closed: replay.continuityFailClosed,
              non_closure_objective_events: replay.nonClosureObjectiveEvents,
              non_closure_objective_rate: replay.nonClosureObjectiveRate,
            },
            closure_replay_response: replayResult.body ?? null,
          },
        };
      }

      const loopStatus = result.body;
      const statusText = String(loopStatus?.status || loopStatus?.work_loop?.status || "unknown");
      const enabled = typeof loopStatus?.enabled === "boolean"
        ? loopStatus.enabled
        : !!loopStatus?.work_loop?.enabled;
      return {
        content: [{ type: "text", text: `Work-loop: ${statusText} (enabled=${enabled ? "yes" : "no"}) | Replay consumer: ${replay.replayStatus} | pair=${replay.pairLabel} | continuity_gate=${replay.continuityGate}${objectiveSegment}` }],
        details: {
          ok: true,
          status: result.status,
          response: result.body,
          closure_replay_consumer: {
            status: replay.replayStatus,
            pair_observed: replay.pairObserved,
            pair_label: replay.pairLabel,
            continuity_gate: replay.continuityGate,
            continuity_fail_closed: replay.continuityFailClosed,
            non_closure_objective_events: replay.nonClosureObjectiveEvents,
            non_closure_objective_rate: replay.nonClosureObjectiveRate,
          },
          closure_replay_response: replayResult.body ?? null,
        },
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
      preflight: Type.Optional(Type.Boolean({ description: "If true, only report intended route/writer and do not mutate work-loop state." })),
      root_work_item_id: Type.Optional(Type.String({ description: "Optional root BD/task/item id. If omitted, tool infers from active task or bd ready." })),
    }),
    async execute(_id, params) {
      const { action, reason, preset, preflight, root_work_item_id } = params as { action: "on" | "pause" | "resume" | "stop"; reason?: string; preset?: "conservative" | "balanced" | "push" | "audit"; preflight?: boolean; root_work_item_id?: string };
      const writerId = await preferredWriterId();

      if (preflight) {
        const route = action === "on" ? "/work-loop/enable" : action === "pause" ? "/work-loop/pause" : action === "resume" ? "/work-loop/resume" : "/work-loop/stop";
        return { content: [{ type: "text", text: `work-loop ${action} preflight → route=${route} writer=${writerId} mutates=false` }], details: { ok: true, action: String(action), status: "preflight", route, writer_id: writerId, mutates: false } } as any;
      }

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

  // ── Spec88 Workpoint Continuity tools ────────────────────────────────────

  function summarizeWorkpointResponse(body: any): string {
    const status = String(body?.status || "unknown");
    const id = String(body?.workpoint_id || body?.active_workpoint_id || "none");
    const canonical = typeof body?.canonical === "boolean" ? String(body.canonical) : "unknown";
    const next = String(body?.next_step_hint || body?.resume_packet?.next_slice || body?.workpoint?.next_slice || "resume from typed workpoint packet");
    return `status=${status} id=${id} canonical=${canonical} next=${next}`;
  }

  pi.registerTool({
    name: "focusa_tool_doctor",
    label: "Focusa Tool Doctor",
    description: "Diagnose Focusa tool-suite readiness, active Workpoint continuity, daemon health, and likely next repair action.",
    promptSnippet: "Use first when Focusa tools seem blocked, degraded, stale, or confusing.",
    parameters: Type.Object({
      scope: Type.Optional(Type.String({ description: "Optional family/surface to diagnose, e.g. workpoint, focus_state, metacog." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const health = await focusaFetchDetailed("/health", { method: "GET" });
      const workpoint = await focusaFetchDetailed("/workpoint/current", { method: "GET" });
      const loop = await focusaFetchDetailed("/work-loop/status", { method: "GET" });
      const ready = health.ok && workpoint.ok;
      const text = `tool doctor → readiness=${ready ? "ready" : "degraded"} scope=${String(p.scope || "all")} health=${health.ok ? "ok" : "blocked"} workpoint=${workpoint.ok ? String(workpoint.body?.status || "ok") : "blocked"} work_loop=${loop.ok ? String(loop.body?.status || "ok") : "blocked"}`;
      return { content: [{ type: "text", text }], details: { ok: ready, status: ready ? "completed" : "degraded", health: health.body, workpoint: workpoint.body, work_loop: loop.body } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_active_object_resolve",
    label: "Focusa Active Object Resolve",
    description: "Resolve likely active object references from the current Workpoint and optional hint without inventing canonical refs.",
    promptSnippet: "Use before linking evidence or acting when target object/file/endpoint is ambiguous.",
    parameters: Type.Object({
      hint: Type.Optional(Type.String({ description: "Optional file/object/endpoint/work item hint." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const ctx = resolveActiveWorkpointContext();
      const packet = S.activeWorkpointPacket || {};
      const workpoint = packet?.resume_packet || packet?.workpoint || packet;
      const refs = Array.from(new Set([...(Array.isArray(workpoint?.active_object_refs) ? workpoint.active_object_refs : []), workpoint?.work_item_id, workpoint?.action_intent?.target_ref, p.hint].filter(Boolean).map(String)));
      const text = `active object resolve → count=${refs.length} verified=false refs=${refs.slice(0, 5).join(",") || "none"}`;
      return { content: [{ type: "text", text }], details: { ok: true, status: "completed", workpoint_id: ctx.workpoint_id, refs, verified: false } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_evidence_capture",
    label: "Focusa Evidence Capture",
    description: "Capture a bounded evidence ref/result and optionally link it to the active Workpoint.",
    promptSnippet: "Use after tests, stress runs, or proof collection to keep handles instead of transcript blobs.",
    parameters: Type.Object({
      target_ref: Type.String({ description: "Object/file/test/endpoint/work item proven by this evidence." }),
      result: Type.String({ description: "Bounded result summary." }),
      evidence_ref: Type.String({ description: "Stable evidence handle/path/test id." }),
      attach_to_workpoint: Type.Optional(Type.Boolean({ description: "Defaults true." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      if (p.attach_to_workpoint === false) {
        return { content: [{ type: "text", text: `evidence capture → captured ref=${p.evidence_ref} attach_to_workpoint=false` }], details: { ok: true, status: "completed", evidence_ref: p.evidence_ref } } as any;
      }
      const res = await focusaFetchDetailed("/workpoint/evidence/link", {
        method: "POST",
        headers: { "x-focusa-writer-id": await preferredWriterId() },
        body: JSON.stringify({ target_ref: p.target_ref, result: p.result, evidence_ref: p.evidence_ref }),
      });
      const text = res.ok ? `evidence capture → linked ${p.evidence_ref}` : `evidence capture blocked → ${explainWorkLoopResult(res, "link failed")}`;
      return { content: [{ type: "text", text }], details: { ok: res.ok, status: String(res.status), evidence_ref: p.evidence_ref, response: res.body } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_workpoint_checkpoint",
    label: "Workpoint Checkpoint",
    description: "Create a typed Focusa Workpoint checkpoint before compaction, resume, context overflow, model switch, or risky continuation. Use this instead of trusting raw transcript memory; Focusa becomes the canonical continuation source and returns an explicit next-step hint.",
    promptSnippet: "Before compact/resume/overflow: checkpoint typed workpoint; do not rely on transcript tail.",
    parameters: Type.Object({
      current_ask: Type.Optional(Type.String({ description: "Current operator ask or mission framing." })),
      work_item_id: Type.Optional(Type.String({ description: "Beads/work item id, e.g. focusa-a2w2.6." })),
      checkpoint_reason: Type.Optional(Type.String({ description: "manual|operator_checkpoint|before_compact|after_compact|context_overflow|session_resume|model_switch|fork" })),
      mission: Type.String({ description: "Current mission/objective to preserve across compaction." }),
      target_objects: Type.Optional(Type.Array(Type.String(), { description: "Ontology/file/component/endpoint refs currently targeted." })),
      current_action: Type.Optional(Type.String({ description: "Typed action, e.g. patch_component_binding or resume_workpoint." })),
      verified_evidence: Type.Optional(Type.Array(Type.String(), { description: "Short evidence refs/results already verified; use handles, not raw logs." })),
      blockers: Type.Optional(Type.Array(Type.String(), { description: "Open blockers or drift boundaries." })),
      next_action: Type.String({ description: "Exact bounded next action to resume after compact/retry." }),
      do_not_drift: Type.Optional(Type.Array(Type.String(), { description: "Actions/scope the next agent must not drift into." })),
      source_turn_id: Type.Optional(Type.String({ description: "Pi/source turn id for provenance." })),
      idempotency_key: Type.Optional(Type.String({ description: "Optional external idempotency key." })),
      canonical: Type.Optional(Type.Boolean({ description: "False only for degraded fallback packets." })),
    }),
    promptGuidelines: [
      "Use before /compact, model switches, session repair, and when context feels near limit.",
      "Store handles/evidence summaries, not raw tool output.",
      "The output is the continuation contract: mission, current action, verified evidence, blockers, next action.",
      "If canonical=false, treat as degraded fallback and reconcile when Focusa is healthy.",
    ],
    async execute(_id, params) {
      const p = params as any;
      const actionType = p.current_action || "checkpoint_workpoint";
      const evidence = Array.isArray(p.verified_evidence) ? p.verified_evidence : [];
      const blockers = Array.isArray(p.blockers) ? p.blockers : [];
      const doNotDrift = Array.isArray(p.do_not_drift) ? p.do_not_drift : [];
      const payload: any = {
        mission: p.mission,
        next_slice: [p.next_action, ...doNotDrift.map((d: string) => `DO_NOT_DRIFT: ${d}`)].filter(Boolean).join("\n"),
        work_item_id: p.work_item_id,
        checkpoint_reason: p.checkpoint_reason || "manual",
        canonical: p.canonical !== false,
        promote: p.canonical !== false,
        source_turn_id: p.source_turn_id,
        idempotency_key: p.idempotency_key,
        active_object_refs: Array.isArray(p.target_objects) ? p.target_objects : [],
        action_intent: {
          action_type: actionType,
          target_ref: p.work_item_id || (Array.isArray(p.target_objects) ? p.target_objects[0] : undefined),
          verification_hooks: evidence,
          status: "ready",
        },
        verification_records: evidence.map((e: string) => ({
          target_ref: p.work_item_id || "workpoint",
          result: e,
          evidence_ref: e.startsWith("HANDLE:") || e.startsWith("[HANDLE:") ? e : undefined,
        })),
        blockers: blockers.map((reason: string) => ({ reason, severity: "medium", status: "open" })),
      };
      const res = await focusaFetchDetailed("/workpoint/checkpoint", {
        method: "POST",
        headers: { "x-focusa-writer-id": await preferredWriterId() },
        body: JSON.stringify(payload),
      });
      const text = res.ok
        ? `workpoint checkpoint → ${summarizeWorkpointResponse(res.body)}`
        : res.body?.status === "validation_rejected"
          ? `workpoint checkpoint validation_rejected → field=${String(res.body?.field || "unknown")} allowed=${Array.isArray(res.body?.allowed_values) ? res.body.allowed_values.join(",") : "unknown"} retry=${String(res.body?.retry_posture || "do_not_retry_unchanged")}`
          : `workpoint checkpoint blocked → ${explainWorkLoopResult(res, "checkpoint failed")}`;
      return {
        content: [{ type: "text", text }],
        details: { ok: res.ok, status: res.status, endpoint: "/workpoint/checkpoint", request: payload, response: res.body },
      };
    },
  });

  pi.registerTool({
    name: "focusa_workpoint_link_evidence",
    label: "Workpoint Link Evidence",
    description: "Attach a stable evidence reference or verification result to the active canonical Workpoint.",
    promptSnippet: "Link proof/evidence to active Workpoint instead of keeping it only in transcript.",
    parameters: Type.Object({
      workpoint_id: Type.Optional(Type.String({ description: "Specific Workpoint id; omit to use active Workpoint." })),
      target_ref: Type.String({ description: "Object/file/test/endpoint/work item the evidence verifies." }),
      result: Type.String({ description: "Bounded verification result summary." }),
      evidence_ref: Type.Optional(Type.String({ description: "Stable evidence handle, file path, test id, or artifact ref." })),
      attach_to_workpoint: Type.Optional(Type.Boolean({ description: "Defaults true; false returns blocked/no-op guidance without linking." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      if (p.attach_to_workpoint === false) {
        const text = "workpoint evidence link → no_op attach_to_workpoint=false";
        return {
          content: [{ type: "text", text }],
          details: { ok: true, status: "no_op", reason: "attach_to_workpoint=false" },
        } as any;
      }
      const res = await focusaFetchDetailed("/workpoint/evidence/link", {
        method: "POST",
        headers: { "x-focusa-writer-id": await preferredWriterId() },
        body: JSON.stringify({ workpoint_id: p.workpoint_id, target_ref: p.target_ref, result: p.result, evidence_ref: p.evidence_ref }),
      });
      const text = res.ok
        ? `workpoint evidence link → status=${String(res.body?.status || "accepted")} id=${String(res.body?.workpoint_id || "none")}`
        : `workpoint evidence link blocked → ${explainWorkLoopResult(res, "link failed")}`;
      return {
        content: [{ type: "text", text }],
        details: { ok: res.ok, status: String(res.status), reason: res.ok ? "linked" : "blocked", endpoint: "/workpoint/evidence/link", response: res.body },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_workpoint_resume",
    label: "Workpoint Resume",
    description: "Fetch the active Focusa WorkpointResumePacket after compaction, resume, context overflow, model switch, or uncertainty. Use this instead of guessing from transcript tail; output includes canonical/degraded status, warnings, and the exact next action.",
    promptSnippet: "After compact/resume/overflow: fetch WorkpointResumePacket and continue from it.",
    parameters: Type.Object({
      workpoint_id: Type.Optional(Type.String({ description: "Specific workpoint id; omit to use active workpoint." })),
      mode: Type.Optional(Type.String({ description: "compact_prompt|full_json|operator_summary" })),
    }),
    promptGuidelines: [
      "Use immediately after compaction or session resume before choosing next work.",
      "If not_found, create a checkpoint before continuing important work.",
      "If canonical=false, state degraded status and avoid treating it as canonical truth.",
    ],
    async execute(_id, params) {
      const p = params as { workpoint_id?: string; mode?: string };
      const payload = { workpoint_id: p.workpoint_id, mode: p.mode || "compact_prompt" };
      const res = await focusaFetchDetailed("/workpoint/resume", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      const text = res.ok
        ? `workpoint resume → ${summarizeWorkpointResponse(res.body)}\n${String(res.body?.rendered_summary || "")}`.trim()
        : `workpoint resume unavailable → ${explainWorkLoopResult(res, "resume failed")}`;
      return {
        content: [{ type: "text", text }],
        details: { ok: res.ok, status: res.status, endpoint: "/workpoint/resume", request: payload, response: res.body },
      };
    },
  });

  // ── Spec80 LLM-native tree/metacog tools ─────────────────────────────────

  function spec80ErrorCode(result: { ok: boolean; status: number; body: any | null }): string {
    if (result.ok) return "OK";
    const bodyCode = String(result.body?.code || result.body?.error || "").trim();
    if (bodyCode) return bodyCode;
    if (result.status === 0) return "DAEMON_UNAVAILABLE";
    if (result.status === 400) return "INVALID_REQUEST";
    if (result.status === 401) return "AUTH_REQUIRED";
    if (result.status === 403) return "AUTHORITY_DENIED";
    if (result.status === 404) return "NOT_FOUND";
    if (result.status === 409) return "CONFLICT";
    if (result.status === 422) return "SCHEMA_INVALID";
    if (result.status >= 500) return "SERVER_ERROR";
    return "REQUEST_FAILED";
  }

  function metacogQualityGate(input: { content?: string; rationale?: string; confidence?: number; evidence_refs?: string[] }) {
    const evidenceRefs = input.evidence_refs || [];
    const contentWords = String(input.content || "").trim().split(/\s+/).filter(Boolean).length;
    let score = 0;
    if (contentWords >= 8) score += 0.35;
    if (String(input.rationale || "").trim().length >= 20) score += 0.25;
    if ((input.confidence ?? 0) >= 0.5) score += 0.15;
    if (evidenceRefs.length > 0) score += 0.25;
    const passed = score >= 0.6;
    return { passed, score: Number(score.toFixed(2)), evidence_refs: evidenceRefs, recommendation: passed ? "eligible_for_retrieval" : "add rationale/evidence before promotion" };
  }

  function spec80Result(
    tool: string,
    endpoint: string,
    request: Record<string, any>,
    result: { ok: boolean; status: number; body: any | null },
    successText: string,
    fallbackText: string,
  ) {
    const text = result.ok && result.body ? successText : `${fallbackText} → ${explainWorkLoopResult(result, "ok")}`;
    return {
      content: [{ type: "text", text }],
      details: {
        ok: result.ok,
        status: result.status,
        code: spec80ErrorCode(result),
        tool,
        endpoint,
        request,
        response: result.body ?? null,
        quality_gate: tool.startsWith("focusa_metacog_") ? metacogQualityGate(request) : undefined,
        evidence_refs: Array.isArray(request.evidence_refs) ? request.evidence_refs : [],
        suggested_metrics: tool.startsWith("focusa_metacog_") ? ["retrieval_reuse", "promotion_precision", "failure_recurrence"] : undefined,
        timestamp: new Date().toISOString(),
      },
    } as any;
  }

  function spec80CompositeResult(
    tool: string,
    endpoint: string,
    request: Record<string, any>,
    ok: boolean,
    status: number,
    response: any,
    successText: string,
    fallbackText: string,
  ) {
    const result = { ok, status, body: response ?? null };
    return spec80Result(tool, endpoint, request, result, successText, fallbackText);
  }

  async function callSpec80Tool(
    tool: string,
    endpoint: string,
    request: Record<string, any>,
    opts: { method?: "GET" | "POST"; writer?: boolean } = {},
  ): Promise<{ ok: boolean; status: number; body: any | null; writerId?: string }> {
    const method = opts.method || "POST";
    const writerId = opts.writer ? await preferredWriterId() : undefined;
    const req: RequestInit = {
      method,
      headers: writerId ? { "x-focusa-writer-id": writerId } : undefined,
      body: method === "POST" ? JSON.stringify(request) : undefined,
    };
    const first = await focusaFetchDetailed(endpoint, req);
    const transient = new Set([0, 429, 502, 503, 504]);
    if (!first.ok && transient.has(first.status)) {
      await new Promise((resolve) => setTimeout(resolve, 150));
      const second = await focusaFetchDetailed(endpoint, req);
      return { ...second, writerId };
    }
    return { ...first, writerId };
  }

  const SPEC81_ID_PATTERN = "^[A-Za-z0-9._:-]+$";
  const SPEC81_TURN_RANGE_PATTERN = "^[A-Za-z0-9_.,:+\\-\\s]+$";
  const SPEC81_ID_RE = /^[A-Za-z0-9._:-]+$/;
  const SPEC81_TURN_RANGE_RE = /^[A-Za-z0-9_.,:+\-\s]+$/;
  const SPEC81_LIMITS = {
    sessionId: 160,
    id: 160,
    snapshotReason: 160,
    kind: 80,
    strategyClass: 80,
    shortText: 240,
    currentAsk: 500,
    rationale: 500,
    longText: 2000,
    turnRange: 120,
    scopeTags: 16,
    failureClasses: 16,
    selectedUpdates: 20,
    observedMetrics: 32,
    tagText: 80,
    updateText: 240,
    metricText: 120,
  };

  function spec80ValidationResult(
    tool: string,
    endpoint: string,
    request: Record<string, any>,
    fallbackText: string,
    error: string,
    code = "SCHEMA_INVALID",
  ) {
    return spec80Result(
      tool,
      endpoint,
      request,
      { ok: false, status: 422, body: { code, error } },
      `${fallbackText}: ok`,
      fallbackText,
    );
  }

  function validateRequiredString(
    name: string,
    value: unknown,
    maxLength: number,
    opts: { pattern?: RegExp } = {},
  ): { ok: true; value: string } | { ok: false; error: string } {
    const text = String(value ?? "").trim();
    if (!text) return { ok: false, error: `${name} required` };
    if (text.length > maxLength) return { ok: false, error: `${name} too long (max ${maxLength})` };
    if (opts.pattern && !opts.pattern.test(text)) return { ok: false, error: `${name} has invalid format` };
    return { ok: true, value: text };
  }

  function validateOptionalString(
    name: string,
    value: unknown,
    maxLength: number,
    opts: { pattern?: RegExp } = {},
  ): { ok: true; value: string | undefined } | { ok: false; error: string } {
    if (value === undefined || value === null) return { ok: true, value: undefined };
    const text = String(value).trim();
    if (!text) return { ok: true, value: undefined };
    if (text.length > maxLength) return { ok: false, error: `${name} too long (max ${maxLength})` };
    if (opts.pattern && !opts.pattern.test(text)) return { ok: false, error: `${name} has invalid format` };
    return { ok: true, value: text };
  }

  function validateStringArray(
    name: string,
    value: unknown,
    opts: { maxItems: number; itemMaxLength: number; pattern?: RegExp },
  ): { ok: true; value: string[] } | { ok: false; error: string } {
    if (value === undefined || value === null) return { ok: true, value: [] };
    if (!Array.isArray(value)) return { ok: false, error: `${name} must be an array` };
    if (value.length > opts.maxItems) return { ok: false, error: `${name} has too many items (max ${opts.maxItems})` };
    const normalized: string[] = [];
    for (const raw of value) {
      if (typeof raw !== "string") return { ok: false, error: `${name} items must be strings` };
      const item = raw.trim();
      if (!item) return { ok: false, error: `${name} items must not be blank` };
      if (item.length > opts.itemMaxLength) return { ok: false, error: `${name} item too long (max ${opts.itemMaxLength})` };
      if (opts.pattern && !opts.pattern.test(item)) return { ok: false, error: `${name} item has invalid format` };
      normalized.push(item);
    }
    return { ok: true, value: normalized };
  }

  function validateNoExtraKeys(
    tool: string,
    params: unknown,
    allowedKeys: string[],
  ): { ok: true; value: Record<string, any> } | { ok: false; error: string } {
    if (!params || typeof params !== "object" || Array.isArray(params)) {
      return { ok: false, error: `${tool} params must be an object` };
    }
    const record = params as Record<string, any>;
    const extras = Object.keys(record).filter((key) => !allowedKeys.includes(key));
    if (extras.length > 0) {
      return { ok: false, error: `unexpected parameter(s): ${extras.join(", ")}` };
    }
    return { ok: true, value: record };
  }

  function strictObject(properties: Record<string, any>) {
    return Type.Object(properties, { additionalProperties: false });
  }

  function summarizeArray(values: unknown[], limit = 3): string {
    if (!Array.isArray(values) || values.length === 0) return "none";
    return values.slice(0, limit).map((value) => String(value)).join(", ");
  }

  function boolLabel(value: unknown): string {
    return value ? "yes" : "no";
  }

  pi.registerTool({
    name: "focusa_tree_head",
    label: "Tree Head",
    description: "Best safe starting point for lineage work. Use first when you need current branch/head context before path, snapshot, diff, or restore work.",
    parameters: strictObject({
      session_id: Type.Optional(Type.String({ maxLength: SPEC81_LIMITS.sessionId, pattern: SPEC81_ID_PATTERN, description: "Optional session id scoping hint." })),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_tree_head", params, ["session_id"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult("focusa_tree_head", "/v1/lineage/head", params as Record<string, any>, "tree head", keyCheck.error);
      }
      const sessionIdCheck = validateOptionalString(
        "session_id",
        keyCheck.value.session_id,
        SPEC81_LIMITS.sessionId,
        { pattern: SPEC81_ID_RE },
      );
      if (!sessionIdCheck.ok) {
        return spec80ValidationResult("focusa_tree_head", "/v1/lineage/head", params as Record<string, any>, "tree head", sessionIdCheck.error);
      }
      const session_id = sessionIdCheck.value;
      const query = session_id ? `?session_id=${encodeURIComponent(session_id)}` : "";
      const req = { session_id: session_id || null };
      const res = await callSpec80Tool("focusa_tree_head", `/lineage/head${query}`, req, { method: "GET" });
      const head = String(res.body?.head || "unknown");
      const branch = String(res.body?.branch_id || "unknown");
      const session = String(res.body?.session_id || session_id || "global");
      return spec80Result(
        "focusa_tree_head",
        "/v1/lineage/head",
        req,
        res,
        `tree head: ${head}\nbranch=${branch} session=${session}\nnext_tools=focusa_tree_path,focusa_tree_snapshot_state`,
        "tree head",
      );
    },
  });

  pi.registerTool({
    name: "focusa_tree_path",
    label: "Tree Path",
    description: "Safe ancestry lookup. Use when branch position or lineage depth matters and you do not want to infer it from prior turns.",
    parameters: strictObject({
      clt_node_id: Type.String({ minLength: 1, maxLength: SPEC81_LIMITS.id, pattern: SPEC81_ID_PATTERN, description: "CLT node id." }),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_tree_path", params, ["clt_node_id"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult("focusa_tree_path", "/v1/lineage/path/{clt_node_id}", params as Record<string, any>, "tree path", keyCheck.error);
      }
      const nodeIdCheck = validateRequiredString(
        "clt_node_id",
        keyCheck.value.clt_node_id,
        SPEC81_LIMITS.id,
        { pattern: SPEC81_ID_RE },
      );
      if (!nodeIdCheck.ok) {
        return spec80ValidationResult("focusa_tree_path", "/v1/lineage/path/{clt_node_id}", params as Record<string, any>, "tree path", nodeIdCheck.error);
      }
      const nodeId = nodeIdCheck.value;
      const res = await callSpec80Tool("focusa_tree_path", `/lineage/path/${encodeURIComponent(nodeId)}`, { clt_node_id: nodeId }, { method: "GET" });
      const depth = Number(res.body?.depth || 0);
      const pathItems = Array.isArray(res.body?.path) ? res.body.path : [];
      return spec80Result(
        "focusa_tree_path",
        "/v1/lineage/path/{clt_node_id}",
        { clt_node_id: nodeId },
        res,
        `tree path: depth=${depth} nodes=${pathItems.length}\npath=${summarizeArray(pathItems, 5)}\nnext_tools=focusa_tree_snapshot_state,focusa_tree_diff_context`,
        "tree path",
      );
    },
  });

  pi.registerTool({
    name: "focusa_tree_snapshot_state",
    label: "Tree Snapshot State",
    description: "Create a recoverable checkpoint before risky work or comparisons. Best write tool for saving current state with a reason.",
    parameters: strictObject({
      clt_node_id: Type.Optional(Type.String({ maxLength: SPEC81_LIMITS.id, pattern: SPEC81_ID_PATTERN, description: "Optional CLT node id. Defaults to current head." })),
      snapshot_reason: Type.Optional(Type.String({ maxLength: SPEC81_LIMITS.snapshotReason, description: "Reason label for snapshot." })),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_tree_snapshot_state", params, ["clt_node_id", "snapshot_reason"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult("focusa_tree_snapshot_state", "/v1/focus/snapshots", params as Record<string, any>, "tree snapshot", keyCheck.error);
      }
      const raw = keyCheck.value as { clt_node_id?: string; snapshot_reason?: string };
      const nodeCheck = validateOptionalString("clt_node_id", raw.clt_node_id, SPEC81_LIMITS.id, { pattern: SPEC81_ID_RE });
      if (!nodeCheck.ok) {
        return spec80ValidationResult("focusa_tree_snapshot_state", "/v1/focus/snapshots", raw as Record<string, any>, "tree snapshot", nodeCheck.error);
      }
      const reasonCheck = validateOptionalString("snapshot_reason", raw.snapshot_reason, SPEC81_LIMITS.snapshotReason);
      if (!reasonCheck.ok) {
        return spec80ValidationResult("focusa_tree_snapshot_state", "/v1/focus/snapshots", raw as Record<string, any>, "tree snapshot", reasonCheck.error);
      }
      const req = { clt_node_id: nodeCheck.value || null, snapshot_reason: reasonCheck.value || null };
      const res = await callSpec80Tool("focusa_tree_snapshot_state", "/focus/snapshots", req, { method: "POST", writer: true });
      return spec80Result(
        "focusa_tree_snapshot_state",
        "/v1/focus/snapshots",
        { ...req, writer_id: res.writerId || null },
        res,
        `tree snapshot: ${String(res.body?.snapshot_id || "created")}\nclt_node=${String(res.body?.clt_node_id || req.clt_node_id || "current")} created_at=${String(res.body?.created_at || "unknown")}\nnext_tools=focusa_tree_diff_context,focusa_tree_restore_state`,
        "tree snapshot",
      );
    },
  });

  pi.registerTool({
    name: "focusa_tree_restore_state",
    label: "Tree Restore State",
    description: "Restore a saved checkpoint when you need rollback or exact/merge recovery. State-changing tool.",
    parameters: strictObject({
      snapshot_id: Type.String({ minLength: 1, maxLength: SPEC81_LIMITS.id, pattern: SPEC81_ID_PATTERN, description: "Snapshot id to restore." }),
      restore_mode: Type.Optional(Type.Union([Type.Literal("exact"), Type.Literal("merge")], { description: "Restore mode: exact|merge" })),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_tree_restore_state", params, ["snapshot_id", "restore_mode"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult("focusa_tree_restore_state", "/v1/focus/snapshots/restore", params as Record<string, any>, "tree restore", keyCheck.error);
      }
      const raw = keyCheck.value as { snapshot_id: string; restore_mode?: string };
      const sidCheck = validateRequiredString("snapshot_id", raw.snapshot_id, SPEC81_LIMITS.id, { pattern: SPEC81_ID_RE });
      if (!sidCheck.ok) {
        return spec80ValidationResult("focusa_tree_restore_state", "/v1/focus/snapshots/restore", raw as Record<string, any>, "tree restore", sidCheck.error);
      }
      const mode = String(raw.restore_mode || "exact").trim().toLowerCase();
      if (mode !== "exact" && mode !== "merge") {
        return spec80ValidationResult(
          "focusa_tree_restore_state",
          "/v1/focus/snapshots/restore",
          { snapshot_id: sidCheck.value, restore_mode: mode },
          "tree restore",
          "restore_mode must be exact|merge",
          "INVALID_REQUEST",
        );
      }
      const req = { snapshot_id: sidCheck.value, restore_mode: mode };
      const res = await callSpec80Tool("focusa_tree_restore_state", "/focus/snapshots/restore", req, { method: "POST", writer: true });
      const conflicts = Array.isArray(res.body?.conflicts) ? res.body.conflicts.length : 0;
      return spec80Result(
        "focusa_tree_restore_state",
        "/v1/focus/snapshots/restore",
        { ...req, writer_id: res.writerId || null },
        res,
        `tree restore: status=${String(res.body?.status || "ok")} snapshot=${String(res.body?.snapshot_id || req.snapshot_id)}\nmode=${mode} conflicts=${conflicts}\nnext_tools=focusa_tree_head,focusa_tree_path`,
        "tree restore",
      );
    },
  });

  pi.registerTool({
    name: "focusa_tree_diff_context",
    label: "Tree Diff Context",
    description: "Best safe compare tool for snapshots. Use this instead of guessing what changed across checkpoints.",
    parameters: strictObject({
      from_snapshot_id: Type.String({ minLength: 1, maxLength: SPEC81_LIMITS.id, pattern: SPEC81_ID_PATTERN, description: "Source snapshot id." }),
      to_snapshot_id: Type.String({ minLength: 1, maxLength: SPEC81_LIMITS.id, pattern: SPEC81_ID_PATTERN, description: "Target snapshot id." }),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_tree_diff_context", params, ["from_snapshot_id", "to_snapshot_id"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult("focusa_tree_diff_context", "/v1/focus/snapshots/diff", params as Record<string, any>, "tree diff", keyCheck.error);
      }
      const raw = keyCheck.value as { from_snapshot_id: string; to_snapshot_id: string };
      const fromCheck = validateRequiredString("from_snapshot_id", raw.from_snapshot_id, SPEC81_LIMITS.id, { pattern: SPEC81_ID_RE });
      if (!fromCheck.ok) {
        return spec80ValidationResult("focusa_tree_diff_context", "/v1/focus/snapshots/diff", raw as Record<string, any>, "tree diff", fromCheck.error);
      }
      const toCheck = validateRequiredString("to_snapshot_id", raw.to_snapshot_id, SPEC81_LIMITS.id, { pattern: SPEC81_ID_RE });
      if (!toCheck.ok) {
        return spec80ValidationResult("focusa_tree_diff_context", "/v1/focus/snapshots/diff", raw as Record<string, any>, "tree diff", toCheck.error);
      }
      const req = { from_snapshot_id: fromCheck.value, to_snapshot_id: toCheck.value };
      const res = await callSpec80Tool("focusa_tree_diff_context", "/focus/snapshots/diff", req, { method: "POST" });
      return spec80Result(
        "focusa_tree_diff_context",
        "/v1/focus/snapshots/diff",
        req,
        res,
        `tree diff: changed=${boolLabel(res.body?.checksum_changed)} version_delta=${String(res.body?.version_delta ?? "unknown")}\nclt_changed=${boolLabel(res.body?.clt_node_changed)} decisions_changed=${boolLabel(res.body?.decisions_delta?.changed)}\nnext_tools=focusa_tree_restore_state,focusa_tree_path`,
        "tree diff",
      );
    },
  });

  pi.registerTool({
    name: "focusa_metacog_capture",
    label: "Metacog Capture",
    description: "Store a reusable learning signal so future reasoning can retrieve it instead of rediscovering the same lesson.",
    parameters: strictObject({
      kind: Type.String({ minLength: 1, maxLength: SPEC81_LIMITS.kind, description: "Signal kind." }),
      content: Type.String({ minLength: 1, maxLength: SPEC81_LIMITS.longText, description: "Signal content." }),
      rationale: Type.Optional(Type.String({ maxLength: SPEC81_LIMITS.rationale, description: "Optional rationale." })),
      evidence_refs: Type.Optional(Type.Array(Type.String(), { description: "Evidence refs supporting this learning signal." })),
      confidence: Type.Optional(Type.Number({ minimum: 0, maximum: 1, description: "Optional confidence 0..1" })),
      strategy_class: Type.Optional(Type.String({ maxLength: SPEC81_LIMITS.strategyClass, description: "Optional strategy class." })),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_metacog_capture", params, ["kind", "content", "rationale", "evidence_refs", "confidence", "strategy_class"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult("focusa_metacog_capture", "/v1/metacognition/capture", params as Record<string, any>, "metacog capture", keyCheck.error);
      }
      const raw = keyCheck.value as { kind: string; content: string; rationale?: string; evidence_refs?: string[]; confidence?: number; strategy_class?: string };
      const kindCheck = validateRequiredString("kind", raw.kind, SPEC81_LIMITS.kind);
      if (!kindCheck.ok) {
        return spec80ValidationResult("focusa_metacog_capture", "/v1/metacognition/capture", raw as Record<string, any>, "metacog capture", kindCheck.error);
      }
      const contentCheck = validateRequiredString("content", raw.content, SPEC81_LIMITS.longText);
      if (!contentCheck.ok) {
        return spec80ValidationResult("focusa_metacog_capture", "/v1/metacognition/capture", raw as Record<string, any>, "metacog capture", contentCheck.error);
      }
      const rationaleCheck = validateOptionalString("rationale", raw.rationale, SPEC81_LIMITS.rationale);
      if (!rationaleCheck.ok) {
        return spec80ValidationResult("focusa_metacog_capture", "/v1/metacognition/capture", raw as Record<string, any>, "metacog capture", rationaleCheck.error);
      }
      const strategyCheck = validateOptionalString("strategy_class", raw.strategy_class, SPEC81_LIMITS.strategyClass);
      if (!strategyCheck.ok) {
        return spec80ValidationResult("focusa_metacog_capture", "/v1/metacognition/capture", raw as Record<string, any>, "metacog capture", strategyCheck.error);
      }
      if (raw.confidence !== undefined && (!Number.isFinite(raw.confidence) || raw.confidence < 0 || raw.confidence > 1)) {
        return spec80ValidationResult("focusa_metacog_capture", "/v1/metacognition/capture", raw as Record<string, any>, "metacog capture", "confidence must be between 0 and 1");
      }
      const req = {
        kind: kindCheck.value,
        content: contentCheck.value,
        rationale: rationaleCheck.value,
        evidence_refs: Array.isArray(raw.evidence_refs) ? raw.evidence_refs.slice(0, 8) : [],
        confidence: raw.confidence,
        strategy_class: strategyCheck.value,
      };
      const res = await callSpec80Tool("focusa_metacog_capture", "/metacognition/capture", req, { method: "POST", writer: true });
      return spec80Result(
        "focusa_metacog_capture",
        "/v1/metacognition/capture",
        { ...req, writer_id: res.writerId || null },
        res,
        `metacog capture: ${String(res.body?.capture_id || "stored")}\nkind=${req.kind} confidence=${req.confidence ?? "n/a"} strategy_class=${req.strategy_class || "none"}\nnext_tools=focusa_metacog_retrieve,focusa_metacog_reflect`,
        "metacog capture",
      );
    },
  });

  pi.registerTool({
    name: "focusa_metacog_retrieve",
    label: "Metacog Retrieve",
    description: "Best safe search tool for past learning signals relevant to the current ask. Use this before planning or reflection.",
    parameters: strictObject({
      current_ask: Type.String({ minLength: 1, maxLength: SPEC81_LIMITS.currentAsk, description: "Current ask." }),
      scope_tags: Type.Optional(Type.Array(Type.String({ maxLength: SPEC81_LIMITS.tagText, description: "Optional scope tag." }), { maxItems: SPEC81_LIMITS.scopeTags, description: "Optional scope tags." })),
      k: Type.Optional(Type.Integer({ minimum: 1, maximum: 50, description: "Top-k candidates (default 5)." })),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_metacog_retrieve", params, ["current_ask", "scope_tags", "k"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult("focusa_metacog_retrieve", "/v1/metacognition/retrieve", params as Record<string, any>, "metacog retrieve", keyCheck.error);
      }
      const raw = keyCheck.value as { current_ask: string; scope_tags?: string[]; k?: number };
      const askCheck = validateRequiredString("current_ask", raw.current_ask, SPEC81_LIMITS.currentAsk);
      if (!askCheck.ok) {
        return spec80ValidationResult("focusa_metacog_retrieve", "/v1/metacognition/retrieve", raw as Record<string, any>, "metacog retrieve", askCheck.error);
      }
      const tagsCheck = validateStringArray("scope_tags", raw.scope_tags, { maxItems: SPEC81_LIMITS.scopeTags, itemMaxLength: SPEC81_LIMITS.tagText });
      if (!tagsCheck.ok) {
        return spec80ValidationResult("focusa_metacog_retrieve", "/v1/metacognition/retrieve", raw as Record<string, any>, "metacog retrieve", tagsCheck.error);
      }
      let normalizedK = Math.trunc(Number(raw.k ?? 5));
      if (!Number.isFinite(normalizedK)) normalizedK = 5;
      normalizedK = Math.max(1, Math.min(50, normalizedK));
      const req = { current_ask: askCheck.value, scope_tags: tagsCheck.value, k: normalizedK };
      const res = await callSpec80Tool("focusa_metacog_retrieve", "/metacognition/retrieve", req, { method: "POST" });
      const candidates = Array.isArray(res.body?.candidates) ? res.body.candidates : [];
      const total = candidates.length;
      const top = candidates[0];
      return spec80Result(
        "focusa_metacog_retrieve",
        "/v1/metacognition/retrieve",
        req,
        res,
        total > 0
          ? `metacog retrieve: candidates=${total} top_capture=${String(top?.capture_id || "none")}\ntop_kind=${String(top?.kind || "unknown")} top_score=${String(top?.score ?? "n/a")}\nnext_tools=focusa_metacog_reflect,focusa_metacog_plan_adjust`
          : `metacog retrieve: candidates=0\nno prior signals matched\nnext_tools=focusa_metacog_capture,focusa_metacog_reflect`,
        "metacog retrieve",
      );
    },
  });

  pi.registerTool({
    name: "focusa_metacog_reflect",
    label: "Metacog Reflect",
    description: "Generate reusable hypotheses and strategy updates from recent turns when you need learning from past outcomes.",
    parameters: strictObject({
      turn_range: Type.String({ minLength: 1, maxLength: SPEC81_LIMITS.turnRange, pattern: SPEC81_TURN_RANGE_PATTERN, description: "Turn range expression." }),
      failure_classes: Type.Optional(Type.Array(Type.String({ maxLength: SPEC81_LIMITS.tagText, description: "Failure class tag." }), { maxItems: SPEC81_LIMITS.failureClasses, description: "Failure class tags." })),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_metacog_reflect", params, ["turn_range", "failure_classes"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult("focusa_metacog_reflect", "/v1/metacognition/reflect", params as Record<string, any>, "metacog reflect", keyCheck.error);
      }
      const raw = keyCheck.value as { turn_range: string; failure_classes?: string[] };
      const turnRangeCheck = validateRequiredString("turn_range", raw.turn_range, SPEC81_LIMITS.turnRange, { pattern: SPEC81_TURN_RANGE_RE });
      if (!turnRangeCheck.ok) {
        return spec80ValidationResult("focusa_metacog_reflect", "/v1/metacognition/reflect", raw as Record<string, any>, "metacog reflect", turnRangeCheck.error);
      }
      const failureCheck = validateStringArray("failure_classes", raw.failure_classes, { maxItems: SPEC81_LIMITS.failureClasses, itemMaxLength: SPEC81_LIMITS.tagText });
      if (!failureCheck.ok) {
        return spec80ValidationResult("focusa_metacog_reflect", "/v1/metacognition/reflect", raw as Record<string, any>, "metacog reflect", failureCheck.error);
      }
      const req = { turn_range: turnRangeCheck.value, failure_classes: failureCheck.value };
      const res = await callSpec80Tool("focusa_metacog_reflect", "/metacognition/reflect", req, { method: "POST", writer: true });
      const updates = Array.isArray(res.body?.strategy_updates) ? res.body.strategy_updates : [];
      return spec80Result(
        "focusa_metacog_reflect",
        "/v1/metacognition/reflect",
        { ...req, writer_id: res.writerId || null },
        res,
        `metacog reflect: ${String(res.body?.reflection_id || "ok")} hypotheses=${Array.isArray(res.body?.hypotheses) ? res.body.hypotheses.length : 0}\nstrategy_updates=${summarizeArray(updates, 4)}\nnext_tools=focusa_metacog_plan_adjust,focusa_metacog_doctor`,
        "metacog reflect",
      );
    },
  });

  pi.registerTool({
    name: "focusa_metacog_plan_adjust",
    label: "Metacog Plan Adjust",
    description: "Turn a reflection into a tracked adjustment artifact that can later be evaluated for real improvement.",
    parameters: strictObject({
      reflection_id: Type.String({ minLength: 1, maxLength: SPEC81_LIMITS.id, pattern: SPEC81_ID_PATTERN, description: "Reflection id." }),
      selected_updates: Type.Optional(Type.Array(Type.String({ maxLength: SPEC81_LIMITS.updateText, description: "Selected update." }), { maxItems: SPEC81_LIMITS.selectedUpdates, description: "Selected updates." })),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_metacog_plan_adjust", params, ["reflection_id", "selected_updates"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult("focusa_metacog_plan_adjust", "/v1/metacognition/adjust", params as Record<string, any>, "metacog adjust", keyCheck.error);
      }
      const raw = keyCheck.value as { reflection_id: string; selected_updates?: string[] };
      const reflectionCheck = validateRequiredString("reflection_id", raw.reflection_id, SPEC81_LIMITS.id, { pattern: SPEC81_ID_RE });
      if (!reflectionCheck.ok) {
        return spec80ValidationResult("focusa_metacog_plan_adjust", "/v1/metacognition/adjust", raw as Record<string, any>, "metacog adjust", reflectionCheck.error);
      }
      const updatesCheck = validateStringArray("selected_updates", raw.selected_updates, { maxItems: SPEC81_LIMITS.selectedUpdates, itemMaxLength: SPEC81_LIMITS.updateText });
      if (!updatesCheck.ok) {
        return spec80ValidationResult("focusa_metacog_plan_adjust", "/v1/metacognition/adjust", raw as Record<string, any>, "metacog adjust", updatesCheck.error);
      }
      const req = { reflection_id: reflectionCheck.value, selected_updates: updatesCheck.value };
      const res = await callSpec80Tool("focusa_metacog_plan_adjust", "/metacognition/adjust", req, { method: "POST", writer: true });
      return spec80Result(
        "focusa_metacog_plan_adjust",
        "/v1/metacognition/adjust",
        { ...req, writer_id: res.writerId || null },
        res,
        `metacog adjust: ${String(res.body?.adjustment_id || "ok")} updates=${updatesCheck.value.length}\nnext_step_policy=${summarizeArray(res.body?.next_step_policy || updatesCheck.value, 4)}\nnext_tools=focusa_metacog_evaluate_outcome,focusa_metacog_doctor`,
        "metacog adjust",
      );
    },
  });

  pi.registerTool({
    name: "focusa_metacog_evaluate_outcome",
    label: "Metacog Evaluate Outcome",
    description: "Judge whether an adjustment improved results and whether the learning should be promoted.",
    parameters: strictObject({
      adjustment_id: Type.String({ minLength: 1, maxLength: SPEC81_LIMITS.id, pattern: SPEC81_ID_PATTERN, description: "Adjustment id." }),
      observed_metrics: Type.Optional(Type.Array(Type.String({ maxLength: SPEC81_LIMITS.metricText, description: "Observed metric id." }), { maxItems: SPEC81_LIMITS.observedMetrics, description: "Observed metric ids." })),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_metacog_evaluate_outcome", params, ["adjustment_id", "observed_metrics"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult("focusa_metacog_evaluate_outcome", "/v1/metacognition/evaluate", params as Record<string, any>, "metacog evaluate", keyCheck.error);
      }
      const raw = keyCheck.value as { adjustment_id: string; observed_metrics?: string[] };
      const adjustmentCheck = validateRequiredString("adjustment_id", raw.adjustment_id, SPEC81_LIMITS.id, { pattern: SPEC81_ID_RE });
      if (!adjustmentCheck.ok) {
        return spec80ValidationResult("focusa_metacog_evaluate_outcome", "/v1/metacognition/evaluate", raw as Record<string, any>, "metacog evaluate", adjustmentCheck.error);
      }
      const metricsCheck = validateStringArray("observed_metrics", raw.observed_metrics, { maxItems: SPEC81_LIMITS.observedMetrics, itemMaxLength: SPEC81_LIMITS.metricText });
      if (!metricsCheck.ok) {
        return spec80ValidationResult("focusa_metacog_evaluate_outcome", "/v1/metacognition/evaluate", raw as Record<string, any>, "metacog evaluate", metricsCheck.error);
      }
      const req = { adjustment_id: adjustmentCheck.value, observed_metrics: metricsCheck.value };
      const res = await callSpec80Tool("focusa_metacog_evaluate_outcome", "/metacognition/evaluate", req, { method: "POST", writer: true });
      const observed = Array.isArray(res.body?.delta_scorecard?.metrics_observed)
        ? res.body.delta_scorecard.metrics_observed
        : metricsCheck.value;
      return spec80Result(
        "focusa_metacog_evaluate_outcome",
        "/v1/metacognition/evaluate",
        { ...req, writer_id: res.writerId || null },
        res,
        `metacog evaluate: decision=${String(res.body?.result || "unknown")} promote=${boolLabel(res.body?.promote_learning)}\nobserved_metrics=${summarizeArray(observed, 4)}\nnext_tools=focusa_metacog_doctor,focusa_metacog_recent_adjustments`,
        "metacog evaluate",
      );
    },
  });

  pi.registerTool({
    name: "focusa_tree_recent_snapshots",
    label: "Tree Recent Snapshots",
    description: "Best safe helper for finding recent snapshot ids. Use this before diff or restore when you do not already know the right snapshot id.",
    parameters: strictObject({
      limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 20, description: "How many recent snapshots to return (default 5)." })),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_tree_recent_snapshots", params, ["limit"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult("focusa_tree_recent_snapshots", "/v1/focus/snapshots/recent", params as Record<string, any>, "tree recent snapshots", keyCheck.error);
      }
      let limit = Math.trunc(Number((keyCheck.value as { limit?: number }).limit ?? 5));
      if (!Number.isFinite(limit)) limit = 5;
      limit = Math.max(1, Math.min(20, limit));
      const endpoint = `/focus/snapshots/recent?limit=${limit}`;
      const res = await callSpec80Tool("focusa_tree_recent_snapshots", endpoint, { limit }, { method: "GET" });
      const items = Array.isArray(res.body?.snapshots) ? res.body.snapshots : [];
      const ids = items.map((item: any) => item?.snapshot_id).filter(Boolean);
      return spec80Result(
        "focusa_tree_recent_snapshots",
        "/v1/focus/snapshots/recent",
        { limit },
        res,
        items.length > 0
          ? `tree recent snapshots: total=${items.length} ids=${summarizeArray(ids, 4)}\nnext_tools=focusa_tree_diff_context,focusa_tree_snapshot_compare_latest`
          : `tree recent snapshots: total=0\nno prior snapshots available\nnext_tools=focusa_tree_snapshot_state`,
        "tree recent snapshots",
      );
    },
  });

  pi.registerTool({
    name: "focusa_tree_snapshot_compare_latest",
    label: "Tree Snapshot Compare Latest",
    description: "Create a fresh snapshot and compare it to the latest prior snapshot in one move. Best tool when you want checkpoint + diff without manual id hunting.",
    parameters: strictObject({
      snapshot_reason: Type.Optional(Type.String({ maxLength: SPEC81_LIMITS.snapshotReason, description: "Reason label for the new snapshot." })),
      baseline_snapshot_id: Type.Optional(Type.String({ maxLength: SPEC81_LIMITS.id, pattern: SPEC81_ID_PATTERN, description: "Optional explicit baseline snapshot id." })),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_tree_snapshot_compare_latest", params, ["snapshot_reason", "baseline_snapshot_id"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult("focusa_tree_snapshot_compare_latest", "/v1/focus/snapshots/recent+create+diff", params as Record<string, any>, "tree snapshot compare latest", keyCheck.error);
      }
      const raw = keyCheck.value as { snapshot_reason?: string; baseline_snapshot_id?: string };
      const reasonCheck = validateOptionalString("snapshot_reason", raw.snapshot_reason, SPEC81_LIMITS.snapshotReason);
      if (!reasonCheck.ok) {
        return spec80ValidationResult("focusa_tree_snapshot_compare_latest", "/v1/focus/snapshots/recent+create+diff", raw as Record<string, any>, "tree snapshot compare latest", reasonCheck.error);
      }
      const baselineCheck = validateOptionalString("baseline_snapshot_id", raw.baseline_snapshot_id, SPEC81_LIMITS.id, { pattern: SPEC81_ID_RE });
      if (!baselineCheck.ok) {
        return spec80ValidationResult("focusa_tree_snapshot_compare_latest", "/v1/focus/snapshots/recent+create+diff", raw as Record<string, any>, "tree snapshot compare latest", baselineCheck.error);
      }

      let baselineSnapshotId = baselineCheck.value;
      if (!baselineSnapshotId) {
        const recentRes = await callSpec80Tool("focusa_tree_snapshot_compare_latest", "/focus/snapshots/recent?limit=1", { limit: 1 }, { method: "GET" });
        if (recentRes.ok) {
          baselineSnapshotId = recentRes.body?.snapshots?.[0]?.snapshot_id;
        }
      }

      const createReq = { snapshot_reason: reasonCheck.value || null };
      const createRes = await callSpec80Tool("focusa_tree_snapshot_compare_latest", "/focus/snapshots", createReq, { method: "POST", writer: true });
      if (!createRes.ok || !createRes.body?.snapshot_id) {
        return spec80CompositeResult(
          "focusa_tree_snapshot_compare_latest",
          "/v1/focus/snapshots/recent+create+diff",
          { ...createReq, baseline_snapshot_id: baselineSnapshotId || null },
          false,
          createRes.status,
          createRes.body,
          "tree snapshot compare latest: ok",
          "tree snapshot compare latest",
        );
      }

      const newSnapshotId = String(createRes.body.snapshot_id);
      if (!baselineSnapshotId) {
        return spec80CompositeResult(
          "focusa_tree_snapshot_compare_latest",
          "/v1/focus/snapshots/recent+create+diff",
          { ...createReq, baseline_snapshot_id: null, writer_id: createRes.writerId || null },
          true,
          createRes.status,
          { snapshot_id: newSnapshotId, baseline_snapshot_id: null, diff: null },
          `tree snapshot compare latest: created=${newSnapshotId}\nno prior snapshot to compare\nnext_tools=focusa_tree_recent_snapshots,focusa_tree_snapshot_state`,
          "tree snapshot compare latest",
        );
      }

      const diffReq = { from_snapshot_id: baselineSnapshotId, to_snapshot_id: newSnapshotId };
      const diffRes = await callSpec80Tool("focusa_tree_snapshot_compare_latest", "/focus/snapshots/diff", diffReq, { method: "POST" });
      return spec80CompositeResult(
        "focusa_tree_snapshot_compare_latest",
        "/v1/focus/snapshots/recent+create+diff",
        { ...createReq, baseline_snapshot_id: baselineSnapshotId, writer_id: createRes.writerId || null },
        diffRes.ok,
        diffRes.status,
        {
          snapshot_id: newSnapshotId,
          baseline_snapshot_id: baselineSnapshotId,
          diff: diffRes.body,
        },
        `tree snapshot compare latest: new=${newSnapshotId} baseline=${baselineSnapshotId}\nchanged=${boolLabel(diffRes.body?.checksum_changed)} version_delta=${String(diffRes.body?.version_delta ?? "unknown")}\nnext_tools=focusa_tree_restore_state,focusa_tree_path`,
        "tree snapshot compare latest",
      );
    },
  });

  pi.registerTool({
    name: "focusa_metacog_recent_reflections",
    label: "Metacog Recent Reflections",
    description: "Best safe helper for finding recent reflection ids and update sets before adjust or promote work.",
    parameters: strictObject({
      limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 20, description: "How many recent reflections to return (default 5)." })),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_metacog_recent_reflections", params, ["limit"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult("focusa_metacog_recent_reflections", "/v1/metacognition/reflections/recent", params as Record<string, any>, "metacog recent reflections", keyCheck.error);
      }
      let limit = Math.trunc(Number((keyCheck.value as { limit?: number }).limit ?? 5));
      if (!Number.isFinite(limit)) limit = 5;
      limit = Math.max(1, Math.min(20, limit));
      const endpoint = `/metacognition/reflections/recent?limit=${limit}`;
      const res = await callSpec80Tool("focusa_metacog_recent_reflections", endpoint, { limit }, { method: "GET" });
      const items = Array.isArray(res.body?.reflections) ? res.body.reflections : [];
      const ids = items.map((item: any) => item?.reflection_id).filter(Boolean);
      return spec80Result(
        "focusa_metacog_recent_reflections",
        "/v1/metacognition/reflections/recent",
        { limit },
        res,
        items.length > 0
          ? `metacog recent reflections: total=${items.length} ids=${summarizeArray(ids, 4)}\nnext_tools=focusa_metacog_plan_adjust,focusa_metacog_loop_run`
          : `metacog recent reflections: total=0\nno prior reflections available\nnext_tools=focusa_metacog_reflect`,
        "metacog recent reflections",
      );
    },
  });

  pi.registerTool({
    name: "focusa_metacog_recent_adjustments",
    label: "Metacog Recent Adjustments",
    description: "Best safe helper for finding recent adjustment ids before evaluation or promotion decisions.",
    parameters: strictObject({
      limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 20, description: "How many recent adjustments to return (default 5)." })),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_metacog_recent_adjustments", params, ["limit"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult("focusa_metacog_recent_adjustments", "/v1/metacognition/adjustments/recent", params as Record<string, any>, "metacog recent adjustments", keyCheck.error);
      }
      let limit = Math.trunc(Number((keyCheck.value as { limit?: number }).limit ?? 5));
      if (!Number.isFinite(limit)) limit = 5;
      limit = Math.max(1, Math.min(20, limit));
      const endpoint = `/metacognition/adjustments/recent?limit=${limit}`;
      const res = await callSpec80Tool("focusa_metacog_recent_adjustments", endpoint, { limit }, { method: "GET" });
      const items = Array.isArray(res.body?.adjustments) ? res.body.adjustments : [];
      const ids = items.map((item: any) => item?.adjustment_id).filter(Boolean);
      return spec80Result(
        "focusa_metacog_recent_adjustments",
        "/v1/metacognition/adjustments/recent",
        { limit },
        res,
        items.length > 0
          ? `metacog recent adjustments: total=${items.length} ids=${summarizeArray(ids, 4)}\nnext_tools=focusa_metacog_evaluate_outcome,focusa_metacog_doctor`
          : `metacog recent adjustments: total=0\nno prior adjustments available\nnext_tools=focusa_metacog_plan_adjust`,
        "metacog recent adjustments",
      );
    },
  });

  pi.registerTool({
    name: "focusa_metacog_loop_run",
    label: "Metacog Loop Run",
    description: "Run capture -> retrieve -> reflect -> adjust -> evaluate in one move. Best composite tool when you want learning workflow compression instead of manual chaining.",
    parameters: strictObject({
      current_ask: Type.String({ minLength: 1, maxLength: SPEC81_LIMITS.currentAsk, description: "Current ask driving retrieval and reuse." }),
      turn_range: Type.String({ minLength: 1, maxLength: SPEC81_LIMITS.turnRange, pattern: SPEC81_TURN_RANGE_PATTERN, description: "Turn range expression for reflection." }),
      kind: Type.Optional(Type.String({ maxLength: SPEC81_LIMITS.kind, description: "Optional capture kind (default workflow_signal)." })),
      content: Type.Optional(Type.String({ maxLength: SPEC81_LIMITS.longText, description: "Optional capture content; defaults to current_ask." })),
      rationale: Type.Optional(Type.String({ maxLength: SPEC81_LIMITS.rationale, description: "Optional capture rationale." })),
      confidence: Type.Optional(Type.Number({ minimum: 0, maximum: 1, description: "Optional confidence 0..1." })),
      strategy_class: Type.Optional(Type.String({ maxLength: SPEC81_LIMITS.strategyClass, description: "Optional strategy class." })),
      scope_tags: Type.Optional(Type.Array(Type.String({ maxLength: SPEC81_LIMITS.tagText }), { maxItems: SPEC81_LIMITS.scopeTags })),
      k: Type.Optional(Type.Integer({ minimum: 1, maximum: 50, description: "Top-k retrieval size." })),
      failure_classes: Type.Optional(Type.Array(Type.String({ maxLength: SPEC81_LIMITS.tagText }), { maxItems: SPEC81_LIMITS.failureClasses })),
      selected_updates: Type.Optional(Type.Array(Type.String({ maxLength: SPEC81_LIMITS.updateText }), { maxItems: SPEC81_LIMITS.selectedUpdates })),
      observed_metrics: Type.Optional(Type.Array(Type.String({ maxLength: SPEC81_LIMITS.metricText }), { maxItems: SPEC81_LIMITS.observedMetrics })),
    }),
    async execute(_id, params) {
      const allowed = ["current_ask", "turn_range", "kind", "content", "rationale", "confidence", "strategy_class", "scope_tags", "k", "failure_classes", "selected_updates", "observed_metrics"];
      const keyCheck = validateNoExtraKeys("focusa_metacog_loop_run", params, allowed);
      if (!keyCheck.ok) {
        return spec80ValidationResult("focusa_metacog_loop_run", "/v1/metacognition/loop-run", params as Record<string, any>, "metacog loop run", keyCheck.error);
      }
      const raw = keyCheck.value as Record<string, any>;
      const askCheck = validateRequiredString("current_ask", raw.current_ask, SPEC81_LIMITS.currentAsk);
      if (!askCheck.ok) return spec80ValidationResult("focusa_metacog_loop_run", "/v1/metacognition/loop-run", raw, "metacog loop run", askCheck.error);
      const turnCheck = validateRequiredString("turn_range", raw.turn_range, SPEC81_LIMITS.turnRange, { pattern: SPEC81_TURN_RANGE_RE });
      if (!turnCheck.ok) return spec80ValidationResult("focusa_metacog_loop_run", "/v1/metacognition/loop-run", raw, "metacog loop run", turnCheck.error);
      const kindCheck = validateOptionalString("kind", raw.kind, SPEC81_LIMITS.kind);
      if (!kindCheck.ok) return spec80ValidationResult("focusa_metacog_loop_run", "/v1/metacognition/loop-run", raw, "metacog loop run", kindCheck.error);
      const contentCheck = validateOptionalString("content", raw.content, SPEC81_LIMITS.longText);
      if (!contentCheck.ok) return spec80ValidationResult("focusa_metacog_loop_run", "/v1/metacognition/loop-run", raw, "metacog loop run", contentCheck.error);
      const rationaleCheck = validateOptionalString("rationale", raw.rationale, SPEC81_LIMITS.rationale);
      if (!rationaleCheck.ok) return spec80ValidationResult("focusa_metacog_loop_run", "/v1/metacognition/loop-run", raw, "metacog loop run", rationaleCheck.error);
      const strategyCheck = validateOptionalString("strategy_class", raw.strategy_class, SPEC81_LIMITS.strategyClass);
      if (!strategyCheck.ok) return spec80ValidationResult("focusa_metacog_loop_run", "/v1/metacognition/loop-run", raw, "metacog loop run", strategyCheck.error);
      if (raw.confidence !== undefined && (!Number.isFinite(raw.confidence) || raw.confidence < 0 || raw.confidence > 1)) {
        return spec80ValidationResult("focusa_metacog_loop_run", "/v1/metacognition/loop-run", raw, "metacog loop run", "confidence must be between 0 and 1");
      }
      const tagsCheck = validateStringArray("scope_tags", raw.scope_tags, { maxItems: SPEC81_LIMITS.scopeTags, itemMaxLength: SPEC81_LIMITS.tagText });
      if (!tagsCheck.ok) return spec80ValidationResult("focusa_metacog_loop_run", "/v1/metacognition/loop-run", raw, "metacog loop run", tagsCheck.error);
      const failuresCheck = validateStringArray("failure_classes", raw.failure_classes, { maxItems: SPEC81_LIMITS.failureClasses, itemMaxLength: SPEC81_LIMITS.tagText });
      if (!failuresCheck.ok) return spec80ValidationResult("focusa_metacog_loop_run", "/v1/metacognition/loop-run", raw, "metacog loop run", failuresCheck.error);
      const selectedCheck = validateStringArray("selected_updates", raw.selected_updates, { maxItems: SPEC81_LIMITS.selectedUpdates, itemMaxLength: SPEC81_LIMITS.updateText });
      if (!selectedCheck.ok) return spec80ValidationResult("focusa_metacog_loop_run", "/v1/metacognition/loop-run", raw, "metacog loop run", selectedCheck.error);
      const metricsCheck = validateStringArray("observed_metrics", raw.observed_metrics, { maxItems: SPEC81_LIMITS.observedMetrics, itemMaxLength: SPEC81_LIMITS.metricText });
      if (!metricsCheck.ok) return spec80ValidationResult("focusa_metacog_loop_run", "/v1/metacognition/loop-run", raw, "metacog loop run", metricsCheck.error);
      let normalizedK = Math.trunc(Number(raw.k ?? 5));
      if (!Number.isFinite(normalizedK)) normalizedK = 5;
      normalizedK = Math.max(1, Math.min(50, normalizedK));

      const captureReq = {
        kind: kindCheck.value || "workflow_signal",
        content: contentCheck.value || askCheck.value,
        rationale: rationaleCheck.value,
        confidence: raw.confidence,
        strategy_class: strategyCheck.value,
      };
      const captureRes = await callSpec80Tool("focusa_metacog_loop_run", "/metacognition/capture", captureReq, { method: "POST", writer: true });
      if (!captureRes.ok) {
        return spec80CompositeResult("focusa_metacog_loop_run", "/v1/metacognition/loop-run", raw, false, captureRes.status, captureRes.body, "metacog loop run: ok", "metacog loop run");
      }
      const retrieveReq = { current_ask: askCheck.value, scope_tags: tagsCheck.value, k: normalizedK };
      const retrieveRes = await callSpec80Tool("focusa_metacog_loop_run", "/metacognition/retrieve", retrieveReq, { method: "POST" });
      if (!retrieveRes.ok) {
        return spec80CompositeResult("focusa_metacog_loop_run", "/v1/metacognition/loop-run", raw, false, retrieveRes.status, retrieveRes.body, "metacog loop run: ok", "metacog loop run");
      }
      const reflectReq = { turn_range: turnCheck.value, failure_classes: failuresCheck.value };
      const reflectRes = await callSpec80Tool("focusa_metacog_loop_run", "/metacognition/reflect", reflectReq, { method: "POST", writer: true });
      if (!reflectRes.ok || !reflectRes.body?.reflection_id) {
        return spec80CompositeResult("focusa_metacog_loop_run", "/v1/metacognition/loop-run", raw, false, reflectRes.status, reflectRes.body, "metacog loop run: ok", "metacog loop run");
      }
      const updates = selectedCheck.value.length > 0
        ? selectedCheck.value
        : (Array.isArray(reflectRes.body?.strategy_updates) ? reflectRes.body.strategy_updates.map((x: any) => String(x)) : []);
      const adjustReq = { reflection_id: String(reflectRes.body.reflection_id), selected_updates: updates };
      const adjustRes = await callSpec80Tool("focusa_metacog_loop_run", "/metacognition/adjust", adjustReq, { method: "POST", writer: true });
      if (!adjustRes.ok || !adjustRes.body?.adjustment_id) {
        return spec80CompositeResult("focusa_metacog_loop_run", "/v1/metacognition/loop-run", raw, false, adjustRes.status, adjustRes.body, "metacog loop run: ok", "metacog loop run");
      }
      const evaluateReq = { adjustment_id: String(adjustRes.body.adjustment_id), observed_metrics: metricsCheck.value };
      const evaluateRes = await callSpec80Tool("focusa_metacog_loop_run", "/metacognition/evaluate", evaluateReq, { method: "POST", writer: true });
      return spec80CompositeResult(
        "focusa_metacog_loop_run",
        "/v1/metacognition/loop-run",
        raw,
        evaluateRes.ok,
        evaluateRes.status,
        {
          capture: captureRes.body,
          retrieve: retrieveRes.body,
          reflect: reflectRes.body,
          adjust: adjustRes.body,
          evaluate: evaluateRes.body,
        },
        `metacog loop run: result=${String(evaluateRes.body?.result || "unknown")} promote=${boolLabel(evaluateRes.body?.promote_learning)}\nreflection=${String(reflectRes.body?.reflection_id || "unknown")} adjustment=${String(adjustRes.body?.adjustment_id || "unknown")}\nnext_tools=focusa_metacog_doctor,focusa_metacog_evaluate_outcome`,
        "metacog loop run",
      );
    },
  });

  pi.registerTool({
    name: "focusa_metacog_doctor",
    label: "Metacog Doctor",
    description: "Diagnose signal quality and retrieval usefulness in one move. Best safe diagnostic tool when deciding whether more capture or reflection work is needed.",
    parameters: strictObject({
      current_ask: Type.String({ minLength: 1, maxLength: SPEC81_LIMITS.currentAsk, description: "Current ask to diagnose against." }),
      scope_tags: Type.Optional(Type.Array(Type.String({ maxLength: SPEC81_LIMITS.tagText }), { maxItems: SPEC81_LIMITS.scopeTags })),
      k: Type.Optional(Type.Integer({ minimum: 1, maximum: 50, description: "Top-k retrieval size." })),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_metacog_doctor", params, ["current_ask", "scope_tags", "k"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult("focusa_metacog_doctor", "/v1/metacognition/doctor", params as Record<string, any>, "metacog doctor", keyCheck.error);
      }
      const raw = keyCheck.value as { current_ask: string; scope_tags?: string[]; k?: number };
      const askCheck = validateRequiredString("current_ask", raw.current_ask, SPEC81_LIMITS.currentAsk);
      if (!askCheck.ok) return spec80ValidationResult("focusa_metacog_doctor", "/v1/metacognition/doctor", raw as Record<string, any>, "metacog doctor", askCheck.error);
      const tagsCheck = validateStringArray("scope_tags", raw.scope_tags, { maxItems: SPEC81_LIMITS.scopeTags, itemMaxLength: SPEC81_LIMITS.tagText });
      if (!tagsCheck.ok) return spec80ValidationResult("focusa_metacog_doctor", "/v1/metacognition/doctor", raw as Record<string, any>, "metacog doctor", tagsCheck.error);
      let normalizedK = Math.trunc(Number(raw.k ?? 5));
      if (!Number.isFinite(normalizedK)) normalizedK = 5;
      normalizedK = Math.max(1, Math.min(50, normalizedK));
      const req = { current_ask: askCheck.value, scope_tags: tagsCheck.value, k: normalizedK, summary_only: true };
      const res = await callSpec80Tool("focusa_metacog_doctor", "/metacognition/retrieve", req, { method: "POST" });
      const candidates = Array.isArray(res.body?.candidates) ? res.body.candidates : [];
      const withConfidence = candidates.filter((item: any) => item?.confidence !== null && item?.confidence !== undefined).length;
      const top = candidates[0];
      return spec80Result(
        "focusa_metacog_doctor",
        "/v1/metacognition/doctor",
        { current_ask: askCheck.value, scope_tags: tagsCheck.value, k: normalizedK },
        { ok: res.ok, status: res.status, body: { ...(res.body || {}), diagnostics: { candidate_count: candidates.length, with_confidence: withConfidence, top_kind: top?.kind || null, top_capture_id: top?.capture_id || null } } },
        candidates.length > 0
          ? `metacog doctor: candidates=${candidates.length} with_confidence=${withConfidence}\ntop_kind=${String(top?.kind || "unknown")} top_capture=${String(top?.capture_id || "none")}\nnext_tools=focusa_metacog_reflect,focusa_metacog_loop_run`
          : `metacog doctor: candidates=0\nno usable prior signals found\nnext_tools=focusa_metacog_capture,focusa_metacog_reflect`,
        "metacog doctor",
      );
    },
  });

  // ── Lineage Intelligence (LI) /tree first-class tools ────────────────────

  pi.registerTool({
    name: "focusa_lineage_tree",
    label: "Lineage Tree",
    description: "Fetch Focusa lineage tree for /tree-aware reasoning and LI addon workflows.",
    parameters: Type.Object({
      session_id: Type.Optional(Type.String({ description: "Optional session id scoping hint." })),
      max_nodes: Type.Optional(Type.Number({ description: "Optional node cap (default 200)." })),
    }),
    async execute(_id, params) {
      const { session_id, max_nodes } = params as { session_id?: string; max_nodes?: number };
      const query = session_id ? `?session_id=${encodeURIComponent(session_id)}` : "";
      const res = await focusaFetchDetailed(`/lineage/tree${query}`);
      if (!res.ok || !res.body) {
        return {
          content: [{ type: "text", text: `lineage tree → ${explainWorkLoopResult(res, "ok")}` }],
          details: { ok: false, status: res.status, response: res.body ?? null },
        } as any;
      }

      const cap = Math.max(1, Math.min(2000, Number(max_nodes || 200)));
      const nodes = Array.isArray(res.body?.nodes) ? res.body.nodes.slice(0, cap) : [];
      const head = String(res.body?.head || "");
      const root = String(res.body?.root || "");
      return {
        content: [{ type: "text", text: `lineage tree: nodes=${nodes.length} head=${head || "unknown"} root=${root || "unknown"}` }],
        details: {
          ok: true,
          status: res.status,
          root,
          head,
          total: Number(res.body?.total || nodes.length),
          nodes,
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_li_tree_extract",
    label: "LI Tree Extract",
    description: "Extract decision/constraint/risk signals and reflection trigger from lineage tree for metacognitive compounding.",
    parameters: Type.Object({
      max_candidates: Type.Optional(Type.Number({ description: "Max extracted signals per category (default 12)." })),
      session_id: Type.Optional(Type.String({ description: "Optional session id scoping hint." })),
    }),
    async execute(_id, params) {
      const { max_candidates, session_id } = params as { max_candidates?: number; session_id?: string };
      const query = session_id ? `?session_id=${encodeURIComponent(session_id)}` : "";
      const res = await focusaFetchDetailed(`/lineage/tree${query}`);
      if (!res.ok || !res.body) {
        return {
          content: [{ type: "text", text: `li extract → ${explainWorkLoopResult(res, "ok")}` }],
          details: { ok: false, status: res.status, response: res.body ?? null },
        } as any;
      }

      const cap = Math.max(1, Math.min(50, Number(max_candidates || 12)));
      const nodes = Array.isArray(res.body?.nodes) ? res.body.nodes : [];
      const byId = new Map<string, any>();
      nodes.forEach((n: any) => {
        const id = String(n?.node_id || "").trim();
        if (id) byId.set(id, n);
      });

      const extractSignals = (keys: string[]): string[] => {
        const out: string[] = [];
        for (const node of nodes) {
          const payload = node?.payload;
          if (!payload || typeof payload !== "object") continue;
          for (const key of keys) {
            const v = (payload as any)[key];
            if (Array.isArray(v)) {
              for (const item of v) {
                const s = String(item || "").trim();
                if (s) out.push(s);
              }
            } else {
              const s = String(v || "").trim();
              if (s) out.push(s);
            }
          }
        }
        return Array.from(new Set(out)).slice(0, cap);
      };

      const decisions = extractSignals(["decisions", "decision", "decision_text"]);
      const constraints = extractSignals(["constraints", "constraint", "constraint_text"]);
      const risks = extractSignals(["risks", "risk", "blockers", "blocker"]);

      const headId = String(res.body?.head || "").trim();
      let depth = 0;
      let cur = headId;
      const seen = new Set<string>();
      while (cur && !seen.has(cur)) {
        seen.add(cur);
        depth += 1;
        const node = byId.get(cur);
        cur = String(node?.parent_id || "").trim();
      }

      const summaryNodes = nodes.filter((n: any) => String(n?.node_type || "").toLowerCase() === "summary").length;
      const summaryRatio = nodes.length > 0 ? summaryNodes / nodes.length : 0;
      const reflectionTrigger = depth >= 24 || summaryRatio >= 0.35 || risks.length >= Math.max(3, Math.floor(cap / 3));

      return {
        content: [{ type: "text", text: `li extract: decisions=${decisions.length} constraints=${constraints.length} risks=${risks.length} depth=${depth} trigger=${reflectionTrigger ? "yes" : "no"}` }],
        details: {
          ok: true,
          status: res.status,
          lineage: {
            root: String(res.body?.root || ""),
            head: headId,
            nodes: nodes.length,
            depth,
            summary_nodes: summaryNodes,
            summary_ratio: summaryRatio,
          },
          signals: { decisions, constraints, risks },
          reflection_trigger: reflectionTrigger,
        },
      } as any;
    },
  });
}
