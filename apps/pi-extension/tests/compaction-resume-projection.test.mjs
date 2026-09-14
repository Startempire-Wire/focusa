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
    project_root: `/srv/${"project".repeat(100)}`,
    continuity_id: `continuity-${"x".repeat(500)}`,
  },
  trajectory: {
    hlt_status: "canonical_explicit",
    hlt: "ship truthful release ".repeat(100),
    warnings: ["warning ".repeat(100)],
  },
  workpoint: {
    status: "active",
    mission: "preserve tactical cognition ".repeat(100),
    next_slice: "run exact acceptance ".repeat(100),
  },
  next: { exact_next_tool: "focusa_workpoint_resume" },
  evidence: { evidence_refs: Array.from({ length: 100 }, (_, i) => `evidence:${i}`) },
  bloatgaurd: { rehydrate_refs: ["focusa_workpoint_resume", "focusa_trajectory_view"] },
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
  ]) {
    assert.ok(rendered.includes(field), `${pressure} omitted mandatory ${field}`);
  }
  assert.ok(!rendered.includes("CompactionMissionPacket"));
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
