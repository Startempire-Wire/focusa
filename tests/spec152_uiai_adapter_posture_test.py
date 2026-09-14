#!/usr/bin/env python3
from pathlib import Path

models = Path("crates/focusa-core/src/install_lifecycle/models.rs").read_text()
receipts = Path("crates/focusa-core/src/install_lifecycle/receipts.rs").read_text()
for marker in [
    "from_independent_uiai_authority",
    'focusa_parent.product == "focusa"',
    'uiai_grant.product == "uiai-engine"',
    "request.requested_features",
    "receipt.request_id == request.request_id",
    "receipt.expires_at > now",
    "parent_lease_digest",
    "child_token_id",
    "entitlement_snapshot_ready",
]:
    assert marker in models, marker
assert "posture.parent_lease_digest" in receipts
assert "posture.lease_id != self.lease_id" not in receipts
assert "posture.entitlement_digest != digest" not in receipts
assert "health" not in models[models.index("from_independent_uiai_authority"):]
print("Spec152 independent UIAI adapter posture gate: PASS")
