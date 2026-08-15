// Session lifecycle events — ONE handler per event type (merged)
// Spec: §11 (outage audit + backoff), §30 (SSE metacog), §34.2A (instance),
//        §35.1 (auto-frame), §36.4 (resume), §36.5 (fork/tree), §37.5 (flags),
//        §35.8 (session display ownership), §37.9 (Context Core), §37.10 (cross-surface SSE),
//        §38.3 (health toggle)

import type { ExtensionAPI } from "@earendil-works/pi-coding-agent";
import { existsSync } from "fs";
import { join } from "path";
import { classifyPiSessionProject, persistedProjectRootFromState } from "./session-classification.js";
import { DaemonRecoveryGate } from "./daemon-recovery-gate.js";
import {
  getAttachmentRuntime,
  focusaFetch,
  focusaPost,
  checkFocusa,
  kickstartFocusaDaemon,
  persistState,
  persistAuthoritativeState,
  getFocusState,
  createPiFrame,
  ensurePiFrame,
  classifyCurrentAsk,
  isNonTaskStatusLikeText,
  isGenericPiFrameForCwd,
  trimFrameText,
  stripQuotedFocusaContext,
  ensureContinuityId,
  adoptPersistedContinuityForSession,
  isProjectRootAuthoritySafe,
  isWorkpointPacketScopedToCurrentSession,
  normalizeWorkpointResumePacketEnvelope,
  refreshTrajectoryClarityLifecycle,
  stampWorkpointPacketForCurrentPiSession,
  resetPiSessionScopedState,
  adoptPiProjectRoot,
  resolvePiProjectRootCandidate,
  normalizeProjectRoot,
  confirmPiProjectRoot,
  projectRootConfirmationRequired,
  projectRootConfirmationSummary,
  buildFocusaSessionIdentity,
  syncRuntimeFieldsToScopeStore,
  getActiveWorkpointPacket,
  setActiveWorkpointPacket,
  getActiveWorkpointSummary,
  setActiveWorkpointSummary,
  getLastTrajectoryClarity,
  setLastTrajectoryClarity,
  getLastProjectVerify,
  setLastProjectVerify,
  setLatestReportSummary,
  getLatestReportSummary,
  getLastProjectRootResolution,
  setLastProjectRootResolution,
  setLastProjectIdentity,
  setTotalCompactions,
  getTurnCount,
  setTurnCount,
  getSessionCwd,
  getContinuityId,
} from "./state.js";
import { loadPersistedRecoveryState } from "./persistence.js";
import { measureNativeSessionPressure, type NativeSessionPressureV1 } from "./session-pressure.js";
import { queueLifecycleAdvisory } from "./lifecycle-advisory.js";
import { pushDelta } from "./tools.js";
import { LifecycleGenerationGuard } from "./lifecycle-guard.js";

// §30 + §37.10: SSE connection for metacognitive + cross-surface events
let sseAbort: AbortController | null = null;
let sseReconnectTimer: ReturnType<typeof setTimeout> | null = null;
const healthLifecycle = new LifecycleGenerationGuard();

export function nativeSessionPressureOperatorAction(
  pressure: Pick<NativeSessionPressureV1, "recommended_action">
): string | null {
  return pressure.recommended_action === "rollover" ? "/focusa-rollover execute" : null;
}

function refreshNativeSessionPressure(ctx: any, reason: string, entries?: any[]): void {
  const sessionFile = ctx?.sessionManager?.getSessionFile?.();
  const pressure = measureNativeSessionPressure({
    adapter: "pi",
    sessionFile,
    entries,
  });
  getAttachmentRuntime().lastNativeSessionPressure = pressure;
  const operatorAction = nativeSessionPressureOperatorAction(pressure);
  const noticeKey = `${pressure.posture}:${pressure.recommended_action}`;
  if (pressure.posture === "normal") {
    if (getAttachmentRuntime().lastNativeSessionPressureNoticeKey)
      ctx?.ui?.setStatus?.("focusa-pressure", undefined);
    getAttachmentRuntime().lastNativeSessionPressureNoticeKey = "";
  } else if (noticeKey !== getAttachmentRuntime().lastNativeSessionPressureNoticeKey) {
    getAttachmentRuntime().lastNativeSessionPressureNoticeKey = noticeKey;
    const mib = Math.ceil(pressure.session_bytes / (1024 * 1024));
    const actionSuffix = operatorAction ? ` · run ${operatorAction}` : "";
    ctx?.ui?.setStatus?.("focusa-pressure", `native session ${pressure.posture} · ${mib} MiB${actionSuffix}`);
    if (pressure.posture !== "soft_pressure") {
      const actionCopy = operatorAction
        ? `Live /compact cannot shrink this append-only segment. Run ${operatorAction} to checkpoint, seal, migrate, open a new session, and verify resume.`
        : `Required action: ${pressure.recommended_action}.`;
      ctx?.ui?.notify?.(`Focusa native-session pressure: ${pressure.posture}. ${actionCopy}`, "warning");
    }
  }
  if (reason === "session_start" || pressure.posture !== "normal") {
    focusaPost("/telemetry/trace", {
      event_type: "pi_native_session_pressure",
      payload: {
        ...pressure,
        native_session_ref: pressure.native_session_ref === "unavailable" ? "unavailable" : "redacted",
        reason,
      },
    });
  }
}

function markerExistsAtCwd(cwd: string): boolean {
  try {
    return existsSync(join(normalizeProjectRoot(cwd || process.cwd()), ".focusa-project.json"));
  } catch {
    return false;
  }
}

function deferLifecycleAdvisory(ctx: any, key: string, text: string, reason: string): void {
  const sessionId = String(
    ctx?.sessionManager?.getSessionId?.() || getAttachmentRuntime().sessionFrameKey || "no-session"
  );
  getAttachmentRuntime().pendingLifecycleAdvisories[sessionId] = {
    key,
    text,
    reason,
    createdAt: Date.now(),
  };
  persistState();
  if (ctx?.hasUI) {
    ctx.ui.notify(`${text.split("\n")[0]} Details will be attached to your next prompt.`, "warning");
  }
  focusaPost("/telemetry/trace", {
    event_type: "pi_lifecycle_advisory_deferred_to_next_turn",
    payload: {
      session_id: sessionId,
      idempotency_key: key,
      reason,
      outcome: "deferred_to_next_turn",
    },
  });
}

function queueUnboundProjectNag(pi: ExtensionAPI, ctx: any, reason: string): void {
  if (pi.getFlag("--nag-suppress")) return;
  const cwd = normalizeProjectRoot(ctx?.cwd || process.cwd());
  if (markerExistsAtCwd(cwd)) return;
  const key = `pi_unbound_project_nag:${getAttachmentRuntime().sessionFrameKey || "no-session"}:${cwd}`;
  if (getAttachmentRuntime().vitalInfoPrompted[key]) return;
  const prompt = [
    "Focusa project not bound: no .focusa-project.json marker found at this Pi session cwd.",
    `cwd: ${cwd}`,
    "Next steps:",
    "- focusa about        # inspect current Focusa/project binding",
    "- focusa init         # create a local project marker when this is the right project root",
    "- focusa onboard --remote <git-url> --project-root <path>  # bind a remote/VPS checkout marker",
    "Suppress for this session with --nag-suppress when intentionally working unbound.",
  ].join("\n");
  focusaPost("/telemetry/trace", {
    event_type: "pi_unbound_project_nag",
    payload: { reason, cwd, session_id: getAttachmentRuntime().sessionFrameKey, suppressed: false },
  });
  const outcome = queueLifecycleAdvisory(pi, ctx, {
    advisoryKey: key,
    advisoryKind: "unbound_project",
    title: "Focusa project is not bound at this Pi session cwd.",
    content: prompt,
    reason,
    projectRoot: cwd,
    sessionId: getAttachmentRuntime().sessionFrameKey,
  });
  getAttachmentRuntime().vitalInfoPrompted[key] = Date.now();
  persistState();
  focusaPost("/telemetry/trace", {
    event_type: "pi_unbound_project_advisory_outcome",
    payload: {
      reason,
      cwd,
      session_id: getAttachmentRuntime().sessionFrameKey,
      outcome,
      trigger_turn: false,
    },
  });
}

function vitalPromptSurfaceEnabled(surface: string): boolean {
  const raw = String(
    getAttachmentRuntime().cfg?.vitalInfoPromptSurfaces || "project_root,workpoint,trajectory"
  );
  return raw
    .split(",")
    .map((part) => part.trim())
    .includes(surface);
}

