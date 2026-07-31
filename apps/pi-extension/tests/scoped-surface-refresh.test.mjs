import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import test from "node:test";

const source = (name) => readFileSync(fileURLToPath(new URL(`../src/${name}`, import.meta.url)), "utf8");
const refresh = source("scoped-surface-refresh.ts");
const widget = source("mission-canvas-widget.ts");
const session = source("session.ts");
const tools = source("tools.ts");
const rail = source("work-rail-widget.ts");
const index = source("index.ts");

test("successful mutations publish exact-scope refresh receipts", () => {
  assert.match(refresh, /focusa\.scoped_state_change_receipt\.v1/);
  assert.match(refresh, /project_root: string/);
  assert.match(refresh, /continuity_id: string/);
  assert.match(refresh, /scopedReceiptMatchesCurrentScope/);
  assert.match(tools, /publishScopedStateChange/);
  assert.match(tools, /responseRoot === requestedRoot/);
  assert.match(tools, /responseContinuity === requestedContinuity/);
});

test("Mission Canvas subscribes independently and polls only when stale", () => {
  assert.match(widget, /subscribeScopedStateChanges/);
  assert.match(widget, /scopedReceiptMatchesCurrentScope/);
  assert.match(widget, /setInterval/);
  assert.match(widget, /age >= 60_000/);
  assert.match(widget, /\/trajectory\/view/);
  assert.match(widget, /\/workpoint\/resume/);
  assert.match(widget, /truthfulStatusLines/);
  assert.match(widget, /startup_cwd/);
  assert.match(widget, /last_refresh_status/);
});

test("SSE refresh rejects foreign scope and survives reconnect through polling", () => {
  assert.match(session, /scopedRefreshEvents/);
  assert.match(session, /eventRoot === currentRoot/);
  assert.match(session, /eventContinuity === currentContinuity/);
  assert.match(session, /publishScopedStateChange/);
  assert.match(session, /connectSSE\(\)/);
});

test("zero proof is truthful and never rendered as success", () => {
  assert.match(rail, /proof missing/);
  assert.match(rail, /snapshot\.proofCount > 0/);
  assert.doesNotMatch(rail, /const proof = ascii \? "proof" : "✓"/);
  assert.match(index, /proof > 0 \? `proof:\$\{proof\}` : "proof:missing"/);
  assert.match(refresh, /proof: "missing" \| "linked" \| "verified"/);
});
