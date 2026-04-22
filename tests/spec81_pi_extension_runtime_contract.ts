import assert from "node:assert/strict";
import http from "node:http";
import { registerTools } from "../apps/pi-extension/src/tools.ts";
import { S } from "../apps/pi-extension/src/state.ts";

type ToolDef = {
  name: string;
  execute: (...args: any[]) => Promise<any>;
};

type Call = { method: string; path: string; headers: http.IncomingHttpHeaders; body: any };

function checkEnvelope(toolName: string, r: any, endpoint: string) {
  assert.equal(r?.details?.tool, toolName, `${toolName} should include tool name`);
  assert.equal(r?.details?.endpoint, endpoint, `${toolName} should include endpoint`);
  assert.ok(r?.details?.timestamp, `${toolName} should include timestamp`);
}

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

  const calls: Call[] = [];
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
      if (path === "/v1/focus/snapshots") return ok({ snapshot_id: "snap-1", created_at: "2026-04-22T00:00:00Z" });
      if (path === "/v1/focus/snapshots/restore") return ok({ status: "restored", snapshot_id: body?.snapshot_id || "snap-1", conflicts: [] });
      if (path === "/v1/focus/snapshots/diff") return ok({ checksum_changed: true, decisions_delta: { changed: true } });
      if (path === "/v1/metacognition/capture") return ok({ capture_id: "cap-1", status: "accepted" });
      if (path === "/v1/metacognition/retrieve") {
        if ((pathHits.get(path) || 0) === 1) return ok({ status: "error", code: "TEMP_UNAVAILABLE" }, 503);
        return ok({
          candidates: [{ capture_id: "cap-1", kind: "note", confidence: 0.8, has_rationale: true, summary: "x", score: 2, rank: 1, evidence_refs: [] }],
          next_cursor: null,
          total_candidates: 1,
          retrieval_budget: { truncated: false },
        });
      }
      if (path === "/v1/metacognition/reflect") return ok({ reflection_id: "refl-1", hypotheses: ["h1"], strategy_updates: ["u1"] });
      if (path === "/v1/metacognition/adjust") return ok({ adjustment_id: "adj-1", next_step_policy: body?.selected_updates || [] });
      if (path === "/v1/metacognition/evaluate") return ok({ evaluation_id: "eval-1", result: "improved", promote_learning: true });

      return ok({ status: "error", code: "NOT_FOUND" }, 404);
    });
  });

  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", () => resolve()));
  const addr = server.address();
  if (!addr || typeof addr === "string") throw new Error("failed to bind mock server");
  const baseUrl = `http://127.0.0.1:${addr.port}/v1`;
  (S as any).cfg = { focusaApiBaseUrl: baseUrl, focusaApiTimeoutMs: 5000, focusaToken: "" };

  const rTreeHead = await tools.get("focusa_tree_head")!.execute("id", { session_id: "sess-1" });
  const rTreePath = await tools.get("focusa_tree_path")!.execute("id", { clt_node_id: "clt-9" });
  const rSnapshot = await tools.get("focusa_tree_snapshot_state")!.execute("id", { clt_node_id: "clt-9", snapshot_reason: "test" });
  const rRestore = await tools.get("focusa_tree_restore_state")!.execute("id", { snapshot_id: "snap-1", restore_mode: "merge" });
  const rDiff = await tools.get("focusa_tree_diff_context")!.execute("id", { from_snapshot_id: "snap-0", to_snapshot_id: "snap-1" });
  const rCapture = await tools.get("focusa_metacog_capture")!.execute("id", { kind: "note", content: "x", rationale: "why", confidence: 0.8 });
  const rRetrieve = await tools.get("focusa_metacog_retrieve")!.execute("id", { current_ask: "x", scope_tags: ["tag-1"], k: 4 });
  const rReflect = await tools.get("focusa_metacog_reflect")!.execute("id", { turn_range: "1..5", failure_classes: [] });
  const rAdjust = await tools.get("focusa_metacog_plan_adjust")!.execute("id", { reflection_id: "refl-1", selected_updates: ["u1"] });
  const rEvaluate = await tools.get("focusa_metacog_evaluate_outcome")!.execute("id", { adjustment_id: "adj-1", observed_metrics: ["m1"] });

  for (const [name, endpoint, result] of [
    ["focusa_tree_head", "/v1/lineage/head", rTreeHead],
    ["focusa_tree_path", "/v1/lineage/path/{clt_node_id}", rTreePath],
    ["focusa_tree_snapshot_state", "/v1/focus/snapshots", rSnapshot],
    ["focusa_tree_restore_state", "/v1/focus/snapshots/restore", rRestore],
    ["focusa_tree_diff_context", "/v1/focus/snapshots/diff", rDiff],
    ["focusa_metacog_capture", "/v1/metacognition/capture", rCapture],
    ["focusa_metacog_retrieve", "/v1/metacognition/retrieve", rRetrieve],
    ["focusa_metacog_reflect", "/v1/metacognition/reflect", rReflect],
    ["focusa_metacog_plan_adjust", "/v1/metacognition/adjust", rAdjust],
    ["focusa_metacog_evaluate_outcome", "/v1/metacognition/evaluate", rEvaluate],
  ] as const) {
    assert.equal(result?.details?.ok, true, `${name} should succeed`);
    assert.equal(result?.details?.code, "OK", `${name} should return code OK`);
    checkEnvelope(name, result, endpoint);
    assert.equal(typeof result?.content?.[0]?.text, "string", `${name} should return human-readable content`);
    assert.ok(String(result?.content?.[0]?.text || "").trim().length > 12, `${name} should expose meaningful summary text`);
  }

  assert.match(String(rTreeHead.content?.[0]?.text || ""), /branch=main/, "tree head summary should include branch");
  assert.match(String(rTreeHead.content?.[0]?.text || ""), /session=sess-1/, "tree head summary should include session");
  assert.match(String(rRetrieve.content?.[0]?.text || ""), /top_capture=cap-1/, "retrieve summary should include top capture");
  assert.match(String(rRetrieve.content?.[0]?.text || ""), /top_kind=note/, "retrieve summary should include top kind");
  assert.match(String(rEvaluate.content?.[0]?.text || ""), /observed_metrics=m1/, "evaluate summary should include observed metric preview");

  const getCall = (match: (c: Call) => boolean, label: string) => {
    const found = calls.find(match);
    assert.ok(found, `missing HTTP call for ${label}`);
    return found!;
  };

  const headCall = getCall((c) => c.method === "GET" && c.path === "/v1/lineage/head?session_id=sess-1", "focusa_tree_head");
  assert.equal(headCall.body, null, "GET lineage head should not send body");

  const pathCall = getCall((c) => c.method === "GET" && c.path === "/v1/lineage/path/clt-9", "focusa_tree_path");
  assert.equal(pathCall.body, null, "GET lineage path should not send body");

  const snapshotCall = getCall((c) => c.method === "POST" && c.path === "/v1/focus/snapshots", "focusa_tree_snapshot_state");
  assert.deepEqual(snapshotCall.body, { clt_node_id: "clt-9", snapshot_reason: "test" });

  const captureCall = getCall((c) => c.method === "POST" && c.path === "/v1/metacognition/capture", "focusa_metacog_capture");
  assert.deepEqual(captureCall.body, { kind: "note", content: "x", rationale: "why", confidence: 0.8 });

  const retrieveHits = pathHits.get("/v1/metacognition/retrieve") || 0;
  assert.equal(retrieveHits, 2, "retrieve should retry once on transient status");

  const writerStatusHits = pathHits.get("/v1/work-loop/status") || 0;
  assert.equal(writerStatusHits, 6, "writer-backed tools should resolve writer id per write operation");

  const restoreWriter = String(getCall((c) => c.path === "/v1/focus/snapshots/restore", "restore").headers["x-focusa-writer-id"] || "").trim();
  const evaluateWriter = String(getCall((c) => c.path === "/v1/metacognition/evaluate", "evaluate").headers["x-focusa-writer-id"] || "").trim();
  assert.ok(restoreWriter.length > 0, "restore should pass a writer id header");
  assert.equal(restoreWriter, evaluateWriter, "writer id should be consistent across write tools");

  const invalidPath = await tools.get("focusa_tree_path")!.execute("id", { clt_node_id: "" });
  assert.equal(invalidPath?.details?.ok, false, "blank clt_node_id should fail");
  assert.equal(invalidPath?.details?.code, "SCHEMA_INVALID", "blank clt_node_id should map to SCHEMA_INVALID");

  const invalidSession = await tools.get("focusa_tree_head")!.execute("id", { session_id: "bad space" });
  assert.equal(invalidSession?.details?.ok, false, "bad session_id should fail");
  assert.equal(invalidSession?.details?.code, "SCHEMA_INVALID");

  const extraHead = await tools.get("focusa_tree_head")!.execute("id", { session_id: "sess-1", unexpected: true });
  assert.equal(extraHead?.details?.ok, false, "extra tree_head param should fail");
  assert.equal(extraHead?.details?.code, "SCHEMA_INVALID");

  const invalidReason = await tools.get("focusa_tree_snapshot_state")!.execute("id", { snapshot_reason: "x".repeat(1000) });
  assert.equal(invalidReason?.details?.ok, false, "too-long reason should fail");
  assert.equal(invalidReason?.details?.code, "SCHEMA_INVALID");

  const invalidCapture = await tools.get("focusa_metacog_capture")!.execute("id", { kind: "note", content: "x", confidence: 2 });
  assert.equal(invalidCapture?.details?.ok, false, "bad confidence should fail");
  assert.equal(invalidCapture?.details?.code, "SCHEMA_INVALID");

  const extraCapture = await tools.get("focusa_metacog_capture")!.execute("id", { kind: "note", content: "x", extra: "bad" });
  assert.equal(extraCapture?.details?.ok, false, "extra capture param should fail");
  assert.equal(extraCapture?.details?.code, "SCHEMA_INVALID");

  const invalidRetrieve = await tools.get("focusa_metacog_retrieve")!.execute("id", { current_ask: "x", scope_tags: Array.from({ length: 30 }, (_, i) => `tag-${i}`) });
  assert.equal(invalidRetrieve?.details?.ok, false, "too many scope tags should fail");
  assert.equal(invalidRetrieve?.details?.code, "SCHEMA_INVALID");

  const extraRetrieve = await tools.get("focusa_metacog_retrieve")!.execute("id", { current_ask: "x", k: 3, cursor: "nope" });
  assert.equal(extraRetrieve?.details?.ok, false, "extra retrieve param should fail");
  assert.equal(extraRetrieve?.details?.code, "SCHEMA_INVALID");

  const invalidReflect = await tools.get("focusa_metacog_reflect")!.execute("id", { turn_range: "@@bad@@" });
  assert.equal(invalidReflect?.details?.ok, false, "bad turn range should fail");
  assert.equal(invalidReflect?.details?.code, "SCHEMA_INVALID");

  server.close();
  console.log("SPEC81 extension runtime contract: PASS");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
