#!/usr/bin/env python3
from pathlib import Path
import re

from structured_contract_loader import load_contract_mapping

ROOT = Path(__file__).resolve().parents[1]
required = [
    "docs/contracts/spec137a-normative-source-coverage.v1.yaml",
    "docs/contracts/spec137a-applicability-matrix.v1.yaml",
    "docs/contracts/spec137a-conformance-class-matrix.v1.yaml",
    "docs/contracts/spec137a-forbidden-placeholder-audit.v1.yaml",
    "docs/contracts/spec137a-parent-override-map.v1.yaml",
    "docs/contracts/spec138a-normative-source-coverage.v1.yaml",
    "docs/contracts/spec138-complete-feature-ledger.v1.yaml",
    "docs/contracts/spec138-delivery-dag.v1.yaml",
    "docs/contracts/spec138-profile-activation-and-conformance-matrix.v1.yaml",
    "docs/contracts/spec138-primitive-ownership-matrix.v1.yaml",
    "docs/contracts/spec138-operation-client-parity-matrix.v1.yaml",
    "docs/contracts/spec138-scorer-and-calibration-matrix.v1.yaml",
    "docs/contracts/spec138-source-independence-and-triangulation-matrix.v1.yaml",
    "docs/contracts/spec138-outcome-resolution-authority-matrix.v1.yaml",
    "docs/contracts/spec138-learning-promotion-and-rollback-matrix.v1.yaml",
    "docs/contracts/spec138-transfer-self-model-and-consolidation-matrix.v1.yaml",
    "docs/contracts/spec138-migration-matrix.v1.yaml",
    "docs/contracts/spec138-security-privacy-retention-matrix.v1.yaml",
    "docs/contracts/spec138-proof-matrix.v1.yaml",
    "docs/contracts/spec138-forbidden-placeholder-audit.v1.yaml",
    "docs/contracts/spec138a-parent-override-map.v1.yaml",
    "docs/contracts/spec144-normative-source-coverage.v1.yaml",
    "docs/contracts/spec144-complete-feature-ledger.v1.yaml",
    "docs/contracts/spec144-delivery-dag.v1.yaml",
    "docs/contracts/spec144-primitive-ownership-matrix.v1.yaml",
    "docs/contracts/spec144-obligation-verifier-matrix.v1.yaml",
    "docs/contracts/spec144-cross-spec-amendment-matrix.v1.yaml",
    "docs/contracts/spec144-client-parity-matrix.v1.yaml",
    "docs/contracts/spec144-vertical-pack-matrix.v1.yaml",
    "docs/contracts/spec144-migration-matrix.v1.yaml",
    "docs/contracts/spec144-proof-matrix.v1.yaml",
    "docs/contracts/spec144-forbidden-placeholder-audit.v1.yaml",
    "docs/contracts/spec144-core-verification-pack.v1.yaml",
    "docs/contracts/spec144-obligation-compilation-and-coverage.v1.yaml",
    "docs/contracts/spec144-execution-placement-and-common-mode.v1.yaml",
    "docs/contracts/spec144-verification-dispute-arbitration.v1.yaml",
    "docs/contracts/spec144-settlement-revalidation.v1.yaml",
]
for rel in required:
    path = ROOT / rel
    assert path.is_file(), rel
    text = path.read_text()
    assert len(text) > 200, f"empty/shell artifact: {rel}"
    data = load_contract_mapping(path)
    claim = data.get("runtime_claim")
    status = data.get("runtime_status")
    assert (claim, status) in {
        ("none", "implementation_open"),
        ("none", "not_activated"),
        ("activated", "verified_complete"),
        ("full_spec138_conformance", "verified_complete"),
    }, rel
    if claim in {"activated", "full_spec138_conformance"}:
        assert data.get("activation_receipt_ref") == "release-proof/audit/spec144-spec150-double-e2e-receipt.json", rel

s137 = (ROOT / "docs/137-focusa-temporal-authority-deadlines-urgency-grounded-forecasting-spec.md").read_text()
s138 = (ROOT / "docs/138-focusa-prediction-outcome-calibration-metacognitive-learning-transfer-and-epistemic-governance-spec.md").read_text()
s144 = (ROOT / "docs/144-focusa-semantic-integrity-rdf-owl-shacl-build-verify-routing-and-vertical-intelligence-spec.md").read_text()
assert "Mandatory companion" in s137 and "Spec 137A" in s137
assert "Mandatory companion" in s138 and "Spec 138A" in s138
for token in ("Spec 137 + Spec 137A", "Spec 138 + Spec 138A", "Spec 139", "focusa.verification.core@1", "ObligationCompilationReceipt", "VerificationExecutionBinding", "CognitiveExecutionIdentity", "SettlementRevalidationTrigger"):
    assert token in s144, token

ledger137 = (ROOT / "docs/contracts/spec137-complete-feature-ledger.v1.yaml").read_text()
assert "combined_normative_source_v2" in ledger137 and "spec137a_requirement_rows" in ledger137

alignment = (ROOT / "docs/evidence/141-focusa-latest-spec-public-doc-alignment.md").read_text()
assert alignment.count("combined full conformance verified") >= 2
assert "runtime implementation verified by `release-proof/audit/spec144-spec150-double-e2e-receipt.json`" in alignment

ci = (ROOT / "scripts/ci/run-spec-gates.sh").read_text()
assert "spec137a_138a_144_documentation_closure_gate.py" in ci
print("Specs 137A/138A/144 documentation architecture closure gate: PASS")


# literal source atom coverage and current-hash validation
for rel in (
    "docs/contracts/spec137a-normative-source-coverage.v1.yaml",
    "docs/contracts/spec138a-normative-source-coverage.v1.yaml",
    "docs/contracts/spec144-normative-source-coverage.v1.yaml",
):
    data = load_contract_mapping(ROOT / rel)
    assert data["source_atom_count"] == len(data["source_atoms"]), rel
    assert not data["unmapped_source_atom_refs"], rel
    for src in data["sources"]:
        text = (ROOT / src["path"]).read_text()
        import hashlib
        assert hashlib.sha256(text.encode()).hexdigest() == src["sha256"], src["path"]

for rel in (
    "docs/90-ontology-backed-tool-contracts-parity-spec.md",
    "docs/95-focusa-ontology-low-latency-intelligence-enhancer-sow.md",
    "docs/104-typed-scoped-runtime-and-singleton-elimination-spec.md",
    "docs/111-agent-context-bootstrap-and-delivery-spec.md",
):
    assert "SPEC137A_138A_144_ARCHITECTURE_CLOSURE" in (ROOT / rel).read_text(), rel

spec139_lines = (ROOT / "docs/139-distributed-presence-environment-awareness-execution-placement-and-multi-daemon-coordination-spec.md").read_text().splitlines()
spec140_lines = (ROOT / "docs/140-project-agent-runtime-constitution-instruction-authority-system-prompt-and-cross-harness-compiler-spec.md").read_text().splitlines()
spec139_depends = next(line for line in spec139_lines if line.startswith("**Depends on:**"))
spec140_depends = next(line for line in spec140_lines if line.startswith("**Depends on:**"))
assert "137A" in spec139_depends and "138A" in spec139_depends
assert "137A" in spec140_depends and "138A" in spec140_depends
print("literal source atom coverage and remaining owner integration: PASS")
