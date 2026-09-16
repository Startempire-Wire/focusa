// Request-local provenance only; canonical scope/adoption remains in state.ts.
type ResumeRuntime = {
  currentAsk?: { text?: string; sourceTurnId?: string; updatedAt?: number } | null;
  sessionFrameKey?: string | null;
  sessionCwd?: string | null;
  continuityId?: string | null;
};

const text = (value: unknown): string => String(value ?? "").trim();

export function captureResumeRequest(runtime: ResumeRuntime, evaluatedAsk: string) {
  return {
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

export function evaluateResumeRequest(
  reply: any,
  captured: ResumeRequestBinding,
  runtime: ResumeRuntime
): { accepted: boolean; reason: string } {
  const current = captureResumeRequest(runtime, captured.evaluatedAsk);
  if (Object.keys(captured).some((key) =>
    captured[key as keyof ResumeRequestBinding] !== current[key as keyof ResumeRequestBinding]
  )) return { accepted: false, reason: "resume_request_changed" };
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

export function stampResumeRequest(candidate: any, captured: ResumeRequestBinding): any {
  return {
    ...candidate,
    current_ask_binding: captured.evaluatedAsk,
    ...(candidate?.workpoint && typeof candidate.workpoint === "object"
      ? { workpoint: { ...candidate.workpoint, current_ask_binding: captured.evaluatedAsk } }
      : {}),
  };
}
