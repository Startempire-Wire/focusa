import assert from "node:assert/strict";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import { performance } from "node:perf_hooks";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { Key, visibleWidth } from "@earendil-works/pi-tui";
import ts from "typescript";

const root = resolve(import.meta.dirname, "..");
const source = readFileSync(resolve(root, "src/mission-canvas-view.ts"), "utf8");
const accessibilitySource = readFileSync(
  resolve(root, "src/mission-canvas-accessibility.ts"),
  "utf8"
);
const accessibilityName = `.mission-canvas-accessibility-performance-${process.pid}.mjs`;
const compiled = ts
  .transpileModule(source, {
    compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
  })
  .outputText.replace("./mission-canvas-accessibility.js", `./${accessibilityName}`);
const modulePath = resolve(root, `.mission-canvas-performance-${process.pid}.mjs`);
const accessibilityPath = resolve(root, accessibilityName);
writeFileSync(modulePath, compiled);
writeFileSync(
  accessibilityPath,
  ts.transpileModule(accessibilitySource, {
    compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
  }).outputText
);
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
  assert(lines.every((line) => visibleWidth(line) <= 120));
}
const heapBefore = process.memoryUsage().heapUsed;
for (let index = 0; index < 5_000; index++) {
  if (index % 7 === 0) view.handleInput("mode-next");
  if (index % 11 === 0) view.handleInput("profile-next");
  if (index % 13 === 0) view.handleInput("surface-next");
  const lines = view.render(index % 2 ? 80 : 160);
  assert(lines.length <= 60);
}
const heapGrowth = process.memoryUsage().heapUsed - heapBefore;
assert(heapGrowth < 32 * 1024 * 1024, `Mission Canvas long-session heap growth ${heapGrowth} exceeded 32 MiB`);
for (let width = 1; width < 40; width++) {
  const lines = view.render(width);
  assert(
    lines.every((line) => visibleWidth(line) <= width),
    `Mission Canvas emitted a line wider than ${width} columns`
  );
}
view.dispose();
timings.sort((a, b) => a - b);
const p95 = timings[Math.floor(timings.length * 0.95)];
rmSync(modulePath, { force: true });
rmSync(accessibilityPath, { force: true });
assert(p95 < 100, `Mission Canvas render p95 ${p95.toFixed(2)}ms exceeded 100ms`);
console.log(`Mission Canvas performance: PASS (5,000 transitions, render p95=${p95.toFixed(2)}ms, heap growth=${heapGrowth})`);
