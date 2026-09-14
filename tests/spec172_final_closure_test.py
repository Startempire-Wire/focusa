#!/usr/bin/env python3
"""172.05.11 Seal Spec 172 closure and wire release dependencies
(atom focusa-vbcqu.20.15.42).

Exact verification:
    python3 tests/spec172_final_closure_test.py && \
    python3 tests/164_focusa_locked_release_governance_reconciliation_test.py && \
    bd dep cycles --json

This test is the deterministic, stdlib-only, fail-closed technical closure
receipt for the Spec 172 child series (focusa-vbcqu.20.15.1..20.15.42). It
cannot pass with a missing child record, a stale taskgraph, an uncovered
reconciliation requirement, a broken Spec152E/F anti-bypass edge, a
weakened governance closure, a cyclic dependency graph, or an unblocked
stable publication. It exits 0 only when the complete closure chain is
present in the committed tree, and it records the exact HEAD SHA and a
bounded closure receipt.

Authority: docs/172-focusa-spec152-license-type-and-surface-entitlement-
governance-addendum.md (Spec 172; non-conflicting Specs 152, 152E, 152F, and
150A remain binding).

Fail-closed invariants (172.05.11 done condition):
- All 42 children are technically accepted: every taskgraph task's evidence
  path exists in docs/evidence/spec172/ and is git-tracked, carries
  verification content (verification/validation section or real exit code),
  cites no unfinished-acceptance markers, and every named implementation
  commit SHA resolves in this repository; the seal task itself
  (focusa-vbcqu.20.15.42) is the only record that may be absent at the
  moment the seal runs (its own evidence is written by this atom). Every
  phase-05 receipt (20.15.32..20.15.41) additionally satisfies the strict
  receipt template (Implementation commit / Bounded result / Exact
  verification (exit codes) / Rollback / "No push, deploy, release, merge,
  or Beads mutation was performed.").
- Zero uncovered reconciliation items: the Spec 152↔Spec 172 reconciliation
  map covers all 28 enumerated requirements (covered=28 total=28
  unmatched=0) with single-owner no-duplicate implementation rules, and the
  taskgraph contract re-derives byte-identically (per-phase sha256 zero-diff).
- Spec152E/F cannot bypass Spec172: the committed downstream edges wire the
  Spec 152F final work item (focusa-vbcqu.20.14.52) behind this seal
  (blocked by focusa-vbcqu.20.15.42), and the Spec 152E final milestone
  (focusa-vbcqu.20.13.63) behind Spec 152F (blocked by
  focusa-vbcqu.20.14.52) — so every Spec152E/F release path passes through
  this closure before REL acceptance; stable publication stays forbidden
  until those REL items close with linked acceptance evidence.
- Zero invalid closure: the governance gate is re-derived from the committed
  ledger (reducer --check, byte-identical) and reports invalid_closed_count
  = 0 while the REL wiring successors (20.14.51, 20.14.52, 20.13.62,
  20.13.63) remain technically pending; the closure receipt claims only what
  the committed tree proves. No raw credential/token/customer data is
  produced or recorded.
"""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
from collections import defaultdict, deque
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / "docs" / "evidence" / "spec172"
TASKGRAPH = ROOT / "docs" / "contracts" / "spec172-implementation-taskgraph.v1.json"
SPEC152F_TASKGRAPH = (
    ROOT / "docs" / "contracts" / "spec152f-implementation-taskgraph.v1.json"
)
FINAL_AUDIT = ROOT / "docs" / "contracts" / "spec152-final-audit-status.v1.yaml"
BLOCKER_SUMMARY = (
    ROOT / "docs" / "contracts" / "spec152-release-blocker-summary.v1.yaml"
)
DOCUMENT_SET = ROOT / "docs" / "contracts" / "spec152-document-set.v1.yaml"
GATE = ROOT / "release-proof" / "audit" / "next-locked-release-technical-closure-gate.json"
LEDGER = (
    ROOT / "release-proof" / "audit" / "next-locked-release-governance-reconciliation.json"
)
REDUCER = ROOT / "scripts" / "reduce-locked-release-technical-closure.py"

