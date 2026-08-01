#!/usr/bin/env python3
"""Legacy reconciliation, generated matrices, and truthful release-state gate."""
from __future__ import annotations

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
subprocess.run(["python3", "scripts/generate-spec135-reconciled-ledger.py", "--check"], cwd=ROOT, check=True)
subprocess.run(["python3", "scripts/generate-spec135-master-acceptance.py"], cwd=ROOT, check=True)
subprocess.run(["python3", "scripts/generate-spec135-mission-canvas-derived-contracts.py", "--check"], cwd=ROOT, check=True)

ledger = json.loads((ROOT / "docs/contracts/spec135-complete-feature-ledger.v1.yaml").read_text())
amendment = json.loads((ROOT / "docs/contracts/spec135-rich-host-delivery-contract.v1.json").read_text())
acceptance = json.loads((ROOT / "docs/contracts/spec135-master-final-acceptance.v1.json").read_text())
reconciliation = json.loads((ROOT / "docs/contracts/spec135-rich-host-reconciliation.v1.json").read_text())
release = json.loads((ROOT / "tests/fixtures/spec135-pi-native-release-matrix.json").read_text())
parity = json.loads((ROOT / "docs/contracts/spec135/mission-canvas-v1/client-parity-matrix.json").read_text())
proof = json.loads((ROOT / "docs/contracts/spec135/mission-canvas-v1/implementation-proof-matrix.json").read_text())

assert ledger["reconciliation"]["legacy_requirement_count"] == 73
assert ledger["requirement_count"] == 73
assert amendment["requirement_count"] == 3
assert {"SPEC135-F13", "SPEC135-F14", "SPEC135-AC1"} == {row["requirement_id"] for row in amendment["requirements"]}
assert ledger["reconciliation"]["verified_count"] == 72
assert ledger["reconciliation"]["blocked_count"] == 1
assert acceptance["passed_count"] == 14
assert acceptance["blocked_count"] == 0
assert acceptance["merge_ready"] is True
assert acceptance["status"] == "verified"
assert reconciliation["series"] == ["135", "135A", "135B", "135C", "135D", "135E", "135F", "135G", "135H", "135I", "135J", "135K"]
assert reconciliation["status"] == "technically_reconciled_release_blocked"
assert parity["operation_count"] == 25 and parity["client_count"] == 6
assert len(proof["layers"]) == 5
assert {item["os"] for item in release["platforms"]} == {"macOS", "Windows", "Linux"}
assert release["platforms"][0]["status"] == "installed_smoke_verified_on_current_mac"
assert all(item["status"] == "portable_package_and_no_native_dependency_verified" for item in release["platforms"][1:])
assert release["remote_status"] == "pending_protected_checks"
assert "explicit operator authorization" in release["release_authority"]

print("Spec 135 reconciliation and release truth: PASS (72/73 legacy + 3/3 amendment; release blocked)")
