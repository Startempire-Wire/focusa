#!/usr/bin/env python3
"""Dependency-free full combined-source conformance gate for Spec138A."""
from __future__ import annotations

import hashlib
from pathlib import Path

from structured_contract_loader import load_contract_mapping

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
PARENT = "docs/138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md"
ADDENDUM = "docs/138a-focusa-epistemic-zero-deferral-profile-completeness-and-omission-firewall-addendum.md"


def load(name: str):
    return load_contract_mapping(CONTRACTS / name)


def test_combined_source_coverage_has_exact_live_hashes_and_no_omissions() -> None:
    coverage = load("spec138a-normative-source-coverage.v1.yaml")
    assert coverage["runtime_status"] == "verified_complete"
    assert coverage["coverage_status"] == "verified_complete"
    assert coverage["source_atom_count"] == len(coverage["source_atoms"])
    assert coverage["source_atom_count"] == sum(
        row["nonempty_source_atom_count"] for row in coverage["sources"]
    )
    assert coverage["normative_requirement_count"] == sum(
        row["normative_requirement_count"] for row in coverage["sources"]
    )
    assert coverage["unmapped_source_atom_refs"] == []
    assert coverage["unmapped_normative_requirement_refs"] == []
    assert coverage["weakened_mapping_refs"] == []
    assert coverage["ambiguous_applicability_refs"] == []
    assert all(row["coverage_status"] == "mapped" for row in coverage["source_atoms"])
    for source in coverage["sources"]:
        digest = hashlib.sha256((ROOT / source["path"]).read_bytes()).hexdigest()
        assert digest == source["sha256"], source["path"]


def test_complete_ledger_matches_generated_coverage_with_no_open_rows() -> None:
    ledger = load("spec138-complete-feature-ledger.v1.yaml")
    coverage = load("spec138a-normative-source-coverage.v1.yaml")
    rows = ledger["requirements"]
    source_counts = {
        row["path"]: row["normative_requirement_count"] for row in coverage["sources"]
    }
    assert ledger["runtime_status"] == "verified_complete"
    assert ledger["full_conformance_status"] == "verified_complete"
    assert len(rows) == coverage["normative_requirement_count"]
    assert sum(row["source_path"] == PARENT for row in rows) == source_counts[PARENT]
    assert sum(row["source_path"] == ADDENDUM for row in rows) == source_counts[ADDENDUM]
    assert len({row["requirement_id"] for row in rows}) == len(rows)
    assert len({row["source_atom_ref"] for row in rows}) == len(rows)
    for row in rows:
        assert row["runtime_status"] == "verified_complete"
        assert row["documentation_status"] == "verified_complete"
        assert row["evidence_status"] == "verified_complete"
        assert row["receipt_status"] == "verified_complete"
        assert row["implementation_refs"] and row["test_refs"]
        assert row["evidence_refs"] and row["receipt_refs"]


def test_runtime_proof_map_is_exact_one_to_one_and_fully_proven() -> None:
    ledger = load("spec138-complete-feature-ledger.v1.yaml")
    proof = load("spec138-runtime-proof-map.v1.yaml")
    assert proof["status"] == "verified_complete"
    assert proof["full_conformance_status"] == "verified_complete"
    assert proof["row_count"] == proof["verified_row_count"] == len(ledger["requirements"])
    assert proof["combined_source_sha256"] == ledger["combined_normative_source_hash"]
    ledger_ids = {row["requirement_id"] for row in ledger["requirements"]}
    proof_ids = {row["requirement_id"] for row in proof["rows"]}
    assert proof_ids == ledger_ids
    for row in proof["rows"]:
        assert row["status"] == "verified_complete"
        assert row["implementation_refs"] and row["test_refs"]
        assert row["evidence_refs"] and row["receipt_refs"]


