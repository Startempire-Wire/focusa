import assert from "node:assert/strict";
import test from "node:test";
import { readFileSync } from "node:fs";
import { LifecycleGenerationGuard } from "../src/lifecycle-guard.ts";

test("shutdown invalidates an in-flight health callback before it can re-arm", async () => {
  const guard = new LifecycleGenerationGuard();
  const token = guard.begin();
  let resolveCheck;
  const check = new Promise((resolve) => { resolveCheck = resolve; });
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

test("a newer session generation rejects callbacks captured by the old session", () => {
  const guard = new LifecycleGenerationGuard();
  const oldToken = guard.begin();
  guard.end();
  const newToken = guard.begin();
  assert.equal(guard.isCurrent(oldToken), false);
  assert.equal(guard.isCurrent(newToken), true);
});
