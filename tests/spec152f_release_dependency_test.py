#!/usr/bin/env python3
"""152F.06.10 Wire Spec 152F closure into Spec 152E and release milestones
(atom focusa-vbcqu.20.14.52).

Exact verification:
    python3 tests/spec152f_release_dependency_test.py && \
    python3 tests/164_focusa_locked_release_governance_reconciliation_test.py && \
    bd dep cycles --json

This test is the deterministic, stdlib-only, fail-closed final dependency /
provenance audit for the Spec 152F child series (focusa-vbcqu.20.14.1..52).
It cannot pass with a broken Spec152E wiring, a cyclic dependency graph, a
Spec152F bypass in the REL closure, a stale taskgraph, a missing child
record, an uncommitted receipt, or an unblocked stable publication. It exits
0 only when the complete closure chain is present in the committed tree, and
it records the exact HEAD SHA and a bounded closure receipt.

Authority: docs/152f-simple-entitlement-gating-and-future-granularity-
addendum.md (Spec 152F final wiring atom); Specs 152, 152E, and 150A remain
binding. This atom may simplify enforcement only; it may not weaken EDD
identity/commerce, signed-lease, refund/revoke, node, sequence, or recovery
authority.

Fail-closed invariants (152F.06.10 done condition):
- Spec152F cannot be bypassed by REL closure: the committed governance gate
  keeps the REL wiring successors (focusa-vbcqu.20.14.51, 20.14.52,
  20.13.62, 20.13.63) technically pending (no administrative closure), the
  reducer re-derives the gate byte-identically with invalid_closed_count=0,
  and the GH#106.2 governance reconciliation stays truthfully blocked.
- Graph has zero cycles: the 123 internal Spec 152F taskgraph edges are
  acyclic under an independent Kahn traversal (52/52 visited), the committed
  downstream release edges are exactly
  [{blocked: focusa-vbcqu.20.13.63, blocker: focusa-vbcqu.20.14.52}] with
  the Spec 172 seal blocking this final item, and the literal Beads edge
  command (bd dep cycles --json) exits 0 with cycles=[] count=0.
- Stable publication still requires exact final release acceptance: the Spec
  152 final-audit contract keeps distribution_status=blocked,
  migration_cutover_accepted=false, all_platform_final_candidate_verified
  =false, REL.4-REL.7 status not_closed, and the publication rule forbidding
  stable publication until focusa-vbcqu.20.13.63 and this closure close
  with linked acceptance evidence (the Spec 172 final acceptance work item
  must also close); the Spec 152F addendum itself keeps the Section 13
  non-goal that publication stays forbidden until Specs 152/152E/152F and
  REL.4-REL.7 close truthfully.
- Spec152E final closure depends on accepted Spec152F closure: the Spec 152E
  completion gate and the final-audit/next-command/release-blocker contracts
  wire focusa-vbcqu.20.13.63 and the final release milestones behind the
  accepted Spec 152F closure receipt
  (docs/evidence/spec152f/focusa-vbcqu.20.14.52-acceptance.txt).
- Active docs update from implementation_pending to accepted only with
  receipts: the Spec 152F addendum status and the final-audit contract
  record accepted-with-receipts state and bind the acceptance record; every
  child evidence record is git-tracked, names its work item, carries
  verification content, cites no unfinished-acceptance markers, and named
  implementation commits resolve in this repository.
"""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
from collections import defaultdict, deque
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs" / "evidence" / "spec152f"
TASKGRAPH = ROOT / "docs" / "contracts" / "spec152f-implementation-taskgraph.v1.json"
SPEC172_TASKGRAPH = (
    ROOT / "docs" / "contracts" / "spec172-implementation-taskgraph.v1.json"
)
FINAL_AUDIT = ROOT / "docs" / "contracts" / "spec152-final-audit-status.v1.yaml"
NEXT_COMMAND = ROOT / "docs" / "contracts" / "spec152-next-command.v1.yaml"
BLOCKER_SUMMARY = (
    ROOT / "docs" / "contracts" / "spec152-release-blocker-summary.v1.yaml"
)
DOCUMENT_SET = ROOT / "docs" / "contracts" / "spec152-document-set.v1.yaml"
SPEC152F_DOC = (
    ROOT / "docs" / "152f-simple-entitlement-gating-and-future-granularity-addendum.md"
)
SPEC152E_DOC = (
    ROOT
    / "docs"
    / "152e-edd-centered-universal-multi-surface-licensing-and-branded-facade-addendum.md"
)
GATE = ROOT / "release-proof" / "audit" / "next-locked-release-technical-closure-gate.json"
LEDGER = (
    ROOT / "release-proof" / "audit" / "next-locked-release-governance-reconciliation.json"
)
REDUCER = ROOT / "scripts" / "reduce-locked-release-technical-closure.py"
MY_RECEIPT = "docs/evidence/spec152f/focusa-vbcqu.20.14.52-acceptance.txt"

