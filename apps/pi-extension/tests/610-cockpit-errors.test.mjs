import assert from "node:assert/strict";
import test from "node:test";
import { loadRegisteredTool } from "./helpers/registered-tool-harness.mjs";

const helpers = [
  ["function safeErrorText(", "import { registerAgentRuntimeTools"],
  ["function scopedResponseFailureClass(", "function typedTrajectoryScopeMatches("],
];

for (const [name, response, expected] of [
  ["empty valid projection", { ok: true, status: 200, body: { status: "ok", worksets: [], callgraph: [], steers: [], background: { active: 0, jobs: [] } } }, true],
  ["missing installed route", { ok: false, status: 404, body: null }, false],
  ["denied scope", { ok: false, status: 403, body: { status: "blocked", reason: "scope denied" } }, false],
  ["storage error over HTTP 200", { ok: true, status: 200, body: { status: "error", error: { message: "storage unavailable" } } }, false],
  ["missing JSON", { ok: true, status: 200, body: null }, false],
  ["incomplete projection", { ok: true, status: 200, body: { status: "ok", worksets: [] } }, false],
]) {
  test(`cockpit preserves ${name} instead of manufacturing an empty board`, async () => {
    const deps = {
      getAttachmentRuntime: () => ({ sessionCwd: "/fixture/project" }),
      focusaFetchDetailed: async (path) => { assert.equal(path, "/cockpit/projection"); return response; },
      toolResult: (ok, status, text, data) => ({ ok, status, text, data }),
    };
    const tool = loadRegisteredTool("focusa_cockpit_projection", deps, helpers);
    const result = await tool.execute("fixture-call", { project_root: "/fixture/project" });
    assert.equal(result.ok, expected);
    if (!expected) {
      assert.doesNotMatch(result.text, /Flywheel: 0/);
      if (response.status === 404) assert.match(result.text, /404.*installed.*route/i);
      if (name.startsWith("storage")) assert.match(result.text, /storage unavailable/);
    }
  });
}