def test_profiles_scorers_migration_operations_and_security_are_complete() -> None:
    profile = load("spec138-profile-activation-and-conformance-matrix.v1.yaml")
    scorer = load("spec138-scorer-and-calibration-matrix.v1.yaml")
    migration = load("spec138-migration-matrix.v1.yaml")
    parity = load("spec138-operation-client-parity-matrix.v1.yaml")
    security = load("spec138-security-privacy-retention-matrix.v1.yaml")
    assert profile["full_conformance_status"] == "verified_complete"
    assert [row["profile"] for row in profile["profiles"]] == list("ABCDEFGH")
    assert all(row["status"] == "verified_complete" for row in profile["profiles"])
    assert scorer["runtime_status"] == "verified_complete"
    assert len(scorer["scorers"]) == 31
    assert migration["runtime_status"] == "verified_complete" and len(migration["sources"]) == 7
    assert parity["runtime_status"] == "verified_complete" and len(parity["rows"]) == 27
    assert security["runtime_status"] == "verified_complete"


def test_overrides_ownership_delivery_dag_and_placeholder_firewall_are_closed() -> None:
    override = load("spec138a-parent-override-map.v1.yaml")
    ownership = load("spec138-primitive-ownership-matrix.v1.yaml")
    dag = load("spec138-delivery-dag.v1.yaml")
    audit = load("spec138-forbidden-placeholder-audit.v1.yaml")
    assert override["runtime_status"] == "verified_complete"
    assert all(row["status"] == "verified_complete" for row in override["overrides"])
    assert ownership["runtime_status"] == "verified_complete"
    assert all(row["status"] == "verified_complete" for row in ownership["rows"])
    assert dag["runtime_status"] == "verified_complete" and len(dag["nodes"]) == 9
    settled: set[str] = set()
    for node in dag["nodes"]:
        assert node["status"] == "verified_complete"
        assert set(node["depends_on"]) <= settled
        assert node["evidence_refs"] and node["receipt_ref"]
        settled.add(node["id"])
    assert audit["runtime_status"] == "verified_complete"
    assert audit["result"] == "verified_zero_hits"
    assert audit["forbidden_hit_refs"] == []


def test_all_nineteen_proof_families_and_exact_sha_receipt_are_verified() -> None:
    matrix = load("spec138-proof-matrix.v1.yaml")
    receipt = load("138a-focusa-zero-deferral-conformance-receipt.v1.yaml")
    ledger = load("spec138-complete-feature-ledger.v1.yaml")
    assert matrix["runtime_status"] == "verified_complete"
    assert matrix["exact_sha_integrated_proof_required"] is True
    assert len(matrix["proof_families"]) == 19
    assert {row["family"] for row in matrix["proof_results"]} == set(matrix["proof_families"])
    assert all(row["status"] == "verified_complete" for row in matrix["proof_results"])
    assert receipt["status"] == "verified_complete"
    assert receipt["verified_requirement_count"] == receipt["normative_requirement_count"] == len(ledger["requirements"])
    assert receipt["combined_source_sha256"] == ledger["combined_normative_source_hash"]
    assert receipt["exact_sha_integrated_proof"] is True
    assert receipt["forbidden_placeholder_hit_count"] == 0


def test_typed_zero_deferral_validator_and_adversarial_tests_are_bound() -> None:
    text = (ROOT / "crates/focusa-core/src/epistemic_conformance.rs").read_text()
    for marker in [
        "DeferredRequirement", "UnknownRequirement", "MissingApplicabilityDecision",
        "MissingRowProof", "MissingProfile", "OperationParityRequired",
        "IntegratedProofRequired", "ForbiddenPlaceholder", "DeliveryDependencyUnsettled",
        "validate_requirement_removal",
    ]:
        assert marker in text


if __name__ == "__main__":
    from run_spec137_138_full_conformance_gates import run_test_functions

    raise SystemExit(
        1 if run_test_functions(globals(), "tests/spec138a_full_conformance_gate.py") else 0
    )