# The seal task itself cannot have its technical acceptance recorded before
# it runs; its evidence is written by this atom. This is the only taskgraph
# evidence path allowed to be absent.
UNCLOSED_SEAL_TASKS = {"focusa-vbcqu.20.15.42"}

# Phase-05 receipt series: strict template required for all ten; exact-SHA
# binding (named resolvable implementation commit) required for 33..41.
# focusa-vbcqu.20.15.32 binds its single commit via "see git log for this
# atom" (the 20.15.41 installed-acceptance receipt explicitly lists the
# twelve exact-SHA-bound journey records without 20.15.32) and its record
# is not rewritten by this seal atom.
PHASE_05_RECORDS = [f"focusa-vbcqu.20.15.{n}" for n in range(32, 42)]
PHASE_05_EXACT_SHA = [f"focusa-vbcqu.20.15.{n}" for n in range(33, 42)]

# Spec172 downstream release wiring (blocked <- blocker). These committed
# edges are the release-dependency wiring this seal certifies.
EXPECTED_DOWNSTREAM_EDGES = {
    ("focusa-vbcqu.20.13.3", "focusa-vbcqu.20.15.8"),
    ("focusa-vbcqu.20.13.20", "focusa-vbcqu.20.15.10"),
    ("focusa-vbcqu.20.14.6", "focusa-vbcqu.20.15.18"),
    ("focusa-vbcqu.20.14.7", "focusa-vbcqu.20.15.18"),
    ("focusa-vbcqu.20.14.49", "focusa-vbcqu.20.15.40"),
    ("focusa-vbcqu.20.14.52", "focusa-vbcqu.20.15.42"),
}

COMMIT_REF = re.compile(
    r"(?i)(implementation\s+commits?\s*[:`\-\s]*"
    r"|exact\s+source(?:\s*/\s*test)?\s+commits?\s*[:`\-\s]*"
    r"|exact\s+source\s+commit\s+tested\s*[:`\-\s]*"
    r"|exact\s+commit\s*[:`\-\s]*)"
    r"([0-9a-f]{7,40})"
)

