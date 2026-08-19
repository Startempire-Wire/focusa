#!/usr/bin/env python3
import json, pathlib
p=pathlib.Path("docs/contracts/spec152-protected-boundary-ledger.v1.json")
j=json.loads(p.read_text())
assert j["schema"]=="focusa.spec152.protected_boundary_ledger.v1"
assert "premium" in j["selected_family"]
# public absence: ensure public api does not contain private owner logic
pub=pathlib.Path("docs/contracts/spec152e-activation-public-openapi.v1.json").read_text()
assert "limit_reservation" not in pub.lower()
print("Spec152 protected boundary: PASS")
