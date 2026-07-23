import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import { LifecycleGenerationGuard } from "../src/lifecycle-guard.ts";
import { DaemonRecoveryGate } from "../src/daemon-recovery-gate.ts";

test("shutdown invalidates an in-flight health callback before it can re-arm", async () => {
  const guard = new LifecycleGenerationGuard();
  const token = guard.begin();
  let resolveCheck;
  const check = new Promise((resolve) => {
    resolveCheck = resolve;
  });
  let staleContextReads = 0;
  let rearms = 0;

  const callback = (async () => {
    if (!guard.isCurrent(token)) return;
    await check;
    if (!guard.isCurrent(token)) return;
    staleContextReads += 1;
    if (guard.isCurrent(token)) rearms += 1;
  })();

  guard.end();
  resolveCheck();
  await callback;

  assert.equal(staleContextReads, 0);
  assert.equal(rearms, 0);
});

test("lifecycle guidance cannot trigger or race an operator agent run", () => {
  const sessionSource = readFileSync(new URL("../src/session.ts", import.meta.url), "utf8");
  const turnsSource = readFileSync(new URL("../src/turns.ts", import.meta.url), "utf8");
  assert.doesNotMatch(sessionSource, /sendUserMessage\s*\(/);
  assert.match(sessionSource, /pi_lifecycle_advisory_deferred_to_next_turn/);
  assert.match(sessionSource, /idempotency_key: key/);
  assert.match(turnsSource, /pi_lifecycle_advisory_delivered_in_next_turn/);
  assert.match(turnsSource, /attachCacheSafeFocusSlice/);
});

test("daemon flapping stays in bounded holdover until recovery is stable", () => {
  const source = readFileSync(new URL("../src/session.ts", import.meta.url), "utf8");
  const gateSource = readFileSync(new URL("../src/daemon-recovery-gate.ts", import.meta.url), "utf8");
  assert.match(source, /new DaemonRecoveryGate\(\)/);
  assert.match(gateSource, /recoveryHealthyThreshold = 3/);
  assert.match(gateSource, /outageNoticeCooldownMs = 5 \* 60_000/);
  assert.match(gateSource, /kickstartCooldownMs = 60_000/);
  assert.match(source, /Focusa daemon stably reconnected/);
  assert.doesNotMatch(source, /Focusa daemon kickstarted — session preserved/);

  const gate = new DaemonRecoveryGate();
  assert.equal(gate.observe(false, 1, 0).outage, false);
  const outage = gate.observe(false, 2, 1);
  assert.equal(outage.enteredOutage, true);
  assert.equal(outage.notifyOutage, true);
  assert.equal(outage.kickstart, true);

  const repeated = gate.observe(false, 3, 30_000);
  assert.equal(repeated.notifyOutage, false);
  assert.equal(repeated.kickstart, false);
  assert.equal(gate.observe(true, 0, 31_000).recoveryHealthyChecks, 1);
  assert.equal(gate.observe(true, 0, 32_000).recoveryHealthyChecks, 2);
  assert.equal(gate.observe(false, 1, 33_000).recoveryHealthyChecks, 0);
  gate.observe(true, 0, 34_000);
  gate.observe(true, 0, 35_000);
  const stable = gate.observe(true, 0, 36_000);
  assert.equal(stable.stableRecovered, true);
  assert.equal(stable.outage, false);

  const reflap = gate.observe(false, 2, 40_000);
  assert.equal(reflap.enteredOutage, true);
  assert.equal(reflap.notifyOutage, false);
  assert.equal(reflap.kickstart, false);
  const cooled = gate.observe(false, 3, 301_000);
  assert.equal(cooled.notifyOutage, true);
  assert.equal(cooled.kickstart, true);
});

test("a newer session generation rejects callbacks captured by the old session", () => {
  const guard = new LifecycleGenerationGuard();
  const oldToken = guard.begin();
  guard.end();
  const newToken = guard.begin();
  assert.equal(guard.isCurrent(oldToken), false);
  assert.equal(guard.isCurrent(newToken), true);
});
