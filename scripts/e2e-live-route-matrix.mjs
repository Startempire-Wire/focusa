#!/usr/bin/env node
// E2E live-route matrix — exercises every closed-bug fix + every feature
// against the running daemon. Machine-readable report; exit 1 on any gap.
const BASE = process.env.FOCUSA_API_BASE || "http://127.0.0.1:8787/v1";

async function call(method, path, body) {
  const res = await fetch(`${BASE}${path}`, {
    method,
    headers: { "Content-Type": "application/json" },
    body: body ? JSON.stringify(body) : undefined,
  });
  let json = null;
  try { json = await res.json(); } catch {}
  return { status: res.status, json };
}

const checks = [];
function check(name, fn) { checks.push({ name, fn }); }

check("health", async () => {
  const r = await call("GET", "/health");
  return r.json?.ok === true;
});

check("constitution (#256)", async () => {
  const r = await call("GET", "/runtime-constitution");
  const c = r.json?.constitution;
  return !!c && typeof c.content_digest === "string" && c.content_digest.startsWith("sha256:");
});

check("bg create/complete loop (#311)", async () => {
  const created = await call("POST", "/background-jobs", {
    name: "e2e-probe", command: "true", cwd: "/tmp",
  });
  const job = created.json?.job;
  if (!job?.job_id) return false;
  const completed = await call("POST", `/background-jobs/${job.job_id}/complete`, { exit_code: 0 });
  const done = completed.json?.job;
  const event = completed.json?.completion_event;
  return done?.status === "completed" && event?.event_type === "background_job_completion"
    && typeof event.output_tail === "string";
});

check("completion claim evaluate (#276/#277)", async () => {
  const r = await call("POST", "/completion-claims/evaluate", {
    schema: "focusa.completion_claim.v1",
    work_item_id: "e2e",
    acceptance_atoms: ["a"], evidence_refs: [], receipts: [],
    claim_text: "x",
  });
  return r.json?.verdict?.allow === false;
});

check("workstream migrate preview (#125)", async () => {
  const r = await call("POST", "/workstreams/migrate", { preview: true });
  return r.json?.status === "preview" && Array.isArray(r.json?.candidates);
});

check("callgraph validate (#254)", async () => {
  const r = await call("POST", "/callgraphs/validate", {
    schema: "focusa.callgraph.v1", graph_id: "e2e", revision: 1,
    scope: { project_root: "/r", continuity_id: "c" },
    mission_ref: "m", title: "t", description: "t",
    entry_frame_ids: ["a"],
    frames: [{ frame_id: "a", name: "a", purpose: "t", kind: "agent",
      input_schema: {}, return_schema: {}, preconditions: [], postconditions: [],
      side_effect_class: "none", capability_refs: [],
      acceptance: { acceptance_atoms: ["a1"], verifier: null } }],
    edges: [], policies: {}, required_evidence: [],
    created_at: "t", created_by: { authority_kind: "operator", reference: "op" },
  });
  return r.json?.valid === true;
});

check("adapter registry (#254 slice 10)", async () => {
  const r = await call("POST", "/adapters", {
    adapter_id: "e2e-adapter", model: "e2e-model", harness: "pi",
    capabilities: ["shell"], healthy: true, last_seen: "2026-08-16T00:00:00Z",
  });
  const list = await call("GET", "/adapters");
  return r.json?.status === "registered"
    && list.json?.adapters?.some((a) => a.adapter_id === "e2e-adapter");
});

check("fanout (#312)", async () => {
  const r = await call("POST", "/silent-sessions/fanout", {
    work_items: ["a", "b", "c", "d"], multiplier: 2,
  });
  const p = r.json?.plan;
  return p?.session_count === 3 && p?.worker_lane_count === 2
    && p?.sessions?.[0]?.role === "orchestrator" && p?.sessions?.[0]?.frame_kind === "agent";
});

check("direction operations (#291)", async () => {
  const r = await call("POST", "/direction/operations", {
    operation: "steer", target_ref: "wp-e2e", direction: "prioritize",
    rationale: "e2e", scope: "workpoint", evidence_ref: "docs/evidence/e2e.md",
  });
  const list = await call("GET", "/direction/operations");
  return r.json?.status === "recorded" && Array.isArray(list.json?.operations);
});

check("compaction epoch (#112)", async () => {
  const r = await call("GET", "/compaction/controller-epoch");
  return r.status < 500;
});