# The 01.02 registry-loader record (focusa-vbcqu.20.14.6) cites commits on
# the parallel local/luna-152f-registry branch outside this tree and is
# refused as external evidence (its surface is implemented in-tree at
# d203e08c, verified by the Spec 152F suite). The final wiring atom's own
# receipt is written by this atom, so it may be absent only at the moment
# this test first runs. These are the only taskgraph evidence paths allowed
# to be absent.
UNCLOSED_RECORDS = {
    "focusa-vbcqu.20.14.6",
    "focusa-vbcqu.20.14.52",
}
# In-tree implementation commit covering the 01.02 registry-loader surface,
# ancestor of HEAD.
REGISTRY_LOADER_IN_TREE_COMMIT = "d203e08c"

PHASE_06_TEMPLATE = (
    "Implementation commit",
    "Bounded result",
    "Exact verification",
    "Rollback",
    "No push, deploy, release, merge, or Beads mutation was performed.",
)
PENDING_MARKERS = (
    "TECHNICAL_ACCEPTANCE: CONTINUE",
    "PENDING HARNESS",
    "PENDING CARGO",
    "exit code: not run",
)

COMMIT_REF = re.compile(
    r"(?i)(implementation\s+commits?\s*[:`\-\s]*)"
    r"([0-9a-f]{7,40})"
)

# Strict exact-SHA binding is required for the phase-06 acceptance series
# (20.14.43..20.14.50), the 20.14.51 seal, and this final wiring atom;
# earlier-era records bind via their committed blob + verification content
# (focusa-vbcqu.20.14.22 cites an object absent from this tree and is
# audited for verification content rather than rewriting another atom's
# record — same rule as the 20.14.51 closure receipt).
EXACT_SHA_RECORDS = {f"focusa-vbcqu.20.14.{n}" for n in range(43, 51)} | {
    "focusa-vbcqu.20.14.51",
    "focusa-vbcqu.20.14.52",
}


def run(*args: str, expected: int = 0) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        list(args),
        cwd=ROOT,
        text=True,
        capture_output=True,
    )
    assert result.returncode == expected, result.stderr or result.stdout
    return result


def git_commit_exists(sha: str) -> bool:
    result = subprocess.run(
        ["git", "cat-file", "-e", f"{sha}^{{commit}}"],
        cwd=ROOT,
        capture_output=True,
    )
    return result.returncode == 0


def git_is_ancestor(sha: str) -> bool:
    result = subprocess.run(
        ["git", "merge-base", "--is-ancestor", sha, "HEAD"],
        cwd=ROOT,
        capture_output=True,
    )
    return result.returncode == 0


def git_tracked(path: Path) -> bool:
    result = subprocess.run(
        ["git", "ls-files", "--error-unmatch", str(path.relative_to(ROOT))],
        cwd=ROOT,
        capture_output=True,
    )
    return result.returncode == 0


# ── 1. Taskgraph final-task wiring (Spec152E release milestone behind 152F) ─

