import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const sourcePath = fileURLToPath(
  new URL("../src/workpoint-mutation-enforcement.ts", import.meta.url),
);
const source = readFileSync(sourcePath, "utf8");
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
}).outputText;
const { evaluateWorkpointMutation } = await import(
  `data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`
);

const packet = {
  workpoint_id: "01a0-workpoint",
  checkpoint_ref: "checkpoint:01a0",
  project_root: "/project",
  target_objects: ["src/exact.ts", "docs/allowed/"],
  do_not_drift: ["no new documents"],
};

assert.equal(
  evaluateWorkpointMutation({
    toolName: "write",
    toolInput: { path: "outside.ts" },
    packet,
    cwd: "/project",
    env: {},
  }).block,
  false,
  "advisory remains the backward-compatible default",
);

const strict = { FOCUSA_WORKPOINT_MUTATION_ENFORCEMENT: "block" };
const allowed = evaluateWorkpointMutation({
  toolName: "edit",
  toolInput: { path: "src/exact.ts" },
  packet,
  cwd: "/project",
  env: strict,
});
assert.equal(allowed.block, false);
assert.equal(allowed.attemptedPath, "/project/src/exact.ts");

assert.equal(
  evaluateWorkpointMutation({
    toolName: "write",
    toolInput: { path: "docs/allowed/new.md" },
    packet,
    cwd: "/project",
    env: strict,
  }).block,
  false,
  "an explicit directory target admits descendants",
);

const blocked = evaluateWorkpointMutation({
  toolName: "write",
  toolInput: { path: "docs/new-plan.md" },
  packet,
  cwd: "/project",
  env: strict,
});
assert.equal(blocked.block, true);
assert.equal(blocked.workpointId, "01a0-workpoint");
assert.equal(blocked.checkpointRef, "checkpoint:01a0");
assert.match(blocked.reason, /outside/);

assert.equal(
  evaluateWorkpointMutation({
    toolName: "write",
    toolInput: { path: "src/exact.ts" },
    packet: null,
    cwd: "/project",
    env: strict,
  }).block,
  true,
  "strict mode fails closed without canonical Workpoint context",
);

assert.equal(
  evaluateWorkpointMutation({
    toolName: "bash",
    toolInput: { command: "printf ok" },
    packet,
    cwd: "/project",
    env: strict,
  }).applicable,
  false,
  "non-file tools remain outside this bounded interceptor",
);

const turnsSource = readFileSync(
  fileURLToPath(new URL("../src/turns.ts", import.meta.url)),
  "utf8",
);
assert.match(turnsSource, /evaluateWorkpointMutation/);
assert.match(turnsSource, /focusa\.workpoint_mutation_block\.v1/);
assert.match(turnsSource, /\/workpoint\/drift-check/);
assert.match(turnsSource, /emit:\s*true/);
assert.match(turnsSource, /return \{ block: true, reason: JSON\.stringify\(receipt\) \}/);

console.log("workpoint mutation enforcement: PASS");
