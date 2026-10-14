type CompactionScope = {
  root_scope?: { root_path?: unknown };
  continuity_id?: unknown;
};

function exact(value: unknown): string {
  return String(value || "")
    .trim()
    .replace(/[\\/]+$/, "");
}

/**
 * A hidden continuation may carry action context only when the daemon packet
 * exactly matches the live project/continuity and both authority layers agree.
 * Blocked/degraded packets are diagnostics, never resumable missions.
 */
export function canInjectCompactionMission(packet: unknown, currentScope: CompactionScope): boolean {
  if (!packet || typeof packet !== "object") return false;
  const candidate = packet as Record<string, any>;
  const scope = candidate.scope && typeof candidate.scope === "object" ? candidate.scope : {};
  const trajectory =
    candidate.trajectory && typeof candidate.trajectory === "object" ? candidate.trajectory : {};
  const workpoint = candidate.workpoint && typeof candidate.workpoint === "object" ? candidate.workpoint : {};
  const expectedRoot = exact(currentScope?.root_scope?.root_path);
  const expectedContinuity = exact(currentScope?.continuity_id);
  return (
    candidate.schema_version === "focusa.compaction_mission_packet.v1" &&
    candidate.status === "verified" &&
    candidate.current_ask?.text_complete === true &&
    typeof candidate.current_ask.text === "string" && candidate.current_ask.text.trim().length > 0 &&
    scope.scope_status === "verified" &&
    exact(scope.project_root) === expectedRoot &&
    exact(scope.continuity_id) === expectedContinuity &&
    exact(trajectory.project_root) === expectedRoot &&
    exact(trajectory.continuity_id) === expectedContinuity &&
    exact(workpoint.project_root) === expectedRoot &&
    exact(workpoint.continuity_id) === expectedContinuity &&
    Boolean(expectedRoot) &&
    Boolean(expectedContinuity) &&
    trajectory.action_authority_from_trajectory === true &&
    workpoint.status === "ready" &&
    workpoint.action_authority === true
  );
}

export function safeCompactionRecoveryContext(currentInput = "", budgetChars = 0): string {
  const warning = [
    "Focusa post-compaction continuation authority is not verified.",
    "Do not rely on a prior mission or next action.",
    "Re-verify the current project scope and resume a canonical Workpoint before continuing.",
  ].join(" ");
  if (!currentInput) return warning;
  // Fresh operator steering survives quarantine; it is not saved-action authority.
  const rendered = `${warning}\nCurrent operator input (steering only): ${JSON.stringify(currentInput)}`;
  if (budgetChars <= 0 || rendered.length > budgetChars) {
    throw new Error("compaction_resume_projection_mandatory_budget_exceeded: current steering requires rehydration");
  }
  return rendered;
}
