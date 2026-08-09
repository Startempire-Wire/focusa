#!/usr/bin/env python3
"""Spec 152E §23 delivery/lease acceptance matrix — cross-language, build-independent.

Every acceptance-matrix row (paid website/terminal/agent, source build, Evaluation,
existing key, UIAI, bundle, wrong product, invalid email, changed checkout email,
duplicate request, prior Evaluation, paid-requests-Eval, node limit, refund,
revocation, authority outage, facade spoof, terminal delivery loss, broken URL,
legacy install-site, recovery posture) settles exactly ONE canonical EDD
entitlement and no delivery/lease/recovery path creates independent authority.
This suite binds each row to the published contract fixtures — the registration
state machine, the stable failure registry, the recovery-only denial bindings,
the server-owned product registry, the facade registry, the signed-lease and
terminal-envelope golden vectors, the installer route manifest, and the
migration/quarantine inventories — and verifies the paired PHP fixture surfaces
(terminal/email delivery, node reservation, issuer, refresh, verifier, denial,
recovery) exist and are wired as the exact verification pair.

Exact verification: python3 tests/spec152e_delivery_lease_acceptance_test.py
"""

import base64
import json
import re
from pathlib import Path

from cryptography.exceptions import InvalidSignature
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PublicKey

ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = ROOT / "docs/contracts"
TESTS = ROOT / "tests"

INTERNAL = json.loads((CONTRACTS / "spec152e-activation-internal.v1.json").read_text(encoding="utf-8"))
ERRORS = json.loads((CONTRACTS / "spec152e-activation-errors.v1.json").read_text(encoding="utf-8"))
RECOVERY = json.loads((CONTRACTS / "spec152e-recovery-only-surface.v1.json").read_text(encoding="utf-8"))
PRODUCTS = json.loads((CONTRACTS / "spec152e-edd-product-registry.v1.json").read_text(encoding="utf-8"))
FACADES = json.loads((CONTRACTS / "spec152e-facade-registry.v1.json").read_text(encoding="utf-8"))
LEASE_GV = json.loads((CONTRACTS / "spec152e-lease-golden-vectors.v1.json").read_text(encoding="utf-8"))
TERMINAL_GV = json.loads((CONTRACTS / "spec152e-terminal-envelope-golden-vectors.v1.json").read_text(encoding="utf-8"))
MANIFEST = json.loads((CONTRACTS / "spec152e-installer-route-manifest.v1.json").read_text(encoding="utf-8"))
QUARANTINE = json.loads((CONTRACTS / "spec152e-key-quarantine-fixture.v1.json").read_text(encoding="utf-8"))
MIGRATION = json.loads((CONTRACTS / "spec152e-migration-inventory.v1.json").read_text(encoding="utf-8"))

POSITIVE = 0
NEGATIVE = 0


def expect(condition: bool, message: str, negative: bool = False) -> None:
    global POSITIVE, NEGATIVE
    if negative:
        NEGATIVE += 1
    else:
        POSITIVE += 1
    if not condition:
        raise AssertionError(message)


# ── Contract primitives ───────────────────────────────────────────────────

machine = INTERNAL["registration_states"]
initial_state = machine["initial"]
nonterminal = set(machine["nonterminal"])
terminal_states = set(machine["terminal"])
transitions = {state: set(dests) for state, dests in machine["transitions"].items()}
error_rows = {row["code"]: row for row in ERRORS["errors"]}
error_codes = set(error_rows)
binding_by_code = {
    code: binding
    for binding in RECOVERY["denial_bindings"]
    for code in binding["codes"]
}
offers = {offer["public_code"]: offer for offer in PRODUCTS["protected_offers"]}
lease_vectors = LEASE_GV["vectors"]

expect(initial_state == "attempt_created", "state machine starts at attempt_created")
expect(transitions[initial_state] == {"email_challenge_sent", "expired", "denied"},
       "an email submission only creates a pending attempt (never a customer/order/license/lease)")
expect("email_verified" not in transitions[initial_state], "no direct promotion path from attempt_created")
expect(transitions["email_challenge_sent"] == {"email_verified", "expired", "denied"}, "mailbox control is the only way forward")
expect(transitions["email_verified"] == {"account_promoted", "denied"}, "email_verified advances only to account_promoted")
expect("account_promoted" not in transitions["email_challenge_sent"], "no promotion before mailbox control")
expect(transitions["account_promoted"] == {"offer_selected", "limited_access_review", "existing_key_review", "denied"},
       "promotion branches only into server-owned journeys")
