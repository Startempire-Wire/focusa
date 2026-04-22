import assert from "node:assert/strict";
import http from "node:http";
import { registerTools } from "../apps/pi-extension/src/tools.ts";
import { S } from "../apps/pi-extension/src/state.ts";

type ToolDef = {
  name: string;
  description?: string;
  parameters?: any;
  execute: (...args: any[]) => Promise<any>;
};

type Call = { method: string; path: string; headers: http.IncomingHttpHeaders; body: any };

async function main() {
  const tools = new Map<string, ToolDef>();
  const pi = {
    registerTool(def: ToolDef) {
      tools.set(def.name, def);
    },
  } as any;

  registerTools(pi);

  const requiredNew = [
    "focusa_tree_recent_snapshots",
    "focusa_tree_snapshot_compare_latest",
    "focusa_metacog_recent_reflections",
    "focusa_metacog_recent_adjustments",
    "focusa_metacog_loop_run",
    "focusa_metacog_doctor",
  ];
  for (const name of requiredNew) {
    assert.ok(tools.has(name), `missing new desirable tool: ${name}`);
  }

  assert.match(tools.get("focusa_tree_head")!.description || "", /Best safe starting point/i);
  assert.match(tools.get("focusa_metacog_retrieve")!.description || "", /Best safe search tool/i);
  assert.match(tools.get("focusa_tree_snapshot_compare_latest")!.description || "", /one move/i);

  const calls: Call[] = [];
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

      calls.push({ method: String(req.method || "GET"), path: String(req.url || "/"), headers: req.headers, body });
      const path = String(req.url || "").split("?")[0];

      const ok = (payload: any, status = 200) => {
        res.writeHead(status, { "content-type": "application/json" });
        res.end(JSON.stringify(payload));
      };

      if (path === "/v1/work-loop/status") return ok({ status: "running", writer_id: "writer-desirable" });
      if (path === "/v1/focus/snapshots/recent") {
        return ok({
          status: "ok",
          total: 1,
          snapshots: [{ snapshot_id: "snap-prev", clt_node_id: "clt-prev", created_at: "2026-04-22T00:00:00Z", checksum: "chk-prev", state_version: 9, lineage_head: "head-prev" }],
        });
      }
      if (path === "/v1/focus/snapshots") {
        return ok({ status: "ok", snapshot_id: "snap-new", clt_node_id: "clt-now", created_at: "2026-04-22T00:01:00Z" });
      }
      if (path === "/v1/focus/snapshots/diff") {
        return ok({ status: "ok", checksum_changed: true, clt_node_changed: false, version_delta: 3, decisions_delta: { changed: true } });
      }
      if (path === "/v1/metacognition/reflections/recent") {
        return ok({ status: "ok", total: 1, reflections: [{ reflection_id: "refl-latest", created_at: "2026-04-22T00:02:00Z", turn_range: "1-3", failure_classes: ["drift"], strategy_updates: ["add verification"] }] });
      }
      if (path === "/v1/metacognition/adjustments/recent") {
        return ok({ status: "ok", total: 1, adjustments: [{ adjustment_id: "adj-latest", reflection_id: "refl-latest", created_at: "2026-04-22T00:03:00Z", selected_updates: ["add verification"] }] });
      }
      if (path === "/v1/metacognition/capture") {
        return ok({ capture_id: "cap-1", status: "accepted" });
      }
      if (path === "/v1/metacognition/retrieve") {
        return ok({
          candidates: [{ capture_id: "cap-1", kind: "workflow_signal", confidence: 0.7, has_rationale: true, summary: "signal", score: 2, rank: 1, evidence_refs: [] }],
          next_cursor: null,
          total_candidates: 1,
          retrieval_budget: { truncated: false },
        });
      }
      if (path === "/v1/metacognition/reflect") {
        return ok({ reflection_id: "refl-1", hypotheses: ["h1"], strategy_updates: ["u1", "u2"] });
      }
      if (path === "/v1/metacognition/adjust") {
        return ok({ adjustment_id: "adj-1", next_step_policy: body?.selected_updates || [] });
      }
      if (path === "/v1/metacognition/evaluate") {
        return ok({ evaluation_id: "eval-1", result: "improved", promote_learning: true, delta_scorecard: { metrics_observed: body?.observed_metrics || [] } });
      }

      return ok({ status: "error", code: "NOT_FOUND" }, 404);
    });
  });

  await new Promise<void>((resolve) => server.listen(0, "127.0.0.1", () => resolve()));
  const addr = server.address();
  if (!addr || typeof addr === "string") throw new Error("failed to bind mock server");
  const baseUrl = `http://127.0.0.1:${addr.port}/v1`;
  (S as any).cfg = { focusaApiBaseUrl: baseUrl, focusaApiTimeoutMs: 5000, focusaToken: "" };

  const recentSnaps = await tools.get("focusa_tree_recent_snapshots")!.execute("id", { limit: 3 });
  assert.equal(recentSnaps?.details?.ok, true);
  assert.match(String(recentSnaps?.content?.[0]?.text || ""), /ids=snap-prev/);
  assert.match(String(recentSnaps?.content?.[0]?.text || ""), /next_tools=/);

  const compareLatest = await tools.get("focusa_tree_snapshot_compare_latest")!.execute("id", { snapshot_reason: "desirability test" });
  assert.equal(compareLatest?.details?.ok, true);
  assert.equal(compareLatest?.details?.response?.baseline_snapshot_id, "snap-prev");
  assert.match(String(compareLatest?.content?.[0]?.text || ""), /changed=yes/);

  const recentRefl = await tools.get("focusa_metacog_recent_reflections")!.execute("id", {});
  assert.equal(recentRefl?.details?.ok, true);
  assert.match(String(recentRefl?.content?.[0]?.text || ""), /refl-latest/);

  const recentAdj = await tools.get("focusa_metacog_recent_adjustments")!.execute("id", {});
  assert.equal(recentAdj?.details?.ok, true);
  assert.match(String(recentAdj?.content?.[0]?.text || ""), /adj-latest/);

  const loopRun = await tools.get("focusa_metacog_loop_run")!.execute("id", {
    current_ask: "spec87 desirability",
    turn_range: "1-3",
    observed_metrics: ["metric-a"],
  });
  assert.equal(loopRun?.details?.ok, true);
  assert.equal(loopRun?.details?.response?.evaluate?.promote_learning, true);
  assert.match(String(loopRun?.content?.[0]?.text || ""), /reflection=refl-1/);

  const doctor = await tools.get("focusa_metacog_doctor")!.execute("id", { current_ask: "spec87 desirability" });
  assert.equal(doctor?.details?.ok, true);
  assert.equal(doctor?.details?.response?.diagnostics?.candidate_count, 1);
  assert.match(String(doctor?.content?.[0]?.text || ""), /top_kind=workflow_signal/);

  const extraDoctor = await tools.get("focusa_metacog_doctor")!.execute("id", { current_ask: "x", extra: true });
  assert.equal(extraDoctor?.details?.ok, false);
  assert.equal(extraDoctor?.details?.code, "SCHEMA_INVALID");

  const diffCall = calls.find((c) => c.path === "/v1/focus/snapshots/diff");
  assert.ok(diffCall, "compare_latest should call diff endpoint");
  assert.deepEqual(diffCall!.body, { from_snapshot_id: "snap-prev", to_snapshot_id: "snap-new" });

  const doctorCall = calls.find((c) => c.path === "/v1/metacognition/retrieve" && c.body?.summary_only === true);
  assert.ok(doctorCall, "doctor should call retrieve with summary_only");

  server.close();
  console.log("SPEC87 extension desirability contract: PASS");
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
