import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const source = readFileSync(
  fileURLToPath(new URL("../src/semantic-surface-truth.ts", import.meta.url)),
  "utf8"
);
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.ESNext,
    target: ts.ScriptTarget.ES2022,
  },
}).outputText;
const { semanticSurfaceTruth } = await import(
  `data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`
);

const operations = Array.from({ length: 43 }, (_, index) => ({
  operation_id: `semantic.integrity.operation.${index + 1}`,
  kind: index < 20 ? "mutation" : "read",
  availability: index < 20 ? "supported" : "schema_only",
}));
const degraded = semanticSurfaceTruth(
  {
    state: "supported",
    degraded: true,
    data: {
      registered_operations: 43,
      supported_operations: 20,
      schema_only_operations: 23,
    },
  },
  { items: operations, degraded: true }
);
assert.equal(degraded.state, "degraded");
assert.equal(degraded.operationCount, 43);
assert.equal(degraded.mutationCount, 20);
assert.equal(degraded.supportedCount, 20);
assert.equal(degraded.schemaOnlyCount, 23);
assert.equal(degraded.operationLines.length, 7);
assert.match(degraded.operationLines[0], /schema_only/);
assert.match(degraded.operationLines.at(-1), /17 more gaps/);
assert.match(degraded.operationLines.at(-1), /semantic-integrity registry/);
assert.doesNotMatch(degraded.operationLines.join("\n"), /unsupported on this Pi surface/);

const complete = semanticSurfaceTruth(
  {
    state: "supported",
    degraded: false,
    data: {
      registered_operations: 2,
      supported_operations: 2,
      schema_only_operations: 0,
    },
  },
  {
    items: [
      { operation_id: "semantic.integrity.status", kind: "read", availability: "supported" },
      { operation_id: "semantic.integrity.validate", kind: "mutation", availability: "supported" },
    ],
    degraded: false,
  }
);
assert.equal(complete.state, "supported");
assert.equal(complete.schemaOnlyCount, 0);
assert.deepEqual(complete.operationLines, []);

console.log("semantic surface truth runtime projection passed");
