#!/usr/bin/env node
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import process from "node:process";

const root = process.cwd();
const readJson = (path) => JSON.parse(readFileSync(`${root}/${path}`, "utf8"));
const readJsonl = (path) =>
  readFileSync(`${root}/${path}`, "utf8")
    .split("\n")
    .filter(Boolean)
    .map((line) => JSON.parse(line));
const stable = (value) => {
  if (Array.isArray(value)) return value.map(stable);
  if (value && typeof value === "object") {
    return Object.fromEntries(Object.keys(value).sort().map((key) => [key, stable(value[key])]));
  }
  return value;
};
const digest = (value) =>
  `sha256:${createHash("sha256").update(JSON.stringify(stable(value))).digest("hex")}`;
const failures = [];
const fail = (message) => failures.push(message);
const sameSet = (label, expected, actual) => {
  const left = new Set(expected);
  const right = new Set(actual);
  const missing = [...left].filter((value) => !right.has(value));
  const extra = [...right].filter((value) => !left.has(value));
  if (missing.length || extra.length) fail(`${label}: missing=${missing.join(",")} extra=${extra.join(",")}`);
};

const manifest = readJson("release-proof/audit/next-locked-release-scope.json");
const decomposition = readJson("release-proof/audit/next-locked-release-decomposition.json");
const definition = readJson("release-proof/audit/next-locked-release-workset-definition.json");
const binding = readJson("release-proof/audit/next-locked-release-workset-provider-binding.json");
const completion = readJson("release-proof/audit/next-locked-release-workset-completion-contract.json");
const proof = readJson("release-proof/audit/next-locked-release-execution-proof.json");
const members = readJsonl("release-proof/audit/next-locked-release-workset-members.jsonl");
const edges = readJsonl("release-proof/audit/next-locked-release-workset-edges.jsonl");
const events = readJsonl("release-proof/audit/next-locked-release-workset-events.jsonl");
const issueRows = readJsonl(".beads/issues.jsonl");
const issues = new Map(issueRows.map((issue) => [issue.id, issue]));

// Derive the sealed member set from every decomposition authority plus every
// existing descendant of the root epic. Root-descendant inclusion repairs the
// former 238-member undercount without admitting unrelated work.
const expected = new Set([decomposition.acceptance_gate, decomposition.decomposition_epic]);
for (const entry of decomposition.original_locked_issues) {
  expected.add(entry.locked_issue_id);
  for (const ref of entry.decomposition_leaf_refs ?? []) expected.add(ref);
}
for (const entry of decomposition.operator_authorized_post_lock_additions ?? []) expected.add(entry.issue_id);
for (const spec of Object.values(decomposition.specs)) {
  expected.add(spec.parent);
  for (const ref of spec.leaf_refs ?? []) expected.add(ref);
  for (const row of spec.requirement_mappings ?? []) expected.add(row.bead_ref);
  for (const row of spec.section_mappings ?? []) expected.add(row.bead_ref);
}
for (const ref of manifest.decomposition_tasks) expected.add(ref);
const children = new Map([...issues.keys()].map((id) => [id, []]));
for (const issue of issues.values()) {
  for (const dependency of issue.dependencies ?? []) {
    if (dependency.type === "parent-child" && children.has(dependency.depends_on_id)) {
      children.get(dependency.depends_on_id).push(issue.id);
    }
  }
}
const stack = ["focusa-vbcqu"];
const descendants = new Set();
while (stack.length) {
  const parent = stack.pop();
  for (const child of children.get(parent) ?? []) {
    if (!descendants.has(child)) {
      descendants.add(child);
      stack.push(child);
    }
  }
}
for (const descendant of descendants) expected.add(descendant);

if (manifest.lock_revision !== 6 || manifest.scope_state !== "locked") fail("scope manifest is not sealed at final revision 6");
if (manifest.current_explicit_issue_count !== 43) fail("explicit issue count is not 43");
if (manifest.current_locked_bead_member_count !== 252) fail("locked member count is not 252");
if (expected.size !== 252) fail(`derived member count is ${expected.size}, expected 252`);
if (manifest.membership_reconciliation?.scope_expansion !== false) fail("membership repair is not declared non-expanding");
if (manifest.membership_reconciliation?.restored_descendant_count !== 13) fail("13 omitted descendants were not reconciled");
if (manifest.scope_additions_closed !== true || manifest.scope_addition_policy !== "closed_no_further_admissions") {
  fail("final scope additions are not durably closed");
}
if (manifest.final_scope_addition_id !== "focusa-o4gkd") fail("final scope addition mismatch");
if (manifest.final_scope_admission?.further_additions_allowed !== false) fail("further scope additions remain allowed");
if (manifest.execution_lock?.status !== "sealed") fail("execution lock is not sealed");
if (manifest.execution_lock?.workset_id !== definition.workset_id) fail("manifest/workset id mismatch");
if (manifest.execution_lock?.member_count !== 252) fail("manifest execution member count is not 252");
if (manifest.execution_lock?.phase0_sequence?.join(",") !== "focusa-627th.4.3,focusa-o4gkd") {
  fail("phase-0 final bug sequence mismatch");
}
if (manifest.execution_lock?.first_touch_issue_id !== "focusa-627th.4.3") fail("first-touch issue mismatch");

