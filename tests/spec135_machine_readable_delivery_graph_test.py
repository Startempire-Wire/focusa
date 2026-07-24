#!/usr/bin/env python3
"""Validate the Spec 135 F1 machine-readable delivery contract artifacts."""
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs" / "contracts"


def load(name: str) -> dict:
    path = CONTRACTS / name
    assert path.is_file(), f"missing contract: {path}"
    # JSON is a strict YAML 1.2 subset and keeps validation dependency-free.
    value = json.loads(path.read_text())
    assert isinstance(value, dict), f"contract must be an object: {name}"
    return value


ledger = load("spec135-complete-feature-ledger.v1.yaml")
dag = load("spec135-delivery-dag.v1.yaml")
clients = load("spec135-client-parity-matrix.v1.yaml")
framework = load("spec135-framework-lock.v1.yaml")
proof = load("spec135-proof-matrix.v1.yaml")

requirements = ledger["requirements"]
assert ledger["schema"] == "focusa.spec135.feature_ledger.v1"
assert ledger["series"] == "135-135K"
assert ledger["requirement_count"] == len(requirements) >= 73

required_fields = {
    "requirement_id", "source_spec", "source_section", "normative_text",
    "primitive_owner", "repository_owner", "current_status", "dependencies",
    "implementation_tasks", "core_types", "reducer_actions", "api_operations",
    "generated_contracts", "generated_ui_surfaces", "client_surfaces",
    "uiai_eval_scenarios", "tests", "evidence_requirements",
    "receipt_requirements", "migration_requirements", "closure_status",
}
status_values = {"missing", "partial", "implemented", "verified"}
closure_values = {"open", "blocked", "verified", "operator_removed"}
ids = set()
for row in requirements:
    assert required_fields <= row.keys(), (row.get("requirement_id"), required_fields - row.keys())
    rid = row["requirement_id"]
    assert rid.startswith("SPEC135-") and rid not in ids
    ids.add(rid)
    assert row["current_status"] in status_values
    assert row["closure_status"] in closure_values
    assert row["implementation_tasks"] and row["implementation_tasks"][0].startswith("focusa-mc-")
    assert row["tests"] and row["evidence_requirements"] and row["receipt_requirements"]
    assert set(row["dependencies"]) <= ids | {r["requirement_id"] for r in requirements}

assert {"SPEC135-F0", "SPEC135-F1", "SPEC135-F12", "SPEC135-ALPHA1", "SPEC135-ALPHA8", "SPEC135-Z5"} <= ids
assert next(r for r in requirements if r["requirement_id"] == "SPEC135-F0")["closure_status"] == "verified"
verified_requirements = {
    *(f"SPEC135-F{i}" for i in range(13)),
    "SPEC135-C1", "SPEC135-C2", "SPEC135-C3",
    "SPEC135-RI1", "SPEC135-RI2", "SPEC135-RI3",
    "SPEC135-P1", "SPEC135-ST1", "SPEC135-ST2", "SPEC135-ST3", "SPEC135-ST4",
    "SPEC135-M1", "SPEC135-M2", "SPEC135-M3", "SPEC135-M4", "SPEC135-M5", "SPEC135-M6", "SPEC135-M7",
    "SPEC135-U1", "SPEC135-U2", "SPEC135-U3", "SPEC135-U4",
    "SPEC135-V1", "SPEC135-V2",
    "SPEC135-ALPHA1", "SPEC135-ALPHA2", "SPEC135-ALPHA3", "SPEC135-ALPHA4", "SPEC135-ALPHA5", "SPEC135-ALPHA6",
}
for completed in verified_requirements:
    row = next(r for r in requirements if r["requirement_id"] == completed)
    assert row["current_status"] == "verified"
    assert row["closure_status"] == "verified"

for pending in ("SPEC135-U5", "SPEC135-U6", "SPEC135-V3", "SPEC135-V5", "SPEC135-V6"):
    row = next(r for r in requirements if r["requirement_id"] == pending)
    assert row["current_status"] == "partial" and row["closure_status"] == "open"

node_ids = {n["requirement_id"] for n in dag["nodes"]}
assert dag["schema"] == "focusa.spec135.delivery_dag.v1"
assert node_ids == ids
assert dag["foundation_order"] == [f"SPEC135-F{i}" for i in range(13)]
assert dag["alpha_order"] == [f"SPEC135-ALPHA{i}" for i in range(1, 9)]
assert dag["parallel_lane_gate"] == "SPEC135-F4"
assert dag["permanent_integration_gate"] == "SPEC135-Z2"
assert dag["final_closure"] == "SPEC135-Z5"

incoming = {rid: set() for rid in ids}
for edge in dag["edges"]:
    assert edge["kind"] == "blocks"
    assert edge["from"] in ids and edge["to"] in ids
    incoming[edge["to"]].add(edge["from"])
remaining = {rid: set(deps) for rid, deps in incoming.items()}
seen = []
while remaining:
    ready = sorted(rid for rid, deps in remaining.items() if not deps)
    assert ready, f"delivery DAG cycle: {sorted(remaining)}"
    seen.extend(ready)
    for rid in ready:
        remaining.pop(rid)
    for deps in remaining.values():
        deps.difference_update(ready)
assert len(seen) == len(ids)

client_ids = {c["client_id"] for c in clients["clients"]}
assert clients["schema"] == "focusa.spec135.client_parity_matrix.v1"
assert {"api", "cli", "pi", "typescript", "go", "mission_canvas", "uiai_engine_cockpit", "menubar", "tui", "pwa"} <= client_ids
assert {r["requirement_id"] for r in clients["requirements"]} == ids
assert all(set(r["required_clients"]) <= client_ids for r in clients["requirements"])

assert framework["schema"] == "focusa.spec135.framework_lock.v1"
lock_text = json.dumps(framework)
for marker in ("OpenAPI 3.0.3", "A2UI v0.9.1", "openapi-typescript", "UIAI Engine Eval", "Playwright in Focusa"):
    assert marker in lock_text
assert framework["adoption_order"][-1] == "custom only after failing conformance fixture"

assert proof["schema"] == "focusa.spec135.proof_matrix.v1"
assert {r["requirement_id"] for r in proof["requirements"]} == ids
assert all(r["tests"] and r["evidence_requirements"] and r["receipt_requirements"] for r in proof["requirements"])

ledger_by_id = {r["requirement_id"]: r for r in requirements}
dag_by_id = {r["requirement_id"]: r for r in dag["nodes"]}
proof_by_id = {r["requirement_id"]: r for r in proof["requirements"]}
for requirement_id in ids:
    ledger_row = ledger_by_id[requirement_id]
    assert dag_by_id[requirement_id]["status"] == ledger_row["current_status"]
    assert proof_by_id[requirement_id]["closure_status"] == ledger_row["closure_status"]
    expected_closure = "verified" if ledger_row["current_status"] == "verified" else "open"
    assert ledger_row["closure_status"] == expected_closure
assert "permanent onboarding-to-exact-resume traversal" in proof["global_closure_gates"]

print(f"Spec 135 machine-readable delivery graph: PASS ({len(ids)} requirements, {len(dag['edges'])} edges)")