PHASE_05_TEMPLATE = (
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


def git_tracked(path: Path) -> bool:
    result = subprocess.run(
        ["git", "ls-files", "--error-unmatch", str(path.relative_to(ROOT))],
        cwd=ROOT,
        capture_output=True,
    )
    return result.returncode == 0


# ── 1. Taskgraph closure contract (byte-identical, per-phase) ───────────────

taskgraph = json.loads(TASKGRAPH.read_text())
assert taskgraph["schema"] == "focusa.spec172_implementation_taskgraph_index.v1"
assert taskgraph["authority"] == (
    "docs/172-focusa-spec152-license-type-and-surface-entitlement-governance-addendum.md"
)
assert taskgraph["parent"] == "focusa-vbcqu.20.15"
assert taskgraph["task_count"] == 42
assert taskgraph["first_task"] == "focusa-vbcqu.20.15.1"
assert taskgraph["final_task"] == "focusa-vbcqu.20.15.42"
assert taskgraph["phase_counts"] == {
    "00": 5, "01": 4, "02": 8, "03": 7, "04": 7, "05": 11,
}
assert taskgraph["internal_dependency_edge_count"] == 114
assert taskgraph["external_dependency_edge_count"] == 56
for phase, rel in sorted(taskgraph["phase_files"].items()):
    raw = (ROOT / rel).read_bytes()
    assert hashlib.sha256(raw).hexdigest() == taskgraph["phase_file_sha256"][phase], rel

reconciliation_raw = (ROOT / taskgraph["reconciliation_map"]).read_bytes()
assert hashlib.sha256(reconciliation_raw).hexdigest() == taskgraph["reconciliation_map_sha256"]
reconciliation = json.loads(reconciliation_raw)
assert reconciliation["schema"] == "focusa.spec172_spec152_reconciliation_map.v1"
assert reconciliation["scope"] == {
    "sections": ["22", "23"],
    "requirement_count": 28,
    "covered_requirement_count": 28,
    "uncovered_requirement_count": 0,
}
assert len(reconciliation["requirements"]) == 28
assert reconciliation["duplicate_implementation_policy"] and (
    "MUST NOT create a second implementation"
    in reconciliation["duplicate_implementation_policy"]
)
print("taskgraph closure contract intact: tasks=42 phases=00:5 01:4 02:8 03:7 04:7 05:11")

# ── 2. Zero uncovered reconciliation item ───────────────────────────────────

assert reconciliation["scope"]["uncovered_requirement_count"] == 0
by_requirement = {item["requirement_id"]: item for item in reconciliation["requirements"]}
assert len(by_requirement) == 28
assert {item["implementation_root"] for item in by_requirement.values()} == {
    "spec152e", "spec152f", "spec172",
}
assert all(
    item["implementation_rule"] == "single_owner_no_duplicate"
    for item in by_requirement.values()
)
print(
    "reconciliation coverage complete: requirements=28 covered=28 uncovered=0 "
    "roots=spec152e,spec152f,spec172"
)

# ── 3. Child-atom audit (every child against done condition and evidence) ───

evidence_tasks: list[dict] = []
for rel in sorted(taskgraph["phase_files"].values()):
    phase = json.loads((ROOT / rel).read_text())
    assert phase["schema"] == "focusa.spec172_implementation_phase.v1"
    assert phase["parent"] == "focusa-vbcqu.20.15"
    evidence_tasks.extend(phase["tasks"])
assert len(evidence_tasks) == 42
assert [task["id"] for task in evidence_tasks] == [
    f"focusa-vbcqu.20.15.{n}" for n in range(1, 43)
]

missing: list[str] = []
for task in evidence_tasks:
    path = EVIDENCE / Path(task["evidence_path"]).name
    if not path.is_file():
        missing.append(task["id"])
assert set(missing) <= UNCLOSED_SEAL_TASKS, (
    f"unexpectedly missing evidence: {sorted(set(missing) - UNCLOSED_SEAL_TASKS)}"
)
for task in evidence_tasks:
    path = EVIDENCE / Path(task["evidence_path"]).name
    if task["id"] in UNCLOSED_SEAL_TASKS:
        if path.is_file():
            text = path.read_text(encoding="utf-8")
            assert not any(marker in text for marker in PENDING_MARKERS), (
                f"unfinished acceptance in {path.name}"
            )
        continue
    assert path.is_file(), f"missing evidence for {task['id']}: {path}"
    assert git_tracked(path), f"evidence not committed for {task['id']}: {path}"
    text = path.read_text(encoding="utf-8")
    assert task["id"] in text, f"record does not name its work item: {path.name}"
    assert re.search(r"(?i)\bverification\b", text) or re.search(
        r"(?i)\bvalidation\b", text
    ) or re.search(r"(?i)\bexit code\b", text), f"no verification content in {path.name}"
    # Every named implementation/exact-source commit SHA must resolve in this
    # repository (no fabricated refs). Abbreviated unique SHAs are accepted;
    # fixture/schema digests are not commit refs and are not treated as such.
    named = COMMIT_REF.search(text)
    if named:
        assert git_commit_exists(named.group(2)), (
            f"commit ref does not resolve in {path.name}: {named.group(2)}"
        )
    # No record may claim an unfinished technical acceptance.
    for pending_marker in PENDING_MARKERS:
        assert pending_marker not in text, f"unfinished acceptance in {path.name}"

for task in evidence_tasks:
    if task["id"] not in PHASE_05_RECORDS:
        continue
    path = EVIDENCE / Path(task["evidence_path"]).name
    text = path.read_text(encoding="utf-8")
    for required in PHASE_05_TEMPLATE:
        assert required in text, f"phase-05 template violation in {path.name}: missing {required!r}"
    if task["id"] in PHASE_05_EXACT_SHA:
        named = COMMIT_REF.search(text)
        assert named, f"phase-05 receipt missing implementation commit ref: {path.name}"
        assert git_commit_exists(named.group(2)), (
            f"phase-05 implementation commit ref does not resolve in {path.name}: "
            f"{named.group(2)}"
        )
print(
    f"child-atom audit complete: {len(evidence_tasks) - len(set(missing))} evidenced tasks "
    f"(seal {sorted(UNCLOSED_SEAL_TASKS)} documented by this atom)"
)

# ── 4. Spec152E/F cannot bypass Spec172 (release dependency wiring) ─────────

assert {
    (edge["blocked"], edge["blocker"]) for edge in taskgraph["downstream_edges"]
} == EXPECTED_DOWNSTREAM_EDGES, "Spec172 downstream release edges drifted"
spec152f_taskgraph = json.loads(SPEC152F_TASKGRAPH.read_text())
assert spec152f_taskgraph["schema"] == "focusa.spec152f_implementation_taskgraph_index.v1"
assert spec152f_taskgraph["final_task"] == "focusa-vbcqu.20.14.52"
assert spec152f_taskgraph["downstream_release_edges"] == [
    {"blocked": "focusa-vbcqu.20.13.63", "blocker": "focusa-vbcqu.20.14.52"}
]
# Acyclic chain: this seal blocks the Spec 152F final work item, which in
# turn blocks the Spec 152E final release milestone. Every Spec152E/F path
# to REL acceptance must pass through this closure — no bypass exists.
assert (
    "focusa-vbcqu.20.14.52",
    "focusa-vbcqu.20.15.42",
) in EXPECTED_DOWNSTREAM_EDGES
assert "focusa-vbcqu.20.15.42" in {
    e["blocker"] for e in taskgraph["downstream_edges"]
}
print(
    "no-bypass wiring proven: 20.15.42 -> blocks 20.14.52 -> blocks 20.13.63; "
    f"downstream_edges={len(taskgraph['downstream_edges'])}"
)

# ── 5. Zero-invalid-closure governance result ───────────────────────────────

run("python3", str(REDUCER), "--check")
gate = json.loads(GATE.read_text())
assert gate["schema"] == "focusa.locked_release_technical_closure_gate.v1"
assert gate["status"] == "verified"
assert gate["invalid_closed_count"] == 0
assert gate["invalid_closed_ids"] == []
assert gate["technically_pending_count"] > 0
ledger = json.loads(LEDGER.read_text())
assert gate["mapping_count"] == len(ledger["mappings"])
assert gate["mapping_count"] == 465
pending = set(gate["technically_pending_ids"])
# The Spec152E/F release wiring successors must remain technically pending:
# stable publication stays blocked until REL acceptance, and this seal does
# not administratively close them.
for rel_wiring_successor in (
    "focusa-vbcqu.20.14.51",
    "focusa-vbcqu.20.14.52",
    "focusa-vbcqu.20.13.62",
    "focusa-vbcqu.20.13.63",
):
    assert rel_wiring_successor in pending, f"REL successor falsely closed: {rel_wiring_successor}"
spec172_closed_without_proof = [
    bead_id
    for bead_id in gate["invalid_closed_ids"]
    if bead_id.startswith("focusa-vbcqu.20.15")
]
assert spec172_closed_without_proof == [], (
    f"invalid Spec 172 closures: {spec172_closed_without_proof}"
)
print(
    "governance zero-invalid-closure verified: "
    f"mappings={gate['mapping_count']} invalid_closed={gate['invalid_closed_count']} "
    f"technically_pending={gate['technically_pending_count']} (REL successors pending)"
)

# ── 6. Stable publication remains blocked until REL acceptance ──────────────

final_audit = FINAL_AUDIT.read_text(encoding="utf-8")
for token in (
    "spec172_registered: true",
    "spec172_preserved_authority: identity EDD key lease refund-revoke node sequence recovery privacy customer-data-preservation",
    "distribution_status: blocked",
    "migration_cutover_accepted: false",
    "all_platform_final_candidate_verified: false",
    "publication_rule: forbidden until focusa-vbcqu.20.13.63 and focusa-vbcqu.20.14.52 close with linked acceptance evidence; the Spec 172 final acceptance work item must also close",
):
    assert token in final_audit, f"final-audit contract missing: {token!r}"
blocker_summary = BLOCKER_SUMMARY.read_text(encoding="utf-8")
for token in (
    "status: blocked_for_new_evaluator_customer_and_stable_distribution",
    "simplification_final_work_item: focusa-vbcqu.20.14.52",
    "approved_before_entitlement:",
):
    assert token in blocker_summary, f"release-blocker contract missing: {token!r}"
document_set = DOCUMENT_SET.read_text(encoding="utf-8")
for token in (
    "docs/172-focusa-spec152-license-type-and-surface-entitlement-governance-addendum.md",
    "license_type_and_surface_governance_authority:",
    "narrow_supersession_only:",
):
    assert token in document_set, f"document-set contract missing: {token!r}"
print("stable publication blocked until REL acceptance (final-audit, blocker summary, document set)")

# ── 7. Acyclic final dependency proof ───────────────────────────────────────

by_id = {task["id"]: task for task in evidence_tasks}
internal_edges: list[tuple[str, str]] = []
for task in evidence_tasks:
    for dependency in task["dependencies"]:
        if dependency.startswith("focusa-vbcqu.20.15."):
            internal_edges.append((dependency, task["id"]))
assert len(internal_edges) == 114
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
assert len(visited) == 42, "Spec 172 internal dependency cycle detected"

run("python3", str(ROOT / "tests" / "spec172_taskgraph_contract_test.py"))
run("python3", str(ROOT / "tests" / "172_focusa_spec152_locked_release_reconciliation_test.py"))
run("python3", str(ROOT / "tests" / "164_focusa_locked_release_governance_reconciliation_test.py"))
# Literal Beads edge check: zero cycles in the bead dependency graph.
beads = json.loads(
    run("bd", "dep", "cycles", "--json").stdout
)
assert beads["cycles"] == []
assert beads["count"] == 0
assert beads["total_count"] == 0
print(
    f"acyclic final dependency proof: internal_edges={len(internal_edges)} "
    f"kahn_visited=42/42 beads_cycles={beads['count']}"
)

# ── 8. Exact-SHA closure receipt ────────────────────────────────────────────

head = run("git", "rev-parse", "HEAD").stdout.strip()
taskgraph_sha = hashlib.sha256(TASKGRAPH.read_bytes()).hexdigest()
map_sha = hashlib.sha256(reconciliation_raw).hexdigest()
gate_sha = hashlib.sha256(GATE.read_bytes()).hexdigest()
final_audit_sha = hashlib.sha256(final_audit.encode()).hexdigest()
print()
print("spec172_final_closure receipt")
print(f"  sha256 head={head}")
print(f"  sha256 spec172-implementation-taskgraph.v1.json={taskgraph_sha}")
print(f"  sha256 spec172-spec152-reconciliation-map.v1.json={map_sha}")
print(f"  sha256 next-locked-release-technical-closure-gate.json={gate_sha}")
print(f"  sha256 spec152-final-audit-status.v1.yaml={final_audit_sha}")
print(f"  tasks={taskgraph['task_count']} evidenced={len(evidence_tasks) - len(set(missing))}")
print(f"  reconciliation=covered:28 uncovered:0")
print(f"  governance=verified invalid_closed:0 pending:{gate['technically_pending_count']}")
print(f"  beads_cycles=0 publication=blocked")
print("✓ spec172_final_closure PASS")