if (definition.schema_version !== "focusa.workset.v1") fail("workset schema mismatch");
if (definition.workset_id !== "workset:focusa-next-locked-release:r6") fail("workset id mismatch");
if (definition.revision !== 6 || definition.admission_state !== "sealed") fail("workset is not sealed at final revision 6");
if (definition.cardinality_mode !== "fixed" || definition.membership_policy !== "exclusive") fail("workset is not fixed/exclusive");
if (definition.scope?.project_root !== "/home/wirebot/focusa") fail("workset project root mismatch");
if (definition.scope?.continuity_id !== "focusa-v0.9.135-locked-14") fail("workset continuity mismatch");
if (binding.binding_id !== "provider:bd:focusa-next-locked-release:r6") fail("provider binding id mismatch");
if (binding.provider !== "bd" || binding.query_semantics !== "explicit_ids" || binding.freshness !== "current") {
  fail("provider binding is not a current explicit-id Beads binding");
}
if (binding.query?.member_count !== 252) fail("provider binding member count mismatch");

const memberIds = members.map((member) => member.member_id);
if (memberIds.length !== new Set(memberIds).size) fail("duplicate workset member ids");
sameSet("sealed workset membership", expected, memberIds);
for (const id of expected) if (!issues.has(id)) fail(`provider missing locked member: ${id}`);
for (const member of members) {
  if (!member.mandatory) fail(`non-mandatory locked member: ${member.member_id}`);
  const expectedBindingRef =
    member.member_id === "focusa-o4gkd"
      ? "provider:bd:focusa-next-locked-release:r6"
      : "provider:bd:focusa-next-locked-release:r5";
  if (member.provider_binding_ref !== expectedBindingRef) fail(`provider binding mismatch: ${member.member_id}`);
  if (!/^execution-phase:[0-8]$/.test(member.task_plan_ref ?? "")) fail(`invalid execution phase: ${member.member_id}`);
  const providerStatus = issues.get(member.member_id)?.status;
  if (member.current_status_projection !== providerStatus) {
    fail(`current provider status projection mismatch: ${member.member_id}`);
  }
  const expectedDisposition = providerStatus === "closed" ? "completed" : "pending";
  if (member.disposition !== expectedDisposition) fail(`admission disposition mismatch: ${member.member_id}`);
}
if (memberIds.includes("spec:150") || memberIds.some((id) => id.includes("spec150"))) {
  fail("Spec 150 was admitted without operator scope authorization");
}
if (definition.membership_digest !== digest(members)) fail("membership digest mismatch");
if (definition.graph_digest !== digest(edges)) fail("graph digest mismatch");
if (binding.query?.member_ids_digest !== digest([...expected].sort())) fail("provider member-id digest mismatch");

const edgeIds = edges.map((edge) => edge.edge_id);
if (edgeIds.length !== new Set(edgeIds).size) fail("duplicate execution edge ids");
const adjacency = new Map(memberIds.map((id) => [id, []]));
const incoming = new Map(memberIds.map((id) => [id, []]));
for (const edge of edges) {
  if (!adjacency.has(edge.from_member_ref) || !adjacency.has(edge.to_member_ref)) {
    fail(`edge references non-member: ${edge.edge_id}`);
    continue;
  }
  adjacency.get(edge.from_member_ref).push(edge.to_member_ref);
  incoming.get(edge.to_member_ref).push(edge.from_member_ref);
}

// Provider blockers that cross the sealed boundary must already be closed.
const unresolvedExternal = [];
for (const member of members) {
  for (const dependency of issues.get(member.member_id)?.dependencies ?? []) {
    if (dependency.type !== "blocks" || expected.has(dependency.depends_on_id)) continue;
    const status = issues.get(dependency.depends_on_id)?.status ?? "missing";
    if (status !== "closed") unresolvedExternal.push(`${dependency.depends_on_id}->${member.member_id}:${status}`);
  }
}
if (unresolvedExternal.length) fail(`unresolved external blockers: ${unresolvedExternal.join(",")}`);

