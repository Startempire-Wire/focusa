import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import test from "node:test";
import path from "node:path";

const repoRoot = process.env.FOCUSA_REPO_ROOT || "/home/wirebot/focusa";

const tools = readFileSync(fileURLToPath(new URL("../src/tools.ts", import.meta.url)), "utf8");
const turns = readFileSync(fileURLToPath(new URL("../src/turns.ts", import.meta.url)), "utf8");
const state = readFileSync(fileURLToPath(new URL("../src/state.ts", import.meta.url)), "utf8");
const session = readFileSync(fileURLToPath(new URL("../src/session.ts", import.meta.url)), "utf8");
const ecsRoute = readFileSync(
  path.join(repoRoot, "crates/..."),
  "utf8"
);
const coreTypes = readFileSync(
  path.join(repoRoot, "crates/..."),
  "utf8"
);
const ontologyRoute = readFileSync(
  path.join(repoRoot, "crates/..."),
  "utf8"
);

function block(source, startToken, endToken) {
  const start = source.indexOf(startToken);
  const end = source.indexOf(endToken, start);
  assert.ok(start >= 0 && end > start, `missing source block ${startToken}`);
  return source.slice(start, end);
}

test("ECS tool output never injects derived trajectory context", () => {
  const matcherSource = block(
    turns,
    "function currentVerifiedTrajectoryScope",
    "function safeExists"
  );
  assert.match(matcherSource, /candidateRoot !== current\.projectRoot/);
  assert.match(matcherSource, /candidateContinuity !== current\.continuityId/);
  assert.match(matcherSource, /activeRoot === current\.projectRoot/);
  assert.match(matcherSource, /activeContinuity === current\.continuityId/);
  assert.doesNotMatch(matcherSource, /getSessionCwd/);
  assert.doesNotMatch(turns, /TRAJECTORY_CONTEXT:/);
  assert.doesNotMatch(turns, /formatHandleTrajectorySummary/);

  const executableMatcher = matcherSource
    .replace(
      'function currentVerifiedTrajectoryScope(): { projectRoot: string; continuityId: string } | null',
      "function currentVerifiedTrajectoryScope()"
    )
    .replace(
      "function handleTrajectoryMatchesCurrentScope(handle: any): boolean",
      "function handleTrajectoryMatchesCurrentScope(handle)"
    );
  const matches = new Function(
    "getCurrentScopeStore",
    "getLastTrajectoryClarity",
    `${executableMatcher}; return handleTrajectoryMatchesCurrentScope;`
  )(
    () => ({
      identity: {
        scopeKind: "project",
        rootPath: "/home/wirebot/focusa",
        continuityId: "focusa-v0.9.135-locked-14",
      },
    }),
    () => ({
      trajectory_id: "trajectory:focusa:canonical",
      project_root: "/home/wirebot/focusa",
      continuity_id: "focusa-v0.9.135-locked-14",
    })
  );
  const scoped = {
    scope: {
      project_root: "/home/wirebot/focusa",
      continuity_id: "focusa-v0.9.135-locked-14",
    },
  };
  assert.equal(
    matches({ trajectory: { ...scoped, trajectory_id: "trajectory:wire-pitch:foreign" } }),
    false,
    "matching outer scope must not admit a foreign trajectory identity"
  );
  assert.equal(matches({ trajectory: { ...scoped, trajectory_id: "trajectory:focusa:canonical" } }), true);
  assert.equal(
    matches({
      trajectory: {
        scope: { project_root: "/home/wirebot/focusa" },
        trajectory_id: "trajectory:focusa:canonical",
      },
    }),
    false,
    "missing continuity fails closed"
  );
  assert.equal(matches({ trajectory: scoped }), false, "missing trajectory identity fails closed");

  for (const toolName of ["read", "bash"]) {
    assert.equal(
      matches({
        label: `${toolName}-output`,
        trajectory: {
          scope: {
            project_root: "/srv/wire-pitch",
            continuity_id: "focusa-v0.9.135-locked-14",
          },
          trajectory_id: "trajectory:wire-pitch:foreign",
        },
      }),
      false,
      `${toolName} output must reject a foreign project even when continuity matches`
    );
    assert.equal(
      matches({
        label: `${toolName}-output`,
        trajectory: {
          scope: {
            project_root: "/home/wirebot/focusa",
            continuity_id: "rotated-continuity",
          },
          trajectory_id: "trajectory:focusa:canonical",
        },
      }),
      false,
      `${toolName} output must reject a rotated continuity`
    );
  }

  const ecs = block(turns, 'focusaFetch("/ecs/store"', "// §7.4 + §33.3: If Focusa unavailable");
  assert.match(ecs, /project_root: getSessionCwd\(\)/);
  assert.match(ecs, /continuity_id: getContinuityId\(\)/);
});

