import {
  buildCurrentAskScopeVerdict,
  formatCurrentAskScopeVerdictLines,
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

const key = makeAttachmentKey({
  projectRoot: "/home/wirebot/focusa",
  continuityId: "cont-current-ask-override",
  sessionId: "session-current-ask-override",
});

await runWithAttachmentRuntime(key, async () => {
  Object.assign(getAttachmentRuntime(), {
    pi: null,
    focusaAvailable: false,
    sessionCwd: "/home/wirebot/focusa",
    continuityId: "cont-current-ask-override",
    projectSwitchLedger: [],
  });
  setActiveWorkpointPacket({
    workpoint_id: "wp-current-ask-override",
    project_root: "/home/wirebot/focusa",
    continuity_id: "cont-current-ask-override",
    mission: "Focusa saved scope",
    next_slice: "Focusa saved next",
    canonical: true,
  });

  observeProjectThreadHintsFromText(
    "PTM remote project active; planmarr scope at /home/planmarr/plan-the-marriage",
    "pi-turn-override-1",
    "current_ask",
    "operator PTM correction",
  );
  const verdict = buildCurrentAskScopeVerdict({
    currentAskText: "wrong place — this is PTM remote project",
    workpointPacket: getActiveWorkpointPacket(),
    projectRoot: "/home/wirebot/focusa",
    continuityId: "cont-current-ask-override",
  });
  assert(
    verdict.status === "conflict",
    `expected conflict: ${JSON.stringify(verdict)}`,
  );
  assert(
    verdict.action_authority_for_current_ask === true,
    "operator steering authority should remain active",
  );
  assert(
    verdict.durable_project_write_authority === false,
    "durable writes should wait for scope verification",
  );
  assert(
    verdict.current_ask_scope.project_root ===
      "/home/planmarr/plan-the-marriage",
    `wrong root: ${JSON.stringify(verdict.current_ask_scope)}`,
  );
  assert(
    verdict.required_next.includes("focusa_project_verify"),
    `missing rebind route: ${JSON.stringify(verdict.required_next)}`,
  );
  const lines = formatCurrentAskScopeVerdictLines(verdict).join("\n");
  assert(lines.includes("FOCUSA_SCOPE"), "scope advisory missing");
  assert(lines.includes("conversation=continue"), "conversation authority missing");
  assert(lines.includes("durable_writes=verify_first"), "durable-write guard missing");
  assert(lines.includes("focusa_project_verify"), "rebind path not visible");
});

const diagnosticKey = makeAttachmentKey({
  projectRoot: "/root",
  continuityId: "cont-diagnostic-path",
  sessionId: "session-diagnostic-path",
});

await runWithAttachmentRuntime(diagnosticKey, async () => {
  Object.assign(getAttachmentRuntime(), {
    pi: null,
    focusaAvailable: false,
    sessionCwd: "/root",
    continuityId: "cont-diagnostic-path",
    projectSwitchLedger: [],
  });
  const stackTrace = `Extension "/home/wirebot/focusa/apps/pi-extension/src/index.ts" error: lease missing\n    at turnWorkLoopWriterHeaders (/home/wirebot/focusa/apps/pi-extension/src/turns.ts:144:11)`;
  observeProjectThreadHintsFromText(stackTrace, "pi-turn-diagnostic-1", "current_ask", "reported error");
  const verdict = buildCurrentAskScopeVerdict({
    currentAskText: stackTrace,
    projectRoot: "/root",
    continuityId: "cont-diagnostic-path",
  });
  assert(verdict.status === "aligned", `diagnostic path caused false conflict: ${JSON.stringify(verdict)}`);
  assert(verdict.action_authority_for_current_ask === true, "diagnostic text suppressed conversation");
  assert(verdict.durable_project_write_authority === false, "unsafe root unexpectedly gained write authority");
  assert(getAttachmentRuntime().projectSwitchLedger.length === 0, "diagnostic path contaminated project ledger");
});

console.log("SPEC current-ask project override runtime proof passed");