check("error envelope parity (#261)", async () => {
  const r = await call("POST", "/direction/operations", {
    operation: "steer", target_ref: "x", direction: "y", rationale: "z", scope: "s",
  });
  return r.json?.failure_class === "direction_verification_failed"
    && !!r.json?.retry_posture && !!r.json?.safe_recovery;
});

check("bg wait route (#311)", async () => {
  const created = await call("POST", "/background-jobs", {
    name: "e2e-wait", command: "true", cwd: "/tmp",
  });
  const job = created.json?.job;
  await call("POST", `/background-jobs/${job.job_id}/complete`, { exit_code: 0 });
  const waited = await call("GET", `/background-jobs/wait?job_id=${job.job_id}&timeout_ms=5000`);
  return waited.json?.status === "done" && waited.json?.completion_event?.job_id === job.job_id;
});

check("callgraph export jsonl/todo (#287)", async () => {
  const stored = await call("POST", "/callgraphs", {
    schema: "focusa.callgraph.v1", graph_id: "e2e-export", revision: 1,
    scope: { project_root: "/r", continuity_id: "c" },
    mission_ref: "m", title: "t", description: "t",
    entry_frame_ids: ["a"],
    frames: [{ frame_id: "a", name: "plan", purpose: "t", kind: "agent",
      input_schema: {}, return_schema: {}, preconditions: [], postconditions: [],
      side_effect_class: "none", capability_refs: [],
      acceptance: { acceptance_atoms: ["a1"], verifier: null } }],
    edges: [], policies: {}, required_evidence: [],
    created_at: "t", created_by: { authority_kind: "operator", reference: "op" },
  });
  if (stored.json?.status !== "stored") return false;
  const jsonl = await call("GET", "/callgraphs/e2e-export/export?revision=1&format=jsonl");
  const todo = await call("GET", "/callgraphs/e2e-export/export?revision=1&format=todo.txt");
  return typeof jsonl.json?.body === "string" && jsonl.json?.body.includes("frame_id")
    && todo.json?.body?.includes("lossy:true");
});

check("callgraph item envelope (#289)", async () => {
  const r = await call("GET", "/callgraph-items/e2e-export/a?revision=1");
  const e = r.json?.envelope;
  return e?.identity?.canonical_ref === "e2e-export:1:a" && !!e?.content_digest;
});

check("remote workspace bindings (#89)", async () => {
  const r = await call("POST", "/remote-workspaces/bindings", {
    schema: "focusa.remote_workspace_binding.v1",
    binding_id: "e2e-binding",
    controller: { daemon_identity: "e2e", controller_origin: "e2e" },
    project: { project_id: "p1", repo_remote: "git@example.com:p.git" },
    transport: { kind: "ssh", host: "example.com", user: "u", port: 22,
      host_reference: null, verified_at: null, verification_evidence: [] },
    roots: { canonical_remote_root: "/srv/p", deploy_root: null, working_subpath: null, worktree_identity: null },
    session: { continuity_id: "cont-e2e", principal: "e2e" },
    state: { status: "pending", freshness: null, revocation: null },
  });
  const list = await call("GET", "/remote-workspaces/bindings");
  return r.json?.status === "created" || r.json?.status === "updated"
    || list.json?.bindings?.some((b) => b.binding_id === "e2e-binding");
});

check("closure validate gates on verdict (#276 settlement)", async () => {
  const r = await call("POST", "/work-items/closure/validate", {
    claim_id: "e2e-claim",
    acceptance_atoms: ["uncovered-atom"],
    evidence_refs: [], receipts: [],
  });
  return r.json?.validation_pass === false
    && r.json?.failure_class === "uncovered_acceptance_atoms";
});

check("silent sessions list (#195)", async () => {
  const r = await call("GET", "/silent-sessions");
  return r.status < 500 && (Array.isArray(r.json?.sessions) || r.json?.sessions === undefined);
});

check("silent completion sweep route (#311)", async () => {
  const r = await call("POST", "/silent-sessions/sweep-completions");
  return r.status < 500;
});

async function main() {
  const results = [];
  for (const c of checks) {
    let pass = false, note = "";
    try { pass = await c.fn(); } catch (error) { note = String(error); }
    results.push({ check: c.name, pass, note });
  }
  const failed = results.filter((r) => !r.pass);
  for (const r of results) console.log(`${r.pass ? "PASS" : "FAIL"}  ${r.check}${r.note ? ` — ${r.note}` : ""}`);
  console.log(`\n${results.length - failed.length}/${results.length} checks passed`);
  process.exit(failed.length ? 1 : 0);
}
main();
