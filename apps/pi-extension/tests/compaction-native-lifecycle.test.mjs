import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const here = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(here, "../../..");
const extensionSource = fs.readFileSync(path.join(root, "apps/pi-extension/src/compaction.ts"), "utf8");
const resumeProjectionSource = fs.readFileSync(
  path.join(root, "apps/pi-extension/src/compaction-resume-projection.ts"),
  "utf8"
);
const stateSource = fs.readFileSync(path.join(root, "apps/pi-extension/src/state.ts"), "utf8");
const turnsSource = fs.readFileSync(path.join(root, "apps/pi-extension/src/turns.ts"), "utf8");
const apiSource = fs.readFileSync(path.join(root, "crates/focusa-api/src/routes/compaction.rs"), "utf8");
const fallbackDoc = fs.readFileSync(path.join(root, "docs/current/COMPACTION_FALLBACKS.md"), "utf8");
const spec142 = fs.readFileSync(
  path.join(root, "docs/142-focusa-seamless-pi-continuation-and-workflow-dependency-onboarding-spec.md"),
  "utf8"
);
const spec130a = fs.readFileSync(
  path.join(root, "docs/130a-zero-waste-compaction-performance-addendum.md"),
  "utf8"
);
const piPatchSource = fs.readFileSync(
  path.join(root, "apps/pi-extension/scripts/patch-pi-shrinkwrap-dependencies.mjs"),
  "utf8"
);

function blockFrom(source, marker, nextMarker) {
  const start = source.indexOf(marker);
  assert.notEqual(start, -1, `missing marker: ${marker}`);
  const end = source.indexOf(nextMarker, start + marker.length);
  assert.notEqual(end, -1, `missing next marker: ${nextMarker}`);
  return source.slice(start, end);
}

