import { S, buildCurrentAskScopeVerdict, observeProjectThreadEvidence, observeProjectThreadHintsFromText } from "../apps/pi-extension/src/state.ts";
import { readFileSync } from "fs";

function assert(cond: any, msg: string) {
  if (!cond) throw new Error(msg);
}

const contract = readFileSync("docs/worksheets/focusa-877z.25-pi-scope-cache-switch-handling.yaml", "utf8");
assert(contract.includes("schema_version: focusa.pi_scope_cache_switch_contract.v1"), "unexpected .25 contract schema");
assert(contract.includes("status: alias_only_same_project_switch_conflict_guarded"), "unexpected .25 contract status");
assert(contract.includes("same_project_alias_policy: no_scope_conflict"), "same-project alias policy missing");

Object.assign(S, {
  pi: null,
  focusaAvailable: false,
  sessionCwd: "/home/wirebot/focusa",
  continuityId: "cont-pi-scope-cache-switch",
  projectSwitchLedger: [],
  lastProjectIdentity: null,
  activeWorkpointPacket: {
    workpoint_id: "wp-pi-scope-cache-switch",
    project_root: "/home/wirebot/focusa",
    continuity_id: "cont-pi-scope-cache-switch",
    mission: "Focusa saved scope",
    next_slice: "Focusa saved next",
    canonical: true,
  },
});

observeProjectThreadEvidence({
  project_alias: "Focusa",
  evidence_ref: "test:alias-only-focusa",
  turn_id: "turn-focusa-alias",
  action: "alias=Focusa",
  confidence: 0.9,
  source: "tool_evidence",
});
const sameProjectVerdict = buildCurrentAskScopeVerdict({
  currentAskText: "continue Focusa implementation",
  workpointPacket: S.activeWorkpointPacket,
  projectRoot: "/home/wirebot/focusa",
  continuityId: "cont-pi-scope-cache-switch",
});
assert(sameProjectVerdict.status !== "conflict", `same-project alias-only ledger must not conflict: ${JSON.stringify(sameProjectVerdict)}`);
assert(sameProjectVerdict.action_authority_for_current_ask === true, "same-project alias-only ledger should preserve action authority");

observeProjectThreadHintsFromText("PTM remote project active at /home/planmarr/plan-the-marriage", "turn-ptm-path", "current_ask", "operator PTM correction");
const differentProjectVerdict = buildCurrentAskScopeVerdict({
  currentAskText: "wrong place — this is PTM remote project",
  workpointPacket: S.activeWorkpointPacket,
  projectRoot: "/home/wirebot/focusa",
  continuityId: "cont-pi-scope-cache-switch",
});
assert(differentProjectVerdict.status === "conflict", `different-project path should conflict: ${JSON.stringify(differentProjectVerdict)}`);
assert(differentProjectVerdict.action_authority_for_current_ask === true, "different-project steering must remain authoritative");
assert(differentProjectVerdict.durable_project_write_authority === false, "different-project writes must wait for verification");
assert(differentProjectVerdict.current_ask_scope.project_root === "/home/planmarr/plan-the-marriage", "explicit PTM root should win as current ask scope");

console.log("Spec98 Pi scope cache/switch handling proof passed");
