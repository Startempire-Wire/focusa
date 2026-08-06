#!/usr/bin/env python3
"""Require a preserving migration when Spec 172 cannot prove zero sales."""
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
    assert "typed_record_counts" not in row, "aggregates do not support genuine/synthetic typing"
    assert row["redacted_aggregate_counts"]
    assert all(isinstance(v, int) and v >= 0 for v in row["redacted_aggregate_counts"].values())
    assert re.fullmatch(r"[0-9a-f]{64}", row["semantic_digest_sha256"])
    assert row["next_read_only_evidence_needed"].strip()
    forbidden_fields = {"customers", "emails", "license_keys", "raw_records", "credentials", "tokens"}
    assert not (forbidden_fields & row.keys())

ambiguous = [row for row in rails if row["classification"] == "ambiguous"]
assert {row["rail"] for row in ambiguous} == {
    "edd", "stripe", "install_site_custom_license_and_webhooks", "manual_keys_and_access_grants"
}
assert all("read" in row["next_read_only_evidence_needed"].lower() for row in ambiguous)

by_rail = {row["rail"]: row for row in rails}
edd = by_rail["edd"]
assert edd["semantic_digest_sha256"] == "fa49fa00ff2a88ea110228c174671021da1ec6a94852e199ee0a164d49b78202"
assert edd["redacted_aggregate_counts"] == {
    "complete_test_manual_orders": 4,
    "complete_test_manual_order_total_minor_units": 8700,
    "transactions": 0,
    "licenses": 11,
    "download_453_licenses": 4,
    "activations": 0,
    "subscriptions": 0,
    "refunds": 0,
}
install = by_rail["install_site_custom_license_and_webhooks"]
assert install["semantic_digest_sha256"] == "0eaa45b557f8cb00334766fd1b2287f64928918e1e2fb46d058d2e552dc8a9fa"
assert install["redacted_aggregate_counts"] == {
    "focusa_cohort_active_stripe": 4,
    "focusa_operator_active_api": 2,
    "focusa_operator_active_stripe": 25,
    "focusa_operator_active_stripe_payment_intent_refs": 25,
    "focusa_operator_active_stripe_activations": 25,
    "focusa_operator_refunded": 14,
    "focusa_operator_revoked": 13,
    "audit_stripe_payment_succeeded": 44,
    "audit_stripe_charge_refunded": 14,
}
assert "potential customer-right-bearing history" in edd["basis"]
assert "potential customer-right-bearing history" in install["basis"]

# Core fail-closed invariant: ambiguity forbids destructive clean cutover but
# does not stop implementation when every possible customer right is migrated.
def validate_decision(decision, ambiguous_rows):
    if ambiguous_rows:
        assert decision["zero_sales_proven"] is False, "false zero-sales claim while ambiguity remains"
        assert decision["clean_cutover_allowed"] is False, "false clean-cutover claim while ambiguity remains"
        assert decision["implementation_may_continue"] is True
        assert decision["status"] == "migration_preserving_path_selected"


decision = data["decision"]
validate_decision(decision, ambiguous)
for forbidden_claim in ("zero_sales_proven", "clean_cutover_allowed"):
    false_decision = dict(decision)
    false_decision[forbidden_claim] = True
    try:
        validate_decision(false_decision, ambiguous)
    except AssertionError:
        pass
    else:
        raise AssertionError(f"validator accepted false {forbidden_claim} claim")

registry = json.loads((ROOT / data["reconciliation"]["product_registry"]).read_text())
assert registry["counts"]["checkout_enabled"] == 0
assert all(not offer["checkout_enabled"] for offer in registry["protected_offers"])
assert data["reconciliation"]["selected_cutover_path"] == "migration_preserving"
assert data["reconciliation"]["legacy_records_preserved"] is True
assert data["reconciliation"]["immutable_stripe_live_test_correlation_required"] is True
assert data["reconciliation"]["migration_preservation_required"] is True

print("Spec 172 migration-preserving sales inventory: PASS")
