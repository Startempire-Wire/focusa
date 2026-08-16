import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import test from "node:test";
import ts from "typescript";

const source = readFileSync(
  fileURLToPath(new URL("../src/scoped-surface-refresh.ts", import.meta.url)),
  "utf8"
);
const stateModule = `
const state = globalThis.__focusaScopedRefreshState;
export const currentProjectBindingDecision = () => state.binding;
export const getActiveWorkpointPacket = () => state.workpoint;
export const getContinuityId = () => state.continuityId;
export const getLastTrajectoryClarity = () => state.trajectory;
export const getSessionCwd = () => state.projectRoot;
export const getAttachmentRuntime = () => state.runtime;
export const normalizeProjectRoot = (value) => {
  const root = String(value || "").trim().replace(/\\/+$/, "");
  return root === "/" ? "/" : root;
};
`;
const stateUrl = `data:text/javascript;base64,${Buffer.from(stateModule).toString("base64")}`;
const compiled = ts.transpileModule(source.replace('from "./state.js"', `from "${stateUrl}"`), {
  compilerOptions: {
    module: ts.ModuleKind.ESNext,
    target: ts.ScriptTarget.ES2022,
  },
}).outputText;

globalThis.__focusaScopedRefreshState = {
  projectRoot: "/root/release-cycle",
  continuityId: "release-cycle-continuity",
  binding: { state: "RECOVERING" },
  trajectory: {
    canonical: true,
    degraded: false,
    project_root: "/root/release-cycle",
    trajectory_id: "trajectory-release-cycle",
    long_term_goal: "Ship release",
    mid_level_goal: "Close locked scope",
    short_term_goal: "Refresh surfaces",
  },
  workpoint: null,
  appended: [],
  runtime: {
    pi: {
      appendEntry: (customType, data) =>
        globalThis.__focusaScopedRefreshState.appended.push({ type: "custom", customType, data }),
    },
  },
};
const refresh = await import(`data:text/javascript;base64,${Buffer.from(compiled).toString("base64")}`);

test("mixed state keeps persisted trajectory visible while project recovers", () => {
  const snapshot = refresh.buildTruthfulScopedSurfaceSnapshot("/root", Date.parse("2026-07-31T00:01:00Z"));
  assert.equal(snapshot.selected_scope, "/root/release-cycle");
  assert.equal(snapshot.startup_cwd, "/root");
  assert.equal(snapshot.project, "recovering");
  assert.equal(snapshot.trajectory, "persisted");
  assert.equal(snapshot.workpoint, "absent");
  assert.equal(snapshot.bead, "absent");
  assert.equal(snapshot.proof, "missing");
  assert.equal(snapshot.proof_count, 0);
});

test("bound selected project outranks broad startup cwd", () => {
  const state = globalThis.__focusaScopedRefreshState;
  const previous = {
    projectRoot: state.projectRoot,
    binding: state.binding,
    trajectory: state.trajectory,
  };
  try {
    state.projectRoot = "/root";
    state.binding = { state: "BOUND", selected_project_root: "/home/wirebot/focusa" };
    state.trajectory = {
      canonical: true,
      degraded: false,
      project_root: "/home/wirebot/focusa",
      trajectory_id: "trajectory-focusa-mvp",
      long_term_goal: "Complete full Focusa MVP",
      mid_level_goal: "Restore locked-release baseline",
      short_term_goal: "Repair Pi surfaces",
    };
    const snapshot = refresh.buildTruthfulScopedSurfaceSnapshot("/root");
    assert.equal(refresh.currentScopedProjectRoot(), "/home/wirebot/focusa");
    assert.equal(snapshot.selected_scope, "/home/wirebot/focusa");
    assert.equal(snapshot.startup_cwd, "/root");
    assert.equal(snapshot.project, "bound");
    assert.equal(snapshot.trajectory, "persisted");
  } finally {
    Object.assign(state, previous);
  }
});

