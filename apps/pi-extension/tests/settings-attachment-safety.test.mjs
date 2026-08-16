import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const extensionRoot = fileURLToPath(new URL("..", import.meta.url));
const commandsSource = readFileSync(join(extensionRoot, "src/commands.ts"), "utf8");
const settingsStart = commandsSource.indexOf('pi.registerCommand("focusa-settings"');
const settingsEnd = commandsSource.indexOf('pi.registerCommand("focusa-rollover"', settingsStart);
assert.ok(settingsStart >= 0 && settingsEnd > settingsStart, "settings command block missing");
const settings = commandsSource.slice(settingsStart, settingsEnd);

assert.match(settings, /const settingsAttachmentKey = currentAttachmentKey\(\)/);
assert.match(settings, /const settingsRuntime = getAttachmentRuntime\(settingsAttachmentKey\)/);
assert.doesNotMatch(
  settings,
  /getAttachmentRuntime\(\)/,
  "deferred settings code must not depend on async context"
);
assert.match(settings, /Focusa setting was not saved; prior configuration remains active/);
assert.match(settings, /prior value restored/);
assert.match(settings, /if \(!persistDraft\(\)\) Object\.assign\(draft, priorDraft\)/);
assert.match(settings, /label: "Footer hints"/);
assert.match(settings, /label: "Footer context badge"/);
assert.match(settings, /const buildSimpleItems/);
assert.match(settings, /const buildAdvancedItems/);

const outDir = mkdtempSync(join(tmpdir(), "focusa-settings-runtime-"));
try {
  execFileSync(
    "./node_modules/.bin/tsc",
    ["-p", "tsconfig.json", "--outDir", outDir, "--noEmit", "false", "--module", "ES2022"],
    { cwd: extensionRoot, stdio: "pipe" }
  );
  writeFileSync(join(outDir, "package.json"), '{"type":"module"}\n');
  const state = await import(pathToFileURL(join(outDir, "state.js")).href);
  state.attachmentRuntimeRegistry.reset();
  const keyA = state.makeAttachmentKey({
    projectRoot: "/tmp/settings-a",
    continuityId: "settings-a",
    sessionId: "settings-a",
  });
  const keyB = state.makeAttachmentKey({
    projectRoot: "/tmp/settings-b",
    continuityId: "settings-b",
    sessionId: "settings-b",
  });
  const runtimeA = state.runWithAttachmentRuntime(keyA, () => state.getAttachmentRuntime(keyA));
  const runtimeB = state.runWithAttachmentRuntime(keyB, () => state.getAttachmentRuntime(keyB));

  // Simulate SettingsList.onChange after AsyncLocalStorage context is gone.
  assert.throws(() => state.getAttachmentRuntime(), /attachment_runtime_key_required/);
  runtimeA.cfg = { ...(runtimeA.cfg || {}), footerHints: false };
  assert.equal(runtimeA.cfg.footerHints, false);
  assert.notEqual(runtimeB.cfg?.footerHints, false, "deferred update crossed attachment scope");
  assert.equal(state.getAttachmentRuntime(keyA).cfg.footerHints, false);
  assert.strictEqual(state.getAttachmentRuntime(keyB), runtimeB);
} finally {
  rmSync(outDir, { recursive: true, force: true });
}

console.log("settings attachment safety: PASS");