// DAG proof.
const indegree = new Map(memberIds.map((id) => [id, incoming.get(id).length]));
const queue = memberIds.filter((id) => indegree.get(id) === 0).sort();
const visited = [];
while (queue.length) {
  const id = queue.shift();
  visited.push(id);
  for (const target of adjacency.get(id)) {
    indegree.set(target, indegree.get(target) - 1);
    if (indegree.get(target) === 0) {
      queue.push(target);
      queue.sort();
    }
  }
}
if (visited.length !== memberIds.length) fail(`dependency cycle detected: visited ${visited.length}/${memberIds.length}`);

const reachesTerminal = (start) => {
  const seen = new Set();
  const pending = [start];
  while (pending.length) {
    const id = pending.pop();
    if (id === "focusa-vbcqu") return true;
    if (seen.has(id)) continue;
    seen.add(id);
    pending.push(...adjacency.get(id));
  }
  return false;
};
const uncovered = memberIds.filter((id) => !reachesTerminal(id));
if (uncovered.length) fail(`members without terminal coverage: ${uncovered.join(",")}`);

const providerCompleted = (id) => issues.get(id)?.status === "closed";
const ready = memberIds
  .filter((id) => !providerCompleted(id) && incoming.get(id).every(providerCompleted))
  .sort();
sameSet("unique executable frontier", ["focusa-o4gkd"], ready);
const activeContainers = new Set(["focusa-vbcqu", "focusa-vbcqu.9"]);
const outOfOrderActive = memberIds.filter(
  (id) => issues.get(id)?.status === "in_progress" && !ready.includes(id) && !activeContainers.has(id),
);
if (outOfOrderActive.length) fail(`out-of-order in-progress members: ${outOfOrderActive.join(",")}`);

const phaseCounts = {};
const phaseByMember = new Map();
for (const member of members) {
  const phase = Number(member.task_plan_ref.split(":").at(-1));
  phaseByMember.set(member.member_id, phase);
  phaseCounts[phase] = (phaseCounts[phase] ?? 0) + 1;
}
const expectedGates = [
  "focusa-o4gkd",
  "focusa-vbcqu.2",
  "focusa-vbcqu.3",
  "focusa-vbcqu.4",
  "focusa-vbcqu.5",
  "focusa-vbcqu.6",
  "focusa-vbcqu.7",
  "focusa-vbcqu.8",
  "focusa-vbcqu",
];
sameSet("phase gates", expectedGates, manifest.execution_lock?.phase_gates ?? []);
const edgeKey = (from, to, type) => `${from}|${to}|${type}`;
const edgeKeys = new Set(edges.map((edge) => edgeKey(edge.from_member_ref, edge.to_member_ref, edge.edge_type)));
for (const edge of edges) {
  if (phaseByMember.get(edge.from_member_ref) > phaseByMember.get(edge.to_member_ref)) {
    fail(`backward phase edge: ${edge.edge_id}`);
  }
}
for (const member of members) {
  const phase = phaseByMember.get(member.member_id);
  const gate = expectedGates[phase];
  if (phase === 0 && member.member_id !== gate) {
    if (member.member_id === "focusa-627th.4.3") {
      if (!edgeKeys.has(edgeKey(member.member_id, gate, "blocks"))) {
        fail("first-touch #14 does not gate final workflow-staleness bug #111");
      }
    } else if (!edgeKeys.has(edgeKey(member.member_id, "focusa-627th.4.3", "release_requires"))) {
      fail(`phase 0 member does not close through first touch: ${member.member_id}`);
    }
  } else if (phase > 0 && phase < 8) {
    const previousGate = expectedGates[phase - 1];
    if (member.member_id === gate) {
      if (!edgeKeys.has(edgeKey(previousGate, gate, "blocks"))) fail(`phase gate lacks prior gate: ${gate}`);
    } else {
      if (!edgeKeys.has(edgeKey(previousGate, member.member_id, "blocks"))) {
        fail(`phase member bypasses prior gate: ${member.member_id}`);
      }
      if (!edgeKeys.has(edgeKey(member.member_id, gate, "release_requires"))) {
        fail(`phase member does not close through gate: ${member.member_id}`);
      }
    }
  }
}
for (const member of members) {
  for (const dependency of issues.get(member.member_id)?.dependencies ?? []) {
    if (dependency.type !== "blocks" || !expected.has(dependency.depends_on_id)) continue;
    if (!edgeKeys.has(edgeKey(dependency.depends_on_id, member.member_id, "blocks"))) {
      fail(`provider blocker missing from Workset: ${dependency.depends_on_id}->${member.member_id}`);
    }
  }
}
if (!edgeKeys.has(edgeKey("focusa-627th.4.3", "focusa-o4gkd", "blocks"))) {
  fail("GitHub #14 does not gate final workflow-staleness bug #111");
}
for (const member of members.filter((item) => item.task_plan_ref === "execution-phase:1")) {
  if (!edgeKeys.has(edgeKey("focusa-o4gkd", member.member_id, "blocks"))) {
    fail(`final workflow-staleness bug does not gate phase-1 member: ${member.member_id}`);
  }
}
for (const [from, to, type] of [
  ["focusa-vbcqu.8", "focusa-vbcqu.9.7", "blocks"],
  ["focusa-vbcqu.9.7", "focusa-vbcqu.9", "release_requires"],
  ["focusa-vbcqu.9", "focusa-vbcqu", "release_requires"],
]) {
  if (!edgeKeys.has(edgeKey(from, to, type))) fail(`terminal chain edge missing: ${from}->${to}`);
}
if (JSON.stringify(phaseCounts) !== JSON.stringify(proof.phase_counts)) fail("phase count proof mismatch");
if (proof.scope_member_count !== 252 || proof.execution_edge_count !== edges.length) fail("execution proof count mismatch");
if (proof.dependency_cycles !== 0 || proof.terminal_coverage_count !== 252) fail("execution proof graph result mismatch");
if (
  proof.scope_expansion !== true ||
  proof.authorized_scope_expansion_count !== 1 ||
  proof.final_scope_addition !== "focusa-o4gkd" ||
  proof.further_scope_additions_allowed !== false
) {
  fail("final authorized scope-addition proof mismatch");
}
sameSet("proof frontier", ready, proof.unique_ready_frontier ?? []);
if (proof.membership_digest !== definition.membership_digest || proof.graph_digest !== definition.graph_digest) {
  fail("execution proof digest mismatch");
}

