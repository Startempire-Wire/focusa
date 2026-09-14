import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import test from "node:test";
import ts from "typescript";
import { Type } from "@sinclair/typebox";

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

test("doctor aggregate rejects each failed diagnostic dependency", () => {
  const doctor = block(tools, 'name: "focusa_tool_doctor"', 'name: "focusa_agent_prompt"');
  const expression = doctor.match(/const ready = ([\s\S]*?);/);
  assert.ok(expression);
  const good = {
    health: { ok: true }, workpoint: { ok: true }, workpointCanonical: true,
    sessionScopeSafe: true, projectRootNeedsConfirmation: false,
    loop: { ok: true }, contractDrift: { drift_detected: false },
  };
  const evaluate = new Function(...Object.keys(good), `return (${expression[1]});`);
  assert.equal(evaluate(...Object.values(good)), true);
  for (const [key, value] of Object.entries({
    health: { ok: false }, workpoint: { ok: false }, workpointCanonical: false,
    sessionScopeSafe: false, projectRootNeedsConfirmation: true,
    loop: { ok: false }, contractDrift: { drift_detected: true },
  })) {
    assert.equal(evaluate(...Object.values({ ...good, [key]: value })), false, key);
  }
  assert.match(doctor, /ok: ready,/);
  assert.match(doctor, /proves_operation_execution: false/);
});

test("doctor recovery guidance does not authorize source runtime replacement", () => {
  const doctor = block(tools, 'name: "focusa_tool_doctor"', 'name: "focusa_agent_prompt"');
  assert.doesNotMatch(doctor, /cargo build|nohup|systemctl restart|rebuild\/restart/);
  assert.match(doctor, /focusa_agent_runtime_doctor/);
  assert.match(doctor, /authorized canonical release\/install/);
});

test("doctor handler keeps text, evidence and details consistent for registry drift", async () => {
  const anchor = tools.indexOf('name: "focusa_tool_doctor"');
  const start = tools.lastIndexOf("pi.registerTool({", anchor);
  const end = tools.indexOf("pi.registerTool({", anchor);
  const compiled = ts.transpileModule(tools.slice(start, end), {
    compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None },
  }).outputText;
  const contract = { name: "fixture", family: "fixture", doc_path: "fixture.md", exemptions: [] };
  for (const drift of [false, true]) {
    let tool;
    const deps = {
      pi: { registerTool: (value) => { tool = value; } }, Type,
      FOCUSA_TOOL_CONTRACTS: [contract],
      focusaFetchDetailed: async (path) => {
        const bodies = {
          "/health": { status: "ok" },
          "/resource/mode": { resource_mode: { mode: "normal" } },
          "/workpoint/current": { status: "active", canonical: true },
          "/work-loop/status?summary_only=true": { status: "paused" },
          "/ontology/tool-contracts": { contracts: [drift ? { ...contract, doc_path: "stale.md" } : contract] },
        };
        assert.ok(Object.hasOwn(bodies, path), `unexpected request: ${path}`);
        return { ok: true, body: bodies[path] };
      },
      getActiveWorkpointPacket: () => null,
      focusaToolWorkpointScope: () => null,
      uiaiBrowserHealthCard: async () => ({ status: "ok", pressure: "normal" }),
      focusaToolContractSummary: () => ({ total: 1, by_family: { fixture: 1 } }),
      stableJson: JSON.stringify,
      getAttachmentRuntime: () => ({ spec92HookTelemetry: [], spec92TokenTelemetry: [] }),
      getTurnCount: () => 1,
      getLastProjectRootResolution: () => ({ projectRoot: "/fixture/project", requiresOperatorConfirmation: false }),
      isProjectRootAuthoritySafe: () => true,
      compactApiEcho: (value) => value,
      focusaEvidenceCaptureSuggestion: (value) => value,
    };
    new Function(...Object.keys(deps), compiled)(...Object.values(deps));
    const result = await tool.execute("fixture-call", {});
    assert.equal(result.details.ok, !drift);
    assert.equal(result.details.status, drift ? "degraded" : "completed");
    assert.match(result.content[0].text, new RegExp(`diagnostics=${result.details.status}`));
    assert.match(result.details.evidence_capture_suggestion.result, new RegExp(`diagnostics=${result.details.status}`));
    assert.match(result.content[0].text, /execution_readiness=unverified/);
    assert.equal(result.details.tool_readiness.proves_operation_execution, false);
    if (drift) assert.ok(result.details.next_tools.includes("focusa_agent_runtime_doctor"));
  }
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
