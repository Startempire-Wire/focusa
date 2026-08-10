import { currentProjectBindingDecision, getAttachmentRuntime, getLastProjectIdentity, getLastProjectVerify, getLastTrajectoryClarity, getScopedWorkpointPacket, } from "./state.js";
function firstString(root, paths) {
    for (const path of paths) {
        let value = root;
        for (const segment of path.split("."))
            value = value?.[segment];
        const text = String(value || "").trim();
        if (text)
            return text;
    }
    return "";
}
function firstArray(root, paths) {
    for (const path of paths) {
        let value = root;
        for (const segment of path.split("."))
            value = value?.[segment];
        if (Array.isArray(value))
            return value;
    }
    return [];
}
export function buildNorthStarSnapshot(trigger) {
    const verify = getLastProjectVerify();
    const identity = getLastProjectIdentity();
    const trajectory = getLastTrajectoryClarity();
    // Never fall back to the process-global active packet. A foreign packet is
    // less useful than an honest missing state and can project stale authority.
    const workpoint = getScopedWorkpointPacket();
    const binding = currentProjectBindingDecision();
    const bindingVerified = binding?.state === "BOUND" && Boolean(String(binding?.selected_project_root || "").trim());
    const identityVerified = identity?.status === "verified" ||
        identity?.project_identity?.status === "verified" ||
        identity?.project?.status === "verified";
    const projectCurrent = verify?.verification?.verified === true ||
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
    const hltCurrent = hltStatus === "canonical_explicit" ||
        Boolean(firstString(trajectory, ["long_term_goal", "hlt.value", "trajectory.long_term_goal"]));
    const mlgCurrent = Boolean(firstString(trajectory, [
        "mid_level_goal",
        "trajectory.mid_level_goal",
        "intelligence_view.mid_level_goal",
    ]));
    const stgCurrent = Boolean(firstString(trajectory, [
        "short_term_goal",
        "trajectory.short_term_goal",
        "intelligence_view.short_term_goal",
    ]));
    const waypointCurrent = firstArray(trajectory, ["waypoints", "trajectory.waypoints", "intelligence_view.waypoints"]).length > 0;
    const gapText = firstString(trajectory, ["active_gap", "gap", "intelligence_view.active_gap"]);
    const workpointCanonical = workpoint?.canonical === true && workpoint?.degraded !== true;
    const workpointAuthority = workpoint?.action_authority_for_current_ask !== false &&
        workpoint?.matches_current_ask_scope !== false &&
        workpoint?.current_ask_scope?.action_authority_for_current_ask !== false;
    const workpointId = String(workpoint?.workpoint_id || workpoint?.id || "").trim() || null;
    const frontier = String(workpoint?.work_item_id ||
        workpoint?.action_intent?.target_ref ||
        workpoint?.current_task?.work_item_id ||
        "").trim();
    const states = {
        project: projectCurrent ? "current" : "blocked",
        hlt: hltCurrent ? "current" : "missing",
        mlg: mlgCurrent ? "current" : "missing",
        stg: stgCurrent ? "current" : "missing",
        waypoint: waypointCurrent ? "current" : "missing",
        gap: gapText ? "current" : "stale",
        workpoint: !workpointCanonical
            ? "missing"
            : workpointAuthority
                ? "current"
                : trigger === "operator_input"
                    ? "steered"
                    : "mismatched",
        frontier: frontier ? "current" : "missing",
    };
    const staleSurfaces = Object.entries(states)
        .filter(([, value]) => value !== "current")
        .map(([key]) => key);
    const hardBlocked = !projectCurrent || !hltCurrent || !mlgCurrent || !stgCurrent || !workpointCanonical;
    const status = hardBlocked ? "blocked" : staleSurfaces.length ? "stale" : "ready";
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
export function renderNorthStarCard(snapshot) {
    const short = (value) => value.replace("mismatched", "mismatch").slice(0, 8);
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
export function updateNorthStarCard(ctx, trigger) {
    const snapshot = buildNorthStarSnapshot(trigger);
    getAttachmentRuntime().northStarSnapshot = snapshot;
    if (ctx?.hasUI) {
        if (getAttachmentRuntime().startupReceptionistActive) {
            ctx.ui.setWidget("focusa-north-star", undefined);
        }
        else {
            ctx.ui.setWidget("focusa-north-star", renderNorthStarCard(snapshot), { placement: "aboveEditor" });
        }
    }
    return snapshot;
}