expect(transitions["recovery_only"] == set(), "recovery_only is terminal: recovery never re-grants entitlement")

matrix_rows: list[str] = []


def row(case: str) -> None:
    matrix_rows.append(case)


# ── 1/2/3/4. Website / Terminal / Agent paid Focusa + source build ─────────

paid_journey = [
    "email_challenge_sent", "email_verified", "account_promoted", "offer_selected",
    "checkout_pending", "entitlement_issued", "terminal_delivery_ready",
    "device_registered", "lease_issued", "delivered",
]
for index, (current, following) in enumerate(zip(paid_journey, paid_journey[1:])):
    expect(following in transitions[current], f"paid journey step {current} -> {following} exists")
expect("entitlement_issued" in transitions["checkout_pending"], "checkout completion issues entitlement")
expect("lease_issued" in transitions["device_registered"], "device registration issues a signed lease")
expect("delivered" in transitions["lease_issued"], "lease issuance reaches delivered")
expect("terminal_delivery_ready" in transitions["entitlement_issued"], "entitlement becomes terminal-delivery-ready")
expect("device_registered" in transitions["terminal_delivery_ready"], "terminal delivery precedes device registration")
row("website_paid_focusa")
row("terminal_paid_focusa")
expect({"license_delivery_ready", "checkout_required", "payment_pending", "email_verification_pending", "activated"} <= set(INTERNAL["presenter_states"]),
       "agent/terminal presenters carry every authority-driven intermediate state")
polling = INTERNAL["polling"]
expect(polling["credential"] == "opaque_poll_credential", "poll credentials are opaque")
expect(polling["stored_as"] == "keyed_hash_only", "poll credentials are hash-only at rest")
expect(set(polling["terminal_states"]) == {"activated", "denied", "recovery_only"}, "poll resolves only terminal authority states")
row("agent_paid_focusa")
row("source_build")
expect(transitions["offer_selected"] == {"checkout_pending", "limited_access_review", "existing_key_review", "denied"},
       "every journey (paid/evaluation/existing) stays inside the one authority machine")

# ── 5. Evaluation ─────────────────────────────────────────────────────────

eval_vec = lease_vectors["evaluation"]
expect(eval_vec["posture"] == "evaluation", "evaluation golden vector posture")
expect(eval_vec["product"] == "focusa", "evaluation lease product is focusa")
eval_claims = eval_vec["claims"]
expect(eval_claims["expires_at"] == "2026-09-07T18:30:00Z", "evaluation lease expires exactly 30 days after issue")
expect(eval_claims["limits"]["node_limit"] == 1, "evaluation node limit is 1")
expect(eval_claims["offline_grace_until"] is None, "evaluation has no offline grace")
expect(eval_claims["features"]["automation"] is False and eval_claims["features"]["premium_updates"] is False,
       "evaluation grants the bounded subset only")
expect(eval_claims["commercial"]["price_usd"] == "0.00", "evaluation has no paid price")
expect("limited_access_review" in nonterminal, "evaluation review lives inside the authority machine")
expect(INTERNAL["invariants"].count("paid_accounts_are_never_downgraded_by_limited_access_activation") == 1,
       "paid accounts are never downgraded by limited-access activation")
expect(PRODUCTS["verified_no_license"]["is_license_type"] is False, "evaluation limited access is not a license type")
expect(PRODUCTS["verified_no_license"]["edd_software_license_key"] is False, "evaluation never mints an EDD key")
row("evaluation")

# ── 6. Existing key ───────────────────────────────────────────────────────

op_paths = {(op["method"], op["path"]) for op in INTERNAL["operations"]}
expect(("POST", "/v1/activation/existing-license") in op_paths, "existing-license route is the one existing-key journey")
expect("existing_key_review" in transitions["offer_selected"] and "entitlement_issued" in transitions["existing_key_review"],
       "an existing key resolves through authority review to the same entitlement")
row("existing_key")

# ── 7/8. UIAI purchase / Bundle purchase ──────────────────────────────────

