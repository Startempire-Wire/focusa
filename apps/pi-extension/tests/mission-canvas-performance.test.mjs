import assert from "node:assert/strict";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import { performance } from "node:perf_hooks";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { Key } from "@earendil-works/pi-tui";
import ts from "typescript";

const root = resolve(import.meta.dirname, "..");
const source = readFileSync(resolve(root, "src/mission-canvas-view.ts"), "utf8");
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
}).outputText;
const modulePath = resolve(root, `.mission-canvas-performance-${process.pid}.mjs`);
writeFileSync(modulePath, compiled);
const { MissionCanvasView } = await import(`${pathToFileURL(modulePath).href}?v=${Date.now()}`);

const many = Array.from({ length: 200 }, (_, index) => `row-${index} evidence and bounded detail`);
const model = {
  mission: "Complete Spec 135",
  trajectory: "Full Mission Canvas runtime",
  nextAction: "Execute next dependency-ready Bead",
  workpointId: "workpoint-1",
  workItemId: "focusa-mc-full-d2",
  workRailDetails: many.slice(0, 10),
  projectRoot: "/safe/project",
  continuityId: "continuity-1",
  evidenceRefs: many,
  blockers: many,
  sessions: many,
  workSurfaces: many,
  workSurfaceDetails: many.map((row) => [row]),
  contention: many,
  researchArtifacts: many,
  history: many,
  contextStatus: "ready",
  roleStatus: "approved",
  interviewStatus: "active",
  specStatus: "reviewed",
  workLoopStatus: "running",
  scopeStatus: "verified",
  workspaceProfile: "general",
  visualVariant: "default",
};
const theme = { fg: (_name, text) => text, bold: (text) => text };
let renders = 0;
const view = new MissionCanvasView(
  model,
  theme,
  () => renders++,
  () => {},
  async () => model,
  () => {}
);
view.handleInput(Key.right); // Work panel exercises the maximum rendered row budget.
for (let index = 0; index < 10; index++) view.render(120); // JIT/module warmup is outside steady-state p95.
const timings = [];
for (let index = 0; index < 100; index++) {
  const started = performance.now();
  const lines = view.render(120);
  timings.push(performance.now() - started);
  assert(lines.length <= 60, lines.length);
  assert(lines.every((line) => line.length <= 120));
}
view.dispose();
timings.sort((a, b) => a - b);
const p95 = timings[Math.floor(timings.length * 0.95)];
rmSync(modulePath, { force: true });
assert(p95 < 100, `Mission Canvas render p95 ${p95.toFixed(2)}ms exceeded 100ms`);
console.log(`Mission Canvas performance: PASS (200 rows, render p95=${p95.toFixed(2)}ms)`);
