import assert from "node:assert/strict";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import ts from "typescript";

const root = resolve(import.meta.dirname, "..");
const compile = (name) =>
  ts.transpileModule(readFileSync(resolve(root, `src/${name}.ts`), "utf8"), {
    compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
  }).outputText;
const token = `inventory-${process.pid}`;
const modelPath = resolve(root, `${token}-model.mjs`);
const inventoryPath = resolve(root, `${token}-inventory.mjs`);
const spec138Path = resolve(root, `${token}-spec138.mjs`);
writeFileSync(spec138Path, compile("generated/spec138-operations"));
writeFileSync(
  modelPath,
  compile("mission-canvas-model").replace("./generated/spec138-operations.js", `./${token}-spec138.mjs`)
);
writeFileSync(
  inventoryPath,
  compile("mission-canvas-session-inventory").replace("./mission-canvas-model.js", `./${token}-model.mjs`)
);
const model = await import(`${pathToFileURL(modelPath).href}?v=${Date.now()}`);
const inventory = await import(`${pathToFileURL(inventoryPath).href}?v=${Date.now()}`);
const discovered = {
  sessions: Array.from({ length: 210 }, (_, index) => ({
    agent: "pi",
    session_id: `pi-${index}`,
    continuity_id: "continuity",
    project_root: "/project",
    last_activity: "2026-07-27T00:00:00Z",
    session_path: `/sessions/${index}`,
  })),
};
const surfaces = model.projectWorkSurfaces({
  surfaces: [
    {
      work_surface_id: "surface-browser",
      kind: "uiai_browser",
      scope: { project_root: "/project", continuity_id: "continuity" },
      primary_attachment: { instance_id: "uiai", session_id: "browser-1", attachment_id: "attachment-1" },
      activity: {
        lifecycle_state: "active",
        health: "healthy",
        pending_approval_count: 2,
        conflict_count: 1,
      },
      isolation: { writer_lease_ref: "lease-1", browser_isolation_class: "isolated_context" },
    },
  ],
});
const rows = inventory.projectSessionInventory(discovered, surfaces, {
  sessions: [
    { session_id: "silent-1", status: "running", project_root: "/project", continuity_id: "continuity" },
  ],
});
assert.equal(rows.length, 200);
assert(rows.some((row) => row.kind === "uiai_browser" && row.browserIsolation === "isolated_context"));
assert(rows.some((row) => row.kind === "silent_session"));
assert(rows.every((row) => row.projectRoot && row.continuityId && row.sessionId));
rmSync(modelPath, { force: true });
rmSync(inventoryPath, { force: true });
rmSync(spec138Path, { force: true });
console.log("Mission Canvas session inventory: PASS (Pi/Silent/UIAI identity, grouping, cap)");
