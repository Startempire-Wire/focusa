#!/usr/bin/env python3
"""Fail closed on drift or overclaim in the bounded Spec 172 UIAI baseline."""

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CONTRACT = ROOT / "docs/contracts/spec172-public-commerce-baseline.v1.json"
data = json.loads(CONTRACT.read_text(encoding="utf-8"))

assert data["schema"] == "focusa.spec172_public_commerce_baseline.v1"
assert data["captured_at"] == "2026-08-06"
assert data["capture_method"] == "uiai_engine_browser_read"
assert data["authentication_required"] is False
posture = data["evidence_posture"]
assert posture == {
    "classification": "bounded_redacted_public_claim_baseline",
    "public_copy_is_entitlement_authority": False,
    "raw_capture_embedded": False,
    "current_recapture_required": False,
    "service_status_after_supplemental_read": "unavailable",
}

sources = {row["surface"]: row for row in data["sources"]}
assert set(sources) == {"focusa", "engine", "install", "wpuiai_edd"}
assert sources["focusa"]["urls"] == [
    "https://focusa.dev/", "https://focusa.dev/pricing/",
    "https://focusa.dev/llms.txt", "https://focusa.dev/.well-known/agent-commerce.json",
]
assert sources["engine"]["urls"] == [
    "https://engine.focusa.dev/", "https://engine.focusa.dev/pricing/",
    "https://engine.focusa.dev/llms.txt", "https://engine.focusa.dev/.well-known/agent-commerce.json",
]
assert sources["install"]["urls"] == [
    "https://install.focusa.dev/", "https://install.focusa.dev/focusa",
    "https://install.focusa.dev/bundle",
]
assert sources["wpuiai_edd"]["urls"][0] == "https://wpuiai.com/wp-sitemap-posts-download-1.xml"
assert all(row["title_basis"] == "not_retained_in_section_18_1" for row in sources.values())

assert data["supplemental_observation"] == {
    "captured_at": "2026-08-07",
    "capture_method": "uiai_engine_browser_open",
    "url": "https://focusa.dev/",
    "title": "Home - Focusa",
    "claims_added": [],
    "evidence_ref": "spec172:18.1-supplement:focusa-root-title:2026-08-07",
}
assert {row["code"] for row in data["observed_claims"]} == {
    "standalone_prices_697", "conflicting_bundle_1097", "anonymous_local_evaluation",
    "gravity_stripe_positioning", "no_phone_home", "broken_routes_and_license_links",
    "legacy_wpuiai_prices",
}
assert len(data["required_replacements"]) == 10
replacements = {row["current"]: row["required"] for row in data["required_replacements"]}
assert replacements["Bundle advertised at $1,097"] == "Bundle price $1,254.60"
assert replacements["No phone home"].startswith("No telemetry; bounded authority communication")
assert replacements["Old WPUIAI $29/$99/$299/$149 offers appear commercially related"] == (
    "Keep legacy WPUIAI catalog explicitly separate from Focusa/UIAI License Types"
)

serialized = CONTRACT.read_text(encoding="utf-8")
for forbidden in ("customer_email", "license_key", "access_token", "cookie", "authenticated capture"):
    assert forbidden not in serialized

print("Spec 172 bounded UIAI public-commerce baseline: PASS")
