import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const source = readFileSync(
  fileURLToPath(new URL("../src/compaction-policy-selector.ts", import.meta.url)),
  "utf8"
);
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
}).outputText;
const { selectCompactionPolicy } = await import(
  `data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`
);

const telemetry = (percent, messages = 10, tools = 0) => ({
  percent,
  branchEntryCount: 20,
  messageEntryCount: messages,
  toolResultCount: tools,
});
const capabilities = (nativeCompaction) => ({ nativeCompaction });

assert.equal(selectCompactionPolicy(telemetry(null), capabilities("unknown")).route, "no_op");
assert.equal(selectCompactionPolicy(telemetry(69.99), capabilities("supported")).route, "no_op");
assert.equal(selectCompactionPolicy(telemetry(75, 10, 4), capabilities("supported")).route, "curate_context");
assert.equal(selectCompactionPolicy(telemetry(78), capabilities("supported")).route, "checkpoint");
assert.equal(selectCompactionPolicy(telemetry(86), capabilities("supported")).route, "summarize");
assert.equal(selectCompactionPolicy(telemetry(96), capabilities("supported")).route, "native_compact");
assert.equal(selectCompactionPolicy(telemetry(96), capabilities("unknown")).route, "rollover");

const first = selectCompactionPolicy(telemetry(96), capabilities("supported"));
const second = selectCompactionPolicy(telemetry(96), capabilities("supported"));
assert.deepEqual(first, second);
assert.equal(first.executionOwner, "pi");
assert.match(first.deterministicKey, /^v1:/);

console.log("deterministic compaction policy selector passed");
