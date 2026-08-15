import {
  currentProjectBindingDecision,
  getAttachmentRuntime,
  getLastProjectIdentity,
  getLastProjectVerify,
  getLastTrajectoryClarity,
  getScopedWorkpointPacket,
} from "./state.js";

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

export function renderNorthStarCard(snapshot: NorthStarSnapshot): string[] {
  const short = (value: string) => value.replace("mismatched", "mismatch").slice(0, 8);
  return [
    `🧭 NORTH STAR ${snapshot.status.toUpperCase()} · ${snapshot.trigger}`,
    `PROJECT ${short(snapshot.project)} → HLT ${short(snapshot.hlt)}`,
    `MLG ${short(snapshot.mlg)} → STG ${short(snapshot.stg)}`,
    `WAYPOINT ${short(snapshot.waypoint)} → GAP ${short(snapshot.gap)}`,
    `WP ${short(snapshot.workpoint)} → FRONTIER ${short(snapshot.frontier)}`,
    snapshot.status === "ready"
      ? `workpoint=${snapshot.workpoint_id || "unknown"}`
      : `recovery=${snapshot.exact_recovery}`,
  ];
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
