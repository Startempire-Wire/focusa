// Request-local provenance only; canonical scope/adoption remains in state.ts.
type ResumeRuntime = {
  currentAsk?: { text?: string; sourceTurnId?: string; updatedAt?: number } | null;
  sessionFrameKey?: string | null;
  sessionCwd?: string | null;
  continuityId?: string | null;
  projectBindingDecisions?: Record<string, {
    state?: string;
    selected_project_root?: string;
    selected_worktree_root?: string;
    canonical_parent_root?: string;
    continuity_id?: string;
    working_subpath_id?: string;
  }>;
};

const text = (value: unknown): string => String(value ?? "").trim();
const root = (value: unknown): string => text(value).replace(/\/+$/, "");

export function verifiedResumeCheckout(runtime: ResumeRuntime) {
  const decision = runtime.projectBindingDecisions?.[text(runtime.sessionFrameKey)];
  const parent = root(decision?.canonical_parent_root || decision?.selected_project_root);
  const worktree = root(decision?.selected_worktree_root);
  if (decision?.state !== "BOUND" || !parent || !worktree ||
      (root(runtime.sessionCwd) !== parent && root(runtime.sessionCwd) !== worktree) ||
      text(decision.continuity_id) !== text(runtime.continuityId)) return null;
  const id = text(decision.working_subpath_id) || (parent === worktree ? "primary" : "");
  return id ? { decision, root: worktree, projectRoot: parent, id, primary: parent === worktree } : null;
}

export function workpointMatchesCheckout(packet: any, expected: string, primary = false): boolean {
  if (!packet || !expected) return false;
  const declared = text(packet.working_subpath_id);
  const identity = text(packet.session_identity?.working_subpath_id);
  const nested = text(packet.session_identity?.project_identity?.working_context?.working_subpath?.working_subpath_id);
  const actual = declared || identity || nested || "primary";
  const matches = (id: string) => id === expected || (primary && id === "primary");
  return matches(actual) && [declared, identity, nested].every(id => !id || matches(id));
}
// Private request ownership, never serialized into a packet or restored grant.
const resumeRuntimeIdentity = Symbol("resumeRuntimeIdentity");

export function captureResumeRequest(runtime: ResumeRuntime, evaluatedAsk: string) {
  const checkout = verifiedResumeCheckout(runtime);
  return {
    [resumeRuntimeIdentity]: runtime,
    evaluatedAsk: text(evaluatedAsk),
    currentAsk: text(runtime.currentAsk?.text),
    turnId: text(runtime.currentAsk?.sourceTurnId),
    askUpdatedAt: runtime.currentAsk?.updatedAt ?? null,
    sessionId: text(runtime.sessionFrameKey),
    projectRoot: text(runtime.sessionCwd),
    continuityId: text(runtime.continuityId),
    workingSubpathId: checkout?.id,
    primaryCheckout: checkout?.primary,
    bindingDecision: runtime.projectBindingDecisions?.[text(runtime.sessionFrameKey)],
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
  if (captured.bindingDecision && (!captured.workingSubpathId ||
      !workpointMatchesCheckout(reply?.resume_packet || reply, captured.workingSubpathId, captured.primaryCheckout)))
    return { accepted: false, reason: "resume_working_subpath_unverified" };
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

// Older daemons store records without a checkout id only in the primary
// checkout. Try that alias only on an exact miss for a verified primary path.
export async function readResumeWithPrimaryCompatibility<T>(
  captured: ResumeRequestBinding,
  runtime: () => ResumeRuntime,
  payload: Record<string, unknown>,
  read: (body: Record<string, unknown>) => Promise<T>,
  unwrap: (response: T) => any = response => response,
): Promise<T> {
  const first = await read(payload);
  if (!captured.primaryCheckout || !captured.workingSubpathId ||
      captured.workingSubpathId === "primary" || payload.workpoint_id ||
      unwrap(first)?.status !== "not_found" || !isResumeRequestCurrent(captured, runtime())) return first;
  const sessionIdentity = payload.session_identity;
  return read({
    ...payload,
    working_subpath_id: "primary",
    session_identity: sessionIdentity && typeof sessionIdentity === "object"
      ? { ...sessionIdentity, working_subpath_id: "primary" } : undefined,
  });
}

// The checkpoint owner has rechecked the task and exact committed goal. Only
// retry after the named marker migration succeeds through its normal route.
export async function repairPrepareMarker<T extends { ok: boolean; status: number; body: any }>(
  reply: T,
  payload: { project_root?: string; continuity_id?: string; idempotency_key?: string },
  stillCurrent: () => boolean,
  repair: (body: Record<string, unknown>) => Promise<T>,
  retry: () => Promise<T>,
): Promise<T> {
  const recovery = reply.body?.prepare_recovery;
  if (reply.ok || reply.status !== 409 || !payload.idempotency_key || !stillCurrent() ||
      recovery?.schema !== "focusa.workpoint_prepare_recovery.v1" ||
      recovery.kind !== "rebuild_derived_trajectory_marker" ||
      !payload.project_root || recovery.project_root !== payload.project_root ||
      !payload.continuity_id || recovery.continuity_id !== payload.continuity_id ||
      !text(recovery.trajectory_id) || !Number.isSafeInteger(recovery.hlt_version) || recovery.hlt_version < 1) return reply;
  const result = await repair({
    action: "migrate", confirm: true,
    project_root: payload.project_root, continuity_id: payload.continuity_id,
    expected_trajectory_id: recovery.trajectory_id, expected_hlt_version: recovery.hlt_version,
  });
  const guard = result.body?.trajectory_integrity_guard;
  if (!result.ok || result.body?.canonical !== true || guard?.status !== "READY" ||
      guard?.expected_trajectory_id !== recovery.trajectory_id || !stillCurrent()) return reply;
  return retry();
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
