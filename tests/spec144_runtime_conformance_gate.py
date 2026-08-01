#!/usr/bin/env python3
"""Fast Spec144 source/evidence gate; --release additionally requires activated ledgers."""
from __future__ import annotations
import argparse, json
from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[1]

def text(path: str) -> str:
    p = ROOT / path
    assert p.is_file(), f"missing {path}"
    return p.read_text()

def require(path: str, tokens: list[str]) -> None:
    body = text(path)
    missing = [token for token in tokens if token not in body]
    assert not missing, f"{path}: missing {missing}"

def preflight() -> None:
    require("crates/focusa-core/src/semantic_integrity.rs", ["canonicalize_semantic_artifact", "validate_semantic_artifact"])
    require("crates/focusa-core/src/semantic_registry.rs", ["SemanticRegistry", "reproducible"])
    require("crates/focusa-core/src/semantic_verification.rs", ["compile_obligations", "route_verification", "settle_verification"])
    require("crates/focusa-core/src/semantic_settlement.rs", ["SettledFull", "SettledPartial", "closure_allowed"])
    require("crates/focusa-core/src/semantic_vertical.rs", ["VerifierCohort", "validate_vertical_activation_with_trust"])
    require("crates/focusa-core/src/semantic_security.rs", ["verify_ed25519_digest", "ShaclSparqlProhibited", "CanonicalSameAsProhibited"])
    require("crates/focusa-core/src/semantic_performance.rs", ["AffectedNeighborhood", "DeferredPreservingAcceptedWork"])
    require("crates/focusa-core/src/semantic_reflex.rs", ["SHARED_SEMANTIC_REFLEXES", "SchemaOnly", "execute_semantic_reflex"])
    require("crates/focusa-core/src/semantic_pair.rs", ["SemanticPair", "ArtifactHandleRef", "SemanticReceipt"])
    require("crates/focusa-core/src/semantic_replay.rs", ["SemanticPairEvent", "previous_hash", "pub fn replay"])
    require("crates/focusa-core/src/semantic_migration.rs", ["compatibility_read", "dry_run", "rollback", "FutureVersion"])
    require("crates/focusa-api/src/routes/semantic_integrity.rs", ["/v1/semantic-integrity/status", "SchemaOnly", "confirmation_required"])
    require("crates/focusa-cli/src/commands/semantic_integrity.rs", ["Status", "Registry", "Artifacts", "Inspect", "Invoke"])
    require("tests/spec144_evaluation_gate.py", ["25", "six_cohorts", "promotion"])
    fixture = json.loads(text("tests/fixtures/spec144/evaluation.json"))
    scenarios = fixture.get("golden_scenarios", fixture.get("scenarios", []))
    assert len(scenarios) == 25, f"expected 25 golden scenarios, got {len(scenarios)}"
    test_files = list((ROOT / "crates/focusa-core/src").glob("semantic_*tests.rs"))
    assert sum(p.read_text().count("#[test]") for p in test_files) >= 30

def release_activation() -> None:
    ledger = yaml.safe_load(text("docs/contracts/spec144-complete-feature-ledger.v1.yaml"))
    assert ledger["runtime_status"] == "verified_complete"
    assert ledger["runtime_claim"] == "activated"
    rows = ledger["requirements"]
    assert rows and all(row.get("runtime_status") == "verified_complete" for row in rows)
    assert all(row.get("runtime_evidence_refs") for row in rows)
    for name in [
        "spec144-proof-matrix.v1.yaml",
        "spec144-client-parity-matrix.v1.yaml",
        "spec144-migration-matrix.v1.yaml",
        "spec144-forbidden-placeholder-audit.v1.yaml",
    ]:
        body = yaml.safe_load(text(f"docs/contracts/{name}"))
        assert body.get("runtime_status") in {"verified_complete", "activated"}, name
        assert body.get("runtime_claim") in {"activated", "verified"}, name

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--release", action="store_true")
    args = parser.parse_args()
    preflight()
    if args.release:
        release_activation()
    print(f"Spec144 runtime conformance gate: PASS ({'release' if args.release else 'preflight'})")
