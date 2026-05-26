import { S, getScopedWorkpointPacket, isProjectRootAuthoritySafe, normalizeProjectRoot } from "./state.js";

function line(value: unknown): string {
  return String(value || "").trim();
}

function compact(value: unknown, fallback = "unknown", max = 180): string {
  const text = line(value).replace(/\s+/g, " ");
  if (!text) return fallback;
  return text.length > max ? `${text.slice(0, Math.max(0, max - 1))}…` : text;
}

function firstValue(...values: unknown[]): string {
  for (const value of values) {
    const text = line(value);
    if (text) return text;
  }
  return "";
}

export function buildFocusaUtilityCard(mode: "system" | "visible" = "system"): string {
  const scopedPacket = getScopedWorkpointPacket();
  const mission = line(scopedPacket?.mission);
  const next = line(scopedPacket?.next_slice);
  const projectRoot = normalizeProjectRoot(scopedPacket?.project_root || S.sessionCwd);
  const continuityId = scopedPacket ? line(scopedPacket?.continuity_id) : line(S.continuityId);
  const status = S.focusaAvailable ? "available" : "offline/degraded";
  const prefix = mode === "visible" ? "# Focusa Utility Card" : "## Focusa Utility Card";
  const safeScope = !!projectRoot && isProjectRootAuthoritySafe(projectRoot);
  const resolution = S.lastProjectRootResolution;
  const confidence = resolution ? ` confidence=${Math.round(resolution.confidenceScore * 100)}% source=${resolution.source}` : "";
  const needsConfirm = resolution?.requiresOperatorConfirmation === true;
  const trajectory = S.lastTrajectoryClarity || {};
  const cachedProjectIdentity = S.lastProjectIdentity && normalizeProjectRoot(S.lastProjectIdentity.project_root) === projectRoot ? S.lastProjectIdentity : null;
  const verifiedProjectIdentity = S.lastProjectVerify?.project_identity && normalizeProjectRoot(S.lastProjectVerify.project_identity.project_root) === projectRoot ? S.lastProjectVerify.project_identity : null;
  const trajectoryProjectIdentity = trajectory.project_identity && normalizeProjectRoot(trajectory.project_identity.project_root) === projectRoot ? trajectory.project_identity : null;
  const projectIdentity = safeScope ? (trajectoryProjectIdentity || cachedProjectIdentity || verifiedProjectIdentity || {}) : {};
  const projectSummary = projectIdentity.project_summary || {};
  const trajectoryFallback = trajectory.fallback_prior_project_trajectory === true;
  const projectUrls = trajectoryFallback ? (projectIdentity.project_urls || projectSummary.urls || {}) : (trajectory.project_urls || projectIdentity.project_urls || projectSummary.urls || {});
  const deployment = trajectoryFallback ? (projectIdentity.deployment || projectSummary.deployment || {}) : (trajectory.deployment || projectIdentity.deployment || projectSummary.deployment || {});
  const trajectorySet = !trajectoryFallback && !!(trajectory.long_term_goal || trajectory.desired_end_state || trajectory.active_gap || trajectory.status);
  const workpointStatus = scopedPacket
    ? "verified"
    : S.activeWorkpointSummary
      ? "summary_only"
      : "unavailable/not_verified";
  const envParts = [
    firstValue(projectUrls.root_url, projectUrls.live_url) ? `root=${compact(firstValue(projectUrls.root_url, projectUrls.live_url), "", 120)}` : "",
    projectUrls.wp_url ? `wp=${compact(projectUrls.wp_url, "", 120)}` : "",
    projectUrls.app_url ? `app=${compact(projectUrls.app_url, "", 120)}` : "",
    projectUrls.auth_url ? `auth=${compact(projectUrls.auth_url, "", 120)}` : "",
    projectUrls.local_url ? `local=${compact(projectUrls.local_url, "", 120)}` : "",
    deployment.environment ? `env=${compact(deployment.environment, "", 50)}` : "",
    firstValue(projectUrls.inference_confidence, deployment.inference_confidence) ? `confidence=${compact(firstValue(projectUrls.inference_confidence, deployment.inference_confidence), "", 40)}` : "",
  ].filter(Boolean).join("; ");

  const missionPacket = [
    "MISSION_PACKET:",
    `- project=${safeScope ? compact(projectIdentity.canonical_name || projectIdentity.project_id, "unknown", 80) : "UNBOUND_UNSAFE_ROOT"} root=${projectRoot || "unknown"}${confidence}`,
    `- trajectory=${trajectoryFallback ? "prior_project_fallback_advisory" : trajectorySet ? "set" : "not_hydrated"}; high=${compact(trajectory.long_term_goal, "unknown")}; desired=${compact(trajectory.desired_end_state, "unknown")}`,
    `- current=${compact(trajectory.current_state, "unknown")}; gap=${compact(trajectory.active_gap || trajectory.short_term_goal, "unknown")}; recommended=${compact(trajectory.recommended_action, "unknown", 120)}`,
    `- workpoint=${workpointStatus}; ${scopedPacket ? "canonical packet matches project_root+continuity_id" : "resume/checkpoint required before treating Workpoint as canonical"}`,
    `- next=${!safeScope || needsConfirm ? "auto-bootstrap project identity with focusa_project_identity before durable work" : next ? compact(next) : compact(trajectory.active_gap || trajectory.short_term_goal || mission || "refresh trajectory then checkpoint mission")}`,
    `- environment=${envParts || "unknown; call focusa_project_identity/trajectory_view for URL/deploy facts"}`,
    `- boundary=operator steering wins; project_root+continuity_id are authority; trajectory similarity/fallback is advisory only`,
  ];

  const friendlyQ = [
    "Friendly Focusa Q (internal orientation, not a blocker):",
    "1. Where am I? project_root + continuity_id → focusa_project_identity / focusa_project_verify.",
    "2. What kind of project is this? canonical name, repo, root/live/local URLs, deploy target/location, workspace kind, infra/architecture boundaries → focusa_project_identity + focusa_traverse.",
    "3. Where are we going? current state, destination, waypoints → focusa_trajectory_view / define_goal / assess.",
    "4. What does the ontology say? project/trajectory/workpoint/evidence/prediction/metacog objects → focusa_traverse surface=ontology.",
    "5. What is the next useful move? mission + active object + next anchor → focusa_workpoint_resume / checkpoint.",
    "6. What proof changes confidence? tests/API/file handles → focusa_active_object_resolve + focusa_evidence_capture/link.",
    "7. What compounds? prediction outcome + reusable lesson → focusa_predict_record + focusa_predict_evaluate + focusa_metacog_*.",
    "8. What can re-bootstrap? project card + learned signals can propose/refresh trajectory hierarchy when goals are stale or missing.",
  ];
  const routeHints = [
    "Tool routes: Orient = focusa_project_identity → focusa_trajectory_view → focusa_workpoint_resume; Ontology = focusa_traverse(surface=ontology) to bind project/trajectory/workpoint/evidence/prediction/metacog objects; Execute = focusa_active_object_resolve → focusa_workpoint_checkpoint; Prove = focusa_evidence_capture / focusa_workpoint_link_evidence → focusa_trajectory_assess; Learn = focusa_predict_record → focusa_predict_evaluate → focusa_metacog_capture/retrieve; Re-bootstrap = metacog/prediction/project-card signals → focusa_trajectory_define_goal/assess; Recover = focusa_tool_doctor → focusa_resource_mode/focusa_traverse/focusa_workpoint_resume.",
    "Missing active Pi frame fallback: Attentive and awaiting operator direction; keep helping from operator/repo context, then checkpoint/resume once scope is safe.",
    "Focus State tools (scratch/decide/constraint/failure/etc.) are note/decision slots; use them with the project route, not instead of it.",
  ];

  if (mode === "visible" && !scopedPacket) {
    return [
      prefix,
      `Status: ${status}`,
      ...missionPacket,
      `Project folder: ${projectRoot || "unknown"}${safeScope ? "" : " (broad/unsafe — no Workpoint auto-resume)"}${confidence}`,
      ...friendlyQ,
      "Project-bound Workpoint: none verified yet; latest operator instruction + trajectory gap are the seed, then checkpoint to create canonical Workpoint.",
      !safeScope || needsConfirm
        ? "Suggested first route: confirm project folder by inferring from cwd/git/beads/repo context; if still unsure, ask operator directly in chat which project folder to bind — no input-only modal."
        : "Suggested first route: run focusa_trajectory_view for goals/gap, then checkpoint once mission and next action are clear.",
      ...routeHints,
    ].join("\n");
  }

  return [
    prefix,
    `Status: ${status}`,
    ...missionPacket,
    scopedPacket ? "Project-bound Workpoint: verified project_root + continuity_id match." : "Project-bound Workpoint: none verified for this logical session; use trajectory gap + operator ask, then checkpoint; ignore stale carryover.",
    !safeScope || needsConfirm ? "Project folder check: confirm the project file folder/container before durable state writes." : "Project root: confirmed project file folder/container; trajectory provides the functional route.",
    mission ? `Mission: ${mission}` : "Mission: use latest operator instruction as seed; bind it to trajectory + Workpoint before long work.",
    next ? `Next anchor: ${next}` : "Next anchor: call focusa_workpoint_resume with current continuity_id if resuming project work or uncertain.",
    projectRoot ? `Project folder: project_root=${projectRoot}${safeScope ? "" : " (broad/unsafe)"}` : "Project folder: bind work to the folder containing project files; reject cross-project resume packets.",
    scopedPacket && continuityId ? `Continuity: continuity_id=${continuityId}` : "Continuity: no Workpoint continuity verified for this Pi session; use resume/checkpoint before trusting same-root state.",
    trajectoryFallback ? `Trajectory: prior-project fallback only from continuity=${compact(trajectory.fallback_source_continuity_id, "unknown", 80)}; refresh/define current continuity before durable trajectory writes.` : trajectorySet ? `Trajectory: high=${compact(trajectory.long_term_goal)}; current=${compact(trajectory.current_state)}; gap=${compact(trajectory.active_gap || trajectory.short_term_goal)}.` : "Trajectory: not hydrated in Utility Card memory; run focusa_trajectory_view before durable state writes; if missing/stale, re-bootstrap from project card + ontology + prediction/metacog signals.",
    "",
    ...friendlyQ,
    ...routeHints,
    "- Compaction/model switch/fork/risky continuation: focusa_workpoint_checkpoint before; focusa_workpoint_resume after.",
    "- Continuous/background work: focusa_work_loop_* after writer-status/preflight; focusa_silent_sessions only when explicitly managing background sessions.",
    "Operator steering always wins; Focusa guides, preserves, and audits.",
  ].join("\n");
}
