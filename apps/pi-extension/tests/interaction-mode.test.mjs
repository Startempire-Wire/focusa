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
const start = commands.indexOf('pi.registerCommand("focusa-mode"');
const end = commands.indexOf('pi.registerCommand("focusa-settings"', start);
assert.ok(start >= 0 && end > start, "interaction mode command must be registered");
const modeCommand = commands.slice(start, end);
assert.match(modeCommand, /const project = attachmentKey\.workstream\.root_scope/);
assert.match(modeCommand, /isProjectRootAuthoritySafe\(project\.root_path\)/);
assert.match(modeCommand, /const cwd = project\.scope_kind === "project" \? project\.root_path : undefined/);
assert.match(modeCommand, /saveConfigOverrides\(cwd, \{ interactionMode: mode \}, scope\)/);
assert.doesNotMatch(modeCommand, /saveConfigOverrides\(ctx\.cwd/);
assert.match(commands, /runtime\.cfg = \{ \.\.\.runtime\.cfg, interactionMode: mode \}/);
assert.match(commands, /priorInteractionModes/);
assert.match(awareness, /`Interaction: \$\{interactionMode\}`/);
console.log("interaction mode contract passed");
