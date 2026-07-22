import assert from "node:assert/strict";
import test from "node:test";
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

test("a newer session generation rejects callbacks captured by the old session", () => {
  const guard = new LifecycleGenerationGuard();
  const oldToken = guard.begin();
  guard.end();
  const newToken = guard.begin();
  assert.equal(guard.isCurrent(oldToken), false);
  assert.equal(guard.isCurrent(newToken), true);
});
