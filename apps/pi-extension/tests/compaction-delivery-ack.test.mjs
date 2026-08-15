// Compaction resume-packet delivery acknowledgment (#262, delivery ack).
// The harness can only confirm receipt via the next agent turn consuming the
// queued nextTurn message; the ack must be same-session-scoped and must never
// fire for a delivery that was superseded, failed, or already acknowledged.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import { compactionDeliveryAckEligible } from "../src/compaction.ts";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "../../..");
const extensionSource = fs.readFileSync(path.join(root, "apps/pi-extension/src/compaction.ts"), "utf8");

const KEY = "compaction_resume:epoch-1:session-frame-abc";

test("pending deliveries are acknowledged at the next agent turn in the same session", () => {
  assert.equal(compactionDeliveryAckEligible(KEY, "unknown_completion", "session-frame-abc"), true);
  assert.equal(compactionDeliveryAckEligible(KEY, "deferred_to_next_turn", "session-frame-abc"), true);
});

test("already-settled deliveries are never re-acknowledged", () => {
  for (const state of ["pending", "delivered", "superseded_by_operator", "failed"]) {
    assert.equal(compactionDeliveryAckEligible(KEY, state, "session-frame-abc"), false, state);
  }
  assert.equal(compactionDeliveryAckEligible(KEY, undefined, "session-frame-abc"), false);
  assert.equal(compactionDeliveryAckEligible(undefined, "unknown_completion", "session-frame-abc"), false);
});

test("a delivery queued by another session is never acknowledged by this one", () => {
  assert.equal(compactionDeliveryAckEligible(KEY, "unknown_completion", "session-frame-xyz"), false);
  assert.equal(compactionDeliveryAckEligible(KEY, "deferred_to_next_turn", ""), false);
});

test("the no-frame-key fallback suffix still matches the queue key shape", () => {
  const noFrameKey = "compaction_resume:epoch-1:session";
  assert.equal(compactionDeliveryAckEligible(noFrameKey, "unknown_completion", ""), true);
});

test("agent_start acknowledges via a durable entry and persist", () => {
  const handlerStart = extensionSource.indexOf('pi.on("agent_start", (_event, _ctx) => {');
  assert.notEqual(handlerStart, -1, "agent_start ack handler must be registered");
  const handlerEnd = extensionSource.indexOf("});", handlerStart);
  const handler = extensionSource.slice(handlerStart, handlerEnd);
  assert.match(handler, /compactionDeliveryAckEligible\(/);
  assert.match(handler, /compactResumeDeliveryState = "delivered"/);
  assert.match(handler, /persistState\(\)/);
  assert.match(handler, /focusa-compaction-delivery-acknowledged/);
  assert.match(handler, /acknowledged_at/);
});
