import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const source = readFileSync(
  fileURLToPath(new URL("../src/compaction-authority-projection.ts", import.meta.url)),
  "utf8"
);
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
}).outputText;
const { reduceCompactionAuthorityEvents } = await import(
  `data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`
);

const events = [
  null,
  { schema: "foreign", kind: "failed" },
  {
    schema: "focusa.auto_compaction_event.v1",
    kind: "pressure_observed",
    coordinator_state: "observing",
    pressure_telemetry: { tokens: 80_000, contentIncluded: false },
    policy_selection: { route: "checkpoint" },
  },
  {
    schema: "focusa.auto_compaction_event.v1",
    kind: "native_compaction_failed",
    epoch_id: "epoch-1",
    coordinator_state: "native_compaction_failed",
  },
  {
    schema: "focusa.auto_compaction_event.v1",
    kind: "projection_rehydrated",
    epoch_id: "epoch-1",
    coordinator_state: "verified",
  },
  {
    schema: "focusa.auto_compaction_event.v1",
    kind: "policy_rollback_required",
    epoch_id: "epoch-1",
    coordinator_state: "verified",
    outcome_evaluation: {
      schema: "focusa.compaction_outcome_evaluation.v1",
      policyKey: "policy-bad",
      rollbackRequired: true,
      rollbackRoute: "checkpoint",
    },
  },
];
const first = reduceCompactionAuthorityEvents(events);
const replay = reduceCompactionAuthorityEvents(events);
assert.deepEqual(first, replay);
assert.equal(first.eventCount, 4);
assert.equal(first.lastKind, "policy_rollback_required");
assert.equal(first.lastEpochId, "epoch-1");
assert.equal(first.coordinatorState, "verified");
assert.equal(first.recoveryRequired, true);
assert.deepEqual(first.quarantinedPolicyKeys, ["policy-bad"]);
assert.equal(first.rollbackRoute, "checkpoint");
assert.equal(first.lastOutcomeEvaluation.policyKey, "policy-bad");
assert.deepEqual(first.lastPolicySelection, { route: "checkpoint" });
assert.deepEqual(first.lastPressureTelemetry, { tokens: 80_000, contentIncluded: false });

const reordered = reduceCompactionAuthorityEvents([events[3], events[2]]);
assert.equal(reordered.lastKind, "pressure_observed");
assert.equal(reordered.recoveryRequired, true);
assert.notDeepEqual(first, reordered);

console.log("compaction authority projection replay passed");