type WorkpointDraft = {
  label?: string;
  mission: string;
  next_slice: string;
  action_type?: string;
  target_ref?: string;
  confidence?: "low" | "medium" | "verified";
};

async function ensureLowConfidenceWorkpoint(reason: string, draft?: WorkpointDraft): Promise<void> {
  if (!getAttachmentRuntime().focusaAvailable) return;
  if (!isProjectRootAuthoritySafe(getSessionCwd() || process.cwd())) return;
  const mission =
    draft?.mission ||
    getAttachmentRuntime().currentAsk?.text ||
    getAttachmentRuntime().activeFrameGoal ||
    getAttachmentRuntime().lastFocusSnapshot.intent ||
    getAttachmentRuntime().lastFocusSnapshot.currentFocus;
  const nextSlice =
    draft?.next_slice ||
    getAttachmentRuntime().lastFocusSnapshot.currentFocus ||
    getAttachmentRuntime().activeFrameGoal ||
    getAttachmentRuntime().currentAsk?.text;
  if (!mission && !nextSlice) return;
  await focusaFetch("/workpoint/checkpoint", {
    method: "POST",
    body: JSON.stringify({
      mission: mission || "Pi session resume",
      next_slice: nextSlice || "Resume from low-confidence session state and immediately refine Workpoint.",
      checkpoint_reason: reason === "session_start" ? "session_start" : "session_resume",
      confidence: draft?.confidence || "low",
      canonical: true,
      promote: true,
      continuity_id: ensureContinuityId(getSessionCwd() || process.cwd()),
      session_id: getAttachmentRuntime().sessionFrameKey,
      project_root: getSessionCwd() || process.cwd(),
      source_turn_id: `pi-turn-${getTurnCount()}`,
      action_intent: {
        action_type: draft?.action_type || "resume_workpoint",
        target_ref:
          draft?.target_ref ||
          getAttachmentRuntime().activeFrameId ||
          getAttachmentRuntime().sessionFrameKey ||
          "pi-session",
        verification_hooks: ["low-confidence checkpoint created because no active workpoint existed"],
        status: "needs_refinement",
      },
    }),
  }).catch(() => null);
}

async function refreshSessionWorkpointPacket(reason: string): Promise<void> {
  if (!getAttachmentRuntime().focusaAvailable) return;
  if (!isProjectRootAuthoritySafe(getSessionCwd() || process.cwd())) {
    setActiveWorkpointPacket(null);
    setActiveWorkpointSummary("");
    return;
  }
  try {
    const packet = await focusaFetch("/workpoint/resume", {
      method: "POST",
      body: JSON.stringify({
        mode: "compact_prompt",
        continuity_id: ensureContinuityId(getSessionCwd() || process.cwd()),
        session_id: getAttachmentRuntime().sessionFrameKey,
        project_root: getSessionCwd() || process.cwd(),
      }),
    });
    if (packet?.status === "rejected_scope_mismatch") {
      setActiveWorkpointPacket(null);
      setActiveWorkpointSummary("");
      return;
    }
    if (packet?.status === "completed") {
      const candidate = normalizeWorkpointResumePacketEnvelope(packet);
      if (!isWorkpointPacketScopedToCurrentSession(candidate)) {
        setActiveWorkpointPacket(null);
        setActiveWorkpointSummary("");
        return;
      }
      setActiveWorkpointPacket(stampWorkpointPacketForCurrentPiSession(candidate));
      setActiveWorkpointSummary(
        packet.rendered_summary || packet.resume_packet_v2?.rendered_summary || packet.next_step_hint || ""
      );
      getAttachmentRuntime().lastWorkpointUpdate = Date.now();
      focusaPost("/telemetry/trace", {
        event_type: "workpoint_resume_packet_loaded",
        payload: { reason, workpoint_id: packet.workpoint_id, canonical: packet.canonical },
      });
    }
  } catch {
    /* best effort */
  }
}

async function promptForConfirmedProjectRoot(
  ctx: any,
  proposedRoot: string,
  reason: string
): Promise<string | null> {
  if (!vitalPromptSurfaceEnabled("project_root")) return proposedRoot;
  if (!projectRootConfirmationRequired(proposedRoot)) return proposedRoot;
  const mode = getAttachmentRuntime().cfg?.vitalInfoPromptMode || "prompt";
  const summary = projectRootConfirmationSummary(proposedRoot);
  focusaPost("/telemetry/trace", {
    event_type: "pi_vital_project_root_agent_inference_required",
    payload: {
      reason,
      project_root: proposedRoot,
      summary,
      mode,
      session_id: getAttachmentRuntime().sessionFrameKey,
    },
  });
  return null;
}

function queueProjectIdentityBootstrapTurn(
  pi: ExtensionAPI,
  ctx: any,
  proposedRoot: string,
  reason: string
): void {
  if (!getAttachmentRuntime().focusaAvailable || !vitalPromptSurfaceEnabled("project_root")) return;
  if (reason !== "session_project_mismatch" && !projectRootConfirmationRequired(proposedRoot)) return;
  const key = `project_identity_bootstrap:${getAttachmentRuntime().sessionFrameKey || "no-session"}:${normalizeProjectRoot(ctx?.cwd || proposedRoot || process.cwd())}`;
  if (getAttachmentRuntime().vitalInfoPrompted[key]) return;
  const summary = projectRootConfirmationSummary(proposedRoot);
  const prompt = [
    "Focusa auto-bootstrap: infer the correct project_root for this Pi session now.",
    `Current detected root is unsafe/unconfirmed: ${summary}`,
    "Use Focusa tools and repo evidence first: call focusa_project_identity with the best explicit project_root candidate, then use project_summary/summary_lines to orient.",
    "If cwd is broad like /root, inspect the session/project context and likely repo folders before asking.",
    "If multiple plausible project folders remain after inference, ask the operator directly in chat which project folder to bind.",
    "Do not show modal/select/input UI. Do not perform durable project-aware writes until identity is verified.",
  ].join("\n");
  const outcome = queueLifecycleAdvisory(pi, ctx, {
    advisoryKey: key,
    advisoryKind: "project_identity_bootstrap",
    title: "Focusa needs a verified project root before project-aware writes.",
    content: prompt,
    reason,
    projectRoot: proposedRoot,
    sessionId: getAttachmentRuntime().sessionFrameKey,
  });
  getAttachmentRuntime().vitalInfoPrompted[key] = Date.now();
  persistState();
  focusaPost("/telemetry/trace", {
    event_type: "pi_vital_project_root_advisory_outcome",
    payload: {
      reason,
      project_root: proposedRoot,
      session_id: getAttachmentRuntime().sessionFrameKey,
      outcome,
      trigger_turn: false,
    },
  });
}

type TrajectoryGoalDraft = {
  long_term_goal: string;
  desired_end_state: string;
  short_term_goal?: string;
  current_state?: string;
  goal_source?: string;
};

function projectNameFromRoot(projectRoot: string): string {
  return (
    String(projectRoot || "project")
      .split("/")
      .filter(Boolean)
      .pop() || "project"
  );
}

function cleanTrajectorySeed(value: unknown): string {
  return trimFrameText(
    stripQuotedFocusaContext(
      String(value || "")
        .replace(/\s+/g, " ")
        .trim()
    ),
    220
  );
}

function currentAskForProject(projectRoot: string): string {
  const ask: any = getAttachmentRuntime().currentAsk;
  if (!ask?.text) return "";
  if (ask.sessionId && ask.sessionId !== getAttachmentRuntime().sessionFrameKey) return "";
  if (ask.projectRoot && adoptPiProjectRoot(ask.projectRoot) !== projectRoot) return "";
  if (ask.continuityId && getContinuityId() && ask.continuityId !== getContinuityId()) return "";
  return cleanTrajectorySeed(ask.text);
}

function trajectoryClarityForProject(projectRoot: string): any | null {
  const clarity: any = getLastTrajectoryClarity() || null;
  if (!clarity) return null;
  if (clarity.project_root && adoptPiProjectRoot(clarity.project_root) !== projectRoot) return null;
  if (clarity.continuity_id && getContinuityId() && clarity.continuity_id !== getContinuityId()) return null;
  if (
    clarity.session_id &&
    getAttachmentRuntime().sessionFrameKey &&
    clarity.session_id !== getAttachmentRuntime().sessionFrameKey &&
    clarity.fallback_prior_project_trajectory !== true
  )
    return null;
  return clarity;
}

