#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PI_EXT="$ROOT/apps/pi-extension"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/focusa-spec130a-runtime.XXXXXX")"
trap 'rm -rf "$TMP"' EXIT

cd "$PI_EXT"
if [[ -x /opt/cpanel/ea-nodejs20/bin/npx ]]; then
  NPX=/opt/cpanel/ea-nodejs20/bin/npx
else
  NPX="$(command -v npx)"
fi
"$NPX" --no-install tsc -p tsconfig.json --noEmit false --outDir "$TMP/build"
ln -s "$PI_EXT/node_modules" "$TMP/build/node_modules"

cat >"$TMP/runtime.mjs" <<'EOF'
import assert from "node:assert/strict";
import {
  DEFAULT_PROACTIVE_COMPACTION_POLICY,
  evaluateProactiveCompactionEligibility,
  proactiveCompactionDecision,
  registerAutoCompaction,
} from "./build/auto-compaction.js";
import { resetCompactionLeaseForTest } from "./build/auto-compaction.js";
const duplicateModule = await import("./build/auto-compaction.js?duplicate-install");
const thirdModule = await import("./build/auto-compaction.js?third-install");

const usage = { tokens: 190_000, contextWindow: 200_000, percent: 95 };
const message = (id, chars) => ({
  type: "message",
  id,
  parentId: null,
  timestamp: 1,
  message: { role: "user", content: "x".repeat(chars), timestamp: 1 },
});
const largeBranch = [
  message("a", 100_000),
  message("b", 100_000),
  message("c", 100_000),
  message("d", 100_000),
];

function harness(
  branch,
  policy = DEFAULT_PROACTIVE_COMPACTION_POLICY,
  register = registerAutoCompaction,
) {
  const handlers = new Map();
  const events = [];
  const notices = [];
  const statuses = [];
  const compactCalls = [];
  const sentMessages = [];
  const pi = {
    on(name, handler) {
      handlers.set(name, handler);
    },
    registerCommand(_command, _handler) {
      /* extension command registration is exercised by the command-hierarchy tests */
    },
    appendEntry(type, data) {
      events.push({ type, data });
    },
    sendUserMessage(message, options) {
      sentMessages.push({ message, options });
    },
  };
  const ctx = {
    cwd: "/tmp/spec130a-project",
    hasUI: true,
    ui: {
      notify(text, level) {
        notices.push({ text, level });
      },
      setStatus(key, text) {
        statuses.push({ key, text });
      },
    },
    isIdle: () => true,
    hasPendingMessages: () => false,
    getContextUsage: () => usage,
    sessionManager: {
      getBranch: () => branch,
      getSessionId: () => "spec130a-session",
    },
    compact(options) {
      compactCalls.push(options);
    },
  };
  register(pi, () => policy);
  return { handlers, events, notices, statuses, compactCalls, sentMessages, ctx };
}

const empty = evaluateProactiveCompactionEligibility([], usage.contextWindow);
assert.equal(empty.reason, "empty_session");
assert.equal(empty.terminal, true);

const eligible = evaluateProactiveCompactionEligibility(largeBranch, usage.contextWindow);
assert.equal(eligible.eligible, true);
assert.ok(eligible.estimatedNetSavingsTokens > 0);
assert.equal(
  proactiveCompactionDecision(usage, DEFAULT_PROACTIVE_COMPACTION_POLICY).trigger,
  true,
);

