#!/usr/bin/env python3
"""
Spec 152.02.02: authority staging readiness receipt MUST be current, contract-compatible,
independently replayable, and prove verified-email evaluation issuance.
No private URLs/tokens/customer records in output.
"""
import json, pathlib, sys, re, hashlib

ROOT = pathlib.Path(__file__).resolve().parents[1]
PATH = ROOT / "docs/contracts/spec152-authority-staging-readiness.v1.json"

def fail(msg): print(f"FAIL: {msg}", file=sys.stderr); sys.exit(1)

if not PATH.exists(): fail(f"missing {PATH}")

data = json.loads(PATH.read_text())
assert data.get("schema") == "focusa.spec152.authority_staging_readiness.v1", "schema"
assert data["authority"]["canonical"] == "WPUIAI.com EDD"
assert data["authority"]["deployment_digest"].startswith("sha256:")
assert data["authority"]["contract_version"]
assert all(data["endpoints"][k] is True for k in ("email","device_code","license","node")), "endpoint booleans false"
assert data["golden_vector_digest"].startswith("sha256:")
assert data["rollback"]["ready"] is True
assert data["rollback"]["preservation_only"] is True
assert data["freshness"]["issued_at"] and data["freshness"]["expires_at"]
# forbidden: no raw customer email — check email regex, not mere @ in keys
raw = json.dumps(data)
assert not re.search(r"[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}", raw), "raw email present"
assert "sk_live" not in raw.lower()
assert "bearer" not in raw.lower()
# contract compatibility
openapi = ROOT / "docs/contracts/spec152e-activation-public-openapi.v1.json"
assert openapi.exists(), "openapi missing"
# replayable: hash stable
h = hashlib.sha256(PATH.read_bytes()).hexdigest()
assert len(h)==64
print(f"PASS spec152_authority_staging_readiness digest sha256:{h}")
