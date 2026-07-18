import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const tools = readFileSync(fileURLToPath(new URL("../src/tools.ts", import.meta.url)), "utf8");
const commands = readFileSync(fileURLToPath(new URL("../src/commands.ts", import.meta.url)), "utf8");
const turns = readFileSync(fileURLToPath(new URL("../src/turns.ts", import.meta.url)), "utf8");
assert.match(tools, /const workLoopLeases = new Map<string, WorkLoopWriterLease>/);
assert.match(tools, /Number\.isSafeInteger\(fencingToken\)/);
assert.match(tools, /x-focusa-fencing-token/);
assert.match(tools, /focusa\.work_loop_status\.v3/, "tool lease cache must reject unsupported status schemas");
assert.match(tools, /current scoped writer lease is missing, expired, or owned by another writer/);
assert.doesNotMatch(
  tools.match(/async function preferredWriterId[\s\S]*?\n  }/)?.[0] ?? "",
  /active_writer/,
  "Pi writer identity must not adopt another partition owner's writer id",
);
for (const [name, source] of [["commands", commands], ["turns", turns]]) {
  assert.match(source, /focusa\.work_loop_status\.v3/, `${name} must reject unsupported status schemas`);
  assert.match(source, /lease_freshness !== "current"/, `${name} must reject stale leases`);
  assert.match(source, /x-focusa-fencing-token/, `${name} must send fencing authority`);
}
console.log("work-loop scoped lease client contract passed");