test("daemon ECS stores attach trajectory only through exact typed scope", () => {
  assert.match(ecsRoute, /trajectory_ladder_context_for_scope/);
  assert.match(coreTypes, /pub fn trajectory_ladder_context_for_scope/);
  assert.match(coreTypes, /context_root != requested_root/);
  assert.match(coreTypes, /requested != actual/);
});

test("trajectory view rejects foreign response scope and foreign cached fallback", () => {
  const trajectory = block(tools, 'name: "focusa_trajectory_view"', 'name: "focusa_hlt_history"');
  assert.match(trajectory, /typedTrajectoryScopeMatches\(body, projectRoot, requestedContinuity\)/);
  assert.match(trajectory, /failure_class: "scope_mismatch"/);
  assert.match(trajectory, /cachedTrajectoryForScope\(projectRoot, requestedContinuity\)/);
  assert.doesNotMatch(trajectory, /\.\.\.\(getLastTrajectoryClarity\(\) \|\| \{\}\)/);
});

test("session restore rejects foreign trajectory ScopeRef and validates same-project fallback", () => {
  const validatorSource = block(
    state,
    "function trajectorySnapshotMatchesStore",
    "/** PI-04: Set lastTrajectoryClarity"
  ).replace(
    "function trajectorySnapshotMatchesStore(snapshot: Record<string, any>, store: TypedScopeStore): boolean",
    "function trajectorySnapshotMatchesStore(snapshot, store)"
  );
  const matches = new Function(
    "normalizeProjectRoot",
    `${validatorSource}; return trajectorySnapshotMatchesStore;`
  )((value) => String(value || "").trim().replace(/\\\/+$/, ""));
  const store = {
    identity: {
      scopeKind: "project",
      scopeId: "project:focusa",
      fingerprint: "fnv1a64:focusa",
      rootPath: "/home/wirebot/focusa",
      continuityId: "focusa-v0.9.135-locked-14",
    },
  };
  const snapshot = {
    trajectory_id: "trajectory:focusa:canonical",
    project_root: "/home/wirebot/focusa",
    continuity_id: "focusa-v0.9.135-locked-14",
    scope_verification: {
      status: "verified_exact",
      rendered_trajectory_id: "trajectory:focusa:canonical",
      source_trajectory_id: "trajectory:focusa:canonical",
      project_root: "/home/wirebot/focusa",
      continuity_id: "focusa-v0.9.135-locked-14",
      scope_ref: {
        scope_kind: "project",
        scope_id: "project:focusa",
        fingerprint: "fnv1a64:focusa",
        root_path: "/home/wirebot/focusa",
      },
    },
    project_identity: {
      status: "verified",
      project_identity_api: {
        scope_ref: {
          scope_kind: "project",
          scope_id: "project:focusa",
          fingerprint: "fnv1a64:focusa",
          root_path: "/home/wirebot/focusa",
        },
      },
    },
  };
  assert.equal(matches(snapshot, store), true);
  assert.equal(
    matches(
      {
        ...snapshot,
        project_identity: {
          ...snapshot.project_identity,
          project_identity_api: {
            scope_ref: {
              scope_kind: "project",
              scope_id: "project:wire-pitch",
              fingerprint: "fnv1a64:wire-pitch",
              root_path: "/srv/wire-pitch",
            },
          },
        },
      },
      store
    ),
    false
  );
  assert.equal(
    matches(
      {
        ...snapshot,
        fallback_prior_project_trajectory: true,
        fallback_source_continuity_id: "focusa-prior-continuity",
        scope_verification: {
          ...snapshot.scope_verification,
          status: "verified_same_project_fallback",
          continuity_id: "focusa-prior-continuity",
        },
      },
      store
    ),
    true,
    "same-project prior-continuity fallback remains available"
  );
  assert.equal(
    matches({ ...snapshot, fallback_prior_project_trajectory: true }, store),
    false,
    "unidentified fallback source fails closed"
  );

  const restore = block(session, "seedCurrentAskFromPersistedState(ctx, d)", "if (d.latestReportSummary?.handle)");
  assert.ok(restore.indexOf("setLastProjectVerify") < restore.indexOf("setLastTrajectoryClarity"));
  assert.match(restore, /setLastTrajectoryClarity\(d\.lastTrajectoryClarity\)/);
  assert.doesNotMatch(restore, /fallback_prior_project_trajectory === true/);
});

