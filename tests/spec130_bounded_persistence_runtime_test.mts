import {
  getAttachmentRuntime,
  makeAttachmentKey,
  maybeCaptureReportSummaryFromAssistantOutput,
  nativeSessionAllowsNonessentialPersistence,
  observeProjectThreadEvidence,
  persistState,
  resetPiSessionScopedState,
  runWithAttachmentRuntime,
} from "../apps/pi-extension/src/state.ts";
import {
  COMPACTION_PERSISTENCE_ANCHOR_REF_SCHEMA,
  COMPACTION_PERSISTENCE_ANCHOR_SCHEMA,
  NATIVE_ANCHOR_MAX_BYTES,
  PROJECT_SWITCH_ANCHOR_MAX_BYTES,
  loadPersistedRecoveryState,
  semanticPersistenceDigest,
} from "../apps/pi-extension/src/persistence.ts";
import { mkdtempSync, readFileSync, readdirSync, rmSync, statSync, writeFileSync } from "fs";
import { tmpdir } from "os";
import { join } from "path";

function assert(condition: any, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

const dataDir = mkdtempSync(join(tmpdir(), "focusa-spec130-anchor-"));
process.env.FOCUSA_DATA_DIR = dataDir;
const entries: Array<{ customType: string; data: any }> = [];
const turnsSource = readFileSync(join(process.cwd(), "apps/pi-extension/src/turns.ts"), "utf8");
const attachmentKey = makeAttachmentKey({
  projectRoot: "/tmp/focusa-spec130-project",
  continuityId: "focusa-cont-dynamic-a",
  sessionId: "pi-spec130-session",
});

try {
  runWithAttachmentRuntime(attachmentKey, () => {
    const S = getAttachmentRuntime();
    resetPiSessionScopedState("spec130_bounded_persistence_test");
    Object.assign(S, {
    pi: {
      appendEntry(customType: string, data: any) {
        entries.push({ customType, data });
      },
    },
    focusaAvailable: false,
    sessionFrameKey: "pi-spec130-session",
    sessionCwd: "/tmp/focusa-spec130-project",
    continuityId: "focusa-cont-dynamic-a",
    activeFrameId: "frame-spec130",
    activeFrameTitle: "Bound native persistence",
    activeFrameGoal: "Keep semantic memory without replay OOM",
    currentAsk: {
      text: "Implement bounded persistence",
      kind: "instruction",
      sourceTurnId: "turn-1",
      updatedAt: 1,
      sessionId: "pi-spec130-session",
      projectRoot: "/tmp/focusa-spec130-project",
      continuityId: "focusa-cont-dynamic-a",
    },
    queryScope: {
      scopeKind: "mission_carryover",
      carryoverPolicy: "allow_if_relevant",
      sourceTurnId: "turn-1",
      updatedAt: 1,
    },
    localDecisions: Array.from({ length: 40 }, (_, i) => `decision-${i}-${"x".repeat(500)}`),
    localConstraints: ["Do not lose canonical Workpoint authority"],
    localFailures: [],
    lastFocusSnapshot: {
      decisions: ["Use bounded anchors"],
      constraints: ["Continuity IDs rotate"],
      failures: [],
      intent: "Eliminate replay OOM",
      currentFocus: "Spec 130 A2",
    },
    activeWorkpointPacket: {
      workpoint_id: "wp-spec130-a2",
      revision: 7,
      checkpoint_id: "checkpoint-spec130-a2",
      project_root: "/tmp/focusa-spec130-project",
      continuity_id: "focusa-cont-dynamic-a",
      session_id: "pi-spec130-session",
      pi_session_frame_key: "pi-spec130-session",
      mission: "Bound Pi persistence",
      next_slice: "Prove semantic dedupe",
      canonical: true,
      raw_blob: "z".repeat(100_000),
    },
    activeWorkpointSummary: "Bound Pi persistence",
    lastTrajectoryClarity: {
      trajectory_id: "trajectory-spec130-a2",
      project_root: "/tmp/focusa-spec130-project",
      continuity_id: "focusa-cont-dynamic-a",
      session_id: "pi-spec130-session",
      hlt_status: "canonical_explicit",
      long_term_goal: "Ship reliable Focusa compaction",
      raw_blob: "y".repeat(80_000),
    },
    projectSwitchLedger: [],
    wbmEnabled: true,
    lastPersistAt: 0,
    lastPersistHash: "",
    persistRevision: 0,
    pendingPersistAnchor: false,
    lastPersistSidecarKey: "",
    lastPersistSidecarBytes: 0,
    lastProjectSwitchPersistHash: "",
  });

  S.lastNativeSessionPressure = {
    posture: "hard_pressure",
    recommended_action: "rollover",
  } as any;
  assert(!nativeSessionAllowsNonessentialPersistence(), "hard pressure allowed nonessential native writes");
  const report = maybeCaptureReportSummaryFromAssistantOutput(
    `Implementation report\nStatus: completed\nProof: ${"bounded recovery anchors remain available with exact evidence refs; ".repeat(5)}`,
    "turn-hard-pressure"
  );
  assert(report !== null, "hard pressure should retain canonical/ECS report capture");
  assert(
    !entries.some((entry) => entry.customType === "focusa-report-summary"),
    "hard pressure appended a nonessential report-summary native entry"
  );
  S.lastNativeSessionPressure = {
    posture: "soft_pressure",
    recommended_action: "checkpoint",
  } as any;
  assert(nativeSessionAllowsNonessentialPersistence(), "soft pressure incorrectly blocked bounded native writes");
  S.lastNativeSessionPressure = {
    posture: "hard_pressure",
    recommended_action: "rollover",
  } as any;

  const utilityStart = turnsSource.indexOf("if (!getAttachmentRuntime().seenFirstBeforeAgentStart)");
  const utilityEnd = turnsSource.indexOf('pi.on("context"', utilityStart);
  assert(utilityStart >= 0 && utilityEnd > utilityStart, "utility-card persistence block missing");
  const utilityBlock = turnsSource.slice(utilityStart, utilityEnd);
  assert(
    utilityBlock.includes("nativeSessionAllowsNonessentialPersistence()"),
    "utility-card native message lacks hard-pressure suppression"
  );
  assert(
    utilityBlock.includes('reason: "native_session_hard_pressure"'),
    "utility-card suppression telemetry is missing"
  );

  persistState();
  const stateEntries = entries.filter((entry) => entry.customType === "focusa-state");
  const wbmEntries = entries.filter((entry) => entry.customType === "focusa-wbm-state");
  assert(stateEntries.length === 1, `expected one state anchor, got ${stateEntries.length}`);
  assert(wbmEntries.length === 1, `expected one WBM ref, got ${wbmEntries.length}`);

  const anchor = stateEntries[0].data;
  assert(anchor.schema === COMPACTION_PERSISTENCE_ANCHOR_SCHEMA, "wrong state anchor schema");
  assert(
    wbmEntries[0].data.schema === COMPACTION_PERSISTENCE_ANCHOR_REF_SCHEMA,
    "WBM did not reuse the bounded sidecar reference"
  );
  assert(
    Buffer.byteLength(JSON.stringify(anchor), "utf8") <= NATIVE_ANCHOR_MAX_BYTES,
    "state anchor exceeded hard cap"
  );
  assert(!("authoritativeDecisions" in anchor), "full Focusa state leaked into native anchor");
  assert(!("raw_blob" in anchor), "raw Workpoint/Trajectory payload leaked into native anchor");

  const sidecarDir = join(dataDir, "pi-session-state");
  const latestSidecarPath = () => {
    const prefix = `${anchor.sidecarKey}.r`;
    const name = readdirSync(sidecarDir)
      .filter((candidate) => candidate.startsWith(prefix) && candidate.endsWith(".json"))
      .sort((a, b) => {
        const revisionA = Number(a.slice(prefix.length).split(".", 1)[0] || 0);
        const revisionB = Number(b.slice(prefix.length).split(".", 1)[0] || 0);
        return revisionB - revisionA;
      })[0];
    assert(name, "bounded recovery sidecar missing");
    return join(sidecarDir, name);
  };
  assert(statSync(latestSidecarPath()).isFile(), "bounded recovery sidecar missing");
  assert(
    (statSync(latestSidecarPath()).mode & 0o077) === 0,
    "recovery sidecar permissions are not private"
  );
  const restored = loadPersistedRecoveryState(anchor);
  assert(restored?.frameGoal === "Keep semantic memory without replay OOM", "sidecar restore failed");
  assert(
    loadPersistedRecoveryState(wbmEntries[0].data)?.frameGoal ===
      "Keep semantic memory without replay OOM",
    "WBM anchor ref did not resolve the shared sidecar"
  );
  assert(
    loadPersistedRecoveryState({ sessionId: "legacy", decisions: ["legacy-compatible"] })
      ?.decisions?.[0] === "legacy-compatible",
    "legacy focusa-state payload is not backward readable"
  );
  assert(restored?.activeWorkpointPacket?.raw_blob === undefined, "oversized Workpoint was not compacted");

  const originalDigest = semanticPersistenceDigest(restored);
  S.currentAsk.updatedAt = 999_999;
  S.queryScope.updatedAt = 999_999;
  S.lastCompactResumeAt = 999_999;
  persistState();
  assert(entries.length === 2, "volatile-only changes appended another native entry");
  assert(S.lastPersistHash === originalDigest, "volatile-only changes changed semantic digest");

  S.activeFrameGoal = "Changed semantic recovery goal";
  persistState();
  assert(entries.length === 2, "changed state bypassed coalescing interval");
  assert(
    loadPersistedRecoveryState(anchor)?.frameGoal === "Changed semantic recovery goal",
    "existing anchor did not resolve the atomically updated sidecar"
  );
  S.lastPersistAt = Date.now() - 10_000;
  persistState();
  assert(
    entries.filter((entry) => entry.customType === "focusa-state").length === 2,
    "pending semantic revision was not anchored after coalescing interval"
  );

  const ledgerStart = entries.filter(
    (entry) => entry.customType === "focusa-project-switch-ledger"
  ).length;
  for (let i = 0; i < 1_000; i++) {
    observeProjectThreadEvidence({
      project_root: "/tmp/focusa-spec130-project",
      project_alias: "Focusa",
      evidence_ref: "evidence:spec130-a2",
      turn_id: `turn-${i}`,
      action: "same semantic observation",
      confidence: 0.9,
      source: "tool_evidence",
    });
  }
  const ledgerEntries = entries.filter(
    (entry) => entry.customType === "focusa-project-switch-ledger"
  );
  assert(ledgerEntries.length - ledgerStart === 1, "project-switch observations were not deduplicated");
  assert(
    Buffer.byteLength(JSON.stringify(ledgerEntries.at(-1)?.data), "utf8") <=
      PROJECT_SWITCH_ANCHOR_MAX_BYTES,
    "project-switch anchor exceeded hard cap"
  );

  assert(
    readdirSync(sidecarDir).filter((candidate) => candidate.startsWith(`${anchor.sidecarKey}.r`))
      .length <= 3,
    "sidecar generation retention exceeded its hard bound"
  );
  const sidecarPath = latestSidecarPath();
  const sidecar = JSON.parse(readFileSync(sidecarPath, "utf8"));
  sidecar.semanticDigest = "sha256:corrupt";
  writeFileSync(sidecarPath, JSON.stringify(sidecar));
  assert(
    loadPersistedRecoveryState(anchor) !== null,
    "reader did not fall back to an older integrity-valid sidecar generation"
  );
  for (const name of readdirSync(sidecarDir).filter((candidate) =>
    candidate.startsWith(`${anchor.sidecarKey}.r`)
  )) {
    const path = join(sidecarDir, name);
    const generation = JSON.parse(readFileSync(path, "utf8"));
    generation.semanticDigest = "sha256:corrupt";
    writeFileSync(path, JSON.stringify(generation));
  }
  assert(loadPersistedRecoveryState(anchor) === null, "corrupt sidecar generations passed validation");

  console.log("PASS: Spec 130 bounded semantic persistence runtime contract");
  });
} finally {
  rmSync(dataDir, { recursive: true, force: true });
}
