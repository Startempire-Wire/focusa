import {
  compareVectorClock,
  isScopeRef,
  isWorkstreamKey,
  reconcileScopedRecord,
  renderScopedResultHuman,
  sameWorkstream,
  type ScopeRef,
  type ScopedCrdtRecord,
  type ScopedResultEnvelope,
  type WorkstreamKey,
} from "../apps/pi-extension/src/scoped-state.ts";

function assert(condition: any, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

const root = (name: string): ScopeRef => ({
  scope_kind: "project",
  scope_id: `project:${name}`,
  root_path: `/workspace/${name}`,
  canonical_name: name,
  fingerprint: `sha256:${name}`,
});
const workstream = (name: string, continuity = "cont"): WorkstreamKey => ({
  root_scope: root(name),
  continuity_id: continuity,
});

assert(isScopeRef(root("a")), "valid ScopeRef rejected");
assert(!isScopeRef({ ...root("a"), fingerprint: "" }), "empty fingerprint accepted");
assert(isWorkstreamKey(workstream("a")), "valid WorkstreamKey rejected");
assert(!isWorkstreamKey({ root_scope: root("a"), continuity_id: "" }), "empty continuity accepted");
assert(!sameWorkstream(workstream("a"), workstream("b")), "continuity alone established authority");
assert(compareVectorClock({ a: 1 }, { a: 2 }) === -1, "causal order mismatch");
assert(compareVectorClock({ a: 2 }, { b: 2 }) === null, "concurrent clocks not detected");

const record = (
  project: string,
  actor: string,
  value: string,
  clock: Record<string, number>,
  updated: string
): ScopedCrdtRecord<string> => ({
  schema: "focusa.scoped_state.v1",
  scope: workstream(project),
  record_id: "record-1",
  actor_id: actor,
  vector_clock: clock,
  lamport_ts: 1,
  updated_at: updated,
  tombstone: false,
  value,
});
const left = record("a", "actor-a", "left", { "actor-a": 1 }, "2026-07-13T00:00:00Z");
const right = record("a", "actor-b", "right", { "actor-b": 1 }, "2026-07-13T00:00:01Z");
const lr = reconcileScopedRecord(left, right);
const rl = reconcileScopedRecord(right, left);
assert(JSON.stringify(lr) === JSON.stringify(rl), "concurrent merge is not commutative");
assert(
  JSON.stringify(reconcileScopedRecord(lr, lr)) === JSON.stringify(lr),
  "CRDT merge is not idempotent"
);
let blocked = false;
try {
  reconcileScopedRecord(left, record("b", "actor-b", "bad", { "actor-b": 1 }, right.updated_at));
} catch (error) {
  blocked = String(error).includes("scope_mismatch");
}
assert(blocked, "cross-project CRDT merge was not blocked");

const envelope: ScopedResultEnvelope<{ record_id: string }> = {
  schema: "focusa.scoped_result.v1",
  scope: workstream("a"),
  authority: { status: "canonical", why: "verified typed root" },
  human: {
    status: "completed",
    summary: "Scoped record reconciled",
    next_action: "Continue",
    why: "Vector clocks converged",
    evidence_refs: ["test:spec104"],
    warnings: [],
  },
  human_readable:
    "completed: Scoped record reconciled. Scope: a · cont. Authority: canonical. Next: Continue. Why: Vector clocks converged",
  data: { record_id: "record-1" },
};
const human = renderScopedResultHuman(envelope);
for (const label of [
  "Human readable:",
  "Status:",
  "Summary:",
  "Scope:",
  "Authority:",
  "Next action:",
  "Why:",
  "Evidence:",
]) assert(human.includes(label), `human output missing ${label}`);
assert(envelope.data.record_id === "record-1", "machine data was lost");

console.log("PASS: Spec104 TS scoped state, CRDT, and dual-output contract");