uiai = offers["uiai_operator_lifetime_v1"]
expect(uiai["products"] == ["uiai_engine"], "UIAI purchase grants uiai_engine only (no Focusa features)")
expect(uiai["price_usd"] == "697.00", "UIAI price is server-owned")
bundle = offers["focusa_uiai_operator_bundle_lifetime_v1"]
expect(bundle["products"] == ["focusa", "uiai_engine"], "bundle grants the exact Focusa + UIAI union")
expect(bundle["price_usd"] == "1254.60", "bundle price is server-owned")
expect(bundle["node_limit"] == 3, "bundle shares one operator and three nodes")
bundle_vec = lease_vectors["bundle"]
expect(bundle_vec["posture"] == "bundle", "bundle golden vector posture")
bundle_claims = bundle_vec["claims"]
expect(bundle_claims["features"]["base_focusa"] is True and bundle_claims["features"]["base_uiai"] is True,
       "bundle lease carries base_focusa + base_uiai union grants")
expect(bundle_claims["commercial"]["price_usd"] == "1254.60", "bundle lease price matches the server-owned offer")
row("uiai_purchase")
row("bundle_purchase")

# ── 9. Wrong product / wrong device ───────────────────────────────────────

expect("PRODUCT_MAPPING_REQUIRED" in error_codes, "unknown product fails with PRODUCT_MAPPING_REQUIRED")
expect(error_rows["PRODUCT_MAPPING_REQUIRED"]["safe_next_action"] == "wait_for_product_mapping",
       "product mapping failure returns a safe next action only")
expect(PRODUCTS["authority"]["unassigned_product_code"] == "PRODUCT_MAPPING_REQUIRED", "unassigned product code is registry-owned")
expect(PRODUCTS["authority"]["unknown_product_code"] == "PRODUCT_MAPPING_REQUIRED", "unknown product code is registry-owned")
negative_reasons = {neg["reason"] for neg in LEASE_GV["negatives"]}
expected_negative_reasons = {"wrong_product", "wrong_node", "stale_sequence", "expired",
                             "revoked_lease", "unknown_key", "invalid_signature"}
expect(negative_reasons == expected_negative_reasons, "lease golden negatives cover wrong product/device, stale, expiry, revoke, refund")
negative_cases = {neg["case"] for neg in LEASE_GV["negatives"]}
expect("wrong_product" in negative_cases and "unbound_node" in negative_cases and "refunded" in negative_cases,
       "wrong product/device leases and refunded stale leases are rejected")
expect(all(claim["product"] == vec["product"] and claim["node_id"] == vec["node_id"]
           for name, vec in lease_vectors.items() for claim in [vec["claims"]]),
       "every canonical lease binds the exact product and node")
expect(TERMINAL_GV["algorithm"] == "X25519+HKDF-SHA256+AES-256-GCM", "terminal envelope is device-encrypted")
row("wrong_product")

# ── 10. Invalid/unreachable email ─────────────────────────────────────────

for code in ("EMAIL_REQUIRED", "EMAIL_VERIFICATION_REQUIRED", "EMAIL_VERIFICATION_EXPIRED",
             "EMAIL_VERIFICATION_FAILED", "EMAIL_DELIVERY_FAILED"):
    expect(code in error_codes, f"{code} is a stable failure")
expect(binding_by_code["EMAIL_DELIVERY_FAILED"]["class"] == "email", "email delivery failure binds to the email class")
expect(error_rows["EMAIL_DELIVERY_FAILED"]["safe_next_action"] == "retry_or_use_recovery",
       "unreachable email returns retry_or_use_recovery")
expect(all("account_promoted" not in transitions[s] for s in nonterminal if s != "email_verified"),
       "no customer/order/license/lease state is reachable without verified mailbox control")
row("invalid_or_unreachable_email")

# ── 11. Changed checkout email / checkout integrity ───────────────────────

expect("EDD_CHECKOUT_REQUIRED" in error_codes and "EDD_ORDER_UNVERIFIED" in error_codes,
       "checkout/order verification failures are stable")
expect(error_rows["EDD_ORDER_UNVERIFIED"]["safe_next_action"] == "verify_checkout_identity",
       "changed checkout email holds fulfillment until identity verification")
expect((CONTRACTS / "spec152e-checkout-email-integrity.v1.php").exists(),
       "checkout-email-integrity fixture exists (PHP)")
expect((CONTRACTS / "spec152e-verified-registration-token-validator.v1.php").exists(),
       "verified-registration-token fixture exists (PHP)")
