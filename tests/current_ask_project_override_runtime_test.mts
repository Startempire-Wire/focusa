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
    verdict.action_authority_for_current_ask === false,
    "action authority should be suppressed",
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
  assert(lines.includes("CURRENT_ASK_SCOPE_VERDICT"), "verdict block missing");
  assert(
    lines.includes("action_authority_for_current_ask=false"),
    "suppression not visible",
  );
  assert(lines.includes("focusa_project_verify"), "rebind path not visible");
});

console.log("SPEC current-ask project override runtime proof passed");
