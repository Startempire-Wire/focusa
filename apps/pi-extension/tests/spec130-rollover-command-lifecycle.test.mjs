import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const commands = readFileSync(new URL("../src/commands.ts", import.meta.url), "utf8");
const session = readFileSync(new URL("../src/session.ts", import.meta.url), "utf8");
const start = commands.indexOf('pi.registerCommand("focusa-rollover"');
const end = commands.indexOf('pi.registerCommand("focusa-status"', start);
assert(start > 0 && end > start, "focusa-rollover command block must exist");
const block = commands.slice(start, end);

const ordered = [
  "await ctx.waitForIdle()",
  "prepareCompactionRollover()",
  "migrateNativeSessionBounded",
  'action: "rollover",',
  "seal_source: true",
  'focusaFetch("/workpoint/rollover/target-materialize"',
  "source_continuity_id: scope.continuity_id",
  "target_continuity_id: targetScope.continuity_id",
  "project_root: scope.root_scope.root_path",
  "checkpoint_ref: checkpointRef",
  "workpoint_packet_ref: workpointPacketRef",
  "compaction_packet_ref: compactionPacketRef",
  "const newSessionResult = await ctx.newSession({",
  "parentSession: sourceSessionId",
  "setup: async",
  "appendMessage",
  "withSession: async (replacementCtx)",
  'focusaFetch("/workpoint/resume"',
  'action: "verify_target",',
  "target_workpoint_id:",
];
let cursor = 0;
for (const token of ordered) {
  const index = block.indexOf(token, cursor);
  assert(index >= cursor, `rollover lifecycle missing or out of order: ${token}`);
  cursor = index + token.length;
}

for (const token of [
  "source_scope: scope",
  "target_scope: targetScope",
  "target_continuity_id: targetScope.continuity_id",
  "source_session_id: sourceSessionId",
  "target_session_id: targetSessionId",
  "checkpoint_ref: checkpointRef",
  "workpoint_packet_ref: workpointPacketRef",
  "compaction_packet_ref: compactionPacketRef",
  'rollover_action: "migrate"',
]) {
  assert(block.includes(token), `rollover transfer missing ${token}`);
}

assert(!/as\s+any\)\.newSession|as\s+any\)\.waitForIdle/.test(block), "must not unsafe-cast command context");
assert(
  !/fingerprint.*continuity|continuity.*fingerprint/i.test(block),
  "must not derive continuity from fingerprint"
);

const materializeStart = block.indexOf('focusaFetch("/workpoint/rollover/target-materialize"');
const materializeEnd = block.indexOf('focusaFetch("/workpoint/resume"', materializeStart);
assert(
  materializeStart >= 0 && materializeEnd > materializeStart,
  "materialize request and resume request order must be defined"
);
assert(
  !/canonical:\s*true/.test(block.slice(materializeStart, materializeEnd)),
  "client must not assert canonical during target materialize request"
);
const replacementStart = block.indexOf("withSession: async (replacementCtx)");
assert(
  replacementStart > block.indexOf("await newSessionWithReplacement") &&
    block.indexOf('focusaFetch("/workpoint/resume"', replacementStart) > replacementStart,
  "post-switch resume verification must run in the replacement-session context"
);
assert(
  block.indexOf("replacementCtx.ui.notify", replacementStart) > replacementStart,
  "post-switch UI must use replacementCtx instead of stale command ctx"
);
assert(block.includes("execute [output-dir]"), "Tier-B execute command should permit a safe default target");
assert(
  block.includes('mode === "execute"') && block.includes("`focusa-rollover-${Date.now()}`"),
  "execute without a path should use a unique target beside the private source session"
);
assert(
  !block.includes("provide an explicit output directory"),
  "exact hard-pressure action must not fail for an omitted optional target"
);

const pressureStart = session.indexOf("function refreshNativeSessionPressure");
const pressureEnd = session.indexOf("function markerExistsAtCwd", pressureStart);
assert(pressureStart > 0 && pressureEnd > pressureStart, "native pressure refresh block must exist");
const pressureBlock = session.slice(pressureStart, pressureEnd);
assert(
  session.includes('pressure.recommended_action === "rollover" ? "/focusa-rollover execute" : null'),
  "hard-pressure policy must map to the exact Tier-B command"
);
assert(
  pressureBlock.includes("Live /compact cannot shrink this append-only segment."),
  "hard-pressure copy must distinguish prompt compaction from native rollover"
);
assert(
  pressureBlock.includes("checkpoint, seal, migrate, open a new session, and verify resume"),
  "hard-pressure copy must state the truthful rollover lifecycle"
);
assert(!pressureBlock.includes("newSession"), "session hooks must not replace native sessions directly");

console.log("spec130 rollover command lifecycle static mock passed");
