import assert from "node:assert/strict";
import http from "node:http";
import { registerTools } from "../apps/pi-extension/src/tools.ts";
import { S } from "../apps/pi-extension/src/state.ts";

type ToolDef = {
  name: string;
  execute: (...args: any[]) => Promise<any>;
};

async function main() {
const tools = new Map<string, ToolDef>();
const pi = {
  registerTool(def: ToolDef) {
    tools.set(def.name, def);
  },
} as any;

registerTools(pi);

const required = [
  "focusa_tree_head",
  "focusa_tree_path",
  "focusa_tree_snapshot_state",
  "focusa_tree_restore_state",
  "focusa_tree_diff_context",
  "focusa_metacog_capture",
  "focusa_metacog_retrieve",
  "focusa_metacog_reflect",
  "focusa_metacog_plan_adjust",
  "focusa_metacog_evaluate_outcome",
];

for (const name of required) {
  assert.ok(tools.has(name), `missing tool registration: ${name}`);
}

const calls: Array<{ method: string; path: string; headers: http.IncomingHttpHeaders; body: any }> = [];
const pathHits = new Map<string, number>();

const server = http.createServer((req, res) => {
  const chunks: Buffer[] = [];
  req.on("data", (c) => chunks.push(Buffer.from(c)));
  req.on("end", () => {
    const bodyText = Buffer.concat(chunks).toString("utf8");
    let body: any = null;
    try {
      body = bodyText ? JSON.parse(bodyText) : null;
    } catch {
      body = bodyText || null;
    }

    calls.push({
      method: String(req.method || "GET"),
      path: String(req.url || "/"),
      headers: req.headers,
      body,
    });

    const path = String(req.url || "").split("?")[0];
    pathHits.set(path, (pathHits.get(path) || 0) + 1);
    const ok = (payload: any, status = 200) => {
      res.writeHead(status, { "content-type": "application/json" });
      res.end(JSON.stringify(payload));
    };

    if (path === "/v1/work-loop/status") return ok({ status: "running", writer_id: "writer-test" });
    if (path === "/v1/lineage/head") return ok({ head: "clt-head-1", branch_id: "main", session_id: "sess-1" });
    if (path.startsWith("/v1/lineage/path/")) return ok({ head: "clt-head-1", depth: 3, path: ["root", "mid", "leaf"] });
    if (path === "/v1/focus/snapshots") return ok({ snapshot_id: "snap-1", created_at: "2026-04-21T00:00:00Z" });
    if (path === "/v1/focus/snapshots/restore") return ok({ status: "restored", snapshot_id: body?.snapshot_id || "snap-1", conflicts: [] });
    if (path === "/v1/focus/snapshots/diff") return ok({ checksum_changed: true, decisions_delta: ["d1"] });
    if (path === "/v1/metacognition/capture") return ok({ capture_id: "cap-1", stored: true });
    if (path === "/v1/metacognition/retrieve") {
      if ((pathHits.get(path) || 0) === 1) return ok({ code: "RETRIEVE_UNAVAILABLE", reason: "transient" }, 503);
      return ok({ candidates: [{ id: "c1" }, { id: "c2" }], ranked_by: "score", retrieval_budget: 5 });
    }
    if (path === "/v1/metacognition/reflect") return ok({ reflection_id: "r-1", hypotheses: ["h1"], strategy_updates: ["u1"] });
    if (path === "/v1/metacognition/adjust") return ok({ adjustment_id: "a-1", next_step_policy: "safe" });
    if (path === "/v1/metacognition/evaluate") return ok({ evaluation_id: "e-1", result: "improved", promote_learning: true });

    return ok({ code: "NOT_FOUND" }, 404);
  });
});

await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", () => resolve()));
const addr = server.address();
assert.ok(addr && typeof addr === "object", "server not bound");
const baseUrl = `http://127.0.0.1:${addr.port}/v1`;
(S as any).cfg = { focusaApiBaseUrl: baseUrl, focusaApiTimeoutMs: 5000, focusaToken: "" };

const checkEnvelope = (toolName: string, r: any, endpoint: string) => {
  assert.equal(r?.details?.ok, true, `${toolName} should succeed`);
  assert.equal(r?.details?.code, "OK", `${toolName} should return code OK`);
  assert.equal(r?.details?.tool, toolName, `${toolName} should include tool name`);
  assert.equal(r?.details?.endpoint, endpoint, `${toolName} should include endpoint`);
  assert.ok(r?.details?.timestamp, `${toolName} should include timestamp`);
};

const rTreeHead = await tools.get("focusa_tree_head")!.execute("id", { session_id: "sess-1" });
const rTreePath = await tools.get("focusa_tree_path")!.execute("id", { clt_node_id: "clt-9" });
const rSnapshot = await tools.get("focusa_tree_snapshot_state")!.execute("id", { clt_node_id: "clt-9", snapshot_reason: "test" });
const rRestore = await tools.get("focusa_tree_restore_state")!.execute("id", { snapshot_id: "snap-1", restore_mode: "merge" });
const rDiff = await tools.get("focusa_tree_diff_context")!.execute("id", { from_snapshot_id: "snap-0", to_snapshot_id: "snap-1" });
const rCapture = await tools.get("focusa_metacog_capture")!.execute("id", { kind: "note", content: "x", confidence: 0.8 });
const rRetrieve = await tools.get("focusa_metacog_retrieve")!.execute("id", { current_ask: "next", scope_tags: ["x"], k: 999 });
const rReflect = await tools.get("focusa_metacog_reflect")!.execute("id", { turn_range: "1..5" });
const rAdjust = await tools.get("focusa_metacog_plan_adjust")!.execute("id", { reflection_id: "r-1", selected_updates: ["u1"] });
const rEvaluate = await tools.get("focusa_metacog_evaluate_outcome")!.execute("id", { adjustment_id: "a-1", observed_metrics: ["m1"] });

checkEnvelope("focusa_tree_head", rTreeHead, "/v1/lineage/head");
checkEnvelope("focusa_tree_path", rTreePath, "/v1/lineage/path/{clt_node_id}");
checkEnvelope("focusa_tree_snapshot_state", rSnapshot, "/v1/focus/snapshots");
checkEnvelope("focusa_tree_restore_state", rRestore, "/v1/focus/snapshots/restore");
checkEnvelope("focusa_tree_diff_context", rDiff, "/v1/focus/snapshots/diff");
checkEnvelope("focusa_metacog_capture", rCapture, "/v1/metacognition/capture");
checkEnvelope("focusa_metacog_retrieve", rRetrieve, "/v1/metacognition/retrieve");
checkEnvelope("focusa_metacog_reflect", rReflect, "/v1/metacognition/reflect");
checkEnvelope("focusa_metacog_plan_adjust", rAdjust, "/v1/metacognition/adjust");
checkEnvelope("focusa_metacog_evaluate_outcome", rEvaluate, "/v1/metacognition/evaluate");

const getCall = (match: (c: { method: string; path: string; headers: http.IncomingHttpHeaders; body: any }) => boolean, label: string) => {
  const found = calls.find(match);
  assert.ok(found, `missing outbound request for ${label}`);
  return found!;
};

const headCall = getCall((c) => c.method === "GET" && c.path.startsWith("/v1/lineage/head"), "focusa_tree_head");
assert.ok(headCall.path.includes("session_id=sess-1"), "focusa_tree_head should forward session_id query");

const pathCall = getCall((c) => c.method === "GET" && c.path === "/v1/lineage/path/clt-9", "focusa_tree_path");
assert.equal(pathCall.body, null, "GET lineage path should not send body");

const snapshotCall = getCall((c) => c.method === "POST" && c.path === "/v1/focus/snapshots", "focusa_tree_snapshot_state");
assert.deepEqual(snapshotCall.body, { clt_node_id: "clt-9", snapshot_reason: "test" }, "snapshot request body should match schema");

const restoreCall = getCall((c) => c.method === "POST" && c.path === "/v1/focus/snapshots/restore", "focusa_tree_restore_state");
assert.deepEqual(restoreCall.body, { snapshot_id: "snap-1", restore_mode: "merge" }, "restore request body should match schema");

const diffCall = getCall((c) => c.method === "POST" && c.path === "/v1/focus/snapshots/diff", "focusa_tree_diff_context");
assert.deepEqual(diffCall.body, { from_snapshot_id: "snap-0", to_snapshot_id: "snap-1" }, "diff request body should match schema");

const captureCall = getCall((c) => c.method === "POST" && c.path === "/v1/metacognition/capture", "focusa_metacog_capture");
assert.deepEqual(captureCall.body, { kind: "note", content: "x", confidence: 0.8 }, "capture request body should match schema");

const retrieveCall = getCall((c) => c.method === "POST" && c.path === "/v1/metacognition/retrieve" && (c.body?.k === 50), "focusa_metacog_retrieve");
assert.deepEqual(retrieveCall.body, { current_ask: "next", scope_tags: ["x"], k: 50 }, "retrieve should clamp k and preserve scope_tags");

const reflectCall = getCall((c) => c.method === "POST" && c.path === "/v1/metacognition/reflect", "focusa_metacog_reflect");
assert.deepEqual(reflectCall.body, { turn_range: "1..5", failure_classes: [] }, "reflect should default failure_classes to []");

const adjustCall = getCall((c) => c.method === "POST" && c.path === "/v1/metacognition/adjust", "focusa_metacog_plan_adjust");
assert.deepEqual(adjustCall.body, { reflection_id: "r-1", selected_updates: ["u1"] }, "adjust request body should match schema");

const evaluateCall = getCall((c) => c.method === "POST" && c.path === "/v1/metacognition/evaluate", "focusa_metacog_evaluate_outcome");
assert.deepEqual(evaluateCall.body, { adjustment_id: "a-1", observed_metrics: ["m1"] }, "evaluate request body should match schema");

const restoreWriter = String(restoreCall.headers["x-focusa-writer-id"] || "").trim();
const captureWriter = String(captureCall.headers["x-focusa-writer-id"] || "").trim();
assert.ok(restoreWriter.length > 0, "restore should pass a writer id header");
assert.ok(captureWriter.length > 0, "capture should pass a writer id header");
assert.equal(captureWriter, restoreWriter, "writer id should be consistent across write tools in one session");

const retrieveHits = pathHits.get("/v1/metacognition/retrieve") || 0;
assert.equal(retrieveHits, 2, "metacog retrieve should retry once on transient status");
const statusHits = pathHits.get("/v1/work-loop/status") || 0;
assert.equal(statusHits, 6, "writer-backed tools should resolve writer id per write operation");

const invalidPath = await tools.get("focusa_tree_path")!.execute("id", { clt_node_id: "" });
assert.equal(invalidPath?.details?.ok, false, "empty clt_node_id should fail");
assert.equal(invalidPath?.details?.code, "CLT_NODE_NOT_FOUND", "empty clt_node_id should map to CLT_NODE_NOT_FOUND");

const invalidRestoreMode = await tools.get("focusa_tree_restore_state")!.execute("id", { snapshot_id: "snap-1", restore_mode: "invalid-mode" });
assert.equal(invalidRestoreMode?.details?.ok, false, "invalid restore mode should fail");
assert.equal(invalidRestoreMode?.details?.code, "INVALID_REQUEST", "invalid restore mode should map to INVALID_REQUEST");

server.close();
console.log("SPEC80 extension runtime contract: PASS");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
