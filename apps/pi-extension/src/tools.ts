// FOCUSA_SCRATCHPAD: two-file model
// Spec: G1-07 §AsccSections + doc 44 §10.5 + §Forbidden
//
// The two-file model:
//   /tmp/pi-scratch/<turn>/notes.txt  → agent's FULL working notebook (unlimited, no Focus State)
//   Focus State (Focusa)               → operator-curated cognitive state only
//
// Extension = thin bridge. Focus State = operator manages.
// Agent uses scratchpad for working notes. Operator manages Focus State.

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { Type } from "@sinclair/typebox";
import { createHash } from "crypto";
import { registerAgentRuntimeTools } from "./agent-runtime-tools.js";
import {
  SPEC138_OPERATIONS,
  bindSpec138OperationPath,
  spec138Operation,
} from "./generated/spec138-operations.js";
import {
  getAttachmentRuntime,
  checkFocusa,
  focusaFetch,
  focusaPost,
  compatibleWorkLoopStatusState,
  ensurePiFrame,
  getFocusState,
  ensureContinuityId,
  isProjectRootAuthoritySafe,
  projectRootAuthorityFailure,
  buildFocusaSessionIdentity,
  normalizeProjectRoot,
  resolvePiProjectRoot,
  resolveFocusWriteProjectRoot,
  confirmPiProjectRoot,
  projectRootConfirmationRequired,
  projectRootConfirmationSummary,
  refreshTrajectoryClarityLifecycle,
  stampWorkpointPacketForCurrentPiSession,
  normalizeWorkpointResumePacketEnvelope,
  adoptWorkpointScopeForFrameRecovery,
  adoptVerifiedContinuityForCurrentSession,
  persistState,
  estimateTokens,
  storeEcsArtifact,
  getTurnCount,
  getActiveFrameId,
  getContinuityId,
  getSessionFrameKey,
  getSessionCwd,
  getCurrentScopeStore,
  getLastProjectRootResolution,
  getLastProjectIdentity,
  setLastProjectIdentity,
  getActiveWorkpointPacket,
  setActiveWorkpointPacket,
  getActiveWorkpointSummary,
  setActiveWorkpointSummary,
  getLastTrajectoryClarity,
  setLastTrajectoryClarity,
  getLastProjectVerify,
  getLatestReportSummary,
  setLastProjectVerify,
  currentProjectBindingDecision,
  setCurrentProjectBindingDecision,
  getToolUsageBatch,
  getCurrentTaskTurnStart,
  currentAttachmentKey,
} from "./state.js";
import {
  FOCUSA_TOOL_CONTRACTS,
  buildFocusaToolAffordanceCatalog,
  focusaToolContractSummary,
} from "./tool-contracts.js";
import {
  buildProjectWorkstreamKey,
  renderScopedResultHuman,
  scopedQueryParams,
  isWorkstreamKey,
  sameWorkstream,
  type ScopedResultEnvelope,
  type WorkstreamKey,
} from "./scoped-state.js";
import { buildNorthStarSnapshot, renderNorthStarCard } from "./north-star.js";
import { projectBindingAllowsDurableWrites, reconcileProjectBindingDecision } from "./project-binding.js";
import { resolveCanonicalMarkerProjectRoot } from "./project-identity-working-context.js";
import { publishScopedStateChange } from "./scoped-surface-refresh.js";
import { modelVisibleDiscoveryPayload as renderDiscoveryPayload } from "./tool-discovery-visible.js";
import { projectEntitlementDecision, type EntitlementDecisionV1 } from "./entitlement-policy-adapter.js";

function modelVisibleDiscoveryPayload(label: string, payload: unknown, maxChars = 12_000): string {
  return renderDiscoveryPayload(label, payload, storeEcsArtifact, maxChars);
}

const SCRATCHPAD_DIR = "/tmp/pi-scratch";

function scratchDir(turn: number): string {
  return `${SCRATCHPAD_DIR}/turn-${String(turn).padStart(4, "0")}`;
}

function ensureScratchDir(): void {
  try {
    const { execSync } = require("child_process");
    execSync(`mkdir -p "${SCRATCHPAD_DIR}"`, { stdio: "pipe" });
  } catch {
    /* best effort */
  }
}

function projectRootPermissionPosture(projectRoot: string): Record<string, unknown> {
  const { statSync } = require("fs") as typeof import("fs");
  const { userInfo } = require("os") as typeof import("os");
  const current = userInfo();
  const metadata = statSync(projectRoot);
  const homeOwner = projectRoot.match(/^\/home\/([^/]+)(?:\/|$)/)?.[1] || null;
  const owner = metadata.uid === current.uid ? current.username : homeOwner || `uid:${metadata.uid}`;
  const crossUserHome = homeOwner != null && owner !== current.username;
  return {
    project_root: projectRoot,
    root_owner: { user: owner, uid: metadata.uid },
    current_user: current.username,
    root_owned_by_current_user: metadata.uid === current.uid,
    root_user_home: homeOwner != null,
    posture: crossUserHome ? "cross_user_home_use_as_owner" : "same_user_or_non_home_root",
    guidance: crossUserHome
      ? `Run repo/file mutations via as-user ${owner}; avoid root-owned files under ${projectRoot}.`
      : "Project root ownership matches current user or is outside /home user space.",
  };
}

function appendScratchpadLine(note: string, tag?: string): { saved: boolean; turn: number } {
  const turn = getTurnCount();
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
  if (!getAttachmentRuntime().cfg?.emitMetrics) return;
  focusaPost("/telemetry/ops", {
    event,
    surface: "pi",
    turn_id: `pi-turn-${getTurnCount()}`,
    frame_id: getActiveFrameId(),
    ...body,
  });
}

function truncateForSummary(s: string, max: number): string {
  if (s.length <= max) return s;
  return s.slice(0, max - 1) + "…";
}

// FOCUSA_FIX-vuop: register a model_select listener that invalidates the
// session frame on model switch so subsequent Focusa daemon requests use
// the correct Pi session identity.
function registerVuopFix(pi: ExtensionAPI): void {
  pi.on("model_select", () => {
    // Model changed; invalidate the cached session frame key and project root
    // so the next tool call refreshes against the new Pi frame.
    getAttachmentRuntime().sessionFrameKey = "";
  });
  // Also refresh on session start/reload
  pi.on("session_start", () => {
    getAttachmentRuntime().sessionFrameKey = "";
  });
}

function stableJson(value: any): string {
  if (Array.isArray(value)) return `[${value.map(stableJson).join(",")}]`;
  if (value && typeof value === "object") {
    return `{${Object.keys(value)
      .sort()
      .map((key) => `${JSON.stringify(key)}:${stableJson(value[key])}`)
      .join(",")}}`;
  }
  return JSON.stringify(value);
}

function deltaTargets(delta: {
  decisions?: string[];
  constraints?: string[];
  failures?: string[];
  intent?: string;
  current_focus?: string;
  next_steps?: string[];
  open_questions?: string[];
  recent_results?: string[];
  notes?: string[];
  artifacts?: Array<{ kind: string; label: string; path_or_id?: string }>;
}): string[] {
  return Object.entries(delta)
    .filter(([, value]) => value !== undefined)
    .map(([key]) => key);
}

function mirrorFailedFocusWrite(
  kind: string,
  reason: PushDeltaFailureReason,
  payload: string,
  meta: Record<string, string | undefined>
): { saved: boolean; turn: number } {
  const note = JSON.stringify({
    type: "focusa_write_fallback",
    kind,
    reason,
    payload,
    meta,
    turn: getTurnCount(),
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

const TASK_PATTERNS =
  /\b(Fix all|Implement|Add|Create|Update|Remove|Check|Verify|Test|Build|Deploy|NEXT:|Signal:)\b/i;
const DEBUG_PATTERNS =
  /(\bDEBUG\b|\bTODO\b|\bstack trace\b|\berror\b|\bfailed\b|\bcrash\b|\bbroken\b|\bbug\b|\bat line\b|\bTraceback\b)/i;
const SELF_REF_PATTERNS =
  /\b(I think|I tried|I'm working|I'm doing|working on|trying to|in this session|while I was|I was just)\b/i;
const MULTI_SENTENCE = /\.\s+\w/;

function validateDecision(decision: string): { valid: boolean; reason?: string } {
  // §AsccSections: decisions = crystallized choices that guide future action.
  // Keep the public validator aligned with pushDelta's canonical Focus State limit.
  if (decision.length > 160) {
    return {
      valid: false,
      reason:
        "Too verbose — distill to ONE crystallized sentence (max 160 chars). Use scratchpad for elaboration.",
    };
  }
  if (TASK_PATTERNS.test(decision)) {
    return {
      valid: false,
      reason:
        "Sounds like a task list — decisions capture ARCHITECTURAL CHOICES, not implementation plans. Write task in scratchpad. Distill the decision.",
    };
  }
  if (DEBUG_PATTERNS.test(decision)) {
    return {
      valid: false,
      reason:
        "Sounds like debugging metadata — decisions are stable choices, not investigation notes. Move to scratchpad.",
    };
  }
  if (SELF_REF_PATTERNS.test(decision)) {
    return {
      valid: false,
      reason:
        "Sounds like stream-of-consciousness — decisions should be objective architectural statements. Distill from scratchpad notes.",
    };
  }
  if (MULTI_SENTENCE.test(decision)) {
    return {
      valid: false,
      reason:
        "Multiple sentences — decisions should be ONE crystallized sentence. Per §AsccSections (<=160 chars).",
    };
  }
  return { valid: true };
}

function validateConstraint(constraint: string, source?: string): { valid: boolean; reason?: string } {
  // §AsccSections: constraints = DISCOVERED REQUIREMENTS (not self-imposed tasks)
  // Constraint is a hard boundary from environment/architecture, not "I should do X".
  // Operator directives are discovered requirements even when phrased with "must/must not".
  const operatorDirective =
    /operator directive/i.test(source || "") || /^operator directive\b/i.test(constraint);
  if (constraint.length > 200) {
    return { valid: false, reason: "Too verbose — distill to one sentence (max 200 chars)." };
  }
  if (!operatorDirective && TASK_PATTERNS.test(constraint)) {
    return {
      valid: false,
      reason:
        "Sounds like a self-imposed task — constraints are DISCOVERED REQUIREMENTS from environment/architecture. Not 'I will do X'.",
    };
  }
  if (!operatorDirective && /\b(will|should|must|need to|going to)\b/i.test(constraint)) {
    return {
      valid: false,
      reason:
        "Sounds like self-imposed obligation — constraints are discovered requirements from environment, not agent commitments. Use scratchpad.",
    };
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
    return {
      valid: false,
      reason:
        "Vague — failures should be SPECIFIC: what failed AND why (or what you suspect). 'It didn't work' = scratchpad.",
    };
  }
  if (
    SELF_REF_PATTERNS.test(failure) &&
    !/^(Build|Test|Deploy|API|Request|Query|Compil|Cargo)/i.test(failure)
  ) {
    return {
      valid: false,
      reason:
        "Sounds like investigation process — failures should be: SPECIFIC COMPONENT failed, with DIAGNOSIS. Move investigation notes to scratchpad.",
    };
  }
  return { valid: true };
}

// §AsccSections: validate_slot — rejects verbose output, task patterns, self-reference.
// MUST run on ALL tool writes before any Focus State update.
function validateSlot(value: string, maxChars: number): boolean {
  if (!value || value.length === 0) return false;
  if (value.length > maxChars) return false;
  const lower = value.toLowerCase();
  if (/\b(implement | add | create | update | remove | fix all | check | verify | next:|signal:)/.test(lower))
    return false;
  if (
    /\b(i think|i tried|i'm working|i was|in this session|while i was|my fs\.|my fix|let me|i need to|i will|i'll need)/.test(
      lower
    )
  )
    return false;
  if (/\b(status:|next action:|blocker:)/.test(lower)) return false;
  if (/(\*\*|\u2705|\u274C|- \[ \]|---|```)/.test(value)) return false;
  if (lower.includes("now") && lower.includes("need to")) return false;
  if (lower.includes("continue") && value.length > 80) return false;
  return true;
}

function validateNamedSlot(
  value: string,
  maxChars: number,
  kind: "intent" | "current_focus" | "next_step" | "open_question" | "recent_result" | "note"
): { valid: boolean; reason?: string } {
  const trimmed = String(value || "").trim();
  if (!trimmed) return { valid: false, reason: `${kind.replace("_", " ")} cannot be empty.` };
  if (trimmed.length > maxChars)
    return { valid: false, reason: `${kind.replace("_", " ")} exceeds ${maxChars} chars.` };
  if (kind === "open_question" && !trimmed.includes("?")) {
    return { valid: false, reason: "Open question should be phrased as a question (include '?')." };
  }
  if (!validateSlot(trimmed, maxChars)) {
    return {
      valid: false,
      reason: `Rejected by Focus State slot validator — distill this ${kind.replace("_", " ")} to concise objective text or move verbose/process notes to scratchpad.`,
    };
  }
  return { valid: true };
}

export type PushDeltaFailureReason =
  | "offline"
  | "no_active_frame"
  | "frame_unavailable"
  | "scope_mismatch"
  | "read_model_lag"
  | "validation_rejected"
  | "write_failed";

export type PushDeltaResult =
  | { ok: true; duplicate_candidate?: boolean; idempotency_key?: string }
  | {
      ok: false;
      reason: PushDeltaFailureReason;
      api_reason?: string;
      duplicate_candidate?: boolean;
      idempotency_key?: string;
    };

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

type FocusaToolStatus =
  "accepted" | "completed" | "no_op" | "blocked" | "validation_rejected" | "degraded" | "offline" | "error";
type FocusaRetryPosture =
  | "safe_retry"
  | "retry_with_idempotency_key"
  | "check_side_effects_first"
  | "do_not_retry_unchanged"
  | "operator_required";
type FocusaFailureClass =
  | "validation_rejected"
  | "schema_invalid"
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
  | "entitlement_blocked"
  | "unknown_ambiguous_completion";

interface FocusaToolResultV1 {
  schema: "focusa.tool_result.v1";
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
  /** Spec 152F §7: canonical entitlement decision projected for a blocked tool. */
  entitlement_decision?: EntitlementDecisionV1;
}

function reflexSuggestionsForFailure(
  failureClass: FocusaFailureClass | null,
  status: FocusaToolStatus,
  nextTools: string[]
): string[] {
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
  if (nextTools.includes("focusa_project_identity") || nextTools.includes("focusa_project_verify"))
    suggestions.add("bind_project_root");
  if (nextTools.includes("focusa_traverse")) suggestions.add("prefer_summary_hot_path");
  return Array.from(suggestions).slice(0, 4);
}

function inferFailureClass(
  status: FocusaToolStatus,
  summary: string,
  message?: string | null,
  canonical?: boolean,
  degraded?: boolean
): FocusaFailureClass | null {
  const text = `${summary} ${message || ""}`.toLowerCase();
  if (
    text.includes("no active pi frame") ||
    text.includes("no active frame") ||
    text.includes("frame recovery")
  )
    return "frame_unavailable";
  if (
    text.includes("payload_equal=false") ||
    text.includes("live registry payload differs") ||
    text.includes("stale daemon registry") ||
    text.includes("stale runtime registry")
  )
    return "stale_runtime_registry";
  if (
    text.includes("oom") ||
    text.includes("out of memory") ||
    text.includes("resource exhausted") ||
    text.includes("killed process")
  )
    return "resource_exhausted";
  if (text.includes("null response") || text.includes("response=null") || text.includes("body=null"))
    return "null_response";
  if (status === "validation_rejected" || text.includes("validation_rejected") || text.includes("rejected"))
    return "validation_rejected";
  if (
    text.includes("schema_invalid") ||
    text.includes("must be an object") ||
    text.includes("expected_schema") ||
    text.includes("invalid request body") ||
    text.includes("missing required")
  )
    return "schema_invalid";
  if (
    text.includes("not_found") ||
    text.includes("not found") ||
    text.includes("missing prediction") ||
    text.includes("no such")
  )
    return "not_found";
  if (
    status === "offline" ||
    text.includes("daemon unavailable") ||
    text.includes("focusa offline") ||
    text.includes("connection refused")
  )
    return "daemon_unavailable";
  if (text.includes("timeout") || text.includes("timed out") || text.includes("abort")) {
    return /(cold|deep|replay|worktree|diagnostic)/.test(text) ? "cold_path_timeout" : "hot_path_timeout";
  }
  if (
    text.includes("claimed by another writer") ||
    text.includes("writer_conflict") ||
    text.includes("controlled by another session")
  )
    return "writer_conflict";
  if (
    text.includes("scope_conflict") ||
    text.includes("action_authority_for_current_ask=false") ||
    text.includes("action authority for current ask") ||
    text.includes("current ask project conflict")
  )
    return "scope_conflict";
  if (
    text.includes("project_root mismatch") ||
    text.includes("scope mismatch") ||
    text.includes("cross-project")
  )
    return "scope_mismatch";
  if (text.includes("approval required") || text.includes("requires approved")) return "approval_required";
  if (text.includes("permission denied") || text.includes("unauthorized") || text.includes("forbidden"))
    return "permission_denied";
  if (text.includes("read model lag") || text.includes("pending") || text.includes("not yet visible"))
    return "read_model_lag";
  if (degraded || canonical === false || text.includes("non-canonical") || text.includes("noncanonical"))
    return "noncanonical_fallback";
  if (status === "blocked" || status === "error") return "unknown_ambiguous_completion";
  return null;
}

function recoveryHintForFailure(
  failureClass: FocusaFailureClass | null,
  status: FocusaToolStatus,
  tool?: string
): { recovery_hint?: string; misuse_hint?: string; next_tools?: string[] } {
  switch (failureClass) {
    case "scope_conflict":
      return {
        recovery_hint:
          "Treat the saved packet as canonical only for its saved scope; verify the current-ask project, then checkpoint/resume in the correct project before file/API action.",
        misuse_hint:
          "Usually caused by operator project correction, alias/path mismatch, or project-switch ledger evidence that predates API-level scope_mismatch.",
        next_tools: [
          "focusa_project_verify",
          "focusa_project_identity",
          "focusa_workpoint_checkpoint",
          "focusa_workpoint_resume",
        ],
      };
    case "scope_mismatch":
      return {
        recovery_hint:
          "Use focusa_project_identity/verify with explicit project_root, then checkpoint/resume in the same continuity; do not retry stale packets unchanged.",
        misuse_hint:
          "Usually caused by broad cwd, cross-project packet reuse, or tool call before project binding.",
        next_tools: [
          "focusa_project_identity",
          "focusa_project_verify",
          "focusa_workpoint_checkpoint",
          "focusa_workpoint_resume",
        ],
      };
    case "frame_unavailable":
      return {
        recovery_hint:
          "Stay attentive to operator direction, continue from repo/operator context, then create/resume a scoped Workpoint before durable Focus State writes.",
        misuse_hint:
          "Focus State note tools were used without an active Pi frame; this is recoverable, not a dead end.",
        next_tools: [
          "focusa_project_identity",
          "focusa_workpoint_checkpoint",
          "focusa_workpoint_resume",
          "focusa_tool_doctor",
        ],
      };
    case "validation_rejected":
      return {
        recovery_hint:
          "Rewrite the durable slot as one compact declarative sentence, or put verbose/debug/task content in focusa_scratch.",
        misuse_hint:
          "Durable Focus State slots reject task lists, verbose reasoning, and non-declarative wording.",
        next_tools: ["focusa_scratch", "focusa_decide"],
      };
    case "schema_invalid":
      return {
        recovery_hint:
          "Inspect the tool's expected_schema (returned with this error) and provide the required fields. Run focusa_traverse surface=tool_registry to see the full parameter schema for this tool.",
        misuse_hint:
          "Tool parameter shape mismatch. The error envelope includes expected_schema with required fields and types — fix the input shape, do not retry unchanged.",
        next_tools: ["focusa_traverse", "focusa_tool_doctor", "focusa_tool_describe"],
      };
    case "not_found":
      return {
        recovery_hint:
          "Use the relevant recent/list/read tool or create the missing record before retrying the mutation.",
        misuse_hint:
          "Likely stale id, missing record, wrong project scope, or evaluating/linking before the source object exists.",
        next_tools: [
          "focusa_project_identity",
          "focusa_workpoint_resume",
          "focusa_predict_recent",
          "focusa_tool_doctor",
        ],
      };
    case "read_model_lag":
      return {
        recovery_hint:
          "Wait briefly, then read/resume the current packet once with the same idempotency scope; avoid duplicate writes.",
        misuse_hint: "A recent accepted write may not be visible in the read model yet.",
        next_tools: ["focusa_workpoint_resume", "focusa_tool_doctor"],
      };
    case "hot_path_timeout":
      return {
        recovery_hint:
          "Retry the bounded hot route once, then run focusa_tool_doctor/resource_mode; avoid cold/full payload reads.",
        misuse_hint: "Hot routes should be bounded; repeated timeouts indicate daemon/resource pressure.",
        next_tools: ["focusa_tool_doctor", "focusa_resource_mode", "focusa_traverse"],
      };
    case "cold_path_timeout":
      return {
        recovery_hint:
          "Switch to summary/traverse slices or explicit rehydrate refs; schedule cold diagnostics separately.",
        misuse_hint: "A cold/deep route was used where a bounded route would answer the next action.",
        next_tools: ["focusa_traverse", "focusa_resource_mode", "focusa_tool_doctor"],
      };
    case "daemon_unavailable":
      return {
        recovery_hint:
          "Run focusa_tool_doctor; if daemon is down or overloaded, continue from operator/repo context and retry after health is ok.",
        misuse_hint:
          "Tool failure is infrastructure/reachability, not a reason to stop all useful repo work.",
        next_tools: ["focusa_tool_doctor", "focusa_resource_mode"],
      };
    case "writer_conflict":
      return {
        recovery_hint:
          "Use writer-status/preflight and avoid mutating work-loop ownership without explicit operator approval.",
        misuse_hint: "A mutating work-loop command was attempted while another writer/session owns the loop.",
        next_tools: ["focusa_work_loop_writer_status", "focusa_work_loop_status"],
      };
    case "approval_required":
      return {
        recovery_hint:
          "Do not infer approval; use preflight/read-only path or wait for explicit approved=true/force=true where required.",
        misuse_hint:
          "A mutating/destructive/background-session action was attempted without required approval fields.",
        next_tools: ["focusa_tool_doctor"],
      };
    case "process_control_failed":
      return {
        recovery_hint:
          "Read the canonical session/run projection and daemon capabilities, then retry only with the exact current target and durable approval.",
        misuse_hint:
          "Likely stale run generation, missing approval, runner disconnect, or process-control race.",
        next_tools: ["focusa_silent_sessions", "focusa_tool_doctor"],
      };
    case "unknown_ambiguous_completion":
      return {
        recovery_hint:
          "Check side effects or canonical read state first, then retry only if no duplicate/cross-scope mutation risk exists.",
        misuse_hint: "Result did not prove success or failure; blind retry can duplicate writes.",
        next_tools: ["focusa_tool_doctor", "focusa_workpoint_resume"],
      };
    default:
      if (status === "blocked" || status === "error")
        return {
          recovery_hint:
            "Read failure_class, retry.posture, and next_tools; prefer project_identity → trajectory_view → workpoint_resume/checkpoint before retrying.",
          misuse_hint: "Likely out-of-order tool use or missing project/continuity context.",
          next_tools: [
            "focusa_project_identity",
            "focusa_trajectory_view",
            "focusa_workpoint_resume",
            "focusa_tool_doctor",
          ],
        };
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
  entitlement_decision?: EntitlementDecisionV1;
}): FocusaToolResultV1 {
  const degraded = params.degraded ?? (params.status === "degraded" || params.status === "offline");
  const canonical = params.canonical ?? (!degraded && params.ok);
  const summary = params.summary.slice(0, 240);
  const failureClass =
    params.failure_class ??
    inferFailureClass(params.status, summary, params.error?.message, canonical, degraded);
  const guidance = recoveryHintForFailure(failureClass, params.status, params.tool);
  const nextTools = (params.next_tools?.length ? params.next_tools : (guidance.next_tools ?? [])).slice(0, 4);
  const reflexSuggestions = reflexSuggestionsForFailure(failureClass, params.status, nextTools);
  return {
    schema: "focusa.tool_result.v1",
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
    ...(params.entitlement_decision ? { entitlement_decision: params.entitlement_decision } : {}),
  };
}

function compactHint(value?: string): string | undefined {
  if (!value) return undefined;
  return value.replace(/\s+/g, " ").trim().slice(0, 140);
}

function compactText(value: unknown, fallback = "unknown", max = 140): string {
  const text = String(value ?? "")
    .replace(/\s+/g, " ")
    .trim();
  return (text || fallback).slice(0, max);
}

function compactApiEcho(value: unknown): unknown {
  if (!value || typeof value !== "object") return value;
  const input = value as Record<string, any>;
  const keys = [
    "status",
    "canonical",
    "degraded",
    "failure_class",
    "error",
    "why",
    "next_step_hint",
    "workpoint_id",
    "packet_id",
    "trajectory_id",
    "endpoint",
    "route_tier",
  ];
  const out: Record<string, unknown> = {};
  for (const key of keys)
    if (input[key] !== undefined)
      out[key] = typeof input[key] === "string" ? input[key].slice(0, 240) : input[key];
  if (Array.isArray(input.next_tools)) out.next_tools = input.next_tools.slice(0, 4);
  return Object.keys(out).length ? out : { omitted: true, reason: "compact_api_echo" };
}

const FOCUSA_TOOL_TRAJECTORY_TTL_MS = 30_000;

function firstNonEmptyText(...values: unknown[]): string {
  for (const value of values) {
    const text = String(value ?? "")
      .replace(/\s+/g, " ")
      .trim();
    if (text) return text;
  }
  return "";
}

function objectValue(value: unknown): Record<string, any> {
  return value && typeof value === "object" && !Array.isArray(value) ? (value as Record<string, any>) : {};
}

function toolTrajectoryProjectRoot(details: Record<string, unknown>): string {
  const projectIdentity = objectValue(details.project_identity);
  const trajectory = objectValue(details.trajectory);
  const candidate = objectValue(details.trajectory_candidate);
  const workpoint = objectValue(details.workpoint);
  const cached = objectValue(getLastTrajectoryClarity());
  const candidates = [
    details.project_root,
    projectIdentity.project_root,
    trajectory.project_root,
    candidate.project_root,
    workpoint.project_root,
    objectValue(getActiveWorkpointPacket()).project_root,
    objectValue(getLastProjectIdentity()).project_root,
    cached.project_root,
    resolvePiProjectRoot(getSessionCwd() || process.cwd()),
  ];
  for (const candidateRoot of candidates) {
    if (typeof candidateRoot !== "string") continue;
    const root = normalizeProjectRoot(candidateRoot);
    if (root && isProjectRootAuthoritySafe(root)) return root;
  }
  return "";
}

function normalizeToolTrajectoryLadder(source: unknown, sourceLabel: string): Record<string, unknown> | null {
  const obj = objectValue(source);
  const ladder = objectValue(obj.trajectory_ladder);
  const hlt = firstNonEmptyText(obj.long_term_goal, obj.hlt, ladder.hlt, ladder.high_level_goal, ladder.high);
  const mlg = firstNonEmptyText(obj.mid_level_goal, obj.mlg, ladder.mlg, ladder.mid);
  const stg = firstNonEmptyText(obj.short_term_goal, obj.stg, ladder.stg, ladder.short);
  const activeGap = firstNonEmptyText(obj.active_gap, ladder.active_gap);
  const waypointValues = Array.isArray(obj.waypoints)
    ? obj.waypoints
    : Array.isArray(ladder.waypoints)
      ? ladder.waypoints
      : [];
  const waypoints = waypointValues
    .map((item: any) =>
      typeof item === "string" ? item : firstNonEmptyText(item?.title, item?.desired_state_delta)
    )
    .filter(Boolean);
  if (!hlt && !mlg && !stg && !activeGap && !waypoints.length) return null;
  return {
    schema: "focusa.trajectory_ladder.v1",
    source: sourceLabel,
    project_root: firstNonEmptyText(obj.project_root, ladder.project_root),
    continuity_id: firstNonEmptyText(obj.continuity_id, ladder.continuity_id),
    session_id: firstNonEmptyText(obj.session_id, ladder.session_id),
    trajectory_id: firstNonEmptyText(obj.trajectory_id, ladder.trajectory_id),
    hlt: hlt || null,
    mlg: mlg || null,
    stg: stg || null,
    desired_end_state: firstNonEmptyText(obj.desired_end_state, ladder.desired_end_state) || null,
    current_state: firstNonEmptyText(obj.current_state, ladder.current_state) || null,
    active_gap: activeGap || null,
    waypoints: waypoints.slice(0, 8),
    fallback_prior_project_trajectory:
      obj.fallback_prior_project_trajectory === true || ladder.fallback_prior_project_trajectory === true,
    fallback_source_continuity_id:
      firstNonEmptyText(obj.fallback_source_continuity_id, ladder.fallback_source_continuity_id) || null,
  };
}

function cachedToolTrajectoryLadder(
  details: Record<string, unknown>,
  refreshed?: unknown
): Record<string, unknown> | null {
  return (
    normalizeToolTrajectoryLadder(details.trajectory_ladder, "tool_details.trajectory_ladder") ||
    normalizeToolTrajectoryLadder(details.trajectory, "tool_details.trajectory") ||
    normalizeToolTrajectoryLadder(details.trajectory_candidate, "tool_details.trajectory_candidate") ||
    normalizeToolTrajectoryLadder(refreshed, "pi_state.lastTrajectoryClarity") ||
    normalizeToolTrajectoryLadder(getLastTrajectoryClarity(), "pi_state.lastTrajectoryClarity")
  );
}

function formatToolTrajectoryLadderSummary(ladder: Record<string, unknown>): string {
  const parts = [
    firstNonEmptyText(ladder.hlt) ? `HLT=${compactText(ladder.hlt, "", 120)}` : "",
    firstNonEmptyText(ladder.mlg) ? `MLG=${compactText(ladder.mlg, "", 100)}` : "",
    firstNonEmptyText(ladder.stg) ? `STG=${compactText(ladder.stg, "", 100)}` : "",
  ].filter(Boolean);
  return parts.join("; ");
}

async function ensureToolTrajectoryClarity(
  tool: string,
  details: Record<string, unknown>
): Promise<Record<string, unknown> | null> {
  const projectRoot = toolTrajectoryProjectRoot(details);
  if (!projectRoot) return objectValue(getLastTrajectoryClarity());
  const cached = objectValue(getLastTrajectoryClarity());
  const cachedRoot = typeof cached.project_root === "string" ? normalizeProjectRoot(cached.project_root) : "";
  const cachedLadder = normalizeToolTrajectoryLadder(cached, "pi_state.lastTrajectoryClarity");
  const cachedAt = Number(cached.refreshed_at || 0);
  const cacheFresh = Boolean(
    cachedLadder &&
    cachedRoot === projectRoot &&
    cachedAt > 0 &&
    Date.now() - cachedAt < FOCUSA_TOOL_TRAJECTORY_TTL_MS
  );
  if (cacheFresh) return cached;
  if (cachedToolTrajectoryLadder(details, null) && cachedRoot === projectRoot) return cached;
  try {
    const refreshed = await refreshTrajectoryClarityLifecycle(`tool_hlt_pickup:${tool}`, projectRoot);
    return objectValue(refreshed || getLastTrajectoryClarity());
  } catch {
    return cached;
  }
}

function focusaToolDetails(
  details: Record<string, unknown>,
  result: FocusaToolResultV1,
  trajectoryClarity?: Record<string, unknown> | null
): Record<string, unknown> {
  const trajectoryLadder = cachedToolTrajectoryLadder(details, trajectoryClarity);
  const trajectorySummary = trajectoryLadder ? formatToolTrajectoryLadderSummary(trajectoryLadder) : "";
  return {
    ...details,
    ...(trajectoryLadder ? { trajectory_ladder: trajectoryLadder } : {}),
    ...(trajectorySummary ? { trajectory_ladder_summary: trajectorySummary } : {}),
    tool_result_v1: result,
  };
}

function focusaEvidenceCaptureSuggestion(input: {
  target_ref: string;
  result: string;
  evidence_ref: string;
  project_root?: string | null;
  attach_to_workpoint?: boolean;
}): Record<string, unknown> {
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

function blockedToolResponse(
  tool: string,
  family: string,
  summary: string,
  failureClass: FocusaFailureClass,
  raw?: unknown,
  nextTools?: string[]
): any {
  const toolResult = focusaToolResult({
    ok: false,
    status: "blocked",
    failure_class: failureClass,
    canonical: false,
    degraded: true,
    summary,
    tool,
    family,
    retry: {
      safe: failureClass !== "validation_rejected",
      posture: failureClass === "validation_rejected" ? "do_not_retry_unchanged" : "safe_retry",
      reason: failureClass,
    },
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

type PiToolTemplateKind = "ok" | "blocked" | "preserved" | "advisory" | "no_op" | "started" | "tail";

interface PiToolTemplate {
  kind: PiToolTemplateKind;
  tool: string;
  summary: string;
  ids?: Array<{ label: string; value: string | number }>;
  fields?: Array<{ label: string; value: string | number | null | undefined }>;
  failureClass?: string | null;
  nextTools?: string[];
  note?: string | null;
}

function formatPiToolTemplate(tpl: PiToolTemplate): string {
  const lines: string[] = [];
  const status = tpl.kind === "ok" ? "ok" : tpl.kind;
  lines.push(`${tpl.tool} ${status} | ${tpl.summary}`.trim());
  if (Array.isArray(tpl.ids) && tpl.ids.length) {
    const idLine = tpl.ids
      .filter((i) => i.value !== null && i.value !== undefined && i.value !== "")
      .map((i) => `${i.label}=${i.value}`)
      .join(" ");
    if (idLine) lines.push(`ids: ${idLine}`);
  }
  if (Array.isArray(tpl.fields) && tpl.fields.length) {
    const fieldLine = tpl.fields
      .filter((f) => f.value !== null && f.value !== undefined && f.value !== "")
      .map((f) => `${f.label}=${typeof f.value === "string" ? f.value : String(f.value)}`)
      .join(" ");
    if (fieldLine) lines.push(`fields: ${fieldLine}`);
  }
  if (tpl.failureClass) lines.push(`class: ${tpl.failureClass}`);
  if (tpl.note) lines.push(`note: ${tpl.note}`.slice(0, 160));
  if (Array.isArray(tpl.nextTools) && tpl.nextTools.length) {
    const next = tpl.nextTools.slice(0, 3).join(" → ") || "focusa_tool_doctor";
    lines.push(`next: ${next}`);
  }
  return lines.join("\n").slice(0, 480);
}

function piToolText(tpl: PiToolTemplate): string {
  return formatPiToolTemplate(tpl);
}

function humanSummary(
  prefix: string,
  ids: Array<{ label: string; value: unknown }>,
  fields: Array<{ label: string; value: unknown }> = [],
  max = 240
): string {
  const idPart = ids
    .filter((i) => i.value !== null && i.value !== undefined && i.value !== "")
    .map((i) => `${i.label}=${i.value}`)
    .join(" ");
  const fieldPart = fields
    .filter((f) => f.value !== null && f.value !== undefined && f.value !== "")
    .map((f) => `${f.label}=${typeof f.value === "string" ? f.value : String(f.value)}`)
    .join(" ");
  const compact = [idPart, fieldPart].filter(Boolean).join("; ");
  return (compact ? `${prefix} ${compact}` : prefix).slice(0, max);
}

const VISIBLE_TOOL_ID_KEYS = [
  "workpoint_id",
  "packet_id",
  "trajectory_id",
  "prediction_id",
  "reflection_id",
  "adjustment_id",
  "snapshot_id",
  "baseline_snapshot_id",
  "from_snapshot_id",
  "to_snapshot_id",
  "evidence_ref",
  "eval_run_id",
  "run_id",
  "algorithm_run_id",
  "project_id",
  "device_id",
  "code",
  "continuity_id",
];

function collectVisibleToolIds(value: unknown, out: Map<string, string>, depth = 0): void {
  if (!value || depth > 2 || out.size >= 10) return;
  if (Array.isArray(value)) {
    for (const item of value.slice(0, 6)) collectVisibleToolIds(item, out, depth + 1);
    return;
  }
  if (typeof value !== "object") return;
  const obj = value as Record<string, unknown>;
  for (const key of VISIBLE_TOOL_ID_KEYS) {
    const raw = obj[key];
    if (typeof raw === "string" || typeof raw === "number") {
      const text = String(raw).trim();
      if (text) out.set(key, text.slice(0, 120));
    }
  }
  for (const key of [
    "workpoint",
    "trajectory",
    "trajectory_candidate",
    "project_identity",
    "response",
    "raw",
    "handle",
    "packet",
    "resume_packet",
    "evaluation",
    "prediction",
    "reflection",
    "adjustment",
    "snapshot",
    "diff",
  ]) {
    collectVisibleToolIds(obj[key], out, depth + 1);
  }
}

function ensureVisibleToolTemplate(
  _tool: string,
  result: any,
  details: Record<string, unknown>,
  toolResult: FocusaToolResultV1
): any {
  if (!Array.isArray(result?.content)) return result;
  const textIndex = result.content.findIndex((entry: any) => entry?.type === "text");
  if (textIndex < 0) return result;
  const originalText = String(result.content[textIndex]?.text || "").trim();
  const ids = new Map<string, string>();
  collectVisibleToolIds(details, ids);
  collectVisibleToolIds(toolResult, ids);
  const appended: string[] = [];
  if (ids.size && !/^ids:/m.test(originalText)) {
    appended.push(
      `ids: ${Array.from(ids.entries())
        .slice(0, 6)
        .map(([k, v]) => `${k}=${v}`)
        .join(" ")}`
    );
  }
  if (!/^(fields|status):/m.test(originalText)) {
    const fields = [`status=${toolResult.status}`];
    if (toolResult.canonical) fields.push("canonical=true");
    if (toolResult.degraded) fields.push("degraded=true");
    appended.push(`fields: ${fields.join(" ")}`);
  }
  if (Array.isArray(toolResult.next_tools) && toolResult.next_tools.length && !/^next:/m.test(originalText)) {
    appended.push(`next: ${toolResult.next_tools.slice(0, 3).join(" → ")}`);
  }
  if (!appended.length) return result;
  const content = result.content.map((entry: any, index: number) =>
    index === textIndex ? { ...entry, text: `${originalText}\n${appended.join("\n")}` } : entry
  );
  return { ...result, content, details: { ...details, visible_tool_template_v1: true } };
}

function timeoutPreservedText(
  surface: string,
  noun = "fallback",
  timeoutMs: number | null = null,
  recoveryHint: string | null = null
): string {
  const parts = [`${surface} preserved cached advisory ${noun}`, "cause=timeout"];
  if (timeoutMs) parts.push(`timeout_ms=${timeoutMs}`);
  parts.push("next=resource_mode/doctor/retry");
  if (recoveryHint) parts.push(`recovery_hint=${recoveryHint}`);
  return parts.join("; ").slice(0, 240);
}

function resolveActiveWorkpointContext(): {
  workpoint_id: string | null;
  evidence_refs: string[];
  summary?: string;
} {
  const packet = getActiveWorkpointPacket() || null;
  const workpoint = packet?.resume_packet?.workpoint || packet?.workpoint || packet;
  const workpointId = String(workpoint?.workpoint_id || packet?.workpoint_id || "") || null;
  const verificationRecords = Array.isArray(workpoint?.verification_records)
    ? workpoint.verification_records
    : [];
  const evidenceRefs = verificationRecords
    .map((record: any) => String(record?.evidence_ref || record?.result || ""))
    .filter(Boolean)
    .slice(0, 8);
  return {
    workpoint_id: workpointId,
    evidence_refs: evidenceRefs,
    summary: getActiveWorkpointSummary() || undefined,
  };
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
  for (const key of ["target_ref", "targetRef", "file", "path", "endpoint", "workpoint_id"])
    add("target", details[key]);
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
  const family = tool.startsWith("focusa_workpoint_")
    ? "workpoint"
    : tool.startsWith("focusa_work_loop_")
      ? "work_loop"
      : tool.startsWith("focusa_tree_")
        ? "tree_snapshot_lineage"
        : tool.startsWith("focusa_metacog_")
          ? "metacognition"
          : tool.startsWith("focusa_lineage") || tool.startsWith("focusa_li_")
            ? "lineage_intelligence"
            : tool === "focusa_scratch"
              ? "scratchpad"
              : "focus_state";
  const explicitStatus = typeof details.status === "string" ? details.status.toLowerCase() : "";
  const mappedExplicitStatus: FocusaToolStatus | null = ["accepted", "completed", "no_op"].includes(
    explicitStatus
  )
    ? (explicitStatus as FocusaToolStatus)
    : ["supported", "ok", "ready", "verified"].includes(explicitStatus)
      ? "completed"
      : explicitStatus === "validation_rejected"
        ? "validation_rejected"
        : ["offline", "unavailable"].includes(explicitStatus)
          ? "offline"
          : ["blocked", "not_found"].includes(explicitStatus)
            ? "blocked"
            : explicitStatus === "degraded"
              ? "degraded"
              : explicitStatus === "error"
                ? "error"
                : null;
  const ok =
    mappedExplicitStatus !== null
      ? ["accepted", "completed", "no_op"].includes(mappedExplicitStatus)
      : details.ok === true ||
        details.valid === true ||
        (!/^❌|blocked|.* unavailable/.test(text) && details.ok !== false && details.valid !== false);
  const validationRejected =
    mappedExplicitStatus === null && (details.valid === false || /validation_rejected|rejected/.test(text));
  const offline = mappedExplicitStatus === null && /offline|unavailable/.test(text);
  const blocked = mappedExplicitStatus === null && /blocked/.test(text);
  const degraded =
    mappedExplicitStatus === null && (details.canonical === false || /degraded|NON-CANONICAL/.test(text));
  const status: FocusaToolStatus =
    mappedExplicitStatus ||
    (validationRejected
      ? "validation_rejected"
      : offline
        ? "offline"
        : blocked
          ? "blocked"
          : degraded
            ? "degraded"
            : ok
              ? "completed"
              : "error");
  const readOnly =
    family === "lineage_intelligence" ||
    tool.endsWith("_status") ||
    tool.endsWith("_resume") ||
    tool.endsWith("_head") ||
    tool.endsWith("_path") ||
    tool.includes("_retrieve") ||
    tool.includes("_recent") ||
    tool.includes("_doctor") ||
    tool.includes("_diff_");
  const detailsFailureClass =
    typeof details.failure_class === "string"
      ? (details.failure_class as FocusaFailureClass)
      : typeof (details.response as any)?.failure_class === "string"
        ? ((details.response as any).failure_class as FocusaFailureClass)
        : undefined;
  // Spec 152F §7: project the canonical entitlement decision when the daemon
  // blocks the tool (focusaFetch returns an ENTITLEMENT_* denial envelope).
  const daemonResponse =
    details.response && typeof details.response === "object"
      ? (details.response as Record<string, unknown>)
      : null;
  const entitlementBlocked =
    detailsFailureClass === "entitlement_blocked" || daemonResponse?.failure_class === "entitlement_blocked";
  const entitlementCode =
    daemonResponse && daemonResponse.error && typeof daemonResponse.error === "object"
      ? String((daemonResponse.error as Record<string, unknown>).code || "ENTITLEMENT_BLOCKED")
      : "ENTITLEMENT_BLOCKED";
  const activeWorkpoint = resolveActiveWorkpointContext();
  const resultWorkpointId =
    String(
      details.response?.workpoint_id ||
        details.response?.active_workpoint_id ||
        details.workpoint_id ||
        activeWorkpoint.workpoint_id ||
        ""
    ) || null;
  return focusaToolResult({
    ok: entitlementBlocked ? false : ok,
    status: entitlementBlocked ? "blocked" : status,
    failure_class: entitlementBlocked ? "entitlement_blocked" : detailsFailureClass,
    canonical: !degraded && !offline,
    degraded,
    summary: text || `${tool} ${status}`,
    tool,
    family,
    endpoint: typeof details.endpoint === "string" ? details.endpoint : undefined,
    workpoint_id: resultWorkpointId,
    retry: {
      safe: entitlementBlocked ? false : readOnly || status === "validation_rejected" || status === "offline",
      posture: entitlementBlocked
        ? "operator_required"
        : status === "validation_rejected"
          ? "do_not_retry_unchanged"
          : readOnly
            ? "safe_retry"
            : "check_side_effects_first",
      reason: entitlementBlocked ? "entitlement_blocked" : status,
    },
    side_effects: entitlementBlocked ? [] : readOnly ? [] : [family],
    evidence_refs: activeWorkpoint.evidence_refs,
    next_tools:
      Array.isArray(details.next_tools) && details.next_tools.length
        ? details.next_tools.map(String)
        : entitlementBlocked
          ? ["focusa_agent_card", "focusa_tool_doctor"]
          : status === "offline"
            ? ["focusa_tool_doctor", "focusa_resource_mode"]
            : family === "workpoint"
              ? ["focusa_workpoint_resume"]
              : [],
    ontology_candidate_delta_refs: ontologyCandidateDeltaRefs(tool, result, status),
    error: entitlementBlocked
      ? { code: entitlementCode, message: text.slice(0, 240) }
      : validationRejected || blocked || offline
        ? { code: status, message: text.slice(0, 240) }
        : null,
    raw: details.response ?? details,
    ...(entitlementBlocked
      ? { entitlement_decision: projectEntitlementDecision(tool, daemonResponse) }
      : {}),
  });
}

function defaultFocusaPromptSnippet(name: string, description?: string): string {
  if (name.startsWith("focusa_workpoint_"))
    return "Use after project folder is verified; pass explicit project_root/continuity_id after compaction or unsafe cwd.";
  if (name.startsWith("focusa_trajectory_"))
    return "Advisory project-goal tool; verify project_root first and do not treat proposals as execution authority.";
  if (name.startsWith("focusa_work_loop_"))
    return "Check writer/status first; preflight pause/resume/stop unless operator explicitly authorized mutation.";
  if (name.startsWith("focusa_metacog_"))
    return "Use for reusable learning signals; store concise evidence-backed lessons, not raw transcript blobs.";
  if (name.startsWith("focusa_tree_") || name === "focusa_lineage_tree" || name === "focusa_li_tree_extract")
    return "Use bounded lineage/snapshot helpers instead of inferring branch/history from transcript memory.";
  if (name.startsWith("focusa_predict_"))
    return "Record/evaluate bounded predictions; predictions guide actions but never override operator steering.";
  if (name.includes("hygiene"))
    return "Diagnose first; apply hygiene only with explicit approved=true and never silently delete state.";
  if (name === "focusa_traverse")
    return "Use bounded traversal/search with explicit limits; opt into large payloads only when needed.";
  if (
    [
      "focusa_intent",
      "focusa_current_focus",
      "focusa_next_step",
      "focusa_open_question",
      "focusa_recent_result",
      "focusa_note",
    ].includes(name)
  )
    return "Write concise Focus State slot updates; use focusa_scratch for working notes and verbose reasoning.";
  return String(
    description || "Use this Focusa tool with explicit project_root when session/cwd is ambiguous."
  ).slice(0, 240);
}

function sleepMs(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function paramsWithAutoIdempotency(toolName: string, params: unknown, id: string): unknown {
  if (
    toolName !== "focusa_workpoint_checkpoint" ||
    !params ||
    typeof params !== "object" ||
    Array.isArray(params)
  )
    return params;
  const record = params as Record<string, any>;
  if (record.idempotency_key) return params;
  const continuity = String(record.continuity_id || getContinuityId() || "session")
    .replace(/[^A-Za-z0-9._:-]/g, "_")
    .slice(0, 80);
  return { ...record, idempotency_key: `pi-tool-${toolName}-${continuity}-${id}`.slice(0, 160) };
}

function shouldAutoRetryWorkpoint(toolName: string, result: any, toolResult: FocusaToolResultV1): boolean {
  if (!["focusa_workpoint_checkpoint", "focusa_workpoint_resume"].includes(toolName)) return false;
  const response = (result?.details as any)?.response || {};
  const text = String(result?.content?.[0]?.text || "").toLowerCase();
  return (
    toolResult.failure_class === "read_model_lag" ||
    response.status === "pending" ||
    response.failure_class === "read_model_lag" ||
    response.failure_class === "resource_exhausted" ||
    /pending|read-model lag|not yet visible/.test(text)
  );
}

function annotateAutoRetry(result: any, attempts: number): any {
  const details = { ...(result?.details || {}) };
  details.auto_retry = { attempts, policy: "bounded_workpoint_pending_retry" };
  return { ...result, details };
}

function capToolText(text: unknown, max = 700): string {
  const normalized = String(text ?? "")
    .replace(/\s+\n/g, "\n")
    .trim();
  return normalized.length <= max ? normalized : `${normalized.slice(0, Math.max(0, max - 1))}…`;
}

function formatOperatorDateTime(ms: number): string {
  return new Date(ms).toLocaleString("en-US", {
    timeZone: "America/Los_Angeles",
    year: "2-digit",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hour12: true,
  });
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
  const elapsedMs = Math.max(
    0,
    now - (getAttachmentRuntime().currentTaskStartTime || getAttachmentRuntime().sessionStartTime || now)
  );
  const providerTotal =
    getAttachmentRuntime().currentTaskProviderInputTokens +
    getAttachmentRuntime().currentTaskProviderOutputTokens;
  const estimatedTotal =
    getAttachmentRuntime().currentTaskInputTokenEstimate +
    getAttachmentRuntime().currentTaskOutputTokenEstimate +
    estimateTokens(JSON.stringify(getToolUsageBatch() || []));
  const totalTokens = providerTotal > 0 ? providerTotal : estimatedTotal;
  return {
    task_timing: {
      started_at: new Date(
        getAttachmentRuntime().currentTaskStartTime || getAttachmentRuntime().sessionStartTime || now
      ).toISOString(),
      started_at_operator: formatOperatorDateTime(
        getAttachmentRuntime().currentTaskStartTime || getAttachmentRuntime().sessionStartTime || now
      ),
      completed_at: new Date(now).toISOString(),
      completed_at_operator: formatOperatorDateTime(now),
      elapsed_ms: elapsedMs,
      elapsed_seconds: Math.floor(elapsedMs / 1000),
      elapsed_hms: formatElapsedHms(elapsedMs),
      turn_start: getCurrentTaskTurnStart(),
      turn_end: getTurnCount(),
      turn_count: Math.max(0, getTurnCount() - (getCurrentTaskTurnStart() || getTurnCount()) + 1),
      task_label: getAttachmentRuntime().currentTaskLabel || getAttachmentRuntime().currentAsk?.text || "",
    },
    token_usage: {
      provider_input_tokens: getAttachmentRuntime().currentTaskProviderInputTokens,
      provider_output_tokens: getAttachmentRuntime().currentTaskProviderOutputTokens,
      provider_total_tokens: providerTotal,
      estimated_input_tokens: getAttachmentRuntime().currentTaskInputTokenEstimate,
      estimated_output_tokens: getAttachmentRuntime().currentTaskOutputTokenEstimate,
      estimated_total_tokens: estimatedTotal,
      total_tokens: totalTokens,
      counting_method: providerTotal > 0 ? "provider_usage_when_available" : "estimate_chars_div_4_fallback",
      tool_calls: getAttachmentRuntime().currentTaskToolCalls,
    },
  };
}

function capToolOutputText(result: any): any {
  if (!Array.isArray(result?.content)) return result;
  return {
    ...result,
    content: result.content.map((entry: any) =>
      entry?.type === "text" ? { ...entry, text: capToolText(entry.text) } : entry
    ),
  };
}

const FOCUSA_TOOL_RESULT_V1_SCHEMA = Type.Object(
  {
    schema: Type.Literal("focusa.tool_result.v1"),
    ok: Type.Boolean(),
    status: Type.Union([
      Type.Literal("accepted"),
      Type.Literal("completed"),
      Type.Literal("no_op"),
      Type.Literal("blocked"),
      Type.Literal("validation_rejected"),
      Type.Literal("degraded"),
      Type.Literal("offline"),
      Type.Literal("error"),
    ]),
    failure_class: Type.Union([Type.String(), Type.Null()]),
    entitlement_decision: Type.Optional(Type.Unknown()),
    canonical: Type.Boolean(),
    degraded: Type.Boolean(),
    summary: Type.String(),
    retry: Type.Object(
      {
        safe: Type.Boolean(),
        posture: Type.String(),
        reason: Type.Optional(Type.String()),
      },
      { additionalProperties: false }
    ),
    side_effects: Type.Array(Type.String()),
    evidence_refs: Type.Array(Type.String()),
    next_tools: Type.Array(Type.String()),
    recovery_hint: Type.Optional(Type.String()),
    misuse_hint: Type.Optional(Type.String()),
    details: Type.Optional(Type.Unknown()),
  },
  { additionalProperties: false }
);

const FOCUSA_TOOL_OUTPUT_SCHEMA = Type.Object(
  {
    content: Type.Array(
      Type.Object(
        {
          type: Type.Literal("text"),
          text: Type.String(),
        },
        { additionalProperties: false }
      )
    ),
    details: Type.Optional(
      Type.Object(
        {
          tool_result_v1: Type.Optional(FOCUSA_TOOL_RESULT_V1_SCHEMA),
        },
        { additionalProperties: true }
      )
    ),
  },
  { additionalProperties: false }
);

function makeToolSchemaStrict(schema: any, seen = new Set<any>()): any {
  if (!schema || typeof schema !== "object" || seen.has(schema)) return schema;
  seen.add(schema);
  if (schema.type === "object" && schema.additionalProperties === undefined) {
    schema.additionalProperties = false;
  }
  for (const value of Object.values(schema)) {
    if (Array.isArray(value)) value.forEach((entry) => makeToolSchemaStrict(entry, seen));
    else makeToolSchemaStrict(value, seen);
  }
  return schema;
}

function withAgentFirstSchemas(tool: any): any {
  if (!tool?.name?.startsWith?.("focusa_")) return tool;
  return {
    ...tool,
    parameters: makeToolSchemaStrict(tool.parameters),
    outputSchema: FOCUSA_TOOL_OUTPUT_SCHEMA,
  };
}

function withToolResultEnvelope(tool: any): any {
  if (!tool?.name?.startsWith?.("focusa_") || typeof tool.execute !== "function") return tool;
  const execute = tool.execute;
  return {
    ...tool,
    promptSnippet: tool.promptSnippet || defaultFocusaPromptSnippet(tool.name, tool.description),
    async execute(id: string, params: unknown) {
      getAttachmentRuntime().currentTaskToolCalls += 1;
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
      const trajectoryClarity = await ensureToolTrajectoryClarity(tool.name, details);
      const enrichedDetails = focusaToolDetails(details, toolResult, trajectoryClarity);
      return capToolOutputText(ensureVisibleToolTemplate(tool.name, result, enrichedDetails, toolResult));
    },
  };
}

function formatPushDeltaFailure(reason: PushDeltaFailureReason): string {
  switch (reason) {
    case "offline":
      return "Focusa offline";
    case "no_active_frame":
    case "frame_unavailable":
      return "Focus State frame unavailable";
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

function pushDeltaFailureRecovery(
  reason: PushDeltaFailureReason,
  apiReason?: string
): {
  failure_class: FocusaFailureClass;
  retry_posture: FocusaRetryPosture;
  recovery_hint: string;
  next_tools: string[];
  api_reason?: string;
} {
  switch (reason) {
    case "offline":
      return {
        failure_class: "daemon_unavailable",
        retry_posture: "safe_retry",
        recovery_hint:
          "Run focusa_tool_doctor; if resource mode is emergency, use focusa_resource_mode before retrying.",
        next_tools: ["focusa_tool_doctor", "focusa_resource_mode"],
        api_reason: apiReason,
      };
    case "no_active_frame":
    case "frame_unavailable":
      return {
        failure_class: "frame_unavailable",
        retry_posture: "safe_retry",
        recovery_hint:
          "Use scratchpad for this note; checkpoint/resume a project-bound Workpoint before retrying Focus State writes.",
        next_tools: [
          "focusa_project_identity",
          "focusa_workpoint_checkpoint",
          "focusa_workpoint_resume",
          "focusa_tool_doctor",
        ],
        api_reason: apiReason,
      };
    case "scope_mismatch":
      return {
        failure_class: "scope_mismatch",
        retry_posture: "do_not_retry_unchanged",
        recovery_hint:
          "Refresh project_root+continuity_id via focusa_project_verify and focusa_workpoint_resume; do not retry with stale project context.",
        next_tools: [
          "focusa_project_verify",
          "focusa_workpoint_resume",
          "focusa_workpoint_checkpoint",
          "focusa_tool_doctor",
        ],
        api_reason: apiReason,
      };
    case "read_model_lag":
      return {
        failure_class: "read_model_lag",
        retry_posture: "safe_retry",
        recovery_hint:
          "Read model may lag a just-created frame or Workpoint; resume/check current packet before retrying once.",
        next_tools: ["focusa_workpoint_resume", "focusa_tool_doctor"],
        api_reason: apiReason,
      };
    case "validation_rejected":
      return {
        failure_class: "validation_rejected",
        retry_posture: "do_not_retry_unchanged",
        recovery_hint: "Rewrite concise canonical wording or store full reasoning in focusa_scratch.",
        next_tools: ["focusa_scratch"],
        api_reason: apiReason,
      };
    case "write_failed":
    default:
      return {
        failure_class: "unknown_ambiguous_completion",
        retry_posture: "check_side_effects_first",
        recovery_hint:
          "Run focusa_tool_doctor and inspect response details before retrying to avoid duplicate or cross-scope writes.",
        next_tools: ["focusa_tool_doctor", "focusa_scratch"],
        api_reason: apiReason,
      };
  }
}

function formatNonCriticalWriteFailure(
  slotLabel: string,
  reason: PushDeltaFailureReason,
  apiReason?: string
): string {
  const base = formatPushDeltaFailure(reason);
  const detail = apiReason ? ` Detail: ${apiReason}` : "";
  const recovery = pushDeltaFailureRecovery(reason, apiReason);
  // BAD-006 fix: Keep messages concise to avoid context pollution
  if (reason === "no_active_frame" || reason === "frame_unavailable")
    return `⚠️ ${slotLabel} not recorded: ${base}. Use scratchpad until project-bound frame exists. Next: ${recovery.recovery_hint}`;
  if (reason === "scope_mismatch" || reason === "read_model_lag")
    return `⚠️ ${slotLabel} not recorded: ${base}. Checkpoint fresh Workpoint. Next: ${recovery.recovery_hint}`;
  if (reason === "offline") return `⚠️ ${slotLabel} not recorded: ${base}. Next: ${recovery.recovery_hint}`;
  if (reason === "validation_rejected")
    return `⚠️ ${slotLabel} not recorded: ${base}. Next: ${recovery.recovery_hint}`;
  return `⚠️ ${slotLabel} not recorded: ${base}. Next: ${recovery.recovery_hint}`;
}

function namedSlotFallback(
  slotLabel: string,
  kind: string,
  reason: PushDeltaFailureReason,
  payload: string,
  apiReason?: string
): { text: string; saved: boolean; turn: number; recovery: ReturnType<typeof pushDeltaFailureRecovery> } {
  const fallback = mirrorFailedFocusWrite(kind, reason, payload, { api_reason: apiReason });
  const recovery = pushDeltaFailureRecovery(reason, apiReason);
  const fallbackText = fallback.saved
    ? ` Saved to scratchpad fallback (turn ${fallback.turn}).`
    : " Scratchpad fallback also failed.";
  return {
    text: `${formatNonCriticalWriteFailure(slotLabel, reason, apiReason)}${fallbackText}`,
    saved: fallback.saved,
    turn: fallback.turn,
    recovery,
  };
}

function conciseObjectiveSuggestion(payload: string): string {
  const text = String(payload || "")
    .replace(/^\s*status\s*:\s*/i, "")
    .replace(/\b(lowmem focusa active|focusa active|builds? only via [^.;]+|deploy wrapper)\b/gi, "")
    .replace(/\b(next action|blocker)\s*:[^.;]+/gi, "")
    .replace(/[`*_#>\[\]]/g, " ")
    .replace(/\s+/g, " ")
    .trim();
  const clause =
    text
      .split(/[.;]\s+/)
      .map((part) => part.trim())
      .filter(Boolean)
      .find((part) => !/\b(script|wrapper|status|blocker|next action)\b/i.test(part)) || text;
  return (clause || "Continue current operator-directed objective").slice(0, 120);
}

function namedSlotValidationFallback(
  slotLabel: string,
  kind: string,
  payload: string,
  reason?: string
): { text: string; saved: boolean; turn: number; suggestion?: string } {
  const fallback = mirrorFailedFocusWrite(kind, "validation_rejected", payload, { validator_reason: reason });
  const fallbackText = fallback.saved
    ? ` Original saved to scratchpad fallback (turn ${fallback.turn}).`
    : " Scratchpad fallback also failed.";
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
  return (
    project?.status === "verified" ||
    project?.quorum_status === "verified" ||
    api?.status === "verified" ||
    body?.verification?.verified === true
  );
}

function focusaToolWorkpointScope(packet: any): { projectRoot: string; continuityId: string } | null {
  if (!packet || typeof packet !== "object") return null;
  const workpoint = packet.resume_packet?.workpoint || packet.workpoint || packet;
  const projectRoot = normalizeProjectRoot(workpoint?.project_root || packet.project_root);
  const continuityId = String(workpoint?.continuity_id || packet.continuity_id || "").trim();
  if (!projectRoot || !continuityId || !isProjectRootAuthoritySafe(projectRoot)) return null;
  if (
    packet.canonical === false ||
    workpoint?.canonical === false ||
    packet.status === "partial" ||
    packet.status === "rejected_scope_mismatch"
  )
    return null;
  return { projectRoot, continuityId };
}

async function resolveFocusaToolProjectRoot(explicitProjectRoot?: unknown): Promise<string> {
  const explicit = normalizeProjectRoot(explicitProjectRoot);
  const ambientCwd = normalizeProjectRoot(process.cwd());
  const ambientMarkerCanonical = resolveCanonicalMarkerProjectRoot(ambientCwd);
  const sessionCwd = ambientMarkerCanonical
    ? ambientCwd
    : normalizeProjectRoot(getSessionCwd() || ambientCwd);
  const markerCanonical = ambientMarkerCanonical || resolveCanonicalMarkerProjectRoot(sessionCwd);
  if (!explicit && markerCanonical && isProjectRootAuthoritySafe(markerCanonical)) {
    return markerCanonical;
  }
  const cachedIdentity: any = getLastProjectIdentity() || {};
  const cachedCanonical = normalizeProjectRoot(
    cachedIdentity.canonical_parent_root || cachedIdentity.project_root
  );
  const cachedWorking = normalizeProjectRoot(
    cachedIdentity.active_worktree_root || cachedIdentity.working_context?.active_worktree_root
  );
  const cachedVerified = cachedIdentity.status === "verified" || cachedIdentity.verified === true;
  if (
    cachedVerified &&
    cachedCanonical &&
    (!explicit || explicit === cachedCanonical || explicit === cachedWorking) &&
    (!sessionCwd || sessionCwd === cachedCanonical || sessionCwd === cachedWorking)
  ) {
    return cachedCanonical;
  }
  if (explicit) {
    const identity: any = await buildFocusaSessionIdentity(explicit, "manual");
    return normalizeProjectRoot(identity.canonical_parent_root || identity.project_root || explicit);
  }
  const sessionRoot = resolvePiProjectRoot(sessionCwd || process.cwd());
  if (isProjectRootAuthoritySafe(sessionRoot)) {
    const identity: any = await buildFocusaSessionIdentity(sessionRoot, "manual");
    return normalizeProjectRoot(identity.canonical_parent_root || identity.project_root || sessionRoot);
  }

  const localScope = focusaToolWorkpointScope(getActiveWorkpointPacket());
  if (localScope) {
    if (!getContinuityId()) getAttachmentRuntime().continuityId = localScope.continuityId;
    return localScope.projectRoot;
  }

  return sessionRoot || normalizeProjectRoot(process.cwd()) || String(process.cwd());
}

function projectRootConfirmationGate(projectRoot: string, explicitProjectRoot?: unknown): any | null {
  if (explicitProjectRoot || !projectRootConfirmationRequired(projectRoot)) return null;
  const resolution = getLastProjectRootResolution();
  const candidates = resolution?.candidates || [];
  return {
    content: [
      {
        type: "text",
        text: `project root confirmation required → ${projectRootConfirmationSummary(projectRoot)}. Ask the operator to confirm the exact project_root before Focusa state writes.`,
      },
    ],
    details: {
      ok: false,
      status: "blocked",
      failure_class: "scope_mismatch",
      reason: "project_root_confidence_below_90",
      project_root: projectRoot,
      project_root_resolution: resolution,
      candidates,
      next_tools: ["focusa_project_identity", "focusa_workpoint_checkpoint"],
      next_actions: [
        {
          action_type: "operator_input_required",
          prompt: "Confirm the exact existing project_root, choose new-project Genesis, or resume an authorized handoff.",
        },
      ],
    },
  } as any;
}

function scopeRecoveryContext(
  body: any,
  projectRoot: string,
  continuityId?: string,
  source = "focusa"
): { text: string; details: Record<string, any> } | null {
  const status = String(body?.status || "");
  const canonical = body?.canonical === true;
  const project = body?.project_identity || body?.resume_packet?.project_identity || {};
  const trajectory = body?.trajectory || body?.resume_packet?.trajectory || {};
  const projectStatus = String(project?.status || project?.project_identity_api?.status || "unknown");
  const definitionStatus = String(trajectory?.definition_status || trajectory?.definition || "unknown");
  const failureClass = String(body?.failure_class || body?.details?.tool_result_v1?.failure_class || "");
  const needsRecovery =
    status === "degraded" ||
    status === "not_found" ||
    canonical === false ||
    projectStatus === "mismatch" ||
    definitionStatus === "conflicted" ||
    failureClass === "scope_mismatch";
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
      safe_next_tools: [
        "focusa_workpoint_checkpoint",
        "focusa_scratch",
        "focusa_project_verify",
        "focusa_workpoint_resume",
      ],
    },
  };
}

function allowsWorkpointBootstrapFromClarity(body: any, projectRoot: string, actionLabel: string): boolean {
  if (actionLabel !== "workpoint checkpoint") return false;
  if (!isProjectRootAuthoritySafe(projectRoot)) return false;
  return projectIdentityVerifiedInPayload(body);
}

// Push delta to Focusa — validates ALL slot values before write.
export async function pushDelta(delta: {
  decisions?: string[];
  constraints?: string[];
  failures?: string[];
  intent?: string;
  current_focus?: string;
  next_steps?: string[];
  open_questions?: string[];
  recent_results?: string[];
  notes?: string[];
  artifacts?: Array<{ kind: string; label: string; path_or_id?: string }>;
}): Promise<PushDeltaResult> {
  const targets = deltaTargets(delta);
  let recoveredFrame = false;
  emitWriteTelemetry("focusa_write_attempt", { targets, had_frame: !!getActiveFrameId() });

  if (!getAttachmentRuntime().focusaAvailable) {
    const recoveredOnline = await checkFocusa().catch(() => false);
    // Health probes can race daemon restarts or stale bridge state. Do not let a
    // failed probe veto a real write; /focus/update is the authoritative check.
    emitWriteTelemetry("focusa_write_recovery_result", {
      targets,
      reason: "offline",
      recovered: recoveredOnline,
      probe_only: true,
    });
  }

  // Validate every string slot before sending.
  if (delta.decisions?.some((v) => !validateSlot(v, 160))) {
    emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected" });
    return { ok: false, reason: "validation_rejected" };
  }
  if (delta.constraints?.some((v) => !validateSlot(v, 200))) {
    emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected" });
    return { ok: false, reason: "validation_rejected" };
  }
  if (delta.failures?.some((v) => !validateSlot(v, 300))) {
    emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected" });
    return { ok: false, reason: "validation_rejected" };
  }
  if (delta.intent && !validateSlot(delta.intent, 500)) {
    emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected" });
    return { ok: false, reason: "validation_rejected" };
  }
  if (delta.current_focus && !validateSlot(delta.current_focus, 300)) {
    emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected" });
    return { ok: false, reason: "validation_rejected" };
  }
  if (delta.next_steps?.some((v) => !validateSlot(v, 160))) {
    emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected" });
    return { ok: false, reason: "validation_rejected" };
  }
  if (delta.open_questions?.some((v) => !validateSlot(v, 200))) {
    emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected" });
    return { ok: false, reason: "validation_rejected" };
  }
  if (delta.recent_results?.some((v) => !validateSlot(v, 300))) {
    emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected" });
    return { ok: false, reason: "validation_rejected" };
  }
  if (delta.notes?.some((v) => !validateSlot(v, 200))) {
    emitWriteTelemetry("focusa_write_failed", { targets, reason: "validation_rejected" });
    return { ok: false, reason: "validation_rejected" };
  }

  if (!getActiveFrameId()) {
    emitWriteTelemetry("focusa_write_recovery_attempt", {
      targets,
      reason: "no_active_frame",
      strategy: "refresh_scoped_frame",
    });
    const refreshed = await getFocusState().catch(() => null);
    if (refreshed?.frame?.id) {
      recoveredFrame = true;
      emitWriteTelemetry("focusa_write_recovery_result", {
        targets,
        reason: "no_active_frame",
        recovered: true,
        strategy: "refresh_scoped_frame",
        frame_id: refreshed.frame.id,
      });
    }
  }

  if (!getActiveFrameId()) {
    emitWriteTelemetry("focusa_write_recovery_attempt", {
      targets,
      reason: "no_active_frame",
      strategy: "create_or_adopt_scoped_frame",
    });
    const frameId = await ensurePiFrame(undefined, undefined, "pi-auto-recover");
    recoveredFrame = recoveredFrame || !!frameId;
    emitWriteTelemetry("focusa_write_recovery_result", {
      targets,
      reason: "no_active_frame",
      recovered: !!frameId,
      strategy: "create_or_adopt_scoped_frame",
    });
    if (!frameId) {
      emitWriteTelemetry("focusa_write_failed", { targets, reason: "no_active_frame" });
      return { ok: false, reason: "no_active_frame" };
    }
  }

  try {
    // Refresh frame identity before writes; stale paused Pi frames are a common
    // source of reducer rejections and scratchpad fallbacks after rescope/compact.
    await getFocusState().catch(() => null);
    // Spec 110 / GH #8 fix: detect CWD change since last session save.
    // If process.cwd() differs from the cached getSessionCwd(), the agent has
    // switched projects via shell `cd`. Force a fresh scope resolution and
    // clear any cached active frame so the write uses the new project root.
    const liveCwd = process.cwd();
    const cachedCwd = getSessionCwd();
    if (cachedCwd && normalizeProjectRoot(liveCwd) !== normalizeProjectRoot(cachedCwd)) {
      emitWriteTelemetry("focusa_cwd_changed", { old: cachedCwd, new: liveCwd, targets });
      getAttachmentRuntime().activeFrameId = null;
    }
    const projectRoot = normalizeProjectRoot(
      resolveFocusWriteProjectRoot(process.cwd(), cachedCwd || liveCwd)
    );
    const continuityId = getContinuityId() || ensureContinuityId(projectRoot);
    if (!isProjectRootAuthoritySafe(projectRoot) || !continuityId) {
      emitWriteTelemetry("focusa_write_failed", {
        targets,
        reason: "scope_mismatch",
        project_root: projectRoot || null,
        continuity_id: continuityId || null,
      });
      return {
        ok: false,
        reason: "scope_mismatch",
        api_reason: "focus_update_requires_safe_project_root_and_continuity_id",
      };
    }
    const postUpdate = () =>
      focusaFetch("/focus/update", {
        method: "POST",
        body: JSON.stringify({
          frame_id: getActiveFrameId(),
          project_root: projectRoot,
          continuity_id: continuityId,
          turn_id: `pi-turn-${getTurnCount()}`,
          delta,
        }),
      });
    let response = await postUpdate();
    if (
      ["no_active_frame", "frame_unavailable", "rejected_scope_mismatch", "scope_mismatch"].includes(
        String(response?.status || "")
      )
    ) {
      emitWriteTelemetry("focusa_write_recovery_attempt", {
        targets,
        reason: ["rejected_scope_mismatch", "scope_mismatch"].includes(String(response?.status || ""))
          ? "scope_mismatch"
          : "stale_frame",
        stale_frame_id: getActiveFrameId(),
        active_frame_id: response?.active_frame_id,
        target_frame_id: response?.target_frame_id,
        failure_class: response?.failure_class,
      });
      getAttachmentRuntime().activeFrameId = null;
      const frameId = await ensurePiFrame(undefined, undefined, "pi-stale-frame-recover");
      recoveredFrame = recoveredFrame || !!frameId;
      emitWriteTelemetry("focusa_write_recovery_result", {
        targets,
        reason: "stale_frame",
        recovered: !!frameId,
      });
      if (frameId) response = await postUpdate();
    }
    if (!response || response.status === "write_failed") {
      emitWriteTelemetry("focusa_write_failed", {
        targets,
        reason: "write_failed",
        recovered_frame: recoveredFrame,
        api_reason: response?.reason,
      });
      return { ok: false, reason: "write_failed", api_reason: response?.reason };
    }
    if (response.status === "no_active_frame" || response.status === "frame_unavailable") {
      emitWriteTelemetry("focusa_write_failed", {
        targets,
        reason: "frame_unavailable",
        recovered_frame: recoveredFrame,
        api_reason: response.reason,
        active_frame_id: response.active_frame_id,
        target_frame_id: response.target_frame_id,
      });
      return { ok: false, reason: "frame_unavailable", api_reason: response.reason };
    }
    if (["rejected_scope_mismatch", "scope_mismatch"].includes(String(response.status))) {
      emitWriteTelemetry("focusa_write_failed", {
        targets,
        reason: "scope_mismatch",
        recovered_frame: recoveredFrame,
        api_reason: response.reason,
        active_frame_id: response.active_frame_id,
        target_frame_id: response.target_frame_id,
        diagnostic_class: response.diagnostic_class,
      });
      return { ok: false, reason: "scope_mismatch", api_reason: response.reason };
    }
    if (response.status === "rejected") {
      emitWriteTelemetry("focusa_write_failed", {
        targets,
        reason: "validation_rejected",
        recovered_frame: recoveredFrame,
        api_reason: response.reason,
      });
      return { ok: false, reason: "validation_rejected", api_reason: response.reason };
    }
    if (response.status !== "accepted") {
      emitWriteTelemetry("focusa_write_failed", {
        targets,
        reason: "write_failed",
        recovered_frame: recoveredFrame,
        status: response.status || "unknown",
        api_reason: response.reason,
      });
      return {
        ok: false,
        reason: "write_failed",
        api_reason: response.reason || response.status || "unknown",
      };
    }
    getAttachmentRuntime().focusaAvailable = true;
    const store = getCurrentScopeStore();
    if (store) store.focusaAvailable = true;
    emitWriteTelemetry("focusa_write_succeeded", {
      targets,
      recovered_frame: recoveredFrame,
      frame_id: response.frame_id || getActiveFrameId(),
    });
    return { ok: true };
  } catch {
    const online = await checkFocusa().catch(() => false);
    const reason: PushDeltaFailureReason = online ? "write_failed" : "offline";
    emitWriteTelemetry("focusa_write_failed", { targets, reason, recovered_frame: recoveredFrame });
    return { ok: false, reason };
  }
}

export function registerTools(pi: ExtensionAPI) {
  // FOCUSA_FIX-vuop: register model_select + session_start listeners that
  // invalidate sessionFrameKey on model switch or session reload.
  registerVuopFix(pi);
  const registerTool = pi.registerTool.bind(pi);
  const agentFirstToolDefinitions = new Map<string, any>();
  pi.registerTool = ((tool: any) => {
    const normalized = withAgentFirstSchemas(withToolResultEnvelope(tool));
    if (normalized?.name?.startsWith?.("focusa_")) {
      agentFirstToolDefinitions.set(normalized.name, normalized);
    }
    return registerTool(normalized);
  }) as typeof pi.registerTool;
  registerAgentRuntimeTools(pi);

  pi.registerTool({
    name: "focusa_daemon_routing_status",
    label: "Daemon Routing Status",
    description:
      "Resolve one explicit project/worktree/continuity/native-session scope against a supplied daemon registry. Never infers a global or foreign daemon.",
    parameters: Type.Object({
      registry: Type.Unknown({ description: "Canonical daemon registry projection from the controller." }),
      project_root: Type.String(),
      continuity_id: Type.String(),
      working_subpath_id: Type.String(),
      native_session_id: Type.String(),
    }),
    async execute(_id, params) {
      const input = params as any;
      const authority = await focusaFetch("/v1/daemon-routing/resolve", {
        method: "POST",
        body: JSON.stringify({
          schema: "focusa.daemon_routing_resolve.v1",
          registry: input.registry,
          route: {
            project_root: input.project_root,
            continuity_id: input.continuity_id,
            working_subpath_id: input.working_subpath_id,
          },
          native_session_id: input.native_session_id,
        }),
      });
      const safe = authority || {
        schema: "focusa.daemon_routing_authority.v1",
        status: "unresolved",
        selected_daemon_id: null,
        recovery_required: true,
        failure_class: "daemon_unavailable",
      };
      return {
        content: [{ type: "text", text: JSON.stringify(safe, null, 2) }],
        details: safe,
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_north_star_gate",
    label: "North Star Gate",
    description:
      "Inspect the current verified Project → HLT → MLG → STG → waypoint → gap → Workpoint → frontier chain before meaningful action. Read-only and fail-closed.",
    promptSnippet:
      "Use before meaningful work and after session/compaction/model/project/provider/writer transitions; stale authority remains advisory.",
    parameters: Type.Object({
      trigger: Type.Optional(Type.String({ description: "Lifecycle or operator trigger being checked." })),
    }),
    async execute(params: any) {
      const trigger = String(params?.trigger || "manual_gate");
      const projectRoot = await resolveFocusaToolProjectRoot();
      if (isProjectRootAuthoritySafe(projectRoot)) {
        try {
          await refreshTrajectoryClarityLifecycle(`north_star_gate:${trigger}`, projectRoot);
        } catch {
          // Snapshot rendering remains fail-closed and carries its recovery route.
        }
      }
      const snapshot = buildNorthStarSnapshot(trigger);
      return {
        content: [{ type: "text", text: renderNorthStarCard(snapshot).join("\n") }],
        details: {
          ok: snapshot.status === "ready",
          status: snapshot.status,
          canonical: false,
          advisory: true,
          snapshot,
          next_tools:
            snapshot.status === "ready"
              ? ["focusa_workpoint_resume"]
              : ["focusa_project_identity", "focusa_trajectory_view", "focusa_workpoint_resume"],
        },
      } as any;
    },
  });

  // ── focusa_scratch ──────────────────────────────────────────────────────
  // Agent's working notebook. Lives at /tmp/pi-scratch/. No Focus State write.
  // ALL working notes welcome: reasoning, task lists, hypotheses, dead ends,
  // self-corrections, design notes, NEXT:/Signal: directives.
  // Operator can read: ls /tmp/pi-scratch/ | cat /tmp/pi-scratch/turn-NNNN/notes.txt
  pi.registerTool({
    name: "focusa_scratch",
    label: "Scratchpad",
    description:
      "Write working notes to /tmp/pi-scratch/ — agent's notebook, no Focus State. Transfer crystallized decision to focusa_decide when done.",
    promptSnippet: "Working notes → scratchpad. Crystallized decision → focusa_decide.",
    parameters: Type.Object({
      note: Type.String({
        description: "Working note — reasoning, task list, hypothesis, dead end. Unlimited length.",
      }),
      tag: Type.Optional(
        Type.String({ description: "Tag: reasoning|task|hypothesis|dead-end|self-correction|next-step" })
      ),
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
        content: [
          {
            type: "text" as const,
            text: `📝 Scratchpad saved (turn ${scratch.turn}): ${note.slice(0, 80)}${note.length > 80 ? "…" : ""}`,
          },
        ],
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
    description:
      "Record a crystallized architectural decision in Focus State. Use focusa_scratch for working notes first. Decisions are ONE sentence (<=160 chars) — architectural choices only, not task lists.",
    promptSnippet: "Crystallized decision → Focus State. Working notes → focusa_scratch first.",
    parameters: Type.Object({
      decision: Type.String({
        description:
          "ONE crystallized architectural choice — what was decided and why (max 160 chars). NOT a task list or debugging note.",
      }),
      rationale: Type.Optional(
        Type.String({
          description:
            "Context: why this decision was made (max 200 chars). Summarize from scratchpad notes.",
        })
      ),
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
          content: [
            {
              type: "text" as const,
              text: `❌ Decision rejected: ${v.reason}\n\nWrite detailed reasoning to focusa_scratch first, then distill ONE crystallized decision.`,
            },
          ],
          details: { valid: false, reason: v.reason, decision, rationale: rationale?.slice(0, 200) },
        };
      }
      const turn = getTurnCount();
      const result = await pushDelta({ decisions: [decision] });
      if (!result.ok) {
        const fallback = mirrorFailedFocusWrite("decision", result.reason, decision, {
          rationale: rationale?.slice(0, 200),
        });
        const recovery = pushDeltaFailureRecovery(result.reason, result.api_reason);
        const fallbackText = fallback.saved
          ? `Saved to scratchpad automatically (turn ${fallback.turn}).`
          : "Scratchpad fallback also failed.";
        return {
          content: [
            {
              type: "text" as const,
              text: `⚠️ Decision not recorded in Focus State: ${formatPushDeltaFailure(result.reason)}. ${fallbackText} Next: ${recovery.recovery_hint}`,
            },
          ],
          details: {
            valid: false,
            reason: result.reason,
            decision,
            rationale: rationale?.slice(0, 200),
            scratch_saved: fallback.saved,
            scratch_turn: fallback.turn,
            ...recovery,
          },
        };
      }
      return {
        content: [
          {
            type: "text" as const,
            text: `✅ Decision recorded (turn ${turn}): ${decision.slice(0, 120)}${decision.length > 120 ? "…" : ""}`,
          },
        ],
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
    description:
      "Record a DISCOVERED REQUIREMENT in Focus State. Constraints are hard boundaries from environment/architecture — NOT self-imposed tasks. Max 200 chars.",
    promptSnippet: "Constraints = discovered requirements. Self-imposed tasks → focusa_scratch.",
    parameters: Type.Object({
      constraint: Type.String({
        description:
          "Discovered requirement — hard boundary from environment or architecture (max 200 chars). NOT a task or agent commitment.",
      }),
      source: Type.Optional(
        Type.String({
          description: "Where discovered: spec file, error message, API docs, operator directive.",
        })
      ),
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
          content: [
            {
              type: "text" as const,
              text: `❌ Constraint rejected: ${v.reason}\n\nDiscovered requirements from environment → focusa_constraint. Self-imposed tasks → focusa_scratch.`,
            },
          ],
          details: { valid: false, reason: v.reason, constraint, source },
        };
      }
      const turn = getTurnCount();
      const result = await pushDelta({ constraints: [constraint] });
      if (!result.ok) {
        const fallback = mirrorFailedFocusWrite("constraint", result.reason, constraint, { source });
        const recovery = pushDeltaFailureRecovery(result.reason, result.api_reason);
        const fallbackText = fallback.saved
          ? `Saved to scratchpad automatically (turn ${fallback.turn}).`
          : "Scratchpad fallback also failed.";
        return {
          content: [
            {
              type: "text" as const,
              text: `⚠️ Constraint not recorded in Focus State: ${formatPushDeltaFailure(result.reason)}. ${fallbackText} Next: ${recovery.recovery_hint}`,
            },
          ],
          details: {
            valid: false,
            reason: result.reason,
            constraint,
            source,
            scratch_saved: fallback.saved,
            scratch_turn: fallback.turn,
            ...recovery,
          },
        };
      }
      return {
        content: [
          {
            type: "text" as const,
            text: `✅ Constraint recorded (turn ${turn}): ${constraint.slice(0, 120)}${constraint.length > 120 ? "…" : ""}`,
          },
        ],
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
    description:
      "Record a specific failure with diagnosis in Focus State. Must identify WHAT failed and WHY (or suspected why). Max 300 chars.",
    promptSnippet: "Failures = specific component + diagnosis. Investigation notes → focusa_scratch.",
    parameters: Type.Object({
      failure: Type.String({
        description:
          "Specific failure: what failed + diagnosis (max 300 chars). Must contain period or colon.",
      }),
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
          content: [
            {
              type: "text" as const,
              text: `❌ Failure rejected: ${v.reason}\n\nBe specific: WHAT failed + WHY. Move investigation to focusa_scratch.`,
            },
          ],
          details: { valid: false, reason: v.reason, failure, recovery },
        };
      }
      const turn = getTurnCount();
      const result = await pushDelta({ failures: [failure] });
      if (!result.ok) {
        const fallback = mirrorFailedFocusWrite("failure", result.reason, failure, { recovery });
        const recoveryPlan = pushDeltaFailureRecovery(result.reason, result.api_reason);
        const fallbackText = fallback.saved
          ? `Saved to scratchpad automatically (turn ${fallback.turn}).`
          : "Scratchpad fallback also failed.";
        return {
          content: [
            {
              type: "text" as const,
              text: `⚠️ Failure not recorded in Focus State: ${formatPushDeltaFailure(result.reason)}. ${fallbackText} Next: ${recoveryPlan.recovery_hint}`,
            },
          ],
          details: {
            valid: false,
            reason: result.reason,
            failure,
            recovery,
            scratch_saved: fallback.saved,
            scratch_turn: fallback.turn,
            ...recoveryPlan,
          },
        };
      }
      return {
        content: [
          {
            type: "text" as const,
            text: `✅ Failure recorded (turn ${turn}): ${failure.slice(0, 120)}${failure.length > 120 ? "…" : ""}`,
          },
        ],
        details: { valid: true, reason: undefined, failure, recovery },
      };
    },
  });

  // ── focusa_intent (§AsccSections) ──────────────────────────────────────────
  // Set the frame intent: what this session is trying to achieve. 1-3 sentences.
  pi.registerTool({
    name: "focusa_intent",
    label: "Set Intent",
    description:
      "Set the frame intent — what this session is trying to achieve (1-3 sentences, max 500 chars).",
    parameters: Type.Object({
      intent: Type.String({
        description: "Intent: what this frame/session is trying to achieve (1-3 sentences, max 500 chars).",
      }),
    }),
    async execute(_id, params) {
      const { intent } = params as { intent: string };
      const v = validateNamedSlot(intent, 500, "intent");
      if (!v.valid) {
        const fallback = namedSlotValidationFallback("intent", "intent", intent.trim(), v.reason);
        return {
          content: [{ type: "text", text: fallback.text }],
          details: {
            valid: false,
            intent,
            reason: "validation_rejected",
            scratch_saved: fallback.saved,
            scratch_turn: fallback.turn,
          },
        } as any;
      }
      const result = await pushDelta({ intent: intent.trim() });
      if (result.ok)
        return {
          content: [{ type: "text", text: `Intent set: ${intent.slice(0, 100)}` }],
          details: { valid: true, reason: undefined, intent },
        };
      const fallback = namedSlotFallback("intent", "intent", result.reason, intent.trim(), result.api_reason);
      return {
        content: [{ type: "text", text: fallback.text }],
        details: {
          valid: false,
          intent,
          reason: result.reason,
          scratch_saved: fallback.saved,
          scratch_turn: fallback.turn,
          ...fallback.recovery,
        },
      } as any;
    },
  });

  // ── focusa_current_focus (§AsccSections) ─────────────────────────────────
  // Update current focus: what the agent is actively working on. Replaces on each update.
  pi.registerTool({
    name: "focusa_current_focus",
    label: "Set Current Focus",
    description:
      "Update current focus — what you are actively working on right now (1-3 sentences, max 300 chars).",
    parameters: Type.Object({
      focus: Type.String({
        description: "Current focus: what you are actively working on (1-3 sentences, max 300 chars).",
      }),
    }),
    async execute(_id, params) {
      const { focus } = params as { focus: string };
      const v = validateNamedSlot(focus, 300, "current_focus");
      if (!v.valid) {
        const fallback = namedSlotValidationFallback(
          "current focus",
          "current_focus",
          focus.trim(),
          v.reason
        );
        return {
          content: [{ type: "text", text: fallback.text }],
          details: {
            valid: false,
            focus,
            reason: "validation_rejected",
            scratch_saved: fallback.saved,
            scratch_turn: fallback.turn,
            suggested_current_focus: fallback.suggestion,
          },
        } as any;
      }
      const result = await pushDelta({ current_focus: focus.trim() });
      if (result.ok)
        return {
          content: [{ type: "text", text: `Current focus set: ${focus.slice(0, 100)}` }],
          details: { valid: true, reason: undefined, focus },
        };
      const fallback = namedSlotFallback(
        "current focus",
        "current_focus",
        result.reason,
        focus.trim(),
        result.api_reason
      );
      return {
        content: [{ type: "text", text: fallback.text }],
        details: {
          valid: false,
          focus,
          reason: result.reason,
          scratch_saved: fallback.saved,
          scratch_turn: fallback.turn,
          ...fallback.recovery,
        },
      } as any;
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
        return {
          content: [{ type: "text", text: fallback.text }],
          details: {
            valid: false,
            step,
            reason: "validation_rejected",
            scratch_saved: fallback.saved,
            scratch_turn: fallback.turn,
          },
        } as any;
      }
      const result = await pushDelta({ next_steps: [step.trim()] });
      if (result.ok)
        return {
          content: [{ type: "text", text: `Next step recorded: ${step.slice(0, 80)}` }],
          details: { valid: true, reason: undefined, step },
        };
      const fallback = namedSlotFallback(
        "next step",
        "next_step",
        result.reason,
        step.trim(),
        result.api_reason
      );
      return {
        content: [{ type: "text", text: fallback.text }],
        details: {
          valid: false,
          step,
          reason: result.reason,
          scratch_saved: fallback.saved,
          scratch_turn: fallback.turn,
          ...fallback.recovery,
        },
      } as any;
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
        const fallback = namedSlotValidationFallback(
          "open question",
          "open_question",
          question.trim(),
          v.reason
        );
        return {
          content: [{ type: "text", text: fallback.text }],
          details: {
            valid: false,
            question,
            reason: "validation_rejected",
            scratch_saved: fallback.saved,
            scratch_turn: fallback.turn,
          },
        } as any;
      }
      const result = await pushDelta({ open_questions: [question.trim()] });
      if (result.ok)
        return {
          content: [{ type: "text", text: `Open question recorded: ${question.slice(0, 80)}` }],
          details: { valid: true, reason: undefined, question },
        };
      const fallback = namedSlotFallback(
        "open question",
        "open_question",
        result.reason,
        question.trim(),
        result.api_reason
      );
      return {
        content: [{ type: "text", text: fallback.text }],
        details: {
          valid: false,
          question,
          reason: result.reason,
          scratch_saved: fallback.saved,
          scratch_turn: fallback.turn,
          ...fallback.recovery,
        },
      } as any;
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
        const fallback = namedSlotValidationFallback(
          "recent result",
          "recent_result",
          result.trim(),
          v.reason
        );
        return {
          content: [{ type: "text", text: fallback.text }],
          details: {
            valid: false,
            result,
            reason: "validation_rejected",
            scratch_saved: fallback.saved,
            scratch_turn: fallback.turn,
          },
        } as any;
      }
      const writeResult = await pushDelta({ recent_results: [result.trim()] });
      if (writeResult.ok)
        return {
          content: [{ type: "text", text: `Result recorded: ${result.slice(0, 80)}` }],
          details: { valid: true, reason: undefined, result },
        };
      const fallback = namedSlotFallback(
        "recent result",
        "recent_result",
        writeResult.reason,
        result.trim(),
        writeResult.api_reason
      );
      return {
        content: [{ type: "text", text: fallback.text }],
        details: {
          valid: false,
          result,
          reason: writeResult.reason,
          scratch_saved: fallback.saved,
          scratch_turn: fallback.turn,
          ...fallback.recovery,
        },
      } as any;
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
        return {
          content: [{ type: "text", text: fallback.text }],
          details: {
            valid: false,
            note,
            reason: "validation_rejected",
            scratch_saved: fallback.saved,
            scratch_turn: fallback.turn,
          },
        } as any;
      }
      const result = await pushDelta({ notes: [note.trim()] });
      if (result.ok)
        return {
          content: [{ type: "text", text: `Note recorded: ${note.slice(0, 80)}` }],
          details: { valid: true, reason: undefined, note },
        };
      const fallback = namedSlotFallback("note", "note", result.reason, note.trim(), result.api_reason);
      return {
        content: [{ type: "text", text: fallback.text }],
        details: {
          valid: false,
          note,
          reason: result.reason,
          scratch_saved: fallback.saved,
          scratch_turn: fallback.turn,
          ...fallback.recovery,
        },
      } as any;
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
      route.includes("context-cognition/proof") ||
      route.includes("context-cognition/optimizer/artifacts") ||
      route.includes("call-stack/verify") ||
      route.includes("include_full_payload=true") ||
      route.includes("mode=full") ||
      /[?&]deep=true/.test(route)
    )
      return "cold";
    if (verb !== "GET") return "warm";
    if (
      route.startsWith("/health") ||
      route.startsWith("/work-loop/status") ||
      route.startsWith("/trajectory/view")
    )
      return "hot";
    return "warm";
  }

  function timeoutFailureClassForRoute(path: string, method?: string): FocusaFailureClass {
    return focusaRouteTier(path, method) === "cold" ? "cold_path_timeout" : "hot_path_timeout";
  }

  function timeoutBudgetForRoute(path: string, method = "GET"): number {
    const configured = getAttachmentRuntime().cfg?.focusaApiTimeoutMs || 5000;
    const tier = focusaRouteTier(path, method);
    if (tier === "hot" && path.startsWith("/trajectory/view"))
      return Math.min(Math.max(configured, 4000), 5000);
    if (tier === "hot") return Math.min(configured, 2500);
    if (tier === "cold") return Math.max(configured, 8000);
    return Math.max(configured, 2500);
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

  async function fetchJsonDetailed(
    url: string,
    timeoutMs = 1500
  ): Promise<{ ok: boolean; status: number; body: any | null; error?: string }> {
    const ac = new AbortController();
    const t = setTimeout(() => ac.abort(), timeoutMs);
    try {
      const r = await fetch(url, { signal: ac.signal });
      let body: any = null;
      try {
        body = await r.json();
      } catch {
        body = null;
      }
      return { ok: r.ok, status: r.status, body };
    } catch (err: any) {
      return {
        ok: false,
        status: err?.name === "AbortError" ? 408 : 0,
        body: null,
        error: String(err?.message || err || "request failed"),
      };
    } finally {
      clearTimeout(t);
    }
  }

  async function uiaiBrowserHealthCard(): Promise<any> {
    const base = String(
      process.env.UIAI_ENGINE_URL || process.env.WPUIAI_ENGINE_URL || "http://127.0.0.1:7456"
    ).replace(/\/$/, "");
    const [health, metrics] = await Promise.all([
      fetchJsonDetailed(`${base}/api/health/browser`, 1200),
      fetchJsonDetailed(`${base}/api/metrics/browser`, 1200),
    ]);
    const body = metrics.body || health.body || {};
    const queue = body.queue || {};
    const currentCapacity = body.current_capacity || body.agent_pressure?.current_capacity || {};
    const capacityAvailable =
      currentCapacity.capacity_available === true ||
      Number(currentCapacity.remaining_page_slots || 0) > 0 ||
      Number(currentCapacity.available_idle_pages || 0) > 0;
    const p95 = Number(queue.p95_wait_ms || 0);
    const p99 = Number(queue.p99_wait_ms || 0);
    const rejected = Number(queue.rejected || 0);
    const status = String(body.status || (health.ok || metrics.ok ? "ok" : "unavailable"));
    const historicalPressure =
      p99 >= 5000 || p95 >= 2500 || rejected > 0 ? "high" : p99 >= 1500 || p95 >= 750 ? "medium" : "low";
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
      recommended_action:
        pressure === "high"
          ? "narrow browser workload, close stale sessions, or retry after queue drains"
          : pressure === "medium"
            ? "monitor browser queue before parallel UIAI work"
            : "continue normally",
      response: compactApiEcho(body),
      error: health.error || metrics.error || null,
    };
  }

  function scopedResponseFailureClass(
    response: { ok: boolean; status: number },
    body: any
  ): FocusaFailureClass {
    const diagnostic = JSON.stringify({
      authority: body?.authority,
      failure_class: body?.failure_class,
      code: body?.code,
      error: body?.error,
      reason: body?.reason,
      human: body?.human,
    }).toLowerCase();
    if (diagnostic.includes("scope")) return "scope_mismatch";
    if (response.status === 400 || /schema|invalid|validation/.test(diagnostic)) return "validation_rejected";
    if (response.status === 403 || diagnostic.includes("permission")) return "permission_denied";
    if (response.status === 404 || diagnostic.includes("not_found")) return "not_found";
    if (response.status === 0) return "daemon_unavailable";
    return "unknown_ambiguous_completion";
  }

  function scopedResponseHuman(body: any, fallback: string): string {
    return String(
      body?.human_readable || body?.human?.summary || body?.summary || body?.reason || body?.error || fallback
    );
  }

  function typedTrajectoryScopeMatches(value: any, projectRoot: string, continuityId: string): boolean {
    const responseRoot = normalizeProjectRoot(
      value?.project_identity?.project_root ||
        value?.trajectory?.project_root ||
        value?.scope?.project_root ||
        value?.project_root
    );
    if (!responseRoot || responseRoot !== normalizeProjectRoot(projectRoot)) return false;
    const responseContinuity = String(
      value?.trajectory?.continuity_id || value?.scope?.continuity_id || value?.continuity_id || ""
    ).trim();
    return !responseContinuity || !continuityId || responseContinuity === continuityId;
  }

  function cachedTrajectoryForScope(projectRoot: string, continuityId: string): Record<string, any> | null {
    const cached = getLastTrajectoryClarity();
    if (!cached) return null;
    const cachedRoot = normalizeProjectRoot(cached.project_root);
    const cachedContinuity = String(cached.continuity_id || "").trim();
    if (!cachedRoot || cachedRoot !== normalizeProjectRoot(projectRoot)) return null;
    if (cachedContinuity && continuityId && cachedContinuity !== continuityId) return null;
    return cached;
  }

  async function focusaFetchDetailed(
    path: string,
    opts: RequestInit = {}
  ): Promise<{ ok: boolean; status: number; body: any | null }> {
    const method = String(opts.method || "GET").toUpperCase();
    const timeout = timeoutBudgetForRoute(path, method);
    const bindingDecision = currentProjectBindingDecision();
    const bindingRecoveryRoute =
      path.startsWith("/project/identity") ||
      path.startsWith("/project/verify") ||
      path.startsWith("/workpoint/resume");
    if (
      !["GET", "HEAD", "OPTIONS"].includes(method) &&
      !bindingRecoveryRoute &&
      !projectBindingAllowsDurableWrites(bindingDecision)
    ) {
      const blockedReason = `project_binding_${String(bindingDecision?.state || "unknown").toLowerCase()}`;
      const selectionKey = `project_binding_mutation_selection:${bindingDecision?.evidence_revision || "unknown"}`;
      const firstMutationSelection =
        bindingDecision?.state === "QUARANTINED" && !getAttachmentRuntime().vitalInfoPrompted[selectionKey];
      if (firstMutationSelection) {
        getAttachmentRuntime().vitalInfoPrompted[selectionKey] = Date.now();
        getAttachmentRuntime().projectBindingTelemetry.operator_interruption_count += 1;
      }
      getAttachmentRuntime().projectBindingTelemetry.blocked_write_reasons[blockedReason] =
        (getAttachmentRuntime().projectBindingTelemetry.blocked_write_reasons[blockedReason] || 0) + 1;
      persistState();
      return {
        ok: false,
        status: 409,
        body: {
          status: "blocked",
          canonical: true,
          degraded: true,
          failure_class: "scope_recovery_required",
          error: "durable project mutation is fenced until ProjectBindingDecisionV1 state is BOUND",
          binding_state: bindingDecision?.state || "unknown",
          capability_tier: bindingDecision?.permitted_capability_tier || "recovery_read_plan",
          operator_selection_required: firstMutationSelection,
          duplicate_selection_suppressed: bindingDecision?.state === "QUARANTINED" && !firstMutationSelection,
          candidates: firstMutationSelection
            ? (bindingDecision?.candidates || []).slice(0, 4).map((candidate) => ({
                project_root: candidate.project_root,
                score: candidate.score,
                sources: candidate.sources,
                markers: candidate.markers,
              }))
            : [],
          next_tools: ["focusa_project_identity", "focusa_project_verify", "focusa_workpoint_resume"],
          retry: { safe: true, posture: "verify_scope_first" },
        },
      };
    }
    const base = getAttachmentRuntime().cfg?.focusaApiBaseUrl || "http://127.0.0.1:8787/v1";
    const token = getAttachmentRuntime().cfg?.focusaToken || "";
    const currentKey = currentAttachmentKey();
    if (!currentKey) throw new Error("attachment_runtime_key_required");
    const activeWorkpoint = getActiveWorkpointPacket() as any;
    const markerProjectRoot = resolveCanonicalMarkerProjectRoot(process.cwd());
    const activeWorkpointRoot = normalizeProjectRoot(activeWorkpoint?.project_root);
    const activeWorkpointContinuity = String(activeWorkpoint?.continuity_id || "").trim();
    const workpointScopeAuthoritative =
      !!activeWorkpoint &&
      activeWorkpoint.canonical !== false &&
      activeWorkpoint.action_authority_for_current_ask !== false &&
      activeWorkpointContinuity !== "extension-bootstrap" &&
      (!markerProjectRoot || activeWorkpointRoot === markerProjectRoot);
    const verifiedRoot = normalizeProjectRoot(
      (workpointScopeAuthoritative ? activeWorkpoint.project_root : "") ||
        getLastProjectIdentity()?.project_root ||
        getLastProjectVerify()?.project_root ||
        ""
    );
    const verifiedContinuity = String(
      (workpointScopeAuthoritative ? activeWorkpoint.continuity_id : "") || getContinuityId() || ""
    ).trim();
    const currentRoot = normalizeProjectRoot(currentKey.workstream.root_scope.root_path);
    const currentContinuity = String(currentKey.workstream.continuity_id || "").trim();
    const verifiedScopeAvailable =
      isProjectRootAuthoritySafe(verifiedRoot) &&
      verifiedContinuity &&
      verifiedContinuity !== "extension-bootstrap";
    const currentScopeAvailable =
      isProjectRootAuthoritySafe(currentRoot) &&
      currentContinuity &&
      currentContinuity !== "extension-bootstrap";
    const attachmentKey =
      verifiedScopeAvailable &&
      (workpointScopeAuthoritative || !currentScopeAvailable || currentRoot !== verifiedRoot)
        ? {
            ...currentKey,
            workstream: buildProjectWorkstreamKey(verifiedRoot, verifiedContinuity),
          }
        : currentKey;
    const scopeHeaders = {
      "x-scope-project-root": attachmentKey.workstream.root_scope.root_path,
      "x-scope-continuity-id": attachmentKey.workstream.continuity_id,
      "x-scope-session-id": getSessionFrameKey() || attachmentKey.session_id,
      "x-scope-id": attachmentKey.workstream.root_scope.scope_id,
      "x-scope-kind": attachmentKey.workstream.root_scope.scope_kind,
      "x-scope-attachment-id": attachmentKey.attachment_id,
    };
    const ac = new AbortController();
    const t = setTimeout(() => ac.abort(), timeout);
    try {
      const r = await fetch(`${base}${path}`, {
        ...opts,
        headers: {
          "Content-Type": "application/json",
          ...scopeHeaders,
          ...(token ? { Authorization: `Bearer ${token}` } : {}),
          ...((opts.headers as Record<string, string>) || {}),
        },
        signal: ac.signal,
      });
      let body: any = null;
      try {
        body = await r.json();
      } catch {
        body = null;
      }
      if (!r.ok && body === null) {
        body = {
          status: "blocked",
          failure_class: "non_json_http_error",
          error: `daemon returned HTTP ${r.status} without a JSON recovery envelope`,
          request_scope: scopeHeaders,
          request_overrides: (opts.headers as Record<string, string>) || {},
        };
      }
      if (r.ok && !["GET", "HEAD", "OPTIONS"].includes(method)) {
        const responseRoot = normalizeProjectRoot(
          body?.scope?.root_scope?.root_path || body?.project_root || body?.project_identity?.project_root
        );
        const responseContinuity = String(body?.scope?.continuity_id || body?.continuity_id || "").trim();
        const requestedRoot = normalizeProjectRoot(attachmentKey.workstream.root_scope.root_path);
        const requestedContinuity = attachmentKey.workstream.continuity_id;
        if (
          (!responseRoot || responseRoot === requestedRoot) &&
          (!responseContinuity || responseContinuity === requestedContinuity)
        ) {
          publishScopedStateChange({
            source: "tool",
            mutation_kind: path.split("?")[0] || path,
            project_root: requestedRoot,
            continuity_id: requestedContinuity,
            status: body?.degraded === true ? "degraded" : "accepted",
            evidence_revision:
              String(
                body?.revision || body?.event_id || body?.receipt_id || body?.workpoint_id || ""
              ).trim() || undefined,
            effective_at: new Date().toISOString(),
          });
        }
      }
      return { ok: r.ok, status: r.status, body };
    } catch (err: any) {
      const aborted = err?.name === "AbortError";
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
          retry: {
            safe: routeTier !== "warm",
            posture: routeTier === "cold" ? "safe_retry" : "check_side_effects_first",
          },
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

  function explainWorkLoopResult(
    result: { ok: boolean; status: number; body: any | null },
    fallback: string
  ): string {
    if (result.ok) return fallback;
    const msg = String(result.body?.error || "").toLowerCase();
    const activeWriter = result.body?.active_writer ? ` (${result.body.active_writer})` : "";
    if (msg.includes("claimed by another writer"))
      return `blocked: loop controlled by another session${activeWriter}`;
    if (msg.includes("worktree is not clean")) return "blocked: worktree has uncommitted changes";
    if (msg.includes("missing required header")) return "blocked: controller identity header missing";
    if (result.body?.failure_class === "cold_path_timeout")
      return "blocked: cold route timed out; hot tools may still be healthy";
    if (result.body?.failure_class === "hot_path_timeout") {
      const timeoutMs = String(
        result.body?.resource_mode?.budget?.hot_route_timeout_ms || result.body?.hot_route_timeout_ms || "250"
      );
      const mode = String(result.body?.resource_mode?.mode || "unknown");
      return `blocked: hot route timed out (limit=${timeoutMs}ms, mode=${mode}); retry after brief backoff or run focusa_resource_mode activate_lowmem to extend budget`;
    }
    if (
      result.body?.failure_class === "scope_mismatch" ||
      result.body?.status === "rejected_scope_mismatch" ||
      result.status === 409
    ) {
      const field = String(result.body?.field || "scope");
      const expected = String(
        result.body?.expected_project_root || result.body?.expected_continuity_id || "unknown"
      );
      const actual = String(
        result.body?.packet_project_root || result.body?.packet_continuity_id || "unknown"
      );
      const hint = String(
        result.body?.next_step_hint || "resume/checkpoint the Workpoint in the same scope before retrying"
      );
      return `blocked: scope mismatch on ${field} expected=${expected} packet=${actual}; ${hint}`;
    }
    if (result.status === 0) return "blocked: daemon unavailable";
    return `blocked: ${result.body?.error || `request failed (${result.status})`}`;
  }

  function trajectoryTimeoutFallbackResult(
    action: string,
    endpoint: string,
    body: any,
    response: any,
    nextTools: string[],
    extra: Record<string, unknown> = {}
  ) {
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
    setLastTrajectoryClarity({
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
    });
    try {
      getAttachmentRuntime().pi?.appendEntry("focusa-trajectory-timeout-fallback", fallback);
    } catch {
      /* best effort */
    }
    persistState();
    return {
      content: [{ type: "text", text: timeoutPreservedText(`trajectory ${action}`) }],
      details: {
        ok: false,
        status: "timeout_preserved",
        endpoint,
        canonical: false,
        degraded: true,
        advisory_only: true,
        failure_class: "hot_path_timeout",
        fallback: compactFallbackPacket(fallback),
        response: compactApiEcho(response),
        next_tools: nextTools.slice(0, 4),
      },
    } as any;
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
    const pairObserved =
      healthy &&
      !!replayPayload?.secondary_loop_closure_replay_evidence?.evidence?.current_task_pair_observed;
    const pairLabel = healthy ? (pairObserved ? "observed" : "missing") : "unknown";
    const continuityGateRaw = String(continuityPayload?.state || (healthy ? "open" : "fail-closed"));
    const continuityGate: "open" | "fail-closed" = continuityGateRaw === "open" ? "open" : "fail-closed";
    const continuityFailClosed = continuityGate !== "open";

    const nonClosureObjectiveEvents =
      objectiveProfile?.non_closure_objective_events != null
        ? Number(objectiveProfile.non_closure_objective_events)
        : null;
    const nonClosureObjectiveRate =
      objectiveProfile?.non_closure_objective_rate != null
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

  async function enforceTrajectoryClarityPrecondition(
    projectRoot: string,
    actionLabel: string,
    opts: { blockOperatorInput?: boolean; continuityId?: string; sessionId?: string } = {}
  ): Promise<{ ok: boolean; text?: string; details: Record<string, any> }> {
    const query = new URLSearchParams();
    query.set("project_root", projectRoot);
    const sessionId = String(opts.sessionId || getAttachmentRuntime().sessionFrameKey || "").trim();
    const continuityId = String(
      opts.continuityId || getContinuityId() || ensureContinuityId(projectRoot) || ""
    ).trim();
    if (sessionId) query.set("session_id", sessionId);
    if (continuityId) query.set("continuity_id", continuityId);
    query.set("mode", "summary");
    const result = await focusaFetchDetailed(`/trajectory/view?${query.toString()}`, { method: "GET" });
    const body = result.body || {};
    const clarity = body.intelligence_view?.clarity_gate || {};
    const status = String(clarity.status || body.trajectory?.definition_status || "unknown");
    const action = String(
      clarity.recommended_action ||
        body.intelligence_view?.context_sufficiency?.recommended_action ||
        "unknown"
    );
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
      return {
        ok: false,
        text: `${actionLabel} blocked → trajectory clarity gate unavailable (${explainWorkLoopResult(result, "trajectory unavailable")})`,
        details: { ...details, failure_class: result.body?.failure_class || "daemon_unavailable" },
      };
    }
    if (projectStatus === "mismatch" || status === "conflicted") {
      const recovery = scopeRecoveryContext(body, projectRoot, continuityId, "trajectory_clarity_gate");
      if (allowsWorkpointBootstrapFromClarity(body, projectRoot, actionLabel)) {
        return {
          ok: true,
          text: recovery?.text,
          details: {
            ...details,
            failure_class: "scope_mismatch",
            bootstrap_allowed: true,
            precondition_warning:
              "trajectory conflicted because existing Focusa context is for another continuity; checkpointing current operator mission is allowed",
            scope_recovery_context: recovery?.details || null,
          },
        };
      }
      return {
        ok: false,
        text: `${actionLabel} blocked → trajectory clarity gate conflicted; verify project identity and trajectory before canonical mutation.${recovery ? ` ${recovery.text}` : ""}`,
        details: {
          ...details,
          failure_class: "scope_mismatch",
          scope_recovery_context: recovery?.details || null,
        },
      };
    }
    if (opts.blockOperatorInput !== false && (status === "unclear" || action === "operator_input")) {
      const recovery = scopeRecoveryContext(body, projectRoot, continuityId, "trajectory_clarity_gate");
      if (allowsWorkpointBootstrapFromClarity(body, projectRoot, actionLabel)) {
        return {
          ok: true,
          text: recovery?.text,
          details: {
            ...details,
            failure_class: "validation_rejected",
            bootstrap_allowed: true,
            precondition_warning:
              "trajectory unclear; checkpointing explicit operator mission is allowed to establish Workpoint continuity",
            scope_recovery_context: recovery?.details || null,
          },
        };
      }
      return {
        ok: false,
        text: `${actionLabel} blocked → trajectory unclear; define or confirm trajectory before canonical mutation.${recovery ? ` ${recovery.text}` : ""}`,
        details: {
          ...details,
          failure_class: "validation_rejected",
          scope_recovery_context: recovery?.details || null,
        },
      };
    }
    return { ok: true, details };
  }

  function evidenceClarityFallbackResult(
    kind: "evidence capture" | "workpoint evidence link",
    p: any,
    projectRoot: string,
    clarity: { text?: string; details: Record<string, any> }
  ): any | null {
    const failureClass = String(clarity.details?.failure_class || "");
    const recoverable = [
      "hot_path_timeout",
      "cold_path_timeout",
      "daemon_unavailable",
      "resource_exhausted",
      "read_model_lag",
    ].includes(failureClass);
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
      next_tools: [
        "focusa_workpoint_checkpoint",
        "focusa_workpoint_resume",
        "focusa_trajectory_view",
        "focusa_tool_doctor",
      ],
      raw: {
        trajectory_clarity_precondition: clarity.details,
        proof_preserved_not_linked: true,
        why,
        project_root: projectRoot,
      },
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

  type WorkLoopWriterLease = {
    writerId: string;
    fencingToken: number;
    expiresAt: string;
  };
  const localWriterId = `pi-${process.pid}`;
  const workLoopLeases = new Map<string, WorkLoopWriterLease>();

  function workLoopLeaseKey(): string {
    const root = resolvePiProjectRoot(getSessionCwd() || process.cwd());
    const continuity = getContinuityId() || ensureContinuityId(root) || "unbound";
    return `${root}|${continuity}`;
  }

  function rememberWorkLoopLease(body: any): WorkLoopWriterLease | null {
    if (!body?.writer_id && compatibleWorkLoopStatusState(body) === "unsupported") return null;
    const writerId = String(body?.writer_id || body?.execution_partition?.writer_key || "").trim();
    const fencingToken = Number(body?.fencing_token ?? body?.execution_partition?.fencing_token);
    const expiresAt = String(body?.lease_expires_at || body?.execution_partition?.lease_expires_at || "");
    if (!writerId || !Number.isSafeInteger(fencingToken) || fencingToken <= 0 || !expiresAt) return null;
    const lease = { writerId, fencingToken, expiresAt };
    workLoopLeases.set(workLoopLeaseKey(), lease);
    return lease;
  }

  async function preferredWriterId(): Promise<string> {
    return localWriterId;
  }

  async function currentWorkLoopLease(): Promise<WorkLoopWriterLease | null> {
    const cached = workLoopLeases.get(workLoopLeaseKey());
    if (cached && cached.writerId === localWriterId && Date.parse(cached.expiresAt) > Date.now())
      return cached;
    const status = await focusaFetchDetailed("/work-loop/status?summary_only=true");
    const lease = rememberWorkLoopLease(status.body);
    return lease?.writerId === localWriterId && Date.parse(lease.expiresAt) > Date.now() ? lease : null;
  }

  function writerLeaseHeaders(writerId: string, lease: WorkLoopWriterLease | null): Record<string, string> {
    const headers: Record<string, string> = { "x-focusa-writer-id": writerId };
    if (lease?.writerId === writerId) headers["x-focusa-fencing-token"] = String(lease.fencingToken);
    return headers;
  }

  async function requiredWriterLeaseHeaders(): Promise<Record<string, string>> {
    const lease = await currentWorkLoopLease();
    if (!lease)
      throw new Error(
        "current scoped Work Loop writer lease is missing, expired, or owned by another writer"
      );
    return writerLeaseHeaders(localWriterId, lease);
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
    description:
      "Read current work-loop writer ownership and mutation preflight guidance without mutating state.",
    parameters: Type.Object({}),
    async execute() {
      const result = await focusaFetchDetailed("/work-loop/status?summary_only=true");
      const body = result.body || {};
      const activeWriter = String(body.active_writer || "none");
      const status = String(body.status || body.current_task?.status || "unknown");
      const text = `work-loop writer-status → active_writer=${activeWriter} status=${status} preflight=read_only`;
      return {
        content: [{ type: "text", text }],
        details: {
          ok: result.ok,
          status: String(result.status),
          active_writer: activeWriter,
          authorship_mode: body.authorship_mode,
          preflight: {
            mutates: false,
            writer_required_for: ["control", "context", "checkpoint", "select_next"],
          },
          response: compactApiEcho(body),
        },
      } as any;
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
          content: [
            {
              type: "text",
              text: `Work-loop summary ${explainWorkLoopResult(result, "ok")} | replay=not_checked_hot_path`,
            },
          ],
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
      const enabled =
        typeof loopStatus?.enabled === "boolean" ? loopStatus.enabled : !!loopStatus?.work_loop?.enabled;
      const activeWriter = String(loopStatus?.active_writer || "none");
      const budget = formatWorkLoopBudgetRemaining(loopStatus?.budget_remaining);
      return {
        content: [
          {
            type: "text",
            text: `Work-loop summary: ${statusText} (enabled=${enabled ? "yes" : "no"}) active_writer=${activeWriter} budget_remaining=${budget} replay=not_checked_hot_path`,
          },
        ],
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
      preset: Type.Optional(
        Type.Union([
          Type.Literal("conservative"),
          Type.Literal("balanced"),
          Type.Literal("push"),
          Type.Literal("audit"),
        ])
      ),
      preflight: Type.Optional(
        Type.Boolean({
          description: "If true, only report intended route/writer and do not mutate work-loop state.",
        })
      ),
      root_work_item_id: Type.Optional(
        Type.String({
          description: "Optional root provider WorkItem id. If omitted, infer from the active scoped task.",
        })
      ),
      renew_budget: Type.Optional(
        Type.Boolean({ description: "Explicitly start a fresh budget epoch when action=resume." })
      ),
      max_turns: Type.Optional(Type.Number({ minimum: 1 })),
      max_wall_clock_ms: Type.Optional(Type.Number({ minimum: 1000 })),
      max_retries: Type.Optional(Type.Number({ minimum: 0 })),
      cooldown_ms: Type.Optional(Type.Number({ minimum: 0 })),
    }),
    async execute(_id, params) {
      const {
        action,
        reason,
        preset,
        preflight,
        root_work_item_id,
        renew_budget,
        max_turns,
        max_wall_clock_ms,
        max_retries,
        cooldown_ms,
      } = params as {
        action: "on" | "pause" | "resume" | "stop";
        reason?: string;
        preset?: "conservative" | "balanced" | "push" | "audit";
        preflight?: boolean;
        root_work_item_id?: string;
        renew_budget?: boolean;
        max_turns?: number;
        max_wall_clock_ms?: number;
        max_retries?: number;
        cooldown_ms?: number;
      };
      const writerId = await preferredWriterId();

      if (preflight) {
        const route =
          action === "on"
            ? "/work-loop/enable"
            : action === "pause"
              ? "/work-loop/pause"
              : action === "resume"
                ? "/work-loop/resume"
                : "/work-loop/stop";
        return {
          content: [
            {
              type: "text",
              text: `work-loop ${action} preflight → route=${route} writer=${writerId} mutates=false`,
            },
          ],
          details: {
            ok: true,
            action: String(action),
            status: "preflight",
            route,
            writer_id: writerId,
            mutates: false,
          },
        } as any;
      }

      if (action === "on") {
        const rootWorkItemId = await inferRootWorkItemId(root_work_item_id);
        const payload = {
          preset: preset || getAttachmentRuntime().cfg?.workLoopPreset || "balanced",
          root_work_item_id: rootWorkItemId || undefined,
          policy_overrides: {
            max_turns: max_turns ?? getAttachmentRuntime().cfg?.workLoopMaxTurns,
            max_wall_clock_ms: max_wall_clock_ms ?? getAttachmentRuntime().cfg?.workLoopMaxWallClockMs,
            max_retries: max_retries ?? getAttachmentRuntime().cfg?.workLoopMaxRetries,
            cooldown_ms: cooldown_ms ?? getAttachmentRuntime().cfg?.workLoopCooldownMs,
            allow_destructive_actions: getAttachmentRuntime().cfg?.workLoopAllowDestructiveActions,
            require_operator_for_governance: getAttachmentRuntime().cfg?.workLoopRequireOperatorForGovernance,
            require_operator_for_scope_change:
              getAttachmentRuntime().cfg?.workLoopRequireOperatorForScopeChange,
            require_verification_before_persist:
              getAttachmentRuntime().cfg?.workLoopRequireVerificationBeforePersist,
            max_consecutive_low_productivity_turns:
              getAttachmentRuntime().cfg?.workLoopMaxConsecutiveLowProductivityTurns,
            max_consecutive_failures: getAttachmentRuntime().cfg?.workLoopMaxConsecutiveFailures,
            auto_pause_on_operator_message: getAttachmentRuntime().cfg?.workLoopAutoPauseOnOperatorMessage,
            require_explainable_continue_reason:
              getAttachmentRuntime().cfg?.workLoopRequireExplainableContinueReason,
            max_same_subproblem_retries: getAttachmentRuntime().cfg?.workLoopMaxSameSubproblemRetries,
            status_heartbeat_ms: getAttachmentRuntime().cfg?.workLoopStatusHeartbeatMs,
          },
        };
        const res = await focusaFetchDetailed("/work-loop/enable", {
          method: "POST",
          headers: { ...writerLeaseHeaders(writerId, null), "x-focusa-approval": "approved" },
          body: JSON.stringify(payload),
        });
        if (res.ok) rememberWorkLoopLease(res.body);
        return {
          content: [
            {
              type: "text",
              text: `work-loop on → ${explainWorkLoopResult(res, String(res.body?.status || "accepted"))}`,
            },
          ],
          details: {
            ok: res.ok,
            action: String(action),
            status: res.status,
            response: compactApiEcho(res.body),
          },
        };
      }

      const lease = await currentWorkLoopLease();
      if (!lease) {
        return {
          content: [
            {
              type: "text",
              text: "work-loop control blocked: current scoped writer lease is missing, expired, or owned by another writer",
            },
          ],
          details: {
            ok: false,
            status: "blocked",
            failure_class: "writer_conflict",
            next_tools: ["focusa_work_loop_writer_status", "focusa_work_loop_control"],
          },
        } as any;
      }
      const route =
        action === "pause"
          ? "/work-loop/pause"
          : action === "resume"
            ? "/work-loop/resume"
            : "/work-loop/stop";
      const res = await focusaFetchDetailed(route, {
        method: "POST",
        headers: {
          ...writerLeaseHeaders(writerId, lease),
          ...(action === "resume" &&
          (renew_budget ||
            max_turns !== undefined ||
            max_wall_clock_ms !== undefined ||
            max_retries !== undefined ||
            cooldown_ms !== undefined)
            ? { "x-focusa-approval": "approved" }
            : {}),
        },
        body: JSON.stringify({
          reason: reason?.slice(0, 200) || `operator ${action} via focusa_work_loop_control`,
          ...(action === "resume"
            ? {
                renew_budget: renew_budget || false,
                policy_overrides:
                  max_turns !== undefined ||
                  max_wall_clock_ms !== undefined ||
                  max_retries !== undefined ||
                  cooldown_ms !== undefined
                    ? { max_turns, max_wall_clock_ms, max_retries, cooldown_ms }
                    : undefined,
              }
            : {}),
        }),
      });
      if (res.ok) {
        if (action === "stop") workLoopLeases.delete(workLoopLeaseKey());
        else rememberWorkLoopLease(res.body);
      }
      return {
        content: [
          {
            type: "text",
            text: `work-loop ${action} → ${explainWorkLoopResult(res, String(res.body?.status || "accepted"))}`,
          },
        ],
        details: {
          ok: res.ok,
          action: String(action),
          status: res.status,
          response: compactApiEcho(res.body),
        },
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
      excluded_context_reason: Type.Optional(
        Type.String({ description: "Reason for excluding carryover context (optional)." })
      ),
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
        return {
          content: [{ type: "text", text: "current_ask required." }],
          details: { ok: false, status: 0, response: null },
        };
      }
      const res = await focusaFetchDetailed("/work-loop/context", {
        method: "POST",
        headers: await requiredWriterLeaseHeaders(),
        body: JSON.stringify({
          current_ask: p.current_ask.slice(0, 240),
          ask_kind: p.ask_kind,
          scope_kind: p.scope_kind,
          carryover_policy: p.carryover_policy,
          excluded_context_reason: p.excluded_context_reason,
          excluded_context_labels: p.excluded_context_labels,
          operator_steering_detected: p.operator_steering_detected,
          source_turn_id: p.source_turn_id || `pi-turn-${getTurnCount()}`,
        }),
      });
      return {
        content: [
          {
            type: "text",
            text: `work-loop context → ${explainWorkLoopResult(res, String(res.body?.status || "accepted"))}`,
          },
        ],
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
      const res = await focusaFetchDetailed("/work-loop/checkpoint", {
        method: "POST",
        headers: await requiredWriterLeaseHeaders(),
        body: JSON.stringify({
          summary: (summary || "manual checkpoint via focusa_work_loop_checkpoint").slice(0, 240),
        }),
      });
      return {
        content: [
          {
            type: "text",
            text: `work-loop checkpoint → ${explainWorkLoopResult(res, String(res.body?.checkpoint_id || res.body?.status || "accepted"))}`,
          },
        ],
        details: { ok: res.ok, status: res.status, response: compactApiEcho(res.body) },
      };
    },
  });

  pi.registerTool({
    name: "focusa_work_loop_select_next",
    label: "Work Loop Select Next",
    description: "Ask daemon to defer blocked work and select next ready work item.",
    parameters: Type.Object({
      parent_work_item_id: Type.Optional(
        Type.String({ description: "Parent work item id. If omitted, use active current_task work_item_id." })
      ),
    }),
    async execute(_id, params) {
      const { parent_work_item_id } = params as { parent_work_item_id?: string };
      const writerId = await preferredWriterId();
      const parentWorkItemId = await inferRootWorkItemId(parent_work_item_id);
      if (!parentWorkItemId) {
        return {
          content: [
            {
              type: "text",
              text: "work-loop select-next → blocked: no active parent work item (pass parent_work_item_id or create ready BD)",
            },
          ],
          details: {
            ok: false,
            status: 422,
            response: {
              error:
                "parent_work_item_id required when no current_task is active and no bd ready item is available",
            },
          },
        };
      }
      const res = await focusaFetchDetailed("/work-loop/select-next", {
        method: "POST",
        headers: await requiredWriterLeaseHeaders(),
        body: JSON.stringify({ parent_work_item_id: parentWorkItemId }),
      });
      return {
        content: [
          {
            type: "text",
            text: `work-loop select-next → ${explainWorkLoopResult(res, String(res.body?.status || "accepted"))}`,
          },
        ],
        details: { ok: res.ok, status: res.status, response: compactApiEcho(res.body) },
      };
    },
  });

  // ── Spec88 Workpoint Continuity tools ────────────────────────────────────

  function summarizeWorkpointResponse(body: any): string {
    const status = String(body?.status || "unknown");
    const id = String(
      body?.workpoint_id || body?.active_workpoint_id || body?.requested_workpoint_id || "none"
    );
    const canonical = typeof body?.canonical === "boolean" ? String(body.canonical) : "unknown";
    const next = String(
      body?.next_step_hint ||
        body?.resume_packet?.next_slice ||
        body?.workpoint?.next_slice ||
        "resume from typed workpoint packet"
    );
    // FOCUSA_FIX-9q5l: include mission + next_slice + action so the operator sees
    // what was checkpointed, not just the next= resume pointer.
    const mission = String(body?.mission || body?.resume_packet?.mission || body?.workpoint?.mission || "");
    const actionRaw =
      body?.action_intent || body?.resume_packet?.action_intent || body?.workpoint?.action_intent || "";
    const action = typeof actionRaw === "string" ? actionRaw : actionRaw ? JSON.stringify(actionRaw) : "";
    let summary = `status=${status} id=${id} canonical=${canonical}`;
    if (mission) summary += ` mission="${truncateForSummary(mission, 80)}"`;
    if (action) summary += ` action="${truncateForSummary(action, 80)}"`;
    summary += ` next=${truncateForSummary(next, 80)}`;
    // FOCUSA_FIX-nzru: Annotate freshness when workpoint packet has age metadata
    const updatedAt = String(
      body?.resume_packet?.updated_at || body?.workpoint?.updated_at || body?.updated_at || ""
    );
    let freshnessMarker = "";
    if (updatedAt) {
      const updatedMs = Date.parse(updatedAt);
      if (!Number.isNaN(updatedMs)) {
        const ageMin = Math.round((Date.now() - updatedMs) / 60000);
        if (ageMin > 60)
          freshnessMarker = ` packet_age=${ageMin}min (consider re-checkpointing if next_action refers to closed items)`;
        else if (ageMin > 0) freshnessMarker = ` packet_age=${ageMin}min`;
      }
    }
    return `${summary}${freshnessMarker}`;
  }

  function buildStateHygieneReport(stackBody: any): any {
    const frames = stackBody?.stack?.frames || [];
    const latest = Array.isArray(frames) ? frames[frames.length - 1] || {} : {};
    const state = latest?.state || latest?.focus_state || {};
    const slots = [
      "intent",
      "current_focus",
      "next_steps",
      "open_questions",
      "decisions",
      "constraints",
      "failures",
      "recent_results",
      "artifacts",
      "notes",
    ];
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
    const duplicate_groups = Array.from(byValue.values())
      .filter((group) => group.length > 1)
      .map((group, group_index) => ({ group_id: `dup:${group_index}`, count: group.length, signals: group }));
    const stale_candidates = signals
      .filter(
        (signal) =>
          ["next_steps", "open_questions", "current_focus"].includes(signal.slot) &&
          /maybe|unclear|todo|fix|check|old|stale|previous/i.test(signal.value)
      )
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
      recommended_action:
        duplicate_groups.length || stale_candidates.length
          ? "review_plan_then_apply_non_destructive_note"
          : "no_hygiene_needed",
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
      return {
        content: [
          {
            type: "text",
            text: `state hygiene doctor → signals=${result.signal_count} duplicate_groups=${result.duplicate_groups.length} stale_candidates=${result.stale_candidates.length} recommended=${result.recommended_action}`,
          },
        ],
        details: {
          ok: stack.ok,
          status: stack.ok ? "completed" : "blocked",
          canonical: stack.ok,
          degraded: !stack.ok,
          failure_class: stack.ok ? null : scopedResponseFailureClass(stack, stack.body),
          human_readable:
            result.recommended_action === "no_hygiene_needed"
              ? "Focus State hygiene is healthy; no cleanup is needed."
              : "Focus State hygiene signals need a proposal review before any approved action.",
          response: result,
          next_tools: ["focusa_state_hygiene_plan", "focusa_workpoint_resume", "focusa_tool_doctor"],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_state_hygiene_plan",
    label: "Focus State Hygiene Plan",
    description: "Create a proposal-style hygiene plan; does not mutate Focus State.",
    parameters: Type.Object({
      reason: Type.Optional(Type.String({ description: "Why hygiene is being considered." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const stack = await focusaFetchDetailed("/focus/stack", { method: "GET" });
      const report = buildStateHygieneReport(stack.body || {});
      const plan = {
        mutates: false,
        reason: String(p.reason || "operator requested hygiene plan"),
        target_frame_id: report.frame_id,
        exact_duplicate_groups: report.duplicate_groups,
        exact_stale_candidates: report.stale_candidates,
        actions: report.proposal_only_actions,
        apply_requires_approval: true,
      };
      return {
        content: [
          {
            type: "text",
            text: `state hygiene plan → duplicate_groups=${report.duplicate_groups.length} stale_candidates=${report.stale_candidates.length} mutates=false`,
          },
        ],
        details: { ok: stack.ok, status: stack.ok ? "completed" : "degraded", plan, report },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_state_hygiene_apply",
    label: "Focus State Hygiene Apply",
    description:
      "Approval-gated, non-destructive hygiene apply; records an auditable Focus State note via reducer-backed /focus/update.",
    parameters: Type.Object({
      approved: Type.Boolean({ description: "Must be true to apply proposal-safe hygiene." }),
      reason: Type.Optional(Type.String()),
    }),
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
        content: [
          {
            type: "text",
            text: accepted
              ? "state hygiene apply → recorded non-destructive Focus State note"
              : `state hygiene apply blocked → ${String(result.body?.reason || result.body?.status || result.status)}`,
          },
        ],
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
            summary: accepted
              ? "state hygiene apply recorded an auditable note"
              : "state hygiene apply could not write Focus State note",
            retry: {
              safe: !accepted,
              posture: accepted ? "do_not_retry_unchanged" : "safe_retry",
              reason: accepted ? "completed" : "focus_update_unavailable",
            },
            side_effects: accepted ? ["focus_state_note_append"] : [],
            evidence_refs: accepted && result.body?.frame_id ? [`focus_frame:${result.body.frame_id}`] : [],
            next_tools: accepted
              ? ["focusa_state_hygiene_doctor"]
              : ["focusa_tool_doctor", "focusa_workpoint_resume"],
            error: accepted
              ? null
              : {
                  code: "focus_update_unavailable",
                  message: String(result.body?.reason || result.body?.status || result.status),
                },
          },
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_silent_sessions",
    label: "Focusa Silent Sessions (daemon facade)",
    description:
      "Daemon-native Spec133 Silent Session client for status, observation, steering, controls, config, receipts, capabilities, and legacy action compatibility; process-control failures return failure_class=process_control_failed with receipt-backed recovery.",
    promptSnippet:
      "Use as a thin daemon API client. Supply exact session_id/run_id/generation and durable approval/idempotency fields for mutations; the daemon remains canonical authority.",
    parameters: Type.Object({
      action: Type.Optional(
        Type.Union([
          Type.Literal("list"),
          Type.Literal("start"),
          Type.Literal("reopen"),
          Type.Literal("tail"),
          Type.Literal("send"),
          Type.Literal("kill"),
          Type.Literal("health"),
          Type.Literal("interrupt"),
          Type.Literal("restart"),
          Type.Literal("preflight"),
          Type.Literal("watch"),
          Type.Literal("pause"),
          Type.Literal("resume"),
          Type.Literal("config"),
          Type.Literal("receipt"),
          Type.Literal("capabilities"),
        ])
      ),
      session_id: Type.Optional(Type.String({ description: "Exact durable Silent Session id." })),
      session_name: Type.Optional(
        Type.String({ description: "Legacy alias for exact session_id; no legacy name normalization." })
      ),
      run_id: Type.Optional(Type.String({ description: "Exact current run id." })),
      generation: Type.Optional(Type.Integer({ minimum: 1, description: "Exact current run generation." })),
      approval_id: Type.Optional(Type.String({ description: "Durable daemon approval id for mutations." })),
      idempotency_key: Type.Optional(Type.String({ description: "Mutation replay key." })),
      text: Type.Optional(Type.String({ description: "Input or steering text." })),
      command: Type.Optional(
        Type.String({ description: "Legacy alias for text; never executed as a shell command." })
      ),
      cursor: Type.Optional(Type.String({ description: "Opaque event/output cursor." })),
      channel: Type.Optional(Type.String({ description: "Output channel; defaults to stdout." })),
      config: Type.Optional(Type.Any({ description: "Typed preflight/config request body." })),
      approved: Type.Optional(
        Type.Boolean({ description: "Legacy compatibility hint only; never grants authority." })
      ),
      force: Type.Optional(
        Type.Boolean({ description: "Legacy compatibility hint only; daemon policy decides force." })
      ),
    }),
    async execute(_id, params) {
      const p = params as any;
      const action = String(p.action || "list");
      const sessionId = String(p.session_id || p.session_name || "").trim();
      const requireSession = () => {
        if (!sessionId) throw new Error("exact session_id is required");
        return encodeURIComponent(sessionId);
      };
      const exactMutation = () => {
        if (!p.run_id || !p.generation || !p.approval_id || !p.idempotency_key) {
          throw new Error("run_id, generation, approval_id and idempotency_key are required");
        }
        return {
          run_id: p.run_id,
          generation: p.generation,
          approval_id: p.approval_id,
          idempotency_key: p.idempotency_key,
        };
      };
      let result: any;
      if (action === "list") {
        result = await focusaFetchDetailed("/silent-sessions", { method: "GET" });
      } else if (action === "capabilities") {
        result = await focusaFetchDetailed("/silent-sessions/capabilities", { method: "GET" });
      } else if (action === "preflight") {
        result = await focusaFetchDetailed("/silent-sessions/preflight", {
          method: "POST",
          body: JSON.stringify(p.config || {}),
        });
      } else if (["reopen", "health"].includes(action)) {
        result = await focusaFetchDetailed(`/silent-sessions/${requireSession()}`, { method: "GET" });
      } else if (["tail", "watch"].includes(action)) {
        if (!p.run_id || !p.generation) throw new Error("run_id and generation are required");
        const query = new URLSearchParams({
          run_id: String(p.run_id),
          generation: String(p.generation),
          follow: "false",
          channel: String(p.channel || "stdout"),
        });
        if (p.cursor) query.set("cursor", String(p.cursor));
        result = await focusaFetchDetailed(`/silent-sessions/${requireSession()}/output?${query}`, {
          method: "GET",
        });
      } else if (action === "receipt") {
        if (!p.run_id || !p.generation) throw new Error("run_id and generation are required");
        const query = new URLSearchParams({ run_id: String(p.run_id), generation: String(p.generation) });
        result = await focusaFetchDetailed(`/silent-sessions/${requireSession()}/receipts?${query}`, {
          method: "GET",
        });
      } else if (action === "send") {
        result = await focusaFetchDetailed(`/silent-sessions/${requireSession()}/input`, {
          method: "POST",
          body: JSON.stringify({ ...exactMutation(), text: String(p.text || p.command || "") }),
        });
      } else if (action === "config") {
        result = await focusaFetchDetailed(`/silent-sessions/${requireSession()}/config/preview`, {
          method: "POST",
          body: JSON.stringify({ run_id: p.run_id, generation: p.generation, ...(p.config || {}) }),
        });
      } else {
        const routeAction: Record<string, string> = {
          kill: "cancel",
          interrupt: "interrupt",
          restart: "restart",
          start: "start",
          pause: "pause",
          resume: "resume",
        };
        const route = routeAction[action];
        if (!route) throw new Error(`unsupported daemon Silent Session action: ${action}`);
        result = await focusaFetchDetailed(`/silent-sessions/${requireSession()}/${route}`, {
          method: "POST",
          body: JSON.stringify(exactMutation()),
        });
      }
      const payload = result?.data ?? result;
      return {
        content: [
          {
            type: "text",
            text: `silent ${action} → ${String(payload?.status || result?.status || "completed")}`,
          },
        ],
        details: {
          ...payload,
          canonical: payload?.canonical !== false,
          parity: "full",
          authority: "daemon",
          side_effects: payload?.side_effects || [],
          next_tools: ["focusa_silent_sessions", "focusa_tool_doctor"],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_tool_doctor",
    label: "Focusa Tool Doctor",
    description:
      "Diagnose Focusa tool-suite readiness, active Workpoint continuity, daemon health, and likely next repair action.",
    promptSnippet: "Use first when Focusa tools seem blocked, degraded, stale, or confusing.",
    parameters: Type.Object({
      scope: Type.Optional(
        Type.String({
          description: "Optional family/surface to diagnose, e.g. workpoint, focus_state, metacog.",
        })
      ),
    }),
    async execute(_id, params) {
      const p = params as any;
      const health = await focusaFetchDetailed("/health", { method: "GET" });
      const resource = await focusaFetchDetailed("/resource/mode", { method: "GET" });
      const localWorkpoint = getActiveWorkpointPacket();
      const localWorkpointScope = focusaToolWorkpointScope(localWorkpoint);
      const workpoint = localWorkpointScope
        ? {
            ok: true,
            status: 200,
            body: {
              ...(localWorkpoint || {}),
              status: "completed",
              canonical: true,
              source: "exact_scoped_pi_resume_packet",
            },
          }
        : await focusaFetchDetailed("/workpoint/current", { method: "GET" });
      const loop = await focusaFetchDetailed("/work-loop/status?summary_only=true", { method: "GET" });
      const liveContracts = await focusaFetchDetailed("/ontology/tool-contracts", { method: "GET" });
      const uiaiBrowser = await uiaiBrowserHealthCard();
      const ready = health.ok && workpoint.ok;
      const contractSummary = focusaToolContractSummary();
      const scopedContracts =
        String(p.scope || "all") === "all"
          ? FOCUSA_TOOL_CONTRACTS
          : FOCUSA_TOOL_CONTRACTS.filter(
              (contract) =>
                contract.family === String(p.scope || "") || contract.name.includes(String(p.scope || ""))
            );
      const missingDocs = scopedContracts
        .filter((contract) => !contract.doc_path)
        .map((contract) => contract.name);
      const knownExemptions = scopedContracts
        .filter((contract) => contract.exemptions.length > 0)
        .map((contract) => ({ name: contract.name, exemptions: contract.exemptions }));
      const liveContractList = Array.isArray(liveContracts.body?.contracts)
        ? liveContracts.body.contracts
        : [];
      const liveNames = new Set(
        liveContractList.map((contract: any) => String(contract.name || "")).filter(Boolean)
      );
      const staticNames = new Set(FOCUSA_TOOL_CONTRACTS.map((contract) => contract.name));
      const missing_live = FOCUSA_TOOL_CONTRACTS.map((contract) => contract.name).filter(
        (name) => !liveNames.has(name)
      );
      const extra_live = liveContractList
        .map((contract: any) => String(contract.name || ""))
        .filter((name: string) => name && !staticNames.has(name));
      const stale_live_contracts = scopedContracts
        .filter((contract) => {
          const live = liveContractList.find((item: any) => item?.name === contract.name);
          return live && stableJson(live) !== stableJson(contract);
        })
        .map((contract) => contract.name);
      const repairProjectRoot =
        getLastProjectRootResolution()?.projectRoot || resolvePiProjectRoot(getSessionCwd() || process.cwd());
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
        drift_detected:
          !liveContracts.ok ||
          missing_live.length > 0 ||
          extra_live.length > 0 ||
          stale_live_contracts.length > 0,
        repair_commands: [
          `cd ${repairProjectRoot}`,
          "cargo build --release --bins",
          portableDaemonRestart,
          "curl -sS --max-time 5 http://127.0.0.1:8787/v1/ontology/tool-contracts | jq '.version, (.contracts|length)'",
          "node scripts/prove-focusa-tool-contracts-live.mjs --safe-fixtures",
        ],
      };
      const hookCounts = getAttachmentRuntime().spec92HookTelemetry.reduce(
        (acc: Record<string, number>, item: any) => {
          const hook = String(item.hook || "unknown");
          acc[hook] = (acc[hook] || 0) + 1;
          return acc;
        },
        {}
      );
      const latestToken = getAttachmentRuntime().spec92TokenTelemetry.at(-1) || null;
      const latestTokenTurn = String((latestToken as any)?.turn_id || "");
      const currentTurnId = `pi-turn-${getTurnCount()}`;
      const latestTokenIsCurrent = latestTokenTurn === currentTurnId || !latestTokenTurn;
      const latestTokenBudgetClass = String((latestToken as any)?.budget_class || "unknown");
      const tokenBudgetStatus = latestTokenIsCurrent
        ? latestTokenBudgetClass
        : `historical:${latestTokenBudgetClass}`;
      const resourceMode = resource.body?.resource_mode || {};
      const latestTransition =
        resourceMode.latest_transition ||
        (Array.isArray(resource.body?.transition_history) ? resource.body.transition_history[0] : null);
      const transitionLabel = latestTransition
        ? `${String(latestTransition.from_mode || "?")}→${String(latestTransition.to_mode || "?")}`
        : "none";
      const sessionResolution = getLastProjectRootResolution();
      const sessionRoot =
        sessionResolution?.projectRoot || resolvePiProjectRoot(getSessionCwd() || process.cwd());
      const sessionScopeSafe = isProjectRootAuthoritySafe(sessionRoot);
      const projectRootNeedsConfirmation = sessionResolution?.requiresOperatorConfirmation === true;
      const workpointStatus = String(workpoint.body?.status || (workpoint.ok ? "ok" : "blocked"));
      const workpointCanonical = workpoint.body?.canonical === true || workpointStatus === "active";
      const recommendations: string[] = [];
      if (!health.ok)
        recommendations.push(
          "Focusa daemon health is blocked; retry hot status or inspect daemon before state writes."
        );
      if (!sessionScopeSafe)
        recommendations.push(
          "Session cwd is broad/unsafe; cd to the project folder or pass explicit project_root to project-aware tools."
        );
      if (projectRootNeedsConfirmation)
        recommendations.push(
          "REQUIRED FIRST: project root confidence is below 90%; use interview/menu to ask the operator which candidate root is correct before Focusa writes."
        );
      if (sessionScopeSafe && !projectRootNeedsConfirmation)
        recommendations.push(
          "REQUIRED NEXT: run focusa_trajectory_view to confirm current functional state, destination, and waypoints before Workpoint/evidence progress tracking."
        );
      if (String(resourceMode.mode || "") === "emergency")
        recommendations.push(
          "Resource mode is emergency; avoid cold/full-payload routes and use focusa_resource_mode for recovery posture."
        );
      // Token budget remediation: spec105 DXUX-008 promises recovery explainability
      if (latestTokenBudgetClass === "critical" || latestTokenBudgetClass === "high") {
        recommendations.push(
          `Token budget is ${latestTokenBudgetClass}. Run focusa_resource_mode activate_lowmem to reduce daemon pressure, then focusa_tool_doctor to confirm recovery.`
        );
      } else if (latestTokenBudgetClass === "elevated") {
        recommendations.push(
          `Token budget is elevated. Prefer summary/bounded tool calls (focusa_trajectory_view, focusa_tool_doctor) and avoid full-payload routes (focusa_traverse with include_payload=true).`
        );
      }
      if (!uiaiBrowser.ok)
        recommendations.push(
          "UIAI browser health/metrics unavailable; browser diagnostics may be stale or unreachable."
        );
      if (uiaiBrowser.pressure === "high")
        recommendations.push(
          "UIAI browser queue pressure is high; narrow browser workload, close stale sessions, or retry after queue drains."
        );
      if (!workpoint.ok || !workpointCanonical) {
        const reason = workpoint.body?.reason || workpoint.body?.status || "no_canonical";
        if (workpointStatus === "not_found") {
          recommendations.push(
            `Workpoint not_found (reason=${reason}). Run focusa_workpoint_checkpoint with project_root=<current> and continuity_id=<current> to create one before evidence or Focus State writes.`
          );
        } else if (workpointStatus === "blocked" || workpointStatus === "rejected_scope_mismatch") {
          recommendations.push(
            `Workpoint ${workpointStatus} (reason=${reason}). Verify project scope with focusa_project_identity, then focusa_workpoint_checkpoint in the current scope before continuing.`
          );
        } else {
          recommendations.push(
            `No canonical active Workpoint (status=${workpointStatus}, reason=${reason}); run focusa_project_identity then focusa_workpoint_checkpoint/resume before evidence or Focus State writes.`
          );
        }
      }
      if (missingDocs.length)
        recommendations.push(
          "Some project-aware tool contracts lack docs; run docs maintenance before release proof."
        );
      if (contractDrift.drift_detected)
        recommendations.push(
          "Tool contract drift detected between Pi static registry and live daemon; rebuild/restart focusa-daemon, then run live contract proof."
        );
      const nextTools = Array.from(
        new Set([
          ...(!health.ok ? ["focusa_tool_doctor"] : []),
          ...(!sessionScopeSafe || projectRootNeedsConfirmation
            ? ["focusa_project_identity", "focusa_trajectory_view"]
            : ["focusa_trajectory_view"]),
          ...(String(resourceMode.mode || "") === "emergency" || uiaiBrowser.pressure === "high"
            ? ["focusa_resource_mode"]
            : []),
          ...(!workpoint.ok || !workpointCanonical
            ? ["focusa_project_identity", "focusa_workpoint_checkpoint", "focusa_workpoint_resume"]
            : []),
          ...(contractDrift.drift_detected ? ["focusa_tool_doctor"] : []),
        ])
      );
      const nextActions =
        !sessionScopeSafe || projectRootNeedsConfirmation
          ? [
              {
                action_type: "operator_input_required",
                prompt:
                  "Confirm the exact existing project_root, choose new-project Genesis, or resume an authorized handoff.",
              },
            ]
          : [];
      const recommendedAction =
        recommendations[0] ||
        "Proceed with explicit project_root for project-aware tools and checkpoint before compaction.";
      const driftCauseCounts = {
        missing_live: contractDrift.missing_live.length,
        extra_live: contractDrift.extra_live.length,
        stale_live_contracts: contractDrift.stale_live_contracts.length,
      };
      const driftSummary = contractDrift.drift_detected
        ? ` drift=yes drift_causes=missing_live:${driftCauseCounts.missing_live},extra_live:${driftCauseCounts.extra_live},stale_live_contracts:${driftCauseCounts.stale_live_contracts} source_refs=static:apps/pi-extension/src/tools.ts,live:/v1/ontology/tool-contracts`
        : "";
      const driftDetails = contractDrift.drift_detected
        ? {
            drift_detected: true,
            cause_counts: driftCauseCounts,
            source_refs: ["static:apps/pi-extension/src/tools.ts", "live:/v1/ontology/tool-contracts"],
            missing_live: contractDrift.missing_live.slice(0, 6),
            extra_live: contractDrift.extra_live.slice(0, 6),
            stale_live_contracts: contractDrift.stale_live_contracts.slice(0, 6),
          }
        : { drift_detected: false };
      const evidenceResult = contractDrift.drift_detected
        ? `readiness=${ready ? "ready" : "degraded"} drift=yes causes=${JSON.stringify(driftCauseCounts)} uiai_browser=${uiaiBrowser.status}/${uiaiBrowser.pressure}`
        : `readiness=${ready ? "ready" : "degraded"} uiai_browser=${uiaiBrowser.status}/${uiaiBrowser.pressure}`;
      const scopeStatus = !sessionScopeSafe
        ? "blocked_unsafe_launcher_cwd"
        : projectRootNeedsConfirmation
          ? "operator_confirmation_required"
          : "verified";
      const text = `tool doctor → readiness=${ready ? "ready" : "degraded"} scope=${String(p.scope || "all")} contracts=${contractSummary.total} live_contracts=${contractDrift.live_ok ? contractDrift.live_count : "blocked"}${driftSummary} scoped=${scopedContracts.length} hooks=${getAttachmentRuntime().spec92HookTelemetry.length} token_budget=${tokenBudgetStatus} resource=${String(resourceMode.mode || "unknown")}/${String(resourceMode.reason || "unknown")} transition=${transitionLabel} health=${health.ok ? "ok" : "blocked"} workpoint=${workpointStatus} work_loop=${loop.ok ? String(loop.body?.status || "ok") : "blocked"} uiai_browser=${uiaiBrowser.status}/${uiaiBrowser.pressure} recommended=${recommendedAction}`;
      return {
        content: [{ type: "text", text }],
        details: {
          ok: ready && !contractDrift.drift_detected,
          status: ready && !contractDrift.drift_detected ? "completed" : "degraded",
          tool_readiness: {
            status: contractDrift.drift_detected ? "degraded" : "ready",
            contracts_total: contractSummary.total,
            live_contracts: contractDrift.live_ok ? contractDrift.live_count : null,
          },
          daemon_health: {
            status: health.ok ? "ready" : "blocked",
            response: compactApiEcho(health.body),
          },
          scope_status: {
            status: scopeStatus,
            project_root: sessionScopeSafe ? sessionRoot : null,
            operator_input_required: nextActions.length > 0,
          },
          workpoint_status: {
            status: workpointStatus,
            canonical: workpointCanonical,
            response: compactApiEcho(workpoint.body),
          },
          work_loop_status: {
            status: loop.ok ? String(loop.body?.status || "ok") : "blocked",
            response: compactApiEcho(loop.body),
          },
          health: compactApiEcho(health.body),
          resource_mode: compactApiEcho(resource.body),
          workpoint: compactApiEcho(workpoint.body),
          work_loop: compactApiEcho(loop.body),
          uiai_browser: compactApiEcho(uiaiBrowser),
          contracts_total: contractSummary.total,
          contracts_by_family: contractSummary.by_family,
          contract_coverage: {
            scoped: scopedContracts.length,
            missing_docs: missingDocs,
            known_exemptions: knownExemptions,
          },
          contract_drift: driftDetails,
          session_scope: {
            cwd: sessionRoot,
            safe: sessionScopeSafe,
            project_root_resolution: compactApiEcho(sessionResolution || null),
          },
          token_budget: {
            status: tokenBudgetStatus,
            budget_class: latestTokenBudgetClass,
            turn_id: latestTokenTurn || null,
            current_turn_id: currentTurnId,
            current: latestTokenIsCurrent,
          },
          recommendations: recommendations.slice(0, 6),
          recommended_action: recommendedAction,
          evidence_capture_suggestion: focusaEvidenceCaptureSuggestion({
            target_ref: "focusa_tool_doctor",
            result: evidenceResult,
            evidence_ref: `focusa_tool_doctor:${String(p.scope || "all")}`,
            project_root: sessionScopeSafe ? sessionRoot : undefined,
            attach_to_workpoint: false,
          }),
          next_tools: nextTools.slice(0, 4),
          next_actions: nextActions,
          spec92: {
            hook_records: getAttachmentRuntime().spec92HookTelemetry.length,
            token_records: getAttachmentRuntime().spec92TokenTelemetry.length,
          },
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_agent_prompt",
    label: "Focusa Agent Prompt",
    description: "Read canonical Pi guidance; prefer focusa_* tools over raw daemon calls.",
    promptSnippet: "Use at session start or when tool routing drifts.",
    parameters: Type.Object({}),
    async execute(_id, _params) {
      const prompt = await focusaFetchDetailed("/agent/prompt", { method: "GET" });
      const body = prompt.body || {};
      return {
        content: [
          {
            type: "text",
            text: `agent prompt → is_agent=${body?.is_agent === true ? "true" : "false"} marker=${String(body?.marker || "")}`,
          },
        ],
        details: {
          ok: prompt.ok,
          status: prompt.ok ? "completed" : "blocked",
          endpoint: "/v1/agent/prompt",
          agent_prompt: body,
          next_tools: [
            "focusa_utility_card",
            "focusa_tool_doctor",
            "focusa_trajectory_view",
            "focusa_project_identity",
          ],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_utility_card",
    label: "Focusa Utility Card",
    description: "Read compact bootstrap, post-compaction, recovery, and brevity guidance.",
    promptSnippet: "Use at startup/resume for exact Focusa scope, recovery, and brevity rules.",
    parameters: strictObject({}),
    execute: async () => {
      const result = await focusaFetchDetailed("/utility/card");
      const body = result.body || {};
      const identity: any = getLastProjectIdentity() || {};
      const canonicalParent = normalizeProjectRoot(identity.canonical_parent_root || identity.project_root);
      const activeWorktree = normalizeProjectRoot(
        identity.active_worktree_root || identity.working_context?.active_worktree_root || canonicalParent
      );
      const workingSubpathId = String(
        identity.working_context?.working_subpath?.working_subpath_id || "primary"
      );
      const ok = result.ok && body.status === "completed";
      const toolResult = focusaToolResult({
        ok,
        status: ok ? "completed" : "blocked",
        summary: `utility card → bootstrap=${Array.isArray(body.bootstrap_card) ? body.bootstrap_card.length : 0} compaction=${Array.isArray(body.post_compaction_card) ? body.post_compaction_card.length : 0}`,
        tool: "focusa_utility_card",
        family: "diagnostics_hygiene",
        side_effects: [],
        evidence_refs: [],
        next_tools: [
          "focusa_agent_prompt",
          "focusa_workpoint_resume",
          "focusa_trajectory_view",
          "focusa_evidence_capture",
        ],
        raw: body,
      });
      return {
        content: [
          {
            type: "text",
            text: `utility card ${body.status || result.status}\nparent=${canonicalParent || "unknown"}\nworktree=${activeWorktree || "unknown"}\nworking_subpath=${workingSubpathId}\nnext=${Array.isArray(body.next_tools) ? body.next_tools.join(", ") : "unknown"}`,
          },
        ],
        details: {
          tool_result_v1: toolResult,
          canonical_parent_root: canonicalParent || null,
          active_worktree_root: activeWorktree || null,
          working_subpath_id: workingSubpathId,
        },
      };
    },
  });

  pi.registerTool({
    name: "focusa_resource_mode",
    label: "Focusa Resource Mode",
    description:
      "Read or control Focusa resource mode, including activating/deactivating LowMem mode when resources are constrained.",
    promptSnippet:
      "Use when resources are low, daemon hot paths risk timeouts, or operator says Activate/Deactivate LowMem mode.",
    parameters: Type.Object({
      action: Type.Optional(
        Type.Union(
          [
            Type.Literal("status"),
            Type.Literal("activate_lowmem"),
            Type.Literal("deactivate_lowmem"),
            Type.Literal("set_mode"),
            Type.Literal("set_normal"),
            Type.Literal("set_constrained"),
            Type.Literal("set_emergency"),
          ],
          {
            description:
              "Mode action. activate_lowmem enables LowMem; deactivate_lowmem clears the runtime override back to auto.",
          }
        )
      ),
      mode: Type.Optional(
        Type.Union(
          [
            Type.Literal("auto"),
            Type.Literal("normal"),
            Type.Literal("constrained"),
            Type.Literal("lowmem"),
            Type.Literal("emergency"),
          ],
          { description: "Optional target mode when action=set_mode." }
        )
      ),
      reason: Type.Optional(Type.String({ description: "Why the mode is being read or changed." })),
      preflight: Type.Optional(
        Type.Boolean({ description: "If true, only read current mode and report intended change." })
      ),
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
            next_tools: [
              "focusa_tool_doctor",
              "focusa_trajectory_view",
              "focusa_workpoint_resume",
              "focusa_traverse",
            ],
            response: compactApiEcho(body),
          },
        } as any;
      }
      const body = { action, mode: p.mode, reason: p.reason || `pi:${action}` };
      const result = await focusaFetchDetailed("/resource/mode", {
        method: "POST",
        body: JSON.stringify(body),
      });
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
          next_tools: response.next_tools || [
            "focusa_tool_doctor",
            "focusa_trajectory_view",
            "focusa_workpoint_resume",
            "focusa_traverse",
          ],
          failure_class: response.failure_class || null,
          response,
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_project_identity",
    label: "Focusa Project Identity",
    description:
      "Resolve bounded ProjectIdentity from cwd/project_root using marker, git, beads, workspace, daemon, and operator project signals.",
    promptSnippet:
      "Use before trusting cross-project Workpoints, Trajectory packets, or project-sensitive context.",
    parameters: Type.Object({
      cwd: Type.Optional(
        Type.String({ description: "Optional cwd/project path hint; defaults to Pi session cwd." })
      ),
      project_root: Type.Optional(Type.String({ description: "Optional expected project root folder." })),
      persisted_project_root: Type.Optional(
        Type.String({
          description:
            "Project root retained by the resumed Pi session; advisory candidate only until current worktree/project evidence verifies it.",
        })
      ),
      remote_host: Type.Optional(
        Type.String({
          description: "Remote SSH host that contains the project root; caller supplies inspected evidence.",
        })
      ),
      remote_user: Type.Optional(Type.String({ description: "Remote SSH user, if known." })),
      remote_port: Type.Optional(
        Type.Integer({ minimum: 1, maximum: 65535, description: "Remote SSH port, if known." })
      ),
      remote_repo_remote: Type.Optional(
        Type.String({ description: "Git origin/repo remote observed on the remote host." })
      ),
      remote_workspace_kind: Type.Optional(
        Type.String({ description: "Workspace kind observed on the remote host." })
      ),
      remote_deploy_root: Type.Optional(
        Type.String({ description: "Deployment/site root observed on the remote host." })
      ),
    }),
    async execute(_id, params) {
      const p = params as {
        cwd?: string;
        project_root?: string;
        persisted_project_root?: string;
        remote_host?: string;
        remote_user?: string;
        remote_port?: number;
        remote_repo_remote?: string;
        remote_workspace_kind?: string;
        remote_deploy_root?: string;
      };
      const query = new URLSearchParams();
      const ambientCwd = normalizeProjectRoot(p.cwd || process.cwd());
      const markerProjectRoot = resolveCanonicalMarkerProjectRoot(ambientCwd);
      const requestCwd = p.cwd || (markerProjectRoot ? ambientCwd : getSessionCwd() || process.cwd());
      const requestedProjectRoot = normalizeProjectRoot(p.project_root);
      const authorityProjectRoot = normalizeProjectRoot(
        markerProjectRoot && requestedProjectRoot === ambientCwd
          ? markerProjectRoot
          : requestedProjectRoot || markerProjectRoot
      );
      query.set("cwd", requestCwd);
      if (authorityProjectRoot) query.set("project_root", authorityProjectRoot);
      if (getSessionFrameKey()) query.set("pi_session_id", getSessionFrameKey());
      const persistedProjectRoot = normalizeProjectRoot(
        p.persisted_project_root ||
          getActiveWorkpointPacket()?.scope?.project_root ||
          getActiveWorkpointPacket()?.project_root ||
          getLastProjectRootResolution()?.projectRoot ||
          getLastProjectIdentity()?.project_root
      );
      if (
        persistedProjectRoot &&
        (!authorityProjectRoot || persistedProjectRoot === authorityProjectRoot)
      ) {
        query.set("persisted_project_root", persistedProjectRoot);
      }
      if (p.remote_host) query.set("remote_host", p.remote_host);
      if (p.remote_user) query.set("remote_user", p.remote_user);
      if (p.remote_port) query.set("remote_port", String(p.remote_port));
      if (p.remote_repo_remote) query.set("remote_repo_remote", p.remote_repo_remote);
      if (p.remote_workspace_kind) query.set("remote_workspace_kind", p.remote_workspace_kind);
      if (p.remote_deploy_root) query.set("remote_deploy_root", p.remote_deploy_root);
      const result = await focusaFetchDetailed(`/project/identity?${query.toString()}`, { method: "GET" });
      const body = result.body || {};
      if (!result.ok && body.failure_class === "hot_path_timeout") {
        const requestedRoot = normalizeProjectRoot(
          p.project_root || p.cwd || getSessionCwd() || process.cwd()
        );
        const cachedIdentity =
          getLastProjectIdentity() &&
          (!requestedRoot || normalizeProjectRoot(getLastProjectIdentity()!.project_root) === requestedRoot)
            ? getLastProjectIdentity()!
            : null;
        return {
          content: [
            {
              type: "text",
              text: timeoutPreservedText(
                "project identity",
                cachedIdentity ? "cached identity" : "empty fallback"
              ),
            },
          ],
          details: {
            ok: false,
            status: "timeout_preserved",
            endpoint: "/v1/project/identity",
            canonical: false,
            degraded: true,
            advisory_only: true,
            project_identity: cachedIdentity || {},
            failure_class: "hot_path_timeout",
            response: compactApiEcho(body),
            next_tools: [
              "focusa_tool_doctor",
              "focusa_resource_mode",
              "focusa_project_identity",
              "focusa_project_verify",
              "focusa_trajectory_view",
            ],
          },
        } as any;
      }
      const identity = body.project_identity || {};
      if (identity && Object.keys(identity).length) {
        // Guard: do not overwrite a verified in-session project identity with a different project's result.
        // After model switch, the session may already hold a valid identity. Overwriting it
        // causes cross-session contamination (SPEC96 emergency fix 2 isolation principle).
        const incomingRoot = normalizeProjectRoot(identity.project_root);
        const existingRoot = normalizeProjectRoot(getLastProjectIdentity()?.project_root);
        const requestedRoot = normalizeProjectRoot(p.project_root);
        const explicitProjectSwitch =
          requestedRoot && incomingRoot === requestedRoot && existingRoot !== requestedRoot;
        const existingConfidence = getLastProjectIdentity()?.confidence;
        const isExistingVerified = existingConfidence === "high" || existingConfidence === "medium";
        const isDifferentProject = existingRoot && incomingRoot && existingRoot !== incomingRoot;
        const isDifferentThanUnverified = isDifferentProject && isExistingVerified && !explicitProjectSwitch;
        if (isDifferentThanUnverified) {
          // Preserve existing verified identity; return it instead of the incoming one.
          const preserved = getLastProjectIdentity()!;
          return {
            content: [
              {
                type: "text",
                text: `project identity → status=verified confidence=${preserved.confidence || "unknown"} parent=${preserved.canonical_parent_root || preserved.project_root || "unknown"} worktree=${preserved.active_worktree_root || preserved.working_context?.active_worktree_root || preserved.project_root || "unknown"} subpath=${preserved.working_context?.working_subpath?.working_subpath_id || "primary"} (preserved from session; incoming result rejected as different project: ${incomingRoot})`,
              },
            ],
            details: {
              ok: true,
              status: "preserved",
              endpoint: "/v1/project/identity",
              canonical: false,
              degraded: false,
              project_identity: preserved,
              project_summary: null,
              summary_lines: [],
              verification: null,
              tool_result_v1: {
                ok: true,
                status: "preserved",
                canonical: false,
                degraded: false,
                failure_class: null,
                retry: { safe: true, posture: "safe_retry" },
                side_effects: [],
                evidence_refs: [],
                next_tools: ["focusa_project_verify", "focusa_trajectory_view", "focusa_workpoint_resume"],
              },
              failure_class: null,
              next_tools: ["focusa_project_verify", "focusa_trajectory_view", "focusa_workpoint_resume"],
              response: compactApiEcho(body),
            },
          } as any;
        }
        setLastProjectIdentity(identity);
        const verifiedRoot = normalizeProjectRoot(identity.project_root);
        if (
          verifiedRoot &&
          identity.status === "verified" &&
          body.binding_decision?.ambiguous !== true &&
          body.status !== "ambiguous_project_binding" &&
          isProjectRootAuthoritySafe(verifiedRoot)
        ) {
          confirmPiProjectRoot(verifiedRoot, "focusa_project_identity_verified");
          ensureContinuityId(verifiedRoot);
          const priorTrajectory = await focusaFetchDetailed(
            `/trajectory/view?project_root=${encodeURIComponent(verifiedRoot)}&mode=summary&allow_prior_project_trajectory=true`,
            { method: "GET" }
          ).catch(() => null);
          const priorContinuity = String(
            priorTrajectory?.body?.trajectory?.fallback_source_continuity_id ||
              priorTrajectory?.body?.project_identity?.continuity_id ||
              priorTrajectory?.body?.continuity_id ||
              ""
          ).trim();
          if (priorContinuity) {
            adoptVerifiedContinuityForCurrentSession(verifiedRoot, priorContinuity);
          }
          persistState();
        }
      }
      const summaryLines = Array.isArray(body.summary_lines)
        ? body.summary_lines.map((line: any) => String(line)).filter(Boolean)
        : Array.isArray(identity.project_summary?.summary_lines)
          ? identity.project_summary.summary_lines.map((line: any) => String(line)).filter(Boolean)
          : [];
      const bindingCandidates = Array.isArray(body.binding_candidates) ? body.binding_candidates : [];
      const bindingSummary = body.binding_decision
        ? `binding=${String(body.binding_decision.status || "unknown")} selected=${String(body.binding_decision.selected_project_root || "none")} candidates=${bindingCandidates.length}`
        : "binding=legacy_unavailable";
      const text = result.ok
        ? [
            `project identity → status=${String(identity.status || body.status || "unknown")} confidence=${String(identity.confidence || "unknown")} parent=${String(identity.canonical_parent_root || identity.project_root || "unknown")} worktree=${String(identity.active_worktree_root || identity.working_context?.active_worktree_root || identity.project_root || "unknown")} subpath=${String(identity.working_context?.working_subpath?.working_subpath_id || "primary")} ${bindingSummary}`,
            body.mismatch_reason ? `mismatch_reason=${body.mismatch_reason}` : null,
            Array.isArray(body.degraded_reasons) && body.degraded_reasons.length > 0
              ? `degraded_reasons=${body.degraded_reasons.map((r: any) => `${r.code}:${r.severity}`).join(", ")}`
              : null,
            ...summaryLines.slice(0, 4),
          ]
            .filter(Boolean)
            .join("\n")
        : `project identity blocked → ${explainWorkLoopResult(result, "project identity unavailable")}`;
      const toolResult = body.details?.tool_result_v1 || {
        ok: result.ok,
        status: result.ok ? String(body.status || "completed") : "blocked",
        canonical: body.canonical === true,
        degraded: body.degraded !== false,
        failure_class: body.failure_class || null,
        retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" },
        side_effects: [],
        evidence_refs: [],
        next_tools: body.next_tools || [
          "focusa_project_verify",
          "focusa_trajectory_view",
          "focusa_workpoint_resume",
        ],
      };
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
          next_tools: toolResult.next_tools ||
            body.next_tools || ["focusa_project_verify", "focusa_trajectory_view", "focusa_workpoint_resume"],
          response: compactApiEcho(body),
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_project_card",
    label: "Focusa Project Card",
    description:
      "Build an advisory project-intelligence card from ProjectIdentity, ontology, trajectory, Workpoint/evidence, prediction, and metacog signals.",
    promptSnippet:
      "Use at bootstrap/re-bootstrap, project reviews, and next-step evaluation before refreshing trajectory hierarchy.",
    parameters: Type.Object({
      cwd: Type.Optional(
        Type.String({ description: "Optional cwd/project path hint; defaults to Pi session cwd." })
      ),
      project_root: Type.Optional(Type.String({ description: "Optional expected project root folder." })),
      current_ask: Type.Optional(
        Type.String({ description: "Optional current ask used to seed bootstrap/re-bootstrap candidate." })
      ),
      remote_host: Type.Optional(
        Type.String({
          description: "Remote SSH host that contains the project root; caller supplies inspected evidence.",
        })
      ),
      remote_user: Type.Optional(Type.String({ description: "Remote SSH user, if known." })),
      remote_port: Type.Optional(
        Type.Number({ minimum: 1, maximum: 65535, description: "Remote SSH port, if known." })
      ),
      remote_repo_remote: Type.Optional(
        Type.String({ description: "Git origin/repo remote observed on the remote host." })
      ),
      remote_workspace_kind: Type.Optional(
        Type.String({ description: "Workspace kind observed on the remote host." })
      ),
      remote_deploy_root: Type.Optional(
        Type.String({ description: "Deployment/site root observed on the remote host." })
      ),
    }),
    async execute(_id, params) {
      const p = params as {
        cwd?: string;
        project_root?: string;
        current_ask?: string;
        remote_host?: string;
        remote_user?: string;
        remote_port?: number;
        remote_repo_remote?: string;
        remote_workspace_kind?: string;
        remote_deploy_root?: string;
      };
      const query = new URLSearchParams();
      const ambientCwd = normalizeProjectRoot(p.cwd || process.cwd());
      const markerProjectRoot = resolveCanonicalMarkerProjectRoot(ambientCwd);
      query.set("cwd", p.cwd || (markerProjectRoot ? ambientCwd : getSessionCwd() || process.cwd()));
      const requestedProjectRoot = normalizeProjectRoot(p.project_root);
      const cardProjectRoot = normalizeProjectRoot(
        markerProjectRoot && requestedProjectRoot === ambientCwd
          ? markerProjectRoot
          : requestedProjectRoot || markerProjectRoot
      );
      if (cardProjectRoot) query.set("project_root", cardProjectRoot);
      if (p.current_ask) query.set("current_ask", p.current_ask);
      if (p.remote_host) query.set("remote_host", p.remote_host);
      if (p.remote_user) query.set("remote_user", p.remote_user);
      if (Number.isFinite(p.remote_port)) query.set("remote_port", String(Math.trunc(Number(p.remote_port))));
      if (p.remote_repo_remote) query.set("remote_repo_remote", p.remote_repo_remote);
      if (p.remote_workspace_kind) query.set("remote_workspace_kind", p.remote_workspace_kind);
      if (p.remote_deploy_root) query.set("remote_deploy_root", p.remote_deploy_root);
      let result = await focusaFetchDetailed(`/project/card?${query.toString()}`, { method: "GET" });
      let body = result.body || {};
      const continuityBeforeRecovery = getContinuityId();
      const priorContinuityCounts = new Map<string, number>();
      for (const frame of body.prior_session_context?.recent_frames || []) {
        const candidate = String(frame?.continuity_id || "").trim();
        if (candidate) priorContinuityCounts.set(candidate, (priorContinuityCounts.get(candidate) || 0) + 1);
      }
      const modalPriorContinuity = [...priorContinuityCounts.entries()]
        .sort((a, b) => b[1] - a[1] || a[0].localeCompare(b[0]))[0]?.[0] || "";
      const authoritativeFallbackContinuity = String(
        body.trajectory_ladder?.fallback_source_continuity_id ||
          body.prior_session_context?.trajectory_ladder?.fallback_source_continuity_id ||
          body.prior_session_context?.fallback_source_continuity_id ||
          modalPriorContinuity ||
          ""
      ).trim();
      const inferredFallbackContinuity = String(
        body.inferred_workpoint_candidate?.source_signals?.prior_session_workpath?.find(
          (entry: any) => entry?.continuity_id
        )?.continuity_id || ""
      ).trim();
      const recoveredContinuity =
        authoritativeFallbackContinuity ||
        (!continuityBeforeRecovery || continuityBeforeRecovery === "extension-bootstrap"
          ? inferredFallbackContinuity
          : "");
      const recoveredRoot = normalizeProjectRoot(
        body.project_identity?.canonical_parent_root || body.project_identity?.project_root
      );
      let continuityRecoveryAdopted = false;
      if (
        recoveredContinuity &&
        recoveredContinuity !== continuityBeforeRecovery &&
        (continuityRecoveryAdopted = adoptVerifiedContinuityForCurrentSession(
          recoveredRoot,
          recoveredContinuity
        ))
      ) {
        result = await focusaFetchDetailed(`/project/card?${query.toString()}`, { method: "GET" });
        body = result.body || {};
      }
      const project = body.project_identity || {};
      const temporalProjectRoot = project.project_root || project.canonical_parent_root || p.project_root;
      const temporalContinuityId = getContinuityId();
      if (temporalProjectRoot && temporalContinuityId) {
        const temporalQuery = new URLSearchParams({
          project_root: String(temporalProjectRoot),
          continuity_id: String(temporalContinuityId),
        });
        const temporalResult = await focusaFetchDetailed(`/temporal/status?${temporalQuery.toString()}`);
        body.temporal_context = temporalResult.body || {
          status: "degraded",
          failure_class: "temporal_projection_unavailable",
        };
      }
      const bootstrap = body.bootstrap || {};
      const prediction = body.prediction || {};
      const ontology = body.ontology || {};
      const prior = body.prior_session_context || {};
      const priorLadder = prior.trajectory_ladder || {};
      const priorDecisionCount = Array.isArray(prior.recent_decisions) ? prior.recent_decisions.length : 0;
      const priorOutcomeCount = Array.isArray(prior.recent_algorithm_outcomes)
        ? prior.recent_algorithm_outcomes.length
        : 0;
      const sequence = body.success_sequence || {};
      const efficiency = body.efficiency_summary || body.trajectory_report_card?.time_and_tokens || {};
      const trajectoryReport = body.trajectory_report_card || {};
      const crosswire = body.crosswire_health || {};
      const inferredWorkpoint =
        body.inferred_workpoint_candidate || bootstrap.candidate?.inferred_workpoint_candidate || {};
      const askToWorkpointBridge =
        body.ask_to_workpoint_bridge || inferredWorkpoint.ask_to_workpoint_bridge || {};
      const waypointSummary = trajectoryReport.accomplishment_summary || {};
      const shortest = sequence.shortest_path_to_success || {};
      const selectedPath = shortest.selected || {};
      const eliminatedCount = Array.isArray(shortest.eliminated_candidates)
        ? shortest.eliminated_candidates.length
        : 0;
      const ontologyCounts = ontology.counts || {};
      const text = result.ok
        ? `project card → project=${String(project.canonical_name || project.project_id || "unknown")} parent=${String(project.canonical_parent_root || project.project_root || "unknown")} worktree=${String(project.active_worktree_root || project.working_context?.active_worktree_root || project.project_root || "unknown")} subpath=${String(project.working_context?.working_subpath?.working_subpath_id || "primary")} bootstrap_needed=${bootstrap.needed === true} hlg=${String(body.trajectory?.hlt || priorLadder.high_level_goal || "missing").slice(0, 80)} stg=${String(body.trajectory?.stg || priorLadder.short_term_goal || "missing").slice(0, 80)} decisions=${priorDecisionCount} outcomes=${priorOutcomeCount} elapsed_avg=${String(efficiency.average_elapsed_hms || "00:00:00")} tokens_avg=${String(efficiency.average_total_tokens ?? 0)} waypoints=${String(waypointSummary.waypoints_accomplished_by_recent_outcomes ?? 0)}/${String(waypointSummary.waypoints_total ?? 0)} crosswire=${String(crosswire.status || (crosswire.prediction_feed?.elapsed_tokens_waypoints_feed_future_predictions === true ? "legacy_unknown" : "check"))} inferred_wp=${String(inferredWorkpoint.current_action || "none")} ask_bridge=${String(askToWorkpointBridge.recommended_bridge_action || "unknown")} exact_next=${String(askToWorkpointBridge.exact_next_action || inferredWorkpoint.next_action || "unknown").slice(0, 80)} next_event=${String(sequence.recommended_first_event || "unknown")} shortest=${String(selectedPath.path_id || "unknown")} cost=${String(selectedPath.cost ?? "unknown")} eliminated=${eliminatedCount} predictions=${String(prediction.total ?? "unknown")}/${String(prediction.evaluated ?? "unknown")} ontology_runtime=${String(ontologyCounts.runtime_objects ?? ontology.runtime_objects ?? "unknown")} ontology_effective=${String(ontologyCounts.effective_project_card_objects ?? ontology.objects ?? "unknown")} ontology_source=${String(ontology.source_index || "unknown")} selector=${String(ontology.selector || "unknown")}`
        : `project card blocked → ${explainWorkLoopResult(result, "project card unavailable")}`;
      const toolResult = body.details?.tool_result_v1 || {
        ok: result.ok,
        status: result.ok ? String(body.status || "completed") : "blocked",
        canonical: false,
        degraded: !result.ok,
        failure_class: body.failure_class || null,
        retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" },
        side_effects: [],
        evidence_refs: [],
        next_tools: body.next_tools || [
          "focusa_project_card_outcome",
          "focusa_traverse",
          "focusa_trajectory_view",
          "focusa_metacog_retrieve",
        ],
      };
      const compactResponse = {
        status: body.status,
        schema: body.schema,
        algorithm_run_id: body.algorithm_run_id,
        bootstrap_needed: bootstrap.needed,
        inferred_workpoint_candidate: inferredWorkpoint,
        ask_to_workpoint_bridge: {
          ask_differs_from_active_workpoint: askToWorkpointBridge.ask_differs_from_active_workpoint,
          recommended_bridge_action: askToWorkpointBridge.recommended_bridge_action,
          exact_next_action: askToWorkpointBridge.exact_next_action,
          checkpoint_payload_hint: askToWorkpointBridge.checkpoint_payload_hint,
          safe_after_identity_verification: askToWorkpointBridge.safe_after_identity_verification,
        },
        trajectory_report_card: trajectoryReport,
        temporal_context: body.temporal_context || { status: "unavailable" },
        efficiency_summary: efficiency,
        crosswire_health: crosswire,
        recommended_first_event: sequence.recommended_first_event,
        ranking_basis: sequence.ranking_basis,
        shortest_path_to_success: {
          selected: selectedPath,
          eliminated_candidates: shortest.eliminated_candidates || [],
        },
        outcome_learning: body.algorithmic_intelligence?.outcome_learning,
        next_tools: body.next_tools,
      };
      return {
        content: [{ type: "text", text }],
        details: {
          ok: result.ok,
          status: String(body.status || (result.ok ? "completed" : "blocked")),
          endpoint: "/v1/project/card",
          advisory_only: body.advisory_only !== false,
          project_identity: project,
          trajectory: body.trajectory || null,
          inferred_workpoint_candidate: inferredWorkpoint,
          ask_to_workpoint_bridge: askToWorkpointBridge,
          trajectory_report_card: trajectoryReport,
          temporal_context: body.temporal_context || { status: "unavailable" },
          efficiency_summary: efficiency,
          crosswire_health: crosswire,
          continuity_recovery: {
            candidate: recoveredContinuity || null,
            project_root: recoveredRoot || null,
            before: continuityBeforeRecovery || null,
            adopted: continuityRecoveryAdopted,
            after: getContinuityId() || null,
          },
          prior_session_context: prior,
          success_sequence: sequence,
          ontology,
          evidence: body.evidence || null,
          prediction,
          algorithmic_intelligence: {
            outcome_learning: body.algorithmic_intelligence?.outcome_learning || null,
            expected_utility: body.algorithmic_intelligence?.expected_utility || null,
          },
          metacognition: body.metacognition || null,
          active_workpoint: body.active_workpoint || null,
          bootstrap,
          possibilities: body.possibilities || [],
          next_step_quality_rule: body.next_step_quality_rule || null,
          tool_result_v1: toolResult,
          next_tools: toolResult.next_tools ||
            body.next_tools || [
              "focusa_workpoint_checkpoint",
              "focusa_project_card_outcome",
              "focusa_traverse",
              "focusa_trajectory_view",
            ],
          response: compactResponse,
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_project_card_outcome",
    label: "Focusa Project Card Outcome",
    description:
      "Attach a final outcome/result to a specific project-card algorithm_run_id and update learned project-card weights.",
    promptSnippet:
      "Use after a project-card-guided action is verified, so future bootstrap/sequence planning learns from the result.",
    parameters: Type.Object({
      algorithm_run_id: Type.String({
        description: "Project-card algorithm_run_id returned by focusa_project_card.",
      }),
      actual_outcome: Type.String({ description: "Observed final outcome/result for that algorithm run." }),
      score: Type.Optional(
        Type.Number({ description: "Optional outcome score from 0.0 to 1.0; defaults to 1.0." })
      ),
      evidence_refs: Type.Optional(
        Type.Array(Type.String(), { description: "Evidence refs proving the outcome." })
      ),
      project_root: Type.Optional(
        Type.String({ description: "Optional project root associated with the run." })
      ),
      notes: Type.Optional(Type.String({ description: "Optional bounded note about the result." })),
      task_timing: Type.Optional(
        Type.Any({
          description: "Optional override timing object; Pi auto-populates elapsed task timing when omitted.",
        })
      ),
      token_usage: Type.Optional(
        Type.Any({
          description:
            "Optional override token usage object; Pi auto-populates provider/estimated token counts when omitted.",
        })
      ),
    }),
    async execute(_id, params) {
      const p = params as {
        algorithm_run_id: string;
        actual_outcome: string;
        score?: number;
        evidence_refs?: string[];
        project_root?: string;
        notes?: string;
        task_timing?: any;
        token_usage?: any;
      };
      const autoAccounting = currentTaskTimingAndTokens();
      const payload = {
        algorithm_run_id: p.algorithm_run_id,
        actual_outcome: p.actual_outcome,
        score: typeof p.score === "number" ? p.score : undefined,
        evidence_refs: Array.isArray(p.evidence_refs) ? p.evidence_refs : [],
        project_root: p.project_root || getSessionCwd() || process.cwd(),
        notes: p.notes,
        task_timing: p.task_timing || autoAccounting.task_timing,
        token_usage: p.token_usage || autoAccounting.token_usage,
      };
      const result = await focusaFetchDetailed("/project/card/outcome", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      const body = result.body || {};
      const outcome = body.outcome || {};
      const text =
        result.ok && String(body.status || "") === "recorded"
          ? `project card outcome → recorded run=${String(outcome.algorithm_run_id || p.algorithm_run_id)} score=${String(outcome.score ?? payload.score ?? "default")} elapsed=${String(outcome.task_timing?.elapsed_hms || payload.task_timing.elapsed_hms)} tokens=${String(outcome.token_usage?.total_tokens ?? payload.token_usage.total_tokens)} evidence=${Array.isArray(outcome.evidence_refs) ? outcome.evidence_refs.length : payload.evidence_refs.length}`
          : `project card outcome blocked → ${explainWorkLoopResult(result, "outcome unavailable")}`;
      const toolResult = body.details?.tool_result_v1 || {
        ok: result.ok && body.status === "recorded",
        status: result.ok ? String(body.status || "completed") : "blocked",
        canonical: false,
        degraded: !result.ok,
        failure_class: body.failure_class || null,
        retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" },
        side_effects: ["project_card_algorithm_outcome_append", "project_card_weight_update"],
        evidence_refs: payload.evidence_refs,
        next_tools: body.flywheel?.next_tools || [
          "focusa_project_card",
          "focusa_predict_record",
          "focusa_metacog_capture",
        ],
      };
      return {
        content: [{ type: "text", text }],
        details: {
          ok: toolResult.ok,
          status: String(body.status || (result.ok ? "completed" : "blocked")),
          endpoint: "/v1/project/card/outcome",
          advisory_only: false,
          outcome,
          storage: body.storage || null,
          flywheel: body.flywheel || null,
          tool_result_v1: toolResult,
          failure_class: toolResult.failure_class || null,
          side_effects: toolResult.side_effects || [],
          evidence_refs: toolResult.evidence_refs || [],
          request: compactApiEcho(payload),
          response: compactApiEcho(body),
          next_tools: toolResult.next_tools ||
            body.flywheel?.next_tools || [
              "focusa_project_card",
              "focusa_predict_record",
              "focusa_metacog_capture",
            ],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_session_transfer",
    label: "Focusa Session Transfer",
    description:
      "Typed save/continue/rollover wrapper for moving long work between Pi sessions without forking or continuity-id fingerprint fallback.",
    promptSnippet:
      "Use when operator wants to save, continue, or roll over a long Focusa/Pi session with explicit source/target scope.",
    parameters: Type.Object({
      action: Type.String({ description: "save|continue|status|rollover" }),
      rollover_action: Type.Optional(
        Type.Union(
          [
            Type.Literal("none"),
            Type.Literal("inspect"),
            Type.Literal("checkpoint"),
            Type.Literal("migrate"),
            Type.Literal("resume"),
            Type.Literal("commit"),
            Type.Literal("rollback"),
          ],
          { description: "Spec130 rollover action; required for rotating continuity workflows." }
        )
      ),
      source_scope: Type.Optional(
        Type.Object({
          scope_kind: Type.Optional(Type.Union([Type.Literal("project"), Type.Literal("host")])),
          scope_id: Type.Optional(Type.String({ description: "Typed scope id from Focusa scope envelope." })),
          root_path: Type.Optional(Type.String({ description: "Verified scope root path." })),
          project_root: Type.Optional(
            Type.String({ description: "Backward-compatible project root field." })
          ),
          canonical_name: Type.Optional(Type.String({ description: "Canonical scope display name." })),
          fingerprint: Type.Optional(
            Type.String({ description: "Scope fingerprint; never used as continuity id." })
          ),
          continuity_id: Type.Optional(
            Type.String({ description: "Workstream continuity id under this scope." })
          ),
        })
      ),
      target_scope: Type.Optional(
        Type.Object({
          scope_kind: Type.Optional(Type.Union([Type.Literal("project"), Type.Literal("host")])),
          scope_id: Type.Optional(Type.String({ description: "Typed scope id from Focusa scope envelope." })),
          root_path: Type.Optional(Type.String({ description: "Verified scope root path." })),
          project_root: Type.Optional(
            Type.String({ description: "Backward-compatible project root field." })
          ),
          canonical_name: Type.Optional(Type.String({ description: "Canonical scope display name." })),
          fingerprint: Type.Optional(
            Type.String({ description: "Scope fingerprint; never used as continuity id." })
          ),
          continuity_id: Type.Optional(
            Type.String({ description: "Workstream continuity id under this scope." })
          ),
        })
      ),
      source_working_subpath_id: Type.Optional(
        Type.String({ description: "Source WorkingSubpath id; defaults to active context or primary." })
      ),
      target_working_subpath_id: Type.Optional(
        Type.String({
          description: "Explicit target WorkingSubpath id for auditable cross-worktree transfer.",
        })
      ),
      target_continuity_id: Type.Optional(
        Type.String({
          description:
            "Explicit target continuity id when target_scope is same root with rotated continuity.",
        })
      ),
      source_session_id: Type.Optional(Type.String({ description: "Source/native Pi session id." })),
      target_session_id: Type.Optional(
        Type.String({ description: "Target/native Pi session id after rollover/transfer." })
      ),
      checkpoint_ref: Type.Optional(
        Type.String({ description: "Pre-created checkpoint ref to bind transfer." })
      ),
      workpoint_packet_ref: Type.Optional(
        Type.String({ description: "Workpoint/resume packet ref to bind transfer." })
      ),
      compaction_packet_ref: Type.Optional(
        Type.String({ description: "Spec130 compaction mission packet ref." })
      ),
      project_root: Type.Optional(
        Type.String({ description: "Deprecated convenience source root; prefer source_scope.root_path." })
      ),
      current_ask: Type.Optional(Type.String({ description: "Current resume/save intent." })),
      mission: Type.Optional(
        Type.String({
          description: "Optional save mission; defaults to current ask or inferred Workpoint mission.",
        })
      ),
      next_action: Type.Optional(Type.String({ description: "Optional exact next action for save." })),
      continuity_id: Type.Optional(
        Type.String({ description: "Deprecated source continuity id; prefer source_scope.continuity_id." })
      ),
      write_preload: Type.Optional(
        Type.Boolean({
          description: "Request preload write guidance; defaults false and never writes implicitly.",
        })
      ),
      preload_target: Type.Optional(
        Type.Union(
          [
            Type.Literal("cursor"),
            Type.Literal("claude"),
            Type.Literal("codex"),
            Type.Literal("pi"),
            Type.Literal("opencode"),
            Type.Literal("generic"),
          ],
          { description: "Target agent surface; defaults cursor." }
        )
      ),
      preload_mode: Type.Optional(
        Type.Union(
          [
            Type.Literal("session_start"),
            Type.Literal("post_compaction"),
            Type.Literal("session_transfer"),
            Type.Literal("recovery"),
            Type.Literal("tool_guidance"),
          ],
          { description: "Preload mode; defaults session_transfer." }
        )
      ),
      receipt_preview: Type.Optional(
        Type.Boolean({ description: "Return a bounded receipt preview; defaults true." })
      ),
      receipt_commit: Type.Optional(
        Type.Boolean({ description: "Explicitly commit the preload receipt; defaults false." })
      ),
    }),
    async execute(_id, params) {
      const p = params as {
        action: string;
        rollover_action?: string;
        source_scope?: Record<string, any>;
        target_scope?: Record<string, any>;
        source_working_subpath_id?: string;
        target_working_subpath_id?: string;
        target_continuity_id?: string;
        source_session_id?: string;
        target_session_id?: string;
        checkpoint_ref?: string;
        workpoint_packet_ref?: string;
        compaction_packet_ref?: string;
        project_root?: string;
        current_ask?: string;
        mission?: string;
        next_action?: string;
        continuity_id?: string;
        write_preload?: boolean;
        preload_target?: string;
        preload_mode?: string;
        receipt_preview?: boolean;
        receipt_commit?: boolean;
      };
      const action = String(p.action || "status").toLowerCase();
      const sourceRootHint = String(
        p.source_scope?.root_path || p.source_scope?.project_root || p.project_root || getSessionCwd() || ""
      );
      const projectRoot = await resolveFocusaToolProjectRoot(sourceRootHint);
      const sourceContinuityId = String(
        p.source_scope?.continuity_id || p.continuity_id || getContinuityId() || ""
      ).trim();
      if (!sourceContinuityId) {
        return {
          content: [
            {
              type: "text",
              text: "session transfer blocked: explicit source continuity_id is required; no fingerprint-derived fallback is allowed",
            },
          ],
          details: {
            ok: false,
            status: "blocked",
            failure_class: "missing_source_continuity_id",
            next_tools: ["focusa_workpoint_resume", "focusa_project_card"],
          },
        } as any;
      }
      const targetRootHint = String(
        p.target_scope?.root_path || p.target_scope?.project_root || sourceRootHint
      );
      const targetProjectRoot = await resolveFocusaToolProjectRoot(targetRootHint);
      const targetContinuityId = String(
        p.target_scope?.continuity_id || p.target_continuity_id || sourceContinuityId
      ).trim();
      const sourceWorkingSubpathId = String(
        p.source_working_subpath_id || process.env.FOCUSA_WORKING_SUBPATH_ID || "primary"
      ).trim();
      const targetWorkingSubpathId = String(p.target_working_subpath_id || sourceWorkingSubpathId).trim();
      const sourceScope = buildProjectWorkstreamKey(
        projectRoot,
        sourceContinuityId,
        p.source_scope?.canonical_name
      );
      const targetScope = buildProjectWorkstreamKey(
        targetProjectRoot,
        targetContinuityId,
        p.target_scope?.canonical_name || p.source_scope?.canonical_name
      );
      const sourceSessionId = String(
        p.source_session_id || getSessionFrameKey() || getAttachmentRuntime().sessionFrameKey || ""
      );
      const targetSessionId = String(p.target_session_id || sourceSessionId || "");
      const rolloverAction = String(p.rollover_action || (action === "rollover" ? "inspect" : "none"));
      const currentAsk =
        p.current_ask ||
        getAttachmentRuntime().currentAsk?.text ||
        (action === "continue" || action === "rollover"
          ? "Continue latest saved Focusa work from typed session transfer scope"
          : "Save current Focusa work for typed session transfer");
      const cardQuery = scopedQueryParams(sourceScope);
      cardQuery.set("project_root", projectRoot);
      cardQuery.set("cwd", projectRoot);
      cardQuery.set("current_ask", currentAsk);
      const transferPayload = {
        action,
        rollover_action: rolloverAction,
        source_scope: sourceScope,
        target_scope: targetScope,
        source_working_subpath_id: sourceWorkingSubpathId,
        target_working_subpath_id: targetWorkingSubpathId,
        target_continuity_id: targetContinuityId,
        source_session_id: sourceSessionId,
        target_session_id: targetSessionId,
        checkpoint_ref: p.checkpoint_ref || null,
        workpoint_packet_ref: p.workpoint_packet_ref || null,
        compaction_packet_ref: p.compaction_packet_ref || null,
        project_root: projectRoot,
        current_ask: currentAsk,
        continuity_id: sourceContinuityId,
        mission: p.mission,
        next_action: p.next_action,
        write_preload: p.write_preload ?? false,
        preload_target: p.preload_target || "cursor",
        preload_mode: p.preload_mode || "session_transfer",
        receipt_preview: p.receipt_preview ?? true,
        receipt_commit: p.receipt_commit ?? false,
      };
      const apiTransfer = await focusaFetchDetailed("/project/session-transfer", {
        method: "POST",
        body: JSON.stringify(transferPayload),
      });
      const apiBody = apiTransfer.body || {};
      const cardRes = await focusaFetchDetailed(`/project/card?${cardQuery.toString()}`, { method: "GET" });
      const card = cardRes.body || {};
      const inferred =
        apiBody.transfer?.inferred_workpoint_candidate ||
        card.inferred_workpoint_candidate ||
        card.bootstrap?.candidate?.inferred_workpoint_candidate ||
        {};
      let checkpoint: any = null;
      let targetCheckpoint: any = null;
      let resume: any = null;
      let trajectory: any = null;
      let transitionVerification: any = null;
      if (action === "save" || rolloverAction === "checkpoint") {
        const hint = inferred.checkpoint_payload_hint || {};
        const mission = p.mission || hint.mission || inferred.mission || currentAsk;
        const nextAction =
          p.next_action ||
          hint.next_action ||
          inferred.next_action ||
          "Continue from saved Focusa session transfer packet";
        checkpoint = await focusaFetchDetailed("/workpoint/checkpoint", {
          method: "POST",
          body: JSON.stringify({
            scope: sourceScope,
            mission,
            next_action: nextAction,
            next_slice: nextAction,
            current_action: hint.current_action || inferred.current_action || "session_transfer_save",
            action_type: hint.current_action || inferred.current_action || "session_transfer_save",
            rollover_action: rolloverAction,
            target_objects: hint.target_objects || inferred.target_objects || [],
            active_object_refs: hint.target_objects || inferred.target_objects || [],
            project_root: projectRoot,
            continuity_id: sourceContinuityId,
            working_subpath_id: sourceWorkingSubpathId,
            target_continuity_id: targetContinuityId,
            session_identity: await buildFocusaSessionIdentity(projectRoot, "session_switch", {
              continuityId: sourceContinuityId,
              sessionId: sourceSessionId,
            }),
            source_session_id: sourceSessionId,
            target_session_id: targetSessionId,
            session_id: sourceSessionId,
            source_turn_id: `pi-turn-${getTurnCount()}`,
            canonical: true,
            checkpoint_reason:
              rolloverAction === "checkpoint"
                ? "session_transfer_rollover_checkpoint"
                : "session_transfer_save",
            checkpoint_ref: p.checkpoint_ref || undefined,
            workpoint_packet_ref: p.workpoint_packet_ref || undefined,
            compaction_packet_ref: p.compaction_packet_ref || undefined,
            idempotency_key: `session-transfer:${projectRoot}:${sourceContinuityId}:${targetContinuityId}:${sourceSessionId}:${targetSessionId}:${Date.now()}`,
          }),
        });
      }
      if (
        action === "rollover" &&
        apiTransfer.ok &&
        targetContinuityId !== sourceContinuityId &&
        targetWorkingSubpathId === sourceWorkingSubpathId
      ) {
        const hint = inferred.checkpoint_payload_hint || {};
        const targetMission = p.mission || hint.mission || inferred.mission || currentAsk;
        const targetNextAction =
          p.next_action ||
          hint.next_action ||
          inferred.next_action ||
          "Resume transferred Focusa mission under the target continuity";
        // The first checkpoint in a target continuity cannot depend on the source partition's writer lease.
        targetCheckpoint = await focusaFetchDetailed("/workpoint/checkpoint", {
          method: "POST",
          body: JSON.stringify({
            scope: targetScope,
            mission: targetMission,
            next_action: targetNextAction,
            next_slice: targetNextAction,
            current_action: "session_transfer_target_materialization",
            action_type: "session_transfer_target_materialization",
            target_objects: hint.target_objects || inferred.target_objects || [],
            active_object_refs: hint.target_objects || inferred.target_objects || [],
            project_root: targetProjectRoot,
            continuity_id: targetContinuityId,
            session_id: targetSessionId,
            source_continuity_id: sourceContinuityId,
            target_continuity_id: targetContinuityId,
            source_session_id: sourceSessionId,
            target_session_id: targetSessionId,
            checkpoint_reason: "session_resume",
            canonical: true,
            promote: true,
            working_subpath_id: targetWorkingSubpathId,
            session_identity: await buildFocusaSessionIdentity(targetRootHint, "session_switch", {
              continuityId: targetContinuityId,
              sessionId: targetSessionId,
            }),
            checkpoint_ref: p.checkpoint_ref || undefined,
            workpoint_packet_ref: p.workpoint_packet_ref || undefined,
            compaction_packet_ref: p.compaction_packet_ref || undefined,
            idempotency_key: `session-transfer-target:${targetProjectRoot}:${targetContinuityId}:${targetSessionId}`,
          }),
        });
      }
      if (["continue", "status", "save", "rollover"].includes(action)) {
        resume = await focusaFetchDetailed("/workpoint/resume", {
          method: "POST",
          body: JSON.stringify({
            scope: targetScope,
            source_scope: sourceScope,
            project_root: targetProjectRoot,
            continuity_id: targetContinuityId,
            source_continuity_id: sourceContinuityId,
            target_continuity_id: targetContinuityId,
            session_id: targetSessionId,
            source_session_id: sourceSessionId,
            target_session_id: targetSessionId,
            checkpoint_ref: p.checkpoint_ref || undefined,
            workpoint_packet_ref: p.workpoint_packet_ref || undefined,
            compaction_packet_ref: p.compaction_packet_ref || undefined,
            rollover_action: rolloverAction,
            mode: "compact_prompt",
            session_identity: await buildFocusaSessionIdentity(targetRootHint, "session_switch", {
              continuityId: targetContinuityId,
              sessionId: targetSessionId,
            }),
          }),
        });
        const tq = scopedQueryParams(targetScope);
        tq.set("project_root", targetProjectRoot);
        tq.set("continuity_id", targetContinuityId);
        tq.set("allow_prior_project_trajectory", "true");
        trajectory = await focusaFetchDetailed(`/trajectory/view?${tq.toString()}`, { method: "GET" });
      }
      if (action === "rollover" && targetCheckpoint?.ok && resume?.ok && resume.body?.canonical === true) {
        const targetWorkpointId =
          resume.body?.workpoint_id ||
          resume.body?.resume_packet?.workpoint?.workpoint_id ||
          targetCheckpoint.body?.workpoint_id ||
          targetCheckpoint.body?.resume_packet?.workpoint?.workpoint_id;
        transitionVerification = await focusaFetchDetailed("/project/session-transfer", {
          method: "POST",
          body: JSON.stringify({
            ...transferPayload,
            action: "verify_target",
            target_workpoint_id: targetWorkpointId,
            target_resume_canonical: true,
            target_resume_packet_ref: targetWorkpointId,
          }),
        });
      }
      const rolloverVerified =
        action !== "rollover" ||
        targetContinuityId === sourceContinuityId ||
        (targetCheckpoint?.ok &&
          resume?.ok &&
          resume.body?.canonical === true &&
          transitionVerification?.ok &&
          transitionVerification.body?.transfer?.transition_receipt?.status === "target_resume_verified");
      const ok =
        apiTransfer.ok &&
        cardRes.ok &&
        (action !== "save" || checkpoint?.ok) &&
        rolloverVerified &&
        (action === "save" ||
          resume?.ok ||
          card.inferred_workpoint_candidate ||
          apiBody.transfer?.inferred_workpoint_candidate);
      const shortest = card.success_sequence?.shortest_path_to_success?.selected || {};
      const text = `session transfer ${action} → source=${projectRoot}/${sourceContinuityId} target=${targetProjectRoot}/${targetContinuityId} rollover=${rolloverAction} saved=${checkpoint?.ok === true} resume=${String(resume?.body?.status || resume?.status || "not_run")} inferred_wp=${String(inferred.current_action || "none")} shortest=${String(shortest.path_id || "unknown")}`;
      const toolResult = card.details?.tool_result_v1 || {
        ok,
        status: ok ? "completed" : "blocked",
        canonical: resume?.body?.canonical === true || checkpoint?.body?.canonical === true,
        degraded: !ok,
        failure_class: ok
          ? null
          : card.failure_class || resume?.body?.failure_class || checkpoint?.body?.failure_class || null,
        retry: { safe: true, posture: "safe_retry" },
        side_effects: checkpoint?.ok ? ["workpoint_checkpoint"] : [],
        evidence_refs: [p.checkpoint_ref, p.workpoint_packet_ref, p.compaction_packet_ref].filter(Boolean),
        next_tools: ["focusa_project_card", "focusa_workpoint_resume", "focusa_trajectory_view"],
      };
      return {
        content: [{ type: "text", text }],
        details: {
          ok,
          status: ok ? "completed" : "blocked",
          endpoint: "session_transfer_wrapper",
          action,
          rollover_action: rolloverAction,
          source_scope: sourceScope,
          target_scope: targetScope,
          source_session_id: sourceSessionId,
          target_session_id: targetSessionId,
          checkpoint_ref: p.checkpoint_ref || null,
          workpoint_packet_ref: p.workpoint_packet_ref || null,
          compaction_packet_ref: p.compaction_packet_ref || null,
          project_root: projectRoot,
          continuity_id: sourceContinuityId,
          target_project_root: targetProjectRoot,
          target_continuity_id: targetContinuityId,
          api_transfer: apiBody,
          session_transfer_save_packet: apiBody.transfer || null,
          workpoint_checkpoint_packet: checkpoint?.body || null,
          target_workpoint_checkpoint_packet: targetCheckpoint?.body || null,
          workpoint_resume_packet: resume?.body || null,
          transition_verification: transitionVerification?.body || null,
          trajectory: trajectory?.body || null,
          project_card: {
            algorithm_run_id: card.algorithm_run_id,
            inferred_workpoint_candidate: inferred,
            trajectory_report_card: card.trajectory_report_card,
            crosswire_health: card.crosswire_health,
            success_sequence: card.success_sequence,
          },
          operator_handoff: apiBody.transfer?.operator_handoff || {
            command: `cd ${targetProjectRoot} && pi`,
            first_tool: `focusa_session_transfer action="continue" source_scope='${JSON.stringify(sourceScope)}' target_continuity_id="${targetContinuityId}" target_session_id="${targetSessionId}"`,
            preload: `focusa preload write --target ${p.preload_target || "cursor"} --project-root ${targetProjectRoot} --continuity-id ${targetContinuityId}`,
            receipt_preview: `focusa preload receipt-preview --target ${p.preload_target || "cursor"} --project-root ${targetProjectRoot} --continuity-id ${targetContinuityId}`,
            authority_boundary: "typed_source_scope_plus_typed_target_scope",
          },
          transfer_payload: compactApiEcho(transferPayload),
          tool_result_v1: toolResult,
          next_tools: ["focusa_workpoint_resume", "focusa_project_card", "focusa_trajectory_view"],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_project_verify",
    label: "Focusa Project Verify",
    description:
      "Verify active project folder against expected ProjectIdentity fields and report mismatches without mutating state.",
    promptSnippet:
      "Use when project folder or session identity is ambiguous before accepting a Workpoint/Trajectory packet as canonical.",
    parameters: Type.Object({
      cwd: Type.Optional(
        Type.String({ description: "Optional cwd/project path hint; defaults to Pi session cwd." })
      ),
      project_root: Type.Optional(Type.String({ description: "Expected project root." })),
      persisted_project_root: Type.Optional(
        Type.String({
          description: "Resumed-session project root to compare as an advisory binding candidate.",
        })
      ),
      project_id: Type.Optional(Type.String({ description: "Expected project id from marker/operator." })),
      canonical_name: Type.Optional(Type.String({ description: "Expected canonical project name." })),
      repo_remote: Type.Optional(Type.String({ description: "Expected git origin remote." })),
      remote_host: Type.Optional(
        Type.String({
          description: "Remote SSH host that contains the project root; caller supplies inspected evidence.",
        })
      ),
      remote_user: Type.Optional(Type.String({ description: "Remote SSH user, if known." })),
      remote_port: Type.Optional(
        Type.Integer({ minimum: 1, maximum: 65535, description: "Remote SSH port, if known." })
      ),
      remote_repo_remote: Type.Optional(
        Type.String({ description: "Git origin/repo remote observed on the remote host." })
      ),
      remote_workspace_kind: Type.Optional(
        Type.String({ description: "Workspace kind observed on the remote host." })
      ),
      remote_deploy_root: Type.Optional(
        Type.String({ description: "Deployment/site root observed on the remote host." })
      ),
    }),
    async execute(_id, params) {
      const p = params as {
        cwd?: string;
        project_root?: string;
        persisted_project_root?: string;
        project_id?: string;
        canonical_name?: string;
        repo_remote?: string;
        remote_host?: string;
        remote_user?: string;
        remote_port?: number;
        remote_repo_remote?: string;
        remote_workspace_kind?: string;
        remote_deploy_root?: string;
      };
      const payload = {
        ...p,
        cwd: p.cwd || getSessionCwd() || process.cwd(),
        persisted_project_root:
          p.persisted_project_root ||
          getActiveWorkpointPacket()?.scope?.project_root ||
          getActiveWorkpointPacket()?.project_root ||
          getLastProjectRootResolution()?.projectRoot ||
          getLastProjectIdentity()?.project_root,
      };
      const result = await focusaFetchDetailed("/project/verify", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      const body = result.body || {};
      if (!result.ok && body.failure_class === "hot_path_timeout") {
        const requestedRoot = normalizeProjectRoot(
          p.project_root || p.cwd || getSessionCwd() || process.cwd()
        );
        const cachedIdentity =
          getLastProjectIdentity() &&
          (!requestedRoot || normalizeProjectRoot(getLastProjectIdentity()!.project_root) === requestedRoot)
            ? getLastProjectIdentity()!
            : null;
        return {
          content: [
            {
              type: "text",
              text: timeoutPreservedText(
                "project verify",
                cachedIdentity ? "cached identity" : "empty fallback"
              ),
            },
          ],
          details: {
            ok: false,
            status: "timeout_preserved",
            endpoint: "/v1/project/verify",
            canonical: false,
            degraded: true,
            advisory_only: true,
            project_identity: cachedIdentity || {},
            verification: { verified: false, reason: "hot_path_timeout" },
            failure_class: "hot_path_timeout",
            response: compactApiEcho(body),
            next_tools: [
              "focusa_tool_doctor",
              "focusa_resource_mode",
              "focusa_project_identity",
              "focusa_project_verify",
              "focusa_trajectory_view",
            ],
          },
        } as any;
      }
      const identity = body.project_identity || {};
      const verified = body.verification?.verified === true;
      const verifiedRoot = normalizeProjectRoot(identity.project_root || p.project_root || p.cwd);
      const bindingCandidates = Array.isArray(body.binding_candidates) ? body.binding_candidates : [];
      const selectedCandidate = bindingCandidates.find(
        (candidate: any) => normalizeProjectRoot(candidate?.project_root) === verifiedRoot
      );
      const previousBinding = currentProjectBindingDecision();
      const bindingDecisionV1 = reconcileProjectBindingDecision({
        selectedProjectRoot: verifiedRoot || undefined,
        selectedWorktreeRoot: selectedCandidate?.active_worktree_root,
        canonicalParentRoot: selectedCandidate?.canonical_parent_root,
        continuityId: verifiedRoot ? ensureContinuityId(verifiedRoot) : getContinuityId(),
        candidates: bindingCandidates,
        ambiguous: body.binding_decision?.ambiguous === true || body.status === "ambiguous_project_binding",
        selectedRootSafe: !!verifiedRoot && isProjectRootAuthoritySafe(verifiedRoot),
        verificationCanonical: verified,
        verificationStatus: String(identity.status || body.status || "unknown"),
        daemonAvailable: result.ok,
        evidenceFreshness: verified ? "current" : "unknown",
        repoFingerprint: selectedCandidate?.repo_fingerprint,
        projectFingerprint: selectedCandidate?.project_fingerprint,
        rejectionReasons: verified
          ? []
          : [String(body.verification?.required_recovery || "project_verify_not_canonical")],
        recoveryPacketRef: `project-scope-recovery:${getSessionFrameKey() || "no-session"}`,
        previousDecision: previousBinding,
      });
      setCurrentProjectBindingDecision(bindingDecisionV1);
      if (identity && Object.keys(identity).length)
        setLastProjectVerify({ ...body, binding_decision_v1: bindingDecisionV1 });
      if (bindingDecisionV1.state === "BOUND" && verifiedRoot) {
        confirmPiProjectRoot(verifiedRoot, "focusa_project_verify_verified");
        ensureContinuityId(verifiedRoot);
      }
      persistState();
      const text = result.ok
        ? `project verify → verified=${verified} status=${String(identity.status || body.status || "unknown")} confidence=${String(identity.confidence || "unknown")} root=${String(identity.project_root || "unknown")}`
        : `project verify blocked → ${explainWorkLoopResult(result, "project verify unavailable")}`;
      const toolResult = body.details?.tool_result_v1 || {
        ok: result.ok && body.status !== "blocked",
        status: result.ok ? String(body.status || "completed") : "blocked",
        canonical: body.canonical === true,
        degraded: body.degraded !== false,
        failure_class: body.failure_class || null,
        retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" },
        side_effects: [],
        evidence_refs: [],
        next_tools: body.next_tools || [
          "focusa_project_identity",
          "focusa_trajectory_view",
          "focusa_workpoint_resume",
        ],
      };
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
          binding_decision_v1: bindingDecisionV1,
          tool_result_v1: toolResult,
          failure_class: toolResult.failure_class || body.failure_class || null,
          next_tools: toolResult.next_tools ||
            body.next_tools || [
              "focusa_project_identity",
              "focusa_trajectory_view",
              "focusa_workpoint_resume",
            ],
          response: compactApiEcho(body),
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_project_bootstrap",
    label: "Focusa Project Bootstrap",
    description:
      "Preview, apply, inspect, or repair the idempotent local project-discipline baseline before Project Genesis.",
    promptSnippet:
      "Use for new/existing project onboarding; preview first, require confirm=true for apply/rollback, and never infer a remote, stack, deployment, domain, or secret.",
    parameters: Type.Object({
      action: Type.Optional(
        Type.Union(
          [Type.Literal("preview"), Type.Literal("apply"), Type.Literal("status"), Type.Literal("repair")],
          { description: "Bootstrap operation; defaults to status." }
        )
      ),
      project_root: Type.String({ description: "Explicit safe absolute project root." }),
      project_id: Type.Optional(Type.String()),
      canonical_name: Type.Optional(Type.String()),
      continuity_id: Type.Optional(Type.String()),
      idempotency_key: Type.Optional(Type.String()),
      discipline_profile: Type.Optional(
        Type.String({ description: "Defaults to standard_software_project." })
      ),
      initialize_git: Type.Optional(Type.Boolean()),
      initialize_task_provider: Type.Optional(Type.Boolean()),
      task_provider: Type.Optional(Type.String()),
      hlt: Type.Optional(Type.String()),
      hlt_confirmed: Type.Optional(Type.Boolean()),
      desired_end_state: Type.Optional(Type.String()),
      current_state: Type.Optional(Type.String()),
      specification_ref: Type.Optional(Type.String()),
      acceptance_criteria: Type.Optional(Type.Array(Type.String())),
      confirm: Type.Optional(Type.Boolean()),
      repair_action: Type.Optional(
        Type.String({ description: "retry or rollback; rollback requires confirm=true." })
      ),
    }),
    async execute(_toolCallId: string, params: any) {
      const action = String(params.action || "status");
      const projectRoot = normalizeProjectRoot(params.project_root);
      if (!projectRoot || !isProjectRootAuthoritySafe(projectRoot)) {
        return {
          content: [
            { type: "text", text: "project bootstrap → blocked: supply an explicit safe project root" },
          ],
          details: {
            status: "blocked",
            failure_class: "unsafe_project_root",
            next_tools: ["focusa_project_verify"],
          },
        } as any;
      }
      const continuityId = params.continuity_id || getContinuityId() || ensureContinuityId(projectRoot);
      let result: any;
      if (action === "status") {
        result = await focusaFetchDetailed(
          `/project/bootstrap/status?project_root=${encodeURIComponent(projectRoot)}`
        );
      } else {
        result = await focusaFetchDetailed(`/project/bootstrap/${encodeURIComponent(action)}`, {
          method: "POST",
          body: JSON.stringify({
            ...params,
            action: undefined,
            project_root: projectRoot,
            project_id: params.project_id || projectRoot.split("/").filter(Boolean).pop() || "project",
            canonical_name:
              params.canonical_name || projectRoot.split("/").filter(Boolean).pop() || "Project",
            continuity_id: continuityId,
            idempotency_key: params.idempotency_key || `bootstrap:${continuityId}`,
          }),
        });
      }
      const body = result.body || {};
      const status = String(body.status || (result.ok ? "completed" : "blocked"));
      return {
        content: [
          {
            type: "text",
            text: `project bootstrap ${action} → ${status}\nnext: ${body.next_action || "inspect readiness"}`,
          },
        ],
        details: {
          ok: result.ok,
          status,
          canonical: status === "ready",
          project_root: projectRoot,
          continuity_id: continuityId,
          bootstrap_packet: compactApiEcho(body),
          next_tools:
            status === "ready"
              ? ["focusa_project_genesis", "focusa_workpoint_resume"]
              : ["focusa_project_bootstrap", "focusa_project_verify"],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_project_genesis",
    label: "Focusa Project Genesis",
    description:
      "Start, resume, inspect, or atomically commit the Project Genesis chain from verified identity and HLT through the first Workpoint.",
    promptSnippet:
      "Use after project verification when onboarding/readiness is incomplete; HLT Impasse asks at most one concise intent question, and commit requires confirm=true.",
    parameters: Type.Object({
      action: Type.Optional(
        Type.Union(
          [Type.Literal("start"), Type.Literal("resume"), Type.Literal("status"), Type.Literal("commit")],
          { description: "Genesis operation; defaults to status." }
        )
      ),
      project_root: Type.Optional(Type.String({ description: "Verified absolute project root." })),
      continuity_id: Type.Optional(Type.String({ description: "Stable project workstream continuity id." })),
      idempotency_key: Type.Optional(Type.String({ description: "Stable transaction replay key." })),
      hlt: Type.Optional(Type.String({ description: "Operator-confirmed High Level Trajectory." })),
      hlt_confirmed: Type.Optional(Type.Boolean()),
      desired_end_state: Type.Optional(Type.String()),
      current_state: Type.Optional(Type.String()),
      specification_ref: Type.Optional(Type.String()),
      acceptance_criteria: Type.Optional(Type.Array(Type.String())),
      mid_level_goal: Type.Optional(Type.String()),
      short_term_goal: Type.Optional(Type.String()),
      waypoints: Type.Optional(Type.Array(Type.String())),
      task_provider: Type.Optional(Type.String()),
      allow_task_decomposition: Type.Optional(Type.Boolean()),
      confirm: Type.Optional(Type.Boolean({ description: "Required true for commit or takeover." })),
      takeover: Type.Optional(
        Type.Boolean({
          description: "Take over a conflicting active project workstream; requires confirm=true.",
        })
      ),
    }),
    async execute(_toolCallId: string, params: any) {
      const action = String(params.action || "status");
      const projectRoot = normalizeProjectRoot(
        params.project_root || getLastProjectIdentity()?.project_root || getSessionCwd()
      );
      if (!projectRoot || !isProjectRootAuthoritySafe(projectRoot)) {
        return {
          content: [{ type: "text", text: "project genesis → blocked: verify a safe project root first" }],
          details: {
            status: "blocked",
            failure_class: "project_identity_required",
            next_tools: ["focusa_project_verify", "focusa_project_identity"],
          },
        } as any;
      }
      const continuityId = params.continuity_id || getContinuityId() || ensureContinuityId(projectRoot);
      let result: any;
      if (action === "status") {
        result = await focusaFetchDetailed(
          `/project/genesis/status?project_root=${encodeURIComponent(projectRoot)}`
        );
      } else {
        result = await focusaFetchDetailed(`/project/genesis/${encodeURIComponent(action)}`, {
          method: "POST",
          body: JSON.stringify({
            ...params,
            action: undefined,
            project_root: projectRoot,
            continuity_id: continuityId,
            idempotency_key:
              params.idempotency_key || `genesis:${continuityId}:${params.specification_ref || "project"}`,
          }),
        });
      }
      const body = result.body || {};
      const status = String(body.status || (result.ok ? "completed" : "blocked"));
      const nextAction = String(body.next_action || "inspect the Genesis packet and repair missing links");
      return {
        content: [
          {
            type: "text",
            text: `project genesis ${action} → ${status}\nnext: ${nextAction}`,
          },
        ],
        details: {
          ok: result.ok,
          status,
          canonical: status === "ready",
          project_root: projectRoot,
          continuity_id: continuityId,
          genesis_packet: compactApiEcho(body),
          next_tools:
            status === "ready"
              ? ["focusa_workpoint_resume", "focusa_trajectory_view"]
              : ["focusa_project_genesis", "focusa_project_verify", "focusa_trajectory_view"],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_reflex_primitives",
    label: "Reflex Primitives",
    description:
      "List bounded Spec97 Reflex Primitive summaries by family/query; read-only routing metadata, never mutation authority.",
    parameters: Type.Object({
      family: Type.Optional(
        Type.String({ description: "Optional primitive family filter, e.g. recovery, evidence, resource." })
      ),
      query: Type.Optional(Type.String({ description: "Optional risk/object/action search text." })),
      limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 50, description: "Bounded result limit." })),
      include_payload: Type.Optional(
        Type.Boolean({ description: "Cold opt-in for full primitive payloads; default false." })
      ),
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
      if (!result.ok)
        return blockedToolResponse(
          "focusa_reflex_primitives",
          "reflex",
          `reflex primitives blocked → ${explainWorkLoopResult(result, "reflex registry unavailable")}`,
          body.failure_class || "daemon_unavailable",
          body,
          ["focusa_traverse", "focusa_tool_doctor"]
        );
      const items = Array.isArray(body.items) ? body.items : [];
      const families = Array.from(new Set(items.map((item: any) => String(item.family || "unknown"))))
        .slice(0, 6)
        .join(",");
      const toolResult =
        body.details?.tool_result_v1 ||
        focusaToolResult({
          ok: true,
          status: "completed",
          summary: `reflex primitives → returned=${items.length} families=${families || "none"}`,
          tool: "focusa_reflex_primitives",
          family: "reflex",
          side_effects: [],
          evidence_refs: [],
          next_tools: ["focusa_traverse", "focusa_tool_doctor"],
          raw: body,
        });
      return {
        content: [
          {
            type: "text",
            text: `reflex primitives → returned=${items.length} families=${families || "none"} truncated=${Boolean(body.bounds?.truncated)}`,
          },
        ],
        details: {
          ok: true,
          status: "completed",
          endpoint: "/v1/reflex/primitives",
          canonical: body.canonical === true,
          degraded: body.degraded === true,
          read_only: body.read_only === true,
          advisory_only: body.advisory_only === true,
          items,
          bounds: body.bounds || null,
          tool_result_v1: toolResult,
          next_tools: toolResult.next_tools || ["focusa_traverse"],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_temporal_authority",
    label: "Focusa Temporal Authority",
    description:
      "Read, commit, revise, observe, forecast, or preflight project-scoped temporal claims without fabricating deadlines or urgency.",
    promptSnippet:
      "Use status first; external commitments require explicit confirmation/evidence, forecasts remain non-canonical ranges, and scope mismatch fails closed.",
    parameters: Type.Object({
      action: Type.Optional(
        Type.Union(
          [
            Type.Literal("status"),
            Type.Literal("commit"),
            Type.Literal("revise"),
            Type.Literal("observe"),
            Type.Literal("forecast"),
            Type.Literal("preflight"),
            Type.Literal("migrate-signatures"),
            Type.Literal("high-consequence-preflight"),
            Type.Literal("capture-clock"),
            Type.Literal("resolve-civil-time"),
            Type.Literal("commit-priority"),
            Type.Literal("settle-closure"),
          ],
          { description: "Temporal operation; defaults to status." }
        )
      ),
      project_root: Type.Optional(Type.String()),
      continuity_id: Type.Optional(Type.String()),
      host_id: Type.Optional(Type.String()),
      operator_id: Type.Optional(Type.String()),
      workpoint_id: Type.Optional(Type.String()),
      item_id: Type.Optional(Type.String()),
      task_id: Type.Optional(Type.String()),
      idempotency_key: Type.Optional(Type.String()),
      confirm: Type.Optional(Type.Boolean()),
      as_of: Type.Optional(Type.String()),
      phase: Type.Optional(Type.String()),
      timezone: Type.Optional(Type.String()),
      tzdb_version: Type.Optional(Type.String()),
      forecast_authority: Type.Optional(
        Type.Object({
          claim_kind: Type.Literal("forecast"),
          target_state: Type.String(),
          scope_revision: Type.String(),
          expires_at: Type.String(),
          estimator_version: Type.String(),
          cohort: Type.String(),
          evidence_basis: Type.Array(Type.String()),
          comparable_sample_count: Type.Number(),
          all_attempt_sample_count: Type.Number(),
          censoring_method: Type.String(),
          correlation_method: Type.String(),
          calibration_profile: Type.String(),
          grounding_status: Type.Literal("grounded"),
          baseline_ref: Type.String(),
          drift_policy_ref: Type.String(),
        })
      ),
      forecast_evaluation: Type.Optional(Type.Any()),
      high_consequence_packet: Type.Optional(Type.Any()),
      civil_time_packet: Type.Optional(Type.Any()),
      temporal_priority_packet: Type.Optional(Type.Any()),
      closure_packet: Type.Optional(Type.Any()),
      duration_ms: Type.Optional(Type.Number()),
      outcome: Type.Optional(Type.String()),
      actual_ms: Type.Optional(Type.Number()),
      evidence_refs: Type.Optional(Type.Array(Type.String())),
      claim: Type.Optional(
        Type.Object({
          claim_id: Type.String(),
          revision: Type.Number(),
          scope: Type.Object({
            project_root: Type.String(),
            continuity_id: Type.String(),
            host_id: Type.Optional(Type.String()),
            operator_id: Type.Optional(Type.String()),
            workpoint_id: Type.Optional(Type.String()),
            item_id: Type.Optional(Type.String()),
            task_id: Type.Optional(Type.String()),
          }),
          kind: Type.String(),
          status: Type.String(),
          subject_ref: Type.String(),
          target_at: Type.Optional(Type.String()),
          duration_ms: Type.Optional(Type.Number()),
          timezone: Type.String(),
          source: Type.String(),
          source_ref: Type.Optional(Type.String()),
          operator_confirmed: Type.Boolean(),
          confidence: Type.String(),
          uncertainty: Type.Optional(
            Type.Object({
              earliest_at: Type.Optional(Type.String()),
              latest_at: Type.Optional(Type.String()),
              coverage_probability: Type.Optional(Type.Number()),
              reason: Type.Optional(Type.String()),
            })
          ),
          observed_at: Type.String(),
          effective_at: Type.String(),
          expires_at: Type.Optional(Type.String()),
          supersedes_revision: Type.Optional(Type.Number()),
          evidence_refs: Type.Array(Type.String()),
          reason_code: Type.Optional(Type.String()),
        })
      ),
    }),
    async execute(_toolCallId: string, params: any) {
      const action = String(params.action || "status");
      const projectRoot = normalizeProjectRoot(
        params.project_root || getLastProjectIdentity()?.project_root || getSessionCwd()
      );
      if (!projectRoot || !isProjectRootAuthoritySafe(projectRoot)) {
        return {
          content: [{ type: "text", text: "temporal authority → blocked: verify a safe project root" }],
          details: {
            status: "blocked",
            failure_class: "project_identity_required",
            next_tools: ["focusa_project_verify"],
          },
        } as any;
      }
      const continuityId = params.continuity_id || getContinuityId() || ensureContinuityId(projectRoot);
      if (action === "commit-priority" && !params.temporal_priority_packet) {
        return {
          content: [
            {
              type: "text",
              text: "temporal priority commit → blocked: calendar, priority frame, guard, ask and action packet required",
            },
          ],
          details: {
            status: "blocked",
            failure_class: "temporal_priority_packet_required",
            canonical: false,
          },
        } as any;
      }
      if (action === "settle-closure" && !params.closure_packet) {
        return {
          content: [
            {
              type: "text",
              text: "temporal closure settlement → blocked: evidence, receipt, outcome, and optional lost-time packet required",
            },
          ],
          details: {
            status: "blocked",
            failure_class: "closure_packet_required",
            canonical: false,
          },
        } as any;
      }
      if (action === "resolve-civil-time" && !params.civil_time_packet) {
        return {
          content: [
            {
              type: "text",
              text: "temporal civil-time resolution → blocked: complete versioned intent packet required",
            },
          ],
          details: { status: "blocked", failure_class: "civil_time_packet_required", canonical: false },
        } as any;
      }
      if (action === "capture-clock" && !params.timezone) {
        return {
          content: [{ type: "text", text: "temporal clock capture → blocked: explicit timezone required" }],
          details: { status: "blocked", failure_class: "timezone_required", canonical: false },
        } as any;
      }
      if (action === "high-consequence-preflight" && !params.high_consequence_packet) {
        return {
          content: [
            {
              type: "text",
              text: "temporal high-consequence preflight → blocked: complete control packet required",
            },
          ],
          details: { status: "blocked", failure_class: "high_consequence_packet_required", canonical: false },
        } as any;
      }
      if (action === "forecast" && !params.forecast_authority) {
        return {
          content: [
            {
              type: "text",
              text: "temporal forecast → blocked: complete forecast authority metadata is required",
            },
          ],
          details: { status: "blocked", failure_class: "forecast_authority_required", canonical: false },
        } as any;
      }
      if (params.forecast_authority) {
        params.authority = params.forecast_authority;
        params.forecast_authority = undefined;
      }
      if (params.forecast_evaluation) {
        params.evaluation = params.forecast_evaluation;
        params.forecast_evaluation = undefined;
      }
      let result: any;
      if (action === "status") {
        const query = new URLSearchParams({ project_root: projectRoot, continuity_id: continuityId });
        for (const key of ["host_id", "operator_id", "workpoint_id", "item_id", "task_id", "as_of"]) {
          if (params[key]) query.set(key, String(params[key]));
        }
        result = await focusaFetchDetailed(`/temporal/status?${query.toString()}`);
      } else {
        const actionPath =
          action === "high-consequence-preflight"
            ? "/temporal/high-consequence/preflight"
            : action === "capture-clock"
              ? "/temporal/clock/capture"
              : action === "resolve-civil-time"
                ? "/temporal/civil/resolve"
                : action === "commit-priority"
                  ? "/temporal/priority/commit"
                  : `/temporal/${encodeURIComponent(action)}`;
        const actionBody =
          action === "high-consequence-preflight"
            ? params.high_consequence_packet || {}
            : action === "resolve-civil-time"
              ? params.civil_time_packet || {}
              : action === "commit-priority"
                ? params.temporal_priority_packet || {}
                : action === "settle-closure"
                  ? params.closure_packet || {}
                  : params;
        result = await focusaFetchDetailed(actionPath, {
          method: "POST",
          body: JSON.stringify({
            ...actionBody,
            action: undefined,
            project_root: projectRoot,
            continuity_id: continuityId,
            idempotency_key: params.idempotency_key || `temporal:${action}:${continuityId}:${Date.now()}`,
          }),
        });
      }
      const body = result.body || {};
      const status = String(body.status || (result.ok ? "completed" : "blocked"));
      return {
        content: [
          {
            type: "text",
            text: `temporal ${action} → ${status}\nnext: ${body.next_action || "inspect temporal projection"}`,
          },
        ],
        details: {
          ok: result.ok,
          status,
          canonical: action === "commit" || action === "revise" ? body.canonical === true : false,
          project_root: projectRoot,
          continuity_id: continuityId,
          temporal_packet: compactApiEcho(body),
          next_tools: ["focusa_temporal_authority", "focusa_trajectory_view", "focusa_workpoint_resume"],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_trajectory_view",
    label: "Trajectory View",
    description:
      "Read the per-project Trajectory Intelligence view: project identity, goal/state/gap/evidence/drift, and next Workpoint candidate.",
    promptSnippet:
      "Use first on project start/resume or when goal/state/next action is unclear; Trajectory is advisory and per-project.",
    parameters: Type.Object({
      project_root: Type.Optional(
        Type.String({ description: "Optional expected project root; defaults to Pi session cwd." })
      ),
      session_id: Type.Optional(
        Type.String({ description: "Optional temporal Pi session id; defaults to Pi session key." })
      ),
      continuity_id: Type.Optional(
        Type.String({
          description:
            "Optional logical continuity id; defaults to Pi continuity id and is part of authority boundary.",
        })
      ),
      mode: Type.Optional(
        Type.Union([Type.Literal("summary"), Type.Literal("full")], {
          description: "View mode; summary is hot-path bounded.",
        })
      ),
      allow_prior_project_trajectory: Type.Optional(
        Type.Boolean({
          description:
            "If true, use the prior same-project trajectory as advisory reload fallback when continuity_id changed.",
        })
      ),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      const query = new URLSearchParams();
      query.set("project_root", projectRoot);
      if (p.session_id || getAttachmentRuntime().sessionFrameKey)
        query.set("session_id", String(p.session_id || getAttachmentRuntime().sessionFrameKey));
      const requestedContinuity = String(p.continuity_id || getContinuityId() || "").trim();
      if (requestedContinuity) query.set("continuity_id", requestedContinuity);
      const viewMode = String(p.mode || "summary");
      query.set("mode", viewMode);
      if (p.allow_prior_project_trajectory === true) query.set("allow_prior_project_trajectory", "true");
      const result = await focusaFetchDetailed(`/trajectory/view?${query.toString()}`, { method: "GET" });
      const body = result.body || {};
      if (!result.ok && body.failure_class === "hot_path_timeout") {
        const fallback = {
          ...(cachedTrajectoryForScope(projectRoot, requestedContinuity) || {}),
          status: "timeout_preserved",
          canonical: false,
          degraded: true,
          advisory_only: true,
          failure_class: "hot_path_timeout",
          project_root: projectRoot,
          continuity_id: String(p.continuity_id || getContinuityId() || "") || null,
          session_id: String(p.session_id || getAttachmentRuntime().sessionFrameKey || "") || null,
          preserved_at: new Date().toISOString(),
          next_step_hint:
            "Retry focusa_trajectory_view after focusa_tool_doctor/resource_mode; use fallback only as advisory orientation.",
        };
        setLastTrajectoryClarity(fallback);
        try {
          getAttachmentRuntime().pi?.appendEntry("focusa-trajectory-timeout-fallback", fallback);
        } catch {
          /* best effort */
        }
        persistState();
        return {
          content: [{ type: "text", text: timeoutPreservedText("trajectory view", "cached clarity") }],
          details: {
            ok: false,
            status: "timeout_preserved",
            endpoint: "/v1/trajectory/view",
            canonical: false,
            degraded: true,
            advisory_only: true,
            trajectory: fallback,
            failure_class: "hot_path_timeout",
            response: compactApiEcho(body),
            next_tools: [
              "focusa_tool_doctor",
              "focusa_resource_mode",
              "focusa_trajectory_view",
              "focusa_workpoint_resume",
            ],
          },
        } as any;
      }
      const project = body.project_identity || {};
      const trajectory = body.trajectory || {};
      if (!typedTrajectoryScopeMatches(body, projectRoot, requestedContinuity)) {
        return {
          content: [
            {
              type: "text",
              text: "trajectory view blocked: response scope does not match the requested project/workstream",
            },
          ],
          details: {
            ok: false,
            status: "blocked",
            canonical: true,
            degraded: true,
            failure_class: "scope_mismatch",
            endpoint: "/v1/trajectory/view",
            requested_scope: { project_root: projectRoot, continuity_id: requestedContinuity || null },
            response_scope: {
              project_root: project.project_root || trajectory.project_root || body.project_root || null,
              continuity_id:
                trajectory.continuity_id || body.scope?.continuity_id || body.continuity_id || null,
            },
            next_tools: ["focusa_project_identity", "focusa_project_verify", "focusa_trajectory_view"],
          },
        } as any;
      }
      const responseRoot = normalizeProjectRoot(project.project_root || trajectory.project_root || projectRoot);
      const responseContinuity = String(
        trajectory.continuity_id || body.scope?.continuity_id || body.continuity_id || requestedContinuity
      ).trim();
      if (!adoptVerifiedContinuityForCurrentSession(responseRoot, responseContinuity)) {
        return {
          content: [{ type: "text", text: "trajectory view blocked → exact verified scope adoption failed" }],
          details: {
            ok: false,
            status: "blocked",
            canonical: true,
            failure_class: "scope_mismatch",
            project_root: responseRoot,
            continuity_id: responseContinuity,
          },
        } as any;
      }
      if (trajectory.short_term_goal && !body.intelligence_view?.focus_trajectory_sync?.current_focus) {
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
      const trajectoryLadder = trajectory.trajectory_ladder || {};
      if (
        trajectory.long_term_goal ||
        trajectoryLadder.hlt ||
        trajectory.mid_level_goal ||
        trajectoryLadder.mlg ||
        trajectory.short_term_goal ||
        trajectoryLadder.stg ||
        trajectory.current_state ||
        trajectory.active_gap
      ) {
        setLastTrajectoryClarity({
          ...(cachedTrajectoryForScope(projectRoot, requestedContinuity) || {}),
          reason: "trajectory_view_tool",
          refreshed_at: Date.now(),
          status: String(
            body.intelligence_view?.clarity_gate?.status ||
              trajectory.definition_status ||
              body.status ||
              "unknown"
          ),
          recommended_action: String(
            body.intelligence_view?.clarity_gate?.recommended_action ||
              body.intelligence_view?.context_sufficiency?.recommended_action ||
              "unknown"
          ),
          canonical: body.canonical === true,
          degraded: body.degraded === true,
          project_root: String(project.project_root || projectRoot),
          continuity_id: String(p.continuity_id || getContinuityId() || body.continuity_id || "") || null,
          session_id:
            String(p.session_id || getAttachmentRuntime().sessionFrameKey || body.session_id || "") || null,
          project_identity_status: String(project.status || "unknown"),
          trajectory_id: trajectory.trajectory_id || null,
          scope_verification:
            trajectory.scope_verification || body.scope_verification || body.trajectory_scope_verification || null,
          fallback_prior_project_trajectory: trajectory.fallback_prior_project_trajectory === true,
          fallback_source_continuity_id: trajectory.fallback_source_continuity_id || null,
          long_term_goal: trajectory.long_term_goal || trajectoryLadder.hlt || null,
          desired_end_state: trajectory.desired_end_state || trajectoryLadder.desired_end_state || null,
          mid_level_goal: trajectory.mid_level_goal || trajectoryLadder.mlg || null,
          short_term_goal: trajectory.short_term_goal || trajectoryLadder.stg || null,
          waypoints: trajectory.waypoints || trajectoryLadder.waypoints || [],
          current_state: trajectory.current_state || null,
          active_gap: trajectory.active_gap || null,
          project_identity: project,
          project_urls: project.project_urls || null,
          focus_trajectory_sync: body.intelligence_view?.focus_trajectory_sync || null,
        });
        persistState();
      }
      const sufficiency = body.intelligence_view?.context_sufficiency || {};
      const posture = String(sufficiency.proceed_posture || sufficiency.recommended_action || "unknown");
      const projectMismatches = Array.isArray(project.mismatches) ? project.mismatches : [];
      const trajectoryUnset =
        body.status === "not_found" &&
        String(project.status || "") === "verified" &&
        projectMismatches.length === 0;
      const trajectoryBootstrapDefault =
        trajectory.bootstrap_default === true || trajectory.needs_definition === true;
      const recovery =
        trajectoryUnset || trajectoryBootstrapDefault
          ? null
          : scopeRecoveryContext(
              body,
              projectRoot,
              String(p.continuity_id || getContinuityId() || ""),
              "trajectory_view"
            );
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
      const toolResult = body.details?.tool_result_v1 || {
        ok: result.ok && body.status !== "degraded" && body.status !== "not_found",
        status: result.ok ? String(body.status || "completed") : String(result.status),
        canonical: body.canonical === true,
        degraded: body.degraded === true,
        failure_class: body.failure_class || null,
        retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" },
        side_effects: [],
        evidence_refs: [],
        next_tools: body.next_tools || ["focusa_workpoint_resume", "focusa_active_object_resolve"],
      };
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
          next_tools: toolResult.next_tools ||
            body.next_tools || ["focusa_workpoint_resume", "focusa_active_object_resolve"],
          response: compactApiEcho(body),
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_hlt_history",
    label: "HLT History",
    description:
      "Read append-only HLT ledger entries with session filters, fallback candidates, and generic HLT tracking. Spec 125 §7.2-7.6.",
    promptSnippet:
      "Use when reconstructing trajectory wording, checking previous-valid fallback, or verifying exact HLT history across sessions.",
    parameters: Type.Object({
      project_root: Type.Optional(Type.String({ description: "Project root for HLT history scope." })),
      continuity_id: Type.Optional(Type.String({ description: "Optional continuity_id filter." })),
      session_id: Type.Optional(
        Type.String({
          description: "Spec 125 §7.6: filter by session. 'current' resolves to active session.",
        })
      ),
      include_cross_session_fallbacks: Type.Optional(
        Type.Boolean({ description: "Include cross-session fallback candidates (default false)." })
      ),
      include_generic: Type.Optional(
        Type.Boolean({ description: "Include generic HLT entries (default false)." })
      ),
      limit: Type.Optional(
        Type.Integer({ minimum: 1, maximum: 500, description: "Max entries to return (defaults to 50)." })
      ),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      const query = new URLSearchParams();
      query.set("project_root", String(projectRoot));
      if (p.continuity_id) query.set("continuity_id", String(p.continuity_id));
      if (p.session_id) query.set("session_id", String(p.session_id));
      if (p.include_cross_session_fallbacks) query.set("include_cross_session_fallbacks", "true");
      if (p.include_generic) query.set("include_generic", "true");
      if (typeof p.limit === "number")
        query.set("limit", String(Math.min(Math.max(Math.trunc(p.limit), 1), 500)));
      const result = await focusaFetchDetailed(`/hlt/history?${query.toString()}`);
      const body = result.body || {};
      if (!result.ok) {
        return blockedToolResponse(
          "focusa_hlt_history",
          "trajectory",
          `hlt history blocked → ${explainWorkLoopResult(result, "hlt history unavailable")}`,
          body.failure_class || "daemon_unavailable",
          body,
          ["focusa_trajectory_view", "focusa_project_verify", "focusa_tool_doctor"]
        );
      }
      const toolResult =
        body.details?.tool_result_v1 ||
        focusaToolResult({
          ok: true,
          status: "completed",
          canonical: body.canonical === true,
          degraded: body.degraded === true,
          summary: `hlt history → project=${projectRoot} continuity=${String(p.continuity_id || "all")} session=${String(p.session_id || "all")} count=${String(body.count || 0)} generic_skipped=${String(body.generic_skipped || 0)} fallbacks=${String(body.fallback_candidates?.length || 0)}`,
          tool: "focusa_hlt_history",
          family: "trajectory",
          side_effects: [],
          evidence_refs: [],
          next_tools: ["focusa_trajectory_view", "focusa_trajectory_define_goal", "focusa_project_verify"],
          raw: body,
        });
      return {
        content: [
          {
            type: "text",
            text: `hlt history → ${projectRoot} continuity=${String(p.continuity_id || "all")} session=${String(p.session_id || "all")} entries=${String(body.count || 0)} generic_skipped=${String(body.generic_skipped || 0)} fallbacks=${String(body.fallback_candidates?.length || 0)} ledger=${String(body.ledger_file || "unknown")}`,
          },
        ],
        details: {
          ok: true,
          status: String(body.status || "completed"),
          endpoint: "/v1/hlt/history",
          canonical: body.canonical === true,
          degraded: body.degraded === true,
          project_root: String(projectRoot),
          continuity_id: p.continuity_id || body.continuity_id || null,
          entries: Array.isArray(body.entries) ? body.entries.slice(0, 200) : [],
          count: body.count || 0,
          ledger_file: body.ledger_file || null,
          tool_result_v1: toolResult,
          next_tools: toolResult.next_tools,
        } as any,
      };
    },
  });

  pi.registerTool({
    name: "focusa_trajectory_define_goal",
    label: "Trajectory Define Goal",
    description:
      "Create an advisory per-project Trajectory goal candidate without changing task/execution authority.",
    promptSnippet:
      "Use when the project trajectory is unclear or operator provides/changes the project goal.",
    parameters: Type.Object({
      long_term_goal: Type.String({ description: "Stable project-level long-term goal." }),
      desired_end_state: Type.String({ description: "Evidence-backed desired project end state." }),
      mid_level_goal: Type.Optional(
        Type.String({ description: "Current mid-level goal (MLG) derived from the HLT." })
      ),
      short_term_goal: Type.Optional(
        Type.String({ description: "Current short-term goal (STG) derived from the HLT/MLG." })
      ),
      waypoints: Type.Optional(
        Type.Array(Type.String(), {
          description: "Concrete HLT-aligned progress markers along the MLG/STG path.",
        })
      ),
      current_state: Type.Optional(Type.String({ description: "Current verified state if known." })),
      current_ask: Type.Optional(
        Type.String({
          description:
            "Explicit current operator intent; satisfies verified state gate (§169-175). Auto-populated from Pi session if omitted.",
        })
      ),
      goal_source: Type.Optional(
        Type.String({
          description: "operator|durable_supersession|focus_state|workpoint|beads|imported|inferred_context",
        })
      ),
      supersedes_trajectory_id: Type.Optional(
        Type.String({ description: "Prior trajectory id if this supersedes one." })
      ),
      operator_confirmed: Type.Optional(
        Type.Boolean({ description: "True when operator explicitly confirmed a root goal change." })
      ),
      supersession_evidence_refs: Type.Optional(
        Type.Array(Type.String(), {
          description:
            "Durable evidence refs allowing root goal supersession without direct operator prompt.",
        })
      ),
      required_evidence_refs: Type.Optional(
        Type.Array(Type.String(), { description: "Evidence refs required to prove the desired end state." })
      ),
      required_checks: Type.Optional(
        Type.Array(Type.String(), {
          description: "Checks required before the trajectory can be considered done.",
        })
      ),
      acceptance_risks: Type.Optional(
        Type.Array(Type.String(), { description: "Known false-completion or acceptance risks." })
      ),
      not_done_if: Type.Optional(
        Type.Array(Type.String(), { description: "Conditions proving the trajectory is not done." })
      ),
      project_root: Type.Optional(
        Type.String({ description: "Optional expected project root; defaults to Pi session cwd." })
      ),
      session_id: Type.Optional(
        Type.String({ description: "Optional temporal Pi session id; defaults to Pi session key." })
      ),
      continuity_id: Type.Optional(
        Type.String({ description: "Optional logical continuity id; defaults to Pi continuity id." })
      ),
      idempotency_key: Type.Optional(Type.String({ description: "Optional external idempotency key." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      const body = {
        ...p,
        project_root: projectRoot,
        session_id: p.session_id || getAttachmentRuntime().sessionFrameKey,
        continuity_id: p.continuity_id || getContinuityId(),
        current_ask: p.current_ask || getAttachmentRuntime().currentAsk?.text || "",
        session_identity: await buildFocusaSessionIdentity(projectRoot, "manual", {
          continuityId: p.continuity_id,
          sessionId: p.session_id,
        }),
      };
      const result = await focusaFetchDetailed("/trajectory/define-goal", {
        method: "POST",
        body: JSON.stringify(body),
      });
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
        setLastTrajectoryClarity({
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
        });
        try {
          getAttachmentRuntime().pi?.appendEntry("focusa-trajectory-timeout-fallback", fallbackCandidate);
        } catch {
          /* best effort */
        }
        persistState();
        return {
          content: [{ type: "text", text: timeoutPreservedText("trajectory define_goal", "candidate") }],
          details: {
            ok: false,
            status: "timeout_preserved",
            endpoint: "/v1/trajectory/define-goal",
            canonical: false,
            degraded: true,
            advisory_only: true,
            trajectory_candidate: fallbackCandidate,
            failure_class: "hot_path_timeout",
            response: compactApiEcho(b),
            next_tools: [
              "focusa_tool_doctor",
              "focusa_resource_mode",
              "focusa_trajectory_define_goal",
              "focusa_trajectory_view",
            ],
          },
        } as any;
      }
      const pendingCandidate =
        String(b.status || "") === "pending" && !b.trajectory_candidate
          ? {
              long_term_goal: body.long_term_goal,
              desired_end_state: body.desired_end_state,
              mid_level_goal: body.mid_level_goal,
              short_term_goal: body.short_term_goal,
              waypoints: body.waypoints || [],
              current_state: body.current_state,
              definition_status: "pending",
            }
          : null;
      const candidate = b.trajectory_candidate || pendingCandidate || {};
      const defineLabel =
        String(b.status || "") === "pending" ? "PENDING" : b.canonical === true ? "SET" : "NOT SET";
      const text = result.ok
        ? `trajectory define_goal → ${defineLabel} HLT=${String(candidate.long_term_goal || "missing")} MLG=${String(candidate.mid_level_goal || "missing")} STG=${String(candidate.short_term_goal || "missing")} waypoints=${Array.isArray(candidate.waypoints) ? candidate.waypoints.length : 0} definition=${String(candidate.definition_status || "unknown")} persisted=${b.persisted === true}`
        : `trajectory define_goal blocked → ${explainWorkLoopResult(result, "define failed")}`;
      const toolResult = b.details?.tool_result_v1 || {
        ok: result.ok && b.status !== "validation_rejected",
        status: result.ok ? String(b.status || "completed") : String(result.status),
        canonical: b.canonical === true,
        degraded: b.degraded === true,
        failure_class: b.failure_class || null,
        retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" },
        side_effects: [],
        evidence_refs: p.supersession_evidence_refs || [],
        next_tools: b.next_tools || ["focusa_trajectory_assess"],
      };
      return {
        content: [{ type: "text", text }],
        details: {
          ok: toolResult.ok,
          status: result.ok ? String(b.status || "completed") : String(result.status),
          endpoint: "/v1/trajectory/define-goal",
          canonical: b.canonical === true,
          degraded: b.degraded === true,
          advisory_only: b.advisory_only === true,
          trajectory_candidate: candidate,
          tool_result_v1: toolResult,
          failure_class: toolResult.failure_class || null,
          side_effects: toolResult.side_effects || [],
          evidence_refs: toolResult.evidence_refs || [],
          response: compactApiEcho(b),
          next_tools: toolResult.next_tools || b.next_tools || ["focusa_trajectory_assess"],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_trajectory_assess",
    label: "Trajectory Assess",
    description:
      "Assess current project state against the desired Trajectory end state and return gaps/recommended action.",
    promptSnippet:
      "Use after trajectory view/define_goal or after verification evidence changes current state.",
    parameters: Type.Object({
      observed_state: Type.Optional(Type.String({ description: "Observed current state override." })),
      evidence_refs: Type.Optional(
        Type.Array(Type.String(), { description: "Evidence refs supporting observed state." })
      ),
      project_root: Type.Optional(
        Type.String({ description: "Optional expected project root; defaults to Pi session cwd." })
      ),
      session_id: Type.Optional(
        Type.String({ description: "Optional temporal Pi session id; defaults to Pi session key." })
      ),
      continuity_id: Type.Optional(
        Type.String({ description: "Optional logical continuity id; defaults to Pi continuity id." })
      ),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      const body = {
        ...p,
        project_root: projectRoot,
        session_id: p.session_id || getAttachmentRuntime().sessionFrameKey,
        continuity_id: p.continuity_id || getContinuityId(),
        session_identity: await buildFocusaSessionIdentity(projectRoot, "manual", {
          continuityId: p.continuity_id,
          sessionId: p.session_id,
        }),
      };
      const result = await focusaFetchDetailed("/trajectory/assess", {
        method: "POST",
        body: JSON.stringify(body),
      });
      const b = result.body || {};
      if (!result.ok && b.failure_class === "hot_path_timeout")
        return trajectoryTimeoutFallbackResult(
          "assess",
          "/v1/trajectory/assess",
          body,
          b,
          [
            "focusa_tool_doctor",
            "focusa_resource_mode",
            "focusa_trajectory_assess",
            "focusa_trajectory_propose_workpoint",
          ],
          { observed_state: body.observed_state || null, evidence_refs: body.evidence_refs || [] }
        );
      const text = result.ok
        ? `trajectory assess → gaps=${Array.isArray(b.gaps) ? b.gaps.length : 0} action=${String(b.recommended_action || "unknown")} canonical=${b.canonical === true}${
            Array.isArray(b.gaps) && b.gaps.length > 0
              ? "\n" +
                b.gaps
                  .slice(0, 3)
                  .map(
                    (g: any, i: number) =>
                      `  gap[${i}]: ${String(g.gap_ref || "unknown")} severity=${String(g.severity || "unknown")} → ${String(g.recommended_action || "none")}`
                  )
                  .join("\n")
              : ""
          }`
        : `trajectory assess blocked → ${explainWorkLoopResult(result, "assess failed")}`;
      const toolResult = b.details?.tool_result_v1 || {
        ok: result.ok,
        status: result.ok ? String(b.status || "completed") : String(result.status),
        canonical: b.canonical === true,
        degraded: b.degraded === true,
        failure_class: b.failure_class || null,
        retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" },
        side_effects: [],
        evidence_refs: p.evidence_refs || [],
        next_tools: b.next_tools || ["focusa_trajectory_propose_workpoint"],
      };
      return {
        content: [{ type: "text", text }],
        details: {
          ok: toolResult.ok,
          status: result.ok ? String(b.status || "completed") : String(result.status),
          endpoint: "/v1/trajectory/assess",
          canonical: b.canonical === true,
          degraded: b.degraded === true,
          gaps: b.gaps || [],
          recommended_action: b.recommended_action || null,
          tool_result_v1: toolResult,
          failure_class: toolResult.failure_class || null,
          side_effects: toolResult.side_effects || [],
          evidence_refs: toolResult.evidence_refs || [],
          response: compactApiEcho(b),
          next_tools: toolResult.next_tools || b.next_tools || ["focusa_trajectory_propose_workpoint"],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_trajectory_propose_workpoint",
    label: "Trajectory Propose Workpoint",
    description:
      "Propose an advisory Workpoint candidate from the active per-project Trajectory gap; does not promote or execute it.",
    promptSnippet:
      "Use after trajectory assess says propose_workpoint; pass candidate to focusa_workpoint_checkpoint only if accepted.",
    parameters: Type.Object({
      trajectory_id: Type.Optional(
        Type.String({ description: "Trajectory id to use; defaults to active project trajectory." })
      ),
      target_ref: Type.Optional(Type.String({ description: "Optional target object/file/ref." })),
      action_type: Type.Optional(Type.String({ description: "Optional action intent type." })),
      project_root: Type.Optional(
        Type.String({ description: "Optional expected project root; defaults to Pi session cwd." })
      ),
      session_id: Type.Optional(
        Type.String({ description: "Optional temporal Pi session id; defaults to Pi session key." })
      ),
      continuity_id: Type.Optional(
        Type.String({ description: "Optional logical continuity id; defaults to Pi continuity id." })
      ),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      const body = {
        ...p,
        project_root: projectRoot,
        session_id: p.session_id || getAttachmentRuntime().sessionFrameKey,
        continuity_id: p.continuity_id || getContinuityId(),
        session_identity: await buildFocusaSessionIdentity(projectRoot, "manual", {
          continuityId: p.continuity_id,
          sessionId: p.session_id,
        }),
      };
      const result = await focusaFetchDetailed("/trajectory/propose-workpoint", {
        method: "POST",
        body: JSON.stringify(body),
      });
      const b = result.body || {};
      if (!result.ok && b.failure_class === "hot_path_timeout")
        return trajectoryTimeoutFallbackResult(
          "propose_workpoint",
          "/v1/trajectory/propose-workpoint",
          body,
          b,
          [
            "focusa_tool_doctor",
            "focusa_resource_mode",
            "focusa_trajectory_propose_workpoint",
            "focusa_workpoint_checkpoint",
          ],
          {
            workpoint_candidate: {
              action_intent: {
                action_type: body.action_type || "unknown",
                target_ref: body.target_ref || body.trajectory_id || "trajectory",
              },
              checkpoint_required: true,
              blockers: [
                {
                  reason: "trajectory proposal timed out before canonical candidate was returned",
                  severity: "medium",
                  status: "open",
                },
              ],
            },
          }
        );
      const candidate = b.workpoint_candidate || {};
      const blockers = Array.isArray(candidate.blockers) ? candidate.blockers.length : 0;
      const text = result.ok
        ? `trajectory propose_workpoint → advisory=${b.advisory_only === true} action=${String(candidate.action_intent?.action_type || "unknown")} checkpoint_required=${candidate.checkpoint_required === true} blockers=${blockers} no_execution=${b.no_execution_side_effects === true}`
        : `trajectory propose_workpoint blocked → ${explainWorkLoopResult(result, "proposal failed")}`;
      const toolResult = b.details?.tool_result_v1 || {
        ok: result.ok,
        status: result.ok ? String(b.status || "completed") : String(result.status),
        canonical: b.canonical === true,
        degraded: b.degraded === true,
        failure_class: b.failure_class || null,
        retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" },
        side_effects: [],
        evidence_refs: [],
        next_tools: b.next_tools || ["focusa_workpoint_checkpoint"],
      };
      return {
        content: [{ type: "text", text }],
        details: {
          ok: toolResult.ok,
          status: result.ok ? String(b.status || "completed") : String(result.status),
          endpoint: "/v1/trajectory/propose-workpoint",
          canonical: b.canonical === true,
          degraded: b.degraded === true,
          advisory_only: b.advisory_only === true,
          no_execution_side_effects: b.no_execution_side_effects === true,
          workpoint_candidate: candidate,
          tool_result_v1: toolResult,
          failure_class: toolResult.failure_class || null,
          side_effects: toolResult.side_effects || [],
          evidence_refs: toolResult.evidence_refs || [],
          response: compactApiEcho(b),
          next_tools: toolResult.next_tools || b.next_tools || ["focusa_workpoint_checkpoint"],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_trajectory_checkpoint",
    label: "Trajectory Checkpoint",
    description:
      "Create an advisory Trajectory checkpoint packet before compaction/model switch; pair with Workpoint checkpoint for canonical continuation.",
    promptSnippet:
      "Use before compaction/model switch alongside focusa_workpoint_checkpoint; this does not replace Workpoint.",
    parameters: Type.Object({
      summary: Type.Optional(Type.String({ description: "Optional bounded Trajectory checkpoint summary." })),
      project_root: Type.Optional(
        Type.String({ description: "Optional expected project root; defaults to Pi session cwd." })
      ),
      session_id: Type.Optional(
        Type.String({ description: "Optional temporal Pi session id; defaults to Pi session key." })
      ),
      continuity_id: Type.Optional(
        Type.String({ description: "Optional logical continuity id; defaults to Pi continuity id." })
      ),
      idempotency_key: Type.Optional(Type.String({ description: "Optional external idempotency key." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      const body = {
        ...p,
        project_root: projectRoot,
        session_id: p.session_id || getAttachmentRuntime().sessionFrameKey,
        continuity_id: p.continuity_id || getContinuityId(),
        session_identity: await buildFocusaSessionIdentity(projectRoot, "compaction", {
          continuityId: p.continuity_id,
          sessionId: p.session_id,
        }),
      };
      const result = await focusaFetchDetailed("/trajectory/checkpoint", {
        method: "POST",
        body: JSON.stringify(body),
      });
      const b = result.body || {};
      if (!result.ok && b.failure_class === "hot_path_timeout")
        return trajectoryTimeoutFallbackResult(
          "checkpoint",
          "/v1/trajectory/checkpoint",
          body,
          b,
          [
            "focusa_tool_doctor",
            "focusa_resource_mode",
            "focusa_trajectory_checkpoint",
            "focusa_workpoint_checkpoint",
          ],
          {
            trajectory_checkpoint: {
              summary: body.summary || "trajectory checkpoint timeout fallback",
              persisted: false,
            },
          }
        );
      const text = result.ok
        ? `trajectory checkpoint → status=${String(b.status || "unknown")} persisted=${b.persisted === true} canonical=${b.canonical === true}`
        : `trajectory checkpoint blocked → ${explainWorkLoopResult(result, "checkpoint failed")}`;
      const toolResult = b.details?.tool_result_v1 || {
        ok: result.ok,
        status: result.ok ? String(b.status || "completed") : String(result.status),
        canonical: b.canonical === true,
        degraded: b.degraded === true,
        failure_class: b.failure_class || null,
        retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" },
        side_effects: [],
        evidence_refs: [],
        next_tools: b.next_tools || ["focusa_workpoint_checkpoint"],
      };
      return {
        content: [{ type: "text", text }],
        details: {
          ok: toolResult.ok,
          status: result.ok ? String(b.status || "completed") : String(result.status),
          endpoint: "/v1/trajectory/checkpoint",
          canonical: b.canonical === true,
          degraded: b.degraded === true,
          persisted: b.persisted === true,
          advisory_only: b.advisory_only === true,
          trajectory_checkpoint: b.trajectory_checkpoint || null,
          tool_result_v1: toolResult,
          failure_class: toolResult.failure_class || null,
          side_effects: toolResult.side_effects || [],
          evidence_refs: toolResult.evidence_refs || [],
          response: compactApiEcho(b),
          next_tools: toolResult.next_tools || b.next_tools || ["focusa_workpoint_checkpoint"],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_trajectory_resume",
    label: "Trajectory Resume",
    description:
      "Resume per-project Trajectory orientation plus Workpoint handoff context after compaction/model switch/session resume.",
    promptSnippet: "Use after compaction/resume before choosing action; inject with Workpoint resume.",
    parameters: Type.Object({
      mode: Type.Optional(
        Type.Union([Type.Literal("summary"), Type.Literal("full")], {
          description: "Resume mode; summary is bounded.",
        })
      ),
      project_root: Type.Optional(
        Type.String({ description: "Optional expected project root; defaults to Pi session cwd." })
      ),
      session_id: Type.Optional(
        Type.String({ description: "Optional temporal Pi session id; defaults to Pi session key." })
      ),
      continuity_id: Type.Optional(
        Type.String({ description: "Optional logical continuity id; defaults to Pi continuity id." })
      ),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      const body = {
        ...p,
        project_root: projectRoot,
        session_id: p.session_id || getAttachmentRuntime().sessionFrameKey,
        continuity_id: p.continuity_id || getContinuityId(),
        session_identity: await buildFocusaSessionIdentity(projectRoot, "session_switch", {
          continuityId: p.continuity_id,
          sessionId: p.session_id,
        }),
      };
      const result = await focusaFetchDetailed("/trajectory/resume", {
        method: "POST",
        body: JSON.stringify(body),
      });
      const b = result.body || {};
      if (!result.ok && b.failure_class === "hot_path_timeout")
        return trajectoryTimeoutFallbackResult(
          "resume",
          "/v1/trajectory/resume",
          body,
          b,
          [
            "focusa_tool_doctor",
            "focusa_resource_mode",
            "focusa_trajectory_resume",
            "focusa_workpoint_resume",
          ],
          { resume_packet: getLastTrajectoryClarity() || null }
        );
      const packet = b.resume_packet || {};
      const text = result.ok
        ? `trajectory resume → status=${String(b.status || "unknown")} canonical=${b.canonical === true} project=${String(packet.project_identity?.status || "unknown")}`
        : `trajectory resume blocked → ${explainWorkLoopResult(result, "resume failed")}`;
      const toolResult = b.details?.tool_result_v1 || {
        ok: result.ok && b.status !== "degraded" && b.status !== "not_found",
        status: result.ok ? String(b.status || "completed") : String(result.status),
        canonical: b.canonical === true,
        degraded: b.degraded === true,
        failure_class: b.failure_class || null,
        retry: { safe: result.ok, posture: result.ok ? "safe_retry" : "check_side_effects_first" },
        side_effects: [],
        evidence_refs: [],
        next_tools: b.next_tools || ["focusa_workpoint_resume"],
      };
      return {
        content: [{ type: "text", text }],
        details: {
          ok: toolResult.ok,
          status: result.ok ? String(b.status || "completed") : String(result.status),
          endpoint: "/v1/trajectory/resume",
          canonical: b.canonical === true,
          degraded: b.degraded === true,
          resume_packet: packet,
          tool_result_v1: toolResult,
          failure_class: toolResult.failure_class || null,
          side_effects: toolResult.side_effects || [],
          evidence_refs: toolResult.evidence_refs || [],
          response: compactApiEcho(b),
          next_tools: toolResult.next_tools || b.next_tools || ["focusa_workpoint_resume"],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_active_object_resolve",
    label: "Focusa Active Object Resolve",
    description:
      "Resolve likely active object references from the current Workpoint and optional hint without inventing canonical refs.",
    promptSnippet: "Use before linking evidence or acting when target object/file/endpoint is ambiguous.",
    parameters: Type.Object({
      hint: Type.Optional(Type.String({ description: "Optional file/object/endpoint/work item hint." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const ctx = resolveActiveWorkpointContext();
      const packet = getActiveWorkpointPacket() || {};
      const workpoint = packet?.resume_packet || packet?.workpoint || packet;
      const refs = Array.from(
        new Set(
          [
            ...(Array.isArray(workpoint?.active_object_refs) ? workpoint.active_object_refs : []),
            workpoint?.work_item_id,
            workpoint?.action_intent?.target_ref,
            p.hint,
          ]
            .filter(Boolean)
            .map(String)
        )
      );
      // FOCUSA_FIX-i4fg: emit active_object_source hint so the agent knows WHY
      // count is 0 (no Workpoint, no hint, or no refs in packet).
      let source: string;
      if (refs.length > 0) {
        source = "refs_collected";
      } else if (!ctx.workpoint_id) {
        source = "no_active_workpoint";
      } else if (!p.hint) {
        source = "no_hint_provided";
      } else {
        source = "workpoint_has_no_object_refs";
      }
      const text = `active object resolve → count=${refs.length} verified=false source=${source} refs=${refs.slice(0, 5).join(",") || "none"}`;
      return {
        content: [{ type: "text", text }],
        details: { ok: true, status: "completed", workpoint_id: ctx.workpoint_id, refs, verified: false },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_evidence_capture",
    label: "Focusa Evidence Capture",
    description: "Capture a bounded evidence ref/result and optionally link it to the active Workpoint.",
    promptSnippet:
      "Use after tests, stress runs, or proof collection to keep handles instead of transcript blobs.",
    parameters: Type.Object({
      target_ref: Type.String({
        description: "Object/file/test/endpoint/work item proven by this evidence.",
      }),
      result: Type.String({ description: "Bounded result summary." }),
      evidence_ref: Type.String({ description: "Stable evidence handle/path/test id." }),
      workpoint_id: Type.Optional(
        Type.String({ description: "Specific Workpoint id; omit to use active Workpoint." })
      ),
      project_root: Type.Optional(
        Type.String({
          description:
            "Explicit safe project folder/root; use after compaction if Pi cwd is broad like /root.",
        })
      ),
      session_id: Type.Optional(
        Type.String({ description: "Optional temporal Pi session id; defaults to this Pi session key." })
      ),
      continuity_id: Type.Optional(
        Type.String({
          description: "Stable logical session/workstream id; defaults to this Pi continuity id.",
        })
      ),
      attach_to_workpoint: Type.Optional(Type.Boolean({ description: "Defaults true." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      if (p.attach_to_workpoint === false) {
        const projectRoot = p.project_root ? await resolveFocusaToolProjectRoot(p.project_root) : null;
        return {
          content: [
            {
              type: "text",
              text: `evidence capture → captured ref=${p.evidence_ref} attach_to_workpoint=false`,
            },
          ],
          details: {
            ok: true,
            status: "completed",
            evidence_ref: p.evidence_ref,
            project_root_permission_posture: projectRoot ? projectRootPermissionPosture(projectRoot) : null,
          },
        } as any;
      }
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      const clarity = await enforceTrajectoryClarityPrecondition(projectRoot, "evidence capture", {
        blockOperatorInput: false,
        continuityId: p.continuity_id,
        sessionId: p.session_id,
      });
      if (!clarity.ok) {
        const degraded = evidenceClarityFallbackResult("evidence capture", p, projectRoot, clarity);
        if (degraded) return degraded;
        return {
          content: [
            {
              type: "text",
              text: `${clarity.text || "evidence capture blocked by trajectory clarity gate"}. Why: trajectory clarity is required before linking proof to canonical Workpoint state; follow next_tools/recovery_hint instead of retrying blindly.`,
            },
          ],
          details: {
            ok: false,
            status: "blocked",
            why: "trajectory clarity is required before canonical evidence linkage",
            ...clarity.details,
          },
        } as any;
      }
      const sessionIdentity = await buildFocusaSessionIdentity(projectRoot, "manual", {
        continuityId: p.continuity_id,
        sessionId: p.session_id,
      });
      // Evidence linkage is governed by explicit session identity and server-side Workpoint scope, not Work Loop ownership.
      const res = await focusaFetchDetailed("/workpoint/evidence/link", {
        method: "POST",
        body: JSON.stringify({
          workpoint_id: p.workpoint_id,
          target_ref: p.target_ref,
          result: p.result,
          evidence_ref: p.evidence_ref,
          session_identity: sessionIdentity,
          trajectory_clarity_precondition: clarity.details,
        }),
      });
      const recovery = res.ok
        ? null
        : scopeRecoveryContext(
            res.body || {},
            projectRoot,
            p.continuity_id || getContinuityId() || "",
            "evidence_capture"
          );
      const text = res.ok
        ? `evidence capture → linked ${p.evidence_ref}`
        : [`evidence capture blocked → ${explainWorkLoopResult(res, "link failed")}`, recovery?.text]
            .filter(Boolean)
            .join("\n");
      return {
        content: [{ type: "text", text }],
        details: {
          ok: res.ok,
          status: String(res.status),
          evidence_ref: p.evidence_ref,
          failure_class: res.body?.failure_class || null,
          scope_recovery_context: recovery?.details || null,
          request_scope: { project_root: projectRoot, continuity_id: sessionIdentity?.continuity_id || null },
          project_root_permission_posture: projectRootPermissionPosture(projectRoot),
          response: compactApiEcho(res.body),
          next_tools: recovery?.details?.safe_next_tools ||
            res.body?.next_tools || ["focusa_workpoint_resume", "focusa_workpoint_checkpoint"],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_browser_diagnostics_intake",
    label: "Browser Diagnostics Intake",
    description:
      "Turn UIAI/browser diagnostics JSON into bounded Focusa evidence, active-object hints, a prediction candidate, and a metacog candidate.",
    promptSnippet:
      "Use after UIAI browser diagnostics/error envelopes to standardize evidence + learning intake before manual interpretation.",
    parameters: Type.Object({
      diagnostics: Type.Optional(
        Type.Any({ description: "Diagnostics JSON object or browser action failure envelope." })
      ),
      diagnostics_ref: Type.Optional(
        Type.String({
          description:
            "Stable file/artifact/URL handle for diagnostics JSON; local files are read best-effort.",
        })
      ),
      target_ref: Type.Optional(
        Type.String({
          description:
            "Object/page/endpoint proven by these diagnostics; inferred from diagnostics when omitted.",
        })
      ),
      result: Type.Optional(Type.String({ description: "Optional bounded result summary override." })),
      workpoint_id: Type.Optional(
        Type.String({ description: "Specific Workpoint id; omit to use active Workpoint." })
      ),
      project_root: Type.Optional(
        Type.String({ description: "Explicit project root for canonical evidence linkage." })
      ),
      session_id: Type.Optional(
        Type.String({ description: "Optional temporal Pi session id; defaults to this Pi session key." })
      ),
      continuity_id: Type.Optional(
        Type.String({
          description: "Stable logical session/workstream id; defaults to this Pi continuity id.",
        })
      ),
      attach_to_workpoint: Type.Optional(
        Type.Boolean({
          description: "Defaults true; false performs dry intake without canonical evidence linkage.",
        })
      ),
      create_prediction: Type.Optional(
        Type.Boolean({ description: "Defaults true; records bounded follow-up prediction candidate." })
      ),
      create_metacog: Type.Optional(
        Type.Boolean({
          description:
            "Defaults false; capture only when this diagnostics pattern should become reusable learning.",
        })
      ),
    }),
    async execute(_id, params) {
      const p = params as any;
      const readJsonArtifact = (ref?: string): any | null => {
        if (!ref || !ref.startsWith("/")) return null;
        try {
          const fs = require("fs");
          const raw = fs.readFileSync(ref, "utf8");
          return JSON.parse(raw);
        } catch {
          return null;
        }
      };
      const diagnostics =
        p.diagnostics && typeof p.diagnostics === "object"
          ? p.diagnostics
          : readJsonArtifact(p.diagnostics_ref) || {};
      const focusaScope =
        diagnostics.focusa_scope ||
        diagnostics.session?.focusa_scope ||
        diagnostics.diagnostics?.focusa_scope ||
        {};
      const scopedWorkpointId = p.workpoint_id || focusaScope.workpoint_id;
      const scopedContinuityId = String(p.continuity_id || focusaScope.continuity_id || "");
      const scopedProjectRoot = p.project_root || focusaScope.project_root;
      const scopedSessionId = p.session_id || focusaScope.session_id;
      const asArray = (value: any): any[] => (Array.isArray(value) ? value : []);
      const dig = (obj: any, keys: string[]): any =>
        keys.reduce((cur, key) => (cur && typeof cur === "object" ? cur[key] : undefined), obj);
      const consoleItems = [
        ...asArray(diagnostics.console),
        ...asArray(diagnostics.diagnostics?.console),
        ...asArray(diagnostics.console_errors),
      ];
      const exceptionItems = [
        ...asArray(diagnostics.exceptions),
        ...asArray(diagnostics.page_errors),
        ...asArray(diagnostics.errors),
      ];
      const failedItems = [
        ...asArray(diagnostics.failed_requests),
        ...asArray(diagnostics.network_failures),
        ...asArray(diagnostics.diagnostics?.failed_requests),
      ];
      const classifyDiagnosticsSeverity = () => {
        const allText = JSON.stringify({
          consoleItems,
          exceptionItems,
          failedItems,
          diagnostics,
        }).toLowerCase();
        const benignAsset =
          failedItems.length > 0 &&
          failedItems.every((item: any) =>
            /\.(png|jpe?g|gif|webp|svg|ico|css|woff2?|ttf)(\?|$)|favicon|analytics|pixel|beacon|tracking/.test(
              String(item.url || item.request_url || item.name || item).toLowerCase()
            )
          );
        if (
          exceptionItems.length > 0 ||
          /blank page|page crashed|navigation failed|main frame|document failed|hydration failed/.test(
            allText
          )
        )
          return {
            severity: "page_breaking",
            alarm: "repair_required",
            recommended_action:
              "capture diagnostics evidence, inspect page-breaking exception/navigation failure, then repair before retry",
          };
        if (
          /selector_not_found|timeout|click failed|form failed|wait failed|429|403|cors|api failed|workflow blocked/.test(
            allText
          )
        )
          return {
            severity: "workflow_blocking",
            alarm: "action_blocked",
            recommended_action:
              "use snapshot/read/diagnostics to choose a different selector or API recovery path",
          };
        if (benignAsset)
          return {
            severity: "benign_asset",
            alarm: "calm",
            recommended_action:
              "record bounded evidence only if relevant; continue workflow without alarming repair loop",
          };
        if (consoleItems.length === 0 && exceptionItems.length === 0 && failedItems.length === 0)
          return {
            severity: "unknown",
            alarm: "baseline",
            recommended_action: "treat as clean baseline unless UI state contradicts diagnostics",
          };
        return {
          severity: "unknown",
          alarm: "review",
          recommended_action: "review diagnostics context before deciding whether repair is needed",
        };
      };
      const severityClassification = classifyDiagnosticsSeverity();
      const errorClass = String(diagnostics.error_class || diagnostics.error?.class || "browser_diagnostics");
      const url = String(
        diagnostics.url ||
          diagnostics.page_url ||
          diagnostics.session?.url ||
          dig(diagnostics, ["diagnostics", "url"]) ||
          "unknown-url"
      );
      const action = String(
        diagnostics.action || diagnostics.operation || diagnostics.selector
          ? `${diagnostics.action || "browser_action"}:${diagnostics.selector || "unknown-selector"}`
          : "browser_diagnostics"
      );
      const targetRef = String(p.target_ref || (url !== "unknown-url" ? url : action));
      const evidenceRef = String(
        p.diagnostics_ref ||
          diagnostics.evidence_ref ||
          focusaScope.evidence_ref ||
          `browser-diagnostics:${new Date().toISOString()}`
      );
      const diagSummary = String(diagnostics.diagnostics_summary || diagnostics.summary || "");
      const resultSummary = String(
        p.result ||
          `${errorClass}: severity=${severityClassification.severity} alarm=${severityClassification.alarm} console=${consoleItems.length} exceptions=${exceptionItems.length} failed_requests=${failedItems.length}${diagSummary ? `; ${diagSummary}` : ""}`
      ).slice(0, 500);
      const activeObjectHints = Array.from(
        new Set(
          [targetRef, url, action, diagnostics.selector, diagnostics.request_url, diagnostics.endpoint]
            .filter(Boolean)
            .map(String)
        )
      ).slice(0, 8);
      const sideEffects: string[] = [];
      const evidenceRefs: string[] = [evidenceRef];
      let evidenceResult: any = null;
      let projectRoot: string | null = null;
      if (p.attach_to_workpoint !== false) {
        projectRoot = await resolveFocusaToolProjectRoot(scopedProjectRoot);
        const projectRootGate = projectRootConfirmationGate(projectRoot, scopedProjectRoot);
        if (projectRootGate) return projectRootGate;
        if (!projectRoot) throw new Error("typed_scope_required");
        const clarity = await enforceTrajectoryClarityPrecondition(
          projectRoot,
          "browser diagnostics intake",
          { blockOperatorInput: false, continuityId: scopedContinuityId, sessionId: scopedSessionId }
        );
        if (!clarity.ok) {
          return {
            content: [
              {
                type: "text",
                text: `${clarity.text || "browser diagnostics intake blocked by trajectory clarity gate"}. next_tools=focusa_trajectory_view,focusa_workpoint_resume`,
              },
            ],
            details: {
              ok: false,
              status: "blocked",
              failure_class: "scope_mismatch",
              target_ref: targetRef,
              evidence_ref: evidenceRef,
              active_object_hints: activeObjectHints,
              ...clarity.details,
            },
          } as any;
        }
        const sessionIdentity = await buildFocusaSessionIdentity(projectRoot, "manual", {
          continuityId: scopedContinuityId,
          sessionId: scopedSessionId,
        });
        // Read-only browser evidence does not acquire continuous-execution writer authority.
        evidenceResult = await focusaFetchDetailed("/workpoint/evidence/link", {
          method: "POST",
          body: JSON.stringify({
            workpoint_id: scopedWorkpointId,
            target_ref: targetRef,
            result: resultSummary,
            evidence_ref: evidenceRef,
            session_identity: sessionIdentity,
            trajectory_clarity_precondition: clarity.details,
          }),
        });
        if (evidenceResult.ok) sideEffects.push("workpoint_evidence_link");
      }
      let predictionResult: any = null;
      if (p.create_prediction !== false) {
        predictionResult = await focusaFetchDetailed("/predictions", {
          method: "POST",
          body: JSON.stringify({
            scope: buildProjectWorkstreamKey(projectRoot || "", scopedContinuityId),
            prediction_type: "browser_diagnostics_next_action",
            predicted_outcome:
              severityClassification.severity === "benign_asset"
                ? "Benign asset diagnostics will not trigger an unnecessary repair loop."
                : failedItems.length || exceptionItems.length || consoleItems.length
                  ? "Diagnostics intake will shorten the next browser-debug loop by preserving concrete console/network/error evidence."
                  : "Diagnostics intake will act as a clean baseline for future browser-debug comparisons.",
            confidence:
              severityClassification.severity === "benign_asset"
                ? 0.82
                : failedItems.length || exceptionItems.length || consoleItems.length
                  ? 0.78
                  : 0.62,
            recommended_action: severityClassification.recommended_action,
            why: resultSummary,
            context_refs: evidenceRefs,
            ontology_context: {
              object_refs: activeObjectHints,
              evidence_refs: evidenceRefs,
              action_refs: [action],
              tool_refs: ["focusa_browser_diagnostics_intake"],
            },
          }),
        });
        if (predictionResult.ok) sideEffects.push("prediction_store");
      }
      let metacogResult: any = null;
      const diagnosticSignalCount = consoleItems.length + exceptionItems.length + failedItems.length;
      const recurringOrSignificant =
        diagnostics.recurring === true ||
        diagnostics.recurring_pattern === true ||
        diagnosticSignalCount >= 2 ||
        failedItems.length >= 1 ||
        exceptionItems.length >= 1;
      if (p.create_metacog === true && recurringOrSignificant) {
        metacogResult = await callSpec80Tool(
          "focusa_metacog_capture",
          "/metacognition/capture",
          {
            kind: "browser_diagnostics_pattern",
            content: `Browser diagnostics pattern for ${targetRef}: ${resultSummary}`.slice(
              0,
              SPEC81_LIMITS.longText
            ),
            rationale:
              "Captured from typed UIAI/browser diagnostics intake because the envelope contains recurring or significant browser failure evidence.",
            evidence_refs: evidenceRefs,
            confidence: 0.74,
            strategy_class: "browser_debugging",
          },
          { method: "POST", writer: true }
        );
        if (metacogResult.ok) sideEffects.push("metacog_capture");
      } else if (p.create_metacog === true) {
        sideEffects.push("metacog_skipped_low_signal");
      }
      const ok = p.attach_to_workpoint === false || evidenceResult?.ok === true;
      const status = ok ? "completed" : "blocked";
      const toolResult = focusaToolResult({
        ok,
        status: ok ? "completed" : "blocked",
        summary: `browser diagnostics intake → ${status} evidence=${evidenceRef}`,
        tool: "focusa_browser_diagnostics_intake",
        family: "workpoint",
        side_effects: sideEffects,
        evidence_refs: evidenceRefs,
        next_tools: [
          "focusa_active_object_resolve",
          "focusa_evidence_capture",
          "focusa_predict_record",
          "focusa_metacog_capture",
          "focusa_context_cognition",
        ],
        raw: {
          evidence: evidenceResult?.body,
          prediction: predictionResult?.body,
          metacog: metacogResult?.body,
          context_cognition_status: "captured",
        },
      });
      return {
        content: [
          {
            type: "text",
            text: `browser diagnostics intake → ${status} severity=${severityClassification.severity} alarm=${severityClassification.alarm} evidence=${evidenceRef}\nactive_object_hints=${activeObjectHints.slice(0, 4).join(",") || "none"}\ncontext_cognition_status=captured (Spec 100 \u00a716 mapping)\nnext_tools=${toolResult.next_tools.join(",")}`,
          },
        ],
        details: {
          ok,
          status,
          target_ref: targetRef,
          evidence_ref: evidenceRef,
          result: resultSummary,
          diagnostics_severity: severityClassification,
          active_object_hints: activeObjectHints,
          counts: {
            console: consoleItems.length,
            exceptions: exceptionItems.length,
            failed_requests: failedItems.length,
          },
          project_root: projectRoot,
          focusa_scope: compactApiEcho(focusaScope),
          scoped_workpoint_id: scopedWorkpointId || null,
          scoped_continuity_id: scopedContinuityId || null,
          tool_result_v1: toolResult,
          side_effects: sideEffects,
          evidence_refs: evidenceRefs,
          evidence_response: compactApiEcho(evidenceResult?.body),
          prediction_response: compactApiEcho(predictionResult?.body),
          metacog_response: compactApiEcho(metacogResult?.body),
          next_tools: toolResult.next_tools,
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_workpoint_checkpoint",
    label: "Workpoint Checkpoint",
    description:
      "Create a typed Focusa Workpoint checkpoint before compaction, resume, context overflow, model switch, or risky continuation. Use this instead of trusting raw transcript memory; Focusa becomes the canonical continuation source and returns an explicit next-step hint.",
    promptSnippet:
      "Before compact/resume/overflow: checkpoint typed workpoint; do not rely on transcript tail.",
    parameters: Type.Object({
      current_ask: Type.Optional(Type.String({ description: "Current operator ask or mission framing." })),
      work_item_id: Type.Optional(Type.String({ description: "Beads/work item id, e.g. focusa-a2w2.6." })),
      continuity_id: Type.Optional(
        Type.String({
          description: "Stable logical session/workstream id; defaults to this Pi session continuity id.",
        })
      ),
      checkpoint_reason: Type.Optional(
        Type.String({
          description:
            "manual|operator_checkpoint|before_compact|after_compact|context_overflow|session_resume|model_switch|fork",
        })
      ),
      mission: Type.String({ description: "Current mission/objective to preserve across compaction." }),
      target_objects: Type.Optional(
        Type.Array(Type.String(), {
          description: "Ontology/file/component/endpoint refs currently targeted.",
        })
      ),
      current_action: Type.Optional(
        Type.String({ description: "Typed action, e.g. patch_component_binding or resume_workpoint." })
      ),
      verified_evidence: Type.Optional(
        Type.Array(Type.String(), {
          description: "Short evidence refs/results already verified; use handles, not raw logs.",
        })
      ),
      blockers: Type.Optional(
        Type.Array(Type.String(), { description: "Open blockers or drift boundaries." })
      ),
      next_action: Type.String({ description: "Exact bounded next action to resume after compact/retry." }),
      do_not_drift: Type.Optional(
        Type.Array(Type.String(), { description: "Actions/areas the next agent must not drift into." })
      ),
      source_turn_id: Type.Optional(Type.String({ description: "Pi/source turn id for provenance." })),
      idempotency_key: Type.Optional(Type.String({ description: "Optional external idempotency key." })),
      canonical: Type.Optional(Type.Boolean({ description: "False only for degraded fallback packets." })),
      project_root: Type.Optional(
        Type.String({
          description: "Explicit safe project folder/root; defaults to Pi session cwd when that cwd is safe.",
        })
      ),
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
        return {
          content: [
            {
              type: "text",
              text: `workpoint checkpoint blocked → unsafe project_root (${reason}); cd into the specific project folder/repo or pass project_root explicitly.`,
            },
          ],
          details: {
            ok: false,
            status: "blocked",
            failure_class: "scope_mismatch",
            project_root: projectRoot,
            project_root_permission_posture: projectRootPermissionPosture(projectRoot),
            reason,
          },
        } as any;
      }
      const sessionIdentity = await buildFocusaSessionIdentity(
        projectRoot,
        p.checkpoint_reason === "before_compact" ? "compaction" : "manual",
        { continuityId: p.continuity_id, sessionId: p.session_id }
      );
      const clarity =
        p.canonical === false
          ? { ok: true, details: { skipped: true, reason: "noncanonical_workpoint" } }
          : await enforceTrajectoryClarityPrecondition(projectRoot, "workpoint checkpoint", {
              blockOperatorInput: true,
              continuityId: p.continuity_id,
              sessionId: p.session_id,
            });
      if (!clarity.ok)
        return {
          content: [
            { type: "text", text: clarity.text || "workpoint checkpoint blocked by trajectory clarity gate" },
          ],
          details: { ok: false, status: "blocked", ...clarity.details },
        } as any;
      const payload: any = {
        mission: p.mission,
        next_slice: [p.next_action, ...doNotDrift.map((d: string) => `DO_NOT_DRIFT: ${d}`)]
          .filter(Boolean)
          .join("\n"),
        work_item_id: p.work_item_id,
        continuity_id: p.continuity_id || ensureContinuityId(projectRoot),
        session_id: p.session_id || getAttachmentRuntime().sessionFrameKey,
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
      // First checkpoint bootstraps Workpoint authority before a Work Loop lease exists.
      // Sending a writer id without a fencing token makes the daemon classify
      // bootstrap as an expired/missing lease and creates a circular deadlock.
      const checkpointLease = await currentWorkLoopLease();
      const res = await focusaFetchDetailed("/workpoint/checkpoint", {
        method: "POST",
        headers: checkpointLease ? writerLeaseHeaders(localWriterId, checkpointLease) : {},
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
          next_step_hint:
            "Retry focusa_workpoint_checkpoint once after focusa_tool_doctor/resource_mode; do not treat timeout fallback as canonical.",
        });
        setActiveWorkpointPacket(fallback);
        setActiveWorkpointSummary(
          `${payload.mission || "Workpoint checkpoint"} (noncanonical timeout fallback)`
        );
        getAttachmentRuntime().lastWorkpointUpdate = Date.now();
        try {
          getAttachmentRuntime().pi?.appendEntry("focusa-workpoint-timeout-fallback", fallback);
        } catch {
          /* best effort */
        }
        persistState();
        return {
          content: [{ type: "text", text: timeoutPreservedText("workpoint checkpoint") }],
          details: {
            ok: false,
            status: "timeout_preserved",
            endpoint: "/workpoint/checkpoint",
            canonical: false,
            degraded: true,
            failure_class: "hot_path_timeout",
            project_root_permission_posture: projectRootPermissionPosture(projectRoot),
            request: compactApiEcho(payload),
            response: compactApiEcho(res.body),
            fallback_packet: compactFallbackPacket(fallback),
            next_tools: [
              "focusa_tool_doctor",
              "focusa_resource_mode",
              "focusa_workpoint_checkpoint",
              "focusa_workpoint_resume",
            ],
          },
        } as any;
      }
      const checkpointSummary = String(
        res.body?.rendered_summary || res.body?.checkpoint_summary?.one_line || ""
      );
      if (res.ok && res.body?.canonical === true && res.body?.workpoint) {
        const adoptedRoot = adoptWorkpointScopeForFrameRecovery(
          res.body.workpoint,
          "workpoint_checkpoint_tool"
        );
        if (adoptedRoot) {
          setActiveWorkpointSummary(checkpointSummary);
          getAttachmentRuntime().lastWorkpointUpdate = Date.now();
          persistState();
        }
      }
      const text = res.ok
        ? `workpoint checkpoint → ${summarizeWorkpointResponse(res.body)}${checkpointSummary ? `; ${checkpointSummary}` : ""}`
        : res.body?.status === "validation_rejected"
          ? `workpoint checkpoint validation_rejected → field=${String(res.body?.field || "unknown")} allowed=${Array.isArray(res.body?.allowed_values) ? res.body.allowed_values.join(",") : "unknown"} retry=${String(res.body?.retry_posture || "do_not_retry_unchanged")}`
          : `workpoint checkpoint blocked → ${explainWorkLoopResult(res, "checkpoint failed")}`;
      return {
        content: [{ type: "text", text }],
        details: {
          ok: res.ok,
          status: res.status,
          endpoint: "/workpoint/checkpoint",
          project_root_permission_posture: projectRootPermissionPosture(projectRoot),
          request: compactApiEcho(payload),
          response: compactApiEcho(res.body),
        },
      };
    },
  });

  pi.registerTool({
    name: "focusa_workpoint_link_evidence",
    label: "Workpoint Link Evidence",
    description:
      "Attach a stable evidence reference or verification result to the active canonical Workpoint.",
    promptSnippet: "Link proof/evidence to active Workpoint instead of keeping it only in transcript.",
    parameters: Type.Object({
      workpoint_id: Type.Optional(
        Type.String({ description: "Specific Workpoint id; omit to use active Workpoint." })
      ),
      target_ref: Type.String({ description: "Object/file/test/endpoint/work item the evidence verifies." }),
      result: Type.String({ description: "Bounded verification result summary." }),
      evidence_ref: Type.Optional(
        Type.String({ description: "Stable evidence handle, file path, test id, or artifact ref." })
      ),
      project_root: Type.Optional(
        Type.String({
          description:
            "Explicit safe project folder/root; use after compaction if Pi cwd is broad like /root.",
        })
      ),
      session_id: Type.Optional(
        Type.String({ description: "Optional temporal Pi session id; defaults to this Pi session key." })
      ),
      continuity_id: Type.Optional(
        Type.String({
          description: "Stable logical session/workstream id; defaults to this Pi continuity id.",
        })
      ),
      attach_to_workpoint: Type.Optional(
        Type.Boolean({ description: "Defaults true; false returns blocked/no-op guidance without linking." })
      ),
    }),
    async execute(_id, params) {
      const p = params as any;
      if (p.attach_to_workpoint === false) {
        const text = "workpoint evidence link → no_op attach_to_workpoint=false";
        return {
          content: [{ type: "text", text }],
          details: {
            ok: true,
            status: "no_op",
            reason: "attach_to_workpoint=false",
            project_root_permission_posture: p.project_root
              ? projectRootPermissionPosture(await resolveFocusaToolProjectRoot(p.project_root))
              : null,
          },
        } as any;
      }
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      const clarity = await enforceTrajectoryClarityPrecondition(projectRoot, "workpoint evidence link", {
        blockOperatorInput: false,
        continuityId: p.continuity_id,
        sessionId: p.session_id,
      });
      if (!clarity.ok) {
        const degraded = evidenceClarityFallbackResult("workpoint evidence link", p, projectRoot, clarity);
        if (degraded) return degraded;
        return {
          content: [
            {
              type: "text",
              text: `${clarity.text || "workpoint evidence link blocked by trajectory clarity gate"}. Why: trajectory clarity is required before linking proof to canonical Workpoint state; follow next_tools/recovery_hint instead of retrying blindly.`,
            },
          ],
          details: {
            ok: false,
            status: "blocked",
            why: "trajectory clarity is required before canonical evidence linkage",
            project_root_permission_posture: projectRootPermissionPosture(projectRoot),
            ...clarity.details,
          },
        } as any;
      }
      // Explicit project/continuity/Workpoint scope is sufficient for durable evidence linkage.
      const res = await focusaFetchDetailed("/workpoint/evidence/link", {
        method: "POST",
        body: JSON.stringify({
          workpoint_id: p.workpoint_id,
          target_ref: p.target_ref,
          result: p.result,
          evidence_ref: p.evidence_ref,
          session_identity: await buildFocusaSessionIdentity(projectRoot, "manual", {
            continuityId: p.continuity_id,
            sessionId: p.session_id,
          }),
          trajectory_clarity_precondition: clarity.details,
        }),
      });
      const text = res.ok
        ? `workpoint evidence link → status=${String(res.body?.status || "accepted")} id=${String(res.body?.workpoint_id || "none")}`
        : `workpoint evidence link blocked → ${explainWorkLoopResult(res, "link failed")}`;
      return {
        content: [{ type: "text", text }],
        details: {
          ok: res.ok,
          status: String(res.status),
          reason: res.ok ? "linked" : "blocked",
          endpoint: "/workpoint/evidence/link",
          project_root_permission_posture: projectRootPermissionPosture(projectRoot),
          response: compactApiEcho(res.body),
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_workpoint_resume",
    label: "Workpoint Resume",
    description:
      "Fetch the active Focusa WorkpointResumePacket after compaction, resume, context overflow, model switch, or uncertainty. Use this instead of guessing from transcript tail; output includes canonical/degraded status, warnings, and the exact next action.",
    promptSnippet: "After compact/resume/overflow: fetch WorkpointResumePacket and continue from it.",
    parameters: Type.Object({
      workpoint_id: Type.Optional(
        Type.String({ description: "Specific workpoint id; omit to use active workpoint." })
      ),
      continuity_id: Type.Optional(
        Type.String({
          description: "Stable logical session/workstream id; defaults to this Pi session continuity id.",
        })
      ),
      session_id: Type.Optional(
        Type.String({ description: "Optional temporal Pi session id; defaults to this Pi session key." })
      ),
      mode: Type.Optional(Type.String({ description: "compact_prompt|full_json|operator_summary" })),
      project_root: Type.Optional(
        Type.String({
          description: "Explicit safe project folder/root; defaults to Pi session cwd when that cwd is safe.",
        })
      ),
      current_ask: Type.Optional(
        Type.String({
          description:
            "Optional latest operator ask used to compute current-action authority; defaults to Pi current ask.",
        })
      ),
    }),
    promptGuidelines: [
      "Use immediately after compaction or session resume before choosing next work.",
      "If not_found, create a checkpoint before continuing important work.",
      "If canonical=false, state degraded status and avoid treating it as canonical truth.",
    ],
    async execute(_id, params) {
      const p = params as {
        workpoint_id?: string;
        continuity_id?: string;
        session_id?: string;
        mode?: string;
        project_root?: string;
        current_ask?: string;
      };
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      if (!isProjectRootAuthoritySafe(projectRoot)) {
        const reason = projectRootAuthorityFailure(projectRoot) || "unsafe_project_root";
        return {
          content: [
            {
              type: "text",
              text: `workpoint resume blocked → unsafe project_root (${reason}); ignore stale packets and follow latest operator instruction.`,
            },
          ],
          details: {
            ok: false,
            status: "blocked",
            failure_class: "scope_mismatch",
            project_root: projectRoot,
            reason,
            next_tools: ["focusa_project_identity", "focusa_tool_doctor"],
          },
        } as any;
      }
      const payload = {
        workpoint_id: p.workpoint_id,
        mode: p.mode || "compact_prompt",
        continuity_id: p.continuity_id || ensureContinuityId(projectRoot),
        session_id: p.session_id || getAttachmentRuntime().sessionFrameKey,
        project_root: projectRoot,
        current_ask: p.current_ask || getAttachmentRuntime().currentAsk?.text || "",
        session_identity: await buildFocusaSessionIdentity(projectRoot, "session_switch", {
          continuityId: p.continuity_id,
          sessionId: p.session_id,
        }),
      };
      const res = await focusaFetchDetailed("/workpoint/resume", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      const rejected = res.body?.status === "rejected_scope_mismatch";
      const recovery = scopeRecoveryContext(
        res.body || {},
        projectRoot,
        payload.continuity_id,
        "workpoint_resume"
      );
      if (!res.ok && res.body?.failure_class === "hot_path_timeout") {
        const fallback = stampWorkpointPacketForCurrentPiSession({
          ...(getActiveWorkpointPacket() || {}),
          status: "timeout_preserved",
          canonical: false,
          degraded: true,
          failure_class: "hot_path_timeout",
          project_root: projectRoot,
          continuity_id: payload.continuity_id,
          session_id: payload.session_id,
          mission:
            getActiveWorkpointPacket()?.mission ||
            getActiveWorkpointSummary() ||
            "Workpoint resume timed out before a canonical packet was returned",
          next_slice:
            getActiveWorkpointPacket()?.next_slice ||
            "Retry focusa_workpoint_resume after focusa_tool_doctor/resource_mode, or create a fresh focusa_workpoint_checkpoint from current operator/repo context.",
          preserved_at: new Date().toISOString(),
          next_step_hint:
            "Retry focusa_workpoint_resume after focusa_tool_doctor/resource_mode; if no canonical packet exists, checkpoint the current mission before treating state as canonical.",
        });
        setActiveWorkpointPacket(fallback);
        setActiveWorkpointSummary(
          `${String(fallback.mission || "Workpoint resume")} (noncanonical timeout fallback)`
        );
        getAttachmentRuntime().lastWorkpointUpdate = Date.now();
        try {
          getAttachmentRuntime().pi?.appendEntry("focusa-workpoint-timeout-fallback", fallback);
        } catch {
          /* best effort */
        }
        persistState();
        return {
          content: [{ type: "text", text: timeoutPreservedText("workpoint resume", "local fallback") }],
          details: {
            ok: false,
            status: "timeout_preserved",
            endpoint: "/workpoint/resume",
            canonical: false,
            degraded: true,
            failure_class: "hot_path_timeout",
            fallback_packet: compactFallbackPacket(fallback),
            scope_recovery_context: compactApiEcho(recovery?.details || null),
            request: compactApiEcho(payload),
            response: compactApiEcho(res.body),
            next_tools: [
              "focusa_tool_doctor",
              "focusa_resource_mode",
              "focusa_workpoint_resume",
              "focusa_traverse",
            ],
          },
        } as any;
      }
      const text =
        res.ok && !rejected
          ? [
              `workpoint resume → ${summarizeWorkpointResponse(res.body)}\n${String(res.body?.rendered_summary || "")}`.trim(),
              recovery?.text,
            ]
              .filter(Boolean)
              .join("\n")
          : rejected
            ? [
                `workpoint resume rejected: project_root mismatch (saved scope ≠ current scope).`,
                `safe_recovery: run focusa_project_verify project_root=<current> then focusa_workpoint_checkpoint in the current project; do NOT retry unchanged.`,
                `action_authority_for_current_ask=false; no executable next step until scope verified.`,
                recovery?.text || "",
              ]
                .filter(Boolean)
                .join("\n")
            : [
                `workpoint resume unavailable → ${explainWorkLoopResult(res, "resume failed")}`,
                recovery?.text,
              ]
                .filter(Boolean)
                .join("\n");
      const v2 = res.body?.resume_packet_v2 || null;
      const canonical = res.body?.canonical === true;
      const actionAuthority =
        res.body?.action_authority_for_current_ask !== false &&
        v2?.action_authority_for_current_ask !== false;
      const matchesCurrentAskScope =
        res.body?.matches_current_ask_scope !== false && v2?.matches_current_ask_scope !== false;
      const scopeConflictReason = String(
        res.body?.scope_conflict_reason || v2?.scope_conflict_reason || "none"
      );
      if (res.ok && canonical && actionAuthority && matchesCurrentAskScope) {
        const candidate = normalizeWorkpointResumePacketEnvelope(res.body);
        const adoptedRoot = adoptWorkpointScopeForFrameRecovery(candidate, "workpoint_resume_tool", {
          projectRoot: params.project_root || "",
          continuityId: params.continuity_id || "",
          allowSessionTransfer: Boolean(params.project_root && params.continuity_id),
        });
        if (adoptedRoot) {
          setActiveWorkpointSummary(String(res.body?.rendered_summary || v2?.rendered_summary || ""));
          getAttachmentRuntime().lastWorkpointUpdate = Date.now();
          persistState();
        }
      }
      // FOCUSA_FIX-r4n9: When authority is suppressed, recoveryPacket blocks execution
      // and the Focus Slice in state.ts cuts next_action to force verification
      const authoritySuppressed = canonical && !actionAuthority;
      const recoveryPacket = authoritySuppressed
        ? {
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
            safe_next_action:
              "verify/rebind the current operator-indicated project, then checkpoint/resume a Workpoint in that corrected scope before file/API action",
            next_tools: [
              "focusa_project_verify",
              "focusa_project_identity",
              "focusa_workpoint_checkpoint",
              "focusa_workpoint_resume",
            ],
            do_not_use: [
              "saved_scope_as_current_action_authority",
              "transcript_tail_as_authority",
              "cross_project_packets",
            ],
          }
        : canonical
          ? null
          : {
              status: "recovery_required",
              authority: "operator_and_current_project_context",
              canonical: false,
              degraded: true,
              reason:
                res.body?.failure_class ||
                res.body?.status ||
                (rejected ? "scope_mismatch" : "no_canonical_workpoint_packet"),
              project_root: projectRoot,
              continuity_id: payload.continuity_id,
              safe_next_action:
                "create a fresh focusa_workpoint_checkpoint from the current operator ask, project root, target objects, verified evidence, blockers, and exact next action before treating continuation state as canonical",
              next_tools: [
                "focusa_project_identity",
                "focusa_trajectory_view",
                "focusa_workpoint_checkpoint",
                "focusa_workpoint_resume",
              ],
              do_not_use: [
                "transcript_tail_as_authority",
                "cross_project_packets",
                "noncanonical_resume_as_truth",
              ],
            };
      const baseToolResult = res.body?.details?.tool_result_v1 ||
        v2?.details?.tool_result_v1 || {
          ok: res.ok && !rejected && canonical,
          status: res.ok ? String(res.body?.status || "completed") : String(res.status),
          canonical,
          degraded: res.body?.degraded === true || !canonical || rejected,
          failure_class:
            res.body?.failure_class || (rejected ? "scope_mismatch" : canonical ? null : "frame_unavailable"),
          retry: {
            safe: res.ok && !rejected,
            posture: canonical ? "safe_retry" : "check_side_effects_first",
          },
          side_effects: [],
          evidence_refs: [],
          next_tools: recoveryPacket?.next_tools ||
            res.body?.next_tools || ["focusa_workpoint_resume", "focusa_trajectory_view", "focusa_traverse"],
        };
      const toolResult = authoritySuppressed
        ? {
            ...baseToolResult,
            ok: false,
            degraded: true,
            failure_class: baseToolResult.failure_class || "scope_conflict",
            canonical_for_saved_scope: true,
            matches_current_ask_scope: matchesCurrentAskScope,
            action_authority_for_current_ask: false,
            scope_conflict_reason: scopeConflictReason,
            retry: { safe: false, posture: "do_not_retry_unchanged" },
            next_tools: recoveryPacket?.next_tools || baseToolResult.next_tools,
          }
        : baseToolResult;
      const authorityText = authoritySuppressed
        ? `\naction authority suppressed → ${scopeConflictReason}; saved Workpoint remains canonical_for_saved_scope=true.`
        : "";
      const recoveryText = recoveryPacket ? `\nrecovery → ${recoveryPacket.safe_next_action}` : "";
      return {
        content: [{ type: "text", text: `${text}${authorityText}${recoveryText}` }],
        details: {
          ok: toolResult.ok,
          status: res.status,
          endpoint: "/workpoint/resume",
          canonical,
          canonical_for_saved_scope: canonical,
          matches_current_ask_scope: matchesCurrentAskScope,
          action_authority_for_current_ask: actionAuthority,
          scope_conflict_reason: scopeConflictReason,
          degraded: res.body?.degraded === true || !canonical || authoritySuppressed,
          failure_class: toolResult.failure_class || null,
          recovery_packet: compactApiEcho(recoveryPacket),
          scope_recovery_context: compactApiEcho(recovery?.details || null),
          resume_packet_v2: compactApiEcho(v2),
          rendered_summary: String(res.body?.rendered_summary || "").slice(0, 240),
          tool_result_v1: toolResult,
          next_tools: (
            toolResult.next_tools ||
            recoveryPacket?.next_tools ||
            res.body?.next_tools || ["focusa_workpoint_resume", "focusa_trajectory_view", "focusa_traverse"]
          ).slice(0, 4),
          request: compactApiEcho(payload),
          response: compactApiEcho(res.body),
        },
      };
    },
  });

  // ── Spec80 LLM-native tree/metacog tools ─────────────────────────────────

  function spec80ErrorCode(result: { ok: boolean; status: number; body: any | null }): string {
    if (result.ok) return "OK";
    const bodyCode = String(
      result.body?.code || result.body?.failure_class || result.body?.error || ""
    ).trim();
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

  function metacogQualityGate(input: {
    content?: string;
    rationale?: string;
    confidence?: number;
    evidence_refs?: string[];
  }) {
    const evidenceRefs = input.evidence_refs || [];
    const contentWords = String(input.content || "")
      .trim()
      .split(/\s+/)
      .filter(Boolean).length;
    let score = 0;
    if (contentWords >= 8) score += 0.35;
    if (String(input.rationale || "").trim().length >= 20) score += 0.25;
    if ((input.confidence ?? 0) >= 0.5) score += 0.15;
    if (evidenceRefs.length > 0) score += 0.25;
    const passed = score >= 0.6;
    return {
      passed,
      score: Number(score.toFixed(2)),
      evidence_refs: evidenceRefs,
      recommendation: passed ? "eligible_for_retrieval" : "add rationale/evidence before promotion",
    };
  }

  function spec80Result(
    tool: string,
    endpoint: string,
    request: Record<string, any>,
    result: { ok: boolean; status: number; body: any | null },
    successText: string,
    fallbackText: string,
    template?: {
      kind: PiToolTemplateKind;
      ids?: Array<{ label: string; value: unknown }>;
      fields?: Array<{ label: string; value: unknown }>;
      failureClass?: string | null;
      nextTools?: string[];
      note?: string | null;
    }
  ) {
    let text: string;
    if (result.ok && result.body) {
      text = template
        ? piToolText({
            kind: template.kind,
            tool,
            summary: successText,
            ids: (template.ids || []).map((i) => ({ label: i.label, value: i.value as string | number })),
            fields: (template.fields || []).map((f) => ({
              label: f.label,
              value: f.value as string | number | null | undefined,
            })),
            failureClass: template.failureClass ?? null,
            nextTools: template.nextTools || [],
            note: template.note ?? null,
          })
        : successText;
    } else {
      text = `${fallbackText} → ${explainWorkLoopResult(result, "ok")}`;
    }
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
        human_readable: typeof result.body?.human_readable === "string" ? result.body.human_readable : null,
        quality_gate: tool.startsWith("focusa_metacog_") ? metacogQualityGate(request) : undefined,
        evidence_refs: Array.isArray(request.evidence_refs) ? request.evidence_refs : [],
        suggested_metrics: tool.startsWith("focusa_metacog_")
          ? ["retrieval_reuse", "promotion_precision", "failure_recurrence"]
          : undefined,
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
    template?: {
      kind: PiToolTemplateKind;
      ids?: Array<{ label: string; value: unknown }>;
      fields?: Array<{ label: string; value: unknown }>;
      failureClass?: string | null;
      nextTools?: string[];
      note?: string | null;
    }
  ) {
    const result = { ok, status, body: response ?? null };
    return spec80Result(tool, endpoint, request, result, successText, fallbackText, template);
  }

  async function callSpec80Tool(
    tool: string,
    endpoint: string,
    request: Record<string, any>,
    opts: { method?: "GET" | "POST"; writer?: boolean } = {}
  ): Promise<{ ok: boolean; status: number; body: any | null; writerId?: string }> {
    const method = opts.method || "POST";
    const writerId = opts.writer ? await preferredWriterId() : undefined;
    const writerLease = writerId ? await currentWorkLoopLease() : null;
    const requestSessionId = String(request.session_id || "").trim();
    const identity: any = getLastProjectIdentity() || {};
    const requestProjectRoot = normalizeProjectRoot(
      resolveCanonicalMarkerProjectRoot(process.cwd()) ||
        identity.canonical_parent_root ||
        identity.project_root ||
        getSessionCwd()
    );
    const requestContinuityId = String(getContinuityId() || "").trim();
    const requestScopeHeaders: Record<string, string> =
      isProjectRootAuthoritySafe(requestProjectRoot) && requestContinuityId
        ? {
            "x-scope-project-root": requestProjectRoot,
            "x-scope-continuity-id": requestContinuityId,
          }
        : {};
    const req: RequestInit = {
      method,
      headers: {
        ...(writerId ? writerLeaseHeaders(writerId, writerLease) : {}),
        ...requestScopeHeaders,
        ...(requestSessionId ? { "x-scope-session-id": requestSessionId } : {}),
      },
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
    expectedSchema?: {
      required_fields: string[];
      field_types?: Record<string, string>;
      example?: Record<string, any>;
    }
  ) {
    const body: any = { code, error };
    if (expectedSchema) {
      body.expected_schema = expectedSchema;
      const rejectedField =
        expectedSchema.required_fields.find(
          (field) =>
            !request || request[field] === undefined || error.toLowerCase().includes(field.toLowerCase())
        ) || expectedSchema.required_fields[0];
      const rejectedValue = request?.[rejectedField];
      body.validation_errors = [
        {
          field: rejectedField,
          code: rejectedValue === undefined ? "required" : "invalid",
          message: error,
          ...(rejectedValue === undefined || typeof rejectedValue === "object"
            ? {}
            : { rejected_value: String(rejectedValue).slice(0, 160) }),
        },
      ];
      body.rejected_field = rejectedField;
      if (rejectedValue !== undefined && typeof rejectedValue !== "object")
        body.rejected_value = String(rejectedValue).slice(0, 160);
      body.recovery_hint = `Provide ${rejectedField} using the returned expected_schema; do not retry unchanged.`;
    }
    const result = spec80Result(
      tool,
      endpoint,
      request,
      { ok: false, status: 422, body },
      `${fallbackText}: ok`,
      fallbackText
    );
    // Auto-inject recovery_hint + misuse_hint when not already present
    if (result?.details?.tool_result_v1) {
      const tr = result.details.tool_result_v1;
      if (!tr.recovery_hint)
        tr.recovery_hint = `Inspect the tool's expected_schema (returned with this error). Run focusa_traverse surface=tool_registry to see the full parameter schema for ${tool}.`;
      if (!tr.misuse_hint)
        tr.misuse_hint = `Tool parameter shape mismatch. The error includes expected_schema with required fields and types — fix the input shape, do not retry unchanged.`;
      if (!tr.next_tools || tr.next_tools.length === 0)
        tr.next_tools = ["focusa_traverse", "focusa_tool_doctor"];
      if (!tr.failure_class) tr.failure_class = "schema_invalid";
      if (!tr.retry) tr.retry = { posture: "do_not_retry_unchanged", safe: false };
    }
    return result;
  }

  function validateRequiredString(
    name: string,
    value: unknown,
    maxLength: number,
    opts: { pattern?: RegExp } = {}
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
    opts: { pattern?: RegExp } = {}
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
    opts: { maxItems: number; itemMaxLength: number; pattern?: RegExp }
  ): { ok: true; value: string[] } | { ok: false; error: string } {
    if (value === undefined || value === null) return { ok: true, value: [] };
    if (!Array.isArray(value)) return { ok: false, error: `${name} must be an array` };
    if (value.length > opts.maxItems)
      return { ok: false, error: `${name} has too many items (max ${opts.maxItems})` };
    const normalized: string[] = [];
    for (const raw of value) {
      if (typeof raw !== "string") return { ok: false, error: `${name} items must be strings` };
      const item = raw.trim();
      if (!item) return { ok: false, error: `${name} items must not be blank` };
      if (item.length > opts.itemMaxLength)
        return { ok: false, error: `${name} item too long (max ${opts.itemMaxLength})` };
      if (opts.pattern && !opts.pattern.test(item))
        return { ok: false, error: `${name} item has invalid format` };
      normalized.push(item);
    }
    return { ok: true, value: normalized };
  }

  function validateNoExtraKeys(
    tool: string,
    params: unknown,
    allowedKeys: string[]
  ): { ok: true; value: Record<string, any> } | { ok: false; error: string } {
    if (!params || typeof params !== "object" || Array.isArray(params)) {
      return {
        ok: false,
        error: `${tool} params must be an object with required keys: ${allowedKeys.join(", ")}. Example: { ${allowedKeys.map((k) => `"${k}": "..."`).join(", ")} }`,
      };
    }
    const record = params as Record<string, any>;
    const extras = Object.keys(record).filter((key) => !allowedKeys.includes(key));
    if (extras.length > 0) {
      return {
        ok: false,
        error: `unexpected parameter(s): ${extras.join(", ")}. Allowed: ${allowedKeys.join(", ")}`,
      };
    }
    return { ok: true, value: record };
  }

  function strictObject(properties: Record<string, any>) {
    // NOTE: previously used { additionalProperties: false } here, but the
    // pi-coding-agent runtime (TypeBox 0.34 + AJV) was rejecting valid params
    // for tools that declared string params (focusa_bloatgaurd_domain,
    // focusa_dxux_requirement, focusa_call_stack_verify, etc.) with the error
    // "name: must have required properties name". The runtime already enforces
    // extra-key rejection via validateNoExtraKeys, so additionalProperties is
    // safe to drop here.
    return Type.Object(properties);
  }

  function summarizeValue(value: unknown): string {
    if (value === null || value === undefined) return "";
    if (typeof value === "string") return value.length > 160 ? `${value.slice(0, 157)}…` : value;
    if (typeof value === "number" || typeof value === "boolean") return String(value);
    if (Array.isArray(value)) return `[${value.slice(0, 4).map(summarizeValue).filter(Boolean).join(", ")}]`;
    if (typeof value === "object") {
      const record = value as Record<string, any>;
      const label =
        record.node_id || record.workpoint_id || record.id || record.anchor || record.label || record.title;
      const kind = record.node_type || record.kind || record.status;
      const payload =
        record.payload && typeof record.payload === "object" ? (record.payload as Record<string, any>) : null;
      const summary =
        record.summary ||
        record.mission ||
        record.next_slice ||
        record.goal ||
        record.content_ref ||
        payload?.content_ref ||
        payload?.summary ||
        payload?.reason ||
        record.created_at;
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
    return items
      .slice(0, limit)
      .map((item, index) => {
        const record = item as Record<string, any>;
        const data =
          record && typeof record === "object" && record.data && typeof record.data === "object"
            ? (record.data as Record<string, any>)
            : record;
        const anchor =
          record?.anchor ||
          record?.label ||
          data?.node_id ||
          data?.event_id ||
          data?.workpoint_id ||
          data?.id ||
          data?.tool ||
          data?.name ||
          "";
        const kind =
          record?.kind ||
          record?.label ||
          data?.node_type ||
          data?.event_type ||
          data?.status ||
          data?.kind ||
          data?.type ||
          "";
        const payload =
          data?.payload && typeof data.payload === "object" ? (data.payload as Record<string, any>) : null;
        const summary =
          record?.summary ||
          data?.summary ||
          data?.mission ||
          data?.next_slice ||
          data?.goal ||
          data?.content_ref ||
          data?.message ||
          data?.result ||
          payload?.content_ref ||
          payload?.summary ||
          payload?.reason ||
          payload?.message ||
          data?.timestamp ||
          data?.created_at ||
          (data && Object.keys(data).length ? summarizeValue(data) : "");
        const parts = [anchor, kind, summary]
          .map(summarizeValue)
          .filter(Boolean)
          .filter((value, partIndex, values) => values.indexOf(value) === partIndex);
        return `${index + 1}. ${parts.join(" | ") || "[no projected fields]"}`;
      })
      .join("\n");
  }

  function boolLabel(value: unknown): string {
    return value ? "yes" : "no";
  }

  pi.registerTool({
    name: "focusa_tree_head",
    label: "Tree Head",
    description:
      "Best safe starting point for lineage work. Use first when you need current branch/head context before path, snapshot, diff, or restore work.",
    parameters: strictObject({
      session_id: Type.Optional(
        Type.String({
          maxLength: SPEC81_LIMITS.sessionId,
          pattern: SPEC81_ID_PATTERN,
          description: "Optional session id scoping hint.",
        })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_tree_head", params, ["session_id"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_tree_head",
          "/v1/lineage/head",
          params as Record<string, any>,
          "tree head",
          keyCheck.error
        );
      }
      const sessionIdCheck = validateOptionalString(
        "session_id",
        keyCheck.value.session_id,
        SPEC81_LIMITS.sessionId,
        { pattern: SPEC81_ID_RE }
      );
      if (!sessionIdCheck.ok) {
        return spec80ValidationResult(
          "focusa_tree_head",
          "/v1/lineage/head",
          params as Record<string, any>,
          "tree head",
          sessionIdCheck.error
        );
      }
      const activeSessionId = String(getSessionFrameKey() || "").trim();
      const requestedSessionId = String(sessionIdCheck.value || "").trim();
      if (requestedSessionId && activeSessionId && requestedSessionId !== activeSessionId) {
        return spec80ValidationResult(
          "focusa_tree_head",
          "/v1/lineage/head",
          params as Record<string, any>,
          "tree head",
          "session_id must match the active native Pi session; foreign lineage is quarantined",
          "SCOPE_MISMATCH"
        );
      }
      const session_id = requestedSessionId || activeSessionId;
      if (!session_id) {
        return spec80ValidationResult(
          "focusa_tree_head",
          "/v1/lineage/head",
          params as Record<string, any>,
          "tree head",
          "active Pi session_id is required; global lineage fallback is prohibited"
        );
      }
      const query = `?session_id=${encodeURIComponent(session_id)}`;
      const req = { session_id };
      const res = await callSpec80Tool("focusa_tree_head", `/lineage/head${query}`, req, { method: "GET" });
      const returnedSession = String(res.body?.session_id || "").trim();
      if (res.ok && returnedSession !== session_id) {
        return spec80ValidationResult(
          "focusa_tree_head",
          "/v1/lineage/head",
          req,
          "tree head",
          "lineage response session scope mismatch"
        );
      }
      const head = String(res.body?.head || "unknown");
      const branch = String(res.body?.branch_id || "unknown");
      const session = returnedSession || session_id;
      return spec80Result(
        "focusa_tree_head",
        "/v1/lineage/head",
        req,
        res,
        `tree head: ${head}\nbranch=${branch} session=${session}\nnext_tools=focusa_tree_path,focusa_tree_snapshot_state`,
        "tree head"
      );
    },
  });

  pi.registerTool({
    name: "focusa_tree_path",
    label: "Tree Path",
    description:
      "Safe ancestry lookup. Use when branch position or lineage depth matters and you do not want to infer it from prior turns.",
    parameters: strictObject({
      clt_node_id: Type.String({
        minLength: 1,
        maxLength: SPEC81_LIMITS.id,
        pattern: SPEC81_ID_PATTERN,
        description: "CLT node id.",
      }),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_tree_path", params, ["clt_node_id"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_tree_path",
          "/v1/lineage/path/{clt_node_id}",
          params as Record<string, any>,
          "tree path",
          keyCheck.error
        );
      }
      const nodeIdCheck = validateRequiredString(
        "clt_node_id",
        keyCheck.value.clt_node_id,
        SPEC81_LIMITS.id,
        { pattern: SPEC81_ID_RE }
      );
      if (!nodeIdCheck.ok) {
        return spec80ValidationResult(
          "focusa_tree_path",
          "/v1/lineage/path/{clt_node_id}",
          params as Record<string, any>,
          "tree path",
          nodeIdCheck.error
        );
      }
      const nodeId = nodeIdCheck.value;
      const res = await callSpec80Tool(
        "focusa_tree_path",
        `/lineage/path/${encodeURIComponent(nodeId)}`,
        { clt_node_id: nodeId },
        { method: "GET" }
      );
      const depth = Number(res.body?.depth || 0);
      const pathItems = Array.isArray(res.body?.path) ? res.body.path : [];
      return spec80Result(
        "focusa_tree_path",
        "/v1/lineage/path/{clt_node_id}",
        { clt_node_id: nodeId },
        res,
        `tree path: depth=${depth} nodes=${pathItems.length}\npath=${summarizeArray(pathItems, 5)}\nnext_tools=focusa_tree_snapshot_state,focusa_tree_diff_context`,
        "tree path"
      );
    },
  });

  pi.registerTool({
    name: "focusa_tree_snapshot_state",
    label: "Tree Snapshot State",
    description:
      "Create a recoverable checkpoint before risky work or comparisons. Best write tool for saving current state with a reason.",
    parameters: strictObject({
      clt_node_id: Type.Optional(
        Type.String({
          maxLength: SPEC81_LIMITS.id,
          pattern: SPEC81_ID_PATTERN,
          description: "Optional CLT node id. Defaults to current head.",
        })
      ),
      snapshot_reason: Type.Optional(
        Type.String({ maxLength: SPEC81_LIMITS.snapshotReason, description: "Reason label for snapshot." })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_tree_snapshot_state", params, [
        "clt_node_id",
        "snapshot_reason",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_tree_snapshot_state",
          "/v1/focus/snapshots",
          params as Record<string, any>,
          "tree snapshot",
          keyCheck.error
        );
      }
      const raw = keyCheck.value as { clt_node_id?: string; snapshot_reason?: string };
      const nodeCheck = validateOptionalString("clt_node_id", raw.clt_node_id, SPEC81_LIMITS.id, {
        pattern: SPEC81_ID_RE,
      });
      if (!nodeCheck.ok) {
        return spec80ValidationResult(
          "focusa_tree_snapshot_state",
          "/v1/focus/snapshots",
          raw as Record<string, any>,
          "tree snapshot",
          nodeCheck.error
        );
      }
      const reasonCheck = validateOptionalString(
        "snapshot_reason",
        raw.snapshot_reason,
        SPEC81_LIMITS.snapshotReason
      );
      if (!reasonCheck.ok) {
        return spec80ValidationResult(
          "focusa_tree_snapshot_state",
          "/v1/focus/snapshots",
          raw as Record<string, any>,
          "tree snapshot",
          reasonCheck.error
        );
      }
      const req = { clt_node_id: nodeCheck.value || null, snapshot_reason: reasonCheck.value || null };
      const res = await callSpec80Tool("focusa_tree_snapshot_state", "/focus/snapshots", req, {
        method: "POST",
        writer: true,
      });
      return spec80Result(
        "focusa_tree_snapshot_state",
        "/v1/focus/snapshots",
        { ...req, writer_id: res.writerId || null },
        res,
        `tree snapshot: ${String(res.body?.snapshot_id || "created")}\nclt_node=${String(res.body?.clt_node_id || req.clt_node_id || "current")} created_at=${String(res.body?.created_at || "unknown")}\nnext_tools=focusa_tree_diff_context,focusa_tree_restore_state`,
        "tree snapshot"
      );
    },
  });

  pi.registerTool({
    name: "focusa_tree_restore_state",
    label: "Tree Restore State",
    description:
      "Restore a saved checkpoint when you need rollback or exact/merge recovery. State-changing tool.",
    parameters: strictObject({
      snapshot_id: Type.String({
        minLength: 1,
        maxLength: SPEC81_LIMITS.id,
        pattern: SPEC81_ID_PATTERN,
        description: "Snapshot id to restore.",
      }),
      restore_mode: Type.Optional(
        Type.Union([Type.Literal("exact"), Type.Literal("merge")], {
          description: "Restore mode: exact|merge",
        })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_tree_restore_state", params, [
        "snapshot_id",
        "restore_mode",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_tree_restore_state",
          "/v1/focus/snapshots/restore",
          params as Record<string, any>,
          "tree restore",
          keyCheck.error
        );
      }
      const raw = keyCheck.value as { snapshot_id: string; restore_mode?: string };
      const sidCheck = validateRequiredString("snapshot_id", raw.snapshot_id, SPEC81_LIMITS.id, {
        pattern: SPEC81_ID_RE,
      });
      if (!sidCheck.ok) {
        return spec80ValidationResult(
          "focusa_tree_restore_state",
          "/v1/focus/snapshots/restore",
          raw as Record<string, any>,
          "tree restore",
          sidCheck.error
        );
      }
      const mode = String(raw.restore_mode || "exact")
        .trim()
        .toLowerCase();
      if (mode !== "exact" && mode !== "merge") {
        return spec80ValidationResult(
          "focusa_tree_restore_state",
          "/v1/focus/snapshots/restore",
          { snapshot_id: sidCheck.value, restore_mode: mode },
          "tree restore",
          "restore_mode must be exact|merge",
          "INVALID_REQUEST"
        );
      }
      const req = { snapshot_id: sidCheck.value, restore_mode: mode };
      const res = await callSpec80Tool("focusa_tree_restore_state", "/focus/snapshots/restore", req, {
        method: "POST",
        writer: true,
      });
      const conflicts = Array.isArray(res.body?.conflicts) ? res.body.conflicts.length : 0;
      return spec80Result(
        "focusa_tree_restore_state",
        "/v1/focus/snapshots/restore",
        { ...req, writer_id: res.writerId || null },
        res,
        `tree restore: status=${String(res.body?.status || "ok")} snapshot=${String(res.body?.snapshot_id || req.snapshot_id)}\nmode=${mode} conflicts=${conflicts}\nnext_tools=focusa_tree_head,focusa_tree_path`,
        "tree restore"
      );
    },
  });

  pi.registerTool({
    name: "focusa_tree_diff_context",
    label: "Tree Diff Context",
    description:
      "Best safe compare tool for snapshots. Use this instead of guessing what changed across checkpoints.",
    parameters: strictObject({
      from_snapshot_id: Type.String({
        minLength: 1,
        maxLength: SPEC81_LIMITS.id,
        pattern: SPEC81_ID_PATTERN,
        description: "Source snapshot id.",
      }),
      to_snapshot_id: Type.String({
        minLength: 1,
        maxLength: SPEC81_LIMITS.id,
        pattern: SPEC81_ID_PATTERN,
        description: "Target snapshot id.",
      }),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_tree_diff_context", params, [
        "from_snapshot_id",
        "to_snapshot_id",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_tree_diff_context",
          "/v1/focus/snapshots/diff",
          params as Record<string, any>,
          "tree diff",
          keyCheck.error
        );
      }
      const raw = keyCheck.value as { from_snapshot_id: string; to_snapshot_id: string };
      const fromCheck = validateRequiredString("from_snapshot_id", raw.from_snapshot_id, SPEC81_LIMITS.id, {
        pattern: SPEC81_ID_RE,
      });
      if (!fromCheck.ok) {
        return spec80ValidationResult(
          "focusa_tree_diff_context",
          "/v1/focus/snapshots/diff",
          raw as Record<string, any>,
          "tree diff",
          fromCheck.error
        );
      }
      const toCheck = validateRequiredString("to_snapshot_id", raw.to_snapshot_id, SPEC81_LIMITS.id, {
        pattern: SPEC81_ID_RE,
      });
      if (!toCheck.ok) {
        return spec80ValidationResult(
          "focusa_tree_diff_context",
          "/v1/focus/snapshots/diff",
          raw as Record<string, any>,
          "tree diff",
          toCheck.error
        );
      }
      const req = { from_snapshot_id: fromCheck.value, to_snapshot_id: toCheck.value };
      const res = await callSpec80Tool("focusa_tree_diff_context", "/focus/snapshots/diff", req, {
        method: "POST",
      });
      return spec80Result(
        "focusa_tree_diff_context",
        "/v1/focus/snapshots/diff",
        req,
        res,
        `tree diff: changed=${boolLabel(res.body?.checksum_changed)} version_delta=${String(res.body?.version_delta ?? "unknown")}\nclt_changed=${boolLabel(res.body?.clt_node_changed)} decisions_changed=${boolLabel(res.body?.decisions_delta?.changed)}\nnext_tools=focusa_tree_restore_state,focusa_tree_path`,
        "tree diff"
      );
    },
  });

  pi.registerTool({
    name: "focusa_metacog_capture",
    label: "Metacog Capture",
    description:
      "Store a reusable learning signal so future reasoning can retrieve it instead of rediscovering the same lesson.",
    parameters: strictObject({
      kind: Type.String({ minLength: 1, maxLength: SPEC81_LIMITS.kind, description: "Signal kind." }),
      content: Type.String({
        minLength: 1,
        maxLength: SPEC81_LIMITS.longText,
        description: "Signal content.",
      }),
      rationale: Type.Optional(
        Type.String({ maxLength: SPEC81_LIMITS.rationale, description: "Optional rationale." })
      ),
      evidence_refs: Type.Optional(
        Type.Array(Type.String(), { description: "Evidence refs supporting this learning signal." })
      ),
      confidence: Type.Optional(
        Type.Number({ minimum: 0, maximum: 1, description: "Optional confidence 0..1" })
      ),
      strategy_class: Type.Optional(
        Type.String({ maxLength: SPEC81_LIMITS.strategyClass, description: "Optional strategy class." })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_metacog_capture", params, [
        "kind",
        "content",
        "rationale",
        "evidence_refs",
        "confidence",
        "strategy_class",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_capture",
          "/v1/metacognition/capture",
          params as Record<string, any>,
          "metacog capture",
          keyCheck.error
        );
      }
      const raw = keyCheck.value as {
        kind: string;
        content: string;
        rationale?: string;
        evidence_refs?: string[];
        confidence?: number;
        strategy_class?: string;
      };
      const kindCheck = validateRequiredString("kind", raw.kind, SPEC81_LIMITS.kind);
      if (!kindCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_capture",
          "/v1/metacognition/capture",
          raw as Record<string, any>,
          "metacog capture",
          kindCheck.error
        );
      }
      const contentCheck = validateRequiredString("content", raw.content, SPEC81_LIMITS.longText);
      if (!contentCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_capture",
          "/v1/metacognition/capture",
          raw as Record<string, any>,
          "metacog capture",
          contentCheck.error
        );
      }
      const rationaleCheck = validateOptionalString("rationale", raw.rationale, SPEC81_LIMITS.rationale);
      if (!rationaleCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_capture",
          "/v1/metacognition/capture",
          raw as Record<string, any>,
          "metacog capture",
          rationaleCheck.error
        );
      }
      const strategyCheck = validateOptionalString(
        "strategy_class",
        raw.strategy_class,
        SPEC81_LIMITS.strategyClass
      );
      if (!strategyCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_capture",
          "/v1/metacognition/capture",
          raw as Record<string, any>,
          "metacog capture",
          strategyCheck.error
        );
      }
      if (
        raw.confidence !== undefined &&
        (!Number.isFinite(raw.confidence) || raw.confidence < 0 || raw.confidence > 1)
      ) {
        return spec80ValidationResult(
          "focusa_metacog_capture",
          "/v1/metacognition/capture",
          raw as Record<string, any>,
          "metacog capture",
          "confidence must be between 0 and 1"
        );
      }
      const req = {
        kind: kindCheck.value,
        content: contentCheck.value,
        rationale: rationaleCheck.value,
        evidence_refs: Array.isArray(raw.evidence_refs) ? raw.evidence_refs.slice(0, 8) : [],
        confidence: raw.confidence,
        strategy_class: strategyCheck.value,
      };
      const res = await callSpec80Tool("focusa_metacog_capture", "/metacognition/capture", req, {
        method: "POST",
        writer: true,
      });
      const captureId = String(res.body?.capture_id || "stored");
      const lessonLine = compactText(req.content, "no lesson content", 120);
      const relevanceReason = compactText(
        req.rationale ||
          (req.evidence_refs.length ? `evidence=${req.evidence_refs[0]}` : "captured for future retrieval"),
        "captured for future retrieval",
        100
      );
      return spec80Result(
        "focusa_metacog_capture",
        "/v1/metacognition/capture",
        {
          ...req,
          writer_id: res.writerId || null,
          compact_lesson_line: { lesson: lessonLine, why_relevant: relevanceReason, rehydrate_id: captureId },
        },
        res,
        `metacog capture: id=${captureId} lesson="${lessonLine}" why="${relevanceReason}" rehydrate_id=${captureId}`,
        "metacog capture",
        {
          kind: "ok",
          ids: [
            { label: "capture_id", value: captureId },
            { label: "rehydrate_id", value: captureId },
            { label: "kind", value: req.kind },
          ],
          fields: [
            { label: "lesson", value: lessonLine },
            { label: "why", value: relevanceReason },
            { label: "strategy_class", value: req.strategy_class || null },
            { label: "confidence", value: req.confidence ?? null },
            {
              label: "evidence_refs",
              value: Array.isArray(req.evidence_refs) ? req.evidence_refs.length : 0,
            },
          ],
          nextTools: ["focusa_metacog_retrieve", "focusa_metacog_reflect", "focusa_metacog_doctor"],
        }
      );
    },
  });

  pi.registerTool({
    name: "focusa_metacog_retrieve",
    label: "Metacog Retrieve",
    description:
      "Best safe search tool for past learning signals relevant to the current ask. Use this before planning or reflection.",
    parameters: strictObject({
      current_ask: Type.String({
        minLength: 1,
        maxLength: SPEC81_LIMITS.currentAsk,
        description: "Current ask.",
      }),
      scope_tags: Type.Optional(
        Type.Array(Type.String({ maxLength: SPEC81_LIMITS.tagText, description: "Optional scope tag." }), {
          maxItems: SPEC81_LIMITS.scopeTags,
          description: "Optional scope tags.",
        })
      ),
      k: Type.Optional(
        Type.Integer({ minimum: 1, maximum: 50, description: "Top-k candidates (default 5)." })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_metacog_retrieve", params, [
        "current_ask",
        "scope_tags",
        "k",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_retrieve",
          "/v1/metacognition/retrieve",
          params as Record<string, any>,
          "metacog retrieve",
          keyCheck.error
        );
      }
      const raw = keyCheck.value as { current_ask: string; scope_tags?: string[]; k?: number };
      const askCheck = validateRequiredString("current_ask", raw.current_ask, SPEC81_LIMITS.currentAsk);
      if (!askCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_retrieve",
          "/v1/metacognition/retrieve",
          raw as Record<string, any>,
          "metacog retrieve",
          askCheck.error
        );
      }
      const tagsCheck = validateStringArray("scope_tags", raw.scope_tags, {
        maxItems: SPEC81_LIMITS.scopeTags,
        itemMaxLength: SPEC81_LIMITS.tagText,
      });
      if (!tagsCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_retrieve",
          "/v1/metacognition/retrieve",
          raw as Record<string, any>,
          "metacog retrieve",
          tagsCheck.error
        );
      }
      let normalizedK = Math.trunc(Number(raw.k ?? 5));
      if (!Number.isFinite(normalizedK)) normalizedK = 5;
      normalizedK = Math.max(1, Math.min(50, normalizedK));
      const req = { current_ask: askCheck.value, scope_tags: tagsCheck.value, k: normalizedK };
      const res = await callSpec80Tool("focusa_metacog_retrieve", "/metacognition/retrieve", req, {
        method: "POST",
      });
      const candidates = Array.isArray(res.body?.candidates) ? res.body.candidates : [];
      const total = candidates.length;
      const top = candidates[0];
      const topCapture = String(top?.capture_id || "none");
      const fullLesson = String(top?.summary || top?.content || top?.signal || "");
      const lessonRequiresRehydrate = fullLesson.length > 120;
      // FOCUSA_FIX-63jd/BAD-AX: never show a mid-sentence truncated lesson.
      // Long lessons render only a rehydrate pointer; compact content remains in details.
      const topLesson = lessonRequiresRehydrate
        ? `see_rehydrate_ref:${topCapture}`
        : compactText(fullLesson || "no lesson content", "no lesson content", 120);
      const rehydrateHint = lessonRequiresRehydrate
        ? `rehydrate_full=true lesson_chars=${fullLesson.length} rehydrate_ref=${topCapture}`
        : null;
      const topWhy = compactText(
        top?.rationale ||
          top?.why_relevant ||
          (top?.score !== undefined
            ? `retrieval_score=${String(top.score)}`
            : `matched current_ask=${req.current_ask}`),
        "matched current ask",
        100
      );
      return spec80Result(
        "focusa_metacog_retrieve",
        "/v1/metacognition/retrieve",
        {
          ...req,
          compact_top_lesson:
            total > 0
              ? {
                  lesson: topLesson,
                  why_relevant: topWhy,
                  rehydrate_id: topCapture,
                  lesson_inline_omitted: lessonRequiresRehydrate,
                  full_lesson_chars: fullLesson.length,
                }
              : null,
        },
        res,
        total > 0
          ? `metacog retrieve: candidates=${total} top_lesson="${topLesson}" why="${topWhy}" rehydrate_id=${topCapture}` +
              (rehydrateHint ? ` ${rehydrateHint}` : "")
          : `metacog retrieve: candidates=0 lesson="none" why="no prior signals matched" rehydrate_id=none`,
        "metacog retrieve",
        {
          kind: total > 0 ? "ok" : "advisory",
          ids: total > 0 ? [{ label: "rehydrate_id", value: topCapture }] : [],
          fields: [
            { label: "candidates", value: total },
            { label: "top_lesson", value: total > 0 ? topLesson : "none" },
            { label: "why", value: total > 0 ? topWhy : "no prior signals matched" },
            { label: "ask", value: req.current_ask },
          ],
          nextTools:
            total > 0
              ? ["focusa_metacog_reflect", "focusa_metacog_plan_adjust", "focusa_metacog_doctor"]
              : ["focusa_metacog_capture", "focusa_metacog_doctor"],
        }
      );
    },
  });

  pi.registerTool({
    name: "focusa_metacog_reflect",
    label: "Metacog Reflect",
    description:
      "Generate reusable hypotheses and strategy updates from recent turns when you need learning from past outcomes.",
    parameters: strictObject({
      turn_range: Type.String({
        minLength: 1,
        maxLength: SPEC81_LIMITS.turnRange,
        pattern: SPEC81_TURN_RANGE_PATTERN,
        description: "Turn range expression.",
      }),
      failure_classes: Type.Optional(
        Type.Array(Type.String({ maxLength: SPEC81_LIMITS.tagText, description: "Failure class tag." }), {
          maxItems: SPEC81_LIMITS.failureClasses,
          description: "Failure class tags.",
        })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_metacog_reflect", params, [
        "turn_range",
        "failure_classes",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_reflect",
          "/v1/metacognition/reflect",
          params as Record<string, any>,
          "metacog reflect",
          keyCheck.error
        );
      }
      const raw = keyCheck.value as { turn_range: string; failure_classes?: string[] };
      const turnRangeCheck = validateRequiredString("turn_range", raw.turn_range, SPEC81_LIMITS.turnRange, {
        pattern: SPEC81_TURN_RANGE_RE,
      });
      if (!turnRangeCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_reflect",
          "/v1/metacognition/reflect",
          raw as Record<string, any>,
          "metacog reflect",
          turnRangeCheck.error
        );
      }
      const failureCheck = validateStringArray("failure_classes", raw.failure_classes, {
        maxItems: SPEC81_LIMITS.failureClasses,
        itemMaxLength: SPEC81_LIMITS.tagText,
      });
      if (!failureCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_reflect",
          "/v1/metacognition/reflect",
          raw as Record<string, any>,
          "metacog reflect",
          failureCheck.error
        );
      }
      const req = { turn_range: turnRangeCheck.value, failure_classes: failureCheck.value };
      const res = await callSpec80Tool("focusa_metacog_reflect", "/metacognition/reflect", req, {
        method: "POST",
        writer: true,
      });
      const updates = Array.isArray(res.body?.strategy_updates) ? res.body.strategy_updates : [];
      return spec80Result(
        "focusa_metacog_reflect",
        "/v1/metacognition/reflect",
        { ...req, writer_id: res.writerId || null },
        res,
        `metacog reflect: ${String(res.body?.reflection_id || "ok")} hypotheses=${Array.isArray(res.body?.hypotheses) ? res.body.hypotheses.length : 0}\nstrategy_updates=${summarizeArray(updates, 4)}\nnext_tools=focusa_metacog_plan_adjust,focusa_metacog_doctor`,
        "metacog reflect",
        {
          kind: "ok",
          ids: [{ label: "reflection_id", value: String(res.body?.reflection_id || "ok") }],
          fields: [
            {
              label: "hypotheses",
              value: Array.isArray(res.body?.hypotheses) ? res.body.hypotheses.length : 0,
            },
            { label: "strategy_updates", value: updates.length },
            { label: "turn_range", value: req.turn_range },
            { label: "failure_classes", value: (req.failure_classes || []).length },
          ],
          nextTools: ["focusa_metacog_plan_adjust", "focusa_metacog_doctor"],
        }
      );
    },
  });

  pi.registerTool({
    name: "focusa_metacog_plan_adjust",
    label: "Metacog Plan Adjust",
    description:
      "Turn a reflection into a tracked adjustment artifact that can later be evaluated for real improvement.",
    parameters: strictObject({
      reflection_id: Type.String({
        minLength: 1,
        maxLength: SPEC81_LIMITS.id,
        pattern: SPEC81_ID_PATTERN,
        description: "Reflection id.",
      }),
      selected_updates: Type.Optional(
        Type.Array(Type.String({ maxLength: SPEC81_LIMITS.updateText, description: "Selected update." }), {
          maxItems: SPEC81_LIMITS.selectedUpdates,
          description: "Selected updates.",
        })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_metacog_plan_adjust", params, [
        "reflection_id",
        "selected_updates",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_plan_adjust",
          "/v1/metacognition/adjust",
          params as Record<string, any>,
          "metacog adjust",
          keyCheck.error
        );
      }
      const raw = keyCheck.value as { reflection_id: string; selected_updates?: string[] };
      const reflectionCheck = validateRequiredString("reflection_id", raw.reflection_id, SPEC81_LIMITS.id, {
        pattern: SPEC81_ID_RE,
      });
      if (!reflectionCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_plan_adjust",
          "/v1/metacognition/adjust",
          raw as Record<string, any>,
          "metacog adjust",
          reflectionCheck.error
        );
      }
      const updatesCheck = validateStringArray("selected_updates", raw.selected_updates, {
        maxItems: SPEC81_LIMITS.selectedUpdates,
        itemMaxLength: SPEC81_LIMITS.updateText,
      });
      if (!updatesCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_plan_adjust",
          "/v1/metacognition/adjust",
          raw as Record<string, any>,
          "metacog adjust",
          updatesCheck.error
        );
      }
      const req = { reflection_id: reflectionCheck.value, selected_updates: updatesCheck.value };
      const res = await callSpec80Tool("focusa_metacog_plan_adjust", "/metacognition/adjust", req, {
        method: "POST",
        writer: true,
      });
      return spec80Result(
        "focusa_metacog_plan_adjust",
        "/v1/metacognition/adjust",
        { ...req, writer_id: res.writerId || null },
        res,
        `metacog adjust: ${String(res.body?.adjustment_id || "ok")} updates=${updatesCheck.value.length}\nnext_step_policy=${summarizeArray(res.body?.next_step_policy || updatesCheck.value, 4)}\nnext_tools=focusa_metacog_evaluate_outcome,focusa_metacog_doctor`,
        "metacog adjust",
        {
          kind: "ok",
          ids: [
            { label: "adjustment_id", value: String(res.body?.adjustment_id || "ok") },
            { label: "reflection_id", value: req.reflection_id },
          ],
          fields: [
            { label: "updates", value: updatesCheck.value.length },
            {
              label: "next_step_policy",
              value: summarizeArray(res.body?.next_step_policy || updatesCheck.value, 4),
            },
          ],
          nextTools: ["focusa_metacog_evaluate_outcome", "focusa_metacog_doctor"],
        }
      );
    },
  });

  pi.registerTool({
    name: "focusa_metacog_evaluate_outcome",
    label: "Metacog Evaluate Outcome",
    description: "Judge whether an adjustment improved results and whether the learning should be promoted.",
    parameters: strictObject({
      adjustment_id: Type.String({
        minLength: 1,
        maxLength: SPEC81_LIMITS.id,
        pattern: SPEC81_ID_PATTERN,
        description: "Adjustment id.",
      }),
      observed_metrics: Type.Optional(
        Type.Array(Type.String({ maxLength: SPEC81_LIMITS.metricText, description: "Observed metric id." }), {
          maxItems: SPEC81_LIMITS.observedMetrics,
          description: "Observed metric ids.",
        })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_metacog_evaluate_outcome", params, [
        "adjustment_id",
        "observed_metrics",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_evaluate_outcome",
          "/v1/metacognition/evaluate",
          params as Record<string, any>,
          "metacog evaluate",
          keyCheck.error
        );
      }
      const raw = keyCheck.value as { adjustment_id: string; observed_metrics?: string[] };
      const adjustmentCheck = validateRequiredString("adjustment_id", raw.adjustment_id, SPEC81_LIMITS.id, {
        pattern: SPEC81_ID_RE,
      });
      if (!adjustmentCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_evaluate_outcome",
          "/v1/metacognition/evaluate",
          raw as Record<string, any>,
          "metacog evaluate",
          adjustmentCheck.error
        );
      }
      const metricsCheck = validateStringArray("observed_metrics", raw.observed_metrics, {
        maxItems: SPEC81_LIMITS.observedMetrics,
        itemMaxLength: SPEC81_LIMITS.metricText,
      });
      if (!metricsCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_evaluate_outcome",
          "/v1/metacognition/evaluate",
          raw as Record<string, any>,
          "metacog evaluate",
          metricsCheck.error
        );
      }
      const req = { adjustment_id: adjustmentCheck.value, observed_metrics: metricsCheck.value };
      const res = await callSpec80Tool("focusa_metacog_evaluate_outcome", "/metacognition/evaluate", req, {
        method: "POST",
        writer: true,
      });
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
        {
          kind: "ok",
          ids: [
            { label: "adjustment_id", value: req.adjustment_id },
            { label: "evaluation_id", value: String(res.body?.evaluation_id || res.body?.result || "ok") },
          ],
          fields: [
            { label: "decision", value: String(res.body?.result || "unknown") },
            { label: "promote", value: boolLabel(res.body?.promote_learning) },
            { label: "observed_metrics", value: summarizeArray(observed, 4) },
          ],
          nextTools: ["focusa_metacog_doctor", "focusa_metacog_recent_adjustments"],
        }
      );
    },
  });

  pi.registerTool({
    name: "focusa_bloatgaurd_report",
    label: "Bloatgaurd Report",
    description: "Spec 101 — read the compact Bloatgaurd budget report for domains 5.1-5.8.",
    promptSnippet:
      "Use before cleanup or context-budget work to inspect Bloatgaurd budget domains and full-payload/deletion gates.",
    parameters: strictObject({}),
    execute: async () => {
      const result = await focusaFetchDetailed("/bloatgaurd/report");
      const body = result.body || {};
      const domains = Array.isArray(body.domains) ? body.domains : [];
      const ok = result.ok && body.status !== "blocked";
      const toolResult = focusaToolResult({
        ok,
        status: ok ? "completed" : "blocked",
        summary: `bloatgaurd report → domains=${domains.length} status=${body.status || result.status}`,
        tool: "focusa_bloatgaurd_report",
        family: "diagnostics_hygiene",
        side_effects: [],
        evidence_refs: [],
        next_tools: [
          "focusa_bloatgaurd_domain",
          "focusa_context_cognition_render",
          "focusa_evidence_capture",
        ],
        raw: body,
      });
      return {
        content: [
          {
            type: "text",
            text: `bloatgaurd report ${body.status || result.status} | domains=${domains.length}\nnext_tools=focusa_bloatgaurd_domain,focusa_context_cognition_render`,
          },
        ],
        details: { tool_result_v1: toolResult },
      };
    },
  });

  pi.registerTool({
    name: "focusa_bloatgaurd_domain",
    label: "Bloatgaurd Domain",
    description: "Spec 101 — read one Bloatgaurd budget domain and its checks/findings.",
    promptSnippet:
      "Use when a gap maps to one budget domain such as output-firewall, docs-diet, or dead-code-safety.",
    parameters: strictObject({
      name: Type.String({ minLength: 1, maxLength: 120, description: "Bloatgaurd domain slug or title." }),
    }),
    execute: async (_toolCallId: string, params: any) => {
      const keyCheck = validateNoExtraKeys("focusa_bloatgaurd_domain", params, ["name"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_bloatgaurd_domain",
          "/v1/bloatgaurd/domain/{name}",
          params as Record<string, any>,
          "bloatgaurd domain",
          keyCheck.error
        );
      }
      const name = encodeURIComponent(String(params.name || ""));
      const result = await focusaFetchDetailed(`/bloatgaurd/domain/${name}`);
      const body = result.body || {};
      const domain = body.domain || null;
      const ok = result.ok && body.status === "completed";
      const toolResult = focusaToolResult({
        ok,
        status: ok ? "completed" : "blocked",
        summary: `bloatgaurd domain → ${domain?.name || params.name} status=${body.status || result.status}`,
        tool: "focusa_bloatgaurd_domain",
        family: "diagnostics_hygiene",
        side_effects: [],
        evidence_refs: [],
        next_tools: ["focusa_bloatgaurd_report", "focusa_traverse", "focusa_evidence_capture"],
        raw: body,
      });
      return {
        content: [
          {
            type: "text",
            text: `bloatgaurd domain ${body.status || result.status} | ${domain?.section || "?"} ${domain?.name || params.name}\nchecks=${Array.isArray(domain?.checks) ? domain.checks.length : 0}`,
          },
        ],
        details: { tool_result_v1: toolResult },
      };
    },
  });

  pi.registerTool({
    name: "focusa_bloatgaurd_tokenbloat_report",
    label: "Bloatgaurd Tokenbloat Report",
    description: "Spec 101 — read Tokenbloat Control report for domains 5.9-5.10.",
    promptSnippet:
      "Use to inspect prompt/runtime tokenbloat controls before reducing context or eliding tool history.",
    parameters: strictObject({}),
    execute: async () => {
      const result = await focusaFetchDetailed("/bloatgaurd/tokenbloat/report");
      const body = result.body || {};
      const controls = Array.isArray(body.controls) ? body.controls : [];
      const ok = result.ok && body.status !== "blocked";
      const toolResult = focusaToolResult({
        ok,
        status: ok ? "completed" : "blocked",
        summary: `bloatgaurd tokenbloat report → controls=${controls.length} status=${body.status || result.status}`,
        tool: "focusa_bloatgaurd_tokenbloat_report",
        family: "diagnostics_hygiene",
        side_effects: [],
        evidence_refs: [],
        next_tools: [
          "focusa_bloatgaurd_tokenbloat_domain",
          "focusa_bloatgaurd_report",
          "focusa_evidence_capture",
        ],
        raw: body,
      });
      return {
        content: [
          {
            type: "text",
            text: `bloatgaurd tokenbloat ${body.status || result.status} | controls=${controls.length}\nnext_tools=focusa_bloatgaurd_tokenbloat_domain,focusa_bloatgaurd_report`,
          },
        ],
        details: { tool_result_v1: toolResult },
      };
    },
  });

  pi.registerTool({
    name: "focusa_bloatgaurd_tokenbloat_domain",
    label: "Bloatgaurd Tokenbloat Domain",
    description: "Spec 101 — read one Tokenbloat Control domain and its prompt-visible fields/boundaries.",
    promptSnippet: "Use when a gap maps to tokenbloat-control or tool-call-history-elision.",
    parameters: strictObject({
      name: Type.String({ minLength: 1, maxLength: 120, description: "Tokenbloat domain slug or title." }),
    }),
    execute: async (_toolCallId: string, params: any) => {
      const keyCheck = validateNoExtraKeys("focusa_bloatgaurd_tokenbloat_domain", params, ["name"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_bloatgaurd_tokenbloat_domain",
          "/v1/bloatgaurd/tokenbloat/domain/{name}",
          params as Record<string, any>,
          "bloatgaurd tokenbloat domain",
          keyCheck.error
        );
      }
      const name = encodeURIComponent(String(params.name || ""));
      const result = await focusaFetchDetailed(`/bloatgaurd/tokenbloat/domain/${name}`);
      const body = result.body || {};
      const domain = body.domain || null;
      const ok = result.ok && body.status === "completed";
      const toolResult = focusaToolResult({
        ok,
        status: ok ? "completed" : "blocked",
        summary: `bloatgaurd tokenbloat domain → ${domain?.name || params.name} status=${body.status || result.status}`,
        tool: "focusa_bloatgaurd_tokenbloat_domain",
        family: "diagnostics_hygiene",
        side_effects: [],
        evidence_refs: [],
        next_tools: ["focusa_bloatgaurd_tokenbloat_report", "focusa_traverse", "focusa_evidence_capture"],
        raw: body,
      });
      return {
        content: [
          {
            type: "text",
            text: `bloatgaurd tokenbloat domain ${body.status || result.status} | ${domain?.section || "?"} ${domain?.name || params.name}\nfields=${Array.isArray(domain?.prompt_visible_fields) ? domain.prompt_visible_fields.length : 0}`,
          },
        ],
        details: { tool_result_v1: toolResult },
      };
    },
  });

  pi.registerTool({
    name: "focusa_bloatgaurd_gate_modes",
    label: "Bloatgaurd Gate Modes",
    description: "Spec 101 — read gate modes A/B/C thresholds, allowlist, and report schema.",
    promptSnippet: "Use before enabling warning/fail-candidate Bloatgaurd gates; this tool is read-only.",
    parameters: strictObject({}),
    execute: async () => {
      const result = await focusaFetchDetailed("/bloatgaurd/gate-modes/report");
      const body = result.body || {};
      const modes = Array.isArray(body.modes) ? body.modes : [];
      const ok = result.ok && body.status !== "blocked";
      const toolResult = focusaToolResult({
        ok,
        status: ok ? "completed" : "blocked",
        summary: `bloatgaurd gate modes → modes=${modes.length} status=${body.status || result.status}`,
        tool: "focusa_bloatgaurd_gate_modes",
        family: "diagnostics_hygiene",
        side_effects: [],
        evidence_refs: [],
        next_tools: ["focusa_bloatgaurd_gate_mode", "focusa_bloatgaurd_report", "focusa_evidence_capture"],
        raw: body,
      });
      return {
        content: [
          {
            type: "text",
            text: `bloatgaurd gate-modes ${body.status || result.status} | modes=${modes.length}\nnext_tools=focusa_bloatgaurd_gate_mode,focusa_bloatgaurd_report`,
          },
        ],
        details: { tool_result_v1: toolResult },
      };
    },
  });

  pi.registerTool({
    name: "focusa_bloatgaurd_gate_mode",
    label: "Bloatgaurd Gate Mode",
    description: "Spec 101 — read one Bloatgaurd gate mode by code/name.",
    promptSnippet:
      "Use to inspect a gate mode such as A/advisory, B/warning, or C/fail-candidate before enforcing it.",
    parameters: strictObject({
      name: Type.String({ minLength: 1, maxLength: 120, description: "Gate mode code or name." }),
    }),
    execute: async (_toolCallId: string, params: any) => {
      const keyCheck = validateNoExtraKeys("focusa_bloatgaurd_gate_mode", params, ["name"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_bloatgaurd_gate_mode",
          "/v1/bloatgaurd/gate-modes/mode/{name}",
          params as Record<string, any>,
          "bloatgaurd gate mode",
          keyCheck.error
        );
      }
      const name = encodeURIComponent(String(params.name || ""));
      const result = await focusaFetchDetailed(`/bloatgaurd/gate-modes/mode/${name}`);
      const body = result.body || {};
      const mode = body.mode || null;
      const ok = result.ok && body.status === "completed";
      const toolResult = focusaToolResult({
        ok,
        status: ok ? "completed" : "blocked",
        summary: `bloatgaurd gate mode → ${mode?.code || params.name} status=${body.status || result.status}`,
        tool: "focusa_bloatgaurd_gate_mode",
        family: "diagnostics_hygiene",
        side_effects: [],
        evidence_refs: [],
        next_tools: ["focusa_bloatgaurd_gate_modes", "focusa_traverse", "focusa_evidence_capture"],
        raw: body,
      });
      return {
        content: [
          {
            type: "text",
            text: `bloatgaurd gate-mode ${body.status || result.status} | ${mode?.code || "?"} ${mode?.name || params.name}\nreport_schema_fields=${Array.isArray(mode?.report_schema_fields) ? mode.report_schema_fields.length : 0}`,
          },
        ],
        details: { tool_result_v1: toolResult },
      };
    },
  });

  pi.registerTool({
    name: "focusa_bloatgaurd_profiles",
    label: "Bloatgaurd Profiles",
    description: "Spec 101 — read profile presets and operator switches.",
    promptSnippet:
      "Use to inspect Daily Driver, Beast Mode, Speedy, Neat Freak, and Tightwad profile settings.",
    parameters: strictObject({}),
    execute: async () => {
      const result = await focusaFetchDetailed("/bloatgaurd/profiles/report");
      const body = result.body || {};
      const profiles = Array.isArray(body.profiles) ? body.profiles : [];
      const ok = result.ok && body.status === "completed";
      const toolResult = focusaToolResult({
        ok,
        status: ok ? "completed" : "blocked",
        summary: `bloatgaurd profiles → profiles=${profiles.length}`,
        tool: "focusa_bloatgaurd_profiles",
        family: "diagnostics_hygiene",
        side_effects: [],
        evidence_refs: [],
        next_tools: ["focusa_bloatgaurd_profile", "focusa_bloatgaurd_routines", "focusa_evidence_capture"],
        raw: body,
      });
      return {
        content: [
          {
            type: "text",
            text: `bloatgaurd profiles ${body.status || result.status} | profiles=${profiles.length}`,
          },
        ],
        details: { tool_result_v1: toolResult },
      };
    },
  });

  pi.registerTool({
    name: "focusa_bloatgaurd_profile",
    label: "Bloatgaurd Profile",
    description: "Spec 101 — read one profile preset by name.",
    promptSnippet: "Use to inspect a profile such as daily_driver or tightwad.",
    parameters: strictObject({
      name: Type.String({ minLength: 1, maxLength: 120, description: "Profile slug or title." }),
    }),
    execute: async (_toolCallId: string, params: any) => {
      const keyCheck = validateNoExtraKeys("focusa_bloatgaurd_profile", params, ["name"]);
      if (!keyCheck.ok)
        return spec80ValidationResult(
          "focusa_bloatgaurd_profile",
          "/v1/bloatgaurd/profiles/profile/{name}",
          params as Record<string, any>,
          "bloatgaurd profile",
          keyCheck.error
        );
      const result = await focusaFetchDetailed(
        `/bloatgaurd/profiles/profile/${encodeURIComponent(String(params.name || ""))}`
      );
      const body = result.body || {};
      const profile = body.profile || null;
      const ok = result.ok && body.status === "completed";
      const toolResult = focusaToolResult({
        ok,
        status: ok ? "completed" : "blocked",
        summary: `bloatgaurd profile → ${profile?.name || params.name}`,
        tool: "focusa_bloatgaurd_profile",
        family: "diagnostics_hygiene",
        side_effects: [],
        evidence_refs: [],
        next_tools: ["focusa_bloatgaurd_profiles", "focusa_bloatgaurd_routines", "focusa_evidence_capture"],
        raw: body,
      });
      return {
        content: [
          {
            type: "text",
            text: `bloatgaurd profile ${body.status || result.status} | ${profile?.name || params.name}`,
          },
        ],
        details: { tool_result_v1: toolResult },
      };
    },
  });

  pi.registerTool({
    name: "focusa_bloatgaurd_routines",
    label: "Bloatgaurd Routines",
    description: "Spec 101 — read named routines and automation matrix.",
    promptSnippet: "Use to inspect Patrol through Scout automation policy.",
    parameters: strictObject({}),
    execute: async () => {
      const result = await focusaFetchDetailed("/bloatgaurd/routines/report");
      const body = result.body || {};
      const routines = Array.isArray(body.routines) ? body.routines : [];
      const ok = result.ok && body.status === "completed";
      const toolResult = focusaToolResult({
        ok,
        status: ok ? "completed" : "blocked",
        summary: `bloatgaurd routines → routines=${routines.length}`,
        tool: "focusa_bloatgaurd_routines",
        family: "diagnostics_hygiene",
        side_effects: [],
        evidence_refs: [],
        next_tools: ["focusa_bloatgaurd_routine", "focusa_bloatgaurd_profiles", "focusa_evidence_capture"],
        raw: body,
      });
      return {
        content: [
          {
            type: "text",
            text: `bloatgaurd routines ${body.status || result.status} | routines=${routines.length}`,
          },
        ],
        details: { tool_result_v1: toolResult },
      };
    },
  });

  pi.registerTool({
    name: "focusa_bloatgaurd_routine",
    label: "Bloatgaurd Routine",
    description: "Spec 101 — read one named routine by name.",
    promptSnippet: "Use to inspect a routine such as patrol, gatekeeper, or scout.",
    parameters: strictObject({
      name: Type.String({ minLength: 1, maxLength: 120, description: "Routine slug or title." }),
    }),
    execute: async (_toolCallId: string, params: any) => {
      const keyCheck = validateNoExtraKeys("focusa_bloatgaurd_routine", params, ["name"]);
      if (!keyCheck.ok)
        return spec80ValidationResult(
          "focusa_bloatgaurd_routine",
          "/v1/bloatgaurd/routines/routine/{name}",
          params as Record<string, any>,
          "bloatgaurd routine",
          keyCheck.error
        );
      const result = await focusaFetchDetailed(
        `/bloatgaurd/routines/routine/${encodeURIComponent(String(params.name || ""))}`
      );
      const body = result.body || {};
      const routine = body.routine || null;
      const ok = result.ok && body.status === "completed";
      const toolResult = focusaToolResult({
        ok,
        status: ok ? "completed" : "blocked",
        summary: `bloatgaurd routine → ${routine?.name || params.name}`,
        tool: "focusa_bloatgaurd_routine",
        family: "diagnostics_hygiene",
        side_effects: [],
        evidence_refs: [],
        next_tools: ["focusa_bloatgaurd_routines", "focusa_bloatgaurd_profiles", "focusa_evidence_capture"],
        raw: body,
      });
      return {
        content: [
          {
            type: "text",
            text: `bloatgaurd routine ${body.status || result.status} | ${routine?.name || params.name}`,
          },
        ],
        details: { tool_result_v1: toolResult },
      };
    },
  });

  pi.registerTool({
    name: "focusa_bloatgaurd_rollout",
    label: "Bloatgaurd Rollout",
    description: "Spec 101 — read rollout phases, acceptance checks, and proof commands.",
    promptSnippet: "Use to verify rollout hardening acceptance/proof for Bloatgaurd MVP.",
    parameters: strictObject({}),
    execute: async () => {
      const result = await focusaFetchDetailed("/bloatgaurd/rollout/report");
      const body = result.body || {};
      const phases = Array.isArray(body.phases) ? body.phases : [];
      const ok = result.ok && body.status === "completed";
      const toolResult = focusaToolResult({
        ok,
        status: ok ? "completed" : "blocked",
        summary: `bloatgaurd rollout → phases=${phases.length}`,
        tool: "focusa_bloatgaurd_rollout",
        family: "diagnostics_hygiene",
        side_effects: [],
        evidence_refs: [],
        next_tools: ["focusa_bloatgaurd_profiles", "focusa_bloatgaurd_routines", "focusa_evidence_capture"],
        raw: body,
      });
      return {
        content: [
          {
            type: "text",
            text: `bloatgaurd rollout ${body.status || result.status} | phases=${phases.length}`,
          },
        ],
        details: { tool_result_v1: toolResult },
      };
    },
  });

  pi.registerTool({
    name: "focusa_dxux_report",
    label: "DX/UX Report",
    description: "Spec105 — read implementation report for DXUX-001..012.",
    promptSnippet:
      "Use to verify Spec105 reliability, continuation, recovery, evidence, and drift UX surfaces.",
    parameters: strictObject({}),
    execute: async () => {
      const result = await focusaFetchDetailed("/dxux/report");
      const body = result.body || {};
      const requirements = Array.isArray(body.requirements) ? body.requirements : [];
      const ok = result.ok && body.status === "completed";
      const toolResult = focusaToolResult({
        ok,
        status: ok ? "completed" : "blocked",
        summary: `dxux report → requirements=${requirements.length}`,
        tool: "focusa_dxux_report",
        family: "diagnostics_hygiene",
        side_effects: [],
        evidence_refs: [],
        next_tools: ["focusa_dxux_requirement", "focusa_dxux_digest", "focusa_evidence_capture"],
        raw: body,
      });
      return {
        content: [
          {
            type: "text",
            text: `dxux report ${body.status || result.status} | requirements=${requirements.length}`,
          },
        ],
        details: { tool_result_v1: toolResult },
      };
    },
  });

  pi.registerTool({
    name: "focusa_dxux_requirement",
    label: "DX/UX Requirement",
    description: "Spec105 — read one DXUX requirement by id.",
    promptSnippet: "Use to inspect a requirement such as DXUX-004 or DXUX-012.",
    parameters: strictObject({
      id: Type.String({ minLength: 1, maxLength: 40, description: "Requirement id, e.g. DXUX-004." }),
    }),
    execute: async (_toolCallId: string, params: any) => {
      const keyCheck = validateNoExtraKeys("focusa_dxux_requirement", params, ["id"]);
      if (!keyCheck.ok)
        return spec80ValidationResult(
          "focusa_dxux_requirement",
          "/v1/dxux/requirement/{id}",
          params as Record<string, any>,
          "dxux requirement",
          keyCheck.error,
          "SCHEMA_INVALID",
          { required_fields: ["id"], field_types: { id: "string" }, example: { id: "DXUX-004" } }
        );
      const idCheck = validateRequiredString("id", keyCheck.value.id, 40, {
        pattern: /^DXUX[-_]\d{3}$/i,
      });
      if (!idCheck.ok)
        return spec80ValidationResult(
          "focusa_dxux_requirement",
          "/v1/dxux/requirement/{id}",
          params as Record<string, any>,
          "dxux requirement",
          idCheck.error,
          "SCHEMA_INVALID",
          { required_fields: ["id"], field_types: { id: "string" }, example: { id: "DXUX-004" } }
        );
      const id = idCheck.value;
      const result = await focusaFetchDetailed(`/dxux/requirement/${encodeURIComponent(id)}`);
      const body = result.body || {};
      const req = body.requirement || null;
      const ok = result.ok && body.status === "completed";
      const toolResult = focusaToolResult({
        ok,
        status: ok ? "completed" : "blocked",
        summary: `dxux requirement → ${req?.id || id}`,
        tool: "focusa_dxux_requirement",
        family: "diagnostics_hygiene",
        side_effects: [],
        evidence_refs: [],
        next_tools: ["focusa_dxux_report", "focusa_dxux_digest", "focusa_evidence_capture"],
        raw: body,
      });
      return {
        content: [
          {
            type: "text",
            text: `dxux requirement ${body.status || result.status} | ${req?.id || id}`,
          },
        ],
        details: { tool_result_v1: toolResult },
      };
    },
  });

  pi.registerTool({
    name: "focusa_dxux_explain",
    label: "DX/UX Explain",
    description: "Spec105 — explain a failure and return recovery commands.",
    promptSnippet: "Use after CI, scope, daemon, or stale-state failures to get recovery commands.",
    parameters: strictObject({
      failure: Type.String({ minLength: 1, maxLength: 240, description: "Failure text to classify." }),
    }),
    execute: async (_toolCallId: string, params: any) => {
      const keyCheck = validateNoExtraKeys("focusa_dxux_explain", params, ["failure"]);
      if (!keyCheck.ok)
        return spec80ValidationResult(
          "focusa_dxux_explain",
          "/v1/dxux/explain/{failure}",
          params as Record<string, any>,
          "dxux explain",
          keyCheck.error,
          "SCHEMA_INVALID",
          {
            required_fields: ["failure"],
            field_types: { failure: "string" },
            example: { failure: "daemon unavailable" },
          }
        );
      const failureCheck = validateRequiredString("failure", keyCheck.value.failure, 240);
      if (!failureCheck.ok)
        return spec80ValidationResult(
          "focusa_dxux_explain",
          "/v1/dxux/explain/{failure}",
          params as Record<string, any>,
          "dxux explain",
          failureCheck.error,
          "SCHEMA_INVALID",
          {
            required_fields: ["failure"],
            field_types: { failure: "string" },
            example: { failure: "daemon unavailable" },
          }
        );
      const result = await focusaFetchDetailed(`/dxux/explain/${encodeURIComponent(failureCheck.value)}`);
      const body = result.body || {};
      const ok = result.ok && body.status === "completed";
      const toolResult = focusaToolResult({
        ok,
        status: ok ? "completed" : "blocked",
        summary: `dxux explain → confidence=${body.confidence || "unknown"}`,
        tool: "focusa_dxux_explain",
        family: "diagnostics_hygiene",
        side_effects: [],
        evidence_refs: [],
        next_tools: ["focusa_dxux_report", "focusa_tool_doctor", "focusa_evidence_capture"],
        raw: body,
      });
      return {
        content: [
          {
            type: "text",
            text: `dxux explain ${body.status || result.status} | confidence=${body.confidence || "unknown"}`,
          },
        ],
        details: { tool_result_v1: toolResult },
      };
    },
  });

  pi.registerTool({
    name: "focusa_dxux_digest",
    label: "DX/UX Digest",
    description: "Spec105 — read compact continuation/doability digest.",
    promptSnippet:
      "Use before compaction/resume handoff to get status, authority, why, exact next action, evidence refs, and rehydrate refs.",
    parameters: strictObject({}),
    execute: async () => {
      const result = await focusaFetchDetailed("/dxux/digest");
      const body = result.body || {};
      const ok = result.ok && body.status === "completed";
      const toolResult = focusaToolResult({
        ok,
        status: ok ? "completed" : "blocked",
        summary: `dxux digest → can_continue=${body.can_continue === true}`,
        tool: "focusa_dxux_digest",
        family: "diagnostics_hygiene",
        side_effects: [],
        evidence_refs: Array.isArray(body.evidence_refs) ? body.evidence_refs : [],
        next_tools: ["focusa_workpoint_resume", "focusa_dxux_report", "focusa_evidence_capture"],
        raw: body,
      });
      return {
        content: [
          {
            type: "text",
            text: `dxux digest ${body.status || result.status} | can_continue=${body.can_continue === true}\nnext=${body.exact_next_action || "unknown"}`,
          },
        ],
        details: { tool_result_v1: toolResult },
      };
    },
  });

  pi.registerTool({
    name: "focusa_context_cognition",
    label: "Context Cognition",
    description:
      "Build the bounded, advisory Spec 100 ContextCognitionPacket for the current project. Returns a typed packet describing scope, authority, freshness, selected context, ontology frame, evidence frame, reasoning frame, optimization frame, and route frame. Never mutates state.",
    promptSnippet:
      "Use when an operator or agent needs a structured, bounded view of the current project context before making decisions. The packet is advisory and never overrides Workpoint or Trajectory.",
    parameters: strictObject({
      project_root: Type.Optional(
        Type.String({
          maxLength: 4096,
          description: "Project root for the packet. Defaults to Pi session cwd.",
        })
      ),
      continuity_id: Type.Optional(
        Type.String({ maxLength: 256, description: "Optional continuity id filter." })
      ),
      session_id: Type.Optional(Type.String({ maxLength: 256, description: "Optional session id filter." })),
      include_rehydrate_refs: Type.Optional(
        Type.Boolean({ description: "When true, return rehydrate_refs for each surface." })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_context_cognition", params, [
        "project_root",
        "continuity_id",
        "session_id",
        "include_rehydrate_refs",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_context_cognition",
          "/v1/context-cognition",
          params as Record<string, any>,
          "context cognition",
          keyCheck.error
        );
      }
      const projectRoot = await resolveFocusaToolProjectRoot((keyCheck.value as any).project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, (keyCheck.value as any).project_root);
      if (projectRootGate) return projectRootGate;
      const query = new URLSearchParams();
      query.set("project_root", String(projectRoot));
      const workingSubpathId = String(process.env.FOCUSA_WORKING_SUBPATH_ID || "").trim();
      if (workingSubpathId) query.set("working_subpath_id", workingSubpathId);
      const cid = (keyCheck.value as any).continuity_id;
      if (typeof cid === "string" && cid.trim() !== "") query.set("continuity_id", cid.trim());
      const sid = (keyCheck.value as any).session_id;
      if (typeof sid === "string" && sid.trim() !== "") query.set("session_id", sid.trim());
      const include = (keyCheck.value as any).include_rehydrate_refs;
      if (typeof include === "boolean") query.set("include_rehydrate_refs", String(include));
      const res = await focusaFetchDetailed(`/context-cognition?${query.toString()}`);
      const body = res.body || {};
      if (!res.ok) {
        return blockedToolResponse(
          "focusa_context_cognition",
          "trajectory",
          `context cognition blocked → ${explainWorkLoopResult(res, "context cognition unavailable")}`,
          body.failure_class || "daemon_unavailable",
          body,
          ["focusa_project_verify", "focusa_workpoint_resume", "focusa_tool_doctor"]
        );
      }
      const schema = String(body.packet?.schema_version || "focusa.context_cognition_packet.v1");
      const scopeStatus = String(body.scope_status || "unknown");
      const workpointId = String(body.packet?.scope?.workpoint_id || "none");
      const trajectoryId = String(body.packet?.scope?.trajectory_id || "none");
      const actionAuthority = String(body.packet?.authority?.action_authority || "unknown");
      const evidenceCount = Array.isArray(body.packet?.evidence_refs) ? body.packet.evidence_refs.length : 0;
      const nextTools = Array.isArray(body.next_tools) ? body.next_tools : [];
      const rehydrateId = String(body.rehydrate_id || "ctx_cognition:v0");
      const toolResult =
        body.details?.tool_result_v1 ||
        focusaToolResult({
          ok: true,
          status: "completed",
          summary: `context cognition → ${schema} scope=${scopeStatus}`,
          tool: "focusa_context_cognition",
          family: "trajectory",
          side_effects: [],
          evidence_refs: Array.isArray(body.packet?.evidence_refs) ? body.packet.evidence_refs : [],
          next_tools: nextTools,
          raw: body,
        });
      return {
        content: [
          {
            type: "text",
            text: piToolText({
              kind: "ok",
              tool: "focusa_context_cognition",
              summary: `context cognition → ${schema} scope=${scopeStatus}`,
              ids: [
                { label: "rehydrate_id", value: rehydrateId },
                { label: "workpoint_id", value: workpointId },
                { label: "trajectory_id", value: trajectoryId },
                { label: "action_authority", value: actionAuthority },
              ],
              fields: [
                { label: "schema", value: schema },
                { label: "scope_status", value: scopeStatus },
                { label: "evidence_refs", value: evidenceCount },
                { label: "advisory", value: "true" },
                { label: "canonical", value: "false" },
              ],
              note: "advisory only; never mutates Workpoint or Trajectory",
              nextTools: nextTools.length
                ? nextTools
                : ["focusa_active_object_resolve", "focusa_workpoint_checkpoint"],
            }),
          },
        ],
        details: {
          ok: true,
          status: "completed",
          endpoint: "/v1/context-cognition",
          canonical: false,
          advisory: true,
          project_root: String(projectRoot),
          packet: body.packet || null,
          next_tools: nextTools,
          rehydrate_id: rehydrateId,
          tool_result_v1: toolResult,
        } as any,
      };
    },
  });

  pi.registerTool({
    name: "focusa_context_cognition_render",
    label: "Context Cognition Render",
    description:
      "Render the Spec 100 ContextCognitionPacket as compact text (for prompt/CLI/menubar). Returns bounded lines + the packet's workpoint_id, trajectory_id, and rehydrate_id. Advisory only.",
    promptSnippet:
      "Use when an operator or agent needs a human-readable view of the context cognition packet without parsing JSON.",
    parameters: strictObject({
      project_root: Type.Optional(
        Type.String({ maxLength: 4096, description: "Project root. Defaults to Pi session cwd." })
      ),
      continuity_id: Type.Optional(
        Type.String({ maxLength: 256, description: "Optional continuity id filter." })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_context_cognition_render", params, [
        "project_root",
        "continuity_id",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_context_cognition_render",
          "/v1/context-cognition/render",
          params as Record<string, any>,
          "context cognition render",
          keyCheck.error
        );
      }
      const projectRoot = await resolveFocusaToolProjectRoot((keyCheck.value as any).project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, (keyCheck.value as any).project_root);
      if (projectRootGate) return projectRootGate;
      const query = new URLSearchParams();
      query.set("project_root", String(projectRoot));
      const workingSubpathId = String(process.env.FOCUSA_WORKING_SUBPATH_ID || "").trim();
      if (workingSubpathId) query.set("working_subpath_id", workingSubpathId);
      const cid = (keyCheck.value as any).continuity_id;
      if (typeof cid === "string" && cid.trim() !== "") query.set("continuity_id", cid.trim());
      const res = await focusaFetchDetailed(`/context-cognition/render?${query.toString()}`);
      const body = res.body || {};
      if (!res.ok) {
        return blockedToolResponse(
          "focusa_context_cognition_render",
          "trajectory",
          `context cognition render blocked → ${explainWorkLoopResult(res, "render unavailable")}`,
          body.failure_class || "daemon_unavailable",
          body,
          ["focusa_context_cognition", "focusa_project_verify", "focusa_tool_doctor"]
        );
      }
      const renderText = String(body.render || "");
      const renderLines = Number(body.render_lines || 0);
      const workpointId = String(body.workpoint_id || "none");
      const rehydrateId = String(body.rehydrate_id || "ctx_cognition:v0");
      const toolResult =
        body.details?.tool_result_v1 ||
        focusaToolResult({
          ok: true,
          status: "completed",
          summary: `context cognition render → ${renderLines} lines`,
          tool: "focusa_context_cognition_render",
          family: "trajectory",
          side_effects: [],
          evidence_refs: [],
          next_tools: ["focusa_context_cognition", "focusa_context_cognition_proof"],
          raw: body,
        });
      return {
        content: [
          {
            type: "text",
            text: piToolText({
              kind: "ok",
              tool: "focusa_context_cognition_render",
              summary: `context cognition render → ${renderLines} lines`,
              ids: [
                { label: "rehydrate_id", value: rehydrateId },
                { label: "workpoint_id", value: workpointId },
              ],
              fields: [
                { label: "render_lines", value: renderLines },
                { label: "format", value: "compact_text" },
                { label: "advisory", value: "true" },
              ],
              nextTools: ["focusa_context_cognition", "focusa_context_cognition_proof"],
            }),
          },
          {
            type: "text",
            text: renderText,
          },
        ],
        details: {
          ok: true,
          status: "completed",
          endpoint: "/v1/context-cognition/render",
          canonical: false,
          advisory: true,
          project_root: String(projectRoot),
          render: renderText,
          render_lines: renderLines,
          workpoint_id: workpointId,
          rehydrate_id: rehydrateId,
          tool_result_v1: toolResult,
        } as any,
      };
    },
  });

  pi.registerTool({
    name: "focusa_context_cognition_proof",
    label: "Context Cognition Proof",
    description:
      "Map Spec 100 ContextCognitionPacket surfaces to proof commands (curl + focusa + audits). Returns bounded command list. Read-only.",
    promptSnippet:
      "Use when an operator wants a one-shot proof bundle for the context cognition packet: curl health, project identity, trajectory, workpoint; focusa CLI; node audit scripts.",
    parameters: strictObject({
      project_root: Type.Optional(
        Type.String({ maxLength: 4096, description: "Project root. Defaults to Pi session cwd." })
      ),
      continuity_id: Type.Optional(
        Type.String({ maxLength: 256, description: "Optional continuity id filter." })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_context_cognition_proof", params, [
        "project_root",
        "continuity_id",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_context_cognition_proof",
          "/v1/context-cognition/proof",
          params as Record<string, any>,
          "context cognition proof",
          keyCheck.error
        );
      }
      const projectRoot = await resolveFocusaToolProjectRoot((keyCheck.value as any).project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, (keyCheck.value as any).project_root);
      if (projectRootGate) return projectRootGate;
      const query = new URLSearchParams();
      query.set("project_root", String(projectRoot));
      const workingSubpathId = String(process.env.FOCUSA_WORKING_SUBPATH_ID || "").trim();
      if (workingSubpathId) query.set("working_subpath_id", workingSubpathId);
      const cid = (keyCheck.value as any).continuity_id;
      if (typeof cid === "string" && cid.trim() !== "") query.set("continuity_id", cid.trim());
      const res = await focusaFetchDetailed(`/context-cognition/proof?${query.toString()}`);
      const body = res.body || {};
      if (!res.ok) {
        return blockedToolResponse(
          "focusa_context_cognition_proof",
          "trajectory",
          `context cognition proof blocked → ${explainWorkLoopResult(res, "proof unavailable")}`,
          body.failure_class || "daemon_unavailable",
          body,
          ["focusa_context_cognition", "focusa_project_verify", "focusa_tool_doctor"]
        );
      }
      const commands = Array.isArray(body.proof_commands) ? body.proof_commands : [];
      const commandCount = Number(body.command_count || commands.length);
      const workpointId = String(body.workpoint_id || "none");
      const rehydrateId = String(body.rehydrate_id || "ctx_cognition:v0");
      const toolResult =
        body.details?.tool_result_v1 ||
        focusaToolResult({
          ok: true,
          status: "completed",
          summary: `context cognition proof → ${commandCount} commands`,
          tool: "focusa_context_cognition_proof",
          family: "trajectory",
          side_effects: [],
          evidence_refs: commands.slice(0, 8),
          next_tools: [
            "focusa_context_cognition",
            "focusa_context_cognition_render",
            "focusa_evidence_capture",
          ],
          raw: body,
        });
      const commandList = commands.map((c: unknown, i: number) => `${i + 1}. ${c}`).join("\n");
      return {
        content: [
          {
            type: "text",
            text: piToolText({
              kind: "ok",
              tool: "focusa_context_cognition_proof",
              summary: `context cognition proof → ${commandCount} commands`,
              ids: [
                { label: "rehydrate_id", value: rehydrateId },
                { label: "workpoint_id", value: workpointId },
              ],
              fields: [
                { label: "command_count", value: commandCount },
                { label: "format", value: "proof_commands" },
                { label: "advisory", value: "true" },
              ],
              nextTools: [
                "focusa_context_cognition",
                "focusa_context_cognition_render",
                "focusa_evidence_capture",
              ],
            }),
          },
          {
            type: "text",
            text: commandList,
          },
        ],
        details: {
          ok: true,
          status: "completed",
          endpoint: "/v1/context-cognition/proof",
          canonical: false,
          advisory: true,
          project_root: String(projectRoot),
          proof_commands: commands,
          command_count: commandCount,
          workpoint_id: workpointId,
          rehydrate_id: rehydrateId,
          tool_result_v1: toolResult,
        } as any,
      };
    },
  });

  pi.registerTool({
    name: "focusa_context_cognition_curate",
    label: "Context Cognition Curate",
    description:
      "Spec 100 Phase 3 — token-budgeted context selection. Takes candidates (files/docs/diffs/snippets/codemaps/evidence) and selects the highest-scoring subset under a token budget. Returns selected_context + excluded_context (with reasons).",
    promptSnippet:
      "Use when the agent or operator has a candidate list (files, docs, diffs, evidence) and needs a token-budgeted selection that maximizes relevance to a target (workpoint next_slice, mission, or query).",
    parameters: strictObject({
      project_root: Type.Optional(
        Type.String({ maxLength: 4096, description: "Project root. Defaults to Pi session cwd." })
      ),
      continuity_id: Type.Optional(
        Type.String({ maxLength: 256, description: "Optional continuity id filter." })
      ),
      target: Type.Optional(
        Type.String({
          description:
            "Curator target string (workpoint next_slice, mission, query). Defaults to the active workpoint's next_slice/mission.",
        })
      ),
      token_budget: Type.Optional(
        Type.Integer({
          minimum: 1,
          maximum: 1000000,
          description: "Token budget for the selection. Defaults to 2000.",
        })
      ),
      candidates: Type.Optional(
        Type.Array(
          Type.Object({
            kind: Type.String({
              description: "Candidate kind: file | doc | diff | snippet | codemap | evidence.",
            }),
            path: Type.String({ description: "Candidate path or ref id." }),
            body: Type.Optional(
              Type.String({
                description: "Optional candidate body; tokens are estimated from word count if absent.",
              })
            ),
            evidence_ref: Type.Optional(
              Type.String({
                description:
                  "Optional evidence ref id; curator boosts candidates matching the supplied evidence_refs.",
              })
            ),
            tokens: Type.Optional(
              Type.Integer({
                minimum: 0,
                description: "Optional explicit token count; overrides body-derived estimate.",
              })
            ),
          }),
          {
            description:
              "Candidates to curate. Each is a {kind, path, body?, evidence_ref?, tokens?} object.",
          }
        )
      ),
      evidence_refs: Type.Optional(
        Type.Array(Type.String(), { description: "Evidence refs that boost candidate ranking when matched." })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_context_cognition_curate", params, [
        "project_root",
        "continuity_id",
        "target",
        "token_budget",
        "candidates",
        "evidence_refs",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_context_cognition_curate",
          "/v1/context-cognition/curate",
          params as Record<string, any>,
          "context cognition curate",
          keyCheck.error
        );
      }
      const projectRoot = await resolveFocusaToolProjectRoot((keyCheck.value as any).project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, (keyCheck.value as any).project_root);
      if (projectRootGate) return projectRootGate;
      const body: Record<string, any> = {
        project_root: String(projectRoot),
        continuity_id: (keyCheck.value as any).continuity_id ?? null,
        target: (keyCheck.value as any).target ?? null,
        token_budget: (keyCheck.value as any).token_budget ?? 2000,
        candidates: Array.isArray((keyCheck.value as any).candidates)
          ? (keyCheck.value as any).candidates
          : [],
        evidence_refs: Array.isArray((keyCheck.value as any).evidence_refs)
          ? (keyCheck.value as any).evidence_refs
          : [],
      };
      const res = await focusaFetchDetailed("/context-cognition/curate", {
        method: "POST",
        body: JSON.stringify(body),
      });
      const resp = res.body || {};
      if (!res.ok) {
        return blockedToolResponse(
          "focusa_context_cognition_curate",
          "trajectory",
          `context cognition curate blocked → ${explainWorkLoopResult(res, "curate unavailable")}`,
          resp.failure_class || "daemon_unavailable",
          resp,
          ["focusa_context_cognition", "focusa_project_verify", "focusa_tool_doctor"]
        );
      }
      const selectedCount = Number(resp.selected_count || 0);
      const excludedCount = Number(resp.excluded_count || 0);
      const tokensUsed = Number(resp.tokens_used || 0);
      const tokenBudget = Number(resp.token_budget || 0);
      const target = String(resp.target || "<none>");
      const rehydrate = String(resp.rehydrate_id || "ctx_curate:v0");
      const toolResult =
        resp.details?.tool_result_v1 ||
        focusaToolResult({
          ok: true,
          status: "completed",
          summary: `context cognition curate → selected=${selectedCount} excluded=${excludedCount}`,
          tool: "focusa_context_cognition_curate",
          family: "trajectory",
          side_effects: [],
          evidence_refs: Array.isArray(resp.evidence_refs) ? resp.evidence_refs : [],
          next_tools: [
            "focusa_context_cognition",
            "focusa_context_cognition_render",
            "focusa_evidence_capture",
          ],
          raw: resp,
        });
      return {
        content: [
          {
            type: "text",
            text: piToolText({
              kind: "ok",
              tool: "focusa_context_cognition_curate",
              summary: `context cognition curate → selected=${selectedCount} excluded=${excludedCount}`,
              ids: [
                { label: "rehydrate_id", value: rehydrate },
                { label: "target", value: target },
              ],
              fields: [
                { label: "selected_count", value: selectedCount },
                { label: "excluded_count", value: excludedCount },
                { label: "tokens_used", value: tokensUsed },
                { label: "token_budget", value: tokenBudget },
                { label: "tokens_remaining", value: Number(resp.tokens_remaining || 0) },
                { label: "advisory", value: "true" },
              ],
              nextTools: [
                "focusa_context_cognition",
                "focusa_context_cognition_render",
                "focusa_evidence_capture",
              ],
            }),
          },
        ],
        details: {
          ok: true,
          status: "completed",
          endpoint: "/v1/context-cognition/curate",
          canonical: false,
          advisory: true,
          project_root: String(projectRoot),
          target,
          token_budget: tokenBudget,
          tokens_used: tokensUsed,
          selected_count: selectedCount,
          excluded_count: excludedCount,
          selected_context: resp.selected_context || [],
          excluded_context: resp.excluded_context || [],
          rehydrate_id: rehydrate,
          tool_result_v1: toolResult,
        } as any,
      };
    },
  });

  pi.registerTool({
    name: "focusa_device_pair_start",
    label: "Device Pair Start",
    description:
      "Mac menubar OAuth-like device pairing (Spec focusa-ui0y). Generate an 8-char pairing code (FOCUS-XXXX-XXXX, 5 min TTL). The operator runs `focusa device pair-complete <code>` on their VPS, then the Mac app polls focusa_device_pair_status to retrieve the long-lived token (30 day TTL).",
    promptSnippet:
      "Use when the operator wants to connect the Focusa Mac menubar app to a remote Focusa daemon. Generates a pairing code; the operator runs the corresponding focusa device pair-complete on the VPS to mint a token.",
    parameters: strictObject({
      device_name: Type.Optional(
        Type.String({
          maxLength: 256,
          description:
            "Human-readable device name (e.g. 'operator-macbook-pro'). Defaults to 'operator-device'.",
        })
      ),
      platform: Type.Optional(Type.String({ description: "Platform string. Default: 'macos'." })),
      daemon_base_url: Type.Optional(
        Type.String({
          description: "Daemon base URL the device will reconnect to. Default: 'http://127.0.0.1:8787'.",
        })
      ),
      scopes: Type.Optional(
        Type.Array(Type.String(), { description: "OAuth-like scopes. Default: ['read', 'write']." })
      ),
    }),
    async execute(_id, params) {
      const body = {
        device_name: params.device_name,
        platform: params.platform ?? "macos",
        daemon_base_url: params.daemon_base_url ?? "http://127.0.0.1:8787",
        scopes: Array.isArray(params.scopes) && params.scopes.length ? params.scopes : ["read", "write"],
      };
      const res = await focusaFetchDetailed("/device/pair/start", {
        method: "POST",
        body: JSON.stringify(body),
      });
      const resp = res.body || {};
      if (!res.ok) {
        return blockedToolResponse(
          "focusa_device_pair_start",
          "session_transfer",
          `device pair start blocked → ${explainWorkLoopResult(res, "pair start unavailable")}`,
          resp.failure_class || "daemon_unavailable",
          resp,
          ["focusa_project_verify", "focusa_tool_doctor"]
        );
      }
      const code = String(resp.code || "none");
      const deviceId = String(resp.device_id || "none");
      const expiresIn = Number(resp.expires_in_secs || 0);
      const rehydrate = String(resp.rehydrate_id || code);
      const toolResult =
        resp.details?.tool_result_v1 ||
        focusaToolResult({
          ok: true,
          status: "completed",
          summary: `device pair start → code=${code} device_id=${deviceId} expires_in=${expiresIn}s`,
          tool: "focusa_device_pair_start",
          family: "session_transfer",
          side_effects: ["device_pair_start_append"],
          evidence_refs: [code],
          next_tools: ["focusa_device_pair_status", "focusa_device_pair_list"],
          raw: resp,
        });
      const handoff = resp.operator_handoff || {};
      return {
        content: [
          {
            type: "text",
            text: piToolText({
              kind: "ok",
              tool: "focusa_device_pair_start",
              summary: `device pair start → code=${code} device_id=${deviceId} expires_in=${expiresIn}s`,
              ids: [
                { label: "code", value: code },
                { label: "device_id", value: deviceId },
                { label: "rehydrate_id", value: rehydrate },
              ],
              fields: [
                { label: "expires_in_secs", value: expiresIn },
                { label: "platform", value: String(body.platform) },
                { label: "on_your_vps_run", value: String(handoff.on_your_vps_run || "") },
                { label: "advisory", value: "true" },
              ],
              note: "mac app: show the code to the operator; they run the on_your_vps_run command on their VPS; mac app polls focusa_device_pair_status until completed; then store token in Keychain and reconnect.",
              nextTools: ["focusa_device_pair_status", "focusa_device_pair_list"],
            }),
          },
        ],
        details: {
          ok: true,
          status: "completed",
          endpoint: "/v1/device/pair/start",
          canonical: false,
          advisory: true,
          device_id: deviceId,
          code,
          rehydrate_id: rehydrate,
          tool_result_v1: toolResult,
        } as any,
      };
    },
  });

  // focusa-ui0y.12: QR-pairing shortcut. Same as pair_start but
  // emphasizes pair_url for QR handoff. Returns the payload the Mac
  // app renders as a QR (Telegram/Discord-style).
  pi.registerTool({
    name: "focusa_device_pair_qr",
    label: "Device Pair QR",
    description:
      "Mac menubar OAuth-like device pairing with QR handoff (Spec focusa-ui0y, Mode B). Calls /v1/device/pair/start and returns pair_url + pair_url_qr_payload prominently so the Mac menubar can render a QR the operator's phone can scan.",
    promptSnippet:
      "Use when the operator wants the Mac to display a QR for phone-based pairing. Requires FOCUSA_PAIRING_URL set on the VPS to a public URL, otherwise pair_url falls back to daemon_base_url (local-only).",
    parameters: strictObject({
      device_name: Type.Optional(
        Type.String({
          maxLength: 256,
          description:
            "Human-readable device name (e.g. 'operator-macbook-pro'). Defaults to 'operator-device'.",
        })
      ),
      platform: Type.Optional(Type.String({ description: "Platform string. Default: 'macos'." })),
      daemon_base_url: Type.Optional(
        Type.String({
          description: "Daemon base URL the device will reconnect to. Default: 'http://127.0.0.1:8787'.",
        })
      ),
      scopes: Type.Optional(
        Type.Array(Type.String(), { description: "OAuth-like scopes. Default: ['read', 'write']." })
      ),
    }),
    async execute(_id, params) {
      const p = params as any;
      const body: Record<string, unknown> = {
        device_name: p.device_name ?? "operator-device",
        platform: p.platform ?? "macos",
        daemon_base_url: p.daemon_base_url ?? "http://127.0.0.1:8787",
        scopes: p.scopes ?? ["read", "write"],
      };
      const resp = await focusaFetch("/device/pair/start", {
        method: "POST",
        body: JSON.stringify(body),
      });
      const code = String(resp.code || "");
      const deviceId = String(resp.device_id || "");
      const pairUrl = String(resp.pair_url || "");
      const pairUrlQrPayload = String(resp.pair_url_qr_payload || pairUrl);
      const expiresIn = Number(resp.expires_in_secs || 0);
      const rehydrate = `pair_qr:${deviceId}`;
      const toolResult = focusaToolResult({
        ok: true,
        status: "completed",
        summary: `device pair qr → code=${code} pair_url=${pairUrl}`,
        tool: "focusa_device_pair_qr",
        family: "session_transfer",
        side_effects: ["pair_code_generated"],
        evidence_refs: [code, pairUrl],
        next_tools: ["focusa_device_pair_status", "focusa_device_pair_list"],
        raw: resp,
      });
      return {
        content: [
          {
            type: "text",
            text: piToolText({
              kind: "ok",
              tool: "focusa_device_pair_qr",
              summary: `device pair qr → code=${code} pair_url=${pairUrl} expires_in=${expiresIn}s`,
              ids: [
                { label: "code", value: code },
                { label: "device_id", value: deviceId },
                { label: "pair_url", value: pairUrl },
                { label: "pair_url_qr_payload", value: pairUrlQrPayload },
                { label: "rehydrate_id", value: rehydrate },
              ],
              fields: [
                { label: "expires_in_secs", value: expiresIn },
                { label: "qr_payload", value: pairUrlQrPayload },
                { label: "advisory", value: "true" },
              ],
              note: "mac app: render pair_url as a QR. Operator scans with phone, opens the focusa-pairing PWA helper at /pair/{device_id}, taps 'Complete on this VPS' to finish pairing. The Mac app then polls focusa_device_pair_status until completed and stores the token in Keychain.",
              nextTools: ["focusa_device_pair_status", "focusa_device_pair_list"],
            }),
          },
        ],
        details: {
          ok: true,
          status: "completed",
          endpoint: "/v1/device/pair/start",
          canonical: false,
          advisory: true,
          device_id: deviceId,
          code,
          pair_url: pairUrl,
          pair_url_qr_payload: pairUrlQrPayload,
          rehydrate_id: rehydrate,
          tool_result_v1: toolResult,
        } as any,
      };
    },
  });

  pi.registerTool({
    name: "focusa_device_pair_complete",
    label: "Device Pair Complete",
    description:
      "Complete a pending pairing (run on the VPS side; returns the long-lived token). Idempotent: re-running with the same code returns the original token.",
    promptSnippet:
      "Use on the VPS side to complete a pending pairing initiated by focusa_device_pair_start. Returns the long-lived token that the Mac app will use for subsequent calls.",
    parameters: strictObject({
      code: Type.String({ description: "The FOCUS-XXXX-XXXX code from focusa_device_pair_start." }),
      host: Type.Optional(Type.String({ description: "Host label (default: 'operator-host')." })),
      operator_id: Type.Optional(Type.String({ description: "Operator id (e.g. 'verious')." })),
      completed_by: Type.Optional(
        Type.String({ description: "Who/what completed the pairing. Default: 'vps-cli'." })
      ),
    }),
    async execute(_id, params) {
      const body = {
        code: String(params.code || "")
          .trim()
          .toUpperCase(),
        host: params.host ?? "operator-host",
        operator_id: params.operator_id ?? null,
        completed_by: params.completed_by ?? "vps-cli",
      };
      const res = await focusaFetchDetailed("/device/pair/complete", {
        method: "POST",
        body: JSON.stringify(body),
      });
      const resp = res.body || {};
      if (!res.ok) {
        return blockedToolResponse(
          "focusa_device_pair_complete",
          "session_transfer",
          `device pair complete blocked → ${explainWorkLoopResult(res, "pair complete unavailable")}`,
          resp.failure_class || "daemon_unavailable",
          resp,
          ["focusa_device_pair_status", "focusa_device_pair_list"]
        );
      }
      const token = String(resp.token || "none");
      const deviceId = String(resp.device_id || "none");
      const rehydrate = String(resp.rehydrate_id || deviceId);
      const toolResult =
        resp.details?.tool_result_v1 ||
        focusaToolResult({
          ok: true,
          status: "completed",
          summary: `device pair complete → token=${token}`,
          tool: "focusa_device_pair_complete",
          family: "session_transfer",
          side_effects: ["device_pair_complete_ledger_append", "device_token_issue"],
          evidence_refs: [deviceId],
          next_tools: ["focusa_device_pair_status", "focusa_device_pair_list"],
          raw: resp,
        });
      return {
        content: [
          {
            type: "text",
            text: piToolText({
              kind: "ok",
              tool: "focusa_device_pair_complete",
              summary: `device pair complete → token issued for device_id=${deviceId}`,
              ids: [
                { label: "device_id", value: deviceId },
                { label: "rehydrate_id", value: rehydrate },
                { label: "token", value: token },
              ],
              fields: [
                { label: "host", value: String(body.host) },
                { label: "operator_id", value: String(params.operator_id || "n/a") },
                { label: "token_ttl_secs", value: Number(resp.token_ttl_secs || 0) },
                { label: "advisory", value: "true" },
              ],
              note: "mac app: the on_your_vps_run response is for the operator; the mac app reads the token from focusa_device_pair_status after the operator runs this command on the VPS.",
              nextTools: ["focusa_device_pair_status", "focusa_device_pair_list"],
            }),
          },
        ],
        details: {
          ok: true,
          status: "completed",
          endpoint: "/v1/device/pair/complete",
          canonical: false,
          advisory: true,
          device_id: deviceId,
          token,
          rehydrate_id: rehydrate,
          tool_result_v1: toolResult,
        } as any,
      };
    },
  });

  pi.registerTool({
    name: "focusa_device_pair_status",
    label: "Device Pair Status",
    description:
      "Check the status of a pending or completed pairing by code OR by device_id. Returns the token (when completed) + status + scopes + expires_at.",
    promptSnippet:
      "Use to poll whether a focusa_device_pair_start code has been completed by the VPS, or to look up the long-lived token for a known device_id.",
    parameters: strictObject({
      code: Type.Optional(Type.String({ description: "Pairing code (mutually exclusive with device_id)." })),
      device_id: Type.Optional(Type.String({ description: "Device id (mutually exclusive with code)." })),
    }),
    async execute(_id, params) {
      if (!params.code && !params.device_id) {
        return spec80ValidationResult(
          "focusa_device_pair_status",
          "/v1/device/pair/status",
          params as Record<string, any>,
          "device pair status",
          "--code or --device-id required"
        );
      }
      const q = new URLSearchParams();
      if (params.code) q.set("code", String(params.code).trim().toUpperCase());
      if (params.device_id) q.set("device_id", String(params.device_id));
      const res = await focusaFetchDetailed(`/device/pair/status?${q.toString()}`);
      const body = res.body || {};
      if (!res.ok) {
        return blockedToolResponse(
          "focusa_device_pair_status",
          "session_transfer",
          `device pair status blocked → ${explainWorkLoopResult(res, "pair status unavailable")}`,
          body.failure_class || "daemon_unavailable",
          body,
          ["focusa_device_pair_start", "focusa_device_pair_list"]
        );
      }
      const status = String(body.status || "unknown");
      const token = body.token || "none";
      const deviceId = String(body.device_id || params.code || params.device_id || "none");
      const rehydrate = String(body.rehydrate_id || deviceId);
      const toolResult =
        body.details?.tool_result_v1 ||
        focusaToolResult({
          ok: true,
          status: "completed",
          summary: `device pair status → status=${status} token=${typeof token === "string" ? token.slice(0, 8) + "..." : "n/a"}`,
          tool: "focusa_device_pair_status",
          family: "session_transfer",
          side_effects: [],
          evidence_refs: typeof token === "string" ? [token] : [],
          next_tools: ["focusa_device_pair_list", "focusa_device_pair_revoke"],
          raw: body,
        });
      return {
        content: [
          {
            type: "text",
            text: piToolText({
              kind: "ok",
              tool: "focusa_device_pair_status",
              summary: `device pair status → status=${status}`,
              ids: [
                { label: "device_id", value: deviceId },
                { label: "rehydrate_id", value: rehydrate },
              ],
              fields: [
                { label: "status", value: status },
                { label: "token", value: typeof token === "string" ? token.slice(0, 8) + "..." : "n/a" },
                { label: "expired", value: body.expired === true ? "yes" : "no" },
                { label: "advisory", value: "true" },
              ],
              nextTools: ["focusa_device_pair_list", "focusa_device_pair_revoke"],
            }),
          },
        ],
        details: {
          ok: true,
          status: "completed",
          endpoint: "/v1/device/pair/status",
          canonical: false,
          advisory: true,
          device_id: deviceId,
          pair_status: status,
          rehydrate_id: rehydrate,
          tool_result_v1: toolResult,
        } as any,
      };
    },
  });

  pi.registerTool({
    name: "focusa_device_pair_list",
    label: "Device Pair List",
    description:
      "List paired devices for a host (append-only JSONL ledger, scope-bounded). Returns the recent device list with name, scopes, paired_at, last_seen_at, revoked.",
    promptSnippet:
      "Use to see which devices are currently paired with this Focusa daemon, and which have been revoked.",
    parameters: strictObject({
      host: Type.Optional(Type.String({ description: "Host label (default: 'operator-host')." })),
      limit: Type.Optional(
        Type.Integer({ minimum: 1, maximum: 200, description: "Max records to return. Default: 50." })
      ),
    }),
    async execute(_id, params) {
      const host = params.host ?? "operator-host";
      const limit = Math.max(1, Math.min(200, Number(params.limit ?? 50)));
      const res = await focusaFetchDetailed(
        `/device/pair/list?host=${encodeURIComponent(host)}&limit=${limit}`
      );
      const body = res.body || {};
      if (!res.ok) {
        return blockedToolResponse(
          "focusa_device_pair_list",
          "session_transfer",
          `device pair list blocked → ${explainWorkLoopResult(res, "pair list unavailable")}`,
          body.failure_class || "daemon_unavailable",
          body,
          ["focusa_tool_doctor"]
        );
      }
      const devices = Array.isArray(body.devices) ? body.devices : [];
      const rehydrate = String(body.rehydrate_id || "no_devices");
      const toolResult =
        body.details?.tool_result_v1 ||
        focusaToolResult({
          ok: true,
          status: "completed",
          summary: `device pair list → count=${devices.length} host=${host}`,
          tool: "focusa_device_pair_list",
          family: "session_transfer",
          side_effects: [],
          evidence_refs: [],
          next_tools: ["focusa_device_pair_revoke", "focusa_session_transfer"],
          raw: body,
        });
      return {
        content: [
          {
            type: "text",
            text: piToolText({
              kind: "ok",
              tool: "focusa_device_pair_list",
              summary: `device pair list → count=${devices.length} host=${host}`,
              ids: [{ label: "rehydrate_id", value: rehydrate }],
              fields: [
                { label: "count", value: devices.length },
                { label: "host", value: host },
                { label: "advisory", value: "true" },
              ],
              nextTools: ["focusa_device_pair_revoke", "focusa_session_transfer"],
            }),
          },
          {
            type: "text",
            text:
              devices
                .slice(0, 5)
                .map(
                  (d: any) =>
                    `  - ${d.device_id || "?"} name=${d.name || "?"} revoked=${d.revoked === true ? "yes" : "no"}`
                )
                .join("\n") || "(no devices)",
          },
        ],
        details: {
          ok: true,
          status: "completed",
          endpoint: "/v1/device/pair/list",
          canonical: false,
          advisory: true,
          host,
          count: devices.length,
          devices,
          rehydrate_id: rehydrate,
          tool_result_v1: toolResult,
        } as any,
      };
    },
  });

  pi.registerTool({
    name: "focusa_device_pair_revoke",
    label: "Device Pair Revoke",
    description:
      "Revoke a paired device. Appends a new entry with revoked=true to the append-only JSONL ledger and removes the in-memory token. The next call from the device will be rejected with status=revoked.",
    promptSnippet:
      "Use to remove a paired device (lost laptop, rotation, security incident). The device will need to re-pair.",
    parameters: strictObject({
      device_id: Type.String({ description: "Device id to revoke." }),
      host: Type.Optional(Type.String({ description: "Host label (default: 'operator-host')." })),
      reason: Type.Optional(
        Type.String({ description: "Optional human-readable reason (audit). Stored in the ledger." })
      ),
    }),
    async execute(_id, params) {
      if (!params.device_id) {
        return spec80ValidationResult(
          "focusa_device_pair_revoke",
          "/v1/device/pair/revoke",
          params as Record<string, any>,
          "device pair revoke",
          "--device-id required"
        );
      }
      const body = {
        device_id: String(params.device_id),
        host: params.host ?? "operator-host",
        reason: params.reason ?? null,
      };
      const res = await focusaFetchDetailed("/device/pair/revoke", {
        method: "POST",
        body: JSON.stringify(body),
      });
      const resp = res.body || {};
      if (!res.ok) {
        return blockedToolResponse(
          "focusa_device_pair_revoke",
          "session_transfer",
          `device pair revoke blocked → ${explainWorkLoopResult(res, "pair revoke unavailable")}`,
          resp.failure_class || "daemon_unavailable",
          resp,
          ["focusa_device_pair_list", "focusa_tool_doctor"]
        );
      }
      const deviceId = String(resp.device_id || body.device_id);
      const rehydrate = String(resp.rehydrate_id || deviceId);
      const appended = resp.ledger_appended === true;
      const toolResult =
        resp.details?.tool_result_v1 ||
        focusaToolResult({
          ok: true,
          status: "completed",
          summary: `device pair revoke → device_id=${deviceId} ledger_appended=${appended}`,
          tool: "focusa_device_pair_revoke",
          family: "session_transfer",
          side_effects: ["device_pair_revoke_ledger_append", "in_memory_token_invalidate"],
          evidence_refs: [deviceId],
          next_tools: ["focusa_device_pair_list"],
          raw: resp,
        });
      return {
        content: [
          {
            type: "text",
            text: piToolText({
              kind: "ok",
              tool: "focusa_device_pair_revoke",
              summary: `device pair revoke → device_id=${deviceId} ledger_appended=${appended}`,
              ids: [
                { label: "device_id", value: deviceId },
                { label: "rehydrate_id", value: rehydrate },
              ],
              fields: [
                { label: "ledger_appended", value: appended ? "yes" : "no" },
                { label: "host", value: String(body.host) },
                { label: "reason", value: String(params.reason || "n/a") },
                { label: "advisory", value: "true" },
              ],
              nextTools: ["focusa_device_pair_list"],
            }),
          },
        ],
        details: {
          ok: true,
          status: "completed",
          endpoint: "/v1/device/pair/revoke",
          canonical: false,
          advisory: true,
          device_id: deviceId,
          rehydrate_id: rehydrate,
          ledger_appended: appended,
          tool_result_v1: toolResult,
        } as any,
      };
    },
  });

  pi.registerTool({
    name: "focusa_context_cognition_curate_eval",
    label: "Context Cognition Curate Eval",
    description:
      "Spec 100 Phase 4 — run a curator eval case. Computes precision/recall/F1 vs. expected_selected_paths. Appends to curator-eval-ledger/{hash}/eval-runs.jsonl. Returns run_id, eval_ref, scores, and promoted flag (F1 > baseline_f1 AND F1 >= score_threshold).",
    promptSnippet:
      "Use when measuring whether the curator's selection matches an operator's expected selection. Captures the result as a focusa_metacog_capture lesson and a focusa_predict_record prediction.",
    parameters: strictObject({
      project_root: Type.Optional(
        Type.String({ maxLength: 4096, description: "Project root. Defaults to Pi session cwd." })
      ),
      continuity_id: Type.Optional(
        Type.String({ maxLength: 256, description: "Optional continuity id filter." })
      ),
      case_id: Type.Optional(Type.String({ description: "Optional case id; defaults to a generated UUID." })),
      target: Type.Optional(Type.String({ description: "Curator target string." })),
      token_budget: Type.Optional(
        Type.Integer({
          minimum: 1,
          maximum: 1000000,
          description: "Token budget for the selection. Defaults to 2000.",
        })
      ),
      candidates: Type.Optional(
        Type.Array(
          Type.Object({
            kind: Type.String(),
            path: Type.String(),
            body: Type.Optional(Type.String()),
            evidence_ref: Type.Optional(Type.String()),
            tokens: Type.Optional(Type.Integer()),
          })
        )
      ),
      expected_selected_paths: Type.Optional(
        Type.Array(Type.String(), {
          description: "Operator-supplied expected selected paths for precision/recall/F1.",
        })
      ),
      score_threshold: Type.Optional(
        Type.Number({ minimum: 0, maximum: 1, description: "F1 threshold for promotion. Defaults to 0.5." })
      ),
      baseline_f1: Type.Optional(
        Type.Number({ minimum: 0, maximum: 1, description: "Baseline F1 to beat. Defaults to 0.0." })
      ),
      evidence_refs: Type.Optional(Type.Array(Type.String())),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_context_cognition_curate_eval", params, [
        "project_root",
        "continuity_id",
        "case_id",
        "target",
        "token_budget",
        "candidates",
        "expected_selected_paths",
        "score_threshold",
        "baseline_f1",
        "evidence_refs",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_context_cognition_curate_eval",
          "/v1/context-cognition/curate/eval",
          params as Record<string, any>,
          "context cognition curate eval",
          keyCheck.error
        );
      }
      const projectRoot = await resolveFocusaToolProjectRoot((keyCheck.value as any).project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, (keyCheck.value as any).project_root);
      if (projectRootGate) return projectRootGate;
      const body: Record<string, any> = {
        project_root: String(projectRoot),
        continuity_id: (keyCheck.value as any).continuity_id ?? null,
        case_id: (keyCheck.value as any).case_id ?? null,
        target: (keyCheck.value as any).target ?? null,
        token_budget: (keyCheck.value as any).token_budget ?? 2000,
        candidates: Array.isArray((keyCheck.value as any).candidates)
          ? (keyCheck.value as any).candidates
          : [],
        expected_selected_paths: Array.isArray((keyCheck.value as any).expected_selected_paths)
          ? (keyCheck.value as any).expected_selected_paths
          : [],
        score_threshold: (keyCheck.value as any).score_threshold ?? 0.5,
        baseline_f1: (keyCheck.value as any).baseline_f1 ?? 0.0,
        evidence_refs: Array.isArray((keyCheck.value as any).evidence_refs)
          ? (keyCheck.value as any).evidence_refs
          : [],
      };
      const res = await focusaFetchDetailed("/context-cognition/curate/eval", {
        method: "POST",
        body: JSON.stringify(body),
      });
      const resp = res.body || {};
      if (!res.ok) {
        return blockedToolResponse(
          "focusa_context_cognition_curate_eval",
          "trajectory",
          `context cognition curate eval blocked → ${explainWorkLoopResult(res, "curate eval unavailable")}`,
          resp.failure_class || "daemon_unavailable",
          resp,
          ["focusa_context_cognition", "focusa_project_verify", "focusa_tool_doctor"]
        );
      }
      const runId = String(resp.run_id || "none");
      const evalRef = String(resp.eval_ref || "none");
      const f1 = Number(resp.f1 || 0);
      const precision = Number(resp.precision || 0);
      const recall = Number(resp.recall || 0);
      const baselineF1 = Number(resp.baseline_f1 || 0);
      const promovido = Boolean(resp.promoted);
      const tokensUsed = Number(resp.tokens_used || 0);
      const toolResult =
        resp.details?.tool_result_v1 ||
        focusaToolResult({
          ok: true,
          status: "completed",
          summary: `context cognition curate eval → f1=${f1.toFixed(2)} promoted=${promovido ? "yes" : "no"}`,
          tool: "focusa_context_cognition_curate_eval",
          family: "trajectory",
          side_effects: ["curator_eval_append"],
          evidence_refs: [evalRef],
          next_tools: [
            "focusa_context_cognition_curate_optimize",
            "focusa_metacog_capture",
            "focusa_predict_record",
          ],
          raw: resp,
        });
      return {
        content: [
          {
            type: "text",
            text: piToolText({
              kind: "ok",
              tool: "focusa_context_cognition_curate_eval",
              summary: `context cognition curate eval → f1=${f1.toFixed(2)} promoted=${promovido ? "yes" : "no"}`,
              ids: [
                { label: "run_id", value: runId },
                { label: "eval_ref", value: evalRef },
                { label: "rehydrate_id", value: runId },
              ],
              fields: [
                { label: "f1", value: Number(f1.toFixed(3)) },
                { label: "precision", value: Number(precision.toFixed(3)) },
                { label: "recall", value: Number(recall.toFixed(3)) },
                { label: "baseline_f1", value: Number(baselineF1.toFixed(3)) },
                { label: "tokens_used", value: tokensUsed },
                { label: "promoted", value: promovido ? "yes" : "no" },
                { label: "advisory", value: "true" },
              ],
              nextTools: [
                "focusa_context_cognition_curate_optimize",
                "focusa_metacog_capture",
                "focusa_predict_record",
              ],
            }),
          },
        ],
        details: {
          ok: true,
          status: "completed",
          endpoint: "/v1/context-cognition/curate/eval",
          canonical: false,
          advisory: true,
          project_root: String(projectRoot),
          run_id: runId,
          eval_ref: evalRef,
          precision,
          recall,
          f1,
          baseline_f1: baselineF1,
          tokens_used: tokensUsed,
          promoted: promovido,
          tool_result_v1: toolResult,
        } as any,
      };
    },
  });

  pi.registerTool({
    name: "focusa_context_cognition_curate_optimize",
    label: "Context Cognition Curate Optimize",
    description:
      "Spec 100 Phase 5 — submit a Cognition Optimizer artifact and get the promote/rollback decision. Returns the decision per the §15 promotion rule (eval_score > baseline_score AND eval_score >= score_threshold). Appends to cognition-optimizer-artifacts/{hash}/artifacts.jsonl.",
    promptSnippet:
      "Use after focusa_context_cognition_curate_eval when the operator has a candidate prompt/module artifact and wants the curator's promotion/rollback decision.",
    parameters: strictObject({
      project_root: Type.Optional(
        Type.String({ maxLength: 4096, description: "Project root. Defaults to Pi session cwd." })
      ),
      continuity_id: Type.Optional(
        Type.String({ maxLength: 256, description: "Optional continuity id filter." })
      ),
      module_name: Type.Optional(Type.String({ description: "Module name (default: curator)." })),
      prompt_artifact_ref: Type.String({
        minLength: 1,
        maxLength: 4096,
        description: "Path or ref id of the candidate prompt/module artifact.",
      }),
      eval_score: Type.Number({ minimum: 0, maximum: 1, description: "Candidate artifact's eval F1 score." }),
      baseline_score: Type.Optional(
        Type.Number({ minimum: 0, maximum: 1, description: "Baseline F1 to beat. Defaults to 0.0." })
      ),
      score_threshold: Type.Optional(
        Type.Number({ minimum: 0, maximum: 1, description: "F1 threshold for promotion. Defaults to 0.5." })
      ),
      eval_run_id: Type.Optional(
        Type.String({ description: "Optional CuratorEvalRun id that produced eval_score." })
      ),
      rollback: Type.Optional(
        Type.Boolean({ description: "Explicit rollback override. Defaults to false." })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_context_cognition_curate_optimize", params, [
        "project_root",
        "continuity_id",
        "module_name",
        "prompt_artifact_ref",
        "eval_score",
        "baseline_score",
        "score_threshold",
        "eval_run_id",
        "rollback",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_context_cognition_curate_optimize",
          "/v1/context-cognition/curate/optimize",
          params as Record<string, any>,
          "context cognition curate optimize",
          keyCheck.error
        );
      }
      const projectRoot = await resolveFocusaToolProjectRoot((keyCheck.value as any).project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, (keyCheck.value as any).project_root);
      if (projectRootGate) return projectRootGate;
      const body: Record<string, any> = {
        project_root: String(projectRoot),
        continuity_id: (keyCheck.value as any).continuity_id ?? null,
        module_name: (keyCheck.value as any).module_name ?? "curator",
        prompt_artifact_ref: (keyCheck.value as any).prompt_artifact_ref,
        eval_score: (keyCheck.value as any).eval_score,
        baseline_score: (keyCheck.value as any).baseline_score ?? 0.0,
        score_threshold: (keyCheck.value as any).score_threshold ?? 0.5,
        eval_run_id: (keyCheck.value as any).eval_run_id ?? null,
        rollback: Boolean((keyCheck.value as any).rollback),
      };
      const res = await focusaFetchDetailed("/context-cognition/curate/optimize", {
        method: "POST",
        body: JSON.stringify(body),
      });
      const resp = res.body || {};
      if (!res.ok) {
        return blockedToolResponse(
          "focusa_context_cognition_curate_optimize",
          "trajectory",
          `context cognition curate optimize blocked → ${explainWorkLoopResult(res, "curate optimize unavailable")}`,
          resp.failure_class || "daemon_unavailable",
          resp,
          ["focusa_context_cognition_curate_eval", "focusa_project_verify", "focusa_tool_doctor"]
        );
      }
      const artifactId = String(resp.artifact_id || "none");
      const decision = String(resp.decision || "unknown");
      const evalScore = Number(resp.eval_score || 0);
      const baselineScore = Number(resp.baseline_score || 0);
      const promovido = Boolean(resp.promoted);
      const rollbackRef = String(resp.rollback_ref || "none");
      const toolResult =
        resp.details?.tool_result_v1 ||
        focusaToolResult({
          ok: true,
          status: "completed",
          summary: `context cognition curate optimize → decision=${decision} promoted=${promovido ? "yes" : "no"}`,
          tool: "focusa_context_cognition_curate_optimize",
          family: "trajectory",
          side_effects: ["cognition_optimizer_artifact_append"],
          evidence_refs: [artifactId],
          next_tools: [
            "focusa_context_cognition_optimizer_artifacts",
            "focusa_predict_record",
            "focusa_metacog_capture",
          ],
          raw: resp,
        });
      return {
        content: [
          {
            type: "text",
            text: piToolText({
              kind: "ok",
              tool: "focusa_context_cognition_curate_optimize",
              summary: `context cognition curate optimize → decision=${decision} promoted=${promovido ? "yes" : "no"}`,
              ids: [
                { label: "artifact_id", value: artifactId },
                { label: "rehydrate_id", value: artifactId },
                { label: "rollback_ref", value: rollbackRef },
              ],
              fields: [
                { label: "decision", value: decision },
                { label: "eval_score", value: Number(evalScore.toFixed(3)) },
                { label: "baseline_score", value: Number(baselineScore.toFixed(3)) },
                { label: "promoted", value: promovido ? "yes" : "no" },
                { label: "advisory", value: "true" },
              ],
              nextTools: [
                "focusa_context_cognition_optimizer_artifacts",
                "focusa_predict_record",
                "focusa_metacog_capture",
              ],
            }),
          },
        ],
        details: {
          ok: true,
          status: "completed",
          endpoint: "/v1/context-cognition/curate/optimize",
          canonical: false,
          advisory: true,
          project_root: String(projectRoot),
          artifact_id: artifactId,
          decision,
          eval_score: evalScore,
          baseline_score: baselineScore,
          promoted: promovido,
          rollback_ref: rollbackRef,
          tool_result_v1: toolResult,
        } as any,
      };
    },
  });

  pi.registerTool({
    name: "focusa_context_cognition_optimizer_artifacts",
    label: "Context Cognition Optimizer Artifacts",
    description:
      "Spec 100 Phase 5 — list Cognition Optimizer artifacts (versioned JSONL) for a project+module. Returns the recent artifact list and the latest promoted artifact (if any).",
    promptSnippet:
      "Use when checking which Cognition Optimizer artifact is currently promoted for a project, or when reviewing the artifact history for rollback decisions.",
    parameters: strictObject({
      project_root: Type.Optional(
        Type.String({ maxLength: 4096, description: "Project root. Defaults to Pi session cwd." })
      ),
      module_name: Type.Optional(Type.String({ description: "Module name (default: curator)." })),
      limit: Type.Optional(
        Type.Integer({ minimum: 1, maximum: 200, description: "Max artifacts to return (default 10)." })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_context_cognition_optimizer_artifacts", params, [
        "project_root",
        "module_name",
        "limit",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_context_cognition_optimizer_artifacts",
          "/v1/context-cognition/optimizer/artifacts",
          params as Record<string, any>,
          "context cognition optimizer artifacts",
          keyCheck.error
        );
      }
      const projectRoot = await resolveFocusaToolProjectRoot((keyCheck.value as any).project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, (keyCheck.value as any).project_root);
      if (projectRootGate) return projectRootGate;
      const query = new URLSearchParams();
      query.set("project_root", String(projectRoot));
      const moduleName = (keyCheck.value as any).module_name ?? "curator";
      query.set("module_name", String(moduleName));
      query.set("limit", String((keyCheck.value as any).limit ?? 10));
      const res = await focusaFetchDetailed(`/context-cognition/optimizer/artifacts?${query.toString()}`);
      const body = res.body || {};
      if (!res.ok) {
        return blockedToolResponse(
          "focusa_context_cognition_optimizer_artifacts",
          "trajectory",
          `context cognition optimizer artifacts blocked → ${explainWorkLoopResult(res, "optimizer artifacts unavailable")}`,
          body.failure_class || "daemon_unavailable",
          body,
          ["focusa_project_verify", "focusa_tool_doctor"]
        );
      }
      const artifacts = Array.isArray(body.artifacts) ? body.artifacts : [];
      const latestPromoted = body.latest_promoted ?? null;
      const rehydrate = String(body.rehydrate_id || "no_artifacts");
      const toolResult =
        body.details?.tool_result_v1 ||
        focusaToolResult({
          ok: true,
          status: "completed",
          summary: `optimizer artifacts → count=${artifacts.length} module=${moduleName}`,
          tool: "focusa_context_cognition_optimizer_artifacts",
          family: "trajectory",
          side_effects: [],
          evidence_refs: latestPromoted ? [String(latestPromoted.artifact_id)] : [],
          next_tools: ["focusa_context_cognition_curate_optimize"],
          raw: body,
        });
      return {
        content: [
          {
            type: "text",
            text: piToolText({
              kind: "ok",
              tool: "focusa_context_cognition_optimizer_artifacts",
              summary: `optimizer artifacts → count=${artifacts.length} module=${moduleName}`,
              ids: [
                { label: "rehydrate_id", value: rehydrate },
                {
                  label: "latest_promoted_id",
                  value: latestPromoted ? String(latestPromoted.artifact_id) : "none",
                },
              ],
              fields: [
                { label: "count", value: artifacts.length },
                { label: "module_name", value: moduleName },
                {
                  label: "latest_promoted",
                  value: latestPromoted
                    ? `${latestPromoted.artifact_id}@${Number(latestPromoted.eval_score ?? 0).toFixed(2)}`
                    : "none",
                },
                { label: "advisory", value: "true" },
              ],
              nextTools: ["focusa_context_cognition_curate_optimize"],
            }),
          },
        ],
        details: {
          ok: true,
          status: "completed",
          endpoint: "/v1/context-cognition/optimizer/artifacts",
          canonical: false,
          advisory: true,
          project_root: String(projectRoot),
          module_name: moduleName,
          count: artifacts.length,
          artifacts,
          latest_promoted: latestPromoted,
          rehydrate_id: rehydrate,
          tool_result_v1: toolResult,
        } as any,
      };
    },
  });

  pi.registerTool({
    name: "focusa_call_stack_design",
    label: "Call Stack Design",
    description:
      "Write a typed, append-only Call Stack Design for a feature before implementation. Returns the standard Focusa call stack scaffold (entry → handlers → services → adapters → storage → output) that the operator/agent fills in for the specific feature. Per Spec 103.",
    promptSnippet:
      "Use when designing a new feature or refactor that will be implemented by an AI agent. The design is a typed artifact, not free-form prose.",
    parameters: strictObject({
      project_root: Type.Optional(
        Type.String({
          maxLength: 4096,
          description: "Project root for the design. Defaults to Pi session cwd.",
        })
      ),
      continuity_id: Type.Optional(
        Type.String({ maxLength: 256, description: "Optional continuity id filter." })
      ),
      mission: Type.String({
        minLength: 1,
        maxLength: 200,
        description: "Short description of the feature this design covers.",
      }),
      entry_surface: Type.Optional(
        Type.Union([Type.Literal("pi_tool"), Type.Literal("cli_command"), Type.Literal("http_route")], {
          description: "Entry surface kind (default: pi_tool).",
        })
      ),
      entry_name: Type.String({
        minLength: 1,
        maxLength: 120,
        description: "Proposed tool/command/route name.",
      }),
      workpoint_id: Type.Optional(
        Type.String({
          maxLength: 256,
          description: "Workpoint to attach the design to (required when attach_to_workpoint=true).",
        })
      ),
      attach_to_workpoint: Type.Optional(
        Type.Boolean({
          description: "When true, the design becomes focusa_evidence linked to the active Workpoint.",
        })
      ),
      attach_to_stg: Type.Optional(
        Type.Boolean({ description: "When true, the design sets the active STG of the active Trajectory." })
      ),
      parent_design_id: Type.Optional(
        Type.String({ maxLength: 256, description: "Optional parent design id to chain refinements." })
      ),
      notes: Type.Optional(
        Type.String({ maxLength: 2048, description: "Optional bounded free-form notes." })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_call_stack_design", params, [
        "project_root",
        "continuity_id",
        "mission",
        "entry_surface",
        "entry_name",
        "workpoint_id",
        "attach_to_workpoint",
        "attach_to_stg",
        "parent_design_id",
        "notes",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_call_stack_design",
          "/v1/call-stack/design",
          params as Record<string, any>,
          "call stack design",
          keyCheck.error
        );
      }
      const projectRoot = await resolveFocusaToolProjectRoot((keyCheck.value as any).project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, (keyCheck.value as any).project_root);
      if (projectRootGate) return projectRootGate;
      const raw: Record<string, any> = {
        ...(keyCheck.value as Record<string, any>),
        project_root: projectRoot,
      };
      const res = await focusaFetchDetailed("/call-stack/design", {
        method: "POST",
        body: JSON.stringify(raw),
      });
      const body = res.body || {};
      if (!res.ok) {
        return blockedToolResponse(
          "focusa_call_stack_design",
          "workpoint",
          `call stack design blocked → ${explainWorkLoopResult(res, "design unavailable")}`,
          body.failure_class || "daemon_unavailable",
          body,
          ["focusa_project_verify", "focusa_workpoint_resume", "focusa_tool_doctor"]
        );
      }
      const designId = String(body.design_id || body.design?.design_id || "stored");
      const entryName = String(body.design?.entry_name || raw.entry_name || "unknown");
      const entrySurface = String(body.design?.entry_surface || raw.entry_surface || "pi_tool");
      const mission = String(body.design?.mission || raw.mission || "").slice(0, 80);
      const toolResult =
        body.details?.tool_result_v1 ||
        focusaToolResult({
          ok: true,
          status: "completed",
          summary: `call stack design → ${designId} entry=${entryName}`,
          tool: "focusa_call_stack_design",
          family: "workpoint",
          side_effects: ["call_stack_design_append"],
          evidence_refs: [],
          next_tools: ["focusa_workpoint_link_evidence", "focusa_trajectory_assess", "focusa_project_verify"],
          raw: body,
        });
      return {
        content: [
          {
            type: "text",
            text: piToolText({
              kind: "ok",
              tool: "focusa_call_stack_design",
              summary: `call stack design → mission="${mission}"`,
              ids: [
                { label: "design_id", value: designId },
                { label: "rehydrate_id", value: designId },
                { label: "entry_name", value: entryName },
                { label: "entry_surface", value: entrySurface },
              ],
              fields: [
                { label: "mission", value: mission },
                { label: "project_root", value: projectRoot },
                { label: "attach_to_workpoint", value: raw.attach_to_workpoint ? "yes" : "no" },
                { label: "attach_to_stg", value: raw.attach_to_stg ? "yes" : "no" },
                { label: "ledger_file", value: String(body.ledger_file || "unknown") },
              ],
              nextTools: [
                "focusa_workpoint_link_evidence",
                "focusa_trajectory_assess",
                "focusa_project_verify",
              ],
            }),
          },
        ],
        details: {
          ok: true,
          status: "completed",
          endpoint: "/v1/call-stack/design",
          canonical: false,
          advisory: true,
          project_root: projectRoot,
          design_id: designId,
          entry_name: entryName,
          entry_surface: entrySurface,
          design: body.design || null,
          next_tools: body.next_tools || toolResult.next_tools,
          ledger_file: body.ledger_file || null,
          tool_result_v1: toolResult,
        } as any,
      };
    },
  });

  pi.registerTool({
    name: "focusa_call_stack_verify",
    label: "Call Stack Verify",
    description:
      "Verify a Call Stack Design against bounded implementation surfaces and report drift: entry surface, handlers, services, adapters, storage, output envelope, evidence, and Workpoint/STG alignment. Advisory only.",
    promptSnippet:
      "Use after focusa_call_stack_design or before implementation review to detect design-vs-code drift without mutating Focus State.",
    parameters: strictObject({
      project_root: Type.Optional(
        Type.String({
          maxLength: 4096,
          description: "Project root for the design. Defaults to Pi session cwd.",
        })
      ),
      continuity_id: Type.Optional(
        Type.String({ maxLength: 256, description: "Optional continuity scope filter." })
      ),
      design_id: Type.Optional(
        Type.String({ maxLength: 256, description: "Specific Call Stack Design id to verify." })
      ),
      entry_name: Type.Optional(
        Type.String({ maxLength: 120, description: "Entry name to verify when design_id is omitted." })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_call_stack_verify", params, [
        "project_root",
        "continuity_id",
        "design_id",
        "entry_name",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_call_stack_verify",
          "/v1/call-stack/verify",
          params as Record<string, any>,
          "call stack verify",
          keyCheck.error
        );
      }
      const projectRoot = await resolveFocusaToolProjectRoot((keyCheck.value as any).project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, (keyCheck.value as any).project_root);
      if (projectRootGate) return projectRootGate;
      const raw: Record<string, any> = {
        ...(keyCheck.value as Record<string, any>),
        project_root: projectRoot,
      };
      const res = await focusaFetchDetailed("/call-stack/verify", {
        method: "POST",
        body: JSON.stringify(raw),
      });
      const body = res.body || {};
      if (!res.ok) {
        return blockedToolResponse(
          "focusa_call_stack_verify",
          "workpoint",
          `call stack verify blocked → ${explainWorkLoopResult(res, "verify unavailable")}`,
          body.failure_class || "daemon_unavailable",
          body,
          ["focusa_call_stack_design", "focusa_project_verify", "focusa_tool_doctor"]
        );
      }
      const designId = String(body.design_id || raw.design_id || "unknown");
      const driftStatus = String(body.drift_status || "unknown");
      const entryName = String(body.entry_name || raw.entry_name || "unknown");
      const failures = String(body.failures ?? 0);
      const warnings = String(body.warnings ?? 0);
      const toolResult =
        body.details?.tool_result_v1 ||
        focusaToolResult({
          ok: true,
          status: "completed",
          summary: `call stack verify → ${driftStatus} design=${designId}`,
          tool: "focusa_call_stack_verify",
          family: "workpoint",
          side_effects: [],
          evidence_refs: [],
          next_tools: [
            "focusa_call_stack_design",
            "focusa_workpoint_link_evidence",
            "focusa_trajectory_assess",
          ],
          raw: body,
        });
      return {
        content: [
          {
            type: "text",
            text: piToolText({
              kind: "ok",
              tool: "focusa_call_stack_verify",
              summary: `call stack verify → ${driftStatus}`,
              ids: [
                { label: "design_id", value: designId },
                { label: "entry_name", value: entryName },
                { label: "rehydrate_id", value: String(body.rehydrate_id || designId) },
              ],
              fields: [
                { label: "drift_status", value: driftStatus },
                { label: "failures", value: failures },
                { label: "warnings", value: warnings },
                { label: "advisory", value: "true" },
              ],
              nextTools: [
                "focusa_call_stack_design",
                "focusa_workpoint_link_evidence",
                "focusa_trajectory_assess",
              ],
            }),
          },
        ],
        details: {
          ok: true,
          status: "completed",
          endpoint: "/v1/call-stack/verify",
          canonical: false,
          advisory: true,
          design_id: designId,
          drift_status: driftStatus,
          failures: body.failures || 0,
          warnings: body.warnings || 0,
          checks: body.checks || [],
          next_tools: body.next_tools || toolResult.next_tools,
          tool_result_v1: toolResult,
        } as any,
      };
    },
  });

  pi.registerTool({
    name: "focusa_tree_recent_snapshots",
    label: "Tree Recent Snapshots",
    description:
      "Best safe helper for finding recent snapshot ids. Use this before diff or restore when you do not already know the right snapshot id.",
    parameters: strictObject({
      limit: Type.Optional(
        Type.Integer({
          minimum: 1,
          maximum: 20,
          description: "How many recent snapshots to return (default 5).",
        })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_tree_recent_snapshots", params, ["limit"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_tree_recent_snapshots",
          "/v1/focus/snapshots/recent",
          params as Record<string, any>,
          "tree recent snapshots",
          keyCheck.error
        );
      }
      let limit = Math.trunc(Number((keyCheck.value as { limit?: number }).limit ?? 5));
      if (!Number.isFinite(limit)) limit = 5;
      limit = Math.max(1, Math.min(20, limit));
      const endpoint = `/focus/snapshots/recent?limit=${limit}`;
      const res = await callSpec80Tool(
        "focusa_tree_recent_snapshots",
        endpoint,
        { limit },
        { method: "GET" }
      );
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
        "tree recent snapshots"
      );
    },
  });

  pi.registerTool({
    name: "focusa_tree_snapshot_compare_latest",
    label: "Tree Snapshot Compare Latest",
    description:
      "Create a fresh snapshot and compare it to the latest prior snapshot in one move. Best tool when you want checkpoint + diff without manual id hunting.",
    parameters: strictObject({
      snapshot_reason: Type.Optional(
        Type.String({
          maxLength: SPEC81_LIMITS.snapshotReason,
          description: "Reason label for the new snapshot.",
        })
      ),
      baseline_snapshot_id: Type.Optional(
        Type.String({
          maxLength: SPEC81_LIMITS.id,
          pattern: SPEC81_ID_PATTERN,
          description: "Optional explicit baseline snapshot id.",
        })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_tree_snapshot_compare_latest", params, [
        "snapshot_reason",
        "baseline_snapshot_id",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_tree_snapshot_compare_latest",
          "/v1/focus/snapshots/recent+create+diff",
          params as Record<string, any>,
          "tree snapshot compare latest",
          keyCheck.error
        );
      }
      const raw = keyCheck.value as { snapshot_reason?: string; baseline_snapshot_id?: string };
      const reasonCheck = validateOptionalString(
        "snapshot_reason",
        raw.snapshot_reason,
        SPEC81_LIMITS.snapshotReason
      );
      if (!reasonCheck.ok) {
        return spec80ValidationResult(
          "focusa_tree_snapshot_compare_latest",
          "/v1/focus/snapshots/recent+create+diff",
          raw as Record<string, any>,
          "tree snapshot compare latest",
          reasonCheck.error
        );
      }
      const baselineCheck = validateOptionalString(
        "baseline_snapshot_id",
        raw.baseline_snapshot_id,
        SPEC81_LIMITS.id,
        { pattern: SPEC81_ID_RE }
      );
      if (!baselineCheck.ok) {
        return spec80ValidationResult(
          "focusa_tree_snapshot_compare_latest",
          "/v1/focus/snapshots/recent+create+diff",
          raw as Record<string, any>,
          "tree snapshot compare latest",
          baselineCheck.error
        );
      }

      let baselineSnapshotId = baselineCheck.value;
      if (!baselineSnapshotId) {
        const recentRes = await callSpec80Tool(
          "focusa_tree_snapshot_compare_latest",
          "/focus/snapshots/recent?limit=1",
          { limit: 1 },
          { method: "GET" }
        );
        if (recentRes.ok) {
          baselineSnapshotId = recentRes.body?.snapshots?.[0]?.snapshot_id;
        }
      }

      const createReq = { snapshot_reason: reasonCheck.value || null };
      const createRes = await callSpec80Tool(
        "focusa_tree_snapshot_compare_latest",
        "/focus/snapshots",
        createReq,
        { method: "POST", writer: true }
      );
      if (!createRes.ok || !createRes.body?.snapshot_id) {
        return spec80CompositeResult(
          "focusa_tree_snapshot_compare_latest",
          "/v1/focus/snapshots/recent+create+diff",
          { ...createReq, baseline_snapshot_id: baselineSnapshotId || null },
          false,
          createRes.status,
          createRes.body,
          "tree snapshot compare latest: ok",
          "tree snapshot compare latest"
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
          "tree snapshot compare latest"
        );
      }

      const diffReq = { from_snapshot_id: baselineSnapshotId, to_snapshot_id: newSnapshotId };
      const diffRes = await callSpec80Tool(
        "focusa_tree_snapshot_compare_latest",
        "/focus/snapshots/diff",
        diffReq,
        { method: "POST" }
      );
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
        "tree snapshot compare latest"
      );
    },
  });

  pi.registerTool({
    name: "focusa_metacog_recent_reflections",
    label: "Metacog Recent Reflections",
    description:
      "Best safe helper for finding recent reflection ids and update sets before adjust or promote work.",
    parameters: strictObject({
      limit: Type.Optional(
        Type.Integer({
          minimum: 1,
          maximum: 20,
          description: "How many recent reflections to return (default 5).",
        })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_metacog_recent_reflections", params, ["limit"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_recent_reflections",
          "/v1/metacognition/reflections/recent",
          params as Record<string, any>,
          "metacog recent reflections",
          keyCheck.error
        );
      }
      let limit = Math.trunc(Number((keyCheck.value as { limit?: number }).limit ?? 5));
      if (!Number.isFinite(limit)) limit = 5;
      limit = Math.max(1, Math.min(20, limit));
      const endpoint = `/metacognition/reflections/recent?limit=${limit}`;
      const res = await callSpec80Tool(
        "focusa_metacog_recent_reflections",
        endpoint,
        { limit },
        { method: "GET" }
      );
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
        {
          kind: items.length > 0 ? "ok" : "advisory",
          ids:
            items.length > 0
              ? ids.slice(0, 4).map((id: unknown, idx: number) => ({
                  label: `reflection_id_${idx + 1}`,
                  value: String(id),
                }))
              : [],
          fields: [
            { label: "total", value: items.length },
            { label: "ids", value: summarizeArray(ids, 4) },
            { label: "limit", value: limit },
          ],
          nextTools:
            items.length > 0
              ? ["focusa_metacog_plan_adjust", "focusa_metacog_loop_run"]
              : ["focusa_metacog_reflect"],
        }
      );
    },
  });

  pi.registerTool({
    name: "focusa_metacog_recent_adjustments",
    label: "Metacog Recent Adjustments",
    description:
      "Best safe helper for finding recent adjustment ids before evaluation or promotion decisions.",
    parameters: strictObject({
      limit: Type.Optional(
        Type.Integer({
          minimum: 1,
          maximum: 20,
          description: "How many recent adjustments to return (default 5).",
        })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_metacog_recent_adjustments", params, ["limit"]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_recent_adjustments",
          "/v1/metacognition/adjustments/recent",
          params as Record<string, any>,
          "metacog recent adjustments",
          keyCheck.error
        );
      }
      let limit = Math.trunc(Number((keyCheck.value as { limit?: number }).limit ?? 5));
      if (!Number.isFinite(limit)) limit = 5;
      limit = Math.max(1, Math.min(20, limit));
      const endpoint = `/metacognition/adjustments/recent?limit=${limit}`;
      const res = await callSpec80Tool(
        "focusa_metacog_recent_adjustments",
        endpoint,
        { limit },
        { method: "GET" }
      );
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
        {
          kind: items.length > 0 ? "ok" : "advisory",
          ids:
            items.length > 0
              ? ids.slice(0, 4).map((id: unknown, idx: number) => ({
                  label: `adjustment_id_${idx + 1}`,
                  value: String(id),
                }))
              : [],
          fields: [
            { label: "total", value: items.length },
            { label: "ids", value: summarizeArray(ids, 4) },
            { label: "limit", value: limit },
          ],
          nextTools:
            items.length > 0
              ? ["focusa_metacog_evaluate_outcome", "focusa_metacog_doctor"]
              : ["focusa_metacog_plan_adjust"],
        }
      );
    },
  });

  pi.registerTool({
    name: "focusa_metacog_loop_run",
    label: "Metacog Loop Run",
    description:
      "Run capture -> retrieve -> reflect -> adjust -> evaluate in one move. Best composite tool when you want learning workflow compression instead of manual chaining.",
    parameters: strictObject({
      current_ask: Type.String({
        minLength: 1,
        maxLength: SPEC81_LIMITS.currentAsk,
        description: "Current ask driving retrieval and reuse.",
      }),
      turn_range: Type.String({
        minLength: 1,
        maxLength: SPEC81_LIMITS.turnRange,
        pattern: SPEC81_TURN_RANGE_PATTERN,
        description: "Turn range expression for reflection.",
      }),
      kind: Type.Optional(
        Type.String({
          maxLength: SPEC81_LIMITS.kind,
          description: "Optional capture kind (default workflow_signal).",
        })
      ),
      content: Type.Optional(
        Type.String({
          maxLength: SPEC81_LIMITS.longText,
          description: "Optional capture content; defaults to current_ask.",
        })
      ),
      rationale: Type.Optional(
        Type.String({ maxLength: SPEC81_LIMITS.rationale, description: "Optional capture rationale." })
      ),
      confidence: Type.Optional(
        Type.Number({ minimum: 0, maximum: 1, description: "Optional confidence 0..1." })
      ),
      strategy_class: Type.Optional(
        Type.String({ maxLength: SPEC81_LIMITS.strategyClass, description: "Optional strategy class." })
      ),
      scope_tags: Type.Optional(
        Type.Array(Type.String({ maxLength: SPEC81_LIMITS.tagText }), { maxItems: SPEC81_LIMITS.scopeTags })
      ),
      k: Type.Optional(Type.Integer({ minimum: 1, maximum: 50, description: "Top-k retrieval size." })),
      failure_classes: Type.Optional(
        Type.Array(Type.String({ maxLength: SPEC81_LIMITS.tagText }), {
          maxItems: SPEC81_LIMITS.failureClasses,
        })
      ),
      selected_updates: Type.Optional(
        Type.Array(Type.String({ maxLength: SPEC81_LIMITS.updateText }), {
          maxItems: SPEC81_LIMITS.selectedUpdates,
        })
      ),
      observed_metrics: Type.Optional(
        Type.Array(Type.String({ maxLength: SPEC81_LIMITS.metricText }), {
          maxItems: SPEC81_LIMITS.observedMetrics,
        })
      ),
    }),
    async execute(_id, params) {
      const allowed = [
        "current_ask",
        "turn_range",
        "kind",
        "content",
        "rationale",
        "confidence",
        "strategy_class",
        "scope_tags",
        "k",
        "failure_classes",
        "selected_updates",
        "observed_metrics",
      ];
      const keyCheck = validateNoExtraKeys("focusa_metacog_loop_run", params, allowed);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_loop_run",
          "/v1/metacognition/loop-run",
          params as Record<string, any>,
          "metacog loop run",
          keyCheck.error
        );
      }
      const raw = keyCheck.value as Record<string, any>;
      const askCheck = validateRequiredString("current_ask", raw.current_ask, SPEC81_LIMITS.currentAsk);
      if (!askCheck.ok)
        return spec80ValidationResult(
          "focusa_metacog_loop_run",
          "/v1/metacognition/loop-run",
          raw,
          "metacog loop run",
          askCheck.error
        );
      const turnCheck = validateRequiredString("turn_range", raw.turn_range, SPEC81_LIMITS.turnRange, {
        pattern: SPEC81_TURN_RANGE_RE,
      });
      if (!turnCheck.ok)
        return spec80ValidationResult(
          "focusa_metacog_loop_run",
          "/v1/metacognition/loop-run",
          raw,
          "metacog loop run",
          turnCheck.error
        );
      const kindCheck = validateOptionalString("kind", raw.kind, SPEC81_LIMITS.kind);
      if (!kindCheck.ok)
        return spec80ValidationResult(
          "focusa_metacog_loop_run",
          "/v1/metacognition/loop-run",
          raw,
          "metacog loop run",
          kindCheck.error
        );
      const contentCheck = validateOptionalString("content", raw.content, SPEC81_LIMITS.longText);
      if (!contentCheck.ok)
        return spec80ValidationResult(
          "focusa_metacog_loop_run",
          "/v1/metacognition/loop-run",
          raw,
          "metacog loop run",
          contentCheck.error
        );
      const rationaleCheck = validateOptionalString("rationale", raw.rationale, SPEC81_LIMITS.rationale);
      if (!rationaleCheck.ok)
        return spec80ValidationResult(
          "focusa_metacog_loop_run",
          "/v1/metacognition/loop-run",
          raw,
          "metacog loop run",
          rationaleCheck.error
        );
      const strategyCheck = validateOptionalString(
        "strategy_class",
        raw.strategy_class,
        SPEC81_LIMITS.strategyClass
      );
      if (!strategyCheck.ok)
        return spec80ValidationResult(
          "focusa_metacog_loop_run",
          "/v1/metacognition/loop-run",
          raw,
          "metacog loop run",
          strategyCheck.error
        );
      if (
        raw.confidence !== undefined &&
        (!Number.isFinite(raw.confidence) || raw.confidence < 0 || raw.confidence > 1)
      ) {
        return spec80ValidationResult(
          "focusa_metacog_loop_run",
          "/v1/metacognition/loop-run",
          raw,
          "metacog loop run",
          "confidence must be between 0 and 1"
        );
      }
      const tagsCheck = validateStringArray("scope_tags", raw.scope_tags, {
        maxItems: SPEC81_LIMITS.scopeTags,
        itemMaxLength: SPEC81_LIMITS.tagText,
      });
      if (!tagsCheck.ok)
        return spec80ValidationResult(
          "focusa_metacog_loop_run",
          "/v1/metacognition/loop-run",
          raw,
          "metacog loop run",
          tagsCheck.error
        );
      const failuresCheck = validateStringArray("failure_classes", raw.failure_classes, {
        maxItems: SPEC81_LIMITS.failureClasses,
        itemMaxLength: SPEC81_LIMITS.tagText,
      });
      if (!failuresCheck.ok)
        return spec80ValidationResult(
          "focusa_metacog_loop_run",
          "/v1/metacognition/loop-run",
          raw,
          "metacog loop run",
          failuresCheck.error
        );
      const selectedCheck = validateStringArray("selected_updates", raw.selected_updates, {
        maxItems: SPEC81_LIMITS.selectedUpdates,
        itemMaxLength: SPEC81_LIMITS.updateText,
      });
      if (!selectedCheck.ok)
        return spec80ValidationResult(
          "focusa_metacog_loop_run",
          "/v1/metacognition/loop-run",
          raw,
          "metacog loop run",
          selectedCheck.error
        );
      const metricsCheck = validateStringArray("observed_metrics", raw.observed_metrics, {
        maxItems: SPEC81_LIMITS.observedMetrics,
        itemMaxLength: SPEC81_LIMITS.metricText,
      });
      if (!metricsCheck.ok)
        return spec80ValidationResult(
          "focusa_metacog_loop_run",
          "/v1/metacognition/loop-run",
          raw,
          "metacog loop run",
          metricsCheck.error
        );
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
      const captureRes = await callSpec80Tool(
        "focusa_metacog_loop_run",
        "/metacognition/capture",
        captureReq,
        { method: "POST", writer: true }
      );
      if (!captureRes.ok) {
        return spec80CompositeResult(
          "focusa_metacog_loop_run",
          "/v1/metacognition/loop-run",
          raw,
          false,
          captureRes.status,
          captureRes.body,
          "metacog loop run: ok",
          "metacog loop run"
        );
      }
      const retrieveReq = { current_ask: askCheck.value, scope_tags: tagsCheck.value, k: normalizedK };
      const retrieveRes = await callSpec80Tool(
        "focusa_metacog_loop_run",
        "/metacognition/retrieve",
        retrieveReq,
        { method: "POST" }
      );
      if (!retrieveRes.ok) {
        return spec80CompositeResult(
          "focusa_metacog_loop_run",
          "/v1/metacognition/loop-run",
          raw,
          false,
          retrieveRes.status,
          retrieveRes.body,
          "metacog loop run: ok",
          "metacog loop run"
        );
      }
      const reflectReq = { turn_range: turnCheck.value, failure_classes: failuresCheck.value };
      const reflectRes = await callSpec80Tool(
        "focusa_metacog_loop_run",
        "/metacognition/reflect",
        reflectReq,
        { method: "POST", writer: true }
      );
      if (!reflectRes.ok || !reflectRes.body?.reflection_id) {
        return spec80CompositeResult(
          "focusa_metacog_loop_run",
          "/v1/metacognition/loop-run",
          raw,
          false,
          reflectRes.status,
          reflectRes.body,
          "metacog loop run: ok",
          "metacog loop run"
        );
      }
      const updates =
        selectedCheck.value.length > 0
          ? selectedCheck.value
          : Array.isArray(reflectRes.body?.strategy_updates)
            ? reflectRes.body.strategy_updates.map((x: any) => String(x))
            : [];
      const adjustReq = { reflection_id: String(reflectRes.body.reflection_id), selected_updates: updates };
      const adjustRes = await callSpec80Tool("focusa_metacog_loop_run", "/metacognition/adjust", adjustReq, {
        method: "POST",
        writer: true,
      });
      if (!adjustRes.ok || !adjustRes.body?.adjustment_id) {
        return spec80CompositeResult(
          "focusa_metacog_loop_run",
          "/v1/metacognition/loop-run",
          raw,
          false,
          adjustRes.status,
          adjustRes.body,
          "metacog loop run: ok",
          "metacog loop run"
        );
      }
      const evaluateReq = {
        adjustment_id: String(adjustRes.body.adjustment_id),
        observed_metrics: metricsCheck.value,
      };
      const evaluateRes = await callSpec80Tool(
        "focusa_metacog_loop_run",
        "/metacognition/evaluate",
        evaluateReq,
        { method: "POST", writer: true }
      );
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
        {
          kind: "ok",
          ids: [
            { label: "capture_id", value: String(captureRes.body?.capture_id || "n/a") },
            { label: "reflection_id", value: String(reflectRes.body?.reflection_id || "unknown") },
            { label: "adjustment_id", value: String(adjustRes.body?.adjustment_id || "unknown") },
            {
              label: "rehydrate_id",
              value: String(captureRes.body?.capture_id || reflectRes.body?.reflection_id || "n/a"),
            },
          ],
          fields: [
            { label: "result", value: String(evaluateRes.body?.result || "unknown") },
            { label: "promote", value: boolLabel(evaluateRes.body?.promote_learning) },
            { label: "current_ask", value: askCheck.value },
            { label: "turn_range", value: turnCheck.value },
          ],
          nextTools: ["focusa_metacog_doctor", "focusa_metacog_evaluate_outcome"],
        }
      );
    },
  });

  pi.registerTool({
    name: "focusa_metacog_doctor",
    label: "Metacog Doctor",
    description:
      "Diagnose signal quality and retrieval usefulness in one move. Best safe diagnostic tool when deciding whether more capture or reflection work is needed.",
    parameters: strictObject({
      current_ask: Type.String({
        minLength: 1,
        maxLength: SPEC81_LIMITS.currentAsk,
        description: "Current ask to diagnose against.",
      }),
      scope_tags: Type.Optional(
        Type.Array(Type.String({ maxLength: SPEC81_LIMITS.tagText }), { maxItems: SPEC81_LIMITS.scopeTags })
      ),
      k: Type.Optional(Type.Integer({ minimum: 1, maximum: 50, description: "Top-k retrieval size." })),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_metacog_doctor", params, [
        "current_ask",
        "scope_tags",
        "k",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_metacog_doctor",
          "/v1/metacognition/doctor",
          params as Record<string, any>,
          "metacog doctor",
          keyCheck.error
        );
      }
      const raw = keyCheck.value as { current_ask: string; scope_tags?: string[]; k?: number };
      const askCheck = validateRequiredString("current_ask", raw.current_ask, SPEC81_LIMITS.currentAsk);
      if (!askCheck.ok)
        return spec80ValidationResult(
          "focusa_metacog_doctor",
          "/v1/metacognition/doctor",
          raw as Record<string, any>,
          "metacog doctor",
          askCheck.error
        );
      const tagsCheck = validateStringArray("scope_tags", raw.scope_tags, {
        maxItems: SPEC81_LIMITS.scopeTags,
        itemMaxLength: SPEC81_LIMITS.tagText,
      });
      if (!tagsCheck.ok)
        return spec80ValidationResult(
          "focusa_metacog_doctor",
          "/v1/metacognition/doctor",
          raw as Record<string, any>,
          "metacog doctor",
          tagsCheck.error
        );
      let normalizedK = Math.trunc(Number(raw.k ?? 5));
      if (!Number.isFinite(normalizedK)) normalizedK = 5;
      normalizedK = Math.max(1, Math.min(50, normalizedK));
      const req = {
        current_ask: askCheck.value,
        scope_tags: tagsCheck.value,
        k: normalizedK,
        summary_only: true,
      };
      const res = await callSpec80Tool("focusa_metacog_doctor", "/metacognition/retrieve", req, {
        method: "POST",
      });
      const candidates = Array.isArray(res.body?.candidates) ? res.body.candidates : [];
      const withConfidence = candidates.filter(
        (item: any) => item?.confidence !== null && item?.confidence !== undefined
      ).length;
      const top = candidates[0];
      return spec80Result(
        "focusa_metacog_doctor",
        "/v1/metacognition/doctor",
        { current_ask: askCheck.value, scope_tags: tagsCheck.value, k: normalizedK },
        {
          ok: res.ok,
          status: res.status,
          body: {
            ...(res.body || {}),
            diagnostics: {
              candidate_count: candidates.length,
              with_confidence: withConfidence,
              top_kind: top?.kind || null,
              top_capture_id: top?.capture_id || null,
            },
          },
        },
        candidates.length > 0
          ? `metacog doctor: candidates=${candidates.length} with_confidence=${withConfidence}\ntop_kind=${String(top?.kind || "unknown")} top_capture=${String(top?.capture_id || "none")}\nnext_tools=focusa_metacog_reflect,focusa_metacog_loop_run`
          : `metacog doctor: candidates=0\nno usable prior signals found\nnext_tools=focusa_metacog_capture,focusa_metacog_reflect`,
        "metacog doctor",
        {
          kind: candidates.length > 0 ? "ok" : "advisory",
          ids:
            candidates.length > 0
              ? [{ label: "top_capture_id", value: String(top?.capture_id || "none") }]
              : [],
          fields: [
            { label: "candidates", value: candidates.length },
            { label: "with_confidence", value: withConfidence },
            { label: "top_kind", value: String(top?.kind || "unknown") },
            { label: "ask", value: askCheck.value },
            { label: "k", value: normalizedK },
          ],
          nextTools:
            candidates.length > 0
              ? ["focusa_metacog_reflect", "focusa_metacog_loop_run"]
              : ["focusa_metacog_capture", "focusa_metacog_reflect"],
        }
      );
    },
  });

  // ── Surgical traversal facade (Spec96) ───────────────────────────────────

  pi.registerTool({
    name: "focusa_traverse",
    label: "Focusa Traverse",
    description:
      "Read-only surgical traversal across large Focusa surfaces. Use for bounded lineage, ontology, evidence, telemetry, Workpoint, and registry slices instead of full payloads.",
    parameters: strictObject({
      surface: Type.String({
        minLength: 1,
        maxLength: 80,
        description: "Surface: lineage|ontology|focus_stack|workpoints|evidence|telemetry|tool_registry etc.",
      }),
      selector: Type.Optional(
        Type.String({
          maxLength: 80,
          description:
            "Selector: window|head|path|children|neighborhood|summaries|search|recent|tags_verify.",
        })
      ),
      anchor: Type.Optional(
        Type.String({ maxLength: SPEC81_LIMITS.id, description: "Optional anchor id/tag/ref." })
      ),
      query: Type.Optional(
        Type.String({ maxLength: SPEC81_LIMITS.currentAsk, description: "Optional search/filter query." })
      ),
      cursor: Type.Optional(Type.String({ maxLength: 80, description: "Optional cursor/offset token." })),
      limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 200, description: "Bounded result limit." })),
      depth: Type.Optional(Type.Integer({ minimum: 1, maximum: 64, description: "Traversal depth cap." })),
      radius: Type.Optional(
        Type.Integer({ minimum: 1, maximum: 8, description: "Neighborhood radius cap." })
      ),
      fields: Type.Optional(
        Type.Array(Type.String({ maxLength: 80 }), {
          maxItems: 16,
          description: "Optional projected fields.",
        })
      ),
      tags: Type.Optional(
        Type.Array(
          Type.Union([
            Type.String({ maxLength: 240 }),
            Type.Object({
              anchor: Type.Optional(Type.String({ maxLength: 160 })),
              tag: Type.String({ maxLength: 240 }),
              ordinal: Type.Optional(Type.Integer({ minimum: 0, maximum: 100000 })),
            }),
          ]),
          {
            maxItems: 32,
            description: "Optional traversal tags to verify as strings or TraverseTagRef-style objects.",
          }
        )
      ),
      tag_mode: Type.Optional(
        Type.Union(
          [
            Type.Literal("item"),
            Type.Literal("range"),
            Type.Literal("window"),
            Type.Literal("surface"),
            Type.Literal("mixed"),
          ],
          { description: "Traversal tag mode; defaults mixed." }
        )
      ),
      include_payload: Type.Optional(
        Type.Boolean({ description: "Spec96 alias for explicit cold opt-in larger payload; defaults false." })
      ),
      include_full_payload: Type.Optional(
        Type.Boolean({
          description: "Compatibility alias for explicit cold opt-in larger payload; defaults false.",
        })
      ),
      include_rehydrate_refs: Type.Optional(
        Type.Boolean({ description: "Include rehydrate refs for omitted/cold slices." })
      ),
      budget_tokens: Type.Optional(
        Type.Integer({ minimum: 1, maximum: 20000, description: "Optional token budget hint." })
      ),
      session_identity: Type.Optional(
        Type.Any({ description: "Optional FocusaSessionIdentity envelope for scoped traversal." })
      ),
    }),
    async execute(_id, params) {
      const keyCheck = validateNoExtraKeys("focusa_traverse", params, [
        "surface",
        "selector",
        "anchor",
        "query",
        "cursor",
        "limit",
        "depth",
        "radius",
        "fields",
        "tags",
        "tag_mode",
        "include_payload",
        "include_full_payload",
        "include_rehydrate_refs",
        "budget_tokens",
        "session_identity",
      ]);
      if (!keyCheck.ok) {
        return spec80ValidationResult(
          "focusa_traverse",
          "/v1/traverse",
          params as Record<string, any>,
          "traverse",
          keyCheck.error
        );
      }
      const raw = keyCheck.value as {
        surface: string;
        selector?: string;
        anchor?: string;
        query?: string;
        cursor?: string;
        limit?: number;
        depth?: number;
        radius?: number;
        fields?: string[];
        tags?: any[];
        tag_mode?: string;
        include_payload?: boolean;
        include_full_payload?: boolean;
        include_rehydrate_refs?: boolean;
        budget_tokens?: number;
        session_identity?: any;
      };
      const surfaceCheck = validateRequiredString("surface", raw.surface, 80);
      if (!surfaceCheck.ok)
        return spec80ValidationResult(
          "focusa_traverse",
          "/v1/traverse",
          raw as Record<string, any>,
          "traverse",
          surfaceCheck.error
        );
      const selectorCheck = validateOptionalString("selector", raw.selector, 80);
      if (!selectorCheck.ok)
        return spec80ValidationResult(
          "focusa_traverse",
          "/v1/traverse",
          raw as Record<string, any>,
          "traverse",
          selectorCheck.error
        );
      const anchorCheck = validateOptionalString("anchor", raw.anchor, SPEC81_LIMITS.id);
      if (!anchorCheck.ok)
        return spec80ValidationResult(
          "focusa_traverse",
          "/v1/traverse",
          raw as Record<string, any>,
          "traverse",
          anchorCheck.error
        );
      const queryCheck = validateOptionalString("query", raw.query, SPEC81_LIMITS.currentAsk);
      if (!queryCheck.ok)
        return spec80ValidationResult(
          "focusa_traverse",
          "/v1/traverse",
          raw as Record<string, any>,
          "traverse",
          queryCheck.error
        );
      const cursorCheck = validateOptionalString("cursor", raw.cursor, 80);
      if (!cursorCheck.ok)
        return spec80ValidationResult(
          "focusa_traverse",
          "/v1/traverse",
          raw as Record<string, any>,
          "traverse",
          cursorCheck.error
        );
      const fieldsCheck = validateStringArray("fields", raw.fields, { maxItems: 16, itemMaxLength: 80 });
      if (!fieldsCheck.ok)
        return spec80ValidationResult(
          "focusa_traverse",
          "/v1/traverse",
          raw as Record<string, any>,
          "traverse",
          fieldsCheck.error
        );
      const tags = Array.isArray(raw.tags)
        ? raw.tags.slice(0, 32).map((tag) => {
            if (typeof tag === "string") return tag.slice(0, 240);
            if (tag && typeof tag === "object" && typeof tag.tag === "string")
              return { ...tag, tag: String(tag.tag).slice(0, 240) };
            return tag;
          })
        : [];
      if (raw.tags !== undefined && !Array.isArray(raw.tags))
        return spec80ValidationResult(
          "focusa_traverse",
          "/v1/traverse",
          raw as Record<string, any>,
          "traverse",
          "tags must be an array of strings or TraverseTagRef objects"
        );
      const selector = selectorCheck.value || "window";
      const req = {
        surface: surfaceCheck.value,
        selector,
        anchor: anchorCheck.value,
        query: queryCheck.value,
        cursor: cursorCheck.value,
        limit:
          raw.limit !== undefined ? Math.max(1, Math.min(200, Math.trunc(Number(raw.limit)))) : undefined,
        depth: raw.depth !== undefined ? Math.max(1, Math.min(64, Math.trunc(Number(raw.depth)))) : undefined,
        radius:
          raw.radius !== undefined ? Math.max(1, Math.min(8, Math.trunc(Number(raw.radius)))) : undefined,
        fields: fieldsCheck.value,
        tags,
        tag_mode: raw.tag_mode,
        // The API treats include_payload as a serde alias for include_full_payload.
        // Send exactly one canonical field; sending both aliases makes Rust reject the body as duplicate.
        include_full_payload: raw.include_full_payload === true || raw.include_payload === true,
        include_rehydrate_refs: raw.include_rehydrate_refs === true,
        budget_tokens:
          raw.budget_tokens !== undefined
            ? Math.max(1, Math.min(20000, Math.trunc(Number(raw.budget_tokens))))
            : undefined,
        session_identity: raw.session_identity,
      };
      const endpoint = selector === "tags_verify" ? "/traverse/verify-tags" : "/traverse";
      const res = await callSpec80Tool("focusa_traverse", endpoint, req, { method: "POST" });
      const items = Array.isArray(res.body?.items) ? res.body.items : [];
      const traversal = res.body?.traversal || {};
      const projection = traversal.fields && typeof traversal.fields === "object" ? traversal.fields : {};
      const omittedFields = Array.isArray(projection.omitted) ? projection.omitted : [];
      const projectionNote = projection.fallback_to_defaults
        ? ` projection=default_fallback omitted=${omittedFields.join(",") || "none"}`
        : omittedFields.length
          ? ` projection_omitted=${omittedFields.join(",")}`
          : "";
      return spec80Result(
        "focusa_traverse",
        endpoint === "/traverse" ? "/v1/traverse" : "/v1/traverse/verify-tags",
        req,
        res,
        `traverse: surface=${req.surface} selector=${selector} returned=${items.length}/${String(traversal.total ?? items.length)} truncated=${Boolean(traversal.truncated)} more_available=${Boolean(traversal.more_available ?? res.body?.more_available)}${projectionNote}
next_cursor=${String(traversal.next_cursor ?? "none")} guidance=${String(traversal.pagination_guidance || res.body?.pagination_guidance || "No pagination guidance returned.")} tags=${Array.isArray(res.body?.tags) ? res.body.tags.length : 0} verified=${Array.isArray(res.body?.verified_tags) ? res.body.verified_tags.length : 0} stale=${Array.isArray(res.body?.stale_tags) ? res.body.stale_tags.length : 0}
${summarizeTraverseItems(items, 8)}
next_tools=focusa_traverse,focusa_trajectory_view,focusa_workpoint_resume`,
        "traverse"
      );
    },
  });

  // ── Awareness packet (Spec108) ────────────────────────────────────────────

  pi.registerTool({
    name: "focusa_awareness_packet",
    label: "Focusa Awareness Packet",
    description:
      "Render a surface-aware AwarenessPacket with DVS-scored visible lines, suppressed lines, metadata, next_tools, and recovery_tools, including Spec 111 preload status surfaces.",
    parameters: strictObject({
      surface: Type.Optional(
        Type.Union(
          [
            Type.Literal("reload"),
            Type.Literal("post_compaction"),
            Type.Literal("warning"),
            Type.Literal("tool_guidance"),
            Type.Literal("uiai_bridge"),
            Type.Literal("agent_preload"),
            Type.Literal("preload_fail"),
            Type.Literal("preload_remediation"),
            Type.Literal("preload_receipt"),
          ],
          { description: "Awareness surface (default: reload)." }
        )
      ),
      mode: Type.Optional(
        Type.Union(
          [
            Type.Literal("minimal"),
            Type.Literal("standard"),
            Type.Literal("rich"),
            Type.Literal("onboarding"),
          ],
          { description: "Awareness rendering mode (default: standard)." }
        )
      ),
    }),
    async execute(_id, params) {
      const { surface = "reload" } = (params as { surface?: string }) || {};
      const route =
        surface === "reload" ? "/v1/awareness/packet" : `/v1/awareness/packet/${encodeURIComponent(surface)}`;
      const result = await focusaFetch(route, { method: "GET" });
      const resultOk = Boolean(result && typeof result === "object" && (result as any).ok === true);
      const raw = result && typeof result === "object" ? (result as any).value : null;
      const packet = raw && typeof raw === "object" ? raw : {};
      const visibleCount = Array.isArray(packet.visibleLines) ? packet.visibleLines.length : 0;
      const textLines = [
        `awareness_packet | surface=${packet.surface || surface} | mode=${packet.mode || "?"} | visible=${visibleCount}`,
      ];
      if (Array.isArray(packet.visibleLines)) {
        const lines = packet.visibleLines.slice(0, 5);
        for (const line of lines) {
          if (!line) continue; // guard against null entries in array
          textLines.push(
            `  ${line.layer || "?"} | ${line.category || "?"} | ${String(line.text || "").slice(0, 80)}`
          );
        }
        if (packet.visibleLines.length > 5) textLines.push(`  ... +${packet.visibleLines.length - 5} more`);
      }
      textLines.push(`schema=${packet.schema || "?"} confidence=${packet.metadata?.confidence || "?"}`);
      return {
        content: [{ type: "text" as const, text: textLines.join("\n") }],
        details: {
          ok: resultOk,
          status: resultOk ? "completed" : "blocked",
          canonical: resultOk,
          degraded: !resultOk,
          failure_class: resultOk ? null : "null_response",
          human_readable: resultOk
            ? `Awareness packet rendered for ${surface}.`
            : `Awareness packet unavailable for ${surface}; retry with focusa_tool_doctor.`,
          surface,
          schema: packet.schema,
          mode: packet.mode,
          visibleCount,
          suppressedCount: Array.isArray(packet.suppressedLines) ? packet.suppressedLines.length : 0,
          confidence: packet.metadata?.confidence,
          next_tools: ["focusa_workpoint_resume", "focusa_trajectory_view", "focusa_tool_doctor"],
        },
      };
    },
  });

  // ── Lineage Intelligence (LI) /tree first-class tools ────────────────────

  pi.registerTool({
    name: "focusa_lineage_tree",
    label: "Lineage Tree",
    description:
      "Fetch a bounded Focusa lineage window for /tree-aware reasoning. Full tree requires explicit cold opt-in.",
    parameters: Type.Object({
      session_id: Type.Optional(Type.String({ description: "Optional session id scoping hint." })),
      max_nodes: Type.Optional(Type.Number({ description: "Optional node cap (default 50)." })),
      include_full_payload: Type.Optional(
        Type.Boolean({ description: "Explicit cold opt-in for larger lineage payload." })
      ),
    }),
    async execute(_id, params) {
      const { session_id, max_nodes, include_full_payload } = params as {
        session_id?: string;
        max_nodes?: number;
        include_full_payload?: boolean;
      };
      const effectiveSessionId = String(session_id || getAttachmentRuntime().sessionFrameKey || "").trim();
      if (!effectiveSessionId) {
        return {
          content: [{ type: "text", text: "lineage tree blocked → active Pi session_id is required" }],
          details: {
            ok: false,
            status: "blocked",
            canonical: true,
            failure_class: "session_scope_required",
            global_fallback: false,
          },
        } as any;
      }
      const cap = Math.max(1, Math.min(include_full_payload ? 2000 : 200, Number(max_nodes || 50)));
      const queryParts = [
        `selector=window`,
        `limit=${encodeURIComponent(String(cap))}`,
        `session_id=${encodeURIComponent(effectiveSessionId)}`,
      ];
      if (include_full_payload === true) queryParts.push(`include_full_payload=true`);
      const query = `?${queryParts.join("&")}`;
      const res = await focusaFetchDetailed(`/lineage/tree${query}`);
      if (res.ok && String(res.body?.session_id || "").trim() !== effectiveSessionId) {
        return {
          content: [{ type: "text", text: "lineage tree blocked → response session scope mismatch" }],
          details: {
            ok: false,
            status: "blocked",
            canonical: true,
            failure_class: "scope_mismatch",
            global_fallback: false,
          },
        } as any;
      }
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
        content: [
          {
            type: "text",
            text: `lineage tree: nodes=${nodes.length} head=${head || "unknown"} root=${root || "unknown"}`,
          },
        ],
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
          session_id: effectiveSessionId,
          scope_provenance: res.body?.scope_provenance ?? null,
          nodes,
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_li_tree_extract",
    label: "LI Tree Extract",
    description:
      "Extract decision/constraint/risk signals and reflection trigger from lineage tree for metacognitive compounding.",
    parameters: Type.Object({
      max_candidates: Type.Optional(
        Type.Number({ description: "Max extracted signals per category (default 12)." })
      ),
      session_id: Type.Optional(Type.String({ description: "Optional session id scoping hint." })),
    }),
    async execute(_id, params) {
      const { max_candidates, session_id } = params as { max_candidates?: number; session_id?: string };
      const effectiveSessionId = String(session_id || getAttachmentRuntime().sessionFrameKey || "").trim();
      if (!effectiveSessionId) {
        return {
          content: [{ type: "text", text: "li extract blocked → active Pi session_id is required" }],
          details: {
            ok: false,
            status: "blocked",
            canonical: true,
            failure_class: "session_scope_required",
            global_fallback: false,
          },
        } as any;
      }
      const cap = Math.max(1, Math.min(50, Number(max_candidates || 12)));
      const queryParts = [
        `selector=summaries`,
        `limit=${encodeURIComponent(String(cap))}`,
        `session_id=${encodeURIComponent(effectiveSessionId)}`,
      ];
      const query = `?${queryParts.join("&")}`;
      const res = await focusaFetchDetailed(`/lineage/tree${query}`);
      if (res.ok && String(res.body?.session_id || "").trim() !== effectiveSessionId) {
        return {
          content: [{ type: "text", text: "li extract blocked → response session scope mismatch" }],
          details: {
            ok: false,
            status: "blocked",
            canonical: true,
            failure_class: "scope_mismatch",
            global_fallback: false,
          },
        } as any;
      }
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

      const summaryNodes = nodes.filter(
        (n: any) => String(n?.node_type || "").toLowerCase() === "summary"
      ).length;
      const summaryRatio = nodes.length > 0 ? summaryNodes / nodes.length : 0;
      const reflectionTrigger =
        depth >= 24 || summaryRatio >= 0.35 || risks.length >= Math.max(3, Math.floor(cap / 3));

      return {
        content: [
          {
            type: "text",
            text: `li extract: decisions=${decisions.length} constraints=${constraints.length} risks=${risks.length} depth=${depth} trigger=${reflectionTrigger ? "yes" : "no"}`,
          },
        ],
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
    description:
      "Record a bounded, inspectable Focusa prediction. Predictions guide decisions; they never override operator steering.",
    parameters: Type.Object({
      prediction_type: Type.String({
        description:
          "Prediction type, e.g. next_action_success|tool_choice|release_failure|stale_state|context_relevance|token_risk|cache_hit|drift_risk|workpoint_resume_success|compaction_recovery",
      }),
      predicted_outcome: Type.String({ description: "Predicted outcome." }),
      confidence: Type.Number({ description: "Confidence from 0.0 to 1.0." }),
      recommended_action: Type.String({ description: "Recommended action if this prediction matters." }),
      why: Type.String({ description: "Evidence-calibrated explanation." }),
      context_refs: Type.Optional(Type.Array(Type.String({ description: "Evidence refs or handles." }))),
      ontology_context: Type.Optional(
        Type.Any({
          description:
            "Bounded ontology refs: object_refs, action_refs, tool_refs, evidence_refs, relation_refs.",
        })
      ),
      project_root: Type.Optional(
        Type.String({
          description: "Optional project root to bind prediction trajectory scope; auto-filled when omitted.",
        })
      ),
      continuity_id: Type.Optional(
        Type.String({
          description:
            "Optional continuity id to bind prediction trajectory scope; auto-filled when omitted.",
        })
      ),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const projectRootGate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (projectRootGate) return projectRootGate;
      const continuityId = String(p.continuity_id || getContinuityId() || "").trim();
      if (!continuityId)
        return blockedToolResponse(
          "focusa_predict_record",
          "prediction",
          "prediction record blocked → typed continuity scope required",
          "scope_mismatch",
          {},
          ["focusa_workpoint_resume", "focusa_project_identity"]
        );
      const scope = buildProjectWorkstreamKey(projectRoot, continuityId);
      const ontologyContext = p.ontology_context || {};
      const payload = {
        scope,
        prediction_type: p.prediction_type,
        predicted_outcome: p.predicted_outcome,
        confidence: p.confidence,
        recommended_action: p.recommended_action,
        why: p.why,
        context_refs: p.context_refs || [],
        // Keep writes compatible with pre-default daemons during rolling upgrades.
        ontology_context: {
          object_refs: Array.isArray(ontologyContext.object_refs) ? ontologyContext.object_refs : [],
          action_refs: Array.isArray(ontologyContext.action_refs) ? ontologyContext.action_refs : [],
          tool_refs: Array.isArray(ontologyContext.tool_refs) ? ontologyContext.tool_refs : [],
          evidence_refs: Array.isArray(ontologyContext.evidence_refs) ? ontologyContext.evidence_refs : [],
          relation_refs: Array.isArray(ontologyContext.relation_refs) ? ontologyContext.relation_refs : [],
        },
      };
      const res = await focusaFetchDetailed("/predictions", {
        method: "POST",
        body: JSON.stringify(payload),
      });
      const body = (res.body || {}) as ScopedResultEnvelope<any>;
      if (!res.ok || body.authority?.status === "blocked") {
        const failureClass = scopedResponseFailureClass(res, body);
        return blockedToolResponse(
          "focusa_predict_record",
          "prediction",
          `prediction record blocked → ${scopedResponseHuman(body, explainWorkLoopResult(res, "prediction write unavailable"))}`,
          failureClass,
          body,
          failureClass === "scope_mismatch"
            ? ["focusa_project_identity", "focusa_workpoint_resume"]
            : ["focusa_predict_recent", "focusa_tool_doctor"]
        );
      }
      const record = body.data?.record || {};
      const prediction = record.prediction || {};
      const predictionId = String(record.record_id || "unknown");
      return {
        content: [{ type: "text", text: renderScopedResultHuman(body) }],
        details: {
          ...body,
          prediction_id: predictionId,
          prediction_type: prediction.prediction_type,
          scope,
          evaluation_hint: `focusa_predict_evaluate prediction_id=${predictionId}`,
          next_tools: ["focusa_predict_evaluate", "focusa_predict_recent"],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_predict_recent",
    label: "Recent Predictions",
    description: "List recent predictions from one typed project/workstream scope.",
    parameters: Type.Object({
      limit: Type.Optional(Type.Number({ description: "Recent prediction count, max 100." })),
      project_root: Type.Optional(Type.String({ description: "Explicit or current verified project root." })),
      continuity_id: Type.Optional(
        Type.String({ description: "Explicit or current workstream continuity id." })
      ),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const gate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (gate) return gate;
      const continuityId = String(p.continuity_id || getContinuityId() || "").trim();
      if (!continuityId)
        return blockedToolResponse(
          "focusa_predict_recent",
          "prediction",
          "predictions recent blocked → typed continuity scope required",
          "scope_mismatch",
          {},
          ["focusa_workpoint_resume"]
        );
      const scope = buildProjectWorkstreamKey(projectRoot, continuityId);
      const query = scopedQueryParams(scope);
      query.set("limit", String(Math.max(1, Math.min(100, Number(p.limit || 20)))));
      const res = await focusaFetchDetailed(`/predictions/recent?${query.toString()}`);
      const body = (res.body || {}) as ScopedResultEnvelope<any>;
      if (!res.ok || body.authority?.status === "blocked") {
        const failureClass = scopedResponseFailureClass(res, body);
        return blockedToolResponse(
          "focusa_predict_recent",
          "prediction",
          `predictions recent blocked → ${scopedResponseHuman(body, "scoped read unavailable")}`,
          failureClass,
          body,
          failureClass === "scope_mismatch"
            ? ["focusa_project_identity", "focusa_workpoint_resume"]
            : ["focusa_tool_doctor"]
        );
      }
      if (!isWorkstreamKey(body.scope) || !sameWorkstream(body.scope, scope)) {
        return blockedToolResponse(
          "focusa_predict_recent",
          "prediction",
          "predictions recent blocked → response scope differs from requested project/workstream",
          "scope_mismatch",
          body,
          ["focusa_project_identity", "focusa_workpoint_resume"]
        );
      }
      const legacyBody = body as any;
      const predictions = Array.isArray(body.data?.predictions)
        ? body.data.predictions
        : Array.isArray(legacyBody.predictions)
          ? legacyBody.predictions
          : [];
      const hint = body.data?.evaluate_hint || legacyBody.evaluate_hint || {};
      return {
        content: [{ type: "text", text: renderScopedResultHuman(body) }],
        details: {
          ...body,
          predictions,
          next_prediction_id: hint.prediction_id || null,
          evaluate_hint: hint,
          scope,
          next_tools: hint.prediction_id
            ? ["focusa_predict_evaluate", "focusa_predict_stats"]
            : ["focusa_predict_record"],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_predict_evaluate",
    label: "Evaluate Prediction",
    description: "Evaluate a prediction inside its exact typed project/workstream scope.",
    parameters: Type.Object({
      prediction_id: Type.String({ description: "Prediction id to evaluate." }),
      actual_outcome: Type.String({ description: "Observed actual outcome." }),
      score: Type.Optional(Type.Number({ description: "Score 0.0 to 1.0." })),
      learning_signal_ref: Type.Optional(
        Type.String({ description: "Optional scoped learning signal ref." })
      ),
      project_root: Type.Optional(Type.String({ description: "Explicit or current verified project root." })),
      continuity_id: Type.Optional(
        Type.String({ description: "Explicit or current workstream continuity id." })
      ),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const gate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (gate) return gate;
      const continuityId = String(p.continuity_id || getContinuityId() || "").trim();
      if (!continuityId)
        return blockedToolResponse(
          "focusa_predict_evaluate",
          "prediction",
          "prediction evaluate blocked → typed continuity scope required",
          "scope_mismatch",
          {},
          ["focusa_workpoint_resume"]
        );
      const scope = buildProjectWorkstreamKey(projectRoot, continuityId);
      const res = await focusaFetchDetailed(`/predictions/${encodeURIComponent(p.prediction_id)}/evaluate`, {
        method: "POST",
        body: JSON.stringify({
          scope,
          actual_outcome: p.actual_outcome,
          score: p.score,
          learning_signal_ref: p.learning_signal_ref,
        }),
      });
      const body = (res.body || {}) as ScopedResultEnvelope<any>;
      if (!res.ok || body.authority?.status === "blocked") {
        const failureClass = scopedResponseFailureClass(res, body);
        return blockedToolResponse(
          "focusa_predict_evaluate",
          "prediction",
          `prediction evaluate blocked → ${scopedResponseHuman(body, "scoped evaluation unavailable")}`,
          failureClass,
          body,
          failureClass === "scope_mismatch"
            ? ["focusa_predict_recent", "focusa_workpoint_resume"]
            : ["focusa_predict_recent", "focusa_tool_doctor"]
        );
      }
      return {
        content: [{ type: "text", text: renderScopedResultHuman(body) }],
        details: {
          ...body,
          prediction_id: p.prediction_id,
          scope,
          next_tools: ["focusa_predict_stats", "focusa_metacog_retrieve"],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_preload_profiles",
    label: "Preload Profiles",
    description: "List bounded Spec 111 agent bootstrap profiles.",
    parameters: strictObject({}),
    async execute() {
      const res = await callSpec80Tool("focusa_preload_profiles", "/preload/profiles", {}, { method: "GET" });
      const profiles = Array.isArray(res.body?.profiles) ? res.body.profiles : [];
      const profileList = profiles
        .map((profile: any) => `${String(profile.id || "unknown")} (${String(profile.label || "unlabeled")})`)
        .join(", ");
      const defaultProfile = String(res.body?.default_profile || "unknown");
      return spec80Result(
        "focusa_preload_profiles",
        "/v1/preload/profiles",
        {},
        res,
        `preload profiles → ${profiles.length} available: ${profileList || "none"}; default=${defaultProfile}`,
        "preload profiles unavailable",
        {
          kind: res.ok ? "ok" : "blocked",
          fields: [
            { label: "profiles", value: profileList || "none" },
            { label: "default_profile", value: defaultProfile },
            { label: "human_readable", value: res.body?.human_readable || null },
          ],
          nextTools: ["focusa_preload_build"],
        }
      );
    },
  });

  const preloadProfile = Type.Optional(
    Type.Union(
      [
        Type.Literal("rules_only"),
        Type.Literal("rules_and_context"),
        Type.Literal("budget_light"),
        Type.Literal("budget_deep"),
      ],
      {
        description: "Preload profile id from focusa_preload_profiles. Defaults to rules_and_context.",
      }
    )
  );
  const preloadScopeParams = {
    profile: preloadProfile,
    project_root: Type.Optional(Type.String({ minLength: 1, maxLength: 4096 })),
    continuity_id: Type.Optional(Type.String({ minLength: 1, maxLength: 256 })),
  };
  const preloadReadTools: Array<[string, string, string, string[]]> = [
    [
      "focusa_preload_build",
      "Build Preload Packet",
      "build",
      ["focusa_preload_write", "focusa_preload_doctor"],
    ],
    [
      "focusa_preload_render",
      "Render Preload Packet",
      "render",
      ["focusa_preload_verify", "focusa_preload_write"],
    ],
    [
      "focusa_preload_verify",
      "Verify Preload Packet",
      "verify",
      ["focusa_preload_write", "focusa_preload_doctor"],
    ],
    ["focusa_preload_doctor", "Doctor Preload Scope", "doctor", ["focusa_preload_profiles"]],
  ];

  for (const [name, label, action, nextTools] of preloadReadTools) {
    pi.registerTool({
      name,
      label,
      description: `${label} through the scoped Spec 111 preload API.`,
      parameters: strictObject(preloadScopeParams),
      async execute(_id, params) {
        const input = params as Record<string, any>;
        const projectRoot = await resolveFocusaToolProjectRoot(input.project_root);
        const request: Record<string, any> = {
          ...input,
          project_root: projectRoot,
          continuity_id: input.continuity_id || getContinuityId() || ensureContinuityId(projectRoot),
          working_subpath_id: process.env.FOCUSA_WORKING_SUBPATH_ID || "primary",
        };
        const res = await callSpec80Tool(name, `/preload/${action}`, request, { method: "POST" });
        const bodyStatus = String(res.body?.status || (res.ok ? "completed" : "failed")).toLowerCase();
        const functionallyFailed = ["failed", "error", "blocked"].includes(bodyStatus);
        const functionalResult = { ...res, ok: res.ok && !functionallyFailed };
        const failureMessage = String(
          res.body?.human_readable || res.body?.error?.message || `${action} preload unavailable`
        );
        return spec80Result(
          name,
          `/v1/preload/${action}`,
          request,
          functionalResult,
          `${action} preload → ${bodyStatus}${res.body?.human_readable ? ` | ${res.body.human_readable}` : ""}`,
          `${action} preload failed: ${failureMessage}`,
          {
            kind: functionallyFailed ? "blocked" : bodyStatus === "degraded" ? "advisory" : "ok",
            failureClass: functionallyFailed ? "validation_rejected" : null,
            fields: [
              { label: "status", value: bodyStatus },
              { label: "profile", value: request.profile || "rules_and_context" },
              { label: "human_readable", value: res.body?.human_readable || null },
            ],
            nextTools: [...nextTools],
          }
        );
      },
    });
  }

  pi.registerTool({
    name: "focusa_preload_write",
    label: "Write Preload Packet",
    description: "Write a Spec 111 preload packet to an allowlisted target with an idempotency key.",
    parameters: strictObject({
      profile: preloadProfile,
      target: Type.String({ minLength: 1, maxLength: 4096 }),
      idempotency_key: Type.String({ minLength: 1, maxLength: 256 }),
      overwrite: Type.Optional(Type.Boolean()),
    }),
    async execute(_id, params) {
      const p = params as Record<string, any>;
      const request = {
        profile_id: p.profile || "rules_and_context",
        target_path: p.target,
        idempotency_key: p.idempotency_key,
        overwrite: p.overwrite || false,
      };
      const res = await callSpec80Tool("focusa_preload_write", "/preload/write", request, {
        method: "POST",
        writer: true,
      });
      return spec80Result(
        "focusa_preload_write",
        "/v1/preload/write",
        request,
        res,
        `preload write → ${res.body?.target_path || p.target}`,
        "preload write blocked",
        {
          kind: res.ok ? "ok" : "blocked",
          nextTools: ["focusa_preload_verify", "focusa_preload_receipt_preview", "focusa_preload_doctor"],
        }
      );
    },
  });

  pi.registerTool({
    name: "focusa_preload_receipt_preview",
    label: "Preview Preload Receipt",
    description: "Preview a Spec 111 bootstrap delivery receipt without committing it.",
    parameters: strictObject({ profile: preloadProfile }),
    async execute(_id, params) {
      const request = params as Record<string, any>;
      const res = await callSpec80Tool(
        "focusa_preload_receipt_preview",
        "/preload/receipt-preview",
        request,
        { method: "POST" }
      );
      return spec80Result(
        "focusa_preload_receipt_preview",
        "/v1/preload/receipt-preview",
        request,
        res,
        `preload receipt preview → ${res.body?.status || "completed"}`,
        "preload receipt preview unavailable",
        {
          kind: res.ok ? "ok" : "blocked",
          nextTools: ["focusa_preload_receipt_commit", "focusa_preload_doctor"],
        }
      );
    },
  });

  pi.registerTool({
    name: "focusa_preload_receipt_commit",
    label: "Commit Preload Receipt",
    description: "Commit an idempotent Spec 111 bootstrap delivery receipt.",
    parameters: strictObject({
      profile: preloadProfile,
      idempotency_key: Type.String({ minLength: 1, maxLength: 256 }),
    }),
    async execute(_id, params) {
      const p = params as Record<string, any>;
      const request = { profile: p.profile || "rules_and_context", idempotency_key: p.idempotency_key };
      const res = await callSpec80Tool("focusa_preload_receipt_commit", "/preload/receipt-commit", request, {
        method: "POST",
        writer: true,
      });
      return spec80Result(
        "focusa_preload_receipt_commit",
        "/v1/preload/receipt-commit",
        request,
        res,
        `preload receipt commit → ${res.body?.status || "completed"}`,
        "preload receipt commit blocked",
        { kind: res.ok ? "ok" : "blocked", nextTools: ["focusa_receipt_verify", "focusa_preload_doctor"] }
      );
    },
  });

  pi.registerTool({
    name: "focusa_predict_stats",
    label: "Prediction Stats",
    description: "Report prediction calibration for one typed project/workstream scope.",
    parameters: Type.Object({
      project_root: Type.Optional(Type.String({ description: "Explicit or current verified project root." })),
      continuity_id: Type.Optional(
        Type.String({ description: "Explicit or current workstream continuity id." })
      ),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const gate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (gate) return gate;
      const continuityId = String(p.continuity_id || getContinuityId() || "").trim();
      if (!continuityId)
        return blockedToolResponse(
          "focusa_predict_stats",
          "prediction",
          "prediction stats blocked → typed continuity scope required",
          "scope_mismatch",
          {},
          ["focusa_workpoint_resume"]
        );
      const scope = buildProjectWorkstreamKey(projectRoot, continuityId);
      const query = scopedQueryParams(scope);
      const res = await focusaFetchDetailed(`/predictions/stats?${query.toString()}`);
      const body = (res.body || {}) as ScopedResultEnvelope<any>;
      if (!res.ok || body.authority?.status === "blocked") {
        const failureClass = scopedResponseFailureClass(res, body);
        return blockedToolResponse(
          "focusa_predict_stats",
          "prediction",
          `prediction stats blocked → ${scopedResponseHuman(body, "scoped stats unavailable")}`,
          failureClass,
          body,
          failureClass === "scope_mismatch"
            ? ["focusa_predict_recent", "focusa_workpoint_resume"]
            : ["focusa_predict_recent", "focusa_tool_doctor"]
        );
      }
      return {
        content: [{ type: "text", text: renderScopedResultHuman(body) }],
        details: {
          ...body,
          ...((body as any).data || (body as any).stats || {}),
          scope,
          next_tools: ["focusa_predict_record", "focusa_predict_recent"],
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_epistemic_operation",
    label: "Epistemic Operation",
    description:
      "Invoke one exact generated Spec 138/138A operation through durable typed API authority; the client never settles authority locally.",
    parameters: Type.Object({
      operation_id: Type.Union(SPEC138_OPERATIONS.map((row) => Type.Literal(row.operation_id)) as any),
      id: Type.Optional(Type.String({ description: "Value for canonical {id} path segments." })),
      event: Type.Optional(Type.Any({ description: "Typed ScopedAuthorityEvent required for mutations." })),
      project_root: Type.Optional(Type.String({ description: "Explicit or current verified project root." })),
      continuity_id: Type.Optional(Type.String({ description: "Explicit or current continuity id." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const descriptor = spec138Operation(String(p.operation_id || ""));
      if (!descriptor)
        return blockedToolResponse(
          "focusa_epistemic_operation", "metacognition", "epistemic operation blocked → unknown operation id",
          "validation_rejected", {}, ["focusa_tool_describe"]
        );
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const gate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (gate) return gate;
      const continuityId = String(p.continuity_id || getContinuityId() || "").trim();
      if (!continuityId)
        return blockedToolResponse(
          "focusa_epistemic_operation", "metacognition", "epistemic operation blocked → typed continuity scope required",
          "scope_mismatch", {}, ["focusa_workpoint_resume"]
        );
      if (descriptor.method === "POST" && !p.event)
        return blockedToolResponse(
          "focusa_epistemic_operation", "metacognition", "epistemic mutation blocked → typed event required",
          "validation_rejected", { operation_id: descriptor.operation_id }, ["focusa_tool_describe"]
        );
      let path: string;
      try { path = bindSpec138OperationPath(descriptor.path, p.id); }
      catch (error) {
        return blockedToolResponse(
          "focusa_epistemic_operation", "metacognition", `epistemic operation blocked → ${String(error)}`,
          "validation_rejected", { operation_id: descriptor.operation_id }, ["focusa_tool_describe"]
        );
      }
      const scope = buildProjectWorkstreamKey(projectRoot, continuityId);
      const endpoint = descriptor.method === "GET"
        ? `${path}?${scopedQueryParams(scope).toString()}`
        : path;
      const res = await focusaFetchDetailed(endpoint, descriptor.method === "POST" ? {
        method: "POST",
        body: JSON.stringify({ operation_id: descriptor.operation_id, scope, event: p.event }),
      } : undefined);
      return {
        content: [{ type: "text", text: `${descriptor.label} → ${res.body?.status || (res.ok ? "completed" : "blocked")}` }],
        details: {
          ok: res.ok, status: res.body?.status, operation: descriptor,
          authority: res.body?.authority, response: res.body,
          project_root: projectRoot, continuity_id: continuityId,
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_prediction_authority",
    label: "Prediction Authority",
    description:
      "Append or project immutable Spec 138 prediction/outcome/learning/transfer authority in one typed project/workstream scope.",
    parameters: Type.Object({
      action: Type.Union([Type.Literal("append"), Type.Literal("projection")]),
      event: Type.Optional(Type.Any({ description: "ScopedAuthorityEvent when action=append." })),
      project_root: Type.Optional(Type.String({ description: "Explicit or current verified project root." })),
      continuity_id: Type.Optional(Type.String({ description: "Explicit or current continuity id." })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const projectRoot = await resolveFocusaToolProjectRoot(p.project_root);
      const gate = projectRootConfirmationGate(projectRoot, p.project_root);
      if (gate) return gate;
      const continuityId = String(p.continuity_id || getContinuityId() || "").trim();
      if (!continuityId)
        return {
          content: [{ type: "text", text: "prediction authority blocked → typed continuity scope required" }],
          details: {
            ok: false,
            status: "blocked",
            recovery: "Provide continuity_id or bind a verified project workstream.",
          },
        } as any;
      if (p.action === "append" && !p.event)
        return {
          content: [{ type: "text", text: "prediction authority append blocked → event required" }],
          details: { ok: false, status: "blocked", recovery: "Provide one ScopedAuthorityEvent." },
        } as any;
      const scope = buildProjectWorkstreamKey(projectRoot, continuityId);
      const endpoint =
        p.action === "append" ? "/prediction-authority/events" : "/prediction-authority/projection";
      const res = await focusaFetchDetailed(endpoint, {
        method: "POST",
        body: JSON.stringify(p.action === "append" ? { scope, event: p.event } : { scope }),
      });
      const summary = `prediction authority ${p.action} → ${res.body?.status || (res.ok ? "completed" : "blocked")}`;
      return {
        content: [{ type: "text", text: summary }],
        details: {
          ok: res.ok,
          status: res.body?.status,
          response: res.body,
          project_root: projectRoot,
          continuity_id: continuityId,
        },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_tool_search",
    label: "Focusa Tool Search",
    description:
      "Search the bounded Focusa capability catalog before loading full schemas. Returns ranked metadata, scope, side-effect, skill, documentation, and discovery refs so agents can select the narrowest tool under token budget.",
    parameters: Type.Object({
      query: Type.String({ description: "Capability, action, object, failure, or workflow search text." }),
      family: Type.Optional(Type.String({ description: "Optional exact Focusa tool family filter." })),
      limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 50, default: 10 })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const query = String(p.query || "")
        .trim()
        .toLowerCase();
      const terms = query.split(/\s+/).filter(Boolean);
      const limit = Math.max(1, Math.min(50, Number(p.limit || 10)));
      const affordances = new Map(buildFocusaToolAffordanceCatalog().map((item) => [item.name, item]));
      const results = FOCUSA_TOOL_CONTRACTS.filter((contract) => !p.family || contract.family === p.family)
        .map((contract) => {
          const affordance = affordances.get(contract.name);
          const haystack = [
            contract.name,
            contract.label,
            contract.purpose,
            contract.family,
            contract.ontology_action,
            ...contract.ontology_objects,
            ...(affordance?.when_to_use || []),
            ...(affordance?.failure_classes || []),
          ]
            .join(" ")
            .toLowerCase();
          const score = terms.reduce(
            (sum, term) => sum + (contract.name.includes(term) ? 5 : haystack.includes(term) ? 1 : 0),
            query === contract.name.toLowerCase() ? 20 : 0
          );
          return { contract, affordance, score };
        })
        .filter((item) => !terms.length || item.score > 0)
        .sort((a, b) => b.score - a.score || a.contract.name.localeCompare(b.contract.name))
        .slice(0, limit)
        .map(({ contract, affordance, score }) => ({
          name: contract.name,
          label: contract.label,
          family: contract.family,
          purpose: contract.purpose,
          score,
          side_effect_profile: contract.side_effect_profile,
          scope_requirement: contract.scope_requirement,
          skill_refs: [`skill:focusa`, `skill:focusa-${contract.family.replaceAll("_", "-")}`],
          likely_next_tools: affordance?.likely_next_tools || [],
          describe_with: "focusa_tool_describe",
        }));
      const payload = {
        schema: "focusa.tool_search_result.v1",
        query,
        family: p.family || null,
        count: results.length,
        results,
        next_tools: results.length ? ["focusa_tool_describe"] : ["focusa_tool_bundle"],
      };
      return {
        content: [
          {
            type: "text",
            text: modelVisibleDiscoveryPayload(
              `tool search → ${results.length} ranked result(s)`,
              payload
            ),
          },
        ],
        details: payload,
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_tool_describe",
    label: "Focusa Tool Describe",
    description:
      "Cold-load one complete runtime Focusa tool definition after search. Returns strict input/output schemas, operational guidance, authority, side effects, failures, recovery, dependencies, skills, docs, and protocol bindings without loading unrelated tools.",
    parameters: Type.Object({
      name: Type.String({ description: "Exact Focusa Pi tool name returned by focusa_tool_search." }),
      include_schemas: Type.Optional(Type.Boolean({ default: true })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const name = String(p.name || "").trim();
      const contract = FOCUSA_TOOL_CONTRACTS.find((item) => item.name === name);
      const affordance = buildFocusaToolAffordanceCatalog().find((item) => item.name === name);
      const definition = agentFirstToolDefinitions.get(name);
      if (!contract || !affordance || !definition) {
        return blockedToolResponse(
          "focusa_tool_describe",
          "traversal",
          `tool describe blocked → unknown capability ${name}`,
          "not_found",
          { name },
          ["focusa_tool_search"]
        );
      }
      const descriptor = {
        name: contract.name,
        family: contract.family,
        label: contract.label,
        purpose: contract.purpose,
        api_routes: contract.api_routes,
        cli_commands: contract.cli_commands,
        docs_ref: contract.doc_path,
        scope_requirement: contract.scope_requirement,
        authority_requirement: contract.authority_requirement,
        side_effect_profile: contract.side_effect_profile,
        affordance,
        input_schema: p.include_schemas === false ? undefined : definition.parameters,
        output_schema: p.include_schemas === false ? undefined : definition.outputSchema,
        schema_loading: p.include_schemas === false ? "metadata_only" : "cold_loaded",
      };
      const payload = {
        schema: "focusa.tool_description.v2",
        descriptor,
        next_tools: affordance.likely_next_tools,
      };
      return {
        content: [
          {
            type: "text",
            text: modelVisibleDiscoveryPayload(`tool describe → ${name}`, payload),
          },
        ],
        details: payload,
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_tool_graph",
    label: "Focusa Tool Graph",
    description:
      "Traverse the bounded capability dependency and likely-next graph from one tool or family. Use it to plan a valid workflow sequence without loading the complete registry or inventing dependencies.",
    parameters: Type.Object({
      anchor: Type.String({ description: "Exact tool name or family." }),
      depth: Type.Optional(Type.Integer({ minimum: 1, maximum: 4, default: 2 })),
      limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 100, default: 40 })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const depth = Math.max(1, Math.min(4, Number(p.depth || 2)));
      const limit = Math.max(1, Math.min(100, Number(p.limit || 40)));
      const catalog = buildFocusaToolAffordanceCatalog();
      const byName = new Map(catalog.map((item) => [item.name, item]));
      let frontier = FOCUSA_TOOL_CONTRACTS.filter(
        (item) => item.name === p.anchor || item.family === p.anchor
      ).map((item) => item.name);
      if (!frontier.length) {
        const alternatives = [...new Set(FOCUSA_TOOL_CONTRACTS.map((item) => item.family))]
          .sort()
          .slice(0, 12);
        return blockedToolResponse(
          "focusa_tool_graph",
          "traversal",
          `tool graph blocked → unknown tool or family ${p.anchor}`,
          "not_found",
          { anchor: p.anchor, valid_family_examples: alternatives },
          ["focusa_tool_search", "focusa_tool_bundle"]
        );
      }
      const seen = new Set(frontier);
      const edges: Array<{ from: string; to: string; relation: string }> = [];
      for (let level = 0; level < depth && frontier.length && seen.size <= limit; level += 1) {
        const next: string[] = [];
        for (const from of frontier) {
          for (const to of byName.get(from)?.likely_next_tools || []) {
            edges.push({ from, to, relation: "likely_next" });
            if (!seen.has(to) && seen.size < limit) {
              seen.add(to);
              next.push(to);
            }
          }
        }
        frontier = next;
      }
      const payload = {
        schema: "focusa.tool_graph.v1",
        anchor: p.anchor,
        depth,
        nodes: [...seen],
        edges,
        next_tools: ["focusa_tool_describe", "focusa_tool_bundle"],
      };
      return {
        content: [
          {
            type: "text",
            text: modelVisibleDiscoveryPayload(
              `tool graph → anchor=${p.anchor} nodes=${seen.size} edges=${edges.length}`,
              payload
            ),
          },
        ],
        details: payload,
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_tool_bundle",
    label: "Focusa Tool Bundle",
    description:
      "Load a bounded family bundle of capability metadata and optionally strict schemas. Use after search or graph traversal when one workflow needs several related tools; avoid broad all-tool prompt injection.",
    parameters: Type.Object({
      family: Type.String({ description: "Exact Focusa tool family." }),
      include_schemas: Type.Optional(Type.Boolean({ default: false })),
      limit: Type.Optional(Type.Integer({ minimum: 1, maximum: 50, default: 25 })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const limit = Math.max(1, Math.min(50, Number(p.limit || 25)));
      const items = FOCUSA_TOOL_CONTRACTS.filter((item) => item.family === p.family)
        .slice(0, limit)
        .map((contract) => {
          const definition = agentFirstToolDefinitions.get(contract.name);
          return {
            name: contract.name,
            family: contract.family,
            label: contract.label,
            purpose: contract.purpose,
            api_routes: contract.api_routes,
            cli_commands: contract.cli_commands,
            docs_ref: contract.doc_path,
            side_effect_profile: contract.side_effect_profile,
            input_schema: p.include_schemas ? definition?.parameters : undefined,
            output_schema: p.include_schemas ? definition?.outputSchema : undefined,
          };
        });
      if (!items.length) {
        const alternatives = [...new Set(FOCUSA_TOOL_CONTRACTS.map((item) => item.family))]
          .sort()
          .slice(0, 12);
        return blockedToolResponse(
          "focusa_tool_bundle",
          "traversal",
          `tool bundle blocked → unknown family ${p.family}`,
          "not_found",
          { family: p.family, valid_family_examples: alternatives },
          ["focusa_tool_search"]
        );
      }
      const payload = {
        schema: "focusa.tool_bundle.v1",
        family: p.family,
        count: items.length,
        schema_loading: p.include_schemas ? "cold_loaded" : "metadata_only",
        tools: items,
        next_tools: ["focusa_tool_describe", "focusa_tool_graph"],
      };
      return {
        content: [
          {
            type: "text",
            text: modelVisibleDiscoveryPayload(
              `tool bundle → family=${p.family} count=${items.length}`,
              payload
            ),
          },
        ],
        details: payload,
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_agent_card",
    label: "Focusa Agent Card",
    description:
      "Read a compact, versioned Focusa Agent Card for cross-harness discovery. Returns interfaces, auth methods, progressive-discovery entry points, capability families, registry digest guidance, and extended-card routes without loading full schemas.",
    parameters: Type.Object({
      include_families: Type.Optional(Type.Boolean({ default: true })),
    }),
    async execute(_id, params) {
      const p = params as any;
      const families = [...new Set(FOCUSA_TOOL_CONTRACTS.map((item) => item.family))].sort();
      const registryDigest = createHash("sha256")
        .update(
          JSON.stringify(
            FOCUSA_TOOL_CONTRACTS.map((item) => ({
              name: item.name,
              family: item.family,
              scope: item.scope_requirement,
              side_effect: item.side_effect_profile,
            }))
          )
        )
        .digest("hex");
      let runtimeVersion = "unknown";
      try {
        const health = await focusaFetch("/health");
        runtimeVersion = String(health?.version || "unknown");
      } catch {
        // Agent Card remains useful offline; unknown is more truthful than a stale hard-coded version.
      }
      const card = {
        schema: "focusa.agent_card.v1",
        name: "Focusa",
        version: runtimeVersion,
        description:
          "Agent-first cognitive infrastructure with scoped Workpoints, Trajectory, evidence, recovery, browser interoperability, and cross-harness contracts.",
        interfaces: ["pi", "mcp", "openai-functions", "cli", "rest"],
        authentication: ["bearer", "device_pairing", "local_trusted"],
        capabilities: {
          streaming: true,
          durable_tasks: true,
          list_changed: true,
          progressive_discovery: true,
          structured_output: true,
        },
        discovery_tools: [
          "focusa_tool_search",
          "focusa_tool_describe",
          "focusa_tool_graph",
          "focusa_tool_bundle",
        ],
        capability_count: FOCUSA_TOOL_CONTRACTS.length,
        capability_families: p.include_families === false ? undefined : families,
        registry_digest: `sha256:${registryDigest}`,
        registry_digest_ref: "/v1/agent/card",
        protocol_bindings: {
          mcp: "focusa_agent_card",
          openai_functions: "/v1/capabilities/openai-functions",
          rest: "/v1/agent/card",
          cli: "focusa capabilities",
        },
        extended_card_path: "/v1/agent/card",
      };
      return {
        content: [
          {
            type: "text",
            text: modelVisibleDiscoveryPayload("Focusa Agent Card", card),
          },
        ],
        details: { ...card, next_tools: card.discovery_tools },
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_browser_capabilities_intake",
    label: "Browser Capabilities Intake",
    description:
      "Validate and govern a UIAI or WebMCP browser capability manifest. Binds page tools to one session and origin, treats page safety annotations as untrusted, requires confirmation/evidence for mutation, and returns Focusa browser capability descriptors without executing them.",
    parameters: Type.Object({
      session_id: Type.String({ description: "Exact active UIAI browser session identifier." }),
      origin: Type.String({ description: "Absolute http(s) page origin bound to these capabilities." }),
      source: Type.Optional(
        Type.Union([Type.Literal("webmcp"), Type.Literal("uiai"), Type.Literal("page_manifest")])
      ),
      trusted_origin: Type.Optional(Type.Boolean({ default: false })),
      tools: Type.Array(
        Type.Object({
          name: Type.String({ description: "Page/browser tool identifier." }),
          description: Type.Optional(Type.String()),
          inputSchema: Type.Object({}, { additionalProperties: true }),
          annotations: Type.Optional(Type.Object({}, { additionalProperties: true })),
        }),
        { maxItems: 50 }
      ),
      project_root: Type.Optional(Type.String()),
      continuity_id: Type.Optional(Type.String()),
      workpoint_id: Type.Optional(Type.String()),
    }),
    async execute(_id, params) {
      const res = await focusaFetchDetailed("/browser/capabilities/intake", {
        method: "POST",
        body: JSON.stringify(params),
      });
      const body = res.body || {};
      if (!res.ok) {
        return blockedToolResponse(
          "focusa_browser_capabilities_intake",
          "browser_interop",
          `browser capability intake blocked → ${scopedResponseHuman(body, "manifest rejected")}`,
          (body.failure_class || "validation_rejected") as FocusaFailureClass,
          body,
          ["focusa_browser_diagnostics_intake", "focusa_tool_doctor"]
        );
      }
      return {
        content: [
          {
            type: "text",
            text: `browser capability intake → accepted=${body.capability_count || 0} session=${body.session_binding?.session_id || "unknown"} advisory_only=true`,
          },
        ],
        details: body,
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_ontology_scope_migration",
    label: "Ontology Scope Migration",
    description:
      "Dry-run, apply, inspect, or roll back granular legacy ontology scope migration. Apply/rollback require explicit confirmation and per-record evidence; ownership is never inferred.",
    parameters: Type.Object({
      action: Type.Union([
        Type.Literal("dry_run"),
        Type.Literal("apply"),
        Type.Literal("status"),
        Type.Literal("rollback"),
      ]),
      migration_id: Type.Optional(
        Type.String({ description: "Stable UUID for apply/retry or rollback target." })
      ),
      rollback_id: Type.Optional(Type.String({ description: "Stable UUID for idempotent rollback/retry." })),
      selections: Type.Optional(
        Type.Array(
          Type.Object({
            record_kind: Type.Union([
              Type.Literal("object"),
              Type.Literal("link"),
              Type.Literal("proposal"),
              Type.Literal("verification"),
              Type.Literal("working_set_refresh"),
              Type.Literal("delta"),
              Type.Literal("pre_proposal"),
            ]),
            source_hash: Type.String(),
            evidence_refs: Type.Array(Type.String(), { minItems: 1 }),
          })
        )
      ),
      evidence_refs: Type.Optional(Type.Array(Type.String(), { minItems: 1 })),
      confirm: Type.Optional(Type.Boolean({ description: "Required true for apply or rollback mutation." })),
    }),
    async execute(_id, params) {
      const mutation = params.action === "apply" || params.action === "rollback";
      if (mutation && params.confirm !== true) {
        return blockedToolResponse(
          "focusa_ontology_scope_migration",
          "ontology",
          `ontology scope migration ${params.action} blocked → explicit confirm=true required`,
          "approval_required",
          { action: params.action, canonical: false, mutation: true },
          ["focusa_ontology_scope_migration", "focusa_project_verify"]
        );
      }
      const res = await focusaFetchDetailed("/ontology/scope-migrations", {
        method: "POST",
        body: JSON.stringify({
          action: params.action,
          migration_id: params.migration_id,
          rollback_id: params.rollback_id,
          selections: params.selections || [],
          evidence_refs: params.evidence_refs || [],
        }),
      });
      const body = res.body || {};
      if (!res.ok) {
        return blockedToolResponse(
          "focusa_ontology_scope_migration",
          "ontology",
          `ontology scope migration blocked → ${scopedResponseHuman(body, "request rejected")}`,
          (body.failure_class || "validation_rejected") as FocusaFailureClass,
          body,
          ["focusa_project_verify", "focusa_tool_doctor"]
        );
      }
      return {
        content: [
          {
            type: "text",
            text: `ontology scope migration → action=${params.action} status=${body.status || "unknown"} candidates=${body.candidate_count || 0} receipts=${body.receipts?.length || 0}`,
          },
        ],
        details: body,
      } as any;
    },
  });

  pi.registerTool({
    name: "focusa_browser_workflow_plan",
    label: "Browser Workflow Plan",
    description:
      "Build the governed UIAI/WebMCP sequence for one browser operation before action. Returns health, read/source, diagnostics, snapshot refs, mutation confirmation, bound execution, evidence intake, Workpoint linkage, and session cleanup steps.",
    parameters: Type.Object({
      operation: Type.String({ description: "Bounded browser action intent." }),
      mutation: Type.Optional(Type.Boolean({ default: false })),
      webmcp_available: Type.Optional(Type.Boolean({ default: false })),
      session_id: Type.Optional(Type.String()),
      origin: Type.Optional(Type.String()),
    }),
    async execute(_id, params) {
      const res = await focusaFetchDetailed("/browser/workflow/plan", {
        method: "POST",
        body: JSON.stringify(params),
      });
      const body = res.body || {};
      if (!res.ok) {
        return blockedToolResponse(
          "focusa_browser_workflow_plan",
          "browser_interop",
          `browser workflow plan blocked → ${scopedResponseHuman(body, "plan rejected")}`,
          (body.failure_class || "validation_rejected") as FocusaFailureClass,
          body,
          ["focusa_browser_diagnostics_intake", "focusa_tool_doctor"]
        );
      }
      return {
        content: [
          {
            type: "text",
            text: `browser workflow plan → route=${body.route} mutation=${body.mutation} steps=${body.steps?.length || 0}`,
          },
        ],
        details: body,
      } as any;
    },
  });
}
