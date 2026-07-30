#!/usr/bin/env python3
"""Dependency-free combined Spec140 + Spec140A full-conformance gate."""
from __future__ import annotations

import hashlib
import json
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
C = ROOT / "docs/contracts"


def load(name: str):
    return yaml.safe_load((C / name).read_text())


def require_text(path: str, markers: list[str]) -> None:
    text = (ROOT / path).read_text()
    missing = [marker for marker in markers if marker not in text]
    assert not missing, f"{path}: missing {missing}"


def test_combined_literal_source_coverage_is_exact_and_reproducible() -> None:
    coverage = load("spec140a-normative-source-coverage.v1.yaml")
    assert coverage["runtime_status"] == "verified_complete"
    assert coverage["coverage_status"] == "verified_complete"
    assert coverage["ledger_requirement_count"] == 62
    assert coverage["source_atom_count"] == len(coverage["source_atoms"])
    assert coverage["unmapped_source_atom_refs"] == []
    assert coverage["ambiguous_owner_refs"] == []
    for source in coverage["sources"]:
        assert hashlib.sha256((ROOT / source["path"]).read_bytes()).hexdigest() == source["sha256"]
    assert all(row["coverage_status"] == "mapped" and row["owner_requirement_id"] for row in coverage["source_atoms"])


def test_parent_and_addendum_ledgers_are_62_of_62_verified() -> None:
    parent = load("spec140-complete-feature-ledger.v1.yaml")
    addendum = load("spec140a-complete-feature-ledger.v1.yaml")
    assert len(parent["requirements"]) == 38
    assert len(addendum["requirements"]) == 24
    for ledger in [parent, addendum]:
        assert ledger["runtime_status"] == "verified_complete"
        assert ledger["full_conformance_status"] == "verified_complete"
        assert ledger["verified_requirement_count"] == len(ledger["requirements"])
        for row in ledger["requirements"]:
            assert row["status"] == "verified"
            assert row["implementation_refs"] and row["test_refs"]
            assert row["evidence_refs"] and row["receipt_refs"]


def test_runtime_proof_map_is_one_to_one_and_complete() -> None:
    proof = load("spec140a-runtime-proof-map.v1.yaml")
    assert proof["runtime_status"] == "verified_complete"
    assert proof["full_conformance_status"] == "verified_complete"
    assert proof["row_count"] == proof["verified_row_count"] == 62
    assert len({row["requirement_id"] for row in proof["rows"]}) == 62
    for row in proof["rows"]:
        assert row["status"] == "verified_complete"
        assert row["implementation_refs"] and row["test_refs"]
        assert row["evidence_refs"] and row["receipt_refs"]


def test_typed_contracts_adaptability_and_two_stage_amendment_are_bound() -> None:
    generated = load("spec140a-generated-contracts.v1.yaml")
    adaptability = load("spec140a-instruction-adaptability-matrix.v1.yaml")
    assert generated["runtime_status"] == "verified_complete"
    assert len(generated["contracts"]) == 7
    assert all(row["status"] == "verified_complete" and row["serde_roundtrip"] for row in generated["contracts"])
    assert adaptability["runtime_status"] == "verified_complete"
    assert [row["class"] for row in adaptability["classes"]] == [
        "invariant", "temporally_adaptive", "operator_selectable", "implementation_discretion"
    ]
    assert len(adaptability["allowed_temporal_fields"]) == 7
    require_text("crates/focusa-core/src/agent_runtime_instruction_integrity.rs", [
        "InstructionAdaptabilityClass", "TemporalInstructionAdaptation",
        "CanonicalInstructionAmendment", "OfficialDocumentationSweepManifest",
        "AmendmentApprovalMissing", "AmendmentSweepMissing", "evaluate_instruction_integrity",
        "dynamic_authority_unavailable_fail_closed", "mission_canvas_authoritative: false",
    ])


def test_api_cli_pi_focus_slice_canvas_menubar_and_tui_have_headless_parity() -> None:
    parity = load("spec140a-headless-surface-parity.v1.yaml")
    assert parity["runtime_status"] == "verified_complete"
    assert len(parity["operations"]) == 9
    assert len(parity["surfaces"]) == 8
    assert all(row["status"] == "verified_complete" for row in parity["operations"])
    require_text("crates/focusa-api/src/routes/agent_runtime_integrity.rs", [
        "instruction-integrity/evaluate", "amendments/propose", "amendments/activate",
        "headless/verify", "append_runtime_constitution_event",
    ])
    require_text("crates/focusa-cli/src/commands/agent_runtime.rs", [
        "IntegrityEvaluate", "IntegrityStatus", "AmendmentPropose", "AmendmentActivate", "HeadlessVerify",
    ])
    require_text("apps/pi-extension/src/agent-runtime-tools.ts", [
        "focusa_instruction_integrity_evaluate", "focusa_instruction_integrity_status",
        "focusa_canonical_instruction_amendment_propose", "focusa_canonical_instruction_amendment_activate",
        "focusa_agent_runtime_headless_verify",
    ])
    require_text("apps/pi-extension/src/turns.ts", ["INSTRUCTION_INTEGRITY", "instructionIntegrityContext"])
    require_text("apps/menubar/src/lib/components/MissionCanvasView.svelte", ["InstructionIntegrityPeek"])
    require_text("crates/focusa-tui/src/mission_control.rs", ["instruction_status", "canvas_authority=false"])


def test_all_twenty_four_mandatory_scenarios_are_real_rust_tests() -> None:
    matrix = load("spec140a-scenario-matrix.v1.yaml")
    source = (ROOT / "crates/focusa-core/src/agent_runtime_instruction_integrity_scenario_test.rs").read_text()
    assert matrix["runtime_status"] == "verified_complete"
    assert len(matrix["scenarios"]) == 24
    assert [row["ordinal"] for row in matrix["scenarios"]] == list(range(1, 25))
    for row in matrix["scenarios"]:
        assert row["status"] == "verified_complete"
        assert f"scenario_{row['ordinal']:02d}" in source


def test_generated_capability_catalog_and_receipt_include_instruction_integrity() -> None:
    catalog = json.loads((C / "spec141/generated-capability-v2/pi-tools.json").read_text())
    serialized = json.dumps(catalog)
    for name in [
        "focusa_instruction_integrity_evaluate", "focusa_instruction_integrity_status",
        "focusa_canonical_instruction_amendment_propose",
        "focusa_canonical_instruction_amendment_activate", "focusa_agent_runtime_headless_verify",
    ]:
        assert name in serialized
    receipt = load("140a-focusa-combined-conformance-receipt.v1.yaml")
    assert receipt["status"] == "verified_complete"
    assert receipt["verified_requirement_count"] == receipt["combined_requirement_count"] == 62
    assert receipt["mandatory_scenario_verified_count"] == 24
    assert receipt["canonical_amendment_two_stage_authority"] is True
    assert receipt["mission_canvas_authoritative"] is False
