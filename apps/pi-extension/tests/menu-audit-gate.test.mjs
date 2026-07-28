import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../../..", import.meta.url));
const commands = readFileSync(`${root}/apps/pi-extension/src/commands.ts`, "utf8");
const session = readFileSync(`${root}/apps/pi-extension/src/session.ts`, "utf8");
const manifest = JSON.parse(
  readFileSync(`${root}/docs/contracts/48-49-focusa-pi-menu-inventory.json`, "utf8")
);
const digest = createHash("sha256")
  .update(commands + session)
  .digest("hex");
assert.equal(manifest.source_sha256, digest, "menu source changed without regenerated audit");
const registered = [...commands.matchAll(/pi\.registerCommand\("([^"]+)"/g)].map((m) => m[1]);
const inventoried = manifest.items.filter((x) => x.kind === "command").map((x) => x.id);
assert.deepEqual(inventoried.sort(), registered.sort());
const settings = manifest.items.filter((x) => x.kind === "setting");
assert.ok(settings.length > 0);
for (const item of settings) {
  assert.ok(
    item.owner &&
      item.user_job &&
      item.placement &&
      item.rationale &&
      item.help &&
      item.scope &&
      item.risk &&
      item.interaction_test
  );
  assert.ok(item.values.length > 0, `${item.id} lacks allowed-value coverage`);
  assert.match(commands, new RegExp(`id === "${item.id}"`), `${item.id} lacks callback coverage`);
}
assert.equal(
  manifest.items.filter((x) => ["select", "confirm", "input", "custom"].includes(x.kind)).length,
  3
);
assert.ok(manifest.after.simple_settings < manifest.baseline.simple_settings);
assert.equal(manifest.after.simple_settings, 8);
assert.deepEqual(
  Object.keys(manifest.migration).sort(),
  ["otaProfile", "vitalInfoPromptSurfaces", "workLoopStatusHeartbeatMs"].sort()
);
assert.match(commands, /try \{/);
assert.match(commands, /prior configuration remains active/);
assert.match(commands, /prior value restored/);
console.log(
  `menu audit gate: PASS (${manifest.items.length} entries; ${settings.length} setting placements)`
);
