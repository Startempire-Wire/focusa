#!/usr/bin/env node
// Full tool-health sweep: probe every route the agent card advertises.
// Proves response contracts, not capability admission; unexpected failures fail CI.
import { pathToFileURL } from "node:url";

export function classifyRouteResponse({ method, path, status, json }) {
  // #526/#609: prove the exact unimplemented boundary, never grant availability
  // from an expected failure or exempt another server error.
  if (method === "GET" && path === "/credentials/providers") {
    return status === 501 && json?.status === "unsupported" &&
      json?.code === "credential_provider_registry_not_implemented" &&
      Array.isArray(json.providers) && json.providers.length === 0
      ? "unavailable" : "broken";
  }
  return status >= 500 || status === 404 || status === 405 ? "broken" : "responsive";
}
const BASE = process.env.FOCUSA_API_BASE || "http://127.0.0.1:8787/v1";
const ROOT_BASE = BASE.replace(/\/v1\/?$/, "");
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
    "/silent-sessions/completions?since_seq=0&limit=1",
    "/silent-sessions/wait?session_id=route-health-probe&since_seq=0&timeout_ms=100",
    "/metacognition/status", "/work-loop/status?summary_only=true",
    "/workpoint/current", "/trajectory/view", "/project/list",
  ];
  for (const path of gets) await probe("GET", path);
  const contractProbe = await probe("GET", "/ontology/tool-contracts");
  const contracts = contractProbe.json?.contracts || [];
  const expectedPurposes = {
    focusa_workset_projection: /deterministic membership, requirement-disposition, and settlement projection/,
    focusa_callgraph_observe: /CallGraph run's ledger row, dispatches, paths, and deterministic replay frontier/,
    focusa_credentials_verify: /Credential Authority.*without exposing secret values/,
    focusa_cockpit_projection: /Worksets, CallGraph frontiers, direction steers, and background jobs/,
    focusa_fast_forward: /deterministic fanout plan.*silent-session lanes/,
  };
  for (const [name, expected] of Object.entries(expectedPurposes)) {
    const contract = contracts.find((item) => item.name === name);
    if (!contract || !expected.test(String(contract.purpose || ""))) {
      results.push({ method: "ASSERT", path: `/ontology/tool-contracts#${name}`, status: 500 });
    }
  }
  // POSTs with minimal payloads; validation errors prove route registration.
  await probe("POST", "/completion-claims/evaluate", {
    schema: "focusa.completion_claim.v1", work_item_id: "probe",
    acceptance_atoms: ["a"], evidence_refs: [], receipts: [], claim_text: "x",
  });
  await probe("POST", "/workstreams/migrate", { preview: true });
  await probe("POST", "/silent-sessions/fanout", {
    work_items: ["a", "b"], multiplier: 2,
  });
  await probe("POST", "/silent-sessions/sweep-completions", {});
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
  const classified = results.map((r) => ({ ...r, classification: classifyRouteResponse(r) }));
  const bad = classified.filter((r) => r.classification === "broken");
  const unavailable = classified.filter((r) => r.classification === "unavailable");
  for (const r of classified) {
    console.log(`${r.status}  ${r.method.padEnd(4)} ${r.path} [${r.classification}]`);
  }
  console.log(`\n${results.length - bad.length - unavailable.length}/${results.length} responsive; ${unavailable.length} unavailable; ${bad.length} broken (response contracts only, not execution readiness)`);
  process.exit(bad.length ? 1 : 0);
};
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) await main();
