#!/usr/bin/env python3
import json, pathlib
ROOT=pathlib.Path(__file__).resolve().parents[1]
p=ROOT/"docs/contracts/spec152-protected-boundary-ledger.v1.json"
j=json.loads(p.read_text())
assert j["schema"]=="focusa.spec152.protected_boundary_ledger.v1"
assert "premium" in j["selected_family"]
pub=(ROOT/"docs/contracts/spec152e-activation-public-openapi.v1.json").read_text()
assert "limit_reservation" not in pub.lower()
print("Spec152 protected boundary: PASS")
