#!/usr/bin/env python3
"""Fail-closed, bounded Spec 135 quality gate over machine-readable contracts."""

import json
import sys
from pathlib import Path

R = Path(__file__).resolve().parents[1]
CONTRACTS = [
    "spec135-q1-security-scope-policy.v1.yaml",
    "spec135-q2-security-privacy-gates.v1.yaml",
    "spec135-q3-performance-budgets.v1.yaml",
    "spec135-q4-recovery-matrix.v1.yaml",
    "spec135-q5-supply-chain-governance.v1.yaml",
    "spec135-client-parity-matrix.v1.yaml",
    "spec135-mission-canvas-portability.v1.yaml",
]

checks = []
for name in CONTRACTS:
    path = R / "docs/contracts" / name
    try:
        payload = json.loads(path.read_text())
        acceptance = payload.get("acceptance", {})
        false_acceptance = (
            sorted(key for key, value in acceptance.items() if value is not True)
            if isinstance(acceptance, dict)
            else []
        )
        ok = bool(payload) and not false_acceptance
        detail = "accepted" if ok else "false acceptance: " + ",".join(false_acceptance)
    except Exception as error:
        ok = False
        detail = type(error).__name__
    checks.append({"contract": name, "ok": ok, "detail": detail[:160]})

required_files = [
    "deny.toml",
    "about.toml",
    "THIRD_PARTY_NOTICES.md",
    "docs/contracts/spec135-complete-feature-ledger.v1.yaml",
    "docs/contracts/spec135-proof-matrix.v1.yaml",
    "packages/generated/spec135/typescript/schema.d.ts",
]
for name in required_files:
    checks.append({"contract": name, "ok": (R / name).exists(), "detail": "required public surface"})

failed = [row for row in checks if not row["ok"]]
result = {
    "schema": "focusa.spec135.quality_gate_result.v1",
    "status": "blocked" if failed else "completed",
    "check_count": len(checks),
    "failed_count": len(failed),
    "checks": checks[:32],
    "evidence_ref": "evidence:spec135-quality-gate:static",
    "recovery": [f"repair:{row['contract']}" for row in failed[:16]],
}
print(json.dumps(result, separators=(",", ":"), sort_keys=True))
sys.exit(1 if failed else 0)
