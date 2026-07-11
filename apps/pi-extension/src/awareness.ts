import {
  S,
  getScopedWorkpointPacket,
  isProjectRootAuthoritySafe,
  normalizeProjectRoot,
  getFocusaAvailable,
  getSessionCwd,
  getContinuityId,
  getActiveWorkpointPacket,
  getActiveWorkpointSummary,
  getLastTrajectoryClarity,
  getLastProjectVerify,
  getLastProjectIdentity,
  getLastProjectRootResolution,
} from "./state.js";

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
  const projectRoot = normalizeProjectRoot(scopedPacket?.project_root || getSessionCwd());
  const continuityId = scopedPacket ? line(scopedPacket?.continuity_id) : line(getContinuityId());
  const status = getFocusaAvailable() ? "available" : "offline/degraded";
  const prefix = mode === "visible" ? "# Focusa Utility Card" : "## Focusa Utility Card";
  const safeScope = !!projectRoot && isProjectRootAuthoritySafe(projectRoot);
  const resolution = getLastProjectRootResolution();
  const confidence = resolution
    ? ` confidence=${Math.round(resolution.confidenceScore * 100)}% source=${resolution.source}`
    : "";
  const needsConfirm = resolution?.requiresOperatorConfirmation === true;
  const trajectory = getLastTrajectoryClarity() || {};
  const lastIdentity = getLastProjectIdentity();
  const cachedProjectIdentity =
    lastIdentity && normalizeProjectRoot(lastIdentity.project_root) === projectRoot ? lastIdentity : null;
  const lastVerify = getLastProjectVerify();
  const verifiedProjectIdentity =
    lastVerify?.project_identity &&
    normalizeProjectRoot(lastVerify.project_identity.project_root) === projectRoot
      ? lastVerify.project_identity
      : null;
  const trajectoryProjectIdentity =
    trajectory.project_identity &&
    normalizeProjectRoot(trajectory.project_identity.project_root) === projectRoot
      ? trajectory.project_identity
      : null;
  const projectIdentity = safeScope
    ? trajectoryProjectIdentity || cachedProjectIdentity || verifiedProjectIdentity || {}
    : {};
  const projectSummary = projectIdentity.project_summary || {};
  const trajectoryFallback = trajectory.fallback_prior_project_trajectory === true;
  const projectUrls = trajectoryFallback
    ? projectIdentity.project_urls || projectSummary.urls || {}
    : trajectory.project_urls || projectIdentity.project_urls || projectSummary.urls || {};
  const deployment = trajectoryFallback
    ? projectIdentity.deployment || projectSummary.deployment || {}
    : trajectory.deployment || projectIdentity.deployment || projectSummary.deployment || {};
  const trajectorySet =
    !trajectoryFallback &&
    !!(
      trajectory.long_term_goal ||
      trajectory.desired_end_state ||
      trajectory.active_gap ||
      trajectory.status
    );
  const workpointStatus = scopedPacket
    ? "verified"
    : getActiveWorkpointSummary()
      ? "summary_only"
      : "unavailable/not_verified";
  const envParts = [
    firstValue(projectUrls.root_url, projectUrls.live_url)
      ? `root=${compact(firstValue(projectUrls.root_url, projectUrls.live_url), "", 120)}`
      : "",
    projectUrls.wp_url ? `wp=${compact(projectUrls.wp_url, "", 120)}` : "",
    projectUrls.app_url ? `app=${compact(projectUrls.app_url, "", 120)}` : "",
    projectUrls.auth_url ? `auth=${compact(projectUrls.auth_url, "", 120)}` : "",
    projectUrls.local_url ? `local=${compact(projectUrls.local_url, "", 120)}` : "",
    deployment.environment ? `env=${compact(deployment.environment, "", 50)}` : "",
    firstValue(projectUrls.inference_confidence, deployment.inference_confidence)
      ? `confidence=${compact(firstValue(projectUrls.inference_confidence, deployment.inference_confidence), "", 40)}`
      : "",
  ]
    .filter(Boolean)
    .join("; ");

  // Spec 125 §10: HLT status fields for MISSION_PACKET.
  const hltStatus = trajectory.hlt_status || "missing_required";
  const trajectoryRequired = trajectory.trajectory_required ?? true;
  const hltRequired = trajectory.hlt_required ?? true;
  const actionAuthority = trajectory.action_authority_from_trajectory ?? false;
  const genericBootstrap = trajectory.generic_bootstrap ?? false;
  const loudWarning = trajectory.loud_warning || null;
  const fallbackLevel = trajectory.fallback_level || "none";
  const fallbackSourceScope = trajectory.fallback_source_scope || null;

  const missionPacket = [
    "MISSION_PACKET:",
    `- project=${safeScope ? compact(projectIdentity.canonical_name || projectIdentity.project_id, "unknown", 80) : "UNBOUND_UNSAFE_ROOT"} root=${projectRoot || "unknown"}${confidence}`,
    `- hlt_status=${hltStatus}; trajectory_required=${trajectoryRequired}; hlt_required=${hltRequired}; action_authority=${actionAuthority}`,
    `- generic_bootstrap=${genericBootstrap}; fallback_level=${fallbackLevel}; fallback_source=${fallbackSourceScope || "none"}`,
    `- trajectory=${trajectoryFallback ? "prior_project_fallback_advisory" : trajectorySet ? "set" : "not_hydrated"}; high=${compact(trajectory.long_term_goal, "unknown")}; desired=${compact(trajectory.desired_end_state, "unknown")}`,
    `- current=${compact(trajectory.current_state, "unknown")}; gap=${compact(trajectory.active_gap || trajectory.short_term_goal, "unknown")}; recommended=${compact(trajectory.recommended_action, "unknown", 120)}`,
    `- workpoint=${workpointStatus}; ${scopedPacket ? "canonical packet matches project_root+continuity_id" : "resume/checkpoint required before treating Workpoint as canonical"}`,
    ...(loudWarning ? [`- LOUD_WARNING: ${loudWarning}`] : []),
    `- next=${!safeScope || needsConfirm ? "auto-bootstrap project identity with focusa_project_identity before durable work" : (hltRequired ? "focusa_trajectory_define_goal (HLT required)" : (next ? compact(next) : compact(trajectory.active_gap || trajectory.short_term_goal || mission || "refresh trajectory then checkpoint mission")))}`,
    `- environment=${envParts || "unknown; call focusa_project_identity/trajectory_view for URL/deploy facts"}`,
    `- boundary=operator steering wins; project_root+continuity_id are authority; trajectory similarity/fallback is advisory only`,
  ];

  const reconciliationActive =
    !safeScope ||
    needsConfirm ||
    trajectoryFallback ||
    (trajectorySet && !scopedPacket && workpointStatus !== "verified");
  const reconciliationEnvelope = reconciliationActive
    ? [
        "RECONCILIATION_ENVELOPE:",
        `- surface_states=workpoint:${workpointStatus}; trajectory:${trajectoryFallback ? "fallback_advisory" : trajectorySet ? "available" : "not_hydrated"}; focus_state:unknown; ontology:unknown; evidence:${scopedPacket ? "workpoint_refs_available" : "checkpoint_first"}; doctor:unknown; work_loop:unknown`,
        `- resolution=${!safeScope || needsConfirm ? "verify_project_scope_first" : trajectoryFallback ? "refresh_current_trajectory" : "checkpoint_or_resume_workpoint"}`,
        `- authority_for_next_action=${!safeScope || needsConfirm ? "project_identity_verification" : scopedPacket ? "canonical_workpoint" : "operator_current_ask_until_checkpoint"}`,
        `- supporting_context=project_root:${projectRoot || "unknown"}; continuity_id:${continuityId || "unknown"}`,
        `- blocked_or_stale_surfaces=${[!safeScope || needsConfirm ? "scope" : "", trajectoryFallback ? "trajectory" : "", !scopedPacket ? "workpoint" : ""].filter(Boolean).join(",") || "none"}`,
        `- next_repair_tool=${!safeScope || needsConfirm ? "focusa_project_identity" : trajectoryFallback ? "focusa_trajectory_view" : "focusa_workpoint_checkpoint"}`,
      ]
    : [];

  const nowWhyHealthDoCards = [
    "NOW_CARD:",
    `- authority=${scopedPacket ? "workpoint" : safeScope && !needsConfirm ? "operator_current_ask" : "blocked"}; scope=project_root:${projectRoot || "unknown"} continuity_id:${continuityId || "unknown"}`,
    `- readiness=scope:${safeScope && !needsConfirm ? "verified" : "unverified"} workpoint:${workpointStatus} trajectory:${trajectorySet ? "available" : "not_hydrated"}`,
    `- exact_next_action=${!safeScope || needsConfirm ? "focusa_project_identity with explicit project_root before durable writes" : next ? compact(next) : compact(trajectory.active_gap || trajectory.short_term_goal || mission || "refresh trajectory then checkpoint mission")}`,
    "WHY_CARD:",
    `- why=included verified project scope, current operator ask, ${scopedPacket ? "scoped Workpoint" : "trajectory/bootstrap route"}; excluded transcript_tail and cross-project fallback as authority`,
    "- source_authority_order=operator_steering > verified_project_identity > canonical_workpoint > trajectory_projection > traverse/evidence > transcript_tail_never",
    "HEALTH_CARD:",
    `- scope=${safeScope && !needsConfirm ? "verified" : "unverified"}; workpoint=${workpointStatus}; trajectory=${trajectorySet ? "available" : "not_hydrated"}; evidence=${scopedPacket ? "workpoint_refs_available" : "link_after_checkpoint"}; token_pressure=unknown; drift=${trajectoryFallback ? "trajectory_fallback_advisory" : "none_known"}; uiai=unknown`,
    "DO_CARD:",
    `- exact_next_action=${!safeScope || needsConfirm ? "focusa_project_identity" : scopedPacket ? "focusa_workpoint_resume or execute next anchor" : "focusa_trajectory_view then focusa_workpoint_checkpoint"}`,
    `- mutates=${!safeScope || needsConfirm ? "nothing" : scopedPacket ? "only when selected execution tool is called" : "workpoint checkpoint if accepted"}; rollback=checkpoint/resume packet; rehydrate_refs=focusa_workpoint_resume,focusa_trajectory_view,focusa_traverse`,
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
    "Golden route: Orient project/Trajectory/Workpoint; Execute active object + checkpoint; Prove with evidence; Learn via prediction/metacog; Recover with tool_doctor.",
    "Missing active Pi frame fallback: Attentive and awaiting operator direction; keep helping from operator/repo context, then checkpoint/resume once scope is safe.",
    "Focus State tools (scratch/decide/constraint/failure/etc.) are note/decision slots; use them with the project route, not instead of it.",
  ];

  if (mode === "visible" && !scopedPacket) {
    return [
      prefix,
      `Status: ${status}`,
      ...missionPacket,
      ...nowWhyHealthDoCards,
      ...reconciliationEnvelope,
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
    ...nowWhyHealthDoCards,
    ...reconciliationEnvelope,
    scopedPacket
      ? "Project-bound Workpoint: verified project_root + continuity_id match."
      : "Project-bound Workpoint: none verified for this logical session; use trajectory gap + operator ask, then checkpoint; ignore stale carryover.",
    !safeScope || needsConfirm
      ? "Project folder check: confirm the project file folder/container before durable state writes."
      : "Project root: confirmed project file folder/container; trajectory provides the functional route.",
    mission
      ? `Mission: ${mission}`
      : "Mission: use latest operator instruction as seed; bind it to trajectory + Workpoint before long work.",
    next
      ? `Next anchor: ${next}`
      : "Next anchor: call focusa_workpoint_resume with current continuity_id if resuming project work or uncertain.",
    projectRoot
      ? `Project folder: project_root=${projectRoot}${safeScope ? "" : " (broad/unsafe)"}`
      : "Project folder: bind work to the folder containing project files; reject cross-project resume packets.",
    scopedPacket && continuityId
      ? `Continuity: continuity_id=${continuityId}`
      : "Continuity: no Workpoint continuity verified for this Pi session; use resume/checkpoint before trusting same-root state.",
    trajectoryFallback
      ? `Trajectory: prior-project fallback only from continuity=${compact(trajectory.fallback_source_continuity_id, "unknown", 80)}; refresh/define current continuity before durable trajectory writes.`
      : trajectorySet
        ? `Trajectory: high=${compact(trajectory.long_term_goal)}; current=${compact(trajectory.current_state)}; gap=${compact(trajectory.active_gap || trajectory.short_term_goal)}.`
        : "Trajectory: not hydrated in Utility Card memory; run focusa_trajectory_view before durable state writes; if missing/stale, re-bootstrap from project card + ontology + prediction/metacog signals.",
    "",
    ...friendlyQ,
    ...routeHints,
    "- Compaction/model switch/fork/risky continuation: focusa_workpoint_checkpoint before; focusa_workpoint_resume after.",
    "- Continuous/background work: focusa_work_loop_* after writer-status/preflight; focusa_silent_sessions only when explicitly managing background sessions.",
    "Operator steering always wins; Focusa guides, preserves, and audits.",
  ].join("\n");
}
