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
import { S, checkFocusa, focusaFetch, focusaPost, ensurePiFrame, getFocusState, ensureContinuityId, isProjectRootAuthoritySafe, projectRootAuthorityFailure, buildFocusaSessionIdentity, normalizeProjectRoot, resolvePiProjectRoot, confirmPiProjectRoot, projectRootConfirmationRequired, projectRootConfirmationSummary, stampWorkpointPacketForCurrentPiSession, persistState, estimateTokens } from "./state.js";
import { FOCUSA_TOOL_CONTRACTS, focusaToolContractSummary } from "./tool-contracts.js";

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

function stableJson(value: any): string {
  if (Array.isArray(value)) return `[${value.map(stableJson).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.keys(value).sort().map((key) => `${JSON.stringify(key)}:${stableJson(value[key])}`).join(",")}}`;
  }
  return JSON.stringify(value);
}

function deltaTargets(delta: { decisions?: string[]; constraints?: string[]; failures?: string[]; intent?: string; current_focus?: string; next_steps?: string[]; open_questions?: string[]; recent_results?: string[]; notes?: string[]; artifacts?: Array<{ kind: string; label: string; path_or_id?: string }> }): string[] {
  return Object.entries(delta)
    .filter(([, value]) => value !== undefined)
    .map(([key]) => key);
}

