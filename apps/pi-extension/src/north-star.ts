import {
  currentProjectBindingDecision,
  getAttachmentRuntime,
  getLastProjectIdentity,
  getLastProjectVerify,
  getLastTrajectoryClarity,
  getScopedWorkpointPacket,
  normalizeProjectRoot,
} from "./state.js";

// Projection qualification only: operation admission remains daemon-owned.
export function canonicalDaemonNorthStarReadiness(
  responseOk: boolean, body: any, projectRoot: string, continuityId: string,
): { ready: boolean; reconcile: boolean; scope_verified: boolean; reason: string } {
  const blocked = (reason: string, scope_verified = false) => ({ ready: false, reconcile: false, scope_verified, reason });
  if (!responseOk) return blocked("daemon_response_rejected");
  if (body?.canonical !== true || body.authority !== "daemon_owned" || body.advisory === true) {
    return blocked("canonical_authority_unverified");
  }
  const expectedRoot = normalizeProjectRoot(projectRoot);
  const rootValues = [body.project_root, body.projection?.project_root, body.workpoint_linkage?.project_root]
    .filter(value => value !== undefined);
  const continuityValues = [body.projection?.continuity_id, body.workpoint_linkage?.continuity_id]
    .filter(value => value !== undefined);
  if (!expectedRoot || !continuityId || typeof body.project_root !== "string" || !continuityValues.length) {
    return blocked("scope_identity_missing");
  }
  if (rootValues.some(value => typeof value !== "string" || normalizeProjectRoot(value) !== expectedRoot) ||
      continuityValues.some(value => typeof value !== "string" || value !== continuityId)) {
    return blocked("scope_mismatch");
  }
  if (body.projection && (body.projection.schema !== "focusa.north_star_projection.v1" ||
      !["complete", "incomplete"].includes(body.projection.status))) {
    return blocked("projection_stale_or_unsupported");
  }
  const guardStatus = body.trajectory_integrity_guard?.status;
  if (guardStatus !== "READY" && guardStatus !== "ready") return blocked("trajectory_guard_not_ready", true);
  const reconcile = body.status === "reconcile" && body.code === "WORKPOINT_FRONTIER_MISSING";
  if (body.status !== "completed" && !reconcile) return blocked("daemon_blocked", true);
  return { ready: true, reconcile, scope_verified: true, reason: "" };
}

export type NorthStarSurfaceState = "current" | "stale" | "missing" | "mismatched" | "steered" | "blocked";

export type NorthStarSnapshot = {
  schema: "focusa.north_star_snapshot.v1";
  status: "ready" | "blocked" | "stale";
  trigger: string;
  checked_at: string;
  project: NorthStarSurfaceState;
  hlt: NorthStarSurfaceState;
  mlg: NorthStarSurfaceState;
  stg: NorthStarSurfaceState;
  waypoint: NorthStarSurfaceState;
  gap: NorthStarSurfaceState;
  workpoint: NorthStarSurfaceState;
  frontier: NorthStarSurfaceState;
  workpoint_id: string | null;
  stale_surfaces: string[];
  exact_recovery: string;
};

function firstString(root: any, paths: string[]): string {
  for (const path of paths) {
    let value = root;
    for (const segment of path.split(".")) value = value?.[segment];
    const text = String(value || "").trim();
    if (text) return text;
  }
  return "";
}

function firstArray(root: any, paths: string[]): any[] {
  for (const path of paths) {
    let value = root;
    for (const segment of path.split(".")) value = value?.[segment];
    if (Array.isArray(value)) return value;
  }
  return [];
}

