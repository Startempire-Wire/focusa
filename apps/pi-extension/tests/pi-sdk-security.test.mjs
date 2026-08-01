import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const pkg = JSON.parse(readFileSync(join(root, "package.json"), "utf8"));
const lock = JSON.parse(readFileSync(join(root, "package-lock.json"), "utf8"));
const installed = JSON.parse(
  readFileSync(
    join(
      root,
      "node_modules",
      "@earendil-works",
      "pi-coding-agent",
      "node_modules",
      "brace-expansion",
      "package.json"
    ),
    "utf8"
  )
);

assert.equal(pkg.devDependencies["@earendil-works/pi-coding-agent"], "0.82.1");
assert.equal(pkg.devDependencies["@earendil-works/pi-tui"], "0.82.1");
assert.equal(pkg.overrides["brace-expansion"], "5.0.8");
assert.equal(
  lock.packages["node_modules/@earendil-works/pi-coding-agent/node_modules/brace-expansion"].version,
  "5.0.8"
);
assert.equal(installed.version, "5.0.8", "installed Pi shrinkwrap dependency must be patched");

console.log("PASS: Pi SDK compatibility upgrade and installed shrinkwrap security overlay");
