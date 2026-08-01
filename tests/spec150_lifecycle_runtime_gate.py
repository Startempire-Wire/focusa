#!/usr/bin/env python3
"""Fast Spec150 lifecycle implementation gate; --release requires activated ledger proof."""
from __future__ import annotations
import argparse
from pathlib import Path
import yaml

ROOT = Path(__file__).resolve().parents[1]

def body(path: str) -> str:
    p = ROOT / path
    assert p.is_file(), f"missing {path}"
    return p.read_text()

def require(path: str, tokens: list[str]) -> None:
    source = body(path)
    missing = [token for token in tokens if token not in source]
    assert not missing, f"{path}: missing {missing}"

def preflight() -> None:
    require("crates/focusa-core/src/install_lifecycle.rs", ["LifecycleState", "LifecycleReceipt", "mod adapters", "mod orchestrator"])
    require("crates/focusa-core/src/install_lifecycle/transactions.rs", ["HostInstallTransaction", "ProjectOnboardingTransaction", "LifecycleMaintenanceTransaction"])
    require("crates/focusa-core/src/install_lifecycle/preflight.rs", ["Preflight", "validate"])
    require("crates/focusa-core/src/install_lifecycle/preservation.rs", ["PreservationDeclaration", "PurgeConfirmed"])
    require("crates/focusa-core/src/install_lifecycle/adapters.rs", ["LifecycleAdapterReceipt", "ProviderAuthHandoff", "CredentialIngestionForbidden", "Compacting", "Saturated"])
    require("crates/focusa-core/src/install_lifecycle/orchestrator.rs", ["LifecycleJournalEntry", "verify_journal", "resume_state", "CompleteVersionSet", "FirstWorkpointNotAccepted"])
    lifecycle_tests = list((ROOT / "crates/focusa-core/src/install_lifecycle").glob("*_tests.rs"))
    assert sum(p.read_text().count("#[test]") for p in lifecycle_tests) >= 16
    assert all(len(p.read_text().splitlines()) < 500 for p in (ROOT / "crates/focusa-core/src/install_lifecycle").glob("*.rs"))

def release_activation() -> None:
    ledger = yaml.safe_load(body("docs/contracts/spec150-complete-feature-ledger.v1.yaml"))
    assert ledger["runtime_status"] == "implementation_verified"
    rows = ledger["requirements"]
    assert len(rows) == ledger["source_atom_count"]
    for row in rows:
        assert row["runtime_status"] == "implementation_verified", row["requirement_id"]
        assert row["implementation_refs"]
        assert row["test_refs"]
        assert row["evidence_refs"]
        assert row["receipt_refs"]

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--release", action="store_true")
    args = parser.parse_args()
    preflight()
    if args.release:
        release_activation()
    print(f"Spec150 lifecycle runtime gate: PASS ({'release' if args.release else 'preflight'})")
