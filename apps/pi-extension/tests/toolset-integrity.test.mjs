import assert from "node:assert/strict";
import { readFile } from "node:fs/promises";

const { inspectFocusaToolsetIntegrity } = await import("../src/tool-contracts.ts");
const expected = ["focusa_tool_doctor", "focusa_workpoint_checkpoint", "focusa_north_star_gate"];

const intact = inspectFocusaToolsetIntegrity({
  expectedToolNames: expected,
  configuredToolNames: expected,
  activeToolNames: expected,
  registeredToolNames: expected,
});
assert.equal(intact.drift_detected, false);
assert.deepEqual(intact.missing_active, []);

const lost = inspectFocusaToolsetIntegrity({
  expectedToolNames: expected,
  configuredToolNames: expected,
  activeToolNames: ["focusa_tool_doctor"],
  registeredToolNames: expected,
});
assert.equal(lost.drift_detected, true);
assert.deepEqual(lost.missing_active, ["focusa_north_star_gate", "focusa_workpoint_checkpoint"]);
assert.match(lost.recovery_action, /Reload the Focusa Pi extension/);
assert.match(lost.recovery_action, /MCP tools\.search cannot restore/);

const toolsSource = await readFile(new URL("../src/tools.ts", import.meta.url), "utf8");
const compactionSource = await readFile(new URL("../src/compaction.ts", import.meta.url), "utf8");
assert.match(toolsSource, /const toolsetIntegrity = focusaToolsetIntegrity\(\);/);
assert.match(toolsSource, /toolset_integrity: toolsetIntegrity/);
assert.match(compactionSource, /FOCUSA_TOOLSET_INTEGRITY_WARNING/);
assert.match(compactionSource, /getActiveTools/);

console.log("toolset integrity contract: PASS");
