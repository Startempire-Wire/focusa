import assert from "node:assert/strict";
import { readFile, writeFile, unlink } from "node:fs/promises";
import { fileURLToPath, pathToFileURL } from "node:url";
import ts from "typescript";

const sourceUrl = new URL("../src/north-star.ts", import.meta.url);
const generatedUrl = new URL("./.north-star-runtime.mjs", import.meta.url);
const stateUrl = new URL("./.north-star-state.mjs", import.meta.url);
const stateSource = `
const state = () => globalThis.__focusaNorthStarTestState || {};
export const getActiveWorkpointPacket = () => state().activeWorkpoint || null;
export const getScopedWorkpointPacket = () => state().scopedWorkpoint || null;
export const getLastProjectIdentity = () => state().projectIdentity || null;
export const getLastProjectVerify = () => state().projectVerify || null;
export const getLastTrajectoryClarity = () => state().trajectory || null;
export const getAttachmentRuntime = () => state().runtime || { northStarSnapshot: null };
`;

const source = await readFile(sourceUrl, "utf8");
const compiled = ts
  .transpileModule(source, {
    compilerOptions: {
      target: ts.ScriptTarget.ES2022,
      module: ts.ModuleKind.ES2022,
    },
  })
  .outputText.replaceAll('"./state.js"', '"./.north-star-state.mjs"');
await writeFile(stateUrl, stateSource);
await writeFile(generatedUrl, compiled);

try {
  const { buildNorthStarSnapshot, renderNorthStarCard } = await import(
    `${pathToFileURL(fileURLToPath(generatedUrl)).href}?v=${Date.now()}`
  );
  const trajectory = {
    hlt_status: "canonical_explicit",
    long_term_goal: "Complete the full Focusa MVP",
    mid_level_goal: "Restore the intended locked-release baseline",
    short_term_goal: "Repair current baseline regression",
    waypoints: ["repair"],
    active_gap: "current operator steering",
  };
  const steeredWorkpoint = {
    canonical: true,
    degraded: false,
    workpoint_id: "wp-1",
    work_item_id: "focusa-vbcqu.9.2.4.3",
    action_authority_for_current_ask: false,
    matches_current_ask_scope: false,
    current_ask_scope: { action_authority_for_current_ask: false },
  };

  globalThis.__focusaNorthStarTestState = {
    projectIdentity: { status: "verified", confidence: "high" },
    projectVerify: null,
    trajectory,
    scopedWorkpoint: steeredWorkpoint,
    runtime: { northStarSnapshot: null },
  };
  const operatorSteering = buildNorthStarSnapshot("operator_input");
  assert.equal(operatorSteering.project, "current");
  assert.equal(operatorSteering.workpoint, "steered");
  assert.equal(operatorSteering.status, "stale");
  assert.match(operatorSteering.exact_recovery, /workpoint_resume.*workpoint_checkpoint/);
  assert.match(renderNorthStarCard(operatorSteering)[0], /NORTH STAR STALE/);
  assert.doesNotMatch(renderNorthStarCard(operatorSteering).join("\n"), /NORTH STAR BLOCKED/);

  const backgroundMismatch = buildNorthStarSnapshot("background_refresh");
  assert.equal(backgroundMismatch.project, "current");
  assert.equal(backgroundMismatch.workpoint, "mismatched");
  assert.equal(backgroundMismatch.status, "stale");

  globalThis.__focusaNorthStarTestState.projectIdentity = null;
  const genuinelyUnverified = buildNorthStarSnapshot("operator_input");
  assert.equal(genuinelyUnverified.project, "blocked");
  assert.equal(genuinelyUnverified.status, "blocked");
} finally {
  delete globalThis.__focusaNorthStarTestState;
  await unlink(generatedUrl).catch(() => {});
  await unlink(stateUrl).catch(() => {});
}

console.log("north-star operator steering runtime transition passed");
