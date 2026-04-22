import assert from "node:assert/strict";
import { registerTools } from "../apps/pi-extension/src/tools.ts";
import { S } from "../apps/pi-extension/src/state.ts";

type ToolDef = {
  name: string;
  execute: (...args: any[]) => Promise<any>;
};

function requireOk(label: string, result: any) {
  assert.equal(result?.details?.ok, true, `${label} should succeed`);
  assert.equal(result?.details?.code, "OK", `${label} should return OK`);
  return result;
}

async function main() {
  const tools = new Map<string, ToolDef>();
  const pi = {
    registerTool(def: ToolDef) {
      tools.set(def.name, def);
    },
  } as any;

  registerTools(pi);
  (S as any).cfg = {
    focusaApiBaseUrl: process.env.FOCUSA_API_URL || "http://127.0.0.1:8787/v1",
    focusaApiTimeoutMs: 5000,
    focusaToken: "",
  };

  const now = Date.now();
  const ask = `spec81 live chain ${now}`;

  const treeHead = requireOk("tree_head", await tools.get("focusa_tree_head")!.execute("id", {}));
  assert.ok(treeHead.details.response, "tree_head should return body");

  const snapA = requireOk(
    "snapshot_a",
    await tools.get("focusa_tree_snapshot_state")!.execute("id", { snapshot_reason: `spec81-a-${now}` }),
  );
  const snapB = requireOk(
    "snapshot_b",
    await tools.get("focusa_tree_snapshot_state")!.execute("id", { snapshot_reason: `spec81-b-${now}` }),
  );

  const snapAId = String(snapA.details.response?.snapshot_id || "");
  const snapBId = String(snapB.details.response?.snapshot_id || "");
  assert.ok(snapAId.length > 0, "snapshot_a id required");
  assert.ok(snapBId.length > 0, "snapshot_b id required");

  const diff = requireOk(
    "tree_diff",
    await tools.get("focusa_tree_diff_context")!.execute("id", {
      from_snapshot_id: snapAId,
      to_snapshot_id: snapBId,
    }),
  );
  assert.ok(diff.details.response?.from_snapshot_id, "diff should return from_snapshot_id");

  const capture = requireOk(
    "metacog_capture",
    await tools.get("focusa_metacog_capture")!.execute("id", {
      kind: "spec81_live_chain",
      content: ask,
      rationale: "runtime chain smoke",
      confidence: 0.9,
      strategy_class: "spec81",
    }),
  );
  assert.ok(capture.details.response?.capture_id, "capture should return capture_id");

  const retrieve = requireOk(
    "metacog_retrieve",
    await tools.get("focusa_metacog_retrieve")!.execute("id", {
      current_ask: ask,
      scope_tags: ["spec81"],
      k: 5,
    }),
  );
  const candidates = retrieve.details.response?.candidates || [];
  assert.ok(Array.isArray(candidates), "retrieve should return candidates array");

  const reflect = requireOk(
    "metacog_reflect",
    await tools.get("focusa_metacog_reflect")!.execute("id", {
      turn_range: "1-3",
      failure_classes: ["spec81_runtime"],
    }),
  );
  const reflectionId = String(reflect.details.response?.reflection_id || "");
  assert.ok(reflectionId.length > 0, "reflect should return reflection_id");

  const updates = Array.isArray(reflect.details.response?.strategy_updates)
    ? reflect.details.response.strategy_updates
    : ["spec81 default update"];

  const adjust = requireOk(
    "metacog_adjust",
    await tools.get("focusa_metacog_plan_adjust")!.execute("id", {
      reflection_id: reflectionId,
      selected_updates: updates,
    }),
  );
  const adjustmentId = String(adjust.details.response?.adjustment_id || "");
  assert.ok(adjustmentId.length > 0, "adjust should return adjustment_id");

  const evaluate = requireOk(
    "metacog_evaluate",
    await tools.get("focusa_metacog_evaluate_outcome")!.execute("id", {
      adjustment_id: adjustmentId,
      observed_metrics: ["spec81_metric_gain"],
    }),
  );
  assert.equal(evaluate.details.response?.promote_learning, true, "evaluate should promote with observed metric");

  console.log(
    JSON.stringify(
      {
        status: "ok",
        workflow: "spec81_extension_live_chain",
        snapshot_a: snapAId,
        snapshot_b: snapBId,
        reflection_id: reflectionId,
        adjustment_id: adjustmentId,
        promote_learning: evaluate.details.response?.promote_learning ?? false,
        candidate_count: Array.isArray(candidates) ? candidates.length : 0,
      },
      null,
      2,
    ),
  );
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