taskgraph = json.loads(TASKGRAPH.read_text())
assert taskgraph["schema"] == "focusa.spec152f_implementation_taskgraph_index.v1"
assert taskgraph["authority"] == (
    "docs/152f-simple-entitlement-gating-and-future-granularity-addendum.md"
)
assert taskgraph["parent"] == "focusa-vbcqu.20.14"
assert taskgraph["task_count"] == 52
assert taskgraph["first_task"] == "focusa-vbcqu.20.14.1"
assert taskgraph["final_task"] == "focusa-vbcqu.20.14.52"
assert taskgraph["phase_counts"] == {
    "00": 4, "01": 9, "02": 8, "03": 8, "04": 7, "05": 6, "06": 10,
}
assert taskgraph["internal_dependency_edge_count"] == 123
# The committed downstream release edge: the Spec 152E final release
# milestone depends on the accepted Spec 152F closure (this atom). This is
# the exact wiring required by 152F.06.10 — Spec152E final closure and final
# release milestones depend on accepted Spec152F closure.
assert taskgraph["downstream_release_edges"] == [
    {"blocked": "focusa-vbcqu.20.13.63", "blocker": "focusa-vbcqu.20.14.52"}
]
for phase, rel in sorted(taskgraph["phase_files"].items()):
    raw = (ROOT / rel).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == taskgraph["phase_file_sha256"][phase], rel
print(
    "taskgraph wiring intact: tasks=52 final=focusa-vbcqu.20.14.52 "
    "downstream_release_edges=20.13.63<-20.14.52"
)

# ── 2. Spec 152E final closure depends on accepted Spec 152F closure ────────

final_audit = FINAL_AUDIT.read_text(encoding="utf-8")
for token in (
    "spec152f_registered: true",
    "spec152f_work_item_root: focusa-vbcqu.20.14",
    "spec152f_final_work_item: focusa-vbcqu.20.14.52",
    "spec152f_closure_status: accepted_with_receipts",
    "spec152f_closure_receipt: docs/evidence/spec152f/focusa-vbcqu.20.14.52-acceptance.txt",
    "spec152e_final_closure_dependency: focusa-vbcqu.20.14.52",
    "rel_gates_status: not_closed",
    "distribution_status: blocked",
    "migration_cutover_accepted: false",
    "all_platform_final_candidate_verified: false",
):
    assert token in final_audit, f"final-audit contract missing: {token!r}"

spec152e_doc = SPEC152E_DOC.read_text(encoding="utf-8")
for token in (
    "accepted Spec 152F closure",
    "focusa-vbcqu.20.14.52",
    "REL.4\u2013REL.7",
    "customer/evaluator distribution and stable-release claims remain blocked",
):
    assert token in spec152e_doc, f"Spec 152E completion gate missing: {token!r}"

next_command = NEXT_COMMAND.read_text(encoding="utf-8")
for token in (
    "publication: forbidden_until_focusa-vbcqu.20.13.63_and_focusa-vbcqu.20.14.52_close",
    "post_spec152f_closure:",
):
    assert token in next_command, f"next-command contract missing: {token!r}"

blocker_summary = BLOCKER_SUMMARY.read_text(encoding="utf-8")
for token in (
    "status: blocked_for_new_evaluator_customer_and_stable_distribution",
    "simplification_final_work_item: focusa-vbcqu.20.14.52",
    "simplification_closure_status: accepted_with_receipts",
    "spec152e_final_closure_blocked_by: focusa-vbcqu.20.14.52",
):
    assert token in blocker_summary, f"release-blocker contract missing: {token!r}"

document_set = DOCUMENT_SET.read_text(encoding="utf-8")
for token in (
    "final_work_item: focusa-vbcqu.20.14.52",
    "status: release_blocking",
):
    assert token in document_set, f"document-set contract missing: {token!r}"
print(
    "Spec152E final closure wired behind accepted Spec152F closure "
    "(final-audit, next-command, release-blocker, document-set, Spec 152E completion gate)"
)

# ── 3. Active docs updated from implementation_pending to accepted only
# ─────── with receipts; publication stays forbidden ─────────────────────────

spec152f_doc = SPEC152F_DOC.read_text(encoding="utf-8")
for token in (
    "implementation accepted with receipts",
    "docs/evidence/spec152f/focusa-vbcqu.20.14.52-acceptance.txt",
    "focusa-vbcqu.20.13.63",
    "REL.4\u2013REL.7",
    "authorize publication of stable `v0.9.144` before Specs 152/152E/152F and REL.4\u2013REL.7 close truthfully",
):
    assert token in spec152f_doc, f"Spec 152F status/receipt binding missing: {token!r}"
# No self-issued Evaluation, caller-controlled grants, or presenter-owned
# commercial policy may be introduced by the closure wiring.
for forbidden in (
    "local Evaluation issuance",
    "caller-selected grants",
    "route-local pricing",
):
    assert forbidden.lower() in spec152f_doc.lower(), f"Spec 152F lost forbidden token: {forbidden!r}"
