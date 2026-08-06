#!/usr/bin/env python3
"""GH#106.1 immutable admission and exclusion inventory gate."""

import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
result = subprocess.run(
    ["python3", "scripts/generate-locked-release-governance-inventory.py", "--check"],
    cwd=ROOT,
    capture_output=True,
    text=True,
)
assert result.returncode == 0, result.stdout + result.stderr
inventory = json.loads(
    (ROOT / "release-proof/audit/next-locked-release-governance-inventory.json").read_text()
)
assert inventory["workset_id"] == "workset:focusa-next-locked-release:r7"
assert inventory["immutable_member_count"] == 275
assert inventory["scope_additions_closed"] is True
assert inventory["further_additions_allowed"] is False
assert {row["bead_id"] for row in inventory["authorized_release_repair_overlay"]} == {
    "focusa-vbcqu.14",
    "focusa-vbcqu.19",
    "focusa-vbcqu.20",
    *{f"focusa-vbcqu.10.{phase}" for phase in range(7, 13)},
}
assert {row["github_issue"] for row in inventory["excluded_reconstructed_epics"]} == {
    45, 52, 89, 101, 107, 112, 114
}
assert all(not row["publication_blocking"] for row in inventory["excluded_reconstructed_epics"])
assert inventory["terminal_release_path"][-1] == "new_monotonic_stable_release"
assert inventory["terminal_release_path"][-3:-1] == [
    "bead:focusa-vbcqu.20",
    "bead:focusa-vbcqu.19",
]
assert "spec152e_correction" in inventory["authority_file_digests"]
print("GH#106.1 locked-release governance inventory: PASS")