test("compaction packets preserve bounded temporal authority without full projection blobs", () => {
  const render = blockFrom(
    resumeProjectionSource,
    "export function renderCompactionResumeProjection",
    "export function compactionProjectionBudgetTokens"
  );
  assert.match(render, /TEMPORAL_STATUS:/);
  assert.match(render, /DEADLINE_STATUS:/);
  assert.match(render, /TEMPORAL_REFS:/);
  assert.doesNotMatch(render, /JSON\.stringify\(temporal/);
  assert.match(apiSource, /bounded_temporal_context/);
  assert.match(apiSource, /cache_safe_refs_only/);
});

test("session_compact is network-free and returns before background verification", () => {
  const handler = blockFrom(extensionSource, 'pi.on("session_compact", (event, ctx) => {', "\n  });\n}");
  assert.equal(handler.includes("await "), false);
  assert.equal(handler.includes("focusaFetch("), false);
  assert.match(handler, /focusa-compaction-verification-pending/);
  assert.match(handler, /setTimeout\(\(\) =>/);
  assert.match(handler, /runPostCompactionVerification\(event, ctx\)/);
});

test("normal lifecycle uses one prepare and one verify RPC", () => {
  const prepare = blockFrom(
    extensionSource,
    "async function prepareCompactionEpoch",
    "\nasync function runPostCompactionVerification"
  );
  const verify = blockFrom(
    extensionSource,
    "async function runPostCompactionVerification",
    "\nexport function registerCompaction"
  );
  assert.equal((prepare.match(/focusaFetch\(/g) || []).length, 1);
  assert.match(prepare, /"\/compaction\/prepare"/);
  assert.match(prepare, /withCompactionDeadline\(event\?\.signal/);
  assert.match(extensionSource, /signal\?\.addEventListener\("abort"/);
  assert.equal((verify.match(/focusaFetch\(/g) || []).length, 1);
  assert.match(verify, /"\/compaction\/verify"/);
  assert.equal(verify.includes("refreshWorkpointResumePacket"), false);
  assert.equal(verify.includes("refreshTrajectoryResumePacket"), false);
});

test("Pi retains native summary while Focusa enriches its preservation instructions", () => {
  const before = blockFrom(
    extensionSource,
    'pi.on("session_before_compact"',
    "\n\n  // Pi awaits session_compact handlers"
  );
  assert.match(before, /prepareCompactionEpoch\(event\)/);
  assert.match(before, /prepared\?\.native_compactor_instructions/);
  assert.match(before, /event\.customInstructions =/);
  assert.match(before, /slice\(0, 12_000\)/);
  assert.match(before, /return undefined/);
  assert.equal(before.includes("compaction: {"), false);
  assert.match(before, /using Pi native compaction/);
});

test("Focusa postinstall overlays Pi with safe active-loop native compaction", () => {
  assert.match(piPatchSource, /FOCUSA_TOOL_BOUNDARY_COMPACTION_V2/);
  assert.match(piPatchSource, /_pendingExtensionToolBoundaryCompaction/);
  assert.match(piPatchSource, /event\.type === "turn_end"/);
  assert.match(piPatchSource, /_runAutoCompaction\(reason, false, request\?\.customInstructions\)/);
  assert.match(piPatchSource, /customInstructions = compactionEvent\.customInstructions/);
  assert.match(piPatchSource, /_focusaRefreshCompactedMessagesOnNextTurn/);
  assert.match(piPatchSource, /messages: refreshCompactedMessages/);
  assert.match(piPatchSource, /this\.agent\.state\.messages\.slice\(\)/);
  assert.match(piPatchSource, /A Focusa compaction request is already queued/);
  assert.doesNotMatch(piPatchSource, /PI_TOOL_BOUNDARY_COMPACTION_TOKEN_CAP/);
});

test("Cardinal Rule keeps Focusa active around one Pi native call", () => {
  assert.match(spec130a, /FOCUSA COMPACTION MUST ONLY IMPROVE ON PI'S COMPACTION/);
  assert.match(spec130a, /ACTIVE ENHANCEMENT/);
  assert.match(spec130a, /FAILURE MONOTONICITY/);
  assert.match(spec130a, /must not replace the integrated path with Pi-only compaction/);
});

test("resume projection is explicit nextTurn with unknown completion and no retry", () => {
  const delivery = blockFrom(
    extensionSource,
    "function queueCompactionResumeContext",
    "\ntype CompactionPrepareResult"
  );
  assert.match(delivery, /triggerTurn: false, deliverAs: "nextTurn"/);
  assert.match(delivery, /compactResumeDeliveryState = "unknown_completion"/);
  assert.equal(extensionSource.includes("scheduleCompactionResumeRetry"), false);
  assert.equal(extensionSource.includes("scheduleCompactionResumeWatchdog"), false);
  assert.equal(extensionSource.includes("compactResumeRetryTimer"), false);
});

test("delivery outcome and pending epoch state are typed and operator-supersedable", () => {
  assert.match(stateSource, /compactionVerifyPendingKey: ""/);
  assert.match(stateSource, /compactResumeDeliveryKey: ""/);
  assert.match(stateSource, /\| "unknown_completion"/);
  assert.match(
    stateSource,
    /compactResumeDeliveryState: getAttachmentRuntime\(\)\.compactResumeDeliveryState/
  );
  assert.match(turnsSource, /compactResumeDeliveryState = "superseded_by_operator"/);
  assert.match(turnsSource, /focusa-compaction-delivery-outcome/);
  assert.match(extensionSource, /compactResumeDeliveryState === "superseded_by_operator"/);
  // An already-acknowledged delivery (delivered at agent_start) is supporting
  // context under new steering; it must never be falsely superseded.
  assert.match(turnsSource, /\["pending", "unknown_completion", "deferred_to_next_turn"\]/);
  assert.doesNotMatch(
    turnsSource,
    /\["pending", "unknown_completion", "deferred_to_next_turn", "delivered"\]/
  );
});

test("daemon exposes bounded prepare and verify routes off the async core writer", () => {
  assert.match(apiSource, /route\("\/v1\/compaction\/prepare", post\(prepare\)\)/);
  assert.match(apiSource, /route\("\/v1\/compaction\/verify", post\(verify\)\)/);
  assert.match(apiSource, /tokio::task::spawn_blocking/);
  assert.match(apiSource, /focusa\.compaction_prepare_result\.v1/);
  assert.match(apiSource, /focusa\.compaction_verify_result\.v1/);
});

test("current docs preserve Pi queue authority and explicit next-turn delivery", () => {
  for (const document of [fallbackDoc, spec142]) {
    assert.match(document, /deliverAs:[`"]?nextTurn/);
    assert.match(document, /triggerTurn:false/);
  }
  assert.match(fallbackDoc, /unknown_completion/);
  assert.match(spec142, /session_compact/);
  assert.match(spec142, /network-free/);
});
