import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const outDir = mkdtempSync(join(tmpdir(), "focusa-ota-activation-build-"));
const stateHome = mkdtempSync(join(tmpdir(), "focusa-ota-activation-state-"));
const previousStateHome = process.env.XDG_STATE_HOME;
const settle = () => new Promise((resolve) => setTimeout(resolve, 20));
try {
  execFileSync(
    "./node_modules/.bin/tsc",
    ["-p", "tsconfig.json", "--outDir", outDir, "--noEmit", "false", "--module", "ES2022"],
    { cwd: new URL("..", import.meta.url), stdio: "pipe" }
  );
  writeFileSync(join(outDir, "package.json"), '{"type":"module","version":"9.9.9"}\n');
  process.env.XDG_STATE_HOME = stateHome;
  const activation = await import(pathToFileURL(join(outDir, "ota-activation.js")).href);
  const paths = activation.otaActivationPaths();
  mkdirSync(join(stateHome, "focusa", "update"), { recursive: true });

  const handlers = new Map();
  const inertPi = {
    registerCommand() {
      assert.fail("OTA activation must not register a conversational control command");
    },
    sendUserMessage() {
      assert.fail("OTA activation must not inject a conversation turn");
    },
    on(name, handler) {
      handlers.set(name, handler);
    },
  };

  writeFileSync(paths.restart, JSON.stringify({ version: "9.9.9" }));
  const cleanupStartup = activation.registerAutomaticOtaActivation(inertPi);
  assert.equal(existsSync(paths.restart), false);
  const startupReceipt = JSON.parse(readFileSync(paths.receipt, "utf8"));
  assert.equal(startupReceipt.status, "activated");
  assert.equal(startupReceipt.activation, "process_start");
  cleanupStartup();

  rmSync(paths.receipt, { force: true });
  writeFileSync(paths.restart, JSON.stringify({ version: "10.0.0" }));
  let reloads = 0;
  const reloadHandlers = new Map();
  const reloadPi = {
    on(name, handler) {
      reloadHandlers.set(name, handler);
    },
    async reloadWhenIdle() {
      reloads++;
    },
  };
  const cleanupReload = activation.registerAutomaticOtaActivation(reloadPi);
  await settle();
  assert.equal(reloads, 1);
  assert.equal(existsSync(paths.restart), false);
  assert.equal(existsSync(paths.activating), true);
  assert.equal(existsSync(paths.receipt), false, "old runtime cannot claim activation");
  cleanupReload();

  writeFileSync(join(outDir, "package.json"), '{"type":"module","version":"10.0.0"}\n');
  const cleanupNewRuntime = activation.registerAutomaticOtaActivation(inertPi);
  assert.equal(existsSync(paths.activating), false);
  assert.equal(JSON.parse(readFileSync(paths.receipt, "utf8")).activation, "safe_idle_reload");
  cleanupNewRuntime();

  rmSync(paths.receipt, { force: true });
  writeFileSync(paths.restart, JSON.stringify({ version: "11.0.0" }));
  const failedPi = {
    on() {},
    async reloadWhenIdle() {
      throw new Error("reload rejected");
    },
  };
  const cleanupFailure = activation.registerAutomaticOtaActivation(failedPi);
  await settle();
  assert.equal(existsSync(paths.restart), true, "failed reload must restore retry marker");
  assert.equal(existsSync(paths.activating), false);
  assert.equal(existsSync(paths.receipt), false);
  cleanupFailure();
  await handlers.get("session_shutdown")?.({}, {});
  console.log("Pi extension silent OTA activation test passed");
} finally {
  if (previousStateHome == null) delete process.env.XDG_STATE_HOME;
  else process.env.XDG_STATE_HOME = previousStateHome;
  rmSync(outDir, { recursive: true, force: true });
  rmSync(stateHome, { recursive: true, force: true });
}
