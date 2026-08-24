#!/usr/bin/env python3
"""Require one definition row for every substantive release-range commit."""
from pathlib import Path
import re, subprocess

ROOT = Path(__file__).resolve().parents[1]
LEDGER = ROOT / "docs/179-focusa-v0-9-184-dev-release-definition-ledger.md"
text = LEDGER.read_text()
assert len(text.splitlines()) < 500
m = re.search(r"\*\*Range:\*\* `v0\.9\.183-dev\.\.([0-9a-f]{9})`", text)
assert m, "ledger range endpoint missing"
end = m.group(1)
commits = subprocess.check_output(
    ["git", "rev-list", "--reverse", f"v0.9.183-dev..{end}"], cwd=ROOT, text=True
).splitlines()
rows = set(re.findall(r"^\| `([0-9a-f]{9})` \|", text, flags=re.MULTILINE))
missing = [commit[:9] for commit in commits if commit[:9] not in rows]
assert not missing, f"release commits lack definitions: {missing}"
assert len(rows) == len(commits), f"row count {len(rows)} != commit count {len(commits)}"
for heading in [
    "## Commit coverage", "## Feature and contract glossary", "## Current truth at this ledger endpoint"
]:
    assert heading in text
for required in [
    "North Star", "output_tail", "AppVeyor emergency provider", "Fail-closed release lane",
    "Emergency signer", "Release Ed25519 key", "Tauri updater key", "Chrome observation",
    "Orientation packet", "Durable approval issuance", "Exact target", "Cursor replay",
    "Release held",
]:
    assert required in text, f"undefined required release term: {required}"
assert "not created" in text and "not yet shipped" in text
print(f"PASS: release definition ledger covers {len(commits)} commits and required new terms")
