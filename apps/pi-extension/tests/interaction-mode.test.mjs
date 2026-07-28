import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("..", import.meta.url));
const config = readFileSync(`${root}/src/config.ts`, "utf8");
const commands = readFileSync(`${root}/src/commands.ts`, "utf8");
const awareness = readFileSync(`${root}/src/awareness.ts`, "utf8");
for (const mode of ["canvas-guided", "terminal-guided", "headless"]) {
  assert.match(config, new RegExp(mode));
  assert.match(commands, new RegExp(mode));
}
assert.match(commands, /registerCommand\("focusa-mode"/);
assert.match(commands, /saveConfigOverrides\(ctx\.cwd, \{ interactionMode: mode \}, scope\)/);
assert.match(commands, /runtime\.cfg = \{ \.\.\.runtime\.cfg, interactionMode: mode \}/);
assert.match(commands, /priorInteractionModes/);
assert.match(awareness, /`Interaction: \$\{interactionMode\}`/);
console.log("interaction mode contract passed");
