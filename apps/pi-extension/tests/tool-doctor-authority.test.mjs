import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import test from "node:test";

const tools = readFileSync(fileURLToPath(new URL("../src/tools.ts", import.meta.url)), "utf8");
const workpoint = readFileSync(
  fileURLToPath(new URL("../../../crates/focusa-api/src/routes/workpoint.rs", import.meta.url)),
  "utf8"
);

function block(source, startToken, endToken) {
  const start = source.indexOf(startToken);
  const end = source.indexOf(endToken, start + startToken.length);
  assert.notEqual(start, -1, `missing ${startToken}`);
  assert.notEqual(end, -1, `missing ${endToken}`);
  return source.slice(start, end);
}

test("tool doctor separates executable tools from operator actions", () => {
  const doctor = block(tools, 'name: "focusa_tool_doctor"', 'name: "focusa_agent_prompt"');
  assert.doesNotMatch(doctor, /["']interview["']/);
  assert.match(doctor, /const nextActions/);
  assert.match(doctor, /action_type: "operator_input_required"/);
  assert.match(doctor, /next_actions: nextActions/);
  assert.match(doctor, /tool_readiness:/);
  assert.match(doctor, /daemon_health:/);
  assert.match(doctor, /scope_status:/);
  assert.match(doctor, /workpoint_status:/);
  assert.match(doctor, /work_loop_status:/);
  assert.match(doctor, /getActiveWorkpointPacket/);
  assert.match(doctor, /exact_scoped_pi_resume_packet/);
});

test("project confirmation envelopes never advertise nonexistent interview tool", () => {
  const confirmationGate = block(
    tools,
    "function projectRootConfirmationGate",
    "function scopeRecoveryContext"
  );
  assert.doesNotMatch(confirmationGate, /["']interview["']/);
  assert.match(confirmationGate, /next_tools: \["focusa_project_identity", "focusa_workpoint_checkpoint"\]/);
  assert.match(confirmationGate, /action_type: "operator_input_required"/);

  const rejection = block(
    workpoint,
    "fn unconfirmed_project_root_rejection",
    "fn session_identity_requires_project_root_confirmation"
  );
  assert.doesNotMatch(rejection, /"interview"/);
  assert.match(rejection, /"next_actions"/);
  assert.match(rejection, /"operator_input_required"/);
});
