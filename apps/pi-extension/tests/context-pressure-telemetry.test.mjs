import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const source = readFileSync(
  fileURLToPath(new URL("../src/context-pressure-telemetry.ts", import.meta.url)),
  "utf8"
);
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.ESNext,
    target: ts.ScriptTarget.ES2022,
  },
}).outputText;
const { contextPressureTelemetry } = await import(
  `data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`
);

const capabilities = {
  contextWindow: 200_000,
  cacheBehavior: "unknown",
};
const telemetry = contextPressureTelemetry(
  {
    getContextUsage: () => ({ tokens: 100_000, contextWindow: 200_000 }),
    sessionManager: {
      getBranch: () => [
        { type: "message", message: { role: "user", content: "private prompt" } },
        { type: "message", message: { role: "toolResult", content: "private output" } },
        { type: "compaction", summary: "private summary" },
      ],
    },
  },
  capabilities
);
assert.equal(telemetry.tokens, 100_000);
assert.equal(telemetry.contextWindow, 200_000);
assert.equal(telemetry.percent, 50);
assert.equal(telemetry.branchEntryCount, 3);
assert.equal(telemetry.messageEntryCount, 2);
assert.equal(telemetry.toolResultCount, 1);
assert.equal(telemetry.priorCompactionCount, 1);
assert.equal(telemetry.contentIncluded, false);
assert.equal(telemetry.groundingStatus, "grounded");
assert.doesNotMatch(JSON.stringify(telemetry), /private prompt|private output|private summary/);

const unknown = contextPressureTelemetry({}, capabilities);
assert.equal(unknown.tokens, null);
assert.equal(unknown.contextWindow, 200_000);
assert.equal(unknown.percent, null);
assert.equal(unknown.tokenSource, "unknown");
assert.equal(unknown.groundingStatus, "partial");

console.log("context pressure telemetry passed");
