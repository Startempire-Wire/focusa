#!/usr/bin/env python3
"""Validate the redacted, migration-preserving Spec 152E record inventory."""

import hashlib
import json
import re
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PATH = ROOT / "docs/contracts/spec152e-migration-inventory.v1.json"
raw = PATH.read_text(encoding="utf-8")
data = json.loads(raw)

assert data["schema"] == "focusa.spec152e.migration_inventory.v1"
assert data["inventory_id"] == "focusa-vbcqu.20.13.49"
assert data["authority"]["canonical"] == "WPUIAI.com EDD"
assert data["authority"]["spec158"] == "excluded"
assert data["decision"] == {
    "zero_sales_proven": False,
    "clean_reset_allowed": False,
    "migration_preserving_path": True,
    "historical_rights_preserved": True,
}

allowed = {
    "canonical", "evidence_backed_import", "verify_first", "duplicate",
    "synthetic_quarantine", "refunded_revoked", "unresolved",
}
records = data["records"]
assert records
assert len({row["handle"] for row in records}) == len(records)
assert {row["disposition"] for row in records} <= allowed
assert all(re.fullmatch(r"rec_[0-9a-f]{24}", row["handle"]) for row in records)
assert all(re.fullmatch(r"ev_[0-9a-f]{24}", row["evidence_handle"]) for row in records)
assert all(row["customer_payload_stored"] is False for row in records)
assert all(row["preserve"] is True for row in records)

required_surfaces = {
    "edd_customer", "edd_customer_email", "edd_order", "edd_order_item",
    "edd_license", "gravity_entry", "gravity_entry_meta", "stripe_test_object",
    "stripe_live_expired_session", "stripe_live_incomplete_payment_intent",
    "install_site_license", "install_site_audit_receipt", "node_record",
}
assert {row["surface"] for row in records} == required_surfaces
surface_counts = Counter(row["surface"] for row in records)
assert surface_counts == data["reconciliation"]["physical_record_counts"]
assert len(records) == data["reconciliation"]["physical_record_total"] == 596
assert Counter(row["disposition"] for row in records) == data["reconciliation"]["disposition_counts"]

aliases = data["correlated_views"]
assert len(aliases["manual_api_grants"]) == 27
assert Counter(row["disposition"] for row in aliases["manual_api_grants"]) == {
    "synthetic_quarantine": 1, "unresolved": 26,
}
assert len(aliases["active_stripe_node_activations"]) == 25
assert all(row["disposition"] == "unresolved" for row in aliases["active_stripe_node_activations"])
assert data["reconciliation"]["overlap_rule"] == "correlated views are non-additive and never create a second physical-record disposition"

assert data["reconciliation"]["genuine_sales_count"] is None
assert data["reconciliation"]["evidence_backed_import_count"] == 0
assert data["reconciliation"]["unresolved_count"] > 0
assert data["reconciliation"]["destructive_reconciliation_forbidden"] is True
assert data["reconciliation"]["next_action"] == "immutable_id_correlation_then_migrate_or_retain_quarantine"
assert data["local_receipt_boundary"]["known_captured_instances"] == 0
assert data["local_receipt_boundary"]["absence_claimed"] is False
assert data["local_receipt_boundary"]["disposition_if_discovered"] == "unresolved"

# No identity, raw key, payment identifier, credential, or secret-shaped value may land here.
assert not re.search(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}", raw)
assert not re.search(r"(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+", raw)
assert "focusa_live_" not in raw
assert "license_key" not in raw
assert "payment_intent_id" not in raw
assert "customer_id" not in raw

print(json.dumps({
    "schema": "focusa.spec152e.migration_inventory_validation.v1",
    "inventory_sha256": hashlib.sha256(raw.encode()).hexdigest(),
    "physical_records": len(records),
    "manual_grant_views": len(aliases["manual_api_grants"]),
    "node_correlation_views": len(aliases["active_stripe_node_activations"]),
    "result": "passed",
}, sort_keys=True))