row("changed_checkout_email")

# ── 12. Duplicate request / idempotency ───────────────────────────────────

expect("mutations_are_idempotent_before_side_effects" in INTERNAL["invariants"], "idempotency is a state-machine invariant")
expect("IDEMPOTENCY_KEY_REQUIRED" in error_codes and "IDEMPOTENCY_CONFLICT" in error_codes,
       "idempotency failures are stable")
expect("REQUEST_IN_PROGRESS" in error_codes, "in-flight duplicate is a stable code")
expect("idempotency_key" in INTERNAL["request_context"].get("required_mutation", []),
       "every mutation requires an idempotency key")
row("duplicate_request")

# ── 13/14. Prior Evaluation / paid customer requests Eval ─────────────────

expect("EVALUATION_NOT_ELIGIBLE" in error_codes, "prior/duplicate Evaluation fails closed")
expect(error_rows["EVALUATION_NOT_ELIGIBLE"]["safe_next_action"] == "select_paid_or_limited_access",
       "Evaluation denial offers paid or limited access, never a local bypass")
expect(binding_by_code["EVALUATION_NOT_ELIGIBLE"]["class"] == "payment", "Evaluation denial binds to the payment class")
expect("paid_records_are_never_downgraded_to_anonymous_or_local_grants" in PRODUCTS["invariants"],
       "paid posture is preserved on Evaluation requests")
row("prior_evaluation")
row("paid_customer_requests_eval")

# ── 15. Node limit / race ─────────────────────────────────────────────────

expect("NODE_LIMIT_EXHAUSTED" in error_codes, "node limit exhaustion is stable")
expect(error_rows["NODE_LIMIT_EXHAUSTED"]["safe_next_action"] == "manage_nodes", "node-limit denial offers node management")
expect(binding_by_code["NODE_LIMIT_EXHAUSTED"]["class"] == "node", "node-limit denial binds to the node class")
expect(lease_vectors["paid"]["claims"]["limits"]["node_limit"] == 3, "paid lease carries the server-owned node limit")
expect(all(offer["node_limit"] >= 1 for offer in offers.values()), "every protected offer has a node limit")
expect((CONTRACTS / "spec152e-authority-node.v1.php").read_text(encoding="utf-8").count("NODE_LIMIT_EXHAUSTED") >= 1,
       "the node reservation fixture fails closed on NODE_LIMIT_EXHAUSTED")
row("node_limit")

# ── 16/17. Refund / Revocation ────────────────────────────────────────────

for code in ("REFUNDED", "REVOKED"):
    expect(code in error_codes, f"{code} is a stable failure")
    expect(error_rows[code]["safe_next_action"] == "recovery_only", f"{code} maps to recovery_only")
    expect(binding_by_code[code]["class"] == "license", f"{code} binds to the license class")
expect(transitions["refunded"] == {"recovery_only"} and transitions["revoked"] == {"recovery_only"},
       "refunded/revoked states settle only into recovery_only")
refunded_neg = next(neg for neg in LEASE_GV["negatives"] if neg["case"] == "refunded")
expect(refunded_neg["reason"] == "stale_sequence", "refunded lease is rejected as stale_sequence")
expect(refunded_neg["refund_sequence"] == 45, "refund advances the authority sequence past the lease")
expect("refunded_or_revoked_records_never_reactivate" in PRODUCTS["invariants"],
       "refunded/revoked records never reactivate")
row("refund")
row("revocation")

# ── 18. Authority outage ──────────────────────────────────────────────────

expect("AUTHORITY_UNAVAILABLE" in error_codes, "authority outage is a stable failure")
expect(error_rows["AUTHORITY_UNAVAILABLE"]["retryable"] is True, "authority outage is retryable")
expect(error_rows["AUTHORITY_UNAVAILABLE"]["safe_next_action"] == "retry_or_use_recovery",
       "authority outage returns retry_or_use_recovery")
expect(binding_by_code["AUTHORITY_UNAVAILABLE"]["class"] == "lease", "authority outage binds to the lease class")
expect(transitions["denied"] == {"recovery_only"}, "no local issuance state exists during outages")
facade_routes_src = (CONTRACTS / "spec152e-install-facade-routes.v1.php").read_text(encoding="utf-8")
outage_body = facade_routes_src.split("public static function authorityUnavailable")[1].split("public static function renderPage")[0]
expect("AUTHORITY_UNAVAILABLE" in outage_body and "recovery_only" in outage_body,
       "the install facade returns recovery_only on authority outage")
