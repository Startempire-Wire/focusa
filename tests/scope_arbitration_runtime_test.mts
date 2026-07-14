import {
  buildCurrentAskScopeVerdict,
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
  continuityId: "cont-scope-arb-runtime",
  sessionId: "session-scope-arb-runtime",
  attachmentId: "attachment-scope-arb-runtime",
});

await runWithAttachmentRuntime(key, async () => {
  const runtime = getAttachmentRuntime();
  Object.assign(runtime, {
    pi: null,
    focusaAvailable: false,
    sessionCwd: "/home/wirebot/focusa",
    continuityId: "cont-scope-arb-runtime",
    projectSwitchLedger: [],
  });
  setActiveWorkpointPacket({
    workpoint_id: "wp-scope-arb-runtime",
    project_root: "/home/wirebot/focusa",
    continuity_id: "cont-scope-arb-runtime",
    mission: "Focusa saved scope",
    next_slice: "Continue Focusa docs",
    canonical: true,
  });

  observeProjectThreadHintsFromText(
    "working in /home/wirebot/focusa",
    "pi-turn-scope-arb-1",
    "tool_evidence",
    "saved Focusa scope",
  );
  let verdict = buildCurrentAskScopeVerdict({
    currentAskText: "write an incident spec in the Focusa directory",
    workpointPacket: getActiveWorkpointPacket(),
    projectRoot: "/home/wirebot/focusa",
    continuityId: "cont-scope-arb-runtime",
  });
  assert(
    verdict.status === "aligned",
    `aligned Focusa ask blocked: ${JSON.stringify(verdict)}`,
  );
  assert(
    verdict.action_authority_for_current_ask === true,
    "aligned Focusa ask should allow action",
  );
  assert(
    verdict.required_next.length === 0,
    `aligned ask should not require rebind: ${JSON.stringify(verdict.required_next)}`,
  );

  observeProjectThreadHintsFromText(
    "PTM remote project active; planmarr scope",
    "pi-turn-scope-arb-2",
    "current_ask",
    "operator PTM correction",
  );
  verdict = buildCurrentAskScopeVerdict({
    currentAskText: "wrong place, this is PTM remote project",
    workpointPacket: getActiveWorkpointPacket(),
    projectRoot: "/home/wirebot/focusa",
    continuityId: "cont-scope-arb-runtime",
  });
  assert(
    verdict.status === "conflict",
    `conflicting PTM ask not conflict: ${JSON.stringify(verdict)}`,
  );
  assert(
    verdict.action_authority_for_current_ask === false,
    "conflicting PTM ask should suppress action",
  );
  assert(
    verdict.required_next.includes("focusa_project_verify"),
    `conflict missing verify route: ${JSON.stringify(verdict.required_next)}`,
  );
});

console.log("SPEC scope arbitration runtime proof passed");
