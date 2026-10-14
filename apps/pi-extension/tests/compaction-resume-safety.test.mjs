import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const source = readFileSync(
  fileURLToPath(new URL("../src/compaction-resume-safety.ts", import.meta.url)),
  "utf8"
);
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.ESNext,
    target: ts.ScriptTarget.ES2022,
  },
}).outputText;
const { canInjectCompactionMission, safeCompactionRecoveryContext } = await import(
  `data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`
);

const scope = {
  root_scope: { root_path: "/home/wirebot/focusa" },
  continuity_id: "focusa-continuity",
};
const verified = {
  schema_version: "focusa.compaction_mission_packet.v1",
  status: "verified",
  current_ask: { text: "Review only.", text_complete: true },
  scope: {
    scope_status: "verified",
    project_root: "/home/wirebot/focusa",
    continuity_id: "focusa-continuity",
  },
  trajectory: {
    project_root: "/home/wirebot/focusa",
    continuity_id: "focusa-continuity",
    action_authority_from_trajectory: true,
  },
  workpoint: {
    project_root: "/home/wirebot/focusa",
    continuity_id: "focusa-continuity",
    status: "ready",
    action_authority: true,
    mission: "Current exact-scope mission",
    next_slice: "Current exact-scope action",
  },
};

assert.equal(canInjectCompactionMission(verified, scope), true);
for (const candidate of [
  { ...verified, status: "blocked" },
  { ...verified, current_ask: undefined },
  { ...verified, current_ask: { text: "partial", text_complete: false } },
  { ...verified, current_ask: { text: "", text_complete: true } },
  { ...verified, status: "degraded" },
  { ...verified, scope: { ...verified.scope, scope_status: "mismatch" } },
  { ...verified, scope: { ...verified.scope, project_root: "/root" } },
  { ...verified, scope: { ...verified.scope, continuity_id: "foreign" } },
  { ...verified, trajectory: { ...verified.trajectory, action_authority_from_trajectory: false } },
  { ...verified, trajectory: { ...verified.trajectory, project_root: "/srv/foreign" } },
  { ...verified, trajectory: { ...verified.trajectory, continuity_id: "foreign" } },
  { ...verified, workpoint: { ...verified.workpoint, project_root: "/srv/foreign" } },
  { ...verified, workpoint: { ...verified.workpoint, continuity_id: "foreign" } },
  { ...verified, workpoint: { ...verified.workpoint, action_authority: false } },
  { ...verified, workpoint: { ...verified.workpoint, status: "missing" } },
]) {
  assert.equal(canInjectCompactionMission(candidate, scope), false);
}

for (const budget of [900, 600, 400, 250]) {
  const steering = safeCompactionRecoveryContext("Read-only review; no changes.", budget * 4);
  assert.ok(steering.includes("Read-only review; no changes."));
  assert.ok(steering.length <= budget * 4);
  assert.throws(() => safeCompactionRecoveryContext("No changes. ".repeat(1000), budget * 4),
    /mandatory_budget_exceeded/);
}
const recovery = safeCompactionRecoveryContext();
assert.match(recovery, /authority is not verified/);
assert.match(recovery, /Do not rely on a prior mission/);
assert.doesNotMatch(recovery, /Current exact-scope mission|Current exact-scope action|DO_NOT_DRIFT/);

console.log("compaction resume authority safety passed");
