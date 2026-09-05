import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { mkdtempSync, rmSync, symlinkSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const projectDir = fileURLToPath(new URL("..", import.meta.url));
const outDir = mkdtempSync(join(tmpdir(), "focusa-compaction-registration-rebind-"));

function extensionApi() {
  const handlers = new Map();
  const tools = [];
  const commands = [];
  const shortcuts = [];
  const flags = [];
  let stale = false;
  const pi = new Proxy(
    {
      appendEntry() {},
      getAllTools() {
        if (stale) {
          throw new Error("This extension ctx is stale after session replacement or reload.");
        }
        return tools;
      },
      getFlag() {
        return undefined;
      },
      on(event, handler) {
        const registered = handlers.get(event) ?? [];
        registered.push(handler);
        handlers.set(event, registered);
      },
      registerCommand(name) {
        commands.push(name);
      },
      registerEntryRenderer() {},
      registerFlag(name) {
        flags.push(name);
      },
      registerMessageRenderer() {},
      registerProvider() {},
      registerShortcut(name) {
        shortcuts.push(name);
      },
      registerTool(tool) {
        tools.push(tool);
      },
    },
    {
      get(target, property, receiver) {
        if (Reflect.has(target, property)) return Reflect.get(target, property, receiver);
        return () => undefined;
      },
    }
  );
  return {
    commands,
    flags,
    handlers,
    markStale() {
      stale = true;
    },
    pi,
    shortcuts,
    tools,
  };
}

try {
  symlinkSync(join(projectDir, "node_modules"), join(outDir, "node_modules"), "dir");
  execFileSync(
    "./node_modules/.bin/tsc",
    ["-p", "tsconfig.json", "--outDir", outDir, "--noEmit", "false", "--module", "ES2022"],
    { cwd: projectDir, stdio: "pipe" }
  );
  writeFileSync(join(outDir, "package.json"), '{"type":"module"}\n');

  const autoCompaction = await import(pathToFileURL(join(outDir, "auto-compaction.js")).href);
  const focusaPiBridge = await import(pathToFileURL(join(outDir, "index.js")).href);
  autoCompaction.resetCompactionLeaseForTest();

  const first = extensionApi();
  assert.equal(autoCompaction.registerAutoCompaction(first.pi), true);
  assert.ok(first.handlers.has("session_start"));
  assert.ok(first.handlers.has("session_shutdown"));

  const activeDuplicate = extensionApi();
  assert.equal(autoCompaction.registerAutoCompaction(activeDuplicate.pi), false);
  assert.equal(activeDuplicate.handlers.size, 0, "an active duplicate must register no handlers");

  first.markStale();
  const replacement = extensionApi();
  assert.equal(autoCompaction.registerAutoCompaction(replacement.pi), true);
  assert.ok(replacement.handlers.has("session_start"));
  assert.ok(replacement.handlers.has("session_shutdown"));
  assert.ok(
    replacement.handlers.size >= first.handlers.size,
    "the replacement API must receive the complete coordinator handler surface"
  );

  autoCompaction.resetCompactionLeaseForTest();
  const firstExtension = extensionApi();
  focusaPiBridge.default(firstExtension.pi);
  const firstToolManifest = firstExtension.tools.map((tool) => tool.name).sort();
  assert.ok(firstToolManifest.length > 0, "the first extension API must receive native tools");
  assert.ok(firstToolManifest.every((name) => name.startsWith("focusa_")));

  const duplicateExtension = extensionApi();
  focusaPiBridge.default(duplicateExtension.pi);
  assert.equal(duplicateExtension.tools.length, 0, "an active duplicate must receive no native tools");
  assert.equal(duplicateExtension.handlers.size, 0, "an active duplicate must receive no hooks");

  firstExtension.markStale();
  const replacementExtension = extensionApi();
  focusaPiBridge.default(replacementExtension.pi);
  assert.deepEqual(
    replacementExtension.tools.map((tool) => tool.name).sort(),
    firstToolManifest,
    "session replacement must preserve the complete native tool manifest"
  );
  assert.deepEqual(replacementExtension.commands.sort(), firstExtension.commands.sort());
  assert.deepEqual(replacementExtension.shortcuts.sort(), firstExtension.shortcuts.sort());
  assert.deepEqual(replacementExtension.flags.sort(), firstExtension.flags.sort());
  assert.deepEqual(
    [...replacementExtension.handlers.keys()].sort(),
    [...firstExtension.handlers.keys()].sort(),
    "session replacement must preserve the complete hook manifest"
  );

  autoCompaction.resetCompactionLeaseForTest();
} finally {
  rmSync(outDir, { recursive: true, force: true });
}
