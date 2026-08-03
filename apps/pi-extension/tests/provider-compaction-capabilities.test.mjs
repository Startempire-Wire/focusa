import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const source = readFileSync(
  fileURLToPath(new URL("../src/provider-compaction-capabilities.ts", import.meta.url)),
  "utf8"
);
const compiled = ts.transpileModule(source, {
  compilerOptions: {
    module: ts.ModuleKind.ESNext,
    target: ts.ScriptTarget.ES2022,
  },
}).outputText;
const { providerCompactionCapabilities } = await import(
  `data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`
);

const grounded = providerCompactionCapabilities({
  model: {
    provider: "fixture-provider",
    id: "fixture-model",
    contextWindow: 200_000,
    cacheBehavior: "explicit",
  },
  getContextUsage() {},
  compact() {},
});
assert.deepEqual(grounded, {
  schema: "focusa.provider_compaction_capabilities.v1",
  providerId: "fixture-provider",
  modelId: "fixture-model",
  contextWindow: 200_000,
  tokenAccounting: "runtime_observed",
  nativeCompaction: "supported",
  cacheBehavior: "explicit",
  groundingStatus: "grounded",
  evidenceRefs: [
    "pi:model.provider",
    "pi:model.id",
    "pi:model.contextWindow",
    "pi:getContextUsage",
    "pi:context.compact",
    "pi:model.cacheBehavior",
  ],
});

const unknown = providerCompactionCapabilities({ model: { provider: "named-only" } });
assert.equal(unknown.providerId, "named-only");
assert.equal(unknown.contextWindow, null);
assert.equal(unknown.tokenAccounting, "unknown");
assert.equal(unknown.nativeCompaction, "unknown");
assert.equal(unknown.cacheBehavior, "unknown");
assert.equal(unknown.groundingStatus, "partial");
assert.doesNotMatch(JSON.stringify(unknown), /supported/);

const unverified = providerCompactionCapabilities(null);
assert.equal(unverified.groundingStatus, "unverified");
assert.deepEqual(unverified.evidenceRefs, []);

console.log("provider compaction capability inventory passed");
