import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import ts from "typescript";
import { Type } from "@sinclair/typebox";

const source = readFileSync(new URL("../src/tools.ts", import.meta.url), "utf8");
function section(start, end) {
  const i = source.indexOf(start);
  const j = source.indexOf(end, i + start.length);
  assert.ok(i >= 0 && j > i);
  return source.slice(i, j);
}
const implementation = section("function focusaApiV1Base()", "// FOCUSA_FIX-vuop") +
  section("async function focusaFetchDetailed(", "function formatWorkLoopBudgetRemaining");
const compiled = ts.transpileModule(implementation, {
  compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None },
}).outputText;

test("self-model handler preserves healthy, empty and precise failure responses", async () => {
  const generated = readFileSync(new URL("../src/generated/spec138-operations.ts", import.meta.url), "utf8");
  const operations = {};
  new Function("exports", ts.transpileModule(generated, {
    compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.CommonJS },
  }).outputText)(operations);
  const anchor = source.indexOf('name: "focusa_epistemic_operation"');
  const registration = source.slice(source.lastIndexOf("pi.registerTool({", anchor), source.indexOf("pi.registerTool({", anchor));
  const helpers = section("function safeErrorText(", "import { registerAgentRuntimeTools") +
    section("function scopedResponseFailureClass(", "function typedTrajectoryScopeMatches(");
  const code = ts.transpileModule(helpers + registration, {
    compilerOptions: { target: ts.ScriptTarget.ES2022, module: ts.ModuleKind.None },
  }).outputText;
  for (const [status, body, ok, reason] of [
    [200, { status: "completed", canonical: true, self_model: { observations: 1 } }, true, ""],
    [200, { status: "completed", canonical: true, self_model: null }, true, ""],
    [403, { status: "blocked", reason: "scope denied" }, false, "scope denied"],
    [404, { status: "blocked", error: "operation route unavailable" }, false, "operation route unavailable"],
    [409, { status: "blocked", error: { message: "storage unavailable" } }, false, "storage unavailable"],
    [200, { status: "blocked", reason: "scope denied" }, false, "scope denied"],
  ]) {
    let tool;
    const deps = {
      ...operations, pi: { registerTool: (value) => { tool = value; } }, Type,
      resolveFocusaToolProjectRoot: async (root) => root,
      projectRootConfirmationGate: () => null,
      buildProjectWorkstreamKey: (root, continuity) => ({ root, continuity }),
      scopedQueryParams: (scope) => new URLSearchParams({ project_root: scope.root, continuity_id: scope.continuity }),
      focusaFetchDetailed: async (endpoint) => {
        const url = new URL(endpoint, "http://fixture.invalid");
        assert.equal(url.pathname, "/v1/self-model");
        assert.equal(url.searchParams.get("project_root"), "/fixture/project");
        assert.equal(url.searchParams.get("continuity_id"), "fixture-continuity");
        return { ok: status === 200, status, body };
      },
    };
    new Function(...Object.keys(deps), code)(...Object.values(deps));
    const result = await tool.execute("fixture-call", {
      operation_id: "self_model.get", project_root: "/fixture/project", continuity_id: "fixture-continuity",
    });
    assert.equal(result.details.ok, ok);
    const expectedResponse = body.self_model === null
      ? {
          ...body,
          supported: true,
          state: "empty",
          reason_code: "no_learning_data",
          next_step: "Record or evaluate a scoped prediction before expecting self-model estimates.",
          self_model: {},
        }
      : body;
    assert.deepEqual(result.details.response, expectedResponse);
    if (body.self_model === null) {
      assert.equal(result.details.response.state, "empty");
      assert.equal(result.details.response.reason_code, "no_learning_data");
    }
    if (!ok) {
      assert.ok(result.content[0].text.includes(reason));
      assert.ok(result.content[0].text.includes(`HTTP ${status}`));
      assert.ok(result.details.next_tools.length > 0);
    }
  }
});

test("scoped fetch uses exactly one API version and preserves attachment headers", async () => {
  for (const base of ["http://fixture.invalid", "http://fixture.invalid/v1", "http://fixture.invalid/v1/"]) {
    for (const path of ["/self-model?project_root=%2Ffixture", "/v1/self-model?project_root=%2Ffixture"]) {
      let observed;
      const key = {
        workstream: { root_scope: { root_path: "/fixture/project", scope_id: "fixture-scope", scope_kind: "project" }, continuity_id: "fixture-continuity" },
        session_id: "fixture-session", attachment_id: "fixture-attachment",
      };
      const deps = {
        getAttachmentRuntime: () => ({ cfg: { focusaApiBaseUrl: base } }),
        timeoutBudgetForRoute: () => 1000,
        currentProjectBindingDecision: () => ({ state: "BOUND" }),
        currentAttachmentKey: () => key,
        getActiveWorkpointPacket: () => null,
        resolveCanonicalMarkerProjectRoot: () => null,
        normalizeProjectRoot: (value) => value || "",
        getLastProjectIdentity: () => null, getLastProjectVerify: () => null,
        getContinuityId: () => "fixture-continuity",
        isProjectRootAuthoritySafe: () => false,
        getSessionFrameKey: () => "fixture-session",
        fetch: async (url, opts) => {
          observed = { url, opts };
          return { ok: false, status: 404, json: async () => { throw new Error("empty HTTP error"); } };
        },
      };
      const request = new Function(...Object.keys(deps), `${compiled}; return focusaFetchDetailed;`)(...Object.values(deps));
      const result = await request(path);
      assert.equal(observed.url, "http://fixture.invalid/v1/self-model?project_root=%2Ffixture");
      assert.equal(observed.opts.headers["x-scope-project-root"], "/fixture/project");
      assert.equal(observed.opts.headers["x-scope-continuity-id"], "fixture-continuity");
      assert.equal(observed.opts.headers["x-scope-attachment-id"], "fixture-attachment");
      assert.equal(result.status, 404);
      assert.equal(result.body.failure_class, "non_json_http_error");
      assert.match(result.body.error, /HTTP 404/);
    }
  }
});
