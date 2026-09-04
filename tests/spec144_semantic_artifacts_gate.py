#!/usr/bin/env python3
"""Spec144 §§5-11 semantic artifact and deterministic registry gate."""
from pathlib import Path
import hashlib
import json
import subprocess

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts/spec144"
REGISTRY_PATH = CONTRACTS / "semantic-artifact-registry-v1.json"
REGISTRY = json.loads(REGISTRY_PATH.read_text())
LEDGER = json.loads((ROOT / "docs/contracts/spec144-complete-feature-ledger.v1.yaml").read_text())
ACTIVATION = json.loads((ROOT / "docs/contracts/spec144-activation.v1.json").read_text())
SPEC143_RECEIPT_PATH = ROOT / "docs/contracts/spec143-completion-receipt.v1.json"
SPEC143_RECEIPT = json.loads(SPEC143_RECEIPT_PATH.read_text())
SPEC143_TRANSPORT = json.loads(
    (ROOT / "docs/contracts/spec143-completion-evidence-transport.v1.json").read_text()
)
INTEGRITY = (ROOT / "crates/focusa-core/src/semantic_integrity.rs").read_text()
SEMANTIC_REGISTRY = (ROOT / "crates/focusa-core/src/semantic_registry.rs").read_text()


def digest_bytes(payload: bytes) -> str:
    return "sha256:" + hashlib.sha256(payload).hexdigest()


def digest(path: Path) -> str:
    return digest_bytes(path.read_bytes())


assert SPEC143_RECEIPT["status"] == "passed"
assert len(SPEC143_RECEIPT["gate_evidence"]) == 7
assert SPEC143_TRANSPORT["schema"] == "focusa.spec143_completion_evidence_transport.v1"
assert SPEC143_TRANSPORT["receipt_ref"] == SPEC143_RECEIPT_PATH.relative_to(ROOT).as_posix()
assert SPEC143_TRANSPORT["receipt_sha256"] == digest(SPEC143_RECEIPT_PATH)
assert SPEC143_TRANSPORT["source_commit"] == SPEC143_RECEIPT["source_commit"]
assert SPEC143_TRANSPORT["current_gate_owner"] == "scripts/ci/run-spec-gates.sh"
historical_root = ROOT / SPEC143_TRANSPORT["immutable_blob_root"]
source_commit = SPEC143_RECEIPT["source_commit"]
for gate in SPEC143_RECEIPT["gate_evidence"]:
    historical_path = historical_root / gate["path"]
    assert historical_path.is_file(), f"missing immutable Spec143 evidence: {historical_path}"
    historical_bytes = historical_path.read_bytes()
    assert digest_bytes(historical_bytes) == gate["sha256"]
    assert (ROOT / gate["path"]).is_file(), f"missing current Spec143 gate: {gate['path']}"
    git_blob = subprocess.run(
        ["git", "show", f"{source_commit}:{gate['path']}"],
        cwd=ROOT,
        capture_output=True,
        check=False,
    )
    if git_blob.returncode == 0:
        assert git_blob.stdout == historical_bytes
    assert gate["result"] == "passed"
assert ACTIVATION["schema"] == "focusa.spec144_activation.v1"
assert ACTIVATION["status"] == "activated"
assert ACTIVATION["activation_receipt_ref"] == "release-proof/audit/spec144-spec150-double-e2e-receipt.json"
assert ACTIVATION["spec143_completion_receipt_ref"] == "docs/contracts/spec143-completion-receipt.v1.json"
assert ACTIVATION["unknown_impact_refs"] == []
assert ACTIVATION["blocking_conflict_refs"] == []
activation_rows = [row for row in LEDGER["requirements"] if 413 <= row["source_line"] <= 460]
assert len(activation_rows) == 23
assert all(row["runtime_status"] == "verified_complete" for row in activation_rows)
assert all(row["runtime_status"] == "verified_complete" for row in LEDGER["requirements"])
assert all(row.get("runtime_evidence_refs") for row in LEDGER["requirements"])
for key in [
    "normative_source_coverage_ref", "feature_ledger_ref", "delivery_dag_ref",
    "ownership_matrix_ref", "client_parity_matrix_ref", "vertical_pack_matrix_ref",
    "migration_matrix_ref", "proof_matrix_ref",
]:
    assert (ROOT / ACTIVATION[key]).is_file(), f"missing activation prerequisite: {key}"

assert REGISTRY["schema"] == "focusa.semantic_artifact_registry.v1"
assert REGISTRY["activation"] == "dormant_until_spec144_release_gates_pass"
assert REGISTRY["canonicalization_algorithm"] == "focusa-rdf-deterministic-v1"
assert len(REGISTRY["standards_profile"]) == 8
assert len(REGISTRY["validation_profile_families"]) == 10
assert len(set(REGISTRY["validation_profile_families"])) == 10
assert len(REGISTRY["named_graph_kinds"]) == 10
assert len(set(REGISTRY["named_graph_kinds"])) == 10

for artifact in REGISTRY["artifacts"]:
    path = ROOT / artifact["path"]
    assert path.is_file(), f"missing semantic artifact: {path}"
    assert digest(path) == artifact["sha256"], f"digest drift: {path}"

spec = ROOT / REGISTRY["source_spec"]
assert digest(spec) == REGISTRY["source_spec_sha256"]

ontology = (CONTRACTS / "focusa-core-ontology-v1.ttl").read_text()
for token in [
    "owl:Ontology",
    "owl:versionIRI",
    "owl:imports",
    "focusa:SemanticArtifact",
    "focusa:SemanticWorkContract",
    "focusa:SemanticValidationReceipt",
    "focusa:ActionPlan",
    "focusa:VerificationPlan",
]:
    assert token in ontology, f"ontology missing {token}"

for epistemic_class in [
    "operator_asserted", "user_asserted", "deterministic_asserted",
    "tool_observed", "runtime_observed", "reducer_asserted", "model_proposed",
    "model_inferred", "reasoner_inferred", "verification_confirmed",
    "legacy_assumed", "contradicted", "invalid", "quarantined",
    "unsupported_opaque",
]:
    assert f"focusa:{epistemic_class}" in ontology

shapes = (CONTRACTS / "focusa-core-shapes-v1.ttl").read_text()
for family in [
    "IntakeProfile", "PromotionProfile", "ActionPreflightProfile",
    "VerificationPlanProfile", "FindingVerdictProfile", "SettlementProfile",
    "DomainPackProfile", "MigrationReplayProfile", "VerticalBundleProfile",
    "OmissionFirewallProfile",
]:
    assert f"focusa:{family} a sh:NodeShape" in shapes
assert shapes.count("a sh:NodeShape") == 10
assert "sh:closed true" in shapes
assert "sh:severity sh:Violation" in shapes

context = json.loads((CONTRACTS / "focusa-core-context-v1.jsonld").read_text())
assert context["@context"]["@version"] == 1.1
assert context["@context"]["focusa"] == REGISTRY["namespace"]

for token in [
    "pub struct SemanticArtifact",
    "pub struct SemanticValidationReceipt",
    "pub struct SemanticWorkContract",
    "pub fn canonicalize_semantic_artifact",
    "pub fn validate_semantic_artifact",
    "pub fn validate_semantic_work_contract",
]:
    assert token in INTEGRITY
for token in [
    "pub enum NamedGraphKind",
    "pub enum EpistemicClass",
    "pub struct SemanticRegistry",
    "pub struct SemanticBuildManifest",
    "pub fn build_semantic_artifact",
]:
    assert token in SEMANTIC_REGISTRY

print("Spec144 semantic artifacts gate: PASS")