export function buildNorthStarSnapshot(trigger: string): NorthStarSnapshot {
  const verify = getLastProjectVerify() as any;
  const identity = getLastProjectIdentity() as any;
  const trajectory = getLastTrajectoryClarity() as any;
  // Never fall back to the process-global active packet. A foreign packet is
  // less useful than an honest missing state and can project stale authority.
  const workpoint = getScopedWorkpointPacket() as any;
  const binding = currentProjectBindingDecision() as any;
  const bindingVerified =
    binding?.state === "BOUND" && Boolean(String(binding?.selected_project_root || "").trim());
  const identityVerified =
    identity?.status === "verified" ||
    identity?.project_identity?.status === "verified" ||
    identity?.project?.status === "verified";
  const projectCurrent =
    verify?.verification?.verified === true ||
    verify?.verified === true ||
    verify?.canonical === true ||
    identityVerified ||
    bindingVerified;
  const hltStatus = firstString(trajectory, [
    "hlt_status",
    "trajectory.hlt_status",
    "intelligence_view.hlt_status",
    "hlt.status",
  ]);
  const hltCurrent =
    hltStatus === "canonical_explicit" ||
    Boolean(firstString(trajectory, ["long_term_goal", "hlt.value", "trajectory.long_term_goal"]));
  const mlgCurrent = Boolean(
    firstString(trajectory, [
      "mid_level_goal",
      "trajectory.mid_level_goal",
      "intelligence_view.mid_level_goal",
    ])
  );
  const stgCurrent = Boolean(
    firstString(trajectory, [
      "short_term_goal",
      "trajectory.short_term_goal",
      "intelligence_view.short_term_goal",
    ])
  );
  const waypointCurrent =
    firstArray(trajectory, ["waypoints", "trajectory.waypoints", "intelligence_view.waypoints"]).length > 0;
  const gapText = firstString(trajectory, ["active_gap", "gap", "intelligence_view.active_gap"]);
  const workpointCanonical = workpoint?.canonical === true && workpoint?.degraded !== true;
  const workpointAuthority =
    workpoint?.action_authority_for_current_ask !== false &&
    workpoint?.matches_current_ask_scope !== false &&
    workpoint?.current_ask_scope?.action_authority_for_current_ask !== false;
  const workpointId = String(workpoint?.workpoint_id || workpoint?.id || "").trim() || null;
  const frontier = String(
    workpoint?.work_item_id ||
      workpoint?.action_intent?.target_ref ||
      workpoint?.current_task?.work_item_id ||
      ""
  ).trim();
  const states = {
    project: projectCurrent ? ("current" as const) : ("blocked" as const),
    hlt: hltCurrent ? ("current" as const) : ("missing" as const),
    mlg: mlgCurrent ? ("current" as const) : ("missing" as const),
    stg: stgCurrent ? ("current" as const) : ("missing" as const),
    waypoint: waypointCurrent ? ("current" as const) : ("missing" as const),
    gap: gapText ? ("current" as const) : ("missing" as const),
    workpoint: !workpointCanonical
      ? ("missing" as const)
      : workpointAuthority
        ? ("current" as const)
        : trigger === "operator_input"
          ? ("steered" as const)
          : ("mismatched" as const),
    frontier: frontier ? ("current" as const) : ("missing" as const),
  };
  const staleSurfaces = Object.entries(states)
    .filter(([, value]) => value !== "current")
    .map(([key]) => key);
  // "missing" surfaces are honest and informational (a verified trajectory can
  // legitimately have no persisted gap text until assess runs); only
  // stale/mismatched/steered surfaces signal an authority problem.
  const impairing = Object.entries(states)
    .filter(([, value]) => ["stale", "mismatched", "steered"].includes(value))
    .map(([key]) => key);
  const hardBlocked = !projectCurrent || !hltCurrent || !mlgCurrent || !stgCurrent || !workpointCanonical;
  const status = hardBlocked ? "blocked" : impairing.length ? "stale" : "ready";
  const exactRecovery = !projectCurrent
    ? "focusa_project_identity → focusa_project_verify"
    : !hltCurrent || !mlgCurrent || !stgCurrent || !waypointCurrent
      ? "focusa_trajectory_view → focusa_trajectory_define_goal/assess"
      : !workpointCanonical || !workpointAuthority
        ? "focusa_workpoint_resume → focusa_workpoint_checkpoint"
        : !frontier
          ? "refresh provider/Workset frontier"
          : "continue current Workpoint";
  return {
    schema: "focusa.north_star_snapshot.v1",
    status,
    trigger,
    checked_at: new Date().toISOString(),
    ...states,
    workpoint_id: workpointId,
    stale_surfaces: staleSurfaces,
    exact_recovery: exactRecovery,
  };
}

export function northStarRecoveryTools(): string[] {
  return ["focusa_agent_prompt", "focusa_project_identity", "focusa_project_bootstrap", "focusa_trajectory_view", "focusa_project_genesis", "focusa_workpoint_resume"];
}

export function northStarRecoveryNotice(): string {
  return "Read the relevant guidance and docs/agent/02-focusa-cohesive-project-flow.md before dependent features. Verify the intended project; inspect bootstrap/Genesis status and reuse or complete its existing operator-confirmed goal hierarchy, milestones, assessed gap and linked action/next step. Cwd or a saved goal sentence is not complete setup. Status inspection is not permission to create goals, scopes or grants. Governed project work requires bootstrap and a canonical Trajectory. A context warning does not revoke existing permission: use supported in-scope recovery and continue independently authorized work. Ask only for genuinely new scope, required consent/credentials or an unresolved consequential choice; real operation refusals still apply. Context warnings are not a blanket work stop. Report the exact failed operation, evidence, recovery attempted and independently authorized next work.";
}