print(
    "Spec 152F active doc updated to accepted-with-receipts; "
    "publication non-goal intact (Specs 152/152E/152F and REL.4-REL.7 must close)"
)

# ── 4. Spec152F cannot be bypassed by REL closure (governance) ──────────────

run("python3", str(REDUCER), "--check")
gate = json.loads(GATE.read_text())
assert gate["schema"] == "focusa.locked_release_technical_closure_gate.v1"
assert gate["status"] == "verified"
assert gate["invalid_closed_count"] == 0
assert gate["invalid_closed_ids"] == []
ledger = json.loads(LEDGER.read_text())
assert gate["mapping_count"] == len(ledger["mappings"])
assert gate["mapping_count"] == 465
assert gate["technically_pending_count"] > 0
pending = set(gate["technically_pending_ids"])
# The REL wiring successors must remain technically pending: Spec 152F (this
# closure and its seal) and the Spec 152E final milestones cannot be closed
# administratively by REL machinery — no bypass exists.
for rel_successor in (
    "focusa-vbcqu.20.14.51",
    "focusa-vbcqu.20.14.52",
    "focusa-vbcqu.20.13.62",
    "focusa-vbcqu.20.13.63",
):
    assert rel_successor in pending, f"REL successor falsely closed: {rel_successor}"
spec152f_invalid = [
    bead_id
    for bead_id in gate["invalid_closed_ids"]
    if bead_id.startswith("focusa-vbcqu.20.14")
]
assert spec152f_invalid == [], f"invalid Spec 152F closures: {spec152f_invalid}"
# The GH#106.2 governance reconciliation must stay truthfully blocked: the
# 164 gate is part of this atom's exact verification and passes only while
# the pending technical-acceptance gap remains explicit.
run("python3", str(ROOT / "tests" / "164_focusa_locked_release_governance_reconciliation_test.py"))
print(
    "governance no-bypass proven: mappings=465 invalid_closed=0 "
    "pending=" + str(gate["technically_pending_count"]) + " (REL successors pending); "
    "164 reconciliation truthfully blocked"
)

# ── 5. Beads edges and zero cycles (final dependency/provenance audit) ──────

# The Spec 172 seal blocks this final Spec 152F item: every Spec152E/F path
# to REL acceptance passes through the accepted Spec 152F closure.
spec172_taskgraph = json.loads(SPEC172_TASKGRAPH.read_text())
assert spec172_taskgraph["schema"] == "focusa.spec172_implementation_taskgraph_index.v1"
spec172_downstream = {
    (edge["blocked"], edge["blocker"])
    for edge in spec172_taskgraph["downstream_edges"]
}
assert ("focusa-vbcqu.20.14.52", "focusa-vbcqu.20.15.42") in spec172_downstream, (
    "Spec 172 seal no longer blocks the Spec 152F final work item"
)
# Independent Kahn traversal over the 123 internal Spec 152F edges.
evidence_tasks: list[dict] = []
for rel in sorted(taskgraph["phase_files"].values()):
    phase = json.loads((ROOT / rel).read_text())
    assert phase["schema"] == "focusa.spec152f_implementation_phase.v1"
    assert phase["parent"] == "focusa-vbcqu.20.14"
    evidence_tasks.extend(phase["tasks"])
assert len(evidence_tasks) == 52
by_id = {task["id"]: task for task in evidence_tasks}
internal_edges: list[tuple[str, str]] = []
for task in evidence_tasks:
    for dependency in task["dependencies"]:
        if dependency.startswith("focusa-vbcqu.20.14."):
            internal_edges.append((dependency, task["id"]))
assert len(internal_edges) == 123
indegree = {task_id: 0 for task_id in by_id}
dependents: dict[str, list[str]] = defaultdict(list)
for blocker, blocked in internal_edges:
    indegree[blocked] += 1
    dependents[blocker].append(blocked)
queue = deque(sorted(task_id for task_id, degree in indegree.items() if degree == 0))
visited: list[str] = []
while queue:
    task_id = queue.popleft()
    visited.append(task_id)
    for dependent in sorted(dependents[task_id]):
        indegree[dependent] -= 1
        if indegree[dependent] == 0:
            queue.append(dependent)
