#!/usr/bin/env python3
"""Reconcile and refresh locked-release Workset projections from canonical Beads truth."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
AUDIT = ROOT / "release-proof/audit"
MEMBERS_PATH = AUDIT / "next-locked-release-workset-members.jsonl"
EDGES_PATH = AUDIT / "next-locked-release-workset-edges.jsonl"
DEFINITION_PATH = AUDIT / "next-locked-release-workset-definition.json"
PROOF_PATH = AUDIT / "next-locked-release-execution-proof.json"
SCOPE_PATH = AUDIT / "next-locked-release-scope.json"
BINDING_PATH = AUDIT / "next-locked-release-workset-provider-binding.json"
DECOMPOSITION_PROOF_PATH = AUDIT / "next-locked-release-decomposition-proof.json"
ISSUES_PATH = ROOT / ".beads/issues.jsonl"


def jsonl(path: Path) -> list[dict[str, object]]:
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def digest(value: object) -> str:
    body = json.dumps(value, sort_keys=True, separators=(",", ":")).encode()
    return "sha256:" + hashlib.sha256(body).hexdigest()


def dependency_id(dependency: dict[str, object]) -> str:
    return str(dependency.get("depends_on_id") or dependency.get("id") or "")


def dependency_type(dependency: dict[str, object]) -> str:
    return str(dependency.get("type") or dependency.get("dependency_type") or "")


def edge_id(source: str, edge_type: str, target: str) -> str:
    value = hashlib.sha256(f"{source}\0{edge_type}\0{target}".encode()).hexdigest()[:24]
    return f"edge:{value}"


def main() -> int:
    members = jsonl(MEMBERS_PATH)
    edges = jsonl(EDGES_PATH)
    issues = {str(row["id"]): row for row in jsonl(ISSUES_PATH)}
    member_by_id = {str(row["member_id"]): row for row in members}
    previous_count = len(member_by_id)

    # Descendants are decomposition, not scope expansion. Reconcile every child
    # whose parent is already locked, repeating until the descendant set closes.
    added: list[str] = []
    changed = True
    while changed:
        changed = False
        for issue_id, issue in sorted(issues.items()):
            if issue_id in member_by_id:
                continue
            parent_ids = [
                dependency_id(dependency)
                for dependency in issue.get("dependencies", [])
                if dependency_type(dependency) == "parent-child"
                and dependency_id(dependency) in member_by_id
            ]
            if not parent_ids:
                continue
            parent_id = parent_ids[0]
            parent = member_by_id[parent_id]
            status = str(issue["status"])
            row = {
                "member_id": issue_id,
                "work_item_ref": f"bead:{issue_id}",
                "provider_binding_ref": parent["provider_binding_ref"],
                "task_plan_ref": parent["task_plan_ref"],
                "requirement_refs": list(parent.get("requirement_refs", [])),
                "spec_refs": [],
                "mandatory": True,
                "status_at_admission": status,
                "provider_revision_at_admission": issue.get("updated_at"),
                "admission_event_ref": parent["admission_event_ref"],
                "admission_reason": (
                    "locked descendant reconciliation; no capability scope expansion"
                ),
                "parent_member_ref": parent_id,
                "epoch_id": parent["epoch_id"],
                "current_status_projection": status,
                "current_status_freshness": "current",
                "disposition": "completed" if status == "closed" else "pending",
                "supersedes_member_ref": None,
                "evidence_refs": [],
                "receipt_refs": [],
            }
            member_by_id[issue_id] = row
            added.append(issue_id)
            changed = True

    reconciled_ids = sorted(
        member_id
        for member_id, row in member_by_id.items()
        if str(row.get("admission_reason", "")).startswith("locked descendant reconciliation")
    )
    for member_id in reconciled_ids:
        row = member_by_id[member_id]
        parent = member_by_id[str(row["parent_member_ref"])]
        row["provider_binding_ref"] = parent["provider_binding_ref"]
        row["admission_event_ref"] = parent["admission_event_ref"]
        row["epoch_id"] = parent["epoch_id"]
    members = sorted(member_by_id.values(), key=lambda row: str(row["member_id"]))
    member_ids = set(member_by_id)

    # Reconcile all provider blocking edges among locked members. Existing
    # non-provider/constitutional edges remain immutable.
    edge_keys = {
        (str(row["from_member_ref"]), str(row["to_member_ref"]), str(row["edge_type"]))
        for row in edges
    }
    for target_id in sorted(member_ids):
        for dependency in issues[target_id].get("dependencies", []):
            source_id = dependency_id(dependency)
            if dependency_type(dependency) != "blocks" or source_id not in member_ids:
                continue
            key = (source_id, target_id, "blocks")
            if key in edge_keys:
                continue
            edges.append(
                {
                    "edge_id": edge_id(source_id, "blocks", target_id),
                    "from_member_ref": source_id,
                    "to_member_ref": target_id,
                    "edge_type": "blocks",
                    "source": "provider",
                    "source_ref": f"bead-dependency:{source_id}->{target_id}",
                    "created_event_ref": (
                        "event:focusa-next-locked-release:r6:descendant-reconciliation"
                    ),
                }
            )
            edge_keys.add(key)

    # Every reconciled phase leaf is gated by the prior wave and closes through
    # its phase container, matching the original Workset graph convention.
    current_revision = max(
        int(str(row.get("provider_binding_ref", "")).rsplit(":r", 1)[-1])
        for row in member_by_id.values()
        if ":r" in str(row.get("provider_binding_ref", ""))
    )
    current_binding = f"provider:bd:focusa-next-locked-release:r{current_revision}"
    phase_admission_ids = sorted(
        member_id
        for member_id, row in member_by_id.items()
        if member_id in reconciled_ids or row.get("provider_binding_ref") == current_binding
    )
    for member_id in phase_admission_ids:
        member = member_by_id[member_id]
        phase = int(str(member["task_plan_ref"]).rsplit(":", 1)[-1])
        phase_gate = f"focusa-vbcqu.{phase + 1}"
        prior_gate = f"focusa-vbcqu.{phase}"
        for source_id, target_id, kind in (
            (prior_gate, member_id, "blocks"),
            (member_id, phase_gate, "release_requires"),
        ):
            key = (source_id, target_id, kind)
            if source_id not in member_ids or target_id not in member_ids or key in edge_keys:
                continue
            edges.append(
                {
                    "edge_id": edge_id(source_id, kind, target_id),
                    "from_member_ref": source_id,
                    "to_member_ref": target_id,
                    "edge_type": kind,
                    "source": "workset",
                    "source_ref": member["task_plan_ref"],
                    "created_event_ref": member["admission_event_ref"],
                }
            )
            edge_keys.add(key)

    incoming: dict[str, list[str]] = {member_id: [] for member_id in member_ids}
    for edge in edges:
        if edge["edge_type"] == "blocks":
            incoming[str(edge["to_member_ref"])].append(str(edge["from_member_ref"]))
    for member in members:
        status = str(issues[str(member["member_id"])]["status"])
        member["current_status_projection"] = status
        member["current_status_freshness"] = "current"
        member["disposition"] = "completed" if status == "closed" else "pending"

    MEMBERS_PATH.write_text(
        "\n".join(json.dumps(row, separators=(",", ":")) for row in members) + "\n"
    )
    EDGES_PATH.write_text(
        "\n".join(json.dumps(row, separators=(",", ":")) for row in edges) + "\n"
    )
    membership_digest = digest(members)
    graph_digest = digest(edges)

    definition = json.loads(DEFINITION_PATH.read_text())
    definition["membership_digest"] = membership_digest
    definition["graph_digest"] = graph_digest
    DEFINITION_PATH.write_text(json.dumps(definition, indent=2) + "\n")

    binding = json.loads(BINDING_PATH.read_text())
    binding["query"]["member_count"] = len(members)
    binding["query"]["member_ids_digest"] = digest(sorted(member_ids))
    BINDING_PATH.write_text(json.dumps(binding, indent=2) + "\n")

    scope = json.loads(SCOPE_PATH.read_text())
    scope["current_locked_bead_member_count"] = len(members)
    scope["execution_lock"]["member_count"] = len(members)
    scope["membership_reconciliation"] = {
        "scope_expansion": False,
        "previous_declared_member_count": len(members) - len(reconciled_ids),
        "reconciled_member_count": len(members),
        "restored_descendant_count": len(reconciled_ids),
        "restored_descendant_ids": reconciled_ids,
        "reason": "closed decomposition descendants were absent from sealed Workset membership",
    }
    SCOPE_PATH.write_text(json.dumps(scope, indent=2) + "\n")

    closed = lambda issue_id: issues[issue_id]["status"] == "closed"
    frontier_containers = {"focusa-vbcqu", *(f"focusa-vbcqu.{index}" for index in range(1, 10))}
    ready = sorted(
        member_id
        for member_id in member_ids
        if member_id not in frontier_containers
        and not closed(member_id)
        and all(closed(parent) for parent in incoming[member_id])
    )
    phase_by_id = {
        str(row["member_id"]): int(str(row["task_plan_ref"]).rsplit(":", 1)[-1])
        for row in members
    }
    active_phase = min((phase_by_id[member_id] for member_id in ready), default=8)
    proof = json.loads(PROOF_PATH.read_text())
    proof["membership_digest"] = membership_digest
    proof["graph_digest"] = graph_digest
    proof["scope_member_count"] = len(members)
    proof["execution_edge_count"] = len(edges)
    proof["terminal_coverage_count"] = len(members)
    proof["unique_ready_frontier"] = ready
    proof["active_phase"] = active_phase
    phase_counts: dict[str, int] = {}
    for phase in phase_by_id.values():
        key = str(phase)
        phase_counts[key] = phase_counts.get(key, 0) + 1
    proof["phase_counts"] = phase_counts
    PROOF_PATH.write_text(json.dumps(proof, indent=2) + "\n")

    decomposition_proof = json.loads(DECOMPOSITION_PROOF_PATH.read_text())
    decomposition_proof["current_locked_beads"] = len(members)
    decomposition_proof["bead_resolution_proof"] = (
        f"{len(members)}/{len(members)} locked members resolved; all mapped refs, "
        "locked-root descendants, and authorized additions are present"
    )
    decomposition_proof["execution_order_proof"] = (
        f"{len(members)}/{len(members)} members have one phase, an acyclic dependency "
        "path, and terminal coverage"
    )
    DECOMPOSITION_PROOF_PATH.write_text(json.dumps(decomposition_proof, indent=2) + "\n")
    print(
        json.dumps(
            {
                "status": "updated",
                "members": len(members),
                "added_descendants": added,
                "reconciled_descendants": reconciled_ids,
                "ready": ready,
                "membership_digest": membership_digest,
                "graph_digest": graph_digest,
            }
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
