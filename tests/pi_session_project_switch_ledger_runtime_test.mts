import { registerVerifiedScopeRef } from "../apps/pi-extension/src/scoped-state.ts";
import {
  buildAttentionRecallVerdict,
  formatProjectSwitchLedgerLines,
  getActiveWorkpointPacket,
  getAttachmentRuntime,
  makeAttachmentKey,
  observeProjectThreadHintsFromText,
  runWithAttachmentRuntime,
  setActiveWorkpointPacket,
} from "../apps/pi-extension/src/state.ts";

function assert(cond: any, msg: string) {
  if (!cond) throw new Error(msg);
}

registerVerifiedScopeRef({
  scope_kind: "project",
  scope_id: "focusa",
  root_path: "/home/wirebot/focusa",
  canonical_name: "Focusa",
  fingerprint: "sha256:focusa-session-ledger-runtime",
});

const key = makeAttachmentKey({
  projectRoot: "/home/wirebot/focusa",
  continuityId: "cont-ledger-runtime",
  sessionId: "session-ledger-runtime",
});

await runWithAttachmentRuntime(key, async () => {
  Object.assign(getAttachmentRuntime(), {
    pi: null,
    focusaAvailable: false,
    sessionCwd: "/home/wirebot/focusa",
    continuityId: "cont-ledger-runtime",
    activeFrameGoal: "Project-switch ledger proof",
    activeFrameTitle: "Project-switch ledger proof",
    lastFocusSnapshot: {
      decisions: [],
      constraints: [],
      failures: [],
      intent: "",
      currentFocus: "",
    },
    projectSwitchLedger: [],
  });
  setActiveWorkpointPacket({
    workpoint_id: "wp-focusa-ledger-runtime",
    project_root: "/home/wirebot/focusa",
    continuity_id: "cont-ledger-runtime",
    mission: "Focusa saved scope",
    next_slice: "Continue Focusa work",
    canonical: true,
  });

  observeProjectThreadHintsFromText(
    "working in /home/wirebot/focusa",
    "pi-turn-ledger-1",
    "tool_evidence",
    "saved Focusa scope",
  );
  observeProjectThreadHintsFromText(
    "PTM remote project active; planmarr auth path observed",
    "pi-turn-ledger-2",
    "current_ask",
    "operator PTM correction",
  );

  const ledgerLines = formatProjectSwitchLedgerLines(
    "wrong place, this is PTM remote project",
  );
  assert(
    ledgerLines.some((line) => /PTM|planmarr|plan-the-marriage/i.test(line)),
    `PTM ledger line missing: ${ledgerLines.join("\n")}`,
  );
  const verdict = buildAttentionRecallVerdict({
    currentAskText: "wrong place, this is PTM remote project",
    projectRoot: "/home/wirebot/focusa",
    continuityId: "cont-ledger-runtime",
    workpointPacket: getActiveWorkpointPacket(),
  });
  assert(
    verdict.status === "conflict",
    `expected conflict: ${JSON.stringify(verdict)}`,
  );
  assert(
    verdict.scope_conflict_reason.includes("project_switch_ledger"),
    `ledger did not drive conflict: ${verdict.scope_conflict_reason}`,
  );
  assert(verdict.memory_anchor.action_authority_for_current_ask === true, "operator steering was suppressed");
  assert(verdict.memory_anchor.durable_project_write_authority === false, "conflicting durable writes were allowed");
  assert(
    verdict.memory_anchor.evidence_refs.some((ref) =>
      ref.includes("project_thread:PTM"),
    ),
    `project_thread evidence missing: ${JSON.stringify(verdict.memory_anchor.evidence_refs)}`,
  );
});

console.log("SPEC project-switch ledger runtime proof passed");