const completed = harness(largeBranch);
const duplicateWarnings = [];
const originalWarn = console.warn;
const originalInfo = console.info;
const captureDiagnostic = (...args) => duplicateWarnings.push(args.map(String).join(" "));
console.warn = captureDiagnostic;
console.info = captureDiagnostic;
const duplicate = harness(
  largeBranch,
  DEFAULT_PROACTIVE_COMPACTION_POLICY,
  duplicateModule.registerAutoCompaction,
);
const third = harness(
  largeBranch,
  DEFAULT_PROACTIVE_COMPACTION_POLICY,
  thirdModule.registerAutoCompaction,
);
console.warn = originalWarn;
console.info = originalInfo;
assert.equal(duplicate.handlers.size, 0, "duplicate extension must not register any handlers");
assert.equal(third.handlers.size, 0, "every additional extension must register no handlers");
assert.equal(duplicateWarnings.length, 1, "duplicates must emit one bounded diagnostic");
assert.match(duplicateWarnings[0], /compaction coordinator retained across session replacement/);
await Promise.all([
  completed.handlers.get("agent_settled")({ type: "agent_settled" }, completed.ctx),
  completed.handlers.get("agent_settled")({ type: "agent_settled" }, completed.ctx),
]);
assert.equal(completed.compactCalls.length, 1, "concurrent settlement must start one native call");
const exactPass = await completed.handlers.get("session_before_compact")(
  {
    preparation: {
      messagesToSummarize: largeBranch.slice(0, 3).map((entry) => entry.message),
      turnPrefixMessages: [],
      tokensBefore: usage.tokens,
      settings: { reserveTokens: 16_384, keepRecentTokens: 20_000 },
    },
  },
  completed.ctx,
);
assert.equal(exactPass, undefined);
await completed.handlers.get("session_compact")({ type: "session_compact" }, completed.ctx);
assert.equal(completed.compactCalls.length, 1, "native/manual event must not release active epoch");
completed.compactCalls[0].onComplete({
  summary: "bounded summary",
  firstKeptEntryId: "d",
  tokensBefore: usage.tokens,
});
assert.deepEqual(
  completed.events.map((entry) => entry.data.kind),
  [
    "pressure_observed",
    "native_compaction_requested",
    "attempt_started",
    "outcome_baseline_recorded",
    "outcome_evaluated",
    "attempt_completed",
  ],
);
assert.ok(completed.statuses.some((entry) => entry.text === undefined));
await completed.handlers.get("session_shutdown")({ type: "session_shutdown" }, completed.ctx);

resetCompactionLeaseForTest();
const rejected = harness(largeBranch);
await rejected.handlers.get("agent_settled")({ type: "agent_settled" }, rejected.ctx);
const exactReject = await rejected.handlers.get("session_before_compact")(
  {
    preparation: {
      messagesToSummarize: [message("tiny", 100).message],
      turnPrefixMessages: [],
      tokensBefore: usage.tokens,
      settings: { reserveTokens: 16_384, keepRecentTokens: 20_000 },
    },
  },
  rejected.ctx,
);
assert.deepEqual(exactReject, { cancel: true });
rejected.compactCalls[0].onError(new Error("Compaction cancelled"));
assert.ok(rejected.events.some((entry) => entry.data.kind === "eligibility_rejected"));
assert.ok(!rejected.events.some((entry) => entry.data.kind === "retry_scheduled"));
await rejected.handlers.get("session_shutdown")({ type: "session_shutdown" }, rejected.ctx);

resetCompactionLeaseForTest();
const terminal = harness([]);
await terminal.handlers.get("agent_settled")({ type: "agent_settled" }, terminal.ctx);
await terminal.handlers.get("agent_settled")({ type: "agent_settled" }, terminal.ctx);
assert.equal(terminal.compactCalls.length, 0);
assert.equal(
  terminal.events.filter((entry) => entry.data.kind === "preflight_rejected").length,
  1,
);
assert.equal(terminal.notices.length, 1);
await terminal.handlers.get("session_shutdown")({ type: "session_shutdown" }, terminal.ctx);

resetCompactionLeaseForTest();
const nativeAutomatic = harness(largeBranch);
const nativeAutomaticReject = await nativeAutomatic.handlers.get("session_before_compact")(
  {
    type: "session_before_compact",
    reason: "threshold",
    preparation: {
      messagesToSummarize: [message("tiny-native", 100).message],
      turnPrefixMessages: [],
      tokensBefore: usage.tokens,
      settings: { reserveTokens: 16_384, keepRecentTokens: 20_000 },
    },
  },
  nativeAutomatic.ctx,
);
assert.equal(nativeAutomaticReject, undefined, "native threshold/overflow recovery is never vetoed");
assert.equal(nativeAutomatic.compactCalls.length, 0);
assert.ok(
  nativeAutomatic.events.some((entry) => entry.data.kind === "native_invocation_observed"),
);
assert.ok(
  nativeAutomatic.events.some((entry) => entry.data.kind === "native_eligibility_observed"),
);
await nativeAutomatic.handlers.get("session_shutdown")(
  { type: "session_shutdown" },
  nativeAutomatic.ctx,
);