test("one exact-scope receipt refreshes subscribers and exposes freshness", async () => {
  let observed = 0;
  const unsubscribe = refresh.subscribeScopedStateChanges((receipt) => {
    if (refresh.scopedReceiptMatchesCurrentScope(receipt)) observed += 1;
  });
  const receipt = refresh.publishScopedStateChange({
    source: "tool",
    mutation_kind: "/trajectory/define-goal",
    project_root: "/root/release-cycle",
    continuity_id: "release-cycle-continuity",
    status: "accepted",
    evidence_revision: "revision-1",
    effective_at: "2026-07-31T00:00:30Z",
  });
  await new Promise((resolve) => queueMicrotask(resolve));
  unsubscribe();
  assert.ok(receipt);
  assert.equal(observed, 1);
  const snapshot = refresh.buildTruthfulScopedSurfaceSnapshot("/root", Date.parse("2026-07-31T00:01:00Z"));
  assert.equal(snapshot.last_refresh_status, "accepted");
  assert.equal(snapshot.stale_age_ms, 30_000);
  assert.equal(globalThis.__focusaScopedRefreshState.appended.at(-1).data.receipt_id, receipt.receipt_id);
});

test("durable scoped receipts rehydrate after session reload", () => {
  const accepted = refresh.rehydrateScopedStateChanges(globalThis.__focusaScopedRefreshState.appended);
  assert.ok(accepted >= 1);
  assert.equal(refresh.latestScopedStateChange()?.schema, "focusa.scoped_state_change_receipt.v1");
});

test("foreign receipts never refresh the current project", () => {
  const foreign = refresh.publishScopedStateChange({
    source: "sse",
    mutation_kind: "trajectory_updated",
    project_root: "/other/project",
    continuity_id: "other-continuity",
    status: "observed",
    effective_at: "2026-07-31T00:00:45Z",
  });
  assert.ok(foreign);
  assert.equal(refresh.scopedReceiptMatchesCurrentScope(foreign), false);
});

test("blocked Workpoint and proof zero remain non-success states", () => {
  globalThis.__focusaScopedRefreshState.binding = { state: "BOUND" };
  globalThis.__focusaScopedRefreshState.workpoint = {
    workpoint_id: "workpoint-1",
    work_item_id: "focusa-vbcqu.4.3",
    status: "blocked",
    blockers: ["waiting for proof"],
    verification_records: [],
  };
  const snapshot = refresh.buildTruthfulScopedSurfaceSnapshot("/root");
  assert.equal(snapshot.project, "bound");
  assert.equal(snapshot.workpoint, "blocked");
  assert.equal(snapshot.bead, "present");
  assert.equal(snapshot.proof, "missing");
  assert.equal(snapshot.proof_count, 0);
});

// bg completion notification loop: the envelope the daemon broadcasts must
// carry a bounded output_tail so the front terminal can render the job
// result (no log-file hop for the agent).
export function bgCompletionEnvelopeContract(envelope) {
  const required = [
    "schema",
    "event_type",
    "job_id",
    "name",
    "command",
    "cwd",
    "status",
    "log_path",
    "started_at",
    "completed_at",
    "output_tail",
  ];
  const missing = required.filter((key) => !(key in envelope));
  if (missing.length) {
    throw new Error(`bg completion envelope missing: ${missing.join(", ")}`);
  }
  if (envelope.event_type !== "background_job_completion") {
    throw new Error("wrong event_type");
  }
  const tail = String(envelope.output_tail);
  if (tail.length > 4096) {
    throw new Error(`output_tail exceeds bound: ${tail.length}`);
  }
  return true;
}

const sample = {
  schema: "focusa.stream_event.v1",
  event_type: "background_job_completion",
  job_id: "j1",
  name: "probe",
  command: "true",
  cwd: ".",
  status: "completed",
  exit_code: 0,
  log_path: "/tmp/probe.log",
  started_at: "t0",
  completed_at: "t1",
  output_tail: "done",
};
if (!bgCompletionEnvelopeContract(sample)) process.exit(1);
console.log("bg-completion-envelope-contract: ok");
