import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import ts from "typescript";
import { Type } from "@sinclair/typebox";

const source = readFileSync(new URL("../src/tools.ts", import.meta.url), "utf8");
const anchor = source.indexOf('name: "focusa_cockpit_projection"');
const registration = source.slice(source.lastIndexOf("pi.registerTool({", anchor), source.indexOf("pi.registerTool({", anchor));
function section(start, end) {
  const i = source.indexOf(start), j = source.indexOf(end, i + start.length);
  assert.ok(i >= 0 && j > i);
  return source.slice(i, j);
}
const helpers = section("function safeErrorText(", "import { registerAgentRuntimeTools") +
  section("function scopedResponseFailureClass(", "function typedTrajectoryScopeMatches(");
const code = ts.transpileModule(helpers + registration, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None },
}).outputText;

for (const [name, response, expected] of [
  ["empty valid projection", { ok: true, status: 200, body: { status: "ok", worksets: [], callgraph: [], steers: [], background: { active: 0, jobs: [] } } }, true],
  ["missing installed route", { ok: false, status: 404, body: null }, false],
  ["denied scope", { ok: false, status: 403, body: { status: "blocked", reason: "scope denied" } }, false],
  ["storage error over HTTP 200", { ok: true, status: 200, body: { status: "error", error: { message: "storage unavailable" } } }, false],
  ["missing JSON", { ok: true, status: 200, body: null }, false],
  ["incomplete projection", { ok: true, status: 200, body: { status: "ok", worksets: [] } }, false],
]) {
  test(`cockpit preserves ${name} instead of manufacturing an empty board`, async () => {
    let tool;
    const deps = {
      pi: { registerTool: (value) => { tool = value; } }, Type,
      getAttachmentRuntime: () => ({ sessionCwd: "/fixture/project" }),
      focusaFetchDetailed: async (path) => { assert.equal(path, "/cockpit/projection"); return response; },
      toolResult: (ok, status, text, data) => ({ ok, status, text, data }),
    };
    new Function(...Object.keys(deps), code)(...Object.values(deps));
    const result = await tool.execute("fixture-call", { project_root: "/fixture/project" });
    assert.equal(result.ok, expected);
    if (!expected) {
      assert.doesNotMatch(result.text, /Flywheel: 0/);
      if (response.status === 404) assert.match(result.text, /404.*installed.*route/i);
      if (name.startsWith("storage")) assert.match(result.text, /storage unavailable/);
    }
  });
}
