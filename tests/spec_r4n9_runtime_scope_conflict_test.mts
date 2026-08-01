import { registerVerifiedScopeRef } from "../apps/pi-extension/src/scoped-state.ts";
import {
  buildAttentionRecallVerdict,
  getAttachmentRuntime,
  makeAttachmentKey,
  runWithAttachmentRuntime,
} from "../apps/pi-extension/src/state.ts";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

registerVerifiedScopeRef({
  scope_kind: "project",
  scope_id: "focusa-r4n9",
  root_path: "/home/wirebot/focusa",
  canonical_name: "Focusa",
  fingerprint: "sha256:focusa-r4n9",
});
const key = makeAttachmentKey({
  projectRoot: "/home/wirebot/focusa",
  continuityId: "focusa-cont-r4n9",
  sessionId: "session-r4n9",
});

await runWithAttachmentRuntime(key, async () => {
  Object.assign(getAttachmentRuntime(), {
    pi: null,
    focusaAvailable: false,
    sessionCwd: "/home/wirebot/focusa",
    continuityId: "focusa-cont-r4n9",
    projectSwitchLedger: [],
  });

  const conflict = buildAttentionRecallVerdict({
    workpointPacket: {
      project_root: "/home/wirebot/focusa",
      continuity_id: "focusa-cont-r4n9",
      mission: "Test mission",
      next_slice: "Continue test work",
    },
    currentAskText: "Work on /home/other/project instead",
    projectRoot: "/home/wirebot/focusa",
    continuityId: "focusa-cont-r4n9",
  });
  assert(conflict.current_ask_scope_status === "conflict", "scope conflict not detected");
  assert(conflict.memory_anchor.action_authority_for_current_ask === true, "operator steering was suppressed");
  assert(conflict.memory_anchor.durable_project_write_authority === false, "conflicting durable writes were allowed");
  assert(!conflict.memory_anchor.next_action.includes("BLOCKED"), "model-flow blocking wording leaked");

  const aligned = buildAttentionRecallVerdict({
    workpointPacket: {
      project_root: "/home/wirebot/focusa",
      continuity_id: "focusa-cont-r4n9",
      mission: "Test mission",
      next_slice: "Continue test work",
    },
    currentAskText: "Continue with the test work",
    projectRoot: "/home/wirebot/focusa",
    continuityId: "focusa-cont-r4n9",
  });
  assert(aligned.memory_anchor.action_authority_for_current_ask === true, "aligned steering authority missing");
  assert(aligned.memory_anchor.durable_project_write_authority === true, "aligned durable writes were gated");
});

console.log("PASS: scope conflict preserves conversation and gates only durable writes");
