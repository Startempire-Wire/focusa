import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../../..");
const session = fs.readFileSync(path.join(root, "apps/pi-extension/src/session.ts"), "utf8");
const tools = fs.readFileSync(path.join(root, "apps/pi-extension/src/tools.ts"), "utf8");
const turns = fs.readFileSync(path.join(root, "apps/pi-extension/src/turns.ts"), "utf8");
const compaction = fs.readFileSync(path.join(root, "apps/pi-extension/src/compaction.ts"), "utf8");
const northStar = fs.readFileSync(path.join(root, "apps/pi-extension/src/north-star.ts"), "utf8");
const state = fs.readFileSync(path.join(root, "apps/pi-extension/src/state.ts"), "utf8");

function sessionStartBlock() {
  const start = session.indexOf('pi.on("session_start"');
  assert.notEqual(start, -1);
  const end = session.indexOf("// §35.8: Pi owns", start);
  assert.notEqual(end, -1);
  return session.slice(start, end);
}

test("north-star gate refreshes exact-scope trajectory before rendering", () => {
  const start = tools.indexOf('name: "focusa_north_star_gate"');
  const end = tools.indexOf('name: "focusa_scratch"', start);
  assert.ok(start >= 0 && end > start);
  const block = tools.slice(start, end);
  const refresh = block.indexOf("refreshTrajectoryClarityLifecycle");
  const snapshot = block.indexOf("buildNorthStarSnapshot");
  assert.ok(refresh >= 0 && snapshot > refresh);
  assert.match(block, /resolveFocusaToolProjectRoot/);
  assert.match(block, /canonical: false/);
  assert.match(block, /advisory: true/);
});

test("session startup fails closed before durable project initialization", () => {
  const block = sessionStartBlock();
  const verify = block.indexOf("const projectBindingDecisionV1 = await promptForProjectVerifyIfNeeded");
  const blocked = block.indexOf("if (!projectVerified)");
  const durableSession = block.indexOf("await ensureFocusaSession");
  assert.ok(verify >= 0 && blocked > verify && durableSession > blocked);
  assert.match(block, /pi_project_startup_waiting_for_binding/);
  assert.match(block, /ctx\.ui\.setWidget\("focusa-north-star", undefined\)/);
  assert.doesNotMatch(block, /North-star gate blocked durable project startup/);
});

test("explicit trajectory view adopts verified continuity before caching North Star state", () => {
  const start = tools.indexOf('name: "focusa_trajectory_view"');
  const end = tools.indexOf('name: "focusa_hlt_history"', start);
  assert.ok(start >= 0 && end > start);
  const block = tools.slice(start, end);
  const adopt = block.indexOf("adoptVerifiedContinuityForCurrentSession");
  const cache = block.indexOf("setLastTrajectoryClarity", adopt);
  assert.ok(adopt >= 0 && cache > adopt);
  assert.match(block, /scope_verification:/);
  assert.match(block, /trajectory\.scope_verification/);
  assert.match(state, /export function adoptVerifiedContinuityForCurrentSession/);
  assert.match(state, /verifiedRoot !== root/);
});

test("north-star startup order is project then trajectory then Workpoint", () => {
  const block = sessionStartBlock();
  const verify = block.indexOf("promptForProjectVerifyIfNeeded");
  const trajectory = block.indexOf('refreshTrajectoryClarityLifecycle("session_start"');
  const workpoint = block.indexOf('refreshSessionWorkpointPacket("session_start"');
  assert.ok(verify >= 0 && trajectory > verify && workpoint > trajectory);
  assert.match(block, /updateNorthStarCard\(ctx, "session_start_ready_check"\)/);
});

test("degraded project verification is non-modal and never promotes durable authority", () => {
  const start = session.indexOf("async function promptForProjectVerifyIfNeeded");
  const end = session.indexOf("async function promptForWorkpointIfNeeded", start);
  const block = session.slice(start, end);
  assert.match(block, /ProjectBindingDecisionV1/);
  assert.match(block, /shouldEmitProjectScopeRecoveryPacket/);
  assert.match(block, /Conversation and diagnosis continue/);
  assert.doesNotMatch(block, /ctx\.ui\.confirm/);
  assert.match(block, /decision\.state === "BOUND"/);
});

test("Workpoint resume binds current ask and rejects stale action authority", () => {
  const start = session.indexOf("async function refreshSessionWorkpointPacket");
  const end = session.indexOf("async function promptForConfirmedProjectRoot", start);
  const block = session.slice(start, end);
  assert.match(block, /current_ask: currentAsk \|\| undefined/);
  assert.match(block, /action_authority_for_current_ask !== true/);
  assert.match(block, /matches_current_ask_scope === false/);
  assert.match(block, /workpoint_resume_rejected_stale_current_ask/);
  assert.match(block, /candidate\.current_ask_binding = currentAsk/);
});

test("Pi session id remains temporal metadata outside Workpoint identity", () => {
  const start = state.indexOf("export function isWorkpointPacketScopedToCurrentSession");
  const end = state.indexOf("export function getScopedWorkpointPacket", start);
  const block = state.slice(start, end);
  assert.match(block, /project_root \+ continuity_id/);
  assert.doesNotMatch(block, /packetSessionId !== currentSessionKey/);
  assert.doesNotMatch(block, /packetPiSessionKey !== currentSessionKey/);
});

test("first Workpoint checkpoint omits fake writer identity when no lease exists", () => {
  const marker = 'name: "focusa_workpoint_checkpoint"';
  const start = tools.indexOf(marker);
  const end = tools.indexOf('name: "focusa_workpoint_link_evidence"', start);
  const block = tools.slice(start, end);
  assert.match(block, /const checkpointLease = await currentWorkLoopLease\(\)/);
  assert.match(
    block,
    /headers: checkpointLease \? writerLeaseHeaders\(localWriterId, checkpointLease\) : \{\}/
  );
  assert.doesNotMatch(block, /headers: writerLeaseHeaders\(localWriterId, await currentWorkLoopLease\(\)\)/);
});

test("operator ask changes demote stale action authority without false project blocking", () => {
  assert.match(turns, /boundAsk !== newTaskText/);
  assert.match(turns, /action_authority_for_current_ask: false/);
  assert.match(turns, /operator_ask_changed_since_workpoint_binding/);
  assert.match(northStar, /getLastProjectIdentity/);
  assert.match(northStar, /identityVerified/);
  assert.match(northStar, /trigger === "operator_input"/);
  assert.match(northStar, /\("steered" as const\)/);
});

test("north-star card continuously refreshes at lifecycle boundaries", () => {
  assert.match(turns, /updateNorthStarCard\(_ctx, "operator_input"\)/);
  assert.match(turns, /updateNorthStarCard\(ctx, "model_switch"\)/);
  assert.match(compaction, /updateNorthStarCard\(ctx, "post_compaction"\)/);
  assert.match(session, /updateNorthStarCard\(ctx, "session_start_ready_check"\)/);
  assert.match(northStar, /PROJECT .* → HLT/);
  assert.match(northStar, /MLG .* → STG/);
  assert.match(northStar, /WAYPOINT .* → GAP/);
  assert.match(northStar, /WP .* → FRONTIER/);
  assert.match(northStar, /const workpoint = getScopedWorkpointPacket\(\)/);
  assert.doesNotMatch(northStar, /getScopedWorkpointPacket\(\) \|\| getActiveWorkpointPacket\(\)/);
  assert.match(northStar, /focusa_project_identity → focusa_project_verify/);
  assert.match(northStar, /focusa_workpoint_resume → focusa_workpoint_checkpoint/);
});
