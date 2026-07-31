import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import test from "node:test";

const tools = readFileSync(fileURLToPath(new URL("../src/tools.ts", import.meta.url)), "utf8");
const turns = readFileSync(fileURLToPath(new URL("../src/turns.ts", import.meta.url)), "utf8");
const ecsRoute = readFileSync(
  fileURLToPath(new URL("../../../crates/focusa-api/src/routes/ecs.rs", import.meta.url)),
  "utf8"
);
const coreTypes = readFileSync(
  fileURLToPath(new URL("../../../crates/focusa-core/src/types.rs", import.meta.url)),
  "utf8"
);

function block(source, startToken, endToken) {
  const start = source.indexOf(startToken);
  const end = source.indexOf(endToken, start);
  assert.ok(start >= 0 && end > start, `missing source block ${startToken}`);
  return source.slice(start, end);
}

test("ECS handle trajectory summaries require exact current project/workstream scope", () => {
  const matcherSource = block(
    turns,
    "function handleTrajectoryMatchesCurrentScope",
    "function formatHandleTrajectorySummary"
  );
  const formatter = block(turns, "function handleTrajectoryMatchesCurrentScope", "function safeExists");
  assert.match(formatter, /candidateRoot !== currentRoot/);
  assert.match(formatter, /candidateContinuity !== currentContinuity/);
  assert.match(formatter, /candidateTrajectoryId === activeTrajectoryId/);
  assert.match(formatter, /if \(!handleTrajectoryMatchesCurrentScope\(handle\)\) return ""/);

  const executableMatcher = matcherSource.replace(
    "function handleTrajectoryMatchesCurrentScope(handle: any): boolean",
    "function handleTrajectoryMatchesCurrentScope(handle)"
  );
  const matches = new Function(
    "getSessionCwd",
    "getContinuityId",
    "getLastTrajectoryClarity",
    `${executableMatcher}; return handleTrajectoryMatchesCurrentScope;`
  )(
    () => "/home/wirebot/focusa",
    () => "focusa-v0.9.135-locked-14",
    () => ({ trajectory_id: "trajectory:focusa:canonical" })
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
  assert.equal(matches({ trajectory: scoped }), false, "missing trajectory identity fails closed");

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

test("recent predictions reject any response outside requested typed workstream", () => {
  const recent = block(tools, 'name: "focusa_predict_recent"', 'name: "focusa_predict_evaluate"');
  assert.match(recent, /buildProjectWorkstreamKey\(projectRoot, continuityId\)/);
  assert.match(recent, /isWorkstreamKey\(body\.scope\)/);
  assert.match(recent, /sameWorkstream\(body\.scope, scope\)/);
  assert.match(recent, /response scope differs from requested project\/workstream/);
});