export function renderNorthStarCard(snapshot: NorthStarSnapshot): string[] {
  const short = (value: string) => value.replace("mismatched", "mismatch").slice(0, 8);
  const contextLabel = snapshot.status === "stale" ? "CONTEXT STALE" : "CONTEXT NOT READY";

  // Ready: one calm line — no chain dump.
  if (snapshot.status === "ready") {
    const wp = snapshot.workpoint_id ? ` · resuming ${snapshot.workpoint_id}` : "";
    return [`🧭 Focusa · ${short(snapshot.project)} · on track${wp}`];
  }

  // Progressive disclosure: name the FIRST thing that needs attention in
  // plain language, offer one next step, and count the rest instead of
  // dumping internal state names.
  // Machine-scannable state ladder (contract: north-star-gate) — one line,
  // bounded by short(), kept alongside the operator prose below.
  const ladder = `PROJECT ${short(snapshot.project)} → HLT ${short(snapshot.hlt)} · MLG ${short(snapshot.mlg)} → STG ${short(snapshot.stg)} · WP ${short(snapshot.workpoint)} → FRONTIER ${short(snapshot.frontier)}`;

  const surfaces: Array<[string, NorthStarSurfaceState, string, string]> = [
    ["project", snapshot.project, "required project context isn't verified", "verify the named project and complete authorized bootstrap; ask only if project identity is ambiguous"],
    ["hlt", snapshot.hlt, "the required canonical project trajectory isn't resolved here", "recover the approved project goal through its existing owner; ask only for unresolved intent"],
    ["mlg", snapshot.mlg, "the approved milestone isn't resolved here", "recover or derive the next milestone within the approved goal"],
    ["stg", snapshot.stg, "the authorized next step isn't resolved here", "recover or define the next authorized bounded step"],
    ["waypoint", snapshot.waypoint, "the milestone marker isn't resolved here", "reuse or derive the milestone marker"],
    ["gap", snapshot.gap, "progress against the desired state hasn't been assessed", "assess progress against the approved goal"],
    ["workpoint", snapshot.workpoint, "a matching saved checkpoint isn't resolved here", "resume matching saved work or checkpoint the authorized next step"],
    ["frontier", snapshot.frontier, "the live work queue hasn't been reconciled", "reconcile the existing work queue through its owner"],
  ];

  const problems = surfaces.filter(([_, state]) => state !== "current" && state !== "steered");
  const staleNames = surfaces
    .filter(([_, state]) => state === "stale")
    .map(([key]) => key);

  if (problems.length === 0) {
    // A changed instruction is not evidence that the saved goal aged out.
    const steered = snapshot.workpoint === "steered";
    return [
      `🧭 NORTH STAR ${contextLabel} · ${short(snapshot.project)} · ${steered ? "current instruction needs reconciliation" : "needs a quick refresh"}`,
      `WAYPOINT ${short(snapshot.waypoint)} → GAP ${short(snapshot.gap)}`,
      steered
        ? "Resume saved work and verify the current instruction before continuing."
        : `${staleNames.length ? `Some details (${staleNames.join(", ")}) aged out` : "Details aged out"} — re-check through the supported owner within the existing assignment.`,
      northStarRecoveryNotice(),
    ];
  }

  // Presentation must lead with the cause of the status, not an informational
  // missing gap/waypoint/frontier. This ranking does not change authority.
  const primary = (snapshot.status === "stale"
    ? surfaces.find(([_, state]) => ["stale", "mismatched", "steered"].includes(state))
    : problems.find(([key, state]) => state === "blocked" ||
        (state === "missing" && ["project", "hlt", "mlg", "stg", "workpoint"].includes(key)))) || problems[0];
  const [firstKey, firstState, defaultWhat, defaultAction] = primary;
  const requestMismatch = firstKey === "workpoint" &&
    (firstState === "mismatched" || firstState === "steered");
  const firstWhat = requestMismatch
    ? "saved work needs reconciliation with the current instruction"
    : defaultWhat;
  const firstAction = requestMismatch ? "resume and verify the current action" : defaultAction;
  const otherProblems = problems.filter(([key]) => key !== firstKey);
  const staleNote = firstState === "stale" ? " needs a refresh — " : firstState === "mismatched" ? " doesn't match what I'm seeing — " : " — ";
  const lines = [
    `🧭 NORTH STAR ${contextLabel} · ${firstWhat}${staleNote}${firstAction}.`,
    `WAYPOINT ${short(snapshot.waypoint)} → GAP ${short(snapshot.gap)}`,
  ];
  if (otherProblems.length > 0) {
    lines.push(`${otherProblems.length} more context item${otherProblems.length > 1 ? "s" : ""} remain; inspect fuller diagnostics as needed.`);
  }
  lines.push(northStarRecoveryNotice());
  lines.push(ladder);
  return lines;
}

export function updateNorthStarCard(ctx: any, trigger: string): NorthStarSnapshot {
  const snapshot = buildNorthStarSnapshot(trigger);
  getAttachmentRuntime().northStarSnapshot = snapshot;
  if (ctx?.hasUI) {
    if (getAttachmentRuntime().startupReceptionistActive) {
      ctx.ui.setWidget("focusa-north-star", undefined);
    } else {
      ctx.ui.setWidget("focusa-north-star", renderNorthStarCard(snapshot), { placement: "aboveEditor" });
    }
  }
  return snapshot;
}
