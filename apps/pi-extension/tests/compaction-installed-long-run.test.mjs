import assert from "node:assert/strict";
import { cpSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import ts from "typescript";

const sourceRoot = new URL("../src/", import.meta.url);

async function loadInstalled(root, name) {
  const source = readFileSync(join(root, `${name}.ts`), "utf8");
  const compiled = ts.transpileModule(source, {
    compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
  }).outputText;
  return import(`data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`);
}

function telemetry(percent, branchEntryCount = 240) {
  return {
    schema: "focusa.context_pressure_telemetry.v1",
    percent,
    branchEntryCount,
    messageEntryCount: branchEntryCount - 20,
    toolResultCount: 20,
  };
}

function capabilities(providerId, modelId, nativeCompaction) {
  return {
    schema: "focusa.provider_compaction_capabilities.v1",
    providerId,
    modelId,
    contextWindow: 200_000,
    tokenAccounting: "runtime_observed",
    nativeCompaction,
    cacheBehavior: "unknown",
    groundingStatus: "grounded",
    evidenceRefs: ["pi:model.provider", "pi:model.id", "pi:getContextUsage"],
  };
}

function continuation(overrides = {}) {
  return {
    projectRoot: "/srv/focusa-project",
    sessionId: "pi-session-a",
    continuityRef: "focusa-long-run",
    workpointRef: "wp-canonical",
    evidenceRefs: ["evidence:scope", "evidence:workpoint"],
    providerOutcome: "succeeded",
    qualityScore: 0.91,
    contextTokens: 84_000,
    ...overrides,
  };
}

async function runInstalledFixture(root, runId) {
  for (const file of [
    "compaction-policy-selector.ts",
    "compaction-outcome-evaluator.ts",
    "compaction-authority-projection.ts",
  ]) {
    cpSync(new URL(file, sourceRoot), join(root, file));
  }
  const policy = await loadInstalled(root, "compaction-policy-selector");
  const outcome = await loadInstalled(root, "compaction-outcome-evaluator");
  const authority = await loadInstalled(root, "compaction-authority-projection");
  let projection = authority.emptyCompactionAuthorityProjection();
  let rollbacks = 0;
  let promotions = 0;
  let providerSwitches = 0;
  let priorProvider = null;
  let boundedCostUnits = 0;

  for (let index = 1; index <= 600; index += 1) {
    const providerId = index % 23 < 11 ? "provider-a" : "provider-b";
    if (priorProvider && providerId !== priorProvider) providerSwitches += 1;
    priorProvider = providerId;
    const nativeCompaction = index % 97 === 0 ? "unknown" : "supported";
    const percent = index % 89 === 0 ? null : index % 97 === 0 ? 96 : 86;
    const candidate = policy.selectCompactionPolicy(
      telemetry(percent),
      capabilities(providerId, `model-${index % 7}`, nativeCompaction)
    );
    const selected = policy.applyCompactionPolicyQuarantine(
      candidate,
      projection.quarantinedPolicyKeys,
      projection.rollbackRoute
    );
    assert.ok(
      ["no_op", "curate_context", "checkpoint", "summarize", "native_compact", "rollover"].includes(
        selected.route
      )
    );
    if (percent === null) assert.equal(selected.route, "no_op");
    if (nativeCompaction === "unknown" && percent === 96) assert.equal(selected.route, "rollover");

    const sessionId =
      index > 400 ? "pi-session-fork" : index > 200 ? "pi-session-model-switch" : "pi-session-a";
    const before = continuation({ sessionId, contextTokens: percent === null ? null : 170_000 });
    const baseline = {
      schema: "focusa.compaction_outcome_baseline.v1",
      policyVersion: selected.policyVersion,
      policyKey: selected.deterministicKey,
      route: selected.route,
      snapshot: before,
    };
    let after = continuation({ sessionId, contextTokens: 82_000, qualityScore: 0.92 });
    if (index % 101 === 0) after = continuation({ sessionId, providerOutcome: "failed" });
    else if (index % 127 === 0) after = continuation({ sessionId, continuityRef: "foreign" });
    else if (index % 131 === 0) after = continuation({ sessionId, workpointRef: "wp-drift" });
    else if (index % 137 === 0) after = continuation({ sessionId, evidenceRefs: [] });
    else if (index % 139 === 0) after = continuation({ sessionId, qualityScore: 0.5 });
    // Model switch and fork alter adapter/session identity outside canonical
    // project, continuity, Workpoint, and evidence authority.
    const evaluation = outcome.evaluateCompactionOutcome(baseline, after);
    const event = {
      schema: "focusa.auto_compaction_event.v1",
      kind: evaluation.rollbackRequired ? "policy_rollback_required" : "policy_promoted",
      recorded_at: new Date(1_800_000_000_000 + index).toISOString(),
      epoch_id: `${runId}-${index}`,
      outcome_evaluation: evaluation,
      quarantined_policy_key: evaluation.rollbackRequired ? evaluation.policyKey : undefined,
      rollback_route: evaluation.rollbackRoute,
    };
    projection = authority.reduceCompactionAuthorityEvents([event], projection);
    if (evaluation.rollbackRequired) rollbacks += 1;
    else if (evaluation.disposition === "promote") promotions += 1;
    boundedCostUnits += selected.route === "native_compact" || selected.route === "summarize" ? 1 : 0;
    assert.equal(after.projectRoot === before.projectRoot || evaluation.rollbackRequired, true);
  }

  assert.ok(providerSwitches > 20);
  assert.ok(rollbacks > 10);
  assert.ok(promotions > 500);
  assert.ok(projection.quarantinedPolicyKeys.length <= rollbacks);
  assert.ok(boundedCostUnits <= 600);
  assert.equal(projection.recoveryRequired, false);
  const receipt = {
    schema: "focusa.compaction_installed_long_run_receipt.v1",
    run_id: runId,
    epochs: 600,
    provider_switches: providerSwitches,
    rollbacks,
    promotions,
    bounded_cost_units: boundedCostUnits,
    scope: continuation().projectRoot,
    continuity_id: continuation().continuityRef,
    workpoint_id: continuation().workpointRef,
    evidence_refs: continuation().evidenceRefs,
  };
  writeFileSync(join(root, "long-run-receipt.json"), JSON.stringify(receipt, null, 2));
  return receipt;
}

const base = mkdtempSync(join(tmpdir(), "focusa-compaction-installed-"));
try {
  const firstRoot = join(base, "install-a");
  const secondRoot = join(base, "install-b");
  cpSync(new URL("../src/", import.meta.url), firstRoot, { recursive: true });
  cpSync(new URL("../src/", import.meta.url), secondRoot, { recursive: true });
  const first = await runInstalledFixture(firstRoot, "installed-a");
  const second = await runInstalledFixture(secondRoot, "installed-b");
  assert.deepEqual(
    { ...first, run_id: "same" },
    { ...second, run_id: "same" },
    "two installed runs must be deterministic"
  );
  console.log(JSON.stringify({ status: "passed", runs: [first, second] }));
} finally {
  rmSync(base, { recursive: true, force: true });
}
