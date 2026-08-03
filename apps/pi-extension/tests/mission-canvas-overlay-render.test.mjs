import assert from "node:assert/strict";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { visibleWidth } from "@earendil-works/pi-tui";
import ts from "typescript";

const root = resolve(import.meta.dirname, "..");
const token = `${process.pid}-${Date.now()}`;
const accessibilityName = `.mission-canvas-overlay-accessibility-${token}.mjs`;
const viewName = `.mission-canvas-overlay-view-${token}.mjs`;
const shellName = `.mission-canvas-overlay-shell-${token}.mjs`;
const accessibilityPath = resolve(root, accessibilityName);
const viewPath = resolve(root, viewName);
const shellPath = resolve(root, shellName);
const compile = (source) =>
  ts.transpileModule(source, {
    compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
  }).outputText;

writeFileSync(
  accessibilityPath,
  compile(readFileSync(resolve(root, "src/mission-canvas-accessibility.ts"), "utf8"))
);
writeFileSync(
  viewPath,
  compile(readFileSync(resolve(root, "src/mission-canvas-view.ts"), "utf8")).replace(
    "./mission-canvas-accessibility.js",
    `./${accessibilityName}`
  )
);
writeFileSync(
  shellPath,
  compile(readFileSync(resolve(root, "src/mission-canvas-shell.ts"), "utf8")).replace(
    "./mission-canvas-view.js",
    `./${viewName}`
  )
);

const detailRows = Array.from(
  { length: 24 },
  (_, index) => `work surface detail ${index + 1} with bounded operational context`
);
const surfaceDetails = [
  ["canvas-only-content", ...detailRows],
  ["runtime-only-content", ...detailRows],
  ["release-only-content", ...detailRows],
];
const model = {
  mission: "Make Mission Canvas stable and workable",
  trajectory: "Pi-native overlay without reset or scroll storms",
  nextAction: "Dogfood the bounded overlay",
  workpointId: "workpoint:canvas-overlay",
  workItemId: "focusa-mc2-overlay",
  workRailDetails: detailRows,
  projectRoot: "/safe/project",
  continuityId: "continuity:overlay",
  evidenceRefs: detailRows,
  blockers: ["No polling", "No root takeover"],
  sessions: ["Pi · active", "UIAI · idle"],
  workSurfaces: ["Canvas repair", "Runtime proof", "Release evidence"],
  workSurfaceDetails: surfaceDetails,
  contention: [],
  researchArtifacts: detailRows,
  history: detailRows,
  contextStatus: "verified",
  roleStatus: "operator",
  interviewStatus: "closed",
  specStatus: "implementation",
  workLoopStatus: "paused",
  scopeStatus: "verified",
  workspaceProfile: "software",
  visualVariant: "default",
};

try {
  const { MissionCanvasShell } = await import(`${pathToFileURL(shellPath).href}?v=${Date.now()}`);
  let closed = 0;
  const shell = new MissionCanvasShell(
    model,
    { fg: (_name, text) => text },
    () => {},
    () => 24,
    () => closed++,
    async () => model,
    { sendUserMessage: async () => {} },
    {
      model: { id: "overlay-render-test" },
      sessionManager: { getEntries: () => [] },
      ui: { setEditorText() {}, notify() {} },
    },
    () => {},
    () => {},
    async () => {}
  );

  const first = shell.render(100);
  assert(first.length <= 24, `overlay rendered ${first.length} rows into a 24-row viewport`);
  assert(first.every((line) => visibleWidth(line) <= 100));
  assert(first.some((line) => line.includes("Canvas repair")));
  assert(first.some((line) => line.includes("canvas-only-content")));
  assert(first.some((line) => line.includes("PROMPT EDITOR")));

  shell.canvas.handleInput("surface-next");
  const switched = shell.render(100);
  assert(switched.some((line) => line.includes("runtime-only-content")));
  assert(switched.every((line) => !line.includes("canvas-only-content")));

  shell.handleInput("\x1b[6~");
  const scrolled = shell.render(100);
  assert.notDeepEqual(scrolled, first, "PageDown must move the bounded Canvas viewport");
  assert(scrolled.length <= 24);
  assert(scrolled.some((line) => line.includes("PROMPT EDITOR")));

  shell.handleInput("\x1b");
  assert.equal(closed, 1, "Escape must dismiss the overlay");
  console.log("Mission Canvas real overlay render: PASS (bounded viewport, scroll, prompt, dismiss)");
} finally {
  rmSync(accessibilityPath, { force: true });
  rmSync(viewPath, { force: true });
  rmSync(shellPath, { force: true });
}
