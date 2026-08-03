import assert from "node:assert/strict";
import { readFileSync, rmSync, writeFileSync } from "node:fs";
import { resolve } from "node:path";
import { pathToFileURL } from "node:url";
import { visibleWidth } from "@earendil-works/pi-tui";
import ts from "typescript";

const root = resolve(import.meta.dirname, "..");
const shellSource = readFileSync(resolve(root, "src/mission-canvas-shell.ts"), "utf8");
const token = `${process.pid}-${Date.now()}`;
const viewName = `.mission-canvas-shell-view-${token}.mjs`;
const shellPath = resolve(root, `.mission-canvas-shell-runtime-${token}.mjs`);
const viewPath = resolve(root, viewName);
const compiled = ts
  .transpileModule(shellSource, {
    compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
  })
  .outputText.replace("./mission-canvas-view.js", `./${viewName}`);

writeFileSync(shellPath, compiled);
writeFileSync(
  viewPath,
  `export class MissionCanvasView {
    constructor() {}
    setConversation() {}
    handleInput() {}
    invalidate() {}
    dispose() {}
    render(width) { return Array.from({ length: 100 }, () => "C".repeat(width)); }
  }\n`
);

try {
  const { MissionCanvasShell } = await import(`${pathToFileURL(shellPath).href}?v=${Date.now()}`);
  const notifications = [];
  const sent = [];
  let disabled = 0;
  let done = 0;
  let restoredDraft = "";
  const ctx = {
    model: { id: "runtime-test-model" },
    sessionManager: { getEntries: () => [] },
    ui: {
      setTitle() {},
      setFooter() {},
      setEditorText(value) {
        restoredDraft = value;
      },
      notify(message, level) {
        notifications.push({ message, level });
      },
    },
  };
  const pi = {
    async sendUserMessage(message) {
      sent.push(message);
    },
  };
  const theme = { fg: (_name, text) => text };
  const shell = new MissionCanvasShell(
    {},
    theme,
    () => {},
    () => 20,
    () => done++,
    async () => ({}),
    pi,
    ctx,
    () => {},
    () => {},
    async () => {
      disabled++;
    }
  );

  shell.focused = false;
  assert.equal(shell.focused, false);
  assert.equal(shell.input.focused, false);
  shell.focused = true;
  assert.equal(shell.input.focused, true);

  for (let width = 1; width < 40; width++) {
    const lines = shell.render(width);
    assert(
      lines.every((line) => visibleWidth(line) <= width),
      `Mission Canvas shell emitted a line wider than ${width} columns`
    );
    assert(lines.length <= 20, "Mission Canvas shell must not scroll beyond terminal height");
  }

  shell.handleInput("\x1b[6~");
  assert(shell.scrollOffset > 0, "PageDown must expose Canvas rows below the viewport");
  assert(shell.render(39).length <= 20);

  for (const character of "/mission-canvas off") shell.handleInput(character);
  shell.handleInput("\n");
  await new Promise((resolvePromise) => setImmediate(resolvePromise));
  assert.equal(disabled, 1, "off command must invoke the local Canvas controller");
  assert.deepEqual(sent, [], "off command must not spend an agent turn");

  for (const character of "continue the work") shell.handleInput(character);
  shell.handleInput("\n");
  await new Promise((resolvePromise) => setImmediate(resolvePromise));
  assert.deepEqual(sent, ["continue the work"]);
  assert.deepEqual(notifications, []);

  for (const character of "preserve this draft") shell.handleInput(character);
  shell.handleInput("\x1b");
  await new Promise((resolvePromise) => setImmediate(resolvePromise));
  assert.equal(done, 1, "Escape must close the Canvas immediately");
  assert.equal(restoredDraft, "preserve this draft");
  console.log("Mission Canvas shell runtime: PASS (stable overlay, bounded height, IME focus, local close)");
} finally {
  rmSync(shellPath, { force: true });
  rmSync(viewPath, { force: true });
}
