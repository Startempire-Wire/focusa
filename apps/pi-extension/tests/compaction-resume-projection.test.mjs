import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import ts from "typescript";

const source = readFileSync(
  fileURLToPath(new URL("../src/compaction-resume-projection.ts", import.meta.url)),
  "utf8"
);
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
}).outputText;
const projection = await import(`data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`);

const packet = {
  status: "verified",
  packet_id: "packet-1",
  scope: {
    scope_status: "verified",
    project_root: "/srv/p",
    continuity_id: "cont-1",
    warning: "Verify scope.",
  },
  trajectory: {
    hlt_status: "canonical_explicit",
    hlt: "Continuity.",
    warnings: [],
  },
  workpoint: {
    status: "active",
    mission: "Verify.",
    next_slice: "Test.",
  },
  next: { exact_next_tool: "focusa_workpoint_resume" },
  current_ask: { text: "Review only.", text_complete: true },
  active_blocker: { exact_blocker_excerpt: "Proof pending." },
  focus_state: { constraints: ["No deploy."] },
  receipt_expectation: { required_before_completion: true },
  evidence: { evidence_refs: ["proof-1"] },
  bloatgaurd: { rehydrate_refs: ["focusa_workpoint_resume"] },
};

for (const pressure of ["normal", "pressure", "critical", "blocked"]) {
  const rendered = projection.renderCompactionResumeProjection(
    pressure === "blocked" ? { ...packet, status: "blocked" } : packet,
    pressure
  );
  assert.ok(rendered.length <= projection.compactionProjectionBudgetTokens(pressure) * 4);
  for (const field of [
    "SCOPE_STATUS:",
    "PROJECT_ROOT:",
    "CONTINUITY_ID:",
    "HLT:",
    "MISSION:",
    "NEXT_SLICE:",
    "EXACT_NEXT_TOOL:",
    "PACKET_ID:",
    "AUTHORITY:",
    "CURRENT_ASK:",
    "SCOPE_WARNING:",
    "ACTIVE_BLOCKER:",
    "CRITICAL_CONSTRAINTS:",
    "RECEIPT_EXPECTATION:",
    "EVIDENCE_REFS:",
    "REHYDRATE_REFS:",
  ]) {
    assert.ok(rendered.includes(field), `${pressure} omitted mandatory ${field}`);
  }
  assert.ok(!rendered.includes("CompactionMissionPacket"));
  assert.ok(rendered.includes("Review only."));
  assert.ok(rendered.includes("No deploy."));
  assert.ok(rendered.includes("Proof pending."));

  // Every pressure must retain the late restriction or explicitly refuse the
  // projection. No per-field clipping may turn the prefix into permission.
  for (const changed of [
    { current_ask: { text: `${"Review this source. ".repeat(180)}NO DEPLOYMENT.`, text_complete: true } },
    { focus_state: { constraints: [`${"Keep this restriction. ".repeat(180)}NEVER CHANGE CUSTOMER DATA.`] } },
    { scope: { ...packet.scope, project_root: `/srv/${"p".repeat(4000)}/different-customer` } },
    { active_blocker: { exact_blocker_excerpt: `${"Blocked. ".repeat(500)}Missing grant.` } },
    { temporal: { execution_guard_ref: `guard-${"required".repeat(800)}` } },
  ]) {
    assert.throws(() => projection.renderCompactionResumeProjection({ ...packet, ...changed }, pressure),
      /compaction_resume_projection_mandatory_budget_exceeded/);
  }
  const late = { ...packet, current_ask: { text: `${"Review. ".repeat(65)}No global writes.`, text_complete: true } };
  try {
    const complete = projection.renderCompactionResumeProjection(late, pressure);
    assert.ok(complete.includes("No global writes."));
    assert.ok(complete.length <= projection.compactionProjectionBudgetTokens(pressure) * 4);
  } catch (error) {
    assert.match(String(error.message), /compaction_resume_projection_mandatory_budget_exceeded/);
  }
  const advisory = projection.renderCompactionResumeProjection({ ...packet, status: "blocked",
    current_ask: { text: "Review only." }, workpoint: { ...packet.workpoint, action_authority: false } }, pressure);
  assert.ok(advisory.includes("Review only."));
  assert.ok(advisory.includes("unverified; saved-action resume blocked"));
  assert.ok(advisory.includes("AUTHORITY: advisory"));
}
assert.equal(projection.projectionPressure({ status: "blocked" }, ""), "blocked");
assert.equal(projection.projectionPressure(packet, "hard"), "critical");
const configSource = readFileSync(new URL("../src/config.ts", import.meta.url), "utf8");
const compactionSource = readFileSync(new URL("../src/compaction.ts", import.meta.url), "utf8");
assert.doesNotMatch(configSource, /microCompactEveryNTurns:\s*[1-9]/);
const cadenceBody = compactionSource.slice(
  compactionSource.indexOf("export async function checkMicroCompact")
);
assert.doesNotMatch(cadenceBody, /\/commands\/submit/);
assert.match(cadenceBody, /automatic fixed-cadence compaction is disabled/);
assert.doesNotMatch(compactionSource, /renderCompactionMissionPacket/);
assert.match(compactionSource, /renderCompactionResumeProjection/);
console.log("globally budgeted compaction resume projection passed");
