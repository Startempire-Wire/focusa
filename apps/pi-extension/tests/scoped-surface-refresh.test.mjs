import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import test from "node:test";

const source = (name) => readFileSync(fileURLToPath(new URL(`../src/${name}`, import.meta.url)), "utf8");
const refresh = source("scoped-surface-refresh.ts");
const widget = source("mission-canvas-widget.ts");
const semantic = source("semantic-surface-truth.ts");
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

test("tool blocked text renders structured daemon errors, never [object Object]", () => {
  // #266: the entitlement middleware returns "error" as a structured object;
  // template interpolation used to flatten it into [object Object], hiding
  // the real denial (code/message). safeErrorText must be used everywhere.
  assert.match(tools, /function safeErrorText/);
  assert.match(tools, /safeErrorText\(result\.body\?\.error\)/);
  assert.match(tools, /safeErrorText\(result\.body\?\.reason\)/);
  assert.doesNotMatch(tools, /\$\{result\.body\?\.error/);
  assert.doesNotMatch(tools, /String\(result\.body\?\.reason/);
});

test("Mission Canvas redraws verified receipts without starting a competing poller", () => {
  assert.match(widget, /subscribeScopedStateChanges/);
  assert.match(widget, /scopedReceiptMatchesCurrentScope/);
  assert.match(widget, /runWithAttachmentRuntime\(owner/);
  assert.match(widget, /available && bound \? getScopedWorkpointPacket\(\) : null/);
  assert.doesNotMatch(widget, /setInterval|\/workpoint\/resume|focusaFetch/);
  assert.match(widget, /refreshUnsubscribe\?\.\(\)/);
  // #138: the always-on widget must NOT render the semantic registry summary
  // or operation rows; those details stay behind diagnostics.
  assert.doesNotMatch(widget, /semantic pair/);
  assert.doesNotMatch(widget, /semanticSupported/);
  assert.doesNotMatch(widget, /semanticSurfaceTruth/);
  assert.match(semantic, /operation\.availability \|\| "unknown"/);
  assert.doesNotMatch(widget + semantic, /unsupported on this Pi surface/);
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