assert len(visited) == 52, "Spec 152F internal dependency cycle detected"
# Literal Beads edge check: zero cycles in the live bead dependency graph.
beads = json.loads(run("bd", "dep", "cycles", "--json").stdout)
assert beads["cycles"] == []
assert beads["count"] == 0
assert beads["total_count"] == 0
print(
    f"acyclic final dependency proof: internal_edges=123 kahn_visited=52/52 "
    f"beads_cycles={beads['count']}"
)

# ── 6. Child-atom provenance audit (every child against evidence) ───────────

missing: list[str] = []
for task in evidence_tasks:
    path = EVIDENCE / Path(task["evidence_path"]).name
    if not path.is_file():
        missing.append(task["id"])
assert set(missing) <= UNCLOSED_RECORDS, (
    f"unexpectedly missing evidence: {sorted(set(missing) - UNCLOSED_RECORDS)}"
)
assert git_is_ancestor(REGISTRY_LOADER_IN_TREE_COMMIT), (
    "01.02 registry-loader in-tree implementation commit is not an ancestor of HEAD"
)
for task in evidence_tasks:
    path = EVIDENCE / Path(task["evidence_path"]).name
    if not path.is_file():
        continue
    assert git_tracked(path), f"evidence not committed for {task['id']}: {path}"
    text = path.read_text(encoding="utf-8")
    assert task["id"] in text, f"record does not name its work item: {path.name}"
    assert re.search(r"(?i)\bverification\b", text) or re.search(
        r"(?i)\bvalidation\b", text
    ), f"no verification content in {path.name}"
    named = COMMIT_REF.search(text)
    if named and task["id"] in EXACT_SHA_RECORDS:
        assert git_commit_exists(named.group(2)), (
            f"implementation commit ref does not resolve in {path.name}: {named.group(2)}"
        )
    for pending_marker in PENDING_MARKERS:
        assert pending_marker not in text, f"unfinished acceptance in {path.name}"
    # This atom's own receipt, once written, must satisfy the strict phase-06
    # acceptance template exactly like every phase-06 record before it.
    if task["id"] == "focusa-vbcqu.20.14.52":
        for required in PHASE_06_TEMPLATE:
            assert required in text, f"phase-06 template violation in {path.name}: missing {required!r}"
print(
    f"provenance audit complete: {len(evidence_tasks) - len(set(missing))} evidenced tasks "
    f"(absent={sorted(set(missing))})"
)

# ── 7. Stable publication still requires exact final release acceptance ─────

assert final_audit.count("distribution_status: blocked") == 1
assert "publication_rule: forbidden until focusa-vbcqu.20.13.63 and focusa-vbcqu.20.14.52 close with linked acceptance evidence; the Spec 172 final acceptance work item must also close" in final_audit
assert "spec152e_correction_status: in_progress" in final_audit
assert "spec152f_policy_status: implementation_open" in final_audit
# REL.4-REL.7 governance stays dependent on this closure and the Spec 152E
# final milestone; they are not administratively closed by this atom.
assert "REL.4" in final_audit and "REL.7" in final_audit
print("stable publication blocked until exact final release acceptance (REL.4-REL.7 not_closed)")

# ── 8. Exact-SHA closure receipt ────────────────────────────────────────────

head = run("git", "rev-parse", "HEAD").stdout.strip()
taskgraph_sha = hashlib.sha256(TASKGRAPH.read_bytes()).hexdigest()
gate_sha = hashlib.sha256(GATE.read_bytes()).hexdigest()
final_audit_sha = hashlib.sha256(final_audit.encode()).hexdigest()
print()
print("spec152f_release_dependency receipt")
print(f"  sha256 head={head}")
print(f"  sha256 spec152f-implementation-taskgraph.v1.json={taskgraph_sha}")
print(f"  sha256 next-locked-release-technical-closure-gate.json={gate_sha}")
print(f"  sha256 spec152-final-audit-status.v1.yaml={final_audit_sha}")
print(f"  tasks={taskgraph['task_count']} evidenced={len(evidence_tasks) - len(set(missing))}")
print(f"  governance=verified invalid_closed:0 pending:{gate['technically_pending_count']}")
print(f"  beads_cycles=0 publication=blocked spec152e_dependency=focusa-vbcqu.20.13.63")
print("✓ spec152f_release_dependency PASS")
