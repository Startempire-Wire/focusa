import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const source = readFileSync(
  fileURLToPath(new URL("../src/compaction-outcome-evaluator.ts", import.meta.url)),
  "utf8"
);
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
}).outputText;
const { evaluateCompactionOutcome } = await import(
  `data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`
);

const baseline = {
  schema: "focusa.compaction_outcome_baseline.v1",
  policyVersion: "1",
  policyKey: "policy-1",
  route: "native_compact",
  snapshot: {
    projectRoot: "/project/a",
    sessionId: "session-a",
    continuityRef: "continuity-a",
    workpointRef: "workpoint-a",
    evidenceRefs: ["evidence:2", "evidence:1", "evidence:1"],
    providerOutcome: "unknown",
    qualityScore: 0.9,
    contextTokens: 100_000,
  },
};

const healthy = evaluateCompactionOutcome(baseline, {
  ...baseline.snapshot,
  providerOutcome: "succeeded",
  qualityScore: 0.88,
  contextTokens: 50_000,
});
assert.equal(healthy.disposition, "promote");
assert.equal(healthy.rollbackRequired, false);
assert.equal(healthy.rollbackRoute, "native_compact");
assert.deepEqual(healthy.reasons, []);
assert.equal(healthy.qualityDelta, -0.02);
assert.equal(healthy.tokenDelta, -50_000);

const unknownQuality = evaluateCompactionOutcome(
  { ...baseline, snapshot: { ...baseline.snapshot, qualityScore: null } },
  {
    ...baseline.snapshot,
    providerOutcome: "succeeded",
    qualityScore: null,
    contextTokens: 40_000,
  }
);
assert.equal(unknownQuality.disposition, "retain");
assert.equal(unknownQuality.rollbackRequired, false);
assert.equal(unknownQuality.qualityDelta, null);

const providerFailure = evaluateCompactionOutcome(baseline, {
  ...baseline.snapshot,
  providerOutcome: "failed",
});
assert.equal(providerFailure.disposition, "quarantine");
assert.equal(providerFailure.rollbackRequired, true);
assert.equal(providerFailure.rollbackRoute, "checkpoint");
assert.deepEqual(providerFailure.reasons, ["provider_failure"]);

const authorityRegression = evaluateCompactionOutcome(baseline, {
  ...baseline.snapshot,
  projectRoot: "/project/b",
  sessionId: "session-b",
  continuityRef: "invented-continuity",
  workpointRef: "invented-workpoint",
  evidenceRefs: ["evidence:2"],
  providerOutcome: "succeeded",
  qualityScore: 0.7,
});
assert.equal(authorityRegression.rollbackRequired, true);
assert.equal(authorityRegression.disposition, "quarantine");
assert.deepEqual(authorityRegression.reasons, [
  "scope_drift",
  "hallucinated_continuity",
  "workpoint_drift",
  "missing_evidence",
  "quality_regression",
]);
assert.deepEqual(authorityRegression.missingEvidenceRefs, ["evidence:1"]);
assert.equal(authorityRegression.rollbackRoute, "checkpoint");

const replay = evaluateCompactionOutcome(baseline, {
  ...baseline.snapshot,
  projectRoot: "/project/b",
  sessionId: "session-b",
  continuityRef: "invented-continuity",
  workpointRef: "invented-workpoint",
  evidenceRefs: ["evidence:2"],
  providerOutcome: "succeeded",
  qualityScore: 0.7,
});
assert.deepEqual(replay, authorityRegression);

console.log("compaction outcome evaluation and deterministic rollback passed");
