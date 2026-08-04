#!/usr/bin/env python3
"""Runtime acceptance for the GH#106.3 technical closure reducer."""

from __future__ import annotations

import json
import subprocess
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "reduce-locked-release-technical-closure.py"
LEDGER = (
    ROOT
    / "release-proof"
    / "audit"
    / "next-locked-release-governance-reconciliation.json"
)
GATE = (
    ROOT / "release-proof" / "audit" / "next-locked-release-technical-closure-gate.json"
)


def run(*args: str, expected: int = 0) -> subprocess.CompletedProcess[str]:
    result = subprocess.run(
        ["python3", str(SCRIPT), *args],
        cwd=ROOT,
        text=True,
        capture_output=True,
    )
    assert result.returncode == expected, result.stderr or result.stdout
    return result


run("--check")
gate = json.loads(GATE.read_text())
assert gate["schema"] == "focusa.locked_release_technical_closure_gate.v1"
assert gate["status"] == "verified"
assert gate["mapping_count"] == 289
assert gate["invalid_closed_count"] == 0
assert gate["invalid_closed_ids"] == []
assert gate["technically_pending_count"] > 0

allowed = run("--bead-id", "focusa-vbcqu.14.2", "--request-state", "closed")
allowed_replay = run("--bead-id", "focusa-vbcqu.14.2", "--request-state", "closed")
assert allowed.stdout == allowed_replay.stdout
allowed_receipt = json.loads(allowed.stdout)
assert allowed_receipt["decision"] == "allow"
assert allowed_receipt["reason"] == "technical_acceptance_satisfied"
assert allowed_receipt["request_digest"].startswith("sha256:")
assert allowed_receipt["receipt_digest"].startswith("sha256:")

blocked = run("--bead-id", "focusa-vbcqu.10", "--request-state", "closed", expected=2)
blocked_replay = run(
    "--bead-id", "focusa-vbcqu.10", "--request-state", "closed", expected=2
)
assert blocked.stdout == blocked_replay.stdout
blocked_receipt = json.loads(blocked.stdout)
assert blocked_receipt["decision"] == "block"
assert blocked_receipt["reason"] == "technical_acceptance_missing"

reopen = json.loads(
    run("--bead-id", "focusa-vbcqu.10", "--request-state", "open").stdout
)
assert reopen["decision"] == "allow"
assert reopen["reason"] == "reopen_is_replay_safe"

unknown = json.loads(
    run(
        "--bead-id", "focusa-not-admitted", "--request-state", "closed", expected=2
    ).stdout
)
assert unknown["decision"] == "block"
assert unknown["reason"] == "unknown_or_unadmitted_bead"

# A replayed administrative close without technical proof makes the aggregate
# gate fail closed, even if the provider claims the record is closed.
tampered = json.loads(LEDGER.read_text())
widget = next(
    row for row in tampered["mappings"] if row["bead_id"] == "focusa-vbcqu.10"
)
widget["provider_state"] = "closed"
widget["closure_receipt"] = {
    "closed_at": "administrative-replay",
    "close_reason": "administrative close only",
    "exact_duplicate_of": None,
}
with tempfile.NamedTemporaryFile("w", suffix=".json", delete=False) as handle:
    json.dump(tampered, handle)
    tampered_path = Path(handle.name)
try:
    failed_gate = json.loads(run("--ledger", str(tampered_path), expected=2).stdout)
finally:
    tampered_path.unlink(missing_ok=True)
assert failed_gate["status"] == "blocked"
assert failed_gate["invalid_closed_ids"] == ["focusa-vbcqu.10"]

print("GH#106.3 technical closure reducer: PASS")
