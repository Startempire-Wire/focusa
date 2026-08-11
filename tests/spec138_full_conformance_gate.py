#!/usr/bin/env python3
"""Fail-closed full-conformance gate for parent Spec 138."""
from __future__ import annotations

import json
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
PARENT = "docs/138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md"


def load(name: str):
    return yaml.safe_load((CONTRACTS / name).read_text())


def require_text(path: str, needles: list[str]) -> None:
    text = (ROOT / path).read_text()
    missing = [needle for needle in needles if needle not in text]
    assert not missing, f"{path}: missing {missing}"


def test_parent_spec138_ledger_has_zero_open_rows() -> None:
    ledger = load("spec138-complete-feature-ledger.v1.yaml")
    rows = [row for row in ledger["requirements"] if row["source_path"] == PARENT]
    assert len(rows) == 273
    assert all(row["runtime_status"] == "verified_complete" for row in rows)
    for row in rows:
        assert row["documentation_status"] == "verified_complete"
        assert row["evidence_status"] == "verified_complete"
        assert row["receipt_status"] == "verified_complete"
        assert row["implementation_refs"]
        assert row["test_refs"]
        assert row["evidence_refs"]
        assert row["receipt_refs"]


def test_profiles_a_through_h_are_evidence_backed_and_cross_cutting_gates_closed() -> None:
    matrix = load("spec138-profile-activation-and-conformance-matrix.v1.yaml")
    assert matrix["runtime_claim"] == "full_spec138_conformance"
    assert matrix["runtime_status"] == "verified_complete"
    assert matrix["full_conformance_status"] == "verified_complete"
    assert matrix["required_scorer_count"] == 31
    for gate in [
        "durable_append_only_history",
        "migration_verified",
        "client_parity_verified",
        "security_verified",
    ]:
        assert matrix[gate] is True
    assert [row["profile"] for row in matrix["profiles"]] == list("ABCDEFGH")
    for row in matrix["profiles"]:
        assert row["status"] == "verified_complete"
        for field in ["runtime_ref", "test_ref", "evidence_ref", "receipt_ref"]:
            assert row[field]


def test_all_legacy_sources_have_noncanonical_migration_proof() -> None:
    matrix = load("spec138-migration-matrix.v1.yaml")
    assert matrix["runtime_status"] == "verified_complete"
    assert matrix["status"] == "verified_complete"
    assert len(matrix["sources"]) == 7
    assert set(matrix["requirements"]) == {
        "readable", "lineage_preserved", "ambiguity_labeled",
        "no_manufactured_authority", "restart", "replay", "rollback", "receipt",
    }
    require_text("crates/focusa-core/src/prediction_migration.rs", [
        "LegacyAuthorityStatus", "ReadableAdvisory", "QuarantinedAmbiguous",
        "source_sha256", "rollback_ref", "LegacyPromotions",
    ])


def test_all_supported_clients_have_operation_parity() -> None:
    matrix = load("spec138-operation-client-parity-matrix.v1.yaml")
    assert matrix["runtime_status"] == "verified_complete"
    assert matrix["full_conformance_status"] == "verified_complete"
    assert len(matrix["rows"]) == 27
    fields = [
        "api_ref", "operation_registry_ref", "generated_contract_ref", "cli_ref",
        "pi_ref", "focus_slice_ref", "mission_canvas_ref", "tui_ref",
        "menubar_ref", "docs_ref", "test_ref",
    ]
    for row in matrix["rows"]:
        assert row["status"] == "verified_complete"
        assert all(row.get(field) for field in fields), (row["operation"], row)


def test_operation_registry_openapi_and_generated_capability_are_live() -> None:
    registry = json.loads((CONTRACTS / "spec135/generated-contract-v1/operation-registry.json").read_text())
    ids = {row["operation_id"] for row in registry["operations"]}
    assert "focusa.prediction_authority.append" in ids
    assert "focusa.prediction_authority.projection" in ids
    openapi = json.loads((CONTRACTS / "spec135/generated-contract-v1/openapi-3.0.3.json").read_text())
    assert openapi["paths"]["/v1/prediction-authority/events"]["post"]["operationId"] == "focusa.prediction_authority.append"
    assert openapi["paths"]["/v1/prediction-authority/projection"]["get"]["operationId"] == "focusa.prediction_authority.projection"
    descriptors = json.loads((CONTRACTS / "spec141/generated-capability-v2/pi-tools.json").read_text())
    assert "focusa_prediction_authority" in json.dumps(descriptors)


def test_api_cli_pi_focus_slice_canvas_menubar_and_tui_surfaces_are_bound() -> None:
    require_text("crates/focusa-api/src/routes/prediction_authority.rs", [
        "PersistentPredictionAuthorityLedger", "projection_get", "profile_conformance",
        '"/v1/prediction-authority/events"', '"/v1/prediction-authority/projection"',
    ])
    require_text("crates/focusa-cli/src/commands/predict.rs", ["AuthorityAppend", "AuthorityProjection"])
    require_text("apps/pi-extension/src/tools.ts", ["focusa_prediction_authority"])
    require_text("apps/pi-extension/src/turns.ts", ["EPISTEMIC_AUTHORITY", "predictionAuthorityContext"])
    require_text("apps/menubar/src/routes/+page.svelte", ["predictionAuthorityPath", "predictionAuthority"])
    require_text("apps/menubar/src/lib/components/MissionCanvasView.svelte", ["EpistemicAuthorityPeek"])
    require_text("crates/focusa-tui/src/app.rs", ["prediction_authority", "encode_query_component"])
    require_text("crates/focusa-tui/src/mission_control.rs", ["epistemic_conformance", "epistemic  events="])


def test_typed_profile_gate_is_fail_closed() -> None:
    require_text("crates/focusa-core/src/prediction_profiles.rs", [
        "FullSpec138Conformance", "MissingProfile", "ScorerRegistryIncomplete",
        "DurableHistoryRequired", "MigrationRequired", "ClientParityRequired",
        "SecurityRequired", "SubsetLabelRequired",
    ])


if __name__ == "__main__":
    from run_spec137_138_full_conformance_gates import run_test_functions

    raise SystemExit(
        1 if run_test_functions(globals(), "tests/spec138_full_conformance_gate.py") else 0
    )
