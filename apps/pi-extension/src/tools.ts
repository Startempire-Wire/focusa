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
import { S, checkFocusa, focusaFetch, focusaPost, ensurePiFrame, getFocusState, ensureContinuityId, isProjectRootAuthoritySafe, projectRootAuthorityFailure, buildFocusaSessionIdentity } from "./state.js";
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
  | "frame_unavailable"
  | "daemon_unavailable"
  | "stale_runtime_registry"
  | "resource_exhausted"
  | "null_response"
  | "hot_path_timeout"
  | "cold_path_timeout"
  | "writer_conflict"
  | "scope_mismatch"
  | "approval_required"
  | "permission_denied"
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
  side_effects: string[];
  evidence_refs: string[];
  next_tools: string[];
  ontology_candidate_delta_refs?: string[];
  error?: { field?: string; code?: string; message?: string; allowed_values?: string[] } | null;
  raw?: unknown;
}

function inferFailureClass(status: FocusaToolStatus, summary: string, message?: string | null, canonical?: boolean, degraded?: boolean): FocusaFailureClass | null {
  const text = `${summary} ${message || ""}`.toLowerCase();
  if (text.includes("no active pi frame") || text.includes("no active frame") || text.includes("frame recovery")) return "frame_unavailable";
  if (text.includes("payload_equal=false") || text.includes("live registry payload differs") || text.includes("stale daemon registry") || text.includes("stale runtime registry")) return "stale_runtime_registry";
  if (text.includes("oom") || text.includes("out of memory") || text.includes("resource exhausted") || text.includes("killed process")) return "resource_exhausted";
  if (text.includes("null response") || text.includes("response=null") || text.includes("body=null")) return "null_response";
  if (status === "validation_rejected" || text.includes("validation_rejected") || text.includes("rejected")) return "validation_rejected";
  if (status === "offline" || text.includes("daemon unavailable") || text.includes("focusa offline") || text.includes("connection refused")) return "daemon_unavailable";
  if (text.includes("timeout") || text.includes("timed out") || text.includes("abort")) {
    return /(cold|deep|replay|worktree|diagnostic)/.test(text) ? "cold_path_timeout" : "hot_path_timeout";
  }
  if (text.includes("claimed by another writer") || text.includes("writer_conflict") || text.includes("controlled by another session")) return "writer_conflict";
  if (text.includes("project_root mismatch") || text.includes("scope mismatch") || text.includes("cross-project")) return "scope_mismatch";
  if (text.includes("approval required") || text.includes("requires approved")) return "approval_required";
  if (text.includes("permission denied") || text.includes("unauthorized") || text.includes("forbidden")) return "permission_denied";
  if (text.includes("read model lag") || text.includes("pending") || text.includes("not yet visible")) return "read_model_lag";
  if (degraded || canonical === false || text.includes("non-canonical") || text.includes("noncanonical")) return "noncanonical_fallback";
  if (status === "blocked" || status === "error") return "unknown_ambiguous_completion";
  return null;
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
  const summary = params.summary.slice(0, 500);
  const failureClass = params.failure_class ?? inferFailureClass(params.status, summary, params.error?.message, canonical, degraded);
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
    side_effects: params.side_effects ?? [],
    evidence_refs: params.evidence_refs ?? [],
    next_tools: params.next_tools ?? [],
    ontology_candidate_delta_refs: params.ontology_candidate_delta_refs ?? [],
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
    next_tools: status === "offline" ? [] : family === "workpoint" ? ["focusa_workpoint_resume"] : [],
    ontology_candidate_delta_refs: ontologyCandidateDeltaRefs(tool, result, status),
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
    case "frame_unavailable":
      return "No active/scoped Pi frame";
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

function formatNonCriticalWriteFailure(slotLabel: string, reason: PushDeltaFailureReason, apiReason?: string): string {
  const base = formatPushDeltaFailure(reason);
  const detail = apiReason ? ` Detail: ${apiReason}` : "";
  if (reason === "no_active_frame" || reason === "frame_unavailable") return `⚠️ ${base} — ${slotLabel} NOT recorded. Frame recovery was attempted; scratchpad fallback is safest until /focusa-status confirms a scoped frame.${detail}`;
  if (reason === "scope_mismatch" || reason === "read_model_lag") return `⚠️ ${base} — ${slotLabel} NOT recorded. Scoped frame/continuity is stale; use latest operator instruction, checkpoint a fresh Workpoint for the verified project, and do not retry unchanged.${detail}`;
  if (reason === "offline") return `⚠️ ${base} — ${slotLabel} NOT recorded. Retry when Focusa is reachable.${detail}`;
  if (reason === "validation_rejected") return `⚠️ ${base} — ${slotLabel} NOT recorded. Distill wording or use scratchpad.${detail}`;
  return `⚠️ ${base} — ${slotLabel} NOT recorded.${detail}`;
}

function namedSlotFallback(slotLabel: string, kind: string, reason: PushDeltaFailureReason, payload: string, apiReason?: string): { text: string; saved: boolean; turn: number } {
  const fallback = mirrorFailedFocusWrite(kind, reason, payload, { api_reason: apiReason });
  const fallbackText = fallback.saved ? ` Saved to scratchpad fallback (turn ${fallback.turn}).` : " Scratchpad fallback also failed.";
  return { text: `${formatNonCriticalWriteFailure(slotLabel, reason, apiReason)}${fallbackText}`, saved: fallback.saved, turn: fallback.turn };
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
    // Refresh frame identity before writes; stale paused Pi frames are a common
    // source of reducer rejections and scratchpad fallbacks after rescope/compact.
    await getFocusState().catch(() => null);
    const postUpdate = () => focusaFetch("/focus/update", {
      method: "POST",
      body: JSON.stringify({
        frame_id: S.activeFrameId,
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
      if (!v.valid) {
        const fallback = namedSlotValidationFallback("intent", "intent", intent.trim(), v.reason);
        return { content: [{ type: "text", text: fallback.text }], details: { valid: false, intent, reason: "validation_rejected", scratch_saved: fallback.saved, scratch_turn: fallback.turn } } as any;
      }
      const result = await pushDelta({ intent: intent.trim() });
      if (result.ok) return { content: [{ type: "text", text: `Intent set: ${intent.slice(0, 100)}` }], details: { valid: true, reason: undefined, intent } };
      const fallback = namedSlotFallback("intent", "intent", result.reason, intent.trim(), result.api_reason);
      return { content: [{ type: "text", text: fallback.text }], details: { valid: false, intent, reason: result.reason, scratch_saved: fallback.saved, scratch_turn: fallback.turn } } as any;
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
      return { content: [{ type: "text", text: fallback.text }], details: { valid: false, focus, reason: result.reason, scratch_saved: fallback.saved, scratch_turn: fallback.turn } } as any;
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
      return { content: [{ type: "text", text: fallback.text }], details: { valid: false, step, reason: result.reason, scratch_saved: fallback.saved, scratch_turn: fallback.turn } } as any;
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
      return { content: [{ type: "text", text: fallback.text }], details: { valid: false, question, reason: result.reason, scratch_saved: fallback.saved, scratch_turn: fallback.turn } } as any;
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
      return { content: [{ type: "text", text: fallback.text }], details: { valid: false, result, reason: writeResult.reason, scratch_saved: fallback.saved, scratch_turn: fallback.turn } } as any;
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
      return { content: [{ type: "text", text: fallback.text }], details: { valid: false, note, reason: result.reason, scratch_saved: fallback.saved, scratch_turn: fallback.turn } } as any;
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

  function explainWorkLoopResult(result: { ok: boolean; status: number; body: any | null }, fallback: string): string {
    if (result.ok) return fallback;
    const msg = String(result.body?.error || "").toLowerCase();
    const activeWriter = result.body?.active_writer ? ` (${result.body.active_writer})` : "";
    if (msg.includes("claimed by another writer")) return `blocked: loop controlled by another session${activeWriter}`;
    if (msg.includes("worktree is not clean")) return "blocked: worktree has uncommitted changes";
    if (msg.includes("missing required header")) return "blocked: controller identity header missing";
    if (result.body?.failure_class === "cold_path_timeout") return "blocked: cold route timed out; hot tools may still be healthy";
    if (result.body?.failure_class === "hot_path_timeout") return "blocked: hot route timed out";
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
      return { content: [{ type: "text", text }], details: { ok: result.ok, status: String(result.status), active_writer: activeWriter, authorship_mode: body.authorship_mode, preflight: { mutates: false, writer_required_for: ["control", "context", "checkpoint", "select_next"] }, response: body } } as any;
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
      const budget = loopStatus?.budget_remaining == null ? "unknown" : String(loopStatus.budget_remaining);
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
    name: "focusa_state_hygiene_doctor",
    label: "Focus State Hygiene Doctor",
    description: "Diagnose stale or duplicate Focus State signals without mutating state.",
    parameters: Type.Object({}),
    async execute() {
      const stack = await focusaFetchDetailed("/focus/stack", { method: "GET" });
      const frames = stack.body?.stack?.frames || [];
      const latest = Array.isArray(frames) ? frames[frames.length - 1] || {} : {};
      const notes = latest?.state?.notes || [];
      const result = { duplicate_candidates: RECENT_COGNITIVE_WRITE_KEYS.length, note_count: Array.isArray(notes) ? notes.length : 0, stale_candidates: [], recommended_action: "plan_before_apply" };
      return { content: [{ type: "text", text: `state hygiene doctor → duplicate_candidates=${result.duplicate_candidates} note_count=${result.note_count} recommended=${result.recommended_action}` }], details: { ok: stack.ok, status: String(stack.status), response: result } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_state_hygiene_plan",
    label: "Focus State Hygiene Plan",
    description: "Create a proposal-style hygiene plan; does not mutate Focus State.",
    parameters: Type.Object({ reason: Type.Optional(Type.String({ description: "Why hygiene is being considered." })) }),
    async execute(_id, params) {
      const p = params as any;
      const plan = { mutates: false, reason: String(p.reason || "operator requested hygiene plan"), actions: ["review duplicate_candidates", "prefer supersede/update over deletion", "apply only with explicit approval"] };
      return { content: [{ type: "text", text: `state hygiene plan → actions=${plan.actions.length} mutates=false` }], details: { ok: true, status: "completed", plan } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_state_hygiene_apply",
    label: "Focus State Hygiene Apply",
    description: "Approval-safe hygiene apply placeholder; requires approved=true and never deletes silently.",
    parameters: Type.Object({ approved: Type.Boolean({ description: "Must be true to apply proposal-safe hygiene." }), reason: Type.Optional(Type.String()) }),
    async execute(_id, params) {
      const p = params as any;
      if (p.approved !== true) return { content: [{ type: "text", text: "state hygiene apply blocked → approval required" }], details: { ok: false, status: "blocked", reason: "approval_required" } } as any;
      return { content: [{ type: "text", text: "state hygiene apply → no destructive changes; proposal-safe hygiene acknowledged" }], details: { ok: true, status: "no_op", mutates: false, reason: p.reason || "approved" } } as any;
    },
  });


  type SilentSessionAction = "list" | "start" | "reopen" | "kill" | "tail" | "send";
  const SILENT_SESSION_PREFIX = "focusa-silent";

  function silentSessionExec(args: string[], timeout = 5000): { ok: boolean; stdout: string; stderr: string; status: number | null } {
    try {
      const { spawnSync } = require("child_process");
      const r = spawnSync("tmux", args, { encoding: "utf8", timeout });
      return { ok: r.status === 0, stdout: r.stdout || "", stderr: r.stderr || "", status: r.status };
    } catch (err: any) {
      return { ok: false, stdout: "", stderr: String(err?.message || err), status: null };
    }
  }

  function silentSessionName(raw?: unknown): string {
    const base = String(raw || "default")
      .toLowerCase()
      .replace(/[^a-z0-9._:-]+/g, "-")
      .replace(/^-+|-+$/g, "")
      .slice(0, 80) || "default";
    return base.startsWith(SILENT_SESSION_PREFIX) ? base : `${SILENT_SESSION_PREFIX}-${base}`;
  }

  function listSilentSessions() {
    const r = silentSessionExec(["list-sessions", "-F", "#{session_name}\t#{session_attached}\t#{session_windows}\t#{session_created}"], 3000);
    if (!r.ok && /no server running|failed to connect/i.test(r.stderr)) return [];
    return r.stdout.split("\n").filter(Boolean).map((line: string) => {
      const [name, attached, windows, created] = line.split("\t");
      return { name, attached: attached === "1", windows: Number(windows || 0), created: Number(created || 0), attach_command: `tmux attach -t ${name}` };
    }).filter((session: any) => String(session.name || "").startsWith(SILENT_SESSION_PREFIX));
  }

  function defaultSilentSessionCommand(p: any, sessionName: string): string {
    const rootDir = String(p.root_dir || S.sessionCwd || process.cwd()).replace(/'/g, `'\\''`);
    const mission = String(p.mission || "Continue Focusa-governed ready beads using trajectory/workpoint context; stop on destructive risk.").replace(/'/g, `'\\''`);
    const bead = String(p.work_item_id || "").replace(/'/g, `'\\''`);
    const lowmem = p.lowmem === false ? "" : "curl -fsS --max-time 5 -X POST http://127.0.0.1:8787/v1/resource/mode -H 'Content-Type: application/json' --data '{\"action\":\"activate_lowmem\",\"reason\":\"SilentSession start\"}' >/tmp/focusa-silent-lowmem.json 2>/tmp/focusa-silent-lowmem.err || true; ";
    return `cd '${rootDir}' && ${lowmem}pi 'SilentSession ${sessionName}: ${mission}${bead ? ` Work item: ${bead}.` : ""} Use Focusa trajectory/workpoint/beads, record evidence, checkpoint often, and stop for destructive/high-risk actions.'`;
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
      ], { description: "SilentSession action. list is default; kill/send/start require approved=true." })),
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
        return { content: [{ type: "text", text }], details: { ok: true, status: "completed", sessions: sessionsBefore, count: sessionsBefore.length, next_tools: ["focusa_silent_sessions", "focusa_resource_mode", "focusa_work_loop_status"] } } as any;
      }

      if (action === "reopen") {
        if (!hasSession) return { content: [{ type: "text", text: `silent session not found → ${sessionName}` }], details: { ok: false, status: "not_found", failure_class: "frame_unavailable", session_name: sessionName, sessions: sessionsBefore } } as any;
        const tail = silentSessionExec(["capture-pane", "-p", "-t", sessionName, "-S", "-80"], 3000);
        return { content: [{ type: "text", text: `silent session reopen → ${sessionName}\nattach: tmux attach -t ${sessionName}` }], details: { ok: true, status: "completed", session_name: sessionName, attach_command: `tmux attach -t ${sessionName}`, tail: tail.stdout.slice(-4000), sessions: sessionsBefore } } as any;
      }

      if (action === "tail") {
        if (!hasSession) return { content: [{ type: "text", text: `silent session not found → ${sessionName}` }], details: { ok: false, status: "not_found", session_name: sessionName, sessions: sessionsBefore } } as any;
        const lines = Math.max(1, Math.min(400, Number(p.lines || 80)));
        const tail = silentSessionExec(["capture-pane", "-p", "-t", sessionName, "-S", `-${lines}`], 3000);
        return { content: [{ type: "text", text: tail.ok ? `silent session tail → ${sessionName}\n${tail.stdout.slice(-4000)}` : `silent session tail blocked → ${tail.stderr}` }], details: { ok: tail.ok, status: tail.ok ? "completed" : "blocked", session_name: sessionName, tail: tail.stdout, error: tail.stderr } } as any;
      }

      if (action === "start") {
        if (p.approved !== true) return { content: [{ type: "text", text: "silent session start blocked → approved=true required" }], details: { ok: false, status: "blocked", failure_class: "approval_required", session_name: sessionName } } as any;
        if (hasSession) return { content: [{ type: "text", text: `silent session already exists → ${sessionName}` }], details: { ok: true, status: "no_op", session_name: sessionName, attach_command: `tmux attach -t ${sessionName}`, sessions: sessionsBefore } } as any;
        const cmd = String(p.command || defaultSilentSessionCommand(p, sessionName));
        const started = silentSessionExec(["new-session", "-d", "-s", sessionName, "--", "bash", "-lc", cmd], 5000);
        const sessionsAfter = listSilentSessions();
        return { content: [{ type: "text", text: started.ok ? `silent session started → ${sessionName}\nattach: tmux attach -t ${sessionName}` : `silent session start blocked → ${started.stderr}` }], details: { ok: started.ok, status: started.ok ? "accepted" : "blocked", session_name: sessionName, attach_command: `tmux attach -t ${sessionName}`, command: cmd, side_effects: started.ok ? ["tmux_new_session"] : [], sessions: sessionsAfter, error: started.stderr } } as any;
      }

      if (action === "send") {
        if (p.approved !== true) return { content: [{ type: "text", text: "silent session send blocked → approved=true required" }], details: { ok: false, status: "blocked", failure_class: "approval_required", session_name: sessionName } } as any;
        if (!hasSession) return { content: [{ type: "text", text: `silent session not found → ${sessionName}` }], details: { ok: false, status: "not_found", session_name: sessionName, sessions: sessionsBefore } } as any;
        const line = String(p.command || "").trim();
        if (!line) return { content: [{ type: "text", text: "silent session send blocked → command required" }], details: { ok: false, status: "validation_rejected", session_name: sessionName } } as any;
        const sent = silentSessionExec(["send-keys", "-t", sessionName, "--", line, "C-m"], 3000);
        return { content: [{ type: "text", text: sent.ok ? `silent session sent → ${sessionName}` : `silent session send blocked → ${sent.stderr}` }], details: { ok: sent.ok, status: sent.ok ? "accepted" : "blocked", session_name: sessionName, side_effects: sent.ok ? ["tmux_send_keys"] : [], error: sent.stderr } } as any;
      }

      if (action === "kill") {
        if (p.approved !== true || p.force !== true) return { content: [{ type: "text", text: "silent session kill blocked → approved=true and force=true required" }], details: { ok: false, status: "blocked", failure_class: "approval_required", session_name: sessionName, sessions: sessionsBefore } } as any;
        if (!hasSession) return { content: [{ type: "text", text: `silent session not found → ${sessionName}` }], details: { ok: true, status: "no_op", session_name: sessionName, sessions: sessionsBefore } } as any;
        const killed = silentSessionExec(["kill-session", "-t", sessionName], 3000);
        const sessionsAfter = listSilentSessions();
        return { content: [{ type: "text", text: killed.ok ? `silent session killed → ${sessionName}` : `silent session kill blocked → ${killed.stderr}` }], details: { ok: killed.ok, status: killed.ok ? "completed" : "blocked", session_name: sessionName, side_effects: killed.ok ? ["tmux_kill_session"] : [], sessions: sessionsAfter, error: killed.stderr } } as any;
      }

      return { content: [{ type: "text", text: `silent session action unsupported → ${action}` }], details: { ok: false, status: "validation_rejected", action } } as any;
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
      const ready = health.ok && workpoint.ok;
      const contractSummary = focusaToolContractSummary();
      const scopedContracts = String(p.scope || "all") === "all"
        ? FOCUSA_TOOL_CONTRACTS
        : FOCUSA_TOOL_CONTRACTS.filter((contract) => contract.family === String(p.scope || "") || contract.name.includes(String(p.scope || "")));
      const missingDocs = scopedContracts.filter((contract) => !contract.doc_path).map((contract) => contract.name);
      const knownExemptions = scopedContracts.filter((contract) => contract.exemptions.length > 0).map((contract) => ({ name: contract.name, exemptions: contract.exemptions }));
      const hookCounts = S.spec92HookTelemetry.reduce((acc: Record<string, number>, item: any) => {
        const hook = String(item.hook || "unknown");
        acc[hook] = (acc[hook] || 0) + 1;
        return acc;
      }, {});
      const latestToken = S.spec92TokenTelemetry.at(-1) || null;
      const resourceMode = resource.body?.resource_mode || {};
      const latestTransition = resourceMode.latest_transition || (Array.isArray(resource.body?.transition_history) ? resource.body.transition_history[0] : null);
      const transitionLabel = latestTransition ? `${String(latestTransition.from_mode || "?")}→${String(latestTransition.to_mode || "?")}` : "none";
      const text = `tool doctor → readiness=${ready ? "ready" : "degraded"} scope=${String(p.scope || "all")} contracts=${contractSummary.total} scoped=${scopedContracts.length} hooks=${S.spec92HookTelemetry.length} token_budget=${String((latestToken as any)?.budget_class || "unknown")} resource=${String(resourceMode.mode || "unknown")}/${String(resourceMode.reason || "unknown")} transition=${transitionLabel} health=${health.ok ? "ok" : "blocked"} workpoint=${workpoint.ok ? String(workpoint.body?.status || "ok") : "blocked"} work_loop=${loop.ok ? String(loop.body?.status || "ok") : "blocked"}`;
      return { content: [{ type: "text", text }], details: { ok: ready, status: ready ? "completed" : "degraded", health: health.body, resource_mode: resource.body, workpoint: workpoint.body, work_loop: loop.body, contracts_total: contractSummary.total, contracts_by_family: contractSummary.by_family, contract_coverage: { scoped: scopedContracts.length, missing_docs: missingDocs, known_exemptions: knownExemptions }, spec92: { hook_records: S.spec92HookTelemetry.length, hook_counts: hookCounts, token_records: S.spec92TokenTelemetry.length, latest_token: latestToken } } } as any;
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
            response: body,
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
    description: "Resolve bounded ProjectIdentity from cwd/project_root using marker, git, beads, workspace, daemon, and operator scope signals.",
    promptSnippet: "Use before trusting cross-project Workpoints, Trajectory packets, or scope-sensitive context.",
    parameters: Type.Object({
      cwd: Type.Optional(Type.String({ description: "Optional cwd/project path hint; defaults to Pi session cwd." })),
      project_root: Type.Optional(Type.String({ description: "Optional expected project root/scope." })),
    }),
    async execute(_id, params) {
      const p = params as { cwd?: string; project_root?: string };
      const query = new URLSearchParams();
      query.set("cwd", p.cwd || S.sessionCwd || process.cwd());
      if (p.project_root) query.set("project_root", p.project_root);
      const result = await focusaFetchDetailed(`/project/identity?${query.toString()}`, { method: "GET" });
      const body = result.body || {};
      const identity = body.project_identity || {};
      const text = result.ok
        ? `project identity → status=${String(identity.status || body.status || "unknown")} confidence=${String(identity.confidence || "unknown")} root=${String(identity.project_root || "unknown")}`
        : `project identity blocked → ${explainWorkLoopResult(result, "project identity unavailable")}`;
      const toolResult = body.details?.tool_result_v1 || { ok: result.ok, status: result.ok ? String(body.status || "completed") : "blocked", canonical: body.canonical === true, degraded: body.degraded !== false, failure_class: body.failure_class || null, retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_scope_or_daemon" }, side_effects: [], evidence_refs: [], next_tools: body.next_tools || ["focusa_project_verify", "focusa_trajectory_view", "focusa_workpoint_resume"] };
      return {
        content: [{ type: "text", text }],
        details: {
          ok: result.ok,
          status: result.ok ? String(body.status || "completed") : "blocked",
          endpoint: "/v1/project/identity",
          canonical: body.canonical === true,
          degraded: body.degraded === true,
          project_identity: identity,
          verification: body.verification,
          tool_result_v1: toolResult,
          failure_class: toolResult.failure_class || body.failure_class || null,
          next_tools: toolResult.next_tools || body.next_tools || ["focusa_project_verify", "focusa_trajectory_view", "focusa_workpoint_resume"],
          response: body,
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_project_verify",
    label: "Focusa Project Verify",
    description: "Verify active project scope against expected ProjectIdentity fields and report mismatches without mutating state.",
    promptSnippet: "Use when project/session scope is ambiguous or before accepting a Workpoint/Trajectory packet as canonical.",
    parameters: Type.Object({
      cwd: Type.Optional(Type.String({ description: "Optional cwd/project path hint; defaults to Pi session cwd." })),
      project_root: Type.Optional(Type.String({ description: "Expected project root." })),
      project_id: Type.Optional(Type.String({ description: "Expected project id from marker/operator." })),
      canonical_name: Type.Optional(Type.String({ description: "Expected canonical project name." })),
      repo_remote: Type.Optional(Type.String({ description: "Expected git origin remote." })),
    }),
    async execute(_id, params) {
      const p = params as { cwd?: string; project_root?: string; project_id?: string; canonical_name?: string; repo_remote?: string };
      const payload = { ...p, cwd: p.cwd || S.sessionCwd || process.cwd() };
      const result = await focusaFetchDetailed("/project/verify", { method: "POST", body: JSON.stringify(payload) });
      const body = result.body || {};
      const identity = body.project_identity || {};
      const verified = body.verification?.verified === true;
      const text = result.ok
        ? `project verify → verified=${verified} status=${String(identity.status || body.status || "unknown")} confidence=${String(identity.confidence || "unknown")} root=${String(identity.project_root || "unknown")}`
        : `project verify blocked → ${explainWorkLoopResult(result, "project verify unavailable")}`;
      const toolResult = body.details?.tool_result_v1 || { ok: result.ok && body.status !== "blocked", status: result.ok ? String(body.status || "completed") : "blocked", canonical: body.canonical === true, degraded: body.degraded !== false, failure_class: body.failure_class || null, retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_scope_or_daemon" }, side_effects: [], evidence_refs: [], next_tools: body.next_tools || ["focusa_project_identity", "focusa_trajectory_view", "focusa_workpoint_resume"] };
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
          response: body,
        },
      } as any;
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
    }),
    async execute(_id, params) {
      const p = params as any;
      const query = new URLSearchParams();
      query.set("project_root", String(p.project_root || S.sessionCwd || process.cwd()));
      if (p.session_id || S.sessionFrameKey) query.set("session_id", String(p.session_id || S.sessionFrameKey));
      if (p.continuity_id || S.continuityId) query.set("continuity_id", String(p.continuity_id || S.continuityId));
      if (p.mode) query.set("mode", String(p.mode));
      const result = await focusaFetchDetailed(`/trajectory/view?${query.toString()}`, { method: "GET" });
      const body = result.body || {};
      const project = body.project_identity || {};
      const trajectory = body.trajectory || {};
      const sufficiency = body.intelligence_view?.context_sufficiency || {};
      const posture = String(sufficiency.proceed_posture || sufficiency.recommended_action || "unknown");
      const projectMismatches = Array.isArray(project.mismatches) ? project.mismatches : [];
      const trajectoryUnset = body.status === "not_found" && String(project.status || "") === "verified" && projectMismatches.length === 0;
      const recovery = trajectoryUnset ? null : scopeRecoveryContext(body, String(p.project_root || S.sessionCwd || process.cwd()), String(p.continuity_id || S.continuityId || ""), "trajectory_view");
      const trajectoryText = trajectoryUnset
        ? `trajectory view → NOT SET for project=${String(project.project_root || p.project_root || S.sessionCwd || process.cwd())}; definition=unclear; posture=${posture}; next=focusa_trajectory_define_goal`
        : body.canonical === true
          ? `trajectory view → SET long_term=${String(trajectory.long_term_goal || "missing")} desired=${String(trajectory.desired_end_state || "missing")} current=${String(trajectory.current_state || "missing")} gap=${String(trajectory.active_gap || "none")} posture=${posture}`
          : `trajectory view → status=${String(body.status || "unknown")} canonical=${body.canonical === true} project=${String(project.status || "unknown")} definition=${String(trajectory.definition_status || "unknown")} posture=${posture}`;
      const text = result.ok
        ? [trajectoryText, recovery?.text].filter(Boolean).join("\n")
        : `trajectory view blocked → ${explainWorkLoopResult(result, "trajectory unavailable")}`;
      const toolResult = body.details?.tool_result_v1 || { ok: result.ok && body.status !== "degraded" && body.status !== "not_found", status: result.ok ? String(body.status || "completed") : String(result.status), canonical: body.canonical === true, degraded: body.degraded === true, failure_class: body.failure_class || null, retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_scope_or_daemon" }, side_effects: [], evidence_refs: [], next_tools: body.next_tools || ["focusa_workpoint_resume", "focusa_active_object_resolve"] };
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
          response: body,
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
      short_term_goal: Type.Optional(Type.String({ description: "Current short-term project goal." })),
      current_state: Type.Optional(Type.String({ description: "Current verified state if known." })),
      goal_source: Type.Optional(Type.String({ description: "operator|durable_supersession|focus_state|workpoint|beads|imported|inferred_context" })),
      supersedes_trajectory_id: Type.Optional(Type.String({ description: "Prior trajectory id if this supersedes one." })),
      operator_confirmed: Type.Optional(Type.Boolean({ description: "True when operator explicitly confirmed a root goal change." })),
      supersession_evidence_refs: Type.Optional(Type.Array(Type.String(), { description: "Durable evidence refs allowing root goal supersession without direct operator prompt." })),
      project_root: Type.Optional(Type.String({ description: "Optional expected project root; defaults to Pi session cwd." })),
      session_id: Type.Optional(Type.String({ description: "Optional temporal Pi session id; defaults to Pi session key." })),
      continuity_id: Type.Optional(Type.String({ description: "Optional logical continuity id; defaults to Pi continuity id." })),
      idempotency_key: Type.Optional(Type.String({ description: "Optional external idempotency key." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = p.project_root || S.sessionCwd || process.cwd();
      const body = { ...p, project_root: projectRoot, session_id: p.session_id || S.sessionFrameKey, continuity_id: p.continuity_id || S.continuityId, session_identity: await buildFocusaSessionIdentity(projectRoot, "manual", { continuityId: p.continuity_id, sessionId: p.session_id }) };
      const result = await focusaFetchDetailed("/trajectory/define-goal", { method: "POST", body: JSON.stringify(body) });
      const b = result.body || {};
      const candidate = b.trajectory_candidate || {};
      const text = result.ok
        ? `trajectory define_goal → ${b.canonical === true ? "SET" : "NOT SET"} long_term=${String(candidate.long_term_goal || "missing")} desired=${String(candidate.desired_end_state || "missing")} definition=${String(candidate.definition_status || "unknown")} persisted=${b.persisted === true}`
        : `trajectory define_goal blocked → ${explainWorkLoopResult(result, "define failed")}`;
      const toolResult = b.details?.tool_result_v1 || { ok: result.ok && b.status !== "validation_rejected", status: result.ok ? String(b.status || "completed") : String(result.status), canonical: b.canonical === true, degraded: b.degraded === true, failure_class: b.failure_class || null, retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_scope_or_daemon" }, side_effects: [], evidence_refs: p.supersession_evidence_refs || [], next_tools: b.next_tools || ["focusa_trajectory_assess"] };
      return { content: [{ type: "text", text }], details: { ok: toolResult.ok, status: result.ok ? String(b.status || "completed") : String(result.status), endpoint: "/v1/trajectory/define-goal", canonical: b.canonical === true, degraded: b.degraded === true, advisory_only: b.advisory_only === true, trajectory_candidate: candidate, tool_result_v1: toolResult, failure_class: toolResult.failure_class || null, side_effects: toolResult.side_effects || [], evidence_refs: toolResult.evidence_refs || [], response: b, next_tools: toolResult.next_tools || b.next_tools || ["focusa_trajectory_assess"] } } as any;
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
      const projectRoot = p.project_root || S.sessionCwd || process.cwd();
      const body = { ...p, project_root: projectRoot, session_id: p.session_id || S.sessionFrameKey, continuity_id: p.continuity_id || S.continuityId, session_identity: await buildFocusaSessionIdentity(projectRoot, "manual", { continuityId: p.continuity_id, sessionId: p.session_id }) };
      const result = await focusaFetchDetailed("/trajectory/assess", { method: "POST", body: JSON.stringify(body) });
      const b = result.body || {};
      const text = result.ok ? `trajectory assess → gaps=${Array.isArray(b.gaps) ? b.gaps.length : 0} action=${String(b.recommended_action || "unknown")} canonical=${b.canonical === true}` : `trajectory assess blocked → ${explainWorkLoopResult(result, "assess failed")}`;
      const toolResult = b.details?.tool_result_v1 || { ok: result.ok, status: result.ok ? String(b.status || "completed") : String(result.status), canonical: b.canonical === true, degraded: b.degraded === true, failure_class: b.failure_class || null, retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_scope_or_daemon" }, side_effects: [], evidence_refs: p.evidence_refs || [], next_tools: b.next_tools || ["focusa_trajectory_propose_workpoint"] };
      return { content: [{ type: "text", text }], details: { ok: toolResult.ok, status: result.ok ? String(b.status || "completed") : String(result.status), endpoint: "/v1/trajectory/assess", canonical: b.canonical === true, degraded: b.degraded === true, gaps: b.gaps || [], recommended_action: b.recommended_action || null, tool_result_v1: toolResult, failure_class: toolResult.failure_class || null, side_effects: toolResult.side_effects || [], evidence_refs: toolResult.evidence_refs || [], response: b, next_tools: toolResult.next_tools || b.next_tools || ["focusa_trajectory_propose_workpoint"] } } as any;
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
      const projectRoot = p.project_root || S.sessionCwd || process.cwd();
      const body = { ...p, project_root: projectRoot, session_id: p.session_id || S.sessionFrameKey, continuity_id: p.continuity_id || S.continuityId, session_identity: await buildFocusaSessionIdentity(projectRoot, "manual", { continuityId: p.continuity_id, sessionId: p.session_id }) };
      const result = await focusaFetchDetailed("/trajectory/propose-workpoint", { method: "POST", body: JSON.stringify(body) });
      const b = result.body || {};
      const candidate = b.workpoint_candidate || {};
      const blockers = Array.isArray(candidate.blockers) ? candidate.blockers.length : 0;
      const text = result.ok ? `trajectory propose_workpoint → advisory=${b.advisory_only === true} action=${String(candidate.action_intent?.action_type || "unknown")} checkpoint_required=${candidate.checkpoint_required === true} blockers=${blockers} no_execution=${b.no_execution_side_effects === true}` : `trajectory propose_workpoint blocked → ${explainWorkLoopResult(result, "proposal failed")}`;
      const toolResult = b.details?.tool_result_v1 || { ok: result.ok, status: result.ok ? String(b.status || "completed") : String(result.status), canonical: b.canonical === true, degraded: b.degraded === true, failure_class: b.failure_class || null, retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_scope_or_daemon" }, side_effects: [], evidence_refs: [], next_tools: b.next_tools || ["focusa_workpoint_checkpoint"] };
      return { content: [{ type: "text", text }], details: { ok: toolResult.ok, status: result.ok ? String(b.status || "completed") : String(result.status), endpoint: "/v1/trajectory/propose-workpoint", canonical: b.canonical === true, degraded: b.degraded === true, advisory_only: b.advisory_only === true, no_execution_side_effects: b.no_execution_side_effects === true, workpoint_candidate: candidate, tool_result_v1: toolResult, failure_class: toolResult.failure_class || null, side_effects: toolResult.side_effects || [], evidence_refs: toolResult.evidence_refs || [], response: b, next_tools: toolResult.next_tools || b.next_tools || ["focusa_workpoint_checkpoint"] } } as any;
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
      const projectRoot = p.project_root || S.sessionCwd || process.cwd();
      const body = { ...p, project_root: projectRoot, session_id: p.session_id || S.sessionFrameKey, continuity_id: p.continuity_id || S.continuityId, session_identity: await buildFocusaSessionIdentity(projectRoot, "compaction", { continuityId: p.continuity_id, sessionId: p.session_id }) };
      const result = await focusaFetchDetailed("/trajectory/checkpoint", { method: "POST", body: JSON.stringify(body) });
      const b = result.body || {};
      const text = result.ok ? `trajectory checkpoint → status=${String(b.status || "unknown")} persisted=${b.persisted === true} canonical=${b.canonical === true}` : `trajectory checkpoint blocked → ${explainWorkLoopResult(result, "checkpoint failed")}`;
      const toolResult = b.details?.tool_result_v1 || { ok: result.ok, status: result.ok ? String(b.status || "completed") : String(result.status), canonical: b.canonical === true, degraded: b.degraded === true, failure_class: b.failure_class || null, retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_scope_or_daemon" }, side_effects: [], evidence_refs: [], next_tools: b.next_tools || ["focusa_workpoint_checkpoint"] };
      return { content: [{ type: "text", text }], details: { ok: toolResult.ok, status: result.ok ? String(b.status || "completed") : String(result.status), endpoint: "/v1/trajectory/checkpoint", canonical: b.canonical === true, degraded: b.degraded === true, persisted: b.persisted === true, advisory_only: b.advisory_only === true, trajectory_checkpoint: b.trajectory_checkpoint || null, tool_result_v1: toolResult, failure_class: toolResult.failure_class || null, side_effects: toolResult.side_effects || [], evidence_refs: toolResult.evidence_refs || [], response: b, next_tools: toolResult.next_tools || b.next_tools || ["focusa_workpoint_checkpoint"] } } as any;
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
      const projectRoot = p.project_root || S.sessionCwd || process.cwd();
      const body = { ...p, project_root: projectRoot, session_id: p.session_id || S.sessionFrameKey, continuity_id: p.continuity_id || S.continuityId, session_identity: await buildFocusaSessionIdentity(projectRoot, "session_switch", { continuityId: p.continuity_id, sessionId: p.session_id }) };
      const result = await focusaFetchDetailed("/trajectory/resume", { method: "POST", body: JSON.stringify(body) });
      const b = result.body || {};
      const packet = b.resume_packet || {};
      const text = result.ok ? `trajectory resume → status=${String(b.status || "unknown")} canonical=${b.canonical === true} project=${String(packet.project_identity?.status || "unknown")}` : `trajectory resume blocked → ${explainWorkLoopResult(result, "resume failed")}`;
      const toolResult = b.details?.tool_result_v1 || { ok: result.ok && b.status !== "degraded" && b.status !== "not_found", status: result.ok ? String(b.status || "completed") : String(result.status), canonical: b.canonical === true, degraded: b.degraded === true, failure_class: b.failure_class || null, retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_scope_or_daemon" }, side_effects: [], evidence_refs: [], next_tools: b.next_tools || ["focusa_workpoint_resume"] };
      return { content: [{ type: "text", text }], details: { ok: toolResult.ok, status: result.ok ? String(b.status || "completed") : String(result.status), endpoint: "/v1/trajectory/resume", canonical: b.canonical === true, degraded: b.degraded === true, resume_packet: packet, tool_result_v1: toolResult, failure_class: toolResult.failure_class || null, side_effects: toolResult.side_effects || [], evidence_refs: toolResult.evidence_refs || [], response: b, next_tools: toolResult.next_tools || b.next_tools || ["focusa_workpoint_resume"] } } as any;
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
      project_root: Type.Optional(Type.String({ description: "Explicit safe project/repo root; use after compaction if Pi cwd is broad like /root." })),
      session_id: Type.Optional(Type.String({ description: "Optional temporal Pi session id; defaults to this Pi session key." })),
      continuity_id: Type.Optional(Type.String({ description: "Stable logical session/workstream id; defaults to this Pi continuity id." })),
      attach_to_workpoint: Type.Optional(Type.Boolean({ description: "Defaults true." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      if (p.attach_to_workpoint === false) {
        return { content: [{ type: "text", text: `evidence capture → captured ref=${p.evidence_ref} attach_to_workpoint=false` }], details: { ok: true, status: "completed", evidence_ref: p.evidence_ref } } as any;
      }
      const projectRoot = p.project_root || S.sessionCwd || process.cwd();
      const clarity = await enforceTrajectoryClarityPrecondition(projectRoot, "evidence capture", { blockOperatorInput: false, continuityId: p.continuity_id, sessionId: p.session_id });
      if (!clarity.ok) return { content: [{ type: "text", text: clarity.text || "evidence capture blocked by trajectory clarity gate" }], details: { ok: false, status: "blocked", ...clarity.details } } as any;
      const res = await focusaFetchDetailed("/workpoint/evidence/link", {
        method: "POST",
        headers: { "x-focusa-writer-id": await preferredWriterId() },
        body: JSON.stringify({ workpoint_id: p.workpoint_id, target_ref: p.target_ref, result: p.result, evidence_ref: p.evidence_ref, session_identity: await buildFocusaSessionIdentity(projectRoot, "manual", { continuityId: p.continuity_id, sessionId: p.session_id }), trajectory_clarity_precondition: clarity.details }),
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
      continuity_id: Type.Optional(Type.String({ description: "Stable logical session/workstream id; defaults to this Pi session continuity id." })),
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
      project_root: Type.Optional(Type.String({ description: "Explicit safe project/repo root; defaults to Pi session cwd when that cwd is safe." })),
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
      const projectRoot = p.project_root || S.sessionCwd || process.cwd();
      if (p.canonical !== false && !isProjectRootAuthoritySafe(projectRoot)) {
        const reason = projectRootAuthorityFailure(projectRoot) || "unsafe_project_root";
        return { content: [{ type: "text", text: `workpoint checkpoint blocked → unsafe project_root (${reason}); cd into a specific project/repo or pass project_root explicitly.` }], details: { ok: false, status: "blocked", failure_class: "scope_mismatch", project_root: projectRoot, reason } } as any;
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
      project_root: Type.Optional(Type.String({ description: "Explicit safe project/repo root; use after compaction if Pi cwd is broad like /root." })),
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
          details: { ok: true, status: "no_op", reason: "attach_to_workpoint=false" },
        } as any;
      }
      const projectRoot = p.project_root || S.sessionCwd || process.cwd();
      const clarity = await enforceTrajectoryClarityPrecondition(projectRoot, "workpoint evidence link", { blockOperatorInput: false, continuityId: p.continuity_id, sessionId: p.session_id });
      if (!clarity.ok) return { content: [{ type: "text", text: clarity.text || "workpoint evidence link blocked by trajectory clarity gate" }], details: { ok: false, status: "blocked", ...clarity.details } } as any;
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
      continuity_id: Type.Optional(Type.String({ description: "Stable logical session/workstream id; defaults to this Pi session continuity id." })),
      session_id: Type.Optional(Type.String({ description: "Optional temporal Pi session id; defaults to this Pi session key." })),
      mode: Type.Optional(Type.String({ description: "compact_prompt|full_json|operator_summary" })),
      project_root: Type.Optional(Type.String({ description: "Explicit safe project/repo root; defaults to Pi session cwd when that cwd is safe." })),
    }),
    promptGuidelines: [
      "Use immediately after compaction or session resume before choosing next work.",
      "If not_found, create a checkpoint before continuing important work.",
      "If canonical=false, state degraded status and avoid treating it as canonical truth.",
    ],
    async execute(_id, params) {
      const p = params as { workpoint_id?: string; continuity_id?: string; session_id?: string; mode?: string; project_root?: string };
      const projectRoot = p.project_root || S.sessionCwd || process.cwd();
      if (!isProjectRootAuthoritySafe(projectRoot)) {
        const reason = projectRootAuthorityFailure(projectRoot) || "unsafe_project_root";
        return { content: [{ type: "text", text: `workpoint resume blocked → unsafe project_root (${reason}); ignore stale packets and follow latest operator instruction.` }], details: { ok: false, status: "blocked", failure_class: "scope_mismatch", project_root: projectRoot, reason, next_tools: ["focusa_project_identity", "focusa_tool_doctor"] } } as any;
      }
      const payload = { workpoint_id: p.workpoint_id, mode: p.mode || "compact_prompt", continuity_id: p.continuity_id || ensureContinuityId(projectRoot), session_id: p.session_id || S.sessionFrameKey, project_root: projectRoot, session_identity: await buildFocusaSessionIdentity(projectRoot, "session_switch", { continuityId: p.continuity_id, sessionId: p.session_id }) };
      const res = await focusaFetchDetailed("/workpoint/resume", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      const rejected = res.body?.status === "rejected_scope_mismatch";
      const recovery = scopeRecoveryContext(res.body || {}, projectRoot, payload.continuity_id, "workpoint_resume");
      const text = res.ok && !rejected
        ? [`workpoint resume → ${summarizeWorkpointResponse(res.body)}\n${String(res.body?.rendered_summary || "")}`.trim(), recovery?.text].filter(Boolean).join("\n")
        : rejected
          ? [`workpoint resume rejected: project_root mismatch. Ignore packet; follow latest operator instruction and current repo.`, recovery?.text].filter(Boolean).join("\n")
          : [`workpoint resume unavailable → ${explainWorkLoopResult(res, "resume failed")}`, recovery?.text].filter(Boolean).join("\n");
      const v2 = res.body?.resume_packet_v2 || null;
      const toolResult = res.body?.details?.tool_result_v1 || v2?.details?.tool_result_v1 || { ok: res.ok && !rejected, status: res.ok ? String(res.body?.status || "completed") : String(res.status), canonical: res.body?.canonical === true, degraded: res.body?.degraded === true || rejected, failure_class: res.body?.failure_class || (rejected ? "scope_mismatch" : null), retry: { safe: res.ok && !rejected, posture: res.ok && !rejected ? "safe_retry" : "do_not_retry_unchanged" }, side_effects: [], evidence_refs: [], next_tools: res.body?.next_tools || ["focusa_workpoint_resume", "focusa_trajectory_view", "focusa_traverse"] };
      return {
        content: [{ type: "text", text }],
        details: { ok: toolResult.ok, status: res.status, endpoint: "/workpoint/resume", canonical: res.body?.canonical === true, degraded: res.body?.degraded === true, failure_class: toolResult.failure_class || null, scope_recovery_context: recovery?.details || null, resume_packet_v2: v2, rendered_summary: res.body?.rendered_summary || "", tool_result_v1: toolResult, next_tools: toolResult.next_tools || res.body?.next_tools || ["focusa_workpoint_resume", "focusa_trajectory_view", "focusa_traverse"], request: payload, response: res.body },
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
          details: { ok: false, status: res.status, response: res.body ?? null },
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
          details: { ok: false, status: res.status, response: res.body ?? null },
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
    }),
    async execute(_id, params) {
      const res = await focusaFetch("/predictions", { method: "POST", body: JSON.stringify(params) });
      return { content: [{ type: "text", text: `prediction record → ${res?.status || "unavailable"}` }], details: res || { status: "blocked" } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_predict_recent",
    label: "Recent Predictions",
    description: "List recent bounded Focusa prediction records.",
    parameters: Type.Object({ limit: Type.Optional(Type.Number({ description: "Recent prediction count, max 100." })) }),
    async execute(_id, params) {
      const limit = Math.max(1, Math.min(100, Number((params as any).limit || 20)));
      const res = await focusaFetch(`/predictions/recent?limit=${limit}`);
      return { content: [{ type: "text", text: `predictions recent → ${res?.predictions?.length || 0}` }], details: res || { status: "blocked" } } as any;
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
      const { prediction_id, ...body } = params as any;
      const res = await focusaFetch(`/predictions/${encodeURIComponent(prediction_id)}/evaluate`, { method: "POST", body: JSON.stringify(body) });
      return { content: [{ type: "text", text: `prediction evaluate → ${res?.status || "unavailable"}` }], details: res || { status: "blocked" } } as any;
    },
  });

  pi.registerTool({
    name: "focusa_predict_stats",
    label: "Prediction Stats",
    description: "Report Focusa prediction accuracy/calibration stats.",
    parameters: Type.Object({}),
    async execute() {
      const res = await focusaFetch("/predictions/stats");
      return { content: [{ type: "text", text: `prediction stats → ${res?.summary || "unavailable"}` }], details: res || { status: "blocked" } } as any;
    },
  });

}
