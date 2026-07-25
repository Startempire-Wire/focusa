import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import test from "node:test";

const sourcePath = path.resolve(import.meta.dirname, "../src/auto-compaction.ts");
const indexPath = path.resolve(import.meta.dirname, "../src/index.ts");
const compactionPath = path.resolve(import.meta.dirname, "../src/compaction.ts");
const turnsPath = path.resolve(import.meta.dirname, "../src/turns.ts");
const statePath = path.resolve(import.meta.dirname, "../src/state.ts");
const packagePath = path.resolve(import.meta.dirname, "../package.json");
const source = fs.readFileSync(sourcePath, "utf8");
const indexSource = fs.readFileSync(indexPath, "utf8");
const compactionSource = fs.readFileSync(compactionPath, "utf8");
const turnsSource = fs.readFileSync(turnsPath, "utf8");
const stateSource = fs.readFileSync(statePath, "utf8");
const packageManifest = JSON.parse(fs.readFileSync(packagePath, "utf8"));

function handlerBodyFrom(content, eventName) {
  const marker = `pi.on("${eventName}"`;
  const start = content.indexOf(marker);
  assert.notEqual(start, -1, `${eventName} handler must exist`);
  const next = content.indexOf("\n  pi.on(", start + marker.length);
  return content.slice(start, next === -1 ? content.length : next);
}

function handlerBody(eventName) {
  return handlerBodyFrom(source, eventName);
}