function mirrorFailedFocusWrite(kind: string, reason: PushDeltaFailureReason, payload: string, meta: Record<string, string | undefined>): { saved: boolean; turn: number } {
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
  // §AsccSections: decisions = crystallized choices that guide future action.
  // Keep the public validator aligned with pushDelta's canonical Focus State limit.
  if (decision.length > 160) {
    return { valid: false, reason: "Too verbose — distill to ONE crystallized sentence (max 160 chars). Use scratchpad for elaboration." };
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
  if (MULTI_SENTENCE.test(decision)) {
    return { valid: false, reason: "Multiple sentences — decisions should be ONE crystallized sentence. Per §AsccSections (<=160 chars)." };
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
  if (!operatorDirective && TASK_PATTERNS.test(constraint)) {
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

export type PushDeltaFailureReason = "offline" | "no_active_frame" | "frame_unavailable" | "scope_mismatch" | "read_model_lag" | "validation_rejected" | "write_failed";

export type PushDeltaResult =
  | { ok: true; duplicate_candidate?: boolean; idempotency_key?: string }
  | { ok: false; reason: PushDeltaFailureReason; api_reason?: string; duplicate_candidate?: boolean; idempotency_key?: string };

const RECENT_COGNITIVE_WRITE_KEYS: string[] = [];

function cognitiveWriteKey(delta: Record<string, any>, explicitKey?: string): string {
  if (explicitKey) return explicitKey;
  return JSON.stringify(delta).toLowerCase().replace(/\s+/g, " ").slice(0, 500);
}

function duplicateCandidateForWrite(key: string): boolean {
  const duplicate = RECENT_COGNITIVE_WRITE_KEYS.includes(key);
  RECENT_COGNITIVE_WRITE_KEYS.push(key);
  while (RECENT_COGNITIVE_WRITE_KEYS.length > 80) RECENT_COGNITIVE_WRITE_KEYS.shift();
  return duplicate;
}

type FocusaToolStatus = "accepted" | "completed" | "no_op" | "blocked" | "validation_rejected" | "degraded" | "offline" | "error";
type FocusaRetryPosture = "safe_retry" | "retry_with_idempotency_key" | "check_side_effects_first" | "do_not_retry_unchanged" | "operator_required";
type FocusaFailureClass =
  | "validation_rejected"
  | "not_found"
  | "frame_unavailable"
  | "daemon_unavailable"
  | "stale_runtime_registry"
  | "resource_exhausted"
  | "null_response"
  | "hot_path_timeout"
  | "cold_path_timeout"
  | "writer_conflict"
  | "scope_mismatch"
  | "scope_conflict"
  | "approval_required"
  | "permission_denied"
  | "process_control_failed"
  | "noncanonical_fallback"
  | "read_model_lag"
  | "unknown_ambiguous_completion";

interface FocusaToolResultV1 {
  ok: boolean;
  status: FocusaToolStatus;
  failure_class: FocusaFailureClass | null;
  canonical: boolean;
  degraded: boolean;
  summary: string;
  tool?: string;
  family?: string;
  endpoint?: string;
  workpoint_id?: string | null;
  retry: { safe: boolean; posture: FocusaRetryPosture; reason?: string };
  recovery_hint?: string;
  misuse_hint?: string;
  side_effects: string[];
  evidence_refs: string[];
  next_tools: string[];
  reflex_suggestions?: string[];
  ontology_candidate_delta_refs?: string[];
  error?: { field?: string; code?: string; message?: string; allowed_values?: string[] } | null;
  raw?: unknown;
}

function reflexSuggestionsForFailure(failureClass: FocusaFailureClass | null, status: FocusaToolStatus, nextTools: string[]): string[] {
  const suggestions = new Set<string>();
  switch (failureClass) {
    case "scope_conflict":
      suggestions.add("detect_semantic_project_scope_conflict");
      suggestions.add("bind_project_root");
      suggestions.add("confirm_continuity_scope");
      break;
    case "scope_mismatch":
      suggestions.add("diagnose_scope_mismatch");
      suggestions.add("detect_cross_project_packet");
      suggestions.add("confirm_continuity_scope");
      break;
    case "hot_path_timeout":
    case "cold_path_timeout":
    case "resource_exhausted":
    case "daemon_unavailable":
      suggestions.add("resource_mode_fallback");
      suggestions.add("degrade_with_recovery");
      break;
    case "read_model_lag":
      suggestions.add("retry_safe_pending");
      break;
    case "noncanonical_fallback":
    case "frame_unavailable":
    case "unknown_ambiguous_completion":
      suggestions.add("route_noncanonical_result");
      break;
    case "not_found":
      suggestions.add("resume_from_canonical_workpoint");
      break;
    case "approval_required":
    case "permission_denied":
      suggestions.add("require_destructive_confirmation");
      break;
    case "writer_conflict":
      suggestions.add("preflight_writer_ownership");
      break;
    case "validation_rejected":
      suggestions.add("guard_stale_focus_state");
      break;
  }
  if (status === "degraded" || status === "offline") suggestions.add("degrade_with_recovery");
  if (nextTools.includes("focusa_workpoint_resume")) suggestions.add("resume_from_canonical_workpoint");
  if (nextTools.includes("focusa_project_identity") || nextTools.includes("focusa_project_verify")) suggestions.add("bind_project_root");
  if (nextTools.includes("focusa_traverse")) suggestions.add("prefer_summary_hot_path");
  return Array.from(suggestions).slice(0, 4);
}

function inferFailureClass(status: FocusaToolStatus, summary: string, message?: string | null, canonical?: boolean, degraded?: boolean): FocusaFailureClass | null {
  const text = `${summary} ${message || ""}`.toLowerCase();
  if (text.includes("no active pi frame") || text.includes("no active frame") || text.includes("frame recovery")) return "frame_unavailable";
  if (text.includes("payload_equal=false") || text.includes("live registry payload differs") || text.includes("stale daemon registry") || text.includes("stale runtime registry")) return "stale_runtime_registry";
  if (text.includes("oom") || text.includes("out of memory") || text.includes("resource exhausted") || text.includes("killed process")) return "resource_exhausted";
  if (text.includes("null response") || text.includes("response=null") || text.includes("body=null")) return "null_response";
  if (status === "validation_rejected" || text.includes("validation_rejected") || text.includes("rejected")) return "validation_rejected";
  if (text.includes("not_found") || text.includes("not found") || text.includes("missing prediction") || text.includes("no such")) return "not_found";
  if (status === "offline" || text.includes("daemon unavailable") || text.includes("focusa offline") || text.includes("connection refused")) return "daemon_unavailable";
  if (text.includes("timeout") || text.includes("timed out") || text.includes("abort")) {
    return /(cold|deep|replay|worktree|diagnostic)/.test(text) ? "cold_path_timeout" : "hot_path_timeout";
  }
  if (text.includes("claimed by another writer") || text.includes("writer_conflict") || text.includes("controlled by another session")) return "writer_conflict";
  if (text.includes("scope_conflict") || text.includes("action_authority_for_current_ask=false") || text.includes("action authority for current ask") || text.includes("current ask project conflict")) return "scope_conflict";
  if (text.includes("project_root mismatch") || text.includes("scope mismatch") || text.includes("cross-project")) return "scope_mismatch";
  if (text.includes("approval required") || text.includes("requires approved")) return "approval_required";
  if (text.includes("permission denied") || text.includes("unauthorized") || text.includes("forbidden")) return "permission_denied";
  if (text.includes("read model lag") || text.includes("pending") || text.includes("not yet visible")) return "read_model_lag";
  if (degraded || canonical === false || text.includes("non-canonical") || text.includes("noncanonical")) return "noncanonical_fallback";
  if (status === "blocked" || status === "error") return "unknown_ambiguous_completion";
  return null;
}

function recoveryHintForFailure(failureClass: FocusaFailureClass | null, status: FocusaToolStatus, tool?: string): { recovery_hint?: string; misuse_hint?: string; next_tools?: string[] } {
  switch (failureClass) {
    case "scope_conflict":
      return { recovery_hint: "Treat the saved packet as canonical only for its saved scope; verify the current-ask project, then checkpoint/resume in the correct project before file/API action.", misuse_hint: "Usually caused by operator project correction, alias/path mismatch, or project-switch ledger evidence that predates API-level scope_mismatch.", next_tools: ["focusa_project_verify", "focusa_project_identity", "focusa_workpoint_checkpoint", "focusa_workpoint_resume"] };
    case "scope_mismatch":
      return { recovery_hint: "Use focusa_project_identity/verify with explicit project_root, then checkpoint/resume in the same continuity; do not retry stale packets unchanged.", misuse_hint: "Usually caused by broad cwd, cross-project packet reuse, or tool call before project binding.", next_tools: ["focusa_project_identity", "focusa_project_verify", "focusa_workpoint_checkpoint", "focusa_workpoint_resume"] };
    case "frame_unavailable":
      return { recovery_hint: "Stay attentive to operator direction, continue from repo/operator context, then create/resume a scoped Workpoint before durable Focus State writes.", misuse_hint: "Focus State note tools were used without an active Pi frame; this is recoverable, not a dead end.", next_tools: ["focusa_project_identity", "focusa_workpoint_checkpoint", "focusa_workpoint_resume", "focusa_tool_doctor"] };
    case "validation_rejected":
      return { recovery_hint: "Rewrite the durable slot as one compact declarative sentence, or put verbose/debug/task content in focusa_scratch.", misuse_hint: "Durable Focus State slots reject task lists, verbose reasoning, and non-declarative wording.", next_tools: ["focusa_scratch", "focusa_decide"] };
    case "not_found":
      return { recovery_hint: "Use the relevant recent/list/read tool or create the missing record before retrying the mutation.", misuse_hint: "Likely stale id, missing record, wrong project scope, or evaluating/linking before the source object exists.", next_tools: ["focusa_project_identity", "focusa_workpoint_resume", "focusa_predict_recent", "focusa_tool_doctor"] };
    case "read_model_lag":
      return { recovery_hint: "Wait briefly, then read/resume the current packet once with the same idempotency scope; avoid duplicate writes.", misuse_hint: "A recent accepted write may not be visible in the read model yet.", next_tools: ["focusa_workpoint_resume", "focusa_tool_doctor"] };
    case "hot_path_timeout":
      return { recovery_hint: "Retry the bounded hot route once, then run focusa_tool_doctor/resource_mode; avoid cold/full payload reads.", misuse_hint: "Hot routes should be bounded; repeated timeouts indicate daemon/resource pressure.", next_tools: ["focusa_tool_doctor", "focusa_resource_mode", "focusa_traverse"] };
    case "cold_path_timeout":
      return { recovery_hint: "Switch to summary/traverse slices or explicit rehydrate refs; schedule cold diagnostics separately.", misuse_hint: "A cold/deep route was used where a bounded route would answer the next action.", next_tools: ["focusa_traverse", "focusa_resource_mode", "focusa_tool_doctor"] };
    case "daemon_unavailable":
      return { recovery_hint: "Run focusa_tool_doctor; if daemon is down or overloaded, continue from operator/repo context and retry after health is ok.", misuse_hint: "Tool failure is infrastructure/reachability, not a reason to stop all useful repo work.", next_tools: ["focusa_tool_doctor", "focusa_resource_mode"] };
    case "writer_conflict":
      return { recovery_hint: "Use writer-status/preflight and avoid mutating work-loop ownership without explicit operator approval.", misuse_hint: "A mutating work-loop command was attempted while another writer/session owns the loop.", next_tools: ["focusa_work_loop_writer_status", "focusa_work_loop_status"] };
    case "approval_required":
      return { recovery_hint: "Do not infer approval; use preflight/read-only path or wait for explicit approved=true/force=true where required.", misuse_hint: "A mutating/destructive/background-session action was attempted without required approval fields.", next_tools: ["focusa_tool_doctor"] };
    case "process_control_failed":
      return { recovery_hint: "List/health/tail the SilentSession, verify tmux run_as_user/root_dir metadata, then retry only after process state is clear.", misuse_hint: "Likely tmux session missing, wrong run_as_user, dead pane, or process-control race.", next_tools: ["focusa_silent_sessions", "focusa_tool_doctor"] };
    case "unknown_ambiguous_completion":
      return { recovery_hint: "Check side effects or canonical read state first, then retry only if no duplicate/cross-scope mutation risk exists.", misuse_hint: "Result did not prove success or failure; blind retry can duplicate writes.", next_tools: ["focusa_tool_doctor", "focusa_workpoint_resume"] };
    default:
      if (status === "blocked" || status === "error") return { recovery_hint: "Read failure_class, retry.posture, and next_tools; prefer project_identity → trajectory_view → workpoint_resume/checkpoint before retrying.", misuse_hint: "Likely out-of-order tool use or missing project/continuity context.", next_tools: ["focusa_project_identity", "focusa_trajectory_view", "focusa_workpoint_resume", "focusa_tool_doctor"] };
      return {};
  }
}

function focusaToolResult(params: {
  ok: boolean;
  status: FocusaToolStatus;
  summary: string;
  failure_class?: FocusaFailureClass | null;
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
  ontology_candidate_delta_refs?: string[];
  error?: FocusaToolResultV1["error"];
  raw?: unknown;
}): FocusaToolResultV1 {
  const degraded = params.degraded ?? (params.status === "degraded" || params.status === "offline");
  const canonical = params.canonical ?? (!degraded && params.ok);
  const summary = params.summary.slice(0, 240);
  const failureClass = params.failure_class ?? inferFailureClass(params.status, summary, params.error?.message, canonical, degraded);
  const guidance = recoveryHintForFailure(failureClass, params.status, params.tool);
  const nextTools = (params.next_tools?.length ? params.next_tools : guidance.next_tools ?? []).slice(0, 4);
  const reflexSuggestions = reflexSuggestionsForFailure(failureClass, params.status, nextTools);
  return {
    ok: params.ok,
    status: params.status,
    failure_class: failureClass,
    canonical,
    degraded,
    summary,
    tool: params.tool,
    family: params.family,
    endpoint: params.endpoint,
    workpoint_id: params.workpoint_id ?? null,
    retry: {
      safe: params.retry?.safe ?? (params.status === "completed" || params.status === "no_op"),
      posture: params.retry?.posture ?? (params.ok ? "safe_retry" : "operator_required"),
      reason: params.retry?.reason ?? failureClass ?? undefined,
    },
    recovery_hint: compactHint(guidance.recovery_hint),
    misuse_hint: compactHint(guidance.misuse_hint),
    side_effects: params.side_effects ?? [],
    evidence_refs: params.evidence_refs ?? [],
    next_tools: nextTools,
    reflex_suggestions: reflexSuggestions,
    ontology_candidate_delta_refs: params.ontology_candidate_delta_refs ?? [],
    error: params.error ?? null,
    raw: compactApiEcho(params.raw),
  };
}

function compactHint(value?: string): string | undefined {
  if (!value) return undefined;
  return value.replace(/\s+/g, " ").trim().slice(0, 140);
}

function compactText(value: unknown, fallback = "unknown", max = 140): string {
  const text = String(value ?? "").replace(/\s+/g, " ").trim();
  return (text || fallback).slice(0, max);
}

function compactApiEcho(value: unknown): unknown {
  if (!value || typeof value !== "object") return value;
  const input = value as Record<string, any>;
  const keys = ["status", "canonical", "degraded", "failure_class", "error", "why", "next_step_hint", "workpoint_id", "packet_id", "trajectory_id", "endpoint", "route_tier"];
  const out: Record<string, unknown> = {};
  for (const key of keys) if (input[key] !== undefined) out[key] = typeof input[key] === "string" ? input[key].slice(0, 240) : input[key];
  if (Array.isArray(input.next_tools)) out.next_tools = input.next_tools.slice(0, 4);
  return Object.keys(out).length ? out : { omitted: true, reason: "compact_api_echo" };
}

function focusaToolDetails(details: Record<string, unknown>, result: FocusaToolResultV1): Record<string, unknown> {
  return { ...details, tool_result_v1: result };
}

function focusaEvidenceCaptureSuggestion(input: { target_ref: string; result: string; evidence_ref: string; project_root?: string | null; attach_to_workpoint?: boolean }): Record<string, unknown> {
  return {
    tool: "focusa_evidence_capture",
    payload: {
      target_ref: input.target_ref,
      result: input.result,
      evidence_ref: input.evidence_ref,
      project_root: input.project_root || undefined,
      attach_to_workpoint: input.attach_to_workpoint ?? true,
    },
  };
}

function blockedToolResponse(tool: string, family: string, summary: string, failureClass: FocusaFailureClass, raw?: unknown, nextTools?: string[]): any {
  const toolResult = focusaToolResult({
    ok: false,
    status: "blocked",
    failure_class: failureClass,
    canonical: false,
    degraded: true,
    summary,
    tool,
    family,
    retry: { safe: failureClass !== "validation_rejected", posture: failureClass === "validation_rejected" ? "do_not_retry_unchanged" : "safe_retry", reason: failureClass },
    side_effects: [],
    evidence_refs: [],
    next_tools: nextTools,
    raw: raw,
  });
  return {
    content: [{ type: "text", text: terseToolText(summary, failureClass, toolResult.next_tools) }],
    details: {
      ok: false,
      status: "blocked",
      failure_class: failureClass,
      recovery_hint: toolResult.recovery_hint,
      misuse_hint: toolResult.misuse_hint,
      next_tools: toolResult.next_tools,
      reflex_suggestions: toolResult.reflex_suggestions,
      tool_result_v1: toolResult,
      response: compactApiEcho(raw),
    },
  } as any;
}

function terseToolText(summary: string, failureClass: string | null, nextTools: string[] = []): string {
  const next = nextTools.slice(0, 3).join(" → ") || "focusa_tool_doctor";
  return `${summary}; class=${failureClass || "none"}; next=${next}`.slice(0, 220);
}

function timeoutPreservedText(surface: string, noun = "fallback"): string {
  return `${surface} preserved cached advisory ${noun}; cause=timeout; next=resource_mode/doctor/retry`.slice(0, 160);
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

function ontologyCandidateDeltaRefs(tool: string, result: any, status: FocusaToolStatus): string[] {
  const details = (result?.details || {}) as Record<string, any>;
  const refs = new Set<string>();
  const add = (kind: string, value: unknown) => {
    const text = String(value || "").trim();
    if (text) refs.add(`${kind}:${text}`.slice(0, 220));
  };
  add("tool", tool);
  add("status", status);
  for (const key of ["target_ref", "targetRef", "file", "path", "endpoint", "workpoint_id"]) add("target", details[key]);
  for (const ref of Array.isArray(details.evidence_refs) ? details.evidence_refs : []) add("evidence", ref);
  const text = String(result?.content?.[0]?.text || details.summary || "");
  const handle = text.match(/\[HANDLE:([^\]]+)\]/)?.[1];
  add("evidence", handle);
  return Array.from(refs).slice(0, 12);
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
  const detailsFailureClass = typeof details.failure_class === "string"
    ? details.failure_class as FocusaFailureClass
    : typeof (details.response as any)?.failure_class === "string"
      ? (details.response as any).failure_class as FocusaFailureClass
      : undefined;
  const activeWorkpoint = resolveActiveWorkpointContext();
  const resultWorkpointId = String(details.response?.workpoint_id || details.response?.active_workpoint_id || details.workpoint_id || activeWorkpoint.workpoint_id || "") || null;
  return focusaToolResult({
    ok,
    status,
    failure_class: detailsFailureClass,
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
    next_tools: Array.isArray(details.next_tools) && details.next_tools.length
      ? details.next_tools.map(String)
      : status === "offline" ? ["focusa_tool_doctor", "focusa_resource_mode"] : family === "workpoint" ? ["focusa_workpoint_resume"] : [],
    ontology_candidate_delta_refs: ontologyCandidateDeltaRefs(tool, result, status),
    error: validationRejected || blocked || offline ? { code: status, message: text.slice(0, 240) } : null,
    raw: details.response ?? details,
  });
}

function defaultFocusaPromptSnippet(name: string, description?: string): string {
  if (name.startsWith("focusa_workpoint_")) return "Use after project folder is verified; pass explicit project_root/continuity_id after compaction or unsafe cwd.";
  if (name.startsWith("focusa_trajectory_")) return "Advisory project-goal tool; verify project_root first and do not treat proposals as execution authority.";
  if (name.startsWith("focusa_work_loop_")) return "Check writer/status first; preflight pause/resume/stop unless operator explicitly authorized mutation.";
  if (name.startsWith("focusa_metacog_")) return "Use for reusable learning signals; store concise evidence-backed lessons, not raw transcript blobs.";
  if (name.startsWith("focusa_tree_") || name === "focusa_lineage_tree" || name === "focusa_li_tree_extract") return "Use bounded lineage/snapshot helpers instead of inferring branch/history from transcript memory.";
  if (name.startsWith("focusa_predict_")) return "Record/evaluate bounded predictions; predictions guide actions but never override operator steering.";
  if (name.includes("hygiene")) return "Diagnose first; apply hygiene only with explicit approved=true and never silently delete state.";
  if (name === "focusa_traverse") return "Use bounded traversal/search with explicit limits; opt into large payloads only when needed.";
  if (["focusa_intent", "focusa_current_focus", "focusa_next_step", "focusa_open_question", "focusa_recent_result", "focusa_note"].includes(name)) return "Write concise Focus State slot updates; use focusa_scratch for working notes and verbose reasoning.";
  return String(description || "Use this Focusa tool with explicit project_root when session/cwd is ambiguous.").slice(0, 240);
}

function sleepMs(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function paramsWithAutoIdempotency(toolName: string, params: unknown, id: string): unknown {
  if (toolName !== "focusa_workpoint_checkpoint" || !params || typeof params !== "object" || Array.isArray(params)) return params;
  const record = params as Record<string, any>;
  if (record.idempotency_key) return params;
  const continuity = String(record.continuity_id || S.continuityId || "session").replace(/[^A-Za-z0-9._:-]/g, "_").slice(0, 80);
  return { ...record, idempotency_key: `pi-tool-${toolName}-${continuity}-${id}`.slice(0, 160) };
}

function shouldAutoRetryWorkpoint(toolName: string, result: any, toolResult: FocusaToolResultV1): boolean {
  if (!["focusa_workpoint_checkpoint", "focusa_workpoint_resume"].includes(toolName)) return false;
  const response = (result?.details as any)?.response || {};
  const text = String(result?.content?.[0]?.text || "").toLowerCase();
  return toolResult.failure_class === "read_model_lag"
    || response.status === "pending"
    || response.failure_class === "read_model_lag"
    || response.failure_class === "resource_exhausted"
    || /pending|read-model lag|not yet visible/.test(text);
}

function annotateAutoRetry(result: any, attempts: number): any {
  const details = { ...(result?.details || {}) };
  details.auto_retry = { attempts, policy: "bounded_workpoint_pending_retry" };
  return { ...result, details };
}

function capToolText(text: unknown, max = 700): string {
  const normalized = String(text ?? "").replace(/\s+\n/g, "\n").trim();
  return normalized.length <= max ? normalized : `${normalized.slice(0, Math.max(0, max - 1))}…`;
}

function formatOperatorDateTime(ms: number): string {
  return new Date(ms).toLocaleString("en-US", { timeZone: "America/Los_Angeles", year: "2-digit", month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit", second: "2-digit", hour12: true });
}

function formatElapsedHms(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = total % 60;
  return `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

function currentTaskTimingAndTokens() {
  const now = Date.now();
  const elapsedMs = Math.max(0, now - (S.currentTaskStartTime || S.sessionStartTime || now));
  const providerTotal = S.currentTaskProviderInputTokens + S.currentTaskProviderOutputTokens;
  const estimatedTotal = S.currentTaskInputTokenEstimate + S.currentTaskOutputTokenEstimate + estimateTokens(JSON.stringify(S.toolUsageBatch || []));
  const totalTokens = providerTotal > 0 ? providerTotal : estimatedTotal;
  return {
    task_timing: {
      started_at: new Date(S.currentTaskStartTime || S.sessionStartTime || now).toISOString(),
      started_at_operator: formatOperatorDateTime(S.currentTaskStartTime || S.sessionStartTime || now),
      completed_at: new Date(now).toISOString(),
      completed_at_operator: formatOperatorDateTime(now),
      elapsed_ms: elapsedMs,
      elapsed_seconds: Math.floor(elapsedMs / 1000),
      elapsed_hms: formatElapsedHms(elapsedMs),
      turn_start: S.currentTaskTurnStart,
      turn_end: S.turnCount,
      turn_count: Math.max(0, S.turnCount - (S.currentTaskTurnStart || S.turnCount) + 1),
      task_label: S.currentTaskLabel || S.currentAsk?.text || "",
    },
    token_usage: {
      provider_input_tokens: S.currentTaskProviderInputTokens,
      provider_output_tokens: S.currentTaskProviderOutputTokens,
      provider_total_tokens: providerTotal,
      estimated_input_tokens: S.currentTaskInputTokenEstimate,
      estimated_output_tokens: S.currentTaskOutputTokenEstimate,
      estimated_total_tokens: estimatedTotal,
      total_tokens: totalTokens,
      counting_method: providerTotal > 0 ? "provider_usage_when_available" : "estimate_chars_div_4_fallback",
      tool_calls: S.currentTaskToolCalls,
    },
  };
}

function capToolOutputText(result: any): any {
  if (!Array.isArray(result?.content)) return result;
  return {
    ...result,
    content: result.content.map((entry: any) => entry?.type === "text" ? { ...entry, text: capToolText(entry.text) } : entry),
  };
}

function withToolResultEnvelope(tool: any): any {
  if (!tool?.name?.startsWith?.("focusa_") || typeof tool.execute !== "function") return tool;
  const execute = tool.execute;
  return {
    ...tool,
    promptSnippet: tool.promptSnippet || defaultFocusaPromptSnippet(tool.name, tool.description),
    async execute(id: string, params: unknown) {
      S.currentTaskToolCalls += 1;
      const executionParams = paramsWithAutoIdempotency(tool.name, params, id);
      let result = await execute(id, executionParams);
      let details = (result?.details || {}) as Record<string, unknown>;
      let toolResult = inferToolResult(tool.name, result);
      if (shouldAutoRetryWorkpoint(tool.name, result, toolResult)) {
        await sleepMs(250);
        result = await execute(`${id}-retry1`, executionParams);
        result = annotateAutoRetry(result, 1);
        details = (result?.details || {}) as Record<string, unknown>;
        toolResult = inferToolResult(tool.name, result);
      }
      return capToolOutputText({ ...result, details: focusaToolDetails(details, toolResult) });
    },
  };
}

function formatPushDeltaFailure(reason: PushDeltaFailureReason): string {
  switch (reason) {
    case "offline":
      return "Focusa offline";
    case "no_active_frame":
    case "frame_unavailable":
      return "Attentive and awaiting operator direction";
    case "scope_mismatch":
      return "Focus State scope mismatch";
    case "read_model_lag":
      return "Focusa read-model lag";
    case "validation_rejected":
      return "Focus State validation rejected the write";
    case "write_failed":
    default:
      return "Focusa write failed";
  }
}

function pushDeltaFailureRecovery(reason: PushDeltaFailureReason, apiReason?: string): {
  failure_class: FocusaFailureClass;
  retry_posture: FocusaRetryPosture;
  recovery_hint: string;
  next_tools: string[];
  api_reason?: string;
} {
  switch (reason) {
    case "offline":
      return { failure_class: "daemon_unavailable", retry_posture: "safe_retry", recovery_hint: "Run focusa_tool_doctor; if resource mode is emergency, use focusa_resource_mode before retrying.", next_tools: ["focusa_tool_doctor", "focusa_resource_mode"], api_reason: apiReason };
    case "no_active_frame":
    case "frame_unavailable":
      return { failure_class: "frame_unavailable", retry_posture: "safe_retry", recovery_hint: "Verify the project folder, checkpoint/resume a Workpoint, then retry the Focus State write from a reloaded Pi session.", next_tools: ["focusa_project_identity", "focusa_workpoint_checkpoint", "focusa_workpoint_resume", "focusa_tool_doctor"], api_reason: apiReason };
    case "scope_mismatch":
      return { failure_class: "scope_mismatch", retry_posture: "do_not_retry_unchanged", recovery_hint: "Refresh project_root+continuity_id via focusa_project_verify and focusa_workpoint_resume; do not retry with stale project context.", next_tools: ["focusa_project_verify", "focusa_workpoint_resume", "focusa_workpoint_checkpoint", "focusa_tool_doctor"], api_reason: apiReason };
    case "read_model_lag":
      return { failure_class: "read_model_lag", retry_posture: "safe_retry", recovery_hint: "Read model may lag a just-created frame or Workpoint; resume/check current packet before retrying once.", next_tools: ["focusa_workpoint_resume", "focusa_tool_doctor"], api_reason: apiReason };
    case "validation_rejected":
      return { failure_class: "validation_rejected", retry_posture: "do_not_retry_unchanged", recovery_hint: "Rewrite concise canonical wording or store full reasoning in focusa_scratch.", next_tools: ["focusa_scratch"], api_reason: apiReason };
    case "write_failed":
    default:
      return { failure_class: "unknown_ambiguous_completion", retry_posture: "check_side_effects_first", recovery_hint: "Run focusa_tool_doctor and inspect response details before retrying to avoid duplicate or cross-scope writes.", next_tools: ["focusa_tool_doctor", "focusa_scratch"], api_reason: apiReason };
  }
}

function formatNonCriticalWriteFailure(slotLabel: string, reason: PushDeltaFailureReason, apiReason?: string): string {
  const base = formatPushDeltaFailure(reason);
  const detail = apiReason ? ` Detail: ${apiReason}` : "";
  const recovery = pushDeltaFailureRecovery(reason, apiReason);
  if (reason === "no_active_frame" || reason === "frame_unavailable") return `⚠️ ${base} — ${slotLabel} NOT recorded. Frame recovery was attempted; scratchpad fallback is safest until a project-bound frame exists.${detail} Next: ${recovery.recovery_hint}`;
  if (reason === "scope_mismatch" || reason === "read_model_lag") return `⚠️ ${base} — ${slotLabel} NOT recorded. Project-bound frame/continuity is stale; use latest operator instruction, checkpoint a fresh Workpoint, and do not retry unchanged.${detail} Next: ${recovery.recovery_hint}`;
  if (reason === "offline") return `⚠️ ${base} — ${slotLabel} NOT recorded.${detail} Next: ${recovery.recovery_hint}`;
  if (reason === "validation_rejected") return `⚠️ ${base} — ${slotLabel} NOT recorded.${detail} Next: ${recovery.recovery_hint}`;
  return `⚠️ ${base} — ${slotLabel} NOT recorded.${detail} Next: ${recovery.recovery_hint}`;
}

function namedSlotFallback(slotLabel: string, kind: string, reason: PushDeltaFailureReason, payload: string, apiReason?: string): { text: string; saved: boolean; turn: number; recovery: ReturnType<typeof pushDeltaFailureRecovery> } {
  const fallback = mirrorFailedFocusWrite(kind, reason, payload, { api_reason: apiReason });
  const recovery = pushDeltaFailureRecovery(reason, apiReason);
  const fallbackText = fallback.saved ? ` Saved to scratchpad fallback (turn ${fallback.turn}).` : " Scratchpad fallback also failed.";
  return { text: `${formatNonCriticalWriteFailure(slotLabel, reason, apiReason)}${fallbackText}`, saved: fallback.saved, turn: fallback.turn, recovery };
}

function conciseObjectiveSuggestion(payload: string): string {
  const text = String(payload || "")
    .replace(/^\s*status\s*:\s*/i, "")
    .replace(/\b(lowmem focusa active|focusa active|builds? only via [^.;]+|deploy wrapper)\b/ig, "")
    .replace(/\b(next action|blocker)\s*:[^.;]+/ig, "")
    .replace(/[`*_#>\[\]]/g, " ")
    .replace(/\s+/g, " ")
    .trim();
  const clause = text
    .split(/[.;]\s+/)
    .map(part => part.trim())
    .filter(Boolean)
    .find(part => !/\b(script|wrapper|status|blocker|next action)\b/i.test(part)) || text;
  return (clause || "Continue current operator-directed objective").slice(0, 120);
}

function namedSlotValidationFallback(slotLabel: string, kind: string, payload: string, reason?: string): { text: string; saved: boolean; turn: number; suggestion?: string } {
  const fallback = mirrorFailedFocusWrite(kind, "validation_rejected", payload, { validator_reason: reason });
  const fallbackText = fallback.saved ? ` Original saved to scratchpad fallback (turn ${fallback.turn}).` : " Scratchpad fallback also failed.";
  const suggestion = kind === "current_focus" ? conciseObjectiveSuggestion(payload) : undefined;
  const suggestionText = suggestion ? ` Suggested current_focus: "${suggestion}".` : "";
  return {
    text: `${reason || "Rejected by Focus State slot validator."}${fallbackText}${suggestionText}`,
    saved: fallback.saved,
    turn: fallback.turn,
    suggestion,
  };
}

function projectIdentityVerifiedInPayload(body: any): boolean {
  const project = body?.project_identity || body?.resume_packet?.project_identity || {};
  const api = project?.project_identity_api || body?.project_identity_api || {};
  return project?.status === "verified" || project?.quorum_status === "verified" || api?.status === "verified" || body?.verification?.verified === true;
}

function focusaToolWorkpointScope(packet: any): { projectRoot: string; continuityId: string } | null {
  if (!packet || typeof packet !== "object") return null;
  const workpoint = packet.resume_packet?.workpoint || packet.workpoint || packet;
  const projectRoot = normalizeProjectRoot(workpoint?.project_root || packet.project_root);
  const continuityId = String(workpoint?.continuity_id || packet.continuity_id || "").trim();
  if (!projectRoot || !continuityId || !isProjectRootAuthoritySafe(projectRoot)) return null;
  if (packet.canonical === false || workpoint?.canonical === false || packet.status === "partial" || packet.status === "rejected_scope_mismatch") return null;
  return { projectRoot, continuityId };
}

async function resolveFocusaToolProjectRoot(explicitProjectRoot?: unknown): Promise<string> {
  const explicit = normalizeProjectRoot(explicitProjectRoot);
  if (explicit) return explicit;
  const sessionRoot = resolvePiProjectRoot(S.sessionCwd || process.cwd());
  if (isProjectRootAuthoritySafe(sessionRoot)) return sessionRoot;

  const localScope = focusaToolWorkpointScope(S.activeWorkpointPacket);
  if (localScope) {
    if (!S.continuityId) S.continuityId = localScope.continuityId;
    return localScope.projectRoot;
  }

  return sessionRoot || normalizeProjectRoot(process.cwd()) || String(process.cwd());
}

function projectRootConfirmationGate(projectRoot: string, explicitProjectRoot?: unknown): any | null {
  if (explicitProjectRoot || !projectRootConfirmationRequired(projectRoot)) return null;
  const resolution = S.lastProjectRootResolution;
  const candidates = resolution?.candidates || [];
  return {
    content: [{ type: "text", text: `project root confirmation required → ${projectRootConfirmationSummary(projectRoot)}. Use interview/menu to confirm the correct project_root before Focusa state writes.` }],
    details: {
      ok: false,
      status: "blocked",
      failure_class: "scope_mismatch",
      reason: "project_root_confidence_below_90",
      project_root: projectRoot,
      project_root_resolution: resolution,
      candidates,
      next_tools: ["interview", "focusa_project_identity", "focusa_workpoint_checkpoint"],
    },
  } as any;
}

function scopeRecoveryContext(body: any, projectRoot: string, continuityId?: string, source = "focusa"): { text: string; details: Record<string, any> } | null {
  const status = String(body?.status || "");
  const canonical = body?.canonical === true;
  const project = body?.project_identity || body?.resume_packet?.project_identity || {};
  const trajectory = body?.trajectory || body?.resume_packet?.trajectory || {};
  const projectStatus = String(project?.status || project?.project_identity_api?.status || "unknown");
  const definitionStatus = String(trajectory?.definition_status || trajectory?.definition || "unknown");
  const failureClass = String(body?.failure_class || body?.details?.tool_result_v1?.failure_class || "");
  const needsRecovery = status === "degraded" || status === "not_found" || canonical === false || projectStatus === "mismatch" || definitionStatus === "conflicted" || failureClass === "scope_mismatch";
  if (!needsRecovery) return null;
  const safeRoot = isProjectRootAuthoritySafe(projectRoot);
  const verifiedProject = projectIdentityVerifiedInPayload(body) || safeRoot;
  const cont = continuityId ? ` continuity_id=${continuityId}` : "";
  return {
    text: `scope recovery → ${verifiedProject ? "verified project, but " : ""}no canonical Focusa packet for project_root=${projectRoot}${cont}; operator steering is authority; create focusa_workpoint_checkpoint for this mission, then retry resume. Store verbose/build/process rules in focusa_scratch, not current_focus.`,
    details: {
      source,
      project_root: projectRoot,
      continuity_id: continuityId || null,
      status,
      canonical,
      project_status: projectStatus,
      definition_status: definitionStatus,
      failure_class: failureClass || null,
      operator_steering_is_authority: true,
      safe_next_tools: ["focusa_workpoint_checkpoint", "focusa_scratch", "focusa_project_verify", "focusa_workpoint_resume"],
    },
  };
}

function allowsWorkpointBootstrapFromClarity(body: any, projectRoot: string, actionLabel: string): boolean {
  if (actionLabel !== "workpoint checkpoint") return false;
  if (!isProjectRootAuthoritySafe(projectRoot)) return false;
  return projectIdentityVerifiedInPayload(body);
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
    emitWriteTelemetry("focusa_write_recovery_attempt", { targets, reason: "no_active_frame", strategy: "refresh_scoped_frame" });
    const refreshed = await getFocusState().catch(() => null);
    if (refreshed?.frame?.id) {
      recoveredFrame = true;
      emitWriteTelemetry("focusa_write_recovery_result", { targets, reason: "no_active_frame", recovered: true, strategy: "refresh_scoped_frame", frame_id: refreshed.frame.id });
    }
  }

  if (!S.activeFrameId) {
    emitWriteTelemetry("focusa_write_recovery_attempt", { targets, reason: "no_active_frame", strategy: "create_or_adopt_scoped_frame" });
    const frameId = await ensurePiFrame(undefined, undefined, "pi-auto-recover");
    recoveredFrame = recoveredFrame || !!frameId;
    emitWriteTelemetry("focusa_write_recovery_result", { targets, reason: "no_active_frame", recovered: !!frameId, strategy: "create_or_adopt_scoped_frame" });
    if (!frameId) {
      emitWriteTelemetry("focusa_write_failed", { targets, reason: "no_active_frame" });
      return { ok: false, reason: "no_active_frame" };
    }
  }

  try {
    // Refresh frame identity before writes; stale paused Pi frames are a common
    // source of reducer rejections and scratchpad fallbacks after rescope/compact.
    await getFocusState().catch(() => null);
    const projectRoot = normalizeProjectRoot(S.sessionCwd || resolvePiProjectRoot(process.cwd()));
    const continuityId = S.continuityId || ensureContinuityId(projectRoot);
    if (!isProjectRootAuthoritySafe(projectRoot) || !continuityId) {
      emitWriteTelemetry("focusa_write_failed", { targets, reason: "scope_mismatch", project_root: projectRoot || null, continuity_id: continuityId || null });
      return { ok: false, reason: "scope_mismatch", api_reason: "focus_update_requires_safe_project_root_and_continuity_id" };
    }
    const postUpdate = () => focusaFetch("/focus/update", {
      method: "POST",
      body: JSON.stringify({
        frame_id: S.activeFrameId,
        project_root: projectRoot,
        continuity_id: continuityId,
        turn_id: `pi-turn-${S.turnCount}`,
        delta,
      }),
    });
    let response = await postUpdate();
    if (["no_active_frame", "frame_unavailable", "rejected_scope_mismatch"].includes(String(response?.status || ""))) {
      emitWriteTelemetry("focusa_write_recovery_attempt", { targets, reason: response?.status === "rejected_scope_mismatch" ? "scope_mismatch" : "stale_frame", stale_frame_id: S.activeFrameId, active_frame_id: response?.active_frame_id, target_frame_id: response?.target_frame_id, failure_class: response?.failure_class });
      S.activeFrameId = null;
      const frameId = await ensurePiFrame(undefined, undefined, "pi-stale-frame-recover");
      recoveredFrame = recoveredFrame || !!frameId;
      emitWriteTelemetry("focusa_write_recovery_result", { targets, reason: "stale_frame", recovered: !!frameId });
      if (frameId) response = await postUpdate();
    }
    if (!response || response.status === "write_failed") {
      emitWriteTelemetry("focusa_write_failed", { targets, reason: "write_failed", recovered_frame: recoveredFrame, api_reason: response?.reason });
      return { ok: false, reason: "write_failed", api_reason: response?.reason };
    }
    if (response.status === "no_active_frame" || response.status === "frame_unavailable") {
      emitWriteTelemetry("focusa_write_failed", { targets, reason: "frame_unavailable", recovered_frame: recoveredFrame, api_reason: response.reason, active_frame_id: response.active_frame_id, target_frame_id: response.target_frame_id });
      return { ok: false, reason: "frame_unavailable", api_reason: response.reason };
    }
    if (response.status === "rejected_scope_mismatch") {
      emitWriteTelemetry("focusa_write_failed", { targets, reason: "scope_mismatch", recovered_frame: recoveredFrame, api_reason: response.reason, active_frame_id: response.active_frame_id, target_frame_id: response.target_frame_id, diagnostic_class: response.diagnostic_class });
      return { ok: false, reason: "scope_mismatch", api_reason: response.reason };
    }
    if (response.status === "rejected") {
      emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected", recovered_frame: recoveredFrame, api_reason: response.reason });
      return { ok: false, reason: "validation_rejected", api_reason: response.reason };
    }
    if (response.status !== "accepted") {
      emitWriteTelemetry("focusa_write_failed", { targets, reason: "write_failed", recovered_frame: recoveredFrame, status: response.status || "unknown", api_reason: response.reason });
      return { ok: false, reason: "write_failed", api_reason: response.reason || response.status || "unknown" };
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

function persistedProjectIdentityFields(): Record<string, string> {
  const identity = S.lastProjectIdentity || {};
  const fields: Record<string, string> = {};
  const root = normalizeProjectRoot(identity.project_root);
  if (root) fields.persisted_project_root = root;
  if (identity.fingerprint) fields.persisted_project_fingerprint = String(identity.fingerprint);
  if (identity.project_id) fields.persisted_project_id = String(identity.project_id);
  if (identity.canonical_name) fields.persisted_canonical_name = String(identity.canonical_name);
  return fields;
}

function appendPersistedProjectIdentityQuery(query: URLSearchParams, explicitProjectRoot?: string): void {
  const persisted = persistedProjectIdentityFields();
  const persistedRoot = normalizeProjectRoot(persisted.persisted_project_root);
  const requestedRoot = normalizeProjectRoot(explicitProjectRoot);
  if (requestedRoot && persistedRoot && requestedRoot !== persistedRoot) return;
  for (const [key, value] of Object.entries(persisted)) {
    if (value) query.set(key, value);
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
  //   - Max 160 chars (canonical §AsccSections limit)
  //
  // Use focusa_scratch for all working notes first. Then distill ONE decision.
  pi.registerTool({
    name: "focusa_decide",
    label: "Record Decision",
    description: "Record a crystallized architectural decision in Focus State. Use focusa_scratch for working notes first. Decisions are ONE sentence (<=160 chars) — architectural choices only, not task lists.",
    promptSnippet: "Crystallized decision → Focus State. Working notes → focusa_scratch first.",
    parameters: Type.Object({
      decision: Type.String({ description: "ONE crystallized architectural choice — what was decided and why (max 160 chars). NOT a task list or debugging note." }),
      rationale: Type.Optional(Type.String({ description: "Context: why this decision was made (max 200 chars). Summarize from scratchpad notes." })),
    }),
    promptGuidelines: [
      "Step 1: Write detailed reasoning in focusa_scratch",
      "Step 2: Distill ONE crystallized sentence → decision field",
      "decision = what was decided (architectural choice, not implementation plan)",
      "rationale = why (1-2 sentences max)",
      "VALIDATION FAILS if: task patterns (Fix/Add/Check), debug patterns (error/failed), self-reference (I think/I tried), multiple sentences, or > 160 chars",
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
        const recovery = pushDeltaFailureRecovery(result.reason, result.api_reason);
        const fallbackText = fallback.saved ? `Saved to scratchpad automatically (turn ${fallback.turn}).` : "Scratchpad fallback also failed.";
        return {
          content: [{ type: "text" as const, text: `⚠️ ${formatPushDeltaFailure(result.reason)} — decision NOT recorded in Focus State. ${fallbackText} Next: ${recovery.recovery_hint}` }],
          details: { valid: false, reason: result.reason, decision, rationale: rationale?.slice(0, 200), scratch_saved: fallback.saved, scratch_turn: fallback.turn, ...recovery },
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
      "Phrase as declarative architecture boundaries; avoid task/negation wording like 'Need to...' or 'Do not...'",
      "If validation rejects, write the full note to focusa_scratch and retry once with noun-phrase boundary wording.",
      "Example VALID: 'Focus State cannot be cleared — /focus/update only accumulates. Stale entries require fresh frame push.'",
      "Example VALID: 'Workpoint identity uses project_root plus continuity_id; Pi session_id is temporal metadata.'",
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
        const recovery = pushDeltaFailureRecovery(result.reason, result.api_reason);
        const fallbackText = fallback.saved ? `Saved to scratchpad automatically (turn ${fallback.turn}).` : "Scratchpad fallback also failed.";
        return {
          content: [{ type: "text" as const, text: `⚠️ ${formatPushDeltaFailure(result.reason)} — constraint NOT recorded in Focus State. ${fallbackText} Next: ${recovery.recovery_hint}` }],
          details: { valid: false, reason: result.reason, constraint, source, scratch_saved: fallback.saved, scratch_turn: fallback.turn, ...recovery },
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
        const recoveryPlan = pushDeltaFailureRecovery(result.reason, result.api_reason);
        const fallbackText = fallback.saved ? `Saved to scratchpad automatically (turn ${fallback.turn}).` : "Scratchpad fallback also failed.";
        return {
          content: [{ type: "text" as const, text: `⚠️ ${formatPushDeltaFailure(result.reason)} — failure NOT recorded in Focus State. ${fallbackText} Next: ${recoveryPlan.recovery_hint}` }],
          details: { valid: false, reason: result.reason, failure, recovery, scratch_saved: fallback.saved, scratch_turn: fallback.turn, ...recoveryPlan },
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
      if (!v.valid) {
        const fallback = namedSlotValidationFallback("intent", "intent", intent.trim(), v.reason);
        return { content: [{ type: "text", text: fallback.text }], details: { valid: false, intent, reason: "validation_rejected", scratch_saved: fallback.saved, scratch_turn: fallback.turn } } as any;
      }
      const result = await pushDelta({ intent: intent.trim() });
      if (result.ok) return { content: [{ type: "text", text: `Intent set: ${intent.slice(0, 100)}` }], details: { valid: true, reason: undefined, intent } };
      const fallback = namedSlotFallback("intent", "intent", result.reason, intent.trim(), result.api_reason);
      return { content: [{ type: "text", text: fallback.text }], details: { valid: false, intent, reason: result.reason, scratch_saved: fallback.saved, scratch_turn: fallback.turn, ...fallback.recovery } } as any;
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
      if (!v.valid) {
        const fallback = namedSlotValidationFallback("current focus", "current_focus", focus.trim(), v.reason);
        return { content: [{ type: "text", text: fallback.text }], details: { valid: false, focus, reason: "validation_rejected", scratch_saved: fallback.saved, scratch_turn: fallback.turn, suggested_current_focus: fallback.suggestion } } as any;
      }
      const result = await pushDelta({ current_focus: focus.trim() });
      if (result.ok) return { content: [{ type: "text", text: `Current focus set: ${focus.slice(0, 100)}` }], details: { valid: true, reason: undefined, focus } };
      const fallback = namedSlotFallback("current focus", "current_focus", result.reason, focus.trim(), result.api_reason);
      return { content: [{ type: "text", text: fallback.text }], details: { valid: false, focus, reason: result.reason, scratch_saved: fallback.saved, scratch_turn: fallback.turn, ...fallback.recovery } } as any;
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
      if (!v.valid) {
        const fallback = namedSlotValidationFallback("next step", "next_step", step.trim(), v.reason);
        return { content: [{ type: "text", text: fallback.text }], details: { valid: false, step, reason: "validation_rejected", scratch_saved: fallback.saved, scratch_turn: fallback.turn } } as any;
      }
      const result = await pushDelta({ next_steps: [step.trim()] });
      if (result.ok) return { content: [{ type: "text", text: `Next step recorded: ${step.slice(0, 80)}` }], details: { valid: true, reason: undefined, step } };
      const fallback = namedSlotFallback("next step", "next_step", result.reason, step.trim(), result.api_reason);
      return { content: [{ type: "text", text: fallback.text }], details: { valid: false, step, reason: result.reason, scratch_saved: fallback.saved, scratch_turn: fallback.turn, ...fallback.recovery } } as any;
    },
  });

  // ── focusa_open_question (§AsccSections) ─────────────────────────────────
  pi.registerTool({
    name: "focusa_open_question",
    label: "Record Open Question",
    description: "Record an open question that needs to be answered (max 180 chars).",
    parameters: Type.Object({
      question: Type.String({ description: "Open question (max 180 chars)." }),
    }),
    async execute(_id, params) {
      const { question } = params as { question: string };
      const v = validateNamedSlot(question, 180, "open_question");
      if (!v.valid) {
        const fallback = namedSlotValidationFallback("open question", "open_question", question.trim(), v.reason);
        return { content: [{ type: "text", text: fallback.text }], details: { valid: false, question, reason: "validation_rejected", scratch_saved: fallback.saved, scratch_turn: fallback.turn } } as any;
      }
      const result = await pushDelta({ open_questions: [question.trim()] });
      if (result.ok) return { content: [{ type: "text", text: `Open question recorded: ${question.slice(0, 80)}` }], details: { valid: true, reason: undefined, question } };
      const fallback = namedSlotFallback("open question", "open_question", result.reason, question.trim(), result.api_reason);
      return { content: [{ type: "text", text: fallback.text }], details: { valid: false, question, reason: result.reason, scratch_saved: fallback.saved, scratch_turn: fallback.turn, ...fallback.recovery } } as any;
    },
  });

  // ── focusa_recent_result (§AsccSections) ─────────────────────────────────
  // Record a recent result. Keeps last 10, newest first.
  pi.registerTool({
    name: "focusa_recent_result",
    label: "Record Recent Result",
    description: "Record a completed result, output, or reference (max 180 chars).",
    parameters: Type.Object({
      result: Type.String({ description: "Recent result (max 180 chars)." }),
    }),
    async execute(_id, params) {
      const { result } = params as { result: string };
      const v = validateNamedSlot(result, 180, "recent_result");
      if (!v.valid) {
        const fallback = namedSlotValidationFallback("recent result", "recent_result", result.trim(), v.reason);
        return { content: [{ type: "text", text: fallback.text }], details: { valid: false, result, reason: "validation_rejected", scratch_saved: fallback.saved, scratch_turn: fallback.turn } } as any;
      }
      const writeResult = await pushDelta({ recent_results: [result.trim()] });
      if (writeResult.ok) return { content: [{ type: "text", text: `Result recorded: ${result.slice(0, 80)}` }], details: { valid: true, reason: undefined, result } };
      const fallback = namedSlotFallback("recent result", "recent_result", writeResult.reason, result.trim(), writeResult.api_reason);
      return { content: [{ type: "text", text: fallback.text }], details: { valid: false, result, reason: writeResult.reason, scratch_saved: fallback.saved, scratch_turn: fallback.turn, ...fallback.recovery } } as any;
    },
  });

  // ── focusa_note (§AsccSections) ───────────────────────────────────────────
  // Misc notes, bounded at 20, oldest decay first.
  pi.registerTool({
    name: "focusa_note",
    label: "Record Note",
    description: "Miscellaneous note (max 180 chars). Bounded at 20, oldest decay first.",
    parameters: Type.Object({
      note: Type.String({ description: "Note (max 180 chars)." }),
    }),
    async execute(_id, params) {
      const { note } = params as { note: string };
      const v = validateNamedSlot(note, 180, "note");
      if (!v.valid) {
        const fallback = namedSlotValidationFallback("note", "note", note.trim(), v.reason);
        return { content: [{ type: "text", text: fallback.text }], details: { valid: false, note, reason: "validation_rejected", scratch_saved: fallback.saved, scratch_turn: fallback.turn } } as any;
      }
      const result = await pushDelta({ notes: [note.trim()] });
      if (result.ok) return { content: [{ type: "text", text: `Note recorded: ${note.slice(0, 80)}` }], details: { valid: true, reason: undefined, note } };
      const fallback = namedSlotFallback("note", "note", result.reason, note.trim(), result.api_reason);
      return { content: [{ type: "text", text: fallback.text }], details: { valid: false, note, reason: result.reason, scratch_saved: fallback.saved, scratch_turn: fallback.turn, ...fallback.recovery } } as any;
    },
  });

  // ── Continuous Work Loop bridge tools (Spec79 §23 small bridge surface) ──

  type FocusaRouteTier = "hot" | "warm" | "cold";

  function focusaRouteTier(path: string, method = "GET"): FocusaRouteTier {
    const route = String(path || "").toLowerCase();
    const verb = String(method || "GET").toUpperCase();
    if (
      route.includes("/deep") ||
      route.includes("/replay/") ||
      route.includes("closure-bundle") ||
      route.includes("closure-evidence") ||
      route.includes("/state/dump") ||
      route.includes("worktree") ||
      route.includes("diagnostic") ||
      route.includes("include_full_payload=true") ||
      route.includes("mode=full") ||
      /[?&]deep=true/.test(route)
    ) return "cold";
    if (verb !== "GET") return "warm";
    return "hot";
  }

  function timeoutFailureClassForRoute(path: string, method?: string): FocusaFailureClass {
    return focusaRouteTier(path, method) === "cold" ? "cold_path_timeout" : "hot_path_timeout";
  }

  function timeoutBudgetForRoute(path: string, method = "GET"): number {
    const configured = S.cfg?.focusaApiTimeoutMs || 5000;
    const tier = focusaRouteTier(path, method);
    if (tier === "hot" && path.startsWith("/trajectory/view")) return Math.min(Math.max(configured, 4000), 5000);
    if (tier === "hot") return Math.min(configured, 2500);
    if (tier === "cold") return Math.max(configured, 8000);
    return configured;
  }

  function compactFallbackPacket(value: any): any {
    if (!value || typeof value !== "object") return value;
    return {
      status: value.status || "timeout_preserved",
      canonical: value.canonical === true,
      degraded: value.degraded !== false,
      failure_class: value.failure_class || "hot_path_timeout",
      workpoint_id: value.workpoint_id || value.fallback_packet?.workpoint_id || null,
      trajectory_id: value.trajectory_id || value.trajectory_candidate?.trajectory_id || null,
      project_root: value.project_root || value.input?.project_root || null,
      continuity_id: value.continuity_id || value.input?.continuity_id || null,
      preserved_at: value.preserved_at || null,
      next_step_hint: value.next_step_hint || "retry after doctor/resource_mode",
    };
  }

  async function fetchJsonDetailed(url: string, timeoutMs = 1500): Promise<{ ok: boolean; status: number; body: any | null; error?: string }> {
    const ac = new AbortController();
    const t = setTimeout(() => ac.abort(), timeoutMs);
    try {
      const r = await fetch(url, { signal: ac.signal });
      let body: any = null;
      try { body = await r.json(); } catch { body = null; }
      return { ok: r.ok, status: r.status, body };
    } catch (err: any) {
      return { ok: false, status: err?.name === "AbortError" ? 408 : 0, body: null, error: String(err?.message || err || "request failed") };
    } finally {
      clearTimeout(t);
    }
  }

  async function uiaiBrowserHealthCard(): Promise<any> {
    const base = String(process.env.UIAI_ENGINE_URL || process.env.WPUIAI_ENGINE_URL || "http://127.0.0.1:7456").replace(/\/$/, "");
    const [health, metrics] = await Promise.all([
      fetchJsonDetailed(`${base}/api/health/browser`, 1200),
      fetchJsonDetailed(`${base}/api/metrics/browser`, 1200),
    ]);
    const body = metrics.body || health.body || {};
    const queue = body.queue || {};
    const currentCapacity = body.current_capacity || body.agent_pressure?.current_capacity || {};
    const capacityAvailable = currentCapacity.capacity_available === true || Number(currentCapacity.remaining_page_slots || 0) > 0 || Number(currentCapacity.available_idle_pages || 0) > 0;
    const p95 = Number(queue.p95_wait_ms || 0);
    const p99 = Number(queue.p99_wait_ms || 0);
    const rejected = Number(queue.rejected || 0);
    const status = String(body.status || (health.ok || metrics.ok ? "ok" : "unavailable"));
    const historicalPressure = p99 >= 5000 || p95 >= 2500 || rejected > 0 ? "high" : p99 >= 1500 || p95 >= 750 ? "medium" : "low";
    const pressure = capacityAvailable ? "low" : historicalPressure;
    return {
      ok: health.ok || metrics.ok,
      status,
      base_url: base,
      health_status: health.status,
      metrics_status: metrics.status,
      browser_alive: body.browser_alive,
      browser_state: body.browser_state,
      queue,
      current_capacity: currentCapacity,
      historical_pressure: historicalPressure,
      pressure,
      recommended_action: pressure === "high" ? "narrow browser workload, close stale sessions, or retry after queue drains" : pressure === "medium" ? "monitor browser queue before parallel UIAI work" : "continue normally",
      response: compactApiEcho(body),
      error: health.error || metrics.error || null,
    };
  }

  async function focusaFetchDetailed(path: string, opts: RequestInit = {}): Promise<{ ok: boolean; status: number; body: any | null }> {
    const timeout = timeoutBudgetForRoute(path, String(opts.method || "GET"));
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
    } catch (err: any) {
      const aborted = err?.name === "AbortError";
      const method = String(opts.method || "GET");
      const routeTier = focusaRouteTier(path, method);
      const failureClass = aborted ? timeoutFailureClassForRoute(path, method) : "daemon_unavailable";
      return {
        ok: false,
        status: aborted ? 408 : 0,
        body: {
          error: aborted ? `${routeTier} route request timed out` : "daemon unavailable",
          failure_class: failureClass,
          route_tier: routeTier,
          endpoint: `/v1${path}`,
          retry: { safe: routeTier !== "warm", posture: routeTier === "cold" ? "safe_retry" : "check_side_effects_first" },
        },
      };
    } finally {
      clearTimeout(t);
    }
  }

  function formatWorkLoopBudgetRemaining(value: any): string {
    if (value == null) return "unknown";
    if (typeof value !== "object") return String(value);
    const fields = [
      "remaining_turn_budget",
      "remaining_wall_clock_ms",
      "remaining_failure_budget",
      "remaining_low_productivity_budget",
      "remaining_same_subproblem_budget",
    ];
    const parts = fields
      .filter((field) => value[field] !== undefined && value[field] !== null)
      .map((field) => `${field}=${String(value[field])}`);
    return parts.length > 0 ? parts.join(",") : "empty";
  }

  function explainWorkLoopResult(result: { ok: boolean; status: number; body: any | null }, fallback: string): string {
    if (result.ok) return fallback;
    const msg = String(result.body?.error || "").toLowerCase();
    const activeWriter = result.body?.active_writer ? ` (${result.body.active_writer})` : "";
    if (msg.includes("claimed by another writer")) return `blocked: loop controlled by another session${activeWriter}`;
    if (msg.includes("worktree is not clean")) return "blocked: worktree has uncommitted changes";
    if (msg.includes("missing required header")) return "blocked: controller identity header missing";
    if (result.body?.failure_class === "cold_path_timeout") return "blocked: cold route timed out; hot tools may still be healthy";
    if (result.body?.failure_class === "hot_path_timeout") return "blocked: hot route timed out";
    if (result.body?.failure_class === "scope_mismatch" || result.body?.status === "rejected_scope_mismatch" || result.status === 409) {
      const field = String(result.body?.field || "scope");
      const expected = String(result.body?.expected_project_root || result.body?.expected_continuity_id || "unknown");
      const actual = String(result.body?.packet_project_root || result.body?.packet_continuity_id || "unknown");
      const hint = String(result.body?.next_step_hint || "resume/checkpoint the Workpoint in the same scope before retrying");
      return `blocked: scope mismatch on ${field} expected=${expected} packet=${actual}; ${hint}`;
    }
    if (result.status === 0) return "blocked: daemon unavailable";
    return `blocked: ${result.body?.error || `request failed (${result.status})`}`;
  }

  function trajectoryTimeoutFallbackResult(action: string, endpoint: string, body: any, response: any, nextTools: string[], extra: Record<string, unknown> = {}) {
    const fallback = {
      ...extra,
      status: "timeout_preserved",
      canonical: false,
      degraded: true,
      advisory_only: true,
      failure_class: "hot_path_timeout",
      project_root: body.project_root,
      continuity_id: body.continuity_id || null,
      session_id: body.session_id || null,
      preserved_at: new Date().toISOString(),
      input: compactApiEcho(body),
      next_step_hint: `Retry ${action} after focusa_tool_doctor/resource_mode; do not treat timeout fallback as canonical.`,
    };
    S.lastTrajectoryClarity = {
      reason: `${action}_timeout_preserved`,
      refreshed_at: Date.now(),
      project_root: body.project_root,
      continuity_id: body.continuity_id || null,
      session_id: body.session_id || null,
      status: "timeout_preserved",
      recommended_action: `retry_${action}_after_tool_doctor_or_resource_mode`,
      canonical: false,
      degraded: true,
      trajectory_id: body.trajectory_id || null,
      current_state: body.observed_state || null,
      active_gap: body.summary || body.action_type || body.target_ref || null,
      timeout_preserved: true,
    };
    try { S.pi?.appendEntry("focusa-trajectory-timeout-fallback", fallback); } catch { /* best effort */ }
    persistState();
    return { content: [{ type: "text", text: timeoutPreservedText(`trajectory ${action}`) }], details: { ok: false, status: "timeout_preserved", endpoint, canonical: false, degraded: true, advisory_only: true, failure_class: "hot_path_timeout", fallback: compactFallbackPacket(fallback), response: compactApiEcho(response), next_tools: nextTools.slice(0, 4) } } as any;
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


  async function enforceTrajectoryClarityPrecondition(projectRoot: string, actionLabel: string, opts: { blockOperatorInput?: boolean; continuityId?: string; sessionId?: string } = {}): Promise<{ ok: boolean; text?: string; details: Record<string, any> }> {
    const query = new URLSearchParams();
    query.set("project_root", projectRoot);
    const sessionId = String(opts.sessionId || S.sessionFrameKey || "").trim();
    const continuityId = String(opts.continuityId || S.continuityId || ensureContinuityId(projectRoot) || "").trim();
    if (sessionId) query.set("session_id", sessionId);
    if (continuityId) query.set("continuity_id", continuityId);
    query.set("mode", "summary");
    const result = await focusaFetchDetailed(`/trajectory/view?${query.toString()}`, { method: "GET" });
    const body = result.body || {};
    const clarity = body.intelligence_view?.clarity_gate || {};
    const status = String(clarity.status || body.trajectory?.definition_status || "unknown");
    const action = String(clarity.recommended_action || body.intelligence_view?.context_sufficiency?.recommended_action || "unknown");
    const projectStatus = String(body.project_identity?.status || "unknown");
    const details = {
      action_label: actionLabel,
      trajectory_precondition: "trajectory_clarity_gate",
      status,
      recommended_action: action,
      project_identity_status: projectStatus,
      canonical: body.canonical === true,
      trajectory_id: body.trajectory?.trajectory_id || null,
      missing_facts: body.intelligence_view?.context_sufficiency?.missing_facts || [],
      next_tools: body.next_tools || ["focusa_trajectory_view", "focusa_project_verify"],
    };
    if (!result.ok) {
      return { ok: false, text: `${actionLabel} blocked → trajectory clarity gate unavailable (${explainWorkLoopResult(result, "trajectory unavailable")})`, details: { ...details, failure_class: result.body?.failure_class || "daemon_unavailable" } };
    }
    if (projectStatus === "mismatch" || status === "conflicted") {
      const recovery = scopeRecoveryContext(body, projectRoot, continuityId, "trajectory_clarity_gate");
      if (allowsWorkpointBootstrapFromClarity(body, projectRoot, actionLabel)) {
        return { ok: true, text: recovery?.text, details: { ...details, failure_class: "scope_mismatch", bootstrap_allowed: true, precondition_warning: "trajectory conflicted because existing Focusa context is for another continuity; checkpointing current operator mission is allowed", scope_recovery_context: recovery?.details || null } };
      }
      return { ok: false, text: `${actionLabel} blocked → trajectory clarity gate conflicted; verify project identity and trajectory before canonical mutation.${recovery ? ` ${recovery.text}` : ""}`, details: { ...details, failure_class: "scope_mismatch", scope_recovery_context: recovery?.details || null } };
    }
    if (opts.blockOperatorInput !== false && (status === "unclear" || action === "operator_input")) {
      const recovery = scopeRecoveryContext(body, projectRoot, continuityId, "trajectory_clarity_gate");
      if (allowsWorkpointBootstrapFromClarity(body, projectRoot, actionLabel)) {
        return { ok: true, text: recovery?.text, details: { ...details, failure_class: "validation_rejected", bootstrap_allowed: true, precondition_warning: "trajectory unclear; checkpointing explicit operator mission is allowed to establish Workpoint continuity", scope_recovery_context: recovery?.details || null } };
      }
      return { ok: false, text: `${actionLabel} blocked → trajectory unclear; define or confirm trajectory before canonical mutation.${recovery ? ` ${recovery.text}` : ""}`, details: { ...details, failure_class: "validation_rejected", scope_recovery_context: recovery?.details || null } };
    }
    return { ok: true, details };
  }

  function evidenceClarityFallbackResult(kind: "evidence capture" | "workpoint evidence link", p: any, projectRoot: string, clarity: { text?: string; details: Record<string, any> }): any | null {
    const failureClass = String(clarity.details?.failure_class || "");
    const recoverable = ["hot_path_timeout", "cold_path_timeout", "daemon_unavailable", "resource_exhausted", "read_model_lag"].includes(failureClass);
    if (!recoverable) return null;
    const evidenceRef = String(p.evidence_ref || `${p.target_ref || "evidence"}:unlinked`).trim();
    const why = `trajectory clarity gate unavailable because ${failureClass}; proof should be preserved but not treated as linked`;
    const text = `${kind} degraded → ${why}. Proof handle preserved in response only; Workpoint link skipped. Next: focusa_workpoint_checkpoint → focusa_workpoint_resume, then retry link once. evidence_ref=${evidenceRef}`;
    const toolResult = focusaToolResult({
      ok: false,
      status: "degraded",
      failure_class: failureClass as FocusaFailureClass,
      canonical: false,
      degraded: true,
      summary: text,
      tool: kind === "evidence capture" ? "focusa_evidence_capture" : "focusa_workpoint_link_evidence",
      family: "workpoint",
      retry: { safe: true, posture: "safe_retry", reason: failureClass },
      side_effects: [],
      evidence_refs: [evidenceRef],
      next_tools: ["focusa_workpoint_checkpoint", "focusa_workpoint_resume", "focusa_trajectory_view", "focusa_tool_doctor"],
      raw: { trajectory_clarity_precondition: clarity.details, proof_preserved_not_linked: true, why, project_root: projectRoot },
    });
    return {
      content: [{ type: "text", text }],
      details: {
        ok: false,
        status: "degraded",
        why,
        proof_preserved_not_linked: true,
        evidence_ref: evidenceRef,
        project_root: projectRoot,
        project_root_permission_posture: projectRootPermissionPosture(projectRoot),
        trajectory_clarity_precondition: clarity.details,
        tool_result_v1: toolResult,
        failure_class: failureClass,
        next_tools: toolResult.next_tools,
        recovery_hint: toolResult.recovery_hint,
        misuse_hint: toolResult.misuse_hint,
      },
    } as any;
  }

  async function preferredWriterId(): Promise<string> {
    const status = await focusaFetchDetailed("/work-loop/status?summary_only=true");
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

    const status = await focusaFetchDetailed("/work-loop/status?summary_only=true");
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
      const result = await focusaFetchDetailed("/work-loop/status?summary_only=true");
      const body = result.body || {};
      const activeWriter = String(body.active_writer || "none");
      const status = String(body.status || body.current_task?.status || "unknown");
      const text = `work-loop writer-status → active_writer=${activeWriter} status=${status} preflight=read_only`;
      return { content: [{ type: "text", text }], details: { ok: result.ok, status: String(result.status), active_writer: activeWriter, authorship_mode: body.authorship_mode, preflight: { mutates: false, writer_required_for: ["control", "context", "checkpoint", "select_next"] }, response: compactApiEcho(body) } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_work_loop_status",
    label: "Work Loop Status",
    description: "Get current continuous work-loop state and budgets.",
    parameters: Type.Object({}),
    async execute() {
      const result = await focusaFetchDetailed("/work-loop/status?summary_only=true");

      if (!result.ok || !result.body) {
        return {
          content: [{ type: "text", text: `Work-loop summary ${explainWorkLoopResult(result, "ok")} | replay=not_checked_hot_path` }],
          details: {
            ok: false,
            status: result.status,
            endpoint: "/v1/work-loop/status?summary_only=true",
            hot_path: true,
            cold_paths_checked: false,
            response: result.body ?? null,
          },
        };
      }

      const loopStatus = result.body;
      const statusText = String(loopStatus?.status || loopStatus?.work_loop?.status || "unknown");
      const enabled = typeof loopStatus?.enabled === "boolean"
        ? loopStatus.enabled
        : !!loopStatus?.work_loop?.enabled;
      const activeWriter = String(loopStatus?.active_writer || "none");
      const budget = formatWorkLoopBudgetRemaining(loopStatus?.budget_remaining);
      return {
        content: [{ type: "text", text: `Work-loop summary: ${statusText} (enabled=${enabled ? "yes" : "no"}) active_writer=${activeWriter} budget_remaining=${budget} replay=not_checked_hot_path` }],
        details: {
          ok: true,
          status: result.status,
          endpoint: "/v1/work-loop/status?summary_only=true",
          hot_path: true,
          cold_paths_checked: false,
          response: result.body,
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
          details: { ok: res.ok, action: String(action), status: res.status, response: compactApiEcho(res.body) },
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
        details: { ok: res.ok, action: String(action), status: res.status, response: compactApiEcho(res.body) },
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
        details: { ok: res.ok, status: res.status, response: compactApiEcho(res.body) },
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
        details: { ok: res.ok, status: res.status, response: compactApiEcho(res.body) },
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
        details: { ok: res.ok, status: res.status, response: compactApiEcho(res.body) },
      };
    },
  });

  // ── Spec88 Workpoint Continuity tools ────────────────────────────────────

  function summarizeWorkpointResponse(body: any): string {
    const status = String(body?.status || "unknown");
    const id = String(body?.workpoint_id || body?.active_workpoint_id || body?.requested_workpoint_id || "none");
    const canonical = typeof body?.canonical === "boolean" ? String(body.canonical) : "unknown";
    const next = String(body?.next_step_hint || body?.resume_packet?.next_slice || body?.workpoint?.next_slice || "resume from typed workpoint packet");
    return `status=${status} id=${id} canonical=${canonical} next=${next}`;
  }

  function buildStateHygieneReport(stackBody: any): any {
    const frames = stackBody?.stack?.frames || [];
    const latest = Array.isArray(frames) ? frames[frames.length - 1] || {} : {};
    const state = latest?.state || latest?.focus_state || {};
    const slots = ["intent", "current_focus", "next_steps", "open_questions", "decisions", "constraints", "failures", "recent_results", "artifacts", "notes"];
    const signals: Array<{ id: string; slot: string; index: number; value: string }> = [];
    for (const slot of slots) {
      const raw = state?.[slot];
      const items = Array.isArray(raw) ? raw : raw ? [raw] : [];
      items.forEach((item: any, index: number) => {
        const value = typeof item === "string" ? item : JSON.stringify(item);
        const trimmed = String(value || "").trim();
        if (trimmed) signals.push({ id: `${slot}:${index}`, slot, index, value: trimmed.slice(0, 240) });
      });
    }
    const byValue = new Map<string, Array<{ id: string; slot: string; index: number; value: string }>>();
    for (const signal of signals) {
      const key = signal.value.toLowerCase().replace(/\s+/g, " ").slice(0, 180);
      byValue.set(key, [...(byValue.get(key) || []), signal]);
    }
    const duplicate_groups = Array.from(byValue.values()).filter((group) => group.length > 1).map((group, group_index) => ({ group_id: `dup:${group_index}`, count: group.length, signals: group }));
    const stale_candidates = signals
      .filter((signal) => ["next_steps", "open_questions", "current_focus"].includes(signal.slot) && /maybe|unclear|todo|fix|check|old|stale|previous/i.test(signal.value))
      .map((signal) => ({ ...signal, reason: "stale_language_or_unclear_marker" }));
    return {
      frame_id: latest?.id || latest?.frame_id || null,
      signal_count: signals.length,
      duplicate_count: duplicate_groups.reduce((sum, group) => sum + group.count, 0),
      duplicate_groups,
      stale_candidates,
      proposal_only_actions: [
        "append fresh superseding Focus State note",
        "checkpoint current mission before ignoring stale entries",
        "avoid deletion; Focus State reducer is append/audit oriented",
      ],
      recommended_action: duplicate_groups.length || stale_candidates.length ? "review_plan_then_apply_non_destructive_note" : "no_hygiene_needed",
    };
  }

  pi.registerTool({
    name: "focusa_state_hygiene_doctor",
    label: "Focus State Hygiene Doctor",
    description: "Diagnose stale or duplicate Focus State signals without mutating state.",
    parameters: Type.Object({}),
    async execute() {
      const stack = await focusaFetchDetailed("/focus/stack", { method: "GET" });
      const result = buildStateHygieneReport(stack.body || {});
      return { content: [{ type: "text", text: `state hygiene doctor → signals=${result.signal_count} duplicate_groups=${result.duplicate_groups.length} stale_candidates=${result.stale_candidates.length} recommended=${result.recommended_action}` }], details: { ok: stack.ok, status: String(stack.status), response: result, next_tools: ["focusa_state_hygiene_plan", "focusa_workpoint_resume", "focusa_tool_doctor"] } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_state_hygiene_plan",
    label: "Focus State Hygiene Plan",
    description: "Create a proposal-style hygiene plan; does not mutate Focus State.",
    parameters: Type.Object({ reason: Type.Optional(Type.String({ description: "Why hygiene is being considered." })) }),
    async execute(_id, params) {
      const p = params as any;
      const stack = await focusaFetchDetailed("/focus/stack", { method: "GET" });
      const report = buildStateHygieneReport(stack.body || {});
      const plan = { mutates: false, reason: String(p.reason || "operator requested hygiene plan"), target_frame_id: report.frame_id, exact_duplicate_groups: report.duplicate_groups, exact_stale_candidates: report.stale_candidates, actions: report.proposal_only_actions, apply_requires_approval: true };
      return { content: [{ type: "text", text: `state hygiene plan → duplicate_groups=${report.duplicate_groups.length} stale_candidates=${report.stale_candidates.length} mutates=false` }], details: { ok: stack.ok, status: stack.ok ? "completed" : "degraded", plan, report } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_state_hygiene_apply",
    label: "Focus State Hygiene Apply",
    description: "Approval-gated, non-destructive hygiene apply; records an auditable Focus State note via reducer-backed /focus/update.",
    parameters: Type.Object({ approved: Type.Boolean({ description: "Must be true to apply proposal-safe hygiene." }), reason: Type.Optional(Type.String()) }),
    async execute(_id, params) {
      const p = params as any;
      if (p.approved !== true) {
        return {
          content: [{ type: "text", text: "state hygiene apply blocked → approval required" }],
          details: {
            ok: false,
            status: "blocked",
            reason: "approval_required",
            tool_result_v1: {
              ok: false,
              status: "blocked",
              canonical: false,
              degraded: false,
              failure_class: "approval_required",
              summary: "state hygiene apply requires approved=true",
              retry: { safe: false, posture: "operator_required", reason: "approval_required" },
              side_effects: [],
              evidence_refs: [],
              next_tools: ["focusa_state_hygiene_plan", "focusa_state_hygiene_doctor"],
              error: { code: "approval_required", message: "approved=true is required" },
            },
          },
        } as any;
      }

      const reason = String(p.reason || "approved hygiene review").slice(0, 120);
      const note = `State hygiene applied; no deletion; reason=${reason}`.slice(0, 200);
      const result = await focusaFetchDetailed("/focus/update", {
        method: "POST",
        body: JSON.stringify({ delta: { notes: [note] } }),
      });
      const accepted = result.ok && String(result.body?.status || "") === "accepted";
      return {
        content: [{ type: "text", text: accepted ? "state hygiene apply → recorded non-destructive Focus State note" : `state hygiene apply blocked → ${String(result.body?.reason || result.body?.status || result.status)}` }],
        details: {
          ok: accepted,
          status: accepted ? "completed" : "blocked",
          mutates: accepted,
          mutation: accepted ? "focus_state_note_append" : "none",
          reason,
          response: result.body,
          tool_result_v1: {
            ok: accepted,
            status: accepted ? "completed" : "blocked",
            canonical: accepted,
            degraded: false,
            failure_class: accepted ? undefined : "focus_update_unavailable",
            summary: accepted ? "state hygiene apply recorded an auditable note" : "state hygiene apply could not write Focus State note",
            retry: { safe: !accepted, posture: accepted ? "do_not_retry_unchanged" : "safe_retry", reason: accepted ? "completed" : "focus_update_unavailable" },
            side_effects: accepted ? ["focus_state_note_append"] : [],
            evidence_refs: accepted && result.body?.frame_id ? [`focus_frame:${result.body.frame_id}`] : [],
            next_tools: accepted ? ["focusa_state_hygiene_doctor"] : ["focusa_tool_doctor", "focusa_workpoint_resume"],
            error: accepted ? null : { code: "focus_update_unavailable", message: String(result.body?.reason || result.body?.status || result.status) },
          },
        },
      } as any;
    },
  });


  type SilentSessionAction = "list" | "start" | "reopen" | "kill" | "tail" | "send" | "health" | "interrupt" | "restart";
  const SILENT_SESSION_PREFIX = "focusa-silent";
  const SILENT_SESSION_LOG_MAX_BYTES = 5 * 1024 * 1024;
  const SILENT_SESSION_LOG_BACKUPS = 3;
  const SILENT_SESSION_STALE_SECONDS = 30 * 60;
  const SILENT_SESSION_REGISTRY_PATH = "/tmp/focusa-silent-registry.json";

  function shellQuote(value: string): string {
    return `'${String(value).replace(/'/g, `'\''`)}'`;
  }

  function currentUserName(): string {
    try { return require("os").userInfo().username || "unknown"; } catch { return "unknown"; }
  }

  function silentSessionExec(args: string[], timeout = 5000, runAsUser?: string | null): { ok: boolean; stdout: string; stderr: string; status: number | null } {
    try {
      const { spawnSync } = require("child_process");
      const current = currentUserName();
      if (runAsUser && runAsUser !== current && runAsUser !== "root") {
        const cmd = `tmux ${args.map((arg) => shellQuote(String(arg))).join(" ")}`;
        const r = spawnSync("as-user", [runAsUser, cmd], { encoding: "utf8", timeout });
        return { ok: r.status === 0, stdout: r.stdout || "", stderr: r.stderr || "", status: r.status };
      }
      const r = spawnSync("tmux", args, { encoding: "utf8", timeout });
      return { ok: r.status === 0, stdout: r.stdout || "", stderr: r.stderr || "", status: r.status };
    } catch (err: any) {
      return { ok: false, stdout: "", stderr: String(err?.message || err), status: null };
    }
  }

  function silentSessionRootOwner(rootDir: string): { user: string; group: string; uid: number | null; ok: boolean; error?: string } {
    try {
      const { spawnSync } = require("child_process");
      const r = spawnSync("stat", ["-c", "%U\t%G\t%u", rootDir], { encoding: "utf8", timeout: 3000 });
      if (r.status !== 0) return { user: currentUserName(), group: "unknown", uid: null, ok: false, error: r.stderr || "stat failed" };
      const [user, group, uid] = String(r.stdout || "").trim().split("\t");
      return { user: user || currentUserName(), group: group || "unknown", uid: uid ? Number(uid) : null, ok: true };
    } catch (err: any) {
      return { user: currentUserName(), group: "unknown", uid: null, ok: false, error: String(err?.message || err) };
    }
  }

  function projectRootPermissionPosture(projectRoot: string): Record<string, unknown> {
    const owner = silentSessionRootOwner(projectRoot);
    const current_user = currentUserName();
    const root_user_home = /^\/home\/[^/]+(?:\/|$)/.test(projectRoot);
    return {
      project_root: projectRoot,
      root_owner: owner,
      current_user,
      root_owned_by_current_user: owner.user === current_user,
      root_user_home,
      posture: root_user_home && owner.user !== current_user ? "cross_user_home_use_as_owner" : "same_user_or_non_home_root",
      guidance: root_user_home && owner.user !== current_user ? `Run repo/file mutations via as-user ${owner.user}; avoid root-owned files under ${projectRoot}.` : "Project root ownership matches current user or is outside /home user space.",
    };
  }

  function silentSessionName(raw?: unknown): string {
    const base = String(raw || "default")
      .toLowerCase()
      .replace(/[^a-z0-9._:-]+/g, "-")
      .replace(/^-+|-+$/g, "")
      .slice(0, 80) || "default";
    return base.startsWith(SILENT_SESSION_PREFIX) ? base : `${SILENT_SESSION_PREFIX}-${base}`;
  }

  function silentSessionAttachCommand(name: string, detachOthers = false): string {
    return `tmux attach${detachOthers ? " -d" : ""} -t ${name}`;
  }

  function silentSessionTailCommand(name: string, lines = 80): string {
    return `tmux capture-pane -p -J -t ${name} -S -${lines}`;
  }

  function silentSessionMetaPath(name: string): string {
    const safe = String(name || "default").replace(/[^a-zA-Z0-9._:-]+/g, "-").slice(0, 100) || "default";
    return `/tmp/${safe}.json`;
  }

  function silentSessionLogPath(name: string, runAsUser?: string | null): string {
    const safe = String(name || "default").replace(/[^a-zA-Z0-9._:-]+/g, "-").slice(0, 100) || "default";
    const user = String(runAsUser || currentUserName() || "unknown").replace(/[^a-zA-Z0-9._:-]+/g, "-").slice(0, 40) || "unknown";
    return `/tmp/${safe}-${user}.log`;
  }

  function silentSessionReadRegistry(): Record<string, any> {
    try {
      const fs = require("fs");
      if (!fs.existsSync(SILENT_SESSION_REGISTRY_PATH)) return {};
      const parsed = JSON.parse(fs.readFileSync(SILENT_SESSION_REGISTRY_PATH, "utf8"));
      return parsed && typeof parsed === "object" ? parsed.sessions || parsed : {};
    } catch { return {}; }
  }

  function silentSessionWriteRegistry(registry: Record<string, any>): void {
    try { require("fs").writeFileSync(SILENT_SESSION_REGISTRY_PATH, JSON.stringify({ schema_version: "focusa.silent_sessions.registry.v1", updated_at: new Date().toISOString(), sessions: registry }, null, 2)); } catch { /* best effort */ }
  }

  function silentSessionReadMeta(name: string): any | null {
    try {
      const fs = require("fs");
      const path = silentSessionMetaPath(name);
      if (fs.existsSync(path)) return JSON.parse(fs.readFileSync(path, "utf8"));
      return silentSessionReadRegistry()[name] || null;
    } catch { return silentSessionReadRegistry()[name] || null; }
  }

  function silentSessionWriteMeta(name: string, meta: any): void {
    try { require("fs").writeFileSync(silentSessionMetaPath(name), JSON.stringify(meta, null, 2)); } catch { /* best effort */ }
    const registry = silentSessionReadRegistry();
    registry[name] = { ...(registry[name] || {}), ...meta, session_name: name, registry_updated_at: new Date().toISOString() };
    silentSessionWriteRegistry(registry);
  }

  function silentSessionRunAsFor(name: string): string | null {
    const meta = silentSessionReadMeta(name);
    return meta?.run_as_user || null;
  }

  function silentSessionLogStats(logPath: string): { exists: boolean; size_bytes: number; mtime: number | null; mtime_iso: string | null; age_seconds: number | null; error?: string } {
    try {
      const fs = require("fs");
      if (!fs.existsSync(logPath)) return { exists: false, size_bytes: 0, mtime: null, mtime_iso: null, age_seconds: null };
      const stat = fs.statSync(logPath);
      const mtime = Math.floor(stat.mtimeMs / 1000);
      const age = Math.max(0, Math.floor(Date.now() / 1000) - mtime);
      return { exists: true, size_bytes: stat.size, mtime, mtime_iso: new Date(mtime * 1000).toISOString(), age_seconds: age };
    } catch (err: any) {
      return { exists: false, size_bytes: 0, mtime: null, mtime_iso: null, age_seconds: null, error: String(err?.message || err) };
    }
  }

  function silentSessionRotateLog(logPath: string): { rotated: boolean; max_bytes: number; backups: number; error?: string } {
    try {
      const fs = require("fs");
      if (!fs.existsSync(logPath) || fs.statSync(logPath).size < SILENT_SESSION_LOG_MAX_BYTES) {
        return { rotated: false, max_bytes: SILENT_SESSION_LOG_MAX_BYTES, backups: SILENT_SESSION_LOG_BACKUPS };
      }
      for (let i = SILENT_SESSION_LOG_BACKUPS - 1; i >= 1; i--) {
        const from = `${logPath}.${i}`;
        const to = `${logPath}.${i + 1}`;
        if (fs.existsSync(from)) fs.renameSync(from, to);
      }
      fs.renameSync(logPath, `${logPath}.1`);
      return { rotated: true, max_bytes: SILENT_SESSION_LOG_MAX_BYTES, backups: SILENT_SESSION_LOG_BACKUPS };
    } catch (err: any) {
      return { rotated: false, max_bytes: SILENT_SESSION_LOG_MAX_BYTES, backups: SILENT_SESSION_LOG_BACKUPS, error: String(err?.message || err) };
    }
  }

  function silentSessionEnablePipeLog(name: string, runAsUser?: string | null): { ok: boolean; log_path: string; rotated: boolean; max_bytes: number; backups: number; error?: string } {
    const logPath = silentSessionLogPath(name, runAsUser);
    const rotation = silentSessionRotateLog(logPath);
    const r = silentSessionExec(["pipe-pane", "-o", "-t", name, `cat >> ${shellQuote(logPath)}`], 3000, runAsUser);
    return { ok: r.ok, log_path: logPath, rotated: rotation.rotated, max_bytes: rotation.max_bytes, backups: rotation.backups, error: r.stderr || rotation.error || undefined };
  }

  function silentSessionVersion(): string {
    const r = silentSessionExec(["-V"], 3000);
    return r.ok ? r.stdout.trim() : "unknown";
  }

  function silentSessionPaneHealth(sessionName: string, sessionInfo?: any): any {
    const runAsUser = silentSessionRunAsFor(sessionName);
    const logPath = silentSessionReadMeta(sessionName)?.log_path || silentSessionLogPath(sessionName, runAsUser);
    const logStats = silentSessionLogStats(logPath);
    const r = silentSessionExec(["list-panes", "-t", sessionName, "-F", "#{pane_id}\t#{pane_active}\t#{pane_dead}\t#{pane_current_command}\t#{pane_pid}\t#{pane_exit_status}"], 3000, runAsUser);
    if (!r.ok) return { ok: false, status: "unknown", panes: [], run_as_user: runAsUser || currentUserName(), log_path: logPath, log_stats: logStats, error: r.stderr || "tmux list-panes failed" };
    const panes = r.stdout.split("\n").filter(Boolean).map((line: string) => {
      const [pane_id, active, dead, current_command, pane_pid, exit_status] = line.split("\t");
      return { pane_id, active: active === "1", dead: dead === "1", current_command: current_command || null, pane_pid: pane_pid ? Number(pane_pid) : null, exit_status: exit_status || null };
    });
    const deadCount = panes.filter((pane: any) => pane.dead).length;
    const baseStatus = panes.length === 0 ? "unknown" : deadCount === panes.length ? "dead" : deadCount > 0 ? "degraded" : "running";
    const now = Math.floor(Date.now() / 1000);
    const activityAge = sessionInfo?.activity ? Math.max(0, now - Number(sessionInfo.activity)) : null;
    const noRecentLog = logStats.age_seconds !== null && logStats.age_seconds >= SILENT_SESSION_STALE_SECONDS;
    const noRecentActivity = activityAge !== null && activityAge >= SILENT_SESSION_STALE_SECONDS;
    const status = baseStatus === "running" && (noRecentLog || noRecentActivity) ? "stale" : baseStatus;
    return { ok: true, status, panes, run_as_user: runAsUser || currentUserName(), log_path: logPath, log_stats: logStats, activity_age_seconds: activityAge, stale_after_seconds: SILENT_SESSION_STALE_SECONDS };
  }

  function listSilentSessions() {
    const format = "#{session_name}\t#{session_attached}\t#{session_windows}\t#{session_created}\t#{session_activity}\t#{session_id}\t#{window_name}";
    const currentR = silentSessionExec(["list-sessions", "-F", format], 3000);
    const registry = silentSessionReadRegistry();
    const metaUsers = (() => {
      try {
        const fs = require("fs");
        const fileUsers = fs.readdirSync("/tmp")
          .filter((file: string) => file.startsWith(SILENT_SESSION_PREFIX) && file.endsWith(".json"))
          .map((file: string) => silentSessionReadMeta(file.replace(/\.json$/, ""))?.run_as_user)
          .filter(Boolean);
        const registryUsers = Object.values(registry).map((meta: any) => meta?.run_as_user).filter(Boolean);
        return Array.from(new Set([...fileUsers, ...registryUsers]));
      } catch { return Array.from(new Set(Object.values(registry).map((meta: any) => meta?.run_as_user).filter(Boolean))); }
    })() as string[];
    const outputs: string[] = [];
    if (currentR.ok) outputs.push(currentR.stdout || "");
    for (const user of metaUsers) {
      if (user === currentUserName()) continue;
      const r = silentSessionExec(["list-sessions", "-F", format], 3000, user);
      if (r.ok) outputs.push(r.stdout || "");
    }
    if (!outputs.length && /no server running|failed to connect/i.test(currentR.stderr)) return [];
    const seen = new Set<string>();
    return outputs.join("\n").split("\n").filter(Boolean).map((line: string) => {
      const [name, attached, windows, created, activity, sessionId, windowName] = line.split("\t");
      const createdNum = Number(created || 0);
      const activityNum = Number(activity || 0);
      if (seen.has(name)) return null;
      seen.add(name);
      const meta = silentSessionReadMeta(name) || {};
      return {
        name,
        attached: attached === "1",
        windows: Number(windows || 0),
        created: createdNum,
        created_iso: createdNum ? new Date(createdNum * 1000).toISOString() : null,
        activity: activityNum,
        activity_iso: activityNum ? new Date(activityNum * 1000).toISOString() : null,
        session_id: sessionId || null,
        window_name: windowName || null,
        attach_command: silentSessionAttachCommand(name),
        attach_detach_others_command: silentSessionAttachCommand(name, true),
        tail_command: silentSessionTailCommand(name),
        log_path: meta.log_path || silentSessionLogPath(name, meta.run_as_user),
        run_as_user: meta.run_as_user || currentUserName(),
        root_owner: meta.root_owner || null,
        permission_posture: meta.permission_posture || "current_user_tmux",
        registry_metadata: meta,
        registry_state: meta.session_name ? "active_registered" : "active_unregistered",
      };
    }).filter((session: any) => session && String(session.name || "").startsWith(SILENT_SESSION_PREFIX));
  }

  function silentSessionRegistrySnapshot(activeSessions: any[]): any {
    const registry = silentSessionReadRegistry();
    const active = new Set(activeSessions.map((session: any) => String(session.name || "")));
    const entries = Object.entries(registry).map(([name, meta]) => ({ name, active: active.has(name), ...(meta as Record<string, unknown>) }));
    return { path: SILENT_SESSION_REGISTRY_PATH, count: entries.length, active_count: entries.filter((entry: any) => entry.active).length, stale_count: entries.filter((entry: any) => !entry.active).length, entries };
  }

  function defaultSilentSessionCommand(p: any, sessionName: string): string {
    const rootDir = String(p.root_dir || S.sessionCwd || process.cwd()).replace(/'/g, `'\\''`);
    const mission = String(p.mission || "Continue Focusa-governed ready beads using trajectory/workpoint context; stop on destructive risk.").replace(/'/g, `'\\''`);
    const bead = String(p.work_item_id || "").replace(/'/g, `'\\''`);
    const lowmem = p.lowmem === false ? "" : "curl -fsS --max-time 5 -X POST http://127.0.0.1:8787/v1/resource/mode -H 'Content-Type: application/json' --data '{\"action\":\"activate_lowmem\",\"reason\":\"SilentSession start\"}' >/tmp/focusa-silent-lowmem.json 2>/tmp/focusa-silent-lowmem.err || true; ";
    return `cd '${rootDir}' && ${lowmem}pi 'SilentSession ${sessionName}: ${mission}${bead ? ` Work item: ${bead}.` : ""} Use Focusa trajectory/workpoint/beads, record evidence, checkpoint often, stop for destructive/high-risk actions, and accept operator steering sent through tmux send-keys.'`;
  }

  function silentSessionBlocked(action: SilentSessionAction | string, sessionName: string, failureClass: FocusaFailureClass, why: string, sessions: any[] = [], extra: Record<string, any> = {}): any {
    const nextTools = failureClass === "approval_required"
      ? ["focusa_silent_sessions", "focusa_work_loop_writer_status"]
      : failureClass === "frame_unavailable"
        ? ["focusa_silent_sessions", "focusa_tool_doctor"]
        : ["focusa_silent_sessions", "focusa_tool_doctor", "focusa_resource_mode"];
    const summary = `silent session ${action} blocked → ${why}`;
    const toolResult = focusaToolResult({
      ok: false,
      status: "blocked",
      failure_class: failureClass,
      canonical: false,
      degraded: true,
      summary,
      tool: "focusa_silent_sessions",
      family: "work_loop",
      retry: { safe: failureClass !== "approval_required", posture: failureClass === "approval_required" ? "operator_required" : "safe_retry", reason: failureClass },
      side_effects: [],
      evidence_refs: [],
      next_tools: nextTools,
      raw: { action, session_name: sessionName, sessions, ...extra },
    });
    return {
      content: [{ type: "text", text: `${summary}. Why: ${why}. Next: ${toolResult.recovery_hint || nextTools.join(" → ")}` }],
      details: { ok: false, status: "blocked", session_name: sessionName, sessions, why, failure_class: failureClass, recovery_hint: toolResult.recovery_hint, misuse_hint: toolResult.misuse_hint, next_tools: toolResult.next_tools, tool_result_v1: toolResult, ...extra },
    } as any;
  }

  pi.registerTool({
    name: "focusa_silent_sessions",
    label: "Focusa Silent Sessions",
    description: "List, start, reopen, tail, send input to, or safely kill tmux-backed Focusa SilentSessions running in the background.",
    promptSnippet: "Use when an operator asks to list, reopen, start, or kill background SilentSessions/autopilot tmux sessions.",
    parameters: Type.Object({
      action: Type.Optional(Type.Union([
        Type.Literal("list"),
        Type.Literal("start"),
        Type.Literal("reopen"),
        Type.Literal("tail"),
        Type.Literal("send"),
        Type.Literal("kill"),
        Type.Literal("health"),
        Type.Literal("interrupt"),
        Type.Literal("restart"),
      ], { description: "SilentSession action. list is default; kill/send/start/interrupt/restart require approved=true." })),
      session_name: Type.Optional(Type.String({ description: "SilentSession name or suffix. Names are normalized under focusa-silent-* prefix." })),
      root_dir: Type.Optional(Type.String({ description: "Working directory for a new SilentSession; defaults to current Pi cwd." })),
      command: Type.Optional(Type.String({ description: "Custom shell command for start or input line for send. Omit for default Focusa-governed Pi autopilot command." })),
      mission: Type.Optional(Type.String({ description: "Mission prompt for default start command." })),
      work_item_id: Type.Optional(Type.String({ description: "Optional bead/work item id to anchor the SilentSession." })),
      lowmem: Type.Optional(Type.Boolean({ description: "Activate LowMem at start; default true." })),
      lines: Type.Optional(Type.Number({ description: "Tail lines for capture-pane; default 80, max 400." })),
      approved: Type.Optional(Type.Boolean({ description: "Required true for start/send/kill because those mutate background process state." })),
      force: Type.Optional(Type.Boolean({ description: "Required true with approved=true to kill a SilentSession." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const action = String(p.action || "list") as SilentSessionAction;
      const sessionsBefore = listSilentSessions();
      const sessionName = silentSessionName(p.session_name || p.work_item_id || "default");
      const hasSession = sessionsBefore.some((session: any) => session.name === sessionName);

      if (action === "list") {
        const text = sessionsBefore.length
          ? `silent sessions → ${sessionsBefore.map((s: any) => `${s.name}${s.attached ? "(attached)" : ""}`).join(", ")}`
          : "silent sessions → none";
        return { content: [{ type: "text", text }], details: { ok: true, status: "completed", sessions: sessionsBefore, count: sessionsBefore.length, registry: silentSessionRegistrySnapshot(sessionsBefore), tmux_version: silentSessionVersion(), next_tools: ["focusa_silent_sessions", "focusa_resource_mode", "focusa_work_loop_status"] } } as any;
      }

      if (action === "reopen") {
        if (!hasSession) return silentSessionBlocked(action, sessionName, "frame_unavailable", "no tmux SilentSession with that normalized name exists; list sessions or start it first", sessionsBefore);
        const runAsUser = silentSessionRunAsFor(sessionName);
        const tail = silentSessionExec(["capture-pane", "-p", "-J", "-t", sessionName, "-S", "-80"], 3000, runAsUser);
        return { content: [{ type: "text", text: `silent session reopen → ${sessionName}\nattach: ${silentSessionAttachCommand(sessionName)}\ndetach others: ${silentSessionAttachCommand(sessionName, true)}` }], details: { ok: true, status: "completed", session_name: sessionName, attach_command: silentSessionAttachCommand(sessionName), attach_detach_others_command: silentSessionAttachCommand(sessionName, true), tail_command: silentSessionTailCommand(sessionName), tail: tail.stdout.slice(-4000), sessions: sessionsBefore, registry_metadata: silentSessionReadMeta(sessionName), registry: silentSessionRegistrySnapshot(sessionsBefore), tmux_version: silentSessionVersion() } } as any;
      }

      if (action === "tail") {
        if (!hasSession) return silentSessionBlocked(action, sessionName, "frame_unavailable", "no tmux SilentSession with that normalized name exists; list sessions or start it first", sessionsBefore);
        const lines = Math.max(1, Math.min(400, Number(p.lines || 80)));
        const runAsUser = silentSessionRunAsFor(sessionName);
        const meta = silentSessionReadMeta(sessionName) || {};
        const tail = silentSessionExec(["capture-pane", "-p", "-J", "-t", sessionName, "-S", `-${lines}`], 3000, runAsUser);
        return { content: [{ type: "text", text: tail.ok ? `silent session tail → ${sessionName}\n${tail.stdout.slice(-4000)}` : `silent session tail blocked → ${tail.stderr}` }], details: { ok: tail.ok, status: tail.ok ? "completed" : "blocked", session_name: sessionName, tail: tail.stdout, tail_command: silentSessionTailCommand(sessionName, lines), log_path: meta.log_path || silentSessionLogPath(sessionName, runAsUser), run_as_user: runAsUser || currentUserName(), registry_metadata: meta, error: tail.stderr, tmux_version: silentSessionVersion() } } as any;
      }

      if (action === "health") {
        if (!hasSession) return silentSessionBlocked(action, sessionName, "frame_unavailable", "no tmux SilentSession with that normalized name exists; list sessions or start it first", sessionsBefore);
        const sessionInfo = sessionsBefore.find((session: any) => session.name === sessionName);
        const health = silentSessionPaneHealth(sessionName, sessionInfo);
        const status = health.status || "unknown";
        const activePane = health.panes?.find?.((pane: any) => pane.active) || health.panes?.[0] || null;
        const text = health.ok
          ? `silent session health → ${sessionName}: ${status}${activePane?.current_command ? ` (${activePane.current_command})` : ""}`
          : `silent session health blocked → ${sessionName}: ${health.error || "unknown"}`;
        return { content: [{ type: "text", text }], details: { ok: health.ok, status: health.ok ? "completed" : "blocked", session_name: sessionName, health_status: status, panes: health.panes || [], active_pane: activePane, log_path: health.log_path || silentSessionLogPath(sessionName, silentSessionRunAsFor(sessionName)), log_stats: health.log_stats, activity_age_seconds: health.activity_age_seconds, stale_after_seconds: health.stale_after_seconds, run_as_user: health.run_as_user || silentSessionRunAsFor(sessionName) || currentUserName(), registry_metadata: silentSessionReadMeta(sessionName), error: health.error, tmux_version: silentSessionVersion(), evidence_capture_suggestion: focusaEvidenceCaptureSuggestion({ target_ref: `silent_session:${sessionName}`, result: `SilentSession health ${status}`, evidence_ref: `tmux:${sessionName}:health`, attach_to_workpoint: false }), next_tools: status === "dead" || status === "stale" ? ["focusa_silent_sessions", "focusa_work_loop_status"] : ["focusa_silent_sessions"] } } as any;
      }

      if (action === "start" || action === "restart") {
        if (p.approved !== true) return silentSessionBlocked(action, sessionName, "approval_required", `${action} mutates background process state; pass approved=true only when operator explicitly wants a background session`, sessionsBefore);
        if (action === "start" && hasSession) return { content: [{ type: "text", text: `silent session already exists → ${sessionName}` }], details: { ok: true, status: "no_op", session_name: sessionName, attach_command: silentSessionAttachCommand(sessionName), attach_detach_others_command: silentSessionAttachCommand(sessionName, true), sessions: sessionsBefore } } as any;
        if (action === "restart" && hasSession) {
          const killed = silentSessionExec(["kill-session", "-t", sessionName], 3000, silentSessionRunAsFor(sessionName));
          if (!killed.ok) return silentSessionBlocked(action, sessionName, "process_control_failed", `tmux restart kill phase failed: ${killed.stderr || "unknown error"}`, sessionsBefore, { error: killed.stderr });
        }
        const priorMeta = action === "restart" ? (silentSessionReadMeta(sessionName) || {}) : {};
        const rootDir = String(p.root_dir || priorMeta.root_dir || S.sessionCwd || process.cwd());
        const cmd = String(p.command || priorMeta.command || defaultSilentSessionCommand({ ...priorMeta, ...p, root_dir: rootDir }, sessionName));
        const owner = silentSessionRootOwner(rootDir);
        const current = currentUserName();
        const runAsUser = owner.ok && owner.user && owner.user !== "root" && current === "root" ? owner.user : (priorMeta.run_as_user || current);
        const permissionPosture = runAsUser !== current ? "project_owner_via_as_user" : "current_user_tmux";
        const started = silentSessionExec(["new-session", "-d", "-s", sessionName, "-n", "agent", "-c", rootDir, "--", "bash", "-lc", cmd], 5000, runAsUser);
        if (started.ok) {
          silentSessionExec(["set-option", "-t", sessionName, "history-limit", "50000"], 3000, runAsUser);
          silentSessionExec(["set-window-option", "-t", sessionName, "remain-on-exit", "on"], 3000, runAsUser);
        }
        const pipeLog = started.ok ? silentSessionEnablePipeLog(sessionName, runAsUser) : { ok: false, log_path: silentSessionLogPath(sessionName, runAsUser), rotated: false, max_bytes: SILENT_SESSION_LOG_MAX_BYTES, backups: SILENT_SESSION_LOG_BACKUPS, error: started.stderr };
        if (started.ok) silentSessionWriteMeta(sessionName, { session_name: sessionName, root_dir: rootDir, root_owner: owner, run_as_user: runAsUser, permission_posture: permissionPosture, command: cmd, mission: p.mission || priorMeta.mission || null, work_item_id: p.work_item_id || priorMeta.work_item_id || null, lowmem: p.lowmem ?? priorMeta.lowmem ?? true, log_path: pipeLog.log_path, log_max_bytes: pipeLog.max_bytes, log_backups: pipeLog.backups, created_at: priorMeta.created_at || new Date().toISOString(), restarted_at: action === "restart" ? new Date().toISOString() : undefined });
        const sessionsAfter = listSilentSessions();
        const verb = action === "restart" ? "restarted" : "started";
        return { content: [{ type: "text", text: started.ok ? `silent session ${verb} → ${sessionName}\nattach: ${silentSessionAttachCommand(sessionName)}\ndetach others: ${silentSessionAttachCommand(sessionName, true)}` : `silent session ${action} blocked → ${started.stderr}` }], details: { ok: started.ok, status: started.ok ? "accepted" : "blocked", session_name: sessionName, attach_command: silentSessionAttachCommand(sessionName), attach_detach_others_command: silentSessionAttachCommand(sessionName, true), tail_command: silentSessionTailCommand(sessionName), command: cmd, window_name: "agent", root_dir: rootDir, root_owner: owner, run_as_user: runAsUser, permission_posture: permissionPosture, ownership_warning: runAsUser === "root" && rootDir.startsWith("/home/") ? "root-run session in /home may create root-owned files" : null, side_effects: started.ok ? [action === "restart" && hasSession ? "tmux_kill_session" : null, "tmux_new_session", "tmux_set_history_limit", "tmux_set_remain_on_exit", pipeLog.ok ? "tmux_pipe_pane_log" : null, "silent_session_registry_update"].filter(Boolean) : [], sessions: sessionsAfter, registry_metadata: silentSessionReadMeta(sessionName), registry: silentSessionRegistrySnapshot(sessionsAfter), error: started.stderr || pipeLog.error, log_path: pipeLog.log_path, pipe_log_ok: pipeLog.ok, log_rotated: pipeLog.rotated, log_max_bytes: pipeLog.max_bytes, log_backups: pipeLog.backups, tmux_version: silentSessionVersion(), evidence_capture_suggestion: started.ok ? focusaEvidenceCaptureSuggestion({ target_ref: `silent_session:${sessionName}`, result: `SilentSession ${verb} in ${rootDir} as ${runAsUser}`, evidence_ref: `tmux:${sessionName}:${action}`, project_root: rootDir, attach_to_workpoint: false }) : undefined } } as any;
      }

      if (action === "interrupt") {
        if (p.approved !== true) return silentSessionBlocked(action, sessionName, "approval_required", "interrupt sends C-c to a background process; pass approved=true only for explicit operator interruption", sessionsBefore);
        if (!hasSession) return silentSessionBlocked(action, sessionName, "frame_unavailable", "no tmux SilentSession with that normalized name exists; list sessions or start it first", sessionsBefore);
        const interrupted = silentSessionExec(["send-keys", "-t", sessionName, "C-c"], 3000, silentSessionRunAsFor(sessionName));
        if (!interrupted.ok) return silentSessionBlocked(action, sessionName, "process_control_failed", `tmux interrupt failed: ${interrupted.stderr || "unknown error"}`, sessionsBefore, { error: interrupted.stderr });
        return { content: [{ type: "text", text: `silent session interrupted → ${sessionName}` }], details: { ok: true, status: "accepted", session_name: sessionName, side_effects: ["tmux_send_interrupt"], next_tools: ["focusa_silent_sessions"] } } as any;
      }

      if (action === "send") {
        if (p.approved !== true) return silentSessionBlocked(action, sessionName, "approval_required", "send mutates a background process; pass approved=true only for explicit operator input", sessionsBefore);
        if (!hasSession) return silentSessionBlocked(action, sessionName, "frame_unavailable", "no tmux SilentSession with that normalized name exists; list sessions or start it first", sessionsBefore);
        const line = String(p.command || "").trim();
        if (!line) return silentSessionBlocked(action, sessionName, "validation_rejected", "send requires command/input text; provide command or use tail/list instead", sessionsBefore);
        const runAsUser = silentSessionRunAsFor(sessionName);
        const sentLiteral = silentSessionExec(["send-keys", "-l", "-t", sessionName, "--", line], 3000, runAsUser);
        const sentEnter = sentLiteral.ok ? silentSessionExec(["send-keys", "-t", sessionName, "C-m"], 3000, runAsUser) : sentLiteral;
        if (!sentLiteral.ok || !sentEnter.ok) return silentSessionBlocked(action, sessionName, "process_control_failed", `tmux send-keys failed: ${sentLiteral.stderr || sentEnter.stderr || "unknown error"}`, sessionsBefore, { error: sentLiteral.stderr || sentEnter.stderr });
        return { content: [{ type: "text", text: `silent session sent → ${sessionName}` }], details: { ok: true, status: "accepted", session_name: sessionName, sent_literal: true, side_effects: ["tmux_send_keys_literal", "tmux_send_enter"] } } as any;
      }

      if (action === "kill") {
        if (p.approved !== true || p.force !== true) return silentSessionBlocked(action, sessionName, "approval_required", "kill is destructive to background work; pass approved=true and force=true only with explicit operator approval", sessionsBefore);
        if (!hasSession) return { content: [{ type: "text", text: `silent session not found → ${sessionName}` }], details: { ok: true, status: "no_op", session_name: sessionName, sessions: sessionsBefore } } as any;
        const killed = silentSessionExec(["kill-session", "-t", sessionName], 3000, silentSessionRunAsFor(sessionName));
        if (killed.ok) silentSessionWriteMeta(sessionName, { ...(silentSessionReadMeta(sessionName) || {}), session_name: sessionName, killed_at: new Date().toISOString(), last_status: "killed" });
        const sessionsAfter = listSilentSessions();
        if (!killed.ok) return silentSessionBlocked(action, sessionName, "process_control_failed", `tmux kill-session failed: ${killed.stderr || "unknown error"}`, sessionsAfter, { error: killed.stderr });
        return { content: [{ type: "text", text: `silent session killed → ${sessionName}` }], details: { ok: true, status: "completed", session_name: sessionName, side_effects: ["tmux_kill_session", "silent_session_registry_update"], sessions: sessionsAfter, registry_metadata: silentSessionReadMeta(sessionName), registry: silentSessionRegistrySnapshot(sessionsAfter) } } as any;
      }

      return silentSessionBlocked(action, sessionName, "validation_rejected", `unsupported action ${action}; use list/start/reopen/tail/health/send/interrupt/restart/kill`, sessionsBefore);
    },
  });

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
      const resource = await focusaFetchDetailed("/resource/mode", { method: "GET" });
      const workpoint = await focusaFetchDetailed("/workpoint/current", { method: "GET" });
      const loop = await focusaFetchDetailed("/work-loop/status?summary_only=true", { method: "GET" });
      const liveContracts = await focusaFetchDetailed("/ontology/tool-contracts", { method: "GET" });
      const uiaiBrowser = await uiaiBrowserHealthCard();
      const ready = health.ok && workpoint.ok;
      const contractSummary = focusaToolContractSummary();
      const scopedContracts = String(p.scope || "all") === "all"
        ? FOCUSA_TOOL_CONTRACTS
        : FOCUSA_TOOL_CONTRACTS.filter((contract) => contract.family === String(p.scope || "") || contract.name.includes(String(p.scope || "")));
      const missingDocs = scopedContracts.filter((contract) => !contract.doc_path).map((contract) => contract.name);
      const knownExemptions = scopedContracts.filter((contract) => contract.exemptions.length > 0).map((contract) => ({ name: contract.name, exemptions: contract.exemptions }));
      const liveContractList = Array.isArray(liveContracts.body?.contracts) ? liveContracts.body.contracts : [];
      const liveNames = new Set(liveContractList.map((contract: any) => String(contract.name || "")).filter(Boolean));
      const staticNames = new Set(FOCUSA_TOOL_CONTRACTS.map((contract) => contract.name));
      const missing_live = FOCUSA_TOOL_CONTRACTS.map((contract) => contract.name).filter((name) => !liveNames.has(name));
      const extra_live = liveContractList.map((contract: any) => String(contract.name || "")).filter((name: string) => name && !staticNames.has(name));
      const stale_live_contracts = scopedContracts.filter((contract) => {
        const live = liveContractList.find((item: any) => item?.name === contract.name);
        return live && stableJson(live) !== stableJson(contract);
      }).map((contract) => contract.name);
      const repairProjectRoot = S.lastProjectRootResolution?.projectRoot || resolvePiProjectRoot(S.sessionCwd || process.cwd());
      const portableDaemonRestart =
        "if command -v focusa-daemon >/dev/null 2>&1; then nohup focusa-daemon >/tmp/focusa-daemon.log 2>&1 & elif command -v systemctl >/dev/null 2>&1; then systemctl restart focusa-daemon; else echo 'start focusa-daemon manually from this checkout' >&2; fi";
      const contractDrift = {
        live_ok: liveContracts.ok,
        static_count: FOCUSA_TOOL_CONTRACTS.length,
        live_count: liveContractList.length,
        version: liveContracts.body?.version || null,
        missing_live,
        extra_live,
        stale_live_contracts,
        drift_detected: !liveContracts.ok || missing_live.length > 0 || extra_live.length > 0 || stale_live_contracts.length > 0,
        repair_commands: [
          `cd ${repairProjectRoot}`,
          "cargo build --release --bins",
          portableDaemonRestart,
          "curl -sS --max-time 5 http://127.0.0.1:8787/v1/ontology/tool-contracts | jq '.version, (.contracts|length)'",
          "node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures",
        ],
      };
      const hookCounts = S.spec92HookTelemetry.reduce((acc: Record<string, number>, item: any) => {
        const hook = String(item.hook || "unknown");
        acc[hook] = (acc[hook] || 0) + 1;
        return acc;
      }, {});
      const latestToken = S.spec92TokenTelemetry.at(-1) || null;
      const latestTokenTurn = String((latestToken as any)?.turn_id || "");
      const currentTurnId = `pi-turn-${S.turnCount}`;
      const latestTokenIsCurrent = latestTokenTurn === currentTurnId || !latestTokenTurn;
      const latestTokenBudgetClass = String((latestToken as any)?.budget_class || "unknown");
      const tokenBudgetStatus = latestTokenIsCurrent ? latestTokenBudgetClass : `historical:${latestTokenBudgetClass}`;
      const resourceMode = resource.body?.resource_mode || {};
      const latestTransition = resourceMode.latest_transition || (Array.isArray(resource.body?.transition_history) ? resource.body.transition_history[0] : null);
      const transitionLabel = latestTransition ? `${String(latestTransition.from_mode || "?")}→${String(latestTransition.to_mode || "?")}` : "none";
      const sessionResolution = S.lastProjectRootResolution;
      const sessionRoot = sessionResolution?.projectRoot || resolvePiProjectRoot(S.sessionCwd || process.cwd());
      const sessionScopeSafe = isProjectRootAuthoritySafe(sessionRoot);
      const projectRootNeedsConfirmation = sessionResolution?.requiresOperatorConfirmation === true;
      const workpointStatus = String(workpoint.body?.status || (workpoint.ok ? "ok" : "blocked"));
      const workpointCanonical = workpoint.body?.canonical === true || workpointStatus === "active";
      const recommendations: string[] = [];
      if (!health.ok) recommendations.push("Focusa daemon health is blocked; retry hot status or inspect daemon before state writes.");
      if (!sessionScopeSafe) recommendations.push("Session cwd is broad/unsafe; cd to the project folder or pass explicit project_root to project-aware tools.");
      if (projectRootNeedsConfirmation) recommendations.push("REQUIRED FIRST: project root confidence is below 90%; use interview/menu to ask the operator which candidate root is correct before Focusa writes.");
      if (sessionScopeSafe && !projectRootNeedsConfirmation) recommendations.push("REQUIRED NEXT: run focusa_trajectory_view to confirm current functional state, destination, and waypoints before Workpoint/evidence progress tracking.");
      if (String(resourceMode.mode || "") === "emergency") recommendations.push("Resource mode is emergency; avoid cold/full-payload routes and use focusa_resource_mode for recovery posture.");
      if (!uiaiBrowser.ok) recommendations.push("UIAI browser health/metrics unavailable; browser diagnostics may be stale or unreachable.");
      if (uiaiBrowser.pressure === "high") recommendations.push("UIAI browser queue pressure is high; narrow browser workload, close stale sessions, or retry after queue drains.");
      if (!workpoint.ok || !workpointCanonical) recommendations.push("No canonical active Workpoint is visible; run focusa_project_identity then focusa_workpoint_checkpoint/resume before evidence or Focus State writes.");
      if (missingDocs.length) recommendations.push("Some project-aware tool contracts lack docs; run docs maintenance before release proof.");
      if (contractDrift.drift_detected) recommendations.push("Tool contract drift detected between Pi static registry and live daemon; rebuild/restart focusa-daemon, then run live contract proof.");
      const nextTools = Array.from(new Set([
        ...(!health.ok ? ["focusa_tool_doctor"] : []),
        ...(!sessionScopeSafe || projectRootNeedsConfirmation ? ["focusa_project_identity", "interview", "focusa_trajectory_view"] : ["focusa_trajectory_view"]),
        ...(String(resourceMode.mode || "") === "emergency" || uiaiBrowser.pressure === "high" ? ["focusa_resource_mode"] : []),
        ...(!workpoint.ok || !workpointCanonical ? ["focusa_project_identity", "focusa_workpoint_checkpoint", "focusa_workpoint_resume"] : []),
        ...(contractDrift.drift_detected ? ["focusa_tool_doctor"] : []),
      ]));
      const recommendedAction = recommendations[0] || "Proceed with explicit project_root for project-aware tools and checkpoint before compaction.";
      const driftCauseCounts = {
        missing_live: contractDrift.missing_live.length,
        extra_live: contractDrift.extra_live.length,
        stale_live_contracts: contractDrift.stale_live_contracts.length,
      };
      const driftSummary = contractDrift.drift_detected
        ? ` drift=yes drift_causes=missing_live:${driftCauseCounts.missing_live},extra_live:${driftCauseCounts.extra_live},stale_live_contracts:${driftCauseCounts.stale_live_contracts} source_refs=static:apps/pi-extension/src/tools.ts,live:/v1/ontology/tool-contracts`
        : "";
      const driftDetails = contractDrift.drift_detected
        ? { drift_detected: true, cause_counts: driftCauseCounts, source_refs: ["static:apps/pi-extension/src/tools.ts", "live:/v1/ontology/tool-contracts"], missing_live: contractDrift.missing_live.slice(0, 6), extra_live: contractDrift.extra_live.slice(0, 6), stale_live_contracts: contractDrift.stale_live_contracts.slice(0, 6) }
        : { drift_detected: false };
      const evidenceResult = contractDrift.drift_detected
        ? `readiness=${ready ? "ready" : "degraded"} drift=yes causes=${JSON.stringify(driftCauseCounts)} uiai_browser=${uiaiBrowser.status}/${uiaiBrowser.pressure}`
        : `readiness=${ready ? "ready" : "degraded"} uiai_browser=${uiaiBrowser.status}/${uiaiBrowser.pressure}`;
      const text = `tool doctor → readiness=${ready ? "ready" : "degraded"} scope=${String(p.scope || "all")} contracts=${contractSummary.total} live_contracts=${contractDrift.live_ok ? contractDrift.live_count : "blocked"}${driftSummary} scoped=${scopedContracts.length} hooks=${S.spec92HookTelemetry.length} token_budget=${tokenBudgetStatus} resource=${String(resourceMode.mode || "unknown")}/${String(resourceMode.reason || "unknown")} transition=${transitionLabel} health=${health.ok ? "ok" : "blocked"} workpoint=${workpointStatus} work_loop=${loop.ok ? String(loop.body?.status || "ok") : "blocked"} uiai_browser=${uiaiBrowser.status}/${uiaiBrowser.pressure} recommended=${recommendedAction}`;
      return { content: [{ type: "text", text }], details: { ok: ready && !contractDrift.drift_detected, status: ready && !contractDrift.drift_detected ? "completed" : "degraded", health: compactApiEcho(health.body), resource_mode: compactApiEcho(resource.body), workpoint: compactApiEcho(workpoint.body), work_loop: compactApiEcho(loop.body), uiai_browser: compactApiEcho(uiaiBrowser), contracts_total: contractSummary.total, contracts_by_family: contractSummary.by_family, contract_coverage: { scoped: scopedContracts.length, missing_docs: missingDocs, known_exemptions: knownExemptions }, contract_drift: driftDetails, session_scope: { cwd: sessionRoot, safe: sessionScopeSafe, project_root_resolution: compactApiEcho(sessionResolution || null) }, token_budget: { status: tokenBudgetStatus, budget_class: latestTokenBudgetClass, turn_id: latestTokenTurn || null, current_turn_id: currentTurnId, current: latestTokenIsCurrent }, recommendations: recommendations.slice(0, 6), recommended_action: recommendedAction, evidence_capture_suggestion: focusaEvidenceCaptureSuggestion({ target_ref: "focusa_tool_doctor", result: evidenceResult, evidence_ref: `focusa_tool_doctor:${String(p.scope || "all")}`, project_root: sessionScopeSafe ? sessionRoot : undefined, attach_to_workpoint: false }), next_tools: nextTools.slice(0, 4), spec92: { hook_records: S.spec92HookTelemetry.length, token_records: S.spec92TokenTelemetry.length } } } as any;
    },
  });


  pi.registerTool({
    name: "focusa_resource_mode",
    label: "Focusa Resource Mode",
    description: "Read or control Focusa resource mode, including activating/deactivating LowMem mode when resources are constrained.",
    promptSnippet: "Use when resources are low, daemon hot paths risk timeouts, or operator says Activate/Deactivate LowMem mode.",
    parameters: Type.Object({
      action: Type.Optional(Type.Union([
        Type.Literal("status"),
        Type.Literal("activate_lowmem"),
        Type.Literal("deactivate_lowmem"),
        Type.Literal("set_mode"),
        Type.Literal("set_normal"),
        Type.Literal("set_constrained"),
        Type.Literal("set_emergency"),
      ], { description: "Mode action. activate_lowmem enables LowMem; deactivate_lowmem clears the runtime override back to auto." })),
      mode: Type.Optional(Type.Union([
        Type.Literal("auto"),
        Type.Literal("normal"),
        Type.Literal("constrained"),
        Type.Literal("lowmem"),
        Type.Literal("emergency"),
      ], { description: "Optional target mode when action=set_mode." })),
      reason: Type.Optional(Type.String({ description: "Why the mode is being read or changed." })),
      preflight: Type.Optional(Type.Boolean({ description: "If true, only read current mode and report intended change." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const action = String(p.action || "status");
      const preflight = p.preflight === true;
      if (action === "status" || preflight) {
        const result = await focusaFetchDetailed("/resource/mode", { method: "GET" });
        const body = result.body || {};
        const mode = body.resource_mode || {};
        const intended = preflight && action !== "status" ? ` intended_action=${action}` : "";
        const text = result.ok
          ? `resource mode → mode=${String(mode.mode || "unknown")} forced=${mode.forced === true} reason=${String(mode.reason || "unknown")}${intended}`
          : `resource mode blocked → ${explainWorkLoopResult(result, "resource mode unavailable")}`;
        return {
          content: [{ type: "text", text }],
          details: {
            ok: result.ok,
            status: result.ok ? "completed" : "blocked",
            endpoint: "/v1/resource/mode",
            resource_mode: mode,
            preflight,
            intended_action: preflight ? action : undefined,
            next_tools: ["focusa_tool_doctor", "focusa_trajectory_view", "focusa_workpoint_resume", "focusa_traverse"],
            response: compactApiEcho(body),
          },
        } as any;
      }
      const body = { action, mode: p.mode, reason: p.reason || `pi:${action}` };
      const result = await focusaFetchDetailed("/resource/mode", { method: "POST", body: JSON.stringify(body) });
      const response = result.body || {};
      const mode = response.resource_mode || {};
      const ok = result.ok && response.status !== "blocked";
      const text = ok
        ? `resource mode → action=${action} mode=${String(mode.mode || "unknown")} reason=${String(mode.reason || "unknown")}`
        : `resource mode blocked → ${String(response.summary || explainWorkLoopResult(result, "resource mode unavailable"))}`;
      return {
        content: [{ type: "text", text }],
        details: {
          ok,
          status: ok ? "completed" : "blocked",
          endpoint: "/v1/resource/mode",
          action,
          requested_mode: response.requested_mode || p.mode || null,
          resource_mode: mode,
          side_effects: response.side_effects || ["runtime_resource_mode_override"],
          next_tools: response.next_tools || ["focusa_tool_doctor", "focusa_trajectory_view", "focusa_workpoint_resume", "focusa_traverse"],
          failure_class: response.failure_class || null,
          response,
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_project_identity",
    label: "Focusa Project Identity",
    description: "Resolve bounded ProjectIdentity from cwd/project_root using marker, git, beads, workspace, daemon, and operator project signals.",
    promptSnippet: "Use before trusting cross-project Workpoints, Trajectory packets, or project-sensitive context.",
    parameters: Type.Object({
      cwd: Type.Optional(Type.String({ description: "Optional cwd/project path hint; defaults to Pi session cwd." })),
      project_root: Type.Optional(Type.String({ description: "Optional expected project root folder." })),
      remote_host: Type.Optional(Type.String({ description: "Remote SSH host that contains the project root; caller supplies inspected evidence." })),
      remote_user: Type.Optional(Type.String({ description: "Remote SSH user, if known." })),
      remote_port: Type.Optional(Type.Integer({ minimum: 1, maximum: 65535, description: "Remote SSH port, if known." })),
      remote_repo_remote: Type.Optional(Type.String({ description: "Git origin/repo remote observed on the remote host." })),
      remote_workspace_kind: Type.Optional(Type.String({ description: "Workspace kind observed on the remote host." })),
      remote_deploy_root: Type.Optional(Type.String({ description: "Deployment/site root observed on the remote host." })),
      persisted_project_root: Type.Optional(Type.String({ description: "Prior ProjectIdentity root from this Pi session; auto-filled when omitted." })),
      persisted_project_fingerprint: Type.Optional(Type.String({ description: "Prior ProjectIdentity fingerprint from this Pi session; auto-filled when omitted." })),
      persisted_project_id: Type.Optional(Type.String({ description: "Prior ProjectIdentity project id from this Pi session; auto-filled when omitted." })),
      persisted_canonical_name: Type.Optional(Type.String({ description: "Prior ProjectIdentity canonical name from this Pi session; auto-filled when omitted." })),
    }),
    async execute(_id, params) {
      const p = params as { cwd?: string; project_root?: string; remote_host?: string; remote_user?: string; remote_port?: number; remote_repo_remote?: string; remote_workspace_kind?: string; remote_deploy_root?: string; persisted_project_root?: string; persisted_project_fingerprint?: string; persisted_project_id?: string; persisted_canonical_name?: string };
      const query = new URLSearchParams();
      query.set("cwd", p.cwd || S.sessionCwd || process.cwd());
      if (p.project_root) query.set("project_root", p.project_root);
      if (p.remote_host) query.set("remote_host", p.remote_host);
      if (p.remote_user) query.set("remote_user", p.remote_user);
      if (p.remote_port) query.set("remote_port", String(p.remote_port));
      if (p.remote_repo_remote) query.set("remote_repo_remote", p.remote_repo_remote);
      if (p.remote_workspace_kind) query.set("remote_workspace_kind", p.remote_workspace_kind);
      if (p.remote_deploy_root) query.set("remote_deploy_root", p.remote_deploy_root);
      appendPersistedProjectIdentityQuery(query, p.project_root);
      if (p.persisted_project_root) query.set("persisted_project_root", p.persisted_project_root);
      if (p.persisted_project_fingerprint) query.set("persisted_project_fingerprint", p.persisted_project_fingerprint);
      if (p.persisted_project_id) query.set("persisted_project_id", p.persisted_project_id);
      if (p.persisted_canonical_name) query.set("persisted_canonical_name", p.persisted_canonical_name);
      const result = await focusaFetchDetailed(`/project/identity?${query.toString()}`, { method: "GET" });
      const body = result.body || {};
      if (!result.ok && body.failure_class === "hot_path_timeout") {
        const requestedRoot = normalizeProjectRoot(p.project_root || p.cwd || S.sessionCwd || process.cwd());
        const cachedIdentity = S.lastProjectIdentity && (!requestedRoot || normalizeProjectRoot(S.lastProjectIdentity.project_root) === requestedRoot) ? S.lastProjectIdentity : null;
        return { content: [{ type: "text", text: timeoutPreservedText("project identity", cachedIdentity ? "cached identity" : "empty fallback") }], details: { ok: false, status: "timeout_preserved", endpoint: "/v1/project/identity", canonical: false, degraded: true, advisory_only: true, project_identity: cachedIdentity || {}, failure_class: "hot_path_timeout", response: compactApiEcho(body), next_tools: ["focusa_tool_doctor", "focusa_resource_mode", "focusa_project_identity", "focusa_project_verify", "focusa_trajectory_view"] } } as any;
      }
      const identity = body.project_identity || {};
      if (identity && Object.keys(identity).length) {
        // Guard: do not overwrite a verified in-session project identity with a different project's result.
        // After model switch, the session may already hold a valid identity. Overwriting it
        // causes cross-session contamination (SPEC96 emergency fix 2 isolation principle).
        const incomingRoot = normalizeProjectRoot(identity.project_root);
        const existingRoot = normalizeProjectRoot(S.lastProjectIdentity?.project_root);
        const requestedRoot = normalizeProjectRoot(p.project_root);
        const explicitProjectSwitch = requestedRoot && incomingRoot === requestedRoot && existingRoot !== requestedRoot;
        const existingConfidence = S.lastProjectIdentity?.confidence;
        const isExistingVerified = existingConfidence === "high" || existingConfidence === "medium";
        const isDifferentProject = existingRoot && incomingRoot && existingRoot !== incomingRoot;
        const isDifferentThanUnverified = isDifferentProject && isExistingVerified && !explicitProjectSwitch;
        if (isDifferentThanUnverified) {
          // Preserve existing verified identity; return it instead of the incoming one.
          const preserved = S.lastProjectIdentity;
          return {
            content: [{ type: "text", text: `project identity → status=verified confidence=${preserved.confidence || "unknown"} root=${preserved.project_root || "unknown"} (preserved from session; incoming result rejected as different project: ${incomingRoot})` }],
            details: { ok: true, status: "preserved", endpoint: "/v1/project/identity", canonical: false, degraded: false, project_identity: preserved, project_summary: null, summary_lines: [], verification: null, tool_result_v1: { ok: true, status: "preserved", canonical: false, degraded: false, failure_class: null, retry: { safe: true, posture: "safe_retry" }, side_effects: [], evidence_refs: [], next_tools: ["focusa_project_verify", "focusa_trajectory_view", "focusa_workpoint_resume"] }, failure_class: null, next_tools: ["focusa_project_verify", "focusa_trajectory_view", "focusa_workpoint_resume"], response: compactApiEcho(body) },
          } as any;
        }
        S.lastProjectIdentity = identity;
        const verifiedRoot = normalizeProjectRoot(identity.project_root);
        if (verifiedRoot && identity.status === "verified" && isProjectRootAuthoritySafe(verifiedRoot)) {
          confirmPiProjectRoot(verifiedRoot, "focusa_project_identity_verified");
          ensureContinuityId(verifiedRoot);
          persistState();
        }
      }
      const summaryLines = Array.isArray(body.summary_lines)
        ? body.summary_lines.map((line: any) => String(line)).filter(Boolean)
        : Array.isArray(identity.project_summary?.summary_lines)
          ? identity.project_summary.summary_lines.map((line: any) => String(line)).filter(Boolean)
          : [];
      const text = result.ok
        ? [`project identity → status=${String(identity.status || body.status || "unknown")} confidence=${String(identity.confidence || "unknown")} root=${String(identity.project_root || "unknown")}`, ...summaryLines.slice(0, 4)].join("\n")
        : `project identity blocked → ${explainWorkLoopResult(result, "project identity unavailable")}`;
      const toolResult = body.details?.tool_result_v1 || { ok: result.ok, status: result.ok ? String(body.status || "completed") : "blocked", canonical: body.canonical === true, degraded: body.degraded !== false, failure_class: body.failure_class || null, retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" }, side_effects: [], evidence_refs: [], next_tools: body.next_tools || ["focusa_project_verify", "focusa_trajectory_view", "focusa_workpoint_resume"] };
      return {
        content: [{ type: "text", text }],
        details: {
          ok: result.ok,
          status: result.ok ? String(body.status || "completed") : "blocked",
          endpoint: "/v1/project/identity",
          canonical: body.canonical === true,
          degraded: body.degraded === true,
          project_identity: identity,
          project_summary: body.project_summary || identity.project_summary || null,
          summary_lines: summaryLines,
          verification: body.verification,
          tool_result_v1: toolResult,
          failure_class: toolResult.failure_class || body.failure_class || null,
          next_tools: toolResult.next_tools || body.next_tools || ["focusa_project_verify", "focusa_trajectory_view", "focusa_workpoint_resume"],
          response: compactApiEcho(body),
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_project_card",
    label: "Focusa Project Card",
    description: "Build an advisory project-intelligence card from ProjectIdentity, ontology, trajectory, Workpoint/evidence, prediction, and metacog signals.",
    promptSnippet: "Use at bootstrap/re-bootstrap, project reviews, and next-step evaluation before refreshing trajectory hierarchy.",
    parameters: Type.Object({
      cwd: Type.Optional(Type.String({ description: "Optional cwd/project path hint; defaults to Pi session cwd." })),
      project_root: Type.Optional(Type.String({ description: "Optional expected project root folder." })),
      current_ask: Type.Optional(Type.String({ description: "Optional current ask used to seed bootstrap/re-bootstrap candidate." })),
      remote_host: Type.Optional(Type.String({ description: "Remote SSH host that contains the project root; caller supplies inspected evidence." })),
      remote_user: Type.Optional(Type.String({ description: "Remote SSH user, if known." })),
      remote_port: Type.Optional(Type.Number({ minimum: 1, maximum: 65535, description: "Remote SSH port, if known." })),
      remote_repo_remote: Type.Optional(Type.String({ description: "Git origin/repo remote observed on the remote host." })),
      remote_workspace_kind: Type.Optional(Type.String({ description: "Workspace kind observed on the remote host." })),
      remote_deploy_root: Type.Optional(Type.String({ description: "Deployment/site root observed on the remote host." })),
    }),
    async execute(_id, params) {
      const p = params as { cwd?: string; project_root?: string; current_ask?: string; remote_host?: string; remote_user?: string; remote_port?: number; remote_repo_remote?: string; remote_workspace_kind?: string; remote_deploy_root?: string };
      const query = new URLSearchParams();
      query.set("cwd", p.cwd || S.sessionCwd || process.cwd());
      if (p.project_root) query.set("project_root", p.project_root);
      if (p.current_ask) query.set("current_ask", p.current_ask);
      if (p.remote_host) query.set("remote_host", p.remote_host);
      if (p.remote_user) query.set("remote_user", p.remote_user);
      if (Number.isFinite(p.remote_port)) query.set("remote_port", String(Math.trunc(Number(p.remote_port))));
      if (p.remote_repo_remote) query.set("remote_repo_remote", p.remote_repo_remote);
      if (p.remote_workspace_kind) query.set("remote_workspace_kind", p.remote_workspace_kind);
      if (p.remote_deploy_root) query.set("remote_deploy_root", p.remote_deploy_root);
      appendPersistedProjectIdentityQuery(query, p.project_root);
      const result = await focusaFetchDetailed(`/project/card?${query.toString()}`, { method: "GET" });
      const body = result.body || {};
      const project = body.project_identity || {};
      const bootstrap = body.bootstrap || {};
      const prediction = body.prediction || {};
      const ontology = body.ontology || {};
      const prior = body.prior_session_context || {};
      const priorLadder = prior.trajectory_ladder || {};
      const priorDecisionCount = Array.isArray(prior.recent_decisions) ? prior.recent_decisions.length : 0;
      const priorOutcomeCount = Array.isArray(prior.recent_algorithm_outcomes) ? prior.recent_algorithm_outcomes.length : 0;
      const sequence = body.success_sequence || {};
      const efficiency = body.efficiency_summary || body.trajectory_report_card?.time_and_tokens || {};
      const trajectoryReport = body.trajectory_report_card || {};
      const crosswire = body.crosswire_health || {};
      const inferredWorkpoint = body.inferred_workpoint_candidate || bootstrap.candidate?.inferred_workpoint_candidate || {};
      const askToWorkpointBridge = body.ask_to_workpoint_bridge || inferredWorkpoint.ask_to_workpoint_bridge || {};
      const waypointSummary = trajectoryReport.accomplishment_summary || {};
      const shortest = sequence.shortest_path_to_success || {};
      const selectedPath = shortest.selected || {};
      const eliminatedCount = Array.isArray(shortest.eliminated_candidates) ? shortest.eliminated_candidates.length : 0;
      const ontologyCounts = ontology.counts || {};
      const text = result.ok
        ? `project card → project=${String(project.canonical_name || project.project_id || "unknown")} root=${String(project.project_root || "unknown")} bootstrap_needed=${bootstrap.needed === true} hlg=${String(priorLadder.high_level_goal || body.trajectory?.hlt || "missing").slice(0, 80)} stg=${String(priorLadder.short_term_goal || body.trajectory?.stg || "missing").slice(0, 80)} decisions=${priorDecisionCount} outcomes=${priorOutcomeCount} elapsed_avg=${String(efficiency.average_elapsed_hms || "00:00:00")} tokens_avg=${String(efficiency.average_total_tokens ?? 0)} waypoints=${String(waypointSummary.waypoints_accomplished_by_recent_outcomes ?? 0)}/${String(waypointSummary.waypoints_total ?? 0)} crosswire=${String(crosswire.prediction_feed?.elapsed_tokens_waypoints_feed_future_predictions === true ? "ok" : "check")} inferred_wp=${String(inferredWorkpoint.current_action || "none")} ask_bridge=${String(askToWorkpointBridge.recommended_bridge_action || "unknown")} exact_next=${String(askToWorkpointBridge.exact_next_action || inferredWorkpoint.next_action || "unknown").slice(0, 80)} next_event=${String(sequence.recommended_first_event || "unknown")} shortest=${String(selectedPath.path_id || "unknown")} cost=${String(selectedPath.cost ?? "unknown")} eliminated=${eliminatedCount} predictions=${String(prediction.total ?? "unknown")}/${String(prediction.evaluated ?? "unknown")} ontology_runtime=${String(ontologyCounts.runtime_objects ?? ontology.runtime_objects ?? "unknown")} ontology_effective=${String(ontologyCounts.effective_project_card_objects ?? ontology.objects ?? "unknown")} ontology_source=${String(ontology.source_index || "unknown")} selector=${String(ontology.selector || "unknown")}`
        : `project card blocked → ${explainWorkLoopResult(result, "project card unavailable")}`;
      const toolResult = body.details?.tool_result_v1 || { ok: result.ok, status: result.ok ? String(body.status || "completed") : "blocked", canonical: false, degraded: !result.ok, failure_class: body.failure_class || null, retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" }, side_effects: [], evidence_refs: [], next_tools: body.next_tools || ["focusa_project_card_outcome", "focusa_traverse", "focusa_trajectory_view", "focusa_metacog_retrieve"] };
      const compactResponse = { status: body.status, schema: body.schema, algorithm_run_id: body.algorithm_run_id, bootstrap_needed: bootstrap.needed, inferred_workpoint_candidate: inferredWorkpoint, ask_to_workpoint_bridge: { ask_differs_from_active_workpoint: askToWorkpointBridge.ask_differs_from_active_workpoint, recommended_bridge_action: askToWorkpointBridge.recommended_bridge_action, exact_next_action: askToWorkpointBridge.exact_next_action, checkpoint_payload_hint: askToWorkpointBridge.checkpoint_payload_hint, safe_after_identity_verification: askToWorkpointBridge.safe_after_identity_verification }, trajectory_report_card: trajectoryReport, efficiency_summary: efficiency, crosswire_health: crosswire, recommended_first_event: sequence.recommended_first_event, ranking_basis: sequence.ranking_basis, shortest_path_to_success: { selected: selectedPath, eliminated_candidates: shortest.eliminated_candidates || [] }, outcome_learning: body.algorithmic_intelligence?.outcome_learning, next_tools: body.next_tools };
      return { content: [{ type: "text", text }], details: { ok: result.ok, status: String(body.status || (result.ok ? "completed" : "blocked")), endpoint: "/v1/project/card", advisory_only: body.advisory_only !== false, project_identity: project, trajectory: body.trajectory || null, inferred_workpoint_candidate: inferredWorkpoint, ask_to_workpoint_bridge: askToWorkpointBridge, trajectory_report_card: trajectoryReport, efficiency_summary: efficiency, crosswire_health: crosswire, prior_session_context: prior, success_sequence: sequence, ontology, evidence: body.evidence || null, prediction, algorithmic_intelligence: { outcome_learning: body.algorithmic_intelligence?.outcome_learning || null, expected_utility: body.algorithmic_intelligence?.expected_utility || null }, metacognition: body.metacognition || null, active_workpoint: body.active_workpoint || null, bootstrap, possibilities: body.possibilities || [], next_step_quality_rule: body.next_step_quality_rule || null, tool_result_v1: toolResult, next_tools: toolResult.next_tools || body.next_tools || ["focusa_workpoint_checkpoint", "focusa_project_card_outcome", "focusa_traverse", "focusa_trajectory_view"], response: compactResponse } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_project_card_outcome",
    label: "Focusa Project Card Outcome",
    description: "Attach a final outcome/result to a specific project-card algorithm_run_id and update learned project-card weights.",
    promptSnippet: "Use after a project-card-guided action is verified, so future bootstrap/sequence planning learns from the result.",
    parameters: Type.Object({
      algorithm_run_id: Type.String({ description: "Project-card algorithm_run_id returned by focusa_project_card." }),
      actual_outcome: Type.String({ description: "Observed final outcome/result for that algorithm run." }),
      score: Type.Optional(Type.Number({ description: "Optional outcome score from 0.0 to 1.0; defaults to 1.0." })),
      evidence_refs: Type.Optional(Type.Array(Type.String(), { description: "Evidence refs proving the outcome." })),
      project_root: Type.Optional(Type.String({ description: "Optional project root associated with the run." })),
      notes: Type.Optional(Type.String({ description: "Optional bounded note about the result." })),
      task_timing: Type.Optional(Type.Any({ description: "Optional override timing object; Pi auto-populates elapsed task timing when omitted." })),
      token_usage: Type.Optional(Type.Any({ description: "Optional override token usage object; Pi auto-populates provider/estimated token counts when omitted." })),
    }),
    async execute(_id, params) {
      const p = params as { algorithm_run_id: string; actual_outcome: string; score?: number; evidence_refs?: string[]; project_root?: string; notes?: string; task_timing?: any; token_usage?: any };
      const autoAccounting = currentTaskTimingAndTokens();
      const payload = {
        algorithm_run_id: p.algorithm_run_id,
        actual_outcome: p.actual_outcome,
        score: typeof p.score === "number" ? p.score : undefined,
        evidence_refs: Array.isArray(p.evidence_refs) ? p.evidence_refs : [],
        project_root: p.project_root || S.sessionCwd || process.cwd(),
        notes: p.notes,
        task_timing: p.task_timing || autoAccounting.task_timing,
        token_usage: p.token_usage || autoAccounting.token_usage,
      };
      const result = await focusaFetchDetailed("/project/card/outcome", { method: "POST", body: JSON.stringify(payload) });
      const body = result.body || {};
      const outcome = body.outcome || {};
      const text = result.ok && String(body.status || "") === "recorded"
        ? `project card outcome → recorded run=${String(outcome.algorithm_run_id || p.algorithm_run_id)} score=${String(outcome.score ?? payload.score ?? "default")} elapsed=${String(outcome.task_timing?.elapsed_hms || payload.task_timing.elapsed_hms)} tokens=${String(outcome.token_usage?.total_tokens ?? payload.token_usage.total_tokens)} evidence=${Array.isArray(outcome.evidence_refs) ? outcome.evidence_refs.length : payload.evidence_refs.length}`
        : `project card outcome blocked → ${explainWorkLoopResult(result, "outcome unavailable")}`;
      const toolResult = body.details?.tool_result_v1 || { ok: result.ok && body.status === "recorded", status: result.ok ? String(body.status || "completed") : "blocked", canonical: false, degraded: !result.ok, failure_class: body.failure_class || null, retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" }, side_effects: ["project_card_algorithm_outcome_append", "project_card_weight_update"], evidence_refs: payload.evidence_refs, next_tools: body.flywheel?.next_tools || ["focusa_project_card", "focusa_predict_record", "focusa_metacog_capture"] };
      return { content: [{ type: "text", text }], details: { ok: toolResult.ok, status: String(body.status || (result.ok ? "completed" : "blocked")), endpoint: "/v1/project/card/outcome", advisory_only: false, outcome, storage: body.storage || null, flywheel: body.flywheel || null, tool_result_v1: toolResult, failure_class: toolResult.failure_class || null, side_effects: toolResult.side_effects || [], evidence_refs: toolResult.evidence_refs || [], request: compactApiEcho(payload), response: compactApiEcho(body), next_tools: toolResult.next_tools || body.flywheel?.next_tools || ["focusa_project_card", "focusa_predict_record", "focusa_metacog_capture"] } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_session_transfer",
    label: "Focusa Session Transfer",
    description: "Easy save/continue wrapper for moving long work between Pi sessions without forking: save a Workpoint packet or continue from project card + Workpoint + trajectory.",
    promptSnippet: "Use when operator wants to save or continue a long Focusa/Pi session like a game save.",
    parameters: Type.Object({
      action: Type.String({ description: "save|continue|status" }),
      project_root: Type.Optional(Type.String({ description: "Project root to transfer; defaults to Pi cwd/session cwd." })),
      current_ask: Type.Optional(Type.String({ description: "Current resume/save intent." })),
      mission: Type.Optional(Type.String({ description: "Optional save mission; defaults to current ask or inferred Workpoint mission." })),
      next_action: Type.Optional(Type.String({ description: "Optional exact next action for save." })),
      continuity_id: Type.Optional(Type.String({ description: "Optional logical continuity id; defaults to project continuity." })),
    }),
    async execute(_id, params) {
      const p = params as { action: string; project_root?: string; current_ask?: string; mission?: string; next_action?: string; continuity_id?: string };
      const action = String(p.action || "status").toLowerCase();
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root || S.sessionCwd || process.cwd());
      const continuityId = p.continuity_id || ensureContinuityId(projectRoot);
      const currentAsk = p.current_ask || S.currentAsk?.text || (action === "continue" ? "Continue latest saved Focusa work like a game save" : "Save current Focusa work for transfer");
      const cardQuery = new URLSearchParams();
      cardQuery.set("project_root", projectRoot);
      cardQuery.set("cwd", projectRoot);
      cardQuery.set("current_ask", currentAsk);
      const apiTransfer = await focusaFetchDetailed("/project/session-transfer", { method: "POST", body: JSON.stringify({ action, project_root: projectRoot, current_ask: currentAsk, continuity_id: continuityId, mission: p.mission, next_action: p.next_action }) });
      const apiBody = apiTransfer.body || {};
      const cardRes = await focusaFetchDetailed(`/project/card?${cardQuery.toString()}`, { method: "GET" });
      const card = cardRes.body || {};
      const inferred = apiBody.transfer?.inferred_workpoint_candidate || card.inferred_workpoint_candidate || card.bootstrap?.candidate?.inferred_workpoint_candidate || {};
      let checkpoint: any = null;
      let resume: any = null;
      let trajectory: any = null;
      if (action === "save") {
        const hint = inferred.checkpoint_payload_hint || {};
        const mission = p.mission || hint.mission || inferred.mission || currentAsk;
        const nextAction = p.next_action || hint.next_action || inferred.next_action || "Continue from saved Focusa session transfer packet";
        checkpoint = await focusaFetchDetailed("/workpoint/checkpoint", {
          method: "POST",
          body: JSON.stringify({
            mission,
            next_action: nextAction,
            next_slice: nextAction,
            current_action: hint.current_action || inferred.current_action || "session_transfer_save",
            action_type: hint.current_action || inferred.current_action || "session_transfer_save",
            target_objects: hint.target_objects || inferred.target_objects || [],
            active_object_refs: hint.target_objects || inferred.target_objects || [],
            project_root: projectRoot,
            continuity_id: continuityId,
            session_id: S.sessionFrameKey,
            source_turn_id: `pi-turn-${S.turnCount}`,
            canonical: true,
            checkpoint_reason: "session_transfer_save",
            idempotency_key: `session-transfer:${projectRoot}:${continuityId}:${Date.now()}`,
          }),
        });
      }
      if (action === "continue" || action === "status" || action === "save") {
        resume = await focusaFetchDetailed("/workpoint/resume", { method: "POST", body: JSON.stringify({ project_root: projectRoot, continuity_id: continuityId, session_id: S.sessionFrameKey, mode: "compact_prompt" }) });
        const tq = new URLSearchParams();
        tq.set("project_root", projectRoot);
        tq.set("continuity_id", continuityId);
        tq.set("allow_prior_project_trajectory", "true");
        trajectory = await focusaFetchDetailed(`/trajectory/view?${tq.toString()}`, { method: "GET" });
      }
      const ok = apiTransfer.ok && cardRes.ok && (action !== "save" || checkpoint?.ok) && (action === "save" || resume?.ok || card.inferred_workpoint_candidate || apiBody.transfer?.inferred_workpoint_candidate);
      const shortest = card.success_sequence?.shortest_path_to_success?.selected || {};
      const text = `session transfer ${action} → project=${String(card.project_identity?.canonical_name || card.project_identity?.project_id || projectRoot)} root=${projectRoot} saved=${checkpoint?.ok === true} resume=${String(resume?.body?.status || resume?.status || "not_run")} inferred_wp=${String(inferred.current_action || "none")} shortest=${String(shortest.path_id || "unknown")}`;
      const toolResult = card.details?.tool_result_v1 || { ok, status: ok ? "completed" : "blocked", canonical: resume?.body?.canonical === true || checkpoint?.body?.canonical === true, degraded: !ok, failure_class: ok ? null : (card.failure_class || resume?.body?.failure_class || checkpoint?.body?.failure_class || null), retry: { safe: true, posture: "safe_retry" }, side_effects: checkpoint?.ok ? ["workpoint_checkpoint"] : [], evidence_refs: [], next_tools: ["focusa_project_card", "focusa_workpoint_resume", "focusa_trajectory_view"] };
      return { content: [{ type: "text", text }], details: { ok, status: ok ? "completed" : "blocked", endpoint: "session_transfer_wrapper", action, project_root: projectRoot, continuity_id: continuityId, api_transfer: apiBody, save_packet: checkpoint?.body || null, resume_packet: resume?.body || null, trajectory: trajectory?.body || null, project_card: { algorithm_run_id: card.algorithm_run_id, inferred_workpoint_candidate: inferred, trajectory_report_card: card.trajectory_report_card, crosswire_health: card.crosswire_health, success_sequence: card.success_sequence }, operator_handoff: apiBody.transfer?.operator_handoff || { command: `cd ${projectRoot} && pi`, first_tool: `focusa_session_transfer action=\"continue\" project_root=\"${projectRoot}\" continuity_id=\"${continuityId}\"`, authority_boundary: "project_root_plus_continuity_id" }, tool_result_v1: toolResult, next_tools: ["focusa_workpoint_resume", "focusa_project_card", "focusa_trajectory_view"] } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_project_verify",
    label: "Focusa Project Verify",
    description: "Verify active project folder against expected ProjectIdentity fields and report mismatches without mutating state.",
    promptSnippet: "Use when project folder or session identity is ambiguous before accepting a Workpoint/Trajectory packet as canonical.",
    parameters: Type.Object({
      cwd: Type.Optional(Type.String({ description: "Optional cwd/project path hint; defaults to Pi session cwd." })),
      project_root: Type.Optional(Type.String({ description: "Expected project root." })),
      project_id: Type.Optional(Type.String({ description: "Expected project id from marker/operator." })),
      canonical_name: Type.Optional(Type.String({ description: "Expected canonical project name." })),
      repo_remote: Type.Optional(Type.String({ description: "Expected git origin remote." })),
      remote_host: Type.Optional(Type.String({ description: "Remote SSH host that contains the project root; caller supplies inspected evidence." })),
      remote_user: Type.Optional(Type.String({ description: "Remote SSH user, if known." })),
      remote_port: Type.Optional(Type.Integer({ minimum: 1, maximum: 65535, description: "Remote SSH port, if known." })),
      remote_repo_remote: Type.Optional(Type.String({ description: "Git origin/repo remote observed on the remote host." })),
      remote_workspace_kind: Type.Optional(Type.String({ description: "Workspace kind observed on the remote host." })),
      remote_deploy_root: Type.Optional(Type.String({ description: "Deployment/site root observed on the remote host." })),
      persisted_project_root: Type.Optional(Type.String({ description: "Prior ProjectIdentity root from this Pi session; auto-filled when omitted." })),
      persisted_project_fingerprint: Type.Optional(Type.String({ description: "Prior ProjectIdentity fingerprint from this Pi session; auto-filled when omitted." })),
      persisted_project_id: Type.Optional(Type.String({ description: "Prior ProjectIdentity project id from this Pi session; auto-filled when omitted." })),
      persisted_canonical_name: Type.Optional(Type.String({ description: "Prior ProjectIdentity canonical name from this Pi session; auto-filled when omitted." })),
    }),
    async execute(_id, params) {
      const p = params as { cwd?: string; project_root?: string; project_id?: string; canonical_name?: string; repo_remote?: string; remote_host?: string; remote_user?: string; remote_port?: number; remote_repo_remote?: string; remote_workspace_kind?: string; remote_deploy_root?: string; persisted_project_root?: string; persisted_project_fingerprint?: string; persisted_project_id?: string; persisted_canonical_name?: string };
      const persisted = persistedProjectIdentityFields();
      const requestedRoot = normalizeProjectRoot(p.project_root);
      const persistedRoot = normalizeProjectRoot(persisted.persisted_project_root);
      const payload = { ...((requestedRoot && persistedRoot && requestedRoot !== persistedRoot) ? {} : persisted), ...p, cwd: p.cwd || S.sessionCwd || process.cwd() };
      const result = await focusaFetchDetailed("/project/verify", { method: "POST", body: JSON.stringify(payload) });
      const body = result.body || {};
      if (!result.ok && body.failure_class === "hot_path_timeout") {
        const requestedRoot = normalizeProjectRoot(p.project_root || p.cwd || S.sessionCwd || process.cwd());
        const cachedIdentity = S.lastProjectIdentity && (!requestedRoot || normalizeProjectRoot(S.lastProjectIdentity.project_root) === requestedRoot) ? S.lastProjectIdentity : null;
        return { content: [{ type: "text", text: timeoutPreservedText("project verify", cachedIdentity ? "cached identity" : "empty fallback") }], details: { ok: false, status: "timeout_preserved", endpoint: "/v1/project/verify", canonical: false, degraded: true, advisory_only: true, project_identity: cachedIdentity || {}, verification: { verified: false, reason: "hot_path_timeout" }, failure_class: "hot_path_timeout", response: compactApiEcho(body), next_tools: ["focusa_tool_doctor", "focusa_resource_mode", "focusa_project_identity", "focusa_project_verify", "focusa_trajectory_view"] } } as any;
      }
      const identity = body.project_identity || {};
      const verified = body.verification?.verified === true;
      if (identity && Object.keys(identity).length) S.lastProjectVerify = body;
      const verifiedRoot = normalizeProjectRoot(identity.project_root);
      if (verified && verifiedRoot && isProjectRootAuthoritySafe(verifiedRoot)) {
        confirmPiProjectRoot(verifiedRoot, "focusa_project_verify_verified");
        ensureContinuityId(verifiedRoot);
        persistState();
      }
      const text = result.ok
        ? `project verify → verified=${verified} status=${String(identity.status || body.status || "unknown")} confidence=${String(identity.confidence || "unknown")} root=${String(identity.project_root || "unknown")}`
        : `project verify blocked → ${explainWorkLoopResult(result, "project verify unavailable")}`;
      const toolResult = body.details?.tool_result_v1 || { ok: result.ok && body.status !== "blocked", status: result.ok ? String(body.status || "completed") : "blocked", canonical: body.canonical === true, degraded: body.degraded !== false, failure_class: body.failure_class || null, retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" }, side_effects: [], evidence_refs: [], next_tools: body.next_tools || ["focusa_project_identity", "focusa_trajectory_view", "focusa_workpoint_resume"] };
      return {
        content: [{ type: "text", text }],
        details: {
          ok: result.ok && body.status !== "blocked",
          status: result.ok ? String(body.status || "completed") : "blocked",
          endpoint: "/v1/project/verify",
          canonical: body.canonical === true,
          degraded: body.degraded === true,
          project_identity: identity,
          verification: body.verification,
          tool_result_v1: toolResult,
          failure_class: toolResult.failure_class || body.failure_class || null,
          next_tools: toolResult.next_tools || body.next_tools || ["focusa_project_identity", "focusa_trajectory_view", "focusa_workpoint_resume"],
          response: compactApiEcho(body),
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_reflex_primitives",
    label: "Reflex Primitives",
    description: "List bounded Spec97 Reflex Primitive summaries by family/query; read-only routing metadata, never mutation authority.",
    parameters: Type.Object({
      family: Type.Optional(Type.String({ description: "Optional primitive family filter, e.g. recovery, evidence, resource." })),
      query: Type.Optional(Type.String({ description: "Optional risk/object/action search text." })),
      limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 50, description: "Bounded result limit." })),
      include_payload: Type.Optional(Type.Boolean({ description: "Cold opt-in for full primitive payloads; default false." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const query = new URLSearchParams();
      if (p.family) query.set("family", String(p.family));
      if (p.query) query.set("query", String(p.query));
      query.set("limit", String(Math.max(1, Math.min(50, Number(p.limit || 20)))));
      if (p.include_payload === true) query.set("include_payload", "true");
      const result = await focusaFetchDetailed(`/reflex/primitives?${query.toString()}`);
      const body = result.body || {};
      if (!result.ok) return blockedToolResponse("focusa_reflex_primitives", "reflex", `reflex primitives blocked → ${explainWorkLoopResult(result, "reflex registry unavailable")}`, body.failure_class || "daemon_unavailable", body, ["focusa_traverse", "focusa_tool_doctor"]);
      const items = Array.isArray(body.items) ? body.items : [];
      const families = Array.from(new Set(items.map((item: any) => String(item.family || "unknown")))).slice(0, 6).join(",");
      const toolResult = body.details?.tool_result_v1 || focusaToolResult({ ok: true, status: "completed", summary: `reflex primitives → returned=${items.length} families=${families || "none"}`, tool: "focusa_reflex_primitives", family: "reflex", side_effects: [], evidence_refs: [], next_tools: ["focusa_traverse", "focusa_tool_doctor"], raw: body });
      return { content: [{ type: "text", text: `reflex primitives → returned=${items.length} families=${families || "none"} truncated=${Boolean(body.bounds?.truncated)}` }], details: { ok: true, status: "completed", endpoint: "/v1/reflex/primitives", canonical: body.canonical === true, degraded: body.degraded === true, read_only: body.read_only === true, advisory_only: body.advisory_only === true, items, bounds: body.bounds || null, tool_result_v1: toolResult, next_tools: toolResult.next_tools || ["focusa_traverse"] } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_trajectory_view",
    label: "Trajectory View",
    description: "Read the per-project Trajectory Intelligence view: project identity, goal/state/gap/evidence/drift, and next Workpoint candidate.",
    promptSnippet: "Use first on project start/resume or when goal/state/next action is unclear; Trajectory is advisory and per-project.",
    parameters: Type.Object({
      project_root: Type.Optional(Type.String({ description: "Optional expected project root; defaults to Pi session cwd." })),
      session_id: Type.Optional(Type.String({ description: "Optional temporal Pi session id; defaults to Pi session key." })),
      continuity_id: Type.Optional(Type.String({ description: "Optional logical continuity id; defaults to Pi continuity id and is part of authority boundary." })),
      mode: Type.Optional(Type.Union([Type.Literal("summary"), Type.Literal("full")], { description: "View mode; summary is hot-path bounded." })),
      allow_prior_project_trajectory: Type.Optional(Type.Boolean({ description: "If true, use the prior same-project trajectory as advisory reload fallback when continuity_id changed." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      const query = new URLSearchParams();
      query.set("project_root", projectRoot);
      if (p.session_id || S.sessionFrameKey) query.set("session_id", String(p.session_id || S.sessionFrameKey));
      if (p.continuity_id || S.continuityId) query.set("continuity_id", String(p.continuity_id || S.continuityId));
      const viewMode = String(p.mode || "summary");
      query.set("mode", viewMode);
      if (p.allow_prior_project_trajectory === true) query.set("allow_prior_project_trajectory", "true");
      const result = await focusaFetchDetailed(`/trajectory/view?${query.toString()}`, { method: "GET" });
      const body = result.body || {};
      if (!result.ok && body.failure_class === "hot_path_timeout") {
        const fallback = {
          ...(S.lastTrajectoryClarity || {}),
          status: "timeout_preserved",
          canonical: false,
          degraded: true,
          advisory_only: true,
          failure_class: "hot_path_timeout",
          project_root: projectRoot,
          continuity_id: String(p.continuity_id || S.continuityId || "") || null,
          session_id: String(p.session_id || S.sessionFrameKey || "") || null,
          preserved_at: new Date().toISOString(),
          next_step_hint: "Retry focusa_trajectory_view after focusa_tool_doctor/resource_mode; use fallback only as advisory orientation.",
        };
        S.lastTrajectoryClarity = fallback;
        try { S.pi?.appendEntry("focusa-trajectory-timeout-fallback", fallback); } catch { /* best effort */ }
        persistState();
        return { content: [{ type: "text", text: timeoutPreservedText("trajectory view", "cached clarity") }], details: { ok: false, status: "timeout_preserved", endpoint: "/v1/trajectory/view", canonical: false, degraded: true, advisory_only: true, trajectory: fallback, failure_class: "hot_path_timeout", response: compactApiEcho(body), next_tools: ["focusa_tool_doctor", "focusa_resource_mode", "focusa_trajectory_view", "focusa_workpoint_resume"] } } as any;
      }
      const project = body.project_identity || {};
      const trajectory = body.trajectory || {};
      if (trajectory.short_term_goal && !(body.intelligence_view?.focus_trajectory_sync?.current_focus)) {
        body.intelligence_view = {
          ...(body.intelligence_view || {}),
          focus_trajectory_sync: {
            ...(body.intelligence_view?.focus_trajectory_sync || {}),
            current_focus: trajectory.short_term_goal,
            current_focus_source: "trajectory_short_term_goal",
            projection_only: true,
          },
        };
      }
      if (trajectory.short_term_goal || trajectory.current_state || trajectory.active_gap) {
        S.lastTrajectoryClarity = {
          ...(S.lastTrajectoryClarity || {}),
          status: String(body.intelligence_view?.clarity_gate?.status || trajectory.definition_status || body.status || "unknown"),
          recommended_action: String(body.intelligence_view?.clarity_gate?.recommended_action || body.intelligence_view?.context_sufficiency?.recommended_action || "unknown"),
          project_root: String(project.project_root || projectRoot),
          trajectory_id: trajectory.trajectory_id || null,
          short_term_goal: trajectory.short_term_goal || null,
          current_state: trajectory.current_state || null,
          active_gap: trajectory.active_gap || null,
          focus_trajectory_sync: body.intelligence_view?.focus_trajectory_sync || null,
        };
        persistState();
      }
      const sufficiency = body.intelligence_view?.context_sufficiency || {};
      const posture = String(sufficiency.proceed_posture || sufficiency.recommended_action || "unknown");
      const projectMismatches = Array.isArray(project.mismatches) ? project.mismatches : [];
      const trajectoryUnset = body.status === "not_found" && String(project.status || "") === "verified" && projectMismatches.length === 0;
      const trajectoryBootstrapDefault = trajectory.bootstrap_default === true || trajectory.needs_definition === true;
      const recovery = trajectoryUnset || trajectoryBootstrapDefault ? null : scopeRecoveryContext(body, projectRoot, String(p.continuity_id || S.continuityId || ""), "trajectory_view");
      const trajectoryText = trajectoryBootstrapDefault
        ? `trajectory view → BOOTSTRAP DEFAULT project=${String(project.project_root || projectRoot)} long_term=${String(trajectory.long_term_goal || "missing")} desired=${String(trajectory.desired_end_state || "missing")} posture=${posture}; needs=focusa_trajectory_define_goal`
        : trajectoryUnset
          ? `trajectory view → NOT SET for project=${String(project.project_root || projectRoot)}; definition=unclear; posture=${posture}; next=focusa_trajectory_define_goal`
          : trajectory.fallback_prior_project_trajectory === true
            ? `trajectory view → PRIOR PROJECT FALLBACK long_term=${String(trajectory.long_term_goal || "missing")} desired=${String(trajectory.desired_end_state || "missing")} short=${String(trajectory.short_term_goal || "missing")} posture=${posture}; refresh short-term goal when needed`
            : body.canonical === true
              ? `trajectory view → SET long_term=${String(trajectory.long_term_goal || "missing")} desired=${String(trajectory.desired_end_state || "missing")} current=${String(trajectory.current_state || "missing")} gap=${String(trajectory.active_gap || "none")} posture=${posture}`
              : `trajectory view → status=${String(body.status || "unknown")} canonical=${body.canonical === true} project=${String(project.status || "unknown")} definition=${String(trajectory.definition_status || "unknown")} posture=${posture}`;
      const text = result.ok
        ? [trajectoryText, recovery?.text].filter(Boolean).join("\n")
        : `trajectory view blocked → ${explainWorkLoopResult(result, "trajectory unavailable")}`;
      const toolResult = body.details?.tool_result_v1 || { ok: result.ok && body.status !== "degraded" && body.status !== "not_found", status: result.ok ? String(body.status || "completed") : String(result.status), canonical: body.canonical === true, degraded: body.degraded === true, failure_class: body.failure_class || null, retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" }, side_effects: [], evidence_refs: [], next_tools: body.next_tools || ["focusa_workpoint_resume", "focusa_active_object_resolve"] };
      return {
        content: [{ type: "text", text }],
        details: {
          ok: toolResult.ok,
          status: result.ok ? String(body.status || "completed") : String(result.status),
          endpoint: "/v1/trajectory/view",
          canonical: body.canonical === true,
          degraded: body.degraded === true,
          project_identity: project,
          trajectory,
          intelligence_view: body.intelligence_view || null,
          scope_recovery_context: recovery?.details || null,
          tool_result_v1: toolResult,
          failure_class: toolResult.failure_class || null,
          evidence_refs: toolResult.evidence_refs || [],
          side_effects: toolResult.side_effects || [],
          next_tools: toolResult.next_tools || body.next_tools || ["focusa_workpoint_resume", "focusa_active_object_resolve"],
          response: compactApiEcho(body),
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_trajectory_define_goal",
    label: "Trajectory Define Goal",
    description: "Create an advisory per-project Trajectory goal candidate without changing task/execution authority.",
    promptSnippet: "Use when the project trajectory is unclear or operator provides/changes the project goal.",
    parameters: Type.Object({
      long_term_goal: Type.String({ description: "Stable project-level long-term goal." }),
      desired_end_state: Type.String({ description: "Evidence-backed desired project end state." }),
      mid_level_goal: Type.Optional(Type.String({ description: "Current mid-level goal (MLG) derived from the HLT." })),
      short_term_goal: Type.Optional(Type.String({ description: "Current short-term goal (STG) derived from the HLT/MLG." })),
      waypoints: Type.Optional(Type.Array(Type.String(), { description: "Concrete HLT-aligned progress markers along the MLG/STG path." })),
      current_state: Type.Optional(Type.String({ description: "Current verified state if known." })),
      goal_source: Type.Optional(Type.String({ description: "operator|durable_supersession|focus_state|workpoint|beads|imported|inferred_context" })),
      supersedes_trajectory_id: Type.Optional(Type.String({ description: "Prior trajectory id if this supersedes one." })),
      operator_confirmed: Type.Optional(Type.Boolean({ description: "True when operator explicitly confirmed a root goal change." })),
      supersession_evidence_refs: Type.Optional(Type.Array(Type.String(), { description: "Durable evidence refs allowing root goal supersession without direct operator prompt." })),
      required_evidence_refs: Type.Optional(Type.Array(Type.String(), { description: "Evidence refs required to prove the desired end state." })),
      required_checks: Type.Optional(Type.Array(Type.String(), { description: "Checks required before the trajectory can be considered done." })),
      acceptance_risks: Type.Optional(Type.Array(Type.String(), { description: "Known false-completion or acceptance risks." })),
      not_done_if: Type.Optional(Type.Array(Type.String(), { description: "Conditions proving the trajectory is not done." })),
      project_root: Type.Optional(Type.String({ description: "Optional expected project root; defaults to Pi session cwd." })),
      session_id: Type.Optional(Type.String({ description: "Optional temporal Pi session id; defaults to Pi session key." })),
      continuity_id: Type.Optional(Type.String({ description: "Optional logical continuity id; defaults to Pi continuity id." })),
      idempotency_key: Type.Optional(Type.String({ description: "Optional external idempotency key." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      const body = { ...p, project_root: projectRoot, session_id: p.session_id || S.sessionFrameKey, continuity_id: p.continuity_id || S.continuityId, session_identity: await buildFocusaSessionIdentity(projectRoot, "manual", { continuityId: p.continuity_id, sessionId: p.session_id }) };
      const result = await focusaFetchDetailed("/trajectory/define-goal", { method: "POST", body: JSON.stringify(body) });
      const b = result.body || {};
      if (!result.ok && b.failure_class === "hot_path_timeout") {
        const fallbackCandidate = {
          ...body,
          definition_status: "timeout_preserved",
          canonical: false,
          degraded: true,
          failure_class: "hot_path_timeout",
          persisted: false,
          preserved_at: new Date().toISOString(),
        };
        S.lastTrajectoryClarity = {
          reason: "define_goal_timeout_preserved",
          refreshed_at: Date.now(),
          project_root: projectRoot,
          continuity_id: body.continuity_id || null,
          session_id: body.session_id || null,
          status: "timeout_preserved",
          recommended_action: "retry_define_goal_after_tool_doctor_or_resource_mode",
          canonical: false,
          degraded: true,
          trajectory_id: null,
          long_term_goal: body.long_term_goal || null,
          desired_end_state: body.desired_end_state || null,
          mid_level_goal: body.mid_level_goal || null,
          short_term_goal: body.short_term_goal || null,
          waypoints: body.waypoints || [],
          current_state: body.current_state || null,
          active_gap: body.short_term_goal || null,
          timeout_preserved: true,
        };
        try { S.pi?.appendEntry("focusa-trajectory-timeout-fallback", fallbackCandidate); } catch { /* best effort */ }
        persistState();
        return { content: [{ type: "text", text: timeoutPreservedText("trajectory define_goal", "candidate") }], details: { ok: false, status: "timeout_preserved", endpoint: "/v1/trajectory/define-goal", canonical: false, degraded: true, advisory_only: true, trajectory_candidate: fallbackCandidate, failure_class: "hot_path_timeout", response: compactApiEcho(b), next_tools: ["focusa_tool_doctor", "focusa_resource_mode", "focusa_trajectory_define_goal", "focusa_trajectory_view"] } } as any;
      }
      const pendingCandidate = String(b.status || "") === "pending" && !b.trajectory_candidate
        ? { long_term_goal: body.long_term_goal, desired_end_state: body.desired_end_state, mid_level_goal: body.mid_level_goal, short_term_goal: body.short_term_goal, waypoints: body.waypoints || [], current_state: body.current_state, definition_status: "pending" }
        : null;
      const candidate = b.trajectory_candidate || pendingCandidate || {};
      const defineLabel = String(b.status || "") === "pending" ? "PENDING" : b.canonical === true ? "SET" : "NOT SET";
      const text = result.ok
        ? `trajectory define_goal → ${defineLabel} HLT=${String(candidate.long_term_goal || "missing")} MLG=${String(candidate.mid_level_goal || "missing")} STG=${String(candidate.short_term_goal || "missing")} waypoints=${Array.isArray(candidate.waypoints) ? candidate.waypoints.length : 0} definition=${String(candidate.definition_status || "unknown")} persisted=${b.persisted === true}`
        : `trajectory define_goal blocked → ${explainWorkLoopResult(result, "define failed")}`;
      const toolResult = b.details?.tool_result_v1 || { ok: result.ok && b.status !== "validation_rejected", status: result.ok ? String(b.status || "completed") : String(result.status), canonical: b.canonical === true, degraded: b.degraded === true, failure_class: b.failure_class || null, retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" }, side_effects: [], evidence_refs: p.supersession_evidence_refs || [], next_tools: b.next_tools || ["focusa_trajectory_assess"] };
      return { content: [{ type: "text", text }], details: { ok: toolResult.ok, status: result.ok ? String(b.status || "completed") : String(result.status), endpoint: "/v1/trajectory/define-goal", canonical: b.canonical === true, degraded: b.degraded === true, advisory_only: b.advisory_only === true, trajectory_candidate: candidate, tool_result_v1: toolResult, failure_class: toolResult.failure_class || null, side_effects: toolResult.side_effects || [], evidence_refs: toolResult.evidence_refs || [], response: compactApiEcho(b), next_tools: toolResult.next_tools || b.next_tools || ["focusa_trajectory_assess"] } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_trajectory_assess",
    label: "Trajectory Assess",
    description: "Assess current project state against the desired Trajectory end state and return gaps/recommended action.",
    promptSnippet: "Use after trajectory view/define_goal or after verification evidence changes current state.",
    parameters: Type.Object({
      observed_state: Type.Optional(Type.String({ description: "Observed current state override." })),
      evidence_refs: Type.Optional(Type.Array(Type.String(), { description: "Evidence refs supporting observed state." })),
      project_root: Type.Optional(Type.String({ description: "Optional expected project root; defaults to Pi session cwd." })),
      session_id: Type.Optional(Type.String({ description: "Optional temporal Pi session id; defaults to Pi session key." })),
      continuity_id: Type.Optional(Type.String({ description: "Optional logical continuity id; defaults to Pi continuity id." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      const body = { ...p, project_root: projectRoot, session_id: p.session_id || S.sessionFrameKey, continuity_id: p.continuity_id || S.continuityId, session_identity: await buildFocusaSessionIdentity(projectRoot, "manual", { continuityId: p.continuity_id, sessionId: p.session_id }) };
      const result = await focusaFetchDetailed("/trajectory/assess", { method: "POST", body: JSON.stringify(body) });
      const b = result.body || {};
      if (!result.ok && b.failure_class === "hot_path_timeout") return trajectoryTimeoutFallbackResult("assess", "/v1/trajectory/assess", body, b, ["focusa_tool_doctor", "focusa_resource_mode", "focusa_trajectory_assess", "focusa_trajectory_propose_workpoint"], { observed_state: body.observed_state || null, evidence_refs: body.evidence_refs || [] });
      const text = result.ok ? `trajectory assess → gaps=${Array.isArray(b.gaps) ? b.gaps.length : 0} action=${String(b.recommended_action || "unknown")} canonical=${b.canonical === true}` : `trajectory assess blocked → ${explainWorkLoopResult(result, "assess failed")}`;
      const toolResult = b.details?.tool_result_v1 || { ok: result.ok, status: result.ok ? String(b.status || "completed") : String(result.status), canonical: b.canonical === true, degraded: b.degraded === true, failure_class: b.failure_class || null, retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" }, side_effects: [], evidence_refs: p.evidence_refs || [], next_tools: b.next_tools || ["focusa_trajectory_propose_workpoint"] };
      return { content: [{ type: "text", text }], details: { ok: toolResult.ok, status: result.ok ? String(b.status || "completed") : String(result.status), endpoint: "/v1/trajectory/assess", canonical: b.canonical === true, degraded: b.degraded === true, gaps: b.gaps || [], recommended_action: b.recommended_action || null, tool_result_v1: toolResult, failure_class: toolResult.failure_class || null, side_effects: toolResult.side_effects || [], evidence_refs: toolResult.evidence_refs || [], response: compactApiEcho(b), next_tools: toolResult.next_tools || b.next_tools || ["focusa_trajectory_propose_workpoint"] } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_trajectory_propose_workpoint",
    label: "Trajectory Propose Workpoint",
    description: "Propose an advisory Workpoint candidate from the active per-project Trajectory gap; does not promote or execute it.",
    promptSnippet: "Use after trajectory assess says propose_workpoint; pass candidate to focusa_workpoint_checkpoint only if accepted.",
    parameters: Type.Object({
      trajectory_id: Type.Optional(Type.String({ description: "Trajectory id to use; defaults to active project trajectory." })),
      target_ref: Type.Optional(Type.String({ description: "Optional target object/file/ref." })),
      action_type: Type.Optional(Type.String({ description: "Optional action intent type." })),
      project_root: Type.Optional(Type.String({ description: "Optional expected project root; defaults to Pi session cwd." })),
      session_id: Type.Optional(Type.String({ description: "Optional temporal Pi session id; defaults to Pi session key." })),
      continuity_id: Type.Optional(Type.String({ description: "Optional logical continuity id; defaults to Pi continuity id." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      const body = { ...p, project_root: projectRoot, session_id: p.session_id || S.sessionFrameKey, continuity_id: p.continuity_id || S.continuityId, session_identity: await buildFocusaSessionIdentity(projectRoot, "manual", { continuityId: p.continuity_id, sessionId: p.session_id }) };
      const result = await focusaFetchDetailed("/trajectory/propose-workpoint", { method: "POST", body: JSON.stringify(body) });
      const b = result.body || {};
      if (!result.ok && b.failure_class === "hot_path_timeout") return trajectoryTimeoutFallbackResult("propose_workpoint", "/v1/trajectory/propose-workpoint", body, b, ["focusa_tool_doctor", "focusa_resource_mode", "focusa_trajectory_propose_workpoint", "focusa_workpoint_checkpoint"], { workpoint_candidate: { action_intent: { action_type: body.action_type || "unknown", target_ref: body.target_ref || body.trajectory_id || "trajectory" }, checkpoint_required: true, blockers: [{ reason: "trajectory proposal timed out before canonical candidate was returned", severity: "medium", status: "open" }] } });
      const candidate = b.workpoint_candidate || {};
      const blockers = Array.isArray(candidate.blockers) ? candidate.blockers.length : 0;
      const text = result.ok ? `trajectory propose_workpoint → advisory=${b.advisory_only === true} action=${String(candidate.action_intent?.action_type || "unknown")} checkpoint_required=${candidate.checkpoint_required === true} blockers=${blockers} no_execution=${b.no_execution_side_effects === true}` : `trajectory propose_workpoint blocked → ${explainWorkLoopResult(result, "proposal failed")}`;
      const toolResult = b.details?.tool_result_v1 || { ok: result.ok, status: result.ok ? String(b.status || "completed") : String(result.status), canonical: b.canonical === true, degraded: b.degraded === true, failure_class: b.failure_class || null, retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" }, side_effects: [], evidence_refs: [], next_tools: b.next_tools || ["focusa_workpoint_checkpoint"] };
      return { content: [{ type: "text", text }], details: { ok: toolResult.ok, status: result.ok ? String(b.status || "completed") : String(result.status), endpoint: "/v1/trajectory/propose-workpoint", canonical: b.canonical === true, degraded: b.degraded === true, advisory_only: b.advisory_only === true, no_execution_side_effects: b.no_execution_side_effects === true, workpoint_candidate: candidate, tool_result_v1: toolResult, failure_class: toolResult.failure_class || null, side_effects: toolResult.side_effects || [], evidence_refs: toolResult.evidence_refs || [], response: compactApiEcho(b), next_tools: toolResult.next_tools || b.next_tools || ["focusa_workpoint_checkpoint"] } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_trajectory_checkpoint",
    label: "Trajectory Checkpoint",
    description: "Create an advisory Trajectory checkpoint packet before compaction/model switch; pair with Workpoint checkpoint for canonical continuation.",
    promptSnippet: "Use before compaction/model switch alongside focusa_workpoint_checkpoint; this does not replace Workpoint.",
    parameters: Type.Object({
      summary: Type.Optional(Type.String({ description: "Optional bounded Trajectory checkpoint summary." })),
      project_root: Type.Optional(Type.String({ description: "Optional expected project root; defaults to Pi session cwd." })),
      session_id: Type.Optional(Type.String({ description: "Optional temporal Pi session id; defaults to Pi session key." })),
      continuity_id: Type.Optional(Type.String({ description: "Optional logical continuity id; defaults to Pi continuity id." })),
      idempotency_key: Type.Optional(Type.String({ description: "Optional external idempotency key." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      const body = { ...p, project_root: projectRoot, session_id: p.session_id || S.sessionFrameKey, continuity_id: p.continuity_id || S.continuityId, session_identity: await buildFocusaSessionIdentity(projectRoot, "compaction", { continuityId: p.continuity_id, sessionId: p.session_id }) };
      const result = await focusaFetchDetailed("/trajectory/checkpoint", { method: "POST", body: JSON.stringify(body) });
      const b = result.body || {};
      if (!result.ok && b.failure_class === "hot_path_timeout") return trajectoryTimeoutFallbackResult("checkpoint", "/v1/trajectory/checkpoint", body, b, ["focusa_tool_doctor", "focusa_resource_mode", "focusa_trajectory_checkpoint", "focusa_workpoint_checkpoint"], { trajectory_checkpoint: { summary: body.summary || "trajectory checkpoint timeout fallback", persisted: false } });
      const text = result.ok ? `trajectory checkpoint → status=${String(b.status || "unknown")} persisted=${b.persisted === true} canonical=${b.canonical === true}` : `trajectory checkpoint blocked → ${explainWorkLoopResult(result, "checkpoint failed")}`;
      const toolResult = b.details?.tool_result_v1 || { ok: result.ok, status: result.ok ? String(b.status || "completed") : String(result.status), canonical: b.canonical === true, degraded: b.degraded === true, failure_class: b.failure_class || null, retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" }, side_effects: [], evidence_refs: [], next_tools: b.next_tools || ["focusa_workpoint_checkpoint"] };
      return { content: [{ type: "text", text }], details: { ok: toolResult.ok, status: result.ok ? String(b.status || "completed") : String(result.status), endpoint: "/v1/trajectory/checkpoint", canonical: b.canonical === true, degraded: b.degraded === true, persisted: b.persisted === true, advisory_only: b.advisory_only === true, trajectory_checkpoint: b.trajectory_checkpoint || null, tool_result_v1: toolResult, failure_class: toolResult.failure_class || null, side_effects: toolResult.side_effects || [], evidence_refs: toolResult.evidence_refs || [], response: compactApiEcho(b), next_tools: toolResult.next_tools || b.next_tools || ["focusa_workpoint_checkpoint"] } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_trajectory_resume",
    label: "Trajectory Resume",
    description: "Resume per-project Trajectory orientation plus Workpoint handoff context after compaction/model switch/session resume.",
    promptSnippet: "Use after compaction/resume before choosing action; inject with Workpoint resume.",
    parameters: Type.Object({
      mode: Type.Optional(Type.Union([Type.Literal("summary"), Type.Literal("full")], { description: "Resume mode; summary is bounded." })),
      project_root: Type.Optional(Type.String({ description: "Optional expected project root; defaults to Pi session cwd." })),
      session_id: Type.Optional(Type.String({ description: "Optional temporal Pi session id; defaults to Pi session key." })),
      continuity_id: Type.Optional(Type.String({ description: "Optional logical continuity id; defaults to Pi continuity id." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      const body = { ...p, project_root: projectRoot, session_id: p.session_id || S.sessionFrameKey, continuity_id: p.continuity_id || S.continuityId, session_identity: await buildFocusaSessionIdentity(projectRoot, "session_switch", { continuityId: p.continuity_id, sessionId: p.session_id }) };
      const result = await focusaFetchDetailed("/trajectory/resume", { method: "POST", body: JSON.stringify(body) });
      const b = result.body || {};
      if (!result.ok && b.failure_class === "hot_path_timeout") return trajectoryTimeoutFallbackResult("resume", "/v1/trajectory/resume", body, b, ["focusa_tool_doctor", "focusa_resource_mode", "focusa_trajectory_resume", "focusa_workpoint_resume"], { resume_packet: S.lastTrajectoryClarity || null });
      const packet = b.resume_packet || {};
      const text = result.ok ? `trajectory resume → status=${String(b.status || "unknown")} canonical=${b.canonical === true} project=${String(packet.project_identity?.status || "unknown")}` : `trajectory resume blocked → ${explainWorkLoopResult(result, "resume failed")}`;
      const toolResult = b.details?.tool_result_v1 || { ok: result.ok && b.status !== "degraded" && b.status !== "not_found", status: result.ok ? String(b.status || "completed") : String(result.status), canonical: b.canonical === true, degraded: b.degraded === true, failure_class: b.failure_class || null, retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" }, side_effects: [], evidence_refs: [], next_tools: b.next_tools || ["focusa_workpoint_resume"] };
      return { content: [{ type: "text", text }], details: { ok: toolResult.ok, status: result.ok ? String(b.status || "completed") : String(result.status), endpoint: "/v1/trajectory/resume", canonical: b.canonical === true, degraded: b.degraded === true, resume_packet: packet, tool_result_v1: toolResult, failure_class: toolResult.failure_class || null, side_effects: toolResult.side_effects || [], evidence_refs: toolResult.evidence_refs || [], response: compactApiEcho(b), next_tools: toolResult.next_tools || b.next_tools || ["focusa_workpoint_resume"] } } as any;
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
      workpoint_id: Type.Optional(Type.String({ description: "Specific Workpoint id; omit to use active Workpoint." })),
      project_root: Type.Optional(Type.String({ description: "Explicit safe project folder/root; use after compaction if Pi cwd is broad like /root." })),
      session_id: Type.Optional(Type.String({ description: "Optional temporal Pi session id; defaults to this Pi session key." })),
      continuity_id: Type.Optional(Type.String({ description: "Stable logical session/workstream id; defaults to this Pi continuity id." })),
      attach_to_workpoint: Type.Optional(Type.Boolean({ description: "Defaults true." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      if (p.attach_to_workpoint === false) {
        const projectRoot = p.project_root ? await resolveFocusaToolProjectRoot(p.project_root) : null;
        return { content: [{ type: "text", text: `evidence capture → captured ref=${p.evidence_ref} attach_to_workpoint=false` }], details: { ok: true, status: "completed", evidence_ref: p.evidence_ref, project_root_permission_posture: projectRoot ? projectRootPermissionPosture(projectRoot) : null } } as any;
      }
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      const clarity = await enforceTrajectoryClarityPrecondition(projectRoot, "evidence capture", { blockOperatorInput: false, continuityId: p.continuity_id, sessionId: p.session_id });
      if (!clarity.ok) {
        const degraded = evidenceClarityFallbackResult("evidence capture", p, projectRoot, clarity);
        if (degraded) return degraded;
        return { content: [{ type: "text", text: `${clarity.text || "evidence capture blocked by trajectory clarity gate"}. Why: trajectory clarity is required before linking proof to canonical Workpoint state; follow next_tools/recovery_hint instead of retrying blindly.` }], details: { ok: false, status: "blocked", why: "trajectory clarity is required before canonical evidence linkage", ...clarity.details } } as any;
      }
      const sessionIdentity = await buildFocusaSessionIdentity(projectRoot, "manual", { continuityId: p.continuity_id, sessionId: p.session_id });
      const res = await focusaFetchDetailed("/workpoint/evidence/link", {
        method: "POST",
        headers: { "x-focusa-writer-id": await preferredWriterId() },
        body: JSON.stringify({ workpoint_id: p.workpoint_id, target_ref: p.target_ref, result: p.result, evidence_ref: p.evidence_ref, session_identity: sessionIdentity, trajectory_clarity_precondition: clarity.details }),
      });
      const recovery = res.ok ? null : scopeRecoveryContext(res.body || {}, projectRoot, p.continuity_id || S.continuityId || "", "evidence_capture");
      const text = res.ok
        ? `evidence capture → linked ${p.evidence_ref}`
        : [`evidence capture blocked → ${explainWorkLoopResult(res, "link failed")}`, recovery?.text].filter(Boolean).join("\n");
      return { content: [{ type: "text", text }], details: { ok: res.ok, status: String(res.status), evidence_ref: p.evidence_ref, failure_class: res.body?.failure_class || null, scope_recovery_context: recovery?.details || null, request_scope: { project_root: projectRoot, continuity_id: sessionIdentity?.continuity_id || null }, project_root_permission_posture: projectRootPermissionPosture(projectRoot), response: compactApiEcho(res.body), next_tools: recovery?.details?.safe_next_tools || res.body?.next_tools || ["focusa_workpoint_resume", "focusa_workpoint_checkpoint"] } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_browser_diagnostics_intake",
    label: "Browser Diagnostics Intake",
    description: "Turn UIAI/browser diagnostics JSON into bounded Focusa evidence, active-object hints, a prediction candidate, and a metacog candidate.",
    promptSnippet: "Use after UIAI browser diagnostics/error envelopes to standardize evidence + learning intake before manual interpretation.",
    parameters: Type.Object({
      diagnostics: Type.Optional(Type.Any({ description: "Diagnostics JSON object or browser action failure envelope." })),
      diagnostics_ref: Type.Optional(Type.String({ description: "Stable file/artifact/URL handle for diagnostics JSON; local files are read best-effort." })),
      target_ref: Type.Optional(Type.String({ description: "Object/page/endpoint proven by these diagnostics; inferred from diagnostics when omitted." })),
      result: Type.Optional(Type.String({ description: "Optional bounded result summary override." })),
      workpoint_id: Type.Optional(Type.String({ description: "Specific Workpoint id; omit to use active Workpoint." })),
      project_root: Type.Optional(Type.String({ description: "Explicit project root for canonical evidence linkage." })),
      session_id: Type.Optional(Type.String({ description: "Optional temporal Pi session id; defaults to this Pi session key." })),
      continuity_id: Type.Optional(Type.String({ description: "Stable logical session/workstream id; defaults to this Pi continuity id." })),
      attach_to_workpoint: Type.Optional(Type.Boolean({ description: "Defaults true; false performs dry intake without canonical evidence linkage." })),
      create_prediction: Type.Optional(Type.Boolean({ description: "Defaults true; records bounded follow-up prediction candidate." })),
      create_metacog: Type.Optional(Type.Boolean({ description: "Defaults false; capture only when this diagnostics pattern should become reusable learning." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const readJsonArtifact = (ref?: string): any | null => {
        if (!ref || !ref.startsWith("/")) return null;
        try {
          const fs = require("fs");
          const raw = fs.readFileSync(ref, "utf8");
          return JSON.parse(raw);
        } catch { return null; }
      };
      const diagnostics = p.diagnostics && typeof p.diagnostics === "object" ? p.diagnostics : readJsonArtifact(p.diagnostics_ref) || {};
      const focusaScope = diagnostics.focusa_scope || diagnostics.session?.focusa_scope || diagnostics.diagnostics?.focusa_scope || {};
      const scopedWorkpointId = p.workpoint_id || focusaScope.workpoint_id;
      const scopedContinuityId = p.continuity_id || focusaScope.continuity_id;
      const scopedProjectRoot = p.project_root || focusaScope.project_root;
      const scopedSessionId = p.session_id || focusaScope.session_id;
      const asArray = (value: any): any[] => Array.isArray(value) ? value : [];
      const dig = (obj: any, keys: string[]): any => keys.reduce((cur, key) => cur && typeof cur === "object" ? cur[key] : undefined, obj);
      const consoleItems = [...asArray(diagnostics.console), ...asArray(diagnostics.diagnostics?.console), ...asArray(diagnostics.console_errors)];
      const exceptionItems = [...asArray(diagnostics.exceptions), ...asArray(diagnostics.page_errors), ...asArray(diagnostics.errors)];
      const failedItems = [...asArray(diagnostics.failed_requests), ...asArray(diagnostics.network_failures), ...asArray(diagnostics.diagnostics?.failed_requests)];
      const classifyDiagnosticsSeverity = () => {
        const allText = JSON.stringify({ consoleItems, exceptionItems, failedItems, diagnostics }).toLowerCase();
        const benignAsset = failedItems.length > 0 && failedItems.every((item: any) => /\.(png|jpe?g|gif|webp|svg|ico|css|woff2?|ttf)(\?|$)|favicon|analytics|pixel|beacon|tracking/.test(String(item.url || item.request_url || item.name || item).toLowerCase()));
        if (exceptionItems.length > 0 || /blank page|page crashed|navigation failed|main frame|document failed|hydration failed/.test(allText)) return { severity: "page_breaking", alarm: "repair_required", recommended_action: "capture diagnostics evidence, inspect page-breaking exception/navigation failure, then repair before retry" };
        if (/selector_not_found|timeout|click failed|form failed|wait failed|429|403|cors|api failed|workflow blocked/.test(allText)) return { severity: "workflow_blocking", alarm: "action_blocked", recommended_action: "use snapshot/read/diagnostics to choose a different selector or API recovery path" };
        if (benignAsset) return { severity: "benign_asset", alarm: "calm", recommended_action: "record bounded evidence only if relevant; continue workflow without alarming repair loop" };
        if (consoleItems.length === 0 && exceptionItems.length === 0 && failedItems.length === 0) return { severity: "unknown", alarm: "baseline", recommended_action: "treat as clean baseline unless UI state contradicts diagnostics" };
        return { severity: "unknown", alarm: "review", recommended_action: "review diagnostics context before deciding whether repair is needed" };
      };
      const severityClassification = classifyDiagnosticsSeverity();
      const errorClass = String(diagnostics.error_class || diagnostics.error?.class || "browser_diagnostics");
      const url = String(diagnostics.url || diagnostics.page_url || diagnostics.session?.url || dig(diagnostics, ["diagnostics", "url"]) || "unknown-url");
      const action = String(diagnostics.action || diagnostics.operation || diagnostics.selector ? `${diagnostics.action || "browser_action"}:${diagnostics.selector || "unknown-selector"}` : "browser_diagnostics");
      const targetRef = String(p.target_ref || (url !== "unknown-url" ? url : action));
      const evidenceRef = String(p.diagnostics_ref || diagnostics.evidence_ref || focusaScope.evidence_ref || `browser-diagnostics:${new Date().toISOString()}`);
      const diagSummary = String(diagnostics.diagnostics_summary || diagnostics.summary || "");
      const resultSummary = String(p.result || `${errorClass}: severity=${severityClassification.severity} alarm=${severityClassification.alarm} console=${consoleItems.length} exceptions=${exceptionItems.length} failed_requests=${failedItems.length}${diagSummary ? `; ${diagSummary}` : ""}`).slice(0, 500);
      const activeObjectHints = Array.from(new Set([targetRef, url, action, diagnostics.selector, diagnostics.request_url, diagnostics.endpoint].filter(Boolean).map(String))).slice(0, 8);
      const sideEffects: string[] = [];
      const evidenceRefs: string[] = [evidenceRef];
      let evidenceResult: any = null;
      let projectRoot: string | null = null;
      if (p.attach_to_workpoint !== false) {
        projectRoot = await resolveFocusaToolProjectRoot(scopedProjectRoot);
        const projectRootGate = projectRootConfirmationGate(projectRoot, scopedProjectRoot);
        if (projectRootGate) return projectRootGate;
        const clarity = await enforceTrajectoryClarityPrecondition(projectRoot, "browser diagnostics intake", { blockOperatorInput: false, continuityId: scopedContinuityId, sessionId: scopedSessionId });
        if (!clarity.ok) {
          return { content: [{ type: "text", text: `${clarity.text || "browser diagnostics intake blocked by trajectory clarity gate"}. next_tools=focusa_trajectory_view,focusa_workpoint_resume` }], details: { ok: false, status: "blocked", failure_class: "scope_mismatch", target_ref: targetRef, evidence_ref: evidenceRef, active_object_hints: activeObjectHints, ...clarity.details } } as any;
        }
        const sessionIdentity = await buildFocusaSessionIdentity(projectRoot, "manual", { continuityId: scopedContinuityId, sessionId: scopedSessionId });
        evidenceResult = await focusaFetchDetailed("/workpoint/evidence/link", {
          method: "POST",
          headers: { "x-focusa-writer-id": await preferredWriterId() },
          body: JSON.stringify({ workpoint_id: scopedWorkpointId, target_ref: targetRef, result: resultSummary, evidence_ref: evidenceRef, session_identity: sessionIdentity, trajectory_clarity_precondition: clarity.details }),
        });
        if (evidenceResult.ok) sideEffects.push("workpoint_evidence_link");
      }
      let predictionResult: any = null;
      if (p.create_prediction !== false) {
        predictionResult = await focusaFetchDetailed("/predictions", { method: "POST", body: JSON.stringify({
          prediction_type: "browser_diagnostics_next_action",
          predicted_outcome: severityClassification.severity === "benign_asset" ? "Benign asset diagnostics will not trigger an unnecessary repair loop." : failedItems.length || exceptionItems.length || consoleItems.length ? "Diagnostics intake will shorten the next browser-debug loop by preserving concrete console/network/error evidence." : "Diagnostics intake will act as a clean baseline for future browser-debug comparisons.",
          confidence: severityClassification.severity === "benign_asset" ? 0.82 : failedItems.length || exceptionItems.length || consoleItems.length ? 0.78 : 0.62,
          recommended_action: severityClassification.recommended_action,
          why: resultSummary,
          context_refs: evidenceRefs,
          ontology_context: { object_refs: activeObjectHints, evidence_refs: evidenceRefs, action_refs: [action], tool_refs: ["focusa_browser_diagnostics_intake"] },
        }) });
        if (predictionResult.ok) sideEffects.push("prediction_store");
      }
      let metacogResult: any = null;
      const diagnosticSignalCount = consoleItems.length + exceptionItems.length + failedItems.length;
      const recurringOrSignificant = diagnostics.recurring === true || diagnostics.recurring_pattern === true || diagnosticSignalCount >= 2 || failedItems.length >= 1 || exceptionItems.length >= 1;
      if (p.create_metacog === true && recurringOrSignificant) {
        metacogResult = await callSpec80Tool("focusa_metacog_capture", "/metacognition/capture", {
          kind: "browser_diagnostics_pattern",
          content: `Browser diagnostics pattern for ${targetRef}: ${resultSummary}`.slice(0, SPEC81_LIMITS.longText),
          rationale: "Captured from typed UIAI/browser diagnostics intake because the envelope contains recurring or significant browser failure evidence.",
          evidence_refs: evidenceRefs,
          confidence: 0.74,
          strategy_class: "browser_debugging",
        }, { method: "POST", writer: true });
        if (metacogResult.ok) sideEffects.push("metacog_capture");
      } else if (p.create_metacog === true) {
        sideEffects.push("metacog_skipped_low_signal");
      }
      const ok = p.attach_to_workpoint === false || evidenceResult?.ok === true;
      const status = ok ? "completed" : "blocked";
      const toolResult = focusaToolResult({ ok, status: ok ? "completed" : "blocked", summary: `browser diagnostics intake → ${status} evidence=${evidenceRef}`, tool: "focusa_browser_diagnostics_intake", family: "workpoint", side_effects: sideEffects, evidence_refs: evidenceRefs, next_tools: ["focusa_active_object_resolve", "focusa_evidence_capture", "focusa_predict_record", "focusa_metacog_capture"], raw: { evidence: evidenceResult?.body, prediction: predictionResult?.body, metacog: metacogResult?.body } });
      return { content: [{ type: "text", text: `browser diagnostics intake → ${status} severity=${severityClassification.severity} alarm=${severityClassification.alarm} evidence=${evidenceRef}\nactive_object_hints=${activeObjectHints.slice(0, 4).join(",") || "none"}\nnext_tools=${toolResult.next_tools.join(",")}` }], details: { ok, status, target_ref: targetRef, evidence_ref: evidenceRef, result: resultSummary, diagnostics_severity: severityClassification, active_object_hints: activeObjectHints, counts: { console: consoleItems.length, exceptions: exceptionItems.length, failed_requests: failedItems.length }, project_root: projectRoot, focusa_scope: compactApiEcho(focusaScope), scoped_workpoint_id: scopedWorkpointId || null, scoped_continuity_id: scopedContinuityId || null, tool_result_v1: toolResult, side_effects: sideEffects, evidence_refs: evidenceRefs, evidence_response: compactApiEcho(evidenceResult?.body), prediction_response: compactApiEcho(predictionResult?.body), metacog_response: compactApiEcho(metacogResult?.body), next_tools: toolResult.next_tools } } as any;
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
      continuity_id: Type.Optional(Type.String({ description: "Stable logical session/workstream id; defaults to this Pi session continuity id." })),
      checkpoint_reason: Type.Optional(Type.String({ description: "manual|operator_checkpoint|before_compact|after_compact|context_overflow|session_resume|model_switch|fork" })),
      mission: Type.String({ description: "Current mission/objective to preserve across compaction." }),
      target_objects: Type.Optional(Type.Array(Type.String(), { description: "Ontology/file/component/endpoint refs currently targeted." })),
      current_action: Type.Optional(Type.String({ description: "Typed action, e.g. patch_component_binding or resume_workpoint." })),
      verified_evidence: Type.Optional(Type.Array(Type.String(), { description: "Short evidence refs/results already verified; use handles, not raw logs." })),
      blockers: Type.Optional(Type.Array(Type.String(), { description: "Open blockers or drift boundaries." })),
      next_action: Type.String({ description: "Exact bounded next action to resume after compact/retry." }),
      do_not_drift: Type.Optional(Type.Array(Type.String(), { description: "Actions/areas the next agent must not drift into." })),
      source_turn_id: Type.Optional(Type.String({ description: "Pi/source turn id for provenance." })),
      idempotency_key: Type.Optional(Type.String({ description: "Optional external idempotency key." })),
      canonical: Type.Optional(Type.Boolean({ description: "False only for degraded fallback packets." })),
      project_root: Type.Optional(Type.String({ description: "Explicit safe project folder/root; defaults to Pi session cwd when that cwd is safe." })),
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
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      if (p.canonical !== false && !isProjectRootAuthoritySafe(projectRoot)) {
        const reason = projectRootAuthorityFailure(projectRoot) || "unsafe_project_root";
        return { content: [{ type: "text", text: `workpoint checkpoint blocked → unsafe project_root (${reason}); cd into the specific project folder/repo or pass project_root explicitly.` }], details: { ok: false, status: "blocked", failure_class: "scope_mismatch", project_root: projectRoot, project_root_permission_posture: projectRootPermissionPosture(projectRoot), reason } } as any;
      }
      const sessionIdentity = await buildFocusaSessionIdentity(projectRoot, p.checkpoint_reason === "before_compact" ? "compaction" : "manual", { continuityId: p.continuity_id, sessionId: p.session_id });
      const clarity = p.canonical === false ? { ok: true, details: { skipped: true, reason: "noncanonical_workpoint" } } : await enforceTrajectoryClarityPrecondition(projectRoot, "workpoint checkpoint", { blockOperatorInput: true, continuityId: p.continuity_id, sessionId: p.session_id });
      if (!clarity.ok) return { content: [{ type: "text", text: clarity.text || "workpoint checkpoint blocked by trajectory clarity gate" }], details: { ok: false, status: "blocked", ...clarity.details } } as any;
      const payload: any = {
        mission: p.mission,
        next_slice: [p.next_action, ...doNotDrift.map((d: string) => `DO_NOT_DRIFT: ${d}`)].filter(Boolean).join("\n"),
        work_item_id: p.work_item_id,
        continuity_id: p.continuity_id || ensureContinuityId(projectRoot),
        session_id: p.session_id || S.sessionFrameKey,
        project_root: projectRoot,
        session_identity: sessionIdentity,
        trajectory_clarity_precondition: clarity.details,
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
      if (!res.ok && res.body?.failure_class === "hot_path_timeout") {
        const fallback = stampWorkpointPacketForCurrentPiSession({
          status: "timeout_preserved",
          canonical: false,
          degraded: true,
          failure_class: "hot_path_timeout",
          project_root: projectRoot,
          continuity_id: payload.continuity_id,
          session_id: payload.session_id,
          mission: payload.mission,
          next_slice: payload.next_slice,
          action_intent: payload.action_intent,
          verification_records: payload.verification_records,
          blockers: payload.blockers,
          checkpoint_reason: payload.checkpoint_reason,
          preserved_at: new Date().toISOString(),
          next_step_hint: "Retry focusa_workpoint_checkpoint once after focusa_tool_doctor/resource_mode; do not treat timeout fallback as canonical.",
        });
        S.activeWorkpointPacket = fallback;
        S.activeWorkpointSummary = `${payload.mission || "Workpoint checkpoint"} (noncanonical timeout fallback)`;
        try { S.pi?.appendEntry("focusa-workpoint-timeout-fallback", fallback); } catch { /* best effort */ }
        persistState();
        return {
          content: [{ type: "text", text: timeoutPreservedText("workpoint checkpoint") }],
          details: { ok: false, status: "timeout_preserved", endpoint: "/workpoint/checkpoint", canonical: false, degraded: true, failure_class: "hot_path_timeout", project_root_permission_posture: projectRootPermissionPosture(projectRoot), request: compactApiEcho(payload), response: compactApiEcho(res.body), fallback_packet: compactFallbackPacket(fallback), next_tools: ["focusa_tool_doctor", "focusa_resource_mode", "focusa_workpoint_checkpoint", "focusa_workpoint_resume"] },
        } as any;
      }
      const text = res.ok
        ? `workpoint checkpoint → ${summarizeWorkpointResponse(res.body)}`
        : res.body?.status === "validation_rejected"
          ? `workpoint checkpoint validation_rejected → field=${String(res.body?.field || "unknown")} allowed=${Array.isArray(res.body?.allowed_values) ? res.body.allowed_values.join(",") : "unknown"} retry=${String(res.body?.retry_posture || "do_not_retry_unchanged")}`
          : `workpoint checkpoint blocked → ${explainWorkLoopResult(res, "checkpoint failed")}`;
      return {
        content: [{ type: "text", text }],
        details: { ok: res.ok, status: res.status, endpoint: "/workpoint/checkpoint", project_root_permission_posture: projectRootPermissionPosture(projectRoot), request: compactApiEcho(payload), response: compactApiEcho(res.body) },
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
      project_root: Type.Optional(Type.String({ description: "Explicit safe project folder/root; use after compaction if Pi cwd is broad like /root." })),
      session_id: Type.Optional(Type.String({ description: "Optional temporal Pi session id; defaults to this Pi session key." })),
      continuity_id: Type.Optional(Type.String({ description: "Stable logical session/workstream id; defaults to this Pi continuity id." })),
      attach_to_workpoint: Type.Optional(Type.Boolean({ description: "Defaults true; false returns blocked/no-op guidance without linking." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      if (p.attach_to_workpoint === false) {
        const text = "workpoint evidence link → no_op attach_to_workpoint=false";
        return {
          content: [{ type: "text", text }],
          details: { ok: true, status: "no_op", reason: "attach_to_workpoint=false", project_root_permission_posture: p.project_root ? projectRootPermissionPosture(await resolveFocusaToolProjectRoot(p.project_root)) : null },
        } as any;
      }
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      const clarity = await enforceTrajectoryClarityPrecondition(projectRoot, "workpoint evidence link", { blockOperatorInput: false, continuityId: p.continuity_id, sessionId: p.session_id });
      if (!clarity.ok) {
        const degraded = evidenceClarityFallbackResult("workpoint evidence link", p, projectRoot, clarity);
        if (degraded) return degraded;
        return { content: [{ type: "text", text: `${clarity.text || "workpoint evidence link blocked by trajectory clarity gate"}. Why: trajectory clarity is required before linking proof to canonical Workpoint state; follow next_tools/recovery_hint instead of retrying blindly.` }], details: { ok: false, status: "blocked", why: "trajectory clarity is required before canonical evidence linkage", project_root_permission_posture: projectRootPermissionPosture(projectRoot), ...clarity.details } } as any;
      }
      const res = await focusaFetchDetailed("/workpoint/evidence/link", {
        method: "POST",
        headers: { "x-focusa-writer-id": await preferredWriterId() },
        body: JSON.stringify({ workpoint_id: p.workpoint_id, target_ref: p.target_ref, result: p.result, evidence_ref: p.evidence_ref, session_identity: await buildFocusaSessionIdentity(projectRoot, "manual", { continuityId: p.continuity_id, sessionId: p.session_id }), trajectory_clarity_precondition: clarity.details }),
      });
      const text = res.ok
        ? `workpoint evidence link → status=${String(res.body?.status || "accepted")} id=${String(res.body?.workpoint_id || "none")}`
        : `workpoint evidence link blocked → ${explainWorkLoopResult(res, "link failed")}`;
      return {
        content: [{ type: "text", text }],
        details: { ok: res.ok, status: String(res.status), reason: res.ok ? "linked" : "blocked", endpoint: "/workpoint/evidence/link", project_root_permission_posture: projectRootPermissionPosture(projectRoot), response: compactApiEcho(res.body) },
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
      continuity_id: Type.Optional(Type.String({ description: "Stable logical session/workstream id; defaults to this Pi session continuity id." })),
      session_id: Type.Optional(Type.String({ description: "Optional temporal Pi session id; defaults to this Pi session key." })),
      mode: Type.Optional(Type.String({ description: "compact_prompt|full_json|operator_summary" })),
      project_root: Type.Optional(Type.String({ description: "Explicit safe project folder/root; defaults to Pi session cwd when that cwd is safe." })),
      current_ask: Type.Optional(Type.String({ description: "Optional latest operator ask used to compute current-action authority; defaults to Pi current ask." })),
    }),
    promptGuidelines: [
      "Use immediately after compaction or session resume before choosing next work.",
      "If not_found, create a checkpoint before continuing important work.",
      "If canonical=false, state degraded status and avoid treating it as canonical truth.",
    ],
    async execute(_id, params) {
      const p = params as { workpoint_id?: string; continuity_id?: string; session_id?: string; mode?: string; project_root?: string; current_ask?: string };
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      if (!isProjectRootAuthoritySafe(projectRoot)) {
        const reason = projectRootAuthorityFailure(projectRoot) || "unsafe_project_root";
        return { content: [{ type: "text", text: `workpoint resume blocked → unsafe project_root (${reason}); ignore stale packets and follow latest operator instruction.` }], details: { ok: false, status: "blocked", failure_class: "scope_mismatch", project_root: projectRoot, reason, next_tools: ["focusa_project_identity", "focusa_tool_doctor"] } } as any;
      }
      const payload = { workpoint_id: p.workpoint_id, mode: p.mode || "compact_prompt", continuity_id: p.continuity_id || ensureContinuityId(projectRoot), session_id: p.session_id || S.sessionFrameKey, project_root: projectRoot, current_ask: p.current_ask || S.currentAsk?.text || "", session_identity: await buildFocusaSessionIdentity(projectRoot, "session_switch", { continuityId: p.continuity_id, sessionId: p.session_id }) };
      const res = await focusaFetchDetailed("/workpoint/resume", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      const rejected = res.body?.status === "rejected_scope_mismatch";
      const recovery = scopeRecoveryContext(res.body || {}, projectRoot, payload.continuity_id, "workpoint_resume");
      if (!res.ok && res.body?.failure_class === "hot_path_timeout") {
        const fallback = stampWorkpointPacketForCurrentPiSession({
          ...(S.activeWorkpointPacket || {}),
          status: "timeout_preserved",
          canonical: false,
          degraded: true,
          failure_class: "hot_path_timeout",
          project_root: projectRoot,
          continuity_id: payload.continuity_id,
          session_id: payload.session_id,
          mission: S.activeWorkpointPacket?.mission || S.activeWorkpointSummary || "Workpoint resume timed out before a canonical packet was returned",
          next_slice: S.activeWorkpointPacket?.next_slice || "Retry focusa_workpoint_resume after focusa_tool_doctor/resource_mode, or create a fresh focusa_workpoint_checkpoint from current operator/repo context.",
          preserved_at: new Date().toISOString(),
          next_step_hint: "Retry focusa_workpoint_resume after focusa_tool_doctor/resource_mode; if no canonical packet exists, checkpoint the current mission before treating state as canonical.",
        });
        S.activeWorkpointPacket = fallback;
        S.activeWorkpointSummary = `${String(fallback.mission || "Workpoint resume")} (noncanonical timeout fallback)`;
        try { S.pi?.appendEntry("focusa-workpoint-timeout-fallback", fallback); } catch { /* best effort */ }
        persistState();
        return {
          content: [{ type: "text", text: timeoutPreservedText("workpoint resume", "local fallback") }],
          details: { ok: false, status: "timeout_preserved", endpoint: "/workpoint/resume", canonical: false, degraded: true, failure_class: "hot_path_timeout", fallback_packet: compactFallbackPacket(fallback), scope_recovery_context: compactApiEcho(recovery?.details || null), request: compactApiEcho(payload), response: compactApiEcho(res.body), next_tools: ["focusa_tool_doctor", "focusa_resource_mode", "focusa_workpoint_resume", "focusa_traverse"] },
        } as any;
      }
      const text = res.ok && !rejected
        ? [`workpoint resume → ${summarizeWorkpointResponse(res.body)}\n${String(res.body?.rendered_summary || "")}`.trim(), recovery?.text].filter(Boolean).join("\n")
        : rejected
          ? [`workpoint resume rejected: project_root mismatch. Ignore packet; follow latest operator instruction and current repo.`, recovery?.text].filter(Boolean).join("\n")
          : [`workpoint resume unavailable → ${explainWorkLoopResult(res, "resume failed")}`, recovery?.text].filter(Boolean).join("\n");
      const v2 = res.body?.resume_packet_v2 || null;
      const canonical = res.body?.canonical === true;
      const actionAuthority = res.body?.action_authority_for_current_ask !== false && v2?.action_authority_for_current_ask !== false;
      const matchesCurrentAskScope = res.body?.matches_current_ask_scope !== false && v2?.matches_current_ask_scope !== false;
      const scopeConflictReason = String(res.body?.scope_conflict_reason || v2?.scope_conflict_reason || "none");
      const authoritySuppressed = canonical && !actionAuthority;
      const recoveryPacket = authoritySuppressed ? {
        status: "action_authority_suppressed",
        authority: "operator_and_current_project_context",
        canonical: true,
        canonical_for_saved_scope: true,
        action_authority_for_current_ask: false,
        matches_current_ask_scope: false,
        degraded: true,
        reason: scopeConflictReason,
        project_root: projectRoot,
        continuity_id: payload.continuity_id,
        safe_next_action: "verify/rebind the current operator-indicated project, then checkpoint/resume a Workpoint in that corrected scope before file/API action",
        next_tools: ["focusa_project_verify", "focusa_project_identity", "focusa_workpoint_checkpoint", "focusa_workpoint_resume"],
        do_not_use: ["saved_scope_as_current_action_authority", "transcript_tail_as_authority", "cross_project_packets"],
      } : canonical ? null : {
        status: "recovery_required",
        authority: "operator_and_current_project_context",
        canonical: false,
        degraded: true,
        reason: res.body?.failure_class || res.body?.status || (rejected ? "scope_mismatch" : "no_canonical_workpoint_packet"),
        project_root: projectRoot,
        continuity_id: payload.continuity_id,
        safe_next_action: "create a fresh focusa_workpoint_checkpoint from the current operator ask, project root, target objects, verified evidence, blockers, and exact next action before treating continuation state as canonical",
        next_tools: ["focusa_project_identity", "focusa_trajectory_view", "focusa_workpoint_checkpoint", "focusa_workpoint_resume"],
        do_not_use: ["transcript_tail_as_authority", "cross_project_packets", "noncanonical_resume_as_truth"],
      };
      const baseToolResult = res.body?.details?.tool_result_v1 || v2?.details?.tool_result_v1 || { ok: res.ok && !rejected && canonical, status: res.ok ? String(res.body?.status || "completed") : String(res.status), canonical, degraded: res.body?.degraded === true || !canonical || rejected, failure_class: res.body?.failure_class || (rejected ? "scope_mismatch" : canonical ? null : "frame_unavailable"), retry: { safe: res.ok && !rejected, posture: canonical ? "safe_retry" : "check_side_effects_first" }, side_effects: [], evidence_refs: [], next_tools: recoveryPacket?.next_tools || res.body?.next_tools || ["focusa_workpoint_resume", "focusa_trajectory_view", "focusa_traverse"] };
      const toolResult = authoritySuppressed ? { ...baseToolResult, ok: false, degraded: true, failure_class: baseToolResult.failure_class || "scope_conflict", canonical_for_saved_scope: true, matches_current_ask_scope: matchesCurrentAskScope, action_authority_for_current_ask: false, scope_conflict_reason: scopeConflictReason, retry: { safe: false, posture: "do_not_retry_unchanged" }, next_tools: recoveryPacket?.next_tools || baseToolResult.next_tools } : baseToolResult;
      const authorityText = authoritySuppressed ? `\naction authority suppressed → ${scopeConflictReason}; saved Workpoint remains canonical_for_saved_scope=true.` : "";
      const recoveryText = recoveryPacket ? `\nrecovery → ${recoveryPacket.safe_next_action}` : "";
      return {
        content: [{ type: "text", text: `${text}${authorityText}${recoveryText}` }],
        details: { ok: toolResult.ok, status: res.status, endpoint: "/workpoint/resume", canonical, canonical_for_saved_scope: canonical, matches_current_ask_scope: matchesCurrentAskScope, action_authority_for_current_ask: actionAuthority, scope_conflict_reason: scopeConflictReason, degraded: res.body?.degraded === true || !canonical || authoritySuppressed, failure_class: toolResult.failure_class || null, recovery_packet: compactApiEcho(recoveryPacket), scope_recovery_context: compactApiEcho(recovery?.details || null), resume_packet_v2: compactApiEcho(v2), rendered_summary: String(res.body?.rendered_summary || "").slice(0, 240), tool_result_v1: toolResult, next_tools: (toolResult.next_tools || recoveryPacket?.next_tools || res.body?.next_tools || ["focusa_workpoint_resume", "focusa_trajectory_view", "focusa_traverse"]).slice(0, 4), request: compactApiEcho(payload), response: compactApiEcho(res.body) },
      };
    },
  });

  // ── Spec80 LLM-native tree/metacog tools ─────────────────────────────────

  function spec80ErrorCode(result: { ok: boolean; status: number; body: any | null }): string {
    if (result.ok) return "OK";
    const bodyCode = String(result.body?.code || result.body?.failure_class || result.body?.error || "").trim();
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

  function summarizeValue(value: unknown): string {
    if (value === null || value === undefined) return "";
    if (typeof value === "string") return value.length > 160 ? `${value.slice(0, 157)}…` : value;
    if (typeof value === "number" || typeof value === "boolean") return String(value);
    if (Array.isArray(value)) return `[${value.slice(0, 4).map(summarizeValue).filter(Boolean).join(", ")}]`;
    if (typeof value === "object") {
      const record = value as Record<string, any>;
      const label = record.node_id || record.workpoint_id || record.id || record.anchor || record.label || record.title;
      const kind = record.node_type || record.kind || record.status;
      const payload = record.payload && typeof record.payload === "object" ? record.payload as Record<string, any> : null;
      const summary = record.summary || record.mission || record.next_slice || record.goal || record.content_ref || payload?.content_ref || payload?.summary || payload?.reason || record.created_at;
      const parts = [label, kind, summary].map(summarizeValue).filter(Boolean);
      if (parts.length) return parts.join(" | ");
      try {
        const json = JSON.stringify(record);
        return json.length > 180 ? `${json.slice(0, 177)}…` : json;
      } catch {
        return "[object]";
      }
    }
    return String(value);
  }

  function summarizeArray(values: unknown[], limit = 3): string {
    if (!Array.isArray(values) || values.length === 0) return "none";
    return values.slice(0, limit).map(summarizeValue).filter(Boolean).join("; ") || "none";
  }

  function summarizeTraverseItems(items: unknown[], limit = 6): string {
    if (!Array.isArray(items) || items.length === 0) return "items=none";
    return items.slice(0, limit).map((item, index) => {
      const record = item as Record<string, any>;
      const data = (record && typeof record === "object" && record.data && typeof record.data === "object") ? record.data as Record<string, any> : record;
      const anchor = record?.anchor || data?.node_id || data?.workpoint_id || data?.id || `#${index + 1}`;
      const kind = record?.kind || data?.node_type || data?.status || data?.kind || "item";
      const payload = data?.payload && typeof data.payload === "object" ? data.payload as Record<string, any> : null;
      const summary = record?.summary || data?.summary || data?.mission || data?.next_slice || data?.goal || data?.content_ref || payload?.content_ref || payload?.summary || payload?.reason || data?.created_at || "";
      return `${index + 1}. ${summarizeValue(anchor)} ${summarizeValue(kind)} ${summarizeValue(summary)}`.trim();
    }).join("\n");
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
      const captureId = String(res.body?.capture_id || "stored");
      const lessonLine = compactText(req.content, "no lesson content", 120);
      const relevanceReason = compactText(req.rationale || (req.evidence_refs.length ? `evidence=${req.evidence_refs[0]}` : "captured for future retrieval"), "captured for future retrieval", 100);
      return spec80Result(
        "focusa_metacog_capture",
        "/v1/metacognition/capture",
        { ...req, writer_id: res.writerId || null, compact_lesson_line: { lesson: lessonLine, why_relevant: relevanceReason, rehydrate_id: captureId } },
        res,
        `metacog capture: id=${captureId} lesson="${lessonLine}" why="${relevanceReason}" rehydrate_id=${captureId}`,
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
      const topCapture = String(top?.capture_id || "none");
      const topLesson = compactText(top?.summary || top?.content || top?.signal || "no lesson content", "no lesson content", 120);
      const topWhy = compactText(top?.rationale || top?.why_relevant || (top?.score !== undefined ? `retrieval_score=${String(top.score)}` : `matched current_ask=${req.current_ask}`), "matched current ask", 100);
      return spec80Result(
        "focusa_metacog_retrieve",
        "/v1/metacognition/retrieve",
        { ...req, compact_top_lesson: total > 0 ? { lesson: topLesson, why_relevant: topWhy, rehydrate_id: topCapture } : null },
        res,
        total > 0
          ? `metacog retrieve: candidates=${total} top_lesson="${topLesson}" why="${topWhy}" rehydrate_id=${topCapture}`
          : `metacog retrieve: candidates=0 lesson="none" why="no prior signals matched" rehydrate_id=none`,
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


  // ── Surgical traversal facade (Spec96) ───────────────────────────────────

  pi.registerTool({
    name: "focusa_traverse",
    label: "Focusa Traverse",
    description: "Read-only surgical traversal across large Focusa surfaces. Use for bounded lineage, ontology, evidence, telemetry, Workpoint, and registry slices instead of full payloads.",
    parameters: strictObject({
      surface: Type.String({ minLength: 1, maxLength: 80, description: "Surface: lineage|ontology|focus_stack|workpoints|evidence|telemetry|tool_registry etc." }),
      selector: Type.Optional(Type.String({ maxLength: 80, description: "Selector: window|head|path|children|neighborhood|summaries|search|recent|tags_verify." })),
      anchor: Type.Optional(Type.String({ maxLength: SPEC81_LIMITS.id, description: "Optional anchor id/tag/ref." })),
      query: Type.Optional(Type.String({ maxLength: SPEC81_LIMITS.currentAsk, description: "Optional search/filter query." })),
      cursor: Type.Optional(Type.String({ maxLength: 80, description: "Optional cursor/offset token." })),
      limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 200, description: "Bounded result limit." })),
      depth: Type.Optional(Type.Integer({ minimum: 1, maximum: 64, description: "Traversal depth cap." })),
      radius: Type.Optional(Type.Integer({ minimum: 1, maximum: 8, description: "Neighborhood radius cap." })),
      fields: Type.Optional(Type.Array(Type.String({ maxLength: 80 }), { maxItems: 16, description: "Optional projected fields." })),
      tags: Type.Optional(Type.Array(Type.Union([
        Type.String({ maxLength: 240 }),
        Type.Object({
          anchor: Type.Optional(Type.String({ maxLength: 160 })),
          tag: Type.String({ maxLength: 240 }),
          ordinal: Type.Optional(Type.Integer({ minimum: 0, maximum: 100000 })),
        }),
      ]), { maxItems: 32, description: "Optional traversal tags to verify as strings or TraverseTagRef-style objects." })),
      tag_mode: Type.Optional(Type.Union([Type.Literal("item"), Type.Literal("range"), Type.Literal("window"), Type.Literal("surface"), Type.Literal("mixed")], { description: "Traversal tag mode; defaults mixed." })),
      include_payload: Type.Optional(Type.Boolean({ description: "Spec96 alias for explicit cold opt-in larger payload; defaults false." })),
      include_full_payload: Type.Optional(Type.Boolean({ description: "Compatibility alias for explicit cold opt-in larger payload; defaults false." })),
      include_rehydrate_refs: Type.Optional(Type.Boolean({ description: "Include rehydrate refs for omitted/cold slices." })),
      budget_tokens: Type.Optional(Type.Integer({ minimum: 1, maximum: 20000, description: "Optional token budget hint." })),
      session_identity: Type.Optional(Type.Any({ description: "Optional FocusaSessionIdentity envelope for scoped traversal." })),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_traverse", params, ["surface", "selector", "anchor", "query", "cursor", "limit", "depth", "radius", "fields", "tags", "tag_mode", "include_payload", "include_full_payload", "include_rehydrate_refs", "budget_tokens", "session_identity"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult("focusa_traverse", "/v1/traverse", params as Record<string, any>, "traverse", keyCheck.error);
      }
      const raw = keyCheck.value as { surface: string; selector?: string; anchor?: string; query?: string; cursor?: string; limit?: number; depth?: number; radius?: number; fields?: string[]; tags?: any[]; tag_mode?: string; include_payload?: boolean; include_full_payload?: boolean; include_rehydrate_refs?: boolean; budget_tokens?: number; session_identity?: any };
      const surfaceCheck = validateRequiredString("surface", raw.surface, 80);
      if (!surfaceCheck.ok) return spec80ValidationResult("focusa_traverse", "/v1/traverse", raw as Record<string, any>, "traverse", surfaceCheck.error);
      const selectorCheck = validateOptionalString("selector", raw.selector, 80);
      if (!selectorCheck.ok) return spec80ValidationResult("focusa_traverse", "/v1/traverse", raw as Record<string, any>, "traverse", selectorCheck.error);
      const anchorCheck = validateOptionalString("anchor", raw.anchor, SPEC81_LIMITS.id);
      if (!anchorCheck.ok) return spec80ValidationResult("focusa_traverse", "/v1/traverse", raw as Record<string, any>, "traverse", anchorCheck.error);
      const queryCheck = validateOptionalString("query", raw.query, SPEC81_LIMITS.currentAsk);
      if (!queryCheck.ok) return spec80ValidationResult("focusa_traverse", "/v1/traverse", raw as Record<string, any>, "traverse", queryCheck.error);
      const cursorCheck = validateOptionalString("cursor", raw.cursor, 80);
      if (!cursorCheck.ok) return spec80ValidationResult("focusa_traverse", "/v1/traverse", raw as Record<string, any>, "traverse", cursorCheck.error);
      const fieldsCheck = validateStringArray("fields", raw.fields, { maxItems: 16, itemMaxLength: 80 });
      if (!fieldsCheck.ok) return spec80ValidationResult("focusa_traverse", "/v1/traverse", raw as Record<string, any>, "traverse", fieldsCheck.error);
      const tags = Array.isArray(raw.tags) ? raw.tags.slice(0, 32).map((tag) => {
        if (typeof tag === "string") return tag.slice(0, 240);
        if (tag && typeof tag === "object" && typeof tag.tag === "string") return { ...tag, tag: String(tag.tag).slice(0, 240) };
        return tag;
      }) : [];
      if (raw.tags !== undefined && !Array.isArray(raw.tags)) return spec80ValidationResult("focusa_traverse", "/v1/traverse", raw as Record<string, any>, "traverse", "tags must be an array of strings or TraverseTagRef objects");
      const selector = selectorCheck.value || "window";
      const req = {
        surface: surfaceCheck.value,
        selector,
        anchor: anchorCheck.value,
        query: queryCheck.value,
        cursor: cursorCheck.value,
        limit: raw.limit !== undefined ? Math.max(1, Math.min(200, Math.trunc(Number(raw.limit)))) : undefined,
        depth: raw.depth !== undefined ? Math.max(1, Math.min(64, Math.trunc(Number(raw.depth)))) : undefined,
        radius: raw.radius !== undefined ? Math.max(1, Math.min(8, Math.trunc(Number(raw.radius)))) : undefined,
        fields: fieldsCheck.value,
        tags,
        tag_mode: raw.tag_mode,
        // The API treats include_payload as a serde alias for include_full_payload.
        // Send exactly one canonical field; sending both aliases makes Rust reject the body as duplicate.
        include_full_payload: raw.include_full_payload === true || raw.include_payload === true,
        include_rehydrate_refs: raw.include_rehydrate_refs === true,
        budget_tokens: raw.budget_tokens !== undefined ? Math.max(1, Math.min(20000, Math.trunc(Number(raw.budget_tokens)))) : undefined,
        session_identity: raw.session_identity,
      };
      const endpoint = selector === "tags_verify" ? "/traverse/verify-tags" : "/traverse";
      const res = await callSpec80Tool("focusa_traverse", endpoint, req, { method: "POST" });
      const items = Array.isArray(res.body?.items) ? res.body.items : [];
      const traversal = res.body?.traversal || {};
      return spec80Result(
        "focusa_traverse",
        endpoint === "/traverse" ? "/v1/traverse" : "/v1/traverse/verify-tags",
        req,
        res,
        `traverse: surface=${req.surface} selector=${selector} returned=${items.length}/${String(traversal.total ?? items.length)} truncated=${Boolean(traversal.truncated)}
next_cursor=${String(traversal.next_cursor ?? "none")} tags=${Array.isArray(res.body?.tags) ? res.body.tags.length : 0} verified=${Array.isArray(res.body?.verified_tags) ? res.body.verified_tags.length : 0} stale=${Array.isArray(res.body?.stale_tags) ? res.body.stale_tags.length : 0}
${summarizeTraverseItems(items, 8)}
next_tools=focusa_traverse,focusa_trajectory_view,focusa_workpoint_resume`,
        "traverse",
      );
    },
  });

  // ── Lineage Intelligence (LI) /tree first-class tools ────────────────────

  pi.registerTool({
    name: "focusa_lineage_tree",
    label: "Lineage Tree",
    description: "Fetch a bounded Focusa lineage window for /tree-aware reasoning. Full tree requires explicit cold opt-in.",
    parameters: Type.Object({
      session_id: Type.Optional(Type.String({ description: "Optional session id scoping hint." })),
      max_nodes: Type.Optional(Type.Number({ description: "Optional node cap (default 50)." })),
      include_full_payload: Type.Optional(Type.Boolean({ description: "Explicit cold opt-in for larger lineage payload." })),
    }),
    async execute(_id, params) {
      const { session_id, max_nodes, include_full_payload } = params as { session_id?: string; max_nodes?: number; include_full_payload?: boolean };
      const cap = Math.max(1, Math.min(include_full_payload ? 2000 : 200, Number(max_nodes || 50)));
      const queryParts = [`selector=window`, `limit=${encodeURIComponent(String(cap))}`];
      if (session_id) queryParts.push(`session_id=${encodeURIComponent(session_id)}`);
      if (include_full_payload === true) queryParts.push(`include_full_payload=true`);
      const query = `?${queryParts.join("&")}`;
      const res = await focusaFetchDetailed(`/lineage/tree${query}`);
      if (!res.ok || !res.body) {
        return {
          content: [{ type: "text", text: `lineage tree → ${explainWorkLoopResult(res, "ok")}` }],
          details: { ok: false, status: res.status, response: compactApiEcho(res.body) ?? null },
        } as any;
      }

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
          returned: Number(res.body?.returned || nodes.length),
          truncated: Boolean(res.body?.truncated),
          next_cursor: res.body?.next_cursor ?? res.body?.traversal?.next_cursor ?? null,
          window_kind: String(res.body?.window_kind || res.body?.traversal?.window_kind || "window"),
          cold_opt_in: include_full_payload === true,
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
      const cap = Math.max(1, Math.min(50, Number(max_candidates || 12)));
      const queryParts = [`selector=summaries`, `limit=${encodeURIComponent(String(cap))}`];
      if (session_id) queryParts.push(`session_id=${encodeURIComponent(session_id)}`);
      const query = `?${queryParts.join("&")}`;
      const res = await focusaFetchDetailed(`/lineage/tree${query}`);
      if (!res.ok || !res.body) {
        return {
          content: [{ type: "text", text: `li extract → ${explainWorkLoopResult(res, "ok")}` }],
          details: { ok: false, status: res.status, response: compactApiEcho(res.body) ?? null },
        } as any;
      }

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

  // ── Spec92 first-class prediction tools ─────────────────────────────────
  pi.registerTool({
    name: "focusa_predict_record",
    label: "Record Prediction",
    description: "Record a bounded, inspectable Focusa prediction. Predictions guide decisions; they never override operator steering.",
    parameters: Type.Object({
      prediction_type: Type.String({ description: "Prediction type, e.g. next_action_success|tool_choice|release_failure|stale_state|context_relevance|token_risk|cache_hit|drift_risk|workpoint_resume_success|compaction_recovery" }),
      predicted_outcome: Type.String({ description: "Predicted outcome." }),
      confidence: Type.Number({ description: "Confidence from 0.0 to 1.0." }),
      recommended_action: Type.String({ description: "Recommended action if this prediction matters." }),
      why: Type.String({ description: "Evidence-calibrated explanation." }),
      context_refs: Type.Optional(Type.Array(Type.String({ description: "Evidence refs or handles." }))),
      ontology_context: Type.Optional(Type.Any({ description: "Bounded ontology refs: object_refs, action_refs, tool_refs, evidence_refs, relation_refs." })),
      project_root: Type.Optional(Type.String({ description: "Optional project root to bind prediction trajectory scope; auto-filled when omitted." })),
      continuity_id: Type.Optional(Type.String({ description: "Optional continuity id to bind prediction trajectory scope; auto-filled when omitted." })),
    }),
    async execute(_id, params) {
      const payload = params && typeof params === "object" ? { ...(params as any) } : params;
      if (payload && typeof payload === "object") {
        const projectRoot = normalizeProjectRoot(payload.project_root || S.lastProjectIdentity?.project_root || S.sessionCwd || process.cwd());
        const continuityId = String(payload.continuity_id || S.continuityId || ensureContinuityId(projectRoot) || "").trim();
        if (projectRoot) payload.project_root = projectRoot;
        if (continuityId) payload.continuity_id = continuityId;
        if (!payload.session_identity && projectRoot) {
          payload.session_identity = await buildFocusaSessionIdentity(projectRoot, "manual", { continuityId, sessionId: S.sessionFrameKey });
        }
      }
      const res = await focusaFetchDetailed("/predictions", { method: "POST", body: JSON.stringify(payload) });
      const body = res.body || {};
      if (!res.ok) return blockedToolResponse("focusa_predict_record", "prediction", `prediction record blocked → ${explainWorkLoopResult(res, "prediction write unavailable")}`, body.failure_class || "daemon_unavailable", body, ["focusa_tool_doctor", "focusa_resource_mode", "focusa_predict_recent"]);
      const prediction = body.prediction || {};
      const predictionId = String(prediction.prediction_id || body.prediction_id || "unknown");
      const predictionScope = `project=${String(prediction.project_root || payload?.project_root || "unknown")} continuity=${String(prediction.continuity_id || payload?.continuity_id || "unknown")}`;
      const predictionEvalHint = `focusa_predict_evaluate prediction_id=${predictionId}`;
      const predictionConfidence = String(prediction.confidence ?? payload?.confidence ?? "unknown");
      const toolResult = body.details?.tool_result_v1 || focusaToolResult({ ok: true, status: "completed", summary: `prediction record → ${body.status || "accepted"} id=${predictionId}`, tool: "focusa_predict_record", family: "prediction", side_effects: ["prediction_store"], evidence_refs: [], next_tools: ["focusa_predict_evaluate", "focusa_predict_recent"], raw: body });
      return { content: [{ type: "text", text: `prediction record → ${body.status || "accepted"} id=${predictionId} confidence=${predictionConfidence} scope=(${predictionScope}) eval_hint="${predictionEvalHint}"` }], details: { ...body, compact_actionability: { prediction_id: predictionId, confidence: predictionConfidence, scope: predictionScope, evaluation_hint: predictionEvalHint }, tool_result_v1: toolResult, next_tools: toolResult.next_tools } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_predict_recent",
    label: "Recent Predictions",
    description: "List recent bounded Focusa prediction records.",
    parameters: Type.Object({ limit: Type.Optional(Type.Number({ description: "Recent prediction count, max 100." })) }),
    async execute(_id, params) {
      const limit = Math.max(1, Math.min(100, Number((params as any).limit || 20)));
      const res = await focusaFetchDetailed(`/predictions/recent?limit=${limit}`);
      const body = res.body || {};
      if (!res.ok) return blockedToolResponse("focusa_predict_recent", "prediction", `predictions recent blocked → ${explainWorkLoopResult(res, "prediction read unavailable")}`, body.failure_class || "daemon_unavailable", body, ["focusa_tool_doctor", "focusa_resource_mode"]);
      const predictions = Array.isArray(body.predictions) ? body.predictions : [];
      const count = predictions.length;
      const actionable = predictions.slice().reverse().find((item: any) => item && !item.evaluated_at && item.prediction_id) || predictions.at(-1) || null;
      const actionLine = actionable
        ? ` next_id=${String(actionable.prediction_id)} confidence=${String(actionable.confidence ?? "unknown")} scope=(project=${String(actionable.project_root || "unknown")} continuity=${String(actionable.continuity_id || "unknown")}) eval_hint="focusa_predict_evaluate prediction_id=${String(actionable.prediction_id)}"`
        : " next_id=none eval_hint=record_prediction_first";
      const toolResult = body.details?.tool_result_v1 || focusaToolResult({ ok: true, status: "completed", summary: `predictions recent → ${count}${actionable ? ` next_id=${String(actionable.prediction_id)}` : ""}`, tool: "focusa_predict_recent", family: "prediction", side_effects: [], evidence_refs: [], next_tools: ["focusa_predict_record", "focusa_predict_evaluate"], raw: body });
      return { content: [{ type: "text", text: `predictions recent → ${count}${actionLine}` }], details: { ...body, compact_actionability: actionable ? { prediction_id: String(actionable.prediction_id), confidence: actionable.confidence ?? null, project_root: actionable.project_root || null, continuity_id: actionable.continuity_id || null, evaluation_hint: `focusa_predict_evaluate prediction_id=${String(actionable.prediction_id)}` } : null, tool_result_v1: toolResult, next_tools: toolResult.next_tools } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_predict_evaluate",
    label: "Evaluate Prediction",
    description: "Evaluate a Focusa prediction against an actual outcome and optional score.",
    parameters: Type.Object({
      prediction_id: Type.String({ description: "Prediction id to evaluate." }),
      actual_outcome: Type.String({ description: "Observed actual outcome." }),
      score: Type.Optional(Type.Number({ description: "Score 0.0 to 1.0." })),
      learning_signal_ref: Type.Optional(Type.String({ description: "Optional metacog/learning signal ref." })),
    }),
    async execute(_id, params) {
      const { prediction_id, ...payload } = params as any;
      const res = await focusaFetchDetailed(`/predictions/${encodeURIComponent(prediction_id)}/evaluate`, { method: "POST", body: JSON.stringify(payload) });
      const body = res.body || {};
      if (!res.ok) return blockedToolResponse("focusa_predict_evaluate", "prediction", `prediction evaluate blocked → ${explainWorkLoopResult(res, "prediction evaluation unavailable")}`, body.failure_class || (res.status === 404 ? "not_found" : "daemon_unavailable"), body, ["focusa_predict_recent", "focusa_predict_record", "focusa_tool_doctor"]);
      const toolResult = body.details?.tool_result_v1 || focusaToolResult({ ok: true, status: "completed", summary: `prediction evaluate → ${body.status || "accepted"}`, tool: "focusa_predict_evaluate", family: "prediction", side_effects: ["prediction_store", "metacog_capture_if_score_high"], evidence_refs: [], next_tools: ["focusa_predict_stats", "focusa_metacog_retrieve", "focusa_predict_record"], raw: body });
      return { content: [{ type: "text", text: `prediction evaluate → ${body.status || "accepted"}` }], details: { ...body, tool_result_v1: toolResult, next_tools: toolResult.next_tools } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_predict_stats",
    label: "Prediction Stats",
    description: "Report Focusa prediction accuracy/calibration stats.",
    parameters: Type.Object({}),
    async execute() {
      const res = await focusaFetchDetailed("/predictions/stats");
      const body = res.body || {};
      if (!res.ok) return blockedToolResponse("focusa_predict_stats", "prediction", `prediction stats blocked → ${explainWorkLoopResult(res, "prediction stats unavailable")}`, body.failure_class || "daemon_unavailable", body, ["focusa_predict_recent", "focusa_tool_doctor"]);
      const summary = String(body.summary || body.status || "available");
      const toolResult = body.details?.tool_result_v1 || focusaToolResult({ ok: true, status: "completed", summary: `prediction stats → ${summary}`, tool: "focusa_predict_stats", family: "prediction", side_effects: [], evidence_refs: [], next_tools: ["focusa_predict_record", "focusa_predict_recent"], raw: body });
      return { content: [{ type: "text", text: `prediction stats → ${summary}` }], details: { ...body, tool_result_v1: toolResult, next_tools: toolResult.next_tools } } as any;
    },
  });

}
