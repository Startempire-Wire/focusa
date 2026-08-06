#!/usr/bin/env python3
"""Fail closed on ambiguous Spec 172 no-sales and clean-cutover claims."""
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PATH = ROOT / "docs/contracts/spec172-no-sales-inventory.v1.json"
data = json.loads(PATH.read_text())

assert data["schema"] == "focusa.spec172.no_sales_inventory.v1"
assert data["inventory_id"] == "focusa-vbcqu.20.15.6"
assert data["evidence_posture"]["raw_records_stored"] is False
assert data["evidence_posture"]["customer_identifiers_stored"] is False
assert data["evidence_posture"]["credentials_stored"] is False

required = {
    "edd", "gravity_forms", "stripe",
    "install_site_custom_license_and_webhooks",
    "manual_keys_and_access_grants", "synthetic_test_records",
}
assert set(data["required_rails"]) == required
rails = data["rails"]
assert {row["rail"] for row in rails} == required
assert len(rails) == len(required)

allowed_classes = {"ambiguous", "synthetic_test_only", "no_offer_match_in_bounded_snapshot"}
for row in rails:
    assert row["classification"] in allowed_classes
    assert row["redacted_aggregate_counts"]
    assert all(isinstance(v, int) and v >= 0 for v in row["redacted_aggregate_counts"].values())
    assert re.fullmatch(r"[0-9a-f]{64}", row["semantic_digest_sha256"])
    assert row["next_read_only_evidence_needed"].strip()
    forbidden_fields = {"customers", "emails", "license_keys", "raw_records", "credentials", "tokens"}
    assert not (forbidden_fields & row.keys())

ambiguous = [row for row in rails if row["classification"] == "ambiguous"]
assert {row["rail"] for row in ambiguous} == {
    "stripe", "install_site_custom_license_and_webhooks", "manual_keys_and_access_grants"
}
assert all("read" in row["next_read_only_evidence_needed"].lower() for row in ambiguous)

# Core fail-closed invariant: either claim is false while any rail is ambiguous.
decision = data["decision"]
if ambiguous:
    assert decision["zero_sales_proven"] is False, "false zero-sales claim while ambiguity remains"
    assert decision["clean_cutover_allowed"] is False, "false clean-cutover claim while ambiguity remains"
    assert decision["status"] == "blocked_ambiguous"

registry = json.loads((ROOT / data["reconciliation"]["product_registry"]).read_text())
assert registry["counts"]["checkout_enabled"] == 0
assert all(not offer["checkout_enabled"] for offer in registry["protected_offers"])
assert data["reconciliation"]["legacy_records_preserved"] is True

print("Spec 172 no-sales inventory blocker: PASS")
