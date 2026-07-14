import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const commands = readFileSync(new URL("../src/commands.ts", import.meta.url), "utf8");
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
  "await ctx.newSession",
  "parentSession: sourceSessionId",
  "setup: async",
  "appendMessage",
  'focusaFetch("/workpoint/resume"',
  'action: "verify_target",',
  'target_workpoint_id:',
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

console.log("spec130 rollover command lifecycle static mock passed");
