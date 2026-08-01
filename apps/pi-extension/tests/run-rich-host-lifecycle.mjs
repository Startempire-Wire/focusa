import { rm } from "node:fs/promises";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const output = join(root, ".tmp-rich-host-test");
await rm(output, { recursive: true, force: true });
const compiler = join(root, "node_modules", "typescript", "bin", "tsc");
const compiled = spawnSync(process.execPath, [compiler, "-p", "tsconfig.json", "--noEmit", "false", "--outDir", ".tmp-rich-host-test"], {
  cwd: root,
  stdio: "inherit",
});
if (compiled.status !== 0) process.exit(compiled.status ?? 1);
const tests = [
  "tests/rich-host-lifecycle.test.mjs",
  "tests/rich-host-entrypoint.integration.mjs",
  "tests/rich-host-frontend.test.mjs",
  "tests/rich-host-stress.test.mjs",
  "tests/uiai-eval-harness.test.mjs",
];
let status = 0;
for (const test of tests) {
  const tested = spawnSync(process.execPath, ["--no-warnings", test], { cwd: root, stdio: "inherit" });
  if (tested.status !== 0) { status = tested.status ?? 1; break; }
}
await rm(output, { recursive: true, force: true });
process.exit(status);