function scopedWorkpointSeed(projectRoot: string): string {
  const packet: any = isWorkpointPacketScopedToCurrentSession(getActiveWorkpointPacket())
    ? getActiveWorkpointPacket()
    : null;
  if (!packet) return "";
  if (packet.project_root && adoptPiProjectRoot(packet.project_root) !== projectRoot) return "";
  return cleanTrajectorySeed(packet.mission || packet.next_slice);
}

function trajectoryDraftOptions(
  projectRoot: string
): Array<{ label: string; draft: TrajectoryGoalDraft | null }> {
  const projectName = projectNameFromRoot(projectRoot);
  const ask = currentAskForProject(projectRoot);
  const workpointMission = scopedWorkpointSeed(projectRoot);
  const frameGoal = "";
  const focus = "";
  const seed = workpointMission || ask || `Improve ${projectName}`;
  const short = ask || workpointMission || `Continue ${projectName} work`;
  const current = "Current verified state unclear";
  const repoGoal = `Improve and verify ${projectName} as the active project`;
  return [
    {
      // Spec 125 §8.3: candidate from project evidence — requires confirmation.
      label: `A) Candidate from project evidence — requires confirmation`,
      draft: {
        long_term_goal: repoGoal,
        desired_end_state: `${projectName} is verified, onboarding-ready, and has evidence-backed trajectory`,
        short_term_goal: short,
        current_state: current,
        goal_source: "project_evidence_candidate",
      },
    },
    {
      // Spec 125 §8.3: candidate from current ask — requires confirmation.
      label: `B) Candidate from current ask — requires confirmation`,
      draft: {
        long_term_goal: seed,
        desired_end_state: `Completed and verified: ${seed}`,
        short_term_goal: short,
        current_state: current,
        goal_source: "current_ask_candidate",
      },
    },
    {
      // Spec 125 §8.3: candidate from Workpoint gap — cannot define HLT alone.
      label: `C) Candidate from Workpoint gap — cannot define HLT alone`,
      draft: {
        long_term_goal: workpointMission || `Advance ${projectName} workpoint mission`,
        desired_end_state: `Workpoint mission completed with evidence`,
        short_term_goal: short,
        current_state: current,
        goal_source: "workpoint_gap_candidate",
      },
    },
    {
      // Spec 125 §8.3: restore previous valid HLT — preferred when available.
      label: `D) Restore previous valid HLT — preferred when available`,
      draft: {
        long_term_goal: repoGoal,
        desired_end_state: `${projectName} is maintained and improved within verified scope`,
        short_term_goal: short,
        current_state: current,
        goal_source: "previous_valid_fallback",
      },
    },
    {
      // Spec 125 §8.3: custom HLT / desired end state.
      label: `E) Custom HLT / desired end state`,
      draft: null,
    },
    {
      // Spec 125 §8.3: skip — leaves HLT_REQUIRED warning active.
      label: `F) Skip — leaves HLT_REQUIRED warning active`,
      draft: null,
    },
  ];
}

function workpointDraftOptions(projectRoot: string): Array<{ label: string; draft: WorkpointDraft | null }> {
  const projectName = projectNameFromRoot(projectRoot);
  const ask = currentAskForProject(projectRoot);
  const clarity = trajectoryClarityForProject(projectRoot);
  const trajectoryShort = cleanTrajectorySeed(clarity?.short_term_goal || clarity?.active_gap);
  const focus = "";
  const frameGoal = "";
  const mission = ask || trajectoryShort || `Continue ${projectName} work`;
  const next = ask || trajectoryShort || `Identify next useful ${projectName} action`;
  return [
    {
      label: `A) Current task checkpoint — ${mission}`,
      draft: {
        mission,
        next_slice: next,
        action_type: "resume_current_task",
        target_ref:
          getAttachmentRuntime().activeFrameId || getAttachmentRuntime().sessionFrameKey || "pi-session",
        confidence: "low",
      },
    },
    {
      label: `B) Trajectory gap follow-up — ${trajectoryShort || next}`,
      draft: {
        mission: trajectoryShort || mission,
        next_slice: trajectoryShort || next,
        action_type: "trajectory_gap_followup",
        target_ref: "trajectory_gap",
        confidence: "low",
      },
    },
    {
      label: "C) Verify first — collect proof before changing code",
      draft: {
        mission: `Verify ${projectName} current state before acting`,
        next_slice: "Run bounded verification and link evidence before committing to implementation",
        action_type: "verify_current_state",
        target_ref: projectRoot,
        confidence: "low",
      },
    },
    { label: "D) Skip for now", draft: null },
    { label: "E) Custom edit (typing)", draft: null },
  ];
}

function parseWorkpointEditor(text: string): WorkpointDraft | null {
  const raw = String(text || "").trim();
  if (!raw) return null;
  const value = (label: string) => {
    const match = raw.match(new RegExp(`^${label}:\\s*(.+)$`, "im"));
    return String(match?.[1] || "").trim();
  };
  const mission = value("MISSION");
  const next_slice = value("NEXT_SLICE");
  if (!mission || !next_slice) return null;
  return {
    mission,
    next_slice,
    action_type: value("ACTION_TYPE") || "resume_workpoint",
    target_ref: value("TARGET_REF") || undefined,
    confidence: "low",
  };
}

function parseTrajectoryEditor(text: string): TrajectoryGoalDraft | null {
  const raw = String(text || "").trim();
  if (!raw) return null;
  const value = (label: string) => {
    const match = raw.match(new RegExp(`^${label}:\\s*(.+)$`, "im"));
    return String(match?.[1] || "").trim();
  };
  const long_term_goal = value("LONG_TERM_GOAL");
  const desired_end_state = value("DESIRED_END_STATE");
  if (!long_term_goal || !desired_end_state) return null;
  return {
    long_term_goal,
    desired_end_state,
    short_term_goal: value("SHORT_TERM_GOAL") || undefined,
    current_state: value("CURRENT_STATE") || undefined,
  };
}

async function promptForProjectVerifyIfNeeded(
  ctx: any,
  projectRoot: string,
  reason: string
): Promise<boolean> {
  const mode = getAttachmentRuntime().cfg?.vitalInfoPromptMode || "prompt";
  if (
    !vitalPromptSurfaceEnabled("project_verify") ||
    mode === "off" ||
    !isProjectRootAuthoritySafe(projectRoot)
  )
    return true;
  const payload = { cwd: projectRoot, project_root: projectRoot };
  const res = await focusaFetch("/project/verify", { method: "POST", body: JSON.stringify(payload) }).catch(
    () => null
  );
  setLastProjectVerify(res || null);
  persistState();
  if (res?.verification?.verified === true || res?.canonical === true) {
    focusaPost("/telemetry/trace", {
      event_type: "pi_vital_project_verify_passed",
      payload: {
        reason,
        project_root: projectRoot,
        status: res?.project_identity?.status || res?.status || "verified",
      },
    });
    return true;
  }
  const status = String(res?.project_identity?.status || res?.status || "unknown");
  const recovery = String(
    res?.verification?.required_recovery ||
      "verify project identity before durable cross-project state writes"
  );
  ctx.ui.setWidget(
    "focusa-vital",
    ["🧭 Focusa project verify needed", `project_root=${projectRoot}`, `status=${status}; ${recovery}`],
    { placement: "belowEditor" }
  );
  ctx.ui.notify(`Focusa project verify is not clean for ${projectRoot}: ${status}`, "warning");
  focusaPost("/telemetry/trace", {
    event_type: "pi_vital_project_verify_failed",
    payload: { reason, project_root: projectRoot, status, mode },
  });
  // Verification uncertainty must never seize the operator input surface.
  // Conversation and diagnosis continue, while durable project writes remain
  // fail-closed until the agent follows the machine-readable recovery route.
  const advisoryKey = `project_verify_recovery:${getAttachmentRuntime().sessionFrameKey || "no-session"}:${projectRoot}:${status}`;
  const pi = getAttachmentRuntime().pi;
  const outcome = pi
    ? queueLifecycleAdvisory(pi, ctx, {
        advisoryKey,
        advisoryKind: "project_verify_recovery",
        title: "Focusa project verification needs agent recovery",
        content: [
          `project_root=${projectRoot}; status=${status}`,
          "Conversation and read-only diagnosis continue without an operator modal.",
          "Durable project-aware writes remain blocked until verification is canonical.",
          "Agent route: focusa_project_identity -> focusa_project_verify -> focusa_workpoint_checkpoint/resume.",
          `Recovery detail: ${recovery}`,
        ].join("\n"),
        reason,
        projectRoot,
        sessionId: getAttachmentRuntime().sessionFrameKey,
      })
    : "pi_runtime_unavailable";
  focusaPost("/telemetry/trace", {
    event_type: "pi_vital_project_verify_recovery_queued",
    payload: { reason, project_root: projectRoot, status, mode, outcome, advisory_key: advisoryKey },
  });
  return false;
}