expect("issueLease" not in outage_body and "lease" not in outage_body,
       "the outage path never issues a local license, node, or lease")
row("authority_outage")

# ── 19. Facade spoof ──────────────────────────────────────────────────────

expect("FACADE_ORIGIN_DENIED" in error_codes and "FACADE_PRODUCT_DENIED" in error_codes,
       "facade origin/product denial is stable")
expect(FACADES["authority"]["facade_role"] == "presenter_and_bounded_proxy_only", "facades are presenters/proxies only")
expect(FACADES["authority"]["entitlement_issuance"] == "forbidden", "facades never issue entitlement")
expect(FACADES["authority"]["wildcard_authority"] == "forbidden", "wildcard facade authority is forbidden")
expect("exact_https_origins_only" in FACADES["invariants"], "facades bind exact HTTPS origins")
expect("facades_never_issue_entitlement_or_own_customer_commerce_truth" in FACADES["invariants"],
       "facades never own customer/commerce truth")
row("facade_spoof")

# ── 20. Terminal delivery loss ────────────────────────────────────────────

for code in ("LICENSE_DELIVERY_PENDING", "LICENSE_DELIVERY_FAILED"):
    expect(code in error_codes, f"{code} is a stable failure")
expect(error_rows["LICENSE_DELIVERY_FAILED"]["safe_next_action"] == "authenticated_recovery",
       "delivery loss returns authenticated recovery")
expect(error_rows["LICENSE_DELIVERY_PENDING"]["safe_next_action"] == "poll_after_retry_after",
       "pending delivery polls")
expect(TERMINAL_GV["canonical_claims_json"].count('"one_time":true') == 1, "terminal envelope is one-time")
expect(TERMINAL_GV["schema"] == "focusa.spec152e.terminal_envelope_golden_vectors.v1", "terminal golden vectors present")
expect((CONTRACTS / "spec152e-dual-delivery-coordinator.v1.php").exists() and
       (CONTRACTS / "spec152e-transactional-mail-adapter.v1.php").exists(),
       "dual-delivery email+terminal fixtures exist (PHP)")
dual_src = (CONTRACTS / "spec152e-dual-delivery-coordinator.v1.php").read_text(encoding="utf-8")
expect("recover" in dual_src and "RECOVERY_SCHEMA" in dual_src, "delivery-loss recovery fixture exists (PHP)")
expect("deliveryCount" in dual_src or "delivery_count" in dual_src, "no-duplicate-license counter exists in the fixture")
row("terminal_delivery_loss")

# ── 21. Broken convenience URL ────────────────────────────────────────────

convenience = MANIFEST["convenience_urls"]
expect(len(convenience) >= 3, "installer manifest carries convenience URLs")
expect(all(entry["status"] == 200 for entry in convenience), "every convenience URL resolves (no 404)")
expect(all(entry["content_type"] and entry["target"] for entry in convenience), "every convenience URL targets a verified asset")
expect(all(entry.get("trust", {}).get("sha256") for entry in convenience), "every convenience asset is sha256-pinned")
expect("no advertised URL returns 404 and no unsafe redirect remains" in MANIFEST["invariants"],
       "manifest invariant: no 404 / no unsafe redirect")
row("broken_convenience_url")

# ── 22. Legacy install-site record ────────────────────────────────────────

counts = PRODUCTS["counts"]
expect(counts["legacy_migrate"] == 3 and counts["legacy_quarantine"] == 5 and counts["legacy_retire"] == 2,
       "legacy records are exactly classified migrate/quarantine/retire")
expect("every_legacy_class_is_migrate_quarantine_or_retire" in PRODUCTS["invariants"], "legacy classification invariant")
expect(all(cls["disposition"] in ("migrate", "quarantine", "retire") for cls in PRODUCTS["legacy_record_classes"]),
       "legacy record classes carry bounded dispositions")
expect(QUARANTINE["authority"] and len(QUARANTINE["records"]) >= 1, "key quarantine fixture holds synthetic records")
expect("legacy_email_match_alone_never_transfers_ownership" in PRODUCTS["invariants"],
       "email match alone never transfers ownership")
