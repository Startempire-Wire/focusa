// Request-local provenance only; canonical scope/adoption remains in state.ts.
type ResumeRuntime = {
  currentAsk?: { text?: string; sourceTurnId?: string; updatedAt?: number } | null;
  sessionFrameKey?: string | null;
  sessionCwd?: string | null;
  continuityId?: string | null;
};

const text = (value: unknown): string => String(value ?? "").trim();
// Private request ownership, never serialized into a packet or restored grant.
const resumeRuntimeIdentity = Symbol("resumeRuntimeIdentity");

export function captureResumeRequest(runtime: ResumeRuntime, evaluatedAsk: string) {
  return {
    [resumeRuntimeIdentity]: runtime,
    evaluatedAsk: text(evaluatedAsk),
    currentAsk: text(runtime.currentAsk?.text),
    turnId: text(runtime.currentAsk?.sourceTurnId),
    askUpdatedAt: runtime.currentAsk?.updatedAt ?? null,
    sessionId: text(runtime.sessionFrameKey),
    projectRoot: text(runtime.sessionCwd),
    continuityId: text(runtime.continuityId),
  };
}

export type ResumeRequestBinding = ReturnType<typeof captureResumeRequest>;

export function isResumeRequestCurrent(captured: ResumeRequestBinding, runtime: ResumeRuntime): boolean {
  if (captured[resumeRuntimeIdentity] !== runtime) return false;
  const current = captureResumeRequest(runtime, captured.evaluatedAsk);
  return Object.keys(captured).every((key) =>
    captured[key as keyof ResumeRequestBinding] === current[key as keyof ResumeRequestBinding]
  );
}

export function evaluateResumeRequest(
  reply: any,
  captured: ResumeRequestBinding,
  runtime: ResumeRuntime
): { accepted: boolean; reason: string } {
  if (!isResumeRequestCurrent(captured, runtime))
    return { accepted: false, reason: "resume_request_changed" };
  if (captured.evaluatedAsk !== captured.currentAsk)
    return { accepted: false, reason: "resume_evaluated_different_ask" };
  const v2 = reply?.resume_packet_v2;
  if (reply?.status !== "completed" || reply?.canonical !== true || reply?.degraded === true ||
      v2?.canonical === false || v2?.degraded === true)
    return { accepted: false, reason: "resume_not_canonical" };
  if (reply?.action_authority_for_current_ask !== true ||
      reply?.matches_current_ask_scope !== true ||
      reply?.current_ask_scope?.action_authority_for_current_ask === false ||
      reply?.current_ask_scope?.matches_current_ask_scope === false ||
      v2?.action_authority_for_current_ask === false ||
      v2?.matches_current_ask_scope === false ||
      v2?.current_ask_scope?.action_authority_for_current_ask === false ||
      v2?.current_ask_scope?.matches_current_ask_scope === false)
    return { accepted: false, reason: "resume_authority_not_verified" };
  return { accepted: true, reason: "none" };
}

// Generic/unscoped missions carry no assignment boundary, so adopting them can
// inherit another assignment's mission text, frames, or next-actions (#624).
// Fail-closed stoplist covers empty/missing plus the daemon/extension fallback
// strings and placeholder missions. Kept exact-match and conservative: a
// specific mission, however short, is never rejected here.
const GENERIC_WORKPOINT_MISSIONS = new Set([
  "test",
  "testing",
  "untitled",
  "todo",
  "tbd",
  "mission",
  "workpoint",
  "unknown",
  "unspecified mission",
  "unspecified target",
]);

export function isGenericWorkpointMission(mission: unknown): boolean {
  const normalized = String(mission ?? "").trim().toLowerCase();
  if (!normalized) return true;
  return GENERIC_WORKPOINT_MISSIONS.has(normalized);
}

// Saved context and changed steering retain orientation, never cached permission.
// Only a fresh daemon response accepted by evaluateResumeRequest can regrant it.
export function revokeWorkpointRequestAuthority(packet: any, reason: string): any {
  if (!packet || typeof packet !== "object") return packet;
  return {
    ...packet,
    action_authority_for_current_ask: false,
    matches_current_ask_scope: false,
    current_ask_scope: {
      ...packet.current_ask_scope,
      action_authority_for_current_ask: false,
      matches_current_ask_scope: false,
      scope_conflict_reason: reason,
    },
  };
}

export function stampResumeRequest(candidate: any, captured: ResumeRequestBinding): any {
  return {
    ...candidate,
    current_ask_binding: captured.evaluatedAsk,
    ...(candidate?.workpoint && typeof candidate.workpoint === "object"
      ? { workpoint: { ...candidate.workpoint, current_ask_binding: captured.evaluatedAsk } }
      : {}),
  };
}
