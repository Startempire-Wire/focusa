import assert from "node:assert/strict";
import { spawn } from "node:child_process";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const child = spawn(process.execPath, ["tests/mission-canvas-uiai-server.mjs"], { cwd: root, stdio: ["ignore", "pipe", "pipe"] });
const ready = await new Promise((resolve, reject) => {
  const timeout = setTimeout(() => reject(new Error("UIAI harness timeout")), 10_000);
  child.once("error", reject);
  child.stderr.on("data", (chunk) => reject(new Error(String(chunk))));
  child.stdout.once("data", (chunk) => { clearTimeout(timeout); resolve(JSON.parse(String(chunk))); });
});
assert.equal(ready.schema, "focusa.mission_canvas.uiai_harness_ready.v1");
for (const scenario of ["populated", "empty-optionals", "single-queue", "zero-queues"]) {
  const response = await fetch(ready.reset_url, { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ scenario }) });
  const result = await response.json();
  const projection = result.projection;
  const layout = JSON.stringify(projection.layout_tree);
  const eligible = new Set(projection.eligible_contributions.map((item) => item.contribution_id));
  const omitted = new Set(projection.omission_diagnostics.map((item) => item.contribution_id));
  for (const id of projection.candidate_contribution_ids) assert.ok(eligible.has(id) || omitted.has(id));
  for (const id of omitted) assert.ok(!layout.includes(id), `${scenario} leaked omitted ${id}`);
  assert.ok(result.evidence_ref && result.receipt_ref);
  if (scenario === "single-queue") assert.equal(projection.layout_tree.columns, 1);
  if (scenario === "zero-queues") assert.ok(!layout.includes("queue"));
}
const first = await (await fetch(`${ready.url}__fixture/state`)).json();
const second = await (await fetch(`${ready.url}__fixture/state`)).json();
assert.deepEqual(first.projection.layout_tree, second.projection.layout_tree);
child.kill("SIGTERM");
await new Promise((resolve) => child.once("exit", resolve));
console.log("Spec 135 governed UIAI fixture/reset/correlation harness: PASS");