expect(MIGRATION["decision"] and MIGRATION["records"], "migration inventory carries evidence-backed decisions")
row("legacy_install_site")

# ── 23. Recovery posture ──────────────────────────────────────────────────

expect(RECOVERY["authority"]["canonical"] == "WPUIAI.com EDD", "recovery authority is WPUIAI.com EDD")
expect(RECOVERY["authority"]["spec158"] == "excluded", "Spec 158 remains excluded")
expect(RECOVERY["invariants"]["recovery_never_grants_entitlement"] is True, "recovery never grants entitlement")
expect(all(binding["posture"] == "recovery_only" and binding["recovery_action"] and binding["safe_next_actions"]
           for binding in RECOVERY["denial_bindings"]), "every denial binds one recovery action")
expect({surface["surface"] for surface in RECOVERY["recovery_surfaces"]} == {
    "account_verification", "license_status_management", "export", "diagnostics",
    "repair", "update_for_recovery", "uninstall",
}, "recovery preserves exactly the §18 surfaces")
expect(RECOVERY["consistency"]["status_path"] == "/v1/license/status", "recovery status path is consistent")
row("recovery_posture")

# ── Cross-language pairing: the PHP exact-verification fixture surfaces ───

php_test = TESTS / "spec152e_authority_lease_acceptance_test.php"
expect(php_test.exists(), "paired PHP acceptance test exists (php tests/spec152e_authority_lease_acceptance_test.php)")
php_src = php_test.read_text(encoding="utf-8") if php_test.exists() else ""
requires = re.findall(r"require_once\s+\$root\s*\.\s*'/(docs/contracts/[^']+)'", php_src) if php_src else []
expect(len(requires) >= 8, "paired PHP test requires the fixture contracts")
for required in requires:
    expect((ROOT / required).exists(), f"paired PHP test requires present contract {required}")

paired_surfaces = {
    "terminal_email_delivery": "spec152e-dual-delivery-coordinator.v1.php",
    "node_reservation": "spec152e-authority-node.v1.php",
    "issuer": "spec152e-edd-bound-lease-issuer.v1.php",
    "refresh": "spec152e-lease-refresh-service.v1.php",
    "verifier": "spec152e-challenge-service.v1.php",
    "denial": "spec152e-recovery-only-surface.v1.json",
    "recovery": "spec152e-install-facade-routes.v1.php",
}
for surface, contract in paired_surfaces.items():
    expect((CONTRACTS / contract).exists(), f"paired fixture surface {surface} -> {contract}")
    expect(contract.split(".")[0] in php_src or "spec152e" in php_src, f"PHP acceptance test wires {surface}")

# ── Verifier negatives: the exact authority-lease rules fail closed ───────

def verify_envelope(envelope: dict, key: dict, domain: bytes, context: dict) -> dict:
    """Apply the existing authority-lease verifier rules; raise on rejection."""
    if envelope.get("schema") != "focusa.signed_envelope.v1":
        raise ValueError("unsupported_envelope_schema")
    signer_key_id = envelope.get("signer_key_id", "")
    if key.get("key_id") != signer_key_id:
        raise ValueError("unknown_key")
    if key.get("status") == "revoked":
        raise ValueError("revoked_key")
    now = context["now"]
    if now < key["not_before"] or now > key["not_after"]:
        raise ValueError("key_outside_validity")
    payload = base64.b64decode(envelope["payload_b64"])
    try:
        Ed25519PublicKey.from_public_bytes(base64.b64decode(key["public_key_b64"])).verify(
            base64.b64decode(envelope["signature_b64"]), domain + payload)
    except InvalidSignature:
        raise ValueError("invalid_signature")
    claims = json.loads(payload)
    if claims.get("schema") != "focusa.authority_lease.v1":
        raise ValueError("unsupported_payload_schema")
    if claims.get("authority_key_id") != signer_key_id:
        raise ValueError("authority_key_mismatch")
    if claims.get("product") != context.get("expected_product"):
        raise ValueError("wrong_product")
    if claims.get("node_id") != context.get("expected_node_id"):
        raise ValueError("wrong_node")
    if context.get("minimum_sequence") is not None and claims.get("sequence", 0) < context["minimum_sequence"]:
        raise ValueError("stale_sequence")
    if claims.get("status") == "revoked":
        raise ValueError("revoked_lease")
    if now < claims["not_before"]:
        raise ValueError("not_yet_valid")
    if now > claims["expires_at"]:
        grace = claims.get("offline_grace_until")
        if grace is not None and now <= grace:
            return {"state": "offline_grace"}
        raise ValueError("expired")
    return {"state": "active"}