resetCompactionLeaseForTest();
const nativeManual = harness(largeBranch);
const nativeManualResult = await nativeManual.handlers.get("session_before_compact")(
  {
    type: "session_before_compact",
    reason: "manual",
    preparation: {
      messagesToSummarize: [message("tiny-manual", 100).message],
      turnPrefixMessages: [],
      tokensBefore: usage.tokens,
      settings: { reserveTokens: 16_384, keepRecentTokens: 20_000 },
    },
  },
  nativeManual.ctx,
);
assert.equal(nativeManualResult, undefined, "explicit manual compaction must outrank ROI optimization");
await nativeManual.handlers.get("session_compact")(
  { type: "session_compact" },
  nativeManual.ctx,
);
await nativeManual.handlers.get("session_shutdown")(
  { type: "session_shutdown" },
  nativeManual.ctx,
);

resetCompactionLeaseForTest();
const terminalTransport = harness(largeBranch, {
  ...DEFAULT_PROACTIVE_COMPACTION_POLICY,
  cooldownMs: 20,
});
await terminalTransport.handlers.get("agent_settled")(
  { type: "agent_settled" },
  terminalTransport.ctx,
);
terminalTransport.compactCalls[0].onError(new Error("Summarization failed: WebSocket error"));
terminalTransport.compactCalls[0].onError(
  new Error("Cannot read properties of undefined (reading 'signal')"),
);
const primaryFailure = terminalTransport.events.find(
  (entry) => entry.data.kind === "attempt_failed",
);
assert.equal(primaryFailure.data.primary_error, "Summarization failed: WebSocket error");
assert.equal(primaryFailure.data.failure_class, "primary_transport");
const secondaryFailure = terminalTransport.events.find(
  (entry) => entry.data.kind === "secondary_duplicate_settlement",
);
assert.equal(secondaryFailure.data.failure_class, "secondary_reentrancy");
assert.match(secondaryFailure.data.secondary_error, /signal/);
await new Promise((resolve) => setTimeout(resolve, 50));
assert.equal(terminalTransport.compactCalls.length, 2);
assert.ok(terminalTransport.events.some((entry) => entry.data.kind === "retry_scheduled"));
const starts = terminalTransport.events.filter((entry) => entry.data.kind === "attempt_started");
assert.equal(starts.length, 2);
assert.equal(starts[1].data.retry_of_epoch_id, starts[0].data.epoch_id);
terminalTransport.compactCalls[1].onComplete({
  summary: "retry summary",
  firstKeptEntryId: "d",
  tokensBefore: usage.tokens,
});
assert.equal(
  terminalTransport.events.filter((entry) => entry.data.kind === "attempt_started").every(
    (entry) => entry.data.native_compaction_call_count === 1,
  ),
  true,
);

await new Promise((resolve) => setTimeout(resolve, 25));
await terminalTransport.handlers.get("agent_settled")(
  { type: "agent_settled" },
  terminalTransport.ctx,
);
terminalTransport.compactCalls[2].onError(new Error("WebSocket error"));
await new Promise((resolve) => setTimeout(resolve, 50));
terminalTransport.compactCalls[3].onError(new Error("WebSocket error"));
assert.equal(terminalTransport.sentMessages.length, 0, "transport failure must not auto-queue rollover");
assert.ok(
  terminalTransport.events.some((entry) => entry.data.kind === "native_recovery_deferred_to_pi"),
  "transport retry exhaustion must defer to Pi native recovery",
);
await terminalTransport.handlers.get("session_shutdown")(
  { type: "session_shutdown" },
  terminalTransport.ctx,
);

console.log(
  JSON.stringify(
    {
      status: "pass",
      eligibility: eligible,
      completed_events: completed.events.map((entry) => entry.data.kind),
      rejection_events: rejected.events.map((entry) => entry.data.kind),
      terminal_events: terminal.events.map((entry) => entry.data.kind),
      native_automatic_events: nativeAutomatic.events.map((entry) => entry.data.kind),
      native_manual_events: nativeManual.events.map((entry) => entry.data.kind),
      terminal_transport_events: terminalTransport.events.map((entry) => entry.data.kind),
      retry_events: terminalTransport.events.map((entry) => entry.data.kind),
    },
    null,
    2,
  ),
);
EOF

node "$TMP/runtime.mjs"
