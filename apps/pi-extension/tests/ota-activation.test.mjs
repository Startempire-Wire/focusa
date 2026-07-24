import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { pathToFileURL } from "node:url";

const outDir = mkdtempSync(join(tmpdir(), "focusa-ota-activation-build-"));
const stateHome = mkdtempSync(join(tmpdir(), "focusa-ota-activation-state-"));
const previousStateHome = process.env.XDG_STATE_HOME;
try {
  execFileSync(
    "./node_modules/.bin/tsc",
    ["-p", "tsconfig.json", "--outDir", outDir, "--noEmit", "false", "--module", "ES2022"],
    { cwd: new URL("..", import.meta.url), stdio: "pipe" }
  );
  writeFileSync(join(outDir, "package.json"), '{"type":"module"}\n');
  process.env.XDG_STATE_HOME = stateHome;
  const activation = await import(pathToFileURL(join(outDir, "ota-activation.js")).href);
  const paths = activation.otaActivationPaths();
  mkdirSync(join(stateHome, "focusa", "update"), { recursive: true });

  const commands = new Map();
  const handlers = new Map();
  const queued = [];
  const pi = {
    registerCommand(name, definition) {
      commands.set(name, definition);
    },
    on(name, handler) {
      handlers.set(name, handler);
    },
    sendUserMessage(message, options) {
      queued.push({ message, options });
    },
  };

  writeFileSync(
    paths.restart,
    JSON.stringify({ schema: "focusa.pi_extension_restart_required.v1", version: "9.9.9" })
  );
  const cleanup = activation.registerAutomaticOtaActivation(pi);
  assert.equal(queued.length, 1);
  assert.equal(queued[0].message, "/focusa-activate-updated-extension");
  assert.equal(queued[0].options.deliverAs, "followUp");

  let waited = 0;
  let reloaded = 0;
  const notices = [];
  await commands.get("focusa-activate-updated-extension").handler("", {
    waitForIdle: async () => {
      waited++;
    },
    reload: async () => {
      reloaded++;
    },
    ui: {
      notify(message, level) {
        notices.push({ message, level });
      },
    },
  });
  assert.equal(waited, 1);
  assert.equal(reloaded, 1);
  assert.equal(existsSync(paths.restart), false);
  assert.equal(existsSync(paths.activating), false);
  assert.equal(JSON.parse(readFileSync(paths.receipt, "utf8")).status, "activated");
  assert.match(notices.at(-1).message, /activated automatically/);

  writeFileSync(paths.restart, JSON.stringify({ version: "10.0.0" }));
  await commands.get("focusa-activate-updated-extension").handler("", {
    waitForIdle: async () => {},
    reload: async () => {
      throw new Error("reload rejected");
    },
    ui: {
      notify(message, level) {
        notices.push({ message, level });
      },
    },
  });
  assert.equal(existsSync(paths.restart), true, "failed reload must restore the retry marker");
  assert.equal(existsSync(paths.activating), false);
  assert.match(notices.at(-1).message, /deferred safely/);

  cleanup();
  await handlers.get("session_shutdown")?.({}, {});
  console.log("Pi extension OTA activation test passed");
} finally {
  if (previousStateHome == null) delete process.env.XDG_STATE_HOME;
  else process.env.XDG_STATE_HOME = previousStateHome;
  rmSync(outDir, { recursive: true, force: true });
  rmSync(stateHome, { recursive: true, force: true });
}
