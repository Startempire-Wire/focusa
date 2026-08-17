#!/usr/bin/env node
// Full tool-health sweep: probe every route the agent card advertises.
// Reports status + classification; exit 1 on any 5xx or 404.
const BASE = process.env.FOCUSA_API_BASE || "http://127.0.0.1:8787/v1";
const ROOT_BASE = "http://127.0.0.1:8787";
const SCOPED = {
  "X-Scope-Project-Root": "/srv/focusa",
  "X-Scope-Continuity-Id": "cont-probe",
};
const results = [];
async function probe(method, path, body) {
  const ROOT_PATHS = ["/llms.txt"];
  const url = path.startsWith("/v1/")
    ? `${ROOT_BASE}${path}`
    : ROOT_PATHS.includes(path)
      ? `${ROOT_BASE}${path}`
      : `${BASE}${path}`;
  const res = await fetch(url, {
    method,
    headers: { "Content-Type": "application/json", ...SCOPED },
    body: body ? JSON.stringify(body) : undefined,
  });
  let json = null;
  try { json = await res.json(); } catch {}
  results.push({ method, path, status: res.status, json });
  return { status: res.status, json };
}
const main = async () => {
  // GETs across every family
  const gets = [
    "/cockpit/projection", "/credentials/providers",
    "/health", "/info", "/llms.txt", // root-level (no /v1)
    "/agent/capabilities",
    "/runtime-constitution", "/background-jobs", "/adapters",
    "/worksets", "/direction/operations", "/work-items/providers",
    "/silent-sessions", "/silent-sessions/capabilities",
    "/metacognition/status", "/work-loop/status?summary_only=true",
    "/workpoint/current", "/trajectory/view", "/project/list",
    "/compaction/controller-epoch", // POST-only; probe below, "/v1/events/stream",
  ];
  for (const path of gets) await probe("GET", path);
  // POSTs with minimal valid payloads
  await probe("POST", "/completion-claims/evaluate", {
    schema: "focusa.completion_claim.v1", work_item_id: "probe",
    acceptance_atoms: ["a"], evidence_refs: [], receipts: [], claim_text: "x",
  });
  await probe("POST", "/workstreams/migrate", { preview: true });
  await probe("POST", "/silent-sessions/fanout", {
    work_items: ["a", "b"], multiplier: 2,
  });
  await probe("POST", "/predictions", {
    scope: {
      root_scope: { scope_kind: "project", scope_id: "focusa", root_path: "/srv/focusa", canonical_name: "focusa", fingerprint: "probe" },
      continuity_id: "cont-probe",
    },
    prediction_type: "wall_clock", context_refs: ["probe"],
    predicted_outcome: "probe", confidence: 0.5,
    recommended_action: "probe", why: "probe",
  });
  await probe("POST", "/metacognition/capture", {
    kind: "reflection", content: "probe", rationale: "probe", confidence: 0.5, strategy_class: "probe",
  });
  const bad = results.filter((r) => r.status >= 500 || r.status === 404);
  for (const r of results) {
    console.log(`${r.status}  ${r.method.padEnd(4)} ${r.path}`);
  }
  console.log(`\n${results.length - bad.length}/${results.length} healthy; ${bad.length} broken`);
  process.exit(bad.length ? 1 : 0);
};
main();