test("agent_end never races Pi native compaction with extension compaction", () => {
  const body = handlerBody("agent_end");
  assert.doesNotMatch(body, /maybeCompact\s*\(/);
  assert.match(body, /native post-run compaction/);
});

test("agent_settled is the extension-owned compaction boundary", () => {
  assert.match(handlerBody("agent_settled"), /maybeCompact\(ctx\)/);
});

test("provider transport failures retain bounded recovery for unchanged context", () => {
  assert.match(source, /websocket\|network\|socket\|timeout/);
  assert.match(source, /return isTransientCompactionError\(message\)/);
  assert.doesNotMatch(source, /automatic retry is suppressed for unchanged context/);
  assert.doesNotMatch(source, /terminalTransportFailure/);
});

test("transient provider failures retain the Spec130A one-retry budget", () => {
  assert.match(source, /maxTransientRetries\s*=\s*1/);
  assert.match(source, /Math\.min\(getPolicy\(\)\.cooldownMs, 60_000\)/);
  assert.match(source, /consecutiveTransientFailures \+ 1,\n\s+priorEpochId/);
  assert.match(source, /live_context_no_longer_requires_action/);
  assert.match(source, /context_epoch_changed/);
  assert.match(source, /failed after \$\{failedAttempts\} attempt/);
});

test("pressure alone cannot invoke compaction without preflight and exact live eligibility", () => {
  assert.ok(
    indexSource.indexOf("registerAutoCompaction(") < indexSource.indexOf("registerCompaction(pi)"),
    "eligibility handler must register before authoritative checkpoint/summary work"
  );
  const settledPath = source.slice(source.indexOf("const maybeCompact"));
  const preflightIndex = settledPath.indexOf("evaluateProactiveCompactionEligibility");
  const compactIndex = settledPath.indexOf("attemptCompaction(ctx, usage)");
  assert.ok(preflightIndex >= 0 && preflightIndex < compactIndex);

  const exactGate = handlerBody("session_before_compact");
  assert.match(exactGate, /messagesToSummarize/);
  assert.match(exactGate, /turnPrefixMessages/);
  assert.match(exactGate, /activeEpoch\.exactEligibility/);
  assert.match(exactGate, /return \{ cancel: true \}/);
  assert.match(source, /insufficient_history/);
  assert.match(source, /insufficient_reclaim/);
  assert.match(source, /negative_roi/);
  assert.match(source, /reserveTokens \* \(turnPrefixMessages\.length > 0 \? 1\.3 : 0\.8\)/);
  assert.match(source, /maxSummaryTokens \+ 1_024/);
});

test("nothing-to-compact is terminal for an unchanged semantic context", () => {
  assert.match(source, /nothing to compact\|already compacted/i);
  assert.match(source, /terminalNoopContextKey === contextKey/);
  assert.match(source, /terminalNoopContextKey = failedEpoch\.contextKey/);
  assert.match(source, /entry\.type !== "custom" \|\| entry\.customType !== EVENT_TYPE/);
});

test("cooldown, warning dedupe, and native/manual completion suppress repeat work", () => {
  assert.match(source, /Date\.now\(\) - lastAttemptAt < policy\.cooldownMs/);
  assert.match(source, /if \(lastNoticeKey === key\) return/);
  const compactHandler = handlerBody("session_compact");
  assert.match(compactHandler, /if \(retryTimer\) clearTimeout\(retryTimer\)/);
  assert.match(compactHandler, /if \(processLease\.inFlightEpochId\) return/);
});

test("attempt, primary failure, retry, rejection, and ROI outcomes are durably logged", () => {
  assert.match(source, /pi\.appendEntry\(EVENT_TYPE/);
  for (const event of [
    "attempt_started",
    "attempt_completed",
    "attempt_failed",
    "preflight_rejected",
    "eligibility_rejected",
    "retry_scheduled",
    "retry_suppressed",
  ]) {
    assert.match(source, new RegExp(`"${event}"`));
  }
  assert.match(source, /primary_error: message/);
  assert.match(source, /saved_tokens: savedTokens/);
  assert.match(source, /net_positive:/);
});

test("one process-wide first-owner coordinator suppresses duplicate registrations", () => {
  assert.match(source, /Symbol\.for\("focusa\.compaction\.coordinator\.v1"\)/);
  assert.match(source, /if \(processLease\.owner\)/);
  assert.match(source, /duplicate extension suppressed/);
  assert.match(source, /Remove the duplicate Focusa installation and reload Pi/);
  assert.match(source, /return false;[\s\S]{0,400}const maxTransientRetries/);
  assert.match(indexSource, /if \(!ownsCompactionCoordinator\) return/);
  assert.match(source, /nativeCompactionCallCount >= 1/);
  assert.match(source, /nativeCompactionCallCount \+= 1/);
  assert.match(
    source,
    new RegExp(
      `const EXTENSION_BUILD = "${packageManifest.name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}@${packageManifest.version.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}"`
    )
  );
});

test("all extension-owned native compaction routes through one coordinator", () => {
  assert.equal((source.match(/ctx\.compact\s*\(/g) || []).length, 1);
  assert.doesNotMatch(compactionSource, /ctx\.compact\s*\(/);
  assert.match(compactionSource, /requestCoordinatedCompaction\(ctx/);
});

test("proactive compaction instructions preserve scoped continuation authority", () => {
  assert.match(source, /current user ask/);
  assert.match(source, /project_root \+ continuity_id authority/);
  assert.match(source, /Workpoint and Trajectory authority/);
  assert.match(source, /verified evidence handles/);
  assert.match(source, /exact next action/);
  assert.match(source, /do-not-drift boundaries/);
});

test("emergency pressure preserves Pi native prompt and steering flow", () => {
  const body = handlerBody("input");
  assert.match(body, /input_passthrough_native_overflow_recovery/);
  assert.match(body, /return \{ action: "continue" as const \}/);
  assert.doesNotMatch(body, /action: "handled"/);
  assert.doesNotMatch(body, /Run \/focusa-rollover execute/);
  assert.doesNotMatch(body, /resend it in the replacement session/);
});

test("prompt-critical Focusa hooks perform no awaited daemon work", () => {
  for (const eventName of ["before_agent_start", "context", "input"]) {
    const body = handlerBodyFrom(turnsSource, eventName);
    assert.doesNotMatch(body, /\bawait\b/, `${eventName} must remain non-blocking`);
  }
  const inputBody = handlerBodyFrom(turnsSource, "input");
  assert.match(inputBody, /void turnWorkLoopWriterHeaders\(\)/);
  assert.match(inputBody, /\.catch\(\(\) => null\)/);
  assert.doesNotMatch(inputBody, /headers:\s*await turnWorkLoopWriterHeaders/);
  assert.match(turnsSource, /getCachedFocusState\(\)/);
  assert.match(turnsSource, /buildCachedRecentTurnsSlice\(4\)/);
  assert.match(turnsSource, /getCachedTrajectoryFocusSliceLines\(\)/);
});

test("transport retry exhaustion defers to Pi instead of auto-queuing rollover", () => {
  assert.match(source, /native_recovery_deferred_to_pi/);
  assert.doesNotMatch(source, /sendUserMessage\("\/focusa-rollover execute"/);
  assert.doesNotMatch(source, /rollover_auto_queued/);
});

test("numeric pressure and artifact filenames cannot become project aliases", () => {
  assert.match(stateSource, /!\/\[a-z\]\//);
  assert.match(stateSource, /\^\\d\+\(\?:\\\.\\d\+\)\+\$/);
  assert.match(stateSource, /NON_PROJECT_ARTIFACT_SUFFIXES/);
  assert.match(stateSource, /isPlausibleProjectAlias\(entry\.project_alias\)/);
});
