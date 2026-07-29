import assert from "node:assert/strict";
import { readFile, unlink, writeFile } from "node:fs/promises";
import { fileURLToPath, pathToFileURL } from "node:url";
import path from "node:path";
import ts from "typescript";

const root = fileURLToPath(new URL("..", import.meta.url));
const sourcePath = path.join(root, "src", "workspace-invalidation.ts");
const compiledPath = path.join(root, `.workspace-invalidation-${process.pid}.mjs`);
const compiled = ts.transpileModule(await readFile(sourcePath, "utf8"), {
  compilerOptions: { module: ts.ModuleKind.ES2022, target: ts.ScriptTarget.ES2022 },
}).outputText;
await writeFile(compiledPath, compiled);
try {
  const { planWorkspaceInvalidation, reconnectInvalidationPlan } = await import(`${pathToFileURL(compiledPath).href}?v=${Date.now()}`);
  const event = {
    schema: "focusa.workspace_event.v1",
    cursor: "cursor:12",
    project_root: "/project",
    continuity_id: "main",
    invalidate: ["mission_canvas.surface_detail", "workspace.artifacts", "workspace.hidden", "unknown.key"],
  };
  const plan = planWorkspaceInvalidation(event, "/project", "main", ["mission_canvas.surface_detail"], ["workspace.artifacts"]);
  assert.equal(plan.accepted, true);
  assert.deepEqual(plan.refetchKeys, ["mission_canvas.surface_detail", "workspace.artifacts"]);
  assert.equal(plan.stale, false);
  assert.equal(planWorkspaceInvalidation(event, "/other", "main", [], []).reason, "cross_project_scope");
  assert.equal(planWorkspaceInvalidation(event, "/project", "other", [], []).reason, "cross_workstream_scope");
  const poll = planWorkspaceInvalidation(event, "/project", "main", [], [], "polling_fallback");
  assert.equal(poll.stale, true);
  assert.equal(reconnectInvalidationPlan("cursor:12").reason, "resume_from_cursor");
  assert.equal(reconnectInvalidationPlan().reason, "snapshot_fallback");
  console.log("Workspace named invalidation/SSE refresh test passed");
} finally {
  await unlink(compiledPath).catch(() => {});
}