if (
  events.length < 2 ||
  events[0]?.event_type !== "workset.sealed" ||
  events[1]?.event_type !== "workset.sealed"
) {
  fail("revision-5/revision-6 Workset seal event chain is incomplete");
}
for (const [index, event] of events.entries()) {
  const { event_hash: eventHash, ...eventWithoutHash } = event;
  if (eventHash !== digest(eventWithoutHash)) fail(`workset event hash mismatch: ${event.event_id}`);
  if (index > 0 && event.previous_event_hash !== events[index - 1]?.event_hash) {
    fail(`workset event chain break: ${event.event_id}`);
  }
}
if (
  events[1]?.workset_revision !== 6 ||
  events[1]?.workset_id !== "workset:focusa-next-locked-release:r6" ||
  events[1]?.previous_event_hash !== events[0]?.event_hash
) {
  fail("revision-6 seal event does not append to revision 5");
}
const github14Completion = events.find(
  (event) =>
    event.event_type === "workset.member_projection_changed" &&
    event.payload?.member_ref === "focusa-627th.4.3"
);
if (
  github14Completion?.payload?.disposition !== "completed" ||
  github14Completion?.payload?.freshness !== "current"
) {
  fail("GitHub #14 Workset completion projection is missing");
}
if (
  completion.require_sealed_revision !== true ||
  completion.require_all_mandatory_dispositions !== true ||
  completion.require_zero_dependency_cycles !== true ||
  completion.require_zero_unresolved_blockers !== true ||
  completion.require_zero_unknown_impacts !== true
) {
  fail("completion contract is weaker than Spec 149 closure law");
}

if (failures.length) {
  console.error(JSON.stringify({ schema: "focusa.locked_release_execution_audit.v1", status: "failed", failures }, null, 2));
  process.exit(1);
}
console.log(
  JSON.stringify(
    {
      schema: "focusa.locked_release_execution_audit.v1",
      status: "verified",
      lock_revision: 6,
      explicit_issues: 43,
      sealed_members: members.length,
      execution_edges: edges.length,
      phases: Object.fromEntries(Object.entries(phaseCounts).sort(([a], [b]) => Number(a) - Number(b))),
      dependency_cycles: 0,
      unresolved_external_blockers: 0,
      terminal_coverage: `${members.length}/${members.length}`,
      unique_ready_frontier: ready,
      out_of_order_in_progress: 0,
      scope_expansion: true,
      final_scope_addition: "focusa-o4gkd",
      further_scope_additions_allowed: false,
    },
    null,
    2,
  ),
);