async function promptForWorkpointIfNeeded(ctx: any, projectRoot: string, reason: string): Promise<boolean> {
  const mode = getAttachmentRuntime().cfg?.vitalInfoPromptMode || "prompt";
  if (
    !vitalPromptSurfaceEnabled("workpoint") ||
    mode !== "prompt" ||
    !isProjectRootAuthoritySafe(projectRoot) ||
    getActiveWorkpointPacket()
  )
    return false;
  const mission =
    getAttachmentRuntime().currentAsk?.text ||
    getAttachmentRuntime().activeFrameGoal ||
    getAttachmentRuntime().lastFocusSnapshot.intent ||
    getAttachmentRuntime().lastFocusSnapshot.currentFocus;
  const nextSlice =
    getAttachmentRuntime().lastFocusSnapshot.currentFocus ||
    getAttachmentRuntime().activeFrameGoal ||
    getAttachmentRuntime().currentAsk?.text;
  if (!mission && !nextSlice) return false;
  const key = `workpoint:${projectRoot}:${reason}`;
  if (getAttachmentRuntime().vitalInfoPrompted[key]) return false;
  getAttachmentRuntime().vitalInfoPrompted[key] = Date.now();
  persistState();
  if (typeof ctx.ui?.select !== "function") {
    ctx.ui?.notify?.("Focusa Workpoint prompt skipped: Pi UI select is unavailable.", "warning");
    focusaPost("/telemetry/trace", {
      event_type: "pi_vital_workpoint_prompt_unavailable",
      payload: { reason, project_root: projectRoot, missing_ui: "select" },
    });
    return false;
  }
  const options = workpointDraftOptions(projectRoot);
  const choice = await ctx.ui.select(
    "Focusa Workpoint is missing — choose a checkpoint draft",
    options.map((option) => option.label)
  );
  if (!choice || String(choice).startsWith("D)")) return false;
  let draft = options.find((option) => option.label === choice)?.draft || null;
  if (String(choice).startsWith("E)")) {
    const seed = options[0]?.draft;
    const template = [
      `MISSION: ${seed?.mission || ""}`,
      `NEXT_SLICE: ${seed?.next_slice || ""}`,
      `ACTION_TYPE: ${seed?.action_type || "resume_workpoint"}`,
      `TARGET_REF: ${seed?.target_ref || "pi-session"}`,
    ].join("\n");
    const edited = await ctx.ui.editor("Define Focusa Workpoint", template);
    draft = parseWorkpointEditor(String(edited || ""));
    if (!draft) {
      ctx.ui.notify("Workpoint not saved: MISSION and NEXT_SLICE are required.", "warning");
      return false;
    }
  }
  if (!draft) return false;
  await ensureLowConfidenceWorkpoint(reason, draft);
  await refreshSessionWorkpointPacket(`${reason}_operator_selected_workpoint`);
  return Boolean(getActiveWorkpointPacket());
}

async function promptForTrajectoryIfNeeded(ctx: any, projectRoot: string, reason: string): Promise<void> {
  const mode = getAttachmentRuntime().cfg?.vitalInfoPromptMode || "prompt";
  if (
    !vitalPromptSurfaceEnabled("trajectory") ||
    mode !== "prompt" ||
    !isProjectRootAuthoritySafe(projectRoot)
  )
    return;
  const clarity: any = trajectoryClarityForProject(projectRoot) || {};
  const priorProjectFallbackLoaded =
    clarity.fallback_prior_project_trajectory === true &&
    Boolean(clarity.long_term_goal || clarity.desired_end_state || clarity.trajectory_id);
  if (priorProjectFallbackLoaded) {
    focusaPost("/telemetry/trace", {
      event_type: "pi_trajectory_prompt_suppressed_prior_project_fallback",
      payload: {
        reason,
        project_root: projectRoot,
        continuity_id: getContinuityId() || null,
        session_id: getAttachmentRuntime().sessionFrameKey || null,
        trajectory_id: clarity.trajectory_id || null,
        fallback_source_continuity_id: clarity.fallback_source_continuity_id || null,
      },
    });
    return;
  }
  const status = String(clarity.status || "unknown");
  const action = String(clarity.recommended_action || "unknown");
  const unclear =
    ["unknown", "unclear", "not_found", "not_set", "missing"].includes(status) ||
    /define_goal|operator_required/.test(action);
  const key = `trajectory:${projectRoot}:${getContinuityId() || "no-continuity"}:${getAttachmentRuntime().sessionFrameKey || "no-session"}:${status}:${action}`;
  if (!unclear || getAttachmentRuntime().vitalInfoPrompted[key]) return;
  getAttachmentRuntime().vitalInfoPrompted[key] = Date.now();
  persistState();
  const options = trajectoryDraftOptions(projectRoot);
  const choice = await ctx.ui.select(
    "Focusa trajectory is not set — choose a draft",
    options.map((option) => option.label)
  );
  if (!choice || String(choice).startsWith("D)")) return;
  let parsed = options.find((option) => option.label === choice)?.draft || null;
  if (String(choice).startsWith("E)")) {
    const seed = options[0]?.draft;
    const template = [
      `LONG_TERM_GOAL: ${seed?.long_term_goal || ""}`,
      `DESIRED_END_STATE: ${seed?.desired_end_state || ""}`,
      `SHORT_TERM_GOAL: ${seed?.short_term_goal || ""}`,
      `CURRENT_STATE: ${seed?.current_state || ""}`,
    ].join("\n");
    const edited = await ctx.ui.editor("Define Focusa trajectory", template);
    parsed = parseTrajectoryEditor(String(edited || ""));
    if (!parsed) {
      ctx.ui.notify("Trajectory not saved: LONG_TERM_GOAL and DESIRED_END_STATE are required.", "warning");
      return;
    }
  }
  if (!parsed) return;
  const body = {
    ...parsed,
    project_root: projectRoot,
    continuity_id: ensureContinuityId(projectRoot),
    session_id: getAttachmentRuntime().sessionFrameKey,
    goal_source: parsed.goal_source || "operator_selected_inference",
    operator_confirmed: true,
    session_identity: await buildFocusaSessionIdentity(projectRoot, "manual", {
      continuityId: ensureContinuityId(projectRoot),
      sessionId: getAttachmentRuntime().sessionFrameKey,
    }),
  };
  const res = await focusaFetch("/trajectory/define-goal", {
    method: "POST",
    body: JSON.stringify(body),
  }).catch(() => null);
  if (res?.canonical === true || res?.persisted === true) {
    ctx.ui.notify("Focusa trajectory defined for this project.", "info");
    await refreshTrajectoryClarityLifecycle(`${reason}_trajectory_defined`, projectRoot);
    persistState();
  } else {
    ctx.ui.notify(
      `Trajectory define_goal did not persist: ${res?.failure_class || res?.status || "unknown"}`,
      "warning"
    );
  }
}

