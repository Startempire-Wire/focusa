import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const tools = readFileSync(fileURLToPath(new URL("../src/tools.ts", import.meta.url)), "utf8");
const state = readFileSync(fileURLToPath(new URL("../src/state.ts", import.meta.url)), "utf8");
const commands = readFileSync(fileURLToPath(new URL("../src/commands.ts", import.meta.url)), "utf8");
const turns = readFileSync(fileURLToPath(new URL("../src/turns.ts", import.meta.url)), "utf8");
assert.match(tools, /const workLoopLeases = new Map<string, WorkLoopWriterLease>/);
assert.match(tools, /Number\.isSafeInteger\(fencingToken\)/);
assert.match(tools, /x-focusa-fencing-token/);
assert.match(tools, /"x-scope-project-root": attachmentKey\.workstream\.root_scope\.root_path/);
assert.match(tools, /"x-scope-continuity-id": attachmentKey\.workstream\.continuity_id/);
assert.match(
  state,
  /focusa\.work_loop_status\.v3/,
  "shared lease helper must reject unsupported status schemas"
);
assert.match(tools, /current scoped writer lease is missing, expired, or owned by another writer/);
const checkpointTool = tools.slice(
  tools.indexOf('name: "focusa_workpoint_checkpoint"'),
  tools.indexOf('name: "focusa_workpoint_link_evidence"')
);
assert.match(checkpointTool, /currentWorkLoopLease\(\)/);
assert.doesNotMatch(
  checkpointTool,
  /requiredWriterLeaseHeaders\(\)/,
  "first Workpoint checkpoint must not require a pre-existing Work Loop lease"
);
assert.doesNotMatch(
  tools.match(/async function preferredWriterId[\s\S]*?\n  }/)?.[0] ?? "",
  /active_writer/,
  "Pi writer identity must not adopt another partition owner's writer id"
);
assert.match(state, /const attachment = currentAttachmentKey\(\)/);
assert.match(state, /compatibleWorkLoopStatusState/);
for (const typedState of [
  "absent",
  "unavailable",
  "stale",
  "unsupported",
  "blocked",
  "exhausted",
  "zero",
  "healthy",
]) {
  assert.match(state, new RegExp(`"${typedState}"`), `state helper must preserve ${typedState}`);
}
assert.match(state, /isProjectRootAuthoritySafe\(root\)/);
assert.match(state, /"X-Scope-Project-Root": root/);
assert.match(state, /"X-Scope-Continuity-Id": continuity/);
assert.match(state, /continuity !== "extension-bootstrap"/);
assert.match(
  tools,
  /compatibleWorkLoopStatusState\(body\)/,
  "tool lease cache must reject unknown typed states"
);
for (const [name, source] of [
  ["commands", commands],
  ["turns", turns],
]) {
  assert.match(source, /compatibleWorkLoopStatusState\(status\)/, `${name} must reject unknown typed states`);
  assert.match(state, /focusa\.work_loop_status\.v3/, `${name} must reject unsupported status schemas`);
  assert.match(source, /lease_freshness !== "current"/, `${name} must reject stale leases`);
  assert.match(source, /x-focusa-fencing-token/, `${name} must send fencing authority`);
}
console.log("work-loop scoped lease client contract passed");
