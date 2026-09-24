import assert from "node:assert/strict";
import fs from "node:fs";
import vm from "node:vm";
import ts from "typescript";

// Execute the production registration, not a duplicate prompt implementation.
const url = new URL("../src/turns.ts", import.meta.url);
const source = fs.readFileSync(url, "utf8");
const ast = ts.createSourceFile(url.pathname, source, ts.ScriptTarget.Latest, true);
const registrations = [];
function visit(node) {
  if (ts.isCallExpression(node) && node.expression.getText(ast) === "pi.on" &&
      node.arguments[0]?.text === "before_agent_start") registrations.push(node);
  ts.forEachChild(node, visit);
}
visit(ast);
assert.equal(registrations.length, 1);
const compiled = ts.transpileModule(registrations[0].getText(ast), {
  compilerOptions: { target: ts.ScriptTarget.ES2022, alwaysStrict: true },
}).outputText;

for (const cacheSafe of [true, false]) {
  for (const seenFirst of [true, false]) {
    const runtime = {
      focusaAvailable: true,
      seenFirstBeforeAgentStart: seenFirst,
      cfg: { cacheSafePromptLayoutEnabled: cacheSafe },
    };
    let handler;
    let projectRoot = "/tmp/project-one";
    const captures = [];
    vm.runInNewContext(compiled, {
      pi: { on: (name, fn) => { assert.equal(name, "before_agent_start"); handler = fn; } },
      getAttachmentRuntime: () => runtime,
      hardGateVitalProjectRoot: () => projectRoot,
      resolveInteractionMode: () => ({ mode: "terminal-guided", source: "test" }),
      getSessionCwd: () => projectRoot,
      buildCachedRecentTurnsSlice: () => "recent-turn-fixture",
      toolOutputVisibleRecapReason: () => "recap-fixture",
      formatWorkpointContextSections: () => ["workpoint-fixture"],
      buildFocusaUtilityCard: () => "utility-fixture",
      cacheSafetyMonitor: { captureSystemPrompt: (key, prompt) => captures.push({ key, prompt }) },
      cacheSessionKey: () => "fixture-session",
      nativeSessionAllowsNonessentialPersistence: () => false,
      queueTraceTelemetry: () => {},
      getTurnCount: () => 1,
    });
    let reads = 0;
    const event = Object.freeze(Object.defineProperty({ type: "before_agent_start" }, "systemPrompt", {
      enumerable: true,
      get() { reads++; return "original Pi prompt"; },
    }));
    const result = handler(event, {});
    assert.equal(reads, 1, "read the input prompt once; never mutate it");
    assert.equal(event.systemPrompt, "original Pi prompt");
    assert.equal(Object.getOwnPropertyDescriptor(event, "systemPrompt").set, undefined);
    assert.ok(result.systemPrompt.startsWith("original Pi prompt\n"));
    for (const section of ["Focusa Cognitive Guidance", "Focusa Interaction Mode", "Focusa Workpoint Continuity Law"])
      assert.ok(result.systemPrompt.includes(section), section);
    assert.equal(captures[0].prompt, result.systemPrompt);
    assert.equal(captures[0].key, "fixture-session");
    for (const dynamic of [projectRoot, "recent-turn-fixture", "workpoint-fixture", "utility-fixture", "recap-fixture"])
      assert.equal(result.systemPrompt.includes(dynamic), !cacheSafe, dynamic);
    projectRoot = "/tmp/project-two";
    const second = handler(event, {});
    assert.equal(second.systemPrompt === result.systemPrompt, cacheSafe);
  }
}
console.log("PASS: getter-only frozen Pi event, returned prompt, telemetry, legacy layout, startup and stable prefix");