function seedCurrentAskFromPersistedState(ctx: any, data: any) {
  const restoredAsk = data?.currentAsk;
  const cleanedRestoredAsk = stripQuotedFocusaContext(restoredAsk?.text || "");
  const cwd = adoptPiProjectRoot(ctx?.cwd || getSessionCwd() || process.cwd());
  if (cleanedRestoredAsk && !isNonTaskStatusLikeText(cleanedRestoredAsk)) {
    if (restoredAsk.sessionId && restoredAsk.sessionId !== getAttachmentRuntime().sessionFrameKey) return;
    if (restoredAsk.projectRoot && adoptPiProjectRoot(restoredAsk.projectRoot) !== cwd) return;
    getAttachmentRuntime().currentAsk = {
      text: trimFrameText(cleanedRestoredAsk, 500),
      kind: restoredAsk.kind || classifyCurrentAsk(cleanedRestoredAsk),
      sourceTurnId: restoredAsk.sourceTurnId || "restored",
      updatedAt: restoredAsk.updatedAt || Date.now(),
      sessionId: restoredAsk.sessionId || getAttachmentRuntime().sessionFrameKey,
      projectRoot: restoredAsk.projectRoot || cwd,
      continuityId: restoredAsk.continuityId || getContinuityId(),
    };
    if (data?.queryScope) getAttachmentRuntime().queryScope = data.queryScope;
    return;
  }

  const goal = stripQuotedFocusaContext(String(data?.frameGoal || "").trim());
  const title = String(data?.frameTitle || "").trim();
  if (!goal || isNonTaskStatusLikeText(goal) || isGenericPiFrameForCwd(cwd, title, goal)) return;
  if (!/^Pi (Task|Question|Correction): /.test(title)) return;

  getAttachmentRuntime().currentAsk = {
    text: trimFrameText(goal, 500),
    kind: classifyCurrentAsk(goal),
    sourceTurnId: "restored-frame-goal",
    updatedAt: Date.now(),
    sessionId: getAttachmentRuntime().sessionFrameKey,
    projectRoot: cwd,
    continuityId: getContinuityId(),
  };
}

async function ensureActiveFrame(ctx: any, sessionId?: string) {
  return ensurePiFrame(adoptPiProjectRoot(ctx.cwd), sessionId, "pi-auto");
}

async function ensureFocusaSession(ctx: any) {
  const status = await focusaFetch("/status").catch(() => null);
  if (status?.session?.status === "active") return status.session;
  const cwd = adoptPiProjectRoot(ctx.cwd || getSessionCwd() || "pi-workspace");
  return focusaFetch("/session/start", {
    method: "POST",
    body: JSON.stringify({
      adapter_id: "pi",
      workspace_id: cwd,
      project_root: cwd,
      continuity_id: ensureContinuityId(cwd),
    }),
  });
}

function connectSSE() {
  if (sseReconnectTimer) {
    clearTimeout(sseReconnectTimer);
    sseReconnectTimer = null;
  }
  if (sseAbort) sseAbort.abort();
  if (!getAttachmentRuntime().focusaAvailable) return;

  const base = getAttachmentRuntime().cfg?.focusaApiBaseUrl || "http://127.0.0.1:8787/v1";
  const controller = new AbortController();
  sseAbort = controller;

  fetch(`${base}/events/stream`, { signal: controller.signal })
    .then(async (res) => {
      if (!res.body) return;
      const reader = res.body.getReader();
      const decoder = new TextDecoder();
      let buffer = "";
      while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split("\n");
        buffer = lines.pop() || "";
        for (const line of lines) {
          if (!line.startsWith("data: ")) continue;
          try {
            const evt = JSON.parse(line.slice(6));
            handleSSEEvent(evt);
          } catch {
            /* malformed SSE */
          }
        }
      }
    })
    .catch(() => {
      if (controller.signal.aborted || !getAttachmentRuntime().focusaAvailable) return;
      // §30: "If background work fails, the extension shows nothing (fail silent)"
      // Reconnect with backoff — use same exponential backoff as health checks (§11)
      sseReconnectTimer = setTimeout(() => {
        sseReconnectTimer = null;
        if (getAttachmentRuntime().focusaAvailable) connectSSE();
      }, getAttachmentRuntime().healthBackoffMs);
    });
}

// §30: Metacognitive awareness indicators + §37.10: Cross-surface events
function handleSSEEvent(evt: any) {
  // #45: the daemon SSE envelope carries `event_type` (focusa.stream_event.v1).
  switch (evt.event_type || evt.type) {
    case "worker_started":
      getAttachmentRuntime().lastMetacogEvent = "thinking...";
      break;
    case "extraction_complete":
      getAttachmentRuntime().lastMetacogEvent = `extracted ${evt.count || "N"} items`;
      setTimeout(() => {
        getAttachmentRuntime().lastMetacogEvent = "";
      }, 5000);
      break;
    case "thesis_updated":
      getAttachmentRuntime().lastMetacogEvent = "thesis updated";
      setTimeout(() => {
        getAttachmentRuntime().lastMetacogEvent = "";
      }, 5000);
      break;
    case "quality_flag":
      getAttachmentRuntime().lastMetacogEvent = `⚠️ ${evt.message || "quality issue"}`;
      break;
    case "focus_state_updated":
      // §37.10: Cross-surface decision notification
      if (evt.source && evt.source !== "pi" && evt.decisions?.length) {
        getAttachmentRuntime()
          .pi?.exec("echo", [], { timeout: 1 })
          .catch(() => {}); // no-op to access ctx
      }
      break;
    case "silent_session_completed":
      // #311: background terminal-blocking queries report their completion
      // back into this terminal instead of the agent polling for them.
      try {
        const session = String(evt?.session_id || "").slice(0, 8);
        const status = String(evt?.status || "completed");
        const kind = status === "failed" || status === "cancelled" ? "warning" : "info";
        const summary = String(evt?.summary || "").slice(0, 120);
        getAttachmentRuntime().uiCtx?.notify(
          `Silent session ${session} ${status}${summary ? `: ${summary}` : ""}`,
          kind
        );
      } catch {
        /* §30: fail silent; notification must never crash Pi */
      }
      break;
    case "trajectory_goal_defined":
      // §93: high-priority agent alert on HLT continuity change
      if (evt.data?.trajectory) {
        const t = evt.data.trajectory;
        const hlt = t.long_term_goal || "unknown";
        const display = hlt.length > 80 ? hlt.substring(0, 77) + "…" : hlt;
        getAttachmentRuntime().uiCtx?.notify(`[HLT CHANGED] ${display}`, "error");
        if (t.mid_level_goal) {
          getAttachmentRuntime().lastMetacogEvent = `[MLG] ${t.mid_level_goal}`;
          setTimeout(() => {
            getAttachmentRuntime().lastMetacogEvent = "";
          }, 15000);
        }
      }
      break;
    default:
      break;
  }
}

async function ensureProjectGenesis(projectRoot: string, reason: string): Promise<boolean> {
  const continuityId = ensureContinuityId(projectRoot);
  const idempotencyKey = `genesis:${continuityId}:project-bootstrap`;
  const status = await focusaFetch(
    `/project/genesis/status?project_root=${encodeURIComponent(projectRoot)}`
  ).catch(() => null);
  if (status?.status === "ready") return true;

  const payload = {
    project_root: projectRoot,
    continuity_id: continuityId,
    idempotency_key: idempotencyKey,
    allow_task_decomposition: true,
  };
  const staged = await focusaFetch("/project/genesis/start", {
    method: "POST",
    body: JSON.stringify(payload),
  }).catch(() => null);
  if (staged?.status === "ready") return true;
  if (staged?.status !== "staged") {
    focusaPost("/telemetry/trace", {
      event_type: "pi_project_genesis_incomplete",
      payload: {
        reason,
        project_root: projectRoot,
        continuity_id: continuityId,
        status: staged?.status || "unavailable",
        missing_links: staged?.missing_links || [],
        next_action: staged?.next_action,
      },
    });
    return false;
  }

  const committed = await focusaFetch("/project/genesis/commit", {
    method: "POST",
    body: JSON.stringify({ ...payload, confirm: true }),
  }).catch(() => null);
  const ready = committed?.status === "ready";
  focusaPost("/telemetry/trace", {
    event_type: ready ? "pi_project_genesis_ready" : "pi_project_genesis_commit_blocked",
    payload: {
      reason,
      project_root: projectRoot,
      continuity_id: continuityId,
      status: committed?.status || "unavailable",
      next_action: committed?.next_action,
    },
  });
  return ready;
}

