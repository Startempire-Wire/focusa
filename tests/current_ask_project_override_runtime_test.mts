import {
  S,
  buildCurrentAskScopeVerdict,
  formatCurrentAskScopeVerdictLines,
  observeProjectThreadHintsFromText,
} from "../apps/pi-extension/src/state.ts";

function assert(cond: any, msg: string) {
  if (!cond) throw new Error(msg);
}

Object.assign(S, {
  pi: null,
  focusaAvailable: false,
  sessionCwd: "/home/wirebot/focusa",
  continuityId: "cont-current-ask-override",
  projectSwitchLedger: [],
  activeWorkpointPacket: {
    workpoint_id: "wp-current-ask-override",
    project_root: "/home/wirebot/focusa",
    continuity_id: "cont-current-ask-override",
    mission: "Focusa saved scope",
    next_slice: "Focusa saved next",
    canonical: true,
  },
});

observeProjectThreadHintsFromText("PTM remote project active; planmarr scope at /home/planmarr/plan-the-marriage", "pi-turn-override-1", "current_ask", "operator PTM correction");
const verdict = buildCurrentAskScopeVerdict({
  currentAskText: "wrong place — this is PTM remote project",
  workpointPacket: S.activeWorkpointPacket,
  projectRoot: "/home/wirebot/focusa",
  continuityId: "cont-current-ask-override",
});
assert(verdict.status === "conflict", `expected conflict: ${JSON.stringify(verdict)}`);
assert(verdict.action_authority_for_current_ask === false, "action authority should be suppressed");
assert(verdict.current_ask_scope.project_root === "/home/planmarr/plan-the-marriage", `wrong root: ${JSON.stringify(verdict.current_ask_scope)}`);
assert(verdict.required_next.includes("focusa_project_verify"), `missing rebind route: ${JSON.stringify(verdict.required_next)}`);
const lines = formatCurrentAskScopeVerdictLines(verdict).join("\n");
assert(lines.includes("CURRENT_ASK_SCOPE_VERDICT"), "verdict block missing");
assert(lines.includes("action_authority_for_current_ask=false"), "suppression not visible");
assert(lines.includes("focusa_project_verify"), "rebind path not visible");

console.log("SPEC current-ask project override runtime proof passed");
