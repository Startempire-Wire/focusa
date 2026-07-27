#!/usr/bin/env python3
import json
from pathlib import Path

R = Path(__file__).resolve().parents[1]
lock = json.loads((R / "docs/contracts/spec135-framework-lock.v1.yaml").read_text())
audit = json.loads((R / "docs/contracts/spec135-framework-runtime-audit.v1.yaml").read_text())
assert audit["schema"] == "focusa.spec135.framework_runtime_audit.v1"
assert audit["forbidden_runtime_dependencies"] == []
locked = {row["area"] for row in lock["locks"]}
audited = {row["area"] for row in audit["areas"]}
assert audited == locked
for row in audit["areas"]:
    assert row["status"] in {"adopted", "partial"}
    assert row["owner_beads"]
    assert all(ref.startswith("focusa-mc-full-") for ref in row["owner_beads"])
    assert all((R / ref).exists() for ref in row["evidence"])
manifest_text = "\n".join(
    path.read_text(errors="replace")
    for pattern in ("Cargo.toml", "package.json")
    for path in R.rglob(pattern)
    if ".git" not in path.parts and "node_modules" not in path.parts
).lower()
for forbidden in lock["forbidden"]:
    assert forbidden.lower() not in manifest_text, forbidden
print("Spec 135 framework runtime audit: PASS (all areas owned, no forbidden runtime dependency)")
