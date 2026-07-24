#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
contract = json.loads(
    (R / "docs/contracts/spec135-q5-supply-chain-governance.v1.yaml").read_text()
)
script = (R / "scripts/generate-supply-chain-artifacts.sh").read_text()
deny = (R / "deny.toml").read_text()
about = (R / "about.toml").read_text()
for tool in ("cargo deny --all-features check", "cargo about generate", "syft"):
    assert tool in script
for output in contract["outputs"]:
    assert output in script
for forbidden in ("AGPL-3.0", "GPL-3.0", "LGPL"):
    assert forbidden in deny
for accepted in ("Apache-2.0", "MIT", "BSD-3-Clause"):
    assert accepted in about
assert "sha256" in script
for ref in contract["evidence_refs"]:
    assert (R / ref).exists(), ref
print("Spec 135 Q5 supply-chain governance strict lint: PASS")