test("ontology inner-world cache requires exact typed scope and lifecycle refresh", () => {
  const validatorSource = block(
    state,
    "function ontologyContextMatchesStore",
    "export function getCachedOntologyContext"
  ).replace(
    "function ontologyContextMatchesStore(packet: Record<string, any>, store: TypedScopeStore): boolean",
    "function ontologyContextMatchesStore(packet, store)"
  );
  const matches = new Function(
    "normalizeProjectRoot",
    `${validatorSource}; return ontologyContextMatchesStore;`
  )((value) => String(value || "").trim().replace(/\\\/+$/, ""));
  const store = {
    identity: {
      scopeKind: "project",
      scopeId: "project:focusa",
      fingerprint: "fnv1a64:focusa",
      rootPath: "/home/wirebot/focusa",
      continuityId: "focusa-v0.9.135-locked-14",
    },
  };
  const scopeRef = {
    scope_kind: "project",
    scope_id: "project:focusa",
    fingerprint: "fnv1a64:focusa",
    root_path: "/home/wirebot/focusa",
  };
  const packet = {
    status: "ok",
    stale: false,
    scope: { root_scope: scopeRef, continuity_id: "focusa-v0.9.135-locked-14" },
    scope_verification: {
      status: "verified_exact",
      scope_ref: scopeRef,
      project_root: "/home/wirebot/focusa",
      continuity_id: "focusa-v0.9.135-locked-14",
    },
  };
  assert.equal(matches(packet, store), true);
  assert.equal(
    matches(
      {
        ...packet,
        scope: {
          ...packet.scope,
          root_scope: { ...scopeRef, scope_id: "project:wire-pitch" },
        },
      },
      store
    ),
    false
  );
  assert.match(turns, /const ontologyContext: any = getCachedOntologyContext\(\)/);
  assert.doesNotMatch(turns, /const ontologyContext: any = null/);
  assert.match(state, /focusaFetch\("\/ontology\/context"/);
  assert.match(session, /refreshOntologyContextLifecycle\("session_start"\)/);
  assert.match(ontologyRoute, /scope: Some\(scope\)/);
  assert.match(ontologyRoute, /fn scoped_ontology_state/);
});

test("begin-session ontology trajectory projection is preserved behind exact scope verification", () => {
  const ontologyProjection = block(
    turns,
    "const ontologyEvidenceLines",
    "const ontologyUncertaintyLines"
  );
  assert.match(ontologyProjection, /item\?\.trajectory/);
  assert.match(ontologyProjection, /handleTrajectoryMatchesCurrentScope\(\{ trajectory \}\)/);
  assert.match(ontologyProjection, /verifiedTrajectory\.stg \|\| verifiedTrajectory\.short_term_goal/);
  assert.match(ontologyProjection, /\(STG=/);
});

test("explicit Workpoint resume adopts only the operator-supplied exact typed scope", () => {
  const adoption = block(
    state,
    "export function adoptWorkpointScopeForFrameRecovery",
    "export function getEffectiveFocusSnapshot"
  );
  assert.match(adoption, /expectedScope\?\.allowSessionTransfer === true/);
  assert.match(adoption, /normalizeProjectRoot\(expectedScope\.projectRoot\) === packetProjectRoot/);
  assert.match(adoption, /expectedScope\.continuityId.*=== packetContinuityId/s);
  assert.match(adoption, /currentContinuityId !== packetContinuityId && !explicitScopeMatch/);

  const resume = block(tools, 'name: "focusa_workpoint_resume"', 'name: "focusa_tree_head"');
  assert.match(resume, /projectRoot: params\.project_root/);
  assert.match(resume, /continuityId: params\.continuity_id/);
  assert.match(resume, /allowSessionTransfer: Boolean\(params\.project_root && params\.continuity_id\)/);
});

test("recent predictions reject any response outside requested typed workstream", () => {
  const recent = block(tools, 'name: "focusa_predict_recent"', 'name: "focusa_predict_evaluate"');
  assert.match(recent, /buildProjectWorkstreamKey\(projectRoot, continuityId\)/);
  assert.match(recent, /isWorkstreamKey\(body\.scope\)/);
  assert.match(recent, /sameWorkstream\(body\.scope, scope\)/);
  assert.match(recent, /response scope differs from requested project\/workstream/);
});
