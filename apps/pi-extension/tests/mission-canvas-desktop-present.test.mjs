import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const read = (name) => readFileSync(resolve(root, `src/${name}`), "utf8");
const commands = read("commands.ts");

assert.match(commands, /desktopPresent\.exactWorkstream/);
assert.match(commands, /compatibilityProjection\.open/);
assert.match(commands, /"\/v1\/mission-canvas\/rich-host\/resolution"/);
assert.match(commands, /"\/v1\/mission-canvas\/rich-host\/focus"/);
assert.match(commands, /"\/v1\/mission-canvas\/rich-host\/launch"/);
assert.match(commands, /if \(action === \"desktop\" \|\| action === \"open\"\)/);
assert.match(commands, /if \(action === \"terminal\" \|\| action === \"overlay\" \|\| action === \"compat\"\)/);

assert.match(commands, /if \(!hostContext\) \{/);
assert.match(commands, /resolution\.payload\?\.selected_renderer !== DESKTOP_TAURI_RENDERER/);
assert.match(commands, /if \(focusResponse\.ok\) return;/);
assert.match(commands, /if \(\[404, 409\]\.includes\(focusResponse\.status\)\) \{/);
assert.match(commands, /await compatibilityProjection\.open\(ctx\);/);

console.log("Mission Canvas desktop present handoff control assertions passed");
