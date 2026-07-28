import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, resolve } from "node:path";
import { pathToFileURL } from "node:url";
import ts from "typescript";

const root = resolve(import.meta.dirname, "..");
const source = readFileSync(join(root, "src/config.ts"), "utf8");
const compiled = ts.transpileModule(source, {
  compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
}).outputText;
const sandbox = mkdtempSync(join(tmpdir(), "focusa-mode-test-"));
const modulePath = join(sandbox, "config.mjs");
writeFileSync(modulePath, compiled);
const config = await import(`${pathToFileURL(modulePath).href}?v=${Date.now()}`);

const oldHome = process.env.HOME;
const oldMode = process.env.FOCUSA_PI_INTERACTION_MODE;
const home = join(sandbox, "home");
const project = join(sandbox, "project");
mkdirSync(join(home, ".pi", "agent"), { recursive: true });
mkdirSync(join(project, ".pi"), { recursive: true });
process.env.HOME = home;
delete process.env.FOCUSA_PI_INTERACTION_MODE;

assert.deepEqual(config.resolveInteractionMode(project), { mode: "canvas-guided", source: "default" });
writeFileSync(
  join(home, ".pi", "agent", "settings.json"),
  JSON.stringify({
    focusaPiBridge: {
      interactionMode: "terminal-guided",
      missionCanvasWorkspaceProfile: "legal",
      missionCanvasVisualVariant: "high-contrast",
    },
  })
);
assert.deepEqual(config.resolveInteractionMode(project), { mode: "terminal-guided", source: "user" });
writeFileSync(
  join(project, ".pi", "settings.json"),
  JSON.stringify({
    focusaPiBridge: { interactionMode: "canvas-guided", missionCanvasWorkspaceProfile: "software" },
  })
);
assert.deepEqual(config.resolveInteractionMode(project), { mode: "canvas-guided", source: "project" });
const layered = config.loadConfig(project).config;
assert.equal(layered.missionCanvasWorkspaceProfile, "software");
assert.equal(layered.missionCanvasVisualVariant, "high-contrast");
process.env.FOCUSA_PI_INTERACTION_MODE = "headless";
assert.deepEqual(config.resolveInteractionMode(project), { mode: "headless", source: "session-env" });

if (oldHome === undefined) delete process.env.HOME;
else process.env.HOME = oldHome;
if (oldMode === undefined) delete process.env.FOCUSA_PI_INTERACTION_MODE;
else process.env.FOCUSA_PI_INTERACTION_MODE = oldMode;
rmSync(sandbox, { recursive: true, force: true });
console.log("Mission Canvas interaction mode precedence passed");