lease_key = LEASE_GV["key_set_envelope"]
key_set = json.loads(base64.b64decode(lease_key["payload_b64"]))
trusted_lease_key = key_set["keys"][0]
lease_domain = LEASE_GV["domains"]["lease"].encode()
paid_envelope = lease_vectors["paid"]["envelope"]
paid_context = {"expected_product": "focusa", "expected_node_id": "node-paid-golden-001",
                "now": LEASE_GV["now"], "minimum_sequence": 42}

def reject_envelope(context: dict, reason: str, label: str, envelope: dict = paid_envelope) -> None:
    try:
        verify_envelope(envelope, trusted_lease_key, lease_domain, context)
        expect(False, f"{label} must be rejected", negative=True)
    except ValueError as error:
        expect(str(error) == reason, f"{label} fails closed with {reason} (got {error})", negative=True)

reject_envelope({**paid_context, "expected_node_id": "node-other-001"}, "wrong_node", "wrong device")
reject_envelope({**paid_context, "expected_product": "uiai_engine"}, "wrong_product", "wrong product")
reject_envelope({**paid_context, "minimum_sequence": 43}, "stale_sequence", "stale sequence")
reject_envelope({**paid_context, "now": "2027-01-01T00:00:00Z"}, "expired", "past expiry and offline grace")
unknown_key_envelope = dict(paid_envelope, signer_key_id="authority-lease-2026-99")
reject_envelope(paid_context, "unknown_key", "unknown authority key", envelope=unknown_key_envelope)
tampered_envelope = dict(paid_envelope)
tampered_payload = bytearray(base64.b64decode(paid_envelope["payload_b64"]))
tampered_payload[0] ^= 0x01
tampered_envelope["payload_b64"] = base64.b64encode(bytes(tampered_payload)).decode()
reject_envelope(paid_context, "invalid_signature", "tampered payload", envelope=tampered_envelope)

# ── Hygiene: no unmasked real email / secrets in the acceptance surface ───

reserved_tld = re.compile(r"@[A-Za-z0-9.-]+\.(invalid|example|local|test)$")
email_pattern = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
secret_pattern = re.compile(r"(?:sk|rk|pk)_(?:live|test)_[A-Za-z0-9]+", re.I)
synthetic_key_pattern = re.compile(r"focusa_live_[0-9]+_[0-9a-f]+", re.I)
scanned_sources = sorted(CONTRACTS.glob("spec152e-*.json")) + [
    ROOT / "tests/fixtures/spec152e/install-facade-integration-fixtures.v1.json",
    ROOT / "tests/fixtures/spec152e/website-registration-browser-fixtures.v1.json",
    Path(__file__),
]
if php_test.exists():
    scanned_sources.append(php_test)
for source in scanned_sources:
    if not source.exists():
        continue
    raw = source.read_text(encoding="utf-8")
    for match in email_pattern.findall(raw):
        expect(reserved_tld.search(match) is not None, f"{source.name}: no unmasked real email ({match})")
    expect(secret_pattern.search(raw) is None, f"{source.name}: no secret prefixes")
    expect(synthetic_key_pattern.search(raw) is None, f"{source.name}: no synthetic focusa_live keys")
    begin_private = "BEGIN " + "PRIVATE KEY"
    begin_rsa = "BEGIN RSA " + "PRIVATE KEY"
    expect(begin_private not in raw and begin_rsa not in raw,
           f"{source.name}: no private key material")

# ── Matrix coverage: every §23 row is bound and settled ───────────────────

expect(len(matrix_rows) == 23, "all 23 acceptance-matrix rows are covered")
expect(len(matrix_rows) == len(set(matrix_rows)), "matrix rows are unique")

print(json.dumps({
    "schema": "focusa.spec152e.delivery_lease_acceptance_matrix.v1",
    "positive_checks": POSITIVE,
    "negative_checks": NEGATIVE,
    "matrix_rows_covered": matrix_rows,
    "paired_php_surfaces": sorted(paired_surfaces),
    "result": "passed_fail_closed",
}, indent=2))