export function registerSession(pi: ExtensionAPI) {
  // ── session_start — single merged handler ──────────────────────────────────
  pi.on("session_start", async (event, ctx) => {
    const lifecycleGeneration = healthLifecycle.begin();
    const healthLifecycleIsCurrent = () => healthLifecycle.isCurrent(lifecycleGeneration);
    getAttachmentRuntime().pi = pi;
    getAttachmentRuntime().uiCtx = ctx.ui; // §93: SSE handler needs ctx.ui for high-priority agent alerts
    getAttachmentRuntime().sessionStartTime = Date.now();
    // Pi 0.81 SessionStartEvent carries a reason, not a synthetic sessionId.
    // sessionManager.getSessionId() is the stable temporal identity boundary.
    const eventSessionId = String(ctx.sessionManager.getSessionId());
    const sessionStartReason = event.reason;
    getAttachmentRuntime().sessionFrameKey = eventSessionId;
    getAttachmentRuntime().sessionCwd = adoptPiProjectRoot(ctx.cwd);
    resetPiSessionScopedState("session_start");
    syncRuntimeFieldsToScopeStore();

    // §37.5: Check CLI flags FIRST
    if (pi.getFlag("--no-focusa")) {
      getAttachmentRuntime().focusaAvailable = false;
      ctx.ui.setStatus("focusa", "⏸️ Focusa disabled");
      return;
    }
    if (pi.getFlag("--wbm")) getAttachmentRuntime().wbmEnabled = true;

    // Health check
    await checkFocusa();

    // §36.4 + §33.5: Restore decisions from Pi session entries.
    // CRITICAL §33.5: Never restore activeFrameId from previous sessions — that
    // points to Wirebot/TEP frames and pollutes Pi sessions with stale Wirebot
    // state. Pi ALWAYS gets its own FRESH frame. Only WBM mode may reuse frames.
    const entries = ctx.sessionManager.getEntries();
    refreshNativeSessionPressure(ctx, "session_start", entries);
    let persistedBindingRoot = "";
    for (let i = entries.length - 1; i >= 0; i--) {
      const entry = entries[i];
      if (
        entry.type === "custom" &&
        (entry.customType === "focusa-wbm-state" || entry.customType === "focusa-state") &&
        entry.data
      ) {
        const recovered = loadPersistedRecoveryState(entry.data);
        persistedBindingRoot = persistedProjectRootFromState(recovered);
        if (persistedBindingRoot) break;
      }
    }

    const localBinding = resolvePiProjectRootCandidate(ctx.cwd);
    const bindingQuery = new URLSearchParams({ cwd: String(ctx.cwd || process.cwd()) });
    if (persistedBindingRoot) bindingQuery.set("persisted_project_root", persistedBindingRoot);
    const bindingPayload = await focusaFetch(`/project/identity?${bindingQuery.toString()}`, {
      method: "GET",
    }).catch(() => null);
    const bindingDecision = bindingPayload?.binding_decision || null;
    const bindingCandidates = Array.isArray(bindingPayload?.binding_candidates)
      ? bindingPayload.binding_candidates
      : localBinding.candidates || [];
    const bindingAmbiguous =
      bindingDecision?.ambiguous === true || bindingPayload?.status === "ambiguous_project_binding";
    const selectedBindingRoot = normalizeProjectRoot(
      bindingDecision?.selected_project_root || localBinding.projectRoot
    );
    const selectedBindingCandidate = bindingCandidates.find(
      (candidate: any) => normalizeProjectRoot(candidate?.project_root) === selectedBindingRoot
    );
    const persistedBindingCandidate = bindingCandidates.find(
      (candidate: any) =>
        normalizeProjectRoot(candidate?.project_root) === normalizeProjectRoot(persistedBindingRoot)
    );
    const sameCanonicalProject =
      !!selectedBindingCandidate?.canonical_parent_root &&
      normalizeProjectRoot(selectedBindingCandidate.canonical_parent_root) ===
        normalizeProjectRoot(persistedBindingCandidate?.canonical_parent_root);
    if (selectedBindingRoot) {
      const score = Number(selectedBindingCandidate?.score || localBinding.confidenceScore * 1000 || 0);
      setLastProjectRootResolution({
        projectRoot: selectedBindingRoot,
        confidence: score >= 900 ? "high" : score >= 700 ? "medium" : "low",
        confidenceScore: Math.min(1, score / 1000),
        source: "core_api_binding_candidates",
        reason: String(bindingDecision?.reason || localBinding.reason),
        safe: isProjectRootAuthoritySafe(selectedBindingRoot),
        requiresOperatorConfirmation: bindingAmbiguous || bindingDecision?.requires_confirmation === true,
        markers: Array.isArray(selectedBindingCandidate?.markers)
          ? selectedBindingCandidate.markers.map(String)
          : localBinding.markers,
        candidates: bindingCandidates.map((candidate: any) => ({
          projectRoot: normalizeProjectRoot(candidate?.project_root),
          confidenceScore: Number(candidate?.score || 0) / 1000,
          markers: Array.isArray(candidate?.markers) ? candidate.markers.map(String) : [],
          source: Array.isArray(candidate?.sources)
            ? candidate.sources.map(String).join("+")
            : "binding_candidate",
        })),
      });
    }

    if (!bindingAmbiguous && selectedBindingRoot && isProjectRootAuthoritySafe(selectedBindingRoot)) {
      getAttachmentRuntime().sessionCwd = selectedBindingRoot;
    }
    let persistedStateFound = false;
    let persistedProjectRoot = "";
    let projectMismatchDetected = bindingAmbiguous;
    for (let i = entries.length - 1; i >= 0; i--) {
      const e = entries[i];
      if (
        e.type === "custom" &&
        (e.customType === "focusa-wbm-state" || e.customType === "focusa-state") &&
        e.data
      ) {
        const d = loadPersistedRecoveryState(e.data);
        if (!d) continue;
        const candidateProjectRoot = persistedProjectRootFromState(d);
        const currentProjectRoot = selectedBindingRoot || normalizeProjectRoot(localBinding.projectRoot);
        const exactRootMatch =
          normalizeProjectRoot(candidateProjectRoot) === normalizeProjectRoot(currentProjectRoot);
        if (candidateProjectRoot && currentProjectRoot && !exactRootMatch && !sameCanonicalProject) {
          projectMismatchDetected = true;
          continue;
        }
        persistedStateFound = !bindingAmbiguous;
        persistedProjectRoot = candidateProjectRoot || currentProjectRoot;
        if (bindingAmbiguous) continue;
        // §33.5 + §33.7: restore resumable session metadata and safe local shadow,
        // but do not blindly reuse stale frame identity outside WBM mode.
        getAttachmentRuntime().localDecisions = d.decisions || [];
        setTurnCount(d.turnCount || 0);
        getAttachmentRuntime().wbmEnabled = d.wbmEnabled || getAttachmentRuntime().wbmEnabled;
        getAttachmentRuntime().wbmNoCatalogue = d.wbmNoCatalogue || false;
        getAttachmentRuntime().cataloguedDecisions = d.cataloguedDecisions || [];
        getAttachmentRuntime().cataloguedFacts = d.cataloguedFacts || [];
        setTotalCompactions(d.totalCompactions || 0);
        getAttachmentRuntime().lastCompactResumeKey = d.lastCompactResumeKey || "";
        getAttachmentRuntime().lastCompactResumeAt = d.lastCompactResumeAt || 0;
        getAttachmentRuntime().activeFrameTitle = d.frameTitle || "";
        getAttachmentRuntime().activeFrameGoal = d.frameGoal || "";
        seedCurrentAskFromPersistedState(ctx, d);
        getAttachmentRuntime().lastFocusSnapshot = {
          decisions: d.authoritativeDecisions || [],
          constraints: d.authoritativeConstraints || [],
          failures: d.authoritativeFailures || [],
          intent: d.intent || "",
          currentFocus: d.currentFocus || "",
        };
        if (d.projectRootResolution) setLastProjectRootResolution(d.projectRootResolution);
        if (d.lastProjectIdentity) {
          const pi = d.lastProjectIdentity;
          const piRoot = pi.project_root ? normalizeProjectRoot(pi.project_root) : "";
          const cwdRoot = normalizeProjectRoot(ctx.cwd);
          setLastProjectIdentity(piRoot && piRoot === cwdRoot ? pi : null);
        }
        if (d.lastTrajectoryClarity) {
          const c = d.lastTrajectoryClarity;
          const cRoot = c.project_root ? adoptPiProjectRoot(c.project_root) : "";
          const cwdRoot = adoptPiProjectRoot(ctx.cwd);
          setLastTrajectoryClarity(
            (!cRoot || cRoot === cwdRoot) &&
              (!c.session_id ||
                c.session_id === eventSessionId ||
                c.fallback_prior_project_trajectory === true)
              ? c
              : null
          );
        }
        if (d.lastProjectVerify) setLastProjectVerify(d.lastProjectVerify);
        if (d.latestReportSummary?.handle) setLatestReportSummary(d.latestReportSummary);
        if (d.toolOutputPressure?.recapRequired)
          getAttachmentRuntime().toolOutputPressure = d.toolOutputPressure;
        if (Array.isArray(d.projectSwitchLedger))
          getAttachmentRuntime().projectSwitchLedger = d.projectSwitchLedger.slice(0, 12);
        if (d.vitalInfoPrompted) getAttachmentRuntime().vitalInfoPrompted = d.vitalInfoPrompted;
        if (d.pendingLifecycleAdvisories)
          getAttachmentRuntime().pendingLifecycleAdvisories = d.pendingLifecycleAdvisories;
        if (d.sessionProjectClassification)
          getAttachmentRuntime().sessionProjectClassification = d.sessionProjectClassification;
        if (d.piSessionProjectRegistry)
          getAttachmentRuntime().piSessionProjectRegistry = d.piSessionProjectRegistry;
        adoptPersistedContinuityForSession(
          d,
          eventSessionId,
          selectedBindingRoot || localBinding.projectRoot
        );
        // Explicitly clear stale pollution — do NOT carry across sessions
        getAttachmentRuntime().localConstraints = [];
        getAttachmentRuntime().localFailures = [];
        break;
      }
    }

    const sessionProjectClassification = projectMismatchDetected
      ? "session_project_mismatch"
      : classifyPiSessionProject({
          reason: sessionStartReason,
          currentProjectRoot: selectedBindingRoot || normalizeProjectRoot(localBinding.projectRoot),
          markerExists:
            (Array.isArray(selectedBindingCandidate?.markers) &&
              selectedBindingCandidate.markers.length > 0) ||
            markerExistsAtCwd(selectedBindingRoot || localBinding.projectRoot),
          persistedStateFound,
          persistedProjectRoot,
          bindingAmbiguous,
          sameCanonicalProject,
          bindingCandidateRoots: (localBinding.candidates || []).map((candidate) => candidate.projectRoot),
          explicitContinuationMetadata: sessionStartReason === "fork",
        });
    getAttachmentRuntime().sessionProjectClassification = sessionProjectClassification;
    if (sessionProjectClassification === "new_session_new_project") {
      queueUnboundProjectNag(pi, ctx, "new_session_new_project");
    }
    const classifiedRoot = selectedBindingRoot || normalizeProjectRoot(localBinding.projectRoot);
    getAttachmentRuntime().piSessionProjectRegistry[eventSessionId] = {
      project_root: classifiedRoot,
      continuity_id: ensureContinuityId(classifiedRoot),
      latest_workpoint_id: getActiveWorkpointPacket()?.workpoint_id,
      classification: sessionProjectClassification,
      last_seen_at: Date.now(),
      provenance: persistedStateFound ? "native_focusa_anchor" : "pi_session_start_plus_project_evidence",
    };
    if (sessionProjectClassification === "resumed_session_recoverable_project") {
      // Missing marker on a verified same-root resume is repaired in runtime
      // without blocking the operator with onboarding/continue UI.
      confirmPiProjectRoot(classifiedRoot);
    }
    focusaPost("/telemetry/trace", {
      event_type: "pi_session_project_classified",
      payload: {
        session_id: eventSessionId,
        reason: sessionStartReason,
        classification: sessionProjectClassification,
        project_root: classifiedRoot,
        persisted_state_found: persistedStateFound,
        marker_exists: markerExistsAtCwd(ctx.cwd),
      },
    });
    if (sessionProjectClassification !== "session_project_mismatch") {
      persistState();
    }

    // §33.5: Always NULL out activeFrameId — force-push fresh Pi frame.
    // This prevents Wirebot/TEP frame state from leaking into Pi sessions.
    // WBM mode may override this via --wbm flag above.
    if (!getAttachmentRuntime().wbmEnabled) getAttachmentRuntime().activeFrameId = null;
    syncRuntimeFieldsToScopeStore();

    if (!getAttachmentRuntime().focusaAvailable) {
      ctx.ui.setStatus("focusa", "📡 Focusa offline");
      return;
    }

    const detectedProjectRoot = adoptPiProjectRoot(ctx.cwd);
    if (sessionProjectClassification === "session_project_mismatch") {
      queueProjectIdentityBootstrapTurn(pi, ctx, detectedProjectRoot, "session_project_mismatch");
      focusaPost("/telemetry/trace", {
        event_type: "pi_session_project_mismatch_blocked",
        payload: { session_id: eventSessionId, project_root: detectedProjectRoot },
      });
      return;
    }
    const projectRoot = await promptForConfirmedProjectRoot(ctx, detectedProjectRoot, "session_start");
    if (!projectRoot) {
      focusaPost("/telemetry/trace", {
        event_type: "pi_session_state_bind_blocked_unconfirmed_project_root",
        payload: {
          project_root: detectedProjectRoot,
          summary: projectRootConfirmationSummary(detectedProjectRoot),
          session_id: eventSessionId,
          prompt_mode: getAttachmentRuntime().cfg?.vitalInfoPromptMode || "prompt",
        },
      });
      queueProjectIdentityBootstrapTurn(pi, ctx, detectedProjectRoot, "session_start");
      return;
    }
    ensureContinuityId(projectRoot);
    await promptForProjectVerifyIfNeeded(ctx, projectRoot, "session_start");
    await ensureFocusaSession({ ...ctx, cwd: projectRoot });
    await ensureActiveFrame(
      { ...ctx, cwd: projectRoot },
      (event as any).sessionId || `pi-session-${Date.now()}`
    );
    await refreshSessionWorkpointPacket("session_start");
    await refreshTrajectoryClarityLifecycle("session_start", projectRoot);
    await promptForTrajectoryIfNeeded(ctx, projectRoot, "session_start");
    if (!getActiveWorkpointPacket()) {
      await refreshTrajectoryClarityLifecycle("session_start_genesis", projectRoot);
      const genesisReady = await ensureProjectGenesis(projectRoot, "session_start");
      if (genesisReady) {
        await refreshSessionWorkpointPacket("session_start_genesis_ready");
        await refreshTrajectoryClarityLifecycle("session_start_genesis_ready", projectRoot);
      } else {
        ctx.ui.notify(
          "Project preparation is incomplete; Focusa preserved the Genesis packet and next action without creating a placeholder Workpoint.",
          "warning"
        );
      }
    }

    // §35.8: Pi owns the session display name (/name, session selector).
    // Focusa may cache its scoped frame title for context/status, but must not call the Pi session naming API.
    const data = await getFocusState().catch(() => null);
    if (data?.frame?.title) {
      getAttachmentRuntime().activeFrameTitle = data.frame.title;
      getAttachmentRuntime().activeFrameGoal = data.frame.goal || getAttachmentRuntime().activeFrameGoal;
    }

    // §37.9: Context Core activity signal + wb me --set pi_active
    focusaPost("/telemetry/activity", { surface: "pi", event: "session_start", cwd: ctx.cwd });
    pi.exec("wb", ["me", "--set", "pi_active=true"]).catch(() => {});

    // §30 + §37.10: Start SSE connection for metacognitive + cross-surface events
    connectSSE();

    // Keep Pi footer task label fresh between explicit commands.
    // Default is event-driven (no periodic polling); polling can be enabled explicitly.
    if (getAttachmentRuntime().footerSyncInterval) clearInterval(getAttachmentRuntime().footerSyncInterval);
    getAttachmentRuntime().footerSyncInterval = null;
    const bridgeSyncMode = getAttachmentRuntime().cfg?.bridgeSyncMode || "event-driven";
    if (bridgeSyncMode === "polling") {
      const footerRefreshMs = Math.max(5_000, getAttachmentRuntime().cfg?.bridgePollMs || 15_000);
      let footerSyncInFlight = false;
      getAttachmentRuntime().footerSyncInterval = setInterval(async () => {
        if (!getAttachmentRuntime().focusaAvailable || footerSyncInFlight) return;
        footerSyncInFlight = true;
        try {
          await getFocusState().catch(() => null);
        } finally {
          footerSyncInFlight = false;
        }
      }, footerRefreshMs);
    }

    // Flapping remains in holdover until a bounded recovery probation passes.
    const daemonRecovery = new DaemonRecoveryGate();

    // §38.3 + §11: Health check with exponential backoff via recursive setTimeout
    function scheduleHealthCheck() {
      if (!healthLifecycleIsCurrent()) return;
      if (getAttachmentRuntime().healthInterval) clearTimeout(getAttachmentRuntime().healthInterval);
      getAttachmentRuntime().healthInterval = setTimeout(() => {
        if (!healthLifecycleIsCurrent()) return;
        void (async () => {
          refreshNativeSessionPressure(ctx, "health_tick");
          await checkFocusa();
          if (!healthLifecycleIsCurrent()) return;
          const recovery = daemonRecovery.observe(
            getAttachmentRuntime().focusaAvailable,
            getAttachmentRuntime().healthFailCount
          );

          if (recovery.enteredOutage) {
            // Confirmed outage (not single blip) — preserve tool availability, enter holdover, and kickstart daemon.
            ctx.ui.setStatus("focusa", "🛟 Focusa holdover · restarting");
            if (recovery.notifyOutage) {
              ctx.ui.notify(
                `Focusa daemon unavailable (${getAttachmentRuntime().healthFailCount} checks) — holdover active; bounded daemon recovery started without restarting session`,
                "warning"
              );
            }
            if (sseAbort) {
              sseAbort.abort();
              sseAbort = null;
            }
            if (recovery.kickstart) await kickstartFocusaDaemon("session_health_check");
          } else if (!getAttachmentRuntime().focusaAvailable && recovery.outage) {
            ctx.ui.setStatus("focusa", "🛟 Focusa holdover · retrying");
            if (recovery.notifyOutage) {
              ctx.ui.notify(
                `Focusa daemon remains unavailable — holdover preserved; the next recovery attempt is cooldown-bounded`,
                "warning"
              );
            }
            if (recovery.kickstart) await kickstartFocusaDaemon("session_health_retry");
          } else if (getAttachmentRuntime().focusaAvailable && recovery.outage) {
            ctx.ui.setStatus(
              "focusa",
              `🛟 Focusa holdover · verifying recovery ${recovery.recoveryHealthyChecks}/3`
            );
          } else if (recovery.stableRecovered) {
            // Came back stably — reconnect SSE and reconcile holdover state; tools were never disabled.
            ctx.ui.setStatus("focusa", getAttachmentRuntime().wbmEnabled ? "🤖 Focusa WBM" : "🧭 Focusa");
            ctx.ui.notify(
              "Focusa daemon stably reconnected — holdover reconciled; session preserved",
              "info"
            );
            await ensureFocusaSession(ctx);
            await ensureActiveFrame(ctx);
            connectSSE();

            // §11/§25.7: Soft resync — reconcile local shadow with Focusa on reconnect
            if (getAttachmentRuntime().activeFrameId) {
              // Push any local shadow accumulated during outage
              if (
                getAttachmentRuntime().localDecisions.length ||
                getAttachmentRuntime().localConstraints.length ||
                getAttachmentRuntime().localFailures.length
              ) {
                await pushDelta({
                  decisions: getAttachmentRuntime().localDecisions.slice(-10),
                  constraints: getAttachmentRuntime().localConstraints.slice(-10),
                  failures: getAttachmentRuntime().localFailures.slice(-5),
                  notes: ["Reconciled after Focusa outage"],
                }).catch(() => null);
              }
              // Fetch fresh state + recent candidates
              const data = await getFocusState();
              if (data?.fs) {
                ctx.ui.notify(
                  `Resync complete — ${data.fs.decisions?.length || 0} decisions, ${data.fs.constraints?.length || 0} constraints`,
                  "info"
                );
              }
              // Fetch recent Focus Gate candidates
              focusaFetch("/focus-gate/candidates?limit=5")
                .then((r: any) => {
                  if (r?.candidates?.length) {
                    ctx.ui.notify(`Focus Gate: ${r.candidates.length} pending candidates`, "info");
                  }
                })
                .catch(() => {});
            }
          }

          // Schedule next check with (possibly updated) backoff interval
          scheduleHealthCheck();
        })().catch(() => {
          if (healthLifecycleIsCurrent()) scheduleHealthCheck();
        });
      }, getAttachmentRuntime().healthBackoffMs);
    }
    scheduleHealthCheck();

    ctx.ui.setStatus("focusa", getAttachmentRuntime().wbmEnabled ? "🤖 Focusa WBM" : "🧭 Focusa");
  });

  // ── session_shutdown — single handler (§33.8, §34.2A, §37.9) ──────────────
  pi.on("session_shutdown", async (_event, _ctx) => {
    healthLifecycle.end();
    if (getAttachmentRuntime().healthInterval) {
      clearTimeout(getAttachmentRuntime().healthInterval);
      getAttachmentRuntime().healthInterval = null;
    }
    await persistAuthoritativeState();

    // §37.9: Tell Context Core Pi is no longer active
    getAttachmentRuntime()
      .pi?.exec("wb", ["me", "--set", "pi_active=false"])
      .catch(() => {});

    // Close SSE
    if (sseReconnectTimer) {
      clearTimeout(sseReconnectTimer);
      sseReconnectTimer = null;
    }
    if (sseAbort) {
      sseAbort.abort();
      sseAbort = null;
    }

    if (getAttachmentRuntime().focusaAvailable) {
      await focusaFetch("/session/close", {
        method: "POST",
        body: JSON.stringify({ reason: "pi_session_shutdown" }),
      });
    }
    if (getAttachmentRuntime().focusaAvailable) {
      focusaPost("/instance/disconnect", { instance_id: `pi-${process.pid}` });
      focusaPost("/telemetry/activity", { surface: "pi", event: "session_shutdown" });
    }
    if (getAttachmentRuntime().footerSyncInterval) {
      clearInterval(getAttachmentRuntime().footerSyncInterval);
      getAttachmentRuntime().footerSyncInterval = null;
    }
  });

  // ── session_before_switch (§37.7) ─────────────────────────────────────────
  pi.on("session_before_switch", async (_event, _ctx) => {
    await persistAuthoritativeState();
    if (getAttachmentRuntime().focusaAvailable && getAttachmentRuntime().activeFrameId) {
      await pushDelta({
        decisions: getAttachmentRuntime().localDecisions.slice(-5),
        constraints: getAttachmentRuntime().localConstraints.slice(-5),
      }).catch(() => null);
    }
    if (getAttachmentRuntime().focusaAvailable) {
      await focusaFetch("/session/close", {
        method: "POST",
        body: JSON.stringify({ reason: "pi_session_switch" }),
      });
    }
  });

  // ── session_before_fork (§36.5) ───────────────────────────────────────────
  pi.on("session_before_fork", async (_event, _ctx) => {
    if (getAttachmentRuntime().focusaAvailable) {
      await focusaFetch("/workpoint/checkpoint", {
        method: "POST",
        body: JSON.stringify({
          mission:
            getAttachmentRuntime().currentAsk?.text ||
            getAttachmentRuntime().activeFrameGoal ||
            getAttachmentRuntime().lastFocusSnapshot.intent ||
            "Pi fork boundary",
          next_slice:
            getAttachmentRuntime().lastFocusSnapshot.currentFocus ||
            "Resume from fork WorkpointResumePacket.",
          checkpoint_reason: "fork",
          canonical: true,
          promote: true,
          continuity_id: ensureContinuityId(getSessionCwd() || process.cwd()),
          session_id: getAttachmentRuntime().sessionFrameKey,
          project_root: getSessionCwd() || process.cwd(),
          source_turn_id: `pi-turn-${getTurnCount()}`,
          action_intent: {
            action_type: "resume_workpoint",
            target_ref: getAttachmentRuntime().activeFrameId || "pi-fork",
            verification_hooks: ["fork refreshes workpoint"],
            status: "ready",
          },
        }),
      }).catch(() => null);
      await refreshSessionWorkpointPacket("fork");
      await refreshTrajectoryClarityLifecycle("handoff_fork", getSessionCwd() || process.cwd());
    }
    await persistAuthoritativeState();
    if (getAttachmentRuntime().focusaAvailable && getAttachmentRuntime().activeFrameId) {
      focusaPost("/focus/update", {
        frame_id: getAttachmentRuntime().activeFrameId,
        project_root: normalizeProjectRoot(getSessionCwd() || process.cwd()),
        continuity_id: ensureContinuityId(getSessionCwd() || process.cwd()),
        turn_id: `pi-turn-${getTurnCount()}`,
        delta: { meta: { event: "fork", timestamp: Date.now() } },
      });
    }
  });

  // ── session_before_tree (§36.5) ───────────────────────────────────────────
  pi.on("session_before_tree", async (_event, _ctx) => {
    await persistAuthoritativeState();
  });
}
